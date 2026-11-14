//! The offence talents (#1495): what a talent does to the holder's attacks: damage, criticals,
//! weapon skills and their actives, free blows, kicks, and foes it is sworn against.
//! [`OffenceEffect`] is the data (a talent's `[[effect]]` of one of [`OffenceEffect::KINDS`]),
//! [`OffenceMods`] the sum every rule reads (`TalentMods::offence`). A new effect of the area is a
//! variant here, its field and its arm in [`OffenceMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/offence.rs`, and its one read.

use super::TalentMods;
use crate::content::{Benefit, Content, DamageTypeId, FreeBlowTrigger, Role, StatusId};
use crate::damage::Kinds;
use crate::world::FreeBlows;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OffenceEffect {
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
    /// An initiative check it makes against a hostile attacker beside it is answered with a
    /// free blow at once (Hair trigger; `Cue::Startled`, `world/free_blow.rs`).
    StartledBlow,
    /// A free blow the actor is given (spec §8 "Free blows"); `one_in` (the gatekeeper's only)
    /// makes it a chance: one in that many, scaled by the stance's `gatekeeper` percent.
    FreeBlow {
        trigger: FreeBlowTrigger,
        #[serde(default)]
        one_in: Option<u32>,
    },
    /// What attacks with some weapon skills gain, as their ranks give it
    /// ([`crate::content::Benefit`]): the weapon skill or style `skill` names, else the chosen
    /// one's (no `role`; the talent takes a choice), else every skill of `role`'s. `ranks`: its
    /// benefits read that many ranks higher (a rank held, never past [`crate::content::MAX_RANK`]);
    /// `bash` only where a free bash is had already.
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
    /// Its kicks at creatures roll one die of this many sides instead of bare hands' dice.
    KickDie { sides: u8 },
    /// A kick of its that hits drives the foe a cell back, when the cell is free (Breaker;
    /// `World::kick_back`).
    KickBack,
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
    /// Its blows that can do damage of kind `damage` (an id in `data/damage_types.toml`) ignore
    /// this much more Protection.
    Penetration { damage: String, value: i32 },
    /// Its blows that hit a creature suffering `status` do `value` more damage (Poison lore).
    StatusDamage { status: String, value: i32 },
    /// Against the creature type it has killed most (ties: the first to get there), its attacks
    /// are this much likelier to hit and do this much more damage (Favoured prey).
    FavouredPrey { to_hit: i32, damage: i32 },
    /// Against creature types that carry the tag it was taken for (a `choice = "tag"`), its
    /// attacks are this much likelier to hit and do this much more damage (Sworn foe).
    SwornFoe { to_hit: i32, damage: i32 },
}

