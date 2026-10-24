//! Light stability (spec §4.4): how each light source's drawn light varies over time, from its
//! preset's [`Stability`]. Visual only; gameplay light stays steady.
//!
//! A source's [`Modulation`] at a moment is a pure function of its stable key, its preset, and
//! [`super::VisualTime`] (so `--freeze-time` captures are reproducible), built from three parts,
//! each optional:
//! - **intensity**: smooth noise ([`noise`]) within ±`amount`;
//! - **hue**: smooth noise, from an independent stream, turning the light's hue within
//!   ±`degrees` with its luminance kept ([`hue_tint`]);
//! - **dips** ([`dip`]): about once every `every` seconds, at a moment drawn from the key, the
//!   intensity eases down by up to `depth` and back over `length` seconds, and the hue turns
//!   toward its deep (red) end, as a guttering flame burns redder.
//!
//! A preset without any of them gets [`Modulation::STEADY`] at every moment, exactly.
//! [`super::lighting::VisualLight::composite`] tints each source's contributions by its hue and
//! weighs its intensity into the per-cell gain that shading applies.

use super::lighting::LUMA;
use bevy::math::{Vec3, Vec4};
use std::f32::consts::TAU;
use warlock_core::content::{Dips, LightPreset};

/// Salts that give the hue and the dips their own random streams per source.
const HUE: u64 = 0x2545_f491_4f6c_dd1d;
const DIPS: u64 = 0x9e6c_63d0_676a_9a99;
const DIP_DEPTH: u64 = 0x632b_e59b_d9b4_e019;

/// A light source's modulation at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Modulation {
    /// Intensity multiplier of what the source lights, applied after shading like the rest of
    /// the cell's gain.
    pub gain: f32,
    /// Per-channel multiplier of the source's light (red, green, blue) that turns its hue
    /// ([`hue_tint`]).
    pub tint: Vec3,
}

impl Modulation {
    /// No modulation: steady light.
    pub const STEADY: Modulation = Modulation {
        gain: 1.0,
        tint: Vec3::ONE,
    };

    /// Light `light` (red, green, blue, UV) in the modulated hue. UV has no hue and is kept.
    pub fn color(&self, light: Vec4) -> Vec4 {
        (light.truncate() * self.tint).extend(light.w)
    }
}

/// The modulation of the source with stable key `key` and preset `preset` at visual time `time`
/// (seconds). Deterministic: the same key, preset, and time give the same value.
pub fn modulation(key: u64, preset: &LightPreset, time: f32) -> Modulation {
    let stability = &preset.stability;
    if stability.is_stable() {
        return Modulation::STEADY;
    }
    let dip = stability.dips.map_or(0.0, |d| dip(key, &d, time));
    let mut gain = 1.0;
    if let Some(d) = stability.intensity {
        gain += d.amount * noise(key, time * d.speed);
    }
    if let Some(d) = stability.dips {
        gain -= d.depth * dip;
    }
    let tint = match stability.hue {
        Some(h) => {
            // A dip pulls the drift toward the deep end: all the way at its bottom.
            let drift = noise(key ^ HUE, time * h.speed) * (1.0 - dip) - dip;
            let [r, g, b, _]: [u16; 4] = preset.color.into();
            hue_tint(Vec3::new(r.into(), g.into(), b.into()), h.degrees * drift)
        }
        None => Vec3::ONE,
    };
    Modulation { gain, tint }
}

/// How deep into a dip the source is at `time`: 0 outside dips, rising smoothly to between 0.5
/// and 1 (drawn per dip) and back. Time is cut into slots of `every` seconds; each slot holds one
/// dip of `length` seconds at a moment drawn from the key and the slot, so dips never overlap and
/// come about once every `every` seconds, irregularly.
pub fn dip(key: u64, dips: &Dips, time: f32) -> f32 {
    let slot = (time / dips.every).floor();
    let index = slot as i64;
    let unit = |salt: u64| 0.5 * (lattice(key ^ salt, index) + 1.0);
    let start = (slot + unit(DIPS) * (1.0 - dips.length / dips.every)) * dips.every;
    let x = (time - start) / dips.length;
    if !(0.0..1.0).contains(&x) {
        return 0.0;
    }
    let strength = 0.5 + 0.5 * unit(DIP_DEPTH);
    strength * 0.5 * (1.0 - (TAU * x).cos())
}

