//! Doors (spec §8 "Doors"): locks, keys, and kicking doors in.
//!
//! **Locks** are per-cell state of a level ([`Lock`], in `LevelTemplate::locks` and
//! `LevelState::locks`), not terrain of their own: a locked door is the closed door terrain with
//! a lock on its cell. So it looks, blocks, muffles, and is remembered exactly like any closed
//! door (the player cannot tell until they try it), every door property stays in one terrain
//! definition, and a lock can carry data a terrain id cannot: its optional lock id. A lock lies
//! only on a closed door; unlocking or breaking the door takes it away.
//!
//! **Keys** are objects (`key` in `data/objects.toml`, [`Key`]): one names the lock id it opens,
//! a skeleton key (`key = "*"`) opens every lock. A lock without an id opens only to a skeleton
//! key. Opening a locked door while carrying a key that fits unlocks it and opens it, in one
//! action (`unlock` plus `open_door`); without one, "The door is locked." and no time passes.
//!
//! **Picking** a lock (the Pick lock ability, the Open lock skill's use) opens it without a key: a
//! Dexterity check plus what Open lock gives against the lock's **difficulty** ([`Lock::
//! difficulty`]: its own, [`LOCK_DIFFICULTY`] unless the level gives it one, plus on a dungeon
//! branch its `locks_per_depth` for each level below the first). Applied in `world/lockpick.rs`.
//!
//! **Kicking** a closed door (locked or not) breaks it with a chance of the kicker's Strength
//! **score** in the door's `sturdiness` ([`kick_breaks`]): one draw of the gameplay stream,
//! `1d(sturdiness) ≤ Strength`. A broken door is the terrain's `breaks_to` (a broken door,
//! open for good).

use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// The key text of a skeleton key: it opens every lock.
pub const SKELETON: &str = "*";

/// How hard a lock is to pick unless its level says otherwise (spec §8 "Doors"): a check of
/// 2d12 plus the picker's modifiers against this.
pub const LOCK_DIFFICULTY: u8 = 14;

/// The lock on a closed door's cell.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lock {
    /// Which keys fit it, besides a skeleton key: those with this lock id. `None`: only a
    /// skeleton key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// How hard it is to pick, if the level gave it a difficulty of its own; `None`:
    /// [`LOCK_DIFFICULTY`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<u8>,
}

impl Lock {
    /// A lock with the id `id` (`None`: only a skeleton key opens it), of the usual difficulty.
    pub fn new(id: Option<&str>) -> Self {
        Self {
            id: id.map(str::to_string),
            difficulty: None,
        }
    }

    /// How hard it is to pick, before what the depth adds: its own difficulty, else
    /// [`LOCK_DIFFICULTY`].
    pub fn difficulty(&self) -> u8 {
        self.difficulty.unwrap_or(LOCK_DIFFICULTY)
    }
}

/// What lock a key opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// Locks with this id.
    Lock(String),
    /// Every lock.
    Skeleton,
}

impl Key {
    /// The key a `key` field of an object names: [`SKELETON`] or a lock id.
    pub fn parse(text: &str) -> Result<Self, String> {
        match text.trim() {
            "" => Err("`key` must name a lock id, or `*` for a skeleton key".into()),
            SKELETON => Ok(Key::Skeleton),
            id => Ok(Key::Lock(id.to_string())),
        }
    }

    /// Whether it opens `lock`.
    pub fn opens(&self, lock: &Lock) -> bool {
        match self {
            Key::Skeleton => true,
            Key::Lock(id) => lock.id.as_deref() == Some(id.as_str()),
        }
    }
}

/// Checks a lock id as levels give it: not empty, and not the skeleton key's `*`.
pub fn check_lock_id(id: &str) -> Result<(), String> {
    if id.trim().is_empty() || id == SKELETON {
        Err(format!(
            "`{id}` is no lock id: a lock id is a name, not empty or `*`"
        ))
    } else {
        Ok(())
    }
}

/// The sturdiness a door may have: a kick breaks it with a chance of Strength in this.
pub const STURDINESS: std::ops::RangeInclusive<u8> = 1..=100;

/// Whether a kick with `strength` (the kicker's Strength score, never its modifier) breaks a door
/// of `sturdiness`: `1d(sturdiness) ≤ strength`, one draw of `rng`. So the chance is `strength`
/// in `sturdiness` ([`break_chance`]), and a kicker at least as strong as the door is sturdy
/// always breaks it.
pub fn kick_breaks(strength: u8, sturdiness: u8, rng: &mut Rng) -> bool {
    rng.below(u32::from(sturdiness.max(1))) < u32::from(strength)
}

/// The chance in percent that a kick with `strength` breaks a door of `sturdiness`, rounded
/// down ([`kick_breaks`]).
pub fn break_chance(strength: u8, sturdiness: u8) -> u32 {
    let sturdiness = u32::from(sturdiness.max(1));
    u32::from(strength).min(sturdiness) * 100 / sturdiness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_open_their_locks_and_skeleton_keys_every_lock() {
        let pantry = Lock::new(Some("pantry"));
        let plain = Lock::new(None);
        let iron = Key::parse("pantry").expect("a lock id");
        let skeleton = Key::parse("*").expect("a skeleton key");
        assert_eq!(skeleton, Key::Skeleton);
        assert!(iron.opens(&pantry));
        assert!(
            !iron.opens(&plain),
            "a lock without an id takes a skeleton key"
        );
        assert!(!Key::Lock("cellar".into()).opens(&pantry));
        assert!(skeleton.opens(&pantry) && skeleton.opens(&plain));
        assert!(Key::parse(" ").is_err());
        assert!(check_lock_id("*").is_err() && check_lock_id("").is_err());
        assert!(check_lock_id("pantry").is_ok());
        assert_eq!(pantry.difficulty(), LOCK_DIFFICULTY);
        let hard = Lock {
            difficulty: Some(20),
            ..pantry
        };
        assert_eq!(hard.difficulty(), 20);
    }

    #[test]
    fn kicks_break_doors_with_a_chance_of_strength_in_sturdiness() {
        assert_eq!(break_chance(10, 30), 33);
        assert_eq!(break_chance(9, 30), 30);
        assert_eq!(break_chance(18, 30), 60);
        assert_eq!(break_chance(30, 30), 100);
        assert_eq!(break_chance(40, 30), 100);
        assert_eq!(break_chance(1, 100), 1);
        // The draws agree with the chance: over every value the die can show.
        let mut rng = Rng::stream(7, "kicks");
        let tries = 30_000;
        let broke = (0..tries).filter(|_| kick_breaks(10, 30, &mut rng)).count();
        let percent = broke * 100 / tries;
        assert!((31..=35).contains(&percent), "{percent}% broke");
        // Deterministic: the same stream breaks the same doors.
        let rolls = |seed| {
            let mut rng = Rng::stream(seed, "kicks");
            (0..20)
                .map(|_| kick_breaks(12, 30, &mut rng))
                .collect::<Vec<_>>()
        };
        assert_eq!(rolls(3), rolls(3));
        // As strong as the door is sturdy: it always breaks.
        let mut rng = Rng::stream(1, "kicks");
        assert!((0..100).all(|_| kick_breaks(30, 30, &mut rng)));
    }
}
