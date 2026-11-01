//! Spell circles (spec §4.8 "Spell circles", docs/design/buff-circles.md): a slowly turning
//! circle of thin rings, a regular polygon, and a rune at each corner about every actor the
//! player sees with a buff's or a hex's status.
//!
//! **The rule** ([`layout`]): the effects of one kind (buffs, or hexes) on one actor, in status
//! order, fill `n × ⌈3 / n⌉` slots (three for one effect, four for two, then `n`) at the corners
//! of a regular polygon inscribed in the innermost ring, their runes repeating in sequence
//! (`A A A`, `A B A B`, `A B C`, ...), with `min(n, 3)` rings; six effects at most show. Buffs are
//! whole (solid lines, clockwise), hexes broken (dashed, counter-clockwise); on an actor with both
//! the buffs' circle is the inner one and the hexes' the outer, a lone hex as compact as a lone
//! buff. A rune is drawn in its school's colour, the rings and sides in the first rune's.
//!
//! **The draw**: `ui/circles.rs` pushes a [`CircleInstance`] per circled actor each frame; here
//! they become one dynamic mesh of one quad each, every vertex carrying the instance as flat
//! attributes, drawn with [`CircleMaterial`] (`circles.wesl`: signed distances for the rings and
//! the polygon, a tap of the [`RuneAtlas`] per rune: its coverage, halo, and backdrop) in one draw
//! call, over the map grid and under the free-glyph layer. The school colours are a uniform,
//! palette-mapped. Each rune is turned to face the centre (`rune_turn`) and pulses (`pulse`);
//! the map's `light_flags` keep runes and lines to seen floor with no other glyph
//! on it, and [`crowding`] and [`under`] keep crowded circles from overlapping (#786).
//!
//! **Motion**: each kind's circle comes as a [`Shown`]: its figure, the figure it changes from
//! while an effect begins or ends and how far (`slot_angle`'s glide, the polygons' and rings'
//! cross-fade), its fade in or out, and its place between the inner and the outer radii
//! (`ui/circle_tracks.rs` keeps them; [`FADE`], [`GLIDE`]).

use super::rune_atlas::{self, RUNE_SIZE, RuneAtlas};
use crate::game::GameContent;
use bevy::asset::{AssetPath, RenderAssetUsages, embedded_asset, embedded_path};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, PrimitiveTopology, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};

/// Draw order: over the map grid (0), under the free-glyph layer (1).
pub const LAYER_Z: f32 = 0.5;

/// The most effects of one kind a circle shows (a seventh has no rune).
pub const MAX_EFFECTS: usize = 6;

/// The most rings a circle has: the distinct runes give the exact count beyond.
pub const MAX_RINGS: usize = 3;

/// The innermost ring's radius, in cell heights: just clear of the actor's glyph (the shader's;
/// the tests check the geometry).
#[allow(dead_code)]
pub const RING: f32 = 0.48;

/// From one ring to the next, in cell heights (the shader's; the tests check the geometry).
#[allow(dead_code)]
pub const RING_GAP: f32 = 0.1;

/// The runes' centres from the actor's, in cell heights: outside the rings.
pub const RUNE_RADIUS: f32 = 1.0;

/// The hexes' circle outside a buffs' one: its innermost ring and its runes, in cell heights.
#[allow(dead_code)]
pub const OUTER_RING: f32 = 1.35;
pub const OUTER_RUNE_RADIUS: f32 = 1.88;

/// Seconds a buffs' circle takes to turn once (clockwise), and a hexes' (counter-clockwise).
pub const BUFF_PERIOD: f32 = 60.0;
pub const HEX_PERIOD: f32 = 45.0;

/// Seconds a circle takes to appear, and to go (`motion` on).
pub const FADE: f32 = 0.5;

/// Seconds a figure takes to change when an effect begins or ends: the runes glide to their new
/// slots, the polygon blends from its old corner count, and a hexes' circle moves out or in.
pub const GLIDE: f32 = 0.4;

/// Seconds of one rune's pulse, and how far it swings either way: its opacity by a share of
/// itself, its size by a share of the rune square (`motion` on; still without it). The shader's
/// (the tests check them).
#[allow(dead_code)]
pub const PULSE_PERIOD: f32 = 3.6;
#[allow(dead_code)]
pub const PULSE_ALPHA: f32 = 0.12;
pub const PULSE_SIZE: f32 = 0.04;

/// How far a rune standing at `angle` on its circle (radians from the centre's right,
/// clockwise on screen) is turned, clockwise: tidally locked, its foot toward the centre, so
/// the rune at the top stands upright and the one at the bottom on its head. The shader's
/// `rune_turn`.
#[cfg(test)]
pub fn rune_turn(angle: f32) -> f32 {
    use std::f32::consts::{FRAC_PI_2, TAU};
    (angle + FRAC_PI_2).rem_euclid(TAU)
}

/// The phase (0 to 1) of the rune at slot `k` of a circle, a hexes' a half period on: the
/// golden ratio's steps, so neighbours never pulse together. The shader's `pulse_phase`.
#[cfg(test)]
pub fn pulse_phase(k: u32, hex: bool) -> f32 {
    let step = k as f32 * 0.618_034 + if hex { 0.5 } else { 0.0 };
    step - step.floor()
}

/// A rune's pulse at `time` with `phase`: -1 to 1, a sine of [`PULSE_PERIOD`]. The shader's
/// `pulse`, scaled there by [`PULSE_ALPHA`] and [`PULSE_SIZE`] (and by nothing without motion).
#[cfg(test)]
pub fn pulse(time: f32, phase: f32) -> f32 {
    (std::f32::consts::TAU * (time / PULSE_PERIOD + phase)).sin()
}

/// The extent (the farthest ink) of a circle `place` of the way from the inner circle's radii
/// to the outer's, in cell heights: its runes' outer edge.
pub fn extent(place: f32) -> f32 {
    lerp(RUNE_RADIUS, OUTER_RUNE_RADIUS, place) + RUNE_SIZE / 2.0
}

