//! Races (`assets/data/races.toml`, spec §8 "Character creation"): the first choice of a new
//! run. A race adjusts the starting scores and the potentials (fixed numbers or dice), gives the
//! player their eyes, sets the range of their starting age, and gives the player their size
//! (spec §8 "Burden": how much they can carry) and whether being Burdened costs them nothing.

use super::ContentError;
use crate::combat::MAX_BASE_HP;
use crate::dice::Dice;
use crate::stats::{PerStat, SCORES, Stat, adjustments};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

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
    size: Option<String>,
    #[serde(default)]
    ignores_burdened: bool,
}

/// The oldest a character may start, in years.
pub const MAX_AGE: u32 = 1000;

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
    /// Its size (spec §8 "Burden"): an index into [`super::Content::sizes`], which scales how
    /// much the player can carry.
    pub size: usize,
    /// Whether the player suffers nothing for being Burdened, the first level of burden (no
    /// slowdown, no cap on Dexterity in Defence, no louder steps); from Strained on they are
    /// treated like everyone else. Dwarves.
    pub ignores_burdened: bool,
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
    /// Parses `data/races.toml`; `has_vision` says whether a vision profile id exists, and
    /// `size_index` where a size id is in [`super::Content::sizes`].
    pub(super) fn parse(
        file: &str,
        text: &str,
        has_vision: impl Fn(&str) -> bool,
        size_index: impl Fn(&str) -> Option<usize>,
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
            let size_id = spec.size.as_deref().unwrap_or(super::MEDIUM);
            let size = size_index(size_id)
                .ok_or_else(|| err(format!("race `{id}`: unknown size `{size_id}`")))?;
            list.push(RaceDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                vision: spec.vision,
                age: young..=old,
                stats,
                potential,
                hp: i32::from(spec.hp),
                size,
                ignores_burdened: spec.ignores_burdened,
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
        Races::parse(
            "races.toml",
            text,
            |v| v == "normal",
            |s| sizes.iter().position(|id| *id == s),
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
        let dwarf =
            parse(&format!("{ONE}size = \"small\"\nignores_burdened = true\n")).expect("valid");
        assert_eq!(dwarf.list[0].size, 0);
        assert!(dwarf.list[0].ignores_burdened);
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
