//! Foraging (`assets/data/forage.toml`, spec §8 "Foraging"; #547): how long a forage takes,
//! and for each landscape of the wild its chance of finding food, what it finds, and how much.
//!
//! The rule itself lives in `world/forage.rs`: who forages (anyone trained in a skill with the
//! `forage` benefit, Survival), where (the overworld and its local areas, on land with a row
//! here, never in water), and what comes of it.
//!
//! The file is optional: without it nobody forages, and Survival stays a passive skill.

use super::{ContentError, Landscape, ObjectId};
use crate::dice::Dice;
use serde::Deserialize;

/// The shortest and longest a forage may take, in seconds.
pub const SECONDS: std::ops::RangeInclusive<u32> = 60..=3600;

/// The most of a find one forage may bring.
pub const MAX_COUNT: i32 = 10;

/// The rules of foraging that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForageRules {
    /// How long a forage takes, in seconds of game time.
    pub seconds: u32,
    /// The grounds, in file order: at most one per landscape.
    pub grounds: Vec<ForageGround>,
}

/// What foraging on one landscape gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForageGround {
    pub landscape: Landscape,
    /// Percent of finding something, before what Survival adds.
    pub chance: u8,
    /// How many of the find.
    pub count: Dice,
    /// What may be found, each with its weight, in file order.
    pub finds: Vec<(ObjectId, u32)>,
}

impl ForageRules {
    /// The ground of `landscape`, if it can be foraged.
    pub fn ground(&self, landscape: Landscape) -> Option<&ForageGround> {
        self.grounds.iter().find(|g| g.landscape == landscape)
    }
}

impl ForageGround {
    /// The sum of the finds' weights.
    pub fn total_weight(&self) -> u32 {
        self.finds.iter().map(|(_, w)| *w).sum()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForageFile {
    seconds: u32,
    #[serde(default)]
    ground: Vec<GroundSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroundSpec {
    landscape: Landscape,
    chance: u8,
    count: Dice,
    finds: Vec<FindSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FindSpec {
    object: String,
    weight: u32,
}

impl ForageRules {
    /// Parses `data/forage.toml`: a time within [`SECONDS`]; each ground on land, once, with a
    /// chance of 1 to 100, a count of at least 1 and at most [`MAX_COUNT`], and finds that are
    /// food objects of weight 1 or more.
    pub(super) fn parse(
        file: &str,
        text: &str,
        content: &super::Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: ForageFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !SECONDS.contains(&parsed.seconds) {
            return Err(err(format!(
                "`seconds` must be {} to {}",
                SECONDS.start(),
                SECONDS.end()
            )));
        }
        let mut grounds: Vec<ForageGround> = Vec::new();
        for spec in parsed.ground {
            let name = spec.landscape.id();
            if !spec.landscape.is_land() {
                return Err(err(format!("ground `{name}`: only land can be foraged")));
            }
            if grounds.iter().any(|g| g.landscape == spec.landscape) {
                return Err(err(format!("ground `{name}` is listed twice")));
            }
            if !(1..=100).contains(&spec.chance) {
                return Err(err(format!("ground `{name}`: `chance` must be 1 to 100")));
            }
            if spec.count.min() < 1 || spec.count.max() > MAX_COUNT {
                return Err(err(format!(
                    "ground `{name}`: `count` must give 1 to {MAX_COUNT}"
                )));
            }
            if spec.finds.is_empty() {
                return Err(err(format!("ground `{name}`: `finds` is empty")));
            }
            let mut finds = Vec::new();
            for find in spec.finds {
                let object = (content.object_id(&find.object))
                    .ok_or_else(|| err(format!("unknown object `{}`", find.object)))?;
                if content.object(object).food.is_none() {
                    return Err(err(format!("`{}` is no food", find.object)));
                }
                if find.weight == 0 {
                    return Err(err(format!(
                        "`{}`: `weight` must be at least 1",
                        find.object
                    )));
                }
                finds.push((object, find.weight));
            }
            grounds.push(ForageGround {
                landscape: spec.landscape,
                chance: spec.chance,
                count: spec.count,
                finds,
            });
        }
        Ok(Self {
            seconds: parsed.seconds,
            grounds,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles, Landscape};

    #[test]
    fn the_wild_gives_food_by_landscape() {
        let content = Content::builtin().expect("content");
        let rules = content.forage().expect("foraging is in the game");
        assert_eq!(rules.seconds, 600);
        // Every land landscape has a row; roads and water none.
        for landscape in [
            Landscape::Plains,
            Landscape::Grassland,
            Landscape::Forest,
            Landscape::DenseForest,
            Landscape::Hills,
            Landscape::Swamp,
        ] {
            let ground = rules.ground(landscape).expect("a row");
            assert!(ground.total_weight() > 0);
            for (object, _) in &ground.finds {
                assert!(content.object(*object).food.is_some());
            }
        }
        for landscape in [Landscape::Road, Landscape::River, Landscape::Lake] {
            assert!(rules.ground(landscape).is_none());
        }
        let forest = rules.ground(Landscape::Forest).expect("forest");
        assert_eq!(forest.chance, 40);
        assert_eq!((forest.count.min(), forest.count.max()), (1, 3));
    }

    #[test]
    fn bad_rows_are_refused_and_the_file_is_optional() {
        let row = |landscape: &str, chance: u8, count: &str, object: &str, weight: u32| {
            format!(
                "seconds = 600\n[[ground]]\nlandscape = \"{landscape}\"\nchance = {chance}\n\
                 count = \"{count}\"\nfinds = [{{ object = \"{object}\", weight = {weight} }}]\n"
            )
        };
        for (text, why) in [
            (row("river", 10, "1d2", "apple", 1), "only land"),
            (
                row("forest", 0, "1d2", "apple", 1),
                "`chance` must be 1 to 100",
            ),
            (
                row("forest", 10, "1d20", "apple", 1),
                "`count` must give 1 to 10",
            ),
            (row("forest", 10, "1d2", "dagger", 1), "`dagger` is no food"),
            (
                row("forest", 10, "1d2", "nothing", 1),
                "unknown object `nothing`",
            ),
            (
                row("forest", 10, "1d2", "apple", 0),
                "`weight` must be at least 1",
            ),
            (
                format!(
                    "{}{}",
                    row("forest", 10, "1d2", "apple", 1),
                    row("forest", 10, "1d2", "apple", 1).replace("seconds = 600\n", "")
                ),
                "listed twice",
            ),
            ("seconds = 5\n".to_string(), "`seconds` must be"),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert("data/forage.toml", &text);
            let err = Content::load(&files).expect_err(why);
            assert!(err.message.contains(why), "{why}: {}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.remove("data/forage.toml");
        let content = Content::load(&files).expect("foraging is optional");
        assert!(content.forage().is_none());
    }
}
