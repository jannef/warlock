//! Glyph animation (spec §4.3): the curves, per-cell phases, and the table of terrain animations
//! the grid shader reads.
//!
//! Everything here is a pure function of visual time, so `--freeze-time` makes frames
//! reproducible. Animation that stays inside its cell (`wave`, `shimmer`, `cycle`) runs in the grid
//! shader (`console.wesl`, which mirrors the curves below) from per-cell data. The free-glyph layer
//! evaluates the same curves on the CPU ([`evaluate`]) for creatures and for `pulse`, which grows
//! beyond its cell.

use super::console::CellAnim;
use crate::settings::{Palette, Role};
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use std::f32::consts::{PI, TAU};
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::content::{Content, GlyphAnim, TerrainId};

/// Shader kind of a grid animation (`GridAnim::kind`); 0 means static.
const KIND_WAVE: u32 = 1;
const KIND_SHIMMER: u32 = 2;
const KIND_CYCLE: u32 = 3;

/// Most animated terrain types the grid shader can tell apart.
pub const MAX_GRID_ANIMS: usize = 32;

/// Most frames a grid `cycle` may have (two 16-bit atlas slots per `u32`).
pub const MAX_CYCLE_FRAMES: usize = 8;

/// Sideways sway of a wave relative to its vertical bob: together they trace an ellipse, the way
/// water moves under a passing wave. Mostly vertical, because wide glyphs like `~` have room to
/// move up and down in their cell but not sideways.
const WAVE_SWAY: f32 = 0.35;

/// Cycles the wave phase advances per cell along x and y, so crests travel across a pool.
const WAVE_TRAVEL: Vec2 = Vec2::new(0.13, 0.06);

/// How far (in cycles) a cell's wave phase may stray from the travelling front.
const WAVE_JITTER: f32 = 0.15;

/// Heartbeat bumps as (center, half width) in beats; the "dub" is smaller than the "lub".
const LUB: (f32, f32) = (0.10, 0.10);
const DUB: (f32, f32) = (0.32, 0.10);
const DUB_HEIGHT: f32 = 0.6;

/// A well-mixed 32-bit hash of a grid position, identical on every platform.
pub fn cell_hash(p: Pos) -> u32 {
    let mut h = (p.x as u32).wrapping_mul(0x8da6_b343) ^ (p.y as u32).wrapping_mul(0xd816_3841);
    // MurmurHash3 finalizer.
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// A phase in `[0, 1)` cycles from a hash.
pub fn hash_phase(hash: u32) -> f32 {
    (hash >> 8) as f32 / (1u32 << 24) as f32
}

/// The phase of a `wave` at `p`: a front travelling across the level, plus per-cell jitter so
/// neighbors never move in lockstep.
pub fn wave_phase(p: Pos) -> f32 {
    let front = p.x as f32 * WAVE_TRAVEL.x + p.y as f32 * WAVE_TRAVEL.y;
    (front + WAVE_JITTER * hash_phase(cell_hash(p))).rem_euclid(1.0)
}

/// The phase of a non-travelling animation at `p` (independent per cell).
pub fn cell_phase(p: Pos) -> f32 {
    hash_phase(cell_hash(p))
}

/// Quantizes a phase to the byte stored in the per-cell animation texture.
pub fn phase_byte(phase: f32) -> u8 {
    ((phase.rem_euclid(1.0) * 256.0) as u32).min(255) as u8
}

/// Offset of a `wave` glyph at `time`, in cell widths (x right, y down): a bob of `amount` with
/// a smaller sideways sway.
pub fn wave_offset(time: f32, speed: f32, amount: f32, phase: f32) -> Vec2 {
    let a = TAU * (time * speed - phase);
    Vec2::new(WAVE_SWAY * a.cos(), a.sin()) * amount
}

/// How far a `shimmer` has moved toward its color at `time`, in `[0, 1]`.
pub fn shimmer_mix(time: f32, speed: f32, phase: f32) -> f32 {
    0.5 - 0.5 * (TAU * (time * speed + phase)).cos()
}

/// The frame a `cycle` of `frames` frames shows at `time`.
pub fn cycle_frame(time: f32, fps: f32, phase: f32, frames: usize) -> usize {
    if frames == 0 {
        return 0;
    }
    let n = frames as f32;
    ((time * fps + phase * n).floor().rem_euclid(n) as usize).min(frames - 1)
}

/// A raised-cosine bump: 1 at `center`, falling smoothly to 0 at `center ± half_width`.
fn bump(x: f32, (center, half_width): (f32, f32)) -> f32 {
    let d = (x - center).abs() / half_width;
    if d >= 1.0 {
        0.0
    } else {
        0.5 + 0.5 * (PI * d).cos()
    }
}

/// The "lub-dub" heartbeat envelope over one beat, in `[0, 1]`: a strong beat, a softer echo,
/// then rest.
pub fn heartbeat(beat: f32) -> f32 {
    let x = beat.rem_euclid(1.0);
    bump(x, LUB) + DUB_HEIGHT * bump(x, DUB)
}

/// Scale of a `pulse` glyph at `time` (1 at rest).
pub fn pulse_scale(time: f32, bpm: f32, amount: f32, phase: f32) -> f32 {
    1.0 + amount * heartbeat(time * bpm / 60.0 + phase)
}

/// An animation evaluated at one moment, for the free-glyph layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimFrame {
    /// Offset from the glyph's cell, in cell widths.
    pub offset: Vec2,
    /// Scale about the cell center.
    pub scale: f32,
    /// The glyph to show instead of the base glyph (`cycle`).
    pub glyph: Option<char>,
    /// Blend of the foreground toward a color (`shimmer`): the color and its weight.
    pub tint: Option<(Rgb, f32)>,
}

