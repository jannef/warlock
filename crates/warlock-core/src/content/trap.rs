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
//!   breaks every wall in it whose hardness is below that (spec §8 "Digging"); with `heat`, it
//!   melts all the ice in it, and with `frost` (mm) it freezes the water in it at once to at
//!   least that thick (spec §5 "Ice");
//! - `bolt`: a line from the trap in one direction, up to `length` cells or the first wall (a
//!   lightning bolt): everyone on it takes `damage`;
//! - `pit`: whoever steps on it falls in, takes `damage`, and loses their next action climbing
//!   out;
//! - `alarm`: nothing but its noise (a bell that draws creatures);
//! - `teleport`: whoever steps on it is moved to a free cell of the level.
//!
//! `burst` and `bolt` name the `spell` effect presentation shows (`data/effects.toml`: what a
//! fireball looks like); `strike`, `burst`, and `bolt` name what hits (`noun`, "fireball") and
//! what it does (`verb`, default: its damage type's verb), for messages ("The fireball burns
//! the kobold."). Every trap that does damage names its `damage_type` (spec §8 "Damage
//! types"): the arrow and the spears piercing, the fireball fire, the pit bludgeoning.
//!
//! A trap on a level ([`Trap`]) is hidden until the player sets it off or sees a creature do it,
//! and from then on known and remembered (spec §8 "Traps"). A level may put one on a door, of any
//! kind but a pit ([`TrapEffect::fits_fixture`]): opening or kicking the door sets it off (spec §8
//! "Trapped doors"); so may a shut chest, from the same list (spec §8 "Chests").

use super::damage_type::TypeNames;
use super::danger::MAX_DANGER;
use super::effect::EffectShape;
use super::object::{ObjectDef, ObjectId};
use super::spell::{self, Bolt, Burst, SpellFields, check_effect, default_verb, one_kind};
use super::spell_ladder::{Unit, parse_rungs};
use super::{Content, ContentError, DamageTypeId};
use crate::color::Rgb;
use crate::combat::{Attack, DEFAULT_CRIT, DEFAULT_THREAT};
use crate::dice::{Dice, Roll};
use crate::geom::Dir;
use crate::noise::{MAX_LOUDNESS, SoundKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Index of a trap definition in [`super::Content::trap_defs`]. Ids are assigned in file order,
/// so the content fingerprint covers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrapId(pub u16);

fn trap_glyph() -> char {
    '^'
}

/// A trap's difficulty where data gives none (spec §8 "Searching").
pub const DEFAULT_TRAP_DIFFICULTY: u8 = 15;

fn trap_difficulty() -> u8 {
    DEFAULT_TRAP_DIFFICULTY
}

fn trap_sign() -> String {
    "a pressure plate".into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrapFile {
    #[serde(default)]
    pub avoid: TrapAvoid,
    #[serde(default)]
    pub disarm: TrapDisarm,
    #[serde(default)]
    pub harm: TrapHarm,
    pub trap: Vec<TrapSpec>,
}

/// The chance of stepping onto a known trap without setting it off (spec §8 "Traps",
/// `[avoid]` in `data/traps.toml`): `base` percent plus `per_point` for each point of the
/// stepper's dodge (the Dexterity and Dodge parts of their Defence), kept within `min` and `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrapAvoid {
    /// The chance, in percent, at a dodge of 0.
    pub base: i32,
    /// Percent more for each point of dodge (less for each point below 0).
    pub per_point: i32,
    /// The least chance, in percent: never hopeless.
    pub min: u8,
    /// The most chance, in percent: never certain.
    pub max: u8,
}

impl Default for TrapAvoid {
    fn default() -> Self {
        Self {
            base: 55,
            per_point: 10,
            min: 15,
            max: 90,
        }
    }
}

impl TrapAvoid {
    /// The chance, in percent, of avoiding a known trap with `dodge`.
    pub fn chance(&self, dodge: i32) -> u32 {
        let percent = self.base + self.per_point * dodge;
        percent.clamp(i32::from(self.min), i32::from(self.max)) as u32
    }

    /// `min` no more than `max`: [`Self::chance`] clamps between them.
    fn check(&self) -> Result<(), String> {
        if self.min > self.max {
            return Err(format!(
                "[avoid]: min ({}) must be no more than max ({})",
                self.min, self.max
            ));
        }
        Ok(())
    }
}

/// Disarming a known trap (`[disarm]` in `data/traps.toml`; `world/skills/disarm.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrapDisarm {
    /// How far short of the trap's difficulty a failed try must fall to set it off.
    pub springs_short_by: i32,
}

