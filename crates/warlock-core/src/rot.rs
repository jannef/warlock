//! Food rots (spec §8 "Hunger"): every food has a shelf life in data ([`Food::keeps`] and
//! [`Food::lasts`]: `food` on objects, corpses by their creature's size) and every stack of food
//! remembers when it was made ([`Stack::died`]; a corpse, when its creature died). Its stage
//! follows from its age alone ([`Stage`]): fresh, old past half of what it keeps, rotten past
//! that, and gone once it has lasted. Stacks of the same food keep the oldest age when they join
//! (`object.rs`), so a pack never splits by age.
//!
//! Nothing here is ever shown to the player: food does not show its stage (the rules and dev
//! dumps read it). Without `data/hunger.toml` nothing can be eaten and food does not rot.
//! Applying it is `world/rot.rs`.
//!
//! [`Food::keeps`]: crate::content::Food::keeps
//! [`Food::lasts`]: crate::content::Food::lasts

use crate::content::Content;
use crate::object::Stack;
use crate::time::GameTime;
use serde::Serialize;

/// How food is by its age.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Fresh,
    /// Past half of what it keeps: still fine to eat.
    Old,
    /// Past what it keeps: eating it is risky.
    Rotten,
    /// It has lasted: rotted away, it is gone.
    Gone,
}

impl Stage {
    /// Its id in dev dumps: `fresh`, `old`, `rotten`, `gone`.
    pub const fn id(self) -> &'static str {
        match self {
            Stage::Fresh => "fresh",
            Stage::Old => "old",
            Stage::Rotten => "rotten",
            Stage::Gone => "gone",
        }
    }

    /// Whether eating it has the rotten food's effects (rotten, or gone and not yet removed).
    pub const fn is_rotten(self) -> bool {
        matches!(self, Stage::Rotten | Stage::Gone)
    }
}

/// How `stack` is at `now`: `None` for what is no food, food that does not know its age, or
/// without `data/hunger.toml` (food does not rot then).
pub(crate) fn stage(content: &Content, stack: &Stack, now: GameTime) -> Option<Stage> {
    content.hunger()?;
    let made = stack.died?;
    let food = content.object(stack.object).food?;
    Some(food.stage(now.since(made)))
}

/// When `stack` has rotted away (`None` as for [`stage`]).
pub(crate) fn gone_at(content: &Content, stack: &Stack) -> Option<GameTime> {
    content.hunger()?;
    let made = stack.died?;
    Some(made + content.object(stack.object).food?.lasts)
}

/// Food among `stacks` that does not know its age yet is made at `now`: what a level, a spawned
/// creature, the player's starting pack, or a quest giver brings into the world. Other stacks
/// are left as they are.
pub(crate) fn made_at<'a>(
    stacks: impl IntoIterator<Item = &'a mut Stack>,
    content: &Content,
    now: GameTime,
) {
    for stack in stacks {
        if stack.died.is_none() && content.object(stack.object).food.is_some() {
            stack.died = Some(now);
        }
    }
}
