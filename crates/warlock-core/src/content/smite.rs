//! Smiting (`assets/data/smite.toml`, spec §8 "Class abilities": "Smite"): what holy wrath falls
//! hardest on, and what a sin does to the smiter's zeal.
//!
//! The rule itself lives in `world/smite.rs` (the blow) and `world/alignment.rs` (the sin). The
//! file is optional: without it nothing is unholy and no sin empties a pool (fixtures).

use super::{Content, ContentError, CreatureId, ResourceId};
use serde::Deserialize;

/// The rules of smiting that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmiteRules {
    /// The tags of the unholy: a creature type carrying any of them takes the smite's `unholy`
    /// dice ([`Content::unholy`]).
    pub unholy: Vec<String>,
    /// The pool a deed toward evil empties (Zeal), if any.
    pub sin_empties: Option<ResourceId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmiteFile {
    smite: SmiteTable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmiteTable {
    unholy: Vec<String>,
    #[serde(default)]
    sin_empties: Option<String>,
}

impl SmiteRules {
    /// Parses `data/smite.toml`: each tag of the tag form, and the pool a sin empties a known
    /// resource.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SmiteFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let table = parsed.smite;
        for tag in &table.unholy {
            super::tag::check(tag).map_err(|m| err(format!("`unholy`: {m}")))?;
        }
        let sin_empties = match &table.sin_empties {
            Some(id) => Some(
                (content.resource_id(id))
                    .ok_or_else(|| err(format!("`sin_empties`: unknown resource `{id}`")))?,
            ),
            None => None,
        };
        Ok(Self {
            unholy: table.unholy,
            sin_empties,
        })
    }
}

impl Content {
    /// Whether creature type `id` is unholy (spec §8 "Class abilities": "Smite"): it carries a
    /// tag of `data/smite.toml`'s `unholy` list. The one reader of the list; nothing is unholy
    /// without the file (fixtures). Read by tags, never by a creature's hidden alignment.
    pub fn unholy(&self, id: CreatureId) -> bool {
        (self.smite()).is_some_and(|rules| rules.unholy.iter().any(|t| self.has_tag(id, t)))
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/smite.toml";

    #[test]
    fn the_shipped_rules_parse() {
        let content = Content::builtin().expect("content");
        let rules = content.smite().expect("smiting is in the game");
        assert!(rules.unholy.iter().any(|t| t == "undead"));
        let zeal = rules.sin_empties.expect("a sin empties zeal");
        assert_eq!(content.resource(zeal).id, "zeal");
        let id = |name: &str| content.creature_id(name).expect(name);
        assert!(content.unholy(id("skeleton")), "undead");
        assert!(!content.unholy(id("goblin")));
    }

    fn with(text: &str) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        Content::load(&files).map_err(|e| e.message)
    }

    #[test]
    fn bad_rules_are_refused() {
        let err =
            with("[smite]\nunholy = [\"undead\"]\nsin_empties = \"piety\"\n").expect_err("refused");
        assert!(err.contains("unknown resource"), "{err}");
        assert!(with("[smite]\nunholy = []\n").is_ok(), "nothing unholy");
    }

    #[test]
    fn without_the_file_nothing_is_unholy() {
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("content");
        let skeleton = content.creature_id("skeleton").expect("skeleton");
        assert!(content.smite().is_none() && !content.unholy(skeleton));
    }
}