impl AnimFrame {
    /// No animation.
    pub const STILL: AnimFrame = AnimFrame {
        offset: Vec2::ZERO,
        scale: 1.0,
        glyph: None,
        tint: None,
    };
}

/// Evaluates `anim` at `time` for a glyph whose phase is `phase` (in cycles).
pub fn evaluate(anim: &GlyphAnim, time: f32, phase: f32) -> AnimFrame {
    let mut frame = AnimFrame::STILL;
    match anim {
        GlyphAnim::Wave { speed, amount } => {
            frame.offset = wave_offset(time, *speed, *amount, phase);
        }
        GlyphAnim::Pulse { bpm, amount } => frame.scale = pulse_scale(time, *bpm, *amount, phase),
        GlyphAnim::Shimmer { color, speed } => {
            frame.tint = Some((*color, shimmer_mix(time, *speed, phase)));
        }
        GlyphAnim::Cycle { frames, fps } => {
            let n = frames.chars().count();
            frame.glyph = frames.chars().nth(cycle_frame(time, *fps, phase, n));
        }
    }
    frame
}

/// Where a terrain type's animation runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainAnim {
    /// In the grid shader; the value is the table entry plus one (as stored per cell).
    Grid { entry: u8, travelling: bool },
    /// On the free-glyph layer, because it grows beyond its cell (`pulse`).
    Free,
}

/// One grid animation as the shader reads it.
#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct GridAnim {
    pub kind: u32,
    /// Cycles per second (`wave`, `shimmer`) or frames per second (`cycle`).
    pub speed: f32,
    /// Sway in cell widths (`wave`).
    pub amount: f32,
    pub frame_count: u32,
    /// Target color, linear RGB (`shimmer`).
    pub color: Vec4,
    /// Atlas slots of the frames, two 16-bit slots per component (`cycle`).
    pub frames: UVec4,
}

/// The grid shader's animation table (a uniform).
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct GridAnimTable {
    pub entries: [GridAnim; MAX_GRID_ANIMS],
}

impl Default for GridAnimTable {
    fn default() -> Self {
        Self {
            entries: [GridAnim::default(); MAX_GRID_ANIMS],
        }
    }
}

