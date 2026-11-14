//! Spell circles (spec §4.8 "Spell circles", docs/design/buff-circles.md): a slowly turning
//! circle of thin rings and a regular polygon about every actor the player sees with a buff's or
//! a hex's status, the actor's own glyph glowing and glinting within it.
//!
//! **The rule** ([`layout`]): the effects of one kind (buffs, or hexes) on one actor, in status
//! order, make a regular polygon of `n × ⌈3 / n⌉` corners (a triangle for one effect, a square for
//! two, then `n`) inscribed in the innermost ring, with `min(n, 3)` rings; six effects at most
//! count. Buffs are whole (solid lines, clockwise), hexes broken (dashed, counter-clockwise); on
//! an actor with both the buffs' circle is the inner one and the hexes' the outer, a lone hex as
//! compact as a lone buff. The rings and sides are drawn in the first effect's school's colour.
//!
//! **The draw**: `ui/circles.rs` pushes a [`CircleInstance`] per circled actor each frame; here
//! they become one dynamic mesh of one quad each, every vertex carrying the instance as flat
//! attributes, drawn with [`CircleMaterial`] (`circles.wesl`: signed distances for the rings and
//! the polygon, with a faint glow; the actor's glyph's halo from [`GlyphGlow`]) in one draw call,
//! over the map grid and under the free-glyph layer; and again with a second material over the
//! free-glyph layer ([`TOP_Z`]) for the glints on the actor's glyph. The school colours are a
//! uniform, palette-mapped. The map's `light_flags` keep the lines to seen floor with no other
//! glyph on it, and [`under`] keeps crowded circles from tangling (#786).
//!
//! **Motion**: each kind's circle comes as a [`Shown`]: its figure, the figure it changes from
//! while an effect begins or ends and how far (the polygons' and rings' cross-fade), its fade in
//! or out, and its place between the inner and the outer radii (`ui/circle_tracks.rs` keeps
//! them; [`FADE`], [`GLIDE`]).

use super::glyph_glow::{self, GlyphGlow};
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

/// Draw order of the glints on circled actors' glyphs: over the free-glyph layer (1), under the
/// post tint (1.5) and the UI console.
pub const TOP_Z: f32 = 1.2;

/// The most effects of one kind a circle counts (a seventh changes nothing).
pub const MAX_EFFECTS: usize = 6;

/// The most rings a circle has.
pub const MAX_RINGS: usize = 3;

/// The innermost ring's radius, in cell heights: just clear of the actor's glyph.
pub const RING: f32 = 0.48;

/// From one ring to the next, in cell heights.
pub const RING_GAP: f32 = 0.1;

/// The hexes' circle outside a buffs' one: its innermost ring, in cell heights, clear of the
/// inner circle's third ring and its glow.
pub const OUTER_RING: f32 = 0.84;

/// Seconds a buffs' circle takes to turn once (clockwise), and a hexes' (counter-clockwise).
pub const BUFF_PERIOD: f32 = 60.0;
pub const HEX_PERIOD: f32 = 45.0;

/// Seconds a circle takes to appear, and to go (`motion` on).
pub const FADE: f32 = 0.5;

/// Seconds a figure takes to change when an effect begins or ends: the polygon blends from its
/// old corner count, a ring comes or goes, and a hexes' circle moves out or in.
pub const GLIDE: f32 = 0.4;

/// The lines' glow's deviation, in cell heights (the shader's `LINE_HALO_SIGMA`).
pub const LINE_GLOW_SIGMA: f32 = 0.045;

/// Seconds between one glint of a glyph and its next, for its first and its second glint (the
/// glow atlas's blue and alpha), and how sharp a glint is: the share of its beat it shows grows
/// smaller as this grows (`motion` on; none without it). The shader's (the tests check them).
#[allow(dead_code)]
pub const GLINT_PERIODS: [f32; 2] = [5.3, 7.1];
#[allow(dead_code)]
pub const GLINT_SHARP: f32 = 24.0;

/// A glint's phase (0 to 1): of the actor's `seed` and the glint `g` (0 or 1), so two actors'
/// glyphs never glint in step. The shader's `glint_phase`.
#[cfg(test)]
pub fn glint_phase(seed: u32, g: u32) -> f32 {
    let x = (seed & 0xffff) as f32 * 0.137_036 + g as f32 * 0.5;
    x - x.floor()
}

