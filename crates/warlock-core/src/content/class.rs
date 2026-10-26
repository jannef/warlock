//! Classes (`assets/data/classes/`, one file per class, spec §8 "Character creation"): the third
//! choice of a new run. A class adjusts the starting scores, by fixed numbers or dice, adds hit
//! points and mana (spec §8 "Mana"), sets how much experience its levels need (spec §8
//! "Experience"), trains the character in skills (spec §8 "Skills"), by dice, names the skills it is known for and how many skill
//! rolls each level gives, and gives starting ranks in weapon skills (spec §8 "Weapon skills")
//! and fighting styles (spec §8 "Fighting styles"): some always, and perhaps one set of a
//! choice, rolled when the character is made. Its `kit` is what the character starts with
//! ([`super::kit`]).
//!
//! Its level table (spec §8 "Class abilities") names the **class abilities** each level gives,
//! talents of the class tree (`data/talents/`, given, never picked) or ordinary talents the class
//! hands out, and the levels that bring a **bonus talent pick** from the trees the class names
//! (any tree when it names none). The talents it names are checked once the talents are loaded
//! ([`ClassDef::check_table`]).
//!
//! The classes are offered in path order: the files are named `NN-id.toml`, so the number keeps
//! the order (adventurers first, then everyday folk) and a new class picks its place.

use super::kit::{Kit, KitLineSpec};
use super::weapon_skill::Ranked;
use super::{
    ActionKind, ContentError, FACTORS, MAX_LEVEL, TalentBook, TalentId, TreeId, WeaponSkillId,
    WeaponSkillTable, duplicate_id,
};
use crate::combat::MAX_BASE_HP;
use crate::dice::Dice;
use crate::mana::MAX_BASE_MANA;
use crate::skills::{self, SkillId};
use crate::stats::{PerStat, Stat, adjustments};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn normal_experience() -> u16 {
    100
}

fn two_rolls() -> u8 {
    2
}

fn least_casting() -> u8 {
    25
}

/// The casting percents a class may have (spec §8 "Spells"): everyone casts a little.
pub const CASTING: std::ops::RangeInclusive<u8> = 1..=100;

/// The most skill rolls a class may give a level.
pub const MAX_ROLLS: u8 = 10;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassFile {
    #[serde(default)]
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
    #[serde(default = "normal_experience")]
    experience: u16,
    #[serde(default)]
    alignment: i32,
    #[serde(default)]
    mana: u16,
    #[serde(default = "least_casting")]
    casting: u8,
    #[serde(default)]
    skills: BTreeMap<String, Dice>,
    #[serde(default)]
    known_for: Vec<String>,
    #[serde(default = "two_rolls")]
    rolls: u8,
    #[serde(default)]
    creation_talents: u8,
    #[serde(default)]
    weapon_skills: BTreeMap<String, Dice>,
    #[serde(default)]
    weapon_skill_choice: Vec<BTreeMap<String, Dice>>,
    #[serde(default)]
    styles: BTreeMap<String, Dice>,
    #[serde(default)]
    style_choice: Vec<BTreeMap<String, Dice>>,
    #[serde(default)]
    kit: Vec<KitLineSpec>,
    #[serde(default)]
    abilities: Vec<AbilitySpec>,
    bonus_talents: Option<BonusSpec>,
}

/// `{ level = 6, talent = "fencing" }`: at that level the class gives that talent.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AbilitySpec {
    level: u8,
    talent: String,
}

/// `{ levels = [10, 20], trees = ["guard"] }`: a bonus talent pick at each of those levels,
/// from those trees (none: any).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BonusSpec {
    levels: Vec<u8>,
    #[serde(default)]
    trees: Vec<String>,
}

