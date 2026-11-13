//! Talents (spec §8 "Talents"; `assets/data/talents.toml` for the settings, one file per talent
//! under `assets/data/talents/`): what a character may pick as they level up. A talent has a
//! tree, prerequisites ([`Needs`]) that must all hold when it is chosen, and typed effects ([`TalentEffect`]) summed by [`crate::talent::TalentMods`] and
//! read where the rules already read what magic items do. A talent may name the systems it
//! touches from the list in the settings file (the wiki's matrix reads them). The loader checks
//! only that the data is well formed: what it names resolves (#1205). Without the files there
//! are no talents, and no level asks for one.

use super::level::OneOrMany;
use super::{
    Build, CombatStance, Content, ContentError, ItemKind, ResourceId, Role, StatusId,
    WeaponSkillId, duplicate_id,
};
use crate::alignment::Requirement;
use crate::dice::Dice;
use crate::skills::SkillId;
use crate::stats::{PerStat, Stat};
use serde::Deserialize;
use std::collections::BTreeMap;

/// `a` or `an` for a noun.
fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

/// A talent, an index into the book (file order). Saves name talents by id, never by this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TalentId(pub u16);

/// A tree, an index into [`TalentBook::trees`] (display order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeId(pub u8);

/// A system of the settings file's list, an index into [`TalentBook::systems`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(pub u8);

/// A tree of talents (`[[tree]]` in the settings file), in display order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeDef {
    pub id: String,
    pub name: String,
    /// One line: what the tree is about.
    pub description: String,
    /// Its talents are class abilities (spec §8 "Class abilities"): given by the class tables
    /// (`data/classes/`), never picked.
    pub class: bool,
}

/// A system of the game that talents may touch (`[[system]]` in the settings file), for the
/// wiki's matrix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDef {
    pub id: String,
    pub name: String,
}

/// What a talent with a `choice` is chosen for (`talent:weapon_focus:long_blades`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    /// A weapon skill (an id in `data/weapon_skills.toml` that is no style).
    WeaponSkill,
    /// A fighting style.
    Style,
    /// A skill (an id in `data/skills.toml`), trained or not.
    Skill,
    /// An element: a damage type (an id in `data/damage_types.toml`).
    Element,
    /// A creature tag (one some creature type carries, `goblin`).
    Tag,
    /// A spell tag a talent may focus on (magic design §6): a school (`evocation`), or an
    /// element as [`Choice::Element`] offers them (`fire`).
    School,
}

impl Choice {
    /// What is chosen, in words: "a weapon skill".
    pub fn words(self) -> &'static str {
        match self {
            Choice::WeaponSkill => "a weapon skill",
            Choice::Style => "a fighting style",
            Choice::Skill => "a skill you are trained in",
            Choice::Element => "an element",
            Choice::Tag => "a kind of creature",
            Choice::School => "a school of magic or an element",
        }
    }

    /// Whether a weapon skill of `role` may be chosen as a weapon skill: one that attacks (a
    /// style is chosen as a style).
    fn attacks(role: Role) -> bool {
        role != Role::Style
    }

    /// Whether it chooses a row of the weapon skill table: a weapon skill or a style.
    pub fn is_weapon_skill(self) -> bool {
        matches!(self, Choice::WeaponSkill | Choice::Style)
    }

    /// Every valid choice of this kind, by id, in file order.
    pub fn options(self, content: &Content) -> Vec<String> {
        match self {
            Choice::WeaponSkill => (content.weapon_skills().ids())
                .filter(|(_, def)| Choice::attacks(def.role))
                .map(|(_, def)| def.id.clone())
                .collect(),
            Choice::Style => (content.weapon_skills().styles())
                .map(|(_, def, _)| def.id.clone())
                .collect(),
            Choice::Skill => content.skills().iter().map(|s| s.id.clone()).collect(),
            Choice::Element => (content.damage_types().iter())
                .filter(|t| !t.physical)
                .map(|t| t.id.clone())
                .collect(),
            Choice::Tag => content.tags().into_iter().map(str::to_string).collect(),
            Choice::School => (super::School::ALL.iter().map(|s| s.id().to_string()))
                .chain(Choice::Element.options(content))
                .collect(),
        }
    }

    /// The name of the choice `id`, if it is one of this kind.
    pub fn name<'c>(self, content: &'c Content, id: &str) -> Option<&'c str> {
        match self {
            Choice::WeaponSkill | Choice::Style => {
                let skill = content.weapon_skills().id(id)?;
                let def = content.weapon_skills().skill(skill);
                let fits = match self {
                    Choice::Style => def.role == Role::Style,
                    _ => Choice::attacks(def.role),
                };
                fits.then_some(def.name.as_str())
            }
            Choice::Skill => {
                let skill = content.skill_id(id)?;
                Some(content.skill(skill).name.as_str())
            }
            Choice::Element => {
                let kind = content.damage_type_id(id)?;
                let def = content.damage_type(kind);
                (!def.physical).then_some(def.name.as_str())
            }
            Choice::Tag => content.tags().into_iter().find(|t| *t == id),
            Choice::School => (super::School::ALL.iter())
                .map(|s| s.id())
                .find(|s| *s == id)
                .or_else(|| Choice::Element.name(content, id)),
        }
    }
}

