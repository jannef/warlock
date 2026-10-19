//! Creature definitions (`assets/data/creatures.toml`).

use super::anim::GlyphAnim;
use super::faction::FactionId;
use super::light::LightId;
use super::object::MAX_BONUS;
use super::particles::ParticleEmitter;
use super::status::{OnHit, OnHitSpec, StatusId};
use crate::color::Rgb;
use crate::combat::{Attack, CRITS, DEFAULT_CRIT, DEFAULT_THREAT, THREATS};
use crate::dice::Dice;
use crate::skills::Skills;
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

fn first_level() -> u8 {
    1
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

/// How a creature fights (spec §8 "Creature behavior"): up close, shooting with the missile
/// weapon it has ready, or casting (spells are intents only: nothing is damaged).
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
    /// Its size (`data/sizes.toml`); none: medium.
    pub size: Option<String>,
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
    /// What its attacks cost, in energy; none: the attack's cost in `data/actions.toml`.
    pub attack_cost: Option<u32>,
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
    /// Hit points beyond its Toughness (spec §8 "Combat").
    #[serde(default)]
    pub base_hp: u16,
    /// Its level (spec §8 "Experience"), 1 unless it says otherwise.
    #[serde(default = "first_level")]
    pub level: u8,
    /// The experience a kill of it is worth at its own level; none: nothing.
    #[serde(default)]
    pub xp: u32,
    /// Its trained skills by skill id, with their values (spec §8 "Skills"); the rest untrained.
    #[serde(default)]
    pub skills: BTreeMap<String, u16>,
    /// Its natural attack, used when it wields no weapon.
    pub attack: Option<NaturalAttackSpec>,
    /// Its natural armour: hide, shell, or scales.
    pub natural: Option<Hide>,
    pub anim: Option<GlyphAnim>,
    #[serde(default)]
    pub glow: f32,
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
}

/// A natural attack as written in content files: `attack = { damage = "1d4", name = "bite",
/// verb = "bites" }`, with an optional `threat`, `crit`, and `to_hit` like a weapon's, and the
/// status effects a hit may cause (`on_hit`, `data/status.toml`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NaturalAttackSpec {
    pub damage: Dice,
    pub name: String,
    pub verb: String,
    pub threat: Option<u8>,
    pub crit: Option<u8>,
    #[serde(default)]
    pub to_hit: i32,
    #[serde(default)]
    pub on_hit: Vec<OnHitSpec>,
}

/// A creature's natural attack (spec §8 "Combat"): what it does, and the words for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NaturalAttack {
    pub attack: Attack,
    /// What it is called: "bite" ("You dodge the giant rat's bite.").
    pub name: String,
    /// What it does to its target: "bites" ("The giant rat bites you.").
    pub verb: String,
    /// The status effects a hit with it may cause, in data order (spec §8 "Status effects"): a
    /// giant rat's bite may poison.
    pub on_hit: Vec<OnHit>,
}

/// A creature's natural armour: `natural = { defence = 3, name = "shell" }`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hide {
    /// Its bonus to Defence (the Natural category).
    pub defence: i32,
    /// What it is called: "shell" ("Your sword glances off the cave beetle's shell.").
    pub name: String,
}

impl NaturalAttackSpec {
    /// Checks the attack like a weapon's and resolves its defaults; `status` resolves a status
    /// id to the status and its `max`.
    pub(super) fn resolve(
        self,
        status: impl Fn(&str) -> Option<(StatusId, u16)>,
    ) -> Result<NaturalAttack, String> {
        let threat = self.threat.unwrap_or(DEFAULT_THREAT);
        let crit = self.crit.unwrap_or(DEFAULT_CRIT);
        if !THREATS.contains(&threat) || !CRITS.contains(&crit) {
            return Err(format!(
                "attack: `threat` must be {} to {} and `crit` {} to {}",
                THREATS.start(),
                THREATS.end(),
                CRITS.start(),
                CRITS.end()
            ));
        }
        if self.name.trim().is_empty() || self.verb.trim().is_empty() {
            return Err("attack: `name` and `verb` must not be empty".into());
        }
        if self.to_hit.abs() > MAX_BONUS {
            return Err(format!("attack: `to_hit` must be within ±{MAX_BONUS}"));
        }
        let on_hit = (self.on_hit.iter())
            .map(|o| o.resolve(&status).map_err(|m| format!("attack: {m}")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NaturalAttack {
            attack: Attack {
                damage: self.damage,
                to_hit: self.to_hit,
                threat,
                crit,
                strength: None,
            },
            name: self.name,
            verb: self.verb,
            on_hit,
        })
    }
}

/// What a creature type's names resolve to when content loads.
pub(super) struct Resolved {
    pub light: Option<LightId>,
    pub faction: Option<FactionId>,
    pub vision: usize,
    pub size: usize,
    pub stats: Stats,
    pub skills: Skills,
    pub attack: Option<NaturalAttack>,
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
            size: r.size,
            faction: r.faction,
            vision: r.vision,
            sight: self.sight,
            combat: self.combat,
            range: self.range.unwrap_or(1),
            opens_doors: self.opens_doors,
            attack_cost: self.attack_cost,
            light: r.light,
            footsteps: self.footsteps,
            hearing: self.hearing,
            stats: r.stats,
            skills: r.skills,
            base_hp: i32::from(self.base_hp),
            level: self.level,
            xp: self.xp,
            attack: r.attack,
            natural: self.natural,
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
    /// Gameplay: its size, an index into [`super::Content::sizes`] (spec §8 "Burden"): how much
    /// it can carry before it is burdened. The player's comes from their race, if the run has a
    /// character.
    pub size: usize,
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
    /// What its attacks (melee, shooting, or casting) cost in energy, if its type says
    /// ([`super::Content::action_cost`]); `None`: the cost of the attack in `data/actions.toml`.
    /// Rules only.
    pub attack_cost: Option<u32>,
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
    /// Gameplay: the skills a creature of this type is trained in (spec §8 "Skills"); every
    /// other skill is untrained. Rules only: a creature's skills are never shown to the player.
    /// The player's come from their race and class.
    pub skills: Skills,
    /// Gameplay: hit points beyond its Toughness (spec §8 "Combat"); the player's come from the
    /// scenario instead.
    pub base_hp: i32,
    /// Gameplay: its level (spec §8 "Experience"); the player's own is their experience's.
    pub level: u8,
    /// Gameplay: the experience a kill of it is worth at its own level.
    pub xp: u32,
    /// Gameplay: its natural attack, used when it wields no weapon; `None`: bare hands (1d3).
    pub attack: Option<NaturalAttack>,
    /// Gameplay: its natural armour, if any.
    pub natural: Option<Hide>,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: glyph particle emitters that move with the creature.
    pub particles: Vec<ParticleEmitter>,
}
