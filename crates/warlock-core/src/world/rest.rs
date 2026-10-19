//! Resting (spec §8 "Time"): [`crate::action::Action::Rest`], its goal, and the turn-by-turn
//! loop of a long action ([`World::keep_at`]) that resting, eating, and digging share.
//! `world/mod.rs` has only the dispatch line.

use super::{ActorId, Spent, World};
use crate::action::{DiseaseNews, Event, MAX_REST, Rest};
use crate::content::ActionKind;
use crate::time::{self, GameTime};

/// How a long action (a rest, digging; [`World::keep_at`]) ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ended {
    /// There was nothing more to do.
    Done,
    /// This creature came into view.
    Came(ActorId),
    /// A message was logged that stops it.
    Said,
    /// The player died.
    Died,
}

impl World {
    /// Rests until `until`: turn after turn of waiting ([`World::keep_at`]), the last cut short to
    /// end exactly at `until`, whatever the player's speed ([`World::price_until`]); then says
    /// how long the player rested. The player heals faster all the while.
    pub(super) fn rest(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        self.resting = true;
        self.rest_until(until, goal, events);
        self.resting = false;
    }

    pub(super) fn rest_until(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        let start = self.state.time;
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| wait.min(w.price_until(until)),
        );
        let rested = self.state.time.since(start).spoken();
        let text = match (ended, goal) {
            (Ended::Died, _) => return,
            (Ended::Came(id), _) => format!(
                "{} comes into view; you stop resting after {rested}.",
                self.creature_called(id)
            ),
            (Ended::Said, _) => format!("You stop resting after {rested}."),
            (Ended::Done, Rest::For(_)) => format!("You rest for {rested}."),
            (Ended::Done, Rest::Dawn) => "You rest until dawn.".into(),
            (Ended::Done, Rest::Dusk) => "You rest until dusk.".into(),
        };
        self.say(text);
    }

    /// Passes turn after turn of a long action (a rest, digging) through the normal pipeline
    /// while `more` says there is more to do: the player does `act` (which returns what it
    /// costs), the others act, perception. It stops after a turn in which a creature came into
    /// view (one not seen the turn before; but for one who talks and fights no one,
    /// [`World::disturbs_rest`]: the player sees who it is) or a message was logged that stops a
    /// rest, and when the player dies. Turns that end before any creature that disturbs a rest
    /// could come into view ([`World::out_of_view_for`]), in which the player heard nothing, in
    /// which the weather the player sees did not change, in which none of the player's diseases
    /// showed or went badly (news that stops a rest), and in which no trap went off (a teleport
    /// moves a creature farther than its steps) pass without perception, which cannot change the
    /// outcome; only the last turn's events are kept, and the player's status effects beginning
    /// and ending in the others, and their diseases' news (so they are said).
    pub(super) fn keep_at(
        &mut self,
        events: &mut Vec<Event>,
        more: impl Fn(&Self) -> bool,
        mut act: impl FnMut(&mut Self, &mut Vec<Event>) -> i64,
    ) -> Ended {
        let said = self.said;
        let mut in_view = self.watched_in_view();
        let mut came = None;
        let alive = |w: &Self| w.state.death.is_none();
        while more(self) && came.is_none() && self.said == said && alive(self) {
            // What was perceived was said.
            events.clear();
            let unseen_until = self.state.time + self.out_of_view_for();
            let sprung = self.sprung;
            loop {
                events.retain(|e| matches!(e, Event::Status { .. } | Event::Disease { .. }));
                let price = act(self, events);
                self.run_turn(price, events);
                let done = !more(self) || self.said != said || !alive(self);
                // Something heard may be news (or someone muttering): perception says. So does a
                // disease showing or going badly (news that stops a rest), and a trap going off
                // (a teleport moves someone farther than its steps).
                let heard = !self.state.player().heard.is_empty();
                let ill = events.iter().any(
                    |e| matches!(e, Event::Disease { news, .. } if *news != DiseaseNews::Recovered),
                );
                let trapped = self.sprung != sprung;
                if done
                    || heard
                    || ill
                    || trapped
                    || self.state.time > unseen_until
                    || self.weather_turned()
                {
                    break;
                }
            }
            self.perceive(events);
            let now_in_view = self.watched_in_view();
            came = now_in_view.iter().find(|a| !in_view.contains(a)).copied();
            in_view = now_in_view;
        }
        match came {
            _ if !alive(self) => Ended::Died,
            Some(id) => Ended::Came(id),
            None if self.said != said => Ended::Said,
            None => Ended::Done,
        }
    }

    /// When a rest toward `goal` ends: at most [`MAX_REST`] from now.
    pub(super) fn start_rest(&mut self, goal: Rest) -> Spent {
        let now = self.state.time;
        let until = match goal {
            Rest::For(duration) => now + duration.min(MAX_REST),
            Rest::Dawn => now.next_at(time::DAWN),
            Rest::Dusk => now.next_at(time::DUSK),
        };
        if until == now {
            self.say("You do not rest.");
            return Spent::Nothing;
        }
        Spent::Rest(until, goal)
    }
}
