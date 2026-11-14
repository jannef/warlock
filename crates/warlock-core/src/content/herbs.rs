//! Gathering herbs (`assets/data/herbs.toml`, spec §8 "Skills": Herbalism; #1340): what the
//! glowing fungus gives, how long a gather takes, and how many herbs it finds.
//!
//! The rule itself lives in `world/gather.rs`. The file is optional: without it nobody gathers.

use super::{ContentError, ObjectId, TerrainId};
use crate::dice::Dice;
use crate::skills::SkillId;
use serde::Deserialize;

/// The rules of gathering that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HerbRules {
    /// The terrain herbs are gathered from (the glowing fungus).
    pub terrain: TerrainId,
    /// How long a gather takes, in seconds of game time.
    pub seconds: u32,
    /// How many herbs a trained gatherer finds, before Herbalism.
    pub count: Dice,
    /// Herbalism's value that gives one more.
    pub value_per_count: u16,
    /// The skill a gatherer must be trained in, whose value gives more (Herbalism; #1400).
    pub skill: SkillId,
    /// The herbs, in file order.
    pub herbs: Vec<Herb>,
}

/// One herb of the table: its weight at each item level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Herb {
    /// The object it is.
    pub object: ObjectId,
    /// Its weight at item level 1.
    pub find: u32,
    /// Added to the weight for each item level past the first (it may be negative).
    pub deeper: i32,
    /// The first item level it is found on.
    pub level: u8,
}

impl Herb {
    /// Its weight at item level `level`: 0 below its own level, else `find` plus `deeper` for
    /// each level past the first, never below 0.
    pub fn weight_at(&self, level: u8) -> u32 {
        let level = level.max(1);
        if level < self.level {
            return 0;
        }
        let weight = i64::from(self.find) + i64::from(self.deeper) * i64::from(level - 1);
        u32::try_from(weight.max(0)).unwrap_or(u32::MAX)
    }
}

