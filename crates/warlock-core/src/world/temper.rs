//! Tempering at a forge's anvil (spec §7 "Special rooms"; the design's §9.12): [`Action::Temper`],
//! the Temper ability of the abilities menu's *Here* group, offered beside an anvil.
//!
//! - **Where**: beside a cell whose terrain a room effect's `feature` names and that `offers`
//!   temper ([`crate::content::Content::offer_at`]); the first such cell around the player in
//!   [`Dir::ALL`] order. Refused, taking no time, anywhere else ("There is no anvil here."), and
//!   for a stack that is not a weapon or armour ([`temperable`]).
//! - **Time**: the table's `seconds` (ten minutes), in turns of the `temper` action
//!   ([`World::keep_at`], the rest's machinery): a creature coming into view, a message, or a
//!   blow stops it, and an interrupted tempering does nothing to the item, but the time is
//!   spent.
//! - **The check**: done, one Strength check is rolled (two gameplay draws, the natural roll):
//!   its margin (the natural roll plus the Strength modifier, what Metalcraft gives
//!   ([`Effect::Temper`]), and the talents' Strength checks, less [`DIFFICULTY`]) times
//!   [`POINTS_PER_MARGIN`], held to the table's `lean`, moves that many points from the bane
//!   (flawed, ruined) to the boon (improved), or the other way when below.
//! - **The outcome**: one drawn from the table so leaned, and its hooks (`outcome.rs`; the
//!   `work` hook changes the item, `magic/temper.rs`). A stack of several is worked one at a time:
//!   the one worked is set apart at the end of the pack, and joins the rest again if it is still
//!   like them. Then the item, if it survived, is identified ("This is ...!" when that says
//!   anything new, or when the work changed it), and Metalcraft may earn a mark ([`Use::Temper`]).
//!   The forge's table has `exhaust = 0`: the anvil is never spent.
//!
//! Every draw is from the gameplay stream.
//!
//! [`Action::Temper`]: crate::action::Action::Temper

use super::loot::Identified;
use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, ItemKind, Offer, Work};
use crate::geom::{Dir, Pos};
use crate::object::{Stack, add_to_inventory};
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// What the Strength check must reach for no lean.
pub const DIFFICULTY: i32 = 13;
/// Lean points for each point of the check's margin, held to the table's `lean`.
pub const POINTS_PER_MARGIN: i32 = 2;

/// Whether `stack` may be tempered: a weapon, a missile weapon, armour, or a shield (not
/// ammunition, rings, or amulets).
pub fn temperable(content: &Content, stack: &Stack) -> bool {
    content.object(stack.object).kind.is_some_and(|k| {
        matches!(k, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Shield) || k.is_armour()
    })
}

/// The lean a check of margin `margin` gives, held to `most` either way.
pub fn lean_for(margin: i32, most: u32) -> i32 {
    let most = i32::try_from(most).unwrap_or(0);
    (margin * POINTS_PER_MARGIN).clamp(-most, most)
}

/// A tempering under way: the anvil's cell, the stack worked, and how long it takes.
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
                .and_then(|t| self.content.offer_at(t))
                .is_some_and(|(offer, _)| offer == Offer::Temper)
        })
    }

    /// The player begins to temper the stack at `item` at the anvil beside them; refused,
    /// taking no time, with none or with a stack that cannot be tempered.
    pub(super) fn start_temper(&mut self, item: usize) -> Spent {
        let Some(anvil) = self.anvil_beside() else {
            self.say("There is no anvil here.");
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if !temperable(&self.content, stack) {
            self.say("Only weapons and armour can be tempered.");
            return Spent::Nothing;
        }
        let terrain = self.state.current().terrain_at(anvil);
        let Some((_, table)) = terrain.and_then(|t| self.content.offer_at(t)) else {
            return Spent::Nothing;
        };
        let seconds = table.seconds;
        let name = self.lore().title(stack);
        self.say(format!("You set to work on the {name} at the anvil."));
        Spent::Temper(Tempering {
            anvil,
            item,
            seconds,
        })
    }

    /// Works turn after turn until the tempering is done or something stops it (module docs):
    /// its last turn cut short to end exactly when its time is up.
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
            _ if self.state.time >= until => self.tempered(work, events),
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

    /// The tempering of `work` is done: the check, the outcome, the item identified, the mark
    /// (module docs).
    fn tempered(&mut self, work: Tempering, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let terrain = self.state.current().terrain_at(work.anvil);
        let Some((_, table)) = terrain.and_then(|t| content.offer_at(t)) else {
            return;
        };
        let pack = &self.state.player().inventory;
        if !pack.get(work.item).is_some_and(|s| temperable(&content, s)) {
            self.say("You have nothing left to work on.");
            return;
        }
        let me = self.state.player();
        let modifier = me.effective_stats(&content).modifier(Stat::Strength)
            + me.skill_effect(&content, Effect::Temper)
            + me.talent_mods(&content).check(Stat::Strength);
        let natural = combat::roll_natural(&mut self.state.rng);
        let lean = lean_for(natural + modifier - DIFFICULTY, table.lean);
        let kinds = table.leaned(lean);
        let (worked, apart) = self.set_apart(work.item);
        if let Some(outcome) = self.draw_outcome(table, kinds) {
            let ruins = outcome.effect.work == Some(Work::Ruin);
            let before = self.state.player().inventory.get(worked).cloned();
            self.befall_on(outcome, work.anvil, Some(worked), events);
            if !ruins {
                let changed = self.state.player().inventory.get(worked).cloned() != before;
                if self.identify(worked) == Identified::Known && changed {
                    let stack = &self.state.player().inventory[worked];
                    let name = self.lore().describe(stack);
                    self.say(format!("This is {name}!"));
                }
                if apart {
                    self.rejoin(worked);
                }
            }
        }
        if self.trains(Use::Temper) {
            self.train(Use::Temper);
        }
    }

    /// The stack at `item`, one of it set apart at the end of the pack when there are several:
    /// the position of the one to work, and whether it was set apart.
    pub(super) fn set_apart(&mut self, item: usize) -> (usize, bool) {
        let pack = &mut self.player_mut().inventory;
        match pack.get_mut(item) {
            Some(stack) if stack.count > 1 => {
                stack.count -= 1;
                let one = stack.single();
                pack.push(one);
                (pack.len() - 1, true)
            }
            _ => (item, false),
        }
    }

    /// The stack at `at`, set apart by [`World::set_apart`] at the end of the pack, joins one
    /// like it again (or stays last).
    pub(super) fn rejoin(&mut self, at: usize) {
        let content = self.content.clone();
        let pack = &mut self.player_mut().inventory;
        if at + 1 == pack.len() {
            let one = pack.remove(at);
            add_to_inventory(pack, one, &content);
        }
    }
}
