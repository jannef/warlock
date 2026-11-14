//! Cackle, the witch's class active at 6 (spec §8 "Class abilities"; `use:cackle`;
//! docs/design/witch-and-wizard.md §5.1): a cackle, for the time of a cast and nothing else (no
//! mana, no cooldown), as loud as its talent's `noise` ([`Act::Cackle`], a voice: those who hear
//! it come to look, by the rules of every noise).
//!
//! Every hex of hers holds a while longer: for every creature the player's current perception
//! sees (in actor-id order), every effect on it whose status a hex lays
//! ([`crate::content::Content::hex_statuses`]) and whose giver is the player grows the talent's
//! `degrees`, through `afflict` (stacking `refresh`: to the status's `max`, its degree's time
//! started afresh; one at its max only starts afresh). Another's hex, and a creature out of
//! sight, are untouched.
//!
//! Refused, taking no time, when no creature in sight is under a hex of hers (the player knows
//! their own hexes: nothing leaks), as well as without the ability and while a status blocks a
//! cast (`World::may_use_class`). No draws, and no marks (no school, no skill: Intimidate is not
//! trained by it).

use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Stacking, StatusId};
use crate::noise::Act;

impl World {
    /// Cackle (module docs), for the time of a cast.
    pub(super) fn cackle(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(cackle) = self.state.player().talent_mods(&content).magic.cackle else {
            return Spent::Nothing;
        };
        let hexes = self.own_hexes_in_sight();
        if hexes.is_empty() {
            self.say("No one you see is under your hex.");
            return Spent::Nothing;
        }
        let price = self.price_of(player, ActionKind::Cast);
        self.say("You cackle.");
        self.make_noise(player, Act::Cackle);
        for (id, status, severity) in hexes {
            // A hex that adds its degrees takes only the cackle's; the rest the sum, and keep
            // the higher.
            let degrees = match content.status(status).stacking {
                Stacking::Add => cackle.degrees,
                Stacking::Refresh | Stacking::Highest => severity + cackle.degrees,
            };
            self.afflict(id, status, degrees, Some(player), events);
        }
        self.say("Your hexes hold.");
        Spent::Turn(price)
    }

    /// Every hex of the player's on a creature they see now: `(creature, status, severity)`, in
    /// actor-id order, each creature's in status order.
    pub(super) fn own_hexes_in_sight(&self) -> Vec<(ActorId, StatusId, u16)> {
        let player = self.state.player;
        let hexes = self.content.hex_statuses();
        let level = &self.state.current_level;
        (self.state.actors.values())
            .filter(|a| a.id != player && a.level == *level)
            .filter(|a| self.perception.sees_actors_at(a.pos))
            .flat_map(|a| {
                (a.effects.iter())
                    .filter(|e| e.by == Some(player) && hexes.contains(&e.status))
                    .map(|e| (a.id, e.status, e.severity))
            })
            .collect()
    }
}