/// How far past its ink a circle's quad reaches, in cell heights: room for the runes' halo and
/// backdrop (the atlas's blur, three deviations) and their pulse.
pub fn quad_margin() -> f32 {
    RUNE_SIZE * (3.0 * rune_atlas::BACKDROP_SIGMA + PULSE_SIZE)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The ease of a glide or a blend, `t` 0 to 1 of its time: slow in and out (smoothstep).
pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Where slot `k` stands on a figure changing from `from` slots to `to`, `blend` (0 to 1) of
/// the way, in radians from the figure's first corner, clockwise: every slot's angle moves from
/// its old place to its new one, so a slot that is new comes out from under the first (a full
/// turn on) and one that goes slides back under it. The shader's `slot_angle` (the tests check
/// the path).
pub fn slot_angle(k: u8, from: u8, to: u8, blend: f32) -> f32 {
    use std::f32::consts::TAU;
    let at = |m: u8| TAU * f32::from(k) / f32::from(m.max(1));
    lerp(at(from), at(to), blend)
}

/// A circled actor's runes, by kind and slot: the buffs' six, then the hexes'.
pub const RUNES: usize = 2 * MAX_EFFECTS;

/// Where a rune gives way to an earlier circle's rune: wholly within [`CROWD_NEAR`] rune
/// squares of it (centre to centre), not at all from [`CROWD_FAR`].
pub const CROWD_NEAR: f32 = 0.8;
pub const CROWD_FAR: f32 = 1.3;

/// Where each rune of `c` stands this frame, in framebuffer pixels at cell height `h`, by kind
/// and slot ([`RUNES`]): `None` where there is none (or its circle is gone).
pub fn rune_places(c: &CircleInstance, h: f32) -> [Option<Vec2>; RUNES] {
    let mut out = [None; RUNES];
    for (kind, shown) in c.kinds.iter().enumerate() {
        let Some(s) = shown.filter(|s| s.fade > 0.0) else {
            continue;
        };
        let (to, was) = (s.ring.slots(), s.from.slots());
        let r = lerp(RUNE_RADIUS, OUTER_RUNE_RADIUS, s.place) * h;
        for k in 0..to.max(was).min(MAX_EFFECTS as u8) {
            let a = s.angle - std::f32::consts::FRAC_PI_2 + slot_angle(k, was, to, s.blend);
            out[kind * MAX_EFFECTS + usize::from(k)] = Some(c.center + r * Vec2::from_angle(a));
        }
    }
    out
}

/// How much of each rune of each circle shows where circles crowd (0 to 1, by [`RUNES`]): a
/// rune that comes near an earlier circle's rune gives way, fading out as it nears
/// ([`CROWD_NEAR`], [`CROWD_FAR`]), so two runes never lie over each other. Earlier is first in
/// `instances`: the player's circle, then the creatures' in the snapshot's order.
pub fn crowding(instances: &[CircleInstance], h: f32) -> Vec<[f32; RUNES]> {
    let places: Vec<_> = instances.iter().map(|c| rune_places(c, h)).collect();
    let (near, far) = (CROWD_NEAR * RUNE_SIZE * h, CROWD_FAR * RUNE_SIZE * h);
    let give = |d: f32| ease((d - near) / (far - near));
    // Circles whose runes cannot meet are passed over at once: a crowd is mostly far apart.
    let reach: Vec<f32> = instances.iter().map(|c| c.extent() * h).collect();
    (places.iter().enumerate())
        .map(|(i, mine)| {
            let mut show = [1.0; RUNES];
            for j in 0..i {
                let apart = instances[i].center.distance(instances[j].center);
                if apart > reach[i] + reach[j] + far {
                    continue;
                }
                for (r, p) in mine.iter().enumerate() {
                    let Some(p) = p else { continue };
                    for q in places[j].iter().flatten() {
                        show[r] = f32::min(show[r], give(p.distance(*q)));
                    }
                }
            }
            show
        })
        .collect()
}

/// How much of a circle shows over framebuffer position `frag`, as the shader's `show_at`:
/// none over a cell that is not seen now ([`super::lighting::LIT`]), is opaque, or is occupied
/// (but the actor's own cell `own`), all over clear seen floor, eased across a cell's edge over
/// [`SOFT`] either side. `origin` is the map grid's cell (0, 0) on screen, `cell` the cell size,
/// `flags` the map's flags of a grid cell.
pub fn show_at(
    frag: Vec2,
    (origin, cell): (Vec2, Vec2),
    own: IVec2,
    flags: &dyn Fn(IVec2) -> u8,
) -> f32 {
    use super::lighting::{LIT, OCCUPIED, OPAQUE};
    let show = |c: IVec2| {
        let f = flags(c);
        if f & (LIT | OPAQUE) != LIT || (f & OCCUPIED != 0 && c != own) {
            0.0
        } else {
            1.0
        }
    };
    let c = (frag - origin) / cell - Vec2::splat(0.5);
    let base = c.floor();
    let smooth = |t: f32| ease((t - (0.5 - SOFT)) / (2.0 * SOFT));
    let f = Vec2::new(smooth(c.x - base.x), smooth(c.y - base.y));
    let b = base.as_ivec2();
    let top = lerp(show(b), show(b + IVec2::X), f.x);
    let bottom = lerp(show(b + IVec2::Y), show(b + IVec2::ONE), f.x);
    lerp(top, bottom, f.y)
}

/// How much of a rune centred at `foot` shows by the cells under it: the least of [`show_at`] at
/// its centre and [`FOOTPRINT`] of its square `size` out along each axis, so a rune is shown or
/// faded whole, never cut in two.
pub fn rune_show(
    foot: Vec2,
    size: f32,
    grid: (Vec2, Vec2),
    own: IVec2,
    flags: &dyn Fn(IVec2) -> u8,
) -> f32 {
    let e = FOOTPRINT * size;
    [Vec2::ZERO, Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .iter()
        .map(|d| show_at(foot + *d * e, grid, own, flags))
        .fold(1.0, f32::min)
}

/// Half the width of the soft edge between cells, in cells, either side of the edge (the
/// shader's `SOFT`).
pub const SOFT: f32 = 0.22;

/// How far from a rune's centre its footprint is checked, in rune squares: the ink's bulk.
pub const FOOTPRINT: f32 = 0.3;

/// The outermost ring of `c`, in cell heights: how far its lines reach (0 for no circle).
pub fn line_reach(c: &CircleInstance) -> f32 {
    (c.kinds.iter().flatten())
        .filter(|s| s.fade > 0.0)
        .map(|s| {
            let rings = s.ring.rings().max(s.from.rings()).max(1);
            lerp(RING, OUTER_RING, s.place) + RING_GAP * f32::from(rings - 1)
        })
        .fold(0.0, f32::max)
}

/// For each circle, the nearest earlier circle whose lines cross its own: `[dx, dy, r, 0]`, its
/// centre from this one's and its outermost ring's radius, in pixels (`r` 0 for none). The
/// shader leaves out this circle's lines inside that ring, so where two circles meet the later
/// passes under the earlier (as [`crowding`] has its runes give way) and their rings never
/// tangle.
pub fn under(instances: &[CircleInstance], h: f32) -> Vec<[f32; 4]> {
    let reach: Vec<f32> = instances.iter().map(|c| line_reach(c) * h).collect();
    (instances.iter().enumerate())
        .map(|(i, c)| {
            (0..i)
                .filter(|&j| reach[j] > 0.0)
                .map(|j| (instances[j].center - c.center, reach[j]))
                .filter(|(d, r)| d.length() < r + reach[i] + 1.0)
                .min_by(|a, b| a.0.length().total_cmp(&b.0.length()))
                .map_or([0.0; 4], |(d, r)| [d.x, d.y, r, 0.0])
        })
        .collect()
}

/// [`crowding`]'s shares as bytes for the shader, four to a word: the buffs' slots 0-3, 4-5,
/// then the hexes' 0-3, 4-5 (255 shows a rune whole).
fn pack_crowding(show: &[f32; RUNES]) -> [u32; 4] {
    let byte = |r: usize| u32::from((show[r].clamp(0.0, 1.0) * 255.0).round() as u8);
    let word = |from: usize, n: usize| (0..n).fold(0, |w, i| w | byte(from + i) << (8 * i));
    [word(0, 4), word(4, 2), word(6, 4), word(10, 2)]
}

/// One rune on a circle: its atlas slot (its row in `data/runes.toml`) and its school (by
/// `School::ALL` position).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rune {
    pub slot: u8,
    pub school: u8,
}

/// The circle of one kind on one actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ring {
    /// The effects shown, in status order: their runes fill the slots in sequence.
    pub runes: [Rune; MAX_EFFECTS],
    /// How many of `runes` there are (1 to [`MAX_EFFECTS`]).
    pub effects: u8,
    /// A hexes' circle: broken lines, turning the other way.
    pub hex: bool,
    /// The outer circle of an actor with both kinds (the larger radii).
    pub outer: bool,
}

