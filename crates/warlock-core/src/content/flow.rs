//! Flow of running water (spec §4.3, §7): which way and how fast the water on a cell moves.
//!
//! **Presentation-only**, like [`super::GlyphAnim`]: levels carry it (set in the legend, see
//! `assets/data/legend.toml`) and the renderer moves running water's ripples along it, but no
//! gameplay rule reads it.

use crate::geom::Dir;
use serde::{Deserialize, Serialize};

/// Slowest and fastest flow speed a level may give, in percent.
pub const FLOW_SPEEDS: std::ops::RangeInclusive<u16> = 10..=300;

/// The flow on one cell of running water.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    /// Which way the water moves.
    pub dir: Dir,
    /// How fast, in percent of the terrain's `flow` animation speed (100 = as declared).
    pub speed: u16,
}

impl Flow {
    pub const fn new(dir: Dir, speed: u16) -> Self {
        Self { dir, speed }
    }
}
