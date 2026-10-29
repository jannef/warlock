//! Talents (spec §8 "Talents"; `assets/data/talents.toml` for the settings, one file per talent
//! under `assets/data/talents/`): what a character may pick as they level up. A talent has a
//! tree, up to [`TalentBook::max_ranks`] ranks, prerequisites ([`Needs`]) that must all hold when
//! it is chosen, and typed effects ([`TalentEffect`]) summed by [`crate::talent::TalentMods`] and
//! read where the rules already read what magic items do. Every talent names the systems it
//! touches from the closed list in the settings file; the **coverage check** at load fails when
//! a system that is not `pending` has fewer talents than `[coverage]` asks, or a tree that is
//! not `pending` (nor `no_free`) has no talent free of prerequisites, so no system is left
//! behind as the list grows. Without the files there are no talents, and no level asks for one.

use super::affix::Once;
use super::{
    Build, CombatStance, Content, ContentError, ItemKind, MAX_ITEM_LEVEL, MAX_LEVEL, MAX_RANK,
    MAX_RESOURCE, ResourceId, Role, WeaponSkillId, duplicate_id,
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

/// A system of the coverage list, an index into [`TalentBook::systems`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(pub u8);

/// The most ranks a talent may have unless the settings say otherwise.
pub const DEFAULT_MAX_RANKS: u8 = 3;
/// What a stat prerequisite may ask for.
const SCORES: std::ops::RangeInclusive<u8> = 1..=40;
/// What a flat bonus (Dodge, natural armour, damage, hearing, noticing) may be.
const BONUSES: std::ops::RangeInclusive<i32> = -20..=20;
/// What a percent effect may be.
const PERCENTS: std::ops::RangeInclusive<i32> = -100..=100;

/// A tree of talents (`[[tree]]` in the settings file), in display order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeDef {
    pub id: String,
    pub name: String,
    /// One line: what the tree is about.
    pub description: String,
    /// Its talents are still to come: the coverage check does not ask it for a free talent yet.
    pub pending: bool,
    /// It never needs a free talent: nothing picks from it alone (the birth signs' tree).
    pub no_free: bool,
    /// Its talents are class abilities (spec §8 "Class abilities"): given by the class tables
    /// (`data/classes/`), never picked, and counted by the coverage check as class abilities,
    /// not talents.
    pub class: bool,
}

/// A system of the game that talents may touch (`[[system]]` in the settings file): the
/// coverage check asks each one that is not `pending` for its talents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDef {
    pub id: String,
    pub name: String,
    /// Its talents are still to come: counted and shown, not yet required.
    pub pending: bool,
    /// The fewest talents it may have, when not the bar's (`[coverage]`): a young system that
    /// ships with one (`creature_gear`, Scavenger).
    pub min_talents: Option<u32>,
}

