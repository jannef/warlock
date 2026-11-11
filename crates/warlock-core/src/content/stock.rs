//! Material stock (`assets/data/stock.toml`; the design: docs/design/professions.md §3, §4): what a
//! piece of each material group is called, and the profession that works it.
//!
//! The stock is the one [`crate::piece`] object, by weight; this file gives each group its form
//! word ("a steel ingot", "3 hardwood billets", "a bolt of wool") and, where a profession works
//! the group, its skill, the stat its checks are made by, and the verb its menu row uses. The gems
//! (no group) have a profession of their own, `gems`. The file is optional: without it every piece
//! is "a piece of X" and no material has a profession. Nothing in code names a group or a skill:
//! [`Stock::profession_of`] reads this table.

use super::{Content, ContentError, MaterialId, SkillId};
use crate::stats::Stat;
use serde::Deserialize;

/// The skill and stat that work a material, and the verb of its menu row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profession {
    pub skill: SkillId,
    pub stat: Stat,
    /// The verb its rows and dialogs use ("Forge", "Make", "Cut").
    pub verb: String,
}

/// One group's stock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockGroup {
    /// The material group (`metal`, `wood`, ...; materials/README.toml).
    pub group: String,
    /// The form word, singular and plural: "ingot"/"ingots".
    pub name: String,
    pub plural: String,
    /// The material comes after the word ("a lump of amber"), not before ("a steel ingot").
    pub of: bool,
    /// What works the group; none for a group no profession takes yet (cloth, glass).
    pub profession: Option<Profession>,
}

/// The stock table: the groups in file order, and the gems' profession.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stock {
    pub groups: Vec<StockGroup>,
    pub gems: Option<Profession>,
}

impl Stock {
    /// The row of `group`, if the file has one.
    pub fn group(&self, group: &str) -> Option<&StockGroup> {
        self.groups.iter().find(|g| g.group == group)
    }

    /// What works `material`: the gems' profession for a gem, else its group's; none for a
    /// material with no group, a group without a row, or a row without a profession.
    pub fn profession_of(&self, content: &Content, material: MaterialId) -> Option<&Profession> {
        let def = content.material(material);
        if def.gem.is_some() {
            return self.gems.as_ref();
        }
        self.group(def.group.as_deref()?)?.profession.as_ref()
    }

