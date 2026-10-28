//! A treasure room's fields (spec §7 "Special rooms"; docs/design/chests-and-treasure.md §6):
//! `secret_doors = "all"` (every door of the room is a secret door, `mapgen/treasury.rs`), and
//! `treasure` (what generators lay inside: chests, heaps of gold, and loose gems).

use crate::dice::Dice;
use serde::Deserialize;

/// The most chests, heaps, or gems a treasure room's dice may give.
pub const MAX_TREASURE: i32 = 16;
/// The most gold coins one die of a heap may be worth.
pub const MAX_COINS_EACH: u32 = 1000;

/// How a special room's doors are hidden (`secret_doors = "all"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretDoors {
    /// Every door of the room is a secret door (a treasure room's), found by searching, never
    /// counted against the branch's `secret_doors`.
    All,
}

/// What a treasure room holds (`treasure = { chests = "1d2", gold = { ... }, gems = "1d3" }`):
/// `chests` chests of the floor's item level, `gold.heaps` heaps of `gold.dice` × `gold.each`
/// coins, and `gems` loose gems (`mapgen/treasury.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Treasury {
    pub chests: Dice,
    pub gold: Heaps,
    pub gems: Dice,
}

/// A treasure room's gold: `heaps` heaps, each `dice` × `each` coins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heaps {
    pub heaps: Dice,
    pub dice: Dice,
    pub each: u32,
}

impl Treasury {
    /// The fault in these numbers, if any: every count 0 to [`MAX_TREASURE`], a heap's dice
    /// never below 0, and `each` 1 to [`MAX_COINS_EACH`].
    pub(super) fn check(&self) -> Result<(), String> {
        for (what, dice) in [
            ("chests", self.chests),
            ("gold.heaps", self.gold.heaps),
            ("gems", self.gems),
        ] {
            if dice.min() < 0 || dice.max() > MAX_TREASURE {
                return Err(format!("treasure.{what} rolls 0 to {MAX_TREASURE}"));
            }
        }
        if self.gold.dice.min() < 0 {
            return Err("treasure.gold.dice never rolls below 0".into());
        }
        if !(1..=MAX_COINS_EACH).contains(&self.gold.each) {
            return Err(format!("treasure.gold.each is 1 to {MAX_COINS_EACH}"));
        }
        Ok(())
    }
}