/// How each terrain type animates, and the shader table for those animated in the grid. Rebuilt
/// when the atlas or palette changes (cycle frames are atlas slots; shimmer colors go through the
/// palette). Empty with glyph animation off.
#[derive(Resource, Debug, Default)]
pub struct AnimTable {
    terrain: Vec<Option<TerrainAnim>>,
    pub gpu: GridAnimTable,
}

impl AnimTable {
    /// Builds the table for `content`; `slot` maps a glyph to its atlas slot. With `enabled`
    /// false (glyph animation off) nothing animates.
    pub fn new(
        content: &Content,
        slot: impl Fn(char) -> u16,
        enabled: bool,
        palette: Palette,
    ) -> Self {
        let mut table = Self::default();
        let mut next = 0;
        for def in content.terrain_defs() {
            let anim = def.anim.as_ref().filter(|_| enabled);
            let entry = match anim {
                None => None,
                Some(GlyphAnim::Pulse { .. }) => Some(TerrainAnim::Free),
                Some(_) if next >= MAX_GRID_ANIMS => {
                    warn!("too many animated terrain types; `{}` stays still", def.id);
                    None
                }
                Some(anim) => grid_anim(anim, &slot, palette).map(|gpu| {
                    table.gpu.entries[next] = gpu;
                    next += 1;
                    TerrainAnim::Grid {
                        entry: next as u8,
                        travelling: gpu.kind == KIND_WAVE,
                    }
                }),
            };
            table.terrain.push(entry);
        }
        table
    }

    /// How terrain `t` animates, if at all.
    pub fn terrain(&self, t: TerrainId) -> Option<TerrainAnim> {
        self.terrain.get(usize::from(t.0)).copied().flatten()
    }

    /// Per-cell animation data for visible terrain `t` at level position `p`.
    pub fn cell(&self, t: TerrainId, p: Pos) -> CellAnim {
        match self.terrain(t) {
            Some(TerrainAnim::Grid { entry, travelling }) => CellAnim {
                entry,
                phase: phase_byte(if travelling {
                    wave_phase(p)
                } else {
                    cell_phase(p)
                }),
            },
            _ => CellAnim::STILL,
        }
    }
}

fn linear(c: Rgb) -> Vec4 {
    let l = Color::srgb_u8(c.r, c.g, c.b).to_linear();
    Vec4::new(l.red, l.green, l.blue, 1.0)
}

