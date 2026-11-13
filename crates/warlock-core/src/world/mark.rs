//! The mark: whose guard the thief's openings are in (spec §8 "Resources"; the openings design
//! `docs/design/thief-openings.md` §3; `docs/dev/openings.md`). A pool `marked` in data
//! (Openings) is worth something against one creature only, the holder's mark
//! ([`Actor::mark`]). Only the player has one: only the player chooses attacks.
//!
//! - [`World::choose_mark`], before every attack the player **chooses** at a creature (a blow by
//!   moving or attacking, a kick, a bash, a class or weapon active's blow, a shot, a throw, a
//!   single-target attack spell, a feint): no mark, it becomes the mark and the pool is kept; the mark,
//!   nothing; another living creature, the pool is emptied (said, when it held anything) and
//!   the target becomes the mark.
//! - [`World::free_blow_at`], before the player's free blows (a riposte, an opportunist's, a
//!   lurk's, a hair trigger's, a cleave): sets the mark when none stands, never moves it.
//! - [`World::builds_on`]: a marked pool builds on nobody but the mark (a sneak attack of a free
//!   blow at a stranger builds nothing; `ambush.rs`).
//! - The clears, the pool kept: the mark dies, by any hand ([`World::mark_dies`], in
//!   `fight.rs::dies`, which first builds `mark_kill` when the kill is the holder's own), or
//!   leaves the game otherwise ([`World::unmark`], where a creature departs); the player changes
//!   level ([`World::travel`]); the pool decays to 0 ([`faded`], in `World::live`). A mark out of
//!   sight stays the mark.
//! - [`World::off_mark`]: an active paid from a marked pool (Low blow, Master thief's finisher)
//!   aimed at a creature that is not the mark is refused while a mark stands.
//!
//! Nothing here draws.

use super::{Actor, ActorId, World};
use crate::content::{Build, ClassActive, Content, ResourceId, possessive};
use crate::geom::{Dir, Pos};

impl World {
    /// The marked pools actor `id` holds (`marked` in data: Openings), in resource order.
    fn marked_pools(&self, id: ActorId) -> Vec<ResourceId> {
        (self.state.actors.get(&id).into_iter())
            .flat_map(|a| a.resources.iter())
            .map(|pool| pool.resource)
            .filter(|&r| self.content.resource(r).marked)
            .collect()
    }

    /// Actor `id`'s mark, while it is still in the game.
    pub(super) fn mark_of(&self, id: ActorId) -> Option<ActorId> {
        let mark = self.state.actors.get(&id)?.mark?;
        self.state.actors.contains_key(&mark).then_some(mark)
    }

    /// The player's mark as they know it, for what they are told: "the kobold" while they see
    /// it, else "an unseen foe".
    fn mark_called(&self, mark: ActorId) -> String {
        match self.state.actors.get(&mark) {
            Some(a) if self.perception.sees_actors_at(a.pos) => self.called(a),
            _ => "an unseen foe".into(),
        }
    }

    /// The player chooses an attack at `target`: the mark rule, before the attack (so its sneak
    /// attack builds in the new mark's pool). Without a marked pool, nothing.
    pub(super) fn choose_mark(&mut self, target: ActorId) {
        let player = self.state.player;
        if target == player || !self.state.actors.contains_key(&target) {
            return;
        }
        let pools = self.marked_pools(player);
        if pools.is_empty() {
            return;
        }
        match self.mark_of(player) {
            Some(mark) if mark == target => return,
            Some(mark) => {
                let called = self.mark_called(mark);
                let me = self.player_mut();
                let mut held = 0u16;
                for &r in &pools {
                    if let Some(pool) = me.resources.get_mut(r) {
                        held = held.saturating_add(pool.value);
                        pool.value = 0;
                    }
                }
                if held > 0 {
                    self.say(format!("Your openings on {called} are lost."));
                }
            }
            None => {}
        }
        self.player_mut().mark = Some(target);
    }

