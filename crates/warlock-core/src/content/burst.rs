//! A creature type's burst (the dread design `docs/design/monster-dread.md` §5.3, #1127):
//! `[creature.burst]`, the burst of its element it goes off in. A vortex bursts **instead of
//! attacking** beside its target and dies of it (`when = "touches"`); an elemental bursts where
//! it dies (`when = "dies"`, its death throes). The rules are `world/creature_burst.rs`.
//!
//! The burst is the spells' shared burst (`world/spell.rs`), centred on the creature's own cell,
//! `radius` cells across (0 to [`RADII`]'s end). Its riders are the spell types' rider form:
//! a status, its severity, and a chance.

use super::damage_type::DamageTypeId;
use super::effect::EffectShape;
use super::spell_type::{RiderSpec, TypeRider};
use super::{Content, ContentError};
use crate::dice::Dice;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// How far a creature's burst may reach, in cells.
pub const RADII: RangeInclusive<i32> = 0..=3;

/// When a creature bursts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BurstWhen {
    /// Where it dies, however it dies: death throes.
    Dies,
    /// Beside its target, instead of attacking, and it dies of it: a vortex. Killed, it goes
    /// quietly.
    Touches,
}

/// A burst as written: `[creature.burst]` (`data/creatures/README.toml` has the fields).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BurstSpec {
    pub damage: Dice,
    pub damage_type: String,
    pub radius: i32,
    pub when: BurstWhen,
    pub effect: String,
    pub noun: Option<String>,
    pub verb: Option<String>,
    #[serde(default)]
    pub rider: Vec<RiderSpec>,
    #[serde(default)]
    pub douses: bool,
    #[serde(default)]
    pub gust: bool,
}

/// A creature type's burst, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureBurst {
    /// What it rolls, once for everyone it reaches.
    pub damage: Dice,
    /// Its element: the kind of damage it does.
    pub kind: DamageTypeId,
    /// How far it reaches around the creature's cell.
    pub radius: i32,
    pub when: BurstWhen,
    /// The burst effect drawn (`data/effects.toml`; presentation only).
    pub effect: String,
    /// What strikes, in the messages: "flame" ("The flame burns you.").
    pub noun: String,
    /// What the creature does, in the message: "comes apart in flame" ("The fire elemental
    /// comes apart in flame!").
    pub verb: String,
    /// The statuses it lays on those it harms, each by its chance.
    pub riders: Vec<TypeRider>,
    /// A wave: it ends what water ends (Burning) on everyone it reaches, and puts out the fires
    /// on its cells.
    pub douses: bool,
    /// A gust: what lies loose on its cells is blown a cell outward.
    pub gust: bool,
}

impl BurstSpec {
    /// Resolves the burst against `content` (its damage type and the riders' statuses, already
    /// loaded), checked.
    pub(super) fn resolve(&self, content: &Content) -> Result<CreatureBurst, String> {
        if self.damage.min() < 1 {
            return Err("burst: `damage` must roll at least 1".into());
        }
        if !RADII.contains(&self.radius) {
            return Err(format!(
                "burst: `radius` must be {} to {}",
                RADII.start(),
                RADII.end()
            ));
        }
        let words = [&self.noun, &self.verb];
        if words
            .iter()
            .any(|w| w.as_deref().is_some_and(|w| w.trim().is_empty()))
        {
            return Err("burst: `noun` and `verb` must not be empty".into());
        }
        let kind = (content.damage_type_id(&self.damage_type))
            .ok_or_else(|| format!("burst: unknown damage type `{}`", self.damage_type))?;
        let riders = (self.rider.iter())
            .map(|r| super::spell_type::rider(r, content).map_err(|m| format!("burst: {m}")))
            .collect::<Result<_, _>>()?;
        let verb = match self.when {
            BurstWhen::Dies => "comes apart",
            BurstWhen::Touches => "bursts",
        };
        Ok(CreatureBurst {
            damage: self.damage,
            kind,
            radius: self.radius,
            when: self.when,
            effect: self.effect.clone(),
            noun: self.noun.clone().unwrap_or_else(|| "burst".into()),
            verb: self.verb.clone().unwrap_or_else(|| verb.into()),
            riders,
            douses: self.douses,
            gust: self.gust,
        })
    }
}

/// With spell effects in content, every burst names a burst effect; checked once effects are
/// loaded, after the creatures.
pub(super) fn check_effects(content: &Content) -> Result<(), ContentError> {
    for def in content.creature_defs() {
        let Some(burst) = &def.burst else {
            continue;
        };
        super::spell::check_effect(content, &burst.effect, EffectShape::Burst).map_err(|m| {
            ContentError::new(
                "data/creatures/",
                format!("creature `{}`: burst: {m}", def.id),
            )
        })?;
    }
    Ok(())
}
