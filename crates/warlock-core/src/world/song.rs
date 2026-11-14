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
//! A song on its hearers ([`SongOn::Hearers`], the Lullaby, #873) is sung over every creature
//! that **hears** it (the Sing noise's hearers, asked before it is made, as War cry asks them:
//! ground truth, so an unseen or invisible foe in earshot is checked like any other) and is
//! hostile to the singer ([`World::stance`]); friends, allies, and the peaceful only hear it.
//! Each, in actor-id order, checks its `check` stat's modifier (and its talents' checks of that
//! stat) against the song's `difficulty` plus the singer's Music craft ([`Effect::SongCraft`],
//! read here and nowhere else), two gameplay draws apiece. Failed, it suffers the song's status
//! at the song's degrees, caused by the singer; made, nothing. The player is told how those they
//! see took it. A song with no hostile to hear it is sung all the same (the cost, the noise,
//! the marks): who hears is the world's, never the player's to know.
//!
//! Refused, taking no time and nothing spent, on the overworld (it is silent) and, by the
//! `song` action's `blocks` (`data/status.toml`), while Silenced. Nothing is drawn but the
//! training marks and a hearers song's checks.

use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::check;
use crate::content::{ActionKind, ClassActive, SongOn, Stance, StatUse, StatusId};
use crate::noise::Act;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
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
        let me = self.state.player();
        let music = me.skill_effect(&content, Effect::SongDegrees);
        let degrees = song.severity + u16::try_from(music.max(0)).unwrap_or(0);
        let craft = me.skill_effect(&content, Effect::SongCraft);
        // Who hears a song on its hearers: the noise it makes, asked before it is made.
        let heard = match song.on {
            SongOn::Singer => Vec::new(),
            SongOn::Hearers => {
                if let Some(def) = content.class_active(active) {
                    self.say(format!("You sing a {}.", def.name.to_lowercase()));
                }
                self.heard_by(Act::Sing)
            }
        };
        self.make_noise(player, Act::Sing);
        match (song.on, song.check) {
            (SongOn::Singer, _) => {
                self.afflict(player, song.status, degrees, Some(player), events);
            }
            (SongOn::Hearers, Some((stat, difficulty))) => {
                for id in heard {
                    if self.stance(player, id) == Stance::Hostile {
                        let against = difficulty + craft;
                        self.lull(id, stat, against, song.status, degrees, events);
                    }
                }
            }
            // The loader refuses a song on its hearers without a check.
            (SongOn::Hearers, None) => {}
        }
        self.train_stat(StatUse::Inspired);
        if self.trains(Use::Song) {
            self.train(Use::Song);
        }
        Spent::Turn(price)
    }

    /// Hearer `id` checks `stat` against a song of `difficulty`, two gameplay draws: its
    /// modifier and its talents' checks of that stat. Failed, it suffers `degrees` of `status`,
    /// caused by the player; the player is told how it took the song if they see it.
    fn lull(
        &mut self,
        id: ActorId,
        stat: Stat,
        difficulty: i32,
        status: StatusId,
        degrees: u16,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(them) = self.state.actors.get(&id) else {
            return;
        };
        let will = them.check_modifier(&content, stat);
        let seen = self.perception.sees_actors_at(them.pos);
        let who = super::talk::capitalized(&self.called(them));
        let falls = !check::check(&mut self.state.rng, will, difficulty);
        if falls {
            let player = self.state.player;
            self.afflict(id, status, degrees, Some(player), events);
        }
        if seen {
            self.say(if falls {
                format!("{who}'s eyes grow heavy.")
            } else {
                format!("{who} shakes off the song.")
            });
        }
    }
}