/// A class's bonus talent picks (spec §8 "Class abilities").
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BonusTalents {
    /// The levels that bring one, ascending.
    pub levels: Vec<u8>,
    /// The trees it comes from, by id; empty: any tree (the commoner).
    pub trees: Vec<String>,
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
    /// What the class adds to the mana (spec §8 "Mana"); half of it comes with each level.
    pub mana: i32,
    /// Its casting percent (spec §8 "Spells"): a character's caster level is this percent of
    /// their level (a wizard 100, a warrior 25), before talents and items.
    pub casting: u8,
    /// How much experience a level needs, in percent (spec §8 "Experience"): multiplied with
    /// the race's onto every threshold. Patient classes pay more (a wizard, 125).
    pub experience: u16,
    /// Its part of a new character's alignment score (spec §8 "Alignment"), added to the
    /// race's: a paladin starts Good.
    pub alignment: i32,
    /// The skills it trains (spec §8 "Skills"), each by dice rolled when the character is made,
    /// in skill order; added to the race's.
    pub skills: Vec<(SkillId, Dice)>,
    /// The skills it is known for (spec §8 "Skills"), of [`ClassDef::skills`], in skill order:
    /// a level-up roll on one throws the die one step up the ladder.
    pub known_for: Vec<SkillId>,
    /// The skill rolls each level gives (spec §8 "Skills"), before the race's and
    /// Intelligence's.
    pub rolls: u8,
    /// Talent picks at creation beyond everyone's (spec §8 "Talents": the bard's one more).
    pub creation_talents: u8,
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
    /// The starting kit as written; resolved by the loader into `kit`.
    pub(super) kit_spec: Vec<KitLineSpec>,
    /// What the class starts with (spec §8 "Character creation"): first in a new character's
    /// pack; filled in by the loader once objects are known.
    pub kit: Kit,
    /// Its class abilities (spec §8 "Class abilities"): at each level, the talent it gives, by
    /// id, ascending by level.
    pub abilities: Vec<(u8, String)>,
    /// Its bonus talent picks.
    pub bonus_talents: BonusTalents,
    /// The file it was read from, for the errors of [`ClassDef::check_table`].
    pub(super) file: String,
}

impl ClassDef {
    /// Every weapon skill and fighting style the class may start a character in: its fixed
    /// ranks and every set of its choices.
    pub fn trainable(&self) -> Vec<WeaponSkillId> {
        let sets = (self.weapon_skill_choice.iter()).chain(&self.style_choice);
        let ranks = (self.weapon_skills.iter())
            .chain(&self.styles)
            .chain(sets.flatten());
        ranks.map(|&(id, _)| id).collect()
    }
}

