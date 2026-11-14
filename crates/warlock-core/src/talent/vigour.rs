//! The vigour talents (#1495): what a talent does to the holder's body: hit points, healing, stats
//! and checks, statuses, diseases, and the once-in-a-life escapes from death. [`VigourEffect`] is
//! the data (a talent's `[[effect]]` of one of [`VigourEffect::KINDS`]), [`VigourMods`] the sum
//! every rule reads (`TalentMods::vigour`). A new effect of the area is a variant here, its field
//! and its arm in [`VigourMods::add`] here, its wiki text in `xtask/src/wiki/talents/vigour.rs`,
//! and its one read.

use super::{SecondWind, per_levels_at};
use crate::content::{Content, StatusId};
use crate::dice::Dice;
use crate::stats::{PerStat, Stat};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VigourEffect {
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
    /// Resting heals this percent faster on top of resting's own pace (Fast healer).
    RestingPercent { value: i32 },
    /// The active Second wind: `per_level` hit points per character level back at once, at
    /// least `at_least`, then the `cooldown` status (an id in `data/status/`), while which
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
    /// Every level past `hp_level` (data/experience.toml) adds hit points again, as the levels
    /// up to it do, and `bonus` more (Hardened). Handed out as the levels come.
    Hardened { bonus: i32 },
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
}

