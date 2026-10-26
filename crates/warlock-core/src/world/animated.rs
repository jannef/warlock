//! Animated objects in play (spec §8 "Animated objects"; docs/dev/animated.md): what falls when
//! one dies, and the talents against them.
//!
//! **Drops**: what an animated creature is made of (its household body) falls **intact** by the
//! drop rule's chance (`[drops] intact`, raised by the player's Salvager whoever made the kill),
//! drawn from `drop/<actor id>` (never the gameplay stream); else nothing of it falls. Anything
//! else it carried falls as any creature's does.
//!
//! **Unbinder** (Bane): the attacker's talents' to-hit and damage against a creature of a tag.

use super::{Actor, ActorId, World};
use crate::combat::Profile;
use crate::object::Stack;
use crate::rng::Rng;

impl World {
    /// What `actor`, a creature that just died, leaves of what it carried: everything, but for an
    /// animated one its body, which falls whole by the drop rule's chance (one draw) or not at all.
    pub(super) fn remains(&self, actor: &Actor) -> Vec<Stack> {
        let Some(body) = self.content.animated_body(actor.creature) else {
            return actor.inventory.clone();
        };
        let salvage = self
            .state
            .player()
            .talent_mods(&self.content)
            .salvage_percent;
        let intact = (i32::from(self.content.animated().drops.intact) + salvage).clamp(0, 100);
        let mut rng = Rng::stream(self.state.seed, &format!("drop/{}", actor.id.0));
        (actor.inventory.iter())
            .filter(|s| s.object != body.object || rng.chance(intact.unsigned_abs()))
            .cloned()
            .collect()
    }

    /// `profile` of an attack by `attacker` at `target`, with its talents' Banes (Unbinder)
    /// against the tags of `target`'s type.
    pub(super) fn bane(&self, attacker: ActorId, target: ActorId, mut profile: Profile) -> Profile {
        let Some(me) = self.state.actors.get(&attacker) else {
            return profile;
        };
        if me.talents.is_empty() {
            return profile;
        }
        let Some(them) = self.state.actors.get(&target) else {
            return profile;
        };
        let tags = &self.content.creature(them.creature).tags;
        for (tag, to_hit, damage) in &me.talent_mods(&self.content).banes {
            if tags.contains(tag) {
                profile.to_hit += to_hit;
                profile.bonus += damage;
            }
        }
        profile
    }
}

// Tests: `world/animated_tests.rs`.
