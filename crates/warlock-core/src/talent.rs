//! Talents an actor holds (spec §8 "Talents"): the runtime side of
//! [`crate::content::TalentBook`].
//!
//! [`Talents`] is what an actor has taken, saved by talent id (like affixes: adding talents to
//! the data keeps saves) with the rank reached and the choice made, on `Actor::talents`; the
//! player's are picked at level-ups, creatures' are empty until data gives them some.
//! [`TalentMods`] is the one aggregation every rule reads: the typed effects of every rank held,
//! summed, read where the rules already read what magic items do (`Actor::defence`, `melee`,
//! `hearing`, `World::healing_rate`, `burden::load`, `noise::noise`, `step_price`, hunger,
//! experience, searching, free blows). A new effect is a variant of
//! [`TalentEffect`], a field here, and one read.
//!
//! [`Candidate`] is everything a prerequisite may ask of whoever would take a talent, and
//! [`unmet`] / [`may_take`] decide a pick, for the rules and for what the player is shown alike.

use crate::alignment::Band;
use crate::character::Character;
use crate::content::{
    Benefit, Choice, CombatStance, Content, DamageTypeId, LightId, MAX_RANK, Need, Role, Sign,
    StatusId, TalentDef, TalentEffect, TalentId, TerrainId,
};
use crate::damage::Kinds;
use crate::dice::Dice;
use crate::skills::{SkillId, Skills};
use crate::stats::{PerStat, Stat, Stats};
use crate::weapon_skills::WeaponSkills;
use crate::world::{Actor, FreeBlows};
use serde::{Deserialize, Serialize};

/// A talent an actor has taken: by id, at this rank, for this choice (a talent with a `choice`
/// is taken once per choice, each with its own rank).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taken {
    pub id: String,
    pub rank: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice: Option<String>,
}

/// The talents an actor holds, in the order taken.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Talents(Vec<Taken>);

impl Talents {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Every talent held, in the order taken.
    pub fn iter(&self) -> impl Iterator<Item = &Taken> {
        self.0.iter()
    }

    /// The rank held of talent `id` taken for `choice` (0: not held).
    pub fn rank_of(&self, id: &str, choice: Option<&str>) -> u8 {
        (self.0.iter())
            .find(|t| t.id == id && t.choice.as_deref() == choice)
            .map_or(0, |t| t.rank)
    }

    /// Whether talent `id` is held at `rank` at least, for any choice.
    pub fn has(&self, id: &str, rank: u8) -> bool {
        self.0.iter().any(|t| t.id == id && t.rank >= rank)
    }

    /// Takes talent `id` for `choice`: one rank more. Returns the rank reached.
    pub(crate) fn take(&mut self, id: &str, choice: Option<&str>) -> u8 {
        if let Some(taken) =
            (self.0.iter_mut()).find(|t| t.id == id && t.choice.as_deref() == choice)
        {
            taken.rank += 1;
            return taken.rank;
        }
        self.0.push(Taken {
            id: id.to_string(),
            rank: 1,
            choice: choice.map(str::to_string),
        });
        1
    }

    /// Replaces the granted talents held (those of the `stars` tree `content` marks `granted`;
    /// a class's abilities are left alone) with `ids`, first and in that order, each at rank 1
    /// (spec §8 "Birth sign": a sign's gifts are computed, never chosen). Picked talents keep their order after them; ids the content lacks are skipped.
    pub(crate) fn regrant(&mut self, content: &Content, ids: &[&str]) {
        let book = content.talents();
        let stars = book.tree_id("stars");
        let is_granted = |id: &str| {
            (book.id(id)).is_some_and(|t| book.get(t).granted && Some(book.get(t).tree) == stars)
        };
        self.0.retain(|t| !is_granted(&t.id));
        let given = (ids.iter()).filter(|id| is_granted(id)).map(|id| Taken {
            id: id.to_string(),
            rank: 1,
            choice: None,
        });
        self.0.splice(0..0, given);
    }

    /// What every rank held does, summed ([`TalentMods`]). Talents the content no longer has
    /// do nothing.
    pub fn mods(&self, content: &Content) -> TalentMods {
        let mut mods = TalentMods::default();
        if self.0.is_empty() {
            return mods;
        }
        let book = content.talents();
        for taken in &self.0 {
            let Some(id) = book.id(&taken.id) else {
                continue;
            };
            let def = book.get(id);
            for rank in 1..=taken.rank.min(def.rank_count()) {
                for effect in def.effects_at(rank) {
                    mods.add(content, effect, taken.choice.as_deref());
                }
            }
        }
        mods
    }
}

