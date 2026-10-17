//! Glyph particles (spec §4.8): sparks, dust motes, splashes, drawn on the free-glyph layer.
//!
//! Particles are stateless: each one is a pure function of its emitter, a seed, and visual time.
//! An ambient emitter emits one particle per time slot (`1 / rate` seconds); the slot number and
//! the emitter's key seed a small visual-only RNG that picks the particle's glyph, offset,
//! velocity, and lifetime. A burst is `count` particles seeded by the event. So particles never
//! depend on frame rate, never touch gameplay randomness, and `--freeze-time` shows the same
//! particles every run.

use super::post::emissive;
use bevy::prelude::*;
use std::f32::consts::TAU;
use warlock_core::color::Rgb;
use warlock_core::content::ParticleEmitter;

/// Most particles one ambient emitter or burst shows at a time.
pub const MAX_PER_EMITTER: usize = 48;

/// How much of the nominal life a particle may lose (lives vary between 70 % and 100 %).
const LIFE_JITTER: f32 = 0.3;

/// Fraction of the life spent fading in, so particles never pop into view.
const FADE_IN: f32 = 0.1;

/// Sideways drift cycles per second (`sway`).
const SWAY_HZ: f32 = 0.35;

/// One particle at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    /// Top-left corner of the particle's glyph cell, in level cells (fractional).
    pub pos: Vec2,
    pub ch: char,
    /// Linear color with fade as alpha; glowing particles exceed 1 (HDR).
    pub color: LinearRgba,
    pub scale: f32,
    /// Whether the particle glows (it is a light, so scene lighting does not shade it).
    pub emissive: bool,
}

/// SplitMix64: a tiny, well-mixed generator for visual randomness only.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    /// A generator for `stream` of `key`; equal inputs give equal sequences on every platform.
    pub fn new(key: u64, stream: u64) -> Self {
        let mut rng = Rng(key ^ stream.wrapping_mul(0xd1b5_4a32_d192_ed03));
        rng.next_u64();
        rng
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in `[-1, 1)`.
    pub fn signed(&mut self) -> f32 {
        self.unit() * 2.0 - 1.0
    }
}

/// An sRGB content color as linear RGB.
pub fn linear(c: Rgb) -> LinearRgba {
    Color::srgb_u8(c.r, c.g, c.b).to_linear()
}

/// The color of a gradient of evenly spaced `stops` at `x` in `[0, 1]`, blended in linear RGB.
/// `paint` maps each stop to the screen (the color palette).
pub fn gradient(stops: &[Rgb], x: f32, paint: impl Fn(Rgb) -> Rgb) -> LinearRgba {
    match stops {
        [] => LinearRgba::WHITE,
        [only] => linear(paint(*only)),
        _ => {
            let t = x.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
            let i = (t.floor() as usize).min(stops.len() - 2);
            let (a, b) = (linear(paint(stops[i])), linear(paint(stops[i + 1])));
            let f = t - i as f32;
            LinearRgba::rgb(
                a.red + (b.red - a.red) * f,
                a.green + (b.green - a.green) * f,
                a.blue + (b.blue - a.blue) * f,
            )
        }
    }
}

/// Opacity over a particle's life (`x` in `[0, 1]`): a quick fade in, then a fade out over the
/// last `fade` of its life.
pub fn fade(x: f32, fade: f32) -> f32 {
    let fade_in = (x / FADE_IN).min(1.0);
    let fade_out = if fade > 0.0 && x > 1.0 - fade {
        (1.0 - x) / fade
    } else {
        1.0
    };
    (fade_in * fade_out).clamp(0.0, 1.0)
}

/// How particles are placed and colored: the emitter's origin (a cell's top-left, in level cells),
/// which way its `y` points (a unit vector; [`Source::UPRIGHT`] as written, or downstream for
/// `obstacle` emitters), and the color palette applied to its colors.
#[derive(Clone, Copy, Debug)]
pub struct Source<P> {
    pub origin: Vec2,
    pub down: Vec2,
    pub paint: P,
}

impl Source<()> {
    /// An emitter as written: `y` points down the screen.
    pub const UPRIGHT: Vec2 = Vec2::new(0.0, 1.0);
}

