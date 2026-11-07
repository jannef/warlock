//! The animal companion's data (`assets/data/companions.toml`; spec §8 "Allies": "The animal
//! companion"; docs/design/ranger-companion.md §12.1; `docs/dev/companion.md`): the starting
//! beasts by race and the rares, the taming numbers, the growth table, and the names. The rules
//! live in `world/companion.rs`; the ally core's constants are `allies.toml`. The file is
//! optional: without it no companion can be made, and nothing here is read.

use super::{Content, ContentError, CreatureId};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The chapter of the background story told to a character with a starting beast.
pub const CHAPTER: &str = "companion";

/// The numbers of the companion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanionRules {
    /// The jackpot, in a thousand: a beast of `rare` instead of the race's list.
    pub rare_per_mille: u32,
    /// The rares, shared by every race, in file order.
    pub rare: Vec<CreatureId>,
    /// A race's list holds beasts of at most this level.
    pub most_level: u8,
    /// Every race's starting beasts, by race id, in file order.
    pub by_race: Vec<(String, Vec<CreatureId>)>,
    pub tame: TameRules,
    pub growth: Growth,
    /// What a companion may be called.
    pub names: Vec<String>,
}

/// The taming check's numbers (the design's C2 reads them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TameRules {
    /// Actions beside the beast before the check.
    pub turns: u32,
    /// The difficulty, plus `per_level` × the beast's level.
    pub difficulty: i32,
    pub per_level: i32,
    /// The bonus for food offered.
    pub food: i32,
}

/// How a companion grows with its master (`world/companion.rs`, the hooks in `world/actor.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    /// The grade in percent of the master's level, never below the type's own level.
    pub grade_percent: u32,
    /// Hit points per level of grade above the type's level.
    pub hp_per_level: i32,
    /// +1 to hit per this many levels above.
    pub to_hit_every: i32,
    /// +1 damage per this many levels above.
    pub damage_bonus_every: i32,
    /// One more die of the attack's kind per this many levels above.
    pub damage_dice_every: i32,
    /// +1 natural Defence per this many levels above.
    pub defence_every: i32,
    /// +1 Strength, Dexterity, and Toughness per this many levels above.
    pub stat_every: i32,
}

impl Growth {
    /// How far grade `grade` of a type of level `level` stands above it, never below 0.
    pub fn above(grade: u8, level: u8) -> i32 {
        i32::from(grade.saturating_sub(level))
    }

