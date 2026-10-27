//! Animated objects in play (spec §8 "Animated objects"; docs/dev/animated.md): what falls when
//! one dies, and the talents against them.
//!
//! **Drops**: what an animated creature is made of (its household body) falls **intact** by the
//! drop rule's chance (`[drops] intact`, raised by the player's Salvager whoever made the kill),
//! drawn from `drop/<actor id>` (never the gameplay stream); else it **breaks into its parts**
//! ([`crate::piece::break_up`]: loose gems, pieces of its grouped materials by `[drops] pieces`,
//! its magic lost), with a line when the player sees it fall ("The candlestick breaks apart.").
//! Anything else it carried falls as any creature's does.
//!
//! **Unbinder** (Bane): the attacker's talents' to-hit and damage against a creature of a tag.

use super::{Actor, ActorId, World};
use crate::combat::Profile;
use crate::geom::Pos;
use crate::object::{Stack, break_up};
use crate::rng::Rng;

impl World {
    /// What `actor`, a creature that just died, leaves of what it carried: everything, but for an
    /// animated one its body, which falls whole by the drop rule's chance (one draw per body
    /// stack) or broken into its parts. Also what broke, in order (for the break lines).
    pub(super) fn remains(&self, actor: &Actor) -> (Vec<Stack>, Vec<Stack>) {
        let Some(body) = self.content.animated_body(actor.creature) else {
            return (actor.inventory.clone(), Vec::new());
        };
        let salvage = self
            .state
            .player()
            .talent_mods(&self.content)
            .salvage_percent;
        let drops = self.content.animated().drops;
        let intact = (i32::from(drops.intact) + salvage).clamp(0, 100);
        let mut rng = Rng::stream(self.state.seed, &format!("drop/{}", actor.id.0));
        let (mut fallen, mut broken) = (Vec::new(), Vec::new());
        for stack in &actor.inventory {
            if stack.object != body.object || rng.chance(intact.unsigned_abs()) {
                fallen.push(stack.clone());
            } else {
                fallen.extend(break_up(stack, &self.content, drops.pieces));
                broken.push(stack.clone());
            }
        }
        (fallen, broken)
    }

    /// The lines for `broken`, an animated thing's components that broke as it fell at `at` on
    /// `level`: one each, if the player sees the cell ("The candlestick breaks apart."), else
    /// none.
    pub(super) fn break_lines(&self, level: &str, at: Pos, broken: &[Stack]) -> Vec<String> {
        if broken.is_empty()
            || level != self.state.current_level
            || !self.perception.sees_actors_at(at)
        {
            return Vec::new();
        }
        (broken.iter())
            .map(|stack| {
                let (one, many) = stack.names(&self.content);
                match stack.count {
                    1 => format!("The {one} breaks apart."),
                    n => format!("The {n} {many} break apart."),
                }
            })
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