impl ClassDef {
    /// Parses the class table (`data/classes/*.toml`, or a single `data/classes.toml`), given
    /// as its files in path order: the classes in that order; `skill_id` says which skill a
    /// skill id is, and `weapon_table` holds the weapon skills starting ranks may name.
    pub(super) fn parse<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
        skill_id: impl Fn(&str) -> Option<SkillId>,
        weapon_table: &WeaponSkillTable,
    ) -> Result<Vec<Self>, ContentError> {
        let mut list: Vec<ClassDef> = Vec::new();
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        let mut last = "data/classes/";
        for (file, text) in files {
            last = file;
            let err = |m: String| ContentError::new(file, m);
            let parsed: ClassFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
            for spec in parsed.class {
                if let Some(first) = seen.insert(spec.id.clone(), file) {
                    return Err(duplicate_id("class", &spec.id, file, first));
                }
                list.push(Self::resolve(file, spec, &skill_id, weapon_table)?);
            }
        }
        if list.is_empty() {
            return Err(ContentError::new(last, "at least one class is required"));
        }
        Ok(list)
    }

    /// One class of `file`, checked.
    fn resolve(
        file: &str,
        spec: ClassSpec,
        skill_id: &impl Fn(&str) -> Option<SkillId>,
        weapon_table: &WeaponSkillTable,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let id = &spec.id;
        let stats =
            adjustments(&spec.stats).map_err(|m| err(format!("class `{id}`: stats: {m}")))?;
        if spec.hp > MAX_BASE_HP {
            return Err(err(format!(
                "class `{id}`: hp {} is more than {MAX_BASE_HP}",
                spec.hp
            )));
        }
        if spec.mana > MAX_BASE_MANA {
            return Err(err(format!(
                "class `{id}`: mana {} is more than {MAX_BASE_MANA}",
                spec.mana
            )));
        }
        if !CASTING.contains(&spec.casting) {
            return Err(err(format!(
                "class `{id}`: casting {} is out of range ({} to {} percent)",
                spec.casting,
                CASTING.start(),
                CASTING.end()
            )));
        }
        if !FACTORS.contains(&spec.experience) {
            return Err(err(format!(
                "class `{id}`: experience {} is out of range ({} to {} percent)",
                spec.experience,
                FACTORS.start(),
                FACTORS.end()
            )));
        }
        let alignment = crate::alignment::check_start(spec.alignment)
            .map_err(|m| err(format!("class `{id}`: {m}")))?;
        let skills = skills::grants(&spec.skills, skill_id)
            .map_err(|m| err(format!("class `{id}`: skills: {m}")))?;
        let mut known_for = Vec::with_capacity(spec.known_for.len());
        for name in &spec.known_for {
            let skill = (skill_id(name).filter(|s| skills.iter().any(|(k, _)| k == s)))
                .ok_or_else(|| {
                    err(format!(
                        "class `{id}`: known_for: `{name}` is not one of the class's skills"
                    ))
                })?;
            if known_for.contains(&skill) {
                return Err(err(format!("class `{id}`: known_for: `{name}` twice")));
            }
            known_for.push(skill);
        }
        known_for.sort();
        if spec.rolls > MAX_ROLLS {
            return Err(err(format!(
                "class `{id}`: rolls {} is more than {MAX_ROLLS}",
                spec.rolls
            )));
        }
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
        let level_ok = |l: u8| (1..=MAX_LEVEL).contains(&l);
        let mut abilities: Vec<(u8, String)> = Vec::with_capacity(spec.abilities.len());
        for a in spec.abilities {
            if !level_ok(a.level) {
                return Err(err(format!(
                    "class `{id}`: abilities: level {} is not 1 to {MAX_LEVEL}",
                    a.level
                )));
            }
            if abilities.last().is_some_and(|(l, _)| *l > a.level) {
                return Err(err(format!(
                    "class `{id}`: abilities are listed by level, lowest first"
                )));
            }
            if abilities.iter().any(|(_, t)| *t == a.talent) {
                return Err(err(format!(
                    "class `{id}`: abilities: `{}` twice",
                    a.talent
                )));
            }
            abilities.push((a.level, a.talent));
        }
        let bonus_talents = match spec.bonus_talents {
            None => BonusTalents::default(),
            Some(b) => {
                let ascending = b.levels.windows(2).all(|w| w[0] < w[1]);
                if b.levels.is_empty() || !ascending || !b.levels.iter().all(|&l| level_ok(l)) {
                    return Err(err(format!(
                        "class `{id}`: bonus_talents: levels are 1 to {MAX_LEVEL}, at least \
                         one, ascending"
                    )));
                }
                BonusTalents {
                    levels: b.levels,
                    trees: b.trees,
                }
            }
        };
        Ok(ClassDef {
            id: spec.id,
            name: spec.name,
            description: spec.description,
            stats,
            hp: i32::from(spec.hp),
            mana: i32::from(spec.mana),
            casting: spec.casting,
            experience: spec.experience,
            alignment,
            skills,
            known_for,
            rolls: spec.rolls,
            creation_talents: spec.creation_talents,
            weapon_skills,
            weapon_skill_choice,
            styles,
            style_choice,
            kit_spec: spec.kit,
            kit: Kit::default(),
            abilities,
            bonus_talents,
            file: file.to_string(),
        })
    }

    /// Whether it trains skill `skill` (any of its grants, known for or not).
    pub fn trains(&self, skill: SkillId) -> bool {
        self.skills.iter().any(|&(s, _)| s == skill)
    }

    /// Whether it is known for skill `skill`.
    pub fn is_known_for(&self, skill: SkillId) -> bool {
        self.known_for.contains(&skill)
    }

    /// The talents its level table gives up to `level`, in table order; none from a book
    /// without a class tree (content whose talents have no class abilities, a fixture's).
    pub fn abilities_to(&self, book: &TalentBook, level: u8) -> Vec<TalentId> {
        if !book.has_class_tree() {
            return Vec::new();
        }
        (self.abilities.iter())
            .take_while(|(l, _)| *l <= level)
            .filter_map(|(_, id)| book.id(id))
            .collect()
    }

    /// The bonus talent picks level `level` brings (0 or 1).
    pub fn bonus_at(&self, level: u8) -> u8 {
        u8::from(self.bonus_talents.levels.contains(&level))
    }

    /// The trees of `book` its bonus picks come from, in display order; `None`: any.
    pub fn bonus_trees(&self, book: &TalentBook) -> Option<Vec<TreeId>> {
        if self.bonus_talents.trees.is_empty() {
            return None;
        }
        let mut trees: Vec<TreeId> = (self.bonus_talents.trees.iter())
            .filter_map(|t| book.tree_id(t))
            .collect();
        trees.sort();
        Some(trees)
    }

    /// Checks its level table against `book` (once the talents are loaded; nothing to check
    /// against a book without a class tree, whose class tables give nothing): every talent it
    /// names is in the book, and every bonus tree is a tree that is no class tree.
    pub(super) fn check_table(&self, book: &TalentBook) -> Result<(), ContentError> {
        if !book.has_class_tree() {
            return Ok(());
        }
        let err = |m: String| ContentError::new(&self.file, format!("class `{}`: {m}", self.id));
        for (_, talent) in &self.abilities {
            if book.id(talent).is_none() {
                return Err(err(format!("abilities: unknown talent `{talent}`")));
            }
        }
        for tree in &self.bonus_talents.trees {
            match book.tree_id(tree) {
                Some(t) if !book.tree(t).class => {}
                Some(_) => {
                    return Err(err(format!(
                        "bonus_talents: `{tree}` is the class tree: nothing is picked from it"
                    )));
                }
                None => return Err(err(format!("bonus_talents: unknown tree `{tree}`"))),
            }
        }
        Ok(())
    }
}

