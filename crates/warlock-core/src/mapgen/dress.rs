//! A special room's dressing (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): an
//! armoury's weapon racks, a workshop's benches. Pieces of a terrain that is seen through but
//! not walked, stood along the room's walls when the room is made, from the level's map stream
//! (only for an effect with `dressing`, so other levels draw as before).
//!
//! A piece stands on plain floor nothing else holds (no stamp, light, creature or patrol, trap,
//! or object) with a wall (any cell that is not walkable) beside it, never beside a door or a
//! way out of the room (a cave chamber's floor running on), and never where it would cut the
//! room's walkable cells apart; the cells are drawn among those
//! left, one after another, until the room has its [`Dressing::pieces`] or none is left.

use super::Canvas;
use crate::content::{Dressing, SpecialRoom};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;

/// Stands `dressing` along the walls of `room` (made already).
pub(super) fn dress(canvas: &mut Canvas, rng: &mut Rng, room: &SpecialRoom, dressing: Dressing) {
    let content = canvas.content;
    let cells = room.cells();
    let mut walkable: Vec<Pos> = (cells.iter().copied())
        .filter(|&p| canvas.is_walkable(p))
        .collect();
    let walked: Vec<Pos> = (canvas.creatures.iter())
        .flat_map(|c| {
            let mut p = c.pos;
            std::iter::once(p).chain(c.route.iter().map(move |d| {
                p = p.step(*d);
                p
            }))
        })
        .collect();
    let around = |p: Pos, dirs: &[Dir], what: &dyn Fn(Pos) -> bool| {
        (dirs.iter()).any(|&d| {
            let q = p.step(d);
            canvas.terrain.in_bounds(q) && what(q)
        })
    };
    let door = |q: Pos| content.terrain(canvas.get(q)).is_door();
    let wall = |q: Pos| !canvas.is_walkable(q);
    // A way out of the room (a cave chamber's floor runs on past its edge).
    let out = |q: Pos| canvas.is_walkable(q) && !room.contains(q);
    let mut free: Vec<Pos> = (cells.iter().copied())
        .filter(|&p| canvas.is_free_floor(p) && !walked.contains(&p))
        .filter(|p| !canvas.traps.contains_key(p) && !canvas.objects.contains_key(p))
        .filter(|&p| around(p, &super::CARDINALS, &wall))
        .filter(|&p| !around(p, &Dir::ALL, &door) && !around(p, &Dir::ALL, &out))
        .collect();
    let mut left = dressing.pieces(walkable.len());
    while left > 0 && !free.is_empty() {
        let at = free.swap_remove(rng.index(free.len()));
        if !super::special::joined(&walkable, at) {
            continue;
        }
        canvas.set(at, dressing.terrain);
        walkable.retain(|&p| p != at);
        left -= 1;
    }
}