/// What an actor's talents do, summed: one field per effect kind, read by one rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TalentMods {
    /// Defence in the Dodge category.
    pub dodge: i32,
    /// Natural armour, on top of the actor's own.
    pub natural: i32,
    /// Natural armour as `natural`, while it wears no body armour.
    pub natural_unarmoured: i32,
    /// Added to every melee blow's damage.
    pub melee_damage: i32,
    /// How many natural points lower every attack threatens a critical.
    pub threat: i32,
    /// A fumble is rolled again, once.
    pub reroll_fumble: bool,
    /// The most to-hit a streak of blows at one creature gives (0: none; Relentless).
    pub streak: u8,
    /// Hit points by level, one entry per rank held: `(per_levels, at_least)`
    /// ([`TalentMods::hp_at`]).
    pub hp_per_levels: Vec<(u8, i32)>,
    /// What its stat gifts add to scores and potentials: handed out when a talent is taken or
    /// given, never read by a rule.
    pub stats: PerStat<i32>,
    /// Percent faster hit points come back.
    pub healing_percent: i32,
    /// Percent faster mana comes back.
    pub mana_regen_percent: i32,
    /// Percent louder footsteps (negative: quieter).
    pub footsteps_percent: i32,
    /// The doors it opens and closes make no noise.
    pub silent_doors: bool,
    /// How much fainter it hears (its hearing threshold, lower).
    pub hearing: i32,
    /// It places the footsteps it hears exactly.
    pub exact_footsteps: bool,
    /// Percent more light its eyes need (negative: less).
    pub threshold_percent: i32,
    /// Cells farther its darkvision reaches.
    pub darkvision: i32,
    /// Its eyes see ultraviolet light.
    pub sees_uv: bool,
    /// Statuses whose sufferers it sees in the dark, each within a radius: `(status, radius)`.
    pub scent: Vec<(StatusId, i32)>,
    /// Cells farther it sees on the overworld at night.
    pub night_sight: i32,
    /// Cells farther it sees on the overworld by day.
    pub day_sight: i32,
    /// The light preset its lantern burns as, if not the lantern's own.
    pub lantern_light: Option<LightId>,
    /// The light it sheds wherever it is, if any.
    pub inner_light: Option<LightId>,
    /// Damage its attacks do a creature that has not seen it, after Protection.
    pub ambush: Option<Dice>,
    /// Percent more carrying limit.
    pub carry_percent: i32,
    /// Being Burdened costs it nothing.
    pub ignore_burdened: bool,
    /// Percent more of the slowdown its burden and Bloated put on its steps (negative: less).
    pub step_slowdown_percent: i32,
    /// Kilograms more carrying limit.
    pub carry_kg: i32,
    /// The coins it carries weigh nothing.
    pub weightless_coins: bool,
    /// Percent more time an overworld journey takes (negative: quicker).
    pub travel_percent: i32,
    /// Wading costs a plain step and gives no disease.
    pub swimmer: bool,
    /// Terrain it steps onto at another's price: `(terrain, priced as)`.
    pub terrain_as: Vec<(TerrainId, TerrainId)>,
    /// A plunge through ice never hurts it.
    pub plunge_proof: bool,
    /// Ice holds it as if it carried nothing.
    pub light_on_ice: bool,
    /// Percent more climbing out onto ice costs (negative: less).
    pub climb_percent: i32,
    /// Wading never gives it a disease.
    pub wading_proof: bool,
    /// Creatures of animal factions are neutral to it until it attacks one of their type.
    pub beast_friend: bool,
    /// The hours ahead whose weather it knows (0: none).
    pub forecast: u8,
    /// Percent faster hit points come back under the open sun.
    pub sun_healing: i32,
    /// Percent faster hunger burns (negative: slower).
    pub hunger_percent: i32,
    /// Percent more sickness from rotten food (negative: less).
    pub rot_harm_percent: i32,
    /// It smells how fresh food is.
    pub smell_rot: bool,
    /// Exertion burns nothing.
    pub no_exertion: bool,
    /// A meal never leaves it Bloated.
    pub no_bloating: bool,
    /// Percent more of a fresh corpse's chances to raise its stats.
    pub corpse_stats_percent: i32,
    /// Percent more experience.
    pub xp_percent: i32,
    /// More skill rolls every level.
    pub skill_rolls: i32,
    /// Percent more often loot it finds rolls magic and rare.
    pub loot_magic_percent: i32,
    /// Percent more time reading a scroll takes (negative: less).
    pub read_percent: i32,
    /// The chance in percent a scroll it reads is not used up (the most counts).
    pub scroll_kept: u8,
    /// Its own bursts never catch it.
    pub spares_self: bool,
    /// Added to its searching, not to noticing in passing.
    pub searching: i32,
    /// Percent more damage traps do it (negative: less).
    pub trap_damage_percent: i32,
    /// Percent more experience from quests.
    pub quest_xp_percent: i32,
    /// Percent more gold from quests.
    pub quest_gold_percent: i32,
    /// Percent more favour in shops, before the cap.
    pub favour_percent: i32,
    /// How much higher the cap on its favour is, in percent.
    pub favour_cap: i32,
    /// Percent more shops pay for what they buy from it.
    pub sell_percent: i32,
    /// The gold keepers let it leave owing (the most counts).
    pub credit: u32,
    /// What it buys or sells is identified.
    pub identify_trades: bool,
    /// Item descriptions show its things' worth.
    pub item_worth: bool,
    /// Item descriptions show its gems' worth.
    pub gem_worth: bool,
    /// The hours a magic item must be carried to be identified, if it has the effect (the
    /// fewest counts).
    pub identify_carried: Option<u32>,
    /// Grades finer the loose gems it finds are cut.
    pub gem_cut: u8,
    /// Percent more often its finds are loose gems.
    pub gem_find_percent: i32,
    /// It notices hidden things in passing, with this much added; `None`: only as its skills
    /// and race allow.
    pub notice: Option<i32>,
    /// It notices hidden things on stone in passing.
    pub stone_sense: bool,
    /// Added to every check to find hidden things.
    pub search: i32,
    /// Cells more its searching reaches.
    pub search_reach: i32,
    /// How much higher some skills count, by skill.
    pub skills: Vec<(SkillId, i32)>,
    /// Anyone not hostile makes way for it.
    pub make_way: bool,
    /// No one makes way for it.
    pub distrusted: bool,
    /// The free blows its talents switch on.
    pub free_blows: FreeBlows,
    /// What attacks with some weapon skills gain ([`TalentMods::weapon_benefit`]).
    pub weapon: Vec<WeaponBonus>,
    /// Cells farther the readied missile weapon shoots.
    pub launcher_range: i32,
    /// Cells farther throws reach.
    pub throw_range: i32,
    /// Percent more often its missiles break (negative: less).
    pub missile_breaks: i32,
    /// Added to worn armour's cap on Dexterity in Defence.
    pub max_dex: i32,
    /// Defence in the Deflection category.
    pub deflection: i32,
    /// Protection, each with what it asks to be worn ([`TalentMods::protection_with`]).
    pub protection: Vec<Guard>,
    /// Percent less of a stance's to-hit penalty, by stance.
    pub stance_to_hit: Vec<(CombatStance, i32)>,
    /// Percent more a fleeing step costs (negative: cheaper).
    pub flee_percent: i32,
    /// Percent more a step out of reach of every hostile beside it costs (negative: cheaper).
    pub slip_percent: i32,
    /// Taken off every hit after the soak, never below 1.
    pub pain: i32,
    /// Damage a creature whose melee blow hits it takes (0: none).
    pub barbs: i32,
    /// Resistances in percent, by element id.
    pub resist: Vec<(String, i32)>,
    /// Percent faster resting heals, on top of resting's own pace.
    pub resting_percent: i32,
    /// The active Second wind, if it has it.
    pub second_wind: Option<SecondWind>,
    /// The birth sign's active Horn's grace, if it has it.
    pub horns_grace: Option<HornsGrace>,
    /// The birth sign's active Transfixing stare, if it has it.
    pub stare: Option<Stare>,
    /// Rotten food and overeating never make it vomit, and rotten food sickens it half as much.
    pub iron_stomach: bool,
    /// Statuses that ease faster, by the percent: `(status, percent)`, one entry per effect.
    pub status_eases: Vec<(StatusId, i32)>,
    /// Statuses that never lower its scores.
    pub no_penalty: Vec<StatusId>,
    /// Wounds bleed less: none from a big hit, a degree less from a critical.
    pub clotting: bool,
    /// Degrees more a big hit wounds it.
    pub wound_more: u16,
    /// Percent more of what its statuses take off its scores (negative: less).
    pub penalty_percent: i32,
    /// Added to its disease checks.
    pub disease_checks: i32,
    /// Added to its checks, by the stat checked.
    pub checks: PerStat<i32>,
    /// Checks fewer won in a row that shake a disease off.
    pub disease_recovery: i32,
    /// The sides of the one die its kicks at creatures roll, if any (the most counts).
    pub kick_die: Option<u8>,
    /// How much stronger its door kicks are.
    pub door_kick: i32,
    /// Percent louder its door kicks are (negative: quieter).
    pub kick_noise_percent: i32,
    /// Percent more work a turn of digging does.
    pub dig_percent: i32,
    /// Percent more its blows with a two-handed weapon cost (negative: cheaper).
    pub two_handed_price: i32,
    /// The status a critical it confirms in melee leaves on the target.
    pub critical_status: Option<StatusId>,
    /// The status a melee hit of its holds the creature with (Drowning grip), if any.
    pub grip: Option<StatusId>,
    /// Added to the roll that confirms its criticals.
    pub critical_confirm: i32,
    /// More penetration for blows that can do damage of a kind: `(kind, value)`.
    pub penetration: Vec<(DamageTypeId, i32)>,
    /// How much higher the burden's cap on its Dexterity in Defence is.
    pub burden_dex: i32,
    /// Percent off the burden's slowdown, on top of Balance.
    pub burden_ease: i32,
    /// A trap it steps onto first gets a Dexterity check.
    pub sure_step: bool,
    /// Pits never catch it.
    pub pit_proof: bool,
    /// It can disarm traps without Disable device.
    pub disarm: bool,
    /// Percent quicker it moves and acts.
    pub speed_percent: i32,
    /// Percents more its steps cost, each while what it asks holds.
    pub step_percent: Vec<StepBonus>,
    /// Burdened costs it nothing.
    pub steadfast: bool,
    /// Dodge added against shots, throws, and trap missiles.
    pub missile_dodge: i32,
    /// Percent more damage bursts and bolts do it (negative: less).
    pub spell_damage_percent: i32,
    /// Percent more marks its attacks earn in a role's weapon skills, by role (`None`: every
    /// weapon skill and style).
    pub weapon_marks: Vec<(Option<Role>, i32)>,
    /// Added to a confirmed critical's multiplier with a two-handed weapon.
    pub two_handed_crit: u8,
    /// The sides of the one die its blows with nothing in hand and its kicks roll, if any.
    pub unarmed_die: Option<u8>,
    /// Percent less of a stance's Dodge penalty, by stance.
    pub stance_dodge: Vec<(CombatStance, i32)>,
    /// Changing its combat stance costs an action.
    pub stance_costs: bool,
    /// It never catches a disease.
    pub disease_immune: bool,
    /// The stats no status lowers.
    pub no_penalty_to: Vec<Stat>,
    /// Percent more of what a status raises its scores by: `(status, percent)`.
    pub raise_percent: Vec<(StatusId, i32)>,
    /// The active Veteran's wind, if it has it.
    pub veterans_wind: Option<SecondWind>,
    /// The active Lay on hands, if it has it.
    pub lay_on_hands: Option<LayOnHands>,
    /// The birth sign's active Blaze, if it has it.
    pub blaze: Option<Blaze>,
    /// Lay on hands heals fully, cures, and its cooldown is halved.
    pub avatar: bool,
    /// The active Rage: the status it gives, if it has it.
    pub rage: Option<StatusId>,
    /// Seconds a kill keeps a rage going longer.
    pub blood_frenzy: u32,
    /// While raging, the statuses that tick nothing, and the last hit point held once; `None`
    /// without Unstoppable.
    pub unstoppable: Option<Vec<StatusId>>,
    /// The active Whirlwind.
    pub whirlwind: bool,
    /// The active Smite, if it has it.
    pub smite: Option<Smite>,
    /// The active Holy radiance: its status and the light it sheds while it lasts.
    pub radiance: Option<(StatusId, LightId)>,
    /// The active Quivering palm, if it has it.
    pub quivering_palm: Option<QuiveringPalm>,
    /// Hit points every level past `hp_level` adds beyond a level's own, if it has Hardened.
    pub hardened: Option<i32>,
    /// Percent more maximum hit points (negative: fewer).
    pub max_hp_percent: i32,
    /// Rise from the ashes, if it has it.
    pub rise: Option<Rise>,
    /// Its rising brings it back whole (Reborn whole).
    pub reborn_whole: bool,
    /// The caster's spells it has (design §4.1 B2), in the order taken.
    pub spells: Vec<crate::content::ClassSpell>,
    /// Added to every die of its spells and spell scrolls.
    pub spell_per_die: i32,
    /// Dice more its spells and spell scrolls roll.
    pub spell_dice: u32,
    /// Percent more mana its spells cost (negative: less).
    pub spell_mana_percent: i32,
    /// Deflection while it wears no body armour.
    pub deflection_unarmoured: i32,
    /// Rotten food neither sickens it nor exposes it to a disease.
    pub purify: bool,
    /// A corpse's own harm never touches it.
    pub corpse_harmless: bool,
    /// Hit points a meal heals besides: `(object, dice)`, `None` for a fresh corpse.
    pub meal_heals: Vec<(Option<crate::content::ObjectId>, Dice)>,
    /// Its diseases never progress.
    pub disease_still: bool,
    /// Damage more against a creature suffering a status: `(status, value)`.
    pub status_damage: Vec<(StatusId, i32)>,
    /// Its once-in-a-life reprieve from death, if it has one.
    pub reprieve: Option<Reprieve>,
}

