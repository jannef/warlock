//! Visual lighting (spec §4.4): the light the map is drawn with, derived every frame from the
//! perceived gameplay light plus visual-only flicker. Gameplay never reads any of it.
//!
//! [`VisualLight::composite`] covers the map grid's cells plus a one-cell margin ([`MARGIN`]). For
//! each visible cell it sums the level's ambient light and every visible source's contribution,
//! and folds in the presentation rules: UV light as a violet tint (perception reports UV only to
//! eyes that see it) and the [`darkvision`] fade. Flicker ([`flicker`]: smooth noise keyed by the
//! source, driven by [`VisualTime`]; none with flicker off) becomes a per-cell brightness
//! gain, the contribution-weighted mean of the flicker of the sources lighting the cell, applied
//! after shading so it reads the same in dim and in overbright light. The result is one
//! [`CellLight`] per cell, uploaded as two textures aligned with the map grid:
//! - `light` (`Rgba32Float`): red, green, and blue light (1.0 = fully lit, more = overbright)
//!   and color saturation;
//! - `light_flags` (`Rgba8Uint`): [`LIT`] on visible cells and [`OPAQUE`] on walls (red), and
//!   the flicker gain (green, see [`gain_byte`]).
//!
//! Creature shadows ([`super::shadows`], visual only as well) take part of each source's
//! contribution away behind the creatures it lights. The shaded light is left out of what is
//! drawn but not out of what darkvision measures, so darkvision's glow never brightens a shadow.
//!
//! The console shader (`lighting.wesl`) lights terrain per pixel, interpolating between cell
//! centers ([`interpolation_weights`]). A neighbor only contributes if it is lit and transparent,
//! so light never bleeds across walls or into unseen or remembered cells, and walls show their own
//! wall-face light, flat. Unlit cells (remembered terrain, the UI console) are drawn as they are.
//! Free glyphs are lit on the CPU by [`VisualLight::sample`] and [`shade`], which mirror the
//! shader: change both together.

use super::grid::MapGrid;
use super::shadows::Shadows;
use super::{ConsoleHandles, ConsoleMaterial, Effects, VisualTime};
use crate::game::Game;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::shader::load_shader_library;
use warlock_core::Pos;
use warlock_core::content::Flicker;
use warlock_core::fov::radius2;
use warlock_core::light::Light;
use warlock_core::perception::{CellView, PerceptionView};

/// Cells of light around the map grid on each side, so interpolation at the edges of the
/// viewport sees the cells just outside it. Mirrored in `lighting.wesl`.
pub const MARGIN: i32 = 1;

/// Cell flag: visible, so lit by current light. Cells without it are drawn as they are.
pub const LIT: u8 = 1;
/// Cell flag: opaque. Walls show their own (wall-face) light and never take or give
/// interpolated light.
pub const OPAQUE: u8 = 2;

/// Lowest exposure (linear) of visible terrain, so dim cells stay readable and a little brighter
/// than remembered terrain. Mirrored in `lighting.wesl`.
pub const TERRAIN_FLOOR: f32 = 0.12;
/// Lowest exposure of creatures: they stand out more than the terrain they stand on.
pub const ACTOR_FLOOR: f32 = 0.3;
/// Overbright light brightens a surface by at most this fraction of its base color, approached
/// smoothly. Mirrored in `lighting.wesl`.
pub const OVERBRIGHT: f32 = 0.6;
/// Overbright light also shows as a faint glow of its own color on the surface, at most this much
/// (linear), so strongly lit floors pool with the light's color where surfaces are already at
/// their brightest. Mirrored in `lighting.wesl`.
pub const GLOW: f32 = 0.03;
/// The violet UV light tints with, per unit of perceived UV.
pub const UV_TINT: Vec3 = Vec3::new(0.5, 0.15, 1.0);
/// Linear-light luma weights (Rec. 709). Mirrored in `lighting.wesl`.
const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

/// Darkvision glow (as light, 1.0 = fully lit) in total darkness next to the viewer and at the
/// edge of the darkvision radius.
const DARKVISION_NEAR: f32 = 0.4;
const DARKVISION_EDGE: f32 = 0.15;
/// Saturation darkvision keeps in total darkness next to the viewer (none at the edge).
const DARKVISION_COLOR: f32 = 0.3;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "lighting.wesl");
        app.init_resource::<VisualLight>()
            .init_resource::<Shadows>()
            .add_systems(PostUpdate, upload.before(super::upload));
    }
}

