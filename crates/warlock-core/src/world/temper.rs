//! Working a metal item at an anvil (spec §7 "Special rooms"; `docs/design/skill-uses.md` §6.7
//! M0): [`Action::Temper`], the Temper ability of the abilities menu's *Here* group, offered
//! beside an anvil. One action does it all; the item's state and its pure rules are
//! `magic/worked.rs`, the numbers `data/temper.toml`.
//!
//! - **Where**: beside a cell whose terrain a room effect's `feature` names and that `offers`
//!   temper ([`crate::content::Content::offer`]); the first such cell around the player in
//!   [`Dir::ALL`] order. Refused, taking no time, anywhere else ("There is no anvil here.").
//! - **What**: a weapon, launcher, armour, or shield whose main part is of a metal, and not a
//!   spirit weapon ([`temperable`]). Refused otherwise, taking no time, and so is an item with
//!   all its improvements ([`workable`] is what the list offers).
//! - **Time**: `seconds` (ten minutes), in turns of the `temper` action ([`World::keep_at`], the
//!   rest's machinery): a creature coming into view, a message, or a blow stops it, and an
//!   interrupted attempt does nothing to the item and counts for nothing, but the time is spent.
//! - **The check**: done, one Strength check (two gameplay draws, the natural roll): the natural
//!   roll plus the Strength modifier, what Metalcraft gives ([`Effect::Temper`]), and the talents'
//!   Strength checks, against [`TemperRules::against`] the attempts made on the item so far. A
//!   natural 2 always fails, a natural 24 always succeeds. Every attempt counts, made or failed.
//! - **The outcome**: made, an improvement in an [`Aspect`] drawn evenly (one gameplay draw) among
//!   those the base item has, or a damaged item repaired; failed, the item damaged, or a damaged
//!   one destroyed. A stack of several is worked one at a time: the one worked is set apart at
//!   the end of the pack, and being worked it joins nothing again. Then the item, if it survived,
//!   is identified, and says what it is now ("This is ...!"). Metalcraft may earn a mark
//!   ([`Use::Temper`]), whatever came of it. The anvil is never spent.
//!
//! [`Action::Temper`]: crate::action::Action::Temper
//! [`TemperRules::against`]: crate::content::TemperRules::against
//! [`Aspect`]: crate::magic::Aspect

use super::loot::Identified;
use super::outcome::remove_stacks;
use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, ItemKind, Offer};
use crate::geom::{Dir, Pos};
use crate::magic::Worked;
use crate::magic::worked::aspects;
use crate::object::Stack;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// The material group an anvil works.
const METAL: &str = "metal";

/// Whether `stack` is of a kind an anvil works at all: a weapon, a missile weapon, armour, or a
/// shield (not ammunition, rings, or amulets).
fn arms(content: &Content, stack: &Stack) -> bool {
    content.object(stack.object).kind.is_some_and(|k| {
        matches!(k, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Shield) || k.is_armour()
    })
}

/// Whether `stack` is of metal, by its main part, and of this world: a spirit weapon is not.
fn metal(content: &Content, stack: &Stack) -> bool {
    !stack.is_spirit()
        && (stack.parts(content).first())
            .is_some_and(|p| content.material(p.material).group.as_deref() == Some(METAL))
}

/// Whether `stack` may be worked at an anvil at all: arms or armour, of metal, with something
/// an improvement could better.
pub fn temperable(content: &Content, stack: &Stack) -> bool {
    arms(content, stack)
        && metal(content, stack)
        && !aspects(content.object(stack.object)).is_empty()
}

/// Whether an anvil accepts `stack` now: [`temperable`], and not at its last improvement.
pub fn workable(content: &Content, stack: &Stack) -> bool {
    temperable(content, stack) && !finished(content, stack)
}

/// Whether `stack` has all the improvements it can take.
fn finished(content: &Content, stack: &Stack) -> bool {
    (stack.worked()).is_some_and(|w| w.finished(content.temper().most))
}

/// Whether a check of natural roll `natural` plus `modifier` reaches `against`: a natural 2
/// always fails, a natural 24 always succeeds.
pub(super) fn check_made(natural: i32, modifier: i32, against: i32) -> bool {
    natural == combat::SURE_HIT || (natural != combat::FUMBLE && natural + modifier >= against)
}

/// An attempt under way: the anvil's cell, the stack worked, and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Tempering {
    anvil: Pos,
    item: usize,
    seconds: u32,
}

