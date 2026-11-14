//! What an area spell does to the items lying in it (`assets/data/ruin.toml`, spec §8 "Spells":
//! "Ruin"; #1100): the chance an item is lost, by the spell's damage type and the item's class,
//! the material of its main part, or that material's group, the first row of the type that
//! matches deciding.
//!
//! The rule itself lives in `world/ruin.rs`: which casts ruin, what is spared, the draws, and
//! what the player is told. The file is optional: without it nothing is ruined.

use super::{Content, ContentError, DamageTypeId, ItemClassId, MaterialId};
use crate::object::Stack;
use serde::Deserialize;

/// What a row matches a stack by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuinOf {
    /// Its item class.
    Class(ItemClassId),
    /// The material of its main part.
    Material(MaterialId),
    /// The group of that material (`metal`, `wood`, `cloth`, ...).
    Group(String),
}

/// One row of the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuinRow {
    /// The damage type of the spells it is for.
    pub kind: DamageTypeId,
    pub of: RuinOf,
    /// The chance each item of a matching stack is lost, in percent (100 or more: every one).
    pub percent: u32,
    /// What is said of one item lost, and of several: "burns up", "burn up".
    pub one: String,
    pub many: String,
}

/// The table, in file order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuinRules {
    pub rows: Vec<RuinRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuinFile {
    ruin: Vec<RowSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowSpec {
    #[serde(rename = "type")]
    kind: String,
    class: Option<String>,
    material: Option<String>,
    group: Option<String>,
    percent: u32,
    says: [String; 2],
}

impl RuinRules {
    /// Parses `data/ruin.toml`: every row names a known damage type and exactly one known class,
    /// material, or material group (a group no material has matches nothing).
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RuinFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut rows = Vec::new();
        for (i, row) in parsed.ruin.into_iter().enumerate() {
            let at = |m: String| err(format!("ruin {}: {m}", i + 1));
            let kind = (content.damage_type_id(&row.kind))
                .ok_or_else(|| at(format!("unknown damage type `{}`", row.kind)))?;
            let of = match (row.class, row.material, row.group) {
                (Some(class), None, None) => RuinOf::Class(
                    (content.item_class_id(&class))
                        .ok_or_else(|| at(format!("unknown item class `{class}`")))?,
                ),
                (None, Some(material), None) => RuinOf::Material(
                    (content.material_id(&material))
                        .ok_or_else(|| at(format!("unknown material `{material}`")))?,
                ),
                (None, None, Some(group)) => RuinOf::Group(group),
                _ => return Err(at("name one of `class`, `material`, or `group`".into())),
            };
            let [one, many] = row.says;
            rows.push(RuinRow {
                kind,
                of,
                percent: row.percent,
                one,
                many,
            });
        }
        Ok(Self { rows })
    }

    /// The row that decides what a spell of `kind` does to `stack`: the first of the type that
    /// matches it. None: it is never touched.
    pub fn row(&self, content: &Content, kind: DamageTypeId, stack: &Stack) -> Option<&RuinRow> {
        let class = content.object(stack.object).class;
        let material = stack.parts(content).first().map(|p| p.material);
        let group = material.and_then(|m| content.material(m).group.as_deref());
        (self.rows.iter())
            .filter(|r| r.kind == kind)
            .find(|r| match &r.of {
                RuinOf::Class(c) => *c == class,
                RuinOf::Material(m) => Some(*m) == material,
                RuinOf::Group(g) => Some(g.as_str()) == group,
            })
    }
}

impl super::RulesFile for RuinRules {
    const PATH: &'static str = "data/ruin.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};
    use crate::object::Stack;

    const PATH: &str = "data/ruin.toml";

    #[test]
    fn fire_burns_scrolls_and_spares_steel() {
        let content = crate::test_support::builtin();
        let fire = content.damage_type_id("fire").expect("fire");
        let scroll = content.object_id("scroll_of_fireball").expect("scroll");
        let sword = content.object_id("long_sword").expect("sword");
        let row = (content.ruin().row(&content, fire, &Stack::new(scroll, 1))).expect("a row");
        assert!(row.percent > 0);
        assert_eq!(row.one, "burns up");
        assert!(
            content
                .ruin()
                .row(&content, fire, &Stack::new(sword, 1))
                .is_none()
        );
    }

    #[test]
    fn a_bad_file_is_refused_and_none_ruins_nothing() {
        let row = |body: &str| format!("[[ruin]]\ntype = \"fire\"\n{body}\n");
        for (text, why) in [
            (
                row("class = \"nothing\"\npercent = 1\nsays = [\"a\", \"b\"]"),
                "unknown item class",
            ),
            (
                row("class = \"scrolls\"\ngroup = \"wood\"\npercent = 1\nsays = [\"a\", \"b\"]"),
                "name one of",
            ),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert(PATH, &text);
            let error = Content::load(&files).expect_err(why);
            assert!(error.to_string().contains(why), "{error} lacks {why}");
        }
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("content without the file");
        assert!(content.ruin().rows.is_empty());
    }
}
