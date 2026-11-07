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
//!   those the base item has, or a damaged item repaired, whatever damaged it (acid, rust, heat,
//!   or a failure); failed, item damage of the kind [`ItemHarm::Work`] (`item_damage.rs`): the
//!   item damaged, or a damaged one destroyed. No material withstands a failure: it is the
//!   craft's own blunder, not the world's. A stack of several is worked one at a time: the one
//!   worked is set apart at the end of the pack, and being worked it joins nothing again. Then the item, if it survived,
//!   is identified, and says what it is now ("This is ...!"). Metalcraft may earn a mark
//!   ([`Use::Temper`]), whatever came of it. The anvil is never spent.
//!
//! **Whittle** (`docs/design/skill-uses.md` W3; [`Action::Whittle`]) is the same work on wood, with
//! a knife, anywhere: [`Craft::Whittle`] against [`Craft::Temper`], on the same rules, numbers
//! (`data/temper.toml`), and item state, so one item has one state. The item's main part is of a
//! wood (`group = "wood"`: bows, staves, clubs); the check is Dexterity's, with Woodcraft
//! ([`Effect::Whittle`]); the mark is [`Use::Whittle`]; a knife must be carried at the start and
//! at the end. Its list shows what [`workable_wood`] accepts.
//!
//! [`Action::Temper`]: crate::action::Action::Temper
//! [`Action::Whittle`]: crate::action::Action::Whittle
//! [`TemperRules::against`]: crate::content::TemperRules::against
//! [`Aspect`]: crate::magic::Aspect

use super::item_damage::Harmed;
use super::loot::Identified;
use super::rest::Ended;
use super::{Actor, Spent, World};
use crate::ability::Unavailable;
use crate::action::Event;
use crate::combat;
use crate::content::{ActionKind, Content, ItemHarm, ItemKind, Offer};
use crate::geom::{Dir, Pos};
use crate::magic::Worked;
use crate::magic::worked::aspects;
use crate::object::Stack;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// What an attempt is made with: a hammer at an anvil on metal, or a knife on wood.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Craft {
    /// At an anvil: metal, by Strength and Metalcraft.
    Temper,
    /// With a knife, anywhere: wood, by Dexterity and Woodcraft.
    Whittle,
}

impl Craft {
    /// The material group it works.
    const fn group(self) -> &'static str {
        match self {
            Craft::Temper => "metal",
            Craft::Whittle => "wood",
        }
    }

    /// The stat its check is made by.
    const fn stat(self) -> Stat {
        match self {
            Craft::Temper => Stat::Strength,
            Craft::Whittle => Stat::Dexterity,
        }
    }

    /// What the skill adds to the check.
    const fn effect(self) -> Effect {
        match self {
            Craft::Temper => Effect::Temper,
            Craft::Whittle => Effect::Whittle,
        }
    }

    /// What earns a mark.
    const fn used(self) -> Use {
        match self {
            Craft::Temper => Use::Temper,
            Craft::Whittle => Use::Whittle,
        }
    }

    /// What its turns are priced as.
    const fn kind(self) -> ActionKind {
        match self {
            Craft::Temper => ActionKind::Temper,
            Craft::Whittle => ActionKind::Whittle,
        }
    }

    /// The tool, as the messages call it.
    const fn tool(self) -> &'static str {
        match self {
            Craft::Temper => "hammer",
            Craft::Whittle => "knife",
        }
    }

    /// What the item does when the work goes well, badly, mends it, and ends it.
    const fn lines(self) -> [&'static str; 4] {
        match self {
            Craft::Temper => [
                "The metal rings true.",
                "The metal cracks under the hammer.",
                "You hammer the damage out.",
                "It shatters under the hammer!",
            ],
            Craft::Whittle => [
                "The wood takes the knife well.",
                "The knife slips and splits the wood.",
                "You pare the damage away.",
                "It splinters under the knife!",
            ],
        }
    }
}

/// What is said of a failed attempt the item withstands (`item_damage.rs`; no material
/// withstands a failure, so only a protection could).
const WITHSTOOD: &str = "The work goes wrong, but no harm is done.";

/// Whether `stack` is of a kind an anvil works at all: a weapon, a missile weapon, armour, or a
/// shield (not ammunition, rings, or amulets).
pub(super) fn arms(content: &Content, stack: &Stack) -> bool {
    content.object(stack.object).kind.is_some_and(|k| {
        matches!(k, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Shield) || k.is_armour()
    })
}

/// Whether `stack` is of the craft's material, by its main part, and of this world: a spirit
/// weapon is not.
fn of_material(content: &Content, stack: &Stack, craft: Craft) -> bool {
    !stack.is_spirit()
        && (stack.parts(content).first())
            .is_some_and(|p| content.material(p.material).group.as_deref() == Some(craft.group()))
}

/// Whether `stack` may be worked by `craft` at all: arms or armour, of its material, with
/// something an improvement could better.
fn craftable(content: &Content, stack: &Stack, craft: Craft) -> bool {
    arms(content, stack)
        && of_material(content, stack, craft)
        && !aspects(content.object(stack.object)).is_empty()
}

/// Whether `stack` may be worked at an anvil at all: arms or armour, of metal, with something
/// an improvement could better.
pub fn temperable(content: &Content, stack: &Stack) -> bool {
    craftable(content, stack, Craft::Temper)
}