/// The bar the coverage check holds every system and tree to (`[coverage]`); without the table,
/// nothing is asked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    /// The fewest talents a system may have.
    pub min_talents: u32,
    /// The fewest class abilities a system may have (0 until every class has its table).
    pub min_class: u32,
    /// The fewest talents without prerequisites a tree may have.
    pub free_per_tree: u32,
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
            Choice::Skill => "a skill",
            Choice::Element => "an element",
            Choice::Tag => "a kind of creature",
            Choice::School => "a school of magic or an element",
        }
    }

    /// Whether a weapon skill of `role` may be chosen as a weapon skill: one that attacks
    /// (shield blocking only defends, and a style is chosen as a style).
    fn attacks(role: Role) -> bool {
        !matches!(role, Role::Style | Role::ShieldBlock)
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
    /// Natural armour as `natural`, only while it wears no body armour (Thick hide).
    NaturalUnarmoured { value: i32 },
    /// Damage added to every melee blow (the main hand's and the off hand's).
    MeleeDamage { value: i32 },
    /// Damage added to every attack it makes while below a third of its hit points (Ferocity).
    Ferocity { value: i32 },
    /// Every attack threatens a critical this many natural points lower.
    Threat { value: i32 },
    /// A natural 2 on an attack roll (a fumble) is rolled again, once.
    RerollFumble,
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
    /// Caster level, this much higher (negative: lower; spec §8 "Spells"), in
    /// [`crate::world::World::caster_level`]; with `per_levels`, `value` per that many character
    /// levels (Arcane training: +1 per 5).
    CasterLevel {
        value: i32,
        #[serde(default)]
        per_levels: u8,
    },
    /// The caster level a wand casts at, this much higher (spec §8 "Magic items": "Wands"; Wand
    /// mastery), in [`crate::world::World`]'s zap.
    WandLevel { value: i32 },
    /// The caster level its magic items give it counts this percent more, rounded up (Attuned:
    /// half again), in [`crate::world::Actor::caster_level`]'s items term; a negative items
    /// term (a curse) is left as it is.
    AttunedPercent { value: i32 },
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
    /// Its eyes see ultraviolet light.
    SeesUv,
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
    /// An initiative check it makes against a hostile attacker beside it is answered with a
    /// free blow at once (Hair trigger; `Cue::Startled`, `world/free_blow.rs`).
    StartledBlow,
    /// Creature types carrying one of `tags` have vitals for its attacks: they may be critically
    /// hit and sneak-attacked by it (Anatomist; `Content::has_vitals`). Never the incorporeal.
    VitalsIn { tags: Vec<String> },
    /// The carrying limit, this percent more.
    CarryPercent { value: i32 },
    /// Being Burdened costs it nothing, as a dwarf's burden (Porter).
    IgnoreBurdened,
    /// The slowdown its burden and being Bloated put on its steps, this percent more (negative:
    /// less; Sure-footed).
    StepSlowdownPercent { value: i32 },
    /// The carrying limit, this many kilograms more (on top of what magic items add).
    CarryKg { value: i32 },
    /// Coins it carries weigh nothing.
    WeightlessCoins,
    /// Journeys on the overworld take this percent more time (negative: quicker).
    TravelPercent { value: i32 },
    /// Wading (water, marsh) costs a plain step, and never gives a disease.
    Swimmer,
    /// A step onto any of `terrains` costs what a step onto `like` does (ids in
    /// `data/terrain.toml`; Pathfinder, Swimmer, Marsh walker).
    TerrainAs { terrains: Vec<String>, like: String },
    /// A plunge through ice into the water never hurts it (Swimmer).
    PlungeProof,
    /// Ice holds it as if it carried nothing (Winter-born).
    LightOnIce,
    /// Climbing out of the water onto ice costs this percent more (negative: less).
    ClimbPercent { value: i32 },
    /// Wading never gives it a disease (Marsh walker).
    WadingProof,
    /// Creatures of animal factions are neutral to it until it attacks one of that type
    /// (Beast-friend).
    BeastFriend,
    /// It knows the weather of the next `hours` hours under the open sky (Weather-wise).
    Forecast { hours: u8 },
    /// Hit points come back this percent faster under the open sun (Sun-blessed).
    SunHealing { value: i32 },
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
    /// A fresh corpse's chances to raise its stats, this percent more (Gourmand).
    CorpseStatsPercent { value: i32 },
    /// The daily stat check (06:00) multiplies the need by this, when it is less than the
    /// data's `daily_factor` (Hard training).
    DailyFactor { value: u16 },
    /// The training marks a potion or an elixir gives, this percent more (Quick study of the
    /// body).
    PotionMarksPercent { value: i32 },
    /// Experience gained, this percent more.
    XpPercent { value: i32 },
    /// Skill rolls every level, this many more (Quick learner).
    SkillRolls { value: i32 },
    /// One more talent pick at character level `level` (1: at creation; Versatile).
    ExtraPick { level: u8 },
    /// The chosen skill (the talent takes a `skill` choice) is trained at these dice, or grows
    /// by them if trained already: rolled once, from the `levels` stream, as the rank is taken
    /// (Apprentice).
    TrainSkill { dice: crate::dice::Dice },
    /// Loot it finds (what generated levels hold, what creatures drop) rolls magic and rare this
    /// percent more often: their rarity weights (Treasure hunter).
    LootMagicPercent { value: i32 },
    /// Reading a scroll takes this percent more time (negative: less; Scroll savant).
    ReadPercent { value: i32 },
    /// Studying a spell (spec §8 "Spells"): `bonus` points off the check's difficulty, and it
    /// takes `percent` more time (negative: less; Quick study).
    Study { bonus: i32, percent: i32 },
    /// A scroll it reads is not used up, by this chance in percent (one gameplay draw a read;
    /// Spell recall).
    ScrollKept { chance: u8 },
    /// A potion it drinks is not used up, by this chance in percent (one gameplay draw a drink;
    /// Thrifty drinker).
    PotionKept { chance: u8 },
    /// An unknown potion it picks up tells its kind (never its type) by its smell (Nose for
    /// draughts).
    PotionSmell,
    /// A healing potion it drinks heals this percent more (Cauldron-wise).
    PotionHeals { value: i32 },
    /// Its own bursts never catch it (Careful reader).
    SparesSelf,
    /// Added to its searching, not to noticing in passing (Keen searcher).
    Searching { value: i32 },
    /// The damage traps do it, this percent more (negative: less; Trap sense).
    TrapDamagePercent { value: i32 },
    /// The experience a quest gives, this percent more (before `xp_percent`).
    QuestXpPercent { value: i32 },
    /// The gold a quest gives, this percent more.
    QuestGoldPercent { value: i32 },
    /// Prices move this many percent more in its favour (spec §7 "Shops"), before the cap.
    FavourPercent { value: i32 },
    /// The cap on its favour, this many percent higher.
    FavourCap { value: i32 },
    /// Shops pay this percent more for what they buy from it, never more than they would ask.
    SellPercent { value: i32 },
    /// Keepers let it leave owing up to this much gold: the debt stays on their bill.
    Credit { gold: u32 },
    /// What it buys or sells in a shop is identified.
    IdentifyTrades,
    /// Temple services cost it half, and a cure also mends a point of each damaged stat
    /// (Pilgrim; `world/service.rs`).
    Pilgrim,
    /// What a creature it kills drops is identified as it lands (spec §8 "Creature gear").
    Scavenger,
    /// What an animated thing is made of falls whole this percent more often when it dies, whoever
    /// made the kill (Salvager; spec §8 "Animated objects").
    SalvagePercent { value: i32 },
    /// Look mode tells when anything an animated thing is made of is magic: "It hums with
    /// power." (Appraiser's eye; spec §8 "Animated objects").
    AppraiseAnimated,
    /// Item descriptions show what its things are worth.
    ItemWorth,
    /// Item descriptions show what each gem in a thing is worth.
    GemWorth,
    /// A magic item it has carried this many hours since it held the effect is identified.
    IdentifyCarried { hours: u32 },
    /// Loose gems it finds (on a level made after it held the effect, or dropped by a kill) are
    /// cut this many grades finer, at most the finest.
    GemCut { value: u8 },
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
    /// Senses the danger of a level (spec §7 "Danger levels"): from 1, how a level feels on
    /// arrival; from 2, a note when something comes to the level (Sixth sense, one per rank).
    DangerSense { value: i32 },
    /// The creatures that come to a level the actor is on come this percent less often (the
    /// time between attempts this percent longer; Quiet delver).
    SpawnEveryPercent { value: i32 },
    /// A special room's nature is known from a glimpse of it, and a whole level's effect is
    /// named on arrival (Sense of place; [`crate::perception::PerceptionView::room`]).
    SenseOfPlace,
    /// It tells a locked chest from an unlocked one at a glance (Chest-wise;
    /// [`crate::perception::PerceptionView::chest_at`]), and a shut chest's trap in its sight
    /// becomes known to it (`World::chest_wise_look`).
    ChestWise,
    /// A disguised mimic within this many cells is seen for what it is (Mimic-wise;
    /// [`crate::perception::Perception`]'s disguises).
    SeesMimics { value: i32 },
    /// A mimic's grip never holds it (Slick; `World::afflict`).
    Slick,
    /// The hint said on arriving on a level with a treasure room tells which way it lies
    /// (Treasure sense; `world/treasury.rs`).
    TreasureSense,
    /// At a forgotten altar of the actor's own side the side's lean counts double, and an
    /// opposed one leans as a neutral one (Devout; the prayer draw, `World::lean`).
    Devout,
    /// At a forgotten altar a great bane drawn becomes nothing (Penitent; the prayer draw,
    /// `world/pray.rs`).
    Penitent,
    /// Where a special room quickens healing or mana, the quickening is this percent more
    /// (Hallowed rest; the room's pace in `World::healing_rate` and `World::mana_rate`).
    RoomPacePercent { value: i32 },
    /// A special room's (or a whole level's) `status` never takes it (Anchored mind: Ebbing;
    /// `World::dwell`).
    RoomStatusImmune { status: String },
    /// A collapsed floor never gives way under it (Light tread; `World::collapse_step`).
    CollapseProof,
    /// A hall of mirrors' reflections within this many cells are seen for what they are, and not
    /// shown (Mirror-wise; `world/mirrors.rs`).
    SeesReflections { value: i32 },
    /// A tempting hoard seals only after this many stacks more are lifted inside it (Hoard-wise;
    /// `World::lift_in_hoard`, `world/hoard.rs`).
    HoardGrace { value: i32 },
    /// Sleepers need this percent more noise to wake from what it does (Hush-foot; the wake
    /// check in `World::spread_noise`, `world/den.rs`).
    WakePercent { value: i32 },
    /// What it kills never rises in a cursed room (Grave warden; `World::curse_corpse`).
    GraveWarden,
    /// Searching reaches this many cells farther.
    SearchReach { value: i32 },
    /// A skill counts this much higher wherever its value is read (an untrained one from 0).
    Skill { skill: String, value: i32 },
    /// Anyone not hostile makes way for it (spec §8 "Displacement"), those keeping a post too.
    MakeWay,
    /// No one makes way for it (Trusted by none), whatever else would have them.
    Distrusted,
    /// A free blow the actor is given (spec §8 "Free blows").
    FreeBlow { trigger: FreeBlowTrigger },
    /// What attacks with some weapon skills gain, as their ranks give it
    /// ([`super::Benefit`]): the weapon skill or style `skill` names, else the chosen one's (no
    /// `role`; the talent takes a choice), else every skill of `role`'s. `ranks`: its benefits read that many ranks higher
    /// (a rank held, never past [`MAX_RANK`]); `bash` only where a free bash is had already.
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
    /// Its missiles break this percent more often when they hit (negative: less often).
    MissileBreaks { value: i32 },
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
    /// Fleeing steps (spec §8 "Stances") cost this percent more (negative: cheaper).
    FleePercent { value: i32 },
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
    /// The class active Lay on hands: `per_level` hit points per character level back at once,
    /// then `degrees` of the `cooldown` status.
    LayOnHands {
        per_level: u8,
        cooldown: String,
        degrees: u16,
    },
    /// Lay on hands heals fully and cures every status that may be cured and every disease, and
    /// its cooldown is half as long (Avatar).
    Avatar,
    /// The class active Rage: the player is granted `resource` (Rage, spec §8 "Resources") and
    /// suffers its status (Raging) while it lasts; no rage while that or the status it `follows`
    /// into lasts.
    Rage { resource: String },
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
    /// What its magic items add to `resource`'s maximum counts this percent more, rounded up
    /// (Bellows: stamina's doubled), in [`crate::world::Actor::resource_max`]'s items term; a
    /// negative items term is left as it is.
    GearedResourcePercent { resource: String, value: i32 },
    /// While raging: the statuses named tick nothing, and a blow that would take the last hit
    /// point leaves it for `cost` of the rage, when that much is held (Unstoppable).
    Unstoppable { statuses: Vec<String>, cost: u16 },
    /// The class active Whirlwind: a blow at every hostile beside it for the price of one
    /// attack.
    Whirlwind,
    /// The class active Smite: a melee blow in a direction with one die of `sides` more damage
    /// per `per_levels` character levels (at least one), then `degrees` of the `cooldown`
    /// status.
    Smite {
        per_levels: u8,
        sides: u8,
        cooldown: String,
        degrees: u16,
    },
    /// The class active Holy radiance: a toggle; while it suffers `status` it sheds light
    /// `light` (an id in `data/lights.toml`).
    Radiance { status: String, light: String },
    /// The class active Quivering palm: an unarmed blow in a direction; a hit that wounds
    /// leaves the target `severity` degrees of `status` (what it costs is the talent's
    /// `resource`: Ki).
    QuiveringPalm { status: String, severity: u16 },
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
    /// Its door kicks are this percent louder (negative: quieter; Lockbreaker).
    KickNoisePercent { value: i32 },
    /// What a chest it kicks open holds breaks this percent more often (negative: less; Deft
    /// breaker; spec §8 "Chests").
    SmashDestroyPercent { value: i32 },
    /// Its digging does this percent more work a turn.
    DigPercent { value: i32 },
    /// Its blows with a two-handed weapon cost this percent more (negative: cheaper).
    TwoHandedPrice { value: i32 },
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
    /// Its Rend rolls the off hand's damage dice too (Second blade's rend).
    RendOffHand,
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
    /// The chance a spell it casts fails with a shield in each hand, this many percent points
    /// smaller (Loose straps; two-shield fighting, #611).
    ShieldCasting { value: i32 },
    /// In combat stance `stance` with a shield in each hand, both shields' Shield bonuses add
    /// up in Defence (Craven's wall; two-shield fighting).
    BothShields { stance: CombatStance },
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
    /// Pits never catch it (Light step's second rank).
    PitProof,
    /// It can disarm traps (the Disarm ability) without Disable device.
    Disarm,
    /// It moves and acts this percent quicker (its speed; the number is hidden in the game).
    SpeedPercent { value: i32 },
    /// Its steps cost this percent more (negative: cheaper), only while it wears no body armour
    /// (if `unarmoured`), is Unburdened (if `unburdened`), and holds no shield (if `no_shield`):
    /// Fast movement, Fleet.
    StepPercent {
        value: i32,
        #[serde(default)]
        unarmoured: bool,
        #[serde(default)]
        unburdened: bool,
        #[serde(default)]
        no_shield: bool,
    },
    /// Burdened costs it nothing, as a dwarf (Steadfast).
    Steadfast,
    /// Strained costs it what Burdened costs others (Stout).
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
    /// Its spells and spell scrolls roll `dice` dice more and add `per_die` to every die
    /// (Scroll adept, Empower, Archmage).
    SpellPower {
        #[serde(default)]
        per_die: i32,
        #[serde(default)]
        dice: u32,
    },
    /// Its spells cost this percent more mana (negative: less; Archmage, Empower), at least 1.
    SpellManaPercent { value: i32 },
    /// The mana a spell above its caster level's tier costs over its price, this percent more
    /// (negative: less; Overreach), on the tier table's `overreach_cost`
    /// ([`crate::content::SpellRules::mana_with`]).
    OverreachCostPercent { value: i32 },
    /// Its spells that carry the chosen tag (the talent takes a `school` choice: a school or an
    /// element) cost `cost` percent more mana (negative: less), reach `reach` cells farther,
    /// never past the spell's own caps (School focus; magic design §6, §10), and may have their
    /// cap lifted `lift` points more than their tier's `max_lift` (Keen mind; the combat
    /// progression design §7). Each defaults to 0.
    SchoolFocus {
        #[serde(default)]
        cost: i32,
        #[serde(default)]
        reach: i32,
        #[serde(default)]
        lift: i32,
    },
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
    /// Its diseases never progress: they still show until shaken off or cured (Timeless body).
    DiseaseStill,
    /// Its blows that hit a creature suffering `status` do `value` more damage (Poison lore).
    StatusDamage { status: String, value: i32 },
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
    /// Against a creature whose type carries `tag`, its attacks are this much likelier to hit and
    /// do this much more damage (Unbinder: `animated`).
    Bane {
        tag: String,
        to_hit: i32,
        damage: i32,
    },
    /// Against creature types that carry the tag it was taken for (a `choice = "tag"`), its
    /// attacks are this much likelier to hit and do this much more damage (Sworn foe).
    SwornFoe { to_hit: i32, damage: i32 },
    /// Arriving on a level that holds a living creature an active kill quest of its still asks
    /// for, it is told so (Quarry sense).
    QuarrySense,
    /// It places every sound it hears where it was made (Tracker).
    ExactSounds,
    /// Its footsteps under the open sky (no roof over the cell stepped to) are this percent
    /// louder (negative: quieter; Camouflage).
    SkyFootstepsPercent { value: i32 },
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
    /// The class active Volley: `shots` shots at a cell for the price of one (what it costs is
    /// the talent's `resource`, if anything).
    Volley { shots: u8 },
    /// The class active Inspire: it suffers `status` (Inspired), for `mana` mana; or, while it
    /// holds `cost` of `resource` (Inspiration), for those in place of the mana.
    Inspire {
        status: String,
        mana: i32,
        #[serde(default)]
        resource: Option<String>,
        #[serde(default)]
        cost: u16,
    },
    /// Inspire gives `status` instead, and costs no mana (Virtuoso).
    Virtuoso { status: String },
    /// The class active Consecrate: `time` (a duration) of prayer, for `mana` mana, lifts an
    /// unholy special room (or a whole level's unholy effect) for good (`world/consecrate.rs`).
    Consecrate { mana: i32, time: String },
    /// The class active Knock: for `mana` mana, the lock of the locked door or chest in a
    /// direction springs open (`world/knock.rs`; spec §8 "Chests").
    Knock { mana: i32 },
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
            TalentEffect::VeteransWind { .. } => A::VeteransWind,
            TalentEffect::Whirlwind => A::Whirlwind,
            TalentEffect::LayOnHands { .. } => A::LayOnHands,
            TalentEffect::Smite { .. } => A::Smite,
            TalentEffect::Radiance { .. } => A::HolyRadiance,
            TalentEffect::QuiveringPalm { .. } => A::QuiveringPalm,
            TalentEffect::Volley { .. } => A::Volley,
            TalentEffect::Lurk { .. } => A::Lurk,
            TalentEffect::LowBlow => A::LowBlow,
            TalentEffect::Finisher => A::MasterThief,
            TalentEffect::Inspire { .. } => A::Inspire,
            TalentEffect::Consecrate { .. } => A::Consecrate,
            TalentEffect::Knock { .. } => A::Knock,
            _ => return None,
        })
    }

    /// Whether the numbers are within what the rules expect, and the statuses and damage types
    /// it names are in `content` (content without statuses or damage types has nothing to check
    /// them by: they stay, and do nothing).
    pub(super) fn check(&self, content: &Content, choice: Option<Choice>) -> Result<(), String> {
        let status = |id: &str| {
            (content.status_ids().next().is_none() || content.status_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown status `{id}`"))
        };
        // A resource it names, by id; `None` for content without resources, which has nothing
        // to check it by (it stays, and does nothing).
        let resource = |id: &str| match content.resource_id(id) {
            Some(r) => Ok(Some(r)),
            None if content.resources().is_empty() => Ok(None),
            None => Err(format!("names an unknown resource `{id}`")),
        };
        // A cost of 1 to the maximum of the resource `id` names, for effect `kind`.
        let resource_cost = |kind: &str, id: &str, cost: u16| {
            if cost == 0 {
                return Err(format!("`{kind}`: cost is at least 1"));
            }
            match resource(id)?.map(|r| content.resource(r).max) {
                Some(max) if cost > max => Err(format!(
                    "`{kind}`: costs {cost} of `{id}`, more than its maximum {max}"
                )),
                _ => Ok(()),
            }
        };
        let bonus = |v: i32| {
            (v != 0 && BONUSES.contains(&v))
                .then_some(())
                .ok_or_else(|| {
                    format!("a bonus is {} to {}, not 0", BONUSES.start(), BONUSES.end())
                })
        };
        let percent = |v: i32| {
            (v != 0 && PERCENTS.contains(&v))
                .then_some(())
                .ok_or_else(|| {
                    format!(
                        "a percent is {} to {}, not 0",
                        PERCENTS.start(),
                        PERCENTS.end()
                    )
                })
        };
        match self {
            TalentEffect::Dodge { value }
            | TalentEffect::Natural { value }
            | TalentEffect::NaturalUnarmoured { value }
            | TalentEffect::Hardened { bonus: value }
            | TalentEffect::MeleeDamage { value }
            | TalentEffect::WandLevel { value }
            | TalentEffect::Hearing { value }
            | TalentEffect::Notice { value }
            | TalentEffect::InitiativeBonus { value }
            | TalentEffect::SurpriseDifficulty { value }
            | TalentEffect::SneakReach { value }
            | TalentEffect::DangerSense { value }
            | TalentEffect::Threat { value }
            | TalentEffect::Search { value }
            | TalentEffect::SearchReach { value }
            | TalentEffect::SeesReflections { value }
            | TalentEffect::SeesMimics { value }
            | TalentEffect::HoardGrace { value }
            | TalentEffect::ThrowRange { value }
            | TalentEffect::MaxDex { value }
            | TalentEffect::Deflection { value }
            | TalentEffect::Pain { value }
            | TalentEffect::DiseaseChecks { value }
            | TalentEffect::DrainChecks { value }
            | TalentEffect::DiseaseRecovery { value }
            | TalentEffect::DoorKick { value }
            | TalentEffect::BurdenDex { value }
            | TalentEffect::MissileDodge { value }
            | TalentEffect::DeflectionUnarmoured { value }
            | TalentEffect::SmallTarget { value }
            | TalentEffect::Ferocity { value }
            | TalentEffect::StoneFind { value }
            | TalentEffect::Searching { value }
            | TalentEffect::SkillRolls { value }
            | TalentEffect::Darkvision { value }
            | TalentEffect::NightSight { value }
            | TalentEffect::DaySight { value }
            | TalentEffect::CarryKg { value }
            | TalentEffect::Checks { value, .. }
            | TalentEffect::CriticalConfirm { value }
            | TalentEffect::AbilityRanks { value }
            | TalentEffect::CriticalPenetration { value }
            | TalentEffect::OffHandDamage { value }
            | TalentEffect::BestSkillThreat { value }
            | TalentEffect::TrapEase { value }
            | TalentEffect::TrainedSkills { value }
            | TalentEffect::Barbs { damage: value } => bonus(*value),
            TalentEffect::FavouredPrey { to_hit, damage }
            | TalentEffect::SwornFoe { to_hit, damage } => {
                if [to_hit, damage].iter().all(|&&v| v == 0)
                    || ![to_hit, damage].iter().all(|v| BONUSES.contains(v))
                {
                    return Err(
                        "`favoured_prey`, `sworn_foe`: to_hit and damage are -20 to 20, not both 0"
                            .into(),
                    );
                }
                Ok(())
            }
            TalentEffect::Bane {
                tag,
                to_hit,
                damage,
            } => {
                super::tag::check(tag).map_err(|m| format!("`bane`: {m}"))?;
                if [to_hit, damage].iter().all(|&&v| v == 0)
                    || ![to_hit, damage].iter().all(|v| BONUSES.contains(v))
                {
                    return Err("`bane`: to_hit and damage are -20 to 20, not both 0".into());
                }
                Ok(())
            }
            TalentEffect::VitalsIn { tags } => {
                if tags.is_empty() {
                    return Err("`vitals_in`: tags names at least one".into());
                }
                for tag in tags {
                    super::tag::check(tag).map_err(|m| format!("`vitals_in`: {m}"))?;
                }
                Ok(())
            }
            TalentEffect::AmbushThreat { value } => {
                if !(1..=5).contains(value) {
                    return Err("`ambush_threat`: value is 1 to 5".into());
                }
                Ok(())
            }
            TalentEffect::GrantsResource { resource: id } => resource(id).map(|_| ()),
            TalentEffect::RapidShot {
                resource: id,
                cost,
                saving,
            } => {
                if !(1..=50).contains(saving) {
                    return Err("`rapid_shot`: saving is 1 to 50".into());
                }
                match resource(id)?.map(|r| content.resource(r).max) {
                    Some(max) if *cost > max => Err(format!(
                        "`rapid_shot`: costs {cost} of `{id}`, more than its maximum {max}"
                    )),
                    _ => Ok(()),
                }
            }
            TalentEffect::Volley { shots } => {
                if !(2..=5).contains(shots) {
                    return Err("`volley`: shots is 2 to 5".into());
                }
                Ok(())
            }
            TalentEffect::Inspire {
                status: id,
                mana,
                resource: instead,
                cost,
            } => {
                if !(0..=100).contains(mana) {
                    return Err("`inspire`: mana is 0 to 100".into());
                }
                if let Some(instead) = instead {
                    resource_cost("inspire", instead, *cost)?;
                } else if *cost != 0 {
                    return Err("`inspire`: a `cost` needs its `resource`".into());
                }
                status(id)
            }
            TalentEffect::FlurryCost { resource: id, cost } => {
                resource_cost("flurry_cost", id, *cost)
            }
            TalentEffect::Virtuoso { status: id } => status(id),
            TalentEffect::Consecrate { mana, time } => {
                if !(0..=100).contains(mana) {
                    return Err("`consecrate`: mana is 0 to 100".into());
                }
                let time: crate::time::Duration = time.parse()?;
                if !(1..=3600).contains(&time.as_seconds()) {
                    return Err("`consecrate`: time is 1s to 1h".into());
                }
                Ok(())
            }
            TalentEffect::Knock { mana } => {
                if !(0..=100).contains(mana) {
                    return Err("`knock`: mana is 0 to 100".into());
                }
                Ok(())
            }
            TalentEffect::RoomStatusImmune { status: id } => status(id),
            TalentEffect::Stat { value, .. } => {
                if *value == 0 || !(-5..=5).contains(value) {
                    return Err("`stat`: value is -5 to 5, not 0".into());
                }
                Ok(())
            }
            TalentEffect::Skill { skill, value } => {
                if !content.skills().is_empty() && content.skill_id(skill).is_none() {
                    return Err(format!("names an unknown skill `{skill}`"));
                }
                percent(*value)
            }
            TalentEffect::StartingGold {
                percent,
                at_least,
                at_most,
            } => {
                if *percent > 1000 || at_least > at_most || *at_most > 10_000 {
                    return Err(
                        "`starting_gold`: percent up to 1000, at_least up to at_most, \
                                at_most up to 10000"
                            .into(),
                    );
                }
                Ok(())
            }
            TalentEffect::StartingGems { min, max, level } => {
                if *min > *max || *max > 10 || !(1..=MAX_ITEM_LEVEL).contains(level) {
                    return Err(format!(
                        "`starting_gems`: min up to max up to 10, level 1 to {MAX_ITEM_LEVEL}"
                    ));
                }
                Ok(())
            }
            TalentEffect::Heirloom {
                weapons,
                fallback,
                min,
                max,
                level,
            } => {
                if !(1..=*max).contains(min) || *max > 6 || !(1..=MAX_ITEM_LEVEL).contains(level) {
                    return Err(format!(
                        "`heirloom`: min 1 to max, max up to 6, level 1 to {MAX_ITEM_LEVEL}"
                    ));
                }
                for race in weapons.keys() {
                    if !content.races().list.is_empty() && content.race(race).is_none() {
                        return Err(format!("`heirloom`: names an unknown race `{race}`"));
                    }
                }
                // Its weapons are checked once the objects load (`check_heirlooms`).
                let _ = fallback;
                Ok(())
            }
            TalentEffect::Credit { gold } => {
                if !(1..=1000).contains(gold) {
                    return Err("`credit`: gold is 1 to 1000".into());
                }
                Ok(())
            }
            TalentEffect::IdentifyCarried { hours } => {
                if !(1..=240).contains(hours) {
                    return Err("`identify_carried`: hours is 1 to 240".into());
                }
                Ok(())
            }
            TalentEffect::TrainSkill { dice } => match choice {
                Some(Choice::Skill) if dice.min() >= 1 && dice.max() <= 100 => Ok(()),
                Some(Choice::Skill) => Err("`train_skill`: its dice roll 1 to 100".into()),
                _ => Err(
                    "`train_skill` is for the chosen skill: it needs `choice = \"skill\"`".into(),
                ),
            },
            TalentEffect::DailyFactor { value } => {
                let factors = super::stat_training::DAILY_FACTORS;
                if !factors.contains(value) {
                    return Err(format!(
                        "`daily_factor`: value is {} to {}",
                        factors.start(),
                        factors.end()
                    ));
                }
                Ok(())
            }
            TalentEffect::ScrollKept { chance } => {
                if !(1..=100).contains(chance) {
                    return Err("`scroll_kept`: chance is 1 to 100".into());
                }
                Ok(())
            }
            TalentEffect::PotionKept { chance } => {
                if !(1..=100).contains(chance) {
                    return Err("`potion_kept`: chance is 1 to 100".into());
                }
                Ok(())
            }
            TalentEffect::PotionHeals { value } => percent(*value),
            TalentEffect::GemCut { value } => {
                if !(1..=5).contains(value) {
                    return Err("`gem_cut`: value is 1 to 5".into());
                }
                Ok(())
            }
            TalentEffect::Protection {
                value,
                body_armour,
                shield,
                per_piece,
            } => {
                if *per_piece && (body_armour.is_some() || *shield) {
                    return Err("`protection`: per_piece asks nothing else".into());
                }
                bonus(*value)?;
                match body_armour {
                    Some(at) if !(1..=20).contains(at) => {
                        Err("`protection`: body_armour is 1 to 20".into())
                    }
                    _ => Ok(()),
                }
            }
            TalentEffect::StanceToHit { percent: value, .. }
            | TalentEffect::StanceDodge { percent: value, .. }
            | TalentEffect::WeaponMarks { value, .. }
            | TalentEffect::StepPercent { value, .. }
            | TalentEffect::FleePercent { value }
            | TalentEffect::SlipPercent { value } => percent(*value),
            TalentEffect::RaisePercent { status: id, value } => {
                status(id)?;
                percent(*value)
            }
            TalentEffect::TwoHandedCrit { value } => {
                if !(1..=3).contains(value) {
                    return Err("`two_handed_crit`: value is 1 to 3".into());
                }
                Ok(())
            }
            TalentEffect::UnarmedDie { sides } => {
                if !(2..=20).contains(sides) {
                    return Err("`unarmed_die`: sides is 2 to 20".into());
                }
                Ok(())
            }
            TalentEffect::VeteransWind {
                per_level,
                at_least,
                cooldown,
            } => {
                if !(1..=20).contains(per_level) || !(0..=100).contains(at_least) {
                    return Err("`veterans_wind`: per_level is 1 to 20, at_least 0 to 100".into());
                }
                status(cooldown)
            }
            TalentEffect::LayOnHands {
                per_level,
                cooldown,
                degrees,
            } => {
                if !(1..=20).contains(per_level) || *degrees == 0 {
                    return Err("`lay_on_hands`: per_level is 1 to 20, degrees at least 1".into());
                }
                status(cooldown)
            }
            TalentEffect::Smite {
                per_levels,
                sides,
                cooldown,
                degrees,
            } => {
                if !(1..=20).contains(per_levels) || !(2..=20).contains(sides) || *degrees == 0 {
                    return Err("`smite`: per_levels is 1 to 20, sides 2 to 20, degrees at \
                                least 1"
                        .into());
                }
                status(cooldown)
            }
            TalentEffect::Lurk { status: id } | TalentEffect::SneakKillStatus { status: id } => {
                status(id)
            }
            TalentEffect::QuiveringPalm {
                status: id,
                severity,
            } => {
                if *severity == 0 {
                    return Err("`quivering_palm`: severity is at least 1".into());
                }
                status(id)
            }
            TalentEffect::Blaze {
                radius,
                damage,
                damage_type,
                spell,
                fire,
                burns,
                cooldown,
            } => {
                if !(1..=3).contains(radius) || damage.max() < 1 || damage.min() < 0 {
                    return Err("`blaze`: radius is 1 to 3, damage dice that can do harm".into());
                }
                if !(1..=3600).contains(burns) || spell.trim().is_empty() {
                    return Err("`blaze`: burns is 1 to 3600 seconds, and it names a spell".into());
                }
                let types = content.damage_types();
                if !types.is_empty() && content.damage_type_id(damage_type).is_none() {
                    return Err(format!("names an unknown damage type `{damage_type}`"));
                }
                if !content.light_presets().is_empty() && content.light_id(fire).is_none() {
                    return Err(format!("names an unknown light preset `{fire}`"));
                }
                status(cooldown)
            }
            TalentEffect::Rage { resource: id } => {
                let r = resource(id)?;
                if r.is_some_and(|r| content.resource(r).status.is_none()) {
                    return Err(format!("`rage`: the resource `{id}` brings no `status`"));
                }
                Ok(())
            }
            TalentEffect::ResourceBuild {
                resource: id,
                amount,
                ..
            } => {
                resource(id)?;
                if !(1..=MAX_RESOURCE).contains(amount) {
                    return Err(format!("`resource_build`: amount is 1 to {MAX_RESOURCE}"));
                }
                Ok(())
            }
            TalentEffect::ResourceMax {
                resource: id,
                value,
            } => {
                resource(id)?;
                if !(1..=MAX_RESOURCE).contains(value) {
                    return Err(format!("`resource_max`: value is 1 to {MAX_RESOURCE}"));
                }
                Ok(())
            }
            TalentEffect::ResourceStart {
                resource: id,
                value,
            } => {
                resource(id)?;
                if !(1..=MAX_RESOURCE).contains(value) {
                    return Err(format!("`resource_start`: value is 1 to {MAX_RESOURCE}"));
                }
                Ok(())
            }
            TalentEffect::ResourceRate {
                resource: id,
                value,
            }
            | TalentEffect::GearedResourcePercent {
                resource: id,
                value,
            } => {
                resource(id)?;
                percent(*value)
            }
            TalentEffect::StatusDamage { status: id, value } => {
                bonus(*value)?;
                status(id)
            }
            TalentEffect::Radiance { status: id, light } => {
                status(id)?;
                if content.light_id(light).is_none() {
                    return Err(format!("names an unknown light `{light}`"));
                }
                Ok(())
            }
            TalentEffect::Unstoppable { statuses, cost } => {
                if !(1..=MAX_RESOURCE).contains(cost) {
                    return Err(format!("`unstoppable`: cost is 1 to {MAX_RESOURCE}"));
                }
                statuses.iter().try_for_each(|id| status(id))
            }
            TalentEffect::Resist {
                value,
                damage: Some(kind),
            } => {
                if !content.damage_types().is_empty() && content.damage_type_id(kind).is_none() {
                    return Err(format!("names an unknown damage type `{kind}`"));
                }
                percent(*value)
            }
            TalentEffect::Resist {
                value,
                damage: None,
            } => match choice {
                Some(Choice::Element) => percent(*value),
                _ => Err(
                    "`resist` without a `damage` is for the chosen element: it needs \
                          `choice = \"element\"`"
                        .into(),
                ),
            },
            TalentEffect::Streak { max, damage_every } => {
                if !(1..=10).contains(max) || *damage_every > 10 {
                    return Err("`streak`: max is 1 to 10, damage_every 0 to 10".into());
                }
                Ok(())
            }
            TalentEffect::WoundMore { value } => {
                if !(1..=5).contains(value) {
                    return Err("`wound_more`: value is 1 to 5".into());
                }
                Ok(())
            }
            TalentEffect::HornsGrace {
                cures,
                diseases,
                cooldown,
                degrees,
            } => {
                if cures.is_empty() && *diseases == 0 {
                    return Err("`horns_grace` cures a status or a disease".into());
                }
                if *degrees == 0 {
                    return Err("`horns_grace`: degrees is at least 1".into());
                }
                cures.iter().try_for_each(|id| status(id))?;
                status(cooldown)
            }
            TalentEffect::HornOfPlenty { per_level, degrees } => {
                if !(1..=20).contains(per_level) || *degrees == 0 {
                    return Err("`horn_of_plenty`: per_level is 1 to 20, degrees at least 1".into());
                }
                Ok(())
            }
            TalentEffect::Stare {
                status: id,
                severity,
                cooldown,
                degrees,
            } => {
                if *severity == 0 || *degrees == 0 {
                    return Err("`stare`: severity and degrees are at least 1".into());
                }
                status(id)?;
                status(cooldown)
            }
            TalentEffect::PetrifyingGaze { severity, degrees } => {
                if *severity == 0 || *degrees == 0 {
                    return Err("`petrifying_gaze`: severity and degrees are at least 1".into());
                }
                Ok(())
            }
            TalentEffect::Wildfire { radius, damage } => {
                if !(1..=3).contains(radius) || damage.max() < 1 || damage.min() < 0 {
                    return Err("`wildfire`: radius is 1 to 3, damage dice that can do harm".into());
                }
                Ok(())
            }
            TalentEffect::FireBreath {
                length,
                damage,
                damage_type,
                spell,
                cooldown,
                degrees,
            } => {
                if !(1..=crate::world::BREATH_RANGE).contains(length)
                    || damage.max() < 1
                    || damage.min() < 0
                    || *degrees == 0
                {
                    return Err(format!(
                        "`fire_breath`: length is 1 to {}, damage dice that can do harm, \
                         degrees at least 1",
                        crate::world::BREATH_RANGE
                    ));
                }
                let types = content.damage_types();
                if !types.is_empty() && content.damage_type_id(damage_type).is_none() {
                    return Err(format!("names an unknown damage type `{damage_type}`"));
                }
                super::spell::check_effect(content, spell, super::effect::EffectShape::Bolt)?;
                status(cooldown)
            }
            TalentEffect::HoundsBay {
                status: id,
                severity,
                cooldown,
            } => {
                if *severity == 0 {
                    return Err("`hounds_bay`: severity is at least 1".into());
                }
                status(id)?;
                status(cooldown)
            }
            TalentEffect::Weapon {
                role,
                skill,
                to_hit,
                damage,
                threat,
                ranks,
                speed,
                reload,
                ease,
                bash,
                shield,
                crit,
                at_rank,
            } => {
                if let Some(id) = skill {
                    if role.is_some() {
                        return Err("`weapon` names a `role` or a `skill`, not both".into());
                    }
                    let table = content.weapon_skills();
                    if !table.skills().is_empty() && table.id(id).is_none() {
                        return Err(format!("names an unknown weapon skill `{id}`"));
                    }
                } else if role.is_none()
                    && !matches!(choice, Some(Choice::WeaponSkill | Choice::Style))
                {
                    return Err(
                        "`weapon` without a `role` or a `skill` is for the chosen weapon skill \
                         or style: it needs that `choice`"
                            .into(),
                    );
                }
                let small = [to_hit, damage, shield].iter().all(|v| BONUSES.contains(v))
                    && [threat, ranks, ease, crit].iter().all(|&&v| v <= 5)
                    && *at_rank <= MAX_RANK;
                let percents = [speed, reload, bash].iter().all(|&&v| v <= 100);
                let any = [to_hit, damage, shield].iter().any(|&&v| v != 0)
                    || [threat, ranks, ease, speed, reload, bash, crit]
                        .iter()
                        .any(|&&v| v != 0);
                if !small || !percents || !any {
                    return Err("`weapon`: to_hit, damage, shield are -20 to 20; threat, \
                                ranks, ease, crit 0 to 5; at_rank 0 to 10; speed, reload, bash \
                                0 to 100; not all 0"
                        .into());
                }
                Ok(())
            }
            TalentEffect::CasterLevel { value, per_levels } => {
                if *per_levels > 20 {
                    return Err("`caster_level`: per_levels is 0 to 20".into());
                }
                bonus(*value)
            }
            TalentEffect::SchoolFocus { cost, reach, lift } => {
                if choice != Some(Choice::School) {
                    return Err(
                        "`school_focus` is for the chosen school or element: it needs \
                                `choice = \"school\"`"
                            .into(),
                    );
                }
                if !(0..=5).contains(reach)
                    || !PERCENTS.contains(cost)
                    || !(0..=100).contains(lift)
                    || (*cost == 0 && *reach == 0 && *lift == 0)
                {
                    return Err(
                        "`school_focus`: cost is a percent, reach 0 to 5, lift 0 to \
                                100, not all 0"
                            .into(),
                    );
                }
                Ok(())
            }
            TalentEffect::Study {
                bonus: b,
                percent: p,
            } => {
                bonus(*b)?;
                percent(*p)
            }
            TalentEffect::LauncherRange { value, skill } => {
                let table = content.weapon_skills();
                if let Some(id) = skill
                    && !table.skills().is_empty()
                    && table.id(id).is_none()
                {
                    return Err(format!("names an unknown weapon skill `{id}`"));
                }
                bonus(*value)
            }
            TalentEffect::ExtraPick { level } => {
                if !(1..=crate::content::MAX_LEVEL).contains(level) {
                    return Err(format!(
                        "`extra_pick`: level is 1 to {}",
                        crate::content::MAX_LEVEL
                    ));
                }
                Ok(())
            }
            TalentEffect::HealingPercent { value }
            | TalentEffect::DiseaseIntervalPercent { value }
            | TalentEffect::ManaRegenPercent { value }
            | TalentEffect::ManaRestingPercent { value }
            | TalentEffect::ManaCostPercent { value }
            | TalentEffect::AttunedPercent { value }
            | TalentEffect::RodCostPercent { value }
            | TalentEffect::FootstepsPercent { value }
            | TalentEffect::CarryPercent { value }
            | TalentEffect::TravelPercent { value }
            | TalentEffect::HungerPercent { value }
            | TalentEffect::XpPercent { value }
            | TalentEffect::RotHarmPercent { value }
            | TalentEffect::RestingPercent { value }
            | TalentEffect::PenaltyPercent { value }
            | TalentEffect::DigPercent { value }
            | TalentEffect::SpawnEveryPercent { value }
            | TalentEffect::RoomPacePercent { value }
            | TalentEffect::WakePercent { value }
            | TalentEffect::SurpriseEase { value }
            | TalentEffect::TwoHandedPrice { value }
            | TalentEffect::WeaponEnergyPercent { value }
            | TalentEffect::SpeedPercent { value }
            | TalentEffect::SpellDamagePercent { value }
            | TalentEffect::SpellManaPercent { value }
            | TalentEffect::OverreachCostPercent { value }
            | TalentEffect::SkyFootstepsPercent { value }
            | TalentEffect::AttackNoisePercent { value }
            | TalentEffect::StepSlowdownPercent { value }
            | TalentEffect::CorpseStatsPercent { value }
            | TalentEffect::PotionMarksPercent { value }
            | TalentEffect::LootMagicPercent { value }
            | TalentEffect::SalvagePercent { value }
            | TalentEffect::GemFindPercent { value }
            | TalentEffect::MaxHpPercent { value }
            | TalentEffect::ReadPercent { value }
            | TalentEffect::TrapDamagePercent { value }
            | TalentEffect::KickNoisePercent { value }
            | TalentEffect::SmashDestroyPercent { value }
            | TalentEffect::ThresholdPercent { value }
            | TalentEffect::ClimbPercent { value }
            | TalentEffect::SunHealing { value }
            | TalentEffect::QuestXpPercent { value }
            | TalentEffect::QuestGoldPercent { value }
            | TalentEffect::FavourPercent { value }
            | TalentEffect::FavourCap { value }
            | TalentEffect::SellPercent { value }
            | TalentEffect::MissileBreaks { value }
            | TalentEffect::BurdenEase { value } => percent(*value),
            TalentEffect::Scent { status: id, radius } => {
                if !(1..=20).contains(radius) {
                    return Err("`scent`: radius is 1 to 20".into());
                }
                status(id)
            }
            TalentEffect::InnerLight { light } => {
                if !content.light_presets().is_empty() && content.light_id(light).is_none() {
                    return Err(format!("names an unknown light `{light}`"));
                }
                Ok(())
            }
            TalentEffect::RiseFromAshes {
                hp_percent,
                status: id,
                spell,
                radius,
                damage,
                damage_type,
            } => {
                if !(1..=100).contains(hp_percent) || !(1..=5).contains(radius) {
                    return Err("`rise_from_ashes`: hp_percent is 1 to 100, radius 1 to 5".into());
                }
                if damage.is_zero() {
                    return Err("`rise_from_ashes`: damage is not 0".into());
                }
                if !content.damage_types().is_empty()
                    && content.damage_type_id(damage_type).is_none()
                {
                    return Err(format!("names an unknown damage type `{damage_type}`"));
                }
                super::spell::check_effect(content, spell, super::effect::EffectShape::Burst)?;
                status(id)
            }
            TalentEffect::LanternLight { light } => {
                if !content.light_presets().is_empty() && content.light_id(light).is_none() {
                    return Err(format!("names an unknown light preset `{light}`"));
                }
                Ok(())
            }
            TalentEffect::Ambush { dice } => {
                if dice.is_zero() {
                    return Err("`ambush`: dice is not 0".into());
                }
                Ok(())
            }
            TalentEffect::TerrainAs { terrains, .. } => {
                // The terrain ids are checked once the terrain has loaded
                // ([`TalentBook::check_terrain`]).
                if terrains.is_empty() {
                    return Err("`terrain_as`: terrains names at least one".into());
                }
                Ok(())
            }
            TalentEffect::Forecast { hours } => {
                if !(1..=24).contains(hours) {
                    return Err("`forecast`: hours is 1 to 24".into());
                }
                Ok(())
            }
            TalentEffect::StatusEases { status: id, value }
            | TalentEffect::StatusLasts { status: id, value } => {
                status(id)?;
                percent(*value)
            }
            TalentEffect::Brace { to_hit, status: id } => {
                status(id)?;
                bonus(*to_hit)
            }
            TalentEffect::ShieldCasting { value } => {
                if !(1..=100).contains(value) {
                    return Err("`shield_casting`: value is 1 to 100".into());
                }
                Ok(())
            }
            TalentEffect::BothShields { .. } => Ok(()),
            // The abilities it names are checked once they are loaded, after the talents
            // (`content/weapon_ability.rs`).
            TalentEffect::WeaponStamina { value, .. } => {
                if !(1..=10).contains(value) {
                    return Err("`weapon_stamina`: value is 1 to 10".into());
                }
                Ok(())
            }
            TalentEffect::NoPenalty { status: id }
            | TalentEffect::CriticalStatus { status: id }
            | TalentEffect::Grip { status: id } => status(id),
            TalentEffect::SecondWind {
                per_level,
                at_least,
                cooldown,
            } => {
                if !(1..=20).contains(per_level) || !(0..=100).contains(at_least) {
                    return Err("`second_wind`: per_level is 1 to 20, at_least 0 to 100".into());
                }
                status(cooldown)
            }
            TalentEffect::KickDie { sides } => {
                if !(2..=20).contains(sides) {
                    return Err("`kick_die`: sides is 2 to 20".into());
                }
                Ok(())
            }
            TalentEffect::Penetration {
                damage: kind,
                value,
            } => {
                if !content.damage_types().is_empty() && content.damage_type_id(kind).is_none() {
                    return Err(format!("names an unknown damage type `{kind}`"));
                }
                bonus(*value)
            }
            TalentEffect::MaxHpPerLevels {
                per_levels,
                at_least,
            } => {
                if !(1..=20).contains(per_levels) || !(0..=20).contains(at_least) {
                    return Err(
                        "`max_hp_per_levels`: per_levels is 1 to 20, at_least 0 to 20".into(),
                    );
                }
                Ok(())
            }
            TalentEffect::MaxManaPerLevels {
                per_levels,
                at_least,
            } => {
                if !(1..=20).contains(per_levels) || !(0..=20).contains(at_least) {
                    return Err(
                        "`max_mana_per_levels`: per_levels is 1 to 20, at_least 0 to 20".into(),
                    );
                }
                Ok(())
            }
            TalentEffect::BloodMagic { hp, mana } => {
                if !(1..=50).contains(hp) || !(1..=50).contains(mana) {
                    return Err("`blood_magic`: hp and mana are 1 to 50".into());
                }
                Ok(())
            }
            TalentEffect::SilentDoors
            | TalentEffect::NoExertion
            | TalentEffect::RerollFumble
            | TalentEffect::RendOffHand
            | TalentEffect::RebornWhole
            | TalentEffect::SecondRising
            | TalentEffect::StarvingHeals
            | TalentEffect::EatsRotten
            | TalentEffect::WadesFree
            | TalentEffect::Swimmer
            | TalentEffect::StoneSense
            | TalentEffect::FreeBlow { .. }
            | TalentEffect::IronStomach
            | TalentEffect::Clotting
            | TalentEffect::SureStep
            | TalentEffect::PitProof
            | TalentEffect::IgnoreBurdened
            | TalentEffect::NoBloating
            | TalentEffect::SparesSelf
            | TalentEffect::SeesUv
            | TalentEffect::SenseOfPlace
            | TalentEffect::ChestWise
            | TalentEffect::Slick
            | TalentEffect::TreasureSense
            | TalentEffect::Devout
            | TalentEffect::Penitent
            | TalentEffect::CollapseProof
            | TalentEffect::ExactFootsteps
            | TalentEffect::SmellRot
            | TalentEffect::PotionSmell
            | TalentEffect::StanceCosts
            | TalentEffect::PlungeProof
            | TalentEffect::LightOnIce
            | TalentEffect::WadingProof
            | TalentEffect::BeastFriend
            | TalentEffect::Steadfast
            | TalentEffect::Stout
            | TalentEffect::DiseaseImmune
            | TalentEffect::NoPenaltyTo { .. }
            | TalentEffect::Avatar
            | TalentEffect::Whirlwind
            | TalentEffect::WeightlessCoins
            | TalentEffect::IdentifyTrades
            | TalentEffect::Pilgrim
            | TalentEffect::Scavenger
            | TalentEffect::ItemWorth
            | TalentEffect::GemWorth
            | TalentEffect::AppraiseAnimated
            | TalentEffect::SeesUnaware
            | TalentEffect::StartledBlow
            | TalentEffect::MakeWay
            | TalentEffect::Distrusted
            | TalentEffect::ExactSounds
            | TalentEffect::QuarrySense
            | TalentEffect::SilentUnburdened
            | TalentEffect::PickLock
            | TalentEffect::Steal
            | TalentEffect::ThrowVoice
            | TalentEffect::Finisher
            | TalentEffect::LowBlow
            | TalentEffect::GraveWarden
            | TalentEffect::Disarm
            | TalentEffect::Purify
            | TalentEffect::CorpseHarmless
            | TalentEffect::DiseaseStill => Ok(()),
            // A spell ability names a spell of data/spells/ of its id, aimed as the ability is
            // (content without spells has nothing to check it by).
            TalentEffect::Spell { spell } => {
                if content.spells().is_empty() || content.spell_id(spell).is_some() {
                    Ok(())
                } else {
                    Err(format!("`spell`: no spell `{spell}` in data/spells/"))
                }
            }
            TalentEffect::SpellPower { per_die, dice } => {
                if !(0..=5).contains(per_die) || *dice > 5 || (*per_die == 0 && *dice == 0) {
                    return Err("`spell_power`: per_die is 0 to 5, dice 0 to 5, not both 0".into());
                }
                Ok(())
            }
            TalentEffect::MealHeals { dice, .. } => {
                if dice.max() < 1 || dice.min() < 0 {
                    return Err("`meal_heals`: dice that heal".into());
                }
                Ok(())
            }
            TalentEffect::Reprieve {
                once,
                status: id,
                stat,
                by,
                message,
            } => {
                if once.trim().is_empty() || message.trim().is_empty() {
                    return Err("`reprieve`: needs `once` and a `message`".into());
                }
                if stat.is_some() != (*by != 0) || !(-5..=5).contains(by) {
                    return Err("`reprieve`: a `stat` with `by` -5 to 5, not 0, or neither".into());
                }
                status(id)
            }
        }
    }
}