/// The light one cell is drawn with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellLight {
    /// Red, green, blue light: 1.0 = fully lit (the base color), more = overbright.
    pub rgb: Vec3,
    /// Color saturation: 1 = full color, 0 = monochrome.
    pub saturation: f32,
    /// Flicker: brightness multiplier of the lit color.
    pub gain: f32,
}

impl CellLight {
    pub const DARK: CellLight = CellLight {
        rgb: Vec3::ZERO,
        saturation: 1.0,
        gain: 1.0,
    };
}

/// Visual light for the map grid, recomposited every frame.
#[derive(Resource, Debug, Default)]
pub struct VisualLight {
    /// Level position of the first cell: [`MARGIN`] cells up and left of the map grid's (0, 0).
    origin: IVec2,
    /// Size in cells.
    size: UVec2,
    /// Row-major; [`CellLight::DARK`] where not [`LIT`].
    cells: Vec<CellLight>,
    flags: Vec<u8>,
    /// Set when the light changed and needs uploading.
    dirty: bool,
}

impl VisualLight {
    /// Recomputes the light of the `size` cells from level position `origin` on, with creature
    /// `shadows` (computed from the same `view`). `time` is the visual time for flicker, `None`
    /// for steady light (flicker off).
    pub fn composite(
        &mut self,
        view: &PerceptionView<'_>,
        shadows: &Shadows,
        origin: IVec2,
        size: UVec2,
        time: Option<f32>,
    ) {
        let content = view.content();
        let n = (size.x * size.y) as usize;
        let index = |p: Pos| cell_index(origin, size, IVec2::new(p.x, p.y));
        let at = |i: usize| {
            let i = i as u32;
            Pos::new(
                origin.x + (i % size.x) as i32,
                origin.y + (i / size.x) as i32,
            )
        };

        // Ambient light on every visible cell, then each source. `flickers` sums each source's
        // flicker weighted by how much light it adds (x), and those weights (y).
        let mut flags = vec![0; n];
        let mut sums = vec![Vec4::ZERO; n];
        let mut flickers = vec![Vec2::ZERO; n];
        let ambient = light_vec(view.ambient());
        for (i, flag) in flags.iter_mut().enumerate() {
            if let CellView::Visible(t) = view.cell(at(i)) {
                *flag = if content.terrain(t).transparent {
                    LIT
                } else {
                    LIT | OPAQUE
                };
                sums[i] = ambient;
                flickers[i] = Vec2::splat(ambient.element_sum());
            }
        }
        // The light creature shadows take away: not drawn, but kept apart from `sums`, which
        // darkvision measures (its glow would otherwise brighten shadows).
        let mut shaded = vec![Vec4::ZERO; n];
        for source in view.light_sources() {
            let gain = match (time, &content.light(source.preset).flicker) {
                (Some(time), Some(f)) => flicker(source.key, f, time),
                _ => 1.0,
            };
            let shadows = shadows.of(source.key);
            for &(p, light) in &source.contributions {
                if let Some(i) = index(p)
                    && flags[i] & LIT != 0
                {
                    let light = light_vec(light);
                    let shade = light * (1.0 - shadows.keep(p));
                    sums[i] += light;
                    shaded[i] += shade;
                    flickers[i] += Vec2::new(gain, 1.0) * (light - shade).element_sum();
                }
            }
        }

        let vision = view.vision();
        let reach2 = radius2(vision.darkvision).max(1) as f32;
        let threshold = f32::from(vision.threshold);
        let cells: Vec<_> = (0..n)
            .map(|i| {
                if flags[i] & LIT == 0 {
                    return CellLight::DARK;
                }
                let sum = sums[i];
                let drawn = sum - shaded[i];
                let rgb = drawn.truncate() + UV_TINT * drawn.w;
                let flicker = flickers[i];
                let gain = if flicker.y > 0.0 {
                    flicker.x / flicker.y
                } else {
                    1.0
                };
                let light = match view.light(at(i)).filter(|l| l.darkvision) {
                    Some(seen) => {
                        // Brightness relative to the eye's threshold, as gameplay measures it.
                        let lit = (sum.truncate().dot(Vec3::new(54.0, 183.0, 19.0) / 256.0)
                            + sum.w)
                            * 255.0
                            / threshold.max(1.0);
                        let far = (seen.dist2 as f32 / reach2).sqrt();
                        darkvision(rgb, lit, far)
                    }
                    None => CellLight {
                        rgb,
                        ..CellLight::DARK
                    },
                };
                CellLight { gain, ..light }
            })
            .collect();

        if (origin, size) != (self.origin, self.size) || cells != self.cells || flags != self.flags
        {
            *self = VisualLight {
                origin,
                size,
                cells,
                flags,
                dirty: true,
            };
        }
    }