/// The numbers for traps files without the table (test fixtures): the shipped file's.
impl Default for TrapDisarm {
    fn default() -> Self {
        Self {
            springs_short_by: 5,
        }
    }
}

/// What a trap's damage may be cut by (`[harm]` in `data/traps.toml`; `world/fixtures/trap.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrapHarm {
    /// The most Trap sense and things of Wariness cut together, in percent.
    pub most_cut: i32,
}

/// The numbers for traps files without the table (test fixtures): the shipped file's.
impl Default for TrapHarm {
    fn default() -> Self {
        Self { most_cut: 75 }
    }
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
    /// How hard it is to find (spec §8 "Searching").
    #[serde(default = "trap_difficulty")]
    difficulty: u8,
    /// What gives it away to whoever finds it: "a pressure plate".
    #[serde(default = "trap_sign")]
    sign: String,
    kind: Kind,
    /// `shot`: the object that flies (and lands).
    object: Option<String>,
    /// `shot`, `strike`, `burst`, `bolt`, `pit`.
    damage: Option<Dice>,
    /// `shot`, `strike`, `burst`, `bolt`, `pit`: the kind of damage it does.
    damage_type: Option<TypeNames>,
    /// Its damage by the level's danger (`[trap.grows]`, [`TrapGrows`]).
    grows: Option<GrowsSpec>,
    /// `shot`, `strike`.
    #[serde(default)]
    to_hit: i32,
    /// `burst`.
    radius: Option<i32>,
    /// `burst`: how strong the explosion is against walls (spec §8 "Digging"); none breaks none.
    blast: Option<u16>,
    /// `burst`: it is hot, and melts the ice it reaches (spec §5 "Ice").
    #[serde(default)]
    heat: bool,
    /// `burst`: it is cold, and freezes the water it reaches to at least this many mm of ice.
    frost: Option<u16>,
    /// `bolt`.
    length: Option<i32>,
    /// `burst`, `bolt`: the effect presentation shows (`data/effects.toml`).
    spell: Option<String>,
    /// `strike`, `burst`, `bolt`: what hits.
    noun: Option<String>,
    /// What it does to whom it hits; default: its damage type's verb.
    verb: Option<String>,
}

/// `[trap.grows]` as written: `damage = "danger 3 1d4, 6 1d6"`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GrowsSpec {
    damage: String,
}

/// A trap's damage by the danger of the level it lies on (spec §8 "Traps", `[trap.grows]` in
/// `data/traps.toml`, in the style of a spell's `[spell.grows]`, `content/spell_ladder.rs`):
/// rungs `danger AT ROLL`, `AT` 1 to [`MAX_DANGER`] strictly ascending; a level of danger *D*
/// reads the last rung at or below *D*, and below the first (a level without a danger: 0) the
/// trap's own `damage`, the **base**.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrapGrows {
    /// `(at, damage)`, `at` strictly ascending.
    pub damage: Vec<(u8, Roll)>,
}

