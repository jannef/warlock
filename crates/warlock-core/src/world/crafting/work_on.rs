//! What long work on the pack works on (#1564): salvage, Temper and Whittle, Cut, Set, Make do
//! and mend, Decipher, the enchanting recipes, and a recipe's making name their stacks by pack
//! position as they begin, and a stack gone meanwhile (food rotting away, which does not stop
//! the work) shifts every later one. So the work driver (`work.rs`) keeps a copy of each work's
//! stacks when its time begins ([`World::work_on`]) and finds those very stacks again when it
//! ends ([`World::worked_on`]): at the old position if they are still there, else wherever they went
//! (an unidentified magic item's time carried, and its identification by carrying, aside); a
//! stack gone from the pack (or changed) ends the work on nothing ("You have nothing left to
//! work on."), never on whatever came to stand where it was.

use super::World;
use crate::object::Stack;

/// The stacks a long work began on, in pick order: each pick's position then and the stack it
/// held (`None` for a pick left out).
#[derive(Clone, Debug)]
pub(super) struct WorkOn(Vec<Option<(usize, Option<Stack>)>>);

impl World {
    /// The player's stacks at `picks` as long work on them begins.
    pub(super) fn work_on(&self, picks: &[Option<usize>]) -> WorkOn {
        let pack = &self.state.player().inventory;
        WorkOn(
            (picks.iter())
                .map(|p| p.map(|at| (at, pack.get(at).cloned())))
                .collect(),
        )
    }

    /// Where the stacks `on` are in the player's pack now, in the same order (`None` for a pick
    /// left out): each at its old position if it is still there, else the first stack the same
    /// as it; `None` when any is gone.
    pub(super) fn worked_on(&self, on: &WorkOn) -> Option<Vec<Option<usize>>> {
        let pack = &self.state.player().inventory;
        let held = |at: usize, stack: &Stack| pack.get(at).is_some_and(|s| same(s, stack));
        let find = |at: usize, stack: &Stack| {
            (held(at, stack))
                .then_some(at)
                .or_else(|| pack.iter().position(|s| same(s, stack)))
        };
        (on.0.iter())
            .map(|pick| match pick {
                None => Some(None),
                Some((at, Some(stack))) => find(*at, stack).map(Some),
                Some((_, None)) => None,
            })
            .collect()
    }
}

/// Whether `a` is the stack `b` was: equal but for what carrying an unidentified magic item
/// changes as the time passes (its time carried, and whether that has taught what it is).
fn same(a: &Stack, b: &Stack) -> bool {
    let plain = |s: &Stack| {
        let mut s = s.clone();
        if let Some(magic) = s.magic_mut() {
            (magic.carried, magic.identified) = (0, false);
        }
        s
    };
    a == b || plain(a) == plain(b)
}

/// A loaf of bread that rots away a minute from now, for the tests of work outlasting a stack
/// ahead of its own.
#[cfg(test)]
pub(super) fn rotting_loaf(w: &World) -> Stack {
    let bread = w.content.object_id("bread").expect("bread");
    let now = w.state.time;
    let gone = crate::rot::gone_at(&w.content, &Stack::corpse(bread, now)).expect("it rots");
    let lasts = gone.since(now).as_seconds();
    let died = crate::time::GameTime::from_seconds(now.seconds() + 60 - lasts);
    Stack::corpse(bread, died)
}