impl Ring {
    /// The polygon's corners, which are the rune slots: `n × ⌈3 / n⌉`.
    pub fn slots(&self) -> u8 {
        let n = self.effects.max(1);
        n * 3u8.div_ceil(n)
    }

    /// The rune at slot `k`: the effects' runes in sequence, repeating (the shader's `k % n`).
    #[cfg(test)]
    pub fn rune_at(&self, k: u8) -> Rune {
        self.runes[usize::from(k % self.effects.max(1))]
    }

    /// One ring per effect, up to [`MAX_RINGS`].
    pub fn rings(&self) -> u8 {
        self.effects.min(MAX_RINGS as u8)
    }

    /// The school of the rings and sides: the first rune's.
    pub fn line_school(&self) -> u8 {
        self.runes[0].school
    }

    /// Whether `other` draws the same figure (wherever it stands, inner or outer).
    pub fn same_figure(&self, other: &Ring) -> bool {
        (self.runes, self.effects, self.hex) == (other.runes, other.effects, other.hex)
    }

    /// The innermost ring's radius and the runes', in cell heights.
    #[cfg(test)]
    pub fn radii(&self) -> (f32, f32) {
        if self.outer {
            (OUTER_RING, OUTER_RUNE_RADIUS)
        } else {
            (RING, RUNE_RADIUS)
        }
    }
}

/// What is drawn about one actor: the buffs' circle, the hexes', both, or nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Circle {
    pub inner: Option<Ring>,
    pub outer: Option<Ring>,
}

impl Circle {
    pub fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    /// The buffs' circle and the hexes', by kind ([`BUFFS`], [`HEXES`]) rather than by place.
    pub fn kinds(&self) -> [Option<Ring>; 2] {
        match self.inner {
            Some(ring) if ring.hex => [None, Some(ring)],
            inner => [inner, self.outer],
        }
    }
}

/// The kinds of circle, as [`Circle::kinds`] and [`CircleInstance::kinds`] index them.
pub const BUFFS: usize = 0;
pub const HEXES: usize = 1;

/// The circle of an actor's runed effects, `(rune, hex)` in status order: the rule above.
pub fn layout(effects: impl IntoIterator<Item = (Rune, bool)>) -> Circle {
    let (mut buffs, mut hexes) = (Vec::new(), Vec::new());
    for (rune, hex) in effects {
        let kind = if hex { &mut hexes } else { &mut buffs };
        if kind.len() < MAX_EFFECTS {
            kind.push(rune);
        }
    }
    let ring = |runes: &[Rune], hex: bool, outer: bool| {
        let mut ring = Ring {
            runes: [Rune::default(); MAX_EFFECTS],
            effects: runes.len() as u8,
            hex,
            outer,
        };
        ring.runes[..runes.len()].copy_from_slice(runes);
        ring
    };
    match (buffs.is_empty(), hexes.is_empty()) {
        (true, true) => Circle::default(),
        (false, true) => Circle {
            inner: Some(ring(&buffs, false, false)),
            outer: None,
        },
        (true, false) => Circle {
            inner: Some(ring(&hexes, true, false)),
            outer: None,
        },
        (false, false) => Circle {
            inner: Some(ring(&buffs, false, false)),
            outer: Some(ring(&hexes, true, true)),
        },
    }
}

/// How far a circle has turned at `time`, in radians, clockwise on screen (y down): a buffs'
/// clockwise, a hexes' the other way; still without motion, the inner at 0 and the outer at a
/// half turn.
pub fn angle(ring: &Ring, time: f32, motion: bool) -> f32 {
    use std::f32::consts::{PI, TAU};
    if !motion {
        return if ring.outer { PI } else { 0.0 };
    }
    let turn = if ring.hex {
        -time / HEX_PERIOD
    } else {
        time / BUFF_PERIOD
    };
    (turn * TAU).rem_euclid(TAU)
}

/// One kind's circle as drawn this frame: its figure and how far along its fades and glides
/// are (`ui/circle_tracks.rs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub ring: Ring,
    /// The figure it changes from while an effect begins or ends; `ring` itself when settled.
    pub from: Ring,
    /// How far the change is, eased: 0 the `from` figure, 1 `ring`.
    pub blend: f32,
    /// 0 unseen to 1 whole: the circle appearing or going.
    pub fade: f32,
    /// 0 at the inner circle's radii to 1 at the outer's.
    pub place: f32,
    /// How far it has turned ([`angle`]).
    pub angle: f32,
}

impl Shown {
    /// `ring` whole and still in its place, as slice 1 drew every circle.
    pub fn settled(ring: Ring, angle: f32) -> Self {
        Shown {
            ring,
            from: ring,
            blend: 1.0,
            fade: 1.0,
            place: if ring.outer { 1.0 } else { 0.0 },
            angle,
        }
    }
}

/// One circled actor this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircleInstance {
    /// The actor's drawn centre, in framebuffer pixels.
    pub center: Vec2,
    /// The buffs' circle and the hexes' ([`BUFFS`], [`HEXES`]).
    pub kinds: [Option<Shown>; 2],
    /// A spell effect's darkening (`VisualLight::keep`): 1 none.
    pub keep: f32,
    /// The map grid cell the actor stands in (console coordinates): its own glyph never hides
    /// its circle (the shader's mask of occupied cells leaves it out).
    pub cell: IVec2,
    /// Half the size of the actor's glyph's ink, in pixels about `center`: no line crosses it.
    pub ink: Vec2,
}

impl CircleInstance {
    /// The farthest ink of either circle, in cell heights.
    pub fn extent(&self) -> f32 {
        (self.kinds.iter().flatten()).fold(0.0, |e, k| e.max(extent(k.place)))
    }
}

/// The circles to draw this frame, and where they may appear.
#[derive(Resource, Debug, Default)]
pub struct Circles {
    instances: Vec<CircleInstance>,
    /// Framebuffer rectangle outside of which nothing is drawn (the map viewport).
    pub clip: Rect,
    /// The cell height in pixels (the font size): every size is in cell heights.
    pub cell_height: f32,
    /// Framebuffer position of the map grid's cell (0, 0) and the cell size: where the shader
    /// finds a pixel's cell in the map's `light_flags` (`lighting.rs`).
    pub grid_origin: Vec2,
    pub cell: Vec2,
    /// Visual time, for the runes' pulse.
    pub time: f32,
    /// Whether the runes pulse (`motion`), and glow (not with reduced effects).
    pub pulse: bool,
    pub halo: bool,
}

impl Circles {
    pub fn push(&mut self, instance: CircleInstance) {
        if instance.kinds.iter().any(Option::is_some) {
            self.instances.push(instance);
        }
    }
}

/// The school colours, palette-mapped, linear, by `School::ALL` position.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct SchoolColors(pub [Vec4; 8]);

pub struct CirclesPlugin;

