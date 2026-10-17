//! Post-processing (spec §4.2, layer 4): bloom for glowing glyphs, and screen tints over the map
//! (the fade on level change, and flashes as a hook for hit-style effects).
//!
//! Bloom runs on an HDR camera. Only colors brighter than 1 bloom (glyphs that content marks as
//! glowing are pushed past 1), and the bloom is added on top, so everything else keeps its exact
//! palette color. Heat-haze distortion is not here: it runs in the console shader, per cell, so
//! text elsewhere stays crisp. With the `glow` effect off there is no bloom; with `screen_fades` off, no tint.

use super::Effects;
use super::grid::MapGrid;
use bevy::asset::{AssetPath, embedded_asset, embedded_path};
use bevy::camera::Hdr;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

/// Draw order of the tint overlay: above the free-glyph layer (1), below the UI console (2).
const TINT_Z: f32 = 1.5;

/// The overlay quad is far larger than any window; the shader clips it to the map viewport.
const QUAD_SIZE: f32 = 1.0e6;

pub struct PostPlugin;

impl Plugin for PostPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "tint.wesl");
        app.add_plugins(Material2dPlugin::<TintMaterial>::default())
            .init_resource::<ScreenTints>()
            .add_systems(Startup, spawn_overlay)
            .add_systems(PostUpdate, (sync_camera, update_overlay));
    }
}

/// HDR brightness gain of a fully glowing glyph: its color is multiplied by
/// `1 + glow × GLOW_GAIN`, which pushes it past the bloom threshold.
pub const GLOW_GAIN: f32 = 1.0;

/// `color` brightened past 1 by `glow` (0 to 1), for bloom. Alpha is kept.
pub fn emissive(color: LinearRgba, glow: f32) -> LinearRgba {
    let k = 1.0 + glow.max(0.0) * GLOW_GAIN;
    LinearRgba::new(color.red * k, color.green * k, color.blue * k, color.alpha)
}

/// `color` with red, green, and blue at most 1: lit colors may be overbright, but only glowing
/// glyphs should bloom.
pub fn in_range(color: LinearRgba) -> LinearRgba {
    LinearRgba::new(
        color.red.min(1.0),
        color.green.min(1.0),
        color.blue.min(1.0),
        color.alpha,
    )
}

/// Bloom for glowing glyphs. Tight and faint: the halo stays within about a cell, so the glyph
/// core and its neighbors stay readable.
pub fn bloom() -> Bloom {
    Bloom {
        intensity: 0.45,
        low_frequency_boost: 0.0,
        low_frequency_boost_curvature: 0.0,
        high_pass_frequency: 0.5,
        prefilter: BloomPrefilter {
            threshold: 1.0,
            threshold_softness: 0.0,
        },
        composite_mode: BloomCompositeMode::Additive,
        ..Bloom::NATURAL
    }
}

/// Turns bloom (and the HDR target it needs) on or off with the effects setting.
fn sync_camera(
    mut commands: Commands,
    effects: Res<Effects>,
    cameras: Query<(Entity, Has<Bloom>), With<Camera2d>>,
) {
    if !effects.is_changed() {
        return;
    }
    for (camera, has_bloom) in &cameras {
        if !effects.glow && has_bloom {
            commands.entity(camera).remove::<(Bloom, Hdr)>();
        } else if effects.glow && !has_bloom {
            commands.entity(camera).insert(bloom());
        }
    }
}

/// A full-screen color that fades out: `color` (its alpha is the peak opacity) at `start`,
/// gone `duration` seconds later.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub color: LinearRgba,
    pub start: f32,
    pub duration: f32,
}

impl Tint {
    /// Opacity at `time`: full at the start, easing out to 0.
    fn alpha(&self, time: f32) -> f32 {
        let x = ((time - self.start) / self.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        self.color.alpha * (1.0 - x * x * (3.0 - 2.0 * x))
    }

    fn is_done(&self, time: f32) -> bool {
        time - self.start >= self.duration
    }
}

/// Running screen tints over the map viewport.
#[derive(Resource, Debug, Default)]
pub struct ScreenTints {
    tints: Vec<Tint>,
}

impl ScreenTints {
    /// Starts a flash of `color` (alpha = peak opacity) at `time` that fades out over `duration`
    /// seconds: black for a fade-in after a level change, a red tint for a future hit flash.
    pub fn flash(&mut self, color: LinearRgba, duration: f32, time: f32) {
        self.tints.push(Tint {
            color,
            start: time,
            duration,
        });
    }

    pub fn clear(&mut self) {
        self.tints.clear();
    }