/// A once-in-a-life reprieve from death (Miracle, Hag's bargain): what it is called in
/// `Progress::once`, the status it leaves, the stat it takes for good, and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reprieve {
    pub once: String,
    pub status: StatusId,
    pub stat: Option<(Stat, i32)>,
    pub message: String,
}

/// Rise from the ashes (spec §8 "Birth sign"): what rising gives back, the burst it rises in,
/// and the status it is left with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rise {
    pub hp_percent: i32,
    pub status: StatusId,
    pub burst: crate::content::Burst,
}

/// A step price change and what it asks ([`TalentEffect::StepPercent`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepBonus {
    pub percent: i32,
    pub unarmoured: bool,
    pub unburdened: bool,
    pub no_shield: bool,
}

/// The active Lay on hands (spec §8 "Class abilities").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayOnHands {
    pub per_level: u8,
    pub cooldown: StatusId,
    pub degrees: u16,
}

/// The active Smite (spec §8 "Class abilities").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Smite {
    pub per_levels: u8,
    pub sides: u8,
    pub cooldown: StatusId,
    pub degrees: u16,
}

impl Smite {
    /// The extra dice at character `level`: one die per `per_levels` levels, at least one.
    pub fn dice_at(self, level: u8) -> crate::dice::Dice {
        let count = (u32::from(level) / u32::from(self.per_levels.max(1))).max(1);
        crate::dice::Dice::of(count, u32::from(self.sides))
    }
}

/// The active Quivering palm (spec §8 "Class abilities").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuiveringPalm {
    pub status: StatusId,
    pub severity: u16,
    pub cooldown: StatusId,
    pub degrees: u16,
}

/// What a talent adds to attacks with some weapon skills: the one named `skill` (the effect's
/// own, or the choice it was taken for), else every skill of `role`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponBonus {
    pub skill: Option<String>,
    pub role: Option<Role>,
    pub benefit: Benefit,
    /// Ranks higher its benefits read.
    pub ranks: u8,
    /// Only for a skill held at this rank at least (0: any).
    pub at_rank: u8,
}

/// Protection a talent adds, while what it asks is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    pub value: i32,
    /// Only while the body armour worn has this much Protection at least.
    pub body_armour: Option<i32>,
    /// Only while holding a shield.
    pub shield: bool,
    /// Once for every armour piece worn.
    pub per_piece: bool,
}

/// The active Second wind (spec §8 "Talents"): hit points back at once, then a cooldown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecondWind {
    pub per_level: u8,
    pub at_least: i32,
    pub cooldown: StatusId,
}

/// The birth sign's active Horn's grace (spec §8 "Birth sign"): what it cures, then a cooldown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HornsGrace {
    pub cures: Vec<StatusId>,
    /// How many of the diseases that show it shakes off.
    pub diseases: u8,
    pub cooldown: StatusId,
}

/// The birth sign's active Transfixing stare (spec §8 "Birth sign"): the status it leaves on
/// the creature stared at, then a cooldown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stare {
    pub status: StatusId,
    pub severity: u16,
    pub cooldown: StatusId,
}

impl SecondWind {
    /// The hit points it gives back at character `level`.
    pub fn hp_at(self, level: u8) -> i32 {
        (i32::from(self.per_level) * i32::from(level)).max(self.at_least)
    }
}

/// The birth sign's active Blaze (spec §8 "Birth sign"): a burst of flame around the player,
/// fires on the cells it reaches, then a cooldown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blaze {
    pub radius: i32,
    pub damage: Dice,
    /// The kind of damage it does; `None` without damage types.
    pub kind: Option<DamageTypeId>,
    /// The effect presentation shows (`data/effects.toml`).
    pub spell: String,
    /// The light the fires it leaves give, and how long they burn, in seconds.
    pub fire: LightId,
    pub burns: u32,
    pub cooldown: StatusId,
}

