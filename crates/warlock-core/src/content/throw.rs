//! A creature type's throw (spec §8 "Ranged combat": "Creatures' throws"): a natural ranged
//! attack, `[creature.throw]`, that needs no ammunition. A giant hurls boulders: what flies is an
//! object (for its look only; it never lands), the attack is a natural attack's.

use super::creature::{NaturalAttack, NaturalAttackSpec};
use super::damage_type::TypeNames;
use super::disease::DiseaseId;
use super::object::ObjectId;
use super::status::{OnHitSpec, StatusId};
use crate::damage::Kinds;
use crate::dice::Dice;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// How far a throw may reach, in cells.
pub const THROW_RANGES: RangeInclusive<i32> = 2..=20;

/// A throw as written: `[creature.throw]` with `object` (what flies), `damage`, `damage_type`,
/// `range`, `verb` (what the thrower does: "hurls"), and optionally `name` (what it is called;
/// default the object's name), `cost` (its energy; default `throw` in `data/actions.toml`), and
/// `to_hit`, `threat`, `crit`, and `on_hit` as for a natural attack.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ThrowSpec {
    pub object: String,
    pub name: Option<String>,
    pub damage: Dice,
    pub damage_type: Option<TypeNames>,
    pub verb: String,
    pub range: i32,
    pub cost: Option<u32>,
    pub threat: Option<u8>,
    pub crit: Option<u8>,
    #[serde(default)]
    pub to_hit: i32,
    #[serde(default)]
    pub on_hit: Vec<OnHitSpec>,
}

/// A creature type's throw: the attack (its `name` what flies, its `verb` what the thrower does
/// as it throws), what flies, how far, and what a throw costs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Throw {
    /// What a hit does, and the words: `name` "boulder" ("The boulder strikes you."), `verb`
    /// "hurls" ("The hill giant hurls a boulder.").
    pub attack: NaturalAttack,
    /// What flies, for its look ([`crate::action::Event::Missile`]); resolved once objects are
    /// known.
    pub object: ObjectId,
    /// How far it flies, in cells.
    pub range: i32,
    /// What a throw costs in energy, if its type says; `None`: the `throw` action's cost.
    pub cost: Option<u32>,
    /// The object as written, and whether `name` was given (else the object's name is taken).
    pub(super) object_id: String,
    pub(super) named: bool,
}

impl ThrowSpec {
    /// Checks the throw and resolves all but its object (`status`, `disease`, and `kinds` as for
    /// a natural attack, [`NaturalAttackSpec::resolve`]).
    pub(super) fn resolve(
        self,
        status: impl Fn(&str) -> Option<(StatusId, u16, bool)>,
        disease: impl Fn(&str) -> Option<DiseaseId>,
        kinds: Kinds,
    ) -> Result<Throw, String> {
        if !THROW_RANGES.contains(&self.range) {
            return Err(format!(
                "throw: `range` must be {} to {}",
                THROW_RANGES.start(),
                THROW_RANGES.end()
            ));
        }
        if let Some(cost) = self.cost {
            super::action::check_cost(cost).map_err(|m| format!("throw: cost: {m}"))?;
        }
        let named = self.name.is_some();
        let attack = NaturalAttackSpec {
            damage: self.damage,
            damage_type: self.damage_type,
            name: self.name.unwrap_or_else(|| self.object.clone()),
            verb: self.verb,
            threat: self.threat,
            crit: self.crit,
            to_hit: self.to_hit,
            on_hit: self.on_hit,
        }
        .resolve(status, disease, kinds)
        .map_err(|m| m.replacen("attack:", "throw:", 1))?;
        Ok(Throw {
            attack,
            object: ObjectId(0),
            range: self.range,
            cost: self.cost,
            object_id: self.object,
            named,
        })
    }
}
