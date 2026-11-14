//! Taming a beast (spec §8 "Allies": Tame; docs/design/ranger-companion.md §5.3;
//! `docs/dev/companion.md`): Handle animal's use, [`crate::Action::Tame`] (`tame:DIR`), chosen
//! from the abilities menu ([`crate::ability::Ability::Tame`]) and aimed in a direction.
//!
//! - **Who**: anyone trained in a skill with the `tame` benefit (Handle animal,
//!   `data/skills/`), at any level, while `data/companions.toml` and `data/allies.toml`
//!   exist; else "You do not know how to tame beasts.". The player has **no companion** (one at
//!   a time, and for good: "You already have a companion."; a summon is no companion and keeps
//!   no one from taming).
//! - **The beast**: a creature the player sees on the cell that way, of a `tameable` type, no
//!   one's ally, and **not hostile** to the player ([`World::stance`]: wildlife, or an animal a
//!   Beast-friend calmed) **or unaware of them** ([`World::unaware`]: asleep, or in the dark to
//!   its eyes). No level gate: a level 1 commoner may try a bear; the check is all that stands in
//!   the way. Refused otherwise, taking no time and no draw.
//! - **The channel**: `[tame] turns` actions beside it, each a `tame` action's time, turn after
//!   turn as a dig is ([`World::keep_at`]): stopped by a creature coming into view, a message, the
//!   player being hurt, or the beast leaving their side (or dying, or becoming someone's), with
//!   nothing lost but the time. It makes no noise.
//! - **The check**, as the last of the turns: two gameplay draws, the player's Willpower modifier
//!   (and what their talents add to Willpower checks) + Handle animal's `tame` benefit + `[tame]
//!   food` if they gave food + [`crate::talent::TalentMods::tame`] (the ranger's Animal
//!   companion), against `[tame] difficulty` + `per_level` × the beast's level. Food: one item of
//!   the first stack carried that is food, eaten by the beast as the check is made (none
//!   carried, no bonus; an interrupted try eats none).
//! - **Made**: the beast is the player's companion where it stands ([`World::adopt`]: their
//!   side, its mind cleared, graded at their level, named); "The wolf comes to your hand. You
//!   call it Græg." **Failed**: "The wolf snaps at you and backs away."; a hostile one has seen
//!   the player now ([`super::Mind::last_seen`]), so it may not be tried again until it loses
//!   them, and one not hostile may be tried again at once (no cooldown: time is the price). A
//!   sleeper wakes either way.
//! - **Marks**: Handle animal ([`Use::Handle`], the training stream), made or failed.

use super::rest::Ended;
use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::check;
use crate::content::{ActionKind, Stance};
use crate::geom::Dir;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::text::capitalized;
use std::cell::Cell;

/// A try at taming under way ([`super::Spent::Tame`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Taming {
    /// The beast.
    beast: ActorId,
}

impl World {
    /// [`crate::Action::Tame`]: the player begins to tame the beast toward `dir`; refused,
    /// taking no time, as the module docs say.
    pub(super) fn start_tame(&mut self, dir: Dir) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let trained = crate::ability::Ability::Tame.had_by(self.state.player(), &content);
        if !trained || content.companions().is_none() || content.allies().is_none() {
            self.say("You do not know how to tame beasts.");
            return Spent::Nothing;
        }
        if self.companion().is_some() {
            self.say("You already have a companion.");
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        let Some(beast) = self.seen_creature_at(at) else {
            self.say("There is no beast there to tame.");
            return Spent::Nothing;
        };
        let actor = &self.state.actors[&beast];
        let called = self.called(actor);
        let name = capitalized(&called);
        if actor.master() == Some(player) {
            self.say(format!("{name} is yours already."));
            return Spent::Nothing;
        }
        if actor.ally.is_some() || !content.creature(actor.creature).tameable {
            self.say(format!("{name} cannot be tamed."));
            return Spent::Nothing;
        }
        if self.stance(player, beast) == Stance::Hostile && !self.unaware(beast, player) {
            self.say(format!("{name} has seen you: it will not be tamed now."));
            return Spent::Nothing;
        }
        let text = match self.first_food() {
            Some(i) => {
                let food = &self.state.player().inventory[i];
                let what = self.lore().describe_object(food.object, 1);
                format!("You crouch by {called} and hold out {what}.")
            }
            None => format!("You crouch by {called} and speak softly to it."),
        };
        self.say(text);
        Spent::Tame(Taming { beast })
    }

