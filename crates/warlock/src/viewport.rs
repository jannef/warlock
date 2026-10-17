//! The surface the game renders to: the primary window, or an offscreen image in `--capture` mode.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Render surface size in physical pixels and its display scale factor.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub size: UVec2,
    pub scale: f32,
}

/// Keeps [`Viewport`] in sync with the primary window (windowed mode only).
pub struct WindowViewportPlugin;

impl Plugin for WindowViewportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, sync_from_window)
            .add_systems(First, sync_from_window);
    }
}

fn sync_from_window(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    viewport: Option<ResMut<Viewport>>,
) {
    let current = Viewport {
        size: UVec2::new(window.physical_width(), window.physical_height()),
        scale: window.scale_factor(),
    };
    match viewport {
        Some(mut v) if *v != current => *v = current,
        Some(_) => {}
        None => commands.insert_resource(current),
    }
}