impl HerbRules {
    /// The table at item level `level`: every herb with a weight above 0 there, in file order.
    pub fn at(&self, level: u8) -> Vec<(ObjectId, u32)> {
        (self.herbs.iter())
            .map(|h| (h.object, h.weight_at(level)))
            .filter(|&(_, w)| w > 0)
            .collect()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HerbsFile {
    terrain: String,
    seconds: u32,
    count: Dice,
    value_per_count: u16,
    skill: String,
    herb: Vec<HerbSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HerbSpec {
    object: String,
    find: u32,
    #[serde(default)]
    deeper: i32,
    #[serde(default = "first_level")]
    level: u8,
}

fn first_level() -> u8 {
    1
}

impl HerbRules {
    /// Parses `data/herbs.toml`: a known terrain and skill, a time of at least a second, a
    /// `value_per_count` above 0 (a divisor), and herbs that are known stackable objects, each
    /// once, with a weight above 0 and a `level` of at least 1.
    pub(super) fn parse(
        file: &str,
        text: &str,
        content: &super::Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: HerbsFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let terrain = (content.terrain_id(&parsed.terrain))
            .ok_or_else(|| err(format!("unknown terrain `{}`", parsed.terrain)))?;
        let skill = (content.skill_id(&parsed.skill))
            .ok_or_else(|| err(format!("unknown skill `{}`", parsed.skill)))?;
        if parsed.seconds == 0 {
            return Err(err("`seconds` must be at least 1".into()));
        }
        if parsed.value_per_count == 0 {
            return Err(err("`value_per_count` must be 1 or more".into()));
        }
        let mut herbs: Vec<Herb> = Vec::new();
        for spec in parsed.herb {
            let object = (content.object_id(&spec.object))
                .ok_or_else(|| err(format!("unknown object `{}`", spec.object)))?;
            if !content.object(object).stackable {
                return Err(err(format!("herb `{}` must be stackable", spec.object)));
            }
            if herbs.iter().any(|h| h.object == object) {
                return Err(err(format!("herb `{}` is listed twice", spec.object)));
            }
            if spec.find == 0 {
                return Err(err(format!(
                    "herb `{}`: `find` must be above 0",
                    spec.object
                )));
            }
            if spec.level == 0 {
                return Err(err(format!(
                    "herb `{}`: `level` must be 1 or more",
                    spec.object
                )));
            }
            herbs.push(Herb {
                object,
                find: spec.find,
                deeper: spec.deeper,
                level: spec.level,
            });
        }
        if herbs.is_empty() {
            return Err(err("there must be at least one `[[herb]]`".into()));
        }
        Ok(HerbRules {
            terrain,
            seconds: parsed.seconds,
            count: parsed.count,
            value_per_count: parsed.value_per_count,
            skill,
            herbs,
        })
    }
}

impl super::RulesFile for HerbRules {
    const PATH: &'static str = "data/herbs.toml";

    fn parse(path: &str, text: &str, content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    /// The share of `id` in the table at `level`, in percent.
    fn percent(content: &Content, id: &str, level: u8) -> f64 {
        let rules = content.herbs().expect("herbs.toml");
        let object = content.object_id(id).expect(id);
        let table = rules.at(level);
        let total: u32 = table.iter().map(|(_, w)| w).sum();
        let mine = table.iter().find(|(o, _)| *o == object).map_or(0, |e| e.1);
        f64::from(mine) * 100.0 / f64::from(total)
    }

    #[test]
    fn deeper_fungus_gives_rarer_herbs() {
        let content = Content::builtin().expect("content");
        // Mandrake: none above level 6, about one draw in ten at item level 8 (Fyrdelf 8's
        // caves: 16 of 207, the design's weights).
        for level in 1..6 {
            assert_eq!(percent(&content, "mandrake", level), 0.0, "level {level}");
        }
        let fyrdelf = percent(&content, "mandrake", 8);
        assert!((7.0..=11.0).contains(&fyrdelf), "{fyrdelf}");
        // Cealdhlāw 1: glowcap, moonwort, and comfrey only.
        let rules = content.herbs().expect("herbs.toml");
        assert_eq!(rules.at(1).len(), 3);
        // Moonwort thins with depth but never goes below nothing.
        assert!(percent(&content, "moonwort", 10) < percent(&content, "moonwort", 1));
        assert!(percent(&content, "moonwort", 60) >= 0.0);
        // Every herb is a known stackable material.
        for herb in &rules.herbs {
            assert!(content.object(herb.object).stackable);
        }
        assert_eq!(rules.herbs.len(), 9);
    }

    #[test]
    fn bad_files_are_refused_and_the_file_is_optional() {
        let text = include_str!("../../../../assets/data/herbs.toml");
        for (from, to, says) in [
            ("seconds = 300", "seconds = 0", "`seconds`"),
            (
                "skill = \"herbalism\"",
                "skill = \"nothing\"",
                "unknown skill",
            ),
            (
                "terrain = \"fungus\"",
                "terrain = \"moss\"",
                "unknown terrain",
            ),
            (
                "object = \"glowcap\"",
                "object = \"nothing\"",
                "unknown object",
            ),
            (
                "object = \"glowcap\"",
                "object = \"long_sword\"",
                "stackable",
            ),
            ("object = \"comfrey\"", "object = \"glowcap\"", "twice"),
            ("find = 40", "find = 0", "`find`"),
            ("level = 6", "level = 0", "`level`"),
            (
                "value_per_count = 50",
                "value_per_count = 0",
                "`value_per_count`",
            ),
        ] {
            let bad = text.replace(from, to);
            assert_ne!(bad, text, "{from}");
            let mut files = ContentFiles::builtin();
            files.insert("data/herbs.toml", &bad);
            let error = Content::load(&files).expect_err(from).to_string();
            assert!(error.contains(says), "{from}: {error}");
        }
    }
}
