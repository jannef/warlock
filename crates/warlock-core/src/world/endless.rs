//! The endless branch (spec §7 "The endless branch", #1010): a dungeon branch with no last floor
//! whose floors are never remembered. Level ids stay `branch:N`, the depth, so everything that
//! reads a depth from an id (danger, item level, locks, names, the stairs' links) works as on any
//! branch; only one floor is ever there, so an id comes back as a different floor.
//!
//! **Made**: [`World::visit`] generates a floor when its id is missing, as for every level. Its
//! seed is [`floor_seed`]: the named stream `endless/<n>` of the run seed, `n` the count of floors
//! made in the run ([`EndlessFloors`], saved), and every stream of the floor (`map/<id>`,
//! `population/<id>`, its spawner's, ...) comes from that seed under its usual label. So two visits
//! to one depth differ, and a replay makes the same floors.
//!
//! **Forgotten**: [`World::forget_floors`], once the action that left is over (`World::apply`):
//! whoever crossed with the player (allies, `follow_along`; hostile stair followers, who arrive
//! after `World::travel` returns) stands on the new floor by then, and only those left behind go
//! with the old one. Its level state goes (terrain, piles, chests, traps, the player's memory),
//! its actors, its cached lights ([`crate::light::LightCache::forget`]), and what a trader on it
//! offered. A unique creature on it may come again: it never met its end (`progress.uniques`).
//!
//! **Kept beasts** never stay behind: a companion or familiar crosses wherever it stands, held fast
//! or not (`follow_along`, [`super::Actor::kept`]), so none is lost here. Any other ally left is
//! gone with the floor, and the player is told (the seen ones have just been said to be left
//! behind): "You will not see the wolf again."
//!
//! **Paid**: a level is worth its arrival's experience once. Here "a level" is a depth: only a
//! floor deeper than any reached before pays ([`EndlessFloors::pays`]), or going up and down one
//! staircase, which always arrives on the way back, would pay for every step.

use super::{ActorId, World, WorldState};
use crate::action::Event;
use crate::content::{Content, parse_branch_level};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The seed of level `id` for a run of `seed`, `made` floors made so far: a floor of an endless
/// branch's own (the module docs), the run's for every other level.
pub(super) fn floor_seed(content: &Content, seed: u64, id: &str, made: u32) -> u64 {
    if content.is_endless_floor(id) {
        crate::rng::stream_key(seed, &format!("endless/{made}"))
    } else {
        seed
    }
}

/// What the run keeps of the endless branches, which keep nothing else (saved; left out while
/// empty, so a run that never goes there hashes as before).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndlessFloors {
    /// Floors made so far in the run: the next floor's stream.
    made: u32,
    /// The deepest floor reached, by branch.
    deepest: BTreeMap<String, u32>,
}

impl EndlessFloors {
    pub fn is_empty(&self) -> bool {
        self.made == 0 && self.deepest.is_empty()
    }

    /// Floor `id` has been made: the count goes on, and its depth may be the deepest yet.
    pub(super) fn made(&mut self, content: &Content, id: &str) {
        if !content.is_endless_floor(id) {
            return;
        }
        self.made = self.made.saturating_add(1);
        if let Some((branch, depth)) = parse_branch_level(id) {
            let deepest = self.deepest.entry(branch.to_string()).or_default();
            *deepest = depth.max(*deepest);
        }
    }

    /// Whether arriving on level `id`, not there yet, pays as a first arrival: any level but a
    /// floor of an endless branch no deeper than it has been.
    pub(super) fn pays(&self, content: &Content, id: &str) -> bool {
        match parse_branch_level(id).filter(|_| content.is_endless_floor(id)) {
            Some((branch, depth)) => depth > self.deepest.get(branch).copied().unwrap_or(0),
            None => true,
        }
    }
}

impl WorldState {
    /// The seed level `id` is made from, if it is made now.
    pub(super) fn level_seed(&self, content: &Content, id: &str) -> u64 {
        floor_seed(content, self.seed, id, self.endless.made)
    }
}

impl World {
    /// After an action that changed level: every floor of an endless branch but the current one
    /// is forgotten (the module docs).
    pub(super) fn forget_floors(&mut self, events: &[Event]) {
        if !(events.iter()).any(|e| matches!(e, Event::LevelChanged { .. })) {
            return;
        }
        let here = &self.state.current_level;
        let left: Vec<String> = (self.state.levels.keys())
            .filter(|id| *id != here && self.content.is_endless_floor(id))
            .cloned()
            .collect();
        for id in left {
            self.forget_floor(&id);
        }
    }

    fn forget_floor(&mut self, id: &str) {
        let player = self.state.player;
        let on_it: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == id && !a.in_retinue())
            .map(|a| a.id)
            .collect();
        let allies: Vec<ActorId> = (on_it.iter().copied())
            .filter(|a| self.state.actors[a].master() == Some(player))
            .collect();
        if !allies.is_empty() {
            let who = self.pack_phrase(&allies);
            self.say(format!("You will not see {who} again."));
        }
        for actor in on_it {
            let Some(actor) = self.state.actors.remove(&actor) else {
                continue;
            };
            self.state.wares.remove(&actor.id);
            let def = self.content.creature(actor.creature);
            if def.unique {
                self.state.progress.uniques.remove(&def.id);
            }
        }
        self.state.levels.remove(id);
        self.lights.forget(id);
    }
}
