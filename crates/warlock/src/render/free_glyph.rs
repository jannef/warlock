//! The free-glyph layer (spec §4.2, layer 2): glyph quads at arbitrary sub-pixel positions with
//! color, alpha, and scale, for anything that moves between cells or grows beyond its cell.
//!
//! Producers push [`FreeGlyph`]s into [`FreeGlyphs`] each frame (it is emptied in `PreUpdate`). In
//! `PostUpdate` they become one dynamic mesh with one material, so the whole layer is one draw
//! call, above the map grid and below the UI console. Glyphs are drawn in push order.

use super::atlas::{self, GlyphAtlas};
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

/// Draw order of the free-glyph layer: above the map grid (0), below the UI console.
pub const LAYER_Z: f32 = 1.0;

/// Atlas slot and flags of a glyph quad's vertices.
const ATTRIBUTE_GLYPH: MeshVertexAttribute =
    MeshVertexAttribute::new("FreeGlyph", 0x5741_524c_0001, VertexFormat::Uint32x2);

/// Glyph flag: read the 1:1 atlas (whole-pixel position, scale 1). Mirrors `CRISP` in the shader.
const CRISP: u32 = 1;

pub struct FreeGlyphPlugin;

impl Plugin for FreeGlyphPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "free_glyph.wesl");
        app.add_plugins(Material2dPlugin::<FreeGlyphMaterial>::default())
            .init_resource::<FreeGlyphs>()
            .add_systems(Startup, spawn_layer.after(super::setup))
            .add_systems(PreUpdate, clear)
            .add_systems(PostUpdate, build_mesh);
    }
}

/// One glyph on the free-glyph layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeGlyph {
    /// Framebuffer position (physical pixels) of the top-left corner of the glyph's cell at
    /// scale 1.
    pub pos: Vec2,
    pub ch: char,
    /// Linear color; alpha fades the glyph.
    pub color: LinearRgba,
    /// Scale about the cell center.
    pub scale: f32,
}

impl FreeGlyph {
    /// Whether the glyph sits exactly on the pixel grid, unscaled, so it can be drawn 1:1.
    pub fn is_crisp(&self) -> bool {
        self.scale == 1.0 && self.pos == self.pos.round()
    }
}

/// The glyphs to draw this frame, in order, and where they may appear.
#[derive(Resource, Debug, Default)]
pub struct FreeGlyphs {
    glyphs: Vec<FreeGlyph>,
    /// Framebuffer rectangle outside of which nothing is drawn.
    pub clip: Rect,
}

impl FreeGlyphs {
    pub fn push(&mut self, glyph: FreeGlyph) {
        self.glyphs.push(glyph);
    }
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct FreeGlyphParams {
    pub clip_min: Vec2,
    pub clip_max: Vec2,
    pub cell_size: Vec2,
    pub atlas_columns: u32,
    pub supersample: u32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FreeGlyphMaterial {
    #[uniform(0)]
    pub params: FreeGlyphParams,
    #[texture(1, sample_type = "float", filterable = false)]
    pub atlas: Handle<Image>,
    #[texture(2, sample_type = "float", filterable = true)]
    #[sampler(3)]
    pub smooth_atlas: Handle<Image>,
}

impl Material2d for FreeGlyphMaterial {
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
            Mesh::ATTRIBUTE_COLOR.at_shader_location(2),
            ATTRIBUTE_GLYPH.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

fn shader() -> ShaderRef {
    ShaderRef::Path(
        AssetPath::from_path_buf(embedded_path!("free_glyph.wesl")).with_source("embedded"),
    )
}

/// The layer's mesh and material.
#[derive(Resource, Debug)]
struct LayerHandles {
    mesh: Handle<Mesh>,
    material: Handle<FreeGlyphMaterial>,
}

fn spawn_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FreeGlyphMaterial>>,
    atlas: Res<GlyphAtlas>,
) {
    let mesh = meshes.add(glyph_mesh(&[], atlas.cell.as_vec2(), |_| 0));
    let material = materials.add(FreeGlyphMaterial {
        params: FreeGlyphParams::default(),
        atlas: atlas.image.clone(),
        smooth_atlas: atlas.smooth.clone(),
    });
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(0.0, 0.0, LAYER_Z),
        // Vertices are in framebuffer pixels, not world space, so culling would be wrong.
        NoFrustumCulling,
    ));
    commands.insert_resource(LayerHandles { mesh, material });
}

fn clear(mut free: ResMut<FreeGlyphs>) {
    free.glyphs.clear();
}

