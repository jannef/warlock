//! Trap definitions (`assets/data/traps.toml`, spec §7, §8 "Traps").
//!
//! Every trap type has an `id`, a `name` ("fireball trap"), the glyph it shows as once known
//! (`^` by default) and its color, an optional `message` said when the player perceives it go off
//! ("A fireball explodes!"), optional `charges` (how often it goes off before it is spent; none:
//! it never runs out), an optional `noise` it makes going off (`{ sound = "explosion", loudness
//! = 150 }`, heard like any noise, spec §5 "Hearing"), and what it does (`kind`):
//!
//! - `shot`: an `object` (an arrow) shoots out of a hidden slit in the nearest wall at whoever
//!   set it off: an attack roll with the trap's `to_hit` and `damage` against their Defence; the
//!   missile then lies where it came down, like any shot's;
//! - `strike`: blades or spears from the floor: an attack roll (`to_hit`, `damage`) against the
//!   Defence of whoever stands on the trap, made with what `noun` names;
//! - `burst`: a blast of `radius` cells around the trap (a fireball): everyone in it takes
//!   `damage`, no roll to hit, half of it (at least 1) beyond the inner cells; with a `blast`, it
//!   breaks every wall in it whose hardness is below that (spec §8 "Digging");
//! - `bolt`: a line from the trap in one direction, up to `length` cells or the first wall (a
//!   lightning bolt): everyone on it takes `damage`;
//! - `pit`: whoever steps on it falls in, takes `damage`, and loses their next action climbing
//!   out;
//! - `alarm`: nothing but its noise (a bell that draws creatures);
//! - `teleport`: whoever steps on it is moved to a free cell of the level.
//!
//! `burst` and `bolt` name the `spell` effect presentation shows (`data/effects.toml`: what a
//! fireball looks like); `strike`, `burst`, and `bolt` name what hits (`noun`, "fireball") and
//! what it does (`verb`, default "hits"), for messages ("The fireball burns the kobold.").
//!
//! A trap on a level ([`Trap`]) is hidden until the player sets it off or sees a creature do it,
//! and from then on known and remembered (spec §8 "Traps").

use super::object::{MAX_BONUS, ObjectDef, ObjectId};
use super::terrain::MAX_HARDNESS;
use super::{Content, ContentError};
use crate::color::Rgb;
use crate::combat::{Attack, DEFAULT_CRIT, DEFAULT_THREAT};
use crate::dice::Dice;
use crate::geom::Dir;
use crate::noise::{MAX_LOUDNESS, SoundKind};
use crate::perception::VIEW_RADIUS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// Index of a trap definition in [`super::Content::trap_defs`]. Ids are assigned in file order,
/// so the content fingerprint covers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrapId(pub u16);

/// The radii a burst may have, in cells.
pub const BURST_RADII: RangeInclusive<i32> = 1..=5;
/// The lengths a bolt may have, in cells (the trap's own cell is the first).
pub const BOLT_LENGTHS: RangeInclusive<i32> = 1..=VIEW_RADIUS;
/// The most charges a trap may have.
pub const MAX_CHARGES: u16 = 1000;

fn trap_glyph() -> char {
    '^'
}

fn hits() -> String {
    "hits".into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrapFile {
    pub trap: Vec<TrapSpec>,
}

/// What kind of thing a trap does, as data names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Shot,
    Strike,
    Burst,
    Bolt,
    Pit,
    Alarm,
    Teleport,
}

impl Kind {
    const fn name(self) -> &'static str {
        match self {
            Kind::Shot => "shot",
            Kind::Strike => "strike",
            Kind::Burst => "burst",
            Kind::Bolt => "bolt",
            Kind::Pit => "pit",
            Kind::Alarm => "alarm",
            Kind::Teleport => "teleport",
        }
    }
}

/// The noise a trap makes going off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrapNoise {
    /// What it sounds like: `bowstring`, `explosion`, `thunder`, `bell`, ...
    pub sound: SoundKind,
    /// How loud it is on the trap's cell, 1 to [`MAX_LOUDNESS`].
    pub loudness: u16,
}

/// A trap type as written in `data/traps.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrapSpec {
    id: String,
    name: String,
    #[serde(default = "trap_glyph")]
    glyph: char,
    fg: Rgb,
    message: Option<String>,
    charges: Option<u16>,
    noise: Option<TrapNoise>,
    kind: Kind,
    /// `shot`: the object that flies (and lands).
    object: Option<String>,
    /// `shot`, `strike`, `burst`, `bolt`, `pit`.
    damage: Option<Dice>,
    /// `shot`, `strike`.
    #[serde(default)]
    to_hit: i32,
    /// `burst`.
    radius: Option<i32>,
    /// `burst`: how strong the explosion is against walls (spec §8 "Digging"); none breaks none.
    blast: Option<u16>,
    /// `bolt`.
    length: Option<i32>,
    /// `burst`, `bolt`: the effect presentation shows (`data/effects.toml`).
    spell: Option<String>,
    /// `strike`, `burst`, `bolt`: what hits.
    noun: Option<String>,
    #[serde(default = "hits")]
    verb: String,
}

