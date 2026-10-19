//! Spell effects (spec §4.8 "Spell effects"): the timeline, strength, darkening, screen shake, and
//! shapes of a fireball, a lightning bolt, and later spells, as pure functions of an effect's age
//! (visual time since it was perceived) and a seed, like particles. They never depend on frame
//! rate, and frozen-time captures are reproducible. `ui::spells` draws perceived spell events
//! with them, from their data (`data/effects.toml`).
//!
//! An effect has three phases ([`Timeline`]): **travel** (a burst's projectile flies from its
//! origin to its center, a bolt draws out along its path), **impact** (full strength: a burst
//! grows to its full size and burns, a bolt flashes), and **linger** (a burst cools from its edge
//! in, a bolt's afterglow dies down).

use super::particles::Rng;
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::content::{EffectDef, EffectShape, Shake};

/// How strong a burst's projectile (and its light) is while it flies, relative to the impact.
pub const TRAVEL_STRENGTH: f32 = 0.35;
/// Longest time the impact takes to rise to its peak, in seconds: a burst flares up at once.
const RISE: f32 = 0.04;
/// A bolt between two flashes: the share of its brightness left before the next one.
const BETWEEN_FLASHES: f32 = 0.2;
/// Each flash of a bolt is this much dimmer than the one before.
const FLASH_FADE: f32 = 0.15;
/// The darkening's shape, as shares of its duration: easing in until [`DARKEN_IN`], holding until
/// [`DARKEN_HOLD`], then easing back.
const DARKEN_IN: f32 = 0.12;
const DARKEN_HOLD: f32 = 0.4;
/// How often the screen shake kicks in a new direction, per second.
const SHAKE_HZ: f32 = 28.0;
/// How much cooler a burst's edge is than its core (heat 1 at the core).
const BURST_SPREAD: f32 = 0.75;
/// How much a burst cools over its linger: past 1, so even the core burns out.
const BURST_COOLING: f32 = 1.2;
/// How much a burst cell's heat flickers, ±.
const BURST_FLICKER: f32 = 0.1;
/// How far sideways a bolt's corners are pushed, at most, in cell widths: its jagged look.
pub const JAG: f32 = 0.8;
/// A bolt's slow wander: over how many corners it turns, and its share of the push (the rest is
/// a sharp jitter at every corner).
const WANDER: f32 = 5.0;
const WANDER_SHARE: f32 = 0.6;
/// With the `motion` effect off: how long an effect shows as a still flash of its cells.
pub const STILL_SECONDS: f32 = 0.3;

/// Smooth step from 0 to 1 over `x` in `[0, 1]`.
pub fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Ease-out cubic over `[0, 1]`: a fast start and a gentle stop.
fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x.clamp(0.0, 1.0)).powi(3)
}

/// Where an effect is at some age.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Not started yet (a later effect of the same turn).
    Before,
    /// The projectile flies or the bolt draws out: how far, 0 to 1.
    Travel(f32),
    /// At full strength: seconds since the impact began.
    Impact(f32),
    /// Fading: how far, 0 to 1.
    Linger(f32),
    /// Over.
    Done,
}

/// The phases of one effect, in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timeline {
    pub travel: f32,
    pub impact: f32,
    pub linger: f32,
}

impl Timeline {
    /// The phases of effect `def` that travels `distance` cells first.
    pub fn new(def: &EffectDef, distance: f32) -> Self {
        Self {
            travel: distance.max(0.0) / def.speed,
            impact: def.impact,
            linger: def.linger,
        }
    }

    /// A still flash of [`STILL_SECONDS`] (the `motion` effect off).
    pub fn still() -> Self {
        Self {
            travel: 0.0,
            impact: STILL_SECONDS,
            linger: 0.0,
        }
    }

    pub fn impact_start(&self) -> f32 {
        self.travel
    }

    pub fn linger_start(&self) -> f32 {
        self.travel + self.impact
    }

    pub fn end(&self) -> f32 {
        self.linger_start() + self.linger
    }

    /// The phase `age` seconds after the start.
    pub fn phase(&self, age: f32) -> Phase {
        if age < 0.0 {
            Phase::Before
        } else if age < self.travel {
            Phase::Travel(age / self.travel)
        } else if age < self.linger_start() {
            Phase::Impact(age - self.travel)
        } else if age < self.end() {
            Phase::Linger((age - self.linger_start()) / self.linger)
        } else {
            Phase::Done
        }
    }
}