    /// The player chooses a shot or a throw at cell `at` (`free`: a free shot, which they did
    /// not choose): [`World::choose_mark`] (or [`World::free_blow_at`]) the creature standing
    /// there, if any. Who the missile meets on its way is not chosen.
    pub(super) fn choose_at_cell(&mut self, at: Pos, free: bool) {
        let level = &self.state.current().id;
        let Some(target) = self.state.actor_at(level, at).map(|a| a.id) else {
            return;
        };
        if free {
            self.free_blow_at(target);
        } else {
            self.choose_mark(target);
        }
    }

    /// The player strikes a free blow at `target`: it becomes the mark when none stands (there
    /// is nothing to lose), else nothing moves.
    pub(super) fn free_blow_at(&mut self, target: ActorId) {
        let player = self.state.player;
        if target == player || self.marked_pools(player).is_empty() {
            return;
        }
        if self.mark_of(player).is_none() && self.state.actors.contains_key(&target) {
            self.player_mut().mark = Some(target);
        }
    }

    /// Whether a marked pool of `holder`'s may build on an attack at `target`: no mark stands,
    /// or `target` is it.
    pub(super) fn builds_on(&self, holder: ActorId, target: ActorId) -> bool {
        self.mark_of(holder).is_none_or(|m| m == target)
    }

    /// Creature `id` dies, killed by `by` (if anyone): the mark's kill builds its killer's
    /// `mark_kill` (Patient blade's two), and then every mark on it is cleared, the pool kept,
    /// whoever killed it. Called from `fight.rs::dies` beside `Build::Kill`.
    pub(super) fn mark_dies(&mut self, id: ActorId, by: Option<ActorId>) {
        if let Some(by) = by.filter(|&b| self.mark_of(b) == Some(id)) {
            self.gain_resource(by, Build::MarkKill);
        }
        self.unmark(id);
    }

    /// Creature `id` leaves the game (it died, or departed): the player's mark on it is
    /// cleared, the pool kept.
    pub(super) fn unmark(&mut self, id: ActorId) {
        let player = self.state.player;
        if let Some(me) = self.state.actors.get_mut(&player)
            && me.mark == Some(id)
        {
            me.mark = None;
        }
    }

    /// The player changes level: the mark is cleared, the pool kept (a mark on another floor
    /// would make the first fight downstairs empty it for no reason they can see).
    pub(super) fn unmark_on_travel(&mut self) {
        let player = self.state.player;
        if let Some(me) = self.state.actors.get_mut(&player) {
            me.mark = None;
        }
    }

    /// Whether class active `active`, aimed `dir`, is refused by the mark (no time; said): it is
    /// paid from a marked pool, a mark stands, and the creature that way is not it.
    pub(super) fn off_mark(&mut self, active: ClassActive, dir: Option<Dir>) -> bool {
        let player = self.state.player;
        let Some(dir) = dir else {
            return false;
        };
        let content = self.content.clone();
        let paid = crate::ability::class_cost(self.state.player(), &content, active)
            .is_some_and(|cost| content.resource(cost.resource).marked);
        let Some(mark) = self.mark_of(player).filter(|_| paid) else {
            return false;
        };
        let to = self.player_pos().step(dir);
        let level = &self.state.current().id;
        let there = self.state.actor_at(level, to).map(|a| a.id);
        if there.is_none_or(|t| t == mark) {
            return false;
        }
        let called = self.mark_called(mark);
        self.say(format!(
            "Your openings are in {} guard.",
            possessive(&called)
        ));
        true
    }
}

/// Time passed for `actor` (`World::live`): its mark is cleared once every marked pool it
/// holds has decayed to 0. The one place time touches the mark.
pub(super) fn faded(actor: &mut Actor, content: &Content) {
    if actor.mark.is_none() {
        return;
    }
    let mut marked = (actor.resources.iter()).filter(|p| content.resource(p.resource).marked);
    if marked.all(|p| p.value == 0) {
        actor.mark = None;
    }
}
