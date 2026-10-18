//! Actors: the player and creatures.

use crate::content::{Content, CreatureId};
use crate::geom::{Dir, Pos};
use crate::noise::Heard;
use crate::object::{Stack, Weight, weight_of};
use crate::stats::Stats;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable actor identity. Ids are never reused within a run; actors ready at the same moment act
/// in ascending id order, after the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActorId(pub u32);

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Player,
    Creature,
}

/// A fixed, looping patrol route: what a creature does while no hostile is about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patrol {
    pub route: Vec<Dir>,
    /// Index of the next step to take.
    pub step: usize,
}

/// What a creature remembers between its actions (spec §8 "Creature behavior"); all `None` while
/// it goes about its routine. The player has none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mind {
    /// Where it last saw a hostile it has since lost sight of: it goes there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Pos>,
    /// Where it heard a noise whose source it could not see: it goes to look.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise: Option<Pos>,
    /// Where it left its routine (its patrol route, or the cell it waited on) to fight or look:
    /// it walks back there once nothing is about, then carries on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<Pos>,
}

impl Mind {
    /// Whether it is going about its routine.
    pub fn is_calm(&self) -> bool {
        *self == Mind::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub kind: ActorKind,
    pub creature: CreatureId,
    pub level: String,
    pub pos: Pos,
    pub patrol: Option<Patrol>,
    /// Scheduler energy (spec §8 "Speed", `world/schedule.rs`): the actor may act while this is
    /// at least 0. It gains the creature type's speed per second of game time while the actor is
    /// on the current level; an action costs 1000 per second it takes.
    pub energy: i64,
    /// Noises the actor heard since it last acted, in the order heard (spec §5 "Hearing"; see
    /// [`crate::noise`]): where each was made, its kind, and how loud it was here. A creature's
    /// list is what it may react to in its next action ([`crate::World`]'s `creature_turn`, where
    /// the AI reads it) and is cleared as it acts; the player's covers the turn that last passed
    /// and becomes the perceived sounds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heard: Vec<Heard>,
    /// What a creature has in mind (`world/ai.rs`).
    #[serde(default, skip_serializing_if = "Mind::is_calm")]
    pub mind: Mind,
    /// What it carries, lettered `a` on (see [`crate::object`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inventory: Vec<Stack>,
    /// Its scores and potentials (spec §8 "Stats"; [`crate::stats`]): from its creature type,
    /// the player's from the scenario. Known to the rules (and dev tools) for every actor; the
    /// player is shown only their own scores.
    pub stats: Stats,
}

impl Actor {
    /// Its creature's base weight plus everything it carries, in grams. Known to the rules (and
    /// dev tools) for every actor; the player is shown only their own.
    pub fn weight(&self, content: &Content) -> Weight {
        Weight {
            base: u64::from(content.creature(self.creature).weight),
            carried: weight_of(&self.inventory, content),
        }
    }
}