/// How strong effect `def` is at `phase`, 0 to 1: its light, and a bolt's brightness. A burst
/// flies at [`TRAVEL_STRENGTH`], flares up to its peak at the impact and burns, then dies down
/// over its linger; a bolt is at full strength as it draws out, flashes `flashes` times (each a
/// little dimmer, dropping to [`BETWEEN_FLASHES`] between them), and its afterglow dies down.
pub fn strength(def: &EffectDef, phase: Phase) -> f32 {
    let after_impact = if def.flashes == 0 {
        1.0
    } else {
        BETWEEN_FLASHES
    };
    match phase {
        Phase::Before | Phase::Done => 0.0,
        Phase::Travel(_) => match def.shape {
            EffectShape::Burst => TRAVEL_STRENGTH,
            EffectShape::Bolt => 1.0,
        },
        Phase::Impact(t) if def.flashes == 0 => smoothstep(t / RISE.min(def.impact)),
        Phase::Impact(t) => flash(t, def.impact, def.flashes),
        Phase::Linger(x) => after_impact * (1.0 - x).powi(2),
    }
}

/// A bolt's brightness `t` seconds into an impact of `impact` seconds with `flashes` flashes:
/// each flash strikes at once and falls off quickly toward [`BETWEEN_FLASHES`].
fn flash(t: f32, impact: f32, flashes: u32) -> f32 {
    let n = flashes.max(1) as f32;
    let period = impact / n;
    let k = (t / period).floor().clamp(0.0, n - 1.0);
    let u = ((t - k * period) / period).clamp(0.0, 1.0);
    let peak = 1.0 - FLASH_FADE * k;
    BETWEEN_FLASHES + (peak - BETWEEN_FLASHES) * (1.0 - u).powi(2)
}

/// Which flash (or other glyph shape) of an effect shows `age` seconds after its start: a new
/// one every `flicker` seconds.
pub fn shape_slot(age: f32, flicker: f32) -> u64 {
    (age.max(0.0) / flicker.max(0.001)).floor() as u64
}

/// How dark the rest of the view is `t` seconds after an impact began, for a darkening of
/// `seconds`, 0 to 1 (times the effect's darken amount): it eases in quickly as the effect
/// flares, holds while it peaks, then eases back to nothing; none before or after.
pub fn darken_curve(t: f32, seconds: f32) -> f32 {
    let x = t / seconds.max(f32::EPSILON);
    if !(0.0..1.0).contains(&x) {
        0.0
    } else if x < DARKEN_IN {
        smoothstep(x / DARKEN_IN)
    } else if x < DARKEN_HOLD {
        1.0
    } else {
        1.0 - smoothstep((x - DARKEN_HOLD) / (1.0 - DARKEN_HOLD))
    }
}

/// How hard the view shakes, in logical pixels, for an effect landing `distance` cells from the
/// player: `pixels` on the player's cell, less the farther, none beyond `radius`.
pub fn shake_amplitude(shake: &Shake, distance: f32) -> f32 {
    let reach = shake.radius as f32 + 1.0;
    shake.pixels * (1.0 - distance / reach).clamp(0.0, 1.0)
}

/// The shake `t` seconds after an impact: an offset in pixels of at most `amplitude` on each
/// axis, kicking in a new direction [`SHAKE_HZ`] times a second (eased between kicks, never a
/// jump) and dying down over `seconds`; nothing before or after. Seeded by `key`.
pub fn shake_offset(t: f32, seconds: f32, amplitude: f32, key: u64) -> Vec2 {
    let x = t / seconds.max(f32::EPSILON);
    if !(0.0..1.0).contains(&x) || amplitude <= 0.0 {
        return Vec2::ZERO;
    }
    let n = t * SHAKE_HZ;
    let i = n.floor() as u64;
    let kick = |i: u64| {
        let mut rng = Rng::new(key, i);
        Vec2::new(rng.signed(), rng.signed())
    };
    let at = kick(i).lerp(kick(i + 1), smoothstep(n - n.floor()));
    at * amplitude * (1.0 - x).powi(2)
}

