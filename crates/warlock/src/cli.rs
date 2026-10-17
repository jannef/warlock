//! Command-line options of the game binary (see spec §9).

use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// Options parsed from the command line. On the web build there is no command line, so the
/// defaults are used.
#[derive(Debug, Clone, Parser, Resource)]
#[command(name = "warlock", version, about = "Warlock — an ASCII roguelike")]
pub struct Args {
    /// Scenario to start (a file stem under `assets/scenarios/`; default `showcase`). Giving it
    /// skips the main menu.
    #[arg(long)]
    pub scenario: Option<String>,

    /// Run seed: a number, or a word that is hashed (`--seed warlock`); defaults to the
    /// scenario's seed.
    #[arg(long, value_parser = warlock_core::rng::parse_seed)]
    pub seed: Option<u64>,

    /// Apply this replay file at startup (its scenario and seed win); hashes are verified and a
    /// divergence is logged as a warning.
    #[arg(long)]
    pub replay: Option<PathBuf>,

    /// Record every applied action to this replay file, rewritten after each action. Includes
    /// the steps of `--replay`, if given.
    #[arg(long)]
    pub record: Option<PathBuf>,

    /// Read content (`data/`, `levels/`, `scenarios/`, `vaults/`) from this directory instead of
    /// the built-in copy, e.g. `--content-dir assets`.
    #[arg(long)]
    pub content_dir: Option<PathBuf>,

    /// Freeze visual time at this many seconds (reproducible screenshots). Glyph animations and
    /// particles show that moment; slides, camera moves, and fades complete instantly.
    #[arg(long)]
    pub freeze_time: Option<f32>,

    /// Force reduced effects on: no glyph animation, light flicker, movement slides, camera
    /// easing, particles, bloom, heat haze, or screen fades (spec §4.7). The settings file can turn
    /// them on too; this flag always wins.
    #[arg(long)]
    pub reduced_effects: bool,

    /// Window size in physical pixels, e.g. `1280x720`.
    #[arg(long, value_parser = parse_size, default_value = "1280x720")]
    pub window: UVec2,

    /// Override the display scale factor (e.g. `1` for reproducible screenshots).
    #[arg(long)]
    pub scale: Option<f32>,

    /// Save a screenshot to this path once the scene has rendered, then exit.
    #[arg(long)]
    pub capture: Option<PathBuf>,

    /// Skip the main menu and start a fresh run (the old save is overwritten on the next save).
    #[arg(long)]
    pub new: bool,

    /// Enable developer features in release builds (F2 reveal map, F3 next vision profile, F4
    /// light values, F5/F9 snapshots). Always on in debug builds.
    #[arg(long)]
    pub dev: bool,

    /// Keep saves in this directory instead of the platform data directory.
    #[arg(long)]
    pub save_dir: Option<PathBuf>,

    /// Keep `settings.toml` in this directory instead of the platform config directory. Captures
    /// and replays otherwise use default settings and never write the file.
    #[arg(long)]
    pub config_dir: Option<PathBuf>,

    /// Dev: open the main menu on this screen (`main`, `new-run`, `settings`, `controls`), even
    /// with `--capture`.
    #[arg(long, value_enum)]
    pub screen: Option<crate::ui::ScreenName>,

    /// Number of frames to render before taking the `--capture` screenshot.
    #[arg(long, default_value_t = 10)]
    pub capture_after_frames: u32,
}

impl Args {
    /// The scenario to start directly.
    pub fn scenario_name(&self) -> &str {
        self.scenario.as_deref().unwrap_or("showcase")
    }

    /// Whether the game opens on the main menu. Flags that name what to run (`--scenario`,
    /// `--new`, `--replay`, `--record`, `--capture`) start it directly, so dev flows and
    /// captures need no input; `--screen` forces the menu.
    pub fn starts_in_menu(&self) -> bool {
        self.screen
            .is_some_and(|s| s != crate::ui::ScreenName::Pause)
            || (self.scenario.is_none()
                && !self.new
                && self.replay.is_none()
                && self.record.is_none()
                && self.capture.is_none())
    }

    /// Parses the process arguments; uses defaults where there is no command line.
    pub fn from_env() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::parse_from(["warlock"])
        } else {
            Self::parse()
        }
    }
}

fn parse_size(s: &str) -> Result<UVec2, String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| format!("expected WIDTHxHEIGHT, got `{s}`"))?;
    let w: u32 = w.parse().map_err(|e| format!("bad width `{w}`: {e}"))?;
    let h: u32 = h.parse().map_err(|e| format!("bad height `{h}`: {e}"))?;
    if w == 0 || h == 0 {
        return Err("window size must be non-zero".into());
    }
    Ok(UVec2::new(w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_window_size() {
        assert_eq!(parse_size("640x480"), Ok(UVec2::new(640, 480)));
        assert!(parse_size("640").is_err());
        assert!(parse_size("0x480").is_err());
    }
}
