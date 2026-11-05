//! Deep water in caves (#1020; `docs/dev/mapgen.md`). On about one `caves` floor in six, a blob of
//! 40 to 70 free floor cells becomes water, grown as the pools are, and every cell of it whose
//! eight neighbours are all of it becomes deep water: the shore stays wadeable. The floor is
//! otherwise finished (stairs, lights, patrols, rooms), so the stream is the only draw it makes,
//! apart from the map's: a floor without it is its old self.

use super::Canvas;
use super::connect::flood;
use crate::geom::{Dir, Pos};
use crate::rng::Rng;
use std::collections::BTreeSet;

/// One floor in this many has a deep pool.
const ONE_IN: u32 = 6;
const SIZE: (i32, i32) = (40, 70);
/// Tries at a blob with a deep middle before the floor goes without.
const TRIES: usize = 8;

/// Lays the floor's deep pool, if the stream says it has one and it fits.
pub(super) fn lay(canvas: &mut Canvas, seed: u64, level: &str) {
    let mut rng = Rng::deep_water(seed, level);
    if rng.below(ONE_IN) != 0 || canvas.pervading.is_some() {
        return;
    }
    let Some(up) = canvas.up else { return };
    // Cells a creature walks must stay walkable.
    let mut walked = BTreeSet::new();
    for c in &canvas.creatures {
        let mut p = c.pos;
        walked.insert(p);
        for d in &c.route {
            p = p.step(*d);
            walked.insert(p);
        }
    }
    let free = |canvas: &Canvas, p: Pos| {
        canvas.is_free_floor(p)
            && !walked.contains(&p)
            && !canvas.rooms.iter().any(|r| r.contains(p))
    };
    let (w, h) = (canvas.width(), canvas.height());
    for _ in 0..TRIES {
        let start = (0..30)
            .map(|_| Pos::new(rng.range(1, w - 2), rng.range(1, h - 2)))
            .find(|p| free(canvas, *p));
        let Some(start) = start else { continue };
        let size = rng.range(SIZE.0, SIZE.1) as usize;
        let mut blob = BTreeSet::new();
        let mut frontier = vec![start];
        while blob.len() < size && !frontier.is_empty() {
            let p = frontier.swap_remove(rng.index(frontier.len()));
            if blob.contains(&p) || !free(canvas, p) {
                continue;
            }
            blob.insert(p);
            frontier.extend(super::CARDINALS.map(|d| p.step(d)));
        }
        let deep: Vec<Pos> = blob
            .iter()
            .copied()
            .filter(|p| Dir::ALL.iter().all(|d| blob.contains(&p.step(*d))))
            .collect();
        if deep.is_empty() {
            continue;
        }
        let (water, deep_water, floor) =
            (canvas.pal.water, canvas.pal.deep_water, canvas.pal.floor);
        let before = reached(canvas, up);
        for p in &blob {
            canvas.set(*p, water);
        }
        for p in &deep {
            canvas.set(*p, deep_water);
        }
        // Everything walkable before stays reachable on foot, bar the deep cells themselves.
        if reached(canvas, up) + deep.len() == before {
            return;
        }
        for p in &blob {
            canvas.set(*p, floor);
        }
    }
}

/// How many cells the up staircase reaches on foot.
fn reached(canvas: &Canvas, up: Pos) -> usize {
    let seen = flood(canvas.width(), canvas.height(), up, |p| {
        canvas.is_passable(p)
    });
    seen.cells().iter().filter(|r| **r).count()
}
