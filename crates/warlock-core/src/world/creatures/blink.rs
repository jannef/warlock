//! The blink to reach you (docs/design/monster-dread.md §5.6, G6b; #1130): a creature type with
//! `blinks` (`content/theft.rs`: the blink dog's) steps through nothing to its target's side
//! when its path there is blocked. Enemy-only. No creature blinks **away** (the owner's word,
//! design §17 D1).
//!
//! **When** ([`World::blinks_at`], asked in `ai.rs::fight` where a melee fighter would close in
//! and cannot: its path is held by someone (`Route::Wait`: a chokepoint, a corridor full of its
//! fellows) or there is none (`Route::Unreachable`: a door it cannot open)): it does not carry
//! its cooldown status (`blink`) and a landing cell exists ([`World::blink_landings`]). Deciding
//! draws nothing.
//!
//! **Where** ([`World::blink_landings`]): a cell beside the target (Chebyshev 1), within the blink's
//! `range` of the creature and in its own line of sight, that the teleport trap would land on
//! (walkable, no trap, no ice, no one there; `trap.rs`), and that it does not shun (a
//! sanctuary). So a closed door is still a door: it blinks only where it sees.
//!
//! **What** ([`World::creature_blinks`]): one cell of those, a gameplay draw
//! (`index(len)`), in row order; it is there at once (`teleport_to`: no step, no tread), takes
//! the cooldown status, and the blink costs a step.
//!
//! **Words**: "The blink dog is suddenly beside you." where the player sees it land, "The blink
//! dog is suddenly beside the kobold." beside another target; unseen, nothing.

use super::{ActorId, World};
use crate::action::Event;
use crate::content::{ActionKind, the_start};
use crate::geom::Pos;
use crate::sight::LineOfSight;

impl World {
    /// Whether creature `id` blinks now to its target on `at`, its path there blocked (module
    /// docs).
    pub(super) fn blinks_at(&self, id: ActorId, at: Pos) -> bool {
        let me = &self.state.actors[&id];
        let Some(blinks) = self.content.creature(me.creature).blinks.as_ref() else {
            return false;
        };
        me.pos.chebyshev(at) > 1
            && !me.effects.has(blinks.status)
            && !self.blink_landings(id, at).is_empty()
    }

    /// Where creature `id` may blink to beside its target on `at`, in row order (module docs).
    pub(super) fn blink_landings(&self, id: ActorId, at: Pos) -> Vec<Pos> {
        let me = &self.state.actors[&id];
        let Some(range) = (self.content.creature(me.creature).blinks.as_ref()).map(|b| b.range)
        else {
            return Vec::new();
        };
        let level = self.state.current();
        let radius = LineOfSight::radius_for(2 * range * range);
        let los = LineOfSight::compute(me.pos, radius, |p| level.is_opaque(&self.content, p));
        let shuns = self.shuns(id);
        (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| Pos::new(at.x + dx, at.y + dy)))
            .filter(|&p| p != at && p.chebyshev(me.pos) <= range && los.contains(p))
            .filter(|&p| self.may_land(p) && !shuns(p))
            .collect()
    }

    /// Creature `id` blinks beside its target on `at` (its AI saw to it that it can). Returns
    /// the price: a step's.
    pub(super) fn creature_blinks(&mut self, id: ActorId, at: Pos, events: &mut Vec<Event>) -> i64 {
        let price = self.price_of(id, ActionKind::Step);
        let cells = self.blink_landings(id, at);
        let me = &self.state.actors[&id];
        let (from, creature) = (me.pos, me.creature);
        let Some(blinks) = self.content.creature(creature).blinks.clone() else {
            return price;
        };
        if cells.is_empty() {
            return price;
        }
        let to = match cells.len() {
            1 => cells[0],
            n => cells[self.state.rng.index(n)],
        };
        self.teleport_to(id, (from, to), events);
        self.afflict(id, blinks.status, blinks.degrees, None, events);
        if self.perception.sees_actors_at(to) {
            let player = self.state.player;
            let level = self.state.current_level.clone();
            let beside = match self.state.actor_at(&level, at) {
                Some(a) if a.id == player => "you".to_string(),
                Some(a) => self.called(a),
                None => return price,
            };
            let name = &self.content.creature(creature).name;
            self.say(format!("{} is suddenly beside {beside}.", the_start(name)));
        }
        price
    }
}
