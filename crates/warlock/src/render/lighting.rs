//! Visual lighting (spec §4.4): the light the map is drawn with, derived every frame from the
//! perceived gameplay light plus visual-only modulation. Gameplay never reads any of it.
//!
//! [`VisualLight::composite`] covers the map grid's cells plus a one-cell margin ([`MARGIN`]). For
//! each visible cell it sums its base light (ambient + bounce, steady) and every source's
//! contribution, and folds in the presentation rules: UV light as a violet tint (perception
//! reports UV only to eyes that see it) and the [`darkvision`] fade. Each source is modulated by
//! its preset's stability ([`super::modulation`]: keyed by the source, driven by [`VisualTime`];
//! steady with the `flicker` effect off): its hue tints its contributions here, exactly,
//! and its intensity becomes a per-cell brightness gain, the mean of the intensity of the sources
//! lighting the cell weighted by their drawn light, applied after shading so it reads the same in
//! dim and in overbright light. The result is one [`CellLight`] per cell, uploaded as two
//! textures aligned with the map grid:
//! - `light` (`Rgba32Float`): red, green, and blue light (1.0 = fully lit, more = overbright)
//!   and color saturation;
//! - `light_flags` (`Rgba8Uint`): [`LIT`] on visible cells, [`OPAQUE`] on walls, and
//!   [`OCCUPIED`] on cells with a glyph of their own (red, [`cell_flags`]; the spell circles'
//!   shader reads it too), and the intensity gain (green, see [`gain_byte`]).
//!
//! Transient lights ([`Transients`], visual only too) add light that is in no snapshot: a burning
//! missile lights its surroundings as it flies (point light falloff from its fractional
//! position, over the cells in its line of sight), and the light of a burning missile that has
//! landed in the rules but is still flying on screen is held back until it arrives.
//!
//! Spell effects ([`SpellLights`], spec §4.8 "Spell effects") light the scene the same way, in
//! any color, and may darken it: while a bright effect peaks, everything but its own light (base
//! light, every source, transient lights, darkvision's glow) is scaled down by
//! [`SpellLights::dim`], and so is the readability floor (here for free glyphs,
//! [`VisualLight::keep`]; in the shader for the map, which also dims the cells it draws unlit,
//! through the map's `dim` parameter). The effect's own light is left out of the dimming, so it
//! dominates the view.
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
use super::modulation::{Modulation, modulation};
use super::shadows::Shadows;
use super::{ConsoleHandles, ConsoleMaterial, Effects, VisualTime};
use crate::game::Game;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::shader::load_shader_library;
use warlock_core::Pos;
use warlock_core::content::{LightId, MAX_DARKEN};
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
/// Cell flag: a visible cell drawn with a glyph of its own over the floor: a creature, an object
/// or a chest, a known trap, or terrain that stands in the way (a wall torch). Lighting ignores
/// it; spell circles fade over it (`circles.wesl`), so their runes never cover another glyph.
pub const OCCUPIED: u8 = 4;

/// The flags of level cell `p` as the player perceives it: [`LIT`] if visible, with [`OPAQUE`]
/// for a wall and [`OCCUPIED`] for another glyph on it; nothing if unseen or remembered.
pub fn cell_flags(view: &PerceptionView<'_>, p: Pos) -> u8 {
    let CellView::Visible(t) = view.cell(p) else {
        return 0;
    };
    let terrain = view.content().terrain(t);
    if !terrain.transparent {
        return LIT | OPAQUE;
    }
    let occupied = !terrain.walkable
        || view.actor_at(p).is_some()
        || !view.objects(p).is_empty()
        || view.chest_at(p).is_some()
        || view.trap(p).is_some();
    if occupied { LIT | OCCUPIED } else { LIT }
}

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
pub const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

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
            .init_resource::<Transients>()
            .init_resource::<SpellLights>()
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
    /// Brightness multiplier of the lit color: the intensity modulation of the light.
    pub gain: f32,
}

impl CellLight {
    pub const DARK: CellLight = CellLight {
        rgb: Vec3::ZERO,
        saturation: 1.0,
        gain: 1.0,
    };
}

/// A light that moves between cells (a burning missile in flight): visual only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransientLight {
    /// Where it is, in level cells: a cell's center is its position plus one half.
    pub at: Vec2,
    /// Its light preset: color, radius, and stability (flicker).
    pub preset: LightId,
    /// Stable for its whole flight: seeds its flicker.
    pub key: u64,
}

