//! Light presets (`assets/data/lights.toml`, spec §5).

use crate::light::Light;
use serde::{Deserialize, Serialize};

/// Largest allowed light radius, in cells.
pub const MAX_LIGHT_RADIUS: i32 = 32;

/// Largest intensity variation ([`Drift::amount`]) and dip depth ([`Dips::depth`]): together they
/// keep a source's brightness between half and one and a half times its steady light.
pub const MAX_INTENSITY_VARIATION: f32 = 0.25;
/// Largest hue drift, in degrees.
pub const MAX_HUE_DRIFT: f32 = 30.0;

/// Index of a light preset in [`super::Content::light_presets`].
///
/// Ids are assigned in file order when content loads, like [`super::TerrainId`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LightId(pub u16);

/// How a light varies over time (spec §4.4): visual only, carried for the renderer. Gameplay
/// rules never read it. The default (every part absent) is a perfectly stable light: constant
/// hue and intensity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stability {
    /// Smooth variation of the intensity.
    pub intensity: Option<Drift>,
    /// Smooth drift of the hue around the light's color.
    pub hue: Option<HueDrift>,
    /// Occasional soft dips of the intensity.
    pub dips: Option<Dips>,
}

/// Smooth intensity variation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Drift {
    /// Relative variation: the intensity stays within ±`amount` (`0.1` = ±10 %), usually within
    /// about a third of that.
    pub amount: f32,
    /// How fast it varies (roughly changes per second).
    pub speed: f32,
}

/// Smooth hue drift: the light's hue turns back and forth within ±`degrees`, its luminance kept.
/// Positive turns red toward yellow (a flame burning hotter), negative toward deeper red.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HueDrift {
    pub degrees: f32,
    /// How fast it drifts (roughly changes per second).
    pub speed: f32,
}

/// Occasional soft dips: about once every `every` seconds, at an irregular moment, the intensity
/// eases down by up to `depth` and back over `length` seconds. With [`HueDrift`] a dip also turns
/// the hue toward its deep end, as a guttering flame burns redder.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dips {
    /// Largest relative drop (`0.15` = 15 % dimmer at the bottom of the deepest dip).
    pub depth: f32,
    /// Mean time between dips, in seconds.
    pub every: f32,
    /// Duration of one dip, in seconds; at most `every`.
    pub length: f32,
}

impl Stability {
    /// True if the light never varies.
    pub fn is_stable(&self) -> bool {
        self.intensity.is_none_or(|d| d.amount == 0.0)
            && self.hue.is_none_or(|h| h.degrees == 0.0)
            && self.dips.is_none_or(|d| d.depth == 0.0)
    }

    /// Checks the ranges, returning what is wrong.
    pub fn check(&self) -> Result<(), String> {
        let rate = |what: &str, speed: f32| {
            if speed.is_finite() && speed > 0.0 && speed <= 20.0 {
                Ok(())
            } else {
                Err(format!("{what} `speed` must be above 0 and at most 20"))
            }
        };
        let relative = |what: &str, value: f32| {
            if (0.0..=MAX_INTENSITY_VARIATION).contains(&value) {
                Ok(())
            } else {
                Err(format!("{what} must be 0 to {MAX_INTENSITY_VARIATION}"))
            }
        };
        if let Some(d) = self.intensity {
            relative("intensity `amount`", d.amount)?;
            rate("intensity", d.speed)?;
        }
        if let Some(h) = self.hue {
            if !(0.0..=MAX_HUE_DRIFT).contains(&h.degrees) {
                return Err(format!("hue `degrees` must be 0 to {MAX_HUE_DRIFT}"));
            }
            rate("hue", h.speed)?;
        }
        if let Some(d) = self.dips {
            relative("dips `depth`", d.depth)?;
            if !(d.length.is_finite() && d.length > 0.0 && d.every.is_finite())
                || d.every < d.length
            {
                return Err("dips need `length` above 0 and `every` at least `length`".into());
            }
        }
        Ok(())
    }
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
    /// The preset this light burns as with its hood lowered (spec §5 "Light sources": the
    /// player's lantern, hooded): an id in `data/lights.toml`, itself without a hood. `None`:
    /// it has no hood.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooded: Option<String>,
    /// Whether its own cell gets none of its light: a hooded lantern's hood keeps the light off
    /// whoever carries it, so it lights the cells around them but not them.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hollow: bool,
    /// Presentation-only: how the drawn light varies over time.
    #[serde(default)]
    pub stability: Stability,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stability_defaults_to_stable_and_checks_ranges() {
        assert!(Stability::default().is_stable());
        assert_eq!(Stability::default().check(), Ok(()));
        let fire = Stability {
            intensity: Some(Drift {
                amount: 0.1,
                speed: 1.0,
            }),
            hue: Some(HueDrift {
                degrees: 6.0,
                speed: 0.4,
            }),
            dips: Some(Dips {
                depth: 0.1,
                every: 8.0,
                length: 0.7,
            }),
        };
        assert!(!fire.is_stable());
        assert_eq!(fire.check(), Ok(()));
        let zero = Stability {
            intensity: Some(Drift {
                amount: 0.0,
                speed: 1.0,
            }),
            ..Stability::default()
        };
        assert!(zero.is_stable());

        let bad = [
            Stability {
                intensity: Some(Drift {
                    amount: 0.4,
                    speed: 1.0,
                }),
                ..fire
            },
            Stability {
                intensity: Some(Drift {
                    amount: 0.1,
                    speed: 0.0,
                }),
                ..fire
            },
            Stability {
                hue: Some(HueDrift {
                    degrees: 90.0,
                    speed: 1.0,
                }),
                ..fire
            },
            Stability {
                hue: Some(HueDrift {
                    degrees: 5.0,
                    speed: f32::NAN,
                }),
                ..fire
            },
            Stability {
                dips: Some(Dips {
                    depth: 0.1,
                    every: 0.5,
                    length: 1.0,
                }),
                ..fire
            },
            Stability {
                dips: Some(Dips {
                    depth: -0.1,
                    every: 5.0,
                    length: 1.0,
                }),
                ..fire
            },
        ];
        for stability in bad {
            assert!(stability.check().is_err(), "{stability:?}");
        }
    }
}