    /// Tames turn after turn until the turns are done or something stops it (the module docs).
    /// The check is the last turn's action, so that turn's creatures and perception know what
    /// came of it.
    pub(super) fn tame(&mut self, taming: Taming, events: &mut Vec<Event>) {
        let Some(rules) = self.content.companions().map(|r| r.tame) else {
            return;
        };
        let player = self.state.player;
        let left = Cell::new(rules.turns);
        let hp = Cell::new(self.state.player().hp);
        let hurt = Cell::new(false);
        let ended = self.keep_at(
            events,
            |w| {
                // Hurt since the last turn: it stops (regeneration does not hide a blow).
                let now = w.state.player().hp;
                if now < hp.get() {
                    hurt.set(true);
                }
                hp.set(now);
                left.get() > 0 && !hurt.get() && w.taming_holds(taming)
            },
            |w, _| {
                left.set(left.get() - 1);
                if left.get() == 0 {
                    w.tame_check(taming.beast);
                }
                w.price_of(player, ActionKind::Tame)
            },
        );
        if ended == Ended::Died || left.get() == 0 {
            return;
        }
        let called = (self.state.actors.get(&taming.beast)).map(|a| self.called(a));
        let called = called.unwrap_or_else(|| "the beast".into());
        let text = match ended {
            _ if !self.taming_holds(taming) => {
                format!(
                    "{} leaves your side; you stop taming it.",
                    capitalized(&called)
                )
            }
            Ended::Came(came) => format!("{}; you stop taming {called}.", self.came_line(came)),
            _ => format!("You stop taming {called}."),
        };
        self.say(text);
    }

    /// Whether the try at taming may go on: the beast is alive beside the player, on their
    /// level, and no one's ally.
    fn taming_holds(&self, taming: Taming) -> bool {
        let me = self.state.player();
        (self.state.actors.get(&taming.beast)).is_some_and(|b| {
            b.level == me.level
                && b.hp > 0
                && b.ally.is_none()
                && b.pos != me.pos
                && b.pos.chebyshev(me.pos) <= 1
        })
    }

    /// The first stack the player carries that is food (by position): what a beast is offered.
    fn first_food(&self) -> Option<usize> {
        (self.state.player().inventory.iter())
            .position(|s| self.content.object(s.object).food.is_some())
    }

    /// The taming check's modifier and difficulty for beast `beast`, `fed` or not (module
    /// docs): the player's Willpower modifier and what their talents add to Willpower checks,
    /// Handle animal's `tame` benefit, the food's bonus, and their talents' `tame` (the ranger's);
    /// against `[tame] difficulty` + `per_level` × the beast's level.
    pub(super) fn tame_odds(&self, beast: ActorId, fed: bool) -> (i32, i32) {
        let content = &self.content;
        let Some(rules) = content.companions().map(|r| r.tame) else {
            return (0, 0);
        };
        let me = self.state.player();
        let mods = me.talent_mods(content);
        let modifier = me.check_modifier(content, Stat::Willpower)
            + me.skill_effect(content, Effect::Tame)
            + if fed { rules.food } else { 0 }
            + mods.tame;
        let level =
            (self.state.actors.get(&beast)).map_or(0, |b| content.creature(b.creature).level);
        (
            modifier,
            rules.difficulty + rules.per_level * i32::from(level),
        )
    }

    /// The turns are done: the food, the check, the mark, and what comes of it (module docs).
    fn tame_check(&mut self, beast: ActorId) {
        let called = (self.state.actors.get(&beast)).map(|a| self.called(a));
        let called = called.unwrap_or_else(|| "the beast".into());
        let fed = self.first_food().is_some_and(|i| {
            let player = self.player_mut();
            if player.inventory[i].count > 1 {
                player.inventory[i].count -= 1;
            } else {
                player.inventory.remove(i);
                (player.equipment).after_removal(&crate::Positions::of([i]));
            }
            true
        });
        let (modifier, difficulty) = self.tame_odds(beast, fed);
        let made = check::check(&mut self.state.rng, modifier, difficulty);
        if self.trains(Use::Handle) {
            self.train(Use::Handle);
        }
        self.wake(beast);
        let name = capitalized(&called);
        if made && let Some(own) = self.adopt(beast) {
            self.say(format!("{name} comes to your hand. You call it {own}."));
        } else {
            // A hostile one has seen the player now: no second try while it hunts them.
            let at = self.player_pos();
            let hostile = self.stance(self.state.player, beast) == Stance::Hostile;
            if let Some(actor) = self.state.actors.get_mut(&beast).filter(|_| hostile) {
                actor.mind.last_seen = Some(at);
            }
            self.say(format!("{name} snaps at you and backs away."));
        }
    }
}
