//! Rendering: the glyph atlas, the console buffer, and the console grid material (spec §4).

pub mod atlas;
pub mod console;

use crate::capture::CaptureTarget;
use crate::cli::Args;
use crate::game::Game;
use crate::viewport::Viewport;
use atlas::{Font, GlyphAtlas, glyph_set, rasterize};
use bevy::asset::{AssetPath, RenderAssetUsages, embedded_asset, embedded_path};
use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin};
use console::Console;

/// Font size in logical pixels.
pub const DEFAULT_FONT_PX: f32 = 18.0;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "console.wesl");
        app.add_plugins(Material2dPlugin::<ConsoleMaterial>::default())
            .init_resource::<Console>()
            .init_resource::<ConsoleLayout>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, (advance_visual_time, layout))
            .add_systems(PostUpdate, upload);
    }
}

/// Visual time drives animation only; gameplay never reads it. `--freeze-time` pins it.
#[derive(Resource, Debug, Default)]
pub struct VisualTime {
    pub seconds: f32,
    pub frozen: bool,
}

/// Where the console sits in the window, in physical pixels.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct ConsoleLayout {
    pub window: UVec2,
    pub scale: f32,
    pub origin: Vec2,
    pub cell: UVec2,
    /// Bumped whenever the layout changes, so UI knows to redraw.
    pub generation: u32,
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub struct ConsoleParams {
    pub origin: Vec2,
    pub cell_size: Vec2,
    pub grid_size: UVec2,
    pub atlas_columns: u32,
    pub time: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ConsoleMaterial {
    #[uniform(0)]
    pub params: ConsoleParams,
    #[texture(1, sample_type = "float", filterable = false)]
    pub atlas: Handle<Image>,
    #[texture(2, sample_type = "u_int")]
    pub glyphs: Handle<Image>,
    #[texture(3, sample_type = "float", filterable = false)]
    pub fg: Handle<Image>,
    #[texture(4, sample_type = "float", filterable = false)]
    pub bg: Handle<Image>,
}

impl Material2d for ConsoleMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("console.wesl")).with_source("embedded"),
        )
    }
}

/// Handle of the console quad's material.
#[derive(Resource)]
struct ConsoleHandles {
    material: Handle<ConsoleMaterial>,
}

/// The console quad is drawn far larger than any window; the shader decides what each pixel
/// shows from its framebuffer position, so the quad only needs to cover the screen.
const QUAD_SIZE: f32 = 1.0e6;

fn cell_texture(images: &mut Assets<Image>, format: TextureFormat) -> Handle<Image> {
    images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        format,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ))
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ConsoleMaterial>>,
    viewport: Res<Viewport>,
    capture: Option<Res<CaptureTarget>>,
    game: Res<Game>,
    args: Res<Args>,
) {
    let mut camera = commands.spawn((Camera2d, Tonemapping::None));
    if let Some(capture) = capture {
        camera.insert(RenderTarget::Image(capture.0.clone().into()));
    }

    let font = Font::load().expect("the bundled font is valid");
    let px = DEFAULT_FONT_PX * viewport.scale;
    let chars = glyph_set(game.world.content().glyphs());
    let atlas = GlyphAtlas::from_bitmap(rasterize(&font.0, px, &chars), px, &mut images);

    let material = materials.add(ConsoleMaterial {
        params: ConsoleParams {
            atlas_columns: atlas::ATLAS_COLUMNS,
            ..default()
        },
        atlas: atlas.image.clone(),
        glyphs: cell_texture(&mut images, TextureFormat::Rgba8Uint),
        fg: cell_texture(&mut images, TextureFormat::Rgba8UnormSrgb),
        bg: cell_texture(&mut images, TextureFormat::Rgba8UnormSrgb),
    });
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(QUAD_SIZE, QUAD_SIZE))),
        MeshMaterial2d(material.clone()),
    ));
    commands.insert_resource(ConsoleHandles { material });
    commands.insert_resource(atlas);
    commands.insert_resource(font);
    commands.insert_resource(VisualTime {
        seconds: args.freeze_time.unwrap_or(0.0),
        frozen: args.freeze_time.is_some(),
    });
}

fn advance_visual_time(mut visual: ResMut<VisualTime>, time: Res<Time>) {
    if !visual.frozen {
        visual.seconds += time.delta_secs();
    }
}

/// Fits the console to the viewport; rebuilds the atlas when the display scale changes.
fn layout(
    viewport: Res<Viewport>,
    mut layout: ResMut<ConsoleLayout>,
    mut console: ResMut<Console>,
    mut atlas: ResMut<GlyphAtlas>,
    font: Res<Font>,
    game: Res<Game>,
    mut images: ResMut<Assets<Image>>,
) {
    let (size, scale) = (viewport.size, viewport.scale);
    if size == layout.window && scale == layout.scale {
        return;
    }
    let px = DEFAULT_FONT_PX * scale;
    if (px - atlas.px).abs() > f32::EPSILON {
        let chars = glyph_set(game.world.content().glyphs());
        *atlas = GlyphAtlas::from_bitmap(rasterize(&font.0, px, &chars), px, &mut images);
    }
    let cell = atlas.cell;
    let cols = (size.x / cell.x).max(1);
    let rows = (size.y / cell.y).max(1);
    let origin = ((size - UVec2::new(cols, rows) * cell) / 2).as_vec2();
    console.resize(cols, rows);
    *layout = ConsoleLayout {
        window: size,
        scale,
        origin,
        cell,
        generation: layout.generation.wrapping_add(1),
    };
}

/// Copies the console into the per-cell textures and updates shader parameters.
fn upload(
    mut console: ResMut<Console>,
    layout: Res<ConsoleLayout>,
    atlas: Res<GlyphAtlas>,
    visual: Res<VisualTime>,
    handles: Res<ConsoleHandles>,
    mut materials: ResMut<Assets<ConsoleMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut material) = materials.get_mut(&handles.material) else {
        return;
    };
    material.params = ConsoleParams {
        origin: layout.origin,
        cell_size: layout.cell.as_vec2(),
        grid_size: UVec2::new(console.cols(), console.rows()),
        atlas_columns: atlas::ATLAS_COLUMNS,
        time: visual.seconds,
    };
    material.atlas = atlas.image.clone();
    if !console.dirty {
        return;
    }
    console.dirty = false;

    let n = console.cells().len();
    let (mut glyphs, mut fg, mut bg) = (
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
    );
    for cell in console.cells() {
        let [lo, hi] = atlas.glyph(cell.ch).to_le_bytes();
        glyphs.extend_from_slice(&[lo, hi, 0, 0]);
        fg.extend_from_slice(&[cell.fg.r, cell.fg.g, cell.fg.b, 255]);
        bg.extend_from_slice(&[cell.bg.r, cell.bg.g, cell.bg.b, 255]);
    }
    let size = Extent3d {
        width: console.cols(),
        height: console.rows(),
        depth_or_array_layers: 1,
    };
    for (handle, data) in [
        (material.glyphs.clone(), glyphs),
        (material.fg.clone(), fg),
        (material.bg.clone(), bg),
    ] {
        if let Some(mut image) = images.get_mut(&handle) {
            if image.texture_descriptor.size != size {
                image.texture_descriptor.size = size;
            }
            image.data = Some(data);
        }
    }
}
