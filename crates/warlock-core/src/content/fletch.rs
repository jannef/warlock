//! Fletching (`assets/data/fletch.toml`, spec §8 "Fletching"; #1021, #1336): what the arrows and
//! bolts recipes (`recipes/woodcraft/arrows.toml`, `bolts.toml`, `count = "fletch"`) read beyond
//! the recipe: the batch's count, what is a knife, and what each launcher is called.
//!
//! The rules live in `world/crafting/fletch.rs` and `world/crafting/craft_make.rs`. The file is optional: without
//! it no recipe has a knife or a launcher to hand, and `count = "fletch"` makes one.

use super::{ContentError, WeaponSkillId};
use crate::dice::Dice;
use serde::Deserialize;

/// The rules of fletching that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FletchRules {
    /// How many a batch makes, before Woodcraft.
    pub count: Dice,
    /// Woodcraft's value that gives one more.
    pub value_per_count: u16,
    /// The weapon skill whose weapons are knives.
    pub knives: WeaponSkillId,
    /// What shoots each kind of ammunition, in file order.
    pub makes: Vec<FletchMakes>,
}

/// What shoots ammunition of one kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FletchMakes {
    /// What the launcher shoots (its `launches`, the ammunition's `ammo`).
    pub launches: String,
    /// The launcher in a word, as the recipe's row says it: "bow".
    pub launcher: String,
}

impl FletchRules {
    /// What shoots `launches`.
    pub fn makes_for(&self, launches: &str) -> Option<&FletchMakes> {
        self.makes.iter().find(|m| m.launches == launches)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FletchFile {
    count: Dice,
    value_per_count: u16,
    knives: String,
    makes: Vec<MakesSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MakesSpec {
    launches: String,
    launcher: String,
}

impl FletchRules {
    /// Parses `data/fletch.toml`: a `value_per_count` above 0 (a divisor), a knife that is a
    /// weapon skill, and each launcher kind once, with a word.
    pub(super) fn parse(
        file: &str,
        text: &str,
        content: &super::Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: FletchFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
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
            if spec.launcher.is_empty() {
                return Err(err(format!("`{}`: an empty `launcher`", spec.launches)));
            }
            makes.push(FletchMakes {
                launches: spec.launches,
                launcher: spec.launcher,
            });
        }
        Ok(FletchRules {
            count: parsed.count,
            value_per_count: parsed.value_per_count,
            knives,
            makes,
        })
    }
}

impl super::RulesFile for FletchRules {
    const PATH: &'static str = "data/fletch.toml";

    fn parse(path: &str, text: &str, content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    #[test]
    fn bad_files_are_refused() {
        let text = include_str!("../../../../assets/data/fletch.toml");
        for (from, to, says) in [
            (
                "knives = \"daggers\"",
                "knives = \"cutlery\"",
                "weapon skill",
            ),
            ("launcher = \"bow\"", "launcher = \"\"", "empty `launcher`"),
            (
                "launches = \"bolt\"",
                "launches = \"arrow\"",
                "listed twice",
            ),
            (
                "value_per_count = ",
                "value_per_count = 0 #",
                "`value_per_count`",
            ),
            ("seconds", "seconds", "unknown field"),
        ] {
            let bad = match from {
                "seconds" => format!("seconds = 600\n{text}"),
                _ => text.replace(from, to),
            };
            assert_ne!(bad, text, "{from}");
            let mut files = ContentFiles::builtin();
            files.insert("data/fletch.toml", &bad);
            let error = Content::load(&files).expect_err(from).to_string();
            assert!(error.contains(says), "{from}: {error}");
        }
    }
}
