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
    Benefit, Choice, CombatStance, Content, DamageTypeId, MAX_RANK, Need, Role, StatusId,
    TalentDef, TalentEffect, TalentId,
};
use crate::damage::Kinds;
use crate::skills::Skills;
use crate::stats::{PerStat, Stats};
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
    /// Added to every melee blow's damage.
    pub melee_damage: i32,
    /// Hit points by level, one entry per rank held: `(per_levels, at_least)`
    /// ([`TalentMods::hp_at`]).
    pub hp_per_levels: Vec<(u8, i32)>,
    /// Percent faster hit points come back.
    pub healing_percent: i32,
    /// Percent louder footsteps (negative: quieter).
    pub footsteps_percent: i32,
    /// The doors it opens and closes make no noise.
    pub silent_doors: bool,
    /// How much fainter it hears (its hearing threshold, lower).
    pub hearing: i32,
    /// Percent more carrying limit.
    pub carry_percent: i32,
    /// Being Burdened costs it nothing.
    pub ignore_burdened: bool,
    /// Percent more of the slowdown its burden and Bloated put on its steps (negative: less).
    pub step_slowdown_percent: i32,
    /// Percent more time an overworld journey takes (negative: quicker).
    pub travel_percent: i32,
    /// Percent faster hunger burns (negative: slower).
    pub hunger_percent: i32,
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
    /// Percent more in its favour in shops.
    pub favour: i32,
    /// How much higher the most its favour may be.
    pub favour_cap: i32,
    /// Percent more gold from quest rewards.
    pub quest_gold_percent: i32,
    /// Percent more experience from quest rewards.
    pub quest_xp_percent: i32,
    /// The hours after which a magic item it carries is identified, if it has that (the
    /// fewest counts).
    pub identifies_carried: Option<u32>,
    /// Item descriptions say what things are worth.
    pub shows_worth: bool,
    /// Percent more often loot it finds rolls magic and rare.
    pub loot_magic_percent: i32,
    /// Percent more time reading a scroll takes (negative: less).
    pub read_percent: i32,
    /// The chance in percent a scroll it reads is not used up (the most counts).
    pub scroll_kept: u8,
    /// Its own bursts never catch it.
    pub spares_self: bool,
    /// It notices hidden things in passing, with this much added; `None`: only as its skills
    /// and race allow.
    pub notice: Option<i32>,
    /// Added to its checks to find what is hidden, searching and in passing.
    pub find: i32,
    /// Added to its searching.
    pub search: i32,
    /// Cells farther its searching reaches.
    pub search_reach: i32,
    /// Percent more damage traps do it (negative: less).
    pub trap_damage_percent: i32,
    /// Added to its checks, by the stat checked.
    pub checks: PerStat<i32>,
    /// The free blows its talents switch on.
    pub free_blows: FreeBlows,
    /// What attacks with some weapon skills gain ([`TalentMods::weapon_benefit`]).
    pub weapon: Vec<WeaponBonus>,
    /// Cells farther the readied missile weapon shoots.
    pub launcher_range: i32,
    /// Cells farther throws reach.
    pub throw_range: i32,
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
    /// Taken off every hit after the soak, never below 1.
    pub pain: i32,
    /// Resistances in percent, by element id.
    pub resist: Vec<(String, i32)>,
    /// Percent faster resting heals, on top of resting's own pace.
    pub resting_percent: i32,
    /// The active Second wind, if it has it.
    pub second_wind: Option<SecondWind>,
    /// Rotten food and overeating never make it vomit, and rotten food sickens it half as much.
    pub iron_stomach: bool,
    /// Statuses that ease faster, by the percent: `(status, percent)`, one entry per effect.
    pub status_eases: Vec<(StatusId, i32)>,
    /// Statuses that never lower its scores.
    pub no_penalty: Vec<StatusId>,
    /// Wounds bleed less: none from a big hit, a degree less from a critical.
    pub clotting: bool,
    /// Percent more of what its statuses take off its scores (negative: less).
    pub penalty_percent: i32,
    /// Added to its disease checks.
    pub disease_checks: i32,
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
    /// More penetration for blows that can do damage of a kind: `(kind, value)`.
    pub penetration: Vec<(DamageTypeId, i32)>,
    /// How much higher the burden's cap on its Dexterity in Defence is.
    pub burden_dex: i32,
    /// A trap it steps onto first gets a Dexterity check.
    pub sure_step: bool,
    /// Pits never catch it.
    pub pit_proof: bool,
    /// Percent quicker it moves and acts.
    pub speed_percent: i32,
    /// Dodge added against shots, throws, and trap missiles.
    pub missile_dodge: i32,
    /// Percent more damage bursts and bolts do it (negative: less).
    pub spell_damage_percent: i32,
}