/// Visual-only light for this frame beyond the perception snapshot (spec §4.4): moving lights,
/// and point sources of the snapshot held back (by position and preset) until what gives them
/// arrives on screen. Written by the missiles (`ui::missiles`) every frame.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Transients {
    pub lights: Vec<TransientLight>,
    pub held: Vec<(Pos, LightId)>,
}

/// A spell effect's light this frame: a point light of any color. Visual only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpellLight {
    /// Where it is, in level cells: a cell's center is its position plus one half.
    pub at: Vec2,
    /// Red, green, blue at full strength there: 1.0 = fully lit, more = overbright.
    pub rgb: Vec3,
    /// Reach in cells; it falls off like gameplay light.
    pub radius: i32,
}

/// Spell effects' light this frame and how much they darken everything else (spec §4.8 "Spell
/// effects"): `dim` 0 is none; 0.6 makes the rest of the view look 60 % darker. Written by the
/// spell effects (`ui::spells`) every frame.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct SpellLights {
    pub lights: Vec<SpellLight>,
    pub dim: f32,
    /// The cells the playing effects were seen on, sorted: open, so their light shines through
    /// them even where the player saw them only by that light (unknown, dark cells).
    pub open: Vec<Pos>,
}

/// The gamma darkening works in: `dim` is how much darker things look on screen (sRGB-like), so
/// the share of linear light kept is `(1 − dim)^DIM_GAMMA`.
const DIM_GAMMA: f32 = 2.2;

impl SpellLights {
    /// The share of every other light (linear) kept: `(1 − dim)^2.2`, `dim` at most
    /// [`MAX_DARKEN`].
    pub fn keep(&self) -> f32 {
        (1.0 - self.dim.clamp(0.0, MAX_DARKEN)).powf(DIM_GAMMA)
    }
}

/// Visual light for the map grid, recomposited every frame.
#[derive(Resource, Debug)]
pub struct VisualLight {
    /// Level position of the first cell: [`MARGIN`] cells up and left of the map grid's (0, 0).
    origin: IVec2,
    /// Size in cells.
    size: UVec2,
    /// Row-major; [`CellLight::DARK`] where not [`LIT`].
    cells: Vec<CellLight>,
    flags: Vec<u8>,
    /// The share of light other than spell effects' kept while they darken the view.
    keep: f32,
    /// Set when the light changed and needs uploading.
    dirty: bool,
}

impl Default for VisualLight {
    fn default() -> Self {
        Self {
            origin: IVec2::ZERO,
            size: UVec2::ZERO,
            cells: Vec::new(),
            flags: Vec::new(),
            keep: 1.0,
            dirty: false,
        }
    }
}

impl VisualLight {
    /// Recomputes the light of the `size` cells from level position `origin` on, with creature
    /// `shadows` (computed from the same `view`). `time` is the visual time for the sources'
    /// modulation, `None` for steady light (the `flicker` effect off).
    #[cfg(test)]
    pub fn composite(
        &mut self,
        view: &PerceptionView<'_>,
        shadows: &Shadows,
        origin: IVec2,
        size: UVec2,
        time: Option<f32>,
    ) {
        let none = Transients::default();
        let spells = SpellLights::default();
        self.composite_with(view, shadows, (&none, &spells), origin, size, time);
    }

