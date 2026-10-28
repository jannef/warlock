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
//!
//! What a **generated** chest holds is rolled by [`roll_contents`] from its band's loot table
//! (spec §8 "Chest loot"), on the level's chest stream (`mapgen/chest.rs`).

use crate::content::{Content, GOLD, MAX_ITEM_LEVEL};
use crate::item_parts::ItemPart;
use crate::magic::LootGrade;
use crate::magic::loot::{self, Luck};
use crate::object::{Stack, add_to_pile};
use crate::rng::Rng;
use crate::variant::Source;
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

/// What a generated chest of `level` holds (spec §8 "Chest loot"), rolled from its band's loot
/// table with the finder's `luck`, as a pile (the last rolled on top, like stacks joined). Draws, in order: the `items` dice; per item one
/// `below(100)` for its grade (under `rare` rare, under `magic` at least magic, else any) and
/// [`loot::roll_with`]'s own at `level + loot_level` (nothing found rolls nothing more); the
/// purse's dice (`dice` × `each` coins of the shops' currency, else `gold_coin`; none at 0);
/// the `gems` dice (0 or less: none) and per gem [`crate::gem::roll`]'s. A mimic standing in
/// for the chest carries the same stacks from the same draws. Empty without bands.
pub fn roll_contents(content: &Content, rng: &mut Rng, level: u8, luck: Luck) -> Vec<Stack> {
    let Some(band) = content.chests().band_of(level) else {
        return Vec::new();
    };
    let table = &band.loot;
    let at = level
        .saturating_add(table.loot_level)
        .clamp(1, MAX_ITEM_LEVEL);
    let mut holds = Vec::new();
    for _ in 0..table.items.roll(rng).max(0) {
        let grade = match rng.below(100) {
            g if g < u32::from(table.rare) => LootGrade::Rare,
            g if g < u32::from(table.magic) => LootGrade::Magic,
            _ => LootGrade::Any,
        };
        if let Some(stack) = loot::roll_with(content, rng, at, grade, Source::Loot, luck) {
            add_to_pile(&mut holds, &[stack], content);
        }
    }
    let coins = table.gold.dice.roll(rng).max(0) as u32 * table.gold.each;
    let currency = (content.shops().map(|s| s.currency)).or_else(|| content.object_id(GOLD));
    if let (true, Some(currency)) = (coins > 0, currency) {
        add_to_pile(&mut holds, &[Stack::new(currency, coins)], content);
    }
    for _ in 0..table.gems.roll(rng).max(0) {
        let (Some(gems), Some((gem, cutting))) =
            (content.gems(), crate::gem::roll(content, rng, at, false))
        else {
            continue;
        };
        let parts = vec![ItemPart::gem(content, None, gem, cutting)];
        let gem = Stack::one(gems.object).with_parts(parts);
        add_to_pile(&mut holds, &[gem], content);
    }
    holds
}