    /// The grade a master of `level` gives: `grade_percent` of it (rounded down), never below
    /// the type's `own` level, nor above the highest level there is.
    pub fn grade(&self, level: u8, own: u8) -> u8 {
        let scaled = u32::from(level) * self.grade_percent / 100;
        u8::try_from(scaled).unwrap_or(u8::MAX).max(own)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    start: Start,
    tame: Tame,
    growth: GrowthSpec,
    names: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    rare_per_mille: u32,
    rare: Vec<String>,
    most_level: u8,
    by_race: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tame {
    turns: u32,
    difficulty: i32,
    per_level: i32,
    food: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GrowthSpec {
    grade_percent: u32,
    hp_per_level: i32,
    to_hit_every: i32,
    damage_bonus_every: i32,
    damage_dice_every: i32,
    defence_every: i32,
    stat_every: i32,
}

impl CompanionRules {
    /// The beasts of race `race`'s list, if it has one.
    pub fn list_of(&self, race: &str) -> Option<&[CreatureId]> {
        (self.by_race.iter())
            .find(|(r, _)| r == race)
            .map(|(_, list)| list.as_slice())
    }

    /// Every beast a start may draw, the races' lists and the rares, each once, in file order.
    pub fn every_start(&self) -> Vec<CreatureId> {
        let mut out: Vec<CreatureId> = Vec::new();
        for id in (self.by_race.iter().flat_map(|(_, l)| l.iter())).chain(&self.rare) {
            if !out.contains(id) {
                out.push(*id);
            }
        }
        out
    }

    /// Parses `data/companions.toml` once the creatures and the races are known: every list is a
    /// known race's and holds known creatures (a race without a list draws no starting beast:
    /// test fixtures add races), the lists and the names are not empty (the draws index them),
    /// and the growth's divisors are 1 or more.
    pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let beast = |where_: &str, name: &str| -> Result<CreatureId, ContentError> {
            (content.creature_id(name))
                .filter(|_| name != "player")
                .ok_or_else(|| err(format!("{where_}: unknown creature `{name}`")))
        };
        // The draws index the lists and the names.
        if file.start.rare.is_empty() {
            return Err(err("`start.rare` lists at least one beast".into()));
        }
        let rare = (file.start.rare.iter())
            .map(|n| beast("`start.rare`", n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut by_race = Vec::new();
        for race in &content.races().list {
            let Some(names) = file.start.by_race.get(&race.id) else {
                continue;
            };
            if names.is_empty() {
                return Err(err(format!(
                    "`start.by_race.{}` lists at least one beast",
                    race.id
                )));
            }
            let where_ = format!("`start.by_race.{}`", race.id);
            let list = (names.iter())
                .map(|name| beast(&where_, name))
                .collect::<Result<Vec<_>, _>>()?;
            by_race.push((race.id.clone(), list));
        }
        // A list for a race the content lacks (where it has races at all: a fixture without
        // `races.toml` keeps the shipped file).
        if !content.races().list.is_empty()
            && let Some(race) = (file.start.by_race.keys()).find(|r| content.race(r).is_none())
        {
            return Err(err(format!("`start.by_race`: unknown race `{race}`")));
        }
        let g = file.growth;
        // Divisors of the growth.
        for (name, every) in [
            ("to_hit_every", g.to_hit_every),
            ("damage_bonus_every", g.damage_bonus_every),
            ("damage_dice_every", g.damage_dice_every),
            ("defence_every", g.defence_every),
            ("stat_every", g.stat_every),
        ] {
            if every < 1 {
                return Err(err(format!("`growth.{name}` is at least 1, not {every}")));
            }
        }
        if file.names.is_empty() {
            return Err(err("`names`: at least one name".into()));
        }
        let rules = CompanionRules {
            rare_per_mille: file.start.rare_per_mille,
            rare,
            most_level: file.start.most_level,
            by_race,
            tame: TameRules {
                turns: file.tame.turns,
                difficulty: file.tame.difficulty,
                per_level: file.tame.per_level,
                food: file.tame.food,
            },
            growth: Growth {
                grade_percent: g.grade_percent,
                hp_per_level: g.hp_per_level,
                to_hit_every: g.to_hit_every,
                damage_bonus_every: g.damage_bonus_every,
                damage_dice_every: g.damage_dice_every,
                defence_every: g.defence_every,
                stat_every: g.stat_every,
            },
            names: file.names,
        };
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Content, ContentFiles};

    const COMPANIONS: &str = include_str!("../../../../assets/data/companions.toml");

    fn with(path: &str, text: &str) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        files.insert(path, text);
        Content::load(&files).map_err(|e| e.message)
    }

    fn companions(edit: impl Fn(&str) -> String) -> Result<Content, String> {
        with("data/companions.toml", &edit(COMPANIONS))
    }

    #[test]
    fn the_shipped_file_loads_and_every_race_has_a_list() {
        let content = Content::builtin().expect("content");
        let rules = content.companions().expect("companions.toml");
        for race in &content.races().list {
            let list = rules
                .list_of(&race.id)
                .unwrap_or_else(|| panic!("{}", race.id));
            assert!(!list.is_empty(), "{}", race.id);
        }
        assert_eq!(rules.rare.len(), 4);
        assert_eq!(rules.growth.grade(30, 1), 30);
        assert_eq!(rules.growth.grade(1, 6), 6, "never below the type's level");
        assert_eq!(super::Growth::above(30, 4), 26);
        assert_eq!(super::Growth::above(2, 6), 0);
    }

    #[test]
    fn the_file_is_checked() {
        let err = |r: Result<Content, String>| r.expect_err("refused");
        assert!(
            err(companions(|t| t.replace("\"wolf\", ", "\"wolfe\", ")))
                .contains("unknown creature `wolfe`"),
        );
        assert!(
            err(companions(
                |t| t.replace("human = [\"hound\", \"boar\"]", "human = []")
            ))
            .contains("lists at least one"),
        );
        assert!(
            err(companions(|t| t.replace("ratfolk = ", "elvish = ")))
                .contains("unknown race `elvish`"),
        );
        assert!(
            err(companions(
                |t| t.replace("to_hit_every = 4", "to_hit_every = 0")
            ))
            .contains("to_hit_every")
        );
        // Without the file, no companion at all, and nothing else minds.
        let mut files = ContentFiles::builtin();
        files.remove("data/companions.toml");
        let content = Content::load(&files).expect("loads without");
        assert!(content.companions().is_none());
    }

    #[test]
    fn the_wolf_is_tameable_and_has_a_nose() {
        let content = Content::builtin().expect("content");
        let wolf = content.creature_id("wolf").unwrap();
        assert_eq!(content.creature(wolf).nose, Some(8));
        assert!(content.creature(wolf).tameable);
        let lynx = content.creature_id("lynx").unwrap();
        assert_eq!(content.creature(lynx).nose, None);
    }
}