    fn flags_at(&self, cell: IVec2) -> u8 {
        cell_index(IVec2::ZERO, self.size, cell).map_or(0, |i| self.flags[i])
    }

    fn is_open(&self, cell: IVec2) -> bool {
        self.flags_at(cell) & (LIT | OPAQUE) == LIT
    }

    /// The light at level position `at` (fractional; cell `p` spans `p` to `p + 1`), interpolated
    /// like the console shader does; `None` if that cell is not lit.
    pub fn sample(&self, at: Vec2) -> Option<CellLight> {
        let pos = at - self.origin.as_vec2();
        let cell = pos.floor().as_ivec2();
        let flags = self.flags_at(cell);
        if flags & LIT == 0 {
            return None;
        }
        let light = |c: IVec2| cell_index(IVec2::ZERO, self.size, c).map(|i| self.cells[i]);
        if flags & OPAQUE != 0 {
            return light(cell);
        }
        let mut out = CellLight {
            rgb: Vec3::ZERO,
            saturation: 0.0,
            gain: 0.0,
        };
        for (offset, weight) in
            interpolation_weights(pos - cell.as_vec2(), |d| self.is_open(cell + d))
        {
            if weight > 0.0
                && let Some(c) = light(cell + offset)
            {
                out.rgb += c.rgb * weight;
                out.saturation += c.saturation * weight;
                out.gain += c.gain * weight;
            }
        }
        Some(out)
    }
}

fn cell_index(origin: IVec2, size: UVec2, p: IVec2) -> Option<usize> {
    let d = p - origin;
    (d.x >= 0 && d.y >= 0 && (d.x as u32) < size.x && (d.y as u32) < size.y)
        .then(|| (d.y as u32 * size.x + d.x as u32) as usize)
}

/// Gameplay light as red, green, blue, UV with 1.0 = fully lit.
fn light_vec(light: Light) -> Vec4 {
    let [r, g, b, uv]: [u16; 4] = light.into();
    Vec4::new(r.into(), g.into(), b.into(), uv.into()) / 255.0
}

/// A flicker gain as stored in the light flags texture: 128 is 1, in steps of 1/256 (0.5 to
/// about 1.5). Decoded in `lighting.wesl`.
pub fn gain_byte(gain: f32) -> u8 {
    (128.0 + (gain - 1.0) * 256.0).round().clamp(0.0, 255.0) as u8
}

/// Intensity multiplier of a flickering source at visual time `time`: smooth noise between
/// `1 - amount` and `1 + amount` that changes about `speed` times a second, different for every
/// source `key`. Deterministic: the same key and time give the same value.
pub fn flicker(key: u64, flicker: &Flicker, time: f32) -> f32 {
    1.0 + flicker.amount * noise(key, time * flicker.speed)
}

