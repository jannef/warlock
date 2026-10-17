//! Scenarios (`assets/scenarios/*.toml`): a named starting setup for a run.

use super::ContentError;
use serde::Deserialize;

/// A scenario: which level a run starts on, plus defaults.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioDef {
    /// File stem; filled in by the loader.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub start_level: String,
    /// Seed used when none is given on the command line.
    #[serde(default)]
    pub seed: u64,
}

impl ScenarioDef {
    pub(super) fn parse(file: &str, id: &str, text: &str) -> Result<Self, ContentError> {
        let mut def: ScenarioDef =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        def.id = id.to_string();
        Ok(def)
    }
}