/// Whether an anvil accepts `stack` now: [`temperable`], and not at its last improvement.
pub fn workable(content: &Content, stack: &Stack) -> bool {
    temperable(content, stack) && !finished(content, stack)
}

/// Whether a knife accepts `stack` now: arms of wood with something to better, and not at its
/// last improvement.
pub fn workable_wood(content: &Content, stack: &Stack) -> bool {
    craftable(content, stack, Craft::Whittle) && !finished(content, stack)
}

/// What `actor` lacks to whittle: a knife carried.
pub(crate) fn whittle_lack(actor: &Actor, content: &Content) -> Option<Unavailable> {
    let knife = (actor.inventory.iter()).any(|stack| content.is_knife(stack.object));
    (!knife).then_some(Unavailable::NoKnife)
}

/// Whether `stack` has all the improvements it can take.
fn finished(content: &Content, stack: &Stack) -> bool {
    (stack.worked()).is_some_and(|w| w.finished(content.temper().most))
}

/// Whether a check of natural roll `natural` plus `modifier` reaches `against`: a natural 2
/// always fails, a natural 24 always succeeds.
pub(super) fn check_made(natural: i32, modifier: i32, against: i32) -> bool {
    crate::check::makes(natural, modifier, against)
}

/// An attempt under way: what it is made with, the stack worked, and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Tempering {
    craft: Craft,
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
        if self.anvil_beside().is_none() {
            self.say("There is no anvil here.");
            return Spent::Nothing;
        }
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
            craft: Craft::Temper,
            item,
            seconds: self.content.temper().seconds,
        })
    }

    /// The player begins to whittle the stack at `item`; refused, taking no time, without a knife
    /// carried, with a stack that is not a wooden weapon, or with one that can be no finer.
    pub(super) fn start_whittle(&mut self, item: usize) -> Spent {
        if whittle_lack(self.state.player(), &self.content).is_some() {
            self.say("You have no knife.");
            return Spent::Nothing;
        }
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if !craftable(&self.content, stack, Craft::Whittle) {
            self.say("Only a weapon of wood can be whittled.");
            return Spent::Nothing;
        }
        if finished(&self.content, stack) {
            self.say("It can be made no finer.");
            return Spent::Nothing;
        }
        let name = self.lore().title(stack);
        self.say(format!("You set to whittling the {name} with your knife."));
        Spent::Temper(Tempering {
            craft: Craft::Whittle,
            item,
            seconds: self.content.temper().seconds,
        })
    }

    /// Works turn after turn until the attempt is done or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up.
    pub(super) fn temper(&mut self, work: Tempering, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, work.craft.kind());
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.tempered(work),
            Ended::Came(came) => {
                let text = format!(
                    "{}; you put down the {}.",
                    self.came_line(came),
                    work.craft.tool()
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => {
                self.say(format!("You put down the {}.", work.craft.tool()));
            }
        }
    }

    /// The attempt of `work` is done: the check, what comes of it, the item identified, the mark
    /// (module docs).
    fn tempered(&mut self, work: Tempering) {
        let content = self.content.clone();
        let pack = &self.state.player().inventory;
        let craft = work.craft;
        let ok = |s: &Stack| craftable(&content, s, craft) && !finished(&content, s);
        let Some(stack) = pack.get(work.item).filter(|s| ok(s)) else {
            self.say("You have nothing left to work on.");
            return;
        };
        // The knife may have been taken while the time passed.
        if craft == Craft::Whittle && whittle_lack(self.state.player(), &content).is_some() {
            self.say("You have nothing left to work with.");
            return;
        }
        let mut worked = stack.worked().cloned().unwrap_or_default();
        let me = self.state.player();
        let modifier = me.effective_stats(&content).modifier(craft.stat())
            + me.skill_effect(&content, craft.effect())
            + me.talent_mods(&content).check(craft.stat());
        let natural = combat::roll_natural(&mut self.state.rng);
        let against = content.temper().against(worked.attempts);
        let made = check_made(natural, modifier, against);
        worked.attempted();
        let at = self.set_apart(work.item);
        if self.work_out(craft, at, worked, made) {
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
        if self.trains(craft.used()) {
            self.train(craft.used());
        }
    }

    /// What came of an attempt `made` or failed by `craft` on the stack at `at`, one item, with
    /// `worked` its state so far (the attempt counted): whether it survived.
    fn work_out(&mut self, craft: Craft, at: usize, mut worked: Worked, made: bool) -> bool {
        let [good, bad, mended, ended] = craft.lines();
        if !made {
            // A failure is item damage like any other (`item_damage.rs`).
            self.player_mut().inventory[at].set_worked(worked);
            let (line, survived) = match self.harm_item(at, ItemHarm::Work) {
                Harmed::Withstood => (WITHSTOOD, true),
                Harmed::Damaged => (bad, true),
                Harmed::Destroyed => (ended, false),
            };
            self.say(line);
            return survived;
        }
        let line = if worked.damaged {
            worked.repair();
            mended
        } else {
            let object = self.state.player().inventory[at].object;
            let among = aspects(self.content.object(object));
            let i = self.state.rng.index(among.len());
            worked.improve(among[i]);
            good
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
