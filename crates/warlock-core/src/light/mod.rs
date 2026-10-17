//! Gameplay light (spec §5): integer RGB + UV values, falloff, and per-source shadow masks.
//!
//! **Units.** Every channel is a `u16`; 255 means "fully lit" (a surface shows its base color) and
//! larger values are overbright. A cell's light is the level's ambient light plus the sum of every
//! source that reaches it, saturating at `u16::MAX`.
//!
//! **Sources.** A source shadowcasts with [`crate::fov::compute_fov`] from its position within its
//! radius, so light never passes through opaque cells. Its contribution at squared distance `d²`
//! is `color · (R² − d²) / R²` with `R² = radius² + radius` (the rounded circle that line of sight
//! uses): the full color at the source, falling off monotonically to zero at the edge. All math is
//! integer — no floating point, no transcendental functions — so light is deterministic on every
//! platform.
//!
//! Masks are cached per source ([`LightCache`]); the perception phase combines them with the
//! player's line of sight, adds one weak bounce ([`bounce`]), and applies the wall-face rule (see
//! [`crate::perception`]). On levels under the open sky the sun and the sky add their light by
//! day ([`Sun`]).

mod bounce;
mod cache;
mod sun;

pub(crate) use bounce::{Area, Bounce};
pub use bounce::{BOUNCE, BOUNCE_STEPS};
#[cfg(test)]
pub(crate) use cache::Mask;
pub(crate) use cache::{LightCache, SUN_KEY};
pub(crate) use sun::Sunlight;
pub use sun::{MAX_SHADOW, NOON_ELEVATION, SHADOW_ONE, ShadowSteps, Sun};

use crate::fov::radius2;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

/// Integer light: red, green, blue, and ultraviolet. Written in content files as `[r, g, b, uv]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "[u16; 4]", into = "[u16; 4]")]
pub struct Light {
    /// Red, green, blue: 255 = fully lit, more = overbright.
    pub rgb: [u16; 3],
    /// Ultraviolet. Only eyes that see UV perceive it.
    pub uv: u16,
}

impl Light {
    /// No light at all.
    pub const DARK: Light = Light::new(0, 0, 0, 0);

    pub const fn new(r: u16, g: u16, b: u16, uv: u16) -> Self {
        Self { rgb: [r, g, b], uv }
    }

    /// Integer Rec. 709 luma of the RGB channels: `(54·r + 183·g + 19·b) / 256`, so white 255
    /// has luma 255.
    pub fn luma(self) -> u16 {
        let [r, g, b] = self.rgb.map(u32::from);
        let luma = (54 * r + 183 * g + 19 * b) / 256;
        u16::try_from(luma).unwrap_or(u16::MAX)
    }

    /// Brightness compared against an eye's threshold: luma plus UV. Apply
    /// [`crate::content::VisionProfile::perceive`] first so UV only counts for eyes that see it.
    pub fn brightness(self) -> u16 {
        self.luma().saturating_add(self.uv)
    }

    /// Channel-wise saturating sum.
    pub fn saturating_add(self, other: Light) -> Light {
        self.zip(other, u16::saturating_add)
    }

    /// Channel-wise maximum.
    pub fn max(self, other: Light) -> Light {
        self.zip(other, u16::max)
    }

    /// The same light without its UV channel.
    pub fn without_uv(self) -> Light {
        Light { uv: 0, ..self }
    }

    pub fn is_dark(self) -> bool {
        self == Light::DARK
    }

    fn zip(self, other: Light, f: impl Fn(u16, u16) -> u16) -> Light {
        let (a, b): ([u16; 4], [u16; 4]) = (self.into(), other.into());
        Light::from(std::array::from_fn::<u16, 4, _>(|i| f(a[i], b[i])))
    }
}

impl From<[u16; 4]> for Light {
    fn from([r, g, b, uv]: [u16; 4]) -> Self {
        Light::new(r, g, b, uv)
    }
}

impl From<Light> for [u16; 4] {
    fn from(l: Light) -> Self {
        [l.rgb[0], l.rgb[1], l.rgb[2], l.uv]
    }
}

