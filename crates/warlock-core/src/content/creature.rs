//! Creature definitions (`assets/data/creatures.toml`).

use super::anim::GlyphAnim;
use crate::color::Rgb;
use serde::{Deserialize, Serialize};

/// Index of a creature definition in [`super::Content::creatures`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CreatureId(pub u16);

/// A creature type. The player is the creature with id `"player"`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureDef {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub fg: Rgb,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
}
