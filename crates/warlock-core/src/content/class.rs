//! Classes (`assets/data/classes.toml`, spec §8 "Character creation"): the third choice of a new
//! run. A class adjusts the starting scores, by fixed numbers or dice, adds hit points, and
//! trains the character in skills (spec §8 "Skills"), by dice.

use super::ContentError;
use crate::combat::MAX_BASE_HP;
use crate::dice::Dice;
use crate::skills::{self, SkillId};
use crate::stats::{PerStat, Stat, adjustments};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassFile {
    class: Vec<ClassSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassSpec {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    stats: BTreeMap<Stat, Dice>,
    #[serde(default)]
    hp: u16,
    #[serde(default)]
    skills: BTreeMap<String, Dice>,
}

/// A class, in the order offered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassDef {
    pub id: String,
    pub name: String,
    /// One line, shown when choosing.
    pub description: String,
    /// Adjustments to the starting scores.
    pub stats: PerStat<Dice>,
    /// What the class adds to the hit points (spec §8 "Combat").
    pub hp: i32,
    /// The skills it trains (spec §8 "Skills"), each by dice rolled when the character is made,
    /// in skill order; added to the race's.
    pub skills: Vec<(SkillId, Dice)>,
}

impl ClassDef {
    /// Parses `data/classes.toml`: the classes in file order; `skill_id` says which skill a
    /// skill id is.
    pub(super) fn parse(
        file: &str,
        text: &str,
        skill_id: impl Fn(&str) -> Option<SkillId>,
    ) -> Result<Vec<Self>, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: ClassFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.class.is_empty() {
            return Err(err("at least one class is required".into()));
        }
        let mut list: Vec<ClassDef> = Vec::new();
        for spec in parsed.class {
            let id = &spec.id;
            if list.iter().any(|c| &c.id == id) {
                return Err(err(format!("duplicate class id `{id}`")));
            }
            let stats =
                adjustments(&spec.stats).map_err(|m| err(format!("class `{id}`: stats: {m}")))?;
            if spec.hp > MAX_BASE_HP {
                return Err(err(format!(
                    "class `{id}`: hp {} is more than {MAX_BASE_HP}",
                    spec.hp
                )));
            }
            let skills = skills::grants(&spec.skills, &skill_id)
                .map_err(|m| err(format!("class `{id}`: skills: {m}")))?;
            list.push(ClassDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                stats,
                hp: i32::from(spec.hp),
                skills,
            });
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = "[[class]]\nid = \"wizard\"\nname = \"Wizard\"\ndescription = \"Books.\"\n";

    #[test]
    fn classes_read_adjustments_and_refuse_bad_ones() {
        let text = format!(
            "{ONE}stats = {{ intelligence = \"2d2+1\", toughness = \"-1d2\", willpower = 1 }}\n"
        );
        let skill = |k: &str| (k == "spellcraft").then_some(SkillId(0));
        let classes = ClassDef::parse("c", &text, skill).expect("valid");
        assert!(classes[0].skills.is_empty());
        let text = format!("{ONE}skills = {{ spellcraft = \"2d6+10\" }}\n");
        let wizard = ClassDef::parse("c", &text, skill).expect("valid");
        assert_eq!(
            wizard[0].skills,
            [(SkillId(0), "2d6+10".parse().expect("dice"))]
        );
        assert_eq!(classes[0].stats[Stat::Intelligence].to_string(), "2d2+1");
        assert_eq!(classes[0].stats[Stat::Toughness].to_string(), "-1d2");
        assert_eq!(classes[0].stats[Stat::Willpower], Dice::fixed(1));
        assert!(classes[0].stats[Stat::Strength].is_zero());
        for (text, why) in [
            (
                format!("{ONE}stats = {{ cunning = 1 }}\n"),
                "unknown stat `cunning`",
            ),
            (
                format!("{ONE}stats = {{ presence = -11 }}\n"),
                "presence -11 is out of range",
            ),
            (
                format!("{ONE}stats = {{ presence = \"-3d4\" }}\n"),
                "presence -3d4 is out of range",
            ),
            (format!("{ONE}{ONE}"), "duplicate class id `wizard`"),
            ("class = []\n".into(), "at least one class"),
            (
                format!("{ONE}skills = {{ necromancy = 5 }}\n"),
                "class `wizard`: skills: unknown skill `necromancy`",
            ),
        ] {
            let e = ClassDef::parse("c", &text, skill).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
