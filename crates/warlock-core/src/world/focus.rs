//! The ranger's Focus (spec §8 "Resources"; the progression design §4.18, R2): an alternative
//! resource that Rapid shot (level 12) grants for good, that comes back a point a base turn but
//! not while resting (`data/resources.toml`), and that two rules spend:
//!
//! - **Rapid shot**: a shot the player looses (`fire`) while they hold its `cost` (2) spends it
//!   and takes `saving` percent (10) less time ([`World::rapid_shot`]); without the focus the
//!   shot is an ordinary one.
//! - **Volley** (level 50): its talent's `resource = { id = "focus", cost = 6 }` makes it
//!   unavailable while short (`ability.rs::short_of`) and is spent as the volley is loosed
//!   (`world/volley.rs`); its shots are priced as one ordinary shot, never by Rapid shot.
//!
//! What a talent grants is held from the moment the talent is ([`hold`]): as it is taken or
//! given (`class::hand_out`), and again when a run is restored (`World::grant_class`), so a
//! character saved before the talent granted anything gets it.

use super::{Actor, ActorId, World};
use crate::action::Event;
use crate::content::{Content, ResourceId, percent_of};

/// `actor` holds each of `resources` it does not hold yet, at its `start` (capped at its
/// maximum); one it holds keeps its value. Says nothing: the talent's news does.
pub(super) fn hold(content: &Content, actor: &mut Actor, resources: &[ResourceId]) {
    for &resource in resources {
        if actor.resources.get(resource).is_none() {
            let start = actor.resource_start(content, resource);
            actor.resources.set(resource, start);
        }
    }
}

impl World {
    /// What a shot priced `price` costs actor `id` with Rapid shot: if they have it and hold its
    /// cost in its resource, that is spent and the shot takes its saving less (at least 1);
    /// otherwise `price` as it is. The one place Rapid shot reads.
    pub(super) fn rapid_shot(&mut self, id: ActorId, price: i64, events: &mut Vec<Event>) -> i64 {
        let Some(rapid) =
            (self.state.actors.get(&id)).and_then(|a| a.talent_mods(&self.content).rapid_shot)
        else {
            return price;
        };
        if !self.spend_resource(id, rapid.resource, rapid.cost, events) {
            return price;
        }
        percent_of(price, 100 - u32::from(rapid.saving.min(100))).max(1)
    }
}
