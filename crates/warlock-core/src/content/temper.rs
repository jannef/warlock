//! The numbers of working an item at an anvil (`assets/data/temper.toml`, spec §7 "Special
//! rooms"; the rules are `world/temper.rs`, the item's state `magic/worked.rs`).

use super::ContentError;
use crate::magic::worked::GRADES;
use serde::Deserialize;

/// The numbers of an attempt on an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemperRules {
    /// How long an attempt takes.
    pub seconds: u32,
    /// What the Strength check must reach on an item's first attempt.
    pub difficulty: i32,
    /// What it must reach more for each attempt ever made on the item.
    pub per_attempt: i32,
    /// The most improvements an item takes.
    pub most: u8,
}

impl Default for TemperRules {
    /// Ten minutes against 13, 2 harder an attempt, three improvements.
    fn default() -> Self {
        Self {
            seconds: 600,
            difficulty: 13,
            per_attempt: 2,
            most: 3,
        }
    }
}

impl TemperRules {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if rules.seconds == 0 {
            return Err(err("`seconds` must be at least 1".into()));
        }
        if rules.per_attempt < 0 {
            return Err(err("`per_attempt` must not be negative".into()));
        }
        if !(1..=GRADES.len()).contains(&usize::from(rules.most)) {
            return Err(err(format!("`most` must be 1 to {}", GRADES.len())));
        }
        Ok(rules)
    }

    /// What the check must reach on an item with `attempts` attempts behind it.
    pub fn against(&self, attempts: u8) -> i32 {
        self.difficulty + self.per_attempt * i32::from(attempts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_parse_and_faults_are_refused() {
        let good = "seconds = 600\ndifficulty = 13\nper_attempt = 2\nmost = 3\n";
        let rules = TemperRules::parse("t.toml", good).expect("valid");
        assert_eq!(rules, TemperRules::default());
        assert_eq!((rules.against(0), rules.against(3)), (13, 19));
        let bad = |text: &str| TemperRules::parse("t.toml", text).is_err();
        assert!(bad(
            "seconds = 0\ndifficulty = 13\nper_attempt = 2\nmost = 3\n"
        ));
        assert!(bad(
            "seconds = 1\ndifficulty = 13\nper_attempt = -1\nmost = 3\n"
        ));
        assert!(bad(
            "seconds = 1\ndifficulty = 13\nper_attempt = 2\nmost = 4\n"
        ));
        assert!(bad("seconds = 1\ndifficulty = 13\nper_attempt = 2\n"));
    }

    #[test]
    fn the_built_in_numbers_are_the_defaults() {
        let content = super::super::Content::builtin().expect("content");
        assert_eq!(*content.temper(), TemperRules::default());
    }
}
