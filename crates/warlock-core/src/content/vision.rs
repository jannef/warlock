//! Vision profiles (`assets/data/vision.toml`, spec §5): how the player's eyes respond to light.

use crate::fov::radius2;
use crate::light::Light;
use serde::{Deserialize, Serialize};

/// An eye: how much light it needs, whether it has darkvision, and whether it sees UV.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisionProfile {
    pub id: String,
    pub name: String,
    /// Minimum perceived brightness ([`Light::brightness`]) for a cell in line of sight to be
    /// visible.
    pub threshold: u16,
    /// Cells within this radius are visible however dark they are (0 = no darkvision).
    #[serde(default)]
    pub darkvision: i32,
    /// Whether ultraviolet light counts toward brightness (and is perceived at all).
    #[serde(default)]
    pub sees_uv: bool,
}

impl VisionProfile {
    /// The profile used when content defines none: it needs no light, so content without
    /// lighting (e.g. small test fixtures) stays fully visible in line of sight.
    pub(super) fn needs_no_light() -> Self {
        Self {
            id: "normal".into(),
            name: "Normal".into(),
            threshold: 0,
            darkvision: 0,
            sees_uv: false,
        }
    }

    /// The part of `light` this eye perceives: UV is dropped unless it sees UV.
    pub fn perceive(&self, light: Light) -> Light {
        if self.sees_uv {
            light
        } else {
            light.without_uv()
        }
    }

    /// Visibility of a cell in line of sight with perceived `light` at squared distance `dist2`
    /// from the eye: bright enough, or within darkvision range.
    pub fn sees(&self, light: Light, dist2: i32) -> bool {
        self.perceive(light).brightness() >= self.threshold || self.in_darkvision(dist2)
    }

    /// Whether squared distance `dist2` is within darkvision range.
    pub fn in_darkvision(&self, dist2: i32) -> bool {
        self.darkvision > 0 && dist2 <= radius2(self.darkvision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(threshold: u16, darkvision: i32, sees_uv: bool) -> VisionProfile {
        VisionProfile {
            id: "t".into(),
            name: "T".into(),
            threshold,
            darkvision,
            sees_uv,
        }
    }

    #[test]
    fn threshold_decides_in_the_light() {
        let normal = profile(40, 0, false);
        let low_light = profile(12, 0, false);
        let dim = Light::new(20, 20, 20, 0);
        assert!(!normal.sees(dim, 4));
        assert!(low_light.sees(dim, 4));
        assert!(normal.sees(Light::new(40, 40, 40, 0), 400));
    }

    #[test]
    fn darkvision_sees_nearby_darkness() {
        let dv = profile(40, 5, false);
        assert!(dv.sees(Light::DARK, 25));
        assert!(dv.sees(Light::DARK, 30), "radius² + radius");
        assert!(!dv.sees(Light::DARK, 31));
        assert!(!profile(40, 0, false).sees(Light::DARK, 0));
    }

    #[test]
    fn only_uv_eyes_perceive_uv() {
        let rune = Light::new(0, 0, 0, 100);
        assert!(!profile(40, 0, false).sees(rune, 1));
        assert!(profile(40, 0, true).sees(rune, 1));
        assert_eq!(profile(40, 0, false).perceive(rune), Light::DARK);
    }
}