/// What a talent does, one typed effect. Each is read by one rule, where the rule already reads
/// what magic items do (spec §8 "Talents"); a new kind is a variant here, its field in
/// [`crate::talent::TalentMods`], and its one read.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TalentEffect {
    /// Defence in the Dodge category (bonuses add up).
    Dodge { value: i32 },
    /// Natural armour: added to the actor's own, whatever the best item gives.
    Natural { value: i32 },
    /// Damage added to every melee blow (the main hand's and the off hand's).
    MeleeDamage { value: i32 },
    /// Damage added to every attack it makes while below a third of its hit points (Ferocity).
    Ferocity { value: i32 },
    /// Every attack threatens a critical this many natural points lower.
    Threat { value: i32 },
    /// A natural 2 on an attack roll (a fumble) is rolled again, once.
    RerollFumble,
    /// What the holder's allies get (spec §8 "Allies": Beast-bond): to-hit on their attacks
    /// and Protection, read through their master (`world/ally.rs`); and what the holder's summon
    /// spells do (`world/summon_spell.rs`): `count` more come every call, a call lasts
    /// `duration_percent` longer, `lasting` calls stay until slain, `most` more of each call's
    /// are kept at once.
    AllyMod {
        #[serde(default)]
        to_hit: i32,
        #[serde(default)]
        protection: i32,
        #[serde(default)]
        count: i32,
        #[serde(default)]
        duration_percent: i32,
        #[serde(default)]
        lasting: bool,
        #[serde(default)]
        most: i32,
    },
    /// The summon draw's bias (the owner's Q9 of #828): a creature of a summon spell's pool that
    /// carries `tag` is `weight` times as likely to come (0: it never comes while another may).
    SummonBias { tag: String, weight: u32 },
    /// Each melee blow at the creature its last blow was at, in a row (its own turn or the one
    /// before), is +1 to hit more, to `max` (Relentless; `World::streak`); with `damage_every`
    /// (Relentless fury), every that many blows before it in the streak add +1 damage too.
    Streak {
        max: u8,
        #[serde(default)]
        damage_every: u8,
    },
    /// Its off hand's blows do this much more damage (Tentacles).
    OffHandDamage { value: i32 },
    /// Its attacks with its best weapon skill (the highest rank it holds, ties all counting)
    /// threaten a critical this many natural points lower (Tempered).
    BestSkillThreat { value: i32 },
    /// Maximum hit points: 1 per `per_levels` character levels, at least `at_least`, handed out
    /// into the actor's base hit points when the talent is taken and as each later level comes
    /// (as levels' own hit points are), so a talent taken early keeps growing.
    MaxHpPerLevels { per_levels: u8, at_least: i32 },
    /// Maximum hit points, this percent more (negative: fewer; never below 1; Frail).
    MaxHpPercent { value: i32 },
    /// A stat's score and its potential, this much higher: handed out into the actor's stats
    /// when the talent is taken or given (a class ability's gift), never read again.
    Stat { stat: Stat, value: i32 },
    /// Hit points come back this percent faster (negative: slower).
    HealingPercent { value: i32 },
    /// Mana comes back this percent faster (negative: slower; Tidal), on
    /// [`crate::world::World::mana_rate`]'s pace.
    ManaRegenPercent { value: i32 },
    /// Maximum mana: 1 per `per_levels` character levels, at least `at_least`, handed out into
    /// the actor's base mana when the talent is taken and as each later level comes, as
    /// `max_hp_per_levels` does hit points (Deep well).
    MaxManaPerLevels { per_levels: u8, at_least: i32 },
    /// Resting brings mana back this percent faster on top of resting's own pace (Meditation).
    ManaRestingPercent { value: i32 },
    /// What abilities cost in mana, this percent more (negative: less; never below 1; Frugal
    /// caster), in [`crate::world::World::spend_mana`].
    ManaCostPercent { value: i32 },
    /// What rods cost in mana, this percent more (negative: less; never below 1; Channeler; spec
    /// §8 "Magic items": "Rods"), before [`TalentEffect::ManaCostPercent`].
    RodCostPercent { value: i32 },
    /// The active Blood magic: give `hp` hit points (never the last) for `mana` mana.
    BloodMagic { hp: i32, mana: i32 },
    /// Footsteps this percent louder (negative: quieter).
    FootstepsPercent { value: i32 },
    /// The doors the actor opens and closes make no noise.
    SilentDoors,
    /// The actor hears sounds this much fainter (its hearing threshold, lower).
    Hearing { value: i32 },
    /// It places the footsteps it hears exactly, at the cell they came from, never roughly
    /// (Watchful).
    ExactFootsteps,
    /// Its eyes need this percent more light to see by (negative: less; Night eyes), on the
    /// vision profile's threshold ([`crate::world::WorldState::eyes`]).
    ThresholdPercent { value: i32 },
    /// Its darkvision reaches this many cells farther (from none: this far).
    Darkvision { value: i32 },
    /// It sees creatures suffering `status` (an id in `data/status.toml`) in its line of sight
    /// within `radius` cells however dark it is (Blood scent).
    Scent { status: String, radius: i32 },
    /// Its overworld sight at night reaches this many cells farther.
    NightSight { value: i32 },
    /// Its overworld sight by day reaches this many cells farther (Eyes of the heights).
    DaySight { value: i32 },
    /// The lit light it has ready in the Tool slot (a lantern) burns as the light preset `light`
    /// (an id in `data/lights.toml`; Lamplighter), not while hooded.
    LanternLight { light: String },
    /// It sheds light `light` (an id in `data/lights.toml`) wherever it is, always (Inner
    /// light): it sees by it, and so does everything else.
    InnerLight { light: String },
    /// Its blows and shots at a creature that has not seen it do `dice` more damage, after
    /// Protection (Ambusher).
    Ambush { dice: Dice },
    /// This much added to its initiative check when it is caught unawares (Wary;
    /// `World::startle`, `world/surprise.rs`).
    InitiativeBonus { value: i32 },
    /// This percent of the extra cost of the action it takes Surprised is spared (Cool head:
    /// 100, none; `World::surprise_cost`).
    SurpriseEase { value: i32 },
    /// Those it catches unawares find their initiative check this much harder (the thief's
    /// Sneak attack; `World::startle`'s craft).
    SurpriseDifficulty { value: i32 },
    /// Its shots, throws, and attack spells sneak-attack this many cells farther than the
    /// rules' `missile_reach` (Long reach; `World::sneaks`).
    SneakReach { value: i32 },
    /// A sneak attack of its that kills leaves it a degree of `status` (Vanishing act:
    /// Light-footed; read where `Build::SneakKill` is raised, `world/thief.rs`).
    SneakKillStatus { status: String },
    /// The perception snapshot tells which creatures in view have not noticed it, and they are
    /// drawn with the unaware mark (Stalker's eye; `SeenActor::unaware`).
    SeesUnaware,
    /// A batch of arrows or bolts (Fletcher, the ranger's level 1; spec §8 "Fletching"): `check`
    /// more on its check, `count_times` the count after the margin, `time_percent` of its time,
    /// and plain at any Woodcraft.
    Fletcher {
        check: i32,
        count_times: i32,
        time_percent: u32,
    },
    /// An animal companion from the first turn (Animal companion, the ranger's level 1; spec §8
    /// "Allies": `World::bring_companion`): a beast of its race's list, or rarely a rare one.
    Companion,
    /// A familiar from the first turn (Familiar, the witch's level 1; spec §8 "Allies":
    /// `World::bring_familiar`): a small beast of its race's list, or rarely a rare one, whose
    /// touch is her evil eye.
    Familiar,
    /// `bonus` on its taming checks (Animal companion's edge; the Tame skill ability reads it).
    Tame { bonus: i32 },
    /// It sees what its companion sees, by its companion's eyes and nose (Hunter's bond; the
    /// second viewpoint in `Perception::compute`).
    SharedSight,
    /// It sees what its familiar sees, by its familiar's eyes and nose (Familiar's sight; the same
    /// second viewpoint, `familiars.toml`'s `nose` at the least); no buff crosses.
    FamiliarSight,
    /// A buff status that lands on it or its companion lands on the other too (Hunter's bond;
    /// `world/bond.rs`); never a hex.
    SharedBuffs,
    /// A creature beside both it and its companion is flanked: its melee blows and the
    /// companion's at it are `to_hit` likelier to hit (Pack tactics; `world/pack.rs`).
    PackTactics { to_hit: i32 },
    /// Its companion's attacks carry its Favoured prey bonus against its favoured prey (Pack
    /// tactics; `world/pack.rs`).
    SharesFavouredPrey,
    /// An initiative check it makes against a hostile attacker beside it is answered with a
    /// free blow at once (Hair trigger; `Cue::Startled`, `world/free_blow.rs`).
    StartledBlow,
    /// The carrying limit, this percent more.
    CarryPercent { value: i32 },
    /// Being Burdened costs it nothing, as a dwarf's burden (Porter).
    IgnoreBurdened,
    /// Coins it carries weigh nothing.
    WeightlessCoins,
    /// Journeys on the overworld take this percent more time (negative: quicker).
    TravelPercent { value: i32 },
    /// Wading (water, marsh) costs a plain step, and never gives a disease.
    Swimmer,
    /// A step onto any of `terrains` costs what a step onto `like` does (ids in
    /// `data/terrain.toml`; Swimmer, Trailblazer).
    TerrainAs { terrains: Vec<String>, like: String },
    /// A plunge through ice into the water never hurts it (Swimmer).
    PlungeProof,
    /// Ice holds it as if it carried nothing (Winter-born).
    LightOnIce,
    /// Climbing out of the water onto ice costs this percent more (negative: less).
    ClimbPercent { value: i32 },
    /// Wading never gives it a disease (Nature's ward).
    WadingProof,
    /// Creatures of animal factions are neutral to it until it attacks one of that type
    /// (Beast-friend).
    BeastFriend,
    /// Each level reached adds to the family's stipend, which its servant hands over (Stipend:
    /// `data/stipend.toml`, `world/stipend.rs`).
    Stipend,
    /// The stipend's gold is `gold` percent more, its loot roll made by `chance` percent more
    /// (percentage points), and its magic and rare rarities `magic` percent more likely (Landed
    /// noble).
    StipendMore { gold: i32, chance: i32, magic: i32 },
    /// A business partner in town invests the gold put with them (Invest: `data/invest.toml`,
    /// `world/invest.rs`).
    Invest,
    /// Hunger burns this percent faster (negative: slower).
    HungerPercent { value: i32 },
    /// The sickness rotten food gives, this percent more (negative: less), rounded down.
    RotHarmPercent { value: i32 },
    /// It smells how fresh food is: the eat list says fresh, turning, or rotten (Hound's nose).
    SmellRot,
    /// Exerting oneself (a blow, a shot, a throw) burns no extra nutrition.
    NoExertion,
    /// A meal never leaves it Bloated: what does not fit below Bloated is lost (Gourmand).
    NoBloating,
    /// Hunger never weakens it: Weak, Fainting, and Starving take nothing off its Strength and
    /// Dexterity (`Actor::penalty`), and it never faints (`World::faint`; Long fast). Starving
    /// still costs hit points.
    NoHungerWeakness,
    /// A fresh corpse's stat training marks and chances to widen a potential, this percent more
    /// (Gourmand).
    CorpseStatsPercent { value: i32 },
    /// Experience gained, this percent more.
    XpPercent { value: i32 },
    /// Skill rolls every level, this many more (Quick learner).
    SkillRolls { value: i32 },
    /// One more talent pick at character level `level` (1: at creation; Versatile).
    ExtraPick { level: u8 },
    /// Handed out once, as the talent is taken or given: every weapon skill and style of
    /// `skills` (none named: every row of `weapon_skills.toml` but the
    /// hidden styles, whatever the content holds) held below rank `to` rises to it (Travelled
    /// hands; `WeaponSkills::raise_to`). `news` is said when a class table gives it.
    WeaponRanks {
        to: u8,
        #[serde(default)]
        skills: Vec<String>,
        #[serde(default)]
        news: Option<String>,
    },
    /// Handed out once, as the talent is taken or given: every skill of `skills.toml` the holder
    /// is not trained in, but those `except` names, is trained at `dice`, rolled from the
    /// `levels` stream, whatever skills the content holds (Jack of all trades;
    /// `world/jack.rs`). `news` is said when a class table gives it.
    TakeUpSkills {
        dice: crate::dice::Dice,
        #[serde(default)]
        except: Vec<String>,
        #[serde(default)]
        news: Option<String>,
    },
    /// Loot it finds (what generated levels hold, what creatures drop) rolls magic and rare this
    /// percent more often: their rarity weights (Treasure hunter).
    LootMagicPercent { value: i32 },
    /// Reading a scroll takes this percent more time (negative: less; Scroll savant).
    ReadPercent { value: i32 },
    /// Studying a spell (spec §8 "Spells"): `bonus` points off the check's difficulty, it
    /// takes `percent` more time (negative: less; Quick study), and its overreach counts
    /// `overreach_percent` more in the check (negative: less, rounded down; Overreach, the
    /// spell scaling design §7.2). Each defaults to 0.
    Study {
        #[serde(default)]
        bonus: i32,
        #[serde(default)]
        percent: i32,
        #[serde(default)]
        overreach_percent: i32,
    },
    /// A scroll it reads is not used up, by this chance in percent (one gameplay draw a read;
    /// Scroll savant).
    ScrollKept { chance: u8 },
    /// A potion it drinks is not used up, by this chance in percent (one gameplay draw a drink;
    /// Thrifty drinker).
    PotionKept { chance: u8 },
    /// Its own bursts never catch it (Scroll savant).
    SparesSelf,
    /// Added to its searching, not to noticing in passing (Keen searcher).
    Searching { value: i32 },
    /// The damage traps do it, this percent more (negative: less; Trap sense).
    TrapDamagePercent { value: i32 },
    /// The gold a quest gives, this percent more.
    QuestGoldPercent { value: i32 },
    /// Prices move this many percent more in its favour (spec §7 "Shops"), before the cap.
    FavourPercent { value: i32 },
    /// Temple services cost it half, and a cure also mends a point of each damaged stat
    /// (Pious; `world/service.rs`).
    Pilgrim,
    /// Item descriptions show what each gem in a thing is worth.
    GemWorth,
    /// A magic item it has carried this many hours since it held the effect is identified.
    IdentifyCarried { hours: u32 },
    /// Loot it finds and creatures drop is a loose gem this percent more often (Hoard sense):
    /// the gems' chance, never above certain.
    GemFindPercent { value: i32 },
    /// Handed out once, as it is taken (a talent only for character creation: Family wealth):
    /// gold coins, `percent` of those the character carries more, at least `at_least` and at
    /// most `at_most` (`world/heirloom.rs`).
    StartingGold {
        percent: u32,
        at_least: u32,
        at_most: u32,
    },
    /// Handed out once, as it is taken (Family wealth): `min` to `max` loose gems found at item
    /// level `level`, drawn from the stream `character/family-wealth/<race>,<gender>,<class>`.
    StartingGems { min: u8, max: u8, level: u8 },
    /// Handed out once, as it is taken (Father's sword): the race's heirloom weapon (`weapons`,
    /// by race id; `fallback` for a race without one), magic with `min` to `max` affixes rolled at
    /// item level `level` and identified, from the stream
    /// `character/fathers-sword/<race>,<gender>,<class>`; wielded in the main hand when it is
    /// free, else carried.
    Heirloom {
        weapons: BTreeMap<String, String>,
        fallback: String,
        min: u8,
        max: u8,
        level: u8,
    },
    /// The actor notices hidden things in passing as if trained in Spot and Detect traps, with
    /// this much added.
    Notice { value: i32 },
    /// The actor notices hidden things on stone in passing, as a dwarf does.
    StoneSense,
    /// Added to its checks to find what is hidden on stone, searching and in passing (Stone
    /// sense).
    StoneFind { value: i32 },
    /// Added to every check to find hidden things, searching and in passing.
    Search { value: i32 },
    /// The creatures that come to a level the actor is on come this percent less often (the
    /// time between attempts this percent longer; Quiet delver).
    SpawnEveryPercent { value: i32 },
    /// It tells a locked chest from an unlocked one at a glance (Chest-wise;
    /// [`crate::perception::PerceptionView::chest_at`]), and a shut chest's trap in its sight
    /// becomes known to it (`World::chest_wise_look`).
    ChestWise,
    /// A disguised mimic within this many cells is seen for what it is (Chest-wise;
    /// [`crate::perception::Perception`]'s disguises).
    SeesMimics { value: i32 },
    /// A mimic's grip never holds it (Chest-wise; `World::afflict`).
    Slick,
    /// At a forgotten altar of the actor's own side the side's lean counts double, and an
    /// opposed one leans as a neutral one (Pious; the prayer draw, `World::lean`).
    Devout,
    /// Where a special room quickens healing or mana, the quickening is this percent more
    /// (Old delver; the room's pace in `World::healing_rate` and `World::mana_rate`).
    RoomPacePercent { value: i32 },
    /// A special room's (or a whole level's) `status` never takes it (Old delver: Ebbing;
    /// `World::dwell`).
    RoomStatusImmune { status: String },
    /// A collapsed floor never gives way under it (Old delver; `World::collapse_step`).
    CollapseProof,
    /// A tempting hoard seals only after this many stacks more are lifted inside it (Old delver;
    /// `World::lift_in_hoard`, `world/hoard.rs`).
    HoardGrace { value: i32 },
    /// What it kills never rises in a cursed room (Old delver; `World::curse_corpse`).
    GraveWarden,
    /// Searching reaches this many cells farther.
    SearchReach { value: i32 },
    /// A skill counts this much higher wherever its value is read (an untrained one from 0):
    /// `skill`, or left out the chosen one (Skill focus; the talent takes a `skill` choice).
    Skill {
        #[serde(default)]
        skill: Option<String>,
        value: i32,
    },
    /// No one makes way for it (Trusted by none), whatever else would have them.
    Distrusted,
    /// A free blow the actor is given (spec §8 "Free blows"); `one_in` (the gatekeeper's only)
    /// makes it a chance: one in that many, scaled by the stance's `gatekeeper` percent.
    FreeBlow {
        trigger: FreeBlowTrigger,
        #[serde(default)]
        one_in: Option<u32>,
    },
    /// What attacks with some weapon skills gain, as their ranks give it
    /// ([`super::Benefit`]): the weapon skill or style `skill` names, else the chosen one's (no
    /// `role`; the talent takes a choice), else every skill of `role`'s. `ranks`: its benefits read that many ranks higher
    /// (a rank held, never past [`super::MAX_RANK`]); `bash` only where a free bash is had already.
    Weapon {
        #[serde(default)]
        role: Option<Role>,
        /// A weapon skill or style by id: its benefits, whatever the choice (Fencing's duelist).
        #[serde(default)]
        skill: Option<String>,
        #[serde(default)]
        to_hit: i32,
        #[serde(default)]
        damage: i32,
        #[serde(default)]
        threat: u8,
        #[serde(default)]
        ranks: u8,
        #[serde(default)]
        speed: u8,
        #[serde(default)]
        reload: u8,
        #[serde(default)]
        ease: u8,
        #[serde(default)]
        bash: u8,
        #[serde(default)]
        shield: i32,
        /// Added to a confirmed critical's multiplier (never past ×5).
        #[serde(default)]
        crit: u8,
        /// Only for a skill held at this rank at least (0: any).
        #[serde(default)]
        at_rank: u8,
    },
    /// The marks its attacks earn in the weapon skills of `role`, this percent more (Weapon
    /// drill); without a `role`, in every weapon skill and fighting style (Forge-born).
    WeaponMarks {
        #[serde(default)]
        role: Option<Role>,
        value: i32,
    },
    /// A confirmed critical's multiplier with a two-handed weapon, this much more (never past
    /// ×5; Brutal critical).
    TwoHandedCrit { value: u8 },
    /// Its blows with no weapon in hand (and its kicks) roll one die of this many sides instead
    /// of bare hands' dice (Martial arts); the biggest counts.
    UnarmedDie { sides: u8 },
    /// The readied missile weapon shoots this many cells farther; with `skill` (a weapon skill
    /// id), only a launcher of that skill (Slinger's slings).
    LauncherRange {
        value: i32,
        #[serde(default)]
        skill: Option<String>,
    },
    /// Throws reach this many cells farther.
    ThrowRange { value: i32 },
    /// Worn armour's cap on Dexterity in Defence (`max_dex`), this much higher.
    MaxDex { value: i32 },
    /// Defence in the Deflection category, on top of what magic items give.
    Deflection { value: i32 },
    /// Protection, added to what is worn; only while the body armour worn has `body_armour`
    /// Protection at least (if set), only while holding a shield (if `shield`), and once for
    /// every armour piece worn (if `per_piece`).
    Protection {
        value: i32,
        #[serde(default)]
        body_armour: Option<i32>,
        #[serde(default)]
        shield: bool,
        /// For every piece of armour worn (anything worn with Protection of its own).
        #[serde(default)]
        per_piece: bool,
    },
    /// The to-hit penalty of combat stance `stance`, this percent smaller (100: none).
    StanceToHit { stance: CombatStance, percent: i32 },
    /// The Dodge penalty of combat stance `stance`, this percent smaller (100: none; Iron
    /// stance).
    StanceDodge { stance: CombatStance, percent: i32 },
    /// Changing its combat stance costs an action, as a wait does (Set in its ways), not
    /// nothing.
    StanceCosts,
    /// A step that takes it out of reach of every hostile beside it costs this percent more
    /// (negative: cheaper; Slip away), whatever its stance.
    SlipPercent { value: i32 },
    /// Every hit on the actor (a blow, a shot, a throw, a trap) does this much less after its
    /// Protection soaked it, never below 1.
    Pain { value: i32 },
    /// A creature whose melee blow hits it takes `damage` (Barbed hide): no blow, no roll, and
    /// no experience if it dies of it.
    Barbs { damage: i32 },
    /// Resistance in percent to damage type `damage` (an id in `data/damage_types.toml`), else
    /// to the chosen element (the talent takes an `element` choice).
    Resist {
        value: i32,
        #[serde(default)]
        damage: Option<String>,
    },
    /// Resting heals this percent faster on top of resting's own pace (Fast healer).
    RestingPercent { value: i32 },
    /// The active Second wind: `per_level` hit points per character level back at once, at
    /// least `at_least`, then the `cooldown` status (an id in `data/status.toml`), while which
    /// it cannot be used again.
    SecondWind {
        per_level: u8,
        at_least: i32,
        cooldown: String,
    },
    /// Once in its life, a hit, a trap, or a status that would kill it does not (Rise from the
    /// ashes; not starving or choking): it rises with `hp_percent` of its hit points, a burst of
    /// `damage` (`damage_type`) strikes every creature beside it (`spell`, an effect in
    /// `data/effects.toml`, drawn over `radius` cells) and melts the ice there, and it suffers
    /// `status` (Ashen).
    RiseFromAshes {
        hp_percent: i32,
        status: String,
        spell: String,
        radius: i32,
        damage: Dice,
        damage_type: String,
    },
    /// Its rising from the ashes brings it back whole (Reborn whole): every hit point, every
    /// status effect and disease burned away, and none of the rising's own status.
    RebornWhole,
    /// It may rise from the ashes once more (Second rising): a second once in its life.
    SecondRising,
    /// Its hit points come back while it is Starving too (Another head).
    StarvingHeals,
    /// The class active Veteran's wind ([`ClassActive::VeteransWind`]): as Second wind.
    VeteransWind {
        per_level: u8,
        at_least: i32,
        cooldown: String,
    },
    /// The class active Lay on hands: `per_level` hit points per character level back at once
    /// (what it costs is the talent's `resource`: Zeal).
    LayOnHands { per_level: u8 },
    /// Lay on hands heals fully and cures every status that may be cured and every disease, for
    /// `hands_cost_percent` of its cost; and Holy wrath's die lands on every foe (Avatar).
    Avatar { hands_cost_percent: u8 },
    /// The class active Rage: the player is granted `resource` (Rage, spec §8 "Resources") and
    /// suffers its status (Raging) while it lasts; no rage while that or the status it `follows`
    /// into lasts.
    Rage { resource: String },
    /// The class active War cry: a roar as loud as a howl; every hostile that hears it, sees the
    /// player, and has vitals rolls Willpower against `difficulty` + the player's Strength
    /// modifier + their Intimidate; failed, it suffers `severity` degrees of `status` (one that
    /// `flees`; `world/war_cry.rs`).
    WarCry {
        status: String,
        severity: u16,
        difficulty: i32,
    },
    /// The class active Cackle: a voice `noise` loud; every hex of the player's on a creature
    /// they see grows `degrees` and starts its degree afresh, to its status's `max`
    /// (`world/witch.rs`).
    Cackle { degrees: u16, noise: u16 },
    /// Event `on` builds `resource` this much more while it is held, beside what the resource's
    /// own `builds` give (Blood frenzy: 5 rage a kill).
    ResourceBuild {
        resource: String,
        on: Build,
        amount: u16,
    },
    /// `resource` holds this much more at most (Mighty rage: 10 more rage).
    ResourceMax { resource: String, value: u16 },
    /// `resource` is granted holding this much more (Battle trance: rage starts 6 higher).
    ResourceStart { resource: String, value: u16 },
    /// `resource` changes over time this percent faster (negative: slower): one that
    /// regenerates comes back sooner, one that decays wears off sooner (Steady hand, Patient
    /// blade).
    ResourceRate { resource: String, value: i32 },
    /// While raging: the statuses named tick nothing, and a blow that would take the last hit
    /// point leaves it for `cost` of the rage, when that much is held (Unstoppable).
    Unstoppable { statuses: Vec<String>, cost: u16 },
    /// The class active Whirlwind: a blow at every hostile beside it for the price of one
    /// attack.
    Whirlwind,
    /// The class active Smite: a melee blow in a direction with one die of `sides` more damage
    /// per `per_levels` character levels (at least one), `unholy` dice more against an unholy
    /// foe (`data/smite.toml`), and the smiter's modifier in `aim` added to hit (what it costs
    /// is the talent's `resource`: Zeal; `world/smite.rs`).
    Smite {
        per_levels: u8,
        sides: u8,
        #[serde(default)]
        aim: Option<Stat>,
        #[serde(default)]
        unholy: u8,
    },
    /// A mode of smiting (the paladin's; `world/smite.rs`, docs/design/paladin-smites.md §6):
    /// the class active of its `mode` (Branding smite, Wrathful smite), the Smite talent's blow
    /// with a rider on a hit that did damage (what it costs is the talent's `resource`: Zeal).
    /// `brand`: the foe suffers `severity` degrees of `status`. `fear`: the foe makes a `check`
    /// of that stat against `difficulty` plus the smiter's Smite `aim` modifier, and failing
    /// suffers `status` (one that flees) at `turns` degrees, caused by the smiter. `banish`
    /// takes nothing more: a summoned foe is unmade, and an unholy one takes the smite's dice
    /// twice (the rule's, `world/smite.rs`).
    SmiteMode {
        mode: SmiteRiderKind,
        #[serde(default)]
        status: String,
        #[serde(default)]
        severity: u16,
        #[serde(default)]
        turns: Option<Dice>,
        #[serde(default)]
        check: Option<Stat>,
        #[serde(default)]
        difficulty: i32,
    },
    /// Every blow, shot, or throw that hits an unholy target (`Content::unholy`; with Avatar any
    /// target) rolls `dice` more, after the blow's own damage, of its kind, resisted but never
    /// soaked (the paladin's Holy wrath, `world/smite.rs`).
    HolyWrath { dice: Dice },
    /// The class active Holy radiance: a toggle; while it suffers `status` it sheds light
    /// `light` (an id in `data/lights.toml`).
    Radiance { status: String, light: String },
    /// The class active Patient defence: a wait, then a degree of `status` (Poised: Dodge up
    /// and the riposte switched on until the next action; what it costs is the talent's
    /// `resource`: Ki; `world/ascetic.rs`).
    PatientDefence { status: String },
    /// The class active Quivering palm: an unarmed blow in a direction; a hit that wounds
    /// leaves the target `severity` degrees of `status` (what it costs is the talent's
    /// `resource`: Ki).
    QuiveringPalm { status: String, severity: u16 },
    /// The class active Stunning strike (the ascetic's, #848): an unarmed blow in a direction;
    /// a hit that wounds leaves the target `severity` degrees of `status` (Stunned; what it
    /// costs is the talent's `resource`: Ki). Quivering palm's blow with another status.
    StunningStrike { status: String, severity: u16 },
    /// The class active Step of the wind: a degree of `status` (Light-footed) on the player,
    /// taking no time (what it costs is the talent's `resource`: Ki; `world/ascetic.rs`).
    WindStep { status: String },
    /// The class active Stillness (the ascetic's, on Still mind, #850): every status of
    /// `statuses` the player suffers ends at once, taking no time (what it costs is the talent's
    /// `resource`: Ki; `world/ascetic.rs`). `name` is the abilities menu's, as the talent that
    /// carries it has a passive name of its own.
    Stillness { name: String, statuses: Vec<String> },
    /// The active Blaze (a birth sign's, spec §8 "Birth sign"): a burst of `radius` around the
    /// player that does `damage` of kind `damage_type` (an id in `data/damage_types.toml`) to
    /// every creature it reaches but them, melts the ice it reaches, and sets each cell it
    /// reaches but theirs burning for `burns` seconds with light preset `fire` (as a burning
    /// missile where it lands); shown as effect `spell` (`data/effects.toml`); then the
    /// `cooldown` status.
    Blaze {
        radius: i32,
        damage: Dice,
        damage_type: String,
        spell: String,
        fire: String,
        burns: u32,
        cooldown: String,
    },
    /// Blaze bursts out to `radius` with `damage` instead of its own (Wildfire).
    Wildfire { radius: i32, damage: Dice },
    /// The active Fire breath (a birth sign's pick, spec §8 "Birth sign"), aimed at a cell
    /// within [`crate::world::BREATH_RANGE`]: a bolt of `damage` of kind `damage_type` along the
    /// line toward it, `length` cells from the player, through everyone on it; shown as effect
    /// `spell` (a bolt); then `degrees` of the `cooldown` status.
    FireBreath {
        length: i32,
        damage: Dice,
        damage_type: String,
        spell: String,
        cooldown: String,
        degrees: u16,
    },
    /// The active Hound's bay (a birth sign's pick): a howl; every hostile that hears it and is
    /// of a lower level than the player suffers `severity` degrees of `status` (one that
    /// `flees`), then the `cooldown` status.
    HoundsBay {
        status: String,
        severity: u16,
        cooldown: String,
    },
    /// Every level past `hp_level` (data/experience.toml) adds hit points again, as the levels
    /// up to it do, and `bonus` more (Hardened). Handed out as the levels come.
    Hardened { bonus: i32 },
    /// The active Horn's grace (a birth sign's): the statuses `cures` end and up to `diseases`
    /// diseases that show are shaken off, then `degrees` of the `cooldown` status.
    HornsGrace {
        cures: Vec<String>,
        diseases: u8,
        cooldown: String,
        #[serde(default = "one_degree")]
        degrees: u16,
    },
    /// Horn's grace also gives back `per_level` hit points per character level, and leaves
    /// `degrees` of its cooldown instead of its own (Horn of plenty).
    HornOfPlenty { per_level: u8, degrees: u16 },
    /// The active Transfixing stare (a birth sign's): a hostile within
    /// [`crate::world::STARE_RANGE`] cells that the player sees and that sees them suffers
    /// `severity` degrees of `status`, then `degrees` of the `cooldown` status.
    Stare {
        status: String,
        severity: u16,
        cooldown: String,
        #[serde(default = "one_degree")]
        degrees: u16,
    },
    /// The Transfixing stare leaves `severity` degrees of its status and `degrees` of its
    /// cooldown instead of its own (Petrifying gaze).
    PetrifyingGaze { severity: u16, degrees: u16 },
    /// Rotten food never makes it vomit and sickens it half as much; overeating never makes it
    /// vomit (Iron stomach).
    IronStomach,
    /// Rotten food never harms it: no vomiting, no sickness, no disease, as a race that eats
    /// rotten food (Deep roots).
    EatsRotten,
    /// Wading (water, and ground a disease is caught wading in) costs it a plain step (Undertow;
    /// unlike `swimmer`, it may still catch a disease).
    WadesFree,
    /// Status `status` eases this percent faster (100: twice as fast).
    StatusEases { status: String, value: i32 },
    /// Status `status` lasts this percent longer: each degree takes that much more time to
    /// ease (50: half again; Strong constitution's potions).
    StatusLasts { status: String, value: i32 },
    /// Status `status` never lowers its scores.
    NoPenalty { status: String },
    /// A big hit makes it bleed no more, a critical one degree less (Clotting): the statuses
    /// that come of wounds.
    Clotting,
    /// A big hit wounds it this many degrees more (Thin-skinned): the statuses that come of
    /// wounds, where the hit wounds at all.
    WoundMore { value: u16 },
    /// What its statuses take off its scores, this percent more (negative: less; Unbowed).
    PenaltyPercent { value: i32 },
    /// Added to its disease checks: catching one and every progression.
    DiseaseChecks { value: i32 },
    /// Added to its checks against a drain (spec §8 "Stats": "Drain").
    DrainChecks { value: i32 },
    /// Added to every check of stat `stat` it makes (spec §8 "Checks").
    Checks { stat: Stat, value: i32 },
    /// A disease's progressions come this percent further apart (negative: closer; Ageless).
    DiseaseIntervalPercent { value: i32 },
    /// A disease needs this many checks fewer won in a row to be shaken off, never fewer than 1.
    DiseaseRecovery { value: i32 },
    /// It never catches a disease (Divine health, Purity of body): no check, no infection.
    DiseaseImmune,
    /// No status lowers its score in `stat` (Still mind).
    NoPenaltyTo { stat: Stat },
    /// What status `status` raises its scores by, this percent more (Mighty rage: Raging's
    /// Strength and Toughness doubled); lowerings are untouched.
    RaisePercent { status: String, value: i32 },
    /// Its kicks at creatures roll one die of this many sides instead of bare hands' dice.
    KickDie { sides: u8 },
    /// Its door kicks as if this much stronger.
    DoorKick { value: i32 },
    /// A kick of its that hits drives the foe a cell back, when the cell is free (Breaker;
    /// `World::kick_back`).
    KickBack,
    /// Its door kicks are this percent louder (negative: quieter; Lockbreaker).
    KickNoisePercent { value: i32 },
    /// What a chest it kicks open holds breaks this percent more often (negative: less;
    /// Lockbreaker; spec §8 "Chests").
    SmashDestroyPercent { value: i32 },
    /// Its digging does this percent more work a turn.
    DigPercent { value: i32 },
    /// Its blows with a two-handed weapon cost this percent more (negative: cheaper).
    TwoHandedPrice { value: i32 },
    /// Its melee attacks cost this percent more (negative: cheaper; Blood frenzy while raging).
    AttackPricePercent { value: i32 },
    /// A critical it confirms in melee leaves the target suffering `status` (one degree).
    CriticalStatus { status: String },
    /// A creature it hits in melee suffers `status` (one degree, caused by it), and is held by
    /// it: the status lasts until it misses that creature, strikes another, or moves (Drowning
    /// grip; the status `blocks` the creature's steps).
    Grip { status: String },
    /// Added to the roll that confirms its criticals (every attack).
    CriticalConfirm { value: i32 },
    /// Its weapon abilities come this many ranks sooner (Early mastery; the combat progression
    /// design §11): their thresholds read its ranks that much higher.
    AbilityRanks { value: i32 },
    /// Its weapon actives take this percent more time (negative: less; Practised technique),
    /// never below half the action's price (#564).
    WeaponEnergyPercent { value: i32 },
    /// Its weapon actives cost `value` points less stamina, at least 1: those named in
    /// `abilities` (ids in `data/weapon_abilities/`), or every one when none are named
    /// (Drillmaster; Armour-breaker's Sunder and Gap shot).
    WeaponStamina {
        value: i32,
        #[serde(default)]
        abilities: Vec<String>,
    },
    /// Its Brace's blow is at `to_hit` more, and a foe it hits suffers `status` (one degree;
    /// Brace of steel).
    Brace { to_hit: i32, status: String },
    /// Its confirmed criticals ignore this much more Protection (every attack; Armour-breaker).
    CriticalPenetration { value: i32 },
    /// Its blows that can do damage of kind `damage` (an id in `data/damage_types.toml`) ignore this
    /// much more Protection.
    Penetration { damage: String, value: i32 },
    /// The burden's cap on its Dexterity in Defence, this much higher.
    BurdenDex { value: i32 },
    /// Percent off the burden's slowdown, on top of what Balance takes off (60: Burdened ×1.10
    /// instead of ×1.25).
    BurdenEase { value: i32 },
    /// A trap it steps onto first gets a Dexterity check against its difficulty: won, it does
    /// not go off under it, and it is found (Light step).
    SureStep,
    /// Pits never catch it (Light step, Bedrock).
    PitProof,
    /// It can disarm traps (the Disarm ability) without Disable device.
    Disarm,
    /// It moves and acts this percent quicker (its speed; the number is hidden in the game).
    SpeedPercent { value: i32 },
    /// Its steps cost this percent more (negative: cheaper), only while it wears no body armour
    /// (if `unarmoured`), is Unburdened (if `unburdened`), and holds no shield (if `no_shield`):
    /// Fleet; Headlong, while raging.
    StepPercent {
        value: i32,
        #[serde(default)]
        unarmoured: bool,
        #[serde(default)]
        unburdened: bool,
        #[serde(default)]
        no_shield: bool,
    },
    /// Strained costs it what Burdened costs others (Beast of burden).
    Stout,
    /// Dodge added against shots, throws, and trap missiles.
    MissileDodge { value: i32 },
    /// Dodge added against the attacks of creatures larger than it (Small target).
    SmallTarget { value: i32 },
    /// The damage bursts and bolts (traps, scrolls) do it, this percent more (negative: less).
    SpellDamagePercent { value: i32 },
    /// A caster's spell (design §4.1 B2, spec §8 "Spells"): its holder has the spell ability
    /// `spell`, the spell of that id in `data/spells/` (a class ability's; each given once).
    Spell { spell: String },
    /// What it does to its spells through the six hooks (spec §8 "Spells": "Hooks";
    /// `content/spell_mod.rs`): rungs, dice, rerolls, riders, reach, degrees, and a cost
    /// percent, for every spell or those of its `when` tags (a talent chosen for a school or an
    /// element: those of the choice). Empower, Archmage, Keen mind, School focus, Scroll
    /// adept, Wand mastery, Sure hand, Brimming.
    SpellMod(super::SpellModSpec),
    /// What it does to a spell type's own rule (`[types]` in `data/spells.toml`) for the damage
    /// types in `when`: `arc_chance`, the percent its spells of them arc at, read in
    /// `world/spell_arc.rs` (Conductor).
    SpellType { arc_chance: u8, when: Vec<String> },
    /// Its spells cost this percent more mana (negative: less; Archmage, Empower), at least 1.
    SpellManaPercent { value: i32 },
    /// Defence in the Deflection category, only while it wears no body armour (Mage armour).
    DeflectionUnarmoured { value: i32 },
    /// Rotten food neither sickens it nor exposes it to a disease (Purify).
    Purify,
    /// A corpse's own harm (`on_eat`: sickness, vomiting) never touches it (Herb lore,
    /// Cauldron).
    CorpseHarmless,
    /// Eating `object` (an id in `data/objects/`; none: a fresh corpse) also heals `dice` hit
    /// points (Herb lore's angelica root, Cauldron's corpses).
    MealHeals {
        #[serde(default)]
        object: Option<String>,
        dice: Dice,
    },
    /// Its blows that hit a creature suffering `status` do `value` more damage (Poison lore).
    StatusDamage { status: String, value: i32 },
    /// When a creature dies under a hex its holder laid (`Content::hex_statuses`), whoever
    /// killed it, the holder's mana rises by the dead creature's level divided by `per_levels`,
    /// at least 1, never past its maximum (Malison; `world/malison.rs`).
    HexTithe { per_levels: u8 },
    /// Once in its life (`once` names it), what would kill it does not (not starving or
    /// choking): it is left at 1 hit point, suffers `status`, and `stat` is `by` lower for good;
    /// `message` is said (Miracle, Hag's bargain).
    Reprieve {
        once: String,
        status: String,
        #[serde(default)]
        stat: Option<Stat>,
        #[serde(default)]
        by: i32,
        message: String,
    },
    /// Against the creature type it has killed most (ties: the first to get there), its attacks
    /// are this much likelier to hit and do this much more damage (Favoured prey).
    FavouredPrey { to_hit: i32, damage: i32 },
    /// Against creature types that carry the tag it was taken for (a `choice = "tag"`), its
    /// attacks are this much likelier to hit and do this much more damage (Sworn foe).
    SwornFoe { to_hit: i32, damage: i32 },
    /// It places every sound it hears where it was made (Tracker).
    ExactSounds,
    /// Its steps make no sound while it is Unburdened (Hide in plain sight).
    SilentUnburdened,
    /// Its attacks are this percent louder (negative: quieter).
    AttackNoisePercent { value: i32 },
    /// Hidden traps are this much easier for it to find (none carries it since #684).
    TrapEase { value: i32 },
    /// It can pick locks (the Pick lock ability) without Open lock.
    PickLock,
    /// It can steal (the Steal ability) without the Steal skill.
    Steal,
    /// It can throw its voice (the Throw your voice ability; the bard's).
    ThrowVoice,
    /// Its attacks on a creature that has not seen it threaten a critical this many points
    /// lower (Master thief).
    AmbushThreat { value: u8 },
    /// The class active Master thief: its finisher, a melee blow in a direction that is a
    /// critical if it hits, with no confirmation roll (its cost is the talent's `resource`).
    Finisher,
    /// The class active Lurk: a wait in ambush, a degree of `status` on it (Lurking: a hostile
    /// stepping up beside it takes its blow at once; `world/thief.rs`).
    Lurk { status: String },
    /// The class active Low blow: a melee blow in a direction that is a sneak attack whatever
    /// the foe knows (its cost is the talent's `resource`: Openings; `world/thief.rs`).
    LowBlow,
    /// Every skill it is trained in counts this much higher (Jack of all trades).
    TrainedSkills { value: i32 },
    /// It holds alternative resource `resource` (spec §8 "Resources") from the moment it has the
    /// talent, for good (Rapid shot's Focus).
    GrantsResource { resource: String },
    /// A shot it looses while it holds `cost` of `resource` spends them and takes `saving`
    /// percent less time (Rapid shot, on Focus).
    RapidShot {
        resource: String,
        cost: u16,
        saving: u8,
    },
    /// Each second blow of its flurry spends `cost` of `resource`; holding less, it strikes
    /// none (Flurry, on Ki).
    FlurryCost { resource: String, cost: u16 },
    /// A hostile's melee attack beside it that lands a blow on it is answered by its smite at
    /// once, free, spending `cost` of `resource`; with less, nothing (Retribution, the
    /// paladin's at 25, on zeal; `world/smite.rs`).
    Retribution { resource: String, cost: u16 },
    /// A missile (a shot, a throw, a trap's) that would hit it while its hands hold no weapon
    /// and no shield spends `cost` of `resource` by itself and misses; holding less, it hits
    /// (Diamond soul's catch, on Ki; `world/ascetic.rs`).
    CatchMissiles { resource: String, cost: u16 },
    /// The class active Volley: `shots` shots at a cell for the price of one (what it costs is
    /// the talent's `resource`, if anything).
    Volley { shots: u8 },
    /// The class active Sic 'em (`world/sic.rs`): the companion is set on a creature the player
    /// sees within `range`, and suffers `blows` degrees of `status`, one that makes its hits
    /// criticals (`auto_critical`; what it costs is the talent's `resource`).
    Sic {
        status: String,
        blows: u16,
        range: i32,
    },
    /// A song (spec §8 "Class abilities", the bard's songs; `world/song.rs`): class active `id`
    /// (`use:inspire`), sung aloud. Who it is `on` suffers `severity` degrees of `status`, and the
    /// singer's Music's `song_degrees` more; it costs **one** thing, `cost` of `resource`
    /// (Inspire's 3 Inspiration) or `mana`. A song on its hearers asks each hostile one a
    /// `check` of that stat against `difficulty` plus the singer's `song_craft` (the Lullaby's,
    /// #873).
    Song {
        id: String,
        on: SongOn,
        status: String,
        #[serde(default = "one_degree")]
        severity: u16,
        #[serde(default)]
        resource: Option<String>,
        #[serde(default)]
        cost: u16,
        #[serde(default)]
        mana: i32,
        #[serde(default)]
        check: Option<Stat>,
        #[serde(default)]
        difficulty: i32,
    },
    /// Every song costs nothing and gives a degree more; a song on the singer gives `status`
    /// instead (Virtuoso: Inspired greatly).
    Virtuoso { status: String },
    /// The class active Consecrate: `time` (a duration) of prayer, for `mana` mana, lifts an
    /// unholy special room (or a whole level's unholy effect) for good (`world/consecrate.rs`).
    Consecrate { mana: i32, time: String },
    /// Communion with the dark (the priest design §6.3, `world/dark_face.rs`): standing in an
    /// unholy place, its prayers add no strain and the place's status never touches it. A
    /// band-gated passive: outside its talent's `alignment` it does nothing.
    Communion,
    /// The class active Sanctuary: `time` (a duration) of a kneeling rite, for `mana` mana,
    /// hallows the ground within `radius` of the player for `lasts`: no hostile sets foot in it,
    /// and those in it heal and regain mana `regen` percent as fast (`world/sanctuary.rs`).
    Sanctuary {
        mana: i32,
        time: String,
        radius: u8,
        lasts: String,
        regen: u16,
    },
    /// The priest's prayer (the class active Orison, `world/orison.rs`; `data/prayer.toml`):
    /// a prayer talent of `tier` (1 to 5, its `title` for each face) or a use added between
    /// them; `uses` it may answer, `answers` problems a prayer (the largest held), `all` every
    /// problem and every pool at once. The tier-1 talent carries the ability.
    Prayer {
        #[serde(default)]
        tier: Option<u8>,
        #[serde(default)]
        title: Option<super::Faced>,
        #[serde(default)]
        uses: Vec<super::PrayerUse>,
        #[serde(default)]
        answers: Option<u8>,
        #[serde(default)]
        all: bool,
    },
    /// The class active Knock: for `mana` mana, the lock of the locked door or chest in a
    /// direction springs open (`world/knock.rs`; spec §8 "Chests").
    Knock { mana: i32 },
    /// The class active Scribe: a scroll of a spell the player knows written on a blank scroll,
    /// for `mana_times` the spell's mana and a study's time (`world/scribe.rs`; spec §8
    /// "Scrolls").
    Scribe { mana_times: u8 },
    /// The class active Merchant's eye: a stack the player carries is identified, for the time of
    /// a reading (`world/merchants_eye.rs`; spec §8 "Class abilities").
    MerchantsEye,
    /// The class active Spirit weapon: a melee weapon of spirit-stuff called into the hand in a
    /// shape chosen (`world/spirit_weapon.rs`; spec §8 "Prestige classes"), which strikes
    /// `to_hit` surer, threatens `threat` lower, and ignores all Protection if
    /// `ignores_protection`; `phrase` is its description's power line.
    SpiritWeapon {
        to_hit: i32,
        threat: u8,
        #[serde(default)]
        ignores_protection: bool,
        phrase: String,
    },
}