/// The class actives (spec §8 "Class abilities"): what a class ability with an active effect
/// does when it is used from the abilities menu, each its own rule (`world/class_ability.rs`).
/// The ability that has one is named by its id (`use:rage`, `use:smite:E`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassActive {
    /// Fly into a rage: the rage's status (Raging), then what it `follows` into.
    Rage,
    /// Hit points back at once, then a cooldown (the warrior's Veteran's wind).
    VeteransWind,
    /// A blow at every hostile beside you for the price of one attack.
    Whirlwind,
    /// Hit points back by the level, then a cooldown (the paladin's Lay on hands).
    LayOnHands,
    /// A blow in a direction with dice more damage, then a cooldown.
    Smite,
    /// A light of your own, on and off (a status that sheds light while it lasts).
    HolyRadiance,
    /// An unarmed blow in a direction that leaves a wound bleeding, then a cooldown.
    QuiveringPalm,
    /// Shots at a cell, several for the price of one, then a cooldown (the ranger's Volley).
    Volley,
    /// A blow in a direction that is a critical if it hits, for openings (the Master thief's
    /// finisher).
    MasterThief,
    /// A song that lifts the player's fighting a while, for mana (the bard's Inspire).
    Inspire,
}

impl ClassActive {
    /// Every active, in menu order.
    pub const ALL: &[ClassActive] = &[
        ClassActive::Rage,
        ClassActive::VeteransWind,
        ClassActive::Whirlwind,
        ClassActive::LayOnHands,
        ClassActive::Smite,
        ClassActive::HolyRadiance,
        ClassActive::QuiveringPalm,
        ClassActive::Volley,
        ClassActive::MasterThief,
        ClassActive::Inspire,
    ];