impl OffenceEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`OffenceEffect`]).
    pub const KINDS: [&'static str; 31] = [
        "ability_ranks",
        "attack_price_percent",
        "best_skill_threat",
        "brace",
        "critical_confirm",
        "critical_penetration",
        "critical_status",
        "favoured_prey",
        "ferocity",
        "free_blow",
        "grip",
        "kick_back",
        "kick_die",
        "launcher_range",
        "melee_damage",
        "off_hand_damage",
        "penetration",
        "reroll_fumble",
        "startled_blow",
        "status_damage",
        "streak",
        "sworn_foe",
        "threat",
        "throw_range",
        "two_handed_crit",
        "two_handed_price",
        "unarmed_die",
        "weapon",
        "weapon_energy_percent",
        "weapon_marks",
        "weapon_stamina",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OffenceMods {
    /// Added to every melee blow's damage.
    pub melee_damage: i32,
    /// Added to every attack's damage below a third of its hit points (Ferocity).
    pub ferocity: i32,
    /// How many natural points lower every attack threatens a critical.
    pub threat: i32,
    /// A fumble is rolled again, once.
    pub reroll_fumble: bool,
    /// The most to-hit a streak of blows at one creature gives (0: none; Relentless).
    pub streak: u8,
    /// Every this many blows before it in a streak add +1 damage (0: none; Relentless fury).
    pub streak_damage_every: u8,
    /// Added to its off hand's blows' damage (Tentacles).
    pub off_hand_damage: i32,
    /// How many natural points lower its attacks with its best weapon skill threaten a
    /// critical (Tempered).
    pub best_skill_threat: i32,
    /// An initiative check it makes against a hostile attacker beside it strikes back (Hair
    /// trigger).
    pub startled_blow: bool,
    /// The free blows its talents switch on.
    pub free_blows: FreeBlows,
    /// The gatekeeper's blow by chance, one in this many (Gatekeeper; the stance scales it,
    /// `free_blow.rs::gatekeeper_odds`): the likeliest of its talents'.
    pub gatekeeper_one_in: Option<u32>,
    /// What attacks with some weapon skills gain ([`OffenceMods::weapon_benefit`]).
    pub weapon: Vec<WeaponBonus>,
    /// Cells farther the readied missile weapon shoots.
    pub launcher_range: i32,
    /// Cells farther a launcher of a weapon skill shoots, by the skill's id.
    pub launcher_range_of: Vec<(String, i32)>,
    /// Cells farther throws reach.
    pub throw_range: i32,
    /// The sides of the one die its kicks at creatures roll, if any (the most counts).
    pub kick_die: Option<u8>,
    /// A kick of its that hits drives the foe a cell back, when the cell is free (Breaker).
    pub kick_back: bool,
    /// Percent more its blows with a two-handed weapon cost (negative: cheaper).
    pub two_handed_price: i32,
    /// Percent more its melee attacks cost (negative: cheaper; Blood frenzy while raging).
    pub attack_price_percent: i32,
    /// The status a critical it confirms in melee leaves on the target.
    pub critical_status: Option<StatusId>,
    /// The status a melee hit of its holds the creature with (Drowning grip), if any.
    pub grip: Option<StatusId>,
    /// Added to the roll that confirms its criticals.
    pub critical_confirm: i32,
    /// Ranks more its weapon abilities' thresholds read (Early mastery).
    pub ability_ranks: i32,
    /// Percent more time its weapon actives take (Practised technique: -25).
    pub weapon_energy_percent: i32,
    /// Stamina points less every weapon active of its costs (Drillmaster).
    pub weapon_stamina: i32,
    /// Stamina points less those weapon actives cost (Armour-breaker's Sunder and Gap shot).
    pub ability_stamina: Vec<(crate::content::WeaponAbilityId, i32)>,
    /// What its Brace's blow gains (Brace of steel): to-hit, and a status on a foe it hits.
    pub brace: Option<(i32, StatusId)>,
    /// The Protection its confirmed criticals ignore, on top of the attack's penetration.
    pub critical_penetration: i32,
    /// More penetration for blows that can do damage of a kind: `(kind, value)`.
    pub penetration: Vec<(DamageTypeId, i32)>,
    /// Percent more marks its attacks earn in a role's weapon skills, by role (`None`: every
    /// weapon skill and style).
    pub weapon_marks: Vec<(Option<Role>, i32)>,
    /// Added to a confirmed critical's multiplier with a two-handed weapon.
    pub two_handed_crit: u8,
    /// The sides of the one die its blows with nothing in hand and its kicks roll, if any.
    pub unarmed_die: Option<u8>,
    /// Damage more against a creature suffering a status: `(status, value)`.
    pub status_damage: Vec<(StatusId, i32)>,
    /// To hit and damage against its favoured prey (Favoured prey, Master hunter).
    pub favoured_prey: (i32, i32),
    /// To hit and damage against the creature types carrying each tag, by tag (Sworn foe).
    pub sworn_foe: Vec<(String, i32, i32)>,
}

impl OffenceMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: OffenceEffect, choice: Option<&str>) {
        match effect {
            OffenceEffect::MeleeDamage { value } => self.melee_damage += value,
            OffenceEffect::Ferocity { value } => self.ferocity += value,
            OffenceEffect::Threat { value } => self.threat += value,
            OffenceEffect::RerollFumble => self.reroll_fumble = true,
            OffenceEffect::Streak { max, damage_every } => {
                self.streak = self.streak.max(max);
                if damage_every > 0 {
                    let every = match self.streak_damage_every {
                        0 => damage_every,
                        had => had.min(damage_every),
                    };
                    self.streak_damage_every = every;
                }
            }
            OffenceEffect::OffHandDamage { value } => self.off_hand_damage += value,
            OffenceEffect::BestSkillThreat { value } => self.best_skill_threat += value,
            OffenceEffect::StartledBlow => self.startled_blow = true,
            OffenceEffect::FreeBlow { trigger, one_in } => {
                use crate::content::FreeBlowTrigger as T;
                match trigger {
                    // Two cleaves together (Cleaving strike and Cleave) cleave on and on.
                    T::Cleave if self.free_blows.cleave => self.free_blows.great_cleave = true,
                    T::Cleave => self.free_blows.cleave = true,
                    T::GreatCleave => self.free_blows.great_cleave = true,
                    T::Riposte => self.free_blows.riposte = true,
                    T::Opportunist => self.free_blows.opportunist = true,
                    T::Flurry => self.free_blows.flurry = true,
                    T::Gatekeeper => match one_in {
                        Some(n) => {
                            let least = self.gatekeeper_one_in.map_or(n, |m| m.min(n));
                            self.gatekeeper_one_in = Some(least);
                        }
                        None => self.free_blows.gatekeeper = true,
                    },
                }
            }
            OffenceEffect::Weapon {
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
            OffenceEffect::LauncherRange { value, skill: None } => self.launcher_range += value,
            OffenceEffect::LauncherRange {
                value,
                skill: Some(skill),
            } => self.launcher_range_of.push((skill, value)),
            OffenceEffect::ThrowRange { value } => self.throw_range += value,
            OffenceEffect::KickDie { sides } => {
                self.kick_die = Some(self.kick_die.map_or(sides, |s| s.max(sides)));
            }
            OffenceEffect::KickBack => self.kick_back = true,
            OffenceEffect::TwoHandedPrice { value } => self.two_handed_price += value,
            OffenceEffect::AttackPricePercent { value } => self.attack_price_percent += value,
            OffenceEffect::Grip { status } => {
                self.grip = content.status_id(&status).or(self.grip);
            }
            OffenceEffect::CriticalStatus { status } => {
                self.critical_status = content.status_id(&status).or(self.critical_status);
            }
            OffenceEffect::CriticalConfirm { value } => self.critical_confirm += value,
            OffenceEffect::AbilityRanks { value } => self.ability_ranks += value,
            OffenceEffect::WeaponEnergyPercent { value } => self.weapon_energy_percent += value,
            OffenceEffect::WeaponStamina { value, abilities } if abilities.is_empty() => {
                self.weapon_stamina += value;
            }
            OffenceEffect::WeaponStamina { value, abilities } => {
                for id in abilities
                    .iter()
                    .filter_map(|a| content.weapon_ability_id(a))
                {
                    self.ability_stamina.push((id, value));
                }
            }
            OffenceEffect::Brace { to_hit, status } => {
                if let Some(status) = content.status_id(&status) {
                    let had = self.brace.map_or(0, |(t, _)| t);
                    self.brace = Some((had + to_hit, status));
                }
            }
            OffenceEffect::CriticalPenetration { value } => self.critical_penetration += value,
            OffenceEffect::Penetration {
                damage: kind,
                value,
            } => {
                if let Some(kind) = content.damage_type_id(&kind) {
                    self.penetration.push((kind, value));
                }
            }
            OffenceEffect::StatusDamage { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.status_damage.push((status, value));
                }
            }
            OffenceEffect::WeaponMarks { role, value } => self.weapon_marks.push((role, value)),
            OffenceEffect::TwoHandedCrit { value } => {
                self.two_handed_crit = self.two_handed_crit.saturating_add(value);
            }
            OffenceEffect::UnarmedDie { sides } => {
                self.unarmed_die = Some(self.unarmed_die.map_or(sides, |s| s.max(sides)));
            }
            OffenceEffect::FavouredPrey { to_hit, damage } => {
                self.favoured_prey.0 += to_hit;
                self.favoured_prey.1 += damage;
            }
            OffenceEffect::SwornFoe { to_hit, damage } => {
                let tag = choice.unwrap_or_default().to_string();
                self.sworn_foe.push((tag, to_hit, damage));
            }
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

    /// The penetration its talents add to a blow that can do damage of `kinds`.
    pub fn penetration_for(&self, kinds: &Kinds) -> i32 {
        (self.penetration.iter())
            .filter(|(kind, _)| kinds.iter().any(|k| k == *kind))
            .map(|(_, v)| v)
            .sum()
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

    /// Cells farther a launcher of weapon skill `skill` (its id) shoots, by every talent.
    pub fn launcher_range_for(&self, skill: Option<&str>) -> i32 {
        let own: i32 = (self.launcher_range_of.iter())
            .filter(|(s, _)| Some(s.as_str()) == skill)
            .map(|(_, v)| v)
            .sum();
        self.launcher_range + own
    }
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