/// One degree: a cooldown's default.
fn one_degree() -> u16 {
    1
}

/// A talent's own active (spec §8 "Talents"; the abilities menu's Talents group), used as
/// [`crate::action::Action::UseTalent`] with the one verb every ability use has
/// (`use:second_wind`, `use:blood_magic`; #644).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum TalentActive {
    /// Second wind: hit points back at once, then Winded for a while.
    SecondWind,
    /// Blood magic: some hit points (never the last) for some mana.
    BloodMagic,
}

impl TalentActive {
    /// Every one, in menu order.
    pub const ALL: &[TalentActive] = &[TalentActive::SecondWind, TalentActive::BloodMagic];

    /// Its id: its effect's kind, and its replay text (`use:second_wind`).
    pub const fn id(self) -> &'static str {
        match self {
            TalentActive::SecondWind => "second_wind",
            TalentActive::BloodMagic => "blood_magic",
        }
    }

    /// The one with id `id`.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.id() == id)
    }
}

/// The free blows a talent may switch on ([`crate::world::FreeBlows`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeBlowTrigger {
    Cleave,
    GreatCleave,
    Riposte,
    Opportunist,
    /// A melee attack with nothing in hand strikes twice, the second blow at −2 (Flurry).
    Flurry,
    /// A hostile that steps up beside it is struck as it comes (Gatekeeper).
    Gatekeeper,
}