impl Plugin for CirclesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "circles.wesl");
        app.add_plugins(Material2dPlugin::<CircleMaterial>::default())
            .init_resource::<Circles>()
            .init_resource::<SchoolColors>()
            .add_systems(Startup, spawn_layer.after(super::setup))
            .add_systems(PreUpdate, clear)
            .add_systems(PostUpdate, build_mesh);
    }
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct CircleParams {
    pub clip_min: Vec2,
    pub clip_max: Vec2,
    /// The map grid's cell (0, 0) in framebuffer pixels, and the cell size.
    pub grid_origin: Vec2,
    pub cell_size: Vec2,
    /// A rune atlas slot's side in texels, its border included, and its padding.
    pub slot: u32,
    pub pad: u32,
    pub columns: u32,
    pub supersample: u32,
    pub rune_size: f32,
    /// The halo's strength (0 off, 1 on).
    pub halo: f32,
    pub schools: [Vec4; 8],
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CircleMaterial {
    #[uniform(0)]
    pub params: CircleParams,
    #[texture(1, sample_type = "float", filterable = true)]
    #[sampler(2)]
    pub runes: Handle<Image>,
    /// The map's per-cell flags (`lighting.rs`: visible, opaque, occupied), shared with the
    /// map's material.
    #[texture(3, sample_type = "u_int")]
    pub light_flags: Handle<Image>,
}

/// The instance's cell height and darkening, and its actor's cell (`[h, keep, x, y]`).
const ATTRIBUTE_SHAPE: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleShape", 0x5741_524c_0101, VertexFormat::Float32x4);
/// Half the size of the actor's glyph's ink, in pixels ([`CircleInstance::ink`]), the visual
/// time, and whether the runes pulse (`[w, h, time, 0 or 1]`): per vertex, not a uniform, so the
/// material never changes from frame to frame.
const ATTRIBUTE_INK: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleInk", 0x5741_524c_0107, VertexFormat::Float32x4);
/// How much of each rune shows ([`pack_crowding`]): by the cells under it ([`rune_show`]) and
/// where circles crowd ([`crowding`]).
const ATTRIBUTE_CROWD: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleCrowd", 0x5741_524c_0108, VertexFormat::Uint32x4);
/// The earlier circle this one passes under ([`under`]).
const ATTRIBUTE_UNDER: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleUnder", 0x5741_524c_0109, VertexFormat::Float32x4);
/// The buffs' and the hexes' circle: each `[head, runes 0-2, runes 3-5, the old figure's head]`
/// ([`pack`]).
const ATTRIBUTE_RINGS: [MeshVertexAttribute; 2] = [
    MeshVertexAttribute::new("CircleInner", 0x5741_524c_0102, VertexFormat::Uint32x4),
    MeshVertexAttribute::new("CircleOuter", 0x5741_524c_0103, VertexFormat::Uint32x4),
];
/// The old figures' runes: `[buffs' 0-2, 3-5, hexes' 0-2, 3-5]`.
const ATTRIBUTE_FROM: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleFrom", 0x5741_524c_0104, VertexFormat::Uint32x4);
/// Each circle's `[angle, fade, blend, place]` ([`Shown`]).
const ATTRIBUTE_MOTION: [MeshVertexAttribute; 2] = [
    MeshVertexAttribute::new(
        "CircleInnerMotion",
        0x5741_524c_0105,
        VertexFormat::Float32x4,
    ),
    MeshVertexAttribute::new(
        "CircleOuterMotion",
        0x5741_524c_0106,
        VertexFormat::Float32x4,
    ),
];

impl Material2d for CircleMaterial {
    fn vertex_shader() -> ShaderRef {
        shader()
    }

    fn fragment_shader() -> ShaderRef {
        shader()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            ATTRIBUTE_SHAPE.at_shader_location(2),
            ATTRIBUTE_RINGS[0].at_shader_location(3),
            ATTRIBUTE_RINGS[1].at_shader_location(4),
            ATTRIBUTE_FROM.at_shader_location(5),
            ATTRIBUTE_MOTION[0].at_shader_location(6),
            ATTRIBUTE_MOTION[1].at_shader_location(7),
            ATTRIBUTE_INK.at_shader_location(8),
            ATTRIBUTE_CROWD.at_shader_location(9),
            ATTRIBUTE_UNDER.at_shader_location(10),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

fn shader() -> ShaderRef {
    ShaderRef::Path(
        AssetPath::from_path_buf(embedded_path!("circles.wesl")).with_source("embedded"),
    )
}

/// A circle packed for the shader: `[head, runes 0-2, runes 3-5, 0]`, the head
/// `effects | hex << 4 | outer << 5 | line school << 8 | slots << 12 | rings << 16` (no effects:
/// no circle), each rune nine bits, `slot | school << 6`.
pub fn pack(ring: Option<&Ring>) -> [u32; 4] {
    let Some(ring) = ring else {
        return [0; 4];
    };
    let head = u32::from(ring.effects)
        | u32::from(ring.hex) << 4
        | u32::from(ring.outer) << 5
        | u32::from(ring.line_school()) << 8
        | u32::from(ring.slots()) << 12
        | u32::from(ring.rings()) << 16;
    let rune = |r: Rune| u32::from(r.slot & 63) | u32::from(r.school & 7) << 6;
    let three = |runes: &[Rune]| (0..3).fold(0, |w, i| w | rune(runes[i]) << (9 * i));
    [head, three(&ring.runes[..3]), three(&ring.runes[3..]), 0]
}

/// One kind's circle for the shader: its ring words (`[head, runes, runes, the old figure's
/// head]`), the old figure's two rune words, and `[angle, fade, blend, place]`. Nothing for no
/// circle: a zero head draws nothing.
fn pack_shown(shown: Option<&Shown>) -> ([u32; 4], [u32; 2], [f32; 4]) {
    let Some(s) = shown else {
        return ([0; 4], [0; 2], [0.0; 4]);
    };
    let [head, a, b, _] = pack(Some(&s.ring));
    let [from, fa, fb, _] = pack(Some(&s.from));
    (
        [head, a, b, from],
        [fa, fb],
        [s.angle, s.fade, s.blend, s.place],
    )
}

#[derive(Resource, Debug)]
struct LayerHandles {
    mesh: Handle<Mesh>,
    material: Handle<CircleMaterial>,
}

fn spawn_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CircleMaterial>>,
    mut images: ResMut<Assets<Image>>,
    content: Res<GameContent>,
    atlas: Res<super::atlas::GlyphAtlas>,
    consoles: Res<Assets<super::ConsoleMaterial>>,
    console_handles: Res<super::ConsoleHandles>,
) {
    let runes = RuneAtlas::new(
        &rune_atlas::font(),
        content.0.runes(),
        atlas.cell.y as f32,
        &mut images,
    );
    let light_flags = consoles
        .get(&console_handles.map)
        .map(|m| m.light_flags.clone())
        .expect("the map's material exists once the console is set up");
    let frame = Frame {
        h: 1.0,
        time: 0.0,
        pulse: false,
        grid: (Vec2::ZERO, Vec2::ONE),
    };
    let mesh = meshes.add(circle_mesh(&[], &frame, &|_| 0));
    let material = materials.add(CircleMaterial {
        params: CircleParams::default(),
        runes: runes.image.clone(),
        light_flags,
    });
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(0.0, 0.0, LAYER_Z),
        // Vertices are in framebuffer pixels, not world space, so culling would be wrong.
        NoFrustumCulling,
    ));
    commands.insert_resource(runes);
    commands.insert_resource(LayerHandles { mesh, material });
}

fn clear(mut circles: ResMut<Circles>) {
    circles.instances.clear();
}

