//! Listening at a door (spec §5 "Hearing"; #1026): the use of the Listen skill, the Listen
//! ability ([`crate::ability::Ability::Listen`], [`crate::Action::Listen`]).
//!
//! The player stops at a closed door next to them for a turn. During it their hearing threshold
//! for a sound that reaches them by way of that door is lower by what Listen gives (its
//! `hearing` effect, one per 10 points) and the `[listen]` `keener` of `data/rules/tuning.toml`
//! besides (`noise::hearers`, which reads [`crate::noise::Listening`]). What is heard comes as
//! ever: sound markers in the perception, and every sound of the turn told in words, or that
//! nothing was heard. Every listen marks Listen, whatever is heard. Refused, taking no time,
//! without the skill or a closed door that way.

use super::{Spent, World};
use crate::ability::Ability;
use crate::content::ActionKind;
use crate::geom::Dir;
use crate::noise::Listening;
use crate::skills::{Effect, Use};

impl World {
    /// The player listens at the door in `dir` (module docs).
    pub(super) fn listen(&mut self, dir: Dir) -> Spent {
        let player = self.state.player;
        if !Ability::Listen.had_by(self.state.player(), &self.content) {
            self.say("You do not know how to listen so.");
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        let level = self.state.current();
        let closed = level
            .terrain
            .get(at)
            .is_some_and(|t| self.content.terrain(*t).opens_to.is_some());
        if !closed {
            self.say("There is no closed door there to listen at.");
            return Spent::Nothing;
        }
        let skill = self
            .state
            .player()
            .skill_effect(&self.content, Effect::Hearing);
        let more = self.content.tuning().listen.keener;
        let keener = u16::try_from(skill.max(0)).unwrap_or(u16::MAX) + more;
        self.state.listening = Some(Listening { at, keener });
        if self.trains(Use::Listen) {
            self.train(Use::Listen);
        }
        self.say("You press your ear to the door and listen.");
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }
}
