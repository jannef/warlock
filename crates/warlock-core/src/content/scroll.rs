//! What reading does (`read` in an object's file, spec §8 "Scrolls"): a scroll casts a spell
//! of `data/spells/` through the one cast path at its own caster level (`spell`), casts a burst
//! or a bolt the reader aims with the traps' rules ([`super::spell`]), or identifies an item the
//! reader carries.
//!
//! ```toml
//! [object.read]
//! kind = "spell"        # the spell of data/spells/ named by `spell`, which it also teaches
//! spell = "magic_missile"
//! ```
//!
//! ```toml
//! [object.read]
//! kind = "burst"        # burst, bolt, spell, or identify
//! spell = "fireball"    # the effect presentation shows (data/effects.toml)
//! noun = "fireball"     # what hits, for messages; verb what it does (default "hits")
//! verb = "burns"
//! message = "The fireball explodes!"
//! radius = 2            # a burst's, with damage, blast, heat or frost as a trap's
//! damage = "3d6"
//! range = 8             # how far a burst flies (a bolt reaches its `length`)
//! noise = { sound = "explosion", loudness = 150 }
//! teaches = "fire_burst" # the spell of data/spells/ studying it teaches (spec §8 "Spells")
//! ```

use super::damage_type::TypeNames;
use super::effect::EffectShape;
use super::object::MAX_RANGE;
use super::spell::{Bolt, Burst, SpellFields, check_effect, default_verb, one_kind};
use super::trap::TrapNoise;
use super::{Content, SpellId};
use crate::dice::Dice;
use crate::noise::MAX_LOUDNESS;
use serde::Deserialize;

/// What kind of reading data names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Burst,
    Bolt,
    Spell,
    Identify,
}

/// `read` as an object's file writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadSpec {
    kind: Kind,
    /// `burst`, `bolt`: the effect presentation shows (`data/effects.toml`); `spell`: the spell
    /// of `data/spells/` it casts and teaches.
    spell: Option<String>,
    /// `burst`, `bolt`: what hits ("fireball") and what it does ("burns", default: its damage
    /// type's verb), for messages.
    noun: Option<String>,
    verb: Option<String>,
    /// Said as it is read, after "You read the scroll of fireball.".
    message: Option<String>,
    /// `burst`, `bolt`.
    damage: Option<Dice>,
    /// `burst`, `bolt`: the kind of damage it does (spec §8 "Damage types").
    damage_type: Option<TypeNames>,
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
    /// `burst`, `bolt`: the spell studying the scroll teaches (`study:b`, spec §8 "Spells"), an
    /// id of `data/spells/`; none: it teaches nothing.
    teaches: Option<String>,
}

/// What reading a scroll does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// It casts a burst or a bolt at a cell the reader aims at.
    Spell(ScrollSpell),
    /// It casts a spell of `data/spells/` through the one cast path, and teaches it.
    Cast(SpellScroll),
    /// It identifies one magic item the reader carries (spec §8 "Magic items").
    Identify,
}

/// A scroll of a spell of `data/spells/` (spec §8 "Scrolls", "Spells"): read, it casts the
/// spell at `level`, free of mana; studied, it teaches it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellScroll {
    pub spell: SpellId,
    /// The caster level a read casts it at: the spell's `[spell.scroll] level`, else its tier's
    /// comfortable level.
    pub level: u8,
    /// How far the reader may aim it before Willpower (its data's reach); `None` for a spell on
    /// oneself, read without aiming.
    pub reach: Option<i32>,
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
    /// The spell studying it teaches (spec §8 "Spells"), if any.
    pub teaches: Option<SpellId>,
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
    /// How far the reader may aim it; `None` for what is not aimed (identify, a spell on
    /// oneself).
    pub fn range(&self) -> Option<i32> {
        match self {
            Reading::Spell(spell) => Some(spell.range),
            Reading::Cast(scroll) => scroll.reach,
            Reading::Identify => None,
        }
    }

    /// The burst or bolt it casts, if it casts one.
    pub fn spell(&self) -> Option<&ScrollSpell> {
        match self {
            Reading::Spell(spell) => Some(spell),
            Reading::Cast(_) | Reading::Identify => None,
        }
    }

    /// The spell of `data/spells/` studying it teaches (spec §8 "Spells"), if any.
    pub fn teaches(&self) -> Option<SpellId> {
        match self {
            Reading::Spell(spell) => spell.teaches,
            Reading::Cast(scroll) => Some(scroll.spell),
            Reading::Identify => None,
        }
    }
}

