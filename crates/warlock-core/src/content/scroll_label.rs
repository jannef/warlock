//! Unidentified scrolls' labels (`assets/data/scroll_labels.toml`, spec §8 "Scrolls"): the
//! syllables labels are made of, how many a label has, what a labelled scroll is called, and the
//! colour it shows in. Which label each scroll type gets in a run is drawn by
//! [`crate::scroll_label::Labels::draw`]. Without the file every scroll is known on sight.

use super::ContentError;
use crate::color::Rgb;
use serde::Deserialize;

/// How many syllables a label may have, at most.
pub const MAX_SYLLABLES: u8 = 6;
/// How many syllables the file must offer, at least.
pub const MIN_PARTS: usize = 8;

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
    /// Parses `data/scroll_labels.toml`: both names with `{label}`, a syllable count of 1 to
    /// [`MAX_SYLLABLES`] (least first), and at least [`MIN_PARTS`] distinct, non-empty syllables.
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let def: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (what, name) in [("name", &def.name), ("plural", &def.plural)] {
            if !name.contains("{label}") {
                return Err(err(format!("{what} `{name}` has no `{{label}}`")));
            }
        }
        let [least, most] = def.syllables;
        if least == 0 || least > most || most > MAX_SYLLABLES {
            return Err(err(format!(
                "syllables [{least}, {most}]: 1 to {MAX_SYLLABLES}, least first"
            )));
        }
        if def.parts.len() < MIN_PARTS {
            return Err(err(format!("at least {MIN_PARTS} parts")));
        }
        for (i, part) in def.parts.iter().enumerate() {
            if part.trim().is_empty() || part.trim() != part {
                return Err(err(format!("part `{part}` is empty or padded")));
            }
            if def.parts[..i].contains(part) {
                return Err(err(format!("duplicate part `{part}`")));
            }
        }
        Ok(def)
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
    }

    #[test]
    fn refuses_bad_files() {
        for (from, to) in [
            ("scroll labelled {label}", "scroll"),
            ("[2, 3]", "[3, 2]"),
            ("[2, 3]", "[0, 2]"),
            ("[2, 3]", "[2, 9]"),
            ("\"H\"]", "\"A\"]"),
            (", \"H\"]", "]"),
        ] {
            let text = GOOD.replace(from, to);
            assert!(ScrollLabelDef::parse("x", &text).is_err(), "{from} -> {to}");
        }
    }
}