/// Builds the layer's mesh from this frame's glyphs (only when they changed).
fn build_mesh(
    free: Res<FreeGlyphs>,
    atlas: Res<GlyphAtlas>,
    handles: Res<LayerHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FreeGlyphMaterial>>,
    mut built: Local<Option<(Vec<FreeGlyph>, UVec2)>>,
) {
    let params = FreeGlyphParams {
        clip_min: free.clip.min,
        clip_max: free.clip.max,
        cell_size: atlas.cell.as_vec2(),
        atlas_columns: atlas.columns,
        supersample: atlas::SUPERSAMPLE,
    };
    if let Some(material) = materials.get(&handles.material)
        && (material.params != params || material.atlas != atlas.image)
        && let Some(mut material) = materials.get_mut(&handles.material)
    {
        material.params = params;
        material.atlas = atlas.image.clone();
        material.smooth_atlas = atlas.smooth.clone();
    }

    let key = (free.glyphs.clone(), atlas.cell);
    if built.as_ref() == Some(&key) {
        return;
    }
    if let Some(mut mesh) = meshes.get_mut(&handles.mesh) {
        *mesh = glyph_mesh(&free.glyphs, atlas.cell.as_vec2(), |c| atlas.glyph(c));
    }
    *built = Some(key);
}

/// One quad per glyph. An empty layer gets a single invisible quad, so the mesh is never empty.
fn glyph_mesh(glyphs: &[FreeGlyph], cell: Vec2, slot: impl Fn(char) -> u16) -> Mesh {
    const HIDDEN: FreeGlyph = FreeGlyph {
        pos: Vec2::ZERO,
        ch: ' ',
        color: LinearRgba::NONE,
        scale: 1.0,
    };
    let glyphs = if glyphs.is_empty() {
        &[HIDDEN][..]
    } else {
        glyphs
    };
    let n = glyphs.len();
    let (mut positions, mut locals, mut colors, mut data, mut indices) = (
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 6),
    );
    for (i, g) in glyphs.iter().enumerate() {
        let size = cell * g.scale;
        let min = g.pos + (cell - size) * 0.5;
        let max = min + size;
        let flags = if g.is_crisp() { CRISP } else { 0 };
        for (corner, local) in [
            (Vec2::new(min.x, min.y), Vec2::ZERO),
            (Vec2::new(max.x, min.y), Vec2::new(cell.x, 0.0)),
            (Vec2::new(max.x, max.y), cell),
            (Vec2::new(min.x, max.y), Vec2::new(0.0, cell.y)),
        ] {
            positions.push([corner.x, corner.y, 0.0]);
            locals.push(local.to_array());
            colors.push(g.color.to_f32_array());
            data.push([u32::from(slot(g.ch)), flags]);
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(ATTRIBUTE_GLYPH, data)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    fn glyph(pos: Vec2, scale: f32) -> FreeGlyph {
        FreeGlyph {
            pos,
            ch: '@',
            color: LinearRgba::WHITE,
            scale,
        }
    }

    #[test]
    fn crisp_only_on_whole_pixels_at_scale_one() {
        assert!(glyph(Vec2::new(10.0, 20.0), 1.0).is_crisp());
        assert!(!glyph(Vec2::new(10.5, 20.0), 1.0).is_crisp());
        assert!(!glyph(Vec2::new(10.0, 20.0), 1.1).is_crisp());
    }

    #[test]
    fn scaled_quads_grow_about_the_cell_center() {
        let cell = Vec2::new(10.0, 20.0);
        let mesh = glyph_mesh(&[glyph(Vec2::new(100.0, 40.0), 1.5)], cell, |_| 3);
        let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions are Float32x3");
        };
        assert_eq!(p[0], [97.5, 35.0, 0.0]);
        assert_eq!(p[2], [112.5, 65.0, 0.0]);
        let Some(VertexAttributeValues::Uint32x2(d)) = mesh.attribute(ATTRIBUTE_GLYPH) else {
            panic!("glyph data is Uint32x2");
        };
        assert_eq!(d[0], [3, 0]);
        assert_eq!(mesh.indices().map(Indices::len), Some(6));
    }

    #[test]
    fn empty_layer_has_one_invisible_quad() {
        let mesh = glyph_mesh(&[], Vec2::new(10.0, 20.0), |_| 0);
        assert_eq!(mesh.count_vertices(), 4);
        let Some(VertexAttributeValues::Float32x4(c)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("colors are Float32x4");
        };
        assert_eq!(c[0][3], 0.0);
    }
}