/// Per-channel multipliers that turn the hue of light of color `color` (linear rgb) by `degrees`
/// of HSV hue (positive turns red toward yellow), keeping its luminance. Light proportional to
/// `color` (a source's contributions) times the tint is the turned light.
///
/// The turn scales the two weaker channels together against the strongest, as a flame's color
/// moves when it burns hotter or cooler: a warm light turns paler yellow one way, deeper and more
/// saturated orange the other (a plain HSV turn would go toward salmon pink). It stays within
/// the strongest channel's side of the hue circle: a channel the color lacks stays dark, and a
/// color whose weaker channels are equal (grey, pure red) has no hue to turn this way.
pub fn hue_tint(color: Vec3, degrees: f32) -> Vec3 {
    let c = color.to_array();
    let top = if c[0] >= c[1] && c[0] >= c[2] {
        0
    } else if c[1] >= c[2] {
        1
    } else {
        2
    };
    // Hue grows from the strongest channel toward the next one in red, green, blue order.
    let (next, prev) = ((top + 1) % 3, (top + 2) % 3);
    let (mid, low, toward) = if c[next] >= c[prev] {
        (c[next], c[prev], 1.0)
    } else {
        (c[prev], c[next], -1.0)
    };
    if mid - low <= 0.0 || c[top] <= 0.0 {
        return Vec3::ONE;
    }
    // Where the hue lies between the strongest channel's primary (0) and the secondary toward
    // the middle channel (1), in steps of 60°; scaling the weaker channels by `f` moves it to
    // f·(mid − low) / (top − f·low).
    let at = (mid - low) / (c[top] - low);
    let to = (at + toward * degrees / 60.0).clamp(0.0, 1.0);
    let f = to * c[top] / (mid - low + to * low);
    let mut tint = Vec3::splat(f);
    tint[top] = 1.0;
    let turned = (color * tint).dot(LUMA);
    if turned <= 0.0 {
        return Vec3::ONE;
    }
    tint * (color.dot(LUMA) / turned)
}

/// Smooth noise in `[-1, 1]`: two octaves of value noise, the second faster and weaker, so the
/// light drifts calmly with a little life in it.
pub fn noise(key: u64, x: f32) -> f32 {
    const OCTAVE: u64 = 0x5bd1_e995_9e37_79b9;
    0.7 * value_noise(key, x) + 0.3 * value_noise(key ^ OCTAVE, x * 2.3 + 0.37)
}

/// Value noise: random values in `[-1, 1]` at whole `x`, eased in between.
fn value_noise(key: u64, x: f32) -> f32 {
    let i = x.floor();
    let t = x - i;
    let ease = t * t * (3.0 - 2.0 * t);
    let (a, b) = (lattice(key, i as i64), lattice(key, i as i64 + 1));
    a + (b - a) * ease
}