impl TalentMods {
    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        *self == TalentMods::default()
    }

    /// Whether it has the birth sign's active `active` (spec §8 "Birth sign").
    pub fn has_sign_active(&self, active: crate::content::SignActive) -> bool {
        use crate::content::SignActive as A;
        match active {
            A::HornsGrace => self.horns_grace.is_some(),
            A::TransfixingStare => self.stare.is_some(),
            A::Blaze => self.blaze.is_some(),
        }
    }

    /// The cooldown status of the birth sign's active `active`, if it has it.
    pub fn sign_cooldown(&self, active: crate::content::SignActive) -> Option<StatusId> {
        use crate::content::SignActive as A;
        match active {
            A::HornsGrace => self.horns_grace.as_ref().map(|g| g.cooldown),
            A::TransfixingStare => self.stare.map(|s| s.cooldown),
            A::Blaze => self.blaze.as_ref().map(|b| b.cooldown),
        }
    }

    /// Whether it has the caster's spell `spell` (spec §8 "Class abilities").
    pub fn has_spell(&self, spell: crate::content::ClassSpell) -> bool {
        self.spells.contains(&spell)
    }

    /// `dice` of one of its spells (or spell scrolls), as its talents make them: `spell_dice`
    /// dice more of the same sides, and `spell_per_die` added to every die.
    pub fn spell_damage(&self, dice: Dice) -> Dice {
        if self.spell_dice == 0 && self.spell_per_die == 0 {
            return dice;
        }
        dice.empowered(self.spell_dice, self.spell_per_die)
    }

    /// The mana one of its spells that costs `mana` costs it: scaled by `spell_mana_percent`,
    /// at least 1.
    pub fn spell_mana(&self, mana: u16) -> i32 {
        let scaled = TalentMods::scaled(u64::from(mana), self.spell_mana_percent);
        i32::try_from(scaled).unwrap_or(i32::MAX).max(1)
    }

    /// Whether it has class active `active` (spec §8 "Class abilities").
    pub fn has_active(&self, active: crate::content::ClassActive) -> bool {
        use crate::content::ClassActive as A;
        match active {
            A::Rage => self.rage.is_some(),
            A::VeteransWind => self.veterans_wind.is_some(),
            A::Whirlwind => self.whirlwind,
            A::LayOnHands => self.lay_on_hands.is_some(),
            A::Smite => self.smite.is_some(),
            A::HolyRadiance => self.radiance.is_some(),
            A::QuiveringPalm => self.quivering_palm.is_some(),
        }
    }

    /// The effects of one rank, summed (what taking it adds); the statuses and damage types
    /// they name are looked up in `content`.
    pub fn of(content: &Content, effects: impl IntoIterator<Item = TalentEffect>) -> Self {
        let mut mods = TalentMods::default();
        for effect in effects {
            mods.add(content, effect, None);
        }
        mods
    }

    /// What a status's penalty of `by` points to a score comes to for it: none from a status
    /// that never lowers its scores, and a lowering scaled by `penalty_percent` toward 0 (a
    /// raise stays as it is).
    pub fn penalty(&self, status: StatusId, stat: Stat, by: i32) -> i32 {
        if by >= 0 {
            // Mighty rage: a raise grows by its percent.
            let more: i32 = (self.raise_percent.iter())
                .filter(|(s, _)| *s == status)
                .map(|(_, p)| p)
                .sum();
            return by * (100 + more).max(0) / 100;
        }
        if self.no_penalty.contains(&status) || self.no_penalty_to.contains(&stat) {
            return 0;
        }
        by * (100 + self.penalty_percent).max(0) / 100
    }

    /// The percent its steps cost more, while it wears body armour (`armoured`), is Burdened or
    /// worse (`burdened`), and holds a shield (`shield`) or not.
    pub fn step_percent(&self, armoured: bool, burdened: bool, shield: bool) -> i32 {
        (self.step_percent.iter())
            .filter(|b| !(b.unarmoured && armoured))
            .filter(|b| !(b.unburdened && burdened))
            .filter(|b| !(b.no_shield && shield))
            .map(|b| b.percent)
            .sum()
    }

    /// `dodge` (a stance's) with its penalty softened as its talents say for `stance`.
    pub fn stance_dodge(&self, stance: CombatStance, dodge: i32) -> i32 {
        let percent = (self.stance_dodge.iter())
            .filter(|(s, _)| *s == stance)
            .map(|(_, p)| *p)
            .sum::<i32>()
            .clamp(0, 100);
        if dodge >= 0 || percent == 0 {
            dodge
        } else {
            -((-dodge) * (100 - percent) / 100)
        }
    }

    /// `marks` weapon skill marks earned in a skill of `role`, more by its talents' (Weapon
    /// drill's for its role, Forge-born's for every one).
    pub fn marks(&self, role: Role, marks: u32) -> u32 {
        let percent: i32 = (self.weapon_marks.iter())
            .filter(|(r, _)| r.is_none_or(|r| r == role))
            .map(|(_, p)| p)
            .sum();
        u32::try_from(TalentMods::scaled(u64::from(marks), percent)).unwrap_or(u32::MAX)
    }

    /// How many seconds status `status` takes to ease by a degree for it, its usual `every`
    /// eased by the percents faster its talents give (at least 1).
    pub fn eases_every(&self, status: StatusId, every: u32) -> u32 {
        let faster: i32 = (self.status_eases.iter())
            .filter(|(s, _)| *s == status)
            .map(|(_, p)| p)
            .sum();
        if faster == 0 {
            return every;
        }
        let pace = u32::try_from((100 + faster).max(1)).unwrap_or(1);
        (every * 100 / pace).max(1)
    }

    /// The penetration its talents add to a blow that can do damage of `kinds`.
    pub fn penetration_for(&self, kinds: &Kinds) -> i32 {
        (self.penetration.iter())
            .filter(|(kind, _)| kinds.iter().any(|k| k == *kind))
            .map(|(_, v)| v)
            .sum()
    }

    /// Adds `effect` of a talent taken for `choice`.
    fn add(&mut self, content: &Content, effect: TalentEffect, choice: Option<&str>) {
        match effect {
            TalentEffect::Dodge { value } => self.dodge += value,
            TalentEffect::Natural { value } => self.natural += value,
            TalentEffect::NaturalUnarmoured { value } => self.natural_unarmoured += value,
            TalentEffect::MeleeDamage { value } => self.melee_damage += value,
            TalentEffect::Threat { value } => self.threat += value,
            TalentEffect::RerollFumble => self.reroll_fumble = true,
            TalentEffect::Streak { max } => self.streak = self.streak.max(max),
            TalentEffect::MaxHpPerLevels {
                per_levels,
                at_least,
            } => self.hp_per_levels.push((per_levels, at_least)),
            TalentEffect::MaxHpPercent { value } => self.max_hp_percent += value,
            TalentEffect::Stat { stat, value } => self.stats[stat] += value,
            TalentEffect::HealingPercent { value } => self.healing_percent += value,
            TalentEffect::ManaRegenPercent { value } => self.mana_regen_percent += value,
            TalentEffect::FootstepsPercent { value } => self.footsteps_percent += value,
            TalentEffect::SilentDoors => self.silent_doors = true,
            TalentEffect::Hearing { value } => self.hearing += value,
            TalentEffect::ExactFootsteps => self.exact_footsteps = true,
            TalentEffect::ThresholdPercent { value } => self.threshold_percent += value,
            TalentEffect::Darkvision { value } => self.darkvision += value,
            TalentEffect::SeesUv => self.sees_uv = true,
            TalentEffect::Scent { status, radius } => {
                if let Some(status) = content.status_id(&status) {
                    self.scent.push((status, radius));
                }
            }
            TalentEffect::NightSight { value } => self.night_sight += value,
            TalentEffect::DaySight { value } => self.day_sight += value,
            TalentEffect::LanternLight { light } => {
                self.lantern_light = content.light_id(&light).or(self.lantern_light);
            }
            TalentEffect::InnerLight { light } => {
                self.inner_light = content.light_id(&light).or(self.inner_light);
            }
            TalentEffect::Ambush { dice } => self.ambush = Some(dice),
            TalentEffect::CarryPercent { value } => self.carry_percent += value,
            TalentEffect::IgnoreBurdened => self.ignore_burdened = true,
            TalentEffect::StepSlowdownPercent { value } => self.step_slowdown_percent += value,
            TalentEffect::CarryKg { value } => self.carry_kg += value,
            TalentEffect::WeightlessCoins => self.weightless_coins = true,
            TalentEffect::TravelPercent { value } => self.travel_percent += value,
            TalentEffect::Swimmer => self.swimmer = true,
            TalentEffect::TerrainAs { terrains, like } => {
                if let Some(like) = content.terrain_id(&like) {
                    let terrains = terrains.iter().filter_map(|t| content.terrain_id(t));
                    self.terrain_as.extend(terrains.map(|t| (t, like)));
                }
            }
            TalentEffect::PlungeProof => self.plunge_proof = true,
            TalentEffect::LightOnIce => self.light_on_ice = true,
            TalentEffect::ClimbPercent { value } => self.climb_percent += value,
            TalentEffect::WadingProof => self.wading_proof = true,
            TalentEffect::BeastFriend => self.beast_friend = true,
            TalentEffect::Forecast { hours } => self.forecast = self.forecast.max(hours),
            TalentEffect::SunHealing { value } => self.sun_healing += value,
            TalentEffect::HungerPercent { value } => self.hunger_percent += value,
            TalentEffect::RotHarmPercent { value } => self.rot_harm_percent += value,
            TalentEffect::SmellRot => self.smell_rot = true,
            TalentEffect::NoExertion => self.no_exertion = true,
            TalentEffect::NoBloating => self.no_bloating = true,
            TalentEffect::CorpseStatsPercent { value } => self.corpse_stats_percent += value,
            TalentEffect::XpPercent { value } => self.xp_percent += value,
            TalentEffect::SkillRolls { value } => self.skill_rolls += value,
            // Handed out as the rank is taken (`World::take_talent`), never read after.
            TalentEffect::TrainSkill { .. } => {}
            TalentEffect::LootMagicPercent { value } => self.loot_magic_percent += value,
            TalentEffect::ReadPercent { value } => self.read_percent += value,
            TalentEffect::ScrollKept { chance } => self.scroll_kept = self.scroll_kept.max(chance),
            TalentEffect::SparesSelf => self.spares_self = true,
            TalentEffect::Searching { value } => self.searching += value,
            TalentEffect::TrapDamagePercent { value } => self.trap_damage_percent += value,
            TalentEffect::QuestXpPercent { value } => self.quest_xp_percent += value,
            TalentEffect::QuestGoldPercent { value } => self.quest_gold_percent += value,
            TalentEffect::FavourPercent { value } => self.favour_percent += value,
            TalentEffect::FavourCap { value } => self.favour_cap += value,
            TalentEffect::SellPercent { value } => self.sell_percent += value,
            TalentEffect::Credit { gold } => self.credit = self.credit.max(gold),
            TalentEffect::IdentifyTrades => self.identify_trades = true,
            TalentEffect::ItemWorth => self.item_worth = true,
            TalentEffect::GemWorth => self.gem_worth = true,
            TalentEffect::IdentifyCarried { hours } => {
                self.identify_carried = Some(self.identify_carried.map_or(hours, |h| h.min(hours)));
            }
            TalentEffect::GemCut { value } => self.gem_cut = self.gem_cut.saturating_add(value),
            TalentEffect::GemFindPercent { value } => self.gem_find_percent += value,
            TalentEffect::Notice { value } => {
                self.notice = Some(self.notice.unwrap_or(0) + value);
            }
            TalentEffect::StoneSense => self.stone_sense = true,
            TalentEffect::Search { value } => self.search += value,
            TalentEffect::SearchReach { value } => self.search_reach += value,
            TalentEffect::Skill { skill, value } => {
                if let Some(id) = content.skill_id(&skill) {
                    match self.skills.iter_mut().find(|(s, _)| *s == id) {
                        Some((_, v)) => *v += value,
                        None => self.skills.push((id, value)),
                    }
                }
            }
            TalentEffect::MakeWay => self.make_way = true,
            TalentEffect::Distrusted => self.distrusted = true,
            TalentEffect::FreeBlow { trigger } => {
                use crate::content::FreeBlowTrigger as T;
                match trigger {
                    // Two cleaves together (Cleaving strike and Cleave) cleave on and on.
                    T::Cleave if self.free_blows.cleave => self.free_blows.great_cleave = true,
                    T::Cleave => self.free_blows.cleave = true,
                    T::GreatCleave => self.free_blows.great_cleave = true,
                    T::Riposte => self.free_blows.riposte = true,
                    T::Opportunist => self.free_blows.opportunist = true,
                    T::Flurry => self.free_blows.flurry = true,
                    T::Gatekeeper => self.free_blows.gatekeeper = true,
                }
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
            } => self.weapon.push(WeaponBonus {
                skill: skill.or_else(|| {
                    role.is_none()
                        .then(|| choice.unwrap_or_default().to_string())
                }),
                role,
                benefit: Benefit {
                    to_hit,
                    damage,
                    shield,
                    threat,
                    speed,
                    reload,
                    ease_main: ease,
                    ease_off: ease,
                    bash,
                    crit,
                    ..Benefit::default()
                },
                ranks,
                at_rank,
            }),
            TalentEffect::LauncherRange { value } => self.launcher_range += value,
            TalentEffect::ThrowRange { value } => self.throw_range += value,
            TalentEffect::MissileBreaks { value } => self.missile_breaks += value,
            TalentEffect::MaxDex { value } => self.max_dex += value,
            TalentEffect::Deflection { value } => self.deflection += value,
            TalentEffect::Protection {
                value,
                body_armour,
                shield,
                per_piece,
            } => self.protection.push(Guard {
                value,
                body_armour,
                shield,
                per_piece,
            }),
            TalentEffect::StanceToHit { stance, percent } => {
                match self.stance_to_hit.iter_mut().find(|(s, _)| *s == stance) {
                    Some((_, p)) => *p += percent,
                    None => self.stance_to_hit.push((stance, percent)),
                }
            }
            TalentEffect::FleePercent { value } => self.flee_percent += value,
            TalentEffect::SlipPercent { value } => self.slip_percent += value,
            TalentEffect::Pain { value } => self.pain += value,
            TalentEffect::Barbs { damage } => self.barbs += damage,
            TalentEffect::Resist { value, damage } => {
                let element = damage.as_deref().or(choice).unwrap_or_default();
                match self.resist.iter_mut().find(|(e, _)| e == element) {
                    Some((_, r)) => *r += value,
                    None => self.resist.push((element.to_string(), value)),
                }
            }
            TalentEffect::RestingPercent { value } => self.resting_percent += value,
            TalentEffect::SecondWind {
                per_level,
                at_least,
                cooldown,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    self.second_wind = Some(SecondWind {
                        per_level,
                        at_least,
                        cooldown,
                    });
                }
            }
            TalentEffect::IronStomach => self.iron_stomach = true,
            TalentEffect::StatusEases { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.status_eases.push((status, value));
                }
            }
            TalentEffect::NoPenalty { status } => {
                self.no_penalty.extend(content.status_id(&status));
            }
            TalentEffect::Clotting => self.clotting = true,
            TalentEffect::WoundMore { value } => self.wound_more += value,
            TalentEffect::PenaltyPercent { value } => self.penalty_percent += value,
            TalentEffect::DiseaseChecks { value } => self.disease_checks += value,
            TalentEffect::Checks { stat, value } => self.checks[stat] += value,
            TalentEffect::DiseaseRecovery { value } => self.disease_recovery += value,
            TalentEffect::KickDie { sides } => {
                self.kick_die = Some(self.kick_die.map_or(sides, |s| s.max(sides)));
            }
            TalentEffect::DoorKick { value } => self.door_kick += value,
            TalentEffect::KickNoisePercent { value } => self.kick_noise_percent += value,
            TalentEffect::DigPercent { value } => self.dig_percent += value,
            TalentEffect::TwoHandedPrice { value } => self.two_handed_price += value,
            TalentEffect::Grip { status } => {
                self.grip = content.status_id(&status).or(self.grip);
            }
            TalentEffect::CriticalStatus { status } => {
                self.critical_status = content.status_id(&status).or(self.critical_status);
            }
            TalentEffect::CriticalConfirm { value } => self.critical_confirm += value,
            TalentEffect::Penetration {
                damage: kind,
                value,
            } => {
                if let Some(kind) = content.damage_type_id(&kind) {
                    self.penetration.push((kind, value));
                }
            }
            TalentEffect::BurdenDex { value } => self.burden_dex += value,
            TalentEffect::BurdenEase { value } => self.burden_ease += value,
            TalentEffect::SureStep => self.sure_step = true,
            TalentEffect::PitProof => self.pit_proof = true,
            TalentEffect::Disarm => self.disarm = true,
            TalentEffect::SpeedPercent { value } => self.speed_percent += value,
            TalentEffect::MissileDodge { value } => self.missile_dodge += value,
            TalentEffect::SpellDamagePercent { value } => self.spell_damage_percent += value,
            TalentEffect::Spell { spell } => {
                if !self.spells.contains(&spell) {
                    self.spells.push(spell);
                }
            }
            TalentEffect::SpellPower { per_die, dice } => {
                self.spell_per_die += per_die;
                self.spell_dice += dice;
            }
            TalentEffect::SpellManaPercent { value } => self.spell_mana_percent += value,
            TalentEffect::DeflectionUnarmoured { value } => self.deflection_unarmoured += value,
            TalentEffect::Purify => self.purify = true,
            TalentEffect::CorpseHarmless => self.corpse_harmless = true,
            TalentEffect::MealHeals { object, dice } => match object {
                None => self.meal_heals.push((None, dice)),
                Some(name) => {
                    if let Some(id) = content.object_id(&name) {
                        self.meal_heals.push((Some(id), dice));
                    }
                }
            },
            TalentEffect::DiseaseStill => self.disease_still = true,
            TalentEffect::StatusDamage { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.status_damage.push((status, value));
                }
            }
            TalentEffect::Reprieve {
                once,
                status,
                stat,
                by,
                message,
            } => {
                if let Some(status) = content.status_id(&status) {
                    self.reprieve = Some(Reprieve {
                        once,
                        status,
                        stat: stat.map(|s| (s, by)),
                        message,
                    });
                }
            }
            TalentEffect::WeaponMarks { role, value } => self.weapon_marks.push((role, value)),
            TalentEffect::TwoHandedCrit { value } => {
                self.two_handed_crit = self.two_handed_crit.saturating_add(value);
            }
            TalentEffect::UnarmedDie { sides } => {
                self.unarmed_die = Some(self.unarmed_die.map_or(sides, |s| s.max(sides)));
            }
            TalentEffect::StanceCosts => self.stance_costs = true,
            TalentEffect::RiseFromAshes {
                hp_percent,
                status,
                spell,
                radius,
                damage,
                damage_type,
            } => {
                if let Some(status) = content.status_id(&status) {
                    let burst = crate::content::Burst {
                        spell,
                        radius,
                        damage,
                        kind: content.damage_type_id(&damage_type),
                        blast: 0,
                        ice: Some(crate::content::BurstIce::Melts),
                    };
                    self.rise = Some(Rise {
                        hp_percent,
                        status,
                        burst,
                    });
                }
            }
            TalentEffect::RebornWhole => self.reborn_whole = true,
            TalentEffect::StanceDodge { stance, percent } => {
                match self.stance_dodge.iter_mut().find(|(s, _)| *s == stance) {
                    Some((_, p)) => *p += percent,
                    None => self.stance_dodge.push((stance, percent)),
                }
            }
            TalentEffect::StepPercent {
                value,
                unarmoured,
                unburdened,
                no_shield,
            } => self.step_percent.push(StepBonus {
                percent: value,
                unarmoured,
                unburdened,
                no_shield,
            }),
            TalentEffect::Steadfast => self.steadfast = true,
            TalentEffect::DiseaseImmune => self.disease_immune = true,
            TalentEffect::NoPenaltyTo { stat } => self.no_penalty_to.push(stat),
            TalentEffect::RaisePercent { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.raise_percent.push((status, value));
                }
            }
            TalentEffect::VeteransWind {
                per_level,
                at_least,
                cooldown,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    self.veterans_wind = Some(SecondWind {
                        per_level,
                        at_least,
                        cooldown,
                    });
                }
            }
            TalentEffect::LayOnHands {
                per_level,
                cooldown,
                degrees,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    self.lay_on_hands = Some(LayOnHands {
                        per_level,
                        cooldown,
                        degrees,
                    });
                }
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
                if let (Some(fire), Some(cooldown)) =
                    (content.light_id(&fire), content.status_id(&cooldown))
                {
                    self.blaze = Some(Blaze {
                        radius,
                        damage,
                        kind: content.damage_type_id(&damage_type),
                        spell,
                        fire,
                        burns,
                        cooldown,
                    });
                }
            }
            TalentEffect::Avatar => self.avatar = true,
            TalentEffect::HornsGrace {
                cures,
                diseases,
                cooldown,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    let cures = cures.iter().filter_map(|s| content.status_id(s)).collect();
                    self.horns_grace = Some(HornsGrace {
                        cures,
                        diseases,
                        cooldown,
                    });
                }
            }
            TalentEffect::Stare {
                status,
                severity,
                cooldown,
            } => {
                if let (Some(status), Some(cooldown)) =
                    (content.status_id(&status), content.status_id(&cooldown))
                {
                    self.stare = Some(Stare {
                        status,
                        severity,
                        cooldown,
                    });
                }
            }
            TalentEffect::Rage { status } => {
                self.rage = content.status_id(&status).or(self.rage);
            }
            TalentEffect::BloodFrenzy { seconds } => self.blood_frenzy += seconds,
            TalentEffect::Unstoppable { statuses } => {
                let ids = statuses.iter().filter_map(|s| content.status_id(s));
                self.unstoppable.get_or_insert_with(Vec::new).extend(ids);
            }
            TalentEffect::Whirlwind => self.whirlwind = true,
            TalentEffect::Smite {
                per_levels,
                sides,
                cooldown,
                degrees,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    self.smite = Some(Smite {
                        per_levels,
                        sides,
                        cooldown,
                        degrees,
                    });
                }
            }
            TalentEffect::Radiance { status, light } => {
                if let (Some(status), Some(light)) =
                    (content.status_id(&status), content.light_id(&light))
                {
                    self.radiance = Some((status, light));
                }
            }
            TalentEffect::QuiveringPalm {
                status,
                severity,
                cooldown,
                degrees,
            } => {
                if let (Some(status), Some(cooldown)) =
                    (content.status_id(&status), content.status_id(&cooldown))
                {
                    self.quivering_palm = Some(QuiveringPalm {
                        status,
                        severity,
                        cooldown,
                        degrees,
                    });
                }
            }
            TalentEffect::Hardened { bonus } => {
                self.hardened = Some(self.hardened.unwrap_or(0) + bonus);
            }
        }
    }

    /// What attacks with weapon skill `id` of `role` gain beyond its ranks, and how many ranks
    /// higher its benefits read.
    pub fn weapon_benefit(&self, id: &str, role: Role, rank: u8) -> (Benefit, u8) {
        let mut benefit = Benefit::default();
        let mut ranks = 0u8;
        for bonus in &self.weapon {
            let applies = match &bonus.skill {
                Some(skill) => skill == id,
                None => bonus.role == Some(role),
            } && rank >= bonus.at_rank;
            if applies {
                benefit = benefit.plus(bonus.benefit);
                ranks = ranks.saturating_add(bonus.ranks);
            }
        }
        (benefit, ranks)
    }

    /// `rank` (held) raised by `ranks`, never past [`MAX_RANK`]; an untrained skill stays 0.
    pub fn raised(rank: u8, ranks: u8) -> u8 {
        if rank == 0 {
            0
        } else {
            rank.saturating_add(ranks).min(MAX_RANK)
        }
    }

    /// The Protection its talents add while it wears body armour of `body` Protection (none:
    /// no body armour), holds a shield or not, and wears `pieces` pieces of armour.
    pub fn protection_with(&self, body: Option<i32>, shield: bool, pieces: i32) -> i32 {
        (self.protection.iter())
            .filter(|g| g.body_armour.is_none_or(|at| body.is_some_and(|b| b >= at)))
            .filter(|g| !g.shield || shield)
            .map(|g| {
                if g.per_piece {
                    g.value * pieces
                } else {
                    g.value
                }
            })
            .sum()
    }

    /// What its talents add to a check of `stat`.
    pub fn check(&self, stat: Stat) -> i32 {
        self.checks[stat]
    }

    /// `to_hit` (a stance's) with its penalty softened as its talents say for `stance`.
    pub fn stance_to_hit(&self, stance: CombatStance, to_hit: i32) -> i32 {
        let percent = (self.stance_to_hit.iter())
            .filter(|(s, _)| *s == stance)
            .map(|(_, p)| *p)
            .sum::<i32>()
            .clamp(0, 100);
        if to_hit >= 0 || percent == 0 {
            to_hit
        } else {
            // Rounded toward 0: half of −3 is −1.
            -((-to_hit) * (100 - percent) / 100)
        }
    }

    /// The hit points the per-level effects give at character `level`: for each rank held,
    /// one per `per_levels` levels, at least `at_least`.
    pub fn hp_at(&self, level: u8) -> i32 {
        (self.hp_per_levels.iter())
            .map(|&(per, least)| (i32::from(level) / i32::from(per.max(1))).max(least))
            .sum()
    }

    /// `value` scaled by a signed percent, rounded down, never below 0: the limit at
    /// `carry_percent`, a gain at `xp_percent`.
    pub fn scaled(value: u64, percent: i32) -> u64 {
        let factor = u64::try_from((100 + percent).max(0)).unwrap_or(0);
        value.saturating_mul(factor) / 100
    }
}

