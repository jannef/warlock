//! Glyph particle emitters (spec §4.8).
//!
//! These are **presentation-only**, like [`super::GlyphAnim`]: the core loads and validates them
//! so all content lives in one place, but no gameplay rule reads them. The renderer evaluates
//! particles from visual time with its own randomness.

use crate::color::Rgb;
use serde::{Deserialize, Serialize};

/// When an emitter produces particles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParticleTrigger {
    /// Continuously, `rate` particles per second, while the source is in view.
    #[default]
    Ambient,
    /// A burst of `count` particles when a creature is seen stepping into a cell of this terrain
    /// (a splash in water).
    Step,
    /// A burst of `count` particles when a cell is seen turning into this terrain (dust when a
    /// door opens or closes).
    Toggle,
    /// Continuously, `rate` particles per second, from running water whose flow runs into
    /// anything but running water: foam at a bank, ripples where a stream enters a pool. The
    /// emitter is turned with the flow: its `y` points downstream.
    Obstacle,
    /// Creatures only: a burst of `count` particles at the target when a creature of this type
    /// is seen attacking it (a spell's sparkle).
    Attack,
}

impl ParticleTrigger {
    /// The trigger as written in content files.
    pub const fn name(self) -> &'static str {
        match self {
            ParticleTrigger::Ambient => "ambient",
            ParticleTrigger::Step => "step",
            ParticleTrigger::Toggle => "toggle",
            ParticleTrigger::Obstacle => "obstacle",
            ParticleTrigger::Attack => "attack",
        }
    }
}

/// A source of glyph particles on a terrain or creature type. Positions are in cells: x right,
/// y down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticleEmitter {
    #[serde(default)]
    pub on: ParticleTrigger,
    /// Glyphs to pick from, one per particle.
    pub glyphs: String,
    /// Color over the particle's life: evenly spaced stops from birth to death.
    pub colors: Vec<Rgb>,
    /// Particles per second (`ambient`, `obstacle`).
    #[serde(default)]
    pub rate: f32,
    /// Particles per burst (`step`, `toggle`).
    #[serde(default)]
    pub count: u32,
    /// Lifetime in seconds; each particle lives between 70 % and 100 % of it.
    pub life: f32,
    /// Initial velocity in cells per second.
    #[serde(default)]
    pub velocity: [f32; 2],
    /// Random variation of the velocity, ± per axis.
    #[serde(default)]
    pub spread: [f32; 2],
    /// Downward acceleration in cells per second²; negative values make particles rise.
    #[serde(default)]
    pub gravity: f32,
    /// Where particles start relative to the cell, in cells (e.g. `[0, -0.4]`: above the glyph).
    #[serde(default)]
    pub offset: [f32; 2],
    /// Random spawn offset around that point, ± per axis, in cells.
    #[serde(default)]
    pub area: [f32; 2],
    /// Sideways drift (a slow sine), in cells.
    #[serde(default)]
    pub sway: f32,
    /// Glyph size relative to a cell.
    #[serde(default = "one")]
    pub scale: f32,
    /// Fraction of the life spent fading out at the end.
    #[serde(default = "half")]
    pub fade: f32,
    /// Emissive strength: how much the particle glows (bloom), 0 to 1.
    #[serde(default)]
    pub glow: f32,
    /// How opaque the particle is at most, 0 to 1 (default 1): faint particles such as rain.
    #[serde(default = "one")]
    pub opacity: f32,
}

fn one() -> f32 {
    1.0
}

fn half() -> f32 {
    0.5
}

impl ParticleEmitter {
    /// Checks that the emitter can produce particles.
    pub fn check(&self) -> Result<(), String> {
        let finite = [
            self.rate,
            self.life,
            self.velocity[0],
            self.velocity[1],
            self.spread[0],
            self.spread[1],
            self.gravity,
            self.offset[0],
            self.offset[1],
            self.area[0],
            self.area[1],
            self.sway,
            self.scale,
            self.fade,
            self.glow,
            self.opacity,
        ];
        if finite.iter().any(|v| !v.is_finite()) {
            return Err("particle values must be finite numbers".into());
        }
        if self.glyphs.is_empty() {
            return Err("particles need at least one glyph".into());
        }
        if self.colors.is_empty() {
            return Err("particles need at least one color".into());
        }
        if self.life <= 0.0 || self.scale <= 0.0 {
            return Err("particle `life` and `scale` must be positive".into());
        }
        if [self.fade, self.glow, self.opacity]
            .iter()
            .any(|v| !(0.0..=1.0).contains(v))
        {
            return Err("particle `fade`, `glow`, and `opacity` must be between 0 and 1".into());
        }
        match self.on {
            ParticleTrigger::Ambient if self.rate <= 0.0 => {
                Err("ambient particles need a positive `rate`".into())
            }
            ParticleTrigger::Obstacle if self.rate <= 0.0 => {
                Err("obstacle particles need a positive `rate`".into())
            }
            ParticleTrigger::Step | ParticleTrigger::Toggle | ParticleTrigger::Attack
                if self.count == 0 =>
            {
                Err("particle bursts need a positive `count`".into())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> ParticleEmitter {
        toml::from_str(text).expect("emitter parses")
    }

    #[test]
    fn defaults_make_an_ambient_emitter() {
        let e = parse("glyphs = \"'`\"\ncolors = [\"#ffa040\"]\nrate = 4\nlife = 0.8\n");
        assert_eq!(e.on, ParticleTrigger::Ambient);
        assert_eq!(e.scale, 1.0);
        assert_eq!(e.fade, 0.5);
        assert_eq!(e.opacity, 1.0);
        assert_eq!(e.velocity, [0.0, 0.0]);
        assert_eq!(e.check(), Ok(()));
    }

    #[test]
    fn checks_reject_emitters_that_cannot_emit() {
        let base = "glyphs = \".\"\ncolors = [\"#ffffff\"]\nlife = 1\n";
        assert!(parse(base).check().is_err(), "ambient without a rate");
        assert!(parse(&format!("{base}on = \"step\"\n")).check().is_err());
        assert_eq!(
            parse(&format!("{base}on = \"toggle\"\ncount = 3\n")).check(),
            Ok(())
        );
        let no_glyphs = "glyphs = \"\"\ncolors = [\"#ffffff\"]\nlife = 1\nrate = 1\n";
        assert!(parse(no_glyphs).check().is_err());
        let no_colors = "glyphs = \".\"\ncolors = []\nlife = 1\nrate = 1\n";
        assert!(parse(no_colors).check().is_err());
        assert!(
            parse(&format!("{base}rate = 1\nglow = 2\n"))
                .check()
                .is_err()
        );
        assert!(
            parse(&format!("{base}rate = 1\nopacity = 1.5\n"))
                .check()
                .is_err()
        );
        assert!(toml::from_str::<ParticleEmitter>(&format!("{base}speed = 1\n")).is_err());
    }
}
