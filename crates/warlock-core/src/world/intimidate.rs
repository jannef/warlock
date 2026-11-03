//! Intimidating (spec §8 "Skills": Intimidate; #868): the Intimidate skill's use,
//! [`crate::Action::Intimidate`] (`intimidate:X,Y`), chosen from the abilities menu
//! ([`crate::ability::Ability::Intimidate`]) and aimed at a place the player sees.
//!
//! War cry's check at one foe (`war_cry.rs`, whose [`World::threat_difficulty`],
//! [`World::heard_by`], and [`World::cow`] it shares): the player, trained in a skill with the
//! `intimidate` benefit (`data/skills.toml`: Intimidate), threatens the hostile creature they see
//! on a cell within the `range` of `data/intimidate.toml`. It is spoken aloud ([`Act::Mutter`]'s
//! voice, where the player stands), takes the `intimidate` action's time, and marks Intimidate
//! ([`Use::Intimidate`], the training stream), whatever comes of it.
//!
//! If the foe **hears** the voice (asked before it is made) and **sees** the player now
//! ([`World::sees_now`]), it checks its nerve, two gameplay draws: its Willpower modifier
//! against the file's `difficulty` + the player's Strength modifier + their Intimidate. Failed,
//! it suffers the file's status (Frightened) at the degrees its ladder gives for what the
//! player's Intimidate adds ([`crate::content::IntimidateRules::degrees_at`]: a little skill one
//! degree, War cry's three at the top); made, it stands. A foe that does not hear or see the
//! player pays no heed: the turn and the noise are spent, and no draw is made.
//!
//! Refused, taking no time and no draw: untrained ("You do not know how to intimidate."), at
//! their own cell, out of range, at no creature they see, at one not hostile to them, or at one
//! without vitals (`Content::has_vitals`: the bones, the spirits, the animated). Nothing limits
//! trying again (the owner's rule): the time is the price.

use super::talk::capitalized;
use super::{Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Stance};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::noise::Act;
use crate::skills::{Effect, Use};

impl World {
    /// [`crate::Action::Intimidate`]: the player threatens the creature on `at` (see the module
    /// docs).
    pub(super) fn intimidate(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let trained = crate::ability::Ability::Intimidate.had_by(self.state.player(), &content);
        let Some(rules) = content.intimidate().filter(|_| trained) else {
            self.say("You do not know how to intimidate.");
            return Spent::Nothing;
        };
        let here = self.player_pos();
        if at == here {
            self.say("You cannot intimidate yourself.");
            return Spent::Nothing;
        }
        if at.dist2(here) > radius2(rules.range) {
            self.say("That is out of range.");
            return Spent::Nothing;
        }
        let Some(foe) = self.seen_creature_at(at) else {
            self.say("You see no one there to intimidate.");
            return Spent::Nothing;
        };
        let called = self.called(&self.state.actors[&foe]);
        let name = capitalized(&called);
        if self.stance(player, foe) != Stance::Hostile {
            self.say(format!("{name} is no foe of yours."));
            return Spent::Nothing;
        }
        if !self.has_vitals(foe) {
            self.say(format!("{name} has no heart to quail."));
            return Spent::Nothing;
        }
        let price = self.price_of(player, ActionKind::Intimidate);
        self.say(format!("You threaten {called}."));
        let heard = self.heard_by(Act::Mutter).binary_search(&foe).is_ok();
        self.make_noise(player, Act::Mutter);
        if self.trains(Use::Intimidate) {
            self.train(Use::Intimidate);
        }
        if heard && self.sees_now(foe, player) {
            let points = self
                .state
                .player()
                .skill_effect(&content, Effect::Intimidate);
            let degrees = rules.degrees_at(points);
            let difficulty = self.threat_difficulty(rules.difficulty);
            self.cow(foe, difficulty, rules.status, degrees, events);
        } else {
            self.say(format!("{name} pays you no heed."));
        }
        Spent::Turn(price)
    }
}