/// A glint's brightness at `time` (0 to 1): the top of a sine of its period, sharpened, so it
/// shows for a moment and is gone. The shader's `glint`.
#[cfg(test)]
pub fn glint(time: f32, phase: f32, g: usize) -> f32 {
    let s = (std::f32::consts::TAU * (time / GLINT_PERIODS[g] + phase)).sin();
    s.max(0.0).powf(GLINT_SHARP)
}

/// The extent (the farthest line) of a circle `place` of the way from the inner circle's radii
/// to the outer's, in cell heights: its outermost ring with the most rings.
pub fn extent(place: f32) -> f32 {
    lerp(RING, OUTER_RING, place) + RING_GAP * (MAX_RINGS - 1) as f32
}

/// How far past its extent a circle's quad reaches, in cell heights: room for the lines' glow
/// (three deviations).
pub fn quad_margin() -> f32 {
    3.0 * LINE_GLOW_SIGMA + 0.02
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The ease of a glide or a blend, `t` 0 to 1 of its time: slow in and out (smoothstep).
pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

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
/// passes under the earlier and their rings never tangle. Earlier is first in `instances`: the
/// player's circle, then the creatures' in the snapshot's order.
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

/// One effect on a circle: which circled status it is (its row in `data/circles.toml`), and
/// its school (by `School::ALL` position).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mark {
    pub slot: u8,
    pub school: u8,
}

/// The circle of one kind on one actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ring {
    /// The effects shown, in status order.
    pub marks: [Mark; MAX_EFFECTS],
    /// How many of `marks` there are (1 to [`MAX_EFFECTS`]).
    pub effects: u8,
    /// A hexes' circle: broken lines, turning the other way.
    pub hex: bool,
    /// The outer circle of an actor with both kinds (the larger radii).
    pub outer: bool,
}

impl Ring {
    /// The polygon's corners: `n × ⌈3 / n⌉`.
    pub fn slots(&self) -> u8 {
        let n = self.effects.max(1);
        n * 3u8.div_ceil(n)
    }

    /// One ring per effect, up to [`MAX_RINGS`].
    pub fn rings(&self) -> u8 {
        self.effects.min(MAX_RINGS as u8)
    }

    /// The school of the rings and sides: the first effect's.
    pub fn line_school(&self) -> u8 {
        self.marks[0].school
    }

    /// Whether `other` draws the same figure (wherever it stands, inner or outer).
    pub fn same_figure(&self, other: &Ring) -> bool {
        (self.marks, self.effects, self.hex) == (other.marks, other.effects, other.hex)
    }

