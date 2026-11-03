//! Songs (spec §8 "Class abilities", the bard's songs; #870): the one place a song is sung.
//!
//! A song is a class active ([`ClassActive::sings`]) that a talent's `song` effect names
//! ([`crate::talent::Song`]), used from the abilities menu (`use:inspire`). [`World::sing`]
//! pays its one cost (its resource, Inspire's Inspiration, or its mana; a Virtuoso's nothing),
//! makes its noise ([`Act::Sing`], a voice of [`crate::noise::SONG_NOISE`] where the singer
//! stands: those who hear it and do not see the singer come to look, as for any noise), then
//! gives its status: a song on the singer (Inspire) gives it `severity` degrees plus what the
//! singer's Music adds ([`Effect::SongDegrees`]), at most the status's `max`. Then the Presence
//! mark ([`StatUse::Inspired`]) and Music's ([`Use::Song`]), and the time of a `song`.
//!
//! Refused, taking no time and nothing spent, on the overworld (it is silent) and, by the
//! `song` action's `blocks` (`data/status.toml`), while Silenced. Nothing is drawn but the
//! training marks.
//!
//! A song on its hearers ([`SongOn::Hearers`]) is data-ready (its `check` and `difficulty`,
//! Music's [`Effect::SongCraft`]); no song sings over hearers yet, and its checks are the
//! Lullaby's (#873).

use super::{Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, ClassActive, SongOn, StatUse};
use crate::noise::Act;
use crate::skills::{Effect, Use};
use crate::talent::SongCost;

impl World {
    /// The player sings song `active` (see the module docs); [`World::may_use_class`] has
    /// asked whether they may.
    pub(super) fn sing(&mut self, active: ClassActive, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(song) = self.state.player().talent_mods(&content).song(active) else {
            return Spent::Nothing;
        };
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let paid = match song.cost {
            Some(SongCost::Resource(cost)) => {
                self.spend_resource(player, cost.resource, cost.amount, events)
            }
            Some(SongCost::Mana(mana)) => self.spend_mana(player, mana),
            None => true,
        };
        if !paid {
            self.say("You have too little to sing it.");
            return Spent::Nothing;
        }
        let price = self.price_of(player, ActionKind::Song);
        self.make_noise(player, Act::Sing);
        let music = (self.state.player()).skill_effect(&content, Effect::SongDegrees);
        let degrees = song.severity + u16::try_from(music.max(0)).unwrap_or(0);
        match song.on {
            SongOn::Singer => self.afflict(player, song.status, degrees, Some(player), events),
            // No song sings over its hearers yet (the Lullaby's, #873).
            SongOn::Hearers => {}
        }
        self.train_stat(StatUse::Inspired);
        if self.trains(Use::Song) {
            self.train(Use::Song);
        }
        Spent::Turn(price)
    }
}
