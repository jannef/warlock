//! Feinting (spec §8 "Skills": Bluff; #1016): Bluff's use, [`crate::Action::Feint`]
//! (`feint:DIR`), chosen from the abilities menu ([`crate::ability::Ability::Feint`]).
//!
//! The player, trained in a skill with the `feint` benefit (`data/skills/`: Bluff), lies with
//! their body to a hostile creature they see next to them. It takes the `feint` action's time and
//! may mark Bluff ([`Use::Feint`], the training stream), whatever comes of it.
//!
//! The foe checks its Willpower, two gameplay draws, as a threat's nerve check is made
//! (`war_cry.rs`, [`World::cow`]): its Willpower modifier and its talents' Willpower checks
//! against the file's `difficulty` + the player's Presence modifier + Bluff's points
//! ([`Effect::Feint`], read here and nowhere else). Failed, it suffers the file's status
//! (Surprised), so the player's next blow finds it flat-footed ([`World::flat_footed`], and the
//! sneak dice, `ambush.rs::sneaks`). Made, the turn is spent: "The kobold is not fooled."
//!
//! **Openings** (`docs/dev/openings.md`; #1462): a feint is a chosen attack, so the mark rule
//! ([`World::choose_mark`]) runs after the refusals and before the check: a feint at a new foe
//! empties the pool first, and a feint that fails still moves the mark. Fooled, the foe gives
//! the feinter's pools [`Build::Feint`] (Openings: one).
//!
//! **The next blow**: Surprised eases with its sufferer's next action
//! ([`World::spend_actions`]), and a foe of the same speed acts between the feint and the blow.
//! So the feint's status goes on [`World::fresh`], as one given during the sufferer's own action
//! does: the first action it takes in this turn does not ease it. It costs that action dear all
//! the same, and the one after, which ends it. A foe quick enough to act twice before the player
//! does has recovered by the blow.
//!
//! Refused, taking no time and no draw: untrained ("You do not know how to feint."), with no one
//! seen there, at one not hostile to them, or at one without a mind (`Content::has_vitals`: the
//! bones, the spirits, the animated). Nothing limits trying again (the owner's rule): the time
//! is the price.

use super::{Spent, World};
use crate::action::Event;
use crate::check;
use crate::content::{ActionKind, Build, Stance};
use crate::geom::Dir;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::text::capitalized;

impl World {
    /// [`crate::Action::Feint`]: the player feints at the creature in `dir` (see the module docs).
    pub(super) fn feint(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let trained = crate::ability::Ability::Feint.had_by(self.state.player(), &content);
        let Some(rules) = content.feint().copied().filter(|_| trained) else {
            self.say("You do not know how to feint.");
            return Spent::Nothing;
        };
        let at = self.player_pos().step(dir);
        let Some(foe) = self.seen_creature_at(at) else {
            self.say("You see no one there to feint at.");
            return Spent::Nothing;
        };
        let called = self.called(&self.state.actors[&foe]);
        let name = capitalized(&called);
        if self.stance(player, foe) != Stance::Hostile {
            self.say(format!("{name} is no foe of yours."));
            return Spent::Nothing;
        }
        if !self.has_vitals(foe) {
            self.say(format!("{name} has no mind to fool."));
            return Spent::Nothing;
        }
        // A chosen attack: the mark rule before the check (`mark.rs`).
        self.choose_mark(foe);
        let price = self.price_of(player, ActionKind::Feint);
        let me = self.state.player();
        let difficulty = rules.difficulty
            + me.effective_stats(&content).modifier(Stat::Presence)
            + me.skill_effect(&content, Effect::Feint);
        let them = &self.state.actors[&foe];
        let nerve = them.check_modifier(&content, Stat::Willpower);
        if check::check(&mut self.state.rng, nerve, difficulty) {
            self.say(format!("{name} is not fooled."));
        } else {
            self.afflict(foe, rules.status, 1, Some(player), events);
            // Its next action does not ease it (the module docs).
            self.fresh.push((foe, rules.status));
            self.say(format!("{name} falls for it."));
            self.gain_resource(player, Build::Feint);
        }
        if self.trains(Use::Feint) {
            self.train(Use::Feint);
        }
        Spent::Turn(price)
    }
}