/// How far a burst's cell `(dx, dy)` from its center is as it looks on screen, in cells up and
/// down: cells are `aspect` times taller than wide, so a cell to the side is nearer than one
/// above, and the ball's heat looks round.
pub fn burst_distance(dx: i32, dy: i32, aspect: f32) -> f32 {
    let aspect = aspect.max(f32::EPSILON);
    Vec2::new(dx as f32, dy as f32 * aspect).length() / aspect
}

/// The heat of a burst's cell `d` cells from its center (of `radius`) at `phase`, its flicker
/// `noise` in `[-1, 1]`: 1 is white-hot (the core), toward 0 at the edge and as it cools. The
/// ball grows from the center to its edge over the first `grow` seconds of the impact, then cools
/// from the edge in over its linger. `None` where it has not reached yet or has burned out.
pub fn burst_heat(d: f32, radius: i32, grow: f32, phase: Phase, noise: f32) -> Option<f32> {
    let edge = radius.max(0) as f32 + 0.5;
    let (grown, cooled) = match phase {
        Phase::Impact(t) if grow > 0.0 => (ease_out(t / grow), 0.0),
        Phase::Impact(_) => (1.0, 0.0),
        Phase::Linger(x) => (1.0, x * BURST_COOLING),
        _ => return None,
    };
    let front = 0.5 + (edge - 0.5) * grown;
    if d > front {
        return None;
    }
    let heat = 1.0 - BURST_SPREAD * d / edge - cooled + BURST_FLICKER * noise;
    (heat > 0.0).then_some(heat.min(1.0))
}

/// The still heat of a burst's cell `d` cells from its center (the `motion` effect off).
pub fn still_heat(d: f32, radius: i32) -> f32 {
    let edge = radius.max(0) as f32 + 0.5;
    (1.0 - BURST_SPREAD * d / edge).clamp(0.0, 1.0)
}

/// The glyph (an index into the effect's glyphs, hottest first) for `heat` among `count`.
pub fn heat_glyph(heat: f32, count: usize) -> usize {
    let i = ((1.0 - heat.clamp(0.0, 1.0)) * count as f32) as usize;
    i.min(count.saturating_sub(1))
}

/// One stroke of a bolt: a line glyph drawn from `from` to `to` (level cells, fractional; a
/// cell's center is its position plus one half), turned along it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub from: Vec2,
    pub to: Vec2,
    /// The path cell it belongs to (an index into the path): it shows when that cell does, and
    /// once the bolt has drawn out that far.
    pub index: usize,
    /// A fork off the line: drawn dimmer and thinner.
    pub fork: bool,
}

