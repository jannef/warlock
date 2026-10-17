//! Warlock game binary.

mod capture;
mod cli;
mod game;
mod input;
mod logging;
mod paths;
mod render;
mod save;
mod settings;
mod ui;
mod viewport;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;
use std::time::Duration;

fn main() -> AppExit {
    let args = cli::Args::from_env();
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
    let menu = args.starts_in_menu().then(|| {
        let has_save = save::SaveStore::from_args(&args).has_save();
        ui::Menu::new(&content, has_save, args.screen)
    });
    // `--screen pause` starts a game and opens the pause menu over it.
    let menu = menu.or_else(|| {
        (args.screen == Some(ui::ScreenName::Pause)).then(|| ui::Menu::pause(&content))
    });
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
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
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
    if let Some(menu) = menu {
        app.insert_resource(menu);
    }
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
