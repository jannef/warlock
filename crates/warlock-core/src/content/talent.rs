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
    CombatStance, Content, ContentError, MAX_LEVEL, MAX_RANK, Role, WeaponSkillId, duplicate_id,
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
}

/// A system of the game that talents may touch (`[[system]]` in the settings file): the
/// coverage check asks each one that is not `pending` for its talents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDef {
    pub id: String,
    pub name: String,
    /// Its talents are still to come: counted and shown, not yet required.
    pub pending: bool,
}

/// The bar the coverage check holds every system and tree to (`[coverage]`); without the table,
/// nothing is asked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    /// The fewest talents a system may have.
    pub min_talents: u32,
    /// The fewest class abilities a system may have (none exist yet: 0 until the class tables
    /// land).
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
    /// An element: a damage type (an id in `data/damage_types.toml`).
    Element,
}

impl Choice {
    /// What is chosen, in words: "a weapon skill".
    pub fn words(self) -> &'static str {
        match self {
            Choice::WeaponSkill => "a weapon skill",
            Choice::Style => "a fighting style",
            Choice::Element => "an element",
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
            Choice::Element => (content.damage_types().iter())
                .filter(|t| !t.physical)
                .map(|t| t.id.clone())
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
            Choice::Element => {
                let kind = content.damage_type_id(id)?;
                let def = content.damage_type(kind);
                (!def.physical).then_some(def.name.as_str())
            }
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
    /// Maximum hit points: 1 per `per_levels` character levels, at least `at_least`, handed out
    /// into the actor's base hit points when the talent is taken and as each later level comes
    /// (as levels' own hit points are), so a talent taken early keeps growing.
    MaxHpPerLevels { per_levels: u8, at_least: i32 },
    /// Hit points come back this percent faster (negative: slower).
    HealingPercent { value: i32 },
    /// Footsteps this percent louder (negative: quieter).
    FootstepsPercent { value: i32 },
    /// The doors the actor opens and closes make no noise.
    SilentDoors,
    /// The actor hears sounds this much fainter (its hearing threshold, lower).
    Hearing { value: i32 },
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
    /// Its lantern burns as the light preset `light` (an id in `data/lights.toml`;
    /// Lamplighter).
    LanternLight { light: String },
    /// Its blows and shots at a creature that has not seen it do `dice` more damage, after
    /// Protection (Ambusher).
    Ambush { dice: Dice },
    /// The carrying limit, this percent more.
    CarryPercent { value: i32 },
    /// Journeys on the overworld take this percent more time (negative: quicker).
    TravelPercent { value: i32 },
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
    /// Exerting oneself (a blow, a shot, a throw) burns no extra nutrition.
    NoExertion,
    /// Experience gained, this percent more.
    XpPercent { value: i32 },
    /// The actor notices hidden things in passing as if trained in Spot, with this much added.
    Notice { value: i32 },
    /// A free blow the actor is given (spec §8 "Free blows").
    FreeBlow { trigger: FreeBlowTrigger },
    /// What attacks with some weapon skills gain, as their ranks give it
    /// ([`super::Benefit`]): the chosen weapon skill or style's (no `role`; the talent takes a
    /// choice), else every skill of `role`'s. `ranks`: its benefits read that many ranks higher
    /// (a rank held, never past [`MAX_RANK`]); `bash` only where a free bash is had already.
    Weapon {
        #[serde(default)]
        role: Option<Role>,
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
    },
    /// The readied missile weapon shoots this many cells farther.
    LauncherRange { value: i32 },
    /// Throws reach this many cells farther.
    ThrowRange { value: i32 },
    /// Worn armour's cap on Dexterity in Defence (`max_dex`), this much higher.
    MaxDex { value: i32 },
    /// Defence in the Deflection category, on top of what magic items give.
    Deflection { value: i32 },
    /// Protection, added to what is worn; only while the body armour worn has `body_armour`
    /// Protection at least (if set), and only while holding a shield (if `shield`).
    Protection {
        value: i32,
        #[serde(default)]
        body_armour: Option<i32>,
        #[serde(default)]
        shield: bool,
    },
    /// The to-hit penalty of combat stance `stance`, this percent smaller (100: none).
    StanceToHit { stance: CombatStance, percent: i32 },
    /// Fleeing steps (spec §8 "Stances") cost this percent more (negative: cheaper).
    FleePercent { value: i32 },
    /// Every hit on the actor (a blow, a shot, a throw, a trap) does this much less after its
    /// Protection soaked it, never below 1.
    Pain { value: i32 },
    /// Resistance in percent to the chosen element (the talent takes an `element` choice).
    Resist { value: i32 },
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
    /// Rotten food never makes it vomit and sickens it half as much; overeating never makes it
    /// vomit (Iron stomach).
    IronStomach,
    /// Status `status` eases this percent faster (100: twice as fast).
    StatusEases { status: String, value: i32 },
    /// Status `status` never lowers its scores.
    NoPenalty { status: String },
    /// A big hit makes it bleed no more, a critical one degree less (Clotting): the statuses
    /// that come of wounds.
    Clotting,
    /// What its statuses take off its scores, this percent more (negative: less; Unbowed).
    PenaltyPercent { value: i32 },
    /// Added to its disease checks: catching one and every progression.
    DiseaseChecks { value: i32 },
    /// A disease needs this many checks fewer won in a row to be shaken off, never fewer than 1.
    DiseaseRecovery { value: i32 },
    /// Its kicks at creatures roll one die of this many sides instead of bare hands' dice.
    KickDie { sides: u8 },
    /// Its door kicks as if this much stronger.
    DoorKick { value: i32 },
    /// Its digging does this percent more work a turn.
    DigPercent { value: i32 },
    /// Its blows with a two-handed weapon cost this percent more (negative: cheaper).
    TwoHandedPrice { value: i32 },
    /// A critical it confirms in melee leaves the target suffering `status` (one degree).
    CriticalStatus { status: String },
    /// Its blows that can do damage of kind `damage` (an id in `data/damage_types.toml`) ignore this
    /// much more Protection.
    Penetration { damage: String, value: i32 },
    /// The burden's cap on its Dexterity in Defence, this much higher.
    BurdenDex { value: i32 },
    /// A trap it steps onto first gets a Dexterity check against its difficulty: won, it does
    /// not go off under it, and it is found (Light step).
    SureStep,
    /// Pits never catch it (Light step's second rank).
    PitProof,
    /// It moves and acts this percent quicker (its speed; the number is hidden in the game).
    SpeedPercent { value: i32 },
    /// Dodge added against shots, throws, and trap missiles.
    MissileDodge { value: i32 },
    /// The damage bursts and bolts (traps, scrolls) do it, this percent more (negative: less).
    SpellDamagePercent { value: i32 },
}

/// The free blows a talent may switch on ([`crate::world::FreeBlows`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeBlowTrigger {
    Cleave,
    GreatCleave,
    Riposte,
    Opportunist,
}