impl TrapGrows {
    fn parse(text: &str, base: Roll) -> Result<Self, String> {
        let bad = |why: String| format!("`[trap.grows]` damage `{text}`: {why}");
        let rest = (text.trim().strip_prefix("danger "))
            .ok_or_else(|| bad("`danger`, then rungs: `danger 3 1d4, 6 1d6`".into()))?;
        let rungs = parse_rungs(rest, Unit::Roll, ("danger", MAX_DANGER)).map_err(bad)?;
        let mut before = base;
        let mut damage = Vec::with_capacity(rungs.len());
        for (at, value) in rungs {
            let roll = value.roll().unwrap_or(before);
            if roll.min() < 0 {
                return Err(bad(format!(
                    "the rung at {at}, {roll}, can roll below 0: harm would turn into healing"
                )));
            }
            before = roll;
            damage.push((at, roll));
        }
        Ok(Self { damage })
    }

    /// The rung a level of `danger` reads: how many rungs are at that danger or below (0, the
    /// base, below the first).
    pub fn rung_at(&self, danger: u8) -> usize {
        self.damage.iter().filter(|(at, _)| *at <= danger).count()
    }

    /// The damage at `danger`: its rung's, else (rung 0) `base`.
    pub fn damage_at(&self, danger: u8, base: Roll) -> Roll {
        (self.rung_at(danger).checked_sub(1))
            .and_then(|i| self.damage.get(i))
            .map_or(base, |(_, roll)| *roll)
    }

    /// In words, as the wiki prints it: `danger 3: 1d4, 6: 1d6`.
    pub fn words(&self) -> String {
        let rungs: Vec<String> = (self.damage.iter())
            .map(|(at, roll)| format!("{at}: {roll}"))
            .collect();
        format!("danger {}", rungs.join(", "))
    }
}

