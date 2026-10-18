//! Classes (`assets/data/classes.toml`, spec §8 "Character creation"): the third choice of a new
//! run. A class adjusts the starting scores, by fixed numbers or dice, and adds hit points.

use super::ContentError;
use crate::combat::MAX_BASE_HP;
use crate::dice::Dice;
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
}

impl ClassDef {
    /// Parses `data/classes.toml`: the classes in file order.
    pub(super) fn parse(file: &str, text: &str) -> Result<Vec<Self>, ContentError> {
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
            list.push(ClassDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                stats,
                hp: i32::from(spec.hp),
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
        let classes = ClassDef::parse("c", &text).expect("valid");
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
        ] {
            let e = ClassDef::parse("c", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