/// The particle drawn from `rng`, `age` seconds after it left its source, or `None` if it is not
/// alive then. Draws the same random numbers in the same order whatever the age, so a particle
/// keeps its identity over its life.
fn particle(
    e: &ParticleEmitter,
    rng: &mut Rng,
    source: Source<impl Fn(Rgb) -> Rgb>,
    age: f32,
) -> Option<Particle> {
    let life = e.life * (1.0 - LIFE_JITTER * rng.unit());
    let n = e.glyphs.chars().count().max(1);
    let ch = e
        .glyphs
        .chars()
        .nth(((rng.unit() * n as f32) as usize).min(n - 1))
        .unwrap_or('.');
    let offset = Vec2::from(e.offset) + Vec2::new(rng.signed(), rng.signed()) * Vec2::from(e.area);
    let velocity =
        Vec2::from(e.velocity) + Vec2::new(rng.signed(), rng.signed()) * Vec2::from(e.spread);
    let sway_phase = rng.unit();
    if !(0.0..life).contains(&age) {
        return None;
    }
    let x = age / life;
    let sway = e.sway * (TAU * (age * SWAY_HZ + sway_phase)).sin();
    let moved = offset + velocity * age + Vec2::new(sway, 0.5 * e.gravity * age * age);
    // Turn the emitter so its `y` points `down` (its `x` stays to the right of it).
    let down = source.down;
    let pos = source.origin + Vec2::new(down.y, -down.x) * moved.x + down * moved.y;
    let mut color = emissive(gradient(&e.colors, x, source.paint), e.glow);
    color.alpha = fade(x, e.fade);
    Some(Particle {
        pos,
        ch,
        color,
        scale: e.scale,
        emissive: e.glow > 0.0,
    })
}

/// The live particles of an ambient emitter at `time`. `key` identifies the source (its cell or
/// creature), so neighboring sources never emit in lockstep.
pub fn ambient(
    e: &ParticleEmitter,
    key: u64,
    source: Source<impl Fn(Rgb) -> Rgb + Copy>,
    time: f32,
    out: &mut Vec<Particle>,
) {
    if e.rate <= 0.0 || e.life <= 0.0 {
        return;
    }
    let period = 1.0 / e.rate;
    // Slots born within one life (plus a slot of jitter) before `time` may still be alive.
    let last = (time / period).floor() as i64;
    let first = ((time - e.life) / period).floor() as i64 - 1;
    let first = first.max(last - MAX_PER_EMITTER as i64 + 1);
    for slot in first..=last {
        let mut rng = Rng::new(key, slot as u64);
        let born = (slot as f32 + rng.unit()) * period;
        if let Some(p) = particle(e, &mut rng, source, time - born) {
            out.push(p);
        }
    }
}

/// The live particles of a burst `age` seconds after it went off.
pub fn burst(
    e: &ParticleEmitter,
    seed: u64,
    source: Source<impl Fn(Rgb) -> Rgb + Copy>,
    age: f32,
    out: &mut Vec<Particle>,
) {
    for i in 0..(e.count as usize).min(MAX_PER_EMITTER) {
        let mut rng = Rng::new(seed, i as u64);
        if let Some(p) = particle(e, &mut rng, source, age) {
            out.push(p);
        }
    }
}