/// What a mode of smiting does besides the blow ([`TalentEffect::SmiteMode`]): each is its own
/// class active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmiteRiderKind {
    /// The foe hit suffers a status (Branded: it sheds light; Branding smite).
    Brand,
    /// The foe hit checks its nerve or flees (Frightened; Wrathful smite).
    Fear,
    /// The summoned foe hit is unmade, the unholy takes the dice twice (Banishing smite).
    Banish,
}

impl SmiteRiderKind {
    /// The class active of this mode.
    pub const fn active(self) -> super::ClassActive {
        match self {
            SmiteRiderKind::Brand => super::ClassActive::BrandingSmite,
            SmiteRiderKind::Fear => super::ClassActive::WrathfulSmite,
            SmiteRiderKind::Banish => super::ClassActive::BanishingSmite,
        }
    }
}

/// Whom a song is sung on ([`TalentEffect::Song`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SongOn {
    /// The singer suffers its status (Inspire).
    #[serde(rename = "self")]
    Singer,
    /// Every hostile creature that hears it checks against it (the Lullaby, #873).
    Hearers,
}

impl TalentEffect {
    /// The birth sign's active it is, if it is one (spec §8 "Birth sign").
    pub fn sign_active(&self) -> Option<super::SignActive> {
        use super::SignActive as A;
        Some(match self {
            TalentEffect::HornsGrace { .. } => A::HornsGrace,
            TalentEffect::Stare { .. } => A::TransfixingStare,
            TalentEffect::Blaze { .. } => A::Blaze,
            TalentEffect::FireBreath { .. } => A::FireBreath,
            TalentEffect::HoundsBay { .. } => A::HoundsBay,
            _ => return None,
        })
    }

