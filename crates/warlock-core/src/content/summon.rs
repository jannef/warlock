//! A creature type's summoning (spec §8 "Summoning", #520): `[creature.summon]`, the enemy-only
//! ability to call forth creatures of its side. The rules are `world/summon.rs`.
//!
//! The pool is a list of creature ids (`creatures`), each as likely, or every type carrying a
//! tag within a level window (`tag`, `levels`), uniques and the summoner itself left out, in
//! creature order. It resolves once every creature type is known ([`resolve`]): a shared summoner
//! may call only shared types, a campaign's its own and shared ones. The cooldown is a status
//! (the house rule for ability limits, `ability.rs`): [`STATUS`], given `ceil(cooldown / eases)`
//! degrees.

use super::creature::CreatureDef;
use super::status::StatusId;
use super::{Content, CreatureId};
use crate::dice::Dice;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The cooldown status a summoning gives (`data/status.toml`).
pub const STATUS: &str = "summoning";
/// The most creatures one summoning may bring.
pub const MAX_COUNT: i32 = 12;
/// A summoning's cooldown, in seconds.
pub const COOLDOWNS: RangeInclusive<u32> = 6..=600;
/// What a summoning may cost in mana.
pub const MANA: RangeInclusive<i32> = 1..=1000;
/// How far away a summoner may see the hostile it summons at, in cells.
pub const RANGES: RangeInclusive<i32> = 1..=20;
/// How far from the summoner the summoned may appear, in cells (Chebyshev).
pub const SPREADS: RangeInclusive<i32> = 1..=5;
/// How many of one summoner's summoned may live at once.
pub const MOSTS: RangeInclusive<u8> = 1..=20;

/// A summoning as written: `[creature.summon]` (`data/creatures/README.toml` has the fields).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SummonSpec {
    #[serde(default)]
    pub creatures: Vec<String>,
    pub tag: Option<String>,
    pub levels: Option<[u8; 2]>,
    pub count: Dice,
    pub cooldown: u32,
    pub mana: i32,
    pub range: Option<i32>,
    pub spread: Option<i32>,
    pub most: Option<u8>,
    pub verb: Option<String>,
}

/// A creature type's summoning, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summon {
    /// What may come, each as likely (a type may stand twice, and is then twice as likely).
    pub pool: Vec<CreatureId>,
    /// How many come at once (at least 1, at most [`MAX_COUNT`]).
    pub count: Dice,
    /// Seconds before it summons again.
    pub cooldown: u32,
    /// The cooldown status, and the degrees of it a summoning gives.
    pub status: StatusId,
    pub degrees: u16,
    /// What a summoning costs in mana.
    pub mana: i32,
    /// It summons only at a hostile it sees within this many cells.
    pub range: i32,
    /// How far from it the summoned appear (Chebyshev).
    pub spread: i32,
    /// How many of its summoned may live at once.
    pub most: u8,
    /// What it does, in the message: "calls forth".
    pub verb: String,
}

impl SummonSpec {
    /// Checks what can be checked before the other creature types are known.
    pub(super) fn check(&self) -> Result<(), String> {
        match (self.creatures.is_empty(), &self.tag) {
            (true, None) => return Err("summon: needs `creatures` or `tag`".into()),
            (false, Some(_)) => return Err("summon: `creatures` or `tag`, not both".into()),
            (true, Some(_)) if self.levels.is_none() => {
                return Err("summon: `tag` needs `levels = [lowest, highest]`".into());
            }
            (false, None) if self.levels.is_some() => {
                return Err("summon: `levels` goes with `tag`".into());
            }
            _ => {}
        }
        if let Some([lo, hi]) = self.levels
            && lo > hi
        {
            return Err("summon: `levels` must be [lowest, highest]".into());
        }
        if self.count.min() < 1 || self.count.max() > MAX_COUNT {
            return Err(format!("summon: `count` must lie within 1 to {MAX_COUNT}"));
        }
        let within = |name: &str, v: i32, r: RangeInclusive<i32>| {
            if r.contains(&v) {
                Ok(())
            } else {
                Err(format!(
                    "summon: `{name}` must be {} to {}",
                    r.start(),
                    r.end()
                ))
            }
        };
        if !COOLDOWNS.contains(&self.cooldown) {
            return Err(format!(
                "summon: `cooldown` must be {} to {} seconds",
                COOLDOWNS.start(),
                COOLDOWNS.end()
            ));
        }
        within("mana", self.mana, MANA)?;
        if let Some(range) = self.range {
            within("range", range, RANGES)?;
        }
        if let Some(spread) = self.spread {
            within("spread", spread, SPREADS)?;
        }
        if let Some(most) = self.most
            && !MOSTS.contains(&most)
        {
            return Err(format!(
                "summon: `most` must be {} to {}",
                MOSTS.start(),
                MOSTS.end()
            ));
        }
        if self.verb.as_deref().is_some_and(|v| v.trim().is_empty()) {
            return Err("summon: `verb` must not be empty".into());
        }
        Ok(())
    }
}

