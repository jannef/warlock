//! The animal companion's data (`assets/data/companions.toml`; spec §8 "Allies": "The animal
//! companion"; docs/design/ranger-companion.md §12.1; `docs/dev/companion.md`): the starting
//! beasts by race and the rares, the taming numbers, the growth table, and the names. The rules
//! live in `world/allies/companion.rs`; the ally core's constants are `allies.toml`. The file is
//! optional: without it no companion can be made, and nothing here is read.

use super::race_beasts::{self, RaceBeasts, StartSpec};
use super::{Content, ContentError};
use serde::Deserialize;

/// The chapter of the background story told to a character with a starting beast.
pub const CHAPTER: &str = "companion";

/// The numbers of the companion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanionRules {
    /// The starting beasts by race and the rares (`[start]`).
    pub start: RaceBeasts,
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

/// How a companion grows with its master (`world/allies/companion.rs`, the hooks in `world/turn/actor.rs`).
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
    start: StartSpec,
    tame: Tame,
    growth: GrowthSpec,
    names: Vec<String>,
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
    /// Parses `data/companions.toml` once the creatures and the races are known: its `[start]`
    /// as [`RaceBeasts::parse`] checks it, the names not empty (the draw indexes them), and the
    /// growth's divisors 1 or more.
    pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let start = RaceBeasts::parse(path, file.start, content)?;
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
        let rules = CompanionRules {
            start,
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
            names: race_beasts::names(path, file.names)?,
        };
        Ok(rules)
    }
}

impl super::RulesFile for CompanionRules {
    const PATH: &'static str = "data/companions.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
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
        let content = crate::test_support::builtin();
        let rules = content.companions().expect("companions.toml");
        for race in &content.races().list {
            let list = rules
                .start
                .list_of(&race.id)
                .unwrap_or_else(|| panic!("{}", race.id));
            assert!(!list.is_empty(), "{}", race.id);
        }
        assert_eq!(rules.start.rare.len(), 4);
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
        let content = crate::test_support::builtin();
        let wolf = content.creature_id("wolf").unwrap();
        assert_eq!(content.creature(wolf).nose, Some(8));
        assert!(content.creature(wolf).tameable);
        let lynx = content.creature_id("lynx").unwrap();
        assert_eq!(content.creature(lynx).nose, None);
    }
}