    /// The innermost ring's radius, in cell heights.
    #[cfg(test)]
    pub fn radius(&self) -> f32 {
        if self.outer { OUTER_RING } else { RING }
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

/// The circle of an actor's circled effects, `(mark, hex)` in status order: the rule above.
pub fn layout(effects: impl IntoIterator<Item = (Mark, bool)>) -> Circle {
    let (mut buffs, mut hexes) = (Vec::new(), Vec::new());
    for (mark, hex) in effects {
        let kind = if hex { &mut hexes } else { &mut buffs };
        if kind.len() < MAX_EFFECTS {
            kind.push(mark);
        }
    }
    let ring = |marks: &[Mark], hex: bool, outer: bool| {
        let mut ring = Ring {
            marks: [Mark::default(); MAX_EFFECTS],
            effects: marks.len() as u8,
            hex,
            outer,
        };
        ring.marks[..marks.len()].copy_from_slice(marks);
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
    /// `ring` whole and still in its place.
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
    /// The actor's drawn centre (its glyph's ink's), in framebuffer pixels.
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
    /// The actor's glyph, which glows and glints within its circle (`None`: it does not).
    pub glyph: Option<char>,
    /// A number of the actor's own (its id), for its glints' beats: two actors' never in step.
    pub seed: u32,
}

impl CircleInstance {
    /// The farthest line of either circle, in cell heights.
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
    /// The map font's size in pixels: the circled actors' glyphs are rasterised at it.
    pub glyph_px: f32,
    /// Visual time, for the glints and the shimmer.
    pub time: f32,
    /// Whether the glints and the shimmer move (`motion`), whether anything glows (not with
    /// reduced effects), and whether the glints bloom (the `glow` switch).
    pub motion: bool,
    pub halo: bool,
    pub bloom: bool,
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
    /// The glyph atlas's ([`GlyphGlow`]) slot side and padding in texels, slots per row, and its
    /// square (a cell height, rounded up) in pixels.
    pub glyph_slot: u32,
    pub glyph_pad: u32,
    pub columns: u32,
    pub glyph_square: f32,
    /// The glows' strength (0 off, 1 on): the lines', and the actor's glyph's; and the glints'.
    pub halo: f32,
    /// 1 with the `glow` switch on: the glints are overbright, so they bloom.
    pub bloom: f32,
    /// 1 for the pass over the free-glyph layer, which draws the actors' glyphs' glints alone;
    /// 0 for the circles under it.
    pub top: u32,
    pub schools: [Vec4; 8],
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CircleMaterial {
    #[uniform(0)]
    pub params: CircleParams,
    /// The map's per-cell flags (`lighting.rs`: visible, opaque, occupied), shared with the
    /// map's material.
    #[texture(1, sample_type = "u_int")]
    pub light_flags: Handle<Image>,
    /// The circled actors' glyphs ([`GlyphGlow`]).
    #[texture(2, sample_type = "float", filterable = true)]
    #[sampler(3)]
    pub glyphs: Handle<Image>,
}

/// The instance's cell height and darkening, and its actor's cell (`[h, keep, x, y]`).
const ATTRIBUTE_SHAPE: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleShape", 0x5741_524c_0101, VertexFormat::Float32x4);
/// Half the size of the actor's glyph's ink, in pixels ([`CircleInstance::ink`]), the visual
/// time, and whether things move (`[w, h, time, 0 or 1]`): per vertex, not a uniform, so the
/// material never changes from frame to frame.
const ATTRIBUTE_INK: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleInk", 0x5741_524c_0107, VertexFormat::Float32x4);
/// The earlier circle this one passes under ([`under`]), and the actor's glyph's slot and seed
/// ([`pack_glyph`]).
const ATTRIBUTE_UNDER: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleUnder", 0x5741_524c_0109, VertexFormat::Float32x4);
/// The buffs' and the hexes' circle: each `[head, the old figure's head]` ([`pack`]).
const ATTRIBUTE_RINGS: [MeshVertexAttribute; 2] = [
    MeshVertexAttribute::new("CircleInner", 0x5741_524c_0102, VertexFormat::Uint32x2),
    MeshVertexAttribute::new("CircleOuter", 0x5741_524c_0103, VertexFormat::Uint32x2),
];
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
            ATTRIBUTE_MOTION[0].at_shader_location(5),
            ATTRIBUTE_MOTION[1].at_shader_location(6),
            ATTRIBUTE_INK.at_shader_location(7),
            ATTRIBUTE_UNDER.at_shader_location(8),
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

/// A circle's head for the shader: `effects | hex << 4 | outer << 5 | line school << 8 | slots
/// << 12 | rings << 16` (no effects: no circle).
pub fn pack(ring: Option<&Ring>) -> u32 {
    let Some(ring) = ring else {
        return 0;
    };
    u32::from(ring.effects)
        | u32::from(ring.hex) << 4
        | u32::from(ring.outer) << 5
        | u32::from(ring.line_school()) << 8
        | u32::from(ring.slots()) << 12
        | u32::from(ring.rings()) << 16
}

/// One kind's circle for the shader: `[head, the old figure's head]` and `[angle, fade, blend,
/// place]`. Nothing for no circle: a zero head draws nothing.
fn pack_shown(shown: Option<&Shown>) -> ([u32; 2], [f32; 4]) {
    let Some(s) = shown else {
        return ([0; 2], [0.0; 4]);
    };
    (
        [pack(Some(&s.ring)), pack(Some(&s.from))],
        [s.angle, s.fade, s.blend, s.place],
    )
}

/// An actor's glyph's slot in the [`GlyphGlow`] atlas (`None`: no glow, [`NO_GLYPH`]) and its
/// seed's low sixteen bits as one `f32` (exact: under 2²⁴), `slot << 16 | seed`: the fourth of
/// [`ATTRIBUTE_UNDER`].
fn pack_glyph(slot: Option<u32>, seed: u32) -> f32 {
    let slot = slot.filter(|&s| s < NO_GLYPH).unwrap_or(NO_GLYPH);
    (slot << 16 | (seed & 0xffff)) as f32
}

/// The glyph slot that means none (the shader's).
const NO_GLYPH: u32 = 255;

/// The layer's one mesh and its two materials: the circles under the free-glyph layer, and the
/// actors' glyphs' glints over it ([`TOP_Z`]).
#[derive(Resource, Debug)]
struct LayerHandles {
    mesh: Handle<Mesh>,
    material: Handle<CircleMaterial>,
    top: Handle<CircleMaterial>,
}

fn spawn_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CircleMaterial>>,
    mut images: ResMut<Assets<Image>>,
    (atlas, font): (Res<super::atlas::GlyphAtlas>, Res<super::atlas::Font>),
    consoles: Res<Assets<super::ConsoleMaterial>>,
    console_handles: Res<super::ConsoleHandles>,
) {
    let h = atlas.cell.y as f32;
    let glyphs = GlyphGlow::new(&font.0, vec!['@'], atlas.px, h, &mut images);
    let light_flags = consoles
        .get(&console_handles.map)
        .map(|m| m.light_flags.clone())
        .expect("the map's material exists once the console is set up");
    let frame = Frame {
        h: 1.0,
        time: 0.0,
        motion: false,
    };
    let mesh = meshes.add(circle_mesh(&[], &frame, &|_| None));
    let new = |top: u32| CircleMaterial {
        params: CircleParams { top, ..default() },
        light_flags: light_flags.clone(),
        glyphs: glyphs.image.clone(),
    };
    let (material, top) = (materials.add(new(0)), materials.add(new(1)));
    for (material, z) in [(&material, LAYER_Z), (&top, TOP_Z)] {
        commands.spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, z),
            // Vertices are in framebuffer pixels, not world space, so culling would be wrong.
            NoFrustumCulling,
        ));
    }
    commands.insert_resource(glyphs);
    commands.insert_resource(LayerHandles {
        mesh,
        material,
        top,
    });
}