/// Everything a prerequisite may ask of whoever would take a talent (spec §8 "Talents"): the
/// talents held, the stored scores (never the effective ones), the trained skills, the weapon
/// skills, the character level, and the character's race, class, and eyes (the race's vision
/// profile), by id.
#[derive(Clone, Copy, Debug)]
pub struct Candidate<'a> {
    pub talents: &'a Talents,
    pub stats: &'a Stats,
    pub skills: &'a Skills,
    pub weapon_skills: &'a WeaponSkills,
    pub level: u8,
    pub race: Option<&'a str>,
    pub class: Option<&'a str>,
    pub eyes: Option<&'a str>,
    /// The alignment band (spec §8 "Alignment"); `None` without alignment rules, when no
    /// talent that asks for one can be taken.
    pub band: Option<Band>,
    /// The birth sign (spec §8 "Birth sign"), if any, and whether it is a lucky day's.
    pub sign: Option<&'a Sign>,
    pub lucky: bool,
}

impl<'a> Candidate<'a> {
    /// The player as prerequisites see them: `actor` with their `character` (none for a
    /// scenario without one: no race, class, or eyes) at character `level`.
    pub fn player(
        content: &'a Content,
        actor: &'a Actor,
        character: Option<&'a Character>,
        level: u8,
    ) -> Self {
        let spec = character.map(|c| &c.spec);
        let race = spec.and_then(|s| content.race(&s.race));
        Candidate {
            talents: &actor.talents,
            stats: &actor.stats,
            skills: &actor.skills,
            weapon_skills: &actor.weapon_skills,
            level,
            race: spec.map(|s| s.race.as_str()),
            class: spec.map(|s| s.class.as_str()),
            eyes: race.map(|r| r.vision.as_str()),
            band: content.alignment().map(|_| actor.band(content)),
            sign: character.and_then(|c| c.sign.as_ref()),
            lucky: character.is_some_and(|c| c.lucky(content).is_some()),
        }
    }

