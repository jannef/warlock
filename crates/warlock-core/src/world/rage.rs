//! The barbarian's rage in the world (spec §8 "Class abilities" and "Resources"; the
//! progression design §4.18, R1): the Rage ability grants the Rage resource, which brings the
//! Raging status (its benefits and what it blocks) for as long as it lasts. Rage decays a point a
//! base turn and ends at zero, taking Raging with it, and Exhausted follows (`World::live`,
//! `World::end_resource`). The talents on it read their one place each: Blood frenzy's rage a
//! kill is a resource build (`World::gain_resource`), Mighty rage's maximum is
//! `Actor::resource_max`, and Unstoppable is here and in `World::live`'s numb statuses.

use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::content::ActionKind;

impl World {
    /// Rage: the player is granted the rage's resource (Rage) and suffers its status (Raging),
    /// for the time of a wait.
    pub(super) fn rage(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(resource) = self.state.player().talent_mods(&self.content).rage else {
            return Spent::Nothing;
        };
        self.grant_resource_with_status(player, resource, Some(player), events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Whether actor `id`, hit for `damage`, holds on with Unstoppable: a blow that would take
    /// its last hit point while it rages leaves it that one, for Unstoppable's cost of its rage
    /// (spent to nothing, the rage ends and what follows it begins). Without that much rage it
    /// falls. The caller then does no more harm.
    pub(super) fn unstoppable(
        &mut self,
        id: ActorId,
        damage: i32,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        if actor.hp - damage > 0 || actor.hp <= 0 {
            return false;
        }
        let mods = actor.talent_mods(&self.content);
        let (Some(rage), Some(unstoppable)) = (mods.rage, &mods.unstoppable) else {
            return false;
        };
        let raging = (mods.rage_status(&self.content)).is_some_and(|s| actor.effects.has(s));
        if !raging || !self.spend_resource(id, rage, unstoppable.cost, events) {
            return false;
        }
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.hp = 1;
        }
        if id == self.state.player {
            self.say("You refuse to fall!");
        }
        true
    }
}
