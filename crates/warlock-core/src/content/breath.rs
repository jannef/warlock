//! A dragon's breath (spec §8 "Creature behavior": "Dragons", #1099): `[creature.breath]`, the
//! enemy-only cone of its element that a `combat = "dragon"` type breathes whenever it is ready.
//! The rules are `world/breath.rs`.
//!
//! The cone is the spells' shared cone (`world/spell.rs::cone_cells`: a quarter turn wide, as
//! Burning hands'), `length` cells long. The cooldown is a status (the house rule for ability
//! limits, `ability.rs`): [`STATUS`], given `ceil(cooldown / eases)` degrees, as a summoning's
//! (`summon.rs`).

use super::damage_type::DamageTypeId;
use super::disease::DiseaseId;
use super::effect::EffectShape;
use super::status::{Affliction, OnHit, OnHitSpec, StatusId};
use super::{Content, ContentError};
use crate::dice::Dice;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The cooldown status a breath gives (`data/status.toml`).
pub const STATUS: &str = "breath";
/// How long a breath's cone may be, in cells.
pub const LENGTHS: RangeInclusive<i32> = 2..=12;
/// A breath's cooldown, in seconds.
pub const COOLDOWNS: RangeInclusive<u32> = 6..=600;

/// A breath as written: `[creature.breath]` (`data/creatures/README.toml` has the fields).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BreathSpec {
    pub damage: Dice,
    pub damage_type: String,
    pub length: i32,
    pub cooldown: u32,
    pub effect: String,
    pub noun: Option<String>,
    pub verb: Option<String>,
    #[serde(default)]
    pub riders: Vec<OnHitSpec>,
}

/// A creature type's breath, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breath {
    /// What it rolls, once for everyone in the cone.
    pub damage: Dice,
    /// Its element: the kind of damage it does.
    pub kind: DamageTypeId,
    /// How far the cone reaches, in cells; it breathes at a hostile within this.
    pub length: i32,
    /// Seconds before it breathes again.
    pub cooldown: u32,
    /// The cooldown status, and the degrees of it a breath gives.
    pub status: StatusId,
    pub degrees: u16,
    /// The burst effect drawn over the cone (`data/effects.toml`; presentation only).
    pub effect: String,
    /// What strikes, in the messages: "fire" ("The fire burns you.").
    pub noun: String,
    /// What the dragon does, in the message: "breathes fire" ("The fire drake breathes fire!").
    pub verb: String,
    /// What it causes on everyone it harmed, each by its own chance, in data order (the
    /// gorgon's stiffening).
    pub riders: Vec<OnHit>,
}

impl BreathSpec {
    /// Resolves the breath against `content` (its damage type and the cooldown status, already
    /// loaded): checked, with the status's degrees; `status` and `disease` resolve its riders
    /// as a gaze's (`gaze.rs`).
    pub(super) fn resolve(
        &self,
        content: &Content,
        status_of: impl Fn(&str) -> Option<(StatusId, u16, bool)>,
        disease_of: impl Fn(&str) -> Option<DiseaseId>,
    ) -> Result<Breath, String> {
        if self.damage.min() < 1 {
            return Err("breath: `damage` must roll at least 1".into());
        }
        if !LENGTHS.contains(&self.length) {
            return Err(format!(
                "breath: `length` must be {} to {}",
                LENGTHS.start(),
                LENGTHS.end()
            ));
        }
        if !COOLDOWNS.contains(&self.cooldown) {
            return Err(format!(
                "breath: `cooldown` must be {} to {} seconds",
                COOLDOWNS.start(),
                COOLDOWNS.end()
            ));
        }
        let words = [&self.noun, &self.verb];
        if words
            .iter()
            .any(|w| w.as_deref().is_some_and(|w| w.trim().is_empty()))
        {
            return Err("breath: `noun` and `verb` must not be empty".into());
        }
        let kind = (content.damage_type_id(&self.damage_type))
            .ok_or_else(|| format!("breath: unknown damage type `{}`", self.damage_type))?;
        let status = (content.status_id(STATUS))
            .ok_or_else(|| format!("breath: needs the `{STATUS}` status in data/status.toml"))?;
        let status_def = content.status(status);
        let eases = (status_def.eases)
            .filter(|_| status_def.tick.is_none())
            .ok_or_else(|| format!("breath: the `{STATUS}` status must ease and never tick"))?;
        let degrees = (u16::try_from(self.cooldown.div_ceil(eases)).ok())
            .filter(|&d| d <= status_def.max)
            .ok_or_else(|| {
                format!(
                    "breath: a cooldown of {} s is more than the `{STATUS}` status lasts",
                    self.cooldown
                )
            })?;
        let riders = (self.riders.iter())
            .map(|r| {
                let rider = r
                    .resolve(&status_of, &disease_of)
                    .map_err(|m| format!("breath: {m}"))?;
                match rider.what {
                    Affliction::Status { .. } | Affliction::Drain(_) => Ok(rider),
                    Affliction::Hold { .. } | Affliction::Disease(_) => Err(
                        "breath: a rider is a status or a drain, never a hold or a disease".into(),
                    ),
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Breath {
            damage: self.damage,
            kind,
            length: self.length,
            cooldown: self.cooldown,
            status,
            degrees,
            effect: self.effect.clone(),
            noun: self.noun.clone().unwrap_or_else(|| "breath".into()),
            verb: self.verb.clone().unwrap_or_else(|| "breathes".into()),
            riders,
        })
    }
}

/// With spell effects in content, every breath names a burst effect (a cone is drawn as a burst
/// around the dragon over its cells); checked once effects are loaded, after the creatures.
pub(super) fn check_effects(content: &Content) -> Result<(), ContentError> {
    for def in content.creature_defs() {
        let Some(breath) = &def.breath else {
            continue;
        };
        super::spell::check_effect(content, &breath.effect, EffectShape::Burst).map_err(|m| {
            ContentError::new(
                "data/creatures/",
                format!("creature `{}`: breath: {m}", def.id),
            )
        })?;
    }
    Ok(())
}