    /// Whether `need` holds, for a talent of `choice` kind taken for `chosen`.
    pub fn meets(
        &self,
        content: &Content,
        need: &Need,
        choice: Option<Choice>,
        chosen: Option<&str>,
    ) -> bool {
        match need {
            Need::Talent(id, rank) => {
                let other = content.talents().get(*id);
                // A chain of talents with a choice runs through the same choice.
                if other.choice.is_some() && other.choice == choice {
                    self.talents.rank_of(&other.id, chosen) >= *rank
                } else {
                    self.talents.has(&other.id, *rank)
                }
            }
            Need::Stat(stat, score) => self.stats.score(*stat) >= *score,
            Need::Skill(id, value) => self.skills.value(*id).is_some_and(|v| v >= *value),
            Need::WeaponSkill(id, rank) => self.weapon_skills.rank(*id) >= *rank,
            Need::AnyWeaponSkill(any) => {
                (any.skills.iter()).any(|id| self.weapon_skills.rank(*id) >= any.rank)
            }
            Need::Chosen(rank) => match (choice, chosen) {
                (Some(Choice::WeaponSkill | Choice::Style), Some(id)) => {
                    (content.weapon_skills().id(id))
                        .is_some_and(|w| self.weapon_skills.rank(w) >= *rank)
                }
                _ => false,
            },
            Need::Level(level) => self.level >= *level,
            Need::Race(id) => self.race == Some(id.as_str()),
            Need::Eyes(id) => self.eyes == Some(id.as_str()),
            Need::Class(id) => self.class == Some(id.as_str()),
            Need::Animal(id) => self.sign.is_some_and(|s| s.is_animal(id)),
            Need::Element(id) => self.sign.is_some_and(|s| s.is_element(id)),
            Need::Lucky => self.lucky,
            Need::Alignment(required) => self.band.is_some_and(|band| required.allows(band)),
            Need::Either(needs) => (needs.iter()).any(|n| self.meets(content, n, choice, chosen)),
        }
    }
}

