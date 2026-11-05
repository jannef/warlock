//! Freshness (spec §8 "Hunger"; #1031): a player trained in Appraise or Survival may tell, in
//! words, how fresh a corpse or food is: the eat list and look mode say `fresh`, `old`, or
//! `rotten` (the names of [`Stage`], which food otherwise never shows).
//!
//! [`judged`] is the one rule, a pure function of the run's seed, the player, the food, and the
//! hour: the chance in percent is what the player's skills give of [`Effect::Freshness`], summed
//! (Appraise's and Survival's), and only a skill with the effect, trained, gives it at all; food
//! that has rotted is plainer to tell, so it adds [`ROTTEN_EASE`] points; at most
//! [`MOST_CHANCE`] and at least 0. One draw from the stream `freshness/<object>/<made>/<hour>`
//! settles it, so a food looked at again in the same hour is told the same way (the screens
//! draw again at every frame), and an hour later it is tried afresh: no state is kept and
//! no gameplay stream moves, which is why looking can be a read-only view method. Eating food
//! the check told trains Appraise ([`Use::Judged`], in [`World::train_judged`]).

use super::{Actor, World};
use crate::content::Content;
use crate::object::Stack;
use crate::rng::Rng;
use crate::rot::{self, Stage};
use crate::skills::{Effect, Use};
use crate::time::{GameTime, HOUR};

/// Points added to the chance for food that has rotted: it shows.
pub const ROTTEN_EASE: i32 = 25;

/// The most the chance is, in percent: a trained eye can still miss.
pub const MOST_CHANCE: i32 = 95;

/// The stage of `stack` at `now` if `actor` can tell it by looking (see the module docs); `None`
/// for what is no food, an eye untrained, and a check failed. `Stage::Gone` is told as rotten.
pub(crate) fn judged(
    content: &Content,
    actor: &Actor,
    seed: u64,
    stack: &Stack,
    now: GameTime,
) -> Option<Stage> {
    let stage = rot::stage(content, stack, now)?;
    if !actor.skills.trains_effect(content, Effect::Freshness) {
        return None;
    }
    let ease = if stage.is_rotten() { ROTTEN_EASE } else { 0 };
    let chance = (actor.skill_effect(content, Effect::Freshness) + ease).clamp(0, MOST_CHANCE);
    let label = format!(
        "freshness/{}/{}/{}",
        stack.object.0,
        stack.died?.seconds(),
        now.seconds() / HOUR
    );
    let told = Rng::stream(seed, &label).chance(chance as u32);
    told.then_some(match stage {
        Stage::Gone => Stage::Rotten,
        other => other,
    })
}

impl World {
    /// The player ate `food`: if their eye could tell its freshness, that trains them.
    pub(super) fn train_judged(&mut self, food: &Stack, at: GameTime) {
        let player = self.state.player();
        if judged(&self.content, player, self.state.seed, food, at).is_some() {
            self.train(Use::Judged);
        }
    }
}
