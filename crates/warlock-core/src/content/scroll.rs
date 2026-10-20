//! What reading does (`read` in an object's file, spec §8 "Scrolls"): a scroll casts a spell
//! the reader aims, with the traps' rules ([`super::spell`]), or identifies an item the reader
//! carries. Scroll types are known on sight: no labels to learn yet.
//!
//! ```toml
//! [object.read]
//! kind = "burst"        # burst, bolt, or identify
//! spell = "fireball"    # the effect presentation shows (data/effects.toml)
//! noun = "fireball"     # what hits, for messages; verb what it does (default "hits")
//! verb = "burns"
//! message = "The fireball explodes!"
//! radius = 2            # a burst's, with damage, blast, heat or frost as a trap's
//! damage = "3d6"
//! range = 8             # how far a burst flies (a bolt reaches its `length`)
//! noise = { sound = "explosion", loudness = 150 }
//! ```

use super::Content;
use super::effect::EffectShape;
use super::object::MAX_RANGE;
use super::spell::{Bolt, Burst, SpellFields, check_effect};
use super::trap::TrapNoise;
use crate::dice::Dice;
use crate::noise::MAX_LOUDNESS;
use serde::Deserialize;

/// What kind of reading data names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Burst,
    Bolt,
    Identify,
}

fn hits() -> String {
    "hits".into()
}

/// `read` as an object's file writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadSpec {
    kind: Kind,
    /// `burst`, `bolt`: the effect presentation shows (`data/effects.toml`).
    spell: Option<String>,
    /// `burst`, `bolt`: what hits ("fireball") and what it does ("burns"), for messages.
    noun: Option<String>,
    #[serde(default = "hits")]
    verb: String,
    /// Said as it is read, after "You read the scroll of fireball.".
    message: Option<String>,
    /// `burst`, `bolt`.
    damage: Option<Dice>,
    /// `burst`.
    radius: Option<i32>,
    /// `burst`: how far it flies, in cells.
    range: Option<i32>,
    /// `burst`: what it does to walls (spec §8 "Digging").
    blast: Option<u16>,
    /// `burst`: it melts the ice it reaches.
    #[serde(default)]
    heat: bool,
    /// `burst`: it freezes the water it reaches to at least this many mm of ice.
    frost: Option<u16>,
    /// `bolt`: how many cells it runs from the reader.
    length: Option<i32>,
    /// `burst`, `bolt`: the noise it makes where it goes off (a burst's center, the reader for a
    /// bolt), heard like a trap's: by everyone in earshot.
    noise: Option<TrapNoise>,
}

/// What reading a scroll does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// It casts a spell at a cell the reader aims at.
    Spell(ScrollSpell),
    /// It identifies one magic item the reader carries (spec §8 "Magic items").
    Identify,
}

/// A spell a scroll casts: what, how far it reaches, and its words and noise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollSpell {
    pub cast: Cast,
    /// How far the reader may aim it, in cells: a burst's `range`, a bolt's `length`.
    pub range: i32,
    /// What hits ("fireball") and what it does ("burns"), for messages.
    pub noun: String,
    pub verb: String,
    /// Said as it is read: "The fireball explodes!"
    pub message: Option<String>,
    pub noise: Option<TrapNoise>,
}

/// The shape of a scroll's spell, with the traps' rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cast {
    /// It flies from the reader toward the cell aimed at and bursts (spec §8 "Scrolls").
    Burst(Burst),
    /// It runs from the reader through the cell aimed at.
    Bolt(Bolt),
}

impl Reading {
    /// How far the reader may aim it; `None` for what is not aimed (identify).
    pub fn range(&self) -> Option<i32> {
        match self {
            Reading::Spell(spell) => Some(spell.range),
            Reading::Identify => None,
        }
    }

    /// The spell it casts, if it casts one.
    pub fn spell(&self) -> Option<&ScrollSpell> {
        match self {
            Reading::Spell(spell) => Some(spell),
            Reading::Identify => None,
        }
    }
}

impl ReadSpec {
    /// Checks the spec (its spell effect is checked once effects are loaded, [`check_spells`]).
    pub(super) fn resolve(self) -> Result<Reading, String> {
        let kind = self.kind;
        let spelled = matches!(kind, Kind::Burst | Kind::Bolt);
        let allowed = |field: &str, given: bool, ok: bool| {
            if given && !ok {
                Err(format!("`read`: `{field}` is not for this kind of reading"))
            } else {
                Ok(())
            }
        };
        let burst = kind == Kind::Burst;
        allowed("spell", self.spell.is_some(), spelled)?;
        allowed("noun", self.noun.is_some(), spelled)?;
        allowed("verb", self.verb != hits(), spelled)?;
        allowed("damage", self.damage.is_some(), spelled)?;
        allowed("noise", self.noise.is_some(), spelled)?;
        allowed("radius", self.radius.is_some(), burst)?;
        allowed("range", self.range.is_some(), burst)?;
        allowed("blast", self.blast.is_some(), burst)?;
        allowed("heat", self.heat, burst)?;
        allowed("frost", self.frost.is_some(), burst)?;
        allowed("length", self.length.is_some(), kind == Kind::Bolt)?;
        if self.message.as_ref().is_some_and(|m| m.trim().is_empty()) {
            return Err("`read`: `message` must not be empty".into());
        }
        if kind == Kind::Identify {
            if self.message.is_some() {
                return Err("`read`: identify says what it found, no `message`".into());
            }
            return Ok(Reading::Identify);
        }
        if let Some(noise) = self.noise
            && !(1..=MAX_LOUDNESS).contains(&noise.loudness)
        {
            return Err(format!(
                "`read`: `noise` loudness must be 1 to {MAX_LOUDNESS}"
            ));
        }
        let noun = (self.noun.clone())
            .filter(|n| !n.trim().is_empty())
            .ok_or("`read`: needs a `noun`: what hits")?;
        if self.verb.trim().is_empty() {
            return Err("`read`: `verb` must not be empty".into());
        }
        let fields = SpellFields {
            spell: self.spell,
            damage: self.damage,
            radius: self.radius,
            blast: self.blast,
            heat: self.heat,
            frost: self.frost,
            length: self.length,
        };
        let (cast, range) = match kind {
            Kind::Burst => {
                let burst = Burst::new(&fields).map_err(|m| format!("`read`: {m}"))?;
                let range = self.range.unwrap_or(0);
                if !(1..=MAX_RANGE).contains(&range) {
                    return Err(format!("`read`: `range` must be 1 to {MAX_RANGE} cells"));
                }
                (Cast::Burst(burst), range)
            }
            _ => {
                let bolt = Bolt::new(&fields).map_err(|m| format!("`read`: {m}"))?;
                let range = bolt.length;
                (Cast::Bolt(bolt), range)
            }
        };
        Ok(Reading::Spell(ScrollSpell {
            cast,
            range,
            noun,
            verb: self.verb,
            message: self.message,
            noise: self.noise,
        }))
    }
}

/// With spell effects in content, every scroll's spell names an effect of its shape (checked
/// once effects are loaded, after the objects).
pub(super) fn check_spells(content: &Content) -> Result<(), String> {
    for o in content.object_defs() {
        let Some(spell) = o.read.as_ref().and_then(Reading::spell) else {
            continue;
        };
        match &spell.cast {
            Cast::Burst(b) => check_effect(content, &b.spell, EffectShape::Burst),
            Cast::Bolt(b) => check_effect(content, &b.spell, EffectShape::Bolt),
        }
        .map_err(|m| format!("object `{}`: `read`: {m}", o.id))?;
    }
    Ok(())
}
