//! Item damage (spec §7 "Materials"; #1168): one damaged state, whatever does it. Acid and rust
//! from a creature's `corrodes` (`corrode.rs`), heat from one whose kind is fire, and a failed
//! attempt at the anvil or with the knife (`temper.rs`) all go through [`World::harm_item`]:
//!
//! - **Protected**: [`World::item_protected`] (#1170), read by every source of item damage,
//!   floor ruin (`ruin.rs`) too, first: a unique item, an item of the `proof` affix ("of
//!   Proof"), and every object a quest asks to be brought or delivered
//!   ([`World::quest_objects`]) are never harmed, by anything. A quest's reward is not
//!   protected unless another quest asks for it.
//! - **Withstood**: [`World::withstands`], the one check every source makes, first: protected,
//!   it withstands everything, drawing nothing. Else the item's
//!   main part's material withstands the [`ItemHarm`] by its `resists` chance in percent
//!   (`data/materials/`): 0 never, 100 always, neither drawing; else one gameplay draw. A failed
//!   attempt ([`ItemHarm::Work`]) is never withstood by the material. Withstood, nothing changes.
//! - **Damaged**: else an undamaged item is damaged, the smith's state ([`Worked`]: its
//!   improvements gone and a point worse, its word the harm's, "rusted"), and a successful
//!   Temper or Whittle repairs it, whatever damaged it.
//! - **Destroyed**: an item damaged already is destroyed, as a second failed temper destroys it.
//!
//! What is said is the source's.

use super::World;
use super::outcome::remove_stacks;
use crate::content::{Content, Effect, ItemHarm, ObjectId};
use crate::magic::{Mods, Worked};
use crate::object::Stack;
use std::collections::BTreeSet;

/// What came of harm reaching an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Harmed {
    /// It withstood it: nothing changed.
    Withstood,
    /// It is damaged now.
    Damaged,
    /// It was damaged already and is gone.
    Destroyed,
}

impl World {
    /// Whether the player's item at pack position `at` withstands `harm` (module docs): the
    /// one check every source of item damage makes.
    pub(super) fn withstands(&mut self, at: usize, harm: ItemHarm) -> bool {
        let content = &self.content;
        let stack = &self.state.player().inventory[at];
        if self.item_protected(stack) {
            return true;
        }
        let main = stack.parts(content).first().map(|p| p.material);
        let chance = main.map_or(0, |m| content.material(m).resists().of(harm));
        match chance {
            0 => false,
            100.. => true,
            chance => self.state.rng.chance(u32::from(chance)),
        }
    }

    /// `harm` reaches the player's item at pack position `at`, one item: withstood, damaged, or
    /// destroyed (module docs). Its state so far is the stack's (a craft's attempt counted).
    pub(super) fn harm_item(&mut self, at: usize, harm: ItemHarm) -> Harmed {
        if self.withstands(at, harm) {
            return Harmed::Withstood;
        }
        let stack = &mut self.player_mut().inventory[at];
        let mut worked = stack.worked().cloned().unwrap_or_else(Worked::default);
        if worked.damaged {
            remove_stacks(self.player_mut(), &[at]);
            self.keep_hp_within_max(self.state.player);
            return Harmed::Destroyed;
        }
        worked.damage(harm);
        stack.set_worked(worked);
        Harmed::Damaged
    }

    /// Whether `stack` is protected from all item damage (module docs): what every source,
    /// floor ruin too, reads.
    pub(super) fn item_protected(&self, stack: &Stack) -> bool {
        protected(&self.content, &self.quest_objects(), stack)
    }

    /// The objects protected for the quests' sake: what a quest asks to be brought or
    /// delivered, a fixed quest's until it is done for good (before it is given too: the
    /// chalice lies in its barrow first), a rolled quest's drawn object while it is active. A
    /// quest's reward is among them only when another quest asks for it.
    pub(super) fn quest_objects(&self) -> BTreeSet<ObjectId> {
        quest_objects(&self.content, &self.state.quests)
    }
}

/// [`World::quest_objects`] of `quests` (the item description reads it too: Eye for the make).
fn quest_objects(content: &Content, quests: &super::Quests) -> BTreeSet<ObjectId> {
    let done = |id: &str| quests.record(id).is_some_and(|r| r.completed > 0);
    (content.quests().iter())
        .filter_map(|def| match def.is_rolled() {
            false if done(&def.id) && !def.repeatable => None,
            false => def.hand_in(),
            true if quests.is_active(def) => quests.given(content, def).hand_in(),
            true => None,
        })
        .map(|(object, _)| object)
        .collect()
}

/// Whether the player knows `stack` to be protected from item damage while the quests stand as
/// `quests` (the item description's Eye for the make, which reads perception, not the world):
/// [`protected`] once its magic is identified; before, only a quest's object, so an unknown
/// unique or Proof is not given away.
pub(crate) fn protected_in(content: &Content, quests: &super::Quests, stack: &Stack) -> bool {
    let quest = quest_objects(content, quests);
    match stack.is_identified() {
        true => protected(content, &quest, stack),
        false => quest.contains(&stack.object),
    }
}

/// Whether `stack` is protected from item damage, `quest` the quests' objects
/// ([`World::quest_objects`], gathered once for many stacks): a unique, of Proof, or a quest's.
pub(super) fn protected(content: &Content, quest: &BTreeSet<ObjectId>, stack: &Stack) -> bool {
    stack.magic().is_some_and(|m| m.unique.is_some())
        || Mods::local(stack, content).get(Effect::Proof) > 0
        || quest.contains(&stack.object)
}
