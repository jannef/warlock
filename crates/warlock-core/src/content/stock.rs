//! Material stock (`assets/data/stock.toml`; the design: docs/design/professions.md §3, §4): what a
//! piece of each material group is called, and the profession that works it.
//!
//! The stock is the one [`crate::piece`] object, by weight; this file gives each group its form
//! word ("a steel ingot", "3 hardwood billets", "a bolt of wool") and, where a profession works
//! the group, its skill, the stat its checks are made by, and the verb its menu row uses, and its
//! `unit`, how many pieces' worth one of its pieces is (a hide is eight: 1.4 kg of leather). The
//! gems (no group) have a profession of their own, `gems`. `[[material]]` rows override one
//! material's profession or form word (flint is knapped, Woodcraft's, though stone is
//! Jewelcraft's; fur comes in pelts). The file is optional: without it every piece is "a piece
//! of X" and no material has a profession. Nothing in code names a group or a skill:
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
    /// How many pieces' worth one piece of the group is (default 1, at least 1): a piece weighs
    /// and is worth `unit` times what 500 g of steel would in its material (hide 8, cloth 4).
    pub unit: u32,
}

/// One material's own row, over its group's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockMaterial {
    pub material: MaterialId,
    /// What works it instead of its group's profession: the skill's stat and verb as the file's
    /// other rows name them.
    pub profession: Option<Profession>,
    /// Its own form word, singular and plural, instead of its group's ("pelt"/"pelts").
    pub name: Option<(String, String)>,
}

/// The stock table: the groups in file order, the gems' profession, and the materials' own rows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stock {
    pub groups: Vec<StockGroup>,
    pub gems: Option<Profession>,
    pub materials: Vec<StockMaterial>,
}

impl Stock {
    /// The row of `group`, if the file has one.
    pub fn group(&self, group: &str) -> Option<&StockGroup> {
        self.groups.iter().find(|g| g.group == group)
    }

    /// The material's own row, if the file has one.
    pub fn material(&self, material: MaterialId) -> Option<&StockMaterial> {
        self.materials.iter().find(|m| m.material == material)
    }

    /// The row of `material`'s group, if it has a group and the file a row for it.
    pub fn group_of(&self, content: &Content, material: MaterialId) -> Option<&StockGroup> {
        self.group(content.material(material).group.as_deref()?)
    }

    /// What works `material`: its own row's profession if it names one, else the gems'
    /// profession for a gem, else its group's; none for a material with no group, a group
    /// without a row, or a row without a profession.
    pub fn profession_of(&self, content: &Content, material: MaterialId) -> Option<&Profession> {
        if let Some(own) = self.material(material).and_then(|m| m.profession.as_ref()) {
            return Some(own);
        }
        if content.material(material).gem.is_some() {
            return self.gems.as_ref();
        }
        self.group_of(content, material)?.profession.as_ref()
    }

    /// The form word of a piece of `material`, singular and plural, and whether the material
    /// comes after it ("lump of amber"): its own row's word if it has one, else its group's;
    /// none for a material whose group has no row.
    pub fn form_of(&self, content: &Content, material: MaterialId) -> Option<(&str, &str, bool)> {
        let group = self.group_of(content, material)?;
        let (name, plural) = match self.material(material).and_then(|m| m.name.as_ref()) {
            Some((name, plural)) => (name.as_str(), plural.as_str()),
            None => (group.name.as_str(), group.plural.as_str()),
        };
        Some((name, plural, group.of))
    }

    /// How many pieces' worth one piece of `material` is: its group's `unit`, 1 without a row.
    pub fn unit_of(&self, content: &Content, material: MaterialId) -> u32 {
        self.group_of(content, material).map_or(1, |g| g.unit)
    }

