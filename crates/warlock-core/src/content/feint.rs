//! Feinting (`assets/data/rules/feint.toml`, spec §8 "Skills": Bluff; #1016): the numbers of
//! Bluff's use, a lie in combat.
//!
//! The rule itself lives in `world/skills/feint.rs`: who can (anyone trained in a skill with the `feint`
//! benefit, `data/skills/`), at whom (a hostile creature they see next to them, with a mind),
//! the check (its Willpower against `difficulty` + their Presence modifier + their Bluff), and
//! the status a failed check leaves.
//!
//! The file is optional: without it nobody feints, and Bluff stays a skill whose use is not in
//! the game (the abilities menu's "not usable yet").

use super::{Content, ContentError, StatusId};
use serde::Deserialize;

/// The rules of feinting that live in data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeintRules {
    /// The check's base difficulty, before the Presence modifier and Bluff are added.
    pub difficulty: i32,
    /// What a foe that fails suffers: `per_action`, `flat_footed`, one degree (Surprised).
    pub status: StatusId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FeintFile {
    difficulty: i32,
    status: String,
}

impl FeintRules {
    /// Parses `data/rules/feint.toml`: the status must be known.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: FeintFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let status = (content.status_id(&parsed.status))
            .ok_or_else(|| err(format!("unknown status `{}`", parsed.status)))?;
        Ok(Self {
            difficulty: parsed.difficulty,
            status,
        })
    }
}

impl super::RulesFile for FeintRules {
    const PATH: &'static str = "data/rules/feint.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/feint.toml";

    #[test]
    fn a_feint_leaves_a_foe_surprised() {
        let content = crate::test_support::builtin();
        let rules = content.feint().expect("feinting is in the game");
        assert_eq!(content.status(rules.status).id, "surprised");
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_feinting() {
        let text = "difficulty = 10\nstatus = \"nobody\"\n";
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        let err = Content::load(&files).expect_err(text);
        assert!(err.message.contains("unknown status"), "{}", err.message);
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("feinting is optional");
        assert!(content.feint().is_none());
    }
}
