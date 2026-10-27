//! Unstable rooms (spec §7 "Special rooms", design §9.6): every step onto a cell inside one, the
//! player's or a creature's, has the effects' `warp` chance (`Here::warp`, one draw of the
//! gameplay stream, only on such a cell) to move the stepper to a free cell of the level, as a
//! teleport trap moves its victim ([`World::teleport`]: where it lands is a hash, not a draw).
//! The trap's own lines say it: "You are wrenched elsewhere." for the player, "The kobold
//! vanishes!" for a creature they saw go. Every step lands through [`World::step_onto_trap`]
//! (or [`World::step_willingly`], a known trap sidestepped), which calls [`World::warp_step`]
//! last, so a trap that already moved the stepper leaves nothing to warp. An incorporeal
//! creature passing through presses no trap and is never warped either.

use super::talk::capitalized;
use super::{ActorId, World};
use crate::action::Event;
use crate::geom::Pos;

impl World {
    /// Actor `who` has just stepped onto `at`: in an unstable room it may be moved elsewhere on
    /// the level (module docs).
    pub(super) fn warp_step(&mut self, at: Pos, who: ActorId, events: &mut Vec<Event>) {
        let level = self.state.current();
        let chance = level.effects_at(&self.content, at).warp();
        let standing =
            (self.state.actors.get(&who)).is_some_and(|a| a.pos == at && a.level == level.id);
        if chance == 0 || !standing || !self.state.rng.chance(u32::from(chance)) {
            return;
        }
        let seen = self.perception.sees_actors_at(at);
        let name = capitalized(&self.called(&self.state.actors[&who]));
        self.teleport(who, at, events);
        if who == self.state.player {
            self.say("You are wrenched elsewhere.".to_string());
        } else if seen {
            self.say(format!("{name} vanishes!"));
        }
    }
}
