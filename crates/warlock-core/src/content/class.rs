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

use super::kit::{Kit, KitLineSpec, RandomScrolls, RandomScrollsSpec};
use super::object::article;
use super::race::RaceDef;
use super::skill::{OptionalSkill, OptionalSpec};
use super::weapon_skill::Ranked;
use super::{
    ActionKind, ContentError, FACTORS, TalentBook, TalentId, TreeId, WeaponSkillId,
    WeaponSkillTable, duplicate_id,
};
use crate::character::Gender;
use crate::dice::Dice;
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
    #[serde(default)]
    affinity: BTreeMap<super::spell_def::School, i8>,
    #[serde(default)]
    skills: BTreeMap<String, Dice>,
    #[serde(default)]
    known_for: Vec<String>,
    #[serde(default)]
    optional_skills: BTreeMap<String, OptionalSpec>,
    skill_choice: Option<SkillChoiceSpec>,
    #[serde(default = "two_rolls")]
    rolls: u8,
    #[serde(default)]
    creation_talents: u8,
    creation_bonus: Option<CreationBonus>,
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
    random_scrolls: Option<RandomScrollsSpec>,
    #[serde(default)]
    abilities: Vec<AbilitySpec>,
    bonus_talents: Option<BonusSpec>,
    max_level: Option<u8>,
    ally_share: Option<u8>,
    #[serde(default)]
    races_not: Vec<String>,
    #[serde(default)]
    only: Vec<OnlySpec>,
    #[serde(default)]
    hidden: bool,
}

/// `{ skills = ["alchemy", "metalcraft"], dice = "2d4+4" }`: one of the skills, picked at
/// random, by the dice.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillChoiceSpec {
    skills: Vec<String>,
    dice: Dice,
}

/// One skill a new character of the class always starts trained in, picked at random from a few
/// (spec §8 "Skills": the commoner's one profession).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillChoice {
    /// The skills it picks from, in skill order (at least two, none of the class's skills).
    pub skills: Vec<SkillId>,
    /// The picked skill's starting value.
    pub dice: Dice,
}

/// `{ race = "drow", genders = ["female"] }`: a character of that race may take the class
/// only with one of those genders.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnlySpec {
    pub race: String,
    pub genders: Vec<Gender>,
}

/// `{ level = 6, talent = "fencing" }`: at that level the class gives that talent.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AbilitySpec {
    level: u8,
    talent: String,
}

/// `{ count = 2, trees = ["crafts"] }`: that many bonus talent picks at creation, from those
/// trees (the craftsperson's two trades; the craftsperson design §4).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreationBonus {
    /// How many (1 to 8).
    pub count: u8,
    /// The trees they come from, by id (at least one).
    pub trees: Vec<String>,
}

/// `{ levels = [10, 20], trees = ["guard"] }`: a bonus talent pick at each of those levels,
/// from those trees (none: any).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BonusSpec {
    levels: Vec<u8>,
    #[serde(default)]
    trees: Vec<String>,
}

/// The `abilities` rows of a class or prestige table, by level, lowest first (stable: a level's
/// rows keep their order), as `abilities_to` reads them.
pub(super) fn table_rows(rows: Vec<AbilitySpec>) -> Vec<(u8, String)> {
    let mut abilities: Vec<(u8, String)> = rows.into_iter().map(|a| (a.level, a.talent)).collect();
    abilities.sort_by_key(|(level, _)| *level);
    abilities
}

/// The `bonus_talents` of a class or prestige table.
pub(super) fn bonus_rows(spec: Option<BonusSpec>) -> BonusTalents {
    spec.map_or_else(BonusTalents::default, |b| BonusTalents {
        levels: b.levels,
        trees: b.trees,
    })
}