    /// Its id: the id of the class ability that has it, and its replay text (`use:rage`).
    pub const fn id(self) -> &'static str {
        match self {
            ClassActive::Rage => "rage",
            ClassActive::VeteransWind => "veterans_wind",
            ClassActive::Whirlwind => "whirlwind",
            ClassActive::LayOnHands => "lay_on_hands",
            ClassActive::Smite => "smite",
            ClassActive::HolyRadiance => "holy_radiance",
            ClassActive::QuiveringPalm => "quivering_palm",
            ClassActive::Volley => "volley",
            ClassActive::MasterThief => "master_thief",
            ClassActive::Inspire => "inspire",
        }
    }

    /// The active with id `id`.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.id() == id)
    }

    /// Whether it asks for a direction before it is used.
    pub const fn aimed(self) -> bool {
        matches!(
            self,
            ClassActive::Smite | ClassActive::QuiveringPalm | ClassActive::MasterThief
        )
    }

    /// Whether it asks for a cell before it is used (`use:volley:12,5`).
    pub const fn aims_cell(self) -> bool {
        matches!(self, ClassActive::Volley)
    }

    /// What it costs, as an action of this kind (`data/actions.toml`).
    pub const fn cost(self) -> ActionKind {
        match self {
            ClassActive::Rage | ClassActive::VeteransWind => ActionKind::Wait,
            ClassActive::Whirlwind
            | ClassActive::Smite
            | ClassActive::QuiveringPalm
            | ClassActive::MasterThief => ActionKind::Melee,
            ClassActive::LayOnHands | ClassActive::HolyRadiance => ActionKind::Cast,
            ClassActive::Volley => ActionKind::Shoot,
            ClassActive::Inspire => ActionKind::Chat,
        }
    }
}

