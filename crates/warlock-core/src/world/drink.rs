//! Drinking at a healing spring (spec §7 "Special rooms"; the design's §9.10): [`Action::Drink`],
//! the Drink ability of the abilities menu's *Here* group, offered beside a spring.
//!
//! - **Where**: beside a cell whose terrain a room effect's `feature` names and that `offers`
//!   drink ([`crate::content::Content::offer_at`]); the first such cell around the player in
//!   [`Dir::ALL`] order. Refused, taking no time, anywhere else.
//! - **Time**: the table's `seconds`, in turns of the `drink` action ([`World::keep_at`], the
//!   rest's machinery; the last cut short to end on time): a creature coming into view, a
//!   message, or a blow stops it, and a broken drink gives nothing, but the time is spent.
//! - **The answer**: one outcome drawn from the table by its kinds' weights (a spring has no
//!   side, so nothing leans), and its hooks (`outcome.rs`). A spring never runs dry: its table's
//!   `exhaust` is 0 and nothing is saved.
//!
//! Every draw is from the gameplay stream.
//!
//! [`Action::Drink`]: crate::action::Action::Drink

use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Offer};
use crate::geom::{Dir, Pos};
use crate::time::Duration;

/// A drink under way: the spring's cell and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Draught {
    spring: Pos,
    seconds: u32,
}

impl World {
    /// The spring beside the player: the first cell around them (in [`Dir::ALL`] order) whose
    /// terrain offers Drink.
    fn spring_beside(&self) -> Option<Pos> {
        let level = self.state.current();
        let here = self.player_pos();
        (Dir::ALL.iter().map(|&d| here.step(d))).find(|&p| {
            (level.terrain_at(p))
                .and_then(|t| self.content.offer_at(t))
                .is_some_and(|(offer, _)| offer == Offer::Drink)
        })
    }

    /// The player begins to drink at the spring beside them; refused, taking no time, with none.
    pub(super) fn start_drink(&mut self) -> Spent {
        let Some(spring) = self.spring_beside() else {
            self.say("There is no spring here to drink from.");
            return Spent::Nothing;
        };
        let terrain = self.state.current().terrain_at(spring);
        let Some((_, table)) = terrain.and_then(|t| self.content.offer_at(t)) else {
            return Spent::Nothing;
        };
        let seconds = table.seconds;
        self.say("You cup your hands and drink from the spring.");
        Spent::Drink(Draught { spring, seconds })
    }

    /// Drinks turn after turn until the drink is done or something breaks it off (module docs):
    /// its last turn cut short to end exactly when its time is up, as a prayer's is.
    pub(super) fn drink(&mut self, draught: Draught, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(draught.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Drink);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.drunk(draught.spring, events),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you stop drinking.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop drinking."),
        }
    }

    /// The drink at the spring on `spring` is done: one outcome (module docs).
    fn drunk(&mut self, spring: Pos, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let terrain = self.state.current().terrain_at(spring);
        let Some((_, table)) = terrain.and_then(|t| content.offer_at(t)) else {
            return;
        };
        if let Some(outcome) = self.draw_outcome(table, table.kinds) {
            self.befall(outcome, spring, events);
        }
    }
}
