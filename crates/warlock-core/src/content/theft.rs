//! A creature type's theft and blink (docs/design/monster-dread.md §5.6, G6; #1130): `steals =
//! { what, chance, cooldown }`, a hit that may snatch a stack from the player's pack, and `blinks
//! = { range, cooldown }`, a step through nothing to the player's side when its path is blocked.
//! Enemy-only. The rules are `world/snatch.rs` and `world/blink.rs`.
//!
//! Each cooldown is a status (the house rule for ability limits, `ability.rs`): [`STEAL_STATUS`]
//! and [`BLINK_STATUS`], given `ceil(cooldown / eases)` degrees, as a summoning's (`summon.rs`).

use super::Content;
use super::status::StatusId;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The cooldown status a theft gives (`data/status.toml`).
pub const STEAL_STATUS: &str = "theft";
/// The cooldown status a blink gives (`data/status.toml`).
pub const BLINK_STATUS: &str = "blink";
/// A theft's or a blink's cooldown, in seconds.
pub const COOLDOWNS: RangeInclusive<u32> = 6..=600;
/// A theft's chance on a hit, in percent.
pub const CHANCES: RangeInclusive<u8> = 1..=100;
/// How far a blink reaches, in cells (Chebyshev from where the creature stands).
pub const RANGES: RangeInclusive<i32> = 2..=12;
/// The item classes a `small` thief takes (`data/item_classes.toml`): coins, gems, potions, a
/// ring, scrolls. A class the content lacks is left out.
pub const SMALL: [&str; 5] = ["coins", "gems", "potions", "rings", "scrolls"];

/// What a thief takes.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Takes {
    /// A stack of one of the [`SMALL`] classes.
    Small,
    /// Any stack.
    Any,
}

/// A theft as written: `steals = { what, chance, cooldown }` (`data/creatures/README.toml`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StealsSpec {
    pub what: Takes,
    pub chance: u8,
    pub cooldown: u32,
}

/// A creature type's theft, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Steals {
    /// What it takes.
    pub what: Takes,
    /// The chance in percent that a hit of its, with the cooldown off, takes something.
    pub chance: u8,
    /// Seconds before it steals again.
    pub cooldown: u32,
    /// The cooldown status, and the degrees of it a theft gives.
    pub status: StatusId,
    pub degrees: u16,
}

/// A blink as written: `blinks = { range, cooldown }` (`data/creatures/README.toml`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BlinksSpec {
    pub range: i32,
    pub cooldown: u32,
}

/// A creature type's blink, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blinks {
    /// How far it blinks, in cells (Chebyshev).
    pub range: i32,
    /// Seconds before it blinks again.
    pub cooldown: u32,
    /// The cooldown status, and the degrees of it a blink gives.
    pub status: StatusId,
    pub degrees: u16,
}

impl StealsSpec {
    /// Resolves the theft against `content` (the cooldown status, already loaded).
    pub(super) fn resolve(&self, content: &Content) -> Result<Steals, String> {
        if !CHANCES.contains(&self.chance) {
            return Err(format!(
                "steals: `chance` must be {} to {}",
                CHANCES.start(),
                CHANCES.end()
            ));
        }
        let (status, degrees) = cooldown(content, "steals", STEAL_STATUS, self.cooldown)?;
        Ok(Steals {
            what: self.what,
            chance: self.chance,
            cooldown: self.cooldown,
            status,
            degrees,
        })
    }
}

impl BlinksSpec {
    /// Resolves the blink against `content` (the cooldown status, already loaded).
    pub(super) fn resolve(&self, content: &Content) -> Result<Blinks, String> {
        if !RANGES.contains(&self.range) {
            return Err(format!(
                "blinks: `range` must be {} to {}",
                RANGES.start(),
                RANGES.end()
            ));
        }
        let (status, degrees) = cooldown(content, "blinks", BLINK_STATUS, self.cooldown)?;
        Ok(Blinks {
            range: self.range,
            cooldown: self.cooldown,
            status,
            degrees,
        })
    }
}

/// The cooldown status `id` and the degrees of it `seconds` take, checked: a status that eases
/// and never ticks, long enough to hold them.
fn cooldown(
    content: &Content,
    field: &str,
    id: &str,
    seconds: u32,
) -> Result<(StatusId, u16), String> {
    if !COOLDOWNS.contains(&seconds) {
        return Err(format!(
            "{field}: `cooldown` must be {} to {} seconds",
            COOLDOWNS.start(),
            COOLDOWNS.end()
        ));
    }
    let status = (content.status_id(id))
        .ok_or_else(|| format!("{field}: needs the `{id}` status in data/status.toml"))?;
    let def = content.status(status);
    let eases = (def.eases)
        .filter(|_| def.tick.is_none())
        .ok_or_else(|| format!("{field}: the `{id}` status must ease and never tick"))?;
    let degrees = (u16::try_from(seconds.div_ceil(eases)).ok())
        .filter(|&d| d <= def.max)
        .ok_or_else(|| {
            format!("{field}: a cooldown of {seconds} s is more than the `{id}` status lasts")
        })?;
    Ok((status, degrees))
}