/// The strokes of a bolt from `origin` along `path` in its shape `slot` (each flash and flicker
/// a new shape), seeded by `key`: a jagged line from the origin's center through the centers of
/// the path's cells, a stroke about one cell wide each, every corner pushed sideways by up to
/// `jag` cell widths (a slow wander plus a sharp jitter, easing to nothing at both ends); and,
/// with chance `branches` at each corner, a fork of one to three strokes leaving the line at an
/// angle, as far as `open` allows (cells the player sees, not walls).
/// Cells are `aspect` times taller than wide; the strokes are the same at any display scale.
pub fn bolt_strokes(
    origin: Pos,
    path: &[Pos],
    aspect: f32,
    jag: f32,
    branches: f32,
    key: u64,
    slot: u64,
    open: impl Fn(Pos) -> bool,
) -> Vec<Stroke> {
    // Work in cell widths, so the line is as jagged across as up and down on screen.
    let screen = Vec2::new(1.0, aspect.max(f32::EPSILON));
    let center = |p: Pos| (Vec2::new(p.x as f32, p.y as f32) + 0.5) * screen;
    let to_level = |v: Vec2| v / screen;
    let base: Vec<Vec2> = std::iter::once(origin)
        .chain(path.iter().copied())
        .map(center)
        .collect();
    let (Some(&first), Some(&last)) = (base.first(), base.last()) else {
        return Vec::new();
    };
    let way = (last - first).normalize_or_zero();
    if path.is_empty() || way == Vec2::ZERO {
        return Vec::new();
    }
    let side = way.perp();
    let seed = key ^ slot.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    // Corners along the base line, about one cell width apart: where each is and which path
    // cell it belongs to.
    let mut corners: Vec<(Vec2, usize)> = vec![(first, 0)];
    for (i, pair) in base.windows(2).enumerate() {
        let (a, b) = (pair[0], pair[1]);
        let n = (b - a).length().round().max(1.0) as usize;
        for k in 1..=n {
            corners.push((a.lerp(b, k as f32 / n as f32), i));
        }
    }
    // Each pushed sideways: a slow wander (smooth noise over WANDER corners) and a sharp jitter,
    // easing to nothing at both ends so the bolt leaves and strikes where it should.
    let knot = |n: usize| Rng::new(seed, (1 << 32) | n as u64).signed();
    let last_corner = corners.len() - 1;
    for (j, (at, _)) in corners.iter_mut().enumerate() {
        let x = j as f32 / WANDER;
        let n = x.floor() as usize;
        let wander = knot(n) + (knot(n + 1) - knot(n)) * smoothstep(x - x.floor());
        let jitter = Rng::new(seed, j as u64).signed();
        let taper = (j.min(last_corner - j) as f32 / 2.0).min(1.0);
        *at += side * jag * taper * (WANDER_SHARE * wander + (1.0 - WANDER_SHARE) * jitter);
    }
    let mut rng = Rng::new(seed, 1 << 33);
    let mut strokes: Vec<Stroke> = (corners.windows(2))
        .map(|pair| Stroke {
            from: to_level(pair[0].0),
            to: to_level(pair[1].0),
            index: pair[1].1,
            fork: false,
        })
        .collect();
    // Forks from the corners between the ends.
    for &(at, index) in corners.iter().skip(1).take(corners.len().saturating_sub(2)) {
        if rng.unit() >= branches {
            continue;
        }
        let turn = (0.45 + 0.5 * rng.unit()) * if rng.unit() < 0.5 { 1.0 } else { -1.0 };
        let heading = Vec2::from_angle(turn).rotate(way);
        let mut from = at;
        for _ in 0..1 + (rng.unit() * 3.0) as usize {
            let to = from + heading + heading.perp() * (jag * rng.signed());
            let level = to_level(to);
            if !open(Pos::new(level.x.floor() as i32, level.y.floor() as i32)) {
                break;
            }
            strokes.push(Stroke {
                from: to_level(from),
                to: level,
                index,
                fork: true,
            });
            from = to;
        }
    }
    strokes
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::content::Content;

    fn builtin(id: &str) -> EffectDef {
        let content = Content::builtin().expect("builtin content");
        content.effect(id).expect("builtin effect").clone()
    }

    #[test]
    fn phases_follow_each_other() {
        let fireball = builtin("fireball");
        let t = Timeline::new(&fireball, 6.0);
        assert!((t.travel - 6.0 / fireball.speed).abs() < 1e-6);
        assert_eq!(t.phase(-0.1), Phase::Before);
        assert_eq!(t.phase(0.0), Phase::Travel(0.0));
        assert_eq!(t.phase(t.travel), Phase::Impact(0.0));
        let mid = t.linger_start() + t.linger * 0.5;
        assert!(matches!(t.phase(mid), Phase::Linger(x) if (x - 0.5).abs() < 1e-4));
        assert_eq!(t.phase(t.end()), Phase::Done);
        // Where it starts there is nothing to travel: the impact comes at once.
        let here = Timeline::new(&fireball, 0.0);
        assert_eq!(here.phase(0.0), Phase::Impact(0.0));
        let still = Timeline::still();
        assert_eq!(still.phase(0.1), Phase::Impact(0.1));
        assert_eq!(still.phase(STILL_SECONDS), Phase::Done);
    }

    #[test]
    fn a_burst_flares_burns_and_dies_down() {
        let fireball = builtin("fireball");
        let at = |p| strength(&fireball, p);
        assert_eq!(at(Phase::Before), 0.0);
        assert_eq!(at(Phase::Travel(0.5)), TRAVEL_STRENGTH);
        assert_eq!(at(Phase::Impact(0.0)), 0.0);
        assert!(at(Phase::Impact(RISE * 0.5)) > 0.3);
        assert_eq!(at(Phase::Impact(RISE)), 1.0);
        assert_eq!(at(Phase::Impact(fireball.impact * 0.99)), 1.0);
        assert_eq!(at(Phase::Linger(0.0)), 1.0);
        assert!(at(Phase::Linger(0.5)) < 0.3);
        assert!(at(Phase::Linger(0.999)) < 1e-4);
        assert_eq!(at(Phase::Done), 0.0);
    }

    #[test]
    fn a_bolt_flashes_its_flashes_each_dimmer() {
        let bolt = builtin("lightning_bolt");
        let n = bolt.flashes as usize;
        let period = bolt.impact / n as f32;
        let at = |t| strength(&bolt, Phase::Impact(t));
        let peaks: Vec<f32> = (0..n).map(|k| at(k as f32 * period)).collect();
        assert_eq!(peaks[0], 1.0);
        assert!(
            peaks.windows(2).all(|w| w[1] < w[0] && w[1] > 0.5),
            "{peaks:?}"
        );
        // Between flashes it drops, never to nothing, and flares again.
        for k in 0..n {
            let low = at((k as f32 + 0.95) * period);
            assert!((BETWEEN_FLASHES..0.35).contains(&low), "{low}");
        }
        let afterglow = strength(&bolt, Phase::Linger(0.0));
        assert!((afterglow - BETWEEN_FLASHES).abs() < 1e-6);
        assert_eq!(strength(&bolt, Phase::Travel(0.3)), 1.0);
    }

    #[test]
    fn darkening_eases_in_holds_and_eases_back() {
        let s = 0.4;
        assert_eq!(darken_curve(-0.01, s), 0.0);
        assert_eq!(darken_curve(0.0, s), 0.0);
        assert!(darken_curve(s * DARKEN_IN * 0.5, s) > 0.3);
        assert_eq!(darken_curve(s * 0.3, s), 1.0);
        let mut last = 1.0;
        for i in 1..=20 {
            let t = s * (DARKEN_HOLD + (1.0 - DARKEN_HOLD) * i as f32 / 20.0);
            let d = darken_curve(t, s);
            assert!(d <= last, "eases back monotonically");
            last = d;
        }
        assert_eq!(darken_curve(s, s), 0.0);
        // Never a jump: small steps in time make small steps in darkness.
        for i in 0..400 {
            let t = i as f32 * 0.001;
            assert!((darken_curve(t + 0.001, s) - darken_curve(t, s)).abs() < 0.05);
        }
    }

    #[test]
    fn shake_decays_within_its_bounds_and_scales_with_distance() {
        let shake = Shake {
            pixels: 4.0,
            radius: 6,
            seconds: 0.3,
        };
        assert_eq!(shake_amplitude(&shake, 0.0), 4.0);
        assert!(shake_amplitude(&shake, 3.0) < 4.0 && shake_amplitude(&shake, 3.0) > 1.0);
        assert!(shake_amplitude(&shake, 6.0) > 0.0);
        assert_eq!(shake_amplitude(&shake, 7.0), 0.0);
        assert_eq!(shake_amplitude(&shake, 20.0), 0.0);
        let a = 4.0;
        let mut early = 0.0f32;
        let mut late = 0.0f32;
        for i in 0..300 {
            let t = i as f32 * 0.001;
            let o = shake_offset(t, shake.seconds, a, 7);
            assert!(o.x.abs() <= a && o.y.abs() <= a, "{o} at {t}");
            if t < 0.05 {
                early = early.max(o.abs().max_element());
            } else if t > 0.25 {
                late = late.max(o.abs().max_element());
            }
            // Smooth: never jumps between frames.
            let next = shake_offset(t + 0.001, shake.seconds, a, 7);
            assert!((next - o).length() < 1.0, "{o} → {next}");
        }
        assert!(early > 1.0 && late < 0.2, "{early} {late}");
        assert_eq!(shake_offset(-0.01, 0.3, a, 7), Vec2::ZERO);
        assert_eq!(shake_offset(0.3, 0.3, a, 7), Vec2::ZERO);
        assert_eq!(shake_offset(0.1, 0.3, 0.0, 7), Vec2::ZERO);
        assert_eq!(shake_offset(0.1, 0.3, a, 7), shake_offset(0.1, 0.3, a, 7));
        assert_ne!(shake_offset(0.1, 0.3, a, 7), shake_offset(0.1, 0.3, a, 8));
    }

    #[test]
    fn a_burst_grows_from_its_core_and_cools_from_its_edge() {
        let (radius, grow) = (2, 0.1);
        let heat = |d: f32, p| burst_heat(d, radius, grow, p, 0.0);
        // At first only the core; at full size the edge too, cooler than the core.
        assert!(heat(0.0, Phase::Impact(0.0)).is_some());
        assert_eq!(heat(2.0, Phase::Impact(0.0)), None);
        let core = heat(0.0, Phase::Impact(grow)).expect("core");
        let edge = heat(2.0, Phase::Impact(grow)).expect("edge");
        assert_eq!(core, 1.0);
        assert!(edge < 0.5 && edge > 0.0);
        assert_eq!(heat(2.8, Phase::Impact(grow)), None, "beyond the radius");
        // Cooling: the edge burns out first, then the core.
        assert_eq!(heat(2.0, Phase::Linger(0.4)), None);
        assert!(heat(0.0, Phase::Linger(0.4)).expect("core still burns") < core);
        assert_eq!(heat(0.0, Phase::Linger(0.99)), None);
        assert_eq!(heat(0.0, Phase::Travel(0.5)), None);
        assert_eq!(heat_glyph(1.0, 4), 0, "hottest glyph at the core");
        assert_eq!(heat_glyph(0.01, 4), 3);
        assert_eq!(heat_glyph(0.5, 4), 2);
        assert!(still_heat(0.0, radius) > still_heat(2.0, radius));
        // On screen, with cells twice as tall as wide, a cell beside the core is as near as one
        // half a cell above it.
        assert_eq!(burst_distance(2, 0, 2.0), 1.0);
        assert_eq!(burst_distance(0, 2, 2.0), 2.0);
        assert_eq!(burst_distance(3, 4, 1.0), 5.0);
    }

    #[test]
    fn bolt_strokes_run_jagged_along_the_path_and_fork() {
        let path: Vec<Pos> = (1..=12).map(|x| Pos::new(x, 5)).collect();
        let origin = Pos::new(0, 5);
        let open = |_: Pos| true;
        let strokes = bolt_strokes(origin, &path, 2.0, JAG, 0.0, 1, 0, open);
        // A stroke a cell wide each, from the origin's center to the last cell's center.
        assert_eq!(strokes.len(), path.len());
        assert!(strokes.iter().all(|s| !s.fork));
        assert_eq!(strokes[0].from, Vec2::new(0.5, 5.5));
        assert_eq!(strokes.last().map(|s| s.to), Some(Vec2::new(12.5, 5.5)));
        for pair in strokes.windows(2) {
            assert_eq!(pair[0].to, pair[1].from, "one connected line");
        }
        // Jagged, but within its cells: corners pushed up to `JAG` cell widths, half a cell
        // height with cells twice as tall.
        let sideways: Vec<f32> = strokes.iter().map(|s| (s.to.y - 5.5).abs()).collect();
        assert!(sideways.iter().any(|&d| d > 0.02));
        assert!(sideways.iter().all(|&d| d <= JAG / 2.0 + 1e-5));
        assert_eq!(strokes[3].index, 3);
        // Another flash, another shape; the same flash, the same shape.
        assert_ne!(
            strokes,
            bolt_strokes(origin, &path, 2.0, JAG, 0.0, 1, 1, open)
        );
        assert_eq!(
            strokes,
            bolt_strokes(origin, &path, 2.0, JAG, 0.0, 1, 0, open)
        );
        // Still (no jag): straight through the cell centers.
        let still = bolt_strokes(origin, &path, 2.0, 0.0, 0.0, 1, 0, open);
        assert!(still.iter().all(|s| s.to.y == 5.5));
        // Going up and down, a stroke is still about a cell wide: two to a cell.
        let down: Vec<Pos> = (1..=4).map(|y| Pos::new(3, y)).collect();
        assert_eq!(
            bolt_strokes(Pos::new(3, 0), &down, 2.0, JAG, 0.0, 1, 0, open).len(),
            8
        );
        // Forks leave the line, only where it is open.
        let forked = bolt_strokes(origin, &path, 2.0, JAG, 1.0, 1, 0, open);
        assert!(forked.iter().any(|s| s.fork && (s.to.y - 5.5).abs() > 0.6));
        let walls = bolt_strokes(origin, &path, 2.0, JAG, 1.0, 1, 0, |p: Pos| p.y == 5);
        assert!(walls.iter().all(|s| !s.fork || s.to.y.floor() == 5.0));
        assert!(bolt_strokes(origin, &[], 2.0, JAG, 1.0, 1, 0, open).is_empty());
    }
}
