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
//! the polygon, a tap of the [`RuneAtlas`] per rune) in one draw call, over the map grid and under
//! the free-glyph layer. The school colours are a uniform, palette-mapped.

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

/// The innermost ring's radius, in cell heights: just clear of the actor's glyph.
pub const RING: f32 = 0.48;

/// From one ring to the next, in cell heights (the shader's; the tests check the geometry).
#[allow(dead_code)]
pub const RING_GAP: f32 = 0.1;

/// The runes' centres from the actor's, in cell heights: outside the rings.
pub const RUNE_RADIUS: f32 = 1.0;

/// The hexes' circle outside a buffs' one: its innermost ring and its runes, in cell heights.
pub const OUTER_RING: f32 = 1.35;
pub const OUTER_RUNE_RADIUS: f32 = 1.88;

/// Seconds a buffs' circle takes to turn once (clockwise), and a hexes' (counter-clockwise).
pub const BUFF_PERIOD: f32 = 60.0;
pub const HEX_PERIOD: f32 = 45.0;

/// The circle's extent (its farthest ink) in cell heights: its outermost runes' outer edge.
pub fn extent(circle: &Circle) -> f32 {
    let ring = circle.outer.as_ref().or(circle.inner.as_ref());
    ring.map_or(0.0, |r| r.radii().1) + RUNE_SIZE / 2.0
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

    /// The innermost ring's radius and the runes', in cell heights.
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
}

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

/// One circled actor this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircleInstance {
    /// The actor's drawn centre, in framebuffer pixels.
    pub center: Vec2,
    pub circle: Circle,
    /// The inner and the outer circle's [`angle`].
    pub angles: [f32; 2],
    /// A spell effect's darkening (`VisualLight::keep`): 1 none.
    pub keep: f32,
}

/// The circles to draw this frame, and where they may appear.
#[derive(Resource, Debug, Default)]
pub struct Circles {
    instances: Vec<CircleInstance>,
    /// Framebuffer rectangle outside of which nothing is drawn (the map viewport).
    pub clip: Rect,
    /// The cell height in pixels (the font size): every size is in cell heights.
    pub cell_height: f32,
}