/// What a talent adds to attacks with some weapon skills: the one named `skill` (the choice it
/// was taken for), else every skill of `role`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponBonus {
    pub skill: Option<String>,
    pub role: Option<Role>,
    pub benefit: Benefit,
    /// Ranks higher its benefits read.
    pub ranks: u8,
}

/// Protection a talent adds, while what it asks is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    pub value: i32,
    /// Only while the body armour worn has this much Protection at least.
    pub body_armour: Option<i32>,
    /// Only while holding a shield.
    pub shield: bool,
}

/// The active Second wind (spec §8 "Talents"): hit points back at once, then a cooldown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecondWind {
    pub per_level: u8,
    pub at_least: i32,
    pub cooldown: StatusId,
}

impl SecondWind {
    /// The hit points it gives back at character `level`.
    pub fn hp_at(self, level: u8) -> i32 {
        (i32::from(self.per_level) * i32::from(level)).max(self.at_least)
    }
}

impl TalentMods {
    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        *self == TalentMods::default()
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
    pub fn penalty(&self, status: StatusId, by: i32) -> i32 {
        if by >= 0 {
            return by;
        }
        if self.no_penalty.contains(&status) {
            return 0;
        }
        by * (100 + self.penalty_percent).max(0) / 100
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
            TalentEffect::MeleeDamage { value } => self.melee_damage += value,
            TalentEffect::MaxHpPerLevels {
                per_levels,
                at_least,
            } => self.hp_per_levels.push((per_levels, at_least)),
            TalentEffect::HealingPercent { value } => self.healing_percent += value,
            TalentEffect::FootstepsPercent { value } => self.footsteps_percent += value,
            TalentEffect::SilentDoors => self.silent_doors = true,
            TalentEffect::Hearing { value } => self.hearing += value,
            TalentEffect::CarryPercent { value } => self.carry_percent += value,
            TalentEffect::IgnoreBurdened => self.ignore_burdened = true,
            TalentEffect::StepSlowdownPercent { value } => self.step_slowdown_percent += value,
            TalentEffect::TravelPercent { value } => self.travel_percent += value,
            TalentEffect::HungerPercent { value } => self.hunger_percent += value,
            TalentEffect::NoExertion => self.no_exertion = true,
            TalentEffect::NoBloating => self.no_bloating = true,
            TalentEffect::CorpseStatsPercent { value } => self.corpse_stats_percent += value,
            TalentEffect::XpPercent { value } => self.xp_percent += value,
            TalentEffect::SkillRolls { value } => self.skill_rolls += value,
            // Handed out as the rank is taken (`World::take_talent`), never read after.
            TalentEffect::TrainSkill { .. } => {}
            TalentEffect::Favour { value, cap } => {
                self.favour += value;
                self.favour_cap += cap;
            }
            TalentEffect::QuestGoldPercent { value } => self.quest_gold_percent += value,
            TalentEffect::QuestXpPercent { value } => self.quest_xp_percent += value,
            TalentEffect::IdentifiesCarried { hours } => {
                self.identifies_carried =
                    Some(self.identifies_carried.map_or(hours, |h| h.min(hours)));
            }
            TalentEffect::ShowsWorth => self.shows_worth = true,
            TalentEffect::LootMagicPercent { value } => self.loot_magic_percent += value,
            TalentEffect::ReadPercent { value } => self.read_percent += value,
            TalentEffect::ScrollKept { chance } => self.scroll_kept = self.scroll_kept.max(chance),
            TalentEffect::SparesSelf => self.spares_self = true,
            TalentEffect::Notice { value } => {
                self.notice = Some(self.notice.unwrap_or(0) + value);
            }
            TalentEffect::Find { value } => self.find += value,
            TalentEffect::Search { value } => self.search += value,
            TalentEffect::SearchReach { value } => self.search_reach += value,
            TalentEffect::TrapDamagePercent { value } => self.trap_damage_percent += value,
            TalentEffect::Checks { stat, value } => self.checks[stat] += value,
            TalentEffect::FreeBlow { trigger } => {
                use crate::content::FreeBlowTrigger as T;
                match trigger {
                    T::Cleave => self.free_blows.cleave = true,
                    T::GreatCleave => self.free_blows.great_cleave = true,
                    T::Riposte => self.free_blows.riposte = true,
                    T::Opportunist => self.free_blows.opportunist = true,
                }
            }
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
            } => self.weapon.push(WeaponBonus {
                skill: role
                    .is_none()
                    .then(|| choice.unwrap_or_default().to_string()),
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
                    ..Benefit::default()
                },
                ranks,
            }),
            TalentEffect::LauncherRange { value } => self.launcher_range += value,
            TalentEffect::ThrowRange { value } => self.throw_range += value,
            TalentEffect::MaxDex { value } => self.max_dex += value,
            TalentEffect::Deflection { value } => self.deflection += value,
            TalentEffect::Protection {
                value,
                body_armour,
                shield,
            } => self.protection.push(Guard {
                value,
                body_armour,
                shield,
            }),
            TalentEffect::StanceToHit { stance, percent } => {
                match self.stance_to_hit.iter_mut().find(|(s, _)| *s == stance) {
                    Some((_, p)) => *p += percent,
                    None => self.stance_to_hit.push((stance, percent)),
                }
            }
            TalentEffect::FleePercent { value } => self.flee_percent += value,
            TalentEffect::Pain { value } => self.pain += value,
            TalentEffect::Resist { value } => {
                let element = choice.unwrap_or_default();
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
            TalentEffect::PenaltyPercent { value } => self.penalty_percent += value,
            TalentEffect::DiseaseChecks { value } => self.disease_checks += value,
            TalentEffect::DiseaseRecovery { value } => self.disease_recovery += value,
            TalentEffect::KickDie { sides } => {
                self.kick_die = Some(self.kick_die.map_or(sides, |s| s.max(sides)));
            }
            TalentEffect::DoorKick { value } => self.door_kick += value,
            TalentEffect::KickNoisePercent { value } => self.kick_noise_percent += value,
            TalentEffect::DigPercent { value } => self.dig_percent += value,
            TalentEffect::TwoHandedPrice { value } => self.two_handed_price += value,
            TalentEffect::CriticalStatus { status } => {
                self.critical_status = content.status_id(&status).or(self.critical_status);
            }
            TalentEffect::Penetration {
                damage: kind,
                value,
            } => {
                if let Some(kind) = content.damage_type_id(&kind) {
                    self.penetration.push((kind, value));
                }
            }
            TalentEffect::BurdenDex { value } => self.burden_dex += value,
            TalentEffect::SureStep => self.sure_step = true,
            TalentEffect::PitProof => self.pit_proof = true,
            TalentEffect::SpeedPercent { value } => self.speed_percent += value,
            TalentEffect::MissileDodge { value } => self.missile_dodge += value,
            TalentEffect::SpellDamagePercent { value } => self.spell_damage_percent += value,
        }
    }

    /// What attacks with weapon skill `id` of `role` gain beyond its ranks, and how many ranks
    /// higher its benefits read.
    pub fn weapon_benefit(&self, id: &str, role: Role) -> (Benefit, u8) {
        let mut benefit = Benefit::default();
        let mut ranks = 0u8;
        for bonus in &self.weapon {
            let applies = match &bonus.skill {
                Some(skill) => skill == id,
                None => bonus.role == Some(role),
            };
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
    /// no body armour) and holds a shield or not.
    pub fn protection_with(&self, body: Option<i32>, shield: bool) -> i32 {
        (self.protection.iter())
            .filter(|g| g.body_armour.is_none_or(|at| body.is_some_and(|b| b >= at)))
            .filter(|g| !g.shield || shield)
            .map(|g| g.value)
            .sum()
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
    match words {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
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
