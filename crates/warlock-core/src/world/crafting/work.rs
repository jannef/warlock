//! The one driver of long work (#1492): Decipher, Temper and Whittle, a field forge, a recipe's
//! making, Cut, the enchanting recipes, salvage, Make do and mend, a rig, and Set each begin as
//! a [`Work`] (the [`Task`] and its seconds, Quick hands' share already taken where it applies,
//! [`World::craft_work`]), and [`World::work`] passes it for all of them:
//!
//! - **The loop**: turn after turn of the task's action ([`Job::kind`], the `craft` action but
//!   for Decipher, Salvage, and Whittle) through [`World::keep_at`], the last turn cut short to
//!   end exactly when the time is up ([`World::price_until`]); on the overworld a task may be
//!   watched ([`Job::watch`]: a recipe's making, as a forager).
//! - **Interrupted**: what stopped it is said with the task's words ([`Job::leave`]: "A kobold
//!   comes into view; you put down your work."); the time is spent and nothing else happens.
//! - **Done**: the stacks the task works on ([`Job::on`]) are found again ([`World::worked_on`],
//!   #1564) and handed to it ([`Job::found`]); any gone ends the work on nothing
//!   ([`Job::nothing_left`]). Then the task finishes ([`Job::done`]): its own checks, what comes
//!   of them, its marks.
//!
//! A new rule for every craft (an interruption, a station check) goes here once.

use crate::action::Event;
use crate::content::{ActionKind, Content};
use crate::time::Duration;
use crate::world::levels::encounter::Watch;
use crate::world::turn::rest::Ended;
use crate::world::{Spent, World};
use crate::world::{
    crafting::craft_make::Crafting, crafting::cut::GemWork, crafting::enchant::Enchanting,
    crafting::field_forge::Forging, crafting::make_do::Mending, crafting::rig::Rigging,
    crafting::salvage::Salvaging, crafting::set::Setting, crafting::temper::Tempering,
    magic::decipher::Deciphering,
};

/// What one kind of long work does around the driver's loop ([`World::work`]).
pub(in crate::world) trait Job {
    /// The action whose turns the work passes.
    fn kind(&self) -> ActionKind {
        ActionKind::Craft
    }

    /// What the player is watched as on the overworld while at it, if anything.
    fn watch(&self) -> Option<Watch> {
        None
    }

    /// The pack positions it works on, in pick order (`None` for a pick that is none).
    fn on(&self, _content: &Content) -> Vec<Option<usize>> {
        Vec::new()
    }

    /// Where the stacks of [`Job::on`] are now that the time is up, in the same order.
    fn found(&mut self, _now: &[Option<usize>]) {}

    /// What the player does as the work is cut short, in lower case: "you put down your work".
    fn leave(&self) -> String {
        "you put down your work".into()
    }

    /// What is said when a stack it works on is gone as the time is up.
    fn nothing_left(&self) -> &'static str {
        "You have nothing left to work on."
    }

    /// The time is up and the stacks are found: the work's own end.
    fn done(&self, w: &mut World, events: &mut Vec<Event>);
}

/// The long work under way, by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) enum Task {
    Craft(Crafting),
    Cut(GemWork),
    Decipher(Deciphering),
    Enchant(Enchanting),
    FieldForge(Forging),
    MakeDo(Mending),
    Rig(Rigging),
    Salvage(Salvaging),
    Set(Setting),
    Temper(Tempering),
}

impl Task {
    fn job(&mut self) -> &mut dyn Job {
        match self {
            Task::Craft(j) => j,
            Task::Cut(j) => j,
            Task::Decipher(j) => j,
            Task::Enchant(j) => j,
            Task::FieldForge(j) => j,
            Task::MakeDo(j) => j,
            Task::Rig(j) => j,
            Task::Salvage(j) => j,
            Task::Set(j) => j,
            Task::Temper(j) => j,
        }
    }
}

/// Long work under way: the task and how long it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::world) struct Work {
    task: Task,
    seconds: u32,
}

impl Work {
    /// `task` for `seconds`, as they are (no Quick hands: [`World::craft_work`] takes it).
    pub(in crate::world) fn spent(task: Task, seconds: u32) -> Spent {
        Spent::Work(Work { task, seconds })
    }
}

impl World {
    /// `task` for `base` seconds as the player's Quick hands makes them
    /// ([`World::craft_seconds`], the one time hook).
    pub(in crate::world) fn craft_work(&self, task: Task, base: u32) -> Spent {
        Work::spent(task, self.craft_seconds(base))
    }

    /// Works turn after turn until the work is done or something stops it (module docs).
    pub(in crate::world) fn work(&mut self, work: Work, events: &mut Vec<Event>) {
        let mut task = work.task;
        let job = task.job();
        let on = self.work_on(&job.on(&self.content));
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, job.kind());
        let watch = job.watch();
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| {
                let price = turn.min(w.price_until(until));
                if let Some(watch) = watch {
                    w.watch_roll(watch, price);
                }
                price
            },
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => match self.worked_on(&on) {
                Some(now) => {
                    job.found(&now);
                    job.done(self, events);
                }
                None => self.say(job.nothing_left()),
            },
            Ended::Came(came) => {
                let text = format!("{}; {}.", self.came_line(came), job.leave());
                self.say(text);
            }
            Ended::Said | Ended::Done => {
                let leave = job.leave();
                let mut chars = leave.chars();
                let first = chars.next().map(|c| c.to_uppercase().to_string());
                self.say(format!("{}{}.", first.unwrap_or_default(), chars.as_str()));
            }
        }
    }
}

/// The one stack `now` names, for a job of one stack: found, so it is there.
pub(in crate::world) fn one(now: &[Option<usize>]) -> usize {
    now.first().copied().flatten().expect("the stack worked on")
}