/// The built-in class files of `files` without their level tables and bonus picks (for tests
/// of talents, which those would put picks and abilities in the way of).
#[cfg(test)]
pub fn strip_tables(files: &mut super::ContentFiles) {
    let classes: Vec<(String, String)> = (files.table("classes"))
        .map(|(path, text)| {
            let mut skipping = false;
            let mut kept = String::new();
            for line in text.lines() {
                if line.starts_with("abilities = [") {
                    skipping = true;
                }
                if !skipping && !line.starts_with("bonus_talents") {
                    kept.push_str(line);
                    kept.push('\n');
                }
                if skipping && line.starts_with(']') {
                    skipping = false;
                }
            }
            (path.to_string(), kept)
        })
        .collect();
    for (path, text) in classes {
        files.insert(&path, &text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = "[[class]]\nid = \"wizard\"\nname = \"Wizard\"\ndescription = \"Books.\"\n";

    /// Parses `text` with the built-in weapon skills and one skill, `spellcraft`.
    fn parse(text: &str) -> Result<Vec<ClassDef>, ContentError> {
        let content = super::super::Content::builtin().expect("content");
        let skill = |k: &str| match k {
            "spellcraft" => Some(SkillId(0)),
            "knowledge" => Some(SkillId(1)),
            _ => None,
        };
        ClassDef::parse([("c", text)], skill, content.weapon_skills())
    }

    #[test]
    fn classes_read_adjustments_and_refuse_bad_ones() {
        let text = format!(
            "{ONE}stats = {{ intelligence = \"2d2+1\", toughness = \"-1d2\", willpower = 1 }}\n"
        );
        let classes = parse(&text).expect("valid");
        assert!(classes[0].skills.is_empty());
        assert_eq!(classes[0].experience, 100, "unless named");
        assert_eq!(
            (classes[0].rolls, classes[0].known_for.len()),
            (2, 0),
            "unless named"
        );
        let text = format!(
            "{ONE}skills = {{ spellcraft = \"2d6+10\", knowledge = 5 }}\n\
             known_for = [\"spellcraft\"]\nrolls = 3\n"
        );
        let wizard = &parse(&text).expect("valid")[0];
        assert_eq!(
            (wizard.rolls, wizard.known_for.as_slice()),
            (3, &[SkillId(0)][..])
        );
        assert!(wizard.is_known_for(SkillId(0)) && !wizard.is_known_for(SkillId(1)));
        assert!(wizard.trains(SkillId(1)));
        let text = format!("{ONE}experience = 125\n");
        assert_eq!(parse(&text).expect("valid")[0].experience, 125);
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
            (
                format!("{ONE}experience = 300\n"),
                "class `wizard`: experience 300 is out of range (50 to 200 percent)",
            ),
            ("class = []\n".into(), "at least one class"),
            (
                format!("{ONE}known_for = [\"spellcraft\"]\n"),
                "known_for: `spellcraft` is not one of the class's skills",
            ),
            (
                format!(
                    "{ONE}skills = {{ spellcraft = 1 }}\nknown_for = [\"spellcraft\", \"spellcraft\"]\n"
                ),
                "`spellcraft` twice",
            ),
            (format!("{ONE}rolls = 11\n"), "rolls 11 is more than 10"),
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

    #[test]
    fn classes_give_mana_up_to_the_most_a_class_may() {
        assert_eq!(
            parse(&format!("{ONE}mana = 8\n")).expect("valid")[0].mana,
            8
        );
        assert_eq!(parse(ONE).expect("valid")[0].mana, 0, "none unless named");
        let e = parse(&format!("{ONE}mana = 1001\n")).unwrap_err();
        assert!(
            e.message.contains("mana 1001 is more than 1000"),
            "{}",
            e.message
        );
    }

    #[test]
    fn classes_cast_a_percent_of_their_level() {
        assert_eq!(parse(ONE).expect("valid")[0].casting, 25, "unless named");
        let wizard = parse(&format!("{ONE}casting = 100\n")).expect("valid");
        assert_eq!(wizard[0].casting, 100);
        for bad in [0, 101] {
            let e = parse(&format!("{ONE}casting = {bad}\n")).unwrap_err();
            assert!(
                e.message
                    .contains(&format!("casting {bad} is out of range (1 to 100")),
                "{}",
                e.message
            );
        }
    }

    /// The built-in content with the aristocrat's file changed by `edit`, loaded.
    fn with_aristocrat(
        edit: impl Fn(&str) -> String,
    ) -> Result<super::super::Content, ContentError> {
        let mut files = super::super::ContentFiles::builtin();
        let path = "data/classes/12-aristocrat.toml";
        let text = files
            .table("classes")
            .find(|(p, _)| *p == path)
            .expect("aristocrat")
            .1;
        let text = edit(text);
        files.insert(path, &text);
        super::super::Content::load(&files)
    }

    #[test]
    fn level_tables_name_known_talents_and_trees_in_order() {
        let content = with_aristocrat(|t| t.to_string()).expect("built-in");
        let aristocrat = content.class("aristocrat").expect("aristocrat");
        let book = content.talents();
        let at = |level| -> Vec<&str> {
            (aristocrat.abilities_to(book, level).into_iter())
                .map(|t| book.get(t).id.as_str())
                .collect()
        };
        assert_eq!(at(1), ["well_dressed"]);
        assert_eq!(at(17), ["well_dressed", "fencing", "noblesse"]);
        assert_eq!(at(60).len(), 9);
        assert_eq!((aristocrat.bonus_at(10), aristocrat.bonus_at(11)), (1, 0));
        let trees = aristocrat.bonus_trees(book).expect("named trees");
        let names: Vec<&str> = trees.iter().map(|&t| book.tree(t).id.as_str()).collect();
        assert_eq!(names, ["weaponcraft", "delver"]);
        let commoner = content.class("commoner").expect("commoner");
        assert_eq!(commoner.bonus_trees(book), None, "any tree");
        for (edit, why) in [
            (
                ("\"fencing\"", "\"fencin\""),
                "abilities: unknown talent `fencin`",
            ),
            (("\"delver\"]", "\"class\"]"), "`class` is the class tree"),
            (("\"delver\"]", "\"nonesuch\"]"), "unknown tree `nonesuch`"),
            (("level = 1,", "level = 0,"), "level 0 is not 1 to"),
            (
                ("level = 12,", "level = 2,"),
                "listed by level, lowest first",
            ),
            (
                ("talent = \"noblesse\"", "talent = \"fencing\""),
                "`fencing` twice",
            ),
            (("[10, 20, 30, 45, 55]", "[20, 10]"), "ascending"),
        ] {
            let e = with_aristocrat(|t| t.replacen(edit.0, edit.1, 1)).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
