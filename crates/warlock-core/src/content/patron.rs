//! The craftsperson's patron (`assets/data/patron.toml`, spec §8 "Class abilities": Patron;
//! #1421, the craftsperson design §7.4): what a patron pays for a commission delivered
//! (`world/patron.rs`).
//!
//! The file is optional: without it no one commissions work.

use super::ContentError;
use serde::Deserialize;

/// The patron's rules that live in data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatronRules {
    /// Percent of the delivered item's worth paid (a maker's mark counted in the worth, its half
    /// again once: #1489).
    pub pays: u32,
}

impl PatronRules {
    /// Parses `data/patron.toml`: it pays something.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: PatronRules = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if rules.pays == 0 {
            return Err(err("`pays` must be above 0".into()));
        }
        Ok(rules)
    }

    /// What `worth` gold of work delivered is paid.
    pub fn paid(&self, worth: u64) -> u64 {
        worth * u64::from(self.pays) / 100
    }
}

impl super::RulesFile for PatronRules {
    const PATH: &'static str = "data/patron.toml";

    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/patron.toml";

    #[test]
    fn the_patron_pays_the_worth() {
        let content = Content::builtin().expect("content");
        let rules = content.patron().expect("the patron is in the game");
        assert_eq!(rules.pays, 100);
        assert_eq!(rules.paid(160), 160);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_patron() {
        for (text, says) in [
            ("pays = 0\n", "above 0"),
            ("pays = 100\nbonus = 50\n", "unknown field"),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert(PATH, text);
            let err = Content::load(&files).expect_err(text);
            assert!(err.message.contains(says), "{}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("the patron is optional");
        assert!(content.patron().is_none());
    }
}
