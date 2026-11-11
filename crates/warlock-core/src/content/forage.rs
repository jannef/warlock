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
    /// What a herbalist may find besides, each with its weight, in file order (#1343; none
    /// without a list).
    pub herbs: Vec<(ObjectId, u32)>,
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
    #[serde(default)]
    herbs: Vec<FindSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FindSpec {
    object: String,
    weight: u32,
}

impl ForageRules {
    /// Parses `data/forage.toml`: a time of at least a second, each ground once (a second
    /// would never be read), and finds that are known objects.
    pub(super) fn parse(
        file: &str,
        text: &str,
        content: &super::Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: ForageFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.seconds == 0 {
            return Err(err("`seconds` must be at least 1".into()));
        }
        let mut grounds: Vec<ForageGround> = Vec::new();
        for spec in parsed.ground {
            if grounds.iter().any(|g| g.landscape == spec.landscape) {
                let name = spec.landscape.id();
                return Err(err(format!("ground `{name}` is listed twice")));
            }
            let objects = |list: Vec<FindSpec>| -> Result<Vec<(ObjectId, u32)>, ContentError> {
                (list.into_iter())
                    .map(|find| {
                        (content.object_id(&find.object))
                            .map(|object| (object, find.weight))
                            .ok_or_else(|| err(format!("unknown object `{}`", find.object)))
                    })
                    .collect()
            };
            grounds.push(ForageGround {
                landscape: spec.landscape,
                chance: spec.chance,
                count: spec.count,
                finds: objects(spec.finds)?,
                herbs: objects(spec.herbs)?,
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
        // A herbalist's extra find (#1343): angelica root where it grows, none on the plains.
        let angelica = content.object_id("angelica_root").expect("angelica root");
        assert_eq!(forest.herbs, [(angelica, 1)]);
        assert!(
            rules
                .ground(Landscape::Plains)
                .expect("plains")
                .herbs
                .is_empty()
        );
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
            (
                row("forest", 10, "1d2", "nothing", 1),
                "unknown object `nothing`",
            ),
            (
                format!(
                    "{}{}",
                    row("forest", 10, "1d2", "apple", 1),
                    row("forest", 10, "1d2", "apple", 1).replace("seconds = 600\n", "")
                ),
                "listed twice",
            ),
            ("seconds = 0\n".to_string(), "`seconds` must be"),
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
