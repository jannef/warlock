//! Shortest paths for creatures (spec §8 "Creature behavior"): A* over king's moves with integer
//! costs and fixed tie-breaks, so the same situation always gives the same step.

use crate::geom::{Dir, Pos};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// What entering a cell costs a path, in steps' worth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cost {
    Impassable,
    /// 1 for a free cell; more where something has to happen first (a door to open) or might
    /// not (another actor in the way).
    Steps(u32),
}

/// A path search: from `from` to the nearest cell `goal` accepts, within `bound` cells
/// (Chebyshev) of `from`. Every goal cell lies within `reach` (Chebyshev) of `target`, which
/// guides the search.
#[derive(Clone, Copy, Debug)]
pub(super) struct Search {
    pub from: Pos,
    pub target: Pos,
    pub reach: i32,
    pub bound: i32,
}

impl Search {
    /// The first step of a cheapest path to a goal cell other than `from`, or `None` if no goal
    /// is reachable within the bound. Ties go to the path whose cells lie nearer the target, then
    /// to the earlier direction in [`Dir::ALL`] order.
    pub(super) fn first_step(
        &self,
        cost: impl Fn(Pos) -> Cost,
        goal: impl Fn(Pos) -> bool,
    ) -> Option<Dir> {
        let bound = self.bound.max(1);
        let side = 2 * bound + 1;
        let index = |p: Pos| {
            let (dx, dy) = (p.x - self.from.x + bound, p.y - self.from.y + bound);
            ((0..side).contains(&dx) && (0..side).contains(&dy))
                .then_some((dy * side + dx) as usize)
        };
        let pos = |i: usize| {
            let i = i as i32;
            Pos::new(
                self.from.x + i % side - bound,
                self.from.y + i / side - bound,
            )
        };
        // A lower bound on the steps left (consistent: it changes by at most 1 a step, and every
        // step costs at least 1).
        let estimate = |p: Pos| (p.chebyshev(self.target) - self.reach).max(0) as u32;

        let n = (side * side) as usize;
        let mut spent = vec![u32::MAX; n];
        let mut first: Vec<Option<Dir>> = vec![None; n];
        let mut done = vec![false; n];
        let mut open = BinaryHeap::new();
        let start = index(self.from)?;
        spent[start] = 0;
        // (estimated total, estimate, squared distance to the target, order pushed, cell)
        open.push(Reverse((estimate(self.from), 0u32, 0i32, 0u32, start)));
        let mut pushed = 0u32;
        while let Some(Reverse((_, _, _, _, i))) = open.pop() {
            if done[i] {
                continue;
            }
            done[i] = true;
            let p = pos(i);
            if i != start && goal(p) {
                return first[i];
            }
            for dir in Dir::ALL {
                let q = p.step(dir);
                let Some(j) = index(q) else { continue };
                if done[j] {
                    continue;
                }
                let Cost::Steps(c) = cost(q) else { continue };
                let total = spent[i].saturating_add(c.max(1));
                if total < spent[j] {
                    spent[j] = total;
                    first[j] = if i == start { Some(dir) } else { first[i] };
                    pushed += 1;
                    let h = estimate(q);
                    open.push(Reverse((
                        total.saturating_add(h),
                        h,
                        q.dist2(self.target),
                        pushed,
                        j,
                    )));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A map: `#` impassable, `+` costs 2, anything else 1; `G` marks goal cells.
    fn search(map: &[&str], from: (i32, i32), target: (i32, i32), reach: i32) -> Option<Dir> {
        let at = |p: Pos| {
            map.get(p.y as usize)
                .and_then(|row| row.chars().nth(p.x as usize))
                .unwrap_or('#')
        };
        let s = Search {
            from: Pos::new(from.0, from.1),
            target: Pos::new(target.0, target.1),
            reach,
            bound: 20,
        };
        s.first_step(
            |p| match at(p) {
                '#' => Cost::Impassable,
                '+' => Cost::Steps(2),
                _ => Cost::Steps(1),
            },
            |p| at(p) == 'G',
        )
    }

    #[test]
    fn walks_around_walls_on_a_shortest_path() {
        let map = [
            "#########", //
            "#.......#", //
            "#.#####.#", //
            "#.#G....#", //
            "#.#####.#", //
            "#...@...#", //
            "#########",
        ];
        // The goal is walled in but for its east side: around the east end.
        assert_eq!(search(&map, (4, 5), (3, 3), 0), Some(Dir::E));
        let blocked = [
            "#####", //
            "#@#G#", //
            "#####",
        ];
        assert_eq!(search(&blocked, (1, 1), (3, 1), 0), None);
    }

    #[test]
    fn ties_go_toward_the_target_then_by_direction() {
        let open = [
            "..........",
            "..........",
            "..........",
            "......G...",
            "..........",
        ];
        // Five east and one south: the first step heads straight for it diagonally.
        assert_eq!(search(&open, (1, 2), (6, 3), 0), Some(Dir::SE));
        // Straight east: no diagonal detour.
        let row = ["......G..."];
        assert_eq!(search(&row, (0, 0), (6, 0), 0), Some(Dir::E));
        // The same search always gives the same answer.
        assert_eq!(search(&open, (1, 2), (6, 3), 0), Some(Dir::SE));
    }

    #[test]
    fn costly_cells_are_avoided_when_a_detour_is_cheaper() {
        // The door costs one more than the way around it (as many steps): around.
        let door = [
            "#####", //
            "#.+G#", //
            "#...#", //
            "#####",
        ];
        assert_eq!(search(&door, (1, 1), (3, 1), 0), Some(Dir::SE));
        // No way around: through.
        let door = [
            "#####", //
            "#.+G#", //
            "#####",
        ];
        assert_eq!(search(&door, (1, 1), (3, 1), 0), Some(Dir::E));
        // A goal within reach of the target: any cell next to it.
        let near = [
            "#######", //
            "#..GGG#", //
            "#..GxG#", //
            "#..GGG#", //
            "#######",
        ];
        assert_eq!(search(&near, (1, 2), (4, 2), 1), Some(Dir::E));
    }
}
