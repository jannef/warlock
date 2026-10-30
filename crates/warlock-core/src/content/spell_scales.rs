//! What a spell grows with (`[spell.scales]`; the combat progression design
//! `docs/design/combat-progression.md` §7, #556): per spell, which stats lift its dice, aim
//! it, widen its reach, and lengthen its status. Derived at cast time from the caster's stats;
//! nothing is saved.
//!
//! - Every weight is a **percent of a stat's modifier** ([`crate::stats::Stats::modifier`]),
//!   summed over the stats a dimension names.
//! - **damage**: the sum is the percent the spell's dice are lifted by, never below 0 and
//!   unbounded ([`super::SpellRules::lift`]; until the ladders replace it).
//! - **to_hit**: an `attack` or `touch` spell's bonus to the attack roll; the sum divided by 100,
//!   a half rounded toward zero (it may be negative, as Dexterity's is).
//! - **reach**, **duration**: cells more of reach, degrees more of a buff's or a hex's status;
//!   the sum, never below 0, divided by 100, a half rounded down (so 50 % is a cell per two
//!   points, as the retired `reach_step` and `presence_step` were).
//!
//! A dimension a spell leaves out keeps the old rule ([`SpellScales::default`]): reach by
//! Willpower, duration by Presence, to-hit by Dexterity, no damage lift.

use crate::stats::{Stat, Stats};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The largest weight a stat may have in one dimension, in percent of its modifier.
pub const MAX_SCALE_WEIGHT: i32 = 200;

/// One dimension's weights: each stat it names with its percent of that stat's modifier, in
/// [`Stat::ALL`] order. Empty: the stats do nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Weights(Vec<(Stat, i32)>);

impl Weights {
    /// One stat at `weight` percent.
    pub fn of(stat: Stat, weight: i32) -> Self {
        Weights(vec![(stat, weight)])
    }

    /// The stats and weights, in [`Stat::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (Stat, i32)> + '_ {
        self.0.iter().copied()
    }

    /// Whether it names no stat.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Σ weight × modifier over `stats`: percent of a modifier point.
    pub fn sum(&self, stats: &Stats) -> i32 {
        self.iter().map(|(s, w)| w * stats.modifier(s)).sum()
    }

    /// Whole steps: [`Weights::sum`], never below 0, over 100, a half rounded down (a reach's
    /// cells, a status's degrees).
    pub fn steps(&self, stats: &Stats) -> i32 {
        let sum = self.sum(stats).max(0);
        (sum + 49) / 100
    }

    /// A signed bonus: [`Weights::sum`] over 100, a half rounded toward zero (a to-hit).
    pub fn signed(&self, stats: &Stats) -> i32 {
        let sum = self.sum(stats);
        sum.signum() * ((sum.abs() + 49) / 100)
    }
}

/// What a spell grows with (module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellScales {
    /// Percent of its cap lifted per modifier point.
    pub damage: Weights,
    /// Percent of each modifier added to the attack roll.
    pub to_hit: Weights,
    /// Percent of a cell of reach per modifier point.
    pub reach: Weights,
    /// Percent of a degree of its status per modifier point.
    pub duration: Weights,
}

impl Default for SpellScales {
    /// The old rule: reach by Willpower and duration by Presence (a step per two points),
    /// to-hit by Dexterity, no damage lift.
    fn default() -> Self {
        SpellScales {
            damage: Weights::default(),
            to_hit: Weights::of(Stat::Dexterity, 100),
            reach: Weights::of(Stat::Willpower, 50),
            duration: Weights::of(Stat::Presence, 50),
        }
    }
}

/// `[spell.scales]` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScalesSpec {
    damage: Option<BTreeMap<String, i32>>,
    to_hit: Option<BTreeMap<String, i32>>,
    reach: Option<BTreeMap<String, i32>>,
    duration: Option<BTreeMap<String, i32>>,
}

/// What a spell's dimensions may be: whether it has a damage or heal roll, an attack roll, a
/// reach, and a status that lasts.
pub(super) struct Has {
    pub roll: bool,
    pub attack: bool,
    pub reach: bool,
    pub status: bool,
}

/// `[spell.scales]` checked against what the spell `has`: known stats, weights 1 to
/// [`MAX_SCALE_WEIGHT`], and only the dimensions the spell has; the defaults where it is silent.
pub(super) fn resolve(spec: Option<ScalesSpec>, has: &Has) -> Result<SpellScales, String> {
    let mut scales = SpellScales::default();
    let Some(spec) = spec else {
        return Ok(scales);
    };
    let dims = [
        ("damage", spec.damage, has.roll, &mut scales.damage),
        ("to_hit", spec.to_hit, has.attack, &mut scales.to_hit),
        ("reach", spec.reach, has.reach, &mut scales.reach),
        ("duration", spec.duration, has.status, &mut scales.duration),
    ];
    for (name, given, allowed, into) in dims {
        let Some(given) = given else {
            continue;
        };
        if !allowed {
            return Err(format!(
                "scales: this spell has no `{name}` (damage: a damage or heal roll; to_hit: a \
                 damage spell's attack or touch roll; reach: a range, radius, or length; duration: a buff's or \
                 a hex's status)"
            ));
        }
        let mut weights = Vec::new();
        for stat in Stat::ALL {
            if let Some(&w) = given.get(stat.id()) {
                if !(1..=MAX_SCALE_WEIGHT).contains(&w) {
                    return Err(format!(
                        "scales: `{name}`'s weights are 1 to {MAX_SCALE_WEIGHT} percent"
                    ));
                }
                weights.push((stat, w));
            }
        }
        if let Some(unknown) = given.keys().find(|k| Stat::from_id(k).is_none()) {
            return Err(format!("scales: unknown stat `{unknown}` in `{name}`"));
        }
        *into = Weights(weights);
    }
    Ok(scales)
}
