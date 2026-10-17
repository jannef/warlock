//! Warlock game binary.

mod capture;
mod cli;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() -> AppExit {
    let args = cli::Args::from_env();

    let mut resolution = WindowResolution::new(args.window.x, args.window.y);
    if let Some(scale) = args.scale {
        resolution = resolution.with_scale_factor_override(scale);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Warlock".into(),
                resolution,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(args)
        .add_plugins(capture::CapturePlugin)
        .add_systems(Startup, setup)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, Tonemapping::None));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(200.0, 120.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.5, 0.2))),
    ));
}