/// Checks a level table (`abilities` and `bonus_talents`) against `book` once the talents are
/// loaded (nothing to check against a book without a class tree, whose tables give nothing):
/// every talent and bonus tree named is in the book.
pub(super) fn check_rows(
    err: &impl Fn(String) -> ContentError,
    abilities: &[(u8, String)],
    bonus: &BonusTalents,
    book: &TalentBook,
) -> Result<(), ContentError> {
    if !book.has_class_tree() {
        return Ok(());
    }
    for (_, talent) in abilities {
        if book.id(talent).is_none() {
            return Err(err(format!("abilities: unknown talent `{talent}`")));
        }
    }
    for tree in &bonus.trees {
        if book.tree_id(tree).is_none() {
            return Err(err(format!("bonus_talents: unknown tree `{tree}`")));
        }
    }
    Ok(())
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
    /// Its affinity with the schools of magic (spec §8 "Spells"): points off the difficulty of
    /// studying a spell of the school (a malus when negative); a school it leaves out, 0.
    pub affinity: BTreeMap<super::spell_def::School, i8>,
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
    /// The skills a new character may start with besides [`ClassDef::skills`], each by its
    /// chance (spec §8 "Skills"), in skill order; none of them one of the class's skills.
    pub optional_skills: Vec<OptionalSkill>,
    /// One skill picked at random that a new character always starts with, by its dice (the
    /// commoner's profession); none for most classes.
    pub skill_choice: Option<SkillChoice>,
    /// The skill rolls each level gives (spec §8 "Skills"), before the race's and
    /// Intelligence's.
    pub rolls: u8,
    /// Talent picks at creation beyond everyone's (spec §8 "Talents": the bard's one more).
    pub creation_talents: u8,
    /// Bonus talent picks at creation, from its own trees (the craftsperson's two trades):
    /// level 1's `Due::bonus` (`world/creation.rs`).
    pub creation_bonus: Option<CreationBonus>,
    /// Starting ranks in weapon skills (spec §8 "Weapon skills"), in skill order: added to the
    /// race's.
    pub weapon_skills: Vec<(WeaponSkillId, Dice)>,
    /// Sets of starting ranks of which one is picked at random (the berserker: axes or great
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
    /// Its `random_scrolls` as written; resolved by the loader into `random_scrolls`.
    pub(super) random_scrolls_spec: Option<RandomScrollsSpec>,
    /// The spell scrolls its kit adds at random (the casters'), after every kit's lines;
    /// filled in by the loader once objects and spells are known.
    pub random_scrolls: Option<RandomScrolls>,
    /// Its class abilities (spec §8 "Class abilities"): at each level, the talent it gives, by
    /// id, ascending by level.
    pub abilities: Vec<(u8, String)>,
    /// Its bonus talent picks.
    pub bonus_talents: BonusTalents,
    /// The highest level the class reaches (spec §8 "Prestige classes"): the file's
    /// `max_level`, else `[levels] class_max` of `data/experience.toml`, filled in by the loader
    /// ([`ClassDef::settle_cap`]; 0 until then). A character's level stops here until a prestige
    /// class takes them on.
    pub max_level: u8,
    /// What an ally's kill pays a character of the class, in percent of their own kill (spec §8
    /// "Allies"): the pet classes' 100 (the ranger, the druid); `None` for `[xp] ally_percent`
    /// of `data/allies.toml`, everyone else's.
    pub ally_share: Option<u8>,
    /// The races that may not take it, by id (spec §8 "Character creation": no goblin paladin).
    pub races_not: Vec<String>,
    /// The races that may take it only with some genders (a drow priest is a woman).
    pub only: Vec<OnlySpec>,
    /// Never offered at character creation (the everyday folk, for now); still taken by
    /// `--character` and a scenario's own character.
    pub hidden: bool,
    /// The file it was read from, for the errors of [`ClassDef::check_table`].
    pub(super) file: String,
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
        let stats = adjustments(&spec.stats);
        // The top keeps every level's threshold within the experience counter.
        if spec.experience > *FACTORS.end() {
            return Err(err(format!(
                "class `{id}`: experience {} is more than {} percent",
                spec.experience,
                FACTORS.end()
            )));
        }
        let skills = skills::grants(&spec.skills, skill_id)
            .map_err(|m| err(format!("class `{id}`: skills: {m}")))?;
        let mut known_for = Vec::with_capacity(spec.known_for.len());
        for name in &spec.known_for {
            let skill = skill_id(name)
                .ok_or_else(|| err(format!("class `{id}`: known_for: unknown skill `{name}`")))?;
            known_for.push(skill);
        }
        known_for.sort();
        let optional_skills = super::skill::optional_skills(&spec.optional_skills, skill_id)
            .map_err(|m| err(format!("class `{id}`: optional_skills: {m}")))?;
        let skill_choice = match spec.skill_choice {
            None => None,
            Some(choice) => {
                let mut picks = Vec::with_capacity(choice.skills.len());
                for name in &choice.skills {
                    let skill = skill_id(name).ok_or_else(|| {
                        err(format!(
                            "class `{id}`: skill_choice: unknown skill `{name}`"
                        ))
                    })?;
                    if skills.iter().any(|&(s, _)| s == skill) || picks.contains(&skill) {
                        return Err(err(format!(
                            "class `{id}`: skill_choice: `{name}` is named twice or is one of its skills"
                        )));
                    }
                    picks.push(skill);
                }
                if picks.len() < 2 {
                    return Err(err(format!(
                        "class `{id}`: skill_choice: name two skills or more"
                    )));
                }
                picks.sort();
                Some(SkillChoice {
                    skills: picks,
                    dice: choice.dice,
                })
            }
        };
        let ranks = |data: &BTreeMap<String, Dice>, ranked| {
            (weapon_table.starting_ranks(data, ranked))
                .map_err(|m| err(format!("class `{id}`: {m}")))
        };
        let choice = |sets: &[BTreeMap<String, Dice>], ranked| {
            (sets.iter())
                .map(|set| ranks(set, ranked))
                .collect::<Result<Vec<_>, _>>()
        };
        let weapon_skills = ranks(&spec.weapon_skills, Ranked::Weapons)?;
        let weapon_skill_choice = choice(&spec.weapon_skill_choice, Ranked::Weapons)?;
        let styles = ranks(&spec.styles, Ranked::Styles)?;
        let style_choice = choice(&spec.style_choice, Ranked::Styles)?;
        let abilities = table_rows(spec.abilities);
        let bonus_talents = bonus_rows(spec.bonus_talents);
        if let Some(b) = &spec.creation_bonus {
            if !(1..=8).contains(&b.count) {
                return Err(err(format!(
                    "class `{id}`: creation_bonus: count {} is not 1 to 8",
                    b.count
                )));
            }
            if b.trees.is_empty() {
                return Err(err(format!("class `{id}`: creation_bonus names no tree")));
            }
        }
        for o in &spec.only {
            if o.genders.is_empty() {
                return Err(err(format!(
                    "class `{id}`: only: race `{}` names no gender",
                    o.race
                )));
            }
        }
        Ok(ClassDef {
            id: spec.id,
            name: spec.name,
            description: spec.description,
            stats,
            hp: i32::from(spec.hp),
            mana: i32::from(spec.mana),
            affinity: spec.affinity,
            experience: spec.experience,
            alignment: spec.alignment,
            skills,
            known_for,
            optional_skills,
            skill_choice,
            rolls: spec.rolls,
            creation_talents: spec.creation_talents,
            creation_bonus: spec.creation_bonus,
            weapon_skills,
            weapon_skill_choice,
            styles,
            style_choice,
            kit_spec: spec.kit,
            kit: Kit::default(),
            random_scrolls_spec: spec.random_scrolls,
            random_scrolls: None,
            abilities,
            bonus_talents,
            max_level: spec.max_level.unwrap_or(0),
            ally_share: spec.ally_share,
            races_not: spec.races_not,
            only: spec.only,
            hidden: spec.hidden,
            file: file.to_string(),
        })
    }

    /// Settles its highest level once the experience rules are read: `default` (`[levels]
    /// class_max`) where the file named none, at most `highest` (the curve's `max_level`), and
    /// every level of its table within it.
    pub(super) fn settle_cap(&mut self, default: u8, highest: u8) -> Result<(), ContentError> {
        let err = |m: String| ContentError::new(&self.file, m);
        let what = format!("class `{}`", self.id);
        if self.max_level == 0 {
            self.max_level = default;
        }
        if self.max_level > highest {
            return Err(err(format!(
                "{what}: max_level {} is more than the curve's {highest}",
                self.max_level
            )));
        }
        Ok(())
    }

    /// Checks the races its `races_not` and `only` name against `is_race` (once the races are
    /// read).
    pub(super) fn check_races(&self, is_race: impl Fn(&str) -> bool) -> Result<(), ContentError> {
        let named = (self.races_not.iter()).chain(self.only.iter().map(|o| &o.race));
        for race in named {
            if !is_race(race) {
                return Err(ContentError::new(
                    &self.file,
                    format!("class `{}`: unknown race `{race}`", self.id),
                ));
            }
        }
        Ok(())
    }

    /// Why a character of `race` and `gender` may not take it, in words (`a goblin cannot be a
    /// paladin`); `None` when they may. Hidden classes are not refused here: they are only left
    /// out of character creation ([`ClassDef::offered_to`]).
    pub fn refuses(&self, race: &RaceDef, gender: Gender) -> Option<String> {
        let who = race.name.to_lowercase();
        let class = self.name.to_lowercase();
        let a = |noun: &str| format!("{} {noun}", article(noun));
        if self.races_not.contains(&race.id) {
            return Some(format!("{} cannot be {}", a(&who), a(&class)));
        }
        let only = self.only.iter().find(|o| o.race == race.id)?;
        if only.genders.contains(&gender) {
            return None;
        }
        let genders: Vec<String> = (only.genders.iter())
            .map(|g| g.name().to_lowercase())
            .collect();
        Some(format!(
            "{} may be {} only if {}",
            a(&who),
            a(&class),
            genders.join(" or ")
        ))
    }

    /// Whether character creation offers it to a character of `race` and `gender`: not hidden
    /// and not refused.
    pub fn offered_to(&self, race: &RaceDef, gender: Gender) -> bool {
        !self.hidden && self.refuses(race, gender).is_none()
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
        super::prestige::Table::Class(self).abilities_to(book, level)
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

    /// The trees of `book` its creation bonus picks come from, in display order; `None` for a
    /// class without them.
    pub fn creation_trees(&self, book: &TalentBook) -> Option<Vec<TreeId>> {
        let bonus = self.creation_bonus.as_ref()?;
        let mut trees: Vec<TreeId> = (bonus.trees.iter())
            .filter_map(|t| book.tree_id(t))
            .collect();
        trees.sort();
        Some(trees)
    }

    /// Checks its level table against `book` (once the talents are loaded; nothing to check
    /// against a book without a class tree, whose class tables give nothing): every talent it
    /// names is in the book, and every bonus tree, its creation bonus's too, is a tree that is no
    /// class tree.
    pub(super) fn check_table(&self, book: &TalentBook) -> Result<(), ContentError> {
        let err = |m: String| ContentError::new(&self.file, format!("class `{}`: {m}", self.id));
        check_rows(&err, &self.abilities, &self.bonus_talents, book)?;
        if !book.has_class_tree() {
            return Ok(());
        }
        for tree in self.creation_bonus.iter().flat_map(|b| &b.trees) {
            if book.tree_id(tree).is_none() {
                return Err(err(format!("creation_bonus: unknown tree `{tree}`")));
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
    /// A roar, for rage, that frightens the hostiles who hear and see you unless they keep
    /// their nerve (the berserker's War cry, `world/war_cry.rs`).
    WarCry,
    /// Hit points back at once, then a cooldown (the warrior's Veteran's wind).
    VeteransWind,
    /// A blow at every hostile beside you for the price of one attack.
    Whirlwind,
    /// Hit points back by the level, then a cooldown (the paladin's Lay on hands).
    LayOnHands,
    /// A blow of holy wrath in a direction with dice more damage, for zeal (`world/smite.rs`).
    Smite,
    /// A smite whose foe sheds holy light a while, for zeal (the paladin's Branding smite).
    BrandingSmite,
    /// A smite whose foe flees unless its will holds, for zeal (the paladin's Wrathful smite).
    WrathfulSmite,
    /// A smite that unmakes the summoned and falls twice on the unholy, for zeal (the paladin's
    /// Banishing smite).
    BanishingSmite,
    /// A light of your own, on and off (a status that sheds light while it lasts).
    HolyRadiance,
    /// A breath taken in the guard, for ki and a wait: Poised until the next action, Dodge up
    /// and a blow dodged or parried answered at once (the ascetic's Patient defence,
    /// `world/ascetic.rs`).
    PatientDefence,
    /// An unarmed blow in a direction that leaves a wounded foe Stunned, for ki (the
    /// ascetic's Stunning strike).
    StunningStrike,
    /// An unarmed blow in a direction that leaves a wound bleeding, for ki.
    QuiveringPalm,
    /// Light on the feet for ki, taking no time: the next action at half its price (the
    /// ascetic's Step of the wind, `world/ascetic.rs`).
    StepOfTheWind,
    /// What clouds the mind or holds the body ended at once, for ki, taking no time (the
    /// ascetic's Stillness, on Still mind, `world/ascetic.rs`).
    Stillness,
    /// Shots at a cell, several for the price of one, then a cooldown (the ranger's Volley).
    Volley,
    /// The companion set on a creature the player sees, savage, for focus (the ranger's Sic 'em,
    /// `world/sic.rs`).
    Sic,
    /// A wait in ambush: a hostile stepping up beside you takes your blow at once (the
    /// thief's Lurk).
    Lurk,
    /// A blow in a direction that is a sneak attack whatever the foe knows, for openings (the
    /// thief's Low blow).
    LowBlow,
    /// A blow in a direction that is a critical if it hits, for openings (the Master thief's
    /// finisher).
    MasterThief,
    /// A song that lifts the player's fighting a while, for inspiration (the bard's Inspire,
    /// `world/song.rs`).
    Inspire,
    /// A song that leaves the foes who hear it and cannot shake it off drowsy, for mana (the
    /// bard's Lullaby, `world/song.rs`).
    Lullaby,
    /// A minute's prayer that lifts an unholy room's effect for good, for mana (the priest's and
    /// the paladin's Consecrate).
    Consecrate,
    /// A prayer that answers what ails the priest most, as far as their prayers reach, under a
    /// hidden strain (the priest's Orison and the tiers after it, `world/orison.rs`).
    Orison,
    /// A minute's kneeling rite that hallows the ground about the priest for an hour, for mana:
    /// no foe sets foot in it and wounds and mind mend there faster (`world/sanctuary.rs`).
    Sanctuary,
    /// A loud cackle that holds every hex of the witch's on the creatures she sees a degree
    /// longer (the witch's Cackle, `world/witch.rs`).
    Cackle,
    /// An hour's sitting that brings a new familiar when the witch has none (the witch's Call
    /// familiar, given by Familiar, `world/familiar.rs`).
    CallFamiliar,
    /// A word that springs the lock of a door or chest open, for mana (the wizard's Knock).
    Knock,
    /// A scroll of a spell the player knows written on a blank scroll, for three casts' worth of
    /// mana and a study's time (the wizard's Scribe, `world/scribe.rs`).
    Scribe,
    /// A careful look that tells a thing the player carries for what it is, as the scroll of
    /// identify would (the merchant's Merchant's eye).
    MerchantsEye,
    /// A trap rigged on a cell beside the player, from the rig list then a direction (the
    /// craftsperson's Trapwright, `world/rig.rs`, `rig:tripwire:E`).
    Rig,
    /// A weapon of spirit-stuff called into the hand in the shape chosen, as a wield (the
    /// Hunter's Spirit weapon, `world/spirit_weapon.rs`).
    SpiritWeapon,
}

impl ClassActive {
    /// Every active, in menu order.
    pub const ALL: &[ClassActive] = &[
        ClassActive::Rage,
        ClassActive::WarCry,
        ClassActive::VeteransWind,
        ClassActive::Whirlwind,
        ClassActive::LayOnHands,
        ClassActive::Smite,
        ClassActive::BrandingSmite,
        ClassActive::WrathfulSmite,
        ClassActive::BanishingSmite,
        ClassActive::HolyRadiance,
        ClassActive::PatientDefence,
        ClassActive::StunningStrike,
        ClassActive::QuiveringPalm,
        ClassActive::StepOfTheWind,
        ClassActive::Stillness,
        ClassActive::Volley,
        ClassActive::Sic,
        ClassActive::Lurk,
        ClassActive::LowBlow,
        ClassActive::MasterThief,
        ClassActive::Inspire,
        ClassActive::Lullaby,
        ClassActive::Consecrate,
        ClassActive::Orison,
        ClassActive::Sanctuary,
        ClassActive::Cackle,
        ClassActive::CallFamiliar,
        ClassActive::Knock,
        ClassActive::Scribe,
        ClassActive::MerchantsEye,
        ClassActive::Rig,
        ClassActive::SpiritWeapon,
    ];

    /// Its id: the id of the class ability that has it, and its replay text (`use:rage`).
    pub const fn id(self) -> &'static str {
        match self {
            ClassActive::Rage => "rage",
            ClassActive::WarCry => "war_cry",
            ClassActive::VeteransWind => "veterans_wind",
            ClassActive::Whirlwind => "whirlwind",
            ClassActive::LayOnHands => "lay_on_hands",
            ClassActive::Smite => "smite",
            ClassActive::BrandingSmite => "branding_smite",
            ClassActive::WrathfulSmite => "wrathful_smite",
            ClassActive::BanishingSmite => "banishing_smite",
            ClassActive::HolyRadiance => "holy_radiance",
            ClassActive::PatientDefence => "patient_defence",
            ClassActive::StunningStrike => "stunning_strike",
            ClassActive::QuiveringPalm => "quivering_palm",
            ClassActive::StepOfTheWind => "step_of_the_wind",
            ClassActive::Stillness => "stillness",
            ClassActive::Volley => "volley",
            ClassActive::Sic => "sic",
            ClassActive::Lurk => "lurk",
            ClassActive::LowBlow => "low_blow",
            ClassActive::MasterThief => "master_thief",
            ClassActive::Inspire => "inspire",
            ClassActive::Lullaby => "lullaby",
            ClassActive::Consecrate => "consecrate",
            ClassActive::Orison => "orison",
            ClassActive::Sanctuary => "sanctuary",
            ClassActive::Cackle => "cackle",
            ClassActive::CallFamiliar => "call_familiar",
            ClassActive::Knock => "knock",
            ClassActive::Scribe => "scribe",
            ClassActive::MerchantsEye => "merchants_eye",
            ClassActive::Rig => "trapwright",
            ClassActive::SpiritWeapon => "spirit_weapon",
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
            ClassActive::Smite
                | ClassActive::BrandingSmite
                | ClassActive::WrathfulSmite
                | ClassActive::BanishingSmite
                | ClassActive::StunningStrike
                | ClassActive::QuiveringPalm
                | ClassActive::LowBlow
                | ClassActive::MasterThief
                | ClassActive::Knock
        )
    }

    /// Whether it is a blow of the bare hand (refused, no time, with a weapon in hand): the
    /// ascetic's Stunning strike and Quivering palm (`World::palm_blow`).
    pub const fn bare_handed(self) -> bool {
        matches!(
            self,
            ClassActive::StunningStrike | ClassActive::QuiveringPalm
        )
    }

    /// Whether it is a blow aimed at someone in its direction (refused with nobody hostile
    /// there); Knock is aimed at a lock instead.
    pub const fn strikes(self) -> bool {
        self.aimed() && !matches!(self, ClassActive::Knock)
    }

    /// Whether it is a holy gift that may be given an `alignment` in its data (spec §8
    /// "Alignment"): used outside those bands, it does nothing (`World::unanswered`).
    pub const fn holy(self) -> bool {
        matches!(
            self,
            ClassActive::LayOnHands
                | ClassActive::Smite
                | ClassActive::BrandingSmite
                | ClassActive::WrathfulSmite
                | ClassActive::BanishingSmite
                | ClassActive::HolyRadiance
                | ClassActive::Consecrate
        )
    }

    /// Whether it is a song (`world/song.rs`): a talent's `song` effect names it, and it is
    /// sung aloud.
    pub const fn sings(self) -> bool {
        matches!(self, ClassActive::Inspire | ClassActive::Lullaby)
    }

    /// Whether it asks for a cell before it is used (`use:volley:12,5`, `use:sic:12,5`).
    pub const fn aims_cell(self) -> bool {
        matches!(self, ClassActive::Volley | ClassActive::Sic)
    }

    /// Whether it asks for a stack the player carries before it is used
    /// (`use:merchants_eye:c`).
    pub const fn aims_item(self) -> bool {
        matches!(self, ClassActive::MerchantsEye)
    }

    /// Whether it asks for a weapon's shape before it is used, an object id
    /// (`use:spirit_weapon:long_sword`).
    pub const fn aims_shape(self) -> bool {
        matches!(self, ClassActive::SpiritWeapon)
    }

    /// Whether it asks for a rig, then a direction, before it is used (the craftsperson's
    /// Trapwright: [`crate::action::Action::Rig`], `rig:tripwire:E`).
    pub const fn aims_rig(self) -> bool {
        matches!(self, ClassActive::Rig)
    }

    /// Whether it asks for a spell the player knows before it is used (the wizard's Scribe,
    /// [`crate::action::Action::Scribe`], `scribe:fireball`).
    pub const fn aims_spell(self) -> bool {
        matches!(self, ClassActive::Scribe)
    }

    /// Whether it takes no time at all (Step of the wind, Stillness): its [`ClassActive::cost`]
    /// then names only the statuses that block it.
    pub const fn takes_no_time(self) -> bool {
        matches!(self, ClassActive::StepOfTheWind | ClassActive::Stillness)
    }

    /// What it costs, as an action of this kind (`data/actions.toml`).
    pub const fn cost(self) -> ActionKind {
        match self {
            ClassActive::Rage
            | ClassActive::WarCry
            | ClassActive::VeteransWind
            | ClassActive::PatientDefence
            | ClassActive::Lurk
            | ClassActive::Sic
            // It takes no time (`takes_no_time`): none block a wait.
            | ClassActive::StepOfTheWind
            | ClassActive::Stillness => ActionKind::Wait,
            ClassActive::Whirlwind
            | ClassActive::Smite
            | ClassActive::BrandingSmite
            | ClassActive::WrathfulSmite
            | ClassActive::BanishingSmite
            | ClassActive::StunningStrike
            | ClassActive::QuiveringPalm
            | ClassActive::LowBlow
            | ClassActive::MasterThief => ActionKind::Melee,
            ClassActive::LayOnHands
            | ClassActive::HolyRadiance
            | ClassActive::Consecrate
            | ClassActive::Orison
            | ClassActive::Sanctuary
            | ClassActive::Cackle
            | ClassActive::Knock => ActionKind::Cast,
            ClassActive::Volley => ActionKind::Shoot,
            ClassActive::Inspire | ClassActive::Lullaby => ActionKind::Song,
            ClassActive::MerchantsEye => ActionKind::Read,
            // A turn of it; a rig takes its `seconds` of them (`world/rig.rs`).
            ClassActive::Rig => ActionKind::Craft,
            // A turn of it; a scroll takes a study's turns (`world/scribe.rs`).
            ClassActive::Scribe => ActionKind::Scribe,
            // A turn of it; the ritual takes `call_turns` of them (`world/familiar.rs`).
            ClassActive::CallFamiliar => ActionKind::CallFamiliar,
            // A weapon taken in hand, as wielding one is.
            ClassActive::SpiritWeapon => ActionKind::Wield,
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
        assert!(classes[0].optional_skills.is_empty(), "unless named");
        let text = format!(
            "{ONE}skills = {{ spellcraft = \"2d6+10\" }}\n\
             optional_skills = {{ knowledge = {{ chance = 30, dice = \"2d4\" }} }}\n"
        );
        assert_eq!(
            parse(&text).expect("valid")[0].optional_skills,
            [OptionalSkill {
                skill: SkillId(1),
                chance: 30,
                dice: "2d4".parse().expect("dice"),
            }]
        );
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
            (format!("{ONE}{ONE}"), "duplicate class id `wizard`"),
            (
                format!("{ONE}experience = 300\n"),
                "class `wizard`: experience 300 is more than 200 percent",
            ),
            ("class = []\n".into(), "at least one class"),
            (
                format!("{ONE}known_for = [\"necromancy\"]\n"),
                "known_for: unknown skill `necromancy`",
            ),
            (
                format!("{ONE}optional_skills = {{ necromancy = {{ chance = 5, dice = 3 }} }}\n"),
                "optional_skills: unknown skill `necromancy`",
            ),
            (
                format!("{ONE}skills = {{ necromancy = 5 }}\n"),
                "class `wizard`: skills: unknown skill `necromancy`",
            ),
            (
                format!("{ONE}weapon_skills = {{ wands = 1 }}\n"),
                "class `wizard`: unknown weapon skill `wands`",
            ),
            (
                format!("{ONE}weapon_skills = {{ duelist = 1 }}\n"),
                "`duelist` is a fighting style: classes give styles in `styles`",
            ),
            (
                format!("{ONE}styles = {{ staves = 1 }}\n"),
                "`staves` is a weapon skill, not a fighting style",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn classes_give_mana() {
        assert_eq!(
            parse(&format!("{ONE}mana = 8\n")).expect("valid")[0].mana,
            8
        );
        assert_eq!(parse(ONE).expect("valid")[0].mana, 0, "none unless named");
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
        // Level 1's (Family wealth, #1404; the Stipend, #1405) and nothing more before Fencing at 6.
        assert_eq!(at(5), ["family_wealth", "stipend"]);
        assert_eq!(at(18), ["family_wealth", "stipend", "fencing", "riposte"]);
        assert!(at(60).contains(&"peerless") && !at(60).contains(&"iron_will"));
        assert_eq!(at(60).len(), 5);
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
            (("\"delver\"]", "\"nonesuch\"]"), "unknown tree `nonesuch`"),
        ] {
            let e = with_aristocrat(|t| t.replacen(edit.0, edit.1, 1)).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        // Rows out of order are read by level, lowest first.
        let content = with_aristocrat(|t| t.replacen("level = 50,", "level = 2,", 1))
            .expect("rows in any order");
        let aristocrat = content.class("aristocrat").expect("aristocrat");
        let levels: Vec<u8> = aristocrat.abilities.iter().map(|(l, _)| *l).collect();
        assert!(levels.windows(2).all(|w| w[0] <= w[1]), "{levels:?}");
    }

    /// The highest level a class reaches (spec §8 "Prestige classes"): the file's `max_level`,
    /// else the experience file's `class_max`, never past the curve.
    #[test]
    fn max_level_defaults_to_the_class_cap() {
        let content = with_aristocrat(|t| t.to_string()).expect("built-in");
        // The bard alone ends early (the bard design §5); every other class at the default.
        for class in content.classes() {
            let cap = match class.id.as_str() {
                "bard" => 30,
                _ => content.experience().class_max(),
            };
            assert_eq!(class.max_level, cap, "{}", class.id);
        }
        assert_eq!(content.experience().class_max(), 60);
        // A class capped at 60 names it: the same.
        let capped = with_aristocrat(|t| format!("{t}max_level = 60\n")).expect("valid");
        assert_eq!(
            capped.class("aristocrat").expect("aristocrat").max_level,
            60
        );
        let e = with_aristocrat(|t| format!("{t}max_level = 101\n")).unwrap_err();
        assert!(
            e.message
                .contains("max_level 101 is more than the curve's 100"),
            "{}",
            e.message
        );
    }
}