impl TalentEffect {
    /// Whether the numbers are within what the rules expect, and the statuses and damage types
    /// it names are in `content` (content without statuses or damage types has nothing to check
    /// them by: they stay, and do nothing).
    fn check(&self, content: &Content, choice: Option<Choice>) -> Result<(), String> {
        let status = |id: &str| {
            (content.status_ids().next().is_none() || content.status_id(id).is_some())
                .then_some(())
                .ok_or_else(|| format!("names an unknown status `{id}`"))
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
            | TalentEffect::MeleeDamage { value }
            | TalentEffect::Hearing { value }
            | TalentEffect::Notice { value }
            | TalentEffect::LauncherRange { value }
            | TalentEffect::ThrowRange { value }
            | TalentEffect::MaxDex { value }
            | TalentEffect::Deflection { value }
            | TalentEffect::Pain { value }
            | TalentEffect::DiseaseChecks { value }
            | TalentEffect::DiseaseRecovery { value }
            | TalentEffect::DoorKick { value }
            | TalentEffect::BurdenDex { value }
            | TalentEffect::MissileDodge { value }
            | TalentEffect::Darkvision { value }
            | TalentEffect::NightSight { value } => bonus(*value),
            TalentEffect::Protection {
                value, body_armour, ..
            } => {
                bonus(*value)?;
                match body_armour {
                    Some(at) if !(1..=20).contains(at) => {
                        Err("`protection`: body_armour is 1 to 20".into())
                    }
                    _ => Ok(()),
                }
            }
            TalentEffect::StanceToHit { percent: value, .. }
            | TalentEffect::FleePercent { value } => percent(*value),
            TalentEffect::Resist { value } => match choice {
                Some(Choice::Element) => percent(*value),
                _ => Err(
                    "`resist` is for the chosen element: it needs `choice = \"element\"`".into(),
                ),
            },
            TalentEffect::Weapon {
                role,
                to_hit,
                damage,
                threat,
                ranks,
                speed,
                reload,
                ease,
                bash,
                shield,
            } => {
                if role.is_none() && !matches!(choice, Some(Choice::WeaponSkill | Choice::Style)) {
                    return Err(
                        "`weapon` without a `role` is for the chosen weapon skill or \
                                style: it needs that `choice`"
                            .into(),
                    );
                }
                let small = [to_hit, damage, shield].iter().all(|v| BONUSES.contains(v))
                    && [threat, ranks, ease].iter().all(|&&v| v <= 5);
                let percents = [speed, reload, bash].iter().all(|&&v| v <= 100);
                let any = [to_hit, damage, shield].iter().any(|&&v| v != 0)
                    || [threat, ranks, ease, speed, reload, bash]
                        .iter()
                        .any(|&&v| v != 0);
                if !small || !percents || !any {
                    return Err("`weapon`: to_hit, damage, shield are -20 to 20; threat, \
                                ranks, ease 0 to 5; speed, reload, bash 0 to 100; not all 0"
                        .into());
                }
                Ok(())
            }
            TalentEffect::HealingPercent { value }
            | TalentEffect::FootstepsPercent { value }
            | TalentEffect::CarryPercent { value }
            | TalentEffect::TravelPercent { value }
            | TalentEffect::HungerPercent { value }
            | TalentEffect::XpPercent { value }
            | TalentEffect::RestingPercent { value }
            | TalentEffect::PenaltyPercent { value }
            | TalentEffect::DigPercent { value }
            | TalentEffect::TwoHandedPrice { value }
            | TalentEffect::SpeedPercent { value }
            | TalentEffect::SpellDamagePercent { value }
            | TalentEffect::ThresholdPercent { value }
            | TalentEffect::ClimbPercent { value }
            | TalentEffect::SunHealing { value } => percent(*value),
            TalentEffect::Scent { status: id, radius } => {
                if !(1..=20).contains(radius) {
                    return Err("`scent`: radius is 1 to 20".into());
                }
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
            TalentEffect::StatusEases { status: id, value } => {
                status(id)?;
                percent(*value)
            }
            TalentEffect::NoPenalty { status: id }
            | TalentEffect::CriticalStatus { status: id } => status(id),
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
            TalentEffect::SilentDoors
            | TalentEffect::NoExertion
            | TalentEffect::FreeBlow { .. }
            | TalentEffect::IronStomach
            | TalentEffect::Clotting
            | TalentEffect::SureStep
            | TalentEffect::PitProof
            | TalentEffect::SeesUv
            | TalentEffect::PlungeProof
            | TalentEffect::LightOnIce
            | TalentEffect::WadingProof
            | TalentEffect::BeastFriend => Ok(()),
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
    /// An alignment band (spec §8 "Alignment"): `{ min = "good" }` is Good or better.
    Alignment(Requirement),
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
            Need::Alignment(required) => format!("to be {}", required.words()),
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
    /// The alignment band asked (`requires_alignment`, as quests and abilities ask it).
    pub alignment: Option<Requirement>,
}

impl Needs {
    /// Whether nothing is asked.
    pub fn is_empty(&self) -> bool {
        *self == Needs::default()
    }

    /// Every prerequisite, in the order they are shown: talents, stats, skills, weapon skills,
    /// the choice, the level, the race, the eyes, the class.
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
        let alignment = self.alignment.map(Need::Alignment);
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
            .chain(alignment)
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
    /// `min_talents` talents and `min_class` class abilities (`class_abilities` counts them by
    /// system id; none exist yet), and every tree not `pending` nor `no_free` has at least
    /// `free_per_tree` talents without prerequisites. The first gap, in words.
    pub fn coverage_gap(&self, class_abilities: &dyn Fn(&str) -> u32) -> Option<String> {
        let bar = self.coverage;
        for (i, system) in (0u8..).zip(&self.systems) {
            if system.pending {
                continue;
            }
            let talents = self.of_system(SystemId(i)).count() as u32;
            if talents < bar.min_talents {
                return Some(format!(
                    "system `{}` has {talents} talent{} but at least {} are needed (mark it \
                     `pending = true` until they land)",
                    system.id,
                    if talents == 1 { "" } else { "s" },
                    bar.min_talents
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
            if tree.pending || tree.no_free {
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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SystemSpec {
    id: String,
    name: String,
    #[serde(default)]
    pending: bool,
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
    requires_alignment: Option<Requirement>,
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
            Some(Choice::Element) => return Err("`chosen` asks a rank: an element has none".into()),
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
    if let Some(required) = spec.requires_alignment {
        required.check()?;
    }
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
        alignment: spec.requires_alignment,
    })
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
    for (i, p) in pending.into_iter().enumerate() {
        let spec = p.spec;
        let err = |m: String| ContentError::new(p.file, format!("talent `{}`: {m}", spec.id));
        if spec.name.is_empty() || spec.description.is_empty() {
            return Err(err("needs a name and a description".into()));
        }
        let tree = (book.tree_id(&spec.tree))
            .ok_or_else(|| err(format!("unknown tree `{}`", spec.tree)))?;
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
        if spec.effects.is_empty() && rank_defs.iter().all(|r| r.effects.is_empty()) {
            return Err(err("does nothing: it needs `effects`".into()));
        }
        book.list.push(TalentDef {
            id: spec.id,
            name: spec.name,
            tree,
            choice: spec.choice,
            description: spec.description,
            needs,
            effects: spec.effects,
            ranks: rank_defs,
            systems,
            granted: spec.granted,
        });
    }
    // A talent that needs one with a choice of its own kind is taken for the same choice: only
    // talents of the same choice kind may chain that way.
    for def in &book.list {
        let needed =
            (def.needs.talents.iter()).chain(def.ranks.iter().flat_map(|r| r.needs.talents.iter()));
        for &(other, _) in needed {
            let other = book.get(other);
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
    if let Some(gap) = book.coverage_gap(&|_| 0) {
        return Err(ContentError::new(first_file, format!("coverage: {gap}")));
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
        assert!(bad("effects = [{ kind = \"xp_percent\", value = 500 }]").contains("a percent"));
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