/// Why a talent cannot be taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// It is given, never picked.
    Granted,
    /// It takes a choice and none was made.
    NeedsChoice(Choice),
    /// It takes no choice and one was made.
    TakesNoChoice,
    /// The choice names nothing of its kind.
    BadChoice(Choice, String),
    /// Every rank is held already.
    AtTop,
    /// These prerequisites do not hold.
    Unmet(Vec<Need>),
}

impl Refusal {
    /// The refusal in words, for a talent `name`: "You cannot take Cleave: you need Strength
    /// 13."
    pub fn words(&self, content: &Content, name: &str) -> String {
        match self {
            Refusal::Granted => format!("{name} is given, not chosen."),
            Refusal::NeedsChoice(kind) => format!("{name} needs a choice: {}.", kind.words()),
            Refusal::TakesNoChoice => format!("{name} takes no choice."),
            Refusal::BadChoice(kind, id) => format!("`{id}` is not {}.", kind.words()),
            Refusal::AtTop => format!("You already have all of {name}."),
            Refusal::Unmet(needs) => {
                let words: Vec<String> = needs.iter().map(|n| n.words(content)).collect();
                format!("You cannot take {name}: you need {}.", list(&words))
            }
        }
    }
}

/// `a`, `a and b`, `a, b, and c`.
fn list(words: &[String]) -> String {
    list_with(words, "and")
}

