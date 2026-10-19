//! Classes (`assets/data/classes.toml`, spec §8 "Character creation"): the third choice of a new
//! run. A class adjusts the starting scores, by fixed numbers or dice, adds hit points, trains
//! the character in skills (spec §8 "Skills"), by dice, and gives starting ranks in weapon skills
//! (spec §8 "Weapon skills") and fighting styles (spec §8 "Fighting styles"): some always, and
//! perhaps one set of a choice, rolled when the character is made.

use super::weapon_skill::Ranked;
use super::{ContentError, WeaponSkillId, WeaponSkillTable};
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
    #[serde(default)]
    weapon_skills: BTreeMap<String, Dice>,
    #[serde(default)]
    weapon_skill_choice: Vec<BTreeMap<String, Dice>>,
    #[serde(default)]
    styles: BTreeMap<String, Dice>,
    #[serde(default)]
    style_choice: Vec<BTreeMap<String, Dice>>,
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
    /// Starting ranks in weapon skills (spec §8 "Weapon skills"), in skill order: added to the
    /// race's.
    pub weapon_skills: Vec<(WeaponSkillId, Dice)>,
    /// Sets of starting ranks of which one is picked at random (the barbarian: axes or great
    /// blades); empty for none.
    pub weapon_skill_choice: Vec<Vec<(WeaponSkillId, Dice)>>,
    /// Starting ranks in fighting styles (spec §8 "Fighting styles"), in skill order.
    pub styles: Vec<(WeaponSkillId, Dice)>,
    /// Sets of starting ranks in fighting styles of which one is picked at random (the thief:
    /// Twin Blades or Duelist); empty for none.
    pub style_choice: Vec<Vec<(WeaponSkillId, Dice)>>,
}

impl ClassDef {
    /// Parses `data/classes.toml`: the classes in file order; `skill_id` says which skill a
    /// skill id is, and `weapon_table` holds the weapon skills starting ranks may name.
    pub(super) fn parse(
        file: &str,
        text: &str,
        skill_id: impl Fn(&str) -> Option<SkillId>,
        weapon_table: &WeaponSkillTable,
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
            let ranks = |data: &BTreeMap<String, Dice>, ranked| {
                (weapon_table.starting_ranks(data, ranked))
                    .map_err(|m| err(format!("class `{id}`: {m}")))
            };
            let choice = |field: &str, sets: &[BTreeMap<String, Dice>], ranked| {
                let sets = (sets.iter())
                    .map(|set| ranks(set, ranked))
                    .collect::<Result<Vec<_>, _>>()?;
                if sets.len() == 1 || sets.iter().any(Vec::is_empty) {
                    return Err(err(format!(
                        "class `{id}`: `{field}` is a choice of two or more sets of ranks"
                    )));
                }
                Ok(sets)
            };
            let weapon_skills = ranks(&spec.weapon_skills, Ranked::Weapons)?;
            let weapon_skill_choice = choice(
                "weapon_skill_choice",
                &spec.weapon_skill_choice,
                Ranked::Weapons,
            )?;
            let styles = ranks(&spec.styles, Ranked::Styles)?;
            let style_choice = choice("style_choice", &spec.style_choice, Ranked::Styles)?;
            list.push(ClassDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                stats,
                hp: i32::from(spec.hp),
                skills,
                weapon_skills,
                weapon_skill_choice,
                styles,
                style_choice,
            });
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = "[[class]]\nid = \"wizard\"\nname = \"Wizard\"\ndescription = \"Books.\"\n";

    /// Parses `text` with the built-in weapon skills and one skill, `spellcraft`.
    fn parse(text: &str) -> Result<Vec<ClassDef>, ContentError> {
        let content = super::super::Content::builtin().expect("content");
        let skill = |k: &str| (k == "spellcraft").then_some(SkillId(0));
        ClassDef::parse("c", text, skill, content.weapon_skills())
    }

    #[test]
    fn classes_read_adjustments_and_refuse_bad_ones() {
        let text = format!(
            "{ONE}stats = {{ intelligence = \"2d2+1\", toughness = \"-1d2\", willpower = 1 }}\n"
        );
        let classes = parse(&text).expect("valid");
        assert!(classes[0].skills.is_empty());
        let text = format!("{ONE}skills = {{ spellcraft = \"2d6+10\" }}\n");
        let wizard = parse(&text).expect("valid");
        assert_eq!(
            wizard[0].skills,
            [(SkillId(0), "2d6+10".parse().expect("dice"))]
        );
        assert_eq!(classes[0].stats[Stat::Intelligence].to_string(), "2d2+1");
        assert_eq!(classes[0].stats[Stat::Toughness].to_string(), "-1d2");
        assert_eq!(classes[0].stats[Stat::Willpower], Dice::fixed(1));
        assert!(classes[0].stats[Stat::Strength].is_zero());
        assert!(classes[0].weapon_skills.is_empty() && classes[0].weapon_skill_choice.is_empty());
        let text = format!(
            "{ONE}weapon_skills = {{ staves = 1 }}\n\
             weapon_skill_choice = [{{ daggers = \"1d2\" }}, {{ slings = 1, thrown = 1 }}]\n"
        );
        let wizard = &parse(&text).expect("valid")[0];
        assert_eq!(wizard.weapon_skills.len(), 1);
        assert_eq!(wizard.weapon_skill_choice.len(), 2);
        assert_eq!(wizard.weapon_skill_choice[1].len(), 2);
        assert!(wizard.styles.is_empty() && wizard.style_choice.is_empty());
        let text = format!(
            "{ONE}styles = {{ staff_fighter = 1 }}\n\
             style_choice = [{{ duelist = \"1d2\" }}, {{ brawler = 1 }}]\n"
        );
        let wizard = &parse(&text).expect("valid")[0];
        assert_eq!(wizard.styles.len(), 1);
        assert_eq!(wizard.style_choice.len(), 2);
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
            (
                format!("{ONE}weapon_skills = {{ wands = 1 }}\n"),
                "class `wizard`: unknown weapon skill `wands`",
            ),
            (
                format!("{ONE}weapon_skill_choice = [{{ staves = 1 }}]\n"),
                "a choice of two or more",
            ),
            (
                format!("{ONE}weapon_skill_choice = [{{ staves = 1 }}, {{}}]\n"),
                "a choice of two or more",
            ),
            (
                format!("{ONE}weapon_skills = {{ duelist = 1 }}\n"),
                "`duelist` is a fighting style: classes give styles in `styles`",
            ),
            (
                format!("{ONE}styles = {{ staves = 1 }}\n"),
                "`staves` is a weapon skill, not a fighting style",
            ),
            (
                format!("{ONE}style_choice = [{{ duelist = 1 }}]\n"),
                "`style_choice` is a choice of two or more",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
