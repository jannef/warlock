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
    /// neighboring cells so ripples move through pools. Still water.
    Wave {
        #[serde(default = "default_speed")]
        speed: f32,
        #[serde(default = "default_amount")]
        amount: f32,
        /// Strength (0 to 1) of occasional glints: brief brightenings of the glyph in the color
        /// of the light on the cell, so none in the dark. 0 (the default): no glints.
        #[serde(default)]
        glint: f32,
    },
    /// Running water: the glyph moves downstream along the cell's flow direction (set per cell by
    /// the level, see [`super::Flow`]), leaving the cell on one side and coming back in on the
    /// other, so a stream of it reads as one moving surface. `speed` is in cell widths per second
    /// (on each axis the flow moves along), times the cell's flow speed. `glint` as for `wave`.
    Flow {
        #[serde(default = "default_speed")]
        speed: f32,
        #[serde(default)]
        glint: f32,
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
    /// Whether this is running water's animation, which follows each cell's [`super::Flow`].
    pub fn is_flow(&self) -> bool {
        matches!(self, GlyphAnim::Flow { .. })
    }

    /// Checks the values a content file gave.
    pub fn check(&self) -> Result<(), String> {
        let (values, glint) = match self {
            GlyphAnim::Wave {
                speed,
                amount,
                glint,
            } => ([*speed, *amount], *glint),
            GlyphAnim::Flow { speed, glint } => ([*speed, 0.0], *glint),
            GlyphAnim::Pulse { bpm, amount } => ([*bpm, *amount], 0.0),
            GlyphAnim::Shimmer { speed, .. } => ([*speed, 0.0], 0.0),
            GlyphAnim::Cycle { fps, .. } => ([*fps, 0.0], 0.0),
        };
        if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err("animation values must be finite and not negative".into());
        }
        if !(0.0..=1.0).contains(&glint) {
            return Err("animation `glint` must be between 0 and 1".into());
        }
        Ok(())
    }

    /// Every glyph this animation may display besides the base glyph.
    pub fn extra_glyphs(&self) -> impl Iterator<Item = char> + '_ {
        let frames = match self {
            GlyphAnim::Cycle { frames, .. } => frames.as_str(),
            _ => "",
        };
        frames.chars()
    }
}
