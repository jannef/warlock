//! Throwing one's voice (spec §5 "Hearing"; #235): the player makes a voice sound at a cell they
//! see within [`VOICE_RANGE`], as loud there as a voice ([`crate::noise::VOICE_NOISE`]) and
//! made there, not where they stand. It goes through the one noise path ([`World::make_noise`],
//! [`Act::ThrowVoice`]): creatures that hear it and do not see its cell go and look there (the
//! AI's listening step, `ai.rs`), as they go to any noise whose source they do not see. The
//! player made it, so they do not hear it; the map shows where it sounded
//! ([`Event::VoiceThrown`]). An ability in the one registry ([`crate::ability`]).

use super::{Spent, World};
use crate::action::Event;
use crate::content::ActionKind;
use crate::geom::Pos;
use crate::noise::Act;

/// How far the player throws their voice, in cells (a disc, as ranges are).
pub const VOICE_RANGE: i32 = 6;

impl World {
    /// The player throws their voice to `at`: refused, taking no time, without the ability (the
    /// bard's Throw your voice), on the overworld (which is silent), at their own cell, out of reach, or at a cell they do not see or that is solid.
    pub(super) fn throw_voice(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        if !crate::ability::Ability::ThrowVoice.had_by(self.state.player(), &self.content) {
            self.say("You do not know how to throw your voice.");
            return Spent::Nothing;
        }
        if !self.can_loose_here() || !self.within_reach(at, VOICE_RANGE, "throw your voice") {
            return Spent::Nothing;
        }
        let solid = self.state.current().is_opaque(&self.content, at);
        if solid || !self.perception.sees_actors_at(at) {
            self.say("You can throw your voice only to a place you see.");
            return Spent::Nothing;
        }
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::ThrowVoice);
        self.make_noise(player, Act::ThrowVoice(at));
        events.push(Event::VoiceThrown { at });
        self.say("You throw your voice.");
        Spent::Turn(price)
    }
}
