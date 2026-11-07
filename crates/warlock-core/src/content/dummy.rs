//! Target dummies (spec §8 "Weapon skills": "Target dummies", #1199): a creature type's
//! `training` table makes it a dummy, a thing to practise on that never acts, never dies, and
//! teaches weapon skills and fighting styles slowly up to a rank (`world/dummy.rs`).
//!
//! ```toml
//! training = { percent = 50, melee = 2, ranged = 4, breaks = 25 }
//! ```

use serde::Deserialize;

use super::MAX_RANK;

/// What a dummy teaches, and what becomes of the missiles loosed at it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DummyTraining {
    /// The percent of the marks a blow at a creature of the dummy's level would earn (1 to 100).
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

impl DummyTraining {
    /// The loader's check: a `percent` of 1 to 100, ranks up to [`MAX_RANK`], a chance.
    pub(super) fn check(&self) -> Result<(), String> {
        if !(1..=100).contains(&self.percent) {
            return Err(format!(
                "`training.percent` is 1 to 100, not {}",
                self.percent
            ));
        }
        if self.melee > MAX_RANK || self.ranged > MAX_RANK {
            return Err(format!(
                "`training.melee` and `training.ranged` are ranks, 0 to {MAX_RANK}"
            ));
        }
        if self.breaks > 100 {
            return Err(format!(
                "`training.breaks` is a chance, 0 to 100, not {}",
                self.breaks
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_training_table_is_checked() {
        let good = DummyTraining {
            percent: 50,
            melee: 2,
            ranged: 4,
            breaks: 25,
        };
        assert_eq!(good.check(), Ok(()));
        let bad = [
            DummyTraining { percent: 0, ..good },
            DummyTraining {
                percent: 101,
                ..good
            },
            DummyTraining { melee: 11, ..good },
            DummyTraining { ranged: 11, ..good },
            DummyTraining {
                breaks: 101,
                ..good
            },
        ];
        for t in bad {
            assert!(t.check().is_err(), "{t:?}");
        }
    }
}
