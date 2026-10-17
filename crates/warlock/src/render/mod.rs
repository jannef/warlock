//! Rendering: the glyph atlas, console buffers, and the layers they are drawn in (spec §4).
//!
//! Layers, bottom to top: the map grid (terrain, in-cell animation; [`grid`]), the free-glyph
//! layer (creatures and glyphs that move or grow; [`free_glyph`]), and the UI console (text,
//! panels, overlays). Both consoles are drawn by the same console material.

pub mod anim;
pub mod atlas;
pub mod console;
pub mod free_glyph;
pub mod grid;

use crate::capture::CaptureTarget;
use crate::cli::Args;
use crate::game::Game;
use crate::viewport::Viewport;
use anim::{AnimTable, GridAnimTable};
use atlas::{Font, GlyphAtlas, glyph_set};
use bevy::asset::{AssetPath, RenderAssetUsages, embedded_asset, embedded_path};
use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use console::Console;
use grid::MapGrid;

/// Font size in logical pixels.
pub const DEFAULT_FONT_PX: f32 = 18.0;

/// Draw order of the map grid (bottom layer).
const MAP_Z: f32 = 0.0;

/// Draw order of the UI console (above the free-glyph layer).
const UI_Z: f32 = 2.0;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "console.wesl");
        app.add_plugins((
            Material2dPlugin::<ConsoleMaterial>::default(),
            free_glyph::FreeGlyphPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<Console>()
        .init_resource::<MapGrid>()
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

/// Visual effect settings (spec §4.7).
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Effects {
    /// Reduced effects: no glyph animation, no slides (instant snap), no camera easing.
    pub reduced: bool,
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
    pub clip_min: Vec2,
    pub clip_max: Vec2,
    pub grid_size: UVec2,
    pub atlas_columns: u32,
    pub supersample: u32,
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
    #[texture(5, sample_type = "u_int")]
    pub anims: Handle<Image>,
    #[texture(6, sample_type = "float", filterable = true)]
    #[sampler(7)]
    pub smooth_atlas: Handle<Image>,
    #[uniform(8)]
    pub anim_table: GridAnimTable,
}

impl Material2d for ConsoleMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("console.wesl")).with_source("embedded"),
        )
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Materials of the two console quads.
#[derive(Resource)]
struct ConsoleHandles {
    map: Handle<ConsoleMaterial>,
    ui: Handle<ConsoleMaterial>,
}

/// The console quads are drawn far larger than any window; the shader decides what each pixel
/// shows from its framebuffer position, so a quad only needs to cover the screen.
const QUAD_SIZE: f32 = 1.0e6;

fn cell_texture(images: &mut Assets<Image>, format: TextureFormat) -> Handle<Image> {
    images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        format,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ))
}

fn console_material(
    images: &mut Assets<Image>,
    atlas: &GlyphAtlas,
    anims: &AnimTable,
) -> ConsoleMaterial {
    ConsoleMaterial {
        params: ConsoleParams::default(),
        atlas: atlas.image.clone(),
        glyphs: cell_texture(images, TextureFormat::Rgba8Uint),
        fg: cell_texture(images, TextureFormat::Rgba8UnormSrgb),
        bg: cell_texture(images, TextureFormat::Rgba8UnormSrgb),
        anims: cell_texture(images, TextureFormat::Rgba8Uint),
        smooth_atlas: atlas.smooth.clone(),
        anim_table: anims.gpu,
    }
}