impl Circles {
    pub fn push(&mut self, instance: CircleInstance) {
        if !instance.circle.is_empty() {
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
    /// A rune atlas slot's side in texels, its border included.
    pub slot: u32,
    pub columns: u32,
    pub supersample: u32,
    pub rune_size: f32,
    pub schools: [Vec4; 8],
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CircleMaterial {
    #[uniform(0)]
    pub params: CircleParams,
    #[texture(1, sample_type = "float", filterable = true)]
    #[sampler(2)]
    pub runes: Handle<Image>,
}

/// The instance's geometry: the cell height, the two angles, and the darkening.
const ATTRIBUTE_SHAPE: MeshVertexAttribute =
    MeshVertexAttribute::new("CircleShape", 0x5741_524c_0101, VertexFormat::Float32x4);
/// The inner and the outer circle: each `[head, runes 0-2, runes 3-5]` ([`pack`]).
const ATTRIBUTE_RINGS: [MeshVertexAttribute; 2] = [
    MeshVertexAttribute::new("CircleInner", 0x5741_524c_0102, VertexFormat::Uint32x4),
    MeshVertexAttribute::new("CircleOuter", 0x5741_524c_0103, VertexFormat::Uint32x4),
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
) {
    let runes = RuneAtlas::new(
        &rune_atlas::font(),
        content.0.runes(),
        atlas.cell.y as f32,
        &mut images,
    );
    let mesh = meshes.add(circle_mesh(&[], 1.0));
    let material = materials.add(CircleMaterial {
        params: CircleParams::default(),
        runes: runes.image.clone(),
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
    mut built: Local<Option<(Vec<CircleInstance>, u32)>>,
) {
    let h = circles.cell_height;
    if h > 0.0 && (h - atlas.cell_height).abs() > f32::EPSILON {
        *atlas = RuneAtlas::new(&rune_atlas::font(), content.0.runes(), h, &mut images);
    }
    let params = CircleParams {
        clip_min: circles.clip.min,
        clip_max: circles.clip.max,
        slot: atlas.slot,
        columns: rune_atlas::COLUMNS,
        supersample: rune_atlas::SUPERSAMPLE,
        rune_size: RUNE_SIZE,
        schools: colors.0,
    };
    if let Some(material) = materials.get(&handles.material)
        && (material.params != params || material.runes != atlas.image)
        && let Some(mut material) = materials.get_mut(&handles.material)
    {
        material.params = params;
        material.runes = atlas.image.clone();
    }
    let key = (circles.instances.clone(), h.to_bits());
    if built.as_ref() == Some(&key) {
        return;
    }
    if let Some(mut mesh) = meshes.get_mut(&handles.mesh) {
        *mesh = circle_mesh(&circles.instances, h);
    }
    *built = Some(key);
}

/// One quad per circle, `2 × extent` square about its centre (and a pixel more). An empty
/// layer gets a single quad with no circle, so the mesh is never empty.
fn circle_mesh(instances: &[CircleInstance], h: f32) -> Mesh {
    const NONE: CircleInstance = CircleInstance {
        center: Vec2::ZERO,
        circle: Circle {
            inner: None,
            outer: None,
        },
        angles: [0.0; 2],
        keep: 1.0,
    };
    let instances = if instances.is_empty() {
        &[NONE][..]
    } else {
        instances
    };
    let n = instances.len();
    let (mut positions, mut locals, mut shapes, mut inner, mut outer, mut indices) = (
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 6),
    );
    for (i, c) in instances.iter().enumerate() {
        let half = extent(&c.circle) * h + 1.0;
        let rings = [pack(c.circle.inner.as_ref()), pack(c.circle.outer.as_ref())];
        for corner in [
            Vec2::new(-half, -half),
            Vec2::new(half, -half),
            Vec2::new(half, half),
            Vec2::new(-half, half),
        ] {
            let at = c.center + corner;
            positions.push([at.x, at.y, 0.0]);
            locals.push(corner.to_array());
            shapes.push([h, c.angles[0], c.angles[1], c.keep]);
            inner.push(rings[0]);
            outer.push(rings[1]);
        }
        let base = (i * 4) as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, locals)
    .with_inserted_attribute(ATTRIBUTE_SHAPE, shapes)
    .with_inserted_attribute(ATTRIBUTE_RINGS[0], inner)
    .with_inserted_attribute(ATTRIBUTE_RINGS[1], outer)
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
        let one = layout([(r(0, 0), false)]);
        let both = layout([(r(0, 0), false), (r(1, 1), true)]);
        assert!(extent(&one) < OUTER_RING);
        assert!(OUTER_RING + 2.0 * RING_GAP + 0.03 < OUTER_RUNE_RADIUS - half_rune);
        assert!((extent(&one) - 1.275).abs() < 1e-6);
        assert!((extent(&both) - 2.155).abs() < 1e-6);
        assert!(
            (extent(&layout([(r(0, 0), true)])) - 1.275).abs() < 1e-6,
            "a lone hex"
        );
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
        let one = CircleInstance {
            center: Vec2::new(100.0, 50.0),
            circle: buffs(&[0]),
            angles: [0.0; 2],
            keep: 1.0,
        };
        let both = CircleInstance {
            circle: layout([(r(0, 0), false), (r(1, 1), true)]),
            ..one
        };
        let mesh = circle_mesh(&[one, both], 18.0);
        assert_eq!(mesh.count_vertices(), 8);
        assert_eq!(mesh.indices().map(Indices::len), Some(12));
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(local)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("locals are Float32x2");
        };
        let half = |i: usize| local[i][0].abs();
        assert!((half(0) - (extent(&one.circle) * 18.0 + 1.0)).abs() < 1e-4);
        assert!((half(4) - (extent(&both.circle) * 18.0 + 1.0)).abs() < 1e-4);
        assert_eq!(circle_mesh(&[], 18.0).count_vertices(), 4, "never empty");
    }
}
