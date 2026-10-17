//! Glyph animation descriptors (spec §4.3).
//!
//! These are **presentation-only**: the core loads and carries them so all content lives in one
//! place, but no gameplay rule reads them.

use crate::color::Rgb;
use serde::{Deserialize, Serialize};

/// How a glyph animates, declared per terrain or creature type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum GlyphAnim {
    /// Gentle bob inside the cell (mostly vertical, a little sway); phase travels across
    /// neighboring cells so ripples move through pools.
    Wave {
        #[serde(default = "default_speed")]
        speed: f32,
        #[serde(default = "default_amount")]
        amount: f32,
    },
    /// Heartbeat-style "lub-dub" scale pulse.
    Pulse {
        #[serde(default = "default_bpm")]
        bpm: f32,
        #[serde(default = "default_amount")]
        amount: f32,
    },
    /// Foreground color oscillates toward `color`.
    Shimmer {
        color: Rgb,
        #[serde(default = "default_speed")]
        speed: f32,
    },
    /// Cycles through glyph frames, e.g. `"~≈"`.
    Cycle {
        frames: String,
        #[serde(default = "default_fps")]
        fps: f32,
    },
}

fn default_speed() -> f32 {
    1.0
}

fn default_amount() -> f32 {
    0.15
}

fn default_bpm() -> f32 {
    60.0
}

fn default_fps() -> f32 {
    1.0
}

impl GlyphAnim {
    /// Every glyph this animation may display besides the base glyph.
    pub fn extra_glyphs(&self) -> impl Iterator<Item = char> + '_ {
        let frames = match self {
            GlyphAnim::Cycle { frames, .. } => frames.as_str(),
            _ => "",
        };
        frames.chars()
    }
}
