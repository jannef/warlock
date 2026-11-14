//! Make do and mend (spec §8 "Class abilities"; the craftsperson design §7.5; #1422): the
//! craftsperson's class active at level 18, `use:make_do_and_mend:c`
//! ([`crate::action::Action::UseOn`]), from the abilities menu's Class group through the mend
//! list ([`mendable`]). It mends a damaged thing the player carries, anywhere, with no anvil and
//! no tool; the numbers are the talent's effect (`make_do { seconds, difficulty }`).
//!
//! - **What**: a stack carried whose [`Worked`](crate::magic::Worked) state is damaged, whatever
//!   damaged it (acid, rust, heat, a failed attempt). Refused, taking no time: no such stack
//!   ("You carry no such thing."), one not damaged ("There is nothing wrong with it."), one worn
//!   or wielded ("Take it off first."), one whose main material no profession works ("You know
//!   no craft that would mend it."), one protected from harm ([`World::item_protected`]: "You
//!   dare not work on it."), and unpaid shop goods (`may_loose`).
//! - **Time**: `seconds` (ten minutes), in turns of the `craft` action ([`World::work`], as
//!   Salvage): interrupted, nothing happens to the item, but the time is spent.
//! - **The check**: done, the stack found again, one check (two gameplay draws): the natural
//!   roll plus the modifier of the stat of the profession that works its main material, the
//!   talents' checks of that stat, and that profession's [`Effect::Craft`]
//!   ([`super::Actor::skill_effect_of`]; untrained, the stat alone, as Salvage's), against
//!   `difficulty` + the material's `work`. A natural 2 always fails, a natural 24 always
//!   succeeds.
//! - **Made**: the damage is gone ([`Worked::repair`](crate::magic::Worked)); the improvements it
//!   lost stay lost, the attempts made on it are untouched (a Temper later is as hard as it was
//!   before), and what is known of it is unchanged. A stack of several mends one, set apart.
//! - **Failed**: nothing happens to it; it stays damaged and is never destroyed. Trying again
//!   costs only the time.
//! - **Marks**: [`Use::Craft`] on that profession alone, if trained, whatever came of it.

use super::work::{Job, Task};
use super::{Actor, Spent, World};
use crate::action::Event;
use crate::combat;
use crate::object::Stack;
use crate::skills::{Effect, Use};

/// Whether `stack` is damaged, so that a mend has something to do. Worn, protected, unpaid, and
/// unworkable stacks are the rules' to refuse.
pub fn mendable(stack: &Stack) -> bool {
    stack.worked().is_some_and(|w| w.damaged)
}

/// Whether `actor` carries anything damaged that it neither wears nor wields: what the ability's
/// row asks ([`crate::ability::Unavailable::NothingToMend`]).
pub(crate) fn mends_any(actor: &Actor) -> bool {
    (actor.inventory.iter().enumerate()).any(|(i, s)| mendable(s) && !actor.equipment.holds(i))
}

/// A mend under way: the stack mended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Mending {
    item: usize,
}

impl Job for Mending {
    fn on(&self, _: &crate::content::Content) -> Vec<Option<usize>> {
        vec![Some(self.item)]
    }

    fn found(&mut self, now: &[Option<usize>]) {
        self.item = super::work::one(now);
    }

    fn done(&self, w: &mut World, _: &mut Vec<Event>) {
        w.mended(self.item);
    }
}

impl World {
    /// The player begins to mend the stack at `item`; refused, taking no time (module docs).
    pub(super) fn start_make_do(&mut self, item: usize) -> Spent {
        let Some(rules) = self.state.player().talent_mods(&self.content).make_do else {
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if !mendable(stack) {
            self.say("There is nothing wrong with it.");
            return Spent::Nothing;
        }
        if self.state.player().equipment.holds(item) {
            self.say("Take it off first.");
            return Spent::Nothing;
        }
        if self.mend_by(stack).is_none() {
            self.say("You know no craft that would mend it.");
            return Spent::Nothing;
        }
        if self.item_protected(stack) {
            self.say("You dare not work on it.");
            return Spent::Nothing;
        }
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let stack = &self.state.player().inventory[item];
        let name = self.lore().title(stack);
        self.say(format!("You set to mending the {name}."));
        self.craft_work(Task::MakeDo(Mending { item }), rules.seconds)
    }

    /// The material of `stack`'s main part and the profession that works it, with its stat.
    fn mend_by(
        &self,
        stack: &Stack,
    ) -> Option<(
        crate::content::MaterialId,
        crate::skills::SkillId,
        crate::stats::Stat,
    )> {
        let main = stack.parts(&self.content).first()?.material;
        let (skill, stat) = self.profession_of(main)?;
        Some((main, skill, stat))
    }

    /// The mend of the stack at `item` is done: the check, the damage gone or not, the mark
    /// (module docs).
    fn mended(&mut self, item: usize) {
        let content = self.content.clone();
        let me = self.state.player();
        let Some(rules) = me.talent_mods(&content).make_do else {
            return;
        };
        let still = (me.inventory.get(item))
            .filter(|s| mendable(s) && !me.equipment.holds(item) && !self.item_protected(s));
        let Some((material, skill, stat)) = still.and_then(|s| self.mend_by(s)) else {
            self.say("You have nothing left to work on.");
            return;
        };
        let modifier = me.effective_stats(&content).modifier(stat)
            + me.talent_mods(&content).check(stat)
            + me.skill_effect_of(&content, skill, Effect::Craft);
        let against = rules.difficulty + content.material(material).work;
        let natural = combat::roll_natural(&mut self.state.rng);
        if crate::check::makes(natural, modifier, against) {
            let at = self.set_apart(item);
            let stack = &mut self.player_mut().inventory[at];
            let mut worked = stack.worked().cloned().unwrap_or_default();
            worked.repair();
            stack.set_worked(worked);
            let what = self
                .lore()
                .describe(&self.state.player().inventory[at].single());
            self.say(format!("You make good the damage: it is {what} again."));
        } else {
            self.say("You cannot make it good here; it will need an anvil.");
        }
        if self.trains(Use::Craft) {
            self.train_skill(Use::Craft, skill);
        }
    }
}