/// Smooth noise in `[-1, 1]`: two octaves of value noise, the second faster and weaker, so the
/// light drifts calmly with a little life in it.
fn noise(key: u64, x: f32) -> f32 {
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

fn lattice(key: u64, i: i64) -> f32 {
    // SplitMix64 finalizer over the key and the lattice index.
    let mut h = key ^ (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h = (h ^ (h >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^= h >> 31;
    (h >> 40) as f32 / (1u64 << 23) as f32 - 1.0
}

/// Darkvision (spec §4.4) for a cell seen only through it, with light `rgb`, brightness `lit`
/// relative to the eye's threshold (1 = just visible without darkvision) and distance `far`
/// relative to the darkvision radius (1 = at its edge). The darker the cell, the more a
/// monochrome glow lights it, fading toward the edge; its color fades toward monochrome the
/// darker it is and the farther from the viewer.
pub fn darkvision(rgb: Vec3, lit: f32, far: f32) -> CellLight {
    let dark = 1.0 - lit.clamp(0.0, 1.0);
    let far = far.clamp(0.0, 1.0);
    let glow = DARKVISION_NEAR + (DARKVISION_EDGE - DARKVISION_NEAR) * far;
    CellLight {
        rgb: rgb + Vec3::splat(glow * dark),
        saturation: darkvision_saturation(lit, far),
        gain: 1.0,
    }
}

/// Saturation of a darkvision cell (see [`darkvision`]): 1 at the eye's threshold, falling the
/// darker and farther it is, to none in total darkness at the edge of the radius.
pub fn darkvision_saturation(lit: f32, far: f32) -> f32 {
    let dark = 1.0 - lit.clamp(0.0, 1.0);
    1.0 - dark * (1.0 - DARKVISION_COLOR * (1.0 - far.clamp(0.0, 1.0)))
}

/// Weights for lighting a point at `frac` (its position inside its cell, 0 to 1 on each axis)
/// from the light at the centers of its cell and of the three neighbors nearest to it, as
/// `(cell offset, weight)` summing to 1. Bilinear, except that neighbors that are not `open`
/// (lit and transparent) get no weight, and neither does the diagonal unless an orthogonal
/// neighbor next to it is open, so light never crosses a wall or a wall corner. The point's own
/// cell always has weight. Mirrored in `lighting.wesl`.
pub fn interpolation_weights(frac: Vec2, open: impl Fn(IVec2) -> bool) -> [(IVec2, f32); 4] {
    let f = frac - Vec2::splat(0.5);
    let step = IVec2::new(
        if f.x < 0.0 { -1 } else { 1 },
        if f.y < 0.0 { -1 } else { 1 },
    );
    let a = f.abs();
    let (dx, dy) = (IVec2::new(step.x, 0), IVec2::new(0, step.y));
    let (open_x, open_y) = (open(dx), open(dy));
    let open_d = (open_x || open_y) && open(step);
    let weights = [
        (IVec2::ZERO, (1.0 - a.x) * (1.0 - a.y)),
        (dx, if open_x { a.x * (1.0 - a.y) } else { 0.0 }),
        (dy, if open_y { (1.0 - a.x) * a.y } else { 0.0 }),
        (step, if open_d { a.x * a.y } else { 0.0 }),
    ];
    let total: f32 = weights.iter().map(|(_, w)| w).sum();
    weights.map(|(d, w)| (d, w / total))
}

/// How bright a surface under light `light` (one channel; 1.0 = fully lit) is relative to its
/// base color: never below `floor`, so it stays readable, linear up to 1, then overbright,
/// approaching `1 + OVERBRIGHT`. Mirrored in `lighting.wesl`.
pub fn exposure(light: f32, floor: f32) -> f32 {
    if light <= 1.0 {
        floor + (1.0 - floor) * light.max(0.0)
    } else {
        1.0 + OVERBRIGHT * knee(light)
    }
}

/// How overbright `light` is, from 0 at full light approaching 1.
fn knee(light: f32) -> f32 {
    let over = (light - 1.0).max(0.0);
    over / (1.0 + over)
}

/// A linear color under `light`, with exposure floor `floor`: scaled by [`exposure`] per
/// channel, plus the overbright [`GLOW`] in the light's hue, desaturated, and scaled by the
/// flicker gain. Mirrored in `lighting.wesl`.
pub fn shade(color: Vec3, light: CellLight, floor: f32) -> Vec3 {
    let [r, g, b] = light.rgb.to_array();
    let peak = light.rgb.max_element();
    let glow = if peak > 1.0 {
        light.rgb / peak * (GLOW * knee(peak))
    } else {
        Vec3::ZERO
    };
    let lit = color * Vec3::new(exposure(r, floor), exposure(g, floor), exposure(b, floor)) + glow;
    Vec3::splat(lit.dot(LUMA)).lerp(lit, light.saturation) * light.gain
}

/// Composites this frame's visual light for the map grid. Runs after the map grid is placed and
/// before free glyphs are lit.
pub fn update(
    game: Res<Game>,
    grid: Res<MapGrid>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    shadows: Res<Shadows>,
    mut light: ResMut<VisualLight>,
) {
    let view = game.world.perception();
    let size = UVec2::new(grid.console.cols(), grid.console.rows()) + 2 * MARGIN as u32;
    let time = effects.flicker.then_some(visual.seconds);
    light.composite(&view, &shadows, grid.level_origin - MARGIN, size, time);
}

/// Writes the visual light into the map material's light textures when it changed.
fn upload(
    mut light: ResMut<VisualLight>,
    handles: Res<ConsoleHandles>,
    materials: Res<Assets<ConsoleMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !light.dirty {
        return;
    }
    let Some(material) = materials.get(&handles.map) else {
        return;
    };
    let size = Extent3d {
        width: light.size.x.max(1),
        height: light.size.y.max(1),
        depth_or_array_layers: 1,
    };
    let texels = (size.width * size.height) as usize;
    let mut values: Vec<u8> = light
        .cells
        .iter()
        .flat_map(|c| [c.rgb.x, c.rgb.y, c.rgb.z, c.saturation])
        .flat_map(f32::to_ne_bytes)
        .collect();
    values.resize(texels * 16, 0);
    let mut flags: Vec<u8> = (light.flags.iter().zip(&light.cells))
        .flat_map(|(&f, c)| [f, gain_byte(c.gain), 0, 0])
        .collect();
    flags.resize(texels * 4, 0);
    for (handle, data) in [(&material.light, values), (&material.light_flags, flags)] {
        if let Some(mut image) = images.get_mut(handle) {
            if image.texture_descriptor.size != size {
                image.texture_descriptor.size = size;
            }
            image.data = Some(data);
        }
    }
    light.dirty = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::shadows::{KEEP, REACH};
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

    const TORCH: Flicker = Flicker {
        amount: 0.2,
        speed: 1.6,
    };

    #[test]
    fn flicker_is_deterministic_smooth_and_in_range() {
        let mut seen = (f32::MAX, f32::MIN);
        let mut last = flicker(7, &TORCH, 0.0);
        for i in 0..20_000 {
            let t = i as f32 * 0.001;
            let f = flicker(7, &TORCH, t);
            assert_eq!(f, flicker(7, &TORCH, t), "same key and time, same value");
            assert!((0.8..=1.2).contains(&f), "{f} at {t}");
            assert!((f - last).abs() < 0.01, "smooth: {last} -> {f} at {t}");
            seen = (seen.0.min(f), seen.1.max(f));
            last = f;
        }
        assert!(seen.1 - seen.0 > 0.15, "it varies: {seen:?}");
        let differs = (0..50).any(|i| {
            let t = i as f32 * 0.37;
            (flicker(7, &TORCH, t) - flicker(8, &TORCH, t)).abs() > 0.01
        });
        assert!(differs, "sources flicker independently");
    }

    #[test]
    fn gains_round_trip_through_a_byte() {
        assert_eq!(gain_byte(1.0), 128);
        for gain in [0.8, 0.93, 1.0, 1.07, 1.2] {
            let back = 1.0 + (f32::from(gain_byte(gain)) - 128.0) / 256.0;
            assert!((back - gain).abs() <= 0.5 / 256.0);
        }
        assert_eq!(gain_byte(9.0), 255);
        assert_eq!(gain_byte(-1.0), 0);
    }

    #[test]
    fn noise_stays_in_range_far_from_zero() {
        for i in 0..10_000 {
            let x = -5_000.0 + i as f32 * 1.37;
            assert!((-1.0..=1.0).contains(&noise(u64::MAX - i, x)));
        }
    }

    fn all_open(_: IVec2) -> bool {
        true
    }

    #[test]
    fn interpolation_is_bilinear_in_the_open() {
        let center = interpolation_weights(Vec2::splat(0.5), all_open);
        assert_eq!(center[0], (IVec2::ZERO, 1.0));
        let corner = interpolation_weights(Vec2::new(0.0, 0.0), all_open);
        assert_eq!(corner.map(|(_, w)| w), [0.25; 4]);
        assert_eq!(corner[3].0, IVec2::new(-1, -1));
        let side = interpolation_weights(Vec2::new(0.9, 0.5), all_open);
        assert_eq!(side[1].0, IVec2::new(1, 0));
        assert!((side[0].1 - 0.6).abs() < 1e-6 && (side[1].1 - 0.4).abs() < 1e-6);
        for frac in [
            Vec2::new(0.1, 0.8),
            Vec2::new(0.7, 0.2),
            Vec2::new(0.99, 0.01),
        ] {
            let total: f32 = interpolation_weights(frac, all_open)
                .iter()
                .map(|w| w.1)
                .sum();
            assert!((total - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn interpolation_never_crosses_walls_or_corners() {
        let wall_east = |d: IVec2| d.x != 1;
        let w = interpolation_weights(Vec2::new(0.95, 0.5), wall_east);
        assert_eq!(w[0], (IVec2::ZERO, 1.0), "nothing from behind the wall");
        // A wall corner: east and south are walls, so the south-east diagonal is cut off too.
        let corner = |d: IVec2| d == IVec2::new(1, 1);
        let w = interpolation_weights(Vec2::new(0.9, 0.9), corner);
        assert_eq!(w[0].1, 1.0);
        assert_eq!(w[3], (IVec2::new(1, 1), 0.0));
        // One open orthogonal neighbor connects the diagonal.
        let w = interpolation_weights(Vec2::new(0.9, 0.9), |d| d != IVec2::new(1, 0));
        assert!(w[3].1 > 0.0 && w[1].1 == 0.0);
    }

    #[test]
    fn darkvision_greys_out_with_darkness_and_distance() {
        assert_eq!(darkvision_saturation(1.0, 0.0), 1.0);
        assert_eq!(
            darkvision_saturation(1.0, 1.0),
            1.0,
            "continuous at the threshold"
        );
        assert_eq!(darkvision_saturation(0.0, 1.0), 0.0);
        assert!((darkvision_saturation(0.0, 0.0) - DARKVISION_COLOR).abs() < 1e-6);
        for i in 0..10 {
            let x = i as f32 / 10.0;
            let y = x + 0.1;
            assert!(darkvision_saturation(x, 0.5) < darkvision_saturation(y, 0.5));
            assert!(darkvision_saturation(0.5, x) > darkvision_saturation(0.5, y));
        }
        // The glow lights darkness, dimming toward the edge, and vanishes at the threshold.
        let near = darkvision(Vec3::ZERO, 0.0, 0.0);
        let edge = darkvision(Vec3::ZERO, 0.0, 1.0);
        assert!(near.rgb.x > edge.rgb.x && edge.rgb.x > 0.0);
        assert_eq!(darkvision(Vec3::ONE, 1.0, 0.5).rgb, Vec3::ONE);
    }

    #[test]
    fn exposure_has_a_floor_and_a_soft_overbright() {
        assert_eq!(exposure(0.0, 0.1), 0.1);
        assert_eq!(exposure(-1.0, 0.1), 0.1);
        assert_eq!(exposure(1.0, 0.1), 1.0);
        assert!((exposure(0.5, 0.1) - 0.55).abs() < 1e-6);
        let mut last = 1.0;
        for l in [1.1, 1.5, 2.0, 4.0, 100.0] {
            let e = exposure(l, 0.1);
            assert!(e > last && e < 1.0 + OVERBRIGHT);
            last = e;
        }
    }

    #[test]
    fn shading_tints_and_desaturates() {
        let grey = Vec3::splat(0.5);
        let full = CellLight {
            rgb: Vec3::ONE,
            ..CellLight::DARK
        };
        assert_eq!(shade(grey, full, 0.1), grey);
        let red = shade(
            grey,
            CellLight {
                rgb: Vec3::new(1.0, 0.0, 0.0),
                ..full
            },
            0.1,
        );
        assert!(red.x > red.y && red.y == red.z);
        // Overbright light brightens a little and glows faintly in its own color, even on black.
        let warm = CellLight {
            rgb: Vec3::new(3.0, 2.0, 1.0),
            ..full
        };
        let lit = shade(grey, warm, 0.1);
        assert!(lit.x > lit.y && lit.y > lit.z && lit.z > 0.5 && lit.z < 0.5 + GLOW);
        let glow = shade(Vec3::ZERO, warm, 0.1);
        assert!(
            glow.x < GLOW && (glow / glow.x - warm.rgb / 3.0).length() < 1e-6,
            "same hue"
        );
        assert_eq!(
            shade(Vec3::ZERO, full, 0.1),
            Vec3::ZERO,
            "no glow at full light"
        );
        let mono = shade(
            Vec3::new(0.8, 0.2, 0.1),
            CellLight {
                saturation: 0.0,
                ..full
            },
            0.1,
        );
        assert!(mono.x == mono.y && mono.y == mono.z);
        let flicker = CellLight { gain: 1.1, ..full };
        assert!((shade(grey, flicker, 0.1) - grey * 1.1).length() < 1e-6);
    }

    fn showcase(actions: &[warlock_core::Action]) -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        for action in actions {
            world.apply(action);
        }
        world
    }

    fn whole_level(world: &World, time: Option<f32>) -> VisualLight {
        let view = world.perception();
        composite_level(&view, &Shadows::compute(&view), time)
    }

    fn composite_level(
        view: &PerceptionView<'_>,
        shadows: &Shadows,
        time: Option<f32>,
    ) -> VisualLight {
        let mut light = VisualLight::default();
        let size = UVec2::new(view.width() as u32, view.height() as u32);
        light.composite(view, shadows, IVec2::ZERO, size, time);
        light
    }

    fn center(p: Pos) -> Vec2 {
        Vec2::new(p.x as f32 + 0.5, p.y as f32 + 0.5)
    }

    #[test]
    fn composited_light_is_the_perceived_light() {
        // Without creature shadows, which are visual only.
        let world = showcase(&[]);
        let view = world.perception();
        let light = composite_level(&view, &Shadows::default(), Some(1.7));
        assert!(light.dirty);
        let mut lit = 0;
        for y in 0..view.height() {
            for x in 0..view.width() {
                let p = Pos::new(x, y);
                let cell = light.sample(Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
                match view.light(p) {
                    Some(seen) => {
                        let cell = cell.expect("visible cells are lit");
                        let expected = Vec3::from_array(seen.rgb.map(f32::from)) / 255.0;
                        assert!((cell.rgb - expected).abs().max_element() < 1e-4, "{p}");
                        lit += 1;
                    }
                    None => assert_eq!(cell, None, "{p}"),
                }
            }
        }
        assert!(lit > 100);
    }

    #[test]
    fn flicker_changes_gain_over_time_but_not_reduced() {
        let world = showcase(&[]);
        let a = whole_level(&world, Some(1.0));
        let b = whole_level(&world, Some(2.3));
        assert_eq!(a.flags, b.flags);
        let gains = |l: &VisualLight| l.cells.iter().map(|c| c.gain).collect::<Vec<_>>();
        assert_ne!(gains(&a), gains(&b), "torches flicker");
        assert!(a.cells.iter().all(|c| (0.9..=1.1).contains(&c.gain)));
        let mut steady = whole_level(&world, None);
        assert!(steady.cells.iter().all(|c| c.gain == 1.0));
        steady.dirty = false;
        let view = world.perception();
        let size = steady.size;
        steady.composite(&view, &Shadows::compute(&view), IVec2::ZERO, size, None);
        assert!(!steady.dirty, "unchanged light is not uploaded again");
    }

    #[test]
    fn creatures_shade_the_light_behind_them() {
        // The player in the start room, between its torches: cells right behind the player (seen
        // from a torch) get less light, nothing gets more, and the player's own cell keeps it all.
        let world = showcase(&[]);
        let view = world.perception();
        let player = view.viewer();
        let shaded = whole_level(&world, None);
        let plain = composite_level(&view, &Shadows::default(), None);
        let mut darker = 0;
        for &p in view
            .light_sources()
            .iter()
            .flat_map(|s| &s.contributions)
            .map(|(p, _)| p)
        {
            let (a, b) = (shaded.sample(center(p)), plain.sample(center(p)));
            let (a, b) = (a.expect("lit").rgb, b.expect("lit").rgb);
            assert!(a.cmple(b + 1e-6).all(), "{p}: {a} > {b}");
            if a.element_sum() < b.element_sum() - 1e-4 {
                assert!(
                    (p.dist2(player) as f32) < REACH * REACH,
                    "{p} is near the player"
                );
                assert!(
                    a.element_sum() >= KEEP * b.element_sum() - 1e-4,
                    "never black: {p}"
                );
                darker += 1;
            }
        }
        assert!(darker > 0, "the player casts shadows");
        assert_eq!(shaded.sample(center(player)), plain.sample(center(player)));
    }

    #[test]
    fn light_does_not_bleed_into_walls_or_out_of_view() {
        let world = showcase(&[]);
        let light = whole_level(&world, None);
        let view = world.perception();
        // The start room's west wall at (0, 15) is lit flat by its own face light; the floor next
        // to it does not take the wall's light, and the unseen corridor beyond the room is dark.
        let wall = light.sample(Vec2::new(0.5, 15.5)).expect("wall in view");
        assert_eq!(light.sample(Vec2::new(0.99, 15.01)), Some(wall));
        let floor = light.sample(Vec2::new(1.5, 15.5)).expect("floor in view");
        assert_eq!(light.sample(Vec2::new(1.01, 15.5)), Some(floor));
        assert!(!view.is_visible(Pos::new(5, 10)));
        assert_eq!(light.sample(Vec2::new(5.5, 10.5)), None);
    }

    /// In the darkness chamber with darkvision and no lantern, at (37, 7).
    fn darkness_chamber() -> World {
        let actions = format!(
            "{} {} {} vision:darkvision",
            "move:N ".repeat(6),
            "move:E ".repeat(30),
            "move:N ".repeat(4)
        );
        let actions = warlock_core::action::parse_actions(&actions).expect("actions parse");
        showcase(&actions)
    }

    #[test]
    fn darkvision_fades_cells_seen_in_the_dark() {
        // The cells around the player are seen only through darkvision, greyer and dimmer toward
        // the edge of the radius.
        let world = darkness_chamber();
        let view = world.perception();
        assert_eq!(view.viewer(), Pos::new(37, 7));
        let light = whole_level(&world, None);
        let at = |p: Pos| {
            let seen = view.light(p).expect("in view");
            assert!(seen.darkvision, "{p} is seen through darkvision");
            light
                .sample(Vec2::new(p.x as f32 + 0.5, p.y as f32 + 0.5))
                .expect("darkvision cells are lit")
        };
        let (near, far) = (at(Pos::new(36, 7)), at(Pos::new(33, 7)));
        assert!(near.saturation < 1.0 && far.saturation < near.saturation);
        assert!(far.rgb.min_element() > 0.0, "the glow lights darkness");
        assert!(far.rgb.x < near.rgb.x, "and fades with distance");
    }

    #[test]
    fn shadows_darken_cells_seen_through_darkvision() {
        // Darkvision's glow follows the perceived light, not the shadows, so shading the faint
        // fungus light around the player never makes a cell brighter.
        let world = darkness_chamber();
        let view = world.perception();
        let everywhere = Shadows::from_cells(view.light_sources().iter().map(|s| {
            let cells = s.contributions.iter().map(|(p, _)| (*p, KEEP)).collect();
            (s.key, cells)
        }));
        let plain = composite_level(&view, &Shadows::default(), None);
        let shaded = composite_level(&view, &everywhere, None);
        let mut darker = 0;
        for &(p, _) in view.light_sources().iter().flat_map(|s| &s.contributions) {
            if !view.light(p).is_some_and(|l| l.darkvision) {
                continue;
            }
            let a = shaded.sample(center(p)).expect("lit").rgb;
            let b = plain.sample(center(p)).expect("lit").rgb;
            assert!(a.cmple(b + 1e-6).all(), "{p}: {a} > {b}");
            darker += usize::from(a.element_sum() < b.element_sum() - 1e-6);
        }
        assert!(darker > 0);
    }
}
