//! Prestige classes (`assets/data/prestige/`, one file per class, spec §8 "Prestige classes"):
//! a second class table a character's levels go into once their own class is full
//! (`ClassDef::max_level`) and they have been **called** ([`super::CallingDef`]). A prestige
//! class is a class in everything a level reads (hit points, mana, skill rolls, the skills it is
//! known for, bonus picks, and its level table of class abilities) and nothing a creation reads:
//! no stats, kit, skills, weapon skills, experience factor, or alignment. Its levels
//! are its own, 1 to `levels`: total level `max_level + n` of the base class is its level `n`
//! ([`crate::character::Character::class_of`]).
//!
//! [`Table`] is the one view of either kind of class the per-level rules read, so a level is
//! handed out the same way whichever class it falls in.

use super::class::{self, BonusTalents, ClassDef};
use super::{ContentError, TalentBook, TalentId, TreeId, duplicate_id};
use crate::skills::SkillId;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The most levels a prestige class may have.
pub const MAX_PRESTIGE_LEVELS: u8 = 60;

fn two_rolls() -> u8 {
    2
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrestigeFile {
    #[serde(default)]
    prestige: Vec<PrestigeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrestigeSpec {
    id: String,
    name: String,
    description: String,
    levels: u8,
    #[serde(default)]
    hp: u16,
    #[serde(default)]
    mana: u16,
    #[serde(default = "two_rolls")]
    rolls: u8,
    #[serde(default)]
    known_for: Vec<String>,
    #[serde(default)]
    abilities: Vec<class::AbilitySpec>,
    bonus_talents: Option<class::BonusSpec>,
}

/// A prestige class (spec §8 "Prestige classes").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrestigeDef {
    pub id: String,
    pub name: String,
    /// One line, for the wiki and the character screen.
    pub description: String,
    /// How many levels it has (1 to [`MAX_PRESTIGE_LEVELS`]): a character's cap grows by this
    /// many when called.
    pub levels: u8,
    /// What each of its levels adds to the hit points, as a class's `hp` (half of it a level).
    pub hp: i32,
    /// What each of its levels adds to the mana, as a class's `mana` (half of it a level).
    pub mana: i32,
    /// The skill rolls each of its levels gives, before the race's and Intelligence's.
    pub rolls: u8,
    /// The skills it is known for (spec §8 "Skills"), in skill order: a level-up roll on one
    /// throws the die one step up the ladder, as the base class's do.
    pub known_for: Vec<SkillId>,
    /// Its class abilities: at each of its own levels, the talent it gives, by id, ascending
    /// by level.
    pub abilities: Vec<(u8, String)>,
    /// Its bonus talent picks, at its own levels.
    pub bonus_talents: BonusTalents,
    /// The file it was read from, for the errors of [`PrestigeDef::check_table`].
    pub(super) file: String,
}

impl PrestigeDef {
    /// Parses the prestige table (`data/prestige/*.toml`), given as its files in path order;
    /// `skill_id` says which skill a skill id is, and `is_class` whether an id is a class's (a
    /// prestige class and a class never share one).
    pub(super) fn parse<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
        skill_id: impl Fn(&str) -> Option<SkillId>,
        is_class: impl Fn(&str) -> bool,
    ) -> Result<Vec<Self>, ContentError> {
        let mut list: Vec<PrestigeDef> = Vec::new();
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (file, text) in files {
            let err = |m: String| ContentError::new(file, m);
            let parsed: PrestigeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
            for spec in parsed.prestige {
                if let Some(first) = seen.insert(spec.id.clone(), file) {
                    return Err(duplicate_id("prestige", &spec.id, file, first));
                }
                if is_class(&spec.id) {
                    return Err(err(format!(
                        "prestige `{}`: a class has this id already",
                        spec.id
                    )));
                }
                list.push(Self::resolve(file, spec, &skill_id)?);
            }
        }
        Ok(list)
    }

    fn resolve(
        file: &str,
        spec: PrestigeSpec,
        skill_id: &impl Fn(&str) -> Option<SkillId>,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let what = format!("prestige `{}`", spec.id);
        if !super::dialogue::is_id(&spec.id) {
            return Err(err(format!(
                "{what}: ids are lower-case letters, digits, and underscores"
            )));
        }
        if !(1..=MAX_PRESTIGE_LEVELS).contains(&spec.levels) {
            return Err(err(format!(
                "{what}: levels {} is not 1 to {MAX_PRESTIGE_LEVELS}",
                spec.levels
            )));
        }
        let mut known_for = Vec::with_capacity(spec.known_for.len());
        for name in &spec.known_for {
            let skill = skill_id(name)
                .ok_or_else(|| err(format!("{what}: known_for: unknown skill `{name}`")))?;
            known_for.push(skill);
        }
        known_for.sort();
        let abilities = class::table_rows(spec.abilities);
        let bonus_talents = class::bonus_rows(spec.bonus_talents);
        Ok(PrestigeDef {
            id: spec.id,
            name: spec.name.trim().to_string(),
            description: spec.description,
            levels: spec.levels,
            hp: i32::from(spec.hp),
            mana: i32::from(spec.mana),
            rolls: spec.rolls,
            known_for,
            abilities,
            bonus_talents,
            file: file.to_string(),
        })
    }

    /// Checks its level table against `book` (once the talents are loaded), as a class's is.
    pub(super) fn check_table(&self, book: &TalentBook) -> Result<(), ContentError> {
        let err = |m: String| ContentError::new(&self.file, format!("prestige `{}`: {m}", self.id));
        class::check_rows(&err, &self.abilities, &self.bonus_talents, book)
    }
}

