//! Regeneration (spec §8 "Creature gimmicks"; the dread design §5.9): a creature type's `regen`
//! is its own healing pace, read with its statuses' in the one place that decides the pace
//! ([`World::healing_rate_at`], through [`crate::status::Effects::regen`]: Burning's and
//! Corroded's `regen = 0` stop it). This module only tells the player of it: "The troll's wounds
//! close." when they see a regenerating creature (its `regen` above 100) get a tenth of its hit
//! points back in one passing of time.

use crate::text::capitalized;
use crate::world::{ActorId, World};

impl World {
    /// Creature `id`'s hit points before time passes for it, if it regenerates (its type's
    /// `regen` above 100) and is not the player: what [`World::say_wounds_close`] compares.
    pub(in crate::world) fn regenerating_hp(&self, id: ActorId) -> Option<i32> {
        let actor = self.state.actors.get(&id)?;
        (id != self.state.player && self.content.creature(actor.creature).regen > 100)
            .then_some(actor.hp)
    }

    /// Creature `id`, which had `before` hit points, has lived through some time: told if the
    /// player sees it and it got a tenth of its hit points back or more.
    pub(in crate::world) fn say_wounds_close(&mut self, id: ActorId, before: Option<i32>) {
        let Some(before) = before else {
            return;
        };
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let tenth = (actor.max_hp(&self.content) / 10).max(1);
        if actor.hp <= 0 || actor.hp - before < tenth || !self.perception.sees_actors_at(actor.pos)
        {
            return;
        }
        let who = capitalized(&self.called(actor));
        self.say(format!("{who}'s wounds close."));
    }
}
