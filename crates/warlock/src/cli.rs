//! Command-line options of the game binary (see spec §9).

use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// Options parsed from the command line. On the web build there is no command line, so the
/// defaults are used.
#[derive(Debug, Clone, Parser, Resource)]
#[command(name = "warlock", version, about = "Warlock — an ASCII roguelike")]
pub struct Args {
    /// Window size in physical pixels, e.g. `1280x720`.
    #[arg(long, value_parser = parse_size, default_value = "1280x720")]
    pub window: UVec2,

    /// Override the display scale factor (e.g. `1` for reproducible screenshots).
    #[arg(long)]
    pub scale: Option<f32>,

    /// Save a screenshot to this path once the scene has rendered, then exit.
    #[arg(long)]
    pub capture: Option<PathBuf>,

    /// Number of frames to render before taking the `--capture` screenshot.
    #[arg(long, default_value_t = 10)]
    pub capture_after_frames: u32,
}

impl Args {
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