impl ReadSpec {
    /// Checks the spec (its spell effect is checked once effects are loaded, [`check_spells`]).
    pub(super) fn resolve(self, content: &Content) -> Result<Reading, String> {
        let kind = self.kind;
        if kind == Kind::Spell {
            return self.spell_scroll(content);
        }
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
        allowed("verb", self.verb.is_some(), spelled)?;
        allowed("damage", self.damage.is_some(), spelled)?;
        allowed("damage_type", self.damage_type.is_some(), spelled)?;
        allowed("noise", self.noise.is_some(), spelled)?;
        allowed("radius", self.radius.is_some(), burst)?;
        allowed("range", self.range.is_some(), burst)?;
        allowed("blast", self.blast.is_some(), burst)?;
        allowed("heat", self.heat, burst)?;
        allowed("frost", self.frost.is_some(), burst)?;
        allowed("length", self.length.is_some(), kind == Kind::Bolt)?;
        allowed("teaches", self.teaches.is_some(), spelled)?;
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
        if self.verb.as_ref().is_some_and(|v| v.trim().is_empty()) {
            return Err("`read`: `verb` must not be empty".into());
        }
        let damage_kind =
            one_kind(content, self.damage_type.as_ref()).map_err(|m| format!("`read`: {m}"))?;
        // A spell it teaches is one the abilities menu can cast, with its file (spells load
        // before objects).
        let teaches = match &self.teaches {
            None => None,
            Some(id) => {
                if content.spells().is_empty() {
                    None
                } else {
                    let spell = (content.spell_id(id)).ok_or_else(|| {
                        format!("`read`: `teaches` names an unknown spell `{id}`")
                    })?;
                    Some(spell)
                }
            }
        };
        let fields = SpellFields {
            spell: self.spell,
            damage: self.damage,
            kind: damage_kind,
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
            verb: self
                .verb
                .unwrap_or_else(|| default_verb(content, damage_kind)),
            message: self.message,
            noise: self.noise,
            teaches,
        }))
    }
}

impl ReadSpec {
    /// A `spell` reading: only `spell`, a spell of `data/spells/` (spells load before objects).
    fn spell_scroll(self, content: &Content) -> Result<Reading, String> {
        let other = self.noun.is_some()
            || self.verb.is_some()
            || self.message.is_some()
            || self.damage.is_some()
            || self.damage_type.is_some()
            || self.noise.is_some()
            || self.teaches.is_some();
        if other {
            return Err(
                "`read`: a `spell` reading names only its `spell`: the spell's own data says the \
                 rest, and it teaches that spell"
                    .into(),
            );
        }
        let id = self
            .spell
            .ok_or("`read`: a `spell` reading needs its `spell`")?;
        let spell = (content.spell_id(&id))
            .ok_or_else(|| format!("`read`: no spell `{id}` in data/spells/"))?;
        let def = content.spell(spell);
        let comfortable = content
            .spell_rules()
            .map_or(1, |r| r.comfortable[usize::from(def.tier)]);
        Ok(Reading::Cast(SpellScroll {
            spell,
            level: def.scroll_level.unwrap_or(comfortable),
            reach: def.base_reach(),
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
    // The meals the talents name are objects (loaded after the talents).
    for (_, def) in content.talents().iter() {
        let meals = (def.effects.iter()).chain(def.ranks.iter().flat_map(|r| r.effects.iter()));
        for effect in meals {
            if let super::TalentEffect::MealHeals {
                object: Some(object),
                ..
            } = effect
                && !content.object_defs().is_empty()
                && content.object_id(object).is_none()
            {
                return Err(format!(
                    "talent `{}`: `meal_heals` names an unknown object `{object}`",
                    def.id
                ));
            }
        }
    }
    Ok(())
}
