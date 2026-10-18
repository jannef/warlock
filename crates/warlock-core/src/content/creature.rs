//! Creature definitions (`assets/data/creatures.toml`).

use super::anim::GlyphAnim;
use super::light::LightId;
use super::particles::ParticleEmitter;
use crate::color::Rgb;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// The speed of a normal creature, and of the player (spec §8 "Speed").
pub const NORMAL_SPEED: u32 = 1000;
/// The speeds a creature type may have: a tenth of normal to ten times normal.
pub const SPEEDS: RangeInclusive<u32> = 100..=10_000;

fn normal_speed() -> u32 {
    NORMAL_SPEED
}

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
    #[serde(default = "normal_speed")]
    pub speed: u32,
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
            speed: self.speed,
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
    /// How quickly it acts ([`NORMAL_SPEED`] is normal; twice that, twice as many actions in the
    /// same game time). Rules only: never shown to the player, nor put in the perception snapshot.
    pub speed: u32,
    /// Light the creature carries with it (e.g. a torch), if any.
    pub light: Option<LightId>,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: glyph particle emitters that move with the creature.
    pub particles: Vec<ParticleEmitter>,
}
