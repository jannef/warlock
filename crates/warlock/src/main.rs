//! Warlock game binary.

mod capture;
mod cli;
mod game;
mod input;
mod logging;
#[cfg(not(target_arch = "wasm32"))]
mod move_data;
mod paths;
mod render;
mod save;
mod settings;
mod storage;
mod ui;
mod viewport;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;
use std::time::Duration;

/// `--paths`, `--move-data`, `--export-keys`, and `--import-keys`: print and exit, with no
/// window and no log file.
#[cfg(not(target_arch = "wasm32"))]
fn file_command(args: &cli::Args) -> Option<AppExit> {
    if args.paths {
        print!("{}", paths::init(args, true).report());
        return Some(AppExit::Success);
    }
    if args.export_keys.is_some() || args.import_keys.is_some() {
        paths::init(args, false);
        return Some(match settings::keymap::command(args) {
            Ok(text) => {
                print!("{text}");
                AppExit::Success
            }
            Err(text) => {
                eprintln!("warlock: {text}");
                AppExit::error()
            }
        });
    }
    let words = args.move_data.as_ref()?;
    let result = move_data::Target::parse(words).and_then(|target| {
        let platform = paths::Inputs::standard()
            .platform
            .ok_or("No home directory was found.")?;
        move_data::run(
            &target,
            args.dry_run,
            paths::init(args, true),
            &platform,
            paths::exe_game_folder().as_deref(),
            &logging::timestamp(),
        )
    });
    Some(match result {
        Ok(text) => {
            print!("{text}");
            AppExit::Success
        }
        Err(text) => {
            eprintln!("warlock: {text}");
            AppExit::error()
        }
    })
}

fn main() -> AppExit {
    let args = cli::Args::from_env();
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(exit) = file_command(&args) {
        return exit;
    }
    for warning in &paths::init(&args, false).warnings {
        eprintln!("warlock: {warning}");
    }
    logging::init();
    logging::maybe_crash_test();
    let content = match game::load_content(&args) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("warlock: {err}");
            return AppExit::error();
        }
    };
    let (settings_store, settings, bindings) = settings::load_for(&args);
    // Flags that name what to run skip the menu; otherwise the game starts when the player picks.
    // The main menu and `--screen` are opened at startup (`ui::screens`).
    let game = if !args.starts_in_menu() {
        match game::Game::start(&args, content.clone()) {
            Ok(game) => Some(game),
            Err(err) => {
                eprintln!("warlock: {err}");
                return AppExit::error();
            }
        }
    } else {
        None
    };

    let mut app = App::new();
    if capture::is_headless(&args) {
        // Offscreen: no window, no winit; the schedule runner drives frames.
        app.add_plugins((
            DefaultPlugins
                .set(logging::log_plugin())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
            // `WARLOCK_UNPACED=1` removes the 60 Hz pacing so `cargo xtask bench-frames` measures
            // real frame cost.
            ScheduleRunnerPlugin::run_loop(if std::env::var_os("WARLOCK_UNPACED").is_some() {
                Duration::ZERO
            } else {
                Duration::from_secs_f64(1.0 / 60.0)
            }),
        ));
    } else {
        let mut resolution = WindowResolution::new(args.window.x, args.window.y);
        if let Some(scale) = args.scale {
            resolution = resolution.with_scale_factor_override(scale);
        }
        app.add_plugins((
            DefaultPlugins.set(logging::log_plugin()).set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Warlock".into(),
                    // Web only (ignored on desktop): draw into web/index.html's canvas.
                    canvas: Some("#warlock-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    resolution,
                    ..default()
                }),
                ..default()
            }),
            viewport::WindowViewportPlugin,
        ));
    }
    app.insert_resource(args)
        .insert_resource(game::GameContent(content))
        .insert_resource(settings)
        .insert_resource(settings_store)
        .insert_resource(bindings);
    if let Some(game) = game {
        app.insert_resource(game);
    }
    app.add_plugins((
        game::GamePlugin,
        input::InputPlugin,
        render::RenderPlugin,
        save::SavePlugin,
        settings::SettingsPlugin,
        ui::UiPlugin,
        capture::CapturePlugin,
    ))
    .run()
}