/// Light a source of `color` and `radius` adds at squared distance `d2`:
/// `color · (R² − d²) / R²` with `R² = radius² + radius`, and zero at and beyond `R²`.
///
/// A radius of 0 lights only the source's own cell.
pub fn falloff(color: Light, radius: i32, d2: i32) -> Light {
    if d2 <= 0 {
        return color;
    }
    let r2 = radius2(radius);
    if d2 >= r2 {
        return Light::DARK;
    }
    let num = u64::from((r2 - d2).unsigned_abs());
    let den = u64::from(r2.unsigned_abs());
    let scale = |c: u16| u16::try_from(u64::from(c) * num / den).unwrap_or(u16::MAX);
    let [r, g, b, uv]: [u16; 4] = color.into();
    Light::new(scale(r), scale(g), scale(b), scale(uv))
}

/// The light one source adds to the transparent cells the viewer has in line of sight, as the eye
/// perceives it (`perceive`, e.g. dropping UV), from the cells it lights (a mask's, or the sun's).
/// Cells with no perceived light are left out; the order is kept.
pub(crate) fn lit_in_sight(
    cells: &[(Pos, Light)],
    los: &Grid<bool>,
    perceive: impl Fn(Light) -> Light,
) -> Vec<(Pos, Light)> {
    cells
        .iter()
        .map(|&(q, light)| (q, perceive(light)))
        .filter(|&(q, light)| !light.is_dark() && los.get(q) == Some(&true))
        .collect()
}

/// **Wall-face rule** for one source: an opaque cell `is_face` accepts is lit by the source as a
/// transparent cell at its own distance would be (`light_at`), but only if the viewer sees a face
/// the source lights: one of its (up to eight) neighbors is transparent, visible, and lit by the
/// source (`lit`, from [`lit_in_sight`]). So light never leaks through a wall to a viewer on its
/// dark side, and a wall is never brighter than the floor at the same distance. Sorted by
/// position.
pub(crate) fn wall_faces(
    lit: &[(Pos, Light)],
    light_at: impl Fn(Pos) -> Light,
    is_visible: impl Fn(Pos) -> bool,
    is_face: impl Fn(Pos) -> bool,
) -> Vec<(Pos, Light)> {
    let mut faces: Vec<Pos> = lit
        .iter()
        .filter(|(q, _)| is_visible(*q))
        .flat_map(|(q, _)| Dir::ALL.map(|d| q.step(d)))
        .filter(|w| is_face(*w))
        .collect();
    faces.sort();
    faces.dedup();
    faces
        .into_iter()
        .map(|w| (w, light_at(w)))
        .filter(|(_, light)| !light.is_dark())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn luma_of_white_is_255() {
        assert_eq!(Light::new(255, 255, 255, 0).luma(), 255);
        assert_eq!(Light::new(0, 0, 0, 90).luma(), 0);
        assert_eq!(Light::new(0, 0, 0, 90).brightness(), 90);
    }

    #[test]
    fn serializes_as_four_numbers() {
        let l = Light::new(1, 2, 3, 4);
        assert_eq!(serde_json::to_string(&l).unwrap(), "[1,2,3,4]");
        let back: Light = serde_json::from_str("[1,2,3,4]").unwrap();
        assert_eq!(back, l);
    }

    #[test]
    fn falloff_is_full_at_source_and_zero_at_edge() {
        let c = Light::new(255, 128, 0, 64);
        assert_eq!(falloff(c, 6, 0), c);
        assert_eq!(falloff(c, 6, 42), Light::DARK, "R² = 6² + 6");
        assert_eq!(falloff(c, 6, 41).rgb[0], 255 / 42);
        assert_eq!(falloff(c, 0, 0), c, "radius 0 lights its own cell");
        assert_eq!(falloff(c, 0, 1), Light::DARK);
    }

    #[test]
    fn saturating_channel_math() {
        let a = Light::new(u16::MAX, 10, 0, 5);
        let b = Light::new(1, 20, 0, 1);
        assert_eq!(a.saturating_add(b), Light::new(u16::MAX, 30, 0, 6));
        assert_eq!(a.max(b), Light::new(u16::MAX, 20, 0, 5));
    }

    proptest! {
        #[test]
        fn falloff_is_monotonic_and_bounded(
            r in 0u16..2000, g in 0u16..2000, b in 0u16..2000, uv in 0u16..2000,
            radius in 0i32..40, d2 in 0i32..2000,
        ) {
            let color = Light::new(r, g, b, uv);
            let here = falloff(color, radius, d2);
            let farther = falloff(color, radius, d2 + 1);
            prop_assert_eq!(here.max(farther), here, "light never grows with distance");
            prop_assert_eq!(here.max(color), color, "never brighter than the source");
            if d2 >= radius2(radius) && d2 > 0 {
                prop_assert!(here.is_dark(), "zero at and beyond the radius");
            }
        }
    }
}
