//! Balance on bad ground and after a stagger (spec §8 "Skills": Balance).
//!
//! - **Rough ground**: a step onto terrain that slows (`move_cost` over 100) but is not water,
//!   off the overworld (bush, reeds, rubble, ice), pays less of what it adds by
//!   [`Effect::Footing`] (read in `World::step_price`) and marks [`Use::RoughStep`].
//! - **Staggered**: the status's extra cost is eased by [`Effect::SteadyEase`] (read in
//!   `World::status_cost`, as Initiative's is for Surprised), and each action the player takes
//!   Staggered marks [`Use::Staggered`] (from `World::spend_actions`).

use super::World;
use crate::content::{LevelKind, TerrainDef, TerrainId};
use crate::skills::Use;

impl World {
    /// Whether a step onto `to` on the current level is onto rough ground: off the overworld,
    /// onto terrain that slows and is not wading ([`World::wading_step`]).
    pub(super) fn rough_step(&self, to: &TerrainDef) -> bool {
        self.state.current().kind != LevelKind::Overworld
            && to.move_cost > 100
            && !self.wading_step(to)
    }

    /// The player stepped onto `onto`: rough ground may mark Balance.
    pub(super) fn train_rough_step(&mut self, onto: TerrainId) {
        let terrain = self.content.terrain(onto);
        if self.rough_step(terrain) && self.trains(Use::RoughStep) {
            self.train(Use::RoughStep);
        }
    }

    /// The player took an action Staggered.
    pub(super) fn train_staggered(&mut self) {
        if self.trains(Use::Staggered) {
            self.train(Use::Staggered);
        }
    }
}
