//! Grimoires (spec §8 "Spells": "Learning"; #1260): the books spells are learned from. A
//! grimoire is generated for every scroll of a spell (an object whose `read` is of kind
//! `spell`) when content loads, from `data/grimoires.toml`: `grimoire_of_X` beside
//! `scroll_of_X`, named "grimoire of X" (the scroll's name with "grimoire" for "scroll"), worth
//! `value` times the scroll, and otherwise the `[object]` table's (glyph, colour, class,
//! materials, weight). Its [`super::ObjectDef::grimoire`] is the scroll's spell; it has no
//! `read`, so it is never labelled, never loot by itself (no `frequency`), and a scroll's
//! reading code never sees it.
//!
//! ```toml
//! chance = 10        # in percent: a spell scroll made at random is its grimoire instead
//! value = 5          # a grimoire is worth this many times its scroll
//!
//! [object]           # what every grimoire is besides its id, names, level, and value
//! glyph = "?"
//! fg = "#b5835a"
//! class = "scrolls"
//! materials = ["paper"]
//! weight = 800
//! stackable = true
//! ```
//!
//! Generation is a pure function of the content (no draws): the grimoires come after every
//! object file, in their scrolls' path order, so the ids and the fingerprint are the same on
//! every load. Without the file there are none.

use super::object::ObjectSpec;
use super::{Content, ContentError, ContentFiles, ObjectId};
use crate::rng::Rng;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The rules file.
pub(super) const RULES: &str = "data/grimoires.toml";

/// Grimoires as content defines them; none without [`RULES`].
#[derive(Clone, Debug, Default)]
pub struct Grimoires {
    /// The chance in percent that a spell scroll made at random is its grimoire instead.
    pub chance: u8,
    /// Each spell scroll's grimoire.
    of: BTreeMap<ObjectId, ObjectId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesFile {
    chance: u8,
    value: u32,
    object: toml::Table,
}

/// What [`specs`] read: the rules and, for each grimoire generated, its scroll's id.
pub(super) struct Generated<'a> {
    pub chance: u8,
    pub specs: Vec<(&'a str, ObjectSpec)>,
    /// Each generated grimoire's id and its scroll's, in the specs' order.
    pub pairs: Vec<(String, String)>,
}

/// The grimoire spec of every spell scroll among `objects` (module docs); `None` without the
/// rules file.
pub(super) fn specs<'a>(
    files: &'a ContentFiles,
    objects: &[(&str, ObjectSpec)],
) -> Result<Option<Generated<'a>>, ContentError> {
    let Some(text) = files.get_optional(RULES) else {
        return Ok(None);
    };
    let err = |m: String| ContentError::new(RULES, m);
    let rules: RulesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut specs = Vec::new();
    let mut pairs = Vec::new();
    for (_, scroll) in objects.iter().filter(|(_, o)| o.casts_spell()) {
        let rest = |s: &str, of: &str| s.strip_prefix(of).unwrap_or(s).to_string();
        let id = format!("grimoire_of_{}", rest(&scroll.id, "scroll_of_"));
        let name = format!("grimoire of {}", rest(&scroll.name, "scroll of "));
        let plural = format!("grimoires of {}", rest(&scroll.name, "scroll of "));
        let mut table = rules.object.clone();
        table.insert("id".into(), id.clone().into());
        table.insert("name".into(), name.into());
        table.insert("plural".into(), plural.into());
        let value = u64::from(scroll.value) * u64::from(rules.value);
        let value = i64::try_from(value).unwrap_or(i64::MAX);
        table.insert("value".into(), value.into());
        if let Some(level) = scroll.level {
            table.insert("level".into(), i64::from(level).into());
        }
        let spec: ObjectSpec = toml::Value::Table(table)
            .try_into()
            .map_err(|e| err(format!("grimoire `{id}`: {e}")))?;
        pairs.push((id, scroll.id.clone()));
        specs.push((RULES, spec));
    }
    Ok(Some(Generated {
        chance: rules.chance,
        specs,
        pairs,
    }))
}

impl Grimoires {
    /// The rules resolved: each pair's ids as objects (both loaded with the objects).
    pub(super) fn new(chance: u8, of: BTreeMap<ObjectId, ObjectId>) -> Self {
        Self { chance, of }
    }

    /// The grimoire of spell scroll `scroll`, if it has one.
    pub fn of(&self, scroll: ObjectId) -> Option<ObjectId> {
        self.of.get(&scroll).copied()
    }
}

impl Content {
    /// What a spell scroll made at random turns out to be (spec §8 "Spells": "Learning"):
    /// `object` itself, or with [`Grimoires::chance`] its grimoire. One draw of `rng`, and only
    /// for a scroll that has a grimoire, so nothing else made draws more.
    pub fn grimoire_or(&self, rng: &mut Rng, object: ObjectId) -> ObjectId {
        match self.grimoires.of(object) {
            Some(grimoire) if rng.below(100) < u32::from(self.grimoires.chance) => grimoire,
            _ => object,
        }
    }

    /// Grimoires (`data/grimoires.toml`): the chance and each spell scroll's.
    pub fn grimoires(&self) -> &Grimoires {
        &self.grimoires
    }
}

#[cfg(test)]
mod tests {
    use super::super::Reading;
    use super::*;

    #[test]
    fn every_spell_scroll_has_a_grimoire_of_its_spell() {
        let content = crate::test_support::builtin();
        let mut scrolls = 0;
        for (i, def) in content.object_defs().iter().enumerate() {
            let Some(Reading::Cast(scroll)) = &def.read else {
                continue;
            };
            scrolls += 1;
            let grimoire = (content.grimoires().of(ObjectId(i as u16))).expect("a grimoire");
            let book = content.object(grimoire);
            assert_eq!(book.grimoire, Some(scroll.spell), "{}", def.id);
            assert_eq!(book.id, def.id.replace("scroll_of_", "grimoire_of_"));
            assert_eq!(
                book.describe(1),
                def.describe(1).replace("scroll", "grimoire")
            );
            assert_eq!(book.value, def.value * 5, "{}", book.id);
            assert!(book.read.is_none() && book.frequency == 0, "{}", book.id);
        }
        assert!(scrolls > 60, "{scrolls}");
        // Nothing else has one: the scroll of restoration casts no spell.
        let restoration = content.object_id("scroll_of_restoration").expect("object");
        assert_eq!(content.grimoires().of(restoration), None);
    }

    #[test]
    fn a_spell_scroll_made_at_random_is_its_grimoire_one_time_in_ten() {
        let content = crate::test_support::builtin();
        let scroll = content.object_id("scroll_of_frost").expect("object");
        let grimoire = content.object_id("grimoire_of_frost").expect("object");
        let mut rng = Rng::stream(1, "grimoire-test");
        let books = (0..10_000)
            .filter(|_| content.grimoire_or(&mut rng, scroll) == grimoire)
            .count();
        assert!((900..=1100).contains(&books), "{books}");
        // Anything else is itself, and draws nothing.
        let dagger = content.object_id("dagger").expect("object");
        let before = rng.clone();
        assert_eq!(content.grimoire_or(&mut rng, dagger), dagger);
        assert_eq!(rng, before);
    }
}