/// Resolves creature type `me`'s summoning `spec` against every creature type: its pool and its
/// cooldown status's degrees.
pub(super) fn resolve(
    content: &Content,
    me: CreatureId,
    spec: &SummonSpec,
) -> Result<Summon, String> {
    let def = content.creature(me);
    // Whether the summoner's file may name `other`: shared types, and its own campaign's.
    let may_call = |other: &CreatureDef| {
        other.campaign.is_none() || other.campaign.as_deref() == def.campaign.as_deref()
    };
    let pool: Vec<CreatureId> = match (&spec.tag, spec.levels) {
        (Some(tag), Some([lo, hi])) => (0..content.creatures.len())
            .map(|i| CreatureId(i as u16))
            .filter(|&c| c != me)
            .filter(|&c| {
                let other = content.creature(c);
                !other.unique
                    && may_call(other)
                    && (lo..=hi).contains(&other.level)
                    && other.tags.iter().any(|t| t == tag)
            })
            .collect(),
        _ => (spec.creatures.iter())
            .map(|name| {
                let c = (content.creature_id(name))
                    .ok_or_else(|| format!("summon: unknown creature `{name}`"))?;
                let other = content.creature(c);
                if !may_call(other) {
                    return Err(format!(
                        "summon: creature `{name}` is campaign `{}`'s",
                        other.campaign.as_deref().unwrap_or_default()
                    ));
                }
                if c == me || content.creature(c).id == "player" {
                    return Err(format!("summon: `{name}` cannot be summoned"));
                }
                Ok(c)
            })
            .collect::<Result<_, _>>()?,
    };
    if pool.is_empty() {
        return Err("summon: no creature fits the pool".into());
    }
    if def.base_mana <= 0 {
        return Err("summon: a summoner needs `mana`".into());
    }
    let status = (content.status_id(STATUS))
        .ok_or_else(|| format!("summon: needs the `{STATUS}` status in data/status.toml"))?;
    let status_def = content.status(status);
    let eases = (status_def.eases)
        .filter(|_| status_def.tick.is_none())
        .ok_or_else(|| format!("summon: the `{STATUS}` status must ease and never tick"))?;
    let degrees = spec.cooldown.div_ceil(eases);
    let degrees = (u16::try_from(degrees).ok())
        .filter(|&d| d <= status_def.max)
        .ok_or_else(|| {
            format!(
                "summon: a cooldown of {} s is more than the `{STATUS}` status lasts",
                spec.cooldown
            )
        })?;
    Ok(Summon {
        pool,
        count: spec.count,
        cooldown: spec.cooldown,
        status,
        degrees,
        mana: spec.mana,
        range: spec.range.unwrap_or(8),
        spread: spec.spread.unwrap_or(2),
        most: spec.most.unwrap_or(6),
        verb: spec.verb.clone().unwrap_or_else(|| "summons".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(text: &str) -> Result<(), String> {
        let spec: SummonSpec = toml::from_str(text).map_err(|e| e.to_string())?;
        spec.check()
    }

    #[test]
    fn a_pool_of_ids_or_a_tag_with_levels() {
        assert!(spec("creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 8").is_ok());
        assert!(
            spec("tag = \"devil\"\nlevels = [5, 20]\ncount = \"2\"\ncooldown = 30\nmana = 8")
                .is_ok()
        );
    }

    #[test]
    fn refusals() {
        for (text, says) in [
            (
                "count = \"1d3\"\ncooldown = 30\nmana = 8",
                "`creatures` or `tag`",
            ),
            (
                "creatures = [\"imp\"]\ntag = \"devil\"\nlevels = [1, 2]\ncount = \"1d3\"\ncooldown = 30\nmana = 8",
                "not both",
            ),
            (
                "tag = \"devil\"\ncount = \"1d3\"\ncooldown = 30\nmana = 8",
                "needs `levels",
            ),
            (
                "tag = \"devil\"\nlevels = [9, 2]\ncount = \"1d3\"\ncooldown = 30\nmana = 8",
                "[lowest, highest]",
            ),
            (
                "creatures = [\"imp\"]\nlevels = [1, 2]\ncount = \"1d3\"\ncooldown = 30\nmana = 8",
                "goes with `tag`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d4-1\"\ncooldown = 30\nmana = 8",
                "`count`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"2d12\"\ncooldown = 30\nmana = 8",
                "`count`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 2\nmana = 8",
                "`cooldown`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 0",
                "`mana`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 8\nrange = 30",
                "`range`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 8\nspread = 0",
                "`spread`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 8\nmost = 0",
                "`most`",
            ),
            (
                "creatures = [\"imp\"]\ncount = \"1d3\"\ncooldown = 30\nmana = 8\nverb = \" \"",
                "`verb`",
            ),
        ] {
            let err = spec(text).expect_err(text);
            assert!(err.contains(says), "{text}: {err}");
        }
    }
}