    /// The class active it is, if it is one (spec §8 "Class abilities").
    pub fn active(&self) -> Option<super::ClassActive> {
        use super::ClassActive as A;
        Some(match self {
            TalentEffect::Rage { .. } => A::Rage,
            TalentEffect::WarCry { .. } => A::WarCry,
            TalentEffect::Cackle { .. } => A::Cackle,
            TalentEffect::Familiar => A::CallFamiliar,
            TalentEffect::VeteransWind { .. } => A::VeteransWind,
            TalentEffect::Whirlwind => A::Whirlwind,
            TalentEffect::LayOnHands { .. } => A::LayOnHands,
            TalentEffect::Smite { .. } => A::Smite,
            TalentEffect::SmiteMode { mode, .. } => mode.active(),
            TalentEffect::Radiance { .. } => A::HolyRadiance,
            TalentEffect::PatientDefence { .. } => A::PatientDefence,
            TalentEffect::StunningStrike { .. } => A::StunningStrike,
            TalentEffect::QuiveringPalm { .. } => A::QuiveringPalm,
            TalentEffect::WindStep { .. } => A::StepOfTheWind,
            TalentEffect::Stillness { .. } => A::Stillness,
            TalentEffect::Volley { .. } => A::Volley,
            TalentEffect::Sic { .. } => A::Sic,
            TalentEffect::Lurk { .. } => A::Lurk,
            TalentEffect::LowBlow => A::LowBlow,
            TalentEffect::Finisher => A::MasterThief,
            TalentEffect::Song { id, .. } => return A::from_id(id).filter(|a| a.sings()),
            TalentEffect::Consecrate { .. } => A::Consecrate,
            TalentEffect::Sanctuary { .. } => A::Sanctuary,
            TalentEffect::Prayer { tier: Some(1), .. } => A::Orison,
            TalentEffect::Knock { .. } => A::Knock,
            TalentEffect::Scribe { .. } => A::Scribe,
            TalentEffect::MerchantsEye => A::MerchantsEye,
            TalentEffect::SpiritWeapon { .. } => A::SpiritWeapon,
            _ => return None,
        })
    }

    /// The name of the class active it is, where that is not its talent's (Stillness, on Still
    /// mind; Call familiar, on Familiar).
    pub fn active_name(&self) -> Option<&str> {
        match self {
            TalentEffect::Stillness { name, .. } => Some(name),
            TalentEffect::Familiar => Some("Call familiar"),
            _ => None,
        }
    }

    /// Whether it is a passive that works only within its talent's `alignment` bands (Communion
    /// with the dark, Retribution; `docs/dev/alignment.md`), as a holy active's gift does.
    pub fn band_gated(&self) -> bool {
        matches!(
            self,
            TalentEffect::Communion | TalentEffect::Retribution { .. }
        )
    }

    /// Whether it is handed down once as it is taken (`world/heirloom.rs`): Family wealth's gold
    /// and gems, Father's sword's heirloom.
    pub fn is_handed_down(&self) -> bool {
        matches!(
            self,
            TalentEffect::StartingGold { .. }
                | TalentEffect::StartingGems { .. }
                | TalentEffect::Heirloom { .. }
        )
    }

    /// Whether it may carry a `while` condition: only a kind the one sum reads
    /// ([`crate::talent::Talents::mods`]), never one read off the talent's effects directly (an
    /// active, a spell given, what is rolled or handed down once when it is taken).
    fn may_be_conditional(&self) -> bool {
        self.active().is_none()
            && !matches!(
                self,
                TalentEffect::SecondWind { .. }
                    | TalentEffect::BloodMagic { .. }
                    | TalentEffect::Spell { .. }
                    | TalentEffect::Prayer { .. }
                    | TalentEffect::Communion
                    | TalentEffect::SpellMod(_)
                    | TalentEffect::SpellType { .. }
                    | TalentEffect::WeaponStamina { .. }
                    | TalentEffect::WeaponRanks { .. }
                    | TalentEffect::TakeUpSkills { .. }
                    | TalentEffect::ExtraPick { .. }
                    | TalentEffect::StartingGold { .. }
                    | TalentEffect::StartingGems { .. }
                    | TalentEffect::Heirloom { .. }
                    | TalentEffect::TerrainAs { .. }
                    | TalentEffect::Scent { .. }
                    | TalentEffect::NightSight { .. }
            )
    }