    /// Parses `data/stock.toml`: each group once and one some material has, non-empty words,
    /// `skill`, `stat`, and `verb` all three or none, known skills, and no skill worked by two
    /// stats. Without `data/skills.toml` the professions are left out.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: StockFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let mut stock = Stock::default();
        // Content without skills (a fixture) has no professions.
        let skills = !content.skills().is_empty();
        let profession = |spec: &ProfessionSpec, of: &str| -> Result<Profession, ContentError> {
            let skill = (content.skill_id(&spec.skill))
                .ok_or_else(|| err(format!("{of}: unknown skill `{}`", spec.skill)))?;
            if spec.verb.is_empty() {
                return Err(err(format!("{of}: `verb` is empty")));
            }
            Ok(Profession {
                skill,
                stat: spec.stat,
                verb: spec.verb.clone(),
            })
        };
        for row in parsed.group {
            let of = format!("group `{}`", row.group);
            if stock.group(&row.group).is_some() {
                return Err(err(format!("{of} is listed twice")));
            }
            if !(content.materials().iter()).any(|m| m.group.as_deref() == Some(&row.group)) {
                return Err(err(format!("{of}: no material is of it")));
            }
            if row.name.is_empty() || row.plural.is_empty() {
                return Err(err(format!("{of}: `name` and `plural` must not be empty")));
            }
            let profession = match (row.skill, row.stat, row.verb) {
                (None, None, None) => None,
                (Some(skill), Some(stat), Some(verb)) if skills => {
                    Some(profession(&ProfessionSpec { skill, stat, verb }, &of)?)
                }
                (Some(_), Some(_), Some(_)) => None,
                _ => {
                    return Err(err(format!(
                        "{of}: `skill`, `stat`, and `verb` go together"
                    )));
                }
            };
            stock.groups.push(StockGroup {
                group: row.group,
                name: row.name,
                plural: row.plural,
                of: row.of,
                profession,
            });
        }
        if let Some(gems) = parsed.gems.as_ref().filter(|_| skills) {
            stock.gems = Some(profession(gems, "`gems`")?);
        }
        let all = (stock.groups.iter().filter_map(|g| g.profession.as_ref())).chain(&stock.gems);
        let mut seen: Vec<&Profession> = Vec::new();
        for p in all {
            if let Some(other) = seen.iter().find(|o| o.skill == p.skill && o.stat != p.stat) {
                return Err(err(format!(
                    "skill `{}` is named with two stats, `{}` and `{}`",
                    content.skill(p.skill).id,
                    other.stat.id(),
                    p.stat.id()
                )));
            }
            seen.push(p);
        }
        Ok(stock)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StockFile {
    #[serde(default)]
    gems: Option<ProfessionSpec>,
    #[serde(default)]
    group: Vec<GroupSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfessionSpec {
    skill: String,
    stat: Stat,
    verb: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupSpec {
    group: String,
    name: String,
    plural: String,
    #[serde(default)]
    of: bool,
    #[serde(default)]
    skill: Option<String>,
    #[serde(default)]
    stat: Option<Stat>,
    #[serde(default)]
    verb: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};
    use crate::stats::Stat;

    const TEXT: &str = include_str!("../../../../assets/data/stock.toml");

    fn content() -> Content {
        Content::builtin().expect("content")
    }

    #[test]
    fn the_groups_name_their_professions() {
        let content = content();
        let stock = content.stock();
        let of = |id: &str| {
            let m = content.material_id(id).expect(id);
            (stock.profession_of(&content, m))
                .map(|p| (content.skill(p.skill).id.as_str(), p.stat, p.verb.as_str()))
        };
        assert_eq!(of("steel"), Some(("metalcraft", Stat::Strength, "Forge")));
        assert_eq!(of("hardwood"), Some(("woodcraft", Stat::Dexterity, "Make")));
        assert_eq!(of("horn"), Some(("woodcraft", Stat::Dexterity, "Make")));
        assert_eq!(of("leather"), Some(("woodcraft", Stat::Dexterity, "Make")));
        // Jewelcraft takes stone, amber, and the gems.
        for id in ["jade", "amber", "ruby"] {
            assert_eq!(of(id), Some(("jewelcraft", Stat::Dexterity, "Cut")), "{id}");
        }
        // No profession takes cloth or glass yet; paper has no group.
        for none in ["wool", "glass", "paper"] {
            assert_eq!(of(none), None, "{none}");
        }
    }

    #[test]
    fn bad_files_are_refused() {
        for (from, to, says) in [
            (
                "skill = \"metalcraft\"",
                "skill = \"smithing\"",
                "unknown skill",
            ),
            ("group = \"wood\"", "group = \"metal\"", "listed twice"),
            (
                "group = \"wood\"",
                "group = \"twigs\"",
                "no material is of it",
            ),
            ("name = \"ingot\"", "name = \"\"", "must not be empty"),
            ("verb = \"Forge\"", "#", "go together"),
            ("stat = \"dexterity\"", "stat = \"strength\"", "two stats"),
            ("stat = \"strength\"", "stat = \"brawn\"", "unknown stat"),
        ] {
            let bad = TEXT.replacen(from, to, 1);
            assert_ne!(bad, TEXT, "{from}");
            let mut files = ContentFiles::builtin();
            files.insert("data/stock.toml", &bad);
            let error = Content::load(&files).expect_err(from).to_string();
            assert!(error.contains(says), "{from}: {error}");
        }
    }

    #[test]
    fn the_gems_have_a_profession_of_their_own() {
        const OURS: &str =
            "gems = { skill = \"jewelcraft\", stat = \"dexterity\", verb = \"Cut\" }";
        assert!(TEXT.contains(OURS));
        let gems = |stat: &str| {
            let line =
                format!("gems = {{ skill = \"metalcraft\", stat = \"{stat}\", verb = \"Cut\" }}");
            TEXT.replacen(OURS, &line, 1)
        };
        let text = gems("strength");
        let mut files = ContentFiles::builtin();
        files.insert("data/stock.toml", &text);
        let content = Content::load(&files).expect("content");
        let ruby = content.material_id("ruby").expect("ruby");
        let p = content
            .stock()
            .profession_of(&content, ruby)
            .expect("a profession");
        assert_eq!(
            (content.skill(p.skill).id.as_str(), p.verb.as_str()),
            ("metalcraft", "Cut")
        );
        let text = gems("dexterity");
        files.insert("data/stock.toml", &text);
        let error = Content::load(&files).expect_err("two stats").to_string();
        assert!(error.contains("two stats"), "{error}");
    }
}