pub(crate) fn setup(
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

    let effects = Effects {
        reduced: args.reduced_effects,
    };
    let font = Font::load().expect("the bundled font is valid");
    let px = DEFAULT_FONT_PX * viewport.scale;
    let content = game.world.content();
    let atlas = GlyphAtlas::new(&font.0, px, &glyph_set(content.glyphs()), &mut images);
    let anims = AnimTable::new(content, |c| atlas.glyph(c), !effects.reduced);

    let quad = meshes.add(Rectangle::new(QUAD_SIZE, QUAD_SIZE));
    let map = materials.add(console_material(&mut images, &atlas, &anims));
    let ui = materials.add(console_material(&mut images, &atlas, &anims));
    for (material, z) in [(&map, MAP_Z), (&ui, UI_Z)] {
        commands.spawn((
            Mesh2d(quad.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, z),
        ));
    }
    commands.insert_resource(ConsoleHandles { map, ui });
    commands.insert_resource(atlas);
    commands.insert_resource(anims);
    commands.insert_resource(font);
    commands.insert_resource(effects);
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
    mut anims: ResMut<AnimTable>,
    effects: Res<Effects>,
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
        let content = game.world.content();
        *atlas = GlyphAtlas::new(&font.0, px, &glyph_set(content.glyphs()), &mut images);
        *anims = AnimTable::new(content, |c| atlas.glyph(c), !effects.reduced);
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

/// Copies both consoles into their per-cell textures and updates shader parameters.
fn upload(
    mut console: ResMut<Console>,
    mut grid: ResMut<MapGrid>,
    layout: Res<ConsoleLayout>,
    atlas: Res<GlyphAtlas>,
    anims: Res<AnimTable>,
    visual: Res<VisualTime>,
    handles: Res<ConsoleHandles>,
    mut materials: ResMut<Assets<ConsoleMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let params = |console: &Console, origin: Vec2, clip: Rect| ConsoleParams {
        origin,
        cell_size: layout.cell.as_vec2(),
        clip_min: clip.min,
        clip_max: clip.max,
        grid_size: UVec2::new(console.cols(), console.rows()),
        atlas_columns: atlas::ATLAS_COLUMNS,
        supersample: atlas::SUPERSAMPLE,
        time: visual.seconds,
    };
    let screen = Rect::from_corners(Vec2::ZERO, layout.window.as_vec2());
    let ui = params(&console, layout.origin, screen);
    let map = params(&grid.console, grid.origin, grid.clip);
    for (handle, console, params) in [
        (&handles.ui, &mut *console, ui),
        (&handles.map, &mut grid.console, map),
    ] {
        let Some(mut material) = materials.get_mut(handle) else {
            continue;
        };
        material.params = params;
        material.atlas = atlas.image.clone();
        material.smooth_atlas = atlas.smooth.clone();
        material.anim_table = anims.gpu;
        if console.dirty {
            console.dirty = false;
            upload_cells(console, &material, &atlas, &mut images);
        }
    }
}

/// Writes a console's cells into its material's per-cell textures.
fn upload_cells(
    console: &Console,
    material: &ConsoleMaterial,
    atlas: &GlyphAtlas,
    images: &mut Assets<Image>,
) {
    let n = console.cells().len();
    let (mut glyphs, mut fg, mut bg, mut anims) = (
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
        Vec::with_capacity(n * 4),
    );
    for (cell, anim) in console.cells().iter().zip(console.anims()) {
        match cell {
            Some(cell) => {
                let [lo, hi] = atlas.glyph(cell.ch).to_le_bytes();
                glyphs.extend_from_slice(&[lo, hi, 0, 0]);
                fg.extend_from_slice(&[cell.fg.r, cell.fg.g, cell.fg.b, 255]);
                bg.extend_from_slice(&[cell.bg.r, cell.bg.g, cell.bg.b, 255]);
            }
            // Transparent: no glyph, background alpha 0.
            None => {
                glyphs.extend_from_slice(&[0; 4]);
                fg.extend_from_slice(&[0; 4]);
                bg.extend_from_slice(&[0; 4]);
            }
        }
        anims.extend_from_slice(&[anim.entry, anim.phase, 0, 0]);
    }
    let size = Extent3d {
        width: console.cols().max(1),
        height: console.rows().max(1),
        depth_or_array_layers: 1,
    };
    for (handle, mut data) in [
        (&material.glyphs, glyphs),
        (&material.fg, fg),
        (&material.bg, bg),
        (&material.anims, anims),
    ] {
        // An empty console still needs one (transparent) texel.
        data.resize((size.width * size.height * 4) as usize, 0);
        if let Some(mut image) = images.get_mut(handle) {
            if image.texture_descriptor.size != size {
                image.texture_descriptor.size = size;
            }
            image.data = Some(data);
        }
    }
}