/// What a trap does when it goes off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrapEffect {
    /// A missile from a hidden slit in the nearest wall: an attack roll; it lands as `object`.
    Shot { object: ObjectId, attack: Attack },
    /// An attack roll against whoever stands on the trap.
    Strike { attack: Attack },
    /// A blast of `radius` cells around the trap: `damage` to everyone in it (half beyond the
    /// inner cells), no roll to hit.
    Burst {
        spell: String,
        radius: i32,
        damage: Dice,
        /// What it does to walls (spec §8 "Digging"): it breaks those in the blast whose hardness
        /// is below this; 0 breaks none.
        blast: u16,
    },
    /// A line from the trap until a wall, at most `length` cells: `damage` to everyone on it.
    Bolt {
        spell: String,
        length: i32,
        damage: Dice,
    },
    /// A fall: `damage`, and the next action lost climbing out.
    Pit { damage: Dice },
    /// Only its noise.
    Alarm,
    /// Whoever steps on it is moved to a free cell of the level.
    Teleport,
}

impl TrapEffect {
    /// The kind's id as data writes it: `shot`, `burst`, ...
    pub const fn kind(&self) -> &'static str {
        match self {
            TrapEffect::Shot { .. } => "shot",
            TrapEffect::Strike { .. } => "strike",
            TrapEffect::Burst { .. } => "burst",
            TrapEffect::Bolt { .. } => "bolt",
            TrapEffect::Pit { .. } => "pit",
            TrapEffect::Alarm => "alarm",
            TrapEffect::Teleport => "teleport",
        }
    }

    /// The damage it does, if it does any.
    pub fn damage(&self) -> Option<Dice> {
        match self {
            TrapEffect::Shot { attack, .. } | TrapEffect::Strike { attack } => Some(attack.damage),
            TrapEffect::Burst { damage, .. }
            | TrapEffect::Bolt { damage, .. }
            | TrapEffect::Pit { damage } => Some(*damage),
            TrapEffect::Alarm | TrapEffect::Teleport => None,
        }
    }
}

/// A trap type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrapDef {
    pub id: String,
    /// "fireball trap": "Really step onto the fireball trap?"
    pub name: String,
    pub glyph: char,
    pub fg: Rgb,
    /// Said when the player perceives it go off: "A fireball explodes!"
    pub message: Option<String>,
    /// How often it goes off before it is spent and gone; `None`: it never runs out.
    pub charges: Option<u16>,
    /// The noise it makes going off, on its cell.
    pub noise: Option<TrapNoise>,
    /// What hits ("arrow", "fireball") and what it does ("hits", "burns"), for messages.
    pub noun: String,
    pub verb: String,
    pub effect: TrapEffect,
}