    /// Whether what it names resolves in `content` (statuses, resources, damage types, skills,
    /// weapon skills, races, lights, spells, effects), its times parse, and its choice-keyed
    /// forms have the choice they read; content without a table has nothing to check those ids
    /// by: they stay, and do nothing. Numbers are the data author's (#1205): only a die's sides
    /// and a divisor are kept from 0.
    pub(super) fn check(&self, content: &Content, choice: Option<Choice>) -> Result<(), String> {
        let status = |id: &str| {
            (content.status_ids().next().is_none() || content.status_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown status `{id}`"))
        };
        let resource = |id: &str| {
            (content.resources().is_empty() || content.resource_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown resource `{id}`"))
        };
        let damage_type = |id: &str| {
            (content.damage_types().is_empty() || content.damage_type_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown damage type `{id}`"))
        };
        let light = |id: &str| {
            (content.light_presets().is_empty() || content.light_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown light preset `{id}`"))
        };
        let weapon_skill = |id: &str| {
            let table = content.weapon_skills();
            (table.skills().is_empty() || table.id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown weapon skill `{id}`"))
        };
        let skill = |id: &str| {
            (content.skills().is_empty() || content.skill_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown skill `{id}`"))
        };
        // A die's sides and a divisor: 0 would roll an empty range or divide by zero.
        let nonzero = |kind: &str, field: &str, v: u32| {
            (v != 0)
                .then_some(())
                .ok_or_else(|| format!("`{kind}`: {field} is at least 1"))
        };
        let duration = |text: &str| text.parse::<crate::time::Duration>().map(|_| ());
        match self {
            TalentEffect::GrantsResource { resource: id }
            | TalentEffect::RapidShot { resource: id, .. }
            | TalentEffect::FlurryCost { resource: id, .. }
            | TalentEffect::Retribution { resource: id, .. }
            | TalentEffect::CatchMissiles { resource: id, .. }
            | TalentEffect::Rage { resource: id }
            | TalentEffect::ResourceBuild { resource: id, .. }
            | TalentEffect::ResourceMax { resource: id, .. }
            | TalentEffect::ResourceStart { resource: id, .. }
            | TalentEffect::ResourceRate { resource: id, .. } => resource(id),
            TalentEffect::Sic { status: id, .. }
            | TalentEffect::Virtuoso { status: id }
            | TalentEffect::RoomStatusImmune { status: id }
            | TalentEffect::RaisePercent { status: id, .. }
            | TalentEffect::VeteransWind { cooldown: id, .. }
            | TalentEffect::SecondWind { cooldown: id, .. }
            | TalentEffect::Lurk { status: id }
            | TalentEffect::SneakKillStatus { status: id }
            | TalentEffect::WindStep { status: id }
            | TalentEffect::QuiveringPalm { status: id, .. }
            | TalentEffect::StunningStrike { status: id, .. }
            | TalentEffect::PatientDefence { status: id }
            | TalentEffect::WarCry { status: id, .. }
            | TalentEffect::StatusDamage { status: id, .. }
            | TalentEffect::Scent { status: id, .. }
            | TalentEffect::StatusEases { status: id, .. }
            | TalentEffect::StatusLasts { status: id, .. }
            | TalentEffect::Brace { status: id, .. }
            | TalentEffect::NoPenalty { status: id }
            | TalentEffect::CriticalStatus { status: id }
            | TalentEffect::Grip { status: id }
            | TalentEffect::Reprieve { status: id, .. } => status(id),
            TalentEffect::Song {
                id,
                status: status_id,
                resource: pool,
                ..
            } => {
                if (super::ClassActive::from_id(id)).is_none_or(|a| !a.sings()) {
                    return Err(format!("`song`: `{id}` is no song's active"));
                }
                if let Some(pool) = pool {
                    resource(pool)?;
                }
                status(status_id)
            }
            TalentEffect::Consecrate { time, .. } => duration(time),
            TalentEffect::Sanctuary { time, lasts, .. } => {
                duration(time)?;
                duration(lasts)
            }
            TalentEffect::Skill { skill: name, .. } => match (name, choice) {
                (Some(name), _) => skill(name),
                (None, Some(Choice::Skill)) => Ok(()),
                (None, _) => Err(
                    "`skill` names a skill, or is for the chosen one: `choice = \"skill\"`".into(),
                ),
            },
            TalentEffect::Heirloom { weapons, .. } => {
                for race in weapons.keys() {
                    if !content.races().list.is_empty() && content.race(race).is_none() {
                        return Err(format!("`heirloom`: names an unknown race `{race}`"));
                    }
                }
                // Its weapons are checked once the objects load (`check_heirlooms`).
                Ok(())
            }
            TalentEffect::WeaponRanks { skills, .. } => {
                skills.iter().try_for_each(|id| weapon_skill(id))
            }
            TalentEffect::TakeUpSkills { except, .. } => except.iter().try_for_each(|id| skill(id)),
            TalentEffect::UnarmedDie { sides } => {
                nonzero("unarmed_die", "sides", u32::from(*sides))
            }
            TalentEffect::KickDie { sides } => nonzero("kick_die", "sides", u32::from(*sides)),
            TalentEffect::Smite {
                per_levels, sides, ..
            } => {
                nonzero("smite", "per_levels", u32::from(*per_levels))?;
                nonzero("smite", "sides", u32::from(*sides))
            }
            TalentEffect::HexTithe { per_levels } => {
                nonzero("hex_tithe", "per_levels", u32::from(*per_levels))
            }
            TalentEffect::MaxHpPerLevels { per_levels, .. } => {
                nonzero("max_hp_per_levels", "per_levels", u32::from(*per_levels))
            }
            TalentEffect::MaxManaPerLevels { per_levels, .. } => {
                nonzero("max_mana_per_levels", "per_levels", u32::from(*per_levels))
            }
            TalentEffect::FreeBlow {
                one_in: Some(n), ..
            } => nonzero("free_blow", "one_in", *n),
            TalentEffect::SmiteMode {
                mode, status: id, ..
            } => match mode {
                SmiteRiderKind::Banish => Ok(()),
                _ => status(id),
            },
            TalentEffect::Stillness { statuses, .. }
            | TalentEffect::Unstoppable { statuses, .. } => {
                statuses.iter().try_for_each(|id| status(id))
            }
            TalentEffect::Blaze {
                damage_type: kind,
                fire,
                cooldown,
                ..
            } => {
                damage_type(kind)?;
                light(fire)?;
                status(cooldown)
            }
            TalentEffect::Radiance {
                status: id,
                light: l,
            } => {
                status(id)?;
                light(l)
            }
            TalentEffect::InnerLight { light: l } | TalentEffect::LanternLight { light: l } => {
                light(l)
            }
            TalentEffect::Resist {
                damage: Some(kind), ..
            }
            | TalentEffect::Penetration { damage: kind, .. } => damage_type(kind),
            TalentEffect::Resist { damage: None, .. } => match choice {
                Some(Choice::Element) => Ok(()),
                _ => Err(
                    "`resist` without a `damage` is for the chosen element: it needs \
                     `choice = \"element\"`"
                        .into(),
                ),
            },
            TalentEffect::HornsGrace {
                cures, cooldown, ..
            } => {
                cures.iter().try_for_each(|id| status(id))?;
                status(cooldown)
            }
            TalentEffect::Stare {
                status: id,
                cooldown,
                ..
            }
            | TalentEffect::HoundsBay {
                status: id,
                cooldown,
                ..
            } => {
                status(id)?;
                status(cooldown)
            }
            TalentEffect::FireBreath {
                damage_type: kind,
                spell,
                cooldown,
                ..
            } => {
                damage_type(kind)?;
                super::spell::check_effect(content, spell, super::effect::EffectShape::Bolt)?;
                status(cooldown)
            }
            TalentEffect::RiseFromAshes {
                status: id,
                spell,
                damage_type: kind,
                ..
            } => {
                damage_type(kind)?;
                super::spell::check_effect(content, spell, super::effect::EffectShape::Burst)?;
                status(id)
            }
            TalentEffect::Weapon { role, skill, .. } => match skill {
                Some(id) => weapon_skill(id),
                None if role.is_some() || choice.is_some_and(Choice::is_weapon_skill) => Ok(()),
                None => Err(
                    "`weapon` without a `role` or a `skill` is for the chosen weapon skill \
                     or style: it needs that `choice`"
                        .into(),
                ),
            },
            TalentEffect::SpellMod(spec) => spec.check(content),
            TalentEffect::SpellType { when, .. } => when.iter().try_for_each(|id| damage_type(id)),
            TalentEffect::LauncherRange {
                skill: Some(id), ..
            } => weapon_skill(id),
            // A spell ability names a spell of data/spells/ of its id (content without spells
            // has nothing to check it by).
            TalentEffect::Spell { spell } => {
                if content.spells().is_empty() || content.spell_id(spell).is_some() {
                    Ok(())
                } else {
                    Err(format!("`spell`: no spell `{spell}` in data/spells/"))
                }
            }
            // Terrain is checked once it has loaded ([`TalentBook::check_terrain`]); the weapon
            // abilities `weapon_stamina` names once they have (`content/weapon_ability.rs`).
            _ => Ok(()),
        }
    }
}

/// One prerequisite of a talent, as it is checked and as it is shown ("Toughness 14").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Need {
    /// Another talent.
    Talent(TalentId),
    /// A stat score (the stored score, never the effective one) at least this.
    Stat(Stat, u8),
    /// A skill trained, at this value at least.
    Skill(SkillId, u16),
    /// A weapon skill or style at this rank at least.
    WeaponSkill(WeaponSkillId, u8),
    /// Any one of several weapon skills at a rank at least ("a melee weapon skill 3").
    AnyWeaponSkill(AnyWeaponSkill),
    /// The chosen weapon skill or style at this rank at least (talents with a `choice`).
    Chosen(u8),
    /// A character level.
    Level(u8),
    /// A race, by id.
    Race(String),
    /// Eyes: the race's vision profile, any one of these ids.
    Eyes(Vec<String>),
    /// A class, by id.
    Class(String),
    /// A birth sign's animal, by id (spec §8 "Birth sign").
    Animal(String),
    /// A birth sign's element, by id.
    Element(String),
    /// Born on a lucky day.
    Lucky,
    /// A new character: taken only at character creation (level 1's creation picks).
    Creation,
    /// Any one of these ("Intelligence 12 or Appraise 20").
    Either(Vec<Need>),
}

impl Need {
    /// The prerequisite in words, as a refusal and the Talents page say it: "Toughness 14",
    /// "Hardy", "Stealth 25", "to be a dwarf".
    pub fn words(&self, content: &Content) -> String {
        match self {
            Need::Talent(id) => content.talents().get(*id).name.clone(),
            Need::Stat(stat, score) => format!("{} {score}", stat.name()),
            Need::Skill(id, value) => format!("{} {value}", content.skill(*id).name),
            Need::WeaponSkill(id, rank) => {
                format!("{} {rank}", content.weapon_skills().skill(*id).name)
            }
            Need::AnyWeaponSkill(any) => format!("{} {}", any.words, any.rank),
            Need::Chosen(rank) => format!("the chosen one at {rank}"),
            Need::Level(level) => format!("level {level}"),
            Need::Race(id) => {
                let name = content.race(id).map_or(id.as_str(), |r| r.name.as_str());
                format!("to be {} {}", article(name), name.to_lowercase())
            }
            Need::Class(id) => {
                let name = content.class(id).map_or(id.as_str(), |c| c.name.as_str());
                format!("to be {} {}", article(name), name.to_lowercase())
            }
            Need::Eyes(ids) => {
                let names: Vec<String> = (ids.iter())
                    .map(|id| content.vision(id).map_or(id.as_str(), |v| v.name.as_str()))
                    .map(str::to_lowercase)
                    .collect();
                format!("{} eyes", crate::talent::list_with(&names, "or"))
            }
            Need::Animal(id) => {
                let name = (content.horoscope())
                    .and_then(|h| h.animals.iter().find(|a| &a.id == id))
                    .map_or(id.as_str(), |a| a.name.as_str());
                format!("to be born in the year of the {name}")
            }
            Need::Element(id) => {
                let name = (content.horoscope())
                    .and_then(|h| h.elements.iter().find(|e| &e.id == id))
                    .map_or(id.as_str(), |e| e.name.as_str());
                format!("to be {name}-born")
            }
            Need::Lucky => "to be born on a lucky day".into(),
            Need::Creation => "to be a new character".into(),
            Need::Either(needs) => (needs.iter())
                .map(|n| n.words(content))
                .collect::<Vec<_>>()
                .join(" or "),
        }
    }
}

/// A prerequisite met by any one of several weapon skills at a rank (`any_weapon_skill`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnyWeaponSkill {
    /// The skills, in file order.
    pub skills: Vec<WeaponSkillId>,
    pub rank: u8,
    /// How it is said: "a melee or unarmed weapon skill", "Maces and clubs or Unarmed".
    pub words: String,
}

/// The prerequisites of a talent (spec §8 "Talents"): all must hold
/// when it is chosen; none is checked again later.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Needs {
    pub talents: Vec<TalentId>,
    /// Scores by stat; 0 asks nothing.
    pub stats: PerStat<u8>,
    pub skills: Vec<(SkillId, u16)>,
    pub weapon_skills: Vec<(WeaponSkillId, u8)>,
    pub any_weapon_skill: Option<AnyWeaponSkill>,
    pub chosen: Option<u8>,
    /// 0 asks nothing.
    pub level: u8,
    pub race: Option<String>,
    /// The race's vision profiles, any one of which will do; empty asks nothing.
    pub eyes: Vec<String>,
    pub class: Option<String>,
    /// The birth sign's animal and element asked, by id, and a lucky day.
    pub animal: Option<String>,
    pub element: Option<String>,
    pub lucky: bool,
    /// Only at character creation (`creation = true`).
    pub creation: bool,
    /// Any one of these, if set (`either`): at least two.
    pub either: Vec<Need>,
}

impl Needs {
    /// Whether nothing is asked.
    pub fn is_empty(&self) -> bool {
        *self == Needs::default()
    }

    /// Every prerequisite, in the order they are shown: talents, stats, skills, weapon skills,
    /// the choice, the level, the race, the eyes, the class, the sign, then any one of the
    /// alternatives.
    pub fn iter(&self) -> impl Iterator<Item = Need> + '_ {
        let talents = self.talents.iter().map(|&t| Need::Talent(t));
        let stats = (self.stats.iter())
            .filter(|&(_, score)| score > 0)
            .map(|(stat, score)| Need::Stat(stat, score));
        let skills = self.skills.iter().map(|&(s, v)| Need::Skill(s, v));
        let weapons = (self.weapon_skills.iter()).map(|&(w, r)| Need::WeaponSkill(w, r));
        let any = self.any_weapon_skill.clone().map(Need::AnyWeaponSkill);
        let chosen = self.chosen.map(Need::Chosen);
        let level = (self.level > 0).then_some(Need::Level(self.level));
        let race = self.race.clone().map(Need::Race);
        let eyes = (!self.eyes.is_empty()).then(|| Need::Eyes(self.eyes.clone()));
        let class = self.class.clone().map(Need::Class);
        let animal = self.animal.clone().map(Need::Animal);
        let element = self.element.clone().map(Need::Element);
        let lucky = self.lucky.then_some(Need::Lucky);
        let creation = self.creation.then_some(Need::Creation);
        let either = (!self.either.is_empty()).then(|| Need::Either(self.either.clone()));
        talents
            .chain(stats)
            .chain(skills)
            .chain(weapons)
            .chain(any)
            .chain(chosen)
            .chain(level)
            .chain(race)
            .chain(eyes)
            .chain(class)
            .chain(animal)
            .chain(element)
            .chain(lucky)
            .chain(creation)
            .chain(either)
    }
}

/// A talent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TalentDef {
    pub id: String,
    pub name: String,
    pub tree: TreeId,
    /// What it is chosen for, if it takes a choice.
    pub choice: Option<Choice>,
    /// What it does, in the game's words (the wiki has the rule).
    pub description: String,
    /// What it asks.
    pub needs: Needs,
    /// What it adds.
    pub effects: Vec<TalentEffect>,
    /// What it adds only while the holder suffers a status (`while = "raging"`:
    /// Battle-scarred's rage half), apart from the plain effects: only the one sum reads them.
    pub effects_while: Vec<(StatusId, TalentEffect)>,
    /// The systems it touches (the coverage list).
    pub systems: Vec<SystemId>,
    /// Given, never picked (a race's, a birth sign's): off every pick list.
    pub granted: bool,
    /// A birth sign's flaw: granted, shown with "but" and dimmed, never a prerequisite.
    pub flaw: bool,
    /// What using its class active costs of an alternative resource (spec §8 "Resources"):
    /// `resource = { id = "focus", cost = 6 }` (0 asks only that it is held), if anything.
    pub resource: Option<ResourceCost>,
    /// The alignment bands its class active works for (spec §8 "Alignment"; `alignment = { min
    /// = "kind" }` in data: the paladin's holy gifts), or its band-gated passive (Communion), if
    /// any. It bars nothing: the ability is had, listed, and used whatever the band, and outside
    /// these bands it does nothing (`World::answered`, `World::communes`).
    pub alignment: Option<Requirement>,
}

/// An ability's cost in an alternative resource (spec §8 "Resources"): unavailable while the
/// user holds less, and spent as it is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceCost {
    pub resource: ResourceId,
    /// 0 asks only that it is held.
    pub amount: u16,
}