/// `a`, `a or b`, `a, b, or c` with `joint` "or".
pub(crate) fn list_with(words: &[String], joint: &str) -> String {
    match words {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} {joint} {b}"),
        [rest @ .., last] => format!("{}, {joint} {last}", rest.join(", ")),
    }
}

/// The prerequisites of taking `rank` (1 first) of `def` for `chosen` that `who` does not
/// meet, in the order they are shown.
pub fn unmet(
    content: &Content,
    who: &Candidate<'_>,
    def: &TalentDef,
    rank: u8,
    chosen: Option<&str>,
) -> Vec<Need> {
    (def.needs_at(rank))
        .filter(|need| !who.meets(content, need, def.choice, chosen))
        .collect()
}

/// Whether `who` may take `def` for `chosen` now: the rank it would reach, or why not.
pub fn may_take(
    content: &Content,
    who: &Candidate<'_>,
    def: &TalentDef,
    chosen: Option<&str>,
) -> Result<u8, Refusal> {
    if def.granted {
        return Err(Refusal::Granted);
    }
    match (def.choice, chosen) {
        (Some(kind), None) => return Err(Refusal::NeedsChoice(kind)),
        (None, Some(_)) => return Err(Refusal::TakesNoChoice),
        (Some(kind), Some(id)) if kind.name(content, id).is_none() => {
            return Err(Refusal::BadChoice(kind, id.to_string()));
        }
        _ => {}
    }
    let rank = who.talents.rank_of(&def.id, chosen) + 1;
    if rank > def.rank_count() {
        return Err(Refusal::AtTop);
    }
    let missing = unmet(content, who, def, rank, chosen);
    if missing.is_empty() {
        Ok(rank)
    } else {
        Err(Refusal::Unmet(missing))
    }
}

/// The choices `who` may take `def` for now: `[None]` for a talent without a choice that may
/// be taken, the valid choice ids for one with a choice, nothing when it cannot be taken.
pub fn openings(content: &Content, who: &Candidate<'_>, def: &TalentDef) -> Vec<Option<String>> {
    match def.choice {
        None => may_take(content, who, def, None)
            .is_ok()
            .then_some(None)
            .into_iter()
            .collect(),
        Some(kind) => (kind.options(content).into_iter())
            .filter(|id| may_take(content, who, def, Some(id)).is_ok())
            .map(Some)
            .collect(),
    }
}

/// Every talent `who` may take now, with the choices it may be taken for, tree by tree in
/// display order (the pick list).
pub fn offers(content: &Content, who: &Candidate<'_>) -> Vec<(TalentId, Vec<Option<String>>)> {
    (content.talents().by_tree())
        .filter_map(|(id, def)| {
            let ways = openings(content, who, def);
            (!ways.is_empty()).then_some((id, ways))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_points_by_level_grow_with_the_level_and_never_under_the_floor() {
        let mods = TalentMods::of(
            &Content::builtin().expect("content"),
            [TalentEffect::MaxHpPerLevels {
                per_levels: 4,
                at_least: 3,
            }],
        );
        assert_eq!(mods.hp_at(1), 3);
        assert_eq!(mods.hp_at(12), 3);
        assert_eq!(mods.hp_at(16), 4);
        assert_eq!(mods.hp_at(60), 15);
        let two = TalentMods::of(
            &Content::builtin().expect("content"),
            [
                TalentEffect::MaxHpPerLevels {
                    per_levels: 4,
                    at_least: 3,
                },
                TalentEffect::MaxHpPerLevels {
                    per_levels: 4,
                    at_least: 3,
                },
            ],
        );
        assert_eq!(two.hp_at(20), 10);
    }

    #[test]
    fn effects_sum_and_flags_switch() {
        let mods = TalentMods::of(
            &Content::builtin().expect("content"),
            [
                TalentEffect::Dodge { value: 1 },
                TalentEffect::Dodge { value: 2 },
                TalentEffect::FootstepsPercent { value: -20 },
                TalentEffect::SilentDoors,
                TalentEffect::Notice { value: 2 },
                TalentEffect::FreeBlow {
                    trigger: crate::content::FreeBlowTrigger::Riposte,
                },
            ],
        );
        assert_eq!(mods.dodge, 3);
        assert_eq!(mods.footsteps_percent, -20);
        assert!(mods.silent_doors);
        assert_eq!(mods.notice, Some(2));
        assert!(mods.free_blows.riposte && !mods.free_blows.cleave);
        assert!(!mods.is_empty());
        assert!(TalentMods::default().is_empty());
        assert_eq!(TalentMods::scaled(300, 10), 330);
        assert_eq!(TalentMods::scaled(300, -15), 255);
        assert_eq!(TalentMods::scaled(300, -150), 0);
    }

    #[test]
    fn taking_counts_ranks_per_choice() {
        let mut talents = Talents::default();
        assert_eq!(talents.take("hardy", None), 1);
        assert_eq!(talents.take("hardy", None), 2);
        assert_eq!(talents.take("weapon_focus", Some("bows")), 1);
        assert_eq!(talents.take("weapon_focus", Some("slings")), 1);
        assert_eq!(talents.rank_of("hardy", None), 2);
        assert_eq!(talents.rank_of("weapon_focus", Some("bows")), 1);
        assert_eq!(talents.rank_of("weapon_focus", None), 0);
        assert!(talents.has("weapon_focus", 1));
        assert!(!talents.has("hardy", 3));
        assert_eq!(talents.len(), 3);
        let json = serde_json::to_string(&talents).expect("json");
        assert_eq!(
            json,
            r#"[{"id":"hardy","rank":2},{"id":"weapon_focus","rank":1,"choice":"bows"},{"id":"weapon_focus","rank":1,"choice":"slings"}]"#
        );
        let back: Talents = serde_json::from_str(&json).expect("back");
        assert_eq!(back, talents);
    }

    #[test]
    fn lists_read_naturally() {
        let w = |s: &[&str]| list(&s.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(w(&["Strength 13"]), "Strength 13");
        assert_eq!(w(&["Strength 13", "Dodge"]), "Strength 13 and Dodge");
        assert_eq!(w(&["a", "b", "c"]), "a, b, and c");
    }
}
