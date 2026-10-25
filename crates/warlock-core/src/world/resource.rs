//! Alternative resources in the world (spec §8 "Resources", #225): the pools beside hit points
//! and mana that something grants a character (`Actor::resources`; the pure rules are
//! [`crate::resource`], the data `content/resource.rs`). These are the only ways a resource
//! changes:
//!
//! - [`World::grant_resource`]: the actor holds it from now on, at its `start` (an ability, a
//!   talent; for now only the dev action `resource:ID`, [`World::grant_resource_dev`]).
//! - [`World::gain_resource`]: an event builds up every resource the actor holds that builds on
//!   it, called where the event happens: a kill in `fight.rs::dies`, a hit, a critical, and being
//!   hit in `resolve`, a sneak attack that hits in `ambush.rs`.
//! - [`World::spend_resource`]: an ability spends it (a class active's `resource` cost,
//!   `world/class_ability.rs`); refused when the actor holds too little.
//! - [`World::end_resource`]: it ends (one that `ends_at_zero`, at 0): the actor loses it, and
//!   its bar with it, until something grants it again.
//! - Time: regeneration and decay are moments of `World::live`'s loop (`world/status.rs`, with a
//!   hit point and a point of mana back), at [`World::resource_rate`]'s pace, so it makes no
//!   difference how time is cut up, and an end at zero comes at its exact moment.
//!
//! The maximum ([`Actor::resource_max`]) and the pace are the one hooks for the talents and class
//! features that move onto resources (the progression design §4.18; R1–R5). Mana is separate:
//! nothing converts between them.

use super::{Actor, ActorId, Spent, World};
use crate::action::Event;
use crate::content::{Build, Content, ResourceId};
use crate::resource::Pace;

impl Actor {
    /// The most of `resource` it may hold: its `max` in data. The one place that decides it
    /// (talents that raise it read here); the player is shown it.
    pub fn resource_max(&self, content: &Content, resource: ResourceId) -> u16 {
        content.resource(resource).max
    }
}

impl World {
    /// The most of `resource` actor `id` may hold ([`Actor::resource_max`]).
    fn resource_max(&self, id: ActorId, resource: ResourceId) -> u16 {
        (self.state.actors.get(&id)).map_or(0, |a| a.resource_max(&self.content, resource))
    }

    /// How fast `resource` changes over time for actor `id` now, in percent of its normal pace:
    /// 100, but none while the player rests for one that does not change while resting
    /// (`resting = false`). The one place that decides it.
    pub(crate) fn resource_rate(&self, id: ActorId, resource: ResourceId) -> u32 {
        let def = self.content.resource(resource);
        if self.resting && id == self.state.player && !def.resting {
            return 0;
        }
        100
    }

    /// The pace of each resource actor `id` holds, in order: what `World::live` passes time at.
    pub(super) fn resource_paces(&self, id: ActorId) -> Vec<Pace> {
        let Some(actor) = self.state.actors.get(&id) else {
            return Vec::new();
        };
        (actor.resources.iter())
            .map(|pool| Pace {
                resource: pool.resource,
                max: self.resource_max(id, pool.resource),
                rate: self.resource_rate(id, pool.resource),
            })
            .collect()
    }

    /// Actor `id` is granted `resource`: it holds it from now on, at its `start` (or what it
    /// already holds, if that is more). Says nothing: what grants it does.
    pub(crate) fn grant_resource(&mut self, id: ActorId, resource: ResourceId) {
        let def = self.content.resource(resource);
        let max = self.resource_max(id, resource);
        let start = def.start.min(max);
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let value = actor.resources.get(resource).map_or(0, |p| p.value);
        actor.resources.set(resource, value.max(start));
    }

    /// Event `on` builds up every resource actor `id` holds that builds on it (`builds` in data),
    /// each by its amount, up to its maximum. Resources it does not hold are not built.
    pub(crate) fn gain_resource(&mut self, id: ActorId, on: Build) {
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let gains: Vec<(ResourceId, u16, u16)> = (actor.resources.iter())
            .filter_map(|pool| {
                let amount = self.content.resource(pool.resource).built_by(on);
                (amount > 0).then(|| (pool.resource, amount, self.resource_max(id, pool.resource)))
            })
            .collect();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        for (resource, amount, max) in gains {
            if let Some(pool) = actor.resources.get_mut(resource) {
                pool.value = pool.value.saturating_add(amount).min(max);
            }
        }
    }

    /// Actor `id` spends `amount` of `resource` (an ability's cost; 0 asks only that it is
    /// held): the one way a resource is spent. Refused, spending nothing, when the actor does
    /// not hold it or holds less; returns whether it was spent. One that ends at zero and is
    /// spent to 0 ends ([`World::end_resource`]). Says nothing: the ability does.
    pub(crate) fn spend_resource(
        &mut self,
        id: ActorId,
        resource: ResourceId,
        amount: u16,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(pool) =
            (self.state.actors.get_mut(&id)).and_then(|a| a.resources.get_mut(resource))
        else {
            return false;
        };
        if pool.value < amount {
            return false;
        }
        pool.value -= amount;
        if pool.value == 0 && self.content.resource(resource).ends_at_zero {
            self.end_resource(id, resource, events);
        }
        true
    }

    /// `resource` ends for actor `id`: it holds it no more, and its bar goes ("Your rage
    /// subsides." for the player, [`Event::ResourceEnded`]). Nothing when it does not hold it.
    pub(crate) fn end_resource(
        &mut self,
        id: ActorId,
        resource: ResourceId,
        events: &mut Vec<Event>,
    ) {
        if let Some(actor) = self.state.actors.get_mut(&id) {
            ended(actor, resource, events);
        }
    }

    /// The dev action `resource:ID` ([`crate::action::Action::Resource`]): the player is granted
    /// resource `id` ([`World::grant_resource`]), for trying resources out and for captures.
    /// Refused for an unknown resource. Takes no time.
    pub(super) fn grant_resource_dev(&mut self, id: &str) -> Spent {
        let Some(resource) = self.content.resource_id(id) else {
            self.say(format!("There is no resource `{id}`."));
            return Spent::Nothing;
        };
        let player = self.state.player;
        self.grant_resource(player, resource);
        let name = self.content.resource(resource).name.clone();
        let value = (self.state.player().resources.get(resource)).map_or(0, |p| p.value);
        self.say(format!("You gain {name}: {value} (dev)."));
        Spent::NoTime
    }
}

/// `resource` ends for `actor` (see [`World::end_resource`]): also how `World::live` ends one
/// that decays to 0, at its moment.
pub(super) fn ended(actor: &mut Actor, resource: ResourceId, events: &mut Vec<Event>) {
    if actor.resources.remove(resource) {
        events.push(Event::ResourceEnded {
            actor: actor.id,
            resource,
        });
    }
}
