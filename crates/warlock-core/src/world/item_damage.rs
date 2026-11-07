//! Item damage (spec §7 "Materials"; #1168): one damaged state, whatever does it. Acid and rust
//! from a creature's `corrodes` (`corrode.rs`), heat from one whose kind is fire, and a failed
//! attempt at the anvil or with the knife (`temper.rs`) all go through [`World::harm_item`]:
//!
//! - **Withstood**: [`World::withstands`], the one check every source makes, first. The item's
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
use crate::content::ItemHarm;
use crate::magic::Worked;

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
}
