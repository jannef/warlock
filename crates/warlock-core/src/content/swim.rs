//! Swimming (`assets/data/swim.toml`, spec §8 "Skills": Swim; #1019): the numbers of a step in
//! deep water. The rule itself lives in `world/swim.rs`.
//!
//! The file is optional: without it nobody swims, and deep water refuses everyone.

use super::{Content, ContentError, DamageTypeId};
use serde::Deserialize;

/// The rules of swimming that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwimRules {
    /// The check's difficulty for an Unburdened swimmer.
    pub difficulty: i32,
    /// What each level of burden above Unburdened adds to it.
    pub per_burden: i32,
    /// The hit points a failed step costs, never the last.
    pub cold: i32,
    /// Their damage type.
    pub damage_type: DamageTypeId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SwimFile {
    difficulty: i32,
    per_burden: i32,
    cold: i32,
    damage_type: String,
}

impl SwimRules {
    /// Parses `data/swim.toml`: `cold` must not be negative (harm would turn into healing), and
    /// the damage type must exist.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SwimFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.cold < 0 {
            return Err(err("`cold` must not be negative".into()));
        }
        let damage_type = (content.damage_type_id(&parsed.damage_type))
            .ok_or_else(|| err(format!("unknown damage type `{}`", parsed.damage_type)))?;
        Ok(Self {
            difficulty: parsed.difficulty,
            per_burden: parsed.per_burden,
            cold: parsed.cold,
            damage_type,
        })
    }
}

impl super::RulesFile for SwimRules {
    const PATH: &'static str = "data/swim.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}
