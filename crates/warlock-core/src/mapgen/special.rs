//! Special rooms in generated levels (spec §7 "Special rooms").
//!
//! Each room of a rooms level (but the arrival room and vaults), and each of [`CAVE_CHAMBERS`]
//! chambers a cave level tries, is special with the branch's `chance` (its `room_effects` table,
//! else the shared one: `Canvas::room_odds`), until the level has `per_level` of them; the
//! effect is picked uniformly among the branch's `effects`. A rooms level's special room is the room
//! with its walls and doors; a cave's is a blob of its floor ([`chamber`]) with the rock around
//! it. The effect's floor, if it has one (a flooded room's shallow water), is laid over the
//! plain floor in it (doorways too). A room whose effect creatures shun (a sanctuary) is never one
//! a creature patrols through. Both generators decide this last, from the same stream, so a level
//! without a special room is exactly what it would be without them.

use super::Canvas;
use crate::content::{RoomEffectId, SpecialRoom};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;
use std::collections::VecDeque;

/// Chambers a cave level tries for a special room: about as many as a rooms level has rooms.
pub(super) const CAVE_CHAMBERS: usize = 8;
/// A cave chamber is the floor within this squared distance of its middle (radius 5)...
const CHAMBER_R2: i32 = 30;
/// ...and has at least this many cells, else there is no room for one there.
const MIN_CHAMBER: usize = 12;
/// A cave chamber's middle lies farther than this (Chebyshev) from the arrival, so the player
/// never arrives inside one.
const AWAY: i32 = 10;

/// Rolls whether the next room or chamber is special and with what effect: `None` when the
/// level has as many special rooms as it may get, or the roll fails.
pub(super) fn roll(canvas: &Canvas, rng: &mut Rng) -> Option<RoomEffectId> {
    let odds = canvas.room_odds;
    if !odds.generated() || canvas.rooms.len() >= usize::from(odds.per_level) {
        return None;
    }
    if !rng.chance(u32::from(odds.chance)) {
        return None;
    }
    odds.effects.get(rng.index(odds.effects.len())).copied()
}

/// Makes `room` special, laying its effect's floor over the plain floor in it: unless it
/// overlaps another special room, or creatures shun it and one patrols through it. Returns
/// whether it did.
pub(super) fn make(canvas: &mut Canvas, room: SpecialRoom) -> bool {
    let effect = canvas.content.room_effect(room.effect);
    let patrolled = || {
        canvas.creatures.iter().any(|c| {
            let mut p = c.pos;
            room.contains(p)
                || c.route.iter().any(|d| {
                    p = p.step(*d);
                    room.contains(p)
                })
        })
    };
    if canvas.rooms.iter().any(|r| r.overlaps(&room)) || (effect.shunned && patrolled()) {
        return false;
    }
    if let Some(floor) = effect.floor {
        for p in room.cells() {
            if canvas.get(p) == canvas.pal.floor && !canvas.is_stamped(p) {
                canvas.set(p, floor);
            }
        }
    }
    canvas.rooms.push(room);
    true
}

/// A cave's special room: a few of its chambers may be special (see the module docs).
pub(super) fn cave_rooms(canvas: &mut Canvas, rng: &mut Rng, up: Pos) {
    for _ in 0..CAVE_CHAMBERS {
        let Some(effect) = roll(canvas, rng) else {
            continue;
        };
        let middles: Vec<Pos> = (canvas.cells())
            .filter(|p| canvas.is_free_floor(*p) && p.chebyshev(up) > AWAY)
            .filter(|p| !canvas.rooms.iter().any(|r| r.contains(*p)))
            .collect();
        let Some(&middle) = rng.pick(&middles) else {
            continue;
        };
        let floor = chamber(canvas, middle);
        if floor.len() < MIN_CHAMBER {
            continue;
        }
        // The rock around the floor belongs to it too, so its walls look as its floor does.
        let mut cells = floor.clone();
        for &p in &floor {
            for q in Dir::ALL.map(|d| p.step(d)) {
                if canvas.terrain.in_bounds(q) && !canvas.is_walkable(q) {
                    cells.push(q);
                }
            }
        }
        make(canvas, SpecialRoom::blob(effect, cells));
    }
}

/// The walkable cells (vaults left out) reachable from `middle` without leaving the circle of
/// [`CHAMBER_R2`] around it, 4-connected, in the order found.
fn chamber(canvas: &Canvas, middle: Pos) -> Vec<Pos> {
    let open =
        |p: Pos| canvas.is_walkable(p) && !canvas.is_stamped(p) && p.dist2(middle) <= CHAMBER_R2;
    let mut found = vec![middle];
    let mut queue = VecDeque::from([middle]);
    while let Some(p) = queue.pop_front() {
        for q in super::CARDINALS.map(|d| p.step(d)) {
            if open(q) && !found.contains(&q) {
                found.push(q);
                queue.push_back(q);
            }
        }
    }
    found
}
