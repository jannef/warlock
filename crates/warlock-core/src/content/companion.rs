//! The animal companion's data (`assets/data/companions.toml`; spec §8 "Allies": "The animal
//! companion"; docs/design/ranger-companion.md §12.1; `docs/dev/companion.md`): the starting
//! beasts by race and the rares, the taming numbers, the growth table, and the names. The rules
//! live in `world/companion.rs`; the ally core's constants are `allies.toml`. The file is
//! optional: without it no companion can be made, and nothing here is read.

use super::{Content, ContentError, CreatureId};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// A race's list holds this many beasts.
pub const LIST_SIZES: RangeInclusive<usize> = 1..=3;
/// The rare list holds this many.
pub const RARE_SIZES: RangeInclusive<usize> = 1..=8;
/// The taming channel, in actions.
pub const TURNS: RangeInclusive<u32> = 1..=100;
/// A grade in percent of the master's level.
pub const GRADE_PERCENTS: RangeInclusive<u32> = 1..=300;
/// Each `every` of the growth table, in levels.
pub const EVERIES: RangeInclusive<u32> = 1..=60;
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

    /// Parses `data/companions.toml` once the creatures, the races, and the background story
    /// are known, and checks it: every list is a known race's and holds 1 to 3 known `tameable`
    /// beasts of at most `most_level` (a race without a list draws no starting beast: test
    /// fixtures add races; `companion_tests` asks every shipped race for one), the rares 1 to 8
    /// known `tameable` beasts, every number in range, at least one name, and, when the content
    /// tells stories and has the companion chapter, an `only` line for every beast listed.
    pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let beast = |where_: &str, name: &str| -> Result<CreatureId, ContentError> {
            let id = (content.creature_id(name))
                .ok_or_else(|| err(format!("{where_}: unknown creature `{name}`")))?;
            if !content.creature(id).tameable {
                return Err(err(format!("{where_}: `{name}` is not `tameable`")));
            }
            Ok(id)
        };
        if !(0..=1000).contains(&file.start.rare_per_mille) {
            return Err(err(format!(
                "`start.rare_per_mille` is 0 to 1000, not {}",
                file.start.rare_per_mille
            )));
        }
        if !RARE_SIZES.contains(&file.start.rare.len()) {
            return Err(err(format!(
                "`start.rare` lists {} to {} beasts, not {}",
                RARE_SIZES.start(),
                RARE_SIZES.end(),
                file.start.rare.len()
            )));
        }
        let rare = (file.start.rare.iter())
            .map(|n| beast("`start.rare`", n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut by_race = Vec::new();
        for race in &content.races().list {
            let Some(names) = file.start.by_race.get(&race.id) else {
                continue;
            };
            if !LIST_SIZES.contains(&names.len()) {
                return Err(err(format!(
                    "`start.by_race.{}` lists {} to {} beasts, not {}",
                    race.id,
                    LIST_SIZES.start(),
                    LIST_SIZES.end(),
                    names.len()
                )));
            }
            let where_ = format!("`start.by_race.{}`", race.id);
            let mut list = Vec::new();
            for name in names {
                let id = beast(&where_, name)?;
                let level = content.creature(id).level;
                if level > file.start.most_level {
                    return Err(err(format!(
                        "{where_}: `{name}` is level {level}, above `most_level` {}",
                        file.start.most_level
                    )));
                }
                list.push(id);
            }
            by_race.push((race.id.clone(), list));
        }
        // A list for a race the content lacks (where it has races at all: a fixture without
        // `races.toml` keeps the shipped file).
        if !content.races().list.is_empty()
            && let Some(race) = (file.start.by_race.keys()).find(|r| content.race(r).is_none())
        {
            return Err(err(format!("`start.by_race`: unknown race `{race}`")));
        }
        if !TURNS.contains(&file.tame.turns) {
            return Err(err(format!(
                "`tame.turns` is {} to {}, not {}",
                TURNS.start(),
                TURNS.end(),
                file.tame.turns
            )));
        }
        if file.tame.difficulty < 0 || file.tame.per_level < 0 || file.tame.food < 0 {
            return Err(err(
                "`tame`: difficulty, per_level, and food are 0 or more".into()
            ));
        }
        let g = file.growth;
        if !GRADE_PERCENTS.contains(&g.grade_percent) {
            return Err(err(format!(
                "`growth.grade_percent` is {} to {}, not {}",
                GRADE_PERCENTS.start(),
                GRADE_PERCENTS.end(),
                g.grade_percent
            )));
        }
        if g.hp_per_level < 0 {
            return Err(err("`growth.hp_per_level` is 0 or more".into()));
        }
        for (name, every) in [
            ("to_hit_every", g.to_hit_every),
            ("damage_bonus_every", g.damage_bonus_every),
            ("damage_dice_every", g.damage_dice_every),
            ("defence_every", g.defence_every),
            ("stat_every", g.stat_every),
        ] {
            if !u32::try_from(every).is_ok_and(|e| EVERIES.contains(&e)) {
                return Err(err(format!(
                    "`growth.{name}` is {} to {}, not {every}",
                    EVERIES.start(),
                    EVERIES.end()
                )));
            }
        }
        if file.names.is_empty() || file.names.iter().any(|n| n.trim().is_empty()) {
            return Err(err("`names`: at least one name, none empty".into()));
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
        // The story's chapter: every beast a start may draw has its `only` line.
        if let Some(chapter) = (content.background()).and_then(|b| {
            b.chapters
                .iter()
                .find(|c| c.id == CHAPTER && c.campaign.is_none())
        }) {
            for id in rules.every_start() {
                let told = (chapter.lines.iter()).any(|l| l.only && l.when.companion == [id]);
                if !told {
                    return Err(err(format!(
                        "the `{CHAPTER}` chapter has no `only` line for `{}` (`when = {{ companion = [\"{}\"] }}`)",
                        content.creature(id).id,
                        content.creature(id).id
                    )));
                }
            }
        }
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
            assert!(super::LIST_SIZES.contains(&list.len()), "{}", race.id);
        }
        assert_eq!(rules.rare.len(), 4);
        assert_eq!(rules.growth.grade(30, 1), 30);
        assert_eq!(rules.growth.grade(1, 6), 6, "never below the type's level");
        assert_eq!(super::Growth::above(30, 4), 26);
        assert_eq!(super::Growth::above(2, 6), 0);
        // Every beast a start may draw has its line in the story.
        let chapter = (content.background().unwrap().chapter("", super::CHAPTER)).unwrap();
        for id in rules.every_start() {
            assert!(
                (chapter.lines.iter()).any(|l| l.only && l.when.companion == [id]),
                "{}",
                content.creature(id).id
            );
        }
    }

    #[test]
    fn the_file_is_checked() {
        let err = |r: Result<Content, String>| r.expect_err("refused");
        assert!(
            err(companions(|t| t.replace("\"wolf\", ", "\"kobold\", "))).contains("not `tameable`"),
            "a rare must be tameable"
        );
        assert!(
            err(companions(|t| t.replace(
                "human = [\"hound\", \"boar\"]",
                "human = [\"bear\"]"
            )))
            .contains("above `most_level`"),
            "a race's list keeps to most_level"
        );
        assert!(
            err(companions(
                |t| t.replace("human = [\"hound\", \"boar\"]", "human = []")
            ))
            .contains("lists 1 to 3"),
        );
        assert!(
            err(companions(|t| t.replace("ratfolk = ", "elvish = ")))
                .contains("unknown race `elvish`"),
        );
        assert!(
            err(companions(
                |t| t.replace("rare_per_mille = 50", "rare_per_mille = 1001")
            ))
            .contains("0 to 1000")
        );
        assert!(
            err(companions(
                |t| t.replace("grade_percent = 100", "grade_percent = 0")
            ))
            .contains("grade_percent")
        );
        assert!(
            err(companions(
                |t| t.replace("to_hit_every = 4", "to_hit_every = 0")
            ))
            .contains("to_hit_every")
        );
        assert!(err(companions(|t| t.replace("turns = 10", "turns = 0"))).contains("tame.turns"));
        assert!(
            err(companions(|t| t.replace("names = [", "names = [\"\", "))).contains("none empty")
        );
        // A start beast without its `only` line in the chapter.
        let chapter = include_str!("../../../../assets/data/background/08-companion.toml");
        let cut = chapter.replace(
            "when = { companion = [\"hound\"] }",
            "when = { companion = [\"boar\"] }",
        );
        assert!(
            err(with("data/background/08-companion.toml", &cut))
                .contains("no `only` line for `hound`")
        );
        // Without the file, no companion at all, and nothing else minds.
        let mut files = ContentFiles::builtin();
        files.remove("data/companions.toml");
        let content = Content::load(&files).expect("loads without");
        assert!(content.companions().is_none());
    }

    #[test]
    fn a_tameable_type_is_a_beast_and_a_nose_is_a_tameable_types() {
        let err = |path: &str, text: &str| with(path, text).expect_err("refused");
        let kobold = include_str!("../../../../assets/data/creatures/kobold.toml");
        assert!(
            err(
                "data/creatures/kobold.toml",
                &kobold.replace("opens_doors = true", "opens_doors = true\ntameable = true")
            )
            .contains("`tameable`, but"),
            "a kobold opens doors and carries a spear"
        );
        let deer = include_str!("../../../../assets/data/creatures/deer.toml");
        assert!(
            err(
                "data/creatures/deer.toml",
                &deer.replace("tameable = true", "nose = 6")
            )
            .contains("`nose` is for a `tameable` type")
        );
        let wolf = include_str!("../../../../assets/data/creatures/wolf.toml");
        assert!(
            err(
                "data/creatures/wolf.toml",
                &wolf.replace("nose = 8", "nose = 21")
            )
            .contains("1 to 20 cells")
        );
        let content = Content::builtin().expect("content");
        let wolf = content.creature_id("wolf").unwrap();
        assert_eq!(content.creature(wolf).nose, Some(8));
        assert!(content.creature(wolf).tameable);
        let lynx = content.creature_id("lynx").unwrap();
        assert_eq!(content.creature(lynx).nose, None);
    }
}
