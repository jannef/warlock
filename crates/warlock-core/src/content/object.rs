//! Object definitions (`assets/data/objects.toml`, spec §7): the things that lie on the floor and
//! are picked up and carried.
//!
//! Every object defines the glyph it is drawn with, its color, its material, and its weight in
//! grams. Glyphs follow ADOM loosely: `)` weapons, `/` missiles, `}` missile weapons, `[` armor,
//! `(` tools, `!` potions, `?` scrolls, `%` food, `*` stones and gems, `$` coins, `=` rings,
//! `"` amulets.

use super::material::MaterialId;
use super::{Content, ContentError};
use crate::color::Rgb;
use crate::object::Stack;
use serde::{Deserialize, Serialize};

/// Index of an object definition in [`super::Content::object_defs`].
///
/// Ids are assigned in file order when content loads, like terrain ids; saves record the content
/// fingerprint to catch a reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectId(pub u16);

/// Most of one object a level file may place with `"id*N"`.
pub const MAX_PLACED: u32 = 9999;

/// An object type as written in content files.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObjectSpec {
    pub id: String,
    pub name: String,
    /// Plural name; the name plus `s` if absent.
    pub plural: Option<String>,
    pub glyph: char,
    pub fg: Rgb,
    /// A material id in `materials.toml`.
    pub material: String,
    /// Grams, at least 1.
    pub weight: u32,
    #[serde(default)]
    pub stackable: bool,
}

/// A resolved object type.
#[derive(Clone, Debug)]
pub struct ObjectDef {
    pub id: String,
    /// Singular, without an article: "dagger".
    pub name: String,
    /// "daggers".
    pub plural: String,
    pub glyph: char,
    pub fg: Rgb,
    pub material: MaterialId,
    /// Weight of one, in grams; at least 1.
    pub weight: u32,
    /// Several lie in one stack (coins, arrows) instead of a stack each.
    pub stackable: bool,
}

impl ObjectDef {
    /// `count` of it in words: "a dagger", "an iron key", "12 arrows".
    pub fn describe(&self, count: u32) -> String {
        if count == 1 {
            format!("{} {}", article(&self.name), self.name)
        } else {
            format!("{count} {}", self.plural)
        }
    }
}

/// `a` or `an` for a noun, by its first letter.
pub fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

impl ObjectSpec {
    /// Checks the definition and resolves its material.
    pub(super) fn resolve(
        self,
        file: &str,
        material: impl Fn(&str) -> Option<MaterialId>,
    ) -> Result<ObjectDef, ContentError> {
        let err = |m: String| ContentError::new(file, format!("object `{}`: {m}", self.id));
        if self.name.trim().is_empty() || self.plural.as_ref().is_some_and(|p| p.trim().is_empty())
        {
            return Err(err("`name` and `plural` must not be empty".into()));
        }
        if self.glyph.is_whitespace() || self.glyph.is_control() {
            return Err(err("`glyph` must be a visible character".into()));
        }
        if self.weight == 0 {
            return Err(err(
                "`weight` is in grams and must be at least 1; every object weighs something".into(),
            ));
        }
        let material = material(&self.material)
            .ok_or_else(|| err(format!("unknown material `{}`", self.material)))?;
        let plural = self.plural.unwrap_or_else(|| format!("{}s", self.name));
        Ok(ObjectDef {
            id: self.id,
            name: self.name,
            plural,
            glyph: self.glyph,
            fg: self.fg,
            material,
            weight: self.weight,
            stackable: self.stackable,
        })
    }
}

/// Parses objects as level files place them: ids, each optionally with a count (`"arrow*12"`),
/// first on top. A stackable object becomes one stack (the same id twice joins it); any other
/// object becomes a stack of one per count.
pub(super) fn parse_stacks(specs: &[String], content: &Content) -> Result<Vec<Stack>, String> {
    let mut stacks: Vec<Stack> = Vec::new();
    for spec in specs {
        let (id, count) = match spec.split_once('*') {
            Some((id, n)) => {
                let count = n
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|n| (1..=MAX_PLACED).contains(n))
                    .ok_or_else(|| {
                        format!("`{spec}`: the count must be a number from 1 to {MAX_PLACED}")
                    })?;
                (id.trim(), count)
            }
            None => (spec.trim(), 1),
        };
        let object = content
            .object_id(id)
            .ok_or_else(|| format!("unknown object `{id}`"))?;
        if content.object(object).stackable {
            match stacks.iter_mut().find(|s| s.object == object) {
                Some(stack) => stack.count += count,
                None => stacks.push(Stack { object, count }),
            }
        } else {
            stacks.extend((0..count).map(|_| Stack { object, count: 1 }));
        }
    }
    Ok(stacks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn articles_and_plurals() {
        let def = |name: &str, plural: &str| ObjectDef {
            id: name.into(),
            name: name.into(),
            plural: plural.into(),
            glyph: ')',
            fg: Rgb::WHITE,
            material: MaterialId(0),
            weight: 1,
            stackable: false,
        };
        assert_eq!(def("dagger", "daggers").describe(1), "a dagger");
        assert_eq!(def("iron key", "iron keys").describe(1), "an iron key");
        assert_eq!(def("arrow", "arrows").describe(12), "12 arrows");
        assert_eq!(
            def("scroll of light", "scrolls of light").describe(2),
            "2 scrolls of light"
        );
    }
}