impl TalentDef {
    /// Whether anyone may take it: no prerequisites at all.
    pub fn is_free(&self) -> bool {
        !self.granted && self.needs.is_empty()
    }

    /// Its name with the choice it was taken for: "Weapon focus (long blades)".
    pub fn name_for(&self, content: &Content, choice: Option<&str>) -> String {
        match (self.choice, choice) {
            (Some(kind), Some(id)) => {
                let name = kind.name(content, id).unwrap_or(id);
                format!("{} ({})", self.name, name.to_lowercase())
            }
            _ => self.name.clone(),
        }
    }
}

/// Every talent, the trees, the systems, and the settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TalentBook {
    pub trees: Vec<TreeDef>,
    pub systems: Vec<SystemDef>,
    list: Vec<TalentDef>,
    ids: BTreeMap<String, TalentId>,
}

impl TalentBook {
    pub fn get(&self, id: TalentId) -> &TalentDef {
        &self.list[usize::from(id.0)]
    }

    /// The talent with data id `name`.
    pub fn id(&self, name: &str) -> Option<TalentId> {
        self.ids.get(name).copied()
    }

    /// Every talent with its id, in file order.
    pub fn iter(&self) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        (0u16..).zip(&self.list).map(|(i, t)| (TalentId(i), t))
    }

    /// Every talent of `tree`, in file order.
    pub fn of_tree(&self, tree: TreeId) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        self.iter().filter(move |(_, t)| t.tree == tree)
    }

    /// Whether the terrain the effects name (`terrain_as`) is in `content`, once its terrain has
    /// loaded (after the talents; content without terrain has nothing to check them by).
    pub(super) fn check_terrain(&self, content: &Content) -> Result<(), ContentError> {
        if content.terrain_defs().is_empty() {
            return Ok(());
        }
        for (_, def) in self.iter() {
            for effect in &def.effects {
                let TalentEffect::TerrainAs { terrains, like } = effect else {
                    continue;
                };
                if let Some(id) =
                    (terrains.iter().chain([like])).find(|id| content.terrain_id(id).is_none())
                {
                    return Err(ContentError::new(
                        &format!("data/talents/{}.toml", def.id),
                        format!("talent `{}` names an unknown terrain `{id}`", def.id),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Every talent with its id, tree by tree in display order, each tree in file order: the
    /// order pick lists and the Talents page use.
    pub fn by_tree(&self) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        (0u8..)
            .zip(&self.trees)
            .flat_map(|(i, _)| self.of_tree(TreeId(i)))
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn tree(&self, id: TreeId) -> &TreeDef {
        &self.trees[usize::from(id.0)]
    }

    /// Whether it has a class tree: whether the classes' level tables give anything.
    pub fn has_class_tree(&self) -> bool {
        self.trees.iter().any(|t| t.class)
    }

    pub fn tree_id(&self, name: &str) -> Option<TreeId> {
        (self.trees.iter().position(|t| t.id == name)).map(|i| TreeId(i as u8))
    }

    pub fn system(&self, id: SystemId) -> &SystemDef {
        &self.systems[usize::from(id.0)]
    }

    pub fn system_id(&self, name: &str) -> Option<SystemId> {
        (self.systems.iter().position(|s| s.id == name)).map(|i| SystemId(i as u8))
    }

    /// The talents that name `system`, in file order.
    pub fn of_system(&self, system: SystemId) -> impl Iterator<Item = (TalentId, &TalentDef)> {
        self.iter()
            .filter(move |(_, t)| t.systems.contains(&system))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentFile {
    #[serde(default)]
    tree: Vec<TreeSpec>,
    #[serde(default)]
    system: Vec<SystemSpec>,
    #[serde(default)]
    talent: Vec<TalentSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TreeSpec {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    class: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SystemSpec {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentSpec {
    id: String,
    name: String,
    tree: String,
    choice: Option<Choice>,
    description: String,
    #[serde(default)]
    needs: NeedsSpec,
    #[serde(default)]
    effects: Vec<EffectSpec>,
    #[serde(default)]
    systems: Vec<String>,
    #[serde(default)]
    granted: bool,
    #[serde(default)]
    flaw: bool,
    resource: Option<CostSpec>,
    alignment: Option<Requirement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CostSpec {
    id: String,
    #[serde(default)]
    cost: u16,
}

/// One effect as written: its kind and fields, and the status it counts only under, if any
/// (`while = "raging"`; the effect's own fields are checked by [`TalentEffect`]).
#[derive(Deserialize)]
struct EffectSpec {
    #[serde(rename = "while", default)]
    under: Option<String>,
    #[serde(flatten)]
    effect: TalentEffect,
}

/// `effects` split into the plain ones and those under a status, each checked; a `while` on
/// content without statuses has nothing to check it by, and its effect does nothing.
fn split_effects(
    specs: Vec<EffectSpec>,
    content: &Content,
    choice: Option<Choice>,
) -> Result<(Vec<TalentEffect>, Vec<(StatusId, TalentEffect)>), String> {
    let (mut plain, mut under) = (Vec::new(), Vec::new());
    for spec in specs {
        spec.effect.check(content, choice)?;
        let Some(name) = spec.under else {
            plain.push(spec.effect);
            continue;
        };
        if !spec.effect.may_be_conditional() {
            return Err(format!(
                "`while = \"{name}\"` is only for an effect the talents' sum reads, not {:?}",
                spec.effect
            ));
        }
        match content.status_id(&name) {
            Some(status) => under.push((status, spec.effect)),
            None if content.status_ids().next().is_none() => {}
            None => return Err(format!("`while` names an unknown status `{name}`")),
        }
    }
    Ok((plain, under))
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
struct NeedsSpec {
    talents: Vec<String>,
    stats: BTreeMap<String, u8>,
    skills: BTreeMap<String, u16>,
    weapon_skills: BTreeMap<String, u8>,
    any_weapon_skill: Option<AnySpec>,
    chosen: Option<u8>,
    level: u8,
    race: Option<String>,
    /// One vision profile or several, any one of which will do.
    eyes: Option<OneOrMany>,
    class: Option<String>,
    animal: Option<String>,
    element: Option<String>,
    lucky: bool,
    /// Only at character creation: never a level's pick.
    creation: bool,
    /// Alternatives: any one of what it asks will do (`either = { stats = { intelligence = 12
    /// }, skills = { appraise = 20 } }`).
    either: Option<Box<NeedsSpec>>,
}

/// `any_weapon_skill = { roles = ["melee"], of = ["unarmed"], rank = 3 }`: any skill of those
/// roles, or any named, at that rank.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnySpec {
    #[serde(default)]
    roles: Vec<super::Role>,
    #[serde(default)]
    of: Vec<String>,
    rank: u8,
}

/// Resolves `any_weapon_skill` against `content`'s weapon skills; `None` without them.
fn resolve_any(spec: AnySpec, content: &Content) -> Result<Option<AnyWeaponSkill>, String> {
    let table = content.weapon_skills();
    if table.skills().is_empty() {
        return Ok(None);
    }
    let mut named = Vec::new();
    for name in &spec.of {
        let id =
            (table.id(name)).ok_or_else(|| format!("needs an unknown weapon skill `{name}`"))?;
        named.push(id);
    }
    let skills: Vec<WeaponSkillId> = (table.ids())
        .filter(|(id, def)| spec.roles.contains(&def.role) || named.contains(id))
        .map(|(id, _)| id)
        .collect();
    let mut parts: Vec<String> = (named.iter())
        .map(|&id| table.skill(id).name.clone())
        .collect();
    if !spec.roles.is_empty() {
        let roles: Vec<String> = (spec.roles.iter())
            .map(|r| match r {
                super::Role::Launcher => "missile".to_string(),
                r => r.id().replace('_', " "),
            })
            .collect();
        parts.insert(0, format!("a {} weapon skill", roles.join(" or ")));
    }
    Ok(Some(AnyWeaponSkill {
        skills,
        rank: spec.rank,
        words: parts.join(" or "),
    }))
}

/// A talent as read, before the talents it needs are known.
struct Pending<'a> {
    file: &'a str,
    spec: TalentSpec,
}

/// Resolves `spec`'s prerequisites against `content` and the talent `ids`.
fn resolve_needs(
    spec: NeedsSpec,
    ids: &BTreeMap<String, TalentId>,
    content: &Content,
) -> Result<Needs, String> {
    let mut talents = Vec::new();
    for name in spec.talents {
        let id = *ids
            .get(&name)
            .ok_or_else(|| format!("needs an unknown talent `{name}`"))?;
        talents.push(id);
    }
    let mut stats = PerStat::splat(0u8);
    for (name, score) in spec.stats {
        let stat = Stat::from_id(&name).ok_or_else(|| format!("needs an unknown stat `{name}`"))?;
        stats[stat] = score;
    }
    // Content without skills (no `data/skills.toml`) has nothing to measure a skill need by:
    // it is left out, as skill affixes are; weapon skills likewise.
    let mut skills = Vec::new();
    for (name, value) in spec.skills {
        if content.skills().is_empty() {
            break;
        }
        let id =
            (content.skill_id(&name)).ok_or_else(|| format!("needs an unknown skill `{name}`"))?;
        skills.push((id, value));
    }
    let mut weapon_skills = Vec::new();
    for (name, rank) in spec.weapon_skills {
        if content.weapon_skills().skills().is_empty() {
            break;
        }
        let id = (content.weapon_skills().id(&name))
            .ok_or_else(|| format!("needs an unknown weapon skill `{name}`"))?;
        weapon_skills.push((id, rank));
    }
    let any_weapon_skill = match spec.any_weapon_skill {
        Some(any) => resolve_any(any, content)?,
        None => None,
    };
    // A race, class, or eyes is checked against its table when the content has one; without
    // (a fixture), the need stays and is simply never met.
    if let Some(race) = &spec.race
        && !content.races().list.is_empty()
        && content.race(race).is_none()
    {
        return Err(format!("needs an unknown race `{race}`"));
    }
    if let Some(class) = &spec.class
        && !content.classes().is_empty()
        && content.class(class).is_none()
    {
        return Err(format!("needs an unknown class `{class}`"));
    }
    let eyes = spec.eyes.as_ref().map_or(&[][..], OneOrMany::ids).to_vec();
    if let Some(unknown) = eyes.iter().find(|e| content.vision(e).is_none()) {
        return Err(format!("needs unknown eyes `{unknown}` (a vision profile)"));
    }
    let either = match spec.either {
        Some(alternatives) => {
            // Content without skills (a fixture) leaves a skill alternative out, as above.
            let alternatives = resolve_needs(*alternatives, ids, content)?;
            alternatives.iter().collect()
        }
        None => Vec::new(),
    };
    Ok(Needs {
        talents,
        stats,
        skills,
        weapon_skills,
        any_weapon_skill,
        chosen: spec.chosen,
        level: spec.level,
        race: spec.race,
        eyes,
        class: spec.class,
        animal: spec.animal,
        element: spec.element,
        lucky: spec.lucky,
        creation: spec.creation,
        either,
    })
}

/// Every `heirloom` effect names weapons (objects load after the talents): the first that
/// names something else, in words.
pub(super) fn check_heirlooms(content: &Content) -> Result<(), String> {
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            let TalentEffect::Heirloom {
                weapons, fallback, ..
            } = effect
            else {
                continue;
            };
            for object in weapons.values().chain([fallback]) {
                let weapon = (content.object_id(object)).and_then(|o| content.object(o).kind);
                if !matches!(weapon, Some(ItemKind::Weapon | ItemKind::Launcher)) {
                    return Err(format!(
                        "talent `{}`: `heirloom`: `{object}` is not a weapon",
                        def.id
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Parses the talent table: the settings (`[[tree]]`, `[[system]]`) and the talents, resolved against `content`, which must have
/// its skills, weapon skills, races, classes, vision profiles, and damage types already.
pub(super) fn parse<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<TalentBook, ContentError> {
    let mut first_file = None;
    let mut trees: Vec<(&str, TreeSpec)> = Vec::new();
    let mut systems: Vec<(&str, SystemSpec)> = Vec::new();
    let mut pending: Vec<Pending<'a>> = Vec::new();
    for (file, text) in files {
        first_file.get_or_insert(file);
        let parsed: TalentFile =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        trees.extend(parsed.tree.into_iter().map(|t| (file, t)));
        systems.extend(parsed.system.into_iter().map(|s| (file, s)));
        pending.extend(parsed.talent.into_iter().map(|spec| Pending { file, spec }));
    }
    let first_file = first_file.unwrap_or("data/talents.toml");
    let mut book = TalentBook {
        trees: Vec::new(),
        systems: Vec::new(),
        list: Vec::new(),
        ids: BTreeMap::new(),
    };
    let mut tree_files: BTreeMap<String, &str> = BTreeMap::new();
    for (file, t) in trees {
        if let Some(first) = tree_files.insert(t.id.clone(), file) {
            return Err(duplicate_id("tree", &t.id, file, first));
        }
        book.trees.push(TreeDef {
            id: t.id,
            name: t.name,
            description: t.description,
            class: t.class,
        });
    }
    let mut system_files: BTreeMap<String, &str> = BTreeMap::new();
    for (file, s) in systems {
        if let Some(first) = system_files.insert(s.id.clone(), file) {
            return Err(duplicate_id("system", &s.id, file, first));
        }
        book.systems.push(SystemDef {
            id: s.id,
            name: s.name,
        });
    }
    if book.trees.len() > usize::from(u8::MAX) || book.systems.len() > usize::from(u8::MAX) {
        return Err(ContentError::new(
            first_file,
            "too many trees or systems (at most 255)",
        ));
    }
    if pending.len() > usize::from(u16::MAX) {
        return Err(ContentError::new(first_file, "too many talents"));
    }
    // Every id first, so a talent may need one defined after it.
    let mut files: BTreeMap<String, &str> = BTreeMap::new();
    for (i, p) in pending.iter().enumerate() {
        let spec = &p.spec;
        if spec.id.is_empty() || spec.id.contains(':') || spec.id.contains(char::is_whitespace) {
            return Err(ContentError::new(
                p.file,
                format!("`{}` is no talent id", spec.id),
            ));
        }
        if let Some(first) = files.insert(spec.id.clone(), p.file) {
            return Err(duplicate_id("talent", &spec.id, p.file, first));
        }
        book.ids.insert(spec.id.clone(), TalentId(i as u16));
    }
    for p in pending {
        let spec = p.spec;
        let err = |m: String| ContentError::new(p.file, format!("talent `{}`: {m}", spec.id));
        let tree = (book.tree_id(&spec.tree))
            .ok_or_else(|| err(format!("unknown tree `{}`", spec.tree)))?;
        let mut systems = Vec::new();
        for name in &spec.systems {
            let id = (book.system_id(name)).ok_or_else(|| {
                err(format!(
                    "names a system `{name}` that is not in the list (data/talents.toml)"
                ))
            })?;
            systems.push(id);
        }
        let needs = resolve_needs(spec.needs, &book.ids, content).map_err(err)?;
        let (effects, effects_while) =
            split_effects(spec.effects, content, spec.choice).map_err(err)?;
        let resource = match &spec.resource {
            None => None,
            Some(cost) => {
                let resource = content.resource_id(&cost.id).ok_or_else(|| {
                    err(format!(
                        "costs a resource `{}` that is not in data/resources.toml",
                        cost.id
                    ))
                })?;
                Some(ResourceCost {
                    resource,
                    amount: cost.cost,
                })
            }
        };
        book.list.push(TalentDef {
            id: spec.id,
            name: spec.name,
            tree,
            choice: spec.choice,
            description: spec.description,
            needs,
            effects,
            effects_while,
            systems,
            granted: spec.granted,
            flaw: spec.flaw,
            resource,
            alignment: spec.alignment,
        });
    }
    Ok(book)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    /// The built-in content with the talent table replaced by `settings` and `talents`.
    fn load(settings: &str, talents: &[(&str, &str)]) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.remove("data/talents.toml");
        files.remove("data/talents/");
        // Background stories name talents of the built-in set.
        files.remove("data/background.toml");
        files.insert("data/talents.toml", settings);
        for (name, text) in talents {
            files.insert(&format!("data/talents/{name}.toml"), text);
        }
        Content::load(&files)
    }

    const SETTINGS: &str = r#"
[[tree]]
id = "vigour"
name = "Vigour"
description = "Hit points."
[[tree]]
id = "shadow"
name = "Shadow"
description = "Stealth."
[[system]]
id = "hit_points"
name = "Hit points"
[[system]]
id = "stealth"
name = "Stealth"
"#;

    const HEALTHY: &str = r#"
[[talent]]
id = "healthy"
name = "Healthy"
tree = "vigour"
description = "Faster healing."
effects = [{ kind = "healing_percent", value = 25 }]
systems = ["hit_points"]
"#;

    const HARDY: &str = r#"
[[talent]]
id = "hardy"
name = "Hardy"
tree = "vigour"
description = "More hit points."
needs = { stats = { toughness = 17 }, talents = ["healthy"] }
effects = [{ kind = "max_hp_per_levels", per_levels = 4, at_least = 3 }]
systems = ["hit_points"]
"#;

    fn err(result: Result<Content, ContentError>) -> String {
        match result {
            Ok(_) => panic!("loaded"),
            Err(e) => e.to_string(),
        }
    }

    #[test]
    fn talents_load_with_needs_and_effects() {
        let content = load(SETTINGS, &[("healthy", HEALTHY), ("hardy", HARDY)]).expect("loads");
        let book = content.talents();
        assert_eq!(book.len(), 2);
        let hardy = book.get(book.id("hardy").expect("hardy"));
        assert_eq!(hardy.tree, book.tree_id("vigour").expect("tree"));
        let needs: Vec<String> = hardy.needs.iter().map(|n| n.words(&content)).collect();
        assert_eq!(
            needs,
            ["Healthy", "Toughness 17"],
            "talents first, then scores"
        );
        assert_eq!(hardy.effects.len(), 1);
        assert!(!hardy.is_free());
        assert!(book.get(book.id("healthy").expect("healthy")).is_free());
        assert_eq!(book.by_tree().count(), 2);
    }

    #[test]
    fn a_duplicate_id_names_both_files() {
        let e = err(load(
            SETTINGS,
            &[("healthy", HEALTHY), ("hardy", HARDY), ("twin", HEALTHY)],
        ));
        assert!(e.contains("duplicate talent id `healthy`"), "{e}");
        assert!(e.contains("data/talents/healthy.toml"), "{e}");
    }

    #[test]
    fn an_effect_under_a_status_is_kept_apart_and_checked() {
        let with = |effects: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                 systems = [\"hit_points\"]\neffects = [{effects}]\n"
            );
            load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            )
        };
        let content = with(
            "{ kind = \"dodge\", value = 1 }, { kind = \"dodge\", value = 2, while = \"raging\" }",
        )
        .expect("loads");
        let book = content.talents();
        let x = book.get(book.id("x").expect("x"));
        let raging = content.status_id("raging").expect("raging");
        assert_eq!(x.effects, vec![TalentEffect::Dodge { value: 1 }]);
        assert_eq!(
            x.effects_while,
            vec![(raging, TalentEffect::Dodge { value: 2 })]
        );
        // Alone, it is something to do.
        with("{ kind = \"dodge\", value = 2, while = \"raging\" }").expect("loads");
        assert!(
            err(with("{ kind = \"dodge\", value = 2, while = \"sulking\" }"))
                .contains("unknown status `sulking`")
        );
        assert!(
            err(with("{ kind = \"dodge\", value = 2, whilst = \"raging\" }"))
                .contains("unknown field")
        );
        // An active is used, not summed: no condition on it.
        assert!(
            err(with(
                "{ kind = \"rage\", resource = \"rage\", while = \"raging\" }"
            ))
            .contains("only for an effect the talents' sum reads")
        );
    }

    #[test]
    fn a_song_names_a_song_and_what_it_costs() {
        let song = |fields: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\n\
                 description = \"d\"\nsystems = [\"hit_points\"]\n\
                 effects = [{{ kind = \"song\", status = \"inspired\", {fields} }}]\n"
            );
            load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            )
        };
        let inspire = "id = \"inspire\", on = \"self\"";
        song(&format!("{inspire}, resource = \"inspiration\", cost = 3")).expect("a pool");
        song(&format!("{inspire}, mana = 6")).expect("mana");
        let refused = |fields: &str| err(song(fields));
        assert!(refused("id = \"rage\", on = \"self\", mana = 6").contains("no song"));
        assert!(
            refused(&format!("{inspire}, resource = \"nerve\", cost = 3"))
                .contains("unknown resource `nerve`")
        );
        song(
            "id = \"inspire\", on = \"hearers\", mana = 6, check = \"willpower\", difficulty = 13",
        )
        .expect("data-ready for the Lullaby");
    }

    #[test]
    fn bad_prerequisites_and_effects_are_refused() {
        let bad = |body: &str| {
            let text = format!(
                "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                 systems = [\"hit_points\"]\n{body}\n"
            );
            err(load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", &text)],
            ))
        };
        // Alignment bars nothing: it is no prerequisite.
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\n\
                 needs = { requires_alignment = { min = \"good\" } }")
            .contains("unknown field")
        );
        // The spell hooks (the spell scaling design §7.1) filter by known tags.
        assert!(
            bad("effects = [{ kind = \"spell_mod\", rungs = 1, when = [\"sneezing\"] }]")
                .contains("no school, kind, shape, or damage type")
        );
        assert!(
            bad("effects = [{ kind = \"spell_mod\", rungs = 1, lift = 10 }]")
                .contains("unknown field")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { race = \"orc\" }")
                .contains("unknown race")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { eyes = \"xray\" }")
                .contains("unknown eyes")
        );
        assert!(
            bad("effects = [{ kind = \"weapon_ranks\", to = 1, skills = [\"x\"] }]")
                .contains("unknown weapon skill `x`")
        );
        assert!(
            bad("effects = [{ kind = \"skill\", value = 25 }]").contains("`choice = \"skill\"`")
        );
        // The Shadow and Wayfarer effects name what exists (#239).
        assert!(
            bad("effects = [{ kind = \"terrain_as\", terrains = [\"lava\", \"bog\"], like = \"floor\" }]")
                .contains("unknown terrain `bog`")
        );
        assert!(
            bad("effects = [{ kind = \"lantern_light\", light = \"sun\" }]")
                .contains("unknown light preset")
        );
        // The element gifts' (#268): Blaze names what exists.
        let blaze = |fields: &str| bad(&format!("effects = [{{ kind = \"blaze\", {fields} }}]"));
        let ok = "damage = \"2d6\", damage_type = \"fire\", spell = \"blaze\", \
                  fire = \"arrow_fire\", burns = 60, cooldown = \"burnt_out\"";
        let fields = ok.replace("\"arrow_fire\"", "\"sun\"");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown light preset"));
        let fields = ok.replace("\"burnt_out\"", "\"smouldering\"");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown status"));
        let fields = ok.replace("\"fire\",", "\"plasma\",");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown damage type"));
        assert!(
            bad("effects = [{ kind = \"scent\", status = \"sniffles\", radius = 4 }]")
                .contains("unknown status")
        );
        assert!(bad("effects = [{ kind = \"kick_die\", sides = 0 }]").contains("at least 1"));
        let no_system = "[[talent]]\nid = \"x\"\nname = \"X\"\ntree = \"vigour\"\ndescription = \"d\"\n\
                         effects = [{ kind = \"dodge\", value = 1 }]\nsystems = [\"nope\"]\n";
        assert!(
            err(load(
                SETTINGS,
                &[("healthy", HEALTHY), ("hardy", HARDY), ("x", no_system)]
            ))
            .contains("not in the list")
        );
    }

    #[test]
    fn choices_list_their_options() {
        let focus = r#"
[[talent]]
id = "weapon_focus"
name = "Weapon focus"
tree = "vigour"
choice = "weapon_skill"
description = "d"
needs = { chosen = 2 }
effects = [{ kind = "melee_damage", value = 1 }]
systems = ["hit_points"]
"#;
        let content = load(SETTINGS, &[("healthy", HEALTHY), ("focus", focus)]).expect("loads");
        let options = Choice::WeaponSkill.options(&content);
        assert!(options.iter().any(|o| o == "long_blades"), "{options:?}");
        assert!(
            !options.iter().any(|o| o == "duelist"),
            "styles are no weapon skills"
        );
        assert!(
            Choice::Style
                .options(&content)
                .iter()
                .any(|o| o == "duelist")
        );
        assert_eq!(Choice::Element.name(&content, "fire"), Some("fire"));
        assert_eq!(Choice::Skill.name(&content, "stealth"), Some("Stealth"));
        assert!(
            Choice::Skill
                .options(&content)
                .iter()
                .any(|o| o == "appraise")
        );
        assert_eq!(Choice::Element.name(&content, "slashing"), None);
    }

    #[test]
    fn the_built_in_talents_load() {
        let content = Content::builtin().expect("content");
        assert!(
            content.talents().len() >= 12,
            "{} talents",
            content.talents().len()
        );
    }
}