    /// Parses `data/stock.toml`: each group once and one some material has, non-empty words,
    /// a `unit` of at least 1, `skill`, `stat`, and `verb` all three or none, known skills, and
    /// no skill worked by two stats; each material row once, of a known material with a group,
    /// naming a skill some group or the gems name (its stat and verb are theirs) or a form word
    /// (`name` and `plural` together), or both. Without `data/skills.toml` the professions are
    /// left out.
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
            if row.unit == 0 {
                return Err(err(format!("{of}: `unit` must be at least 1")));
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
                unit: row.unit,
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
        for row in parsed.material {
            let of = format!("material `{}`", row.material);
            let material = (content.material_id(&row.material))
                .ok_or_else(|| err(format!("{of}: unknown material")))?;
            if content.material(material).group.is_none() {
                return Err(err(format!("{of} has no group")));
            }
            if stock.material(material).is_some() {
                return Err(err(format!("{of} is listed twice")));
            }
            let name = match (row.name, row.plural) {
                (None, None) => None,
                (Some(name), Some(plural)) if !name.is_empty() && !plural.is_empty() => {
                    Some((name, plural))
                }
                _ => {
                    return Err(err(format!(
                        "{of}: `name` and `plural` go together, neither empty"
                    )));
                }
            };
            if row.skill.is_none() && name.is_none() {
                return Err(err(format!("{of}: names neither a skill nor a form word")));
            }
            let profession = match row.skill {
                // Content without skills (a fixture) has no professions.
                Some(skill) if skills => {
                    let id = (content.skill_id(&skill))
                        .ok_or_else(|| err(format!("{of}: unknown skill `{skill}`")))?;
                    let named = (stock.groups.iter().filter_map(|g| g.profession.as_ref()))
                        .chain(&stock.gems)
                        .find(|p| p.skill == id);
                    Some(named.cloned().ok_or_else(|| {
                        err(format!(
                            "{of}: skill `{skill}` works no group, so it has no stat or verb"
                        ))
                    })?)
                }
                _ => None,
            };
            stock.materials.push(StockMaterial {
                material,
                profession,
                name,
            });
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
    #[serde(default)]
    material: Vec<MaterialSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialSpec {
    material: String,
    #[serde(default)]
    skill: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    plural: Option<String>,
}

fn one() -> u32 {
    1
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
    #[serde(default = "one")]
    unit: u32,
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
        assert_eq!(of("sinew"), Some(("woodcraft", Stat::Dexterity, "Make")));
        // Jewelcraft cuts jade, amber, and the gems; flint, stone, and obsidian are knapped by
        // hand, Woodcraft's by their own rows, with Woodcraft's stat and verb.
        for cut in ["jade", "amber", "ruby"] {
            assert_eq!(
                of(cut),
                Some(("jewelcraft", Stat::Dexterity, "Cut")),
                "{cut}"
            );
        }
        for knapped in ["flint", "stone", "obsidian"] {
            assert_eq!(
                of(knapped),
                Some(("woodcraft", Stat::Dexterity, "Make")),
                "{knapped}"
            );
        }
        // No profession takes cloth or glass yet; paper has no group.
        for none in ["wool", "glass", "paper"] {
            assert_eq!(of(none), None, "{none}");
        }
    }

    #[test]
    fn units_and_form_words() {
        let content = content();
        let stock = content.stock();
        let m = |id: &str| content.material_id(id).expect(id);
        let units: Vec<u32> = (["steel", "leather", "fur", "wool", "sinew", "paper"].iter())
            .map(|id| stock.unit_of(&content, m(id)))
            .collect();
        assert_eq!(units, [1, 8, 8, 4, 1, 1]);
        assert_eq!(
            stock.form_of(&content, m("fur")),
            Some(("pelt", "pelts", false))
        );
        assert_eq!(
            stock.form_of(&content, m("leather")),
            Some(("hide", "hides", false))
        );
        assert_eq!(
            stock.form_of(&content, m("sinew")),
            Some(("length", "lengths", true))
        );
        assert_eq!(stock.form_of(&content, m("paper")), None);
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
            ("unit = 8", "unit = 0", "at least 1"),
            ("unit = 8", "unit = -1", "invalid value"),
            (
                "material = \"flint\"",
                "material = \"flintstone\"",
                "unknown material",
            ),
            (
                "material = \"flint\"",
                "material = \"paper\"",
                "has no group",
            ),
            ("material = \"flint\"", "material = \"fur\"", "listed twice"),
            (
                "material = \"flint\"\nskill = \"woodcraft\"",
                "material = \"flint\"\nskill = \"heal\"",
                "works no group",
            ),
            (
                "material = \"flint\"\nskill = \"woodcraft\"",
                "material = \"flint\"\nskill = \"knapping\"",
                "unknown skill",
            ),
            (
                "material = \"flint\"\nskill = \"woodcraft\"",
                "material = \"flint\"",
                "names neither",
            ),
            ("plural = \"pelts\"", "#", "go together"),
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
        let line = "gems = { skill = \"jewelcraft\", stat = \"dexterity\", verb = \"Cut\" }";
        assert!(TEXT.contains(line));
        let gems = |stat: &str| {
            let to =
                format!("gems = {{ skill = \"metalcraft\", stat = \"{stat}\", verb = \"Cut\" }}");
            TEXT.replace(line, &to)
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
