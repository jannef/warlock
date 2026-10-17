//! Terrain definitions (`assets/data/terrain.toml`).

use super::anim::GlyphAnim;
use super::light::LightId;
use super::particles::ParticleEmitter;
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
    /// Time to step onto this terrain, in percent of the level's turn (spec §8 "Time").
    #[serde(default = "full_cost")]
    pub move_cost: u16,
    /// For a closed door: the terrain it becomes when opened.
    pub opens_to: Option<String>,
    /// For an open door: the terrain it becomes when closed.
    pub closes_to: Option<String>,
    pub stairs: Option<StairsDir>,
    /// Light preset the terrain emits (torch sconce, glowing fungus, rune). Must be transparent.
    pub light: Option<String>,
    /// Tint of the light the surface bounces (spec §5); white (neutral) by default.
    #[serde(default = "neutral")]
    pub bounce: Rgb,
    pub anim: Option<GlyphAnim>,
    #[serde(default)]
    pub glow: f32,
    #[serde(default)]
    pub haze: f32,
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
}

fn yes() -> bool {
    true
}

fn full_cost() -> u16 {
    100
}

fn neutral() -> Rgb {
    Rgb::WHITE
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
    /// Time to step onto this terrain, in percent of the level's turn: 100 is a normal step,
    /// 200 (shallow water) takes twice as long. Always at least 1.
    pub move_cost: u16,
    pub opens_to: Option<TerrainId>,
    pub closes_to: Option<TerrainId>,
    pub stairs: Option<StairsDir>,
    /// Light this terrain emits, if any.
    pub light: Option<LightId>,
    /// Gameplay: the tint of the light this surface bounces, per channel (white = neutral, the
    /// full [`crate::light::BOUNCE`] fraction).
    pub bounce: Rgb,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: strength of the heat haze rising above the cell, 0 to 1.
    pub haze: f32,
    /// Presentation-only: glyph particle emitters.
    pub particles: Vec<ParticleEmitter>,
}

impl TerrainDef {
    pub fn is_closed_door(&self) -> bool {
        self.opens_to.is_some()
    }

    pub fn is_open_door(&self) -> bool {
        self.closes_to.is_some()
    }

    /// Running water: every cell of it has a [`super::Flow`] (presentation-only).
    pub fn flows(&self) -> bool {
        self.anim.as_ref().is_some_and(GlyphAnim::is_flow)
    }
}