    /// [`VisualLight::composite`] with this frame's transient lights (moving lights added, held
    /// sources left out) and spell effects' lights (added, and everything else dimmed by their
    /// darkening).
    pub fn composite_with(
        &mut self,
        view: &PerceptionView<'_>,
        shadows: &Shadows,
        (transients, spells): (&Transients, &SpellLights),
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

        // Base light on every visible cell, then each source. `sums` is the perceived light,
        // which darkvision measures; `drawn` is what is drawn: modulated in hue, less what
        // creature shadows take away (darkvision's glow would otherwise brighten shadows).
        // `gains` sums each source's intensity weighted by how much light it draws (x), and
        // those weights (y); the base light is steady.
        let mut flags = vec![0; n];
        let mut sums = vec![Vec4::ZERO; n];
        let mut drawn = vec![Vec4::ZERO; n];
        let mut gains = vec![Vec2::ZERO; n];
        for (i, flag) in flags.iter_mut().enumerate() {
            *flag = cell_flags(view, at(i));
            if *flag & LIT != 0 {
                let base = light_vec(view.base_light(at(i)).unwrap_or_default());
                sums[i] = base;
                drawn[i] = base;
                gains[i] = Vec2::splat(base.element_sum());
            }
        }
        for source in view.light_sources() {
            if let (Some(pos), Some(preset)) = (source.pos(), source.preset())
                && transients.held.contains(&(pos, preset))
            {
                continue;
            }
            // The sun and the moons have no preset: they are never modulated.
            let modulation = match (time, source.preset()) {
                (Some(time), Some(preset)) => modulation(source.key, content.light(preset), time),
                _ => Modulation::STEADY,
            };
            let shadows = shadows.of(source.key);
            for &(p, light) in &source.contributions {
                if let Some(i) = index(p)
                    && flags[i] & LIT != 0
                {
                    let light = light_vec(light);
                    let draw = modulation.color(light) * shadows.keep(p);
                    sums[i] += light;
                    drawn[i] += draw;
                    gains[i] += Vec2::new(modulation.gain, 1.0) * draw.element_sum();
                }
            }
        }

        for light in &transients.lights {
            let preset = content.light(light.preset);
            let modulation = match time {
                Some(time) => modulation(light.key, preset, time),
                None => Modulation::STEADY,
            };
            let color = light_vec(preset.color);
            for (p, share) in transient_cells(view, light.at, preset.radius, &[]) {
                if let Some(i) = index(p)
                    && flags[i] & LIT != 0
                {
                    // A shadowed room keeps only part of this light too, as of all gameplay light.
                    let kept = f32::from(view.light_kept(p).unwrap_or(100)) / 100.0;
                    let light = color * share * kept;
                    let draw = modulation.color(light);
                    sums[i] += light;
                    drawn[i] += draw;
                    gains[i] += Vec2::new(modulation.gain, 1.0) * draw.element_sum();
                }
            }
        }

        // Spell effects' light: steady, and never dimmed by their darkening. It is far brighter
        // than any lamp, so it falls off more softly (the square of the gameplay falloff), or its
        // reach would show as an edge.
        let mut bright = vec![Vec3::ZERO; n];
        for light in &spells.lights {
            for (p, share) in transient_cells(view, light.at, light.radius, &spells.open) {
                if let Some(i) = index(p)
                    && flags[i] & LIT != 0
                {
                    let light = light.rgb * share * share;
                    sums[i] += light.extend(0.0);
                    bright[i] += light;
                }
            }
        }
        let keep = spells.keep();

        let vision = view.vision();
        let reach2 = radius2(vision.darkvision).max(1) as f32;
        let threshold = f32::from(vision.threshold);
        let cells: Vec<_> = (0..n)
            .map(|i| {
                if flags[i] & LIT == 0 {
                    return CellLight::DARK;
                }
                let sum = sums[i];
                let rgb = drawn[i].truncate() + UV_TINT * drawn[i].w;
                // Darkened light weighs less; a spell's light counts as steady.
                let weight = gains[i] * keep + Vec2::splat(bright[i].element_sum());
                let gain = if weight.y > 0.0 {
                    weight.x / weight.y
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
                CellLight {
                    rgb: light.rgb * keep + bright[i],
                    gain,
                    ..light
                }
            })
            .collect();
        self.keep = keep;

        if (origin, size) != (self.origin, self.size) || cells != self.cells || flags != self.flags
        {
            *self = VisualLight {
                origin,
                size,
                cells,
                flags,
                keep,
                dirty: true,
            };
        }
    }

    /// The share of light (and of the readability floor) kept while spell effects darken the
    /// view; 1 when they do not. Free glyphs are lit with their floor scaled by it, as the map
    /// shader does.
    pub fn keep(&self) -> f32 {
        self.keep
    }

    /// Every cell's flags, row-major (what `light_flags` uploads).
    pub fn flags(&self) -> &[u8] {
        &self.flags
    }

    /// The flags of map grid cell `cell` (console coordinates; the margin is outside), as the
    /// shaders read them from `light_flags`; 0 beyond.
    pub fn console_flags(&self, cell: IVec2) -> u8 {
        self.flags_at(cell + MARGIN)
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

/// The cells a light at `at` (level cells, fractional) of `radius` reaches, each with its share
/// of the light: the gameplay falloff, `(R² − d²) / R²` with `R² = radius² + radius`, from the
/// light to the cell's center, over the cells it has a line of sight to through terrain the
/// player knows to be transparent (an opaque cell takes the light of its face, like a wall), or
/// that is among the `open` cells (sorted): cells the player saw a spell effect on, in the dark
/// too, which are open by the rules.
fn transient_cells(
    view: &PerceptionView<'_>,
    at: Vec2,
    radius: i32,
    open: &[Pos],
) -> Vec<(Pos, f32)> {
    let content = view.content();
    let home = Pos::new(at.x.floor() as i32, at.y.floor() as i32);
    let r2 = radius2(radius) as f32;
    let clear = |p: Pos| match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => content.terrain(t).transparent,
        CellView::Unknown => open.binary_search(&p).is_ok(),
    };
    let mut cells = Vec::new();
    for y in home.y - radius..=home.y + radius {
        for x in home.x - radius..=home.x + radius {
            let p = Pos::new(x, y);
            let d2 = (Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - at).length_squared();
            if d2 >= r2 || !home.line_between(p).all(clear) || (p != home && !clear(home)) {
                continue;
            }
            cells.push((p, (r2 - d2) / r2));
        }
    }
    cells
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

/// An intensity gain as stored in the light flags texture: 128 is 1, in steps of 1/256 (0.5 to
/// about 1.5). Decoded in `lighting.wesl`.
pub fn gain_byte(gain: f32) -> u8 {
    (128.0 + (gain - 1.0) * 256.0).round().clamp(0.0, 255.0) as u8
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
/// intensity gain. Mirrored in `lighting.wesl`.
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
    transients: Res<Transients>,
    spells: Res<SpellLights>,
    mut light: ResMut<VisualLight>,
) {
    let view = game.world.perception();
    let size = UVec2::new(grid.console.cols(), grid.console.rows()) + 2 * MARGIN as u32;
    let time = effects.flicker.then_some(visual.seconds);
    let origin = grid.level_origin - MARGIN;
    let extra = (&*transients, &*spells);
    light.composite_with(&view, &shadows, extra, origin, size, time);
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
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

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
        // Steady (no modulation) and without creature shadows, which are visual only. Modulated,
        // the hue drift keeps each cell's luminance.
        let world = showcase(&[]);
        let view = world.perception();
        let steady = composite_level(&view, &Shadows::default(), None);
        let modulated = composite_level(&view, &Shadows::default(), Some(1.7));
        assert!(steady.dirty);
        let luma = |c: Vec3| c.dot(LUMA);
        let mut lit = 0;
        for y in 0..view.height() {
            for x in 0..view.width() {
                let p = Pos::new(x, y);
                let cell = steady.sample(center(p));
                match view.light(p) {
                    Some(seen) => {
                        let cell = cell.expect("visible cells are lit");
                        let expected = Vec3::from_array(seen.rgb.map(f32::from)) / 255.0;
                        assert!((cell.rgb - expected).abs().max_element() < 1e-4, "{p}");
                        let turned = modulated.sample(center(p)).expect("lit").rgb;
                        assert!((luma(turned) - luma(expected)).abs() < 1e-3, "{p}");
                        lit += 1;
                    }
                    None => assert_eq!(cell, None, "{p}"),
                }
            }
        }
        assert!(lit > 100);
    }

    #[test]
    fn firelight_drifts_in_hue_and_intensity_but_not_reduced() {
        // The start room is lit by its sconces.
        let world = showcase(&[]);
        let frames = [0.3, 1.1, 2.4, 3.7].map(|t| whole_level(&world, Some(t)));
        let rgb = |l: &VisualLight| l.cells.iter().map(|c| c.rgb).collect::<Vec<_>>();
        let gains = |l: &VisualLight| l.cells.iter().map(|c| c.gain).collect::<Vec<_>>();
        for pair in frames.windows(2) {
            assert_eq!(pair[0].flags, pair[1].flags);
            assert_ne!(gains(&pair[0]), gains(&pair[1]), "torches flicker");
            assert_ne!(rgb(&pair[0]), rgb(&pair[1]), "and drift in hue");
        }
        for frame in &frames {
            assert!(frame.cells.iter().all(|c| (0.75..=1.25).contains(&c.gain)));
        }
        let mut steady = whole_level(&world, None);
        assert!(steady.cells.iter().all(|c| c.gain == 1.0));
        steady.dirty = false;
        let view = world.perception();
        let size = steady.size;
        steady.composite(&view, &Shadows::compute(&view), IVec2::ZERO, size, None);
        assert!(!steady.dirty, "unchanged light is not uploaded again");
    }

    #[test]
    fn magical_light_is_constant() {
        // In the light hall, cells lit only by stable sources (the crystals) and the steady base
        // light look the same at every moment; light from the corridor lamps varies.
        let actions = format!(
            "{} open:N move:N move:N {} open:N move:N move:N",
            "move:N ".repeat(3),
            "move:E ".repeat(6)
        );
        let actions = warlock_core::action::parse_actions(&actions).expect("actions parse");
        let world = showcase(&actions);
        let view = world.perception();
        let content = view.content();
        let (mut stable, mut varying) = (BTreeSet::new(), BTreeSet::new());
        for source in view.light_sources() {
            let cells = source.contributions.iter().map(|(p, _)| *p);
            if source
                .preset()
                .is_none_or(|id| content.light(id).stability.is_stable())
            {
                stable.extend(cells);
            } else {
                varying.extend(cells);
            }
        }
        let constant: Vec<Pos> = stable.difference(&varying).copied().collect();
        assert!(constant.len() > 50, "{}", constant.len());
        let steady = whole_level(&world, None);
        for t in [0.3, 1.1, 2.4, 3.7] {
            let frame = whole_level(&world, Some(t));
            for &p in &constant {
                assert_eq!(
                    frame.sample(center(p)),
                    steady.sample(center(p)),
                    "{p} at {t}"
                );
            }
        }
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
    fn a_spell_lights_its_surroundings_and_darkens_everything_else() {
        // The start room, lit by its sconces; a spell's light at (11, 13), darkening the rest.
        let world = showcase(&[]);
        let view = world.perception();
        let size = UVec2::new(view.width() as u32, view.height() as u32);
        let plain = composite_level(&view, &Shadows::default(), None);
        let spells = SpellLights {
            lights: vec![SpellLight {
                at: Vec2::new(11.5, 13.5),
                rgb: Vec3::new(4.0, 2.0, 0.5),
                radius: 4,
            }],
            dim: 0.6,
            open: Vec::new(),
        };
        let keep = spells.keep();
        assert!(
            (keep - 0.4f32.powf(2.2)).abs() < 1e-6,
            "60 % darker on screen"
        );
        let mut lit = VisualLight::default();
        let extra = (&Transients::default(), &spells);
        lit.composite_with(&view, &Shadows::default(), extra, IVec2::ZERO, size, None);
        assert_eq!(lit.keep(), keep);
        let at = |l: &VisualLight, x: f32, y: f32| l.sample(Vec2::new(x, y)).expect("lit").rgb;
        // Out of its reach, everything is darkened; at it, its own light dominates.
        let far = (at(&lit, 2.5, 18.5), at(&plain, 2.5, 18.5));
        assert!((far.0 - far.1 * keep).length() < 1e-4, "{far:?}");
        let near = (at(&lit, 11.5, 13.5), at(&plain, 11.5, 13.5));
        assert!(near.0.x > near.1.x * keep + 3.9, "{near:?}");
        assert!(near.0.x > near.1.x, "brighter than without it");
        // No darkening: nothing changes but where it shines.
        let mut undimmed = VisualLight::default();
        let calm = SpellLights {
            dim: 0.0,
            ..spells.clone()
        };
        let extra = (&Transients::default(), &calm);
        undimmed.composite_with(&view, &Shadows::default(), extra, IVec2::ZERO, size, None);
        assert_eq!(at(&undimmed, 2.5, 18.5), at(&plain, 2.5, 18.5));
        assert_eq!(undimmed.keep(), 1.0);
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

    // The per-cell mask the spell circles' shader reads (#786): visible, opaque, occupied.
    #[test]
    fn cells_are_flagged_visible_opaque_and_occupied() {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        let world = World::new(content, "circles", None).expect("world");
        let view = world.perception();
        let light = composite_level(&view, &Shadows::default(), None);
        let cases = [
            ((16, 6), LIT, "plain floor in view"),
            ((17, 6), LIT | OCCUPIED, "the player"),
            ((11, 4), LIT | OCCUPIED, "a kobold"),
            ((13, 4), LIT | OCCUPIED, "a wall torch"),
            ((17, 0), LIT | OPAQUE, "a wall"),
            (
                (3, 6),
                LIT | OCCUPIED,
                "the alcove's kobold, seen by its glow",
            ),
            ((5, 6), 0, "the dark corridor to it"),
            ((2, 2), 0, "the room behind the wall"),
        ];
        for ((x, y), flags, what) in cases {
            let p = Pos::new(x, y);
            assert_eq!(cell_flags(&view, p), flags, "{what}");
            assert_eq!(
                light.flags_at(IVec2::new(x, y)),
                flags,
                "{what}, composited"
            );
        }
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
