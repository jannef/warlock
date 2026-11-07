//! Spell effects (`assets/data/effects.toml`, spec §4.8 "Spell effects", §7): what a fireball, a
//! lightning bolt, and later other spells look like.
//!
//! **Presentation only**, like particles: the core loads and checks the file so all content
//! lives in one place, but no rule reads it. The renderer draws each perceived
//! [`crate::Event::Spell`] with its effect; the dev action `effect:ID@X,Y` reads `shape` and
//! `radius` to make such an event for captures. Without the file there are no effects to show.

use super::{ContentError, ParticleEmitter, ParticleTrigger};
use crate::color::Rgb;
use serde::Deserialize;

/// Most the renderer lets an effect dim the rest of the view: never all the way to black.
pub const MAX_DARKEN: f32 = 0.9;
/// The glyph a burst's body is drawn with behind its glyphs ([`EffectDef::fill`]).
pub const FILL_GLYPH: char = '█';

/// How an effect is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectShape {
    /// A ball over the cells around a center (a fireball): [`crate::SpellShape::Burst`].
    Burst,
    /// A line along a path (a lightning bolt): [`crate::SpellShape::Bolt`].
    Bolt,
}

/// The light an effect gives while it burns: a point light at a burst's center, or along a
/// bolt's path every other cell. Visual only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectLight {
    /// At full strength, `[r, g, b]`: 255 = fully lit, more = overbright (as in
    /// `data/lights.toml`).
    pub color: [u16; 3],
    /// Reach in cells; it falls off like gameplay light.
    pub radius: i32,
}

/// How much everything but the effect's own light dims while it peaks, and for how long.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Darken {
    /// The share of the other light taken away at the deepest (the renderer stops at
    /// [`MAX_DARKEN`]).
    pub amount: f32,
    /// From the start of the impact until the view is back to normal.
    pub seconds: f32,
}

/// The screen shake when the effect lands near the player.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shake {
    /// The largest offset, in logical pixels, when it lands on the player's cell.
    pub pixels: f32,
    /// It shakes when it lands within this many cells of the player, less the farther.
    pub radius: i32,
    /// How long the shake takes to die down.
    pub seconds: f32,
}

/// One spell effect.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectDef {
    pub id: String,
    pub shape: EffectShape,
    /// Bursts: the radius of the dev action's burst (traps and spells give their own).
    #[serde(default = "one")]
    pub radius: i32,
    /// Cells per second: a burst's projectile flying from its origin to its center, a bolt
    /// drawing out along its path.
    pub speed: f32,
    /// Bursts: seconds it takes to grow to its full size, at the start of the impact.
    #[serde(default)]
    pub grow: f32,
    /// Seconds at full strength.
    pub impact: f32,
    /// Seconds fading out after the impact.
    pub linger: f32,
    /// How many times it flashes during the impact (a bolt); 0 burns steadily (a burst).
    #[serde(default)]
    pub flashes: u32,
    /// Seconds each glyph shape lasts before another is picked.
    pub flicker: f32,
    /// Bursts: glyphs by heat, hottest first. Bolts: the line glyph its strokes are drawn with
    /// (turned along each), then the one of its forks (the same if left out).
    pub glyphs: String,
    /// Bursts: the glyph that flies from the origin to the center (when they differ).
    #[serde(default)]
    pub projectile: Option<char>,
    /// Colors, hottest (brightest) first: a burst's core and a flash at its brightest, then
    /// outward and as it cools.
    pub colors: Vec<Rgb>,
    /// How much its hottest glyphs glow (bloom), 0 to 1.
    #[serde(default)]
    pub glow: f32,
    /// Bursts: how opaque the glowing body behind each cell's glyph is at its hottest, 0 (none)
    /// to 1: a solid block a shade cooler than the glyph, thinning out toward the edge.
    #[serde(default)]
    pub fill: f32,
    /// Bolts: the chance, per cell and flash, of a short fork off the line, 0 to 1.
    #[serde(default)]
    pub branches: f32,
    #[serde(default)]
    pub light: Option<EffectLight>,
    #[serde(default)]
    pub darken: Option<Darken>,
    #[serde(default)]
    pub shake: Option<Shake>,
    /// `flight` (from a burst's projectile), `impact`, and `linger` emitters.
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
}

fn one() -> i32 {
    1
}

