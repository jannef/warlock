//! Terrain definitions (`assets/data/terrain.toml`).

use super::action::{BASE_TURN, price};
use super::anim::GlyphAnim;
use super::level::LevelKind;
use super::light::LightId;
use super::overworld::Landscape;
use super::particles::ParticleEmitter;
use crate::color::Rgb;
use crate::time::Duration;
use serde::{Deserialize, Serialize};

/// Tallest terrain for sun shadows, in cells: it bounds how far a shadow can reach.
pub const MAX_HEIGHT: u8 = 4;

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
    /// What a step onto this terrain costs, in percent of a normal step (spec §8 "Time").
    #[serde(default = "full_cost")]
    pub move_cost: u16,
    /// Overworld terrain: minutes one overworld step onto it takes (spec §8 "Time").
    pub travel_minutes: Option<u16>,
    /// Overworld terrain: the ground of the local area entered from it (`>` on the cell); none
    /// means the cell has no local area.
    pub local: Option<String>,
    /// Overworld terrain: what its land is like in local areas (the generator of its own local
    /// areas, or what it adds to its neighbors').
    pub landscape: Option<Landscape>,
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
    /// How tall it stands for sun shadows, in cells (spec §5 "Sunlight"); default 1 for opaque
    /// terrain, 0 for transparent terrain.
    pub height: Option<u8>,
    /// Under a roof: never lit by the sun or the sky (indoor floors on outdoor levels).
    #[serde(default)]
    pub roofed: bool,
    /// Whether blood stains it (spec §8 "Blood"); water, lava, and fire do not hold it.
    #[serde(default = "yes")]
    pub stains: bool,
    /// How loud steps onto it are, in percent of the walker's footsteps (spec §5 "Hearing").
    #[serde(default = "full_noise")]
    pub step_noise: u16,
    /// For a door: how loud opening or closing it is; none: silent.
    #[serde(default)]
    pub toggle_noise: u16,
    /// Loudness a sound loses entering the cell; none: 0, or for solid terrain (neither walkable
    /// nor transparent) blocking sound.
    pub muffle: Option<u16>,
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

fn full_noise() -> u16 {
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
    /// What a step onto this terrain costs, in percent of a normal step (the `step` action's
    /// cost): 100 is a normal step, 200 (shallow water) takes twice as long. Always at least 1.
    /// Local levels only: on the overworld a step takes the terrain's `travel_minutes`.
    pub move_cost: u16,
    /// Overworld terrain: minutes one overworld step onto it takes (1 to a day). Every walkable
    /// terrain on the overworld has it.
    pub travel_minutes: Option<u16>,
    /// Overworld terrain: the ground of the local area entered from a cell of it; `None` if such
    /// cells have no local area (roads have one; bridges and locations do not).
    pub local: Option<TerrainId>,
    /// Overworld terrain: what its land is like in local areas. Every terrain with a `local`
    /// ground has a land or road landscape.
    pub landscape: Option<Landscape>,
    pub opens_to: Option<TerrainId>,
    pub closes_to: Option<TerrainId>,
    pub stairs: Option<StairsDir>,
    /// Light this terrain emits, if any.
    pub light: Option<LightId>,
    /// Gameplay: the tint of the light this surface bounces, per channel (white = neutral, the
    /// full [`crate::light::BOUNCE`] fraction).
    pub bounce: Rgb,
    /// Gameplay: how tall it stands, in cells, for sun shadows ([`crate::light::Sun`]); 0 casts
    /// none. At most [`MAX_HEIGHT`].
    pub height: u8,
    /// Gameplay: under a roof, so never lit by the sun or the sky.
    pub roofed: bool,
    /// Presentation-only: whether blood stains it (spec §8 "Blood"); water, lava, and fire do
    /// not hold it.
    pub stains: bool,
    /// Gameplay: how loud steps onto it are, in percent of the walker's footsteps
    /// ([`crate::noise::noise`]): water splashes (200), grass is soft (70).
    pub step_noise: u16,
    /// Gameplay: for a door, how loud opening it (a closed door) or closing it (an open one) is;
    /// 0 is silent.
    pub toggle_noise: u16,
    /// Gameplay: the loudness a sound loses entering the cell ([`crate::noise::propagate`]);
    /// `None` blocks sound (solid terrain without a `muffle`: walls, rock, trees). A closed door
    /// muffles.
    pub muffle: Option<u16>,
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
    /// The energy price of stepping onto this terrain on a level of `kind` for an actor whose
    /// step costs `step` (spec §8 "Time"; the rules ask through
    /// [`crate::world::LevelState::step_price`]): on the overworld the step's cost over the
    /// terrain's `travel_minutes`; elsewhere the step's cost scaled by its `move_cost` percent
    /// over the 6-second base ([`BASE_TURN`]). The actor's speed decides how long it takes.
    pub fn step_price(&self, step: u32, kind: LevelKind) -> i64 {
        match (kind, self.travel_minutes) {
            (LevelKind::Overworld, Some(minutes)) => {
                price(step, Duration::minutes(u64::from(minutes)))
            }
            _ => (price(step, BASE_TURN) * i64::from(self.move_cost) / 100).max(1),
        }
    }

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