impl World {
    /// The anvil beside the player: the first cell around them (in [`Dir::ALL`] order) whose
    /// terrain offers Temper.
    fn anvil_beside(&self) -> Option<Pos> {
        let level = self.state.current();
        let here = self.player_pos();
        (Dir::ALL.iter().map(|&d| here.step(d))).find(|&p| {
            (level.terrain_at(p))
                .and_then(|t| self.content.offer(t))
                .is_some_and(|offer| offer == Offer::Temper)
        })
    }

    /// The player begins to work the stack at `item` at the anvil beside them; refused, taking no
    /// time, with none, with a stack that cannot be worked, or with one that can be no finer.
    pub(super) fn start_temper(&mut self, item: usize) -> Spent {
        let Some(anvil) = self.anvil_beside() else {
            self.say("There is no anvil here.");
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if !arms(&self.content, stack) {
            self.say("Only weapons and armour can be tempered.");
            return Spent::Nothing;
        }
        if !temperable(&self.content, stack) {
            self.say("Only metal can be worked at an anvil.");
            return Spent::Nothing;
        }
        if finished(&self.content, stack) {
            self.say("It can be made no finer.");
            return Spent::Nothing;
        }
        let name = self.lore().title(stack);
        self.say(format!("You set to work on the {name} at the anvil."));
        Spent::Temper(Tempering {
            anvil,
            item,
            seconds: self.content.temper().seconds,
        })
    }

    /// Works turn after turn until the attempt is done or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up.
    pub(super) fn temper(&mut self, work: Tempering, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Temper);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.tempered(work),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you put down the hammer.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You put down the hammer."),
        }
    }

    /// The attempt of `work` is done: the check, what comes of it, the item identified, the mark
    /// (module docs).
    fn tempered(&mut self, work: Tempering) {
        let content = self.content.clone();
        let pack = &self.state.player().inventory;
        let Some(stack) = pack.get(work.item).filter(|s| workable(&content, s)) else {
            self.say("You have nothing left to work on.");
            return;
        };
        let mut worked = stack.worked().cloned().unwrap_or_default();
        let me = self.state.player();
        let modifier = me.effective_stats(&content).modifier(Stat::Strength)
            + me.skill_effect(&content, Effect::Temper)
            + me.talent_mods(&content).check(Stat::Strength);
        let natural = combat::roll_natural(&mut self.state.rng);
        let against = content.temper().against(worked.attempts);
        let made = check_made(natural, modifier, against);
        worked.attempted();
        let at = self.set_apart(work.item);
        if self.work_out(at, worked, made) {
            match self.identify(at) {
                // Learning what it is said its name; else its name is what is new.
                Identified::Now(_) | Identified::NoSuchItem => {}
                Identified::Known => {
                    let stack = &self.state.player().inventory[at];
                    let name = self.lore().describe(stack);
                    self.say(format!("This is {name}!"));
                }
            }
        }
        if self.trains(Use::Temper) {
            self.train(Use::Temper);
        }
    }

    /// What came of an attempt `made` or failed on the stack at `at`, one item, with `worked`
    /// its state so far (the attempt counted): whether it survived.
    fn work_out(&mut self, at: usize, mut worked: Worked, made: bool) -> bool {
        let line = match (worked.damaged, made) {
            (false, true) => {
                let object = self.state.player().inventory[at].object;
                let among = aspects(self.content.object(object));
                let i = self.state.rng.index(among.len());
                worked.improve(among[i]);
                "The metal rings true."
            }
            (false, false) => {
                worked.damage();
                "The metal cracks under the hammer."
            }
            (true, true) => {
                worked.repair();
                "You hammer the damage out."
            }
            (true, false) => {
                self.say("It shatters under the hammer!");
                remove_stacks(self.player_mut(), &[at]);
                self.keep_hp_within_max(self.state.player);
                return false;
            }
        };
        self.say(line);
        self.player_mut().inventory[at].set_worked(worked);
        true
    }

    /// The stack at `item`, one of it set apart at the end of the pack when there are several:
    /// the position of the one to work.
    fn set_apart(&mut self, item: usize) -> usize {
        let pack = &mut self.player_mut().inventory;
        match pack.get_mut(item) {
            Some(stack) if stack.count > 1 => {
                stack.count -= 1;
                let one = stack.single();
                pack.push(one);
                pack.len() - 1
            }
            _ => item,
        }
    }
}
