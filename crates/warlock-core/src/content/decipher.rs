//! Reading a scroll by a check (`assets/data/decipher.toml`, spec §8 "Scrolls"; #1024): its
//! numbers. The rule itself lives in `world/decipher.rs`.
//!
//! The file is optional: without it every scroll simply reads.

use super::ContentError;
use serde::Deserialize;

/// The numbers of the check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecipherRules {
    /// How long the Decipher ability takes, in seconds of game time.
    pub seconds: u32,
    /// The difficulty of a scroll of level 0.
    pub base: i32,
    /// How many spell levels make one point more, rounded down.
    pub levels_per_point: i32,
}

impl DecipherRules {
    /// Parses `data/decipher.toml`: `levels_per_point` must be positive.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.seconds < 1 {
            return Err(err("`seconds` must be at least 1".into()));
        }
        if parsed.levels_per_point < 1 {
            return Err(err("`levels_per_point` must be at least 1".into()));
        }
        Ok(parsed)
    }

    /// The difficulty of reading a scroll of spell level `level`.
    pub fn difficulty(&self, level: u8) -> i32 {
        self.base + i32::from(level) / self.levels_per_point
    }
}
