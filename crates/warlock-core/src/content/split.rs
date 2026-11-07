//! A creature type's splitting and what it dies into (docs/design/monster-dread.md §5.5, G5a
//! and G5b; #1129): `splits` and `dies_into`. The rules are `world/split.rs`.
//!
//! - `splits = { on = ["slashing", "piercing"], least = 8 }`: a blow of a listed damage type
//!   that leaves it alive with at least `2 × least` hit points cuts it in two.
//! - `dies_into = { creatures = ["grave_worm"], count = "2d4", spread = 1 }`: what crawls out
//!   of its corpse, each of the pool as likely, within `spread` of where it died.
//!
//! Both resolve once every creature type is known ([`Content::load_splits`], after the
//! summons): the damage types by id, the pool's creatures by id (a shared type names shared
//! types only, a campaign's its own too; never the player, a unique, or itself).

use super::{Content, ContentError, CreatureId, DamageTypeId};
use crate::dice::Dice;
use serde::Deserialize;
/// What a half cut off a creature pays when killed, in percent of its type's experience.
pub const HALF_SHARE: u8 = 25;
/// What a creature that crawled out of a corpse pays when killed, in percent.
pub const CORPSE_SHARE: u8 = 10;

/// `splits` as written.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SplitsSpec {
    pub on: Vec<String>,
    pub least: i32,
}

/// A creature type's splitting, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Splits {
    /// The kinds of damage that cut it in two, in data order.
    pub on: Vec<DamageTypeId>,
    /// The fewest hit points a half has: it splits only with at least twice this left.
    pub least: i32,
}

/// `dies_into` as written.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiesIntoSpec {
    pub creatures: Vec<String>,
    pub count: Dice,
    pub spread: Option<i32>,
}

/// What a creature type dies into, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiesInto {
    /// What may crawl out, each as likely (one named twice is twice as likely).
    pub pool: Vec<CreatureId>,
    /// How many (1 to [`MAX_COUNT`]).
    pub count: Dice,
    /// How far from the corpse they appear (Chebyshev; default 1).
    pub spread: i32,
}

impl Content {
    /// Resolves every creature type's `splits` and `dies_into` (module docs).
    pub(super) fn load_splits(&mut self) -> Result<(), ContentError> {
        for i in 0..self.creatures.len() {
            let me = CreatureId(i as u16);
            let (splits, dies_into) = self.creatures[i].split_spec.clone();
            let err = |m: String| {
                ContentError::new(
                    "data/creatures/",
                    format!("creature `{}`: {m}", self.creatures[i].id),
                )
            };
            let splits = (splits.as_ref())
                .map(|s| self.resolve_splits(s))
                .transpose()
                .map_err(err)?;
            let dies_into = (dies_into.as_ref())
                .map(|d| self.resolve_dies_into(me, d))
                .transpose()
                .map_err(err)?;
            self.creatures[i].splits = splits;
            self.creatures[i].dies_into = dies_into;
        }
        Ok(())
    }

    fn resolve_splits(&self, spec: &SplitsSpec) -> Result<Splits, String> {
        // Each half keeps at least `least` hit points: never none.
        if spec.least < 1 {
            return Err("splits: `least` must be 1 or more".into());
        }
        let on = (spec.on.iter())
            .map(|name| {
                (self.damage_type_id(name))
                    .ok_or_else(|| format!("splits: unknown damage type `{name}`"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Splits {
            on,
            least: spec.least,
        })
    }

    fn resolve_dies_into(&self, me: CreatureId, spec: &DiesIntoSpec) -> Result<DiesInto, String> {
        if spec.creatures.is_empty() {
            return Err("dies_into: `creatures` names none".into());
        }
        let spread = spec.spread.unwrap_or(1);
        let campaign = self.creature(me).campaign.as_deref();
        let pool = (spec.creatures.iter())
            .map(|name| {
                let c = (self.creature_id(name))
                    .ok_or_else(|| format!("dies_into: unknown creature `{name}`"))?;
                let other = self.creature(c);
                if other.campaign.is_some() && other.campaign.as_deref() != campaign {
                    return Err(format!(
                        "dies_into: creature `{name}` is campaign `{}`'s",
                        other.campaign.as_deref().unwrap_or_default()
                    ));
                }
                if other.unique || other.id == "player" {
                    return Err(format!("dies_into: `{name}` cannot crawl out of a corpse"));
                }
                Ok(c)
            })
            .collect::<Result<_, _>>()?;
        Ok(DiesInto {
            pool,
            count: spec.count,
            spread,
        })
    }
}
