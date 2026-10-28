//! Chests (spec §8 "Chests"; docs/design/chests-and-treasure.md §3): a chest is a **fixture of
//! a cell**, as a door is: one of three terrains (shut, open, smashed; their `chest` field in
//! `data/terrain.toml`) plus a [`Chest`] on the cell in `LevelState::chests`: its level and what
//! it holds. Its lock lies in the level's `locks`, as a door's does, so keys, Pick lock, and
//! Knock work on it as on a door.
//!
//! A chest's **level** is its floor's (a branch floor's depth, a hand-made level's
//! `item_level`) unless a legend gives one; its **band** (`data/chests.toml`,
//! [`crate::content::ChestRules::band_of`]) follows from the level and is never saved, so the
//! bands can be renamed or re-cut without breaking a save. What a shut chest holds is ground
//! truth: perception sees only its band and whether it is shut, open, or smashed
//! ([`ChestLook`]). Opened, its contents spill into the pile on its cell; its entry stays (empty)
//! so an open or smashed chest is still named by its band.
//!
//! **Smashing** (a kick that breaks it) destroys each stack inside with `[smash] destroyed`
//! percent, one gameplay draw a stack, sparing coins and loose gems; **blowing up** (a blast
//! that beats its band's hardness) rolls every stack at the same percent and spares nothing
//! ([`destroyed`]).

use crate::object::Stack;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// A chest on a cell (ground truth).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chest {
    /// Its level: its band and its numbers follow from it.
    pub level: u8,
    /// What it holds, top first, while it is shut; empty once open (it lies in the pile).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holds: Vec<Stack>,
}

/// What a chest terrain is: shut (not walkable), open, or smashed (both walkable, a pile on
/// them). A terrain's `chest` field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChestState {
    Shut,
    Open,
    Smashed,
}

impl ChestState {
    /// Every state, in data order.
    pub const ALL: &[ChestState] = &[ChestState::Shut, ChestState::Open, ChestState::Smashed];

    /// Its id, as `data/terrain.toml` and the JSON dump write it.
    pub const fn id(self) -> &'static str {
        match self {
            ChestState::Shut => "shut",
            ChestState::Open => "open",
            ChestState::Smashed => "smashed",
        }
    }
}

/// What the player sees of a chest on a cell they see or remember (spec §8 "Chests"): its band
/// (an index into [`crate::content::ChestRules::bands`]), its state, and, with Chest-wise, on a
/// cell in sight, whether a shut one is locked. Never its level or what it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ChestLook {
    pub band: usize,
    pub state: ChestState,
    /// Whether it is locked, when the player can tell (Chest-wise, a shut chest in sight).
    pub locked: Option<bool>,
}

impl ChestLook {
    /// What the player calls it, by its band (`rules`; the terrain's `fallback` name without
    /// one): "coffer", "open coffer", "smashed coffer"; with Chest-wise "locked coffer" or
    /// "unlocked coffer".
    pub fn name(&self, rules: &crate::content::ChestRules, fallback: &str) -> String {
        let Some(band) = rules.bands.get(self.band) else {
            return fallback.to_string();
        };
        let name = &band.name;
        match (self.state, self.locked) {
            (ChestState::Shut, Some(true)) => format!("locked {name}"),
            (ChestState::Shut, Some(false)) => format!("unlocked {name}"),
            (ChestState::Shut, None) => name.clone(),
            (ChestState::Open, _) => format!("open {name}"),
            (ChestState::Smashed, _) => format!("smashed {name}"),
        }
    }
}

/// How the contents of a chest are treated as it breaks open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breaking {
    /// Kicked open: coins and loose gems are spared.
    Smashed,
    /// Blown apart by a blast: nothing is spared.
    BlownUp,
}

/// Splits `holds` into what survives and what is destroyed as a chest breaks `how`: each stack
/// in order is destroyed with `percent`, one draw of `rng` per stack rolled; on a smash, the
/// stacks `spared` says are coins or gems are never rolled. Returns (kept, destroyed), each in
/// the order they were held.
pub fn destroyed(
    holds: Vec<Stack>,
    how: Breaking,
    percent: u32,
    spared: impl Fn(&Stack) -> bool,
    rng: &mut Rng,
) -> (Vec<Stack>, Vec<Stack>) {
    let mut kept = Vec::new();
    let mut lost = Vec::new();
    for stack in holds {
        if how == Breaking::Smashed && spared(&stack) {
            kept.push(stack);
        } else if rng.chance(percent) {
            lost.push(stack);
        } else {
            kept.push(stack);
        }
    }
    (kept, lost)
}