/// What a trap does when it goes off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrapEffect {
    /// A missile from a hidden slit in the nearest wall: an attack roll; it lands as `object`.
    Shot { object: ObjectId, attack: Attack },
    /// An attack roll against whoever stands on the trap.
    Strike { attack: Attack },
    /// A blast of `radius` cells around the trap ([`Burst`], as a scroll of fireball casts one):
    /// `damage` to everyone in it (half beyond the inner cells), no roll to hit.
    Burst(Burst),
    /// A line from the trap until a wall, at most `length` cells ([`Bolt`]): `damage` to
    /// everyone on it.
    Bolt(Bolt),
    /// A fall: `damage` of `kind` (bludgeoning), and the next action lost climbing out.
    Pit {
        damage: Roll,
        kind: Option<DamageTypeId>,
    },
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

    /// Whether a trap of this kind may lie on a fixture: a door (spec §8 "Trapped doors") or a
    /// shut chest (spec §8 "Chests"). Its trap goes off at whoever opens or kicks it, from the
    /// next cell: everything that strikes its victim where it stands or bursts out around the
    /// trap works there, but a pit needs its victim on its own cell, to fall in and climb out.
    pub const fn fits_fixture(&self) -> bool {
        !matches!(self, TrapEffect::Pit { .. })
    }

    /// Whether this trap, going off on a shut chest, blows the chest up (spec §8 "Chests"): a
    /// burst goes off inside the box; a shot, a strike, a bolt, an alarm, or a teleport leaves it
    /// as it was.
    pub const fn blows_fixture(&self) -> bool {
        matches!(self, TrapEffect::Burst(_))
    }

    /// The damage it does, if it does any.
    pub fn damage(&self) -> Option<Roll> {
        match self {
            TrapEffect::Shot { attack, .. } | TrapEffect::Strike { attack } => Some(attack.damage),
            TrapEffect::Burst(Burst { damage, .. }) | TrapEffect::Bolt(Bolt { damage, .. }) => {
                Some(*damage)
            }
            TrapEffect::Pit { damage, .. } => Some(*damage),
            TrapEffect::Alarm | TrapEffect::Teleport => None,
        }
    }

    /// It with `damage` in place of its own (a rung of [`TrapGrows`]); one that does none, as
    /// it is.
    pub fn with_damage(&self, damage: Roll) -> Self {
        let mut effect = self.clone();
        match &mut effect {
            TrapEffect::Shot { attack, .. } | TrapEffect::Strike { attack } => {
                attack.damage = damage;
            }
            TrapEffect::Burst(Burst { damage: d, .. })
            | TrapEffect::Bolt(Bolt { damage: d, .. })
            | TrapEffect::Pit { damage: d, .. } => *d = damage,
            TrapEffect::Alarm | TrapEffect::Teleport => {}
        }
        effect
    }

    /// The kind of damage it does, if it does any (spec §8 "Damage types").
    pub fn damage_kind(&self) -> Option<DamageTypeId> {
        match self {
            TrapEffect::Shot { attack, .. } | TrapEffect::Strike { attack } => attack.kinds.main(),
            TrapEffect::Burst(Burst { kind, .. })
            | TrapEffect::Bolt(Bolt { kind, .. })
            | TrapEffect::Pit { kind, .. } => *kind,
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
    /// How hard it is to find while hidden (spec §8 "Searching"): what a search must reach.
    pub difficulty: u8,
    /// What gives it away to whoever finds it: "You notice a pressure plate: an arrow trap."
    pub sign: String,
    /// What hits ("arrow", "fireball") and what it does ("hits", "burns"), for messages.
    pub noun: String,
    pub verb: String,
    /// What it does, its damage the base (rung 0).
    pub effect: TrapEffect,
    /// Its damage by the level's danger; `None`: the base at every danger.
    pub grows: Option<TrapGrows>,
}

impl TrapDef {
    /// Its damage on a level of `danger` (spec §8 "Traps": [`TrapGrows`]), if it does any.
    pub fn damage_at(&self, danger: u8) -> Option<Roll> {
        let base = self.effect.damage()?;
        Some((self.grows.as_ref()).map_or(base, |g| g.damage_at(danger, base)))
    }

    /// What it does on a level of `danger`: its effect with that danger's damage.
    pub fn effect_at(&self, danger: u8) -> TrapEffect {
        match self.damage_at(danger) {
            Some(damage) => self.effect.with_damage(damage),
            None => self.effect.clone(),
        }
    }
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
        allowed("heat", self.heat, &[Burst])?;
        allowed("frost", self.frost.is_some(), &[Burst])?;
        allowed("length", self.length.is_some(), &[Bolt])?;
        allowed("spell", self.spell.is_some(), &[Burst, Bolt])?;
        allowed("noun", self.noun.is_some(), &[Strike, Burst, Bolt])?;
        allowed(
            "damage",
            self.damage.is_some(),
            &[Shot, Strike, Burst, Bolt, Pit],
        )?;
        allowed(
            "damage_type",
            self.damage_type.is_some(),
            &[Shot, Strike, Burst, Bolt, Pit],
        )?;
        // What does damage does it of a kind.
        let damage_kind = if self.damage.is_some() {
            one_kind(content, self.damage_type.as_ref()).map_err(err)?
        } else {
            None
        };
        if let Some(noise) = self.noise
            && noise.loudness > MAX_LOUDNESS
        {
            return Err(err(format!(
                "`noise` loudness must be at most {MAX_LOUDNESS}"
            )));
        }
        let damage = || {
            self.damage.filter(|d| d.min() >= 0).ok_or_else(|| {
                err("needs `damage` dice that never roll below 0, e.g. \"1d6\"".into())
            })
        };
        let attack = |damage: Dice| Attack {
            damage: damage.into(),
            to_hit: self.to_hit,
            threat: DEFAULT_THREAT,
            crit: DEFAULT_CRIT,
            strength: None,
            finesse: false,
            kinds: damage_kind.map_or(crate::damage::Kinds::NONE, crate::damage::Kinds::one),
        };
        let fields = SpellFields {
            spell: self.spell.clone(),
            damage: self.damage,
            kind: damage_kind,
            radius: self.radius,
            blast: self.blast,
            heat: self.heat,
            frost: self.frost,
            length: self.length,
        };
        let noun = || (self.noun.clone()).ok_or_else(|| err("needs a `noun`: what hits".into()));
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
            Burst => (
                TrapEffect::Burst(spell::Burst::new(&fields).map_err(err)?),
                noun()?,
            ),
            Bolt => (
                TrapEffect::Bolt(spell::Bolt::new(&fields).map_err(err)?),
                noun()?,
            ),
            Pit => (
                TrapEffect::Pit {
                    damage: damage()?.into(),
                    kind: damage_kind,
                },
                "fall".into(),
            ),
            Alarm => (TrapEffect::Alarm, "bell".into()),
            Teleport => (TrapEffect::Teleport, "magic".into()),
        };
        // With spell effects in content, a burst or bolt names one of its own shape.
        match &effect {
            TrapEffect::Burst(b) => check_effect(content, &b.spell, EffectShape::Burst),
            TrapEffect::Bolt(b) => check_effect(content, &b.spell, EffectShape::Bolt),
            _ => Ok(()),
        }
        .map_err(err)?;
        let grows = match (&self.grows, effect.damage()) {
            (None, _) => None,
            (Some(g), Some(base)) => Some(TrapGrows::parse(&g.damage, base).map_err(err)?),
            (Some(_), None) => {
                return Err(err("`[trap.grows]` is for a trap that does damage".into()));
            }
        };
        Ok(TrapDef {
            id: self.id,
            name: self.name,
            glyph: self.glyph,
            fg: self.fg,
            message: self.message,
            charges: self.charges,
            noise: self.noise,
            difficulty: self.difficulty,
            sign: self.sign,
            noun,
            verb: self
                .verb
                .unwrap_or_else(|| default_verb(content, damage_kind)),
            effect,
            grows,
        })
    }
}