/// A stable 64-bit key for a level position, used to seed a source's particles.
pub fn cell_key(x: i32, y: i32) -> u64 {
    (u64::from(x as u32) << 32) | u64::from(y as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::content::ParticleTrigger;

    fn sparks() -> ParticleEmitter {
        ParticleEmitter {
            on: ParticleTrigger::Ambient,
            glyphs: "'`.".into(),
            colors: vec![Rgb::WHITE, Rgb::new(255, 128, 0)],
            rate: 8.0,
            count: 0,
            life: 1.0,
            velocity: [0.0, -1.5],
            spread: [0.4, 0.3],
            gravity: 0.6,
            offset: [0.0, 0.0],
            area: [0.3, 0.1],
            sway: 0.1,
            scale: 0.7,
            fade: 0.5,
            glow: 0.5,
        }
    }

    fn at(x: f32, y: f32) -> Source<fn(Rgb) -> Rgb> {
        Source {
            origin: Vec2::new(x, y),
            down: Source::UPRIGHT,
            paint: |c| c,
        }
    }

    #[test]
    fn turned_emitters_move_their_way() {
        let mut e = sparks();
        e.gravity = 0.0;
        e.sway = 0.0;
        e.velocity = [0.0, 1.0];
        e.spread = [0.0, 0.0];
        e.area = [0.0, 0.0];
        e.offset = [0.2, 0.0];
        let mut rng = Rng::new(1, 1);
        let upright = particle(&e, &mut rng.clone(), at(0.0, 0.0), 0.5).expect("alive");
        assert!((upright.pos - Vec2::new(0.2, 0.5)).length() < 1e-5);
        // Turned to run east: `y` (downstream) is east, `x` is north.
        let east = Source {
            down: Vec2::new(1.0, 0.0),
            ..at(0.0, 0.0)
        };
        let turned = particle(&e, &mut rng, east, 0.5).expect("alive");
        assert!((turned.pos - Vec2::new(0.5, -0.2)).length() < 1e-5);
    }

    #[test]
    fn rng_is_deterministic_and_uniform() {
        let a: Vec<u64> = (0..4).map(|_| Rng::new(7, 3).next_u64()).collect();
        assert!(a.windows(2).all(|w| w[0] == w[1]));
        assert_ne!(Rng::new(7, 3).next_u64(), Rng::new(7, 4).next_u64());
        assert_ne!(Rng::new(7, 3).next_u64(), Rng::new(8, 3).next_u64());
        let mut rng = Rng::new(1, 2);
        let mut buckets = [0u32; 4];
        for _ in 0..4000 {
            let u = rng.unit();
            assert!((0.0..1.0).contains(&u));
            buckets[(u * 4.0) as usize] += 1;
        }
        assert!(
            buckets.iter().all(|&n| (850..1150).contains(&n)),
            "{buckets:?}"
        );
    }

    #[test]
    fn ambient_particles_are_a_pure_function_of_time() {
        let e = sparks();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        ambient(&e, cell_key(3, 4), at(3.0, 4.0), 12.34, &mut a);
        ambient(&e, cell_key(3, 4), at(3.0, 4.0), 12.34, &mut b);
        assert_eq!(a, b);
        // About rate × mean life particles are alive.
        assert!((4..=10).contains(&a.len()), "{} particles", a.len());
        let mut other = Vec::new();
        ambient(&e, cell_key(4, 4), at(3.0, 4.0), 12.34, &mut other);
        assert_ne!(a, other, "different sources emit differently");
    }

    #[test]
    fn particles_rise_fade_and_stay_near_their_source() {
        let e = sparks();
        let mut all = Vec::new();
        for i in 0..50 {
            ambient(&e, 99, at(10.0, 10.0), i as f32 * 0.21, &mut all);
        }
        for p in &all {
            // Rising sparks: at most 1.5 cells/s for a second, gravity slows them.
            assert!(
                p.pos.y <= 10.0 + e.area[1] + 0.05 && p.pos.y > 8.0,
                "{:?}",
                p.pos
            );
            assert!((p.pos.x - 10.0).abs() <= 0.3 + 0.4 + 0.1 + 1e-4);
            assert!((0.0..=1.0).contains(&p.color.alpha));
            assert!("'`.".contains(p.ch));
            assert!(p.emissive);
        }
    }

    #[test]
    fn a_particle_keeps_its_identity_over_its_life() {
        let e = sparks();
        let mut rng = Rng::new(5, 5);
        let young = particle(&e, &mut rng.clone(), at(0.0, 0.0), 0.1).expect("alive");
        let old = particle(&e, &mut rng, at(0.0, 0.0), 0.5).expect("alive");
        assert_eq!(young.ch, old.ch);
        assert!(old.pos.y < young.pos.y, "it rises");
        assert_eq!(particle(&e, &mut Rng::new(5, 5), at(0.0, 0.0), -0.1), None);
        assert_eq!(particle(&e, &mut Rng::new(5, 5), at(0.0, 0.0), 1.0), None);
    }

    #[test]
    fn bursts_show_count_particles_then_end() {
        let mut e = sparks();
        e.on = ParticleTrigger::Step;
        e.count = 6;
        let mut out = Vec::new();
        burst(&e, 42, at(0.0, 0.0), 0.05, &mut out);
        assert_eq!(out.len(), 6);
        out.clear();
        burst(&e, 42, at(0.0, 0.0), 1.0, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn fade_ramps_in_and_out() {
        assert_eq!(fade(0.0, 0.5), 0.0);
        assert_eq!(fade(0.3, 0.5), 1.0);
        assert!((fade(0.75, 0.5) - 0.5).abs() < 1e-6);
        assert_eq!(fade(1.0, 0.5), 0.0);
        assert_eq!(fade(0.9, 0.0), 1.0);
    }

    #[test]
    fn gradient_interpolates_stops() {
        let stops = [Rgb::BLACK, Rgb::WHITE, Rgb::BLACK];
        let same = |c| c;
        assert_eq!(gradient(&stops, 0.0, same), LinearRgba::BLACK);
        assert_eq!(gradient(&stops, 0.5, same), LinearRgba::WHITE);
        assert_eq!(gradient(&stops, 1.0, same), LinearRgba::BLACK);
        assert!((gradient(&stops, 0.25, same).red - 0.5).abs() < 1e-6);
        assert_eq!(gradient(&[Rgb::WHITE], 0.7, same), LinearRgba::WHITE);
        // The palette applies to every stop.
        let invert = |c: Rgb| Rgb::new(255 - c.r, 255 - c.g, 255 - c.b);
        assert_eq!(gradient(&stops, 0.0, invert), LinearRgba::WHITE);
    }
}