/// One prerequisite of a talent, as it is checked and as it is shown ("Toughness 14").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Need {
    /// Another talent, at this rank at least.
    Talent(TalentId, u8),
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
    /// Eyes: the race's vision profile, by id.
    Eyes(String),
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
    /// "Hardy 2", "Stealth 25", "to be a dwarf".
    pub fn words(&self, content: &Content) -> String {
        match self {
            Need::Talent(id, rank) => {
                let name = &content.talents().get(*id).name;
                if *rank > 1 {
                    format!("{name} {rank}")
                } else {
                    name.clone()
                }
            }
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
            Need::Eyes(id) => {
                let name = content.vision(id).map_or(id.as_str(), |v| v.name.as_str());
                format!("{} eyes", name.to_lowercase())
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

/// The prerequisites of a talent, or of one of its ranks (spec §8 "Talents"): all must hold
/// when it is chosen; none is checked again later.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Needs {
    pub talents: Vec<(TalentId, u8)>,
    /// Scores by stat; 0 asks nothing.
    pub stats: PerStat<u8>,
    pub skills: Vec<(SkillId, u16)>,
    pub weapon_skills: Vec<(WeaponSkillId, u8)>,
    pub any_weapon_skill: Option<AnyWeaponSkill>,
    pub chosen: Option<u8>,
    /// 0 asks nothing.
    pub level: u8,
    pub race: Option<String>,
    pub eyes: Option<String>,
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
        let talents = self.talents.iter().map(|&(t, r)| Need::Talent(t, r));
        let stats = (self.stats.iter())
            .filter(|&(_, score)| score > 0)
            .map(|(stat, score)| Need::Stat(stat, score));
        let skills = self.skills.iter().map(|&(s, v)| Need::Skill(s, v));
        let weapons = (self.weapon_skills.iter()).map(|&(w, r)| Need::WeaponSkill(w, r));
        let any = self.any_weapon_skill.clone().map(Need::AnyWeaponSkill);
        let chosen = self.chosen.map(Need::Chosen);
        let level = (self.level > 0).then_some(Need::Level(self.level));
        let race = self.race.clone().map(Need::Race);
        let eyes = self.eyes.clone().map(Need::Eyes);
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

/// One rank of a talent: what it asks besides the talent's own needs, and what it adds besides
/// the talent's per-rank effects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rank {
    pub needs: Needs,
    pub effects: Vec<TalentEffect>,
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
    /// What every rank asks.
    pub needs: Needs,
    /// What every rank held adds.
    pub effects: Vec<TalentEffect>,
    /// Its ranks, first to last (at least one).
    pub ranks: Vec<Rank>,
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
    /// = "kind" }` in data: the paladin's holy gifts), if any. It bars nothing: the ability is
    /// had, listed, and used whatever the band, and outside these bands it does nothing
    /// (`World::answered`).
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
    /// How many ranks it has.
    pub fn rank_count(&self) -> u8 {
        self.ranks.len() as u8
    }

    /// What taking `rank` (1 first) asks: the talent's needs, then that rank's.
    pub fn needs_at(&self, rank: u8) -> impl Iterator<Item = Need> + '_ {
        let own = (self.ranks.get(usize::from(rank.max(1)) - 1)).map(|r| &r.needs);
        self.needs
            .iter()
            .chain(own.into_iter().flat_map(Needs::iter))
    }

    /// What holding `rank` (1 first) adds: the per-rank effects, then that rank's own.
    pub fn effects_at(&self, rank: u8) -> impl Iterator<Item = TalentEffect> + '_ {
        let own = (self.ranks.get(usize::from(rank.max(1)) - 1)).map(|r| r.effects.as_slice());
        (self.effects.iter().cloned()).chain(own.into_iter().flatten().cloned())
    }

    /// Whether its first rank may be taken by anyone: no prerequisites at all.
    pub fn is_free(&self) -> bool {
        !self.granted && self.needs_at(1).next().is_none()
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
    pub coverage: Coverage,
    /// The most ranks a talent may have.
    pub max_ranks: u8,
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
            let effects = (def.effects.iter()).chain(def.ranks.iter().flat_map(|r| &r.effects));
            for effect in effects {
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

    /// The coverage check (spec §8 "Talents"): every system not `pending` has at least
    /// `min_talents` talents (its own `min_talents`, if it gives one) (those of a class tree are class abilities, not talents) and
    /// `min_class` class abilities (`class_abilities` counts them by system id: the talents the
    /// class tables give), and every tree not `pending`, `no_free`, nor a class tree has at least
    /// `free_per_tree` talents without prerequisites. The first gap, in words.
    pub fn coverage_gap(&self, class_abilities: &dyn Fn(&str) -> u32) -> Option<String> {
        let bar = self.coverage;
        for (i, system) in (0u8..).zip(&self.systems) {
            if system.pending {
                continue;
            }
            let talents = (self.of_system(SystemId(i)))
                .filter(|(_, t)| !self.tree(t.tree).class)
                .count() as u32;
            let least = system.min_talents.unwrap_or(bar.min_talents);
            if talents < least {
                return Some(format!(
                    "system `{}` has {talents} talent{} but at least {least} are needed (mark it \
                     `pending = true` until they land)",
                    system.id,
                    if talents == 1 { "" } else { "s" },
                ));
            }
            let abilities = class_abilities(&system.id);
            if abilities < bar.min_class {
                return Some(format!(
                    "system `{}` has {abilities} class abilit{} but at least {} are needed",
                    system.id,
                    if abilities == 1 { "y" } else { "ies" },
                    bar.min_class
                ));
            }
        }
        for (i, tree) in (0u8..).zip(&self.trees) {
            if tree.pending || tree.no_free || tree.class {
                continue;
            }
            let free = self.of_tree(TreeId(i)).filter(|(_, t)| t.is_free()).count() as u32;
            if free < bar.free_per_tree {
                return Some(format!(
                    "tree `{}` has {free} talent{} without prerequisites but at least {} are \
                     needed, so a pick from it is never impossible",
                    tree.id,
                    if free == 1 { "" } else { "s" },
                    bar.free_per_tree
                ));
            }
        }
        None
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentFile {
    max_ranks: Option<u8>,
    coverage: Option<Coverage>,
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
    pending: bool,
    #[serde(default)]
    no_free: bool,
    #[serde(default)]
    class: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SystemSpec {
    id: String,
    name: String,
    #[serde(default)]
    pending: bool,
    min_talents: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentSpec {
    id: String,
    name: String,
    tree: String,
    ranks: Option<u8>,
    choice: Option<Choice>,
    description: String,
    #[serde(default)]
    needs: NeedsSpec,
    #[serde(default)]
    effects: Vec<TalentEffect>,
    #[serde(default)]
    rank: Vec<RankSpec>,
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

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RankSpec {
    #[serde(default)]
    needs: NeedsSpec,
    #[serde(default)]
    effects: Vec<TalentEffect>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
struct NeedsSpec {
    talents: BTreeMap<String, u8>,
    stats: BTreeMap<String, u8>,
    skills: BTreeMap<String, u16>,
    weapon_skills: BTreeMap<String, u8>,
    any_weapon_skill: Option<AnySpec>,
    chosen: Option<u8>,
    level: u8,
    race: Option<String>,
    eyes: Option<String>,
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

impl NeedsSpec {
    /// How many prerequisites it names.
    fn count(&self) -> usize {
        let one = |b: bool| usize::from(b);
        self.talents.len()
            + self.stats.len()
            + self.skills.len()
            + self.weapon_skills.len()
            + one(self.any_weapon_skill.is_some())
            + one(self.chosen.is_some())
            + one(self.level > 0)
            + one(self.race.is_some())
            + one(self.eyes.is_some())
            + one(self.class.is_some())
    }
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
    if spec.rank == 0 || spec.rank > MAX_RANK {
        return Err(format!("`any_weapon_skill`: ranks are 1 to {MAX_RANK}"));
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
    if skills.len() < 2 {
        return Err("`any_weapon_skill` names fewer than two skills".into());
    }
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

/// Resolves `spec`'s prerequisites against `content` and the talent `ids`; `choice` is the
/// talent's, for `chosen`.
fn resolve_needs(
    spec: NeedsSpec,
    ids: &BTreeMap<String, TalentId>,
    ranks_of: &dyn Fn(TalentId) -> u8,
    choice: Option<Choice>,
    content: &Content,
) -> Result<Needs, String> {
    let mut talents = Vec::new();
    for (name, rank) in spec.talents {
        let id = *ids
            .get(&name)
            .ok_or_else(|| format!("needs an unknown talent `{name}`"))?;
        if rank == 0 || rank > ranks_of(id) {
            return Err(format!(
                "needs `{name}` at rank {rank}, which it does not have (it has {})",
                ranks_of(id)
            ));
        }
        talents.push((id, rank));
    }
    let mut stats = PerStat::splat(0u8);
    for (name, score) in spec.stats {
        let stat = Stat::from_id(&name).ok_or_else(|| format!("needs an unknown stat `{name}`"))?;
        if !SCORES.contains(&score) {
            return Err(format!(
                "needs {} {score}: scores are {} to {}",
                stat.name(),
                SCORES.start(),
                SCORES.end()
            ));
        }
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
        if value == 0 || value > crate::skills::MAX_VALUE {
            return Err(format!(
                "needs {name} {value}: values are 1 to {}",
                crate::skills::MAX_VALUE
            ));
        }
        skills.push((id, value));
    }
    let mut weapon_skills = Vec::new();
    for (name, rank) in spec.weapon_skills {
        if content.weapon_skills().skills().is_empty() {
            break;
        }
        let id = (content.weapon_skills().id(&name))
            .ok_or_else(|| format!("needs an unknown weapon skill `{name}`"))?;
        if rank == 0 || rank > MAX_RANK {
            return Err(format!("needs {name} {rank}: ranks are 1 to {MAX_RANK}"));
        }
        weapon_skills.push((id, rank));
    }
    let any_weapon_skill = match spec.any_weapon_skill {
        Some(any) => resolve_any(any, content)?,
        None => None,
    };
    if let Some(rank) = spec.chosen {
        match choice {
            None => return Err("`chosen` needs a `choice`".into()),
            Some(Choice::Element | Choice::Skill | Choice::School) => {
                return Err("`chosen` asks a weapon skill's or style's rank".into());
            }
            Some(_) if rank == 0 || rank > MAX_RANK => {
                return Err(format!("`chosen` is a rank, 1 to {MAX_RANK}"));
            }
            Some(_) => {}
        }
    }
    if spec.level > MAX_LEVEL {
        return Err(format!(
            "needs level {}: levels are 1 to {MAX_LEVEL}",
            spec.level
        ));
    }
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
    if let Some(eyes) = &spec.eyes
        && content.vision(eyes).is_none()
    {
        return Err(format!("needs unknown eyes `{eyes}` (a vision profile)"));
    }
    let either = match spec.either {
        Some(alternatives) => {
            if alternatives.either.is_some() {
                return Err("`either` within `either`".into());
            }
            if alternatives.count() < 2 {
                return Err("`either` names fewer than two alternatives".into());
            }
            // Content without skills (a fixture) leaves a skill alternative out, as above.
            let alternatives = resolve_needs(*alternatives, ids, ranks_of, choice, content)?;
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
        eyes: spec.eyes,
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
        let effects = (def.effects.iter()).chain(def.ranks.iter().flat_map(|r| &r.effects));
        for effect in effects {
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

/// Parses the talent table: the settings (`max_ranks`, `[coverage]`, `[[tree]]`, `[[system]]`,
/// each set in at most one file) and the talents, resolved against `content`, which must have
/// its skills, weapon skills, races, classes, vision profiles, and damage types already.
pub(super) fn parse<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<TalentBook, ContentError> {
    let mut max_ranks = Once::new("max_ranks");
    let mut coverage = Once::new("coverage");
    let mut first_file = None;
    let mut trees: Vec<(&str, TreeSpec)> = Vec::new();
    let mut systems: Vec<(&str, SystemSpec)> = Vec::new();
    let mut pending: Vec<Pending<'a>> = Vec::new();
    for (file, text) in files {
        first_file.get_or_insert(file);
        let parsed: TalentFile =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        max_ranks.take(file, parsed.max_ranks)?;
        coverage.take(file, parsed.coverage)?;
        trees.extend(parsed.tree.into_iter().map(|t| (file, t)));
        systems.extend(parsed.system.into_iter().map(|s| (file, s)));
        pending.extend(parsed.talent.into_iter().map(|spec| Pending { file, spec }));
    }
    let first_file = first_file.unwrap_or("data/talents.toml");
    let max_ranks = max_ranks.get_or(DEFAULT_MAX_RANKS);
    let coverage = coverage.get_or(Coverage::default());
    if max_ranks == 0 {
        return Err(ContentError::new(first_file, "`max_ranks` is at least 1"));
    }
    let mut book = TalentBook {
        trees: Vec::new(),
        systems: Vec::new(),
        coverage,
        max_ranks,
        list: Vec::new(),
        ids: BTreeMap::new(),
    };
    let mut tree_files: BTreeMap<String, &str> = BTreeMap::new();
    for (file, t) in trees {
        if let Some(first) = tree_files.insert(t.id.clone(), file) {
            return Err(duplicate_id("tree", &t.id, file, first));
        }
        if t.name.is_empty() || t.description.is_empty() {
            return Err(ContentError::new(
                file,
                format!("tree `{}` needs a name and a description", t.id),
            ));
        }
        book.trees.push(TreeDef {
            id: t.id,
            name: t.name,
            description: t.description,
            pending: t.pending,
            no_free: t.no_free,
            class: t.class,
        });
    }
    let mut system_files: BTreeMap<String, &str> = BTreeMap::new();
    for (file, s) in systems {
        if let Some(first) = system_files.insert(s.id.clone(), file) {
            return Err(duplicate_id("system", &s.id, file, first));
        }
        if s.name.is_empty() {
            return Err(ContentError::new(
                file,
                format!("system `{}` needs a name", s.id),
            ));
        }
        book.systems.push(SystemDef {
            id: s.id,
            name: s.name,
            pending: s.pending,
            min_talents: s.min_talents,
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
    let mut ranks: Vec<u8> = Vec::new();
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
        let err = |m: String| ContentError::new(p.file, format!("talent `{}`: {m}", spec.id));
        let count = match (spec.ranks, spec.rank.len()) {
            (None, 0) => 1,
            (None, n) => n as u8,
            (Some(r), 0) => r,
            (Some(r), n) if usize::from(r) == n => r,
            (Some(r), n) => {
                return Err(err(format!("`ranks = {r}` but {n} `[[rank]]` entries")));
            }
        };
        if count == 0 || count > max_ranks {
            return Err(err(format!("a talent has 1 to {max_ranks} ranks")));
        }
        book.ids.insert(spec.id.clone(), TalentId(i as u16));
        ranks.push(count);
    }
    let ranks_of = |id: TalentId| ranks[usize::from(id.0)];
    let mut spell_ids = std::collections::BTreeSet::new();
    for (i, p) in pending.into_iter().enumerate() {
        let spec = p.spec;
        let err = |m: String| ContentError::new(p.file, format!("talent `{}`: {m}", spec.id));
        if spec.name.is_empty() || spec.description.is_empty() {
            return Err(err("needs a name and a description".into()));
        }
        let tree = (book.tree_id(&spec.tree))
            .ok_or_else(|| err(format!("unknown tree `{}`", spec.tree)))?;
        if book.tree(tree).class && !spec.granted {
            return Err(err(format!(
                "is in the class tree `{}`: a class ability needs `granted = true`",
                spec.tree
            )));
        }
        if spec.systems.is_empty() {
            return Err(err(
                "names no `systems` (the coverage list in data/talents.toml)".into(),
            ));
        }
        let mut systems = Vec::new();
        for name in &spec.systems {
            let id = (book.system_id(name)).ok_or_else(|| {
                err(format!(
                    "names a system `{name}` that is not in the list (data/talents.toml)"
                ))
            })?;
            if systems.contains(&id) {
                return Err(err(format!("names system `{name}` twice")));
            }
            systems.push(id);
        }
        let count = ranks[i];
        let needs =
            resolve_needs(spec.needs, &book.ids, &ranks_of, spec.choice, content).map_err(err)?;
        if needs.talents.iter().any(|(t, _)| *t == TalentId(i as u16)) {
            return Err(err("needs itself".into()));
        }
        for effect in &spec.effects {
            effect.check(content, spec.choice).map_err(err)?;
        }
        let effects = spec.effects;
        for effect in &effects {
            if let TalentEffect::Spell { spell } = effect
                && !spell_ids.insert(spell.clone())
            {
                return Err(err(format!("spell `{spell}` is given twice")));
            }
        }
        let mut rank_defs = Vec::new();
        for (n, rank) in spec.rank.into_iter().enumerate() {
            let own = resolve_needs(rank.needs, &book.ids, &ranks_of, spec.choice, content)
                .map_err(|m| err(format!("rank {}: {m}", n + 1)))?;
            if own.talents.iter().any(|(t, _)| *t == TalentId(i as u16)) {
                return Err(err(format!("rank {}: needs itself", n + 1)));
            }
            for effect in &rank.effects {
                effect
                    .check(content, spec.choice)
                    .map_err(|m| err(format!("rank {}: {m}", n + 1)))?;
            }
            rank_defs.push(Rank {
                needs: own,
                effects: rank.effects,
            });
        }
        rank_defs.resize_with(usize::from(count), Rank::default);
        if effects.is_empty() && rank_defs.iter().all(|r| r.effects.is_empty()) {
            return Err(err("does nothing: it needs `effects`".into()));
        }
        if spec.flaw && !spec.granted {
            return Err(err("a `flaw` is `granted`: nobody picks a flaw".into()));
        }
        let resource = match &spec.resource {
            None => None,
            Some(cost) => {
                if !effects.iter().any(|e| e.active().is_some()) {
                    return Err(err("a `resource` cost is for a class active".into()));
                }
                let resource = content.resource_id(&cost.id).ok_or_else(|| {
                    err(format!(
                        "costs a resource `{}` that is not in data/resources.toml",
                        cost.id
                    ))
                })?;
                if cost.cost > content.resource(resource).max {
                    return Err(err(format!("costs more `{}` than it can hold", cost.id)));
                }
                Some(ResourceCost {
                    resource,
                    amount: cost.cost,
                })
            }
        };
        if let Some(bands) = spec.alignment {
            if !effects.iter().any(|e| e.active().is_some_and(|a| a.holy())) {
                return Err(err(
                    "an `alignment` is for a holy class active (lay_on_hands, smite, \
                     holy_radiance, consecrate)"
                        .into(),
                ));
            }
            bands.check().map_err(err)?;
        }
        book.list.push(TalentDef {
            id: spec.id,
            name: spec.name,
            tree,
            choice: spec.choice,
            description: spec.description,
            needs,
            effects,
            ranks: rank_defs,
            systems,
            granted: spec.granted,
            flaw: spec.flaw,
            resource,
            alignment: spec.alignment,
        });
    }
    // A talent that needs one with a choice of its own kind is taken for the same choice: only
    // talents of the same choice kind may chain that way.
    for def in &book.list {
        let needed =
            (def.needs.talents.iter()).chain(def.ranks.iter().flat_map(|r| r.needs.talents.iter()));
        for &(other, _) in needed {
            let other = book.get(other);
            if other.flaw {
                return Err(ContentError::new(
                    files[&def.id],
                    format!(
                        "talent `{}` needs `{}`, a flaw: a flaw is never a prerequisite",
                        def.id, other.id
                    ),
                ));
            }
            if let Some(kind) = other.choice
                && def.choice != Some(kind)
            {
                return Err(ContentError::new(
                    files[&def.id],
                    format!(
                        "talent `{}` needs `{}`, which takes {}: only a talent with the same \
                         choice may need it",
                        def.id,
                        other.id,
                        kind.words()
                    ),
                ));
            }
        }
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
        files.insert("data/talents.toml", settings);
        for (name, text) in talents {
            files.insert(&format!("data/talents/{name}.toml"), text);
        }
        Content::load(&files)
    }

    const SETTINGS: &str = r#"
max_ranks = 3
[coverage]
min_talents = 2
min_class = 0
free_per_tree = 1
[[tree]]
id = "vigour"
name = "Vigour"
description = "Hit points."
[[tree]]
id = "shadow"
name = "Shadow"
description = "Stealth."
pending = true
[[system]]
id = "hit_points"
name = "Hit points"
[[system]]
id = "stealth"
name = "Stealth"
pending = true
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
ranks = 3
description = "More hit points."
effects = [{ kind = "max_hp_per_levels", per_levels = 4, at_least = 3 }]
systems = ["hit_points"]
[[talent.rank]]
needs = { stats = { toughness = 11 } }
[[talent.rank]]
needs = { stats = { toughness = 14 } }
[[talent.rank]]
needs = { stats = { toughness = 17 }, talents = { healthy = 1 } }
"#;

    fn err(result: Result<Content, ContentError>) -> String {
        match result {
            Ok(_) => panic!("loaded"),
            Err(e) => e.to_string(),
        }
    }

    #[test]
    fn talents_load_with_ranks_needs_and_effects() {
        let content = load(SETTINGS, &[("healthy", HEALTHY), ("hardy", HARDY)]).expect("loads");
        let book = content.talents();
        assert_eq!(book.len(), 2);
        let hardy = book.get(book.id("hardy").expect("hardy"));
        assert_eq!(hardy.rank_count(), 3);
        assert_eq!(hardy.tree, book.tree_id("vigour").expect("tree"));
        let needs: Vec<String> = hardy.needs_at(3).map(|n| n.words(&content)).collect();
        assert_eq!(
            needs,
            ["Healthy", "Toughness 17"],
            "talents first, then scores"
        );
        assert_eq!(hardy.effects_at(2).count(), 1);
        assert!(!hardy.is_free());
        assert!(book.get(book.id("healthy").expect("healthy")).is_free());
        assert_eq!(book.by_tree().count(), 2);
        assert_eq!(book.coverage_gap(&|_| 0), None);
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
        assert!(bad("effects = [{ kind = \"dodge\", value = 0 }]").contains("a bonus"));
        // Alignment bars nothing: it is neither a prerequisite nor for anything but a holy gift.
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nalignment = { min = \"good\" }")
                .contains("is for a holy class active")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\n\
                 needs = { requires_alignment = { min = \"good\" } }")
            .contains("unknown field")
        );
        assert!(bad("effects = [{ kind = \"xp_percent\", value = 500 }]").contains("a percent"));
        // The Spellcraft tree's (#400): School focus asks for its choice and a sane reach.
        assert!(
            bad("effects = [{ kind = \"school_focus\", cost = -10, reach = 1 }]")
                .contains("is for the chosen school")
        );
        assert!(
            bad("choice = \"school\"\neffects = [{ kind = \"school_focus\", cost = -10, reach = 9 }]")
                .contains("reach 0 to 5")
        );
        assert!(
            bad("effects = [{ kind = \"caster_level\", value = 1, per_levels = 50 }]")
                .contains("per_levels is 0 to 20")
        );
        assert!(
            bad("effects = [{ kind = \"overreach_cost_percent\", value = -200 }]")
                .contains("a percent")
        );
        assert!(bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { stats = { toughness = 50 } }").contains("scores are"));
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { talents = { hardy = 4 } }")
                .contains("rank 4")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { talents = { x = 1 } }")
                .contains("needs itself")
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
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { chosen = 2 }")
                .contains("needs a `choice`")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nranks = 4").contains("1 to 3 ranks")
        );
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nranks = 2\n[[talent.rank]]\n")
                .contains("`ranks = 2` but 1")
        );
        assert!(bad("").contains("does nothing"));
        assert!(
            bad("effects = [{ kind = \"dodge\", value = 1 }]\nneeds = { either = { level = 3 } }")
                .contains("fewer than two")
        );
        assert!(
            bad("effects = [{ kind = \"train_skill\", dice = \"1d6\" }]")
                .contains("needs `choice = \"skill\"`")
        );
        // The Shadow and Wayfarer effects name what exists (#239).
        assert!(
            bad("effects = [{ kind = \"terrain_as\", terrains = [\"lava\", \"bog\"], like = \"floor\" }]")
                .contains("unknown terrain `bog`")
        );
        assert!(
            bad("effects = [{ kind = \"terrain_as\", terrains = [], like = \"floor\" }]")
                .contains("at least one")
        );
        assert!(
            bad("effects = [{ kind = \"lantern_light\", light = \"sun\" }]")
                .contains("unknown light preset")
        );
        // The element gifts' (#268): Blaze names what exists and keeps its numbers in range.
        let blaze = |fields: &str| bad(&format!("effects = [{{ kind = \"blaze\", {fields} }}]"));
        let ok = "damage = \"2d6\", damage_type = \"fire\", spell = \"blaze\", \
                  fire = \"arrow_fire\", burns = 60, cooldown = \"burnt_out\"";
        assert!(blaze(&format!("radius = 4, {ok}")).contains("radius is 1 to 3"));
        let fields = ok.replace("\"arrow_fire\"", "\"sun\"");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown light preset"));
        let fields = ok.replace("\"burnt_out\"", "\"smouldering\"");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown status"));
        let fields = ok.replace("\"fire\",", "\"plasma\",");
        assert!(blaze(&format!("radius = 1, {fields}")).contains("unknown damage type"));
        assert!(
            bad("effects = [{ kind = \"scent\", status = \"bleeding\", radius = 0 }]")
                .contains("radius is 1 to 20")
        );
        assert!(
            bad("effects = [{ kind = \"scent\", status = \"sniffles\", radius = 4 }]")
                .contains("unknown status")
        );
        assert!(bad("effects = [{ kind = \"forecast\", hours = 0 }]").contains("1 to 24"));
        assert!(bad("effects = [{ kind = \"ambush\", dice = \"0\" }]").contains("not 0"));
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
    fn coverage_holds_systems_and_trees_to_the_bar() {
        // Stealth is pending with no talents: fine. Un-pend it: it has none.
        let strict = SETTINGS.replace("name = \"Stealth\"\npending = true", "name = \"Stealth\"");
        let e = err(load(&strict, &[("healthy", HEALTHY), ("hardy", HARDY)]));
        assert!(e.contains("system `stealth` has 0 talents"), "{e}");
        // Hit points with one talent is under the bar of two.
        let e = err(load(
            SETTINGS,
            &[(
                "hardy",
                HARDY.replace(", talents = { healthy = 1 }", "").as_str(),
            )],
        ));
        assert!(e.contains("system `hit_points` has 1 talent but"), "{e}");
        // A tree whose only talents have prerequisites, not pending.
        let strict = SETTINGS.replace(
            "name = \"Shadow\"\ndescription = \"Stealth.\"\npending = true",
            "name = \"Shadow\"\ndescription = \"Stealth.\"",
        );
        let e = err(load(&strict, &[("healthy", HEALTHY), ("hardy", HARDY)]));
        assert!(
            e.contains("tree `shadow` has 0 talents without prerequisites"),
            "{e}"
        );
        // A class bar nobody meets yet.
        let strict = SETTINGS.replace("min_class = 0", "min_class = 1");
        let e = err(load(&strict, &[("healthy", HEALTHY), ("hardy", HARDY)]));
        assert!(e.contains("class abilities"), "{e}");
    }

    #[test]
    fn choices_chain_only_among_their_own_kind() {
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
        let spec = r#"
[[talent]]
id = "specialization"
name = "Specialization"
tree = "vigour"
description = "d"
needs = { talents = { weapon_focus = 1 } }
effects = [{ kind = "melee_damage", value = 2 }]
systems = ["hit_points"]
"#;
        let e = err(load(
            SETTINGS,
            &[("healthy", HEALTHY), ("focus", focus), ("spec", spec)],
        ));
        assert!(e.contains("same choice"), "{e}");
        let same = spec.replace(
            "description = \"d\"",
            "description = \"d\"\nchoice = \"weapon_skill\"",
        );
        let content = load(
            SETTINGS,
            &[("healthy", HEALTHY), ("focus", focus), ("spec", &same)],
        )
        .expect("loads");
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
    fn the_built_in_talents_cover_what_they_promise() {
        let content = Content::builtin().expect("content");
        let book = content.talents();
        assert!(book.len() >= 12, "{} talents", book.len());
        assert_eq!(book.coverage_gap(&|_| 0), None);
        // Every system and tree that is not pending meets the bar; the pending ones are named
        // so the gaps show in the wiki.
        assert!(
            book.systems.iter().any(|s| !s.pending),
            "some system is held to the bar"
        );
        assert!(
            book.trees.iter().any(|t| !t.pending),
            "some tree is held to the bar"
        );
    }
}
