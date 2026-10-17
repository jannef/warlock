//! Terrain definitions (`assets/data/terrain.toml`).

use super::anim::GlyphAnim;
use super::light::LightId;
use crate::color::Rgb;
use serde::{Deserialize, Serialize};

/// Index of a terrain definition in [`super::Content::terrain`].
///
/// Ids are assigned in file order when content loads; saves must not assume they are stable
/// across content changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TerrainId(pub u16);

/// Which way a staircase leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StairsDir {
    Up,
    Down,
}

/// A terrain type as written in content files.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TerrainSpec {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub fg: Rgb,
    #[serde(default)]
    pub bg: Rgb,
    #[serde(default = "yes")]
    pub walkable: bool,
    #[serde(default = "yes")]
    pub transparent: bool,
    /// For a closed door: the terrain it becomes when opened.
    pub opens_to: Option<String>,
    /// For an open door: the terrain it becomes when closed.
    pub closes_to: Option<String>,
    pub stairs: Option<StairsDir>,
    /// Light preset the terrain emits (torch sconce, glowing fungus, rune). Must be transparent.
    pub light: Option<String>,
    pub anim: Option<GlyphAnim>,
}

fn yes() -> bool {
    true
}

/// A resolved terrain definition.
#[derive(Clone, Debug)]
pub struct TerrainDef {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub fg: Rgb,
    pub bg: Rgb,
    pub walkable: bool,
    pub transparent: bool,
    pub opens_to: Option<TerrainId>,
    pub closes_to: Option<TerrainId>,
    pub stairs: Option<StairsDir>,
    /// Light this terrain emits, if any.
    pub light: Option<LightId>,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
}

impl TerrainDef {
    pub fn is_closed_door(&self) -> bool {
        self.opens_to.is_some()
    }

    pub fn is_open_door(&self) -> bool {
        self.closes_to.is_some()
    }
}