/// What `data/traps.toml` holds: the trap types by id, and the rules of avoiding a known one,
/// disarming one, and cutting its damage.
pub(super) type ParsedTraps = (
    Vec<TrapDef>,
    BTreeMap<String, TrapId>,
    (TrapAvoid, TrapDisarm, TrapHarm),
);

/// Parses `data/traps.toml` against `content`'s objects.
pub(super) fn parse(
    path: &str,
    text: &str,
    content: &Content,
) -> Result<ParsedTraps, ContentError> {
    let err = |m: String| ContentError::new(path, m);
    let parsed: TrapFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    parsed.avoid.check().map_err(err)?;
    if parsed.disarm.springs_short_by < 1 {
        return Err(err("[disarm]: springs_short_by must be at least 1".into()));
    }
    if !(0..=100).contains(&parsed.harm.most_cut) {
        return Err(err("[harm]: most_cut must be 0 to 100".into()));
    }
    let mut defs = Vec::with_capacity(parsed.trap.len());
    let mut ids = BTreeMap::new();
    for (i, spec) in parsed.trap.into_iter().enumerate() {
        let id = TrapId(u16::try_from(i).map_err(|_| err("too many traps".into()))?);
        if ids.insert(spec.id.clone(), id).is_some() {
            return Err(err(format!("duplicate trap id `{}`", spec.id)));
        }
        defs.push(spec.resolve(content).map_err(err)?);
    }
    Ok((defs, ids, (parsed.avoid, parsed.disarm, parsed.harm)))
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
    /// Rigged by the player (the craftsperson's Trapwright, `world/crafting/rig.rs`): what it kills is
    /// the player's kill.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rigged: bool,
}

impl Trap {
    /// A trap of type `kind` (`def`) with all its charges.
    pub fn new(kind: TrapId, def: &TrapDef, known: bool, dir: Option<Dir>) -> Self {
        Self {
            kind,
            known,
            charges: def.charges,
            dir,
            rigged: false,
        }
    }
}
