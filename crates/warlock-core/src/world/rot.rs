//! Food rotting in the world (spec §8 "Hunger"; the stages are [`crate::rot`]).
//!
//! - **In a pack** (the player's and creatures'): food rots away at its exact moment as the
//!   carrier lives through the time ([`World::rot_pack`], called by `World::live` in
//!   `status.rs`, which ends a step at [`until_rots`]), so it makes no difference how the time is
//!   cut up. The player is told ("Your loaf of bread has rotted away."), which does not stop a
//!   rest; the equipment follows the letters. Creatures on other levels are frozen, as with
//!   everything else; their food goes as they first live through time again.
//! - **In piles**: decided from the age alone, so nothing needs doing until it matters: food
//!   that has rotted away leaves the piles of the level the player is on as perception runs
//!   ([`World::rot_away`], like burnt-out fires), and those of another level when the player
//!   arrives there.

use super::{ActorId, World};
use crate::action::Positions;
use crate::object::Stack;
use crate::rot::{self, Stage};
use crate::time::GameTime;

impl World {
    /// How `stack` is by its age now: `None` for anything that is no food (or without hunger).
    pub(crate) fn stage(&self, stack: &Stack) -> Option<Stage> {
        rot::stage(&self.content, stack, self.state.time)
    }

    /// Food that has rotted away by `now` leaves actor `id`'s pack, the equipment following the
    /// positions; the player is told of each ("Your loaf of bread has rotted away.", logged without
    /// stopping a rest).
    pub(super) fn rot_pack(&mut self, id: ActorId, now: GameTime) {
        let content = &self.content;
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let gone = |s: &Stack| rot::stage(content, s, now) == Some(Stage::Gone);
        let rotted = Positions::of(
            (actor.inventory.iter().enumerate()).filter_map(|(i, s)| gone(s).then_some(i)),
        );
        if rotted.is_empty() {
            return;
        }
        let told: Vec<String> = (rotted.iter())
            .map(|i| {
                let stack = &actor.inventory[i];
                let def = content.object(stack.object);
                match stack.count {
                    1 => format!("Your {} has rotted away.", def.name),
                    n => format!("Your {n} {} have rotted away.", def.plural),
                }
            })
            .collect();
        let mut i = 0;
        actor.inventory.retain(|_| {
            let keep = !rotted.contains(i);
            i += 1;
            keep
        });
        actor.equipment.after_removal(&rotted);
        if id == self.state.player {
            for text in told {
                self.note(text);
            }
        }
    }

    /// Food that has rotted away leaves the piles of the level the player is on. Checked as
    /// perception runs; food lying elsewhere goes when the player comes there.
    pub(super) fn rot_away(&mut self) {
        let content = self.content.clone();
        let now = self.state.time;
        if content.hunger().is_none() {
            return;
        }
        let gone = |s: &Stack| rot::stage(&content, s, now) == Some(Stage::Gone);
        let level = self.state.current_mut();
        let cells: Vec<_> = (level.objects.iter())
            .filter(|(_, pile)| pile.iter().any(gone))
            .map(|(p, _)| *p)
            .collect();
        for p in cells {
            let pile: Vec<Stack> = (level.objects_at(p).iter())
                .filter(|s| !gone(s))
                .cloned()
                .collect();
            level.set_pile(p, pile);
        }
    }
}

/// How long from `now`, in seconds, until the first food in `inventory` rots away (0 when some
/// already has); `None` when none will.
pub(super) fn until_rots(
    inventory: &[Stack],
    content: &crate::content::Content,
    now: GameTime,
) -> Option<u64> {
    (inventory.iter())
        .filter_map(|s| rot::gone_at(content, s))
        .map(|at| at.since(now).as_seconds())
        .min()
}