/// The glyphs `circles` needs glowing that `glow` lacks, with the ones it has (a new atlas is
/// due), or `None` when it holds them all.
fn glyphs_wanted(circles: &[CircleInstance], glow: &GlyphGlow) -> Option<Vec<char>> {
    let mut chars = glow.chars.clone();
    for c in circles.iter().filter_map(|c| c.glyph) {
        if !chars.contains(&c) {
            chars.push(c);
        }
    }
    // A full atlas stays as it is: a glyph beyond it only goes without its glow.
    (chars.len() > glow.chars.len() && glow.chars.len() < glyph_glow::MAX_GLYPHS).then_some(chars)
}

fn clear(mut circles: ResMut<Circles>) {
    circles.instances.clear();
}

/// Builds the layer's mesh from this frame's circles (only when they changed), and the glyph
/// atlas again when a new glyph is circled or the cell height changed.
fn build_mesh(
    circles: Res<Circles>,
    colors: Res<SchoolColors>,
    handles: Res<LayerHandles>,
    (mut glow, font): (ResMut<GlyphGlow>, Res<super::atlas::Font>),
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CircleMaterial>>,
    mut built: Local<Option<(Vec<CircleInstance>, [u32; 2])>>,
) {
    let (h, px) = (circles.cell_height, circles.glyph_px);
    if h > 0.0 && px > 0.0 {
        let resized = (h, px) != (glow.cell_height, glow.px);
        let wanted = glyphs_wanted(&circles.instances, &glow);
        if let Some(chars) = wanted.or_else(|| resized.then(|| glow.chars.clone())) {
            *glow = GlyphGlow::new(&font.0, chars, px, h, &mut images);
        }
    }
    let on = |b: bool| if b { 1.0 } else { 0.0 };
    let params = CircleParams {
        clip_min: circles.clip.min,
        clip_max: circles.clip.max,
        grid_origin: circles.grid_origin,
        cell_size: circles.cell,
        glyph_slot: glow.slot,
        glyph_pad: glow.pad,
        columns: glyph_glow::COLUMNS,
        glyph_square: glow.cell_height.ceil(),
        halo: on(circles.halo),
        bloom: on(circles.bloom),
        top: 0,
        schools: colors.0,
    };
    for (handle, top) in [(&handles.material, 0), (&handles.top, 1)] {
        let params = CircleParams { top, ..params };
        if let Some(material) = materials.get(handle)
            && (material.params != params || material.glyphs != glow.image)
            && let Some(mut material) = materials.get_mut(handle)
        {
            material.params = params;
            material.glyphs = glow.image.clone();
        }
    }
    let frame = Frame {
        h,
        time: if circles.motion { circles.time } else { 0.0 },
        motion: circles.motion,
    };
    let key = (circles.instances.clone(), [h, frame.time].map(f32::to_bits));
    if built.as_ref() == Some(&key) {
        return;
    }
    if let Some(mut mesh) = meshes.get_mut(&handles.mesh) {
        *mesh = circle_mesh(&circles.instances, &frame, &|c| glow.slot_of(c));
    }
    *built = Some(key);
}