/// The shader form of a grid animation; `None` for animations that do nothing.
fn grid_anim(anim: &GlyphAnim, slot: impl Fn(char) -> u16, palette: Palette) -> Option<GridAnim> {
    Some(match anim {
        GlyphAnim::Wave { speed, amount } => GridAnim {
            kind: KIND_WAVE,
            speed: *speed,
            amount: *amount,
            ..default()
        },
        GlyphAnim::Shimmer { color, speed } => GridAnim {
            kind: KIND_SHIMMER,
            speed: *speed,
            color: linear(palette.apply(*color, Role::Fg)),
            ..default()
        },
        GlyphAnim::Cycle { frames, fps } => {
            let slots: Vec<u32> = frames
                .chars()
                .take(MAX_CYCLE_FRAMES)
                .map(|c| u32::from(slot(c)))
                .collect();
            if slots.len() < 2 {
                return None;
            }
            let mut packed = [0u32; MAX_CYCLE_FRAMES / 2];
            for (i, s) in slots.iter().enumerate() {
                packed[i / 2] |= s << (16 * (i % 2));
            }
            GridAnim {
                kind: KIND_CYCLE,
                speed: *fps,
                frame_count: slots.len() as u32,
                frames: UVec4::from_array(packed),
                ..default()
            }
        }
        GlyphAnim::Pulse { .. } => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_neighbors_differ() {
        assert_eq!(cell_hash(Pos::new(3, 7)), cell_hash(Pos::new(3, 7)));
        let h = cell_hash(Pos::new(10, 10));
        for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1)] {
            assert_ne!(h, cell_hash(Pos::new(10 + dx, 10 + dy)));
        }
        assert_ne!(cell_hash(Pos::new(1, 2)), cell_hash(Pos::new(2, 1)));
    }

    #[test]
    fn hash_phases_spread_over_the_cycle() {
        let mut buckets = [0u32; 8];
        for y in 0..40 {
            for x in 0..40 {
                let phase = cell_phase(Pos::new(x, y));
                assert!((0.0..1.0).contains(&phase));
                buckets[(phase * 8.0) as usize] += 1;
            }
        }
        // 1600 samples, 200 expected per bucket.
        assert!(
            buckets.iter().all(|&n| (140..260).contains(&n)),
            "{buckets:?}"
        );
    }

    #[test]
    fn wave_phase_travels_with_jitter() {
        let a = wave_phase(Pos::new(0, 0));
        let b = wave_phase(Pos::new(1, 0));
        let ahead = (b - a).rem_euclid(1.0);
        // The front advances by WAVE_TRAVEL.x, jitter moves it by less than WAVE_JITTER.
        assert!((ahead - WAVE_TRAVEL.x).abs() < WAVE_JITTER || ahead > 1.0 - WAVE_JITTER);
        assert!((0.0..1.0).contains(&wave_phase(Pos::new(-5, 300))));
    }

    #[test]
    fn phase_bytes_round_trip() {
        assert_eq!(phase_byte(0.0), 0);
        assert_eq!(phase_byte(0.5), 128);
        assert_eq!(phase_byte(0.999_999), 255);
        assert_eq!(phase_byte(1.25), 64);
        assert_eq!(phase_byte(-0.25), 192);
    }

    #[test]
    fn heartbeat_has_lub_then_smaller_dub_then_rest() {
        let lub = heartbeat(LUB.0);
        let dub = heartbeat(DUB.0);
        assert!((lub - 1.0).abs() < 1e-6);
        assert!((dub - DUB_HEIGHT).abs() < 1e-6);
        assert!(heartbeat(0.21) < 0.05, "a pause between lub and dub");
        assert_eq!(heartbeat(0.7), 0.0);
        // Continuous across the beat boundary.
        assert!((heartbeat(0.999) - heartbeat(1.001)).abs() < 1e-3);
        let peak = (0..1000)
            .map(|i| heartbeat(i as f32 / 1000.0))
            .fold(0.0, f32::max);
        assert!(peak <= 1.0 + 1e-6);
    }

    #[test]
    fn pulse_grows_beyond_the_cell_and_rests_at_one() {
        assert_eq!(pulse_scale(0.7, 60.0, 0.2, 0.0), 1.0);
        let peak = pulse_scale(LUB.0, 60.0, 0.2, 0.0);
        assert!((peak - 1.2).abs() < 1e-5);
        // 120 bpm beats twice as fast.
        assert!((pulse_scale(LUB.0 / 2.0, 120.0, 0.2, 0.0) - 1.2).abs() < 1e-5);
    }

    #[test]
    fn wave_bobs_within_amount() {
        for i in 0..100 {
            let o = wave_offset(i as f32 * 0.037, 0.6, 0.25, 0.3);
            assert!(o.y.abs() <= 0.25 + 1e-6 && o.x.abs() <= 0.25 * WAVE_SWAY + 1e-6);
        }
        // A quarter period apart, the glyph moves from center height to the bottom of its bob.
        let speed = 0.5;
        let a = wave_offset(0.0, speed, 0.2, 0.0);
        let b = wave_offset(0.25 / speed, speed, 0.2, 0.0);
        assert!(a.y.abs() < 1e-6 && (b.y - 0.2).abs() < 1e-5);
        // Phase shifts the wave in time: a cell further along the front lags behind.
        let ahead = wave_offset(1.0, speed, 0.2, 0.25);
        assert!((ahead - wave_offset(0.5, speed, 0.2, 0.0)).length() < 1e-5);
    }

    #[test]
    fn shimmer_oscillates_between_base_and_color() {
        assert!(shimmer_mix(0.0, 1.0, 0.0).abs() < 1e-6);
        assert!((shimmer_mix(0.5, 1.0, 0.0) - 1.0).abs() < 1e-6);
        assert!((shimmer_mix(0.0, 1.0, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cycle_steps_through_frames() {
        let frames: Vec<usize> = (0..6)
            .map(|i| cycle_frame(i as f32 * 0.5, 2.0, 0.0, 3))
            .collect();
        assert_eq!(frames, vec![0, 1, 2, 0, 1, 2]);
        assert_eq!(cycle_frame(0.0, 1.0, 0.5, 2), 1);
        assert_eq!(cycle_frame(3.0, 1.0, 0.0, 0), 0);
    }

    #[test]
    fn evaluate_maps_each_kind() {
        let wave = GlyphAnim::Wave {
            speed: 1.0,
            amount: 0.2,
        };
        assert!((evaluate(&wave, 0.25, 0.0).offset.y - 0.2).abs() < 1e-5);
        let cycle = GlyphAnim::Cycle {
            frames: "~≈".into(),
            fps: 1.0,
        };
        assert_eq!(evaluate(&cycle, 1.5, 0.0).glyph, Some('≈'));
        let shimmer = GlyphAnim::Shimmer {
            color: Rgb::WHITE,
            speed: 1.0,
        };
        let (color, weight) = evaluate(&shimmer, 0.5, 0.0).tint.expect("shimmer tints");
        assert_eq!(color, Rgb::WHITE);
        assert!((weight - 1.0).abs() < 1e-5);
        let pulse = GlyphAnim::Pulse {
            bpm: 60.0,
            amount: 0.5,
        };
        assert!(evaluate(&pulse, LUB.0, 0.0).scale > 1.4);
    }

    #[test]
    fn table_assigns_grid_entries_and_free_pulses() {
        let content = Content::builtin().expect("built-in content loads");
        let water = content.terrain_id("water").expect("water exists");
        let altar = content.terrain_id("altar").expect("altar exists");
        let floor = content.terrain_id("floor").expect("floor exists");
        let table = AnimTable::new(&content, |_| 7, true, Palette::Default);
        assert!(matches!(
            table.terrain(water),
            Some(TerrainAnim::Grid {
                travelling: true,
                ..
            })
        ));
        assert_eq!(table.terrain(altar), Some(TerrainAnim::Free));
        assert_eq!(table.terrain(floor), None);
        let cell = table.cell(water, Pos::new(4, 2));
        assert!(cell.entry > 0);
        assert_eq!(
            table.gpu.entries[usize::from(cell.entry) - 1].kind,
            KIND_WAVE
        );
        assert_eq!(table.cell(floor, Pos::new(4, 2)), CellAnim::STILL);

        let reduced = AnimTable::new(&content, |_| 7, false, Palette::Default);
        assert_eq!(reduced.terrain(water), None);
        assert_eq!(reduced.terrain(altar), None);
    }

    #[test]
    fn shimmer_color_in_the_shader_table_uses_the_palette() {
        let shimmer = GlyphAnim::Shimmer {
            color: Rgb::new(0xff, 0xd0, 0x60),
            speed: 1.0,
        };
        let table = |palette| grid_anim(&shimmer, |_| 0, palette).expect("shimmer").color;
        assert_eq!(table(Palette::Default), linear(Rgb::new(0xff, 0xd0, 0x60)));
        let high = Palette::HighContrast.apply(Rgb::new(0xff, 0xd0, 0x60), Role::Fg);
        assert_eq!(table(Palette::HighContrast), linear(high));
        assert_ne!(table(Palette::HighContrast), table(Palette::Default));
    }

    #[test]
    fn cycle_frames_pack_two_slots_per_word() {
        let anim = GlyphAnim::Cycle {
            frames: "abc".into(),
            fps: 2.0,
        };
        let gpu = grid_anim(&anim, |c| c as u16, Palette::Default).expect("three frames animate");
        assert_eq!(gpu.frame_count, 3);
        assert_eq!(gpu.frames.x, u32::from(b'a') | (u32::from(b'b') << 16));
        assert_eq!(gpu.frames.y, u32::from(b'c'));
        let single = GlyphAnim::Cycle {
            frames: "a".into(),
            fps: 2.0,
        };
        assert_eq!(grid_anim(&single, |c| c as u16, Palette::Default), None);
    }
}
