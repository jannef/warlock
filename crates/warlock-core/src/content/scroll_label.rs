//! Unidentified scrolls' labels (`assets/data/scroll_labels.toml`, spec §8 "Scrolls"): the
//! syllables labels are made of, how many a label has, what a labelled scroll is called, and the
//! colour it shows in. Which label each scroll type gets in a run is drawn by
//! [`crate::scroll_label::Labels::draw`]. Without the file every scroll is known on sight.

use super::ContentError;
use crate::color::Rgb;
use serde::Deserialize;

/// `data/scroll_labels.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrollLabelDef {
    /// What an unknown scroll is called, `{label}` its label: "scroll labelled {label}".
    pub name: String,
    /// The same, plural: "scrolls labelled {label}".
    pub plural: String,
    /// The colour every unknown scroll shows in (its type's own would give it away).
    pub fg: Rgb,
    /// How many syllables a label has, least and most (each drawn evenly).
    pub syllables: [u8; 2],
    /// The syllables, written as labels show them ("ZUN", "BAR").
    pub parts: Vec<String>,
}

impl ScrollLabelDef {
    /// Parses `data/scroll_labels.toml`. The draw tolerates any syllable counts and parts: two
    /// labels drawn the same are told apart by a number (`scroll_label::draw_into`).
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))
    }

    /// `label`'s scroll's names, singular and plural: "scroll labelled ZUNBAR".
    pub fn names(&self, label: &str) -> (String, String) {
        (
            self.name.replace("{label}", label),
            self.plural.replace("{label}", label),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r##"
name = "scroll labelled {label}"
plural = "scrolls labelled {label}"
fg = "#ece4cc"
syllables = [2, 3]
parts = ["A", "B", "C", "D", "E", "F", "G", "H"]
"##;

    #[test]
    fn parses_and_names() {
        let def = ScrollLabelDef::parse("x", GOOD).expect("valid");
        assert_eq!(
            def.names("ZUN"),
            (
                "scroll labelled ZUN".to_string(),
                "scrolls labelled ZUN".to_string()
            )
        );
        let odd = GOOD.replace("[2, 3]", "[3, 2]");
        assert!(ScrollLabelDef::parse("x", &odd).is_ok());
        assert!(ScrollLabelDef::parse("x", &GOOD.replace("fg", "colour")).is_err());
    }
}
