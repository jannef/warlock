//! Creature definitions (`assets/data/creatures.toml`).

use super::anim::GlyphAnim;
use super::light::LightId;
use super::particles::ParticleEmitter;
use crate::color::Rgb;
use serde::{Deserialize, Serialize};

/// Index of a creature definition in [`super::Content::creatures`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CreatureId(pub u16);

/// A creature type as written in content files.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreatureSpec {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub fg: Rgb,
    /// Light preset the creature carries.
    pub light: Option<String>,
    pub anim: Option<GlyphAnim>,
    #[serde(default)]
    pub glow: f32,
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
}

impl CreatureSpec {
    pub(super) fn resolve(self, light: Option<LightId>) -> CreatureDef {
        CreatureDef {
            id: self.id,
            name: self.name,
            glyph: self.glyph,
            fg: self.fg,
            light,
            anim: self.anim,
            glow: self.glow,
            particles: self.particles,
        }
    }
}

/// A creature type. The player is the creature with id `"player"`.
#[derive(Clone, Debug)]
pub struct CreatureDef {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub fg: Rgb,
    /// Light the creature carries with it (e.g. a torch), if any.
    pub light: Option<LightId>,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: glyph particle emitters that move with the creature.
    pub particles: Vec<ParticleEmitter>,
}