/// What every circle of a frame shares: the cell height, the visual time, and whether things
/// move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub h: f32,
    pub time: f32,
    pub motion: bool,
}

/// One quad per circle, `2 × extent` square about its centre (and a pixel more). An empty
/// layer gets a single quad with no circle, so the mesh is never empty. `glyphs` gives an
/// actor's glyph's slot in the [`GlyphGlow`] atlas.
fn circle_mesh(
    instances: &[CircleInstance],
    frame: &Frame,
    glyphs: &dyn Fn(char) -> Option<u32>,
) -> Mesh {
    let h = frame.h;
    const NONE: CircleInstance = CircleInstance {
        center: Vec2::ZERO,
        kinds: [None; 2],
        keep: 1.0,
        cell: IVec2::ZERO,
        ink: Vec2::ZERO,
        glyph: None,
        seed: 0,
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
    let under = under(instances, h);
    let mut unders: Vec<[f32; 4]> = Vec::with_capacity(v);
    let mut rings: [Vec<[u32; 2]>; 2] = [Vec::with_capacity(v), Vec::with_capacity(v)];
    let mut motions: [Vec<[f32; 4]>; 2] = [Vec::with_capacity(v), Vec::with_capacity(v)];
    let mut indices = Vec::with_capacity(n * 6);
    let motion = if frame.motion { 1.0 } else { 0.0 };
    for (i, c) in instances.iter().enumerate() {
        let half = (c.extent() + quad_margin()) * h + 1.0;
        let kinds = c.kinds.map(|k| pack_shown(k.as_ref()));
        let mut under = under[i];
        under[3] = pack_glyph(c.glyph.and_then(glyphs), c.seed);
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
            inks.push([c.ink.x, c.ink.y, frame.time, motion]);
            unders.push(under);
            for (k, kind) in kinds.iter().enumerate() {
                rings[k].push(kind.0);
                motions[k].push(kind.1);
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
    .with_inserted_attribute(ATTRIBUTE_UNDER, unders)
    .with_inserted_attribute(ATTRIBUTE_RINGS[0], buff_rings)
    .with_inserted_attribute(ATTRIBUTE_RINGS[1], hex_rings)
    .with_inserted_attribute(ATTRIBUTE_MOTION[0], buff_motion)
    .with_inserted_attribute(ATTRIBUTE_MOTION[1], hex_motion)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_constants_match() {
        use crate::render::lighting::{LIT, MARGIN, OCCUPIED, OPAQUE};
        let src = include_str!("circles.wesl");
        for (name, value) in [
            ("RING", RING),
            ("RING_GAP", RING_GAP),
            ("OUTER_RING", OUTER_RING),
            ("LINE_HALO_SIGMA", LINE_GLOW_SIGMA),
            ("GLINT_PERIOD_A", GLINT_PERIODS[0]),
            ("GLINT_PERIOD_B", GLINT_PERIODS[1]),
            ("GLINT_SHARP", GLINT_SHARP),
            ("NO_GLYPH", NO_GLYPH as f32),
            ("LIT", f32::from(LIT)),
            ("OPAQUE", f32::from(OPAQUE)),
            ("OCCUPIED", f32::from(OCCUPIED)),
            ("MARGIN", MARGIN as f32),
        ] {
            assert_eq!(crate::render::wesl_const(src, name), value, "{name}");
        }
        // `glint_phase`'s step per seed, the one number of the glint's math not a constant.
        assert!(src.contains("f32(seed) * 0.137036"));
    }

    /// Effect `slot` of school `school`.
    fn r(slot: u8, school: u8) -> Mark {
        Mark { slot, school }
    }

    fn buffs(slots: &[u8]) -> Circle {
        layout(slots.iter().map(|&s| (r(s, s % 7), false)))
    }

    #[test]
    fn nothing_circled_draws_nothing() {
        assert!(layout([]).is_empty());
    }

    // §4 rule 1 and 2: one effect is a triangle on one ring.
    #[test]
    fn one_effect_is_a_triangle_on_one_ring() {
        let ring = buffs(&[0]).inner.unwrap();
        assert_eq!((ring.slots(), ring.rings()), (3, 1));
        assert!(!ring.hex && !ring.outer);
    }

    // Two buffs: a square on two rings.
    #[test]
    fn two_effects_are_a_square_on_two_rings() {
        let ring = buffs(&[0, 3]).inner.unwrap();
        assert_eq!((ring.slots(), ring.rings()), (4, 2));
    }

    // Three or more: their polygon, three rings at most.
    #[test]
    fn three_to_six_effects_are_their_polygon_on_three_rings() {
        for n in 3..=6 {
            let slots: Vec<u8> = (0..n).collect();
            let ring = buffs(&slots).inner.unwrap();
            assert_eq!(ring.slots(), n, "{n} effects");
            assert_eq!(ring.rings(), 3, "rings capped at three");
        }
    }

    // §4 rule 4: six count, a seventh and later (in status order) change nothing.
    #[test]
    fn a_seventh_effect_changes_nothing() {
        let ring = buffs(&[0, 1, 2, 3, 4, 5, 6, 7]).inner.unwrap();
        assert_eq!(ring.effects, 6);
        assert_eq!(ring.marks.map(|m| m.slot), [0, 1, 2, 3, 4, 5]);
    }

    // The line colour is the first effect's school, in status order (never sorted).
    #[test]
    fn the_lines_take_the_first_effects_school() {
        let ring = layout([(r(5, 3), false), (r(1, 6), false)]).inner.unwrap();
        assert_eq!(ring.line_school(), 3);
    }

    // §5: a lone hex is broken and as compact as a buff.
    #[test]
    fn a_lone_hex_is_broken_and_compact() {
        let c = layout([(r(4, 5), true)]);
        let ring = c.inner.unwrap();
        assert!(ring.hex && !ring.outer);
        assert_eq!(ring.radius(), RING);
        assert_eq!(c.outer, None);
    }

    // §5: both kinds, the buffs inside, the hexes outside at the larger radii.
    #[test]
    fn buffs_go_inside_and_hexes_outside() {
        let c = layout([(r(3, 2), false), (r(4, 5), true), (r(1, 4), false)]);
        let (inner, outer) = (c.inner.unwrap(), c.outer.unwrap());
        assert!(!inner.hex && outer.hex && outer.outer);
        assert_eq!((inner.slots(), outer.slots()), (4, 3));
        assert_eq!(outer.radius(), OUTER_RING);
    }

    // The sizes hold together: the outer circle's rings clear the inner's third ring and its
    // glow, and the extents are as the spec says.
    #[test]
    fn the_circles_never_cross() {
        let third = RING + 2.0 * RING_GAP;
        assert!(OUTER_RING > third + 2.0 * LINE_GLOW_SIGMA);
        assert!((extent(0.0) - 0.68).abs() < 1e-5);
        assert!((extent(1.0) - 1.04).abs() < 1e-5);
        let settled = |c: Circle| CircleInstance {
            center: Vec2::ZERO,
            kinds: c.kinds().map(|k| k.map(|ring| Shown::settled(ring, 0.0))),
            keep: 1.0,
            cell: IVec2::ZERO,
            ink: Vec2::ZERO,
            glyph: None,
            seed: 0,
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

    #[test]
    fn the_ease_is_slow_in_and_out_and_clamped() {
        assert_eq!(ease(-1.0), 0.0);
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(0.5), 0.5);
        assert_eq!(ease(1.0), 1.0);
        assert_eq!(ease(2.0), 1.0);
        assert!(ease(0.1) < 0.1 && ease(0.9) > 0.9);
    }

    // Figures compare by their effects, not by where they stand.
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
        assert_eq!(pack(Some(&ring)), 2 | 1 << 4 | 4 << 8 | 4 << 12 | 2 << 16);
        assert_eq!(pack(None), 0);
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
            glyph: Some('k'),
            seed: 0x1_beef,
        };
        let both = CircleInstance {
            kinds: settled(layout([(r(0, 0), false), (r(1, 1), true)])),
            ..one
        };
        let frame = Frame {
            h: 18.0,
            time: 2.5,
            motion: true,
        };
        let mesh = circle_mesh(&[one, both], &frame, &|_| Some(3));
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
        // The lines' glow fits, and so does the glyph's square with its halo and glints.
        assert!(quad_margin() > 3.0 * LINE_GLOW_SIGMA);
        let glyph_reach = 0.5 + (3.0 * glyph_glow::GLOW_SIGMA).max(glyph_glow::GLINT_RAY);
        assert!(extent(0.0) + quad_margin() > glyph_reach);
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
            "the glyph's ink, the time, motion"
        );
        // The glyph's slot and the actor's seed ride in the fourth of `under`, exactly.
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(under)) =
            mesh.attribute(ATTRIBUTE_UNDER)
        else {
            panic!("unders are Float32x4");
        };
        assert_eq!(under[0][3] as u32, 3 << 16 | 0xbeef);
        assert_eq!(pack_glyph(None, 7) as u32, NO_GLYPH << 16 | 7);
        let none = circle_mesh(&[], &frame, &|_| None);
        assert_eq!(none.count_vertices(), 4, "never empty");
    }

    fn at(center: Vec2, c: Circle) -> CircleInstance {
        CircleInstance {
            center,
            kinds: c.kinds().map(|k| k.map(|ring| Shown::settled(ring, 0.0))),
            keep: 1.0,
            cell: IVec2::ZERO,
            ink: Vec2::ZERO,
            glyph: None,
            seed: 0,
        }
    }

    // Where circles crowd, a later circle's lines pass under the earlier's rings; apart,
    // nothing.
    #[test]
    fn crowded_circles_pass_under_earlier_ones() {
        let h = 18.0;
        let one = buffs(&[0]);
        let first = at(Vec2::ZERO, one);
        let near = at(Vec2::new(h, 0.0), one);
        let wide = at(Vec2::new(10.0 * h, 0.0), one);
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

    // The glints: a moment's sparkle now and then (most of the time none), each actor's on its
    // own beat, the same at the same moment (a frozen capture).
    #[test]
    fn glints_come_and_go_out_of_step() {
        for (g, period) in GLINT_PERIODS.iter().enumerate() {
            let samples: Vec<f32> = (0..2000)
                .map(|i| glint(i as f32 * 0.01 * period, 0.3, g))
                .collect();
            let shown = samples.iter().filter(|&&v| v > 0.1).count() as f32 / 2000.0;
            assert!((0.04..0.15).contains(&shown), "glint {g} shows {shown}");
            assert!(samples.iter().any(|&v| v > 0.99), "glint {g} peaks");
            assert!(samples.iter().all(|&v| (0.0..=1.0).contains(&v)));
        }
        assert_eq!(glint(1.7, 0.4, 0), glint(1.7, 0.4, 0));
        let d = (glint_phase(1, 0) - glint_phase(2, 0)).abs();
        assert!(d.min(1.0 - d) > 0.1, "two actors out of step");
        let d = (glint_phase(1, 0) - glint_phase(1, 1)).abs();
        assert!(d.min(1.0 - d) > 0.1, "an actor's two glints out of step");
    }
}