    /// The running tints composited in start order (later over earlier), as one non-premultiplied
    /// color; alpha 0 when nothing is showing.
    pub fn overlay(&self, time: f32) -> LinearRgba {
        let mut rgb = Vec3::ZERO;
        let mut alpha = 0.0;
        for tint in &self.tints {
            let a = tint.alpha(time);
            let c = Vec3::new(tint.color.red, tint.color.green, tint.color.blue);
            // `tint` over what is below, premultiplied.
            rgb = c * a + rgb * (1.0 - a);
            alpha = a + alpha * (1.0 - a);
        }
        if alpha <= 0.0 {
            return LinearRgba::NONE;
        }
        LinearRgba::new(rgb.x / alpha, rgb.y / alpha, rgb.z / alpha, alpha)
    }

    fn prune(&mut self, time: f32) {
        self.tints.retain(|t| !t.is_done(time));
    }
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub struct TintParams {
    pub clip_min: Vec2,
    pub clip_max: Vec2,
    pub color: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TintMaterial {
    #[uniform(0)]
    pub params: TintParams,
}

impl Material2d for TintMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("tint.wesl")).with_source("embedded"),
        )
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Resource, Debug)]
struct Overlay {
    entity: Entity,
    material: Handle<TintMaterial>,
}

fn spawn_overlay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TintMaterial>>,
) {
    let material = materials.add(TintMaterial {
        params: TintParams::default(),
    });
    let entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(QUAD_SIZE, QUAD_SIZE))),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, TINT_Z),
            Visibility::Hidden,
        ))
        .id();
    commands.insert_resource(Overlay { entity, material });
}

/// Shows the composited tints over the map viewport; the overlay is hidden when idle, so it
/// costs nothing and changes no pixel.
fn update_overlay(
    time: Res<super::VisualTime>,
    grid: Res<MapGrid>,
    overlay: Res<Overlay>,
    effects: Res<Effects>,
    mut tints: ResMut<ScreenTints>,
    mut materials: ResMut<Assets<TintMaterial>>,
    mut visibility: Query<&mut Visibility>,
) {
    if !effects.screen_fades {
        tints.clear();
    }
    tints.prune(time.seconds);
    let color = tints.overlay(time.seconds);
    let shown = color.alpha > 0.0;
    if let Ok(mut v) = visibility.get_mut(overlay.entity) {
        v.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if !shown {
        return;
    }
    let params = TintParams {
        clip_min: grid.clip.min,
        clip_max: grid.clip.max,
        color: Vec4::new(color.red, color.green, color.blue, color.alpha),
    };
    if materials
        .get(&overlay.material)
        .is_some_and(|m| m.params != params)
        && let Some(mut material) = materials.get_mut(&overlay.material)
    {
        material.params = params;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flash_eases_out_and_ends() {
        let mut tints = ScreenTints::default();
        tints.flash(LinearRgba::new(1.0, 0.0, 0.0, 0.5), 0.2, 10.0);
        let at = |t| tints.overlay(t);
        assert_eq!(at(10.0), LinearRgba::new(1.0, 0.0, 0.0, 0.5));
        let mid = at(10.1).alpha;
        assert!((mid - 0.25).abs() < 1e-5, "{mid}");
        assert!(at(10.15).alpha < mid);
        assert_eq!(at(10.2), LinearRgba::NONE);
        assert_eq!(
            at(9.0).alpha,
            0.5,
            "before the start it shows at full strength"
        );
        tints.prune(10.25);
        assert!(tints.tints.is_empty());
    }

    #[test]
    fn later_tints_composite_over_earlier_ones() {
        let mut tints = ScreenTints::default();
        tints.flash(LinearRgba::new(0.0, 0.0, 0.0, 1.0), 1.0, 0.0);
        tints.flash(LinearRgba::new(1.0, 1.0, 1.0, 0.5), 1.0, 0.0);
        let c = tints.overlay(0.0);
        assert!((c.alpha - 1.0).abs() < 1e-6);
        // Half white over opaque black: mid grey.
        assert!((c.red - 0.5).abs() < 1e-5 && (c.blue - 0.5).abs() < 1e-5);
        tints.clear();
        assert_eq!(tints.overlay(0.0), LinearRgba::NONE);
    }

    #[test]
    fn glow_brightens_past_one() {
        let c = emissive(LinearRgba::new(1.0, 0.5, 0.0, 0.25), 1.0);
        assert!(c.red > 1.0 && (c.green - 0.5 * (1.0 + GLOW_GAIN)).abs() < 1e-6);
        assert_eq!(c.alpha, 0.25);
        assert_eq!(emissive(LinearRgba::WHITE, 0.0), LinearRgba::WHITE);
        let bright = LinearRgba::new(1.4, 0.5, 2.0, 0.5);
        assert_eq!(in_range(bright), LinearRgba::new(1.0, 0.5, 1.0, 0.5));
    }

    #[test]
    fn bloom_only_takes_overbright_colors() {
        let b = bloom();
        assert_eq!(b.prefilter.threshold, 1.0);
        assert_eq!(b.composite_mode, BloomCompositeMode::Additive);
    }
}
