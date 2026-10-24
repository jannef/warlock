//! Races (`assets/data/races.toml`, spec §8 "Character creation"): the first choice of a new
//! run. A race adjusts the starting scores and the potentials (fixed numbers or dice), gives the
//! player their eyes, sets the range of their starting age, gives the player their size
//! (spec §8 "Burden": how much they can carry), whether being Burdened costs them nothing, and
//! how much they need to eat (spec §8 "Hunger"), how much experience their levels need (spec §8
//! "Experience"), trains them in skills (spec §8 "Skills"; dice,
//! and humans' extra class skill), gives them starting ranks in weapon skills (spec §8
//! "Weapon skills"; fixed numbers or dice), says what they notice of hidden features in passing
//! (spec §8 "Searching"), and adds a few things to the starting kit (`kit`, [`super::kit`]).

use super::kit::{Kit, KitLineSpec};
use super::weapon_skill::Ranked;
use super::{ContentError, FACTORS, PassiveSearch, WeaponSkillId, WeaponSkillTable};
use crate::combat::MAX_BASE_HP;
use crate::dice::Dice;
use crate::skills::{self, GRANTS, SkillId};
use crate::stats::{PerStat, SCORES, Stat, adjustments};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// How much a race may need to eat, in percent of a human: a tenth to ten times as much.
pub const HUNGER: RangeInclusive<u16> = 10..=1000;

fn normal_hunger() -> u16 {
    100
}

fn normal_experience() -> u16 {
    100
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RaceFile {
    base_potential: u8,
    race: Vec<RaceSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RaceSpec {
    id: String,
    name: String,
    description: String,
    vision: String,
    age: [u32; 2],
    #[serde(default)]
    stats: BTreeMap<Stat, Dice>,
    #[serde(default)]
    potential: BTreeMap<Stat, Dice>,
    #[serde(default)]
    hp: u16,
    #[serde(default)]
    alignment: i32,
    size: Option<String>,
    #[serde(default)]
    ignores_burdened: bool,
    passive_search: Option<PassiveSearch>,
    #[serde(default)]
    search: i32,
    #[serde(default)]
    skills: BTreeMap<String, Dice>,
    #[serde(default)]
    class_skill: Dice,
    #[serde(default = "normal_hunger")]
    hunger: u16,
    #[serde(default = "normal_experience")]
    experience: u16,
    #[serde(default)]
    weapon_skills: BTreeMap<String, Dice>,
    #[serde(default)]
    kit: Vec<KitLineSpec>,
}

/// The oldest a character may start, in years.
pub const MAX_AGE: u32 = 1000;

/// What a race may add to searching and noticing (spec §8 "Searching").
pub const SEARCH_BONUSES: RangeInclusive<i32> = -10..=10;

/// A race, in the order offered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceDef {
    pub id: String,
    pub name: String,
    /// One line, shown when choosing.
    pub description: String,
    /// The race's eyes: a profile id in `data/vision.toml`.
    pub vision: String,
    /// The starting age in years: a character is this old, uniformly at random, when a run
    /// starts.
    pub age: RangeInclusive<u32>,
    /// Adjustments to the starting scores.
    pub stats: PerStat<Dice>,
    /// Adjustments to the potentials on [`Races::base_potential`]; hidden from the player.
    pub potential: PerStat<Dice>,
    /// What the race adds to the hit points (spec §8 "Combat"): maximum hit points are
    /// Toughness plus the race's and the class's.
    pub hp: i32,
    /// Its part of a new character's alignment score (spec §8 "Alignment"), added to the
    /// class's.
    pub alignment: i32,
    /// Its size (spec §8 "Burden"): an index into [`super::Content::sizes`], which scales how
    /// much the player can carry.
    pub size: usize,
    /// Whether the player suffers nothing for being Burdened, the first level of burden (no
    /// slowdown, no cap on Dexterity in Defence, no louder steps); from Strained on they are
    /// treated like everyone else. Dwarves.
    pub ignores_burdened: bool,
    /// Which hidden features the player notices in passing, untrained (spec §8 "Searching"):
    /// elves everything, dwarves what is on stone; `None`: only with the Spot skill trained.
    pub passive_search: Option<PassiveSearch>,
    /// Points added to searching and to noticing in passing alike (spec §8 "Searching").
    pub search: i32,
    /// The skills it trains (spec §8 "Skills"), each by dice rolled when the character is made,
    /// in skill order; added to the class's.
    pub skills: Vec<(SkillId, Dice)>,
    /// Dice added to one of the class's skills, picked when the character is made (0: none).
    /// Humans: one extra class skill.
    pub class_skill: Dice,
    /// How much the player needs to eat, in percent of a human (spec §8 "Hunger"): game time
    /// burns their nutrition this much faster (a halfling, 125) or slower (an elf, 80).
    pub hunger: u16,
    /// How much experience a level needs, in percent (spec §8 "Experience"): multiplied with
    /// the class's onto every threshold. Long-lived races pay more (an elf, 140).
    pub experience: u16,
    /// Starting ranks in weapon skills (spec §8 "Weapon skills"), in skill order: added to the
    /// class's.
    pub weapon_skills: Vec<(WeaponSkillId, Dice)>,
    /// The starting kit as written; resolved by the loader into `kit`.
    pub(super) kit_spec: Vec<KitLineSpec>,
    /// What the race adds to a new character's pack after the class's kit (spec §8 "Character
    /// creation"); filled in by the loader once objects are known.
    pub kit: Kit,
}

/// Every race, and the base every potential starts from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Races {
    /// What every potential is before the race adjusts it.
    pub base_potential: u8,
    /// In file order: the order offered.
    pub list: Vec<RaceDef>,
}