/// Builds the layer's mesh from this frame's circles (only when they changed), and the rune
/// atlas again when the cell height changed.
#[allow(clippy::too_many_arguments)]
fn build_mesh(
    circles: Res<Circles>,
    colors: Res<SchoolColors>,
    content: Res<GameContent>,
    handles: Res<LayerHandles>,
    mut atlas: ResMut<RuneAtlas>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CircleMaterial>>,
    light: Res<super::lighting::VisualLight>,
    mut built: Local<Option<(Vec<CircleInstance>, [u32; 6], Vec<u8>)>>,
) {
    let h = circles.cell_height;
    if h > 0.0 && (h - atlas.cell_height).abs() > f32::EPSILON {
        *atlas = RuneAtlas::new(&rune_atlas::font(), content.0.runes(), h, &mut images);
    }
    let on = |b: bool| if b { 1.0 } else { 0.0 };
    let params = CircleParams {
        clip_min: circles.clip.min,
        clip_max: circles.clip.max,
        grid_origin: circles.grid_origin,
        cell_size: circles.cell,
        slot: atlas.slot,
        pad: atlas.pad,
        columns: rune_atlas::COLUMNS,
        supersample: rune_atlas::SUPERSAMPLE,
        rune_size: RUNE_SIZE,
        halo: on(circles.halo),
        schools: colors.0,
    };
    if let Some(material) = materials.get(&handles.material)
        && (material.params != params || material.runes != atlas.image)
        && let Some(mut material) = materials.get_mut(&handles.material)
    {
        material.params = params;
        material.runes = atlas.image.clone();
    }
    let frame = Frame {
        h,
        time: if circles.pulse { circles.time } else { 0.0 },
        pulse: circles.pulse,
        grid: (circles.grid_origin, circles.cell),
    };
    let (o, c) = frame.grid;
    let numbers = [h, frame.time, o.x, o.y, c.x, c.y].map(f32::to_bits);
    // The runes' masks read the map's flags: a changed view redraws the circles too.
    let flags = if circles.instances.is_empty() {
        Vec::new()
    } else {
        light.flags().to_vec()
    };
    let key = (circles.instances.clone(), numbers, flags);
    if built.as_ref() == Some(&key) {
        return;
    }
    if let Some(mut mesh) = meshes.get_mut(&handles.mesh) {
        *mesh = circle_mesh(&circles.instances, &frame, &|cell| {
            light.console_flags(cell)
        });
    }
    *built = Some(key);
}

/// What every circle of a frame shares: the cell height, the visual time and whether the runes
/// pulse, and the map grid's place on screen (its cell (0, 0) and the cell size).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub h: f32,
    pub time: f32,
    pub pulse: bool,
    pub grid: (Vec2, Vec2),
}

