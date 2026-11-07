//! Target dummies (spec §8 "Weapon skills": "Target dummies", #1199): a creature type's
//! `training` table makes it a dummy, a thing to practise on that never acts, never dies, and
//! teaches weapon skills and fighting styles slowly up to a rank (`world/dummy.rs`).
//!
//! ```toml
//! training = { percent = 50, melee = 2, ranged = 4, breaks = 25 }
//! ```

use serde::Deserialize;

/// What a dummy teaches, and what becomes of the missiles loosed at it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DummyTraining {
    /// The percent of the marks a blow at a creature of the dummy's level would earn.
    pub percent: u32,
    /// The rank past which a weapon skill or style of a blow, a bash, or a throw learns nothing
    /// more here.
    pub melee: u8,
    /// The rank past which a launcher's skill and its style learn nothing more here.
    pub ranged: u8,
    /// The chance in percent that a missile loosed at it is destroyed instead of dropping beside
    /// it (one that never breaks never does).
    pub breaks: u32,
}
