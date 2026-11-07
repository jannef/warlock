//! Fletching (`assets/data/fletch.toml`, spec §8 "Fletching"; #1021): how long a batch takes, its
//! check, its count, what is a knife, and what each launcher's ammunition is made as.
//!
//! The rule itself lives in `world/fletch.rs`. The file is optional: without it nobody fletches,
//! and Woodcraft is listed as a skill with no use.

use super::{ContentError, ObjectId, WeaponSkillId};
use crate::dice::Dice;
use serde::Deserialize;

/// The rules of fletching that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FletchRules {
    /// How long a batch takes, in seconds of game time.
    pub seconds: u32,
    /// What the check must reach for the whole count.
    pub difficulty: i32,
    /// How many, before Woodcraft.
    pub count: Dice,
    /// Woodcraft's value that gives one more.
    pub value_per_count: u16,
    /// How far below the difficulty still makes half, rounded up.
    pub half_within: i32,
    /// How far below the difficulty still makes one.
    pub one_within: i32,
    /// Woodcraft's value from which they come out plain.
    pub plain_from: u16,
    /// The weapon skill whose weapons are knives.
    pub knives: WeaponSkillId,
    /// What each launcher's ammunition is made as, in file order.
    pub makes: Vec<FletchMakes>,
}

/// What is made for a launcher of one kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FletchMakes {
    /// What the launcher shoots (`launches`).
    pub launches: String,
    /// Made under [`FletchRules::plain_from`].
    pub brittle: ObjectId,
    /// Made from [`FletchRules::plain_from`].
    pub plain: ObjectId,
}

impl FletchRules {
    /// What is made for a launcher that shoots `launches`.
    pub fn makes_for(&self, launches: &str) -> Option<&FletchMakes> {
        self.makes.iter().find(|m| m.launches == launches)
    }

    /// How many of `count` a check of margin `margin` makes: the whole count, half rounded up,
    /// one, or none.
    pub fn made(&self, margin: i32, count: i32) -> i32 {
        if margin >= 0 {
            count
        } else if margin >= -self.half_within {
            (count + 1) / 2
        } else if margin >= -self.one_within {
            1
        } else {
            0
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FletchFile {
    seconds: u32,
    difficulty: i32,
    count: Dice,
    value_per_count: u16,
    half_within: i32,
    one_within: i32,
    plain_from: u16,
    knives: String,
    makes: Vec<MakesSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MakesSpec {
    launches: String,
    brittle: String,
    plain: String,
}

impl FletchRules {
    /// Parses `data/fletch.toml`: a time of at least a second, a `value_per_count` above 0 (a
    /// divisor), a knife that is a weapon skill, and for each launcher kind, once, known objects.
    pub(super) fn parse(
        file: &str,
        text: &str,
        content: &super::Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: FletchFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.seconds == 0 {
            return Err(err("`seconds` must be at least 1".into()));
        }
        if parsed.value_per_count == 0 {
            return Err(err("`value_per_count` must be 1 or more".into()));
        }
        let knives = (content.weapon_skills().id(&parsed.knives))
            .ok_or_else(|| err(format!("unknown weapon skill `{}`", parsed.knives)))?;
        let mut makes: Vec<FletchMakes> = Vec::new();
        for spec in parsed.makes {
            if makes.iter().any(|m| m.launches == spec.launches) {
                return Err(err(format!("`{}` is listed twice", spec.launches)));
            }
            let object = |id: &str| -> Result<ObjectId, ContentError> {
                (content.object_id(id)).ok_or_else(|| err(format!("unknown object `{id}`")))
            };
            let brittle = object(&spec.brittle)?;
            let plain = object(&spec.plain)?;
            makes.push(FletchMakes {
                launches: spec.launches,
                brittle,
                plain,
            });
        }
        Ok(FletchRules {
            seconds: parsed.seconds,
            difficulty: parsed.difficulty,
            count: parsed.count,
            value_per_count: parsed.value_per_count,
            half_within: parsed.half_within,
            one_within: parsed.one_within,
            plain_from: parsed.plain_from,
            knives,
            makes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Content, ContentFiles};

    fn rules() -> FletchRules {
        Content::builtin()
            .expect("content")
            .fletch()
            .expect("fletch.toml")
            .clone()
    }

    #[test]
    fn the_margin_scales_the_count_by_its_bands() {
        let rules = rules();
        for (margin, made) in [(5, 5), (0, 5), (-1, 3), (-3, 3), (-4, 1), (-6, 1), (-7, 0)] {
            assert_eq!(rules.made(margin, 5), made, "margin {margin}");
        }
    }

    #[test]
    fn bad_files_are_refused() {
        let text = include_str!("../../../../assets/data/fletch.toml");
        for (from, to, says) in [
            ("seconds = 600", "seconds = 0", "`seconds`"),
            (
                "knives = \"daggers\"",
                "knives = \"cutlery\"",
                "weapon skill",
            ),
            ("plain = \"bolt\"", "plain = \"nothing\"", "unknown object"),
            (
                "value_per_count = ",
                "value_per_count = 0 #",
                "`value_per_count`",
            ),
        ] {
            let bad = text.replace(from, to);
            assert_ne!(bad, text, "{from}");
            let mut files = ContentFiles::builtin();
            files.insert("data/fletch.toml", &bad);
            let error = Content::load(&files).expect_err(from).to_string();
            assert!(error.contains(says), "{from}: {error}");
        }
    }
}
