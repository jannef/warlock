//! A treasure room's fields (spec §7 "Special rooms"; docs/design/chests-and-treasure.md §6):
//! `secret_doors = "all"` (every door of the room is a secret door, `mapgen/treasury.rs`),
//! `sealed = "dig"` (a greater treasure room: no door at all, a box in the rock opened by breaking
//! a wall, its one wall guaranteed thin the room's breach, `SpecialRoom::breach`), and `treasure`
//! (what generators lay inside: chests, heaps of gold, and loose gems).

use super::{Content, SpecialRoom, TerrainId};
use crate::dice::Dice;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
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

/// How a special room is sealed with no door at all (`sealed = "dig"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sealed {
    /// No door and no corridor: a wall must be dug through or blasted (a greater treasure
    /// room's), its breach found by searching.
    Dig,
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

/// Why `breach` does not open `room` on a level of `terrain`, if it does not (spec §7 "Special
/// rooms": the rule a sealed room's breach keeps, for generators and level files alike): it is a
/// wall that breaks (a terrain with a hardness, not walkable), on the room's rim and not on the
/// level's edge, and 4-adjacent both to a cell outside the room that `reached` holds (reached
/// from the arrival) and to a walkable cell of the room. One wall to break, never two.
pub(crate) fn breach_fault(
    terrain: &Grid<TerrainId>,
    content: &Content,
    room: &SpecialRoom,
    breach: Pos,
    reached: impl Fn(Pos) -> bool,
) -> Option<&'static str> {
    let (w, h) = (terrain.width(), terrain.height());
    let edge = breach.x <= 0 || breach.y <= 0 || breach.x >= w - 1 || breach.y >= h - 1;
    let def = terrain.get(breach).map(|t| content.terrain(*t));
    let walkable = |p: Pos| terrain.get(p).is_some_and(|t| content.terrain(*t).walkable);
    let sides = [Dir::N, Dir::E, Dir::S, Dir::W].map(|d| breach.step(d));
    if edge || !room.contains(breach) {
        Some("is on the level's edge or off the room")
    } else if def.is_none_or(|d| d.walkable || d.breaks.is_none()) {
        Some("is not a wall that breaks")
    } else if !sides.iter().any(|&p| !room.contains(p) && reached(p)) {
        Some("is beside no cell reached from the arrival")
    } else if !sides.iter().any(|&p| room.contains(p) && walkable(p)) {
        Some("is beside no floor of the room")
    } else {
        None
    }
}