impl Races {
    /// Parses `data/races.toml`; `has_vision` says whether a vision profile id exists,
    /// `size_index` where a size id is in [`super::Content::sizes`], `skill_id` which skill a
    /// skill id is, and `weapon_table` the weapon skills starting ranks may name.
    pub(super) fn parse(
        file: &str,
        text: &str,
        has_vision: impl Fn(&str) -> bool,
        size_index: impl Fn(&str) -> Option<usize>,
        skill_id: impl Fn(&str) -> Option<SkillId>,
        weapon_table: &WeaponSkillTable,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RaceFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !SCORES.contains(&parsed.base_potential) {
            return Err(err(format!(
                "base_potential {} is out of range ({} to {})",
                parsed.base_potential,
                SCORES.start(),
                SCORES.end()
            )));
        }
        if parsed.race.is_empty() {
            return Err(err("at least one race is required".into()));
        }
        let mut list: Vec<RaceDef> = Vec::new();
        for spec in parsed.race {
            let id = &spec.id;
            if list.iter().any(|r| &r.id == id) {
                return Err(err(format!("duplicate race id `{id}`")));
            }
            if !has_vision(&spec.vision) {
                return Err(err(format!(
                    "race `{id}`: unknown vision profile `{}`",
                    spec.vision
                )));
            }
            let [young, old] = spec.age;
            if young == 0 || young > old || old > MAX_AGE {
                return Err(err(format!(
                    "race `{id}`: age [{young}, {old}] must be [youngest, oldest] within 1 to \
                     {MAX_AGE}"
                )));
            }
            let stats =
                adjustments(&spec.stats).map_err(|m| err(format!("race `{id}`: stats: {m}")))?;
            let potential = adjustments(&spec.potential)
                .map_err(|m| err(format!("race `{id}`: potential: {m}")))?;
            if spec.hp > MAX_BASE_HP {
                return Err(err(format!(
                    "race `{id}`: hp {} is more than {MAX_BASE_HP}",
                    spec.hp
                )));
            }
            let alignment = crate::alignment::check_start(spec.alignment)
                .map_err(|m| err(format!("race `{id}`: {m}")))?;
            let size_id = spec.size.as_deref().unwrap_or(super::MEDIUM);
            let size = size_index(size_id)
                .ok_or_else(|| err(format!("race `{id}`: unknown size `{size_id}`")))?;
            let skills = skills::grants(&spec.skills, &skill_id)
                .map_err(|m| err(format!("race `{id}`: skills: {m}")))?;
            let extra = spec.class_skill;
            if !GRANTS.contains(&extra.min()) || !GRANTS.contains(&extra.max()) {
                return Err(err(format!(
                    "race `{id}`: class_skill {extra} is out of range ({} to {})",
                    GRANTS.start(),
                    GRANTS.end()
                )));
            }
            if !HUNGER.contains(&spec.hunger) {
                return Err(err(format!(
                    "race `{id}`: hunger {} is out of range ({} to {} percent)",
                    spec.hunger,
                    HUNGER.start(),
                    HUNGER.end()
                )));
            }
            if !FACTORS.contains(&spec.experience) {
                return Err(err(format!(
                    "race `{id}`: experience {} is out of range ({} to {} percent)",
                    spec.experience,
                    FACTORS.start(),
                    FACTORS.end()
                )));
            }
            if !SEARCH_BONUSES.contains(&spec.search) {
                return Err(err(format!(
                    "race `{id}`: search {} is out of range ({} to {})",
                    spec.search,
                    SEARCH_BONUSES.start(),
                    SEARCH_BONUSES.end()
                )));
            }
            let weapon_skills = (weapon_table.starting_ranks(&spec.weapon_skills, Ranked::Weapons))
                .map_err(|m| err(format!("race `{id}`: {m}")))?;
            list.push(RaceDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                vision: spec.vision,
                age: young..=old,
                stats,
                potential,
                hp: i32::from(spec.hp),
                alignment,
                size,
                ignores_burdened: spec.ignores_burdened,
                passive_search: spec.passive_search,
                search: spec.search,
                skills,
                class_skill: extra,
                hunger: spec.hunger,
                experience: spec.experience,
                weapon_skills,
                kit_spec: spec.kit,
                kit: Kit::default(),
            });
        }
        Ok(Self {
            base_potential: parsed.base_potential,
            list,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Races, ContentError> {
        let sizes = ["small", "medium"];
        let skills = ["listen", "spot"];
        let content = super::super::Content::builtin().expect("content");
        Races::parse(
            "races.toml",
            text,
            |v| v == "normal",
            |s| sizes.iter().position(|id| *id == s),
            |k| (skills.iter().position(|id| *id == k)).map(|i| SkillId(i as u8)),
            content.weapon_skills(),
        )
    }

    const ONE: &str = "base_potential = 15\n[[race]]\nid = \"human\"\nname = \"Human\"\n\
                       description = \"Everywhere.\"\nvision = \"normal\"\n\
                       age = [16, 35]\n";

    #[test]
    fn races_read_adjustments_that_default_to_zero() {
        let races = parse(&format!(
            "{ONE}stats = {{ strength = 2, toughness = \"1d2+1\" }}\n\
             potential = {{ presence = -1, willpower = \"1d2\" }}\n"
        ))
        .expect("valid");
        assert_eq!(races.base_potential, 15);
        let human = &races.list[0];
        assert_eq!(human.age, 16..=35);
        assert_eq!(human.stats[Stat::Strength], Dice::fixed(2));
        assert_eq!(human.stats[Stat::Toughness].to_string(), "1d2+1");
        assert!(human.stats[Stat::Dexterity].is_zero());
        assert_eq!(human.potential[Stat::Presence], Dice::fixed(-1));
        assert_eq!(human.potential[Stat::Willpower].to_string(), "1d2");
        assert_eq!(human.size, 1, "medium unless named");
        assert!(!human.ignores_burdened);
        assert!(human.skills.is_empty() && human.class_skill.is_zero());
        assert_eq!(human.hunger, 100, "a human's hunger unless named");
        assert_eq!(human.experience, 100, "a human's experience unless named");
        let dwarf = parse(&format!(
            "{ONE}size = \"small\"\nignores_burdened = true\nhunger = 110\nexperience = 115\n"
        ))
        .expect("valid");
        assert_eq!(dwarf.list[0].size, 0);
        assert!(dwarf.list[0].ignores_burdened);
        assert_eq!(dwarf.list[0].hunger, 110);
        assert_eq!(dwarf.list[0].experience, 115);
        let elf = parse(&format!(
            "{ONE}skills = {{ spot = \"1d6+4\", listen = 3 }}\nclass_skill = \"1d4\"\n"
        ))
        .expect("valid");
        let elf = &elf.list[0];
        assert_eq!(
            elf.skills,
            [
                (SkillId(0), Dice::fixed(3)),
                (SkillId(1), "1d6+4".parse().expect("dice"))
            ],
            "in skill order"
        );
        assert_eq!(elf.class_skill.to_string(), "1d4");
        assert!(human.weapon_skills.is_empty());
        let elf = parse(&format!(
            "{ONE}weapon_skills = {{ long_blades = 1, bows = \"1d2\" }}\n"
        ))
        .expect("valid");
        let ranks: Vec<String> = (elf.list[0].weapon_skills.iter())
            .map(|(id, dice)| format!("{} {dice}", id.0))
            .collect();
        // In skill order: long blades (2) before bows (10).
        assert_eq!(ranks, ["2 +1", "10 1d2"]);
    }

    #[test]
    fn bad_races_are_refused() {
        for (text, why) in [
            (
                ONE.replace("normal", "x-ray"),
                "unknown vision profile `x-ray`",
            ),
            (
                format!("{ONE}stats = {{ luck = 1 }}\n"),
                "unknown stat `luck`",
            ),
            (
                format!("{ONE}stats = {{ strength = 11 }}\n"),
                "strength +11 is out of range",
            ),
            (
                format!("{ONE}stats = {{ strength = \"2d6\" }}\n"),
                "strength 2d6 is out of range",
            ),
            (
                format!("{ONE}stats = {{ strength = \"2x\" }}\n"),
                "bad dice `2x`",
            ),
            (ONE.replace("[16, 35]", "[35, 16]"), "age [35, 16]"),
            (ONE.replace("[16, 35]", "[0, 16]"), "age [0, 16]"),
            (ONE.replace("[16, 35]", "[16, 1001]"), "within 1 to 1000"),
            (ONE.replace("age = [16, 35]\n", ""), "missing field `age`"),
            (
                format!("{ONE}potential = {{ toughness = -12 }}\n"),
                "potential: toughness",
            ),
            (ONE.replace("15", "0"), "base_potential 0"),
            (format!("{ONE}size = \"huge\"\n"), "unknown size `huge`"),
            (format!("{ONE}hunger = 5\n"), "hunger 5 is out of range"),
            (
                format!("{ONE}experience = 49\n"),
                "experience 49 is out of range",
            ),
            (
                format!("{ONE}experience = 201\n"),
                "experience 201 is out of range",
            ),
            (
                format!("{ONE}skills = {{ luck = 5 }}\n"),
                "race `human`: skills: unknown skill `luck`",
            ),
            (
                format!("{ONE}skills = {{ spot = \"1d6-3\" }}\n"),
                "spot 1d6-3 is out of range",
            ),
            (
                format!("{ONE}class_skill = \"1d100+5\"\n"),
                "class_skill 1d100+5 is out of range",
            ),
            (
                format!("{ONE}weapon_skills = {{ whips = 1 }}\n"),
                "race `human`: unknown weapon skill `whips`",
            ),
            (
                format!("{ONE}weapon_skills = {{ bows = \"1d12\" }}\n"),
                "must give a rank from 0 to 10",
            ),
            (
                format!("{ONE}weapon_skills = {{ bows = -1 }}\n"),
                "must give a rank from 0 to 10",
            ),
            (
                format!("{ONE}{}", ONE.replace("base_potential = 15\n", "")),
                "duplicate race id `human`",
            ),
            (
                "base_potential = 15\nrace = []\n".into(),
                "at least one race",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
