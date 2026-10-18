//! Creature definitions (`assets/data/creatures.toml`).

use super::anim::GlyphAnim;
use super::faction::FactionId;
use super::light::LightId;
use super::particles::ParticleEmitter;
use crate::color::Rgb;
use crate::stats::{Stat, Stats};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The speed of a normal creature, and of the player (spec §8 "Speed").
pub const NORMAL_SPEED: u32 = 1000;
/// The speeds a creature type may have: a tenth of normal to ten times normal.
pub const SPEEDS: RangeInclusive<u32> = 100..=10_000;
/// How far a creature sees unless its type says otherwise, in cells (spec §5 "Creature sight").
pub const DEFAULT_SIGHT: i32 = 10;

fn normal_speed() -> u32 {
    NORMAL_SPEED
}

fn default_sight() -> i32 {
    DEFAULT_SIGHT
}

fn normal_vision() -> String {
    "normal".into()
}

/// Index of a creature definition in [`super::Content::creatures`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CreatureId(pub u16);

/// How a creature fights (spec §8 "Creature behavior"). Attacks are intents only: nothing is
/// damaged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Combat {
    /// Closes in and attacks from an adjacent cell.
    #[default]
    Melee,
    /// Shoots from up to its `range` away, with a clear line of fire.
    Ranged,
    /// Casts spells from up to its `range` away, with a clear line of fire.
    Caster,
}

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
    /// Base weight in grams, without anything carried.
    pub weight: u32,
    /// The faction it belongs to (`data/factions.toml`).
    pub faction: Option<String>,
    /// Its eyes: a vision profile in `data/vision.toml`.
    #[serde(default = "normal_vision")]
    pub vision: String,
    /// How far it sees, in cells.
    #[serde(default = "default_sight")]
    pub sight: i32,
    #[serde(default)]
    pub combat: Combat,
    /// How far a `ranged` or `caster` creature reaches, in cells.
    pub range: Option<i32>,
    /// Whether it opens closed doors in its way.
    #[serde(default)]
    pub opens_doors: bool,
    /// Light preset the creature carries.
    pub light: Option<String>,
    /// How loud its steps are (spec §5 "Hearing"); none: it moves silently.
    #[serde(default)]
    pub footsteps: u16,
    /// The faintest loudness it hears; none: it is deaf.
    pub hearing: Option<u16>,
    /// Its scores by stat (spec §8 "Stats"); a missing one is average.
    #[serde(default)]
    pub stats: BTreeMap<Stat, u8>,
    /// Its potentials by stat; a missing one equals the score.
    #[serde(default)]
    pub potential: BTreeMap<Stat, u8>,
    pub anim: Option<GlyphAnim>,
    #[serde(default)]
    pub glow: f32,
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
}

/// What a creature type's names resolve to when content loads.
pub(super) struct Resolved {
    pub light: Option<LightId>,
    pub faction: Option<FactionId>,
    pub vision: usize,
    pub stats: Stats,
}

impl CreatureSpec {
    pub(super) fn resolve(self, r: Resolved) -> CreatureDef {
        CreatureDef {
            id: self.id,
            name: self.name,
            glyph: self.glyph,
            fg: self.fg,
            speed: self.speed,
            weight: self.weight,
            faction: r.faction,
            vision: r.vision,
            sight: self.sight,
            combat: self.combat,
            range: self.range.unwrap_or(1),
            opens_doors: self.opens_doors,
            light: r.light,
            footsteps: self.footsteps,
            hearing: self.hearing,
            stats: r.stats,
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
    /// Gameplay: the creature's own weight in grams, without what it carries (see
    /// [`crate::object::Weight`]). At least 1.
    pub weight: u32,
    /// The faction it belongs to; `None` when the content defines no factions. The player's
    /// creature belongs to [`super::PLAYER_FACTION`].
    pub faction: Option<FactionId>,
    /// Its eyes: an index into [`super::Content::vision_profiles`]. The player's eyes come from
    /// the scenario (and the dev vision switch) instead.
    pub vision: usize,
    /// How far it sees, in cells (a Euclidean radius, like every radius).
    pub sight: i32,
    /// How it fights. Rules only, like the rest of its behavior.
    pub combat: Combat,
    /// How far its attacks reach, in cells: 1 for melee.
    pub range: i32,
    /// Whether it opens closed doors in its way.
    pub opens_doors: bool,
    /// Light the creature carries with it (e.g. a torch), if any.
    pub light: Option<LightId>,
    /// Gameplay: the loudness of its steps on terrain with a `step_noise` of 100
    /// ([`crate::noise::noise`]); 0 moves silently. At most [`crate::noise::MAX_LOUDNESS`].
    pub footsteps: u16,
    /// Gameplay: the faintest loudness it hears (1 to [`crate::noise::MAX_LOUDNESS`]); `None` is
    /// deaf.
    pub hearing: Option<u16>,
    /// Gameplay: the scores and potentials a creature of this type starts with (spec §8
    /// "Stats"). Rules only: a creature's stats are never shown to the player. The player's
    /// come from the scenario instead ([`super::ScenarioDef::player_stats`]); theirs are average.
    pub stats: Stats,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: glyph particle emitters that move with the creature.
    pub particles: Vec<ParticleEmitter>,
}
