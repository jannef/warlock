//! The noise an action makes (spec §5 "Hearing"; the loudness and its reach are
//! [`crate::noise`]): made where the action happens, heard at once by everyone it reaches.
//! `world/mod.rs` has nothing of it; the actions call these where they happen.

use super::{ActorId, World};
use crate::geom::Pos;
use crate::noise::{self, Act};
use crate::skills::Use;

impl World {
    /// The noise of `act` by `actor` on the current level (under its burden and its skills),
    /// heard by those it reaches. A step of the player's that a creature could have heard, but
    /// for their stealth, trains stealth.
    pub(super) fn make_noise(&mut self, actor: ActorId, act: Act) {
        self.make_noise_at(actor, act, 100);
    }

    /// The noise of `actor` stepping onto `to` (it has just moved), `fleeing` or not
    /// ([`World::flees`]): a stance may quieten a flight.
    pub(super) fn make_step_noise(&mut self, actor: ActorId, to: Pos, fleeing: bool) {
        let percent = match self.state.actors.get(&actor) {
            Some(a) if fleeing => self.content.stance_def(a.stance).flee_noise,
            _ => 100,
        };
        self.make_noise_at(actor, Act::Step(to), percent);
    }

    /// [`World::make_noise`] with the noise's loudness scaled to `percent`.
    pub(super) fn make_noise_at(&mut self, actor: ActorId, act: Act, percent: u32) {
        let (state, content) = (&self.state, &self.content);
        let Some(a) = state.actors.get(&actor) else {
            return;
        };
        let burden = state.load(content, a).penalty();
        let made = noise::noise(content, state.current(), a, burden, act);
        let made = made.map(|mut n| {
            n.loudness = u16::try_from(u32::from(n.loudness) * percent / 100).unwrap_or(u16::MAX);
            n
        });
        let made = made.filter(|n| n.loudness > 0);
        let sneaking = actor == state.player
            && matches!(act, Act::Step(_))
            && self.trains(Use::Sneak)
            && noise::plain_noise(content, state.current(), a, burden, act)
                .is_some_and(|plain| noise::heard_by_anyone(state, content, plain));
        if let Some(made) = made {
            self.spread_noise(made);
        }
        if sneaking {
            self.train(Use::Sneak);
        }
    }
}
