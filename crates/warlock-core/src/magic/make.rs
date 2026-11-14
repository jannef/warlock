//! An item's make (#1436, #1494): Fine, Marvellous, or Masterwork as found or as a creature's
//! gear spawns (`data/gear_condition.toml`). Loot's make ([`make`]) is a step of making a find
//! ([`super::loot`]); creature gear's condition (`world/gear_condition.rs`) improves its pieces
//! the same way ([`improve`]).
//!
//! **Loot's make** ([`make`]): loot's arms and armour (floor loot, chests, hoards, dens, room
//! loot: every find [`super::loot`] makes with `Source::Loot`) are as made but for the
//! `[[depth]]` row of their item level: Fine, Marvellous, or Masterwork by its chances, never
//! damaged. One draw below [`PER_MILLE`] from the loot stream, after the wand's rung and before
//! the trim, then one draw per improvement for its [`Aspect`](super::Aspect) among those its base
//! item has, as the anvil draws them; none where the row gives nothing (the shallow levels,
//! whose draws are as before) or for what takes no make ([`improvable`]).

use super::Worked;
use super::worked::{arms, aspects};
use crate::content::{Content, PER_MILLE};
use crate::object::Stack;
use crate::rng::Rng;

/// Loot's make (module docs): `stack`, a find at item level `level`, made Fine, Marvellous, or
/// Masterwork by the `[[depth]]` row there, from `rng`; no draw where the row gives nothing or
/// `stack` takes no make.
pub(crate) fn make(content: &Content, rng: &mut Rng, level: u8, stack: &mut Stack) {
    let odds = content.gear_condition().depth_at(level).odds();
    if odds.improved() == 0 || !improvable(content, stack) {
        return;
    }
    if let Some(improved) = odds.of(rng.below(PER_MILLE)) {
        improve(content, rng, improved, stack);
    }
}

/// `stack` given `improved` improvements (at most `rules/temper.toml`'s `most`), each one attempt,
/// in aspects drawn as the anvil draws them; nothing for none.
pub(crate) fn improve(content: &Content, rng: &mut Rng, improved: u8, stack: &mut Stack) {
    let among = aspects(content.object(stack.object));
    let most = improved.min(content.temper().most);
    if most == 0 {
        return;
    }
    let mut worked = Worked::default();
    for _ in 0..most {
        worked.attempted();
        worked.improve(among[rng.index(among.len())]);
    }
    stack.set_worked(worked);
}

/// Whether `stack` may be made finer: a single piece of arms or armour with something to better,
/// of this world, and worked by nobody yet.
pub(crate) fn improvable(content: &Content, stack: &Stack) -> bool {
    stack.count == 1
        && !stack.is_spirit()
        && stack.worked().is_none()
        && arms(content, stack)
        && !aspects(content.object(stack.object)).is_empty()
}
