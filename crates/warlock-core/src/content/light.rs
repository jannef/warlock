//! Light presets (`assets/data/lights.toml`, spec §5).

use crate::light::Light;
use serde::{Deserialize, Serialize};

/// Largest allowed light radius, in cells.
pub const MAX_LIGHT_RADIUS: i32 = 32;

/// Index of a light preset in [`super::Content::light_presets`].
///
/// Ids are assigned in file order when content loads, like [`super::TerrainId`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LightId(pub u16);

/// Visual-only flicker, carried for the renderer. Gameplay rules never read it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flicker {
    /// Relative intensity variation, e.g. `0.15` for about ±15 %.
    pub amount: f32,
    /// How fast the light varies (roughly changes per second).
    pub speed: f32,
}

/// A kind of light: a torch, a lamp, glowing fungus, a UV rune.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightPreset {
    pub id: String,
    /// Color and intensity at the source, `[r, g, b, uv]`: 255 = fully lit, more = overbright.
    pub color: Light,
    /// Reach in cells; the light fades to zero at squared distance `radius² + radius`.
    pub radius: i32,
    /// Presentation-only.
    pub flicker: Option<Flicker>,
}
