//! The lenient read of a saved item's affix tiers (docs/design/affixes-systems.md §8,
//! "Regrading"): a ladder regraded since a save was made may have moved the range a saved tier
//! holds, so every magic stack in the state is re-tiered ([`crate::magic::Magic::retier`])
//! before the state is checked. No save is refused for it; a value no tier holds is held to the
//! nearest tier's range (an old Potent, the spell scaling design §7.3); each move is a line for
//! the log.

use super::WorldState;
use crate::content::Content;

/// Re-tiers every magic stack of `state`: what actors carry, and on every level the piles, the
/// piles remembered, and what chests hold. A line per affix moved, naming where it was.
pub(super) fn retier(state: &mut WorldState, content: &Content) -> Vec<String> {
    let mut lines = Vec::new();
    for actor in state.actors.values_mut() {
        for stack in &mut actor.inventory {
            let moved = stack.retier(content);
            lines.extend(
                moved
                    .into_iter()
                    .map(|l| format!("actor {}: {l}", actor.id)),
            );
        }
    }
    for level in state.levels.values_mut() {
        let held = level.chests.values_mut().map(|c| &mut c.holds);
        let piles = (level
            .objects
            .values_mut()
            .chain(level.object_memory.values_mut()))
        .chain(held);
        for stack in piles.flatten() {
            let moved = stack.retier(content);
            lines.extend(
                moved
                    .into_iter()
                    .map(|l| format!("level {}: {l}", level.id)),
            );
        }
    }
    lines
}
