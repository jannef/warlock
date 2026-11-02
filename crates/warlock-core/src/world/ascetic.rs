//! The ascetic's ki tricks (spec §8 "Class abilities"; the ki tricks design §6.2): class actives
//! paid in ki ([`crate::content::ClassActive`]), each one function here with one line in
//! `use_class` (`world/class_ability.rs`).
//!
//! - **Patient defence** (`use:patient_defence`, level 18, for a point of ki and a wait's
//!   price): a degree of its status (Poised) on the player until their next action. The status's
//!   own `combat` gives Dodge +4, and while it lasts [`World::free_blows_of`] switches the
//!   riposte on, which answers a melee blow missed by the player's Parry **or dodged** (missed by
//!   their Dexterity or Dodge, [`World::poised`]). Given during the player's own priced action,
//!   the status sits on `World::fresh`, so that action does not ease it: the creatures' turns in
//!   between meet it, and the player's next action, whatever it is, ends it. Refused, no time and
//!   no ki, only by `may_use_class` with no ki; never for "nobody near", and a second use while
//!   Poised keeps it for one more action.
//!
//! Nothing here draws.

use super::{Actor, Spent, World};
use crate::action::Event;
use crate::content::ActionKind;

// Patient defence (#849).
impl World {
    /// Patient defence: the player waits in their guard (a degree of its status, Poised), for
    /// the time of a wait; its ki was spent by `use_class`.
    pub(super) fn patient_defence(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(status) = self
            .state
            .player()
            .talent_mods(&self.content)
            .patient_defence
        else {
            return Spent::Nothing;
        };
        self.afflict(player, status, 1, Some(player), events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Whether `actor` is Poised: it suffers the status of its Patient defence. Its riposte is
    /// on ([`World::free_blows_of`]) and answers a blow it dodged as well as one it parried.
    pub(super) fn poised(&self, actor: &Actor) -> bool {
        !actor.talents.is_empty()
            && (actor.talent_mods(&self.content).patient_defence)
                .is_some_and(|s| actor.effects.has(s))
    }
}
