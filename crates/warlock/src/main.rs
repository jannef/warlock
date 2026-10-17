//! Warlock game binary.

mod capture;
mod cli;
mod game;
mod input;
mod render;
mod ui;
mod viewport;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;
use std::time::Duration;

fn main() -> AppExit {
    let args = cli::Args::from_env();
    let game = match game::Game::start(&args) {
        Ok(game) => game,
        Err(err) => {
            eprintln!("warlock: {err}");
            return AppExit::error();
        }
    };

    let mut app = App::new();
    if capture::is_headless(&args) {
        // Offscreen: no window, no winit; the schedule runner drives frames.
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ));
    } else {
        let mut resolution = WindowResolution::new(args.window.x, args.window.y);
        if let Some(scale) = args.scale {
            resolution = resolution.with_scale_factor_override(scale);
        }
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Warlock".into(),
                    resolution,
                    ..default()
                }),
                ..default()
            }),
            viewport::WindowViewportPlugin,
        ));
    }
    app.insert_resource(args)
        .insert_resource(game)
        .add_plugins((
            game::GamePlugin,
            input::InputPlugin,
            render::RenderPlugin,
            ui::UiPlugin,
            capture::CapturePlugin,
        ))
        .run()
}