/// A random value in `[-1, 1)` for `key` at lattice index `i`.
fn lattice(key: u64, i: i64) -> f32 {
    // SplitMix64 finalizer over the key and the lattice index.
    let mut h = key ^ (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h = (h ^ (h >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^= h >> 31;
    (h >> 40) as f32 / (1u64 << 23) as f32 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::content::{Drift, HueDrift, Stability};
    use warlock_core::light::Light;

    fn preset(color: [u16; 3], stability: Stability) -> LightPreset {
        LightPreset {
            id: "test".into(),
            color: Light::new(color[0], color[1], color[2], 0),
            radius: 6,
            hooded: None,
            hollow: false,
            stability,
        }
    }

    fn fire() -> LightPreset {
        preset(
            [300, 180, 90],
            Stability {
                intensity: Some(Drift {
                    amount: 0.12,
                    speed: 1.0,
                }),
                hue: Some(HueDrift {
                    degrees: 6.0,
                    speed: 0.4,
                }),
                dips: Some(Dips {
                    depth: 0.12,
                    every: 7.0,
                    length: 0.8,
                }),
            },
        )
    }

    /// Torch light as a source contributes it somewhere: its color, dimmed.
    fn torch_light() -> Vec4 {
        Vec4::new(300.0, 180.0, 90.0, 0.0) / 255.0 * 0.6
    }

    /// The hue of an rgb color, in degrees (HSV).
    fn hue_of(c: Vec3) -> f32 {
        let (max, min) = (c.max_element(), c.min_element());
        let d = max - min;
        let h = if max == c.x {
            (c.y - c.z) / d
        } else if max == c.y {
            2.0 + (c.z - c.x) / d
        } else {
            4.0 + (c.x - c.y) / d
        };
        (h * 60.0).rem_euclid(360.0)
    }

    #[test]
    fn modulation_is_deterministic_smooth_and_bounded() {
        let fire = fire();
        let torch = torch_light();
        let base_hue = hue_of(torch.truncate());
        let mut last = modulation(7, &fire, 0.0);
        let (mut gains, mut hues) = ((f32::MAX, f32::MIN), (f32::MAX, f32::MIN));
        for i in 0..60_000 {
            let t = i as f32 * 0.001;
            let m = modulation(7, &fire, t);
            assert_eq!(m, modulation(7, &fire, t), "same key and time, same value");
            assert!((0.76..=1.12).contains(&m.gain), "{} at {t}", m.gain);
            let color = m.color(torch).truncate();
            assert!(
                (color.dot(LUMA) - torch.truncate().dot(LUMA)).abs() < 1e-5,
                "the hue drift keeps luminance"
            );
            let hue = hue_of(color);
            assert!((hue - base_hue).abs() <= 6.0 + 1e-3, "{hue} at {t}");
            assert!((m.gain - last.gain).abs() < 0.01, "smooth at {t}");
            assert!(
                (hue - hue_of(last.color(torch).truncate())).abs() < 0.1,
                "smooth hue at {t}"
            );
            gains = (gains.0.min(m.gain), gains.1.max(m.gain));
            hues = (hues.0.min(hue), hues.1.max(hue));
            last = m;
        }
        assert!(gains.1 - gains.0 > 0.1, "the intensity varies: {gains:?}");
        assert!(hues.1 - hues.0 > 4.0, "the hue drifts: {hues:?}");
        let differs = (0..50).any(|i| {
            let t = i as f32 * 0.37;
            (modulation(7, &fire, t).gain - modulation(8, &fire, t).gain).abs() > 0.01
        });
        assert!(differs, "sources vary independently");
    }

    #[test]
    fn stable_presets_never_change() {
        let crystal = preset([340, 50, 40], Stability::default());
        let still = preset(
            [300, 180, 90],
            Stability {
                hue: Some(HueDrift {
                    degrees: 0.0,
                    speed: 1.0,
                }),
                ..Stability::default()
            },
        );
        for i in 0..1_000 {
            let t = i as f32 * 0.731;
            assert_eq!(modulation(i, &crystal, t), Modulation::STEADY);
            assert_eq!(modulation(i, &still, t), Modulation::STEADY);
        }
        let light = Vec4::new(1.3, 0.2, 0.15, 0.4);
        assert_eq!(Modulation::STEADY.color(light), light);
    }

    #[test]
    fn dips_are_rare_soft_and_bounded() {
        let fire = fire();
        let dips = fire.stability.dips.expect("dips");
        let step = 0.01;
        let (mut count, mut inside, mut last) = (0, 0, 0.0);
        let mut deepest: f32 = 0.0;
        for i in 0..70_000 {
            let d = dip(3, &dips, i as f32 * step);
            assert!((0.0..=1.0).contains(&d));
            assert!((d - last).abs() < 0.05, "soft: {last} -> {d}");
            if d > 0.0 && last == 0.0 {
                count += 1;
            }
            inside += usize::from(d > 0.0);
            deepest = deepest.max(d);
            last = d;
        }
        // 700 s: one dip per 7 s slot, each 0.8 s long.
        assert_eq!(count, 100);
        assert!((inside as f32 * step - 80.0).abs() < 1.0, "{inside}");
        assert!(deepest > 0.9);
        // A dip pulls the hue toward red.
        let t = (0..70_000)
            .map(|i| i as f32 * step)
            .max_by(|&a, &b| dip(9, &dips, a).total_cmp(&dip(9, &dips, b)))
            .expect("a dip");
        assert!(dip(9, &dips, t) > 0.9);
        let torch = torch_light();
        let deep = modulation(9, &fire, t).color(torch).truncate();
        assert!(hue_of(deep) < hue_of(torch.truncate()) - 4.5);
    }

    #[test]
    fn hue_tints_turn_red_toward_yellow_keeping_luminance() {
        let orange = Vec3::new(1.0, 0.5, 0.2);
        assert!(hue_tint(orange, 0.0).abs_diff_eq(Vec3::ONE, 1e-6));
        let saturation = |c: Vec3| 1.0 - c.min_element() / c.max_element();
        for degrees in [-8.0, -3.0, 3.0, 8.0] {
            // Any amount of light of that color turns the same way.
            for scale in [0.05, 1.0, 3.0] {
                let turned = orange * scale * hue_tint(orange, degrees);
                assert!((hue_of(turned) - hue_of(orange) - degrees).abs() < 1e-3);
                assert!((turned.dot(LUMA) - scale * orange.dot(LUMA)).abs() < 1e-5);
                // Deeper orange is more saturated, yellow paler, like a flame's color.
                let more = saturation(turned) > saturation(orange);
                assert_eq!(more, degrees < 0.0, "{degrees}");
            }
        }
        // Colors whose middle channel precedes the strongest turn the same way round.
        let pink = Vec3::new(1.0, 0.2, 0.5);
        let turned = pink * hue_tint(pink, 5.0);
        assert!((hue_of(turned) - (hue_of(pink) + 5.0)).abs() < 1e-3);
        let blue = Vec3::new(0.2, 0.4, 1.0);
        let turned = blue * hue_tint(blue, -5.0);
        assert!((hue_of(turned) - (hue_of(blue) - 5.0)).abs() < 1e-3);
        // Grey and pure primaries have no hue to turn, black no light; a missing channel stays
        // dark, and the turn stops at the edge of the strongest channel's side.
        assert_eq!(hue_tint(Vec3::splat(0.4), 10.0), Vec3::ONE);
        assert_eq!(hue_tint(Vec3::ZERO, 10.0), Vec3::ONE);
        assert_eq!(hue_tint(Vec3::new(1.0, 0.3, 0.3), 10.0), Vec3::ONE);
        let green = Vec3::new(0.0, 1.0, 0.5);
        assert_eq!((green * hue_tint(green, 10.0)).x, 0.0);
        let far = orange * hue_tint(orange, 90.0);
        assert!((hue_of(far) - 60.0).abs() < 1e-3, "{far}");
    }

    #[test]
    fn noise_stays_in_range_far_from_zero() {
        for i in 0..10_000 {
            let x = -5_000.0 + i as f32 * 1.37;
            assert!((-1.0..=1.0).contains(&noise(u64::MAX - i, x)));
        }
    }
}
