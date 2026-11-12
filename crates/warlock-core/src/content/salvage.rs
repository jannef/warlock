//! The numbers of salvaging an item for its materials (`assets/data/salvage.toml`; the rules are
//! `world/salvage.rs`, docs/dev/crafting.md). The file is optional: without it nobody salvages.

use super::ContentError;
use serde::Deserialize;

/// The numbers of a salvage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalvageRules {
    /// How long an attempt takes, in seconds of game time.
    pub seconds: u32,
    /// What each material's check must reach, before the material's `work`.
    pub difficulty: i32,
    /// The percent of a material's grams a made check gives back as stock.
    #[serde(rename = "yield")]
    pub percent: u8,
}

impl SalvageRules {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        // No time would pass.
        if rules.seconds == 0 {
            return Err(err("`seconds` must be at least 1".into()));
        }
        if !(1..=100).contains(&rules.percent) {
            return Err(err("`yield` must be 1 to 100".into()));
        }
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_parse_and_faults_are_refused() {
        let rules = SalvageRules::parse("s.toml", "seconds = 600\ndifficulty = 12\nyield = 50\n")
            .expect("valid");
        assert_eq!(
            rules,
            SalvageRules {
                seconds: 600,
                difficulty: 12,
                percent: 50
            }
        );
        let bad = |text: &str| SalvageRules::parse("s.toml", text).is_err();
        assert!(bad("seconds = 0\ndifficulty = 12\nyield = 50\n"));
        assert!(bad("seconds = 600\ndifficulty = 12\nyield = 0\n"));
        assert!(bad("seconds = 600\ndifficulty = 12\nyield = 101\n"));
        assert!(bad("seconds = 600\ndifficulty = 12\n"));
    }
}