impl VigourEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`VigourEffect`]).
    pub const KINDS: [&'static str; 26] = [
        "checks",
        "clotting",
        "disease_checks",
        "disease_immune",
        "disease_interval_percent",
        "disease_recovery",
        "drain_checks",
        "hardened",
        "healing_percent",
        "max_hp_per_levels",
        "max_hp_percent",
        "no_penalty",
        "no_penalty_to",
        "penalty_percent",
        "raise_percent",
        "reborn_whole",
        "reprieve",
        "resting_percent",
        "rise_from_ashes",
        "second_rising",
        "second_wind",
        "starving_heals",
        "stat",
        "status_eases",
        "status_lasts",
        "wound_more",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VigourMods {
    /// Hit points by level, one entry per talent: `(per_levels, at_least)`
    /// ([`VigourMods::hp_at`]).
    pub hp_per_levels: Vec<(u8, i32)>,
    /// What its stat gifts add to scores and potentials: handed out when a talent is taken or
    /// given, never read by a rule.
    pub stats: PerStat<i32>,
    /// Percent faster hit points come back.
    pub healing_percent: i32,
    /// Percent faster resting heals, on top of resting's own pace.
    pub resting_percent: i32,
    /// The active Second wind, if it has it.
    pub second_wind: Option<SecondWind>,
    /// Statuses that ease faster, by the percent: `(status, percent)`, one entry per effect.
    pub status_eases: Vec<(StatusId, i32)>,
    /// Statuses that last longer, by the percent: `(status, percent)`, one entry per effect.
    pub status_lasts: Vec<(StatusId, i32)>,
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
    /// Added to its checks against a drain (spec §8 "Stats": "Drain").
    pub drain_checks: i32,
    /// Added to its checks, by the stat checked.
    pub checks: PerStat<i32>,
    /// Checks fewer won in a row that shake a disease off.
    pub disease_recovery: i32,
    /// Percent further apart a disease's progressions come.
    pub disease_interval_percent: i32,
    /// It never catches a disease.
    pub disease_immune: bool,
    /// The stats no status lowers.
    pub no_penalty_to: Vec<Stat>,
    /// Percent more of what a status raises its scores by: `(status, percent)`.
    pub raise_percent: Vec<(StatusId, i32)>,
    /// Hit points every level past `hp_level` adds beyond a level's own, if it has Hardened.
    pub hardened: Option<i32>,
    /// Percent more maximum hit points (negative: fewer).
    pub max_hp_percent: i32,
    /// Rise from the ashes, if it has it.
    pub rise: Option<Rise>,
    /// Its rising brings it back whole (Reborn whole).
    pub reborn_whole: bool,
    /// Its once-in-a-life reprieve from death, if it has one.
    pub reprieve: Option<Reprieve>,
    /// It may rise from the ashes once more (Second rising).
    pub second_rising: bool,
    /// Its hit points come back while it is Starving too (Another head).
    pub starving_heals: bool,
}

impl VigourMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: VigourEffect) {
        match effect {
            VigourEffect::MaxHpPerLevels {
                per_levels,
                at_least,
            } => self.hp_per_levels.push((per_levels, at_least)),
            VigourEffect::MaxHpPercent { value } => self.max_hp_percent += value,
            VigourEffect::Stat { stat, value } => self.stats[stat] += value,
            VigourEffect::HealingPercent { value } => self.healing_percent += value,
            VigourEffect::RestingPercent { value } => self.resting_percent += value,
            VigourEffect::SecondWind {
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
            VigourEffect::StatusEases { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.status_eases.push((status, value));
                }
            }
            VigourEffect::StatusLasts { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.status_lasts.push((status, value));
                }
            }
            VigourEffect::NoPenalty { status } => {
                self.no_penalty.extend(content.status_id(&status));
            }
            VigourEffect::Clotting => self.clotting = true,
            VigourEffect::WoundMore { value } => self.wound_more += value,
            VigourEffect::PenaltyPercent { value } => self.penalty_percent += value,
            VigourEffect::DiseaseChecks { value } => self.disease_checks += value,
            VigourEffect::DrainChecks { value } => self.drain_checks += value,
            VigourEffect::Checks { stat, value } => self.checks[stat] += value,
            VigourEffect::DiseaseRecovery { value } => self.disease_recovery += value,
            VigourEffect::DiseaseIntervalPercent { value } => {
                self.disease_interval_percent += value;
            }
            VigourEffect::Reprieve {
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
            VigourEffect::RiseFromAshes {
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
                        damage: damage.into(),
                        reroll: 0,
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
            VigourEffect::RebornWhole => self.reborn_whole = true,
            VigourEffect::SecondRising => self.second_rising = true,
            VigourEffect::StarvingHeals => self.starving_heals = true,
            VigourEffect::DiseaseImmune => self.disease_immune = true,
            VigourEffect::NoPenaltyTo { stat } => self.no_penalty_to.push(stat),
            VigourEffect::RaisePercent { status, value } => {
                if let Some(status) = content.status_id(&status) {
                    self.raise_percent.push((status, value));
                }
            }
            VigourEffect::Hardened { bonus } => {
                self.hardened = Some(self.hardened.unwrap_or(0) + bonus);
            }
        }
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

    /// How many seconds status `status` takes to ease by a degree for it, its usual `every`
    /// eased by the percents faster its talents give, then lengthened by the percents longer
    /// they make it last (at least 1).
    pub fn eases_every(&self, status: StatusId, every: u32) -> u32 {
        let sum = |list: &[(StatusId, i32)]| -> i32 {
            (list.iter())
                .filter(|(s, _)| *s == status)
                .map(|(_, p)| p)
                .sum()
        };
        let (faster, longer) = (sum(&self.status_eases), sum(&self.status_lasts));
        let mut every = u64::from(every);
        if faster != 0 {
            let pace = u64::try_from((100 + faster).max(1)).unwrap_or(1);
            every = every * 100 / pace;
        }
        if longer != 0 {
            let lasts = u64::try_from((100 + longer).max(1)).unwrap_or(1);
            every = every * lasts / 100;
        }
        u32::try_from(every).unwrap_or(u32::MAX).max(1)
    }

    /// What its talents add to a check of `stat`.
    pub fn check(&self, stat: Stat) -> i32 {
        self.checks[stat]
    }

    /// The hit points the per-level effects give at character `level`: for each talent,
    /// one per `per_levels` levels, at least `at_least`.
    pub fn hp_at(&self, level: u8) -> i32 {
        per_levels_at(&self.hp_per_levels, level)
    }
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