/// A class table a level reads (spec §8 "Prestige classes"): the character's own class, or
/// their prestige class once called. Every per-level rule (hit points, mana, skill rolls, the
/// known-for step, bonus picks, class abilities) reads the class of the level through this, so
/// neither kind is a special case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table<'c> {
    Class(&'c ClassDef),
    Prestige(&'c PrestigeDef),
}

impl<'c> Table<'c> {
    pub fn id(self) -> &'c str {
        match self {
            Table::Class(c) => &c.id,
            Table::Prestige(p) => &p.id,
        }
    }

    pub fn name(self) -> &'c str {
        match self {
            Table::Class(c) => &c.name,
            Table::Prestige(p) => &p.name,
        }
    }

    /// Whether it is a prestige class's table.
    pub fn is_prestige(self) -> bool {
        matches!(self, Table::Prestige(_))
    }

    /// What one of its levels adds to the hit points before halving (a class's `hp`).
    pub fn hp(self) -> i32 {
        match self {
            Table::Class(c) => c.hp,
            Table::Prestige(p) => p.hp,
        }
    }

    /// What one of its levels adds to the mana before halving (a class's `mana`).
    pub fn mana(self) -> i32 {
        match self {
            Table::Class(c) => c.mana,
            Table::Prestige(p) => p.mana,
        }
    }

    /// The skill rolls one of its levels gives.
    pub fn rolls(self) -> u8 {
        match self {
            Table::Class(c) => c.rolls,
            Table::Prestige(p) => p.rolls,
        }
    }

    /// Whether it is known for skill `skill`.
    pub fn is_known_for(self, skill: SkillId) -> bool {
        match self {
            Table::Class(c) => c.is_known_for(skill),
            Table::Prestige(p) => p.known_for.contains(&skill),
        }
    }

    /// Its level table: at each of its own levels, the talent it gives, ascending.
    pub fn abilities(self) -> &'c [(u8, String)] {
        match self {
            Table::Class(c) => &c.abilities,
            Table::Prestige(p) => &p.abilities,
        }
    }

    /// Its bonus talent picks.
    pub fn bonus_talents(self) -> &'c BonusTalents {
        match self {
            Table::Class(c) => &c.bonus_talents,
            Table::Prestige(p) => &p.bonus_talents,
        }
    }

    /// The talents its table gives up to its own level `level`, in table order; none from a
    /// book without a class tree (a fixture's).
    pub fn abilities_to(self, book: &TalentBook, level: u8) -> Vec<TalentId> {
        if !book.has_class_tree() {
            return Vec::new();
        }
        (self.abilities().iter())
            .take_while(|(l, _)| *l <= level)
            .filter_map(|(_, id)| book.id(id))
            .collect()
    }

    /// The bonus talent picks its own level `level` brings (0 or 1).
    pub fn bonus_at(self, level: u8) -> u8 {
        u8::from(self.bonus_talents().levels.contains(&level))
    }

    /// The trees of `book` its bonus picks come from, in display order; `None`: any.
    pub fn bonus_trees(self, book: &TalentBook) -> Option<Vec<TreeId>> {
        let bonus = self.bonus_talents();
        if bonus.trees.is_empty() {
            return None;
        }
        let mut trees: Vec<TreeId> = (bonus.trees.iter())
            .filter_map(|t| book.tree_id(t))
            .collect();
        trees.sort();
        Some(trees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str =
        "[[prestige]]\nid = \"hunter\"\nname = \"Hunter\"\ndescription = \"Sworn.\"\n";

    fn parse(text: &str) -> Result<Vec<PrestigeDef>, ContentError> {
        let skill = |k: &str| match k {
            "athletics" => Some(SkillId(0)),
            "survival" => Some(SkillId(1)),
            _ => None,
        };
        PrestigeDef::parse([("p", text)], skill, |id| id == "bard")
    }

    #[test]
    fn a_prestige_class_reads_its_table_and_defaults() {
        let text = format!(
            "{ONE}levels = 30\nhp = 7\nknown_for = [\"survival\", \"athletics\"]\n\
             bonus_talents = {{ levels = [5, 10], trees = [\"guard\"] }}\n\
             abilities = [{{ level = 1, talent = \"spirit_weapon\" }}]\n"
        );
        let list = parse(&text).expect("valid");
        let p = &list[0];
        assert_eq!((p.levels, p.hp, p.mana, p.rolls), (30, 7, 0, 2));
        assert_eq!(p.known_for, vec![SkillId(0), SkillId(1)], "in skill order");
        assert_eq!(p.abilities, vec![(1, "spirit_weapon".to_string())]);
        assert_eq!(p.bonus_talents.levels, vec![5, 10]);
        let table = Table::Prestige(p);
        assert!(table.is_prestige() && table.is_known_for(SkillId(1)));
        assert_eq!((table.bonus_at(5), table.bonus_at(6)), (1, 0));
        assert_eq!((table.hp(), table.rolls(), table.name()), (7, 2, "Hunter"));
    }

    #[test]
    fn a_prestige_class_refuses_bad_tables() {
        for (text, why) in [
            (format!("{ONE}levels = 0\n"), "levels 0 is not 1 to 60"),
            (format!("{ONE}levels = 61\n"), "levels 61 is not 1 to 60"),
            (
                format!("{ONE}levels = 10\nknown_for = [\"stealth\"]\n"),
                "unknown skill `stealth`",
            ),
            (format!("{ONE}levels = 10\nstats = {{}}\n"), "unknown field"),
        ] {
            let err = parse(&text).expect_err(why);
            assert!(err.to_string().contains(why), "{why}: {err}");
        }
        let bard = ONE.replace("hunter", "bard");
        let err = parse(&format!("{bard}levels = 10\n")).expect_err("a class's id");
        assert!(err.to_string().contains("a class has this id"), "{err}");
    }
}