/// One quad per circle, `2 × extent` square about its centre (and a pixel more). An empty
/// layer gets a single quad with no circle, so the mesh is never empty. `flags` gives the map's
/// flags of a grid cell, for the runes' masks.
fn circle_mesh(instances: &[CircleInstance], frame: &Frame, flags: &dyn Fn(IVec2) -> u8) -> Mesh {
    let h = frame.h;
    const NONE: CircleInstance = CircleInstance {
        center: Vec2::ZERO,
        kinds: [None; 2],
        keep: 1.0,
        cell: IVec2::ZERO,
        ink: Vec2::ZERO,
    };
    let instances = if instances.is_empty() {
        &[NONE][..]
    } else {
        instances
    };
    let n = instances.len();
    let v = n * 4;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(v);
    let mut locals: Vec<[f32; 2]> = Vec::with_capacity(v);
    let mut shapes: Vec<[f32; 4]> = Vec::with_capacity(v);
    let mut inks: Vec<[f32; 4]> = Vec::with_capacity(v);
    let mut crowds: Vec<[u32; 4]> = Vec::with_capacity(v);
    let crowding = crowding(instances, h);
    let under = under(instances, h);
    let mut unders: Vec<[f32; 4]> = Vec::with_capacity(v);
    let mut from: Vec<[u32; 4]> = Vec::with_capacity(v);
    let mut rings: [Vec<[u32; 4]>; 2] = [Vec::with_capacity(v), Vec::with_capacity(v)];
    let mut motions: [Vec<[f32; 4]>; 2] = [Vec::with_capacity(v), Vec::with_capacity(v)];
    let mut indices = Vec::with_capacity(n * 6);
    for (i, c) in instances.iter().enumerate() {
        let half = (c.extent() + quad_margin()) * h + 1.0;
        let [buffs, hexes] = c.kinds.map(|k| pack_shown(k.as_ref()));
        let olds = [buffs.1[0], buffs.1[1], hexes.1[0], hexes.1[1]];
        // Each rune shows by the cells under it, and gives way to an earlier circle's.
        let mut show = crowding[i];
        for (r, place) in rune_places(c, h).iter().enumerate() {
            if let Some(p) = place {
                show[r] *= rune_show(*p, RUNE_SIZE * h, frame.grid, c.cell, flags);
            }
        }
        let crowd = pack_crowding(&show);
        let pulse = if frame.pulse { 1.0 } else { 0.0 };
        for corner in [
            Vec2::new(-half, -half),
            Vec2::new(half, -half),
            Vec2::new(half, half),
            Vec2::new(-half, half),
        ] {
            let at = c.center + corner;
            positions.push([at.x, at.y, 0.0]);
            locals.push(corner.to_array());
            shapes.push([h, c.keep, c.cell.x as f32, c.cell.y as f32]);
            inks.push([c.ink.x, c.ink.y, frame.time, pulse]);
            crowds.push(crowd);
            unders.push(under[i]);
            from.push(olds);
            for (k, kind) in [buffs, hexes].iter().enumerate() {
                rings[k].push(kind.0);
                motions[k].push(kind.2);
            }
        }
        let base = (i * 4) as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let ([buff_rings, hex_rings], [buff_motion, hex_motion]) = (rings, motions);
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, locals)
    .with_inserted_attribute(ATTRIBUTE_SHAPE, shapes)
    .with_inserted_attribute(ATTRIBUTE_INK, inks)
    .with_inserted_attribute(ATTRIBUTE_CROWD, crowds)
    .with_inserted_attribute(ATTRIBUTE_UNDER, unders)
    .with_inserted_attribute(ATTRIBUTE_RINGS[0], buff_rings)
    .with_inserted_attribute(ATTRIBUTE_RINGS[1], hex_rings)
    .with_inserted_attribute(ATTRIBUTE_FROM, from)
    .with_inserted_attribute(ATTRIBUTE_MOTION[0], buff_motion)
    .with_inserted_attribute(ATTRIBUTE_MOTION[1], hex_motion)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rune `slot` of school `school`.
    fn r(slot: u8, school: u8) -> Rune {
        Rune { slot, school }
    }

    fn buffs(slots: &[u8]) -> Circle {
        layout(slots.iter().map(|&s| (r(s, s % 7), false)))
    }

    /// The runes at the slots of `ring`, in slot order.
    fn sequence(ring: &Ring) -> Vec<u8> {
        (0..ring.slots()).map(|k| ring.rune_at(k).slot).collect()
    }

    #[test]
    fn nothing_runed_draws_nothing() {
        assert!(layout([]).is_empty());
    }

    // §4 rule 1 and 2, and the sketch of one buff (Light): never a lone rune, a triangle of it.
    #[test]
    fn one_effect_is_a_triangle_of_its_rune_on_one_ring() {
        let ring = buffs(&[0]).inner.unwrap();
        assert_eq!(ring.slots(), 3);
        assert_eq!(sequence(&ring), [0, 0, 0]);
        assert_eq!(ring.rings(), 1);
        assert!(!ring.hex && !ring.outer);
    }

    // The sketch of two buffs (Light and Blessing): a square A B A B on two rings.
    #[test]
    fn two_effects_are_a_square_alternating_on_two_rings() {
        let ring = buffs(&[0, 3]).inner.unwrap();
        assert_eq!(ring.slots(), 4);
        assert_eq!(sequence(&ring), [0, 3, 0, 3]);
        assert_eq!(ring.rings(), 2);
    }

    // Three buffs (Blessing, Haste, Barkskin): a triangle A B C; four, a square A B C D.
    #[test]
    fn three_to_six_effects_are_their_polygon_on_three_rings() {
        for (n, rings) in [(3, 3), (4, 3), (5, 3), (6, 3)] {
            let slots: Vec<u8> = (0..n).collect();
            let ring = buffs(&slots).inner.unwrap();
            assert_eq!(ring.slots(), n, "{n} effects");
            assert_eq!(sequence(&ring), slots, "every rune once");
            assert_eq!(ring.rings(), rings, "rings capped at three");
        }
    }

    // §4 rule 4: six show, a seventh and later (in status order) have no rune.
    #[test]
    fn a_seventh_effect_has_no_rune() {
        let ring = buffs(&[0, 1, 2, 3, 4, 5, 6, 7]).inner.unwrap();
        assert_eq!(ring.effects, 6);
        assert_eq!(sequence(&ring), [0, 1, 2, 3, 4, 5]);
    }

    // §4 rule 2: every rune the same number of times, evenly spaced, for every count.
    #[test]
    fn every_figure_has_the_polygons_full_symmetry() {
        for n in 1..=6u8 {
            let ring = buffs(&(0..n).collect::<Vec<_>>()).inner.unwrap();
            let seq = sequence(&ring);
            let m = seq.len();
            for shift in (0..m).filter(|s| s % usize::from(n) == 0) {
                let turned: Vec<u8> = (0..m).map(|k| seq[(k + shift) % m]).collect();
                assert_eq!(turned, seq, "{n}: turning by {shift} slots changes nothing");
            }
            for rune in 0..n {
                let times = seq.iter().filter(|&&s| s == rune).count();
                assert_eq!(times, m / usize::from(n), "{n}: rune {rune}");
            }
        }
    }

    // §4 rule 5: the runes are in the order given (status order), never sorted.
    #[test]
    fn runes_keep_the_order_given() {
        assert_eq!(sequence(&buffs(&[5, 1, 3]).inner.unwrap()), [5, 1, 3]);
    }

    // §5 and the sketch of one hex (Evil eye): broken, alone as compact as a buff.
    #[test]
    fn a_lone_hex_is_broken_and_compact() {
        let c = layout([(r(4, 5), true)]);
        let ring = c.inner.unwrap();
        assert!(ring.hex && !ring.outer);
        assert_eq!(ring.radii(), (RING, RUNE_RADIUS));
        assert_eq!(c.outer, None);
        assert_eq!(sequence(&ring), [4, 4, 4]);
    }

    // Two hexes on a sleeping, webbed kobold: a dashed square ネ 井 ネ 井 on two rings.
    #[test]
    fn two_hexes_are_a_dashed_square() {
        let ring = layout([(r(7, 5), true), (r(8, 4), true)]).inner.unwrap();
        assert!(ring.hex);
        assert_eq!(sequence(&ring), [7, 8, 7, 8]);
        assert_eq!(ring.rings(), 2);
        assert_eq!(ring.line_school(), 5, "the first rune's school");
    }

    // §5: both kinds, the buffs inside, the hexes outside at the larger radii.
    #[test]
    fn buffs_go_inside_and_hexes_outside() {
        let c = layout([(r(3, 2), false), (r(4, 5), true), (r(1, 4), false)]);
        let (inner, outer) = (c.inner.unwrap(), c.outer.unwrap());
        assert!(!inner.hex && outer.hex && outer.outer);
        assert_eq!(sequence(&inner), [3, 1, 3, 1]);
        assert_eq!(sequence(&outer), [4, 4, 4]);
        assert_eq!(outer.radii(), (OUTER_RING, OUTER_RUNE_RADIUS));
    }

    // §3: the sizes hold together: the third ring stays inside the runes' band, the outer
    // circle's rings clear the inner's runes, and the outer runes clear its third ring.
    #[test]
    fn the_rings_never_cross_the_runes() {
        let half_rune = RUNE_SIZE / 2.0;
        let third = RING + 2.0 * RING_GAP;
        assert!(third + 0.03 < RUNE_RADIUS - half_rune);
        assert!(extent(0.0) < OUTER_RING);
        assert!(OUTER_RING + 2.0 * RING_GAP + 0.03 < OUTER_RUNE_RADIUS - half_rune);
        assert!((extent(0.0) - 1.275).abs() < 1e-6);
        assert!((extent(1.0) - 2.155).abs() < 1e-6);
        let settled = |c: Circle| CircleInstance {
            center: Vec2::ZERO,
            kinds: c.kinds().map(|k| k.map(|ring| Shown::settled(ring, 0.0))),
            keep: 1.0,
            cell: IVec2::ZERO,
            ink: Vec2::ZERO,
        };
        let one = settled(layout([(r(0, 0), false)]));
        let both = settled(layout([(r(0, 0), false), (r(1, 1), true)]));
        let hex = settled(layout([(r(0, 0), true)]));
        assert_eq!(one.extent(), extent(0.0));
        assert_eq!(both.extent(), extent(1.0));
        assert_eq!(hex.extent(), extent(0.0), "a lone hex");
    }

    // The kinds: the buffs' circle first, the hexes' second, wherever each stands.
    #[test]
    fn circles_sort_by_kind() {
        let [b, h] = layout([(r(0, 0), false)]).kinds();
        assert!(b.is_some_and(|b| !b.hex) && h.is_none());
        let [b, h] = layout([(r(0, 0), true)]).kinds();
        assert!(b.is_none() && h.is_some_and(|h| h.hex && !h.outer));
        let [b, h] = layout([(r(0, 0), false), (r(1, 1), true)]).kinds();
        assert!(b.is_some_and(|b| !b.hex) && h.is_some_and(|h| h.hex && h.outer));
    }

    // §4 rule 5 and §11: a figure's change glides every slot from its old place to its new one.
    #[test]
    fn slots_glide_from_the_old_figure_to_the_new() {
        use std::f32::consts::TAU;
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        // Settled, a slot is where the figure's corner is, whatever it was before.
        for (from, to) in [(3, 4), (4, 3), (3, 6), (5, 5)] {
            for k in 0..to {
                let at = TAU * f32::from(k) / f32::from(to);
                assert!(close(slot_angle(k, from, to, 1.0), at), "{from}→{to}: {k}");
            }
        }
        // One effect to two (a triangle to a square): the first slot stays, the others close up,
        // and the new fourth comes out from under the first (a full turn on).
        assert!(close(slot_angle(0, 3, 4, 0.5), 0.0));
        assert!(close(slot_angle(1, 3, 4, 0.0), TAU / 3.0));
        assert!(close(
            slot_angle(1, 3, 4, 0.5),
            (TAU / 3.0 + TAU / 4.0) / 2.0
        ));
        assert!(close(slot_angle(3, 3, 4, 0.0), TAU));
        assert!(close(slot_angle(3, 3, 4, 1.0), 3.0 * TAU / 4.0));
        // And back: the fourth slides back under the first as it goes.
        assert!(close(slot_angle(3, 4, 3, 1.0), TAU));
        // The path is monotonic, never past either end.
        for k in 0..4 {
            let (a, b) = (slot_angle(k, 3, 4, 0.0), slot_angle(k, 3, 4, 1.0));
            let mut last = a;
            for i in 1..=10 {
                let now = slot_angle(k, 3, 4, i as f32 / 10.0);
                assert!(now <= last + 1e-6 && now >= b - 1e-6, "slot {k} step {i}");
                last = now;
            }
        }
    }

    #[test]
    fn the_ease_is_slow_in_and_out_and_clamped() {
        assert_eq!(ease(-1.0), 0.0);
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(0.5), 0.5);
        assert_eq!(ease(1.0), 1.0);
        assert_eq!(ease(2.0), 1.0);
        assert!(ease(0.1) < 0.1 && ease(0.9) > 0.9);
    }

    // Figures compare by their runes, not by where they stand.
    #[test]
    fn a_figure_is_the_same_inner_or_outer() {
        let lone = layout([(r(4, 5), true)]).kinds()[HEXES].unwrap();
        let caged = layout([(r(0, 0), false), (r(4, 5), true)]).kinds()[HEXES].unwrap();
        assert!(lone.same_figure(&caged) && lone != caged);
        let two = layout([(r(4, 5), true), (r(7, 5), true)]).kinds()[HEXES].unwrap();
        assert!(!lone.same_figure(&two));
    }

    // §5 and §11: buffs clockwise in 60 s, hexes the other way in 45 s; still without motion,
    // the outer circle at a half turn.
    #[test]
    fn circles_turn_their_own_ways_and_stand_still_without_motion() {
        use std::f32::consts::{PI, TAU};
        let c = layout([(r(0, 0), false), (r(1, 1), true)]);
        let (inner, outer) = (c.inner.unwrap(), c.outer.unwrap());
        assert!((angle(&inner, 15.0, true) - TAU / 4.0).abs() < 1e-5);
        assert!((angle(&outer, 11.25, true) - (TAU - TAU / 4.0)).abs() < 1e-4);
        assert!((angle(&inner, 1.0, true) - 6f32.to_radians()).abs() < 1e-5);
        assert_eq!(angle(&inner, 33.0, false), 0.0);
        assert_eq!(angle(&outer, 33.0, false), PI);
    }

    #[test]
    fn circles_pack_into_words() {
        let ring = layout([(r(8, 4), true), (r(63, 6), true)]).inner.unwrap();
        let [head, a, b, _] = pack(Some(&ring));
        assert_eq!(head, 2 | 1 << 4 | 4 << 8 | 4 << 12 | 2 << 16);
        assert_eq!(a & 0x1ff, 8 | 4 << 6);
        assert_eq!(a >> 9 & 0x1ff, 63 | 6 << 6);
        assert_eq!(b, 0);
        assert_eq!(pack(None), [0; 4]);
    }

    #[test]
    fn each_circle_is_one_quad_of_its_extent() {
        let settled = |c: Circle| c.kinds().map(|k| k.map(|ring| Shown::settled(ring, 0.0)));
        let one = CircleInstance {
            center: Vec2::new(100.0, 50.0),
            kinds: settled(buffs(&[0])),
            keep: 1.0,
            cell: IVec2::new(3, 4),
            ink: Vec2::new(4.0, 7.0),
        };
        let both = CircleInstance {
            kinds: settled(layout([(r(0, 0), false), (r(1, 1), true)])),
            ..one
        };
        let frame = Frame {
            h: 18.0,
            time: 2.5,
            pulse: true,
            grid: (Vec2::ZERO, Vec2::new(9.0, 18.0)),
        };
        let floor = |_: IVec2| super::super::lighting::LIT;
        let mesh = circle_mesh(&[one, both], &frame, &floor);
        assert_eq!(mesh.count_vertices(), 8);
        assert_eq!(mesh.indices().map(Indices::len), Some(12));
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(local)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("locals are Float32x2");
        };
        let half = |i: usize| local[i][0].abs();
        let quad = |c: &CircleInstance| (c.extent() + quad_margin()) * 18.0 + 1.0;
        assert!((half(0) - quad(&one)).abs() < 1e-4);
        assert!((half(4) - quad(&both)).abs() < 1e-4);
        // The halo and the backdrop fit: the quad reaches past the ink by their blur.
        assert!(quad_margin() > 3.0 * rune_atlas::BACKDROP_SIGMA * RUNE_SIZE);
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(shape)) =
            mesh.attribute(ATTRIBUTE_SHAPE)
        else {
            panic!("shapes are Float32x4");
        };
        assert_eq!(shape[0], [18.0, 1.0, 3.0, 4.0], "the actor's own cell");
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(ink)) = mesh.attribute(ATTRIBUTE_INK)
        else {
            panic!("inks are Float32x4");
        };
        assert_eq!(
            ink[3],
            [4.0, 7.0, 2.5, 1.0],
            "the glyph's ink and the pulse's time"
        );
        // Over open floor every rune of the lone triangle shows whole, and no other slot.
        let Some(bevy::mesh::VertexAttributeValues::Uint32x4(crowd)) =
            mesh.attribute(ATTRIBUTE_CROWD)
        else {
            panic!("crowds are Uint32x4");
        };
        assert_eq!(crowd[0], [0xffff_ffff, 0xffff, 0xffff_ffff, 0xffff]);
        // Over walls the triangle's three runes show not at all.
        let wall = |_: IVec2| super::super::lighting::LIT | super::super::lighting::OPAQUE;
        let walled = circle_mesh(&[one], &frame, &wall);
        let Some(bevy::mesh::VertexAttributeValues::Uint32x4(crowd)) =
            walled.attribute(ATTRIBUTE_CROWD)
        else {
            panic!("crowds are Uint32x4");
        };
        assert_eq!(
            crowd[0][0], 0xff00_0000,
            "slots 0 to 2 hidden, slot 3 unused"
        );
        let none = circle_mesh(&[], &frame, &floor);
        assert_eq!(none.count_vertices(), 4, "never empty");
    }

    // The mask a circle reads (the shader's `show_at`, the runes' on the CPU): clear seen floor
    // shows, a wall, an unseen cell, and another glyph do not (the actor's own cell does),
    // softly across the edge; a rune shows by the least clear of the cells under it.
    #[test]
    fn circles_show_only_over_clear_seen_floor() {
        use crate::render::lighting::{LIT, OCCUPIED, OPAQUE};
        // A row of cells 10 px wide from x = 0: floor, the actor (occupied, its own), floor,
        // another creature, floor, a wall, unseen.
        let row = [
            LIT,
            LIT | OCCUPIED,
            LIT,
            LIT | OCCUPIED,
            LIT,
            LIT | OPAQUE,
            0,
        ];
        let flags = move |c: IVec2| {
            usize::try_from(c.x)
                .ok()
                .and_then(|x| row.get(x))
                .copied()
                .unwrap_or(0)
                * u8::from(c.y == 0)
        };
        let grid = (Vec2::ZERO, Vec2::new(10.0, 18.0));
        let own = IVec2::new(1, 0);
        let mid = |x: usize| Vec2::new(x as f32 * 10.0 + 5.0, 9.0);
        let at = |p: Vec2| show_at(p, grid, own, &flags);
        assert_eq!(at(mid(0)), 1.0, "floor");
        assert_eq!(at(mid(1)), 1.0, "the actor's own cell");
        assert_eq!(at(mid(2)), 1.0, "floor");
        assert_eq!(at(mid(3)), 0.0, "another creature");
        assert_eq!(at(mid(5)), 0.0, "a wall");
        assert_eq!(at(mid(6)), 0.0, "unseen");
        assert_eq!(at(mid(8)), 0.0, "outside the view");
        // Soft: halfway at the edge between floor and the creature, all within SOFT of it.
        let edge = Vec2::new(30.0, 9.0);
        assert!((at(edge) - 0.5).abs() < 1e-5);
        assert_eq!(at(edge - Vec2::new(10.0 * SOFT + 0.01, 0.0)), 1.0);
        assert!(at(edge - Vec2::new(1.0, 0.0)) > 0.5);
        // A rune 6 px square in the middle of floor shows; nudged toward the creature it fades
        // as a whole before its centre gets there.
        assert_eq!(rune_show(mid(2), 6.0, grid, own, &flags), 1.0);
        let nudged = rune_show(mid(2) + Vec2::new(2.5, 0.0), 6.0, grid, own, &flags);
        assert!(nudged < 1.0 && at(mid(2) + Vec2::new(2.5, 0.0)) == 1.0);
        assert_eq!(
            rune_show(mid(4), 6.0, grid, own, &flags),
            1.0,
            "floor by a wall"
        );
    }

    fn at(center: Vec2, c: Circle) -> CircleInstance {
        CircleInstance {
            center,
            kinds: c.kinds().map(|k| k.map(|ring| Shown::settled(ring, 0.0))),
            keep: 1.0,
            cell: IVec2::ZERO,
            ink: Vec2::ZERO,
        }
    }

    // Where circles crowd, a later circle's rune gives way to an earlier's near it, and its
    // lines pass under the earlier's rings; apart, nothing gives way.
    #[test]
    fn crowded_circles_give_way_to_earlier_ones() {
        let h = 18.0;
        let one = buffs(&[0]);
        // The first rune of a settled circle at angle 0 stands straight up, a rune radius out.
        let places = rune_places(&at(Vec2::ZERO, one), h);
        assert!(places[0].unwrap().distance(Vec2::new(0.0, -h)) < 1e-3);
        assert_eq!(places.iter().flatten().count(), 3, "a triangle");
        // Two circles, the second's top rune exactly on the first's: it goes, the rest stay.
        let first = at(Vec2::ZERO, one);
        let second = at(Vec2::new(0.75f32.sqrt() * h, 1.5 * h), one);
        let wide = at(Vec2::new(10.0 * h, 0.0), one);
        let show = crowding(&[first, second, wide], h);
        assert_eq!(show[0], [1.0; RUNES], "the earliest never gives way");
        assert!(
            show[1][0] < 1e-3,
            "its top rune lies on the first's lower right one"
        );
        assert_eq!(show[1][1..], [1.0; RUNES - 1], "the rest are clear");
        assert_eq!(show[2], [1.0; RUNES], "far apart");
        let packed = pack_crowding(&show[1]);
        assert_eq!(packed[0] & 0xff, (show[1][0] * 255.0).round() as u32);
        assert_eq!(
            pack_crowding(&[1.0; RUNES]),
            [0xffff_ffff, 0xffff, 0xffff_ffff, 0xffff]
        );
        // Lines: the near one passes under the first, the far one under nothing.
        let near = at(Vec2::new(h, 0.0), one);
        let under = under(&[first, near, wide], h);
        assert_eq!(under[0], [0.0; 4], "the earliest is under nothing");
        assert_eq!(
            under[1],
            [-h, 0.0, RING * h, 0.0],
            "the first, by its outer ring"
        );
        assert_eq!(under[2], [0.0; 4]);
        assert!((line_reach(&at(Vec2::ZERO, buffs(&[0, 1, 2]))) - (RING + 0.2)).abs() < 1e-6);
    }

    // Tidal lock: every rune's foot points at the centre, the top one upright.
    #[test]
    fn runes_turn_their_feet_to_the_centre() {
        use std::f32::consts::{FRAC_PI_2, PI, TAU};
        assert!(
            rune_turn(-FRAC_PI_2).abs() < 1e-6,
            "the top rune is upright"
        );
        assert!(
            (rune_turn(FRAC_PI_2) - PI).abs() < 1e-6,
            "the bottom one on its head"
        );
        for i in 0..24 {
            let a = TAU * i as f32 / 24.0 - 1.0;
            // The rune's foot (down, +y) turned clockwise on screen (y down) by `rune_turn`.
            let foot = Vec2::from_angle(rune_turn(a)).rotate(Vec2::Y);
            let inward = -Vec2::from_angle(a);
            assert!(
                foot.distance(inward) < 1e-5,
                "slot at {a}: {foot} vs {inward}"
            );
            assert!((0.0..TAU).contains(&rune_turn(a)));
        }
    }

    // The pulse: a gentle sine of its period, each rune on its own phase, the same at the same
    // moment (a frozen capture).
    #[test]
    fn runes_pulse_gently_each_on_its_own_phase() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        assert!(close(pulse(0.0, 0.0), 0.0));
        assert!(close(pulse(PULSE_PERIOD / 4.0, 0.0), 1.0));
        assert!(
            close(pulse(1.3, 0.2), pulse(1.3 + PULSE_PERIOD, 0.2)),
            "periodic"
        );
        assert!(
            close(pulse(0.0, 0.25), pulse(PULSE_PERIOD / 4.0, 0.0)),
            "phase"
        );
        for t in 0..100 {
            assert!(pulse(t as f32 * 0.37, 0.4).abs() <= 1.0);
        }
        // Subtle: a tenth or so of the rune's opacity, a few hundredths of its size, slow.
        const { assert!(PULSE_ALPHA <= 0.15 && PULSE_SIZE <= 0.05 && PULSE_PERIOD >= 3.0) };
        // Neighbours, and the two circles' runes, are out of step.
        let phases: Vec<f32> = (0..6).map(|k| pulse_phase(k, false)).collect();
        for (k, p) in phases.iter().enumerate() {
            assert!((0.0..1.0).contains(p));
            if k > 0 {
                let d = (p - phases[k - 1]).abs();
                assert!(d.min(1.0 - d) > 0.2, "slots {k} and {}", k - 1);
            }
        }
        assert!(close(pulse_phase(0, true), 0.5));
    }

    // A settled circle's old figure is its own; a changing one carries the old runes and head.
    #[test]
    fn a_changing_figure_packs_its_old_one() {
        let new = buffs(&[0, 3]).inner.unwrap();
        let old = buffs(&[0]).inner.unwrap();
        let (rings, _, motion) = pack_shown(Some(&Shown::settled(new, 0.5)));
        assert_eq!(rings[3], pack(Some(&new))[0], "settled: its own head");
        assert_eq!(motion, [0.5, 1.0, 1.0, 0.0]);
        let changing = Shown {
            from: old,
            blend: 0.25,
            ..Shown::settled(new, 0.0)
        };
        let (rings, olds, motion) = pack_shown(Some(&changing));
        let [old_head, old_a, old_b, _] = pack(Some(&old));
        assert_eq!(rings[..3], pack(Some(&new))[..3]);
        assert_eq!((rings[3], olds), (old_head, [old_a, old_b]));
        assert_eq!(motion[2], 0.25);
        assert_eq!(pack_shown(None).0[0], 0, "nothing drawn");
    }
}