impl EffectDef {
    /// Every glyph it may draw (its body's fill is the full block, `█`).
    pub fn glyphs(&self) -> impl Iterator<Item = char> + '_ {
        (self.glyphs.chars())
            .chain((self.fill > 0.0).then_some(FILL_GLYPH))
            .chain(self.projectile)
            .chain(self.particles.iter().flat_map(|e| e.glyphs.chars()))
    }

    /// Checks that it is well formed, returning what is wrong: an id action text can name,
    /// times that are numbers, a projectile that arrives, colors and glyphs to draw with, and no
    /// field its shape would ignore.
    fn check(&self) -> Result<(), String> {
        let bad_id = |c: char| matches!(c, ':' | '@' | '>') || c.is_whitespace();
        if self.id.is_empty() || self.id.contains(bad_id) {
            return Err("an effect id is a word without `:`, `@`, `>`, or spaces".into());
        }
        let times = [
            self.impact,
            self.linger,
            self.flicker,
            self.grow,
            self.darken.map_or(0.0, |d| d.seconds),
            self.shake.map_or(0.0, |s| s.seconds),
        ];
        if times.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err("times are seconds, 0 or more".into());
        }
        if !(self.speed.is_finite() && self.speed > 0.0) {
            return Err("`speed` must be above 0 cells a second".into());
        }
        if self.colors.is_empty() {
            return Err("an effect needs at least one color".into());
        }
        let glyphs = self.glyphs.chars().count();
        match self.shape {
            EffectShape::Burst => {
                if glyphs == 0 {
                    return Err("a burst needs at least one glyph".into());
                }
            }
            EffectShape::Bolt => {
                if !(1..=2).contains(&glyphs) {
                    return Err("a bolt has one or two glyphs: its line, then its forks".into());
                }
                if self.projectile.is_some() || self.fill > 0.0 {
                    return Err("a bolt has no `projectile` or `fill`".into());
                }
            }
        }
        for e in &self.particles {
            e.check().map_err(|m| format!("particles: {m}"))?;
            let fits = match e.on {
                ParticleTrigger::Impact | ParticleTrigger::Linger => true,
                ParticleTrigger::Flight => self.projectile.is_some(),
                _ => false,
            };
            if !fits {
                return Err(format!(
                    "particles: an effect has `impact` and `linger` emitters, and `flight` \
                     ones with a `projectile`, not `{}`",
                    e.on.name()
                ));
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectFile {
    #[serde(default)]
    effect: Vec<EffectDef>,
}

/// Parses and checks `data/effects.toml`: the effects in file order, ids unique.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<EffectDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: EffectFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    for (i, effect) in parsed.effect.iter().enumerate() {
        effect
            .check()
            .map_err(|m| err(format!("effect `{}`: {m}", effect.id)))?;
        if parsed.effect[..i].iter().any(|e| e.id == effect.id) {
            return Err(err(format!("effect `{}` is defined twice", effect.id)));
        }
    }
    Ok(parsed.effect)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BALL: &str = r##"
[[effect]]
id = "ball"
shape = "burst"
radius = 2
speed = 20
grow = 0.1
impact = 0.2
linger = 0.4
flicker = 0.05
glyphs = "▓▒░"
projectile = "*"
colors = ["#ffffff", "#ff8000"]
light = { color = [800, 400, 100], radius = 6 }
darken = { amount = 0.6, seconds = 0.3 }
shake = { pixels = 4, radius = 6, seconds = 0.25 }

[[effect.particles]]
on = "impact"
glyphs = "'"
colors = ["#ffc040"]
count = 2
life = 0.5
"##;

    const BOLT: &str = r##"
[[effect]]
id = "bolt"
shape = "bolt"
speed = 200
impact = 0.3
linger = 0.2
flashes = 3
flicker = 0.04
glyphs = "━─"
colors = ["#ffffff", "#80a0ff"]
branches = 0.3
"##;

    #[test]
    fn parses_bursts_and_bolts_with_defaults() {
        let effects = parse("effects", &format!("{BALL}{BOLT}")).expect("valid");
        let (ball, bolt) = (&effects[0], &effects[1]);
        assert_eq!((ball.shape, ball.radius), (EffectShape::Burst, 2));
        assert_eq!(ball.light.map(|l| l.color), Some([800, 400, 100]));
        assert_eq!(ball.particles[0].on, ParticleTrigger::Impact);
        assert_eq!(ball.glyphs().collect::<String>(), "▓▒░*'");
        assert_eq!(
            (bolt.shape, bolt.radius, bolt.grow),
            (EffectShape::Bolt, 1, 0.0)
        );
        assert_eq!((bolt.light, bolt.darken, bolt.shake), (None, None, None));
        assert!(parse("effects", "").expect("no effects").is_empty());
    }

    #[test]
    fn rejects_bad_effects() {
        for (bad, why) in [
            (format!("{BALL}{BALL}"), "twice"),
            (BALL.replace("\"ball\"", "\"fire ball\""), "effect id"),
            (BALL.replace("\"ball\"", "\"ball@2\""), "effect id"),
            (BALL.replace("speed = 20", "speed = 0"), "speed"),
            (BALL.replace("linger = 0.4", "linger = -1"), "seconds"),
            (BALL.replace("seconds = 0.25", "seconds = nan"), "seconds"),
            (BALL.replace("glyphs = \"▓▒░\"", "glyphs = \"\""), "glyph"),
            (
                BALL.replace("colors = [\"#ffffff\", \"#ff8000\"]", "colors = []"),
                "color",
            ),
            (
                BALL.replace("on = \"impact\"", "on = \"step\""),
                "particles",
            ),
            (
                BALL.replace("projectile = \"*\"\n", "")
                    .replace("\"impact\"", "\"flight\""),
                "particles",
            ),
            (BOLT.replace("━─", "━─│"), "one or two glyphs"),
            (BOLT.replace("\"━─\"", "\"\""), "one or two glyphs"),
            (format!("{BOLT}projectile = \"*\"\n"), "projectile"),
            (format!("{BOLT}sparkle = 1\n"), "unknown field"),
        ] {
            let e = parse("effects", &bad).expect_err(why);
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn the_builtin_effects_are_a_fireball_and_a_lightning_bolt() {
        let content = super::super::Content::builtin().expect("builtin content");
        let fireball = content.effect("fireball").expect("a fireball");
        assert_eq!(fireball.shape, EffectShape::Burst);
        assert!(fireball.darken.is_some() && fireball.shake.is_some() && fireball.light.is_some());
        let bolt = content.effect("lightning_bolt").expect("a lightning bolt");
        assert_eq!(bolt.shape, EffectShape::Bolt);
        assert!(
            (2..=3).contains(&bolt.flashes),
            "it flickers two or three times"
        );
        // Their glyphs are in the atlas.
        let glyphs = content.glyphs();
        assert!(
            fireball
                .glyphs()
                .chain(bolt.glyphs())
                .all(|c| glyphs.contains(&c))
        );
    }
}
