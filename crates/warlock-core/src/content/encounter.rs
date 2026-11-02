//! Encounters on the overworld (`assets/data/encounters.toml`, spec §7 "Leaving an outdoor area";
//! docs/design/overworld-encounters.md): for now only the edge rule, how near a hostile creature
//! holds the player in an outdoor area when they walk off its edge (`world/edge_leave.rs`).
//!
//! Without the file the rules are [`EncounterRules::default`].

use super::ContentError;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// How near a hostile creature may hold the player at an area's edge, in cells (0: never).
pub const EDGE_REACHES: RangeInclusive<i32> = 0..=10;

/// The numbers of encounters.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterRules {
    /// Walking off the edge of an outdoor area is refused while a creature hostile to the player
    /// is within this many cells (Chebyshev) of them: 3. 0 never refuses it.
    pub edge_reach: i32,
}

impl Default for EncounterRules {
    fn default() -> Self {
        Self { edge_reach: 3 }
    }
}

impl EncounterRules {
    /// Parses `data/encounters.toml`: an `edge_reach` within [`EDGE_REACHES`].
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !EDGE_REACHES.contains(&rules.edge_reach) {
            return Err(err(format!(
                "edge_reach {} is out of range ({} to {})",
                rules.edge_reach,
                EDGE_REACHES.start(),
                EDGE_REACHES.end()
            )));
        }
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_file_holds_at_three() {
        let text = include_str!("../../../../assets/data/encounters.toml");
        let rules = EncounterRules::parse("data/encounters.toml", text).unwrap();
        assert_eq!(rules.edge_reach, 3);
    }

    #[test]
    fn an_edge_reach_out_of_range_is_refused() {
        let err = EncounterRules::parse("x", "edge_reach = 11").unwrap_err();
        assert!(err.to_string().contains("out of range"), "{err}");
        assert!(EncounterRules::parse("x", "edge_reach = -1").is_err());
        assert_eq!(
            EncounterRules::parse("x", "edge_reach = 0")
                .unwrap()
                .edge_reach,
            0
        );
    }
}