impl TrapSpec {
    /// Checks the spec and resolves it against `content`'s objects.
    pub(super) fn resolve(self, content: &Content) -> Result<TrapDef, String> {
        let object = |name: &str| -> Option<(ObjectId, &ObjectDef)> {
            let id = content.object_id(name)?;
            Some((id, content.object(id)))
        };
        let id = self.id.clone();
        let err = |m: String| format!("trap `{id}`: {m}");
        let kind = self.kind;
        let allowed = |field: &str, given: bool, kinds: &[Kind]| {
            if given && !kinds.contains(&kind) {
                Err(err(format!(
                    "`{field}` is not for a trap of kind `{}`",
                    kind.name()
                )))
            } else {
                Ok(())
            }
        };
        use Kind::*;
        allowed("object", self.object.is_some(), &[Shot])?;
        allowed("to_hit", self.to_hit != 0, &[Shot, Strike])?;
        allowed("radius", self.radius.is_some(), &[Burst])?;
        allowed("blast", self.blast.is_some(), &[Burst])?;
        allowed("length", self.length.is_some(), &[Bolt])?;
        allowed("spell", self.spell.is_some(), &[Burst, Bolt])?;
        allowed("noun", self.noun.is_some(), &[Strike, Burst, Bolt])?;
        allowed(
            "damage",
            self.damage.is_some(),
            &[Shot, Strike, Burst, Bolt, Pit],
        )?;
        if self.name.trim().is_empty() || self.verb.trim().is_empty() {
            return Err(err("`name` and `verb` must not be empty".into()));
        }
        if self.to_hit.abs() > MAX_BONUS {
            return Err(err(format!("`to_hit` must be within ±{MAX_BONUS}")));
        }
        if self
            .charges
            .is_some_and(|c| !(1..=MAX_CHARGES).contains(&c))
        {
            return Err(err(format!("`charges` must be 1 to {MAX_CHARGES}")));
        }
        if let Some(noise) = self.noise
            && !(1..=MAX_LOUDNESS).contains(&noise.loudness)
        {
            return Err(err(format!("`noise` loudness must be 1 to {MAX_LOUDNESS}")));
        }
        if kind == Alarm && self.noise.is_none() {
            return Err(err("an alarm needs a `noise`".into()));
        }
        let damage = || {
            self.damage
                .filter(|d| d.min() >= 0 && d.max() >= 1)
                .ok_or_else(|| err("needs `damage` dice that can do harm, e.g. \"1d6\"".into()))
        };
        let attack = |damage: Dice| Attack {
            damage,
            to_hit: self.to_hit,
            threat: DEFAULT_THREAT,
            crit: DEFAULT_CRIT,
            strength: None,
        };
        let spell = || {
            (self.spell.clone())
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| err("needs the `spell` effect it shows".into()))
        };
        let noun = || {
            (self.noun.clone())
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| err("needs a `noun`: what hits".into()))
        };
        let (effect, noun) = match kind {
            Shot => {
                let name = self.object.as_deref().unwrap_or_default();
                let (object, def) = object(name)
                    .ok_or_else(|| err(format!("`object` must name an object, not `{name}`")))?;
                (
                    TrapEffect::Shot {
                        object,
                        attack: attack(damage()?),
                    },
                    def.name.clone(),
                )
            }
            Strike => (
                TrapEffect::Strike {
                    attack: attack(damage()?),
                },
                noun()?,
            ),
            Burst => {
                let radius = self.radius.unwrap_or(0);
                if !BURST_RADII.contains(&radius) {
                    return Err(err(format!(
                        "`radius` must be {} to {}",
                        BURST_RADII.start(),
                        BURST_RADII.end()
                    )));
                }
                let blast = self.blast.unwrap_or(0);
                if blast > MAX_HARDNESS {
                    return Err(err(format!("`blast` must be 0 to {MAX_HARDNESS}")));
                }
                let effect = TrapEffect::Burst {
                    spell: spell()?,
                    radius,
                    damage: damage()?,
                    blast,
                };
                (effect, noun()?)
            }
            Bolt => {
                let length = self.length.unwrap_or(0);
                if !BOLT_LENGTHS.contains(&length) {
                    return Err(err(format!(
                        "`length` must be {} to {}",
                        BOLT_LENGTHS.start(),
                        BOLT_LENGTHS.end()
                    )));
                }
                let effect = TrapEffect::Bolt {
                    spell: spell()?,
                    length,
                    damage: damage()?,
                };
                (effect, noun()?)
            }
            Pit => (TrapEffect::Pit { damage: damage()? }, "fall".into()),
            Alarm => (TrapEffect::Alarm, "bell".into()),
            Teleport => (TrapEffect::Teleport, "magic".into()),
        };
        Ok(TrapDef {
            id: self.id,
            name: self.name,
            glyph: self.glyph,
            fg: self.fg,
            message: self.message,
            charges: self.charges,
            noise: self.noise,
            noun,
            verb: self.verb,
            effect,
        })
    }
}

/// Parses `data/traps.toml` against `content`'s objects.
pub(super) fn parse(
    path: &str,
    text: &str,
    content: &Content,
) -> Result<(Vec<TrapDef>, BTreeMap<String, TrapId>), ContentError> {
    let err = |m: String| ContentError::new(path, m);
    let parsed: TrapFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut defs = Vec::with_capacity(parsed.trap.len());
    let mut ids = BTreeMap::new();
    for (i, spec) in parsed.trap.into_iter().enumerate() {
        let id = TrapId(u16::try_from(i).map_err(|_| err("too many traps".into()))?);
        if ids.insert(spec.id.clone(), id).is_some() {
            return Err(err(format!("duplicate trap id `{}`", spec.id)));
        }
        defs.push(spec.resolve(content).map_err(err)?);
    }
    Ok((defs, ids))
}

/// A trap lying on a level's cell (spec §8 "Traps"): ground truth, kept in the level state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trap {
    /// Its type.
    pub kind: TrapId,
    /// Whether the player knows it is there: they set it off, saw a creature do it, or the level
    /// shows it (`known = true`). Known traps show as their glyph and stay in memory.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub known: bool,
    /// Charges left; `None`: it never runs out. At 0 it is spent and gone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charges: Option<u16>,
    /// The way a bolt runs from it; `None`: along the longest clear line from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<Dir>,
}

impl Trap {
    /// A trap of type `kind` (`def`) with all its charges.
    pub fn new(kind: TrapId, def: &TrapDef, known: bool, dir: Option<Dir>) -> Self {
        Self {
            kind,
            known,
            charges: def.charges,
            dir,
        }
    }
}
