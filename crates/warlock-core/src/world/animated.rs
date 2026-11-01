//! Animated objects in play (spec §8 "Animated objects"; docs/dev/animated.md): what falls when
//! one dies.
//!
//! **Drops**: what an animated creature is made of (its household body) falls **intact** by the
//! drop rule's chance (`[drops] intact`),
//! drawn from `drop/<actor id>` (never the gameplay stream); else it **breaks into its parts**
//! ([`crate::piece::break_up`]: loose gems, pieces of its grouped materials by `[drops] pieces`,
//! its magic lost), with a line when the player sees it fall ("The candlestick breaks apart.").
//! Anything else it carried falls as any creature's does.

use super::{Actor, World};
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
        let drops = self.content.animated().drops;
        let intact = u32::from(drops.intact);
        let mut rng = Rng::stream(self.state.seed, &format!("drop/{}", actor.id.0));
        let (mut fallen, mut broken) = (Vec::new(), Vec::new());
        for stack in &actor.inventory {
            if stack.object != body.object || rng.chance(intact) {
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
}

// Tests: `world/animated_tests.rs`.
