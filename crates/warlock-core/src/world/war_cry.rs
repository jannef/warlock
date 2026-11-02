//! War cry, the berserker's class active at 25 (spec §8 "Class abilities"; `use:war_cry`;
//! docs/design/berserker.md §6.4): a roar, for 4 rage (its talent's `resource`, spent by
//! `World::use_class`) and the time of a wait, as loud as a howl ([`Act::Howl`]: heard across
//! the floor, it wakes a den's sleepers and sends patrols to look, by the rules of every noise).
//!
//! Every hostile that **hears** it (the howl's hearers, asked before it is made, as Hound's bay
//! does), **sees** the player now ([`World::sees_now`]: fear needs the sight of the roaring
//! berserker), and **has vitals** (`Content::has_vitals`: the bones, the spirits, the animated
//! have no heart to quail) rolls a check, in actor-id order, two gameplay draws apiece: its
//! Willpower modifier (and its talents' Willpower checks) against the talent's `difficulty` +
//! the player's Strength modifier + their Intimidate ([`Effect::Intimidate`], read here and
//! nowhere else). Failed, it suffers the talent's status (Frightened, one that `flees`) at its
//! degrees, caused by the player; made, it stands. No level cut (the owner: "check alone").
//!
//! The one refusal is the rage's: the cost not held ("You have not the rage for it.", in
//! `use_class`). A roar with nobody to hear it is no refusal: the turn, the rage, and the noise
//! are spent, and nobody quails. Each cry marks Intimidate ([`Use::Intimidate`], the training
//! stream), whatever comes of it.

use super::{Spent, World};
use crate::action::Event;
use crate::check;
use crate::content::{ActionKind, Stance};
use crate::noise::{self, Act};
use crate::skills::{Effect, Use};
use crate::stats::Stat;

impl World {
    /// War cry (module docs), for the time of a wait; its rage was spent by `use_class`.
    pub(super) fn war_cry(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(cry) = me.talent_mods(&content).war_cry else {
            return Spent::Nothing;
        };
        let price = self.price_of(player, ActionKind::Wait);
        let difficulty = cry.difficulty
            + me.effective_stats(&content).modifier(Stat::Strength)
            + me.skill_effect(&content, Effect::Intimidate);
        // Who hears it: the same noise the howl makes, before it is made.
        let burden = self.state.load(&content, me).penalty();
        let howl = noise::noise(&content, self.state.current(), me, burden, Act::Howl);
        self.say("You roar your fury!");
        let mut heard: Vec<_> = howl.map_or_else(Vec::new, |n| {
            (noise::hearers(&self.state, &content, n).into_iter())
                .map(|(id, _)| id)
                .collect()
        });
        heard.sort_unstable();
        self.make_noise(player, Act::Howl);
        if self.trains(Use::Intimidate) {
            self.train(Use::Intimidate);
        }
        for id in heard {
            if self.stance(player, id) != Stance::Hostile
                || !self.has_vitals(id)
                || !self.sees_now(id, player)
            {
                continue;
            }
            let Some(them) = self.state.actors.get(&id) else {
                continue;
            };
            let nerve = them.effective_stats(&content).modifier(Stat::Willpower)
                + them.talent_mods(&content).check(Stat::Willpower);
            let seen = self.perception.sees_actors_at(them.pos);
            let who = super::talk::capitalized(&self.called(them));
            let quails = !check::check(&mut self.state.rng, nerve, difficulty);
            if quails {
                self.afflict(id, cry.status, cry.severity, Some(player), events);
            }
            if seen {
                self.say(if quails {
                    format!("{who} quails and flees from you.")
                } else {
                    format!("{who} does not flinch.")
                });
            }
        }
        Spent::Turn(price)
    }
}
