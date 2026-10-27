//! Special rooms in generated levels (spec §7 "Special rooms"; `docs/dev/special-rooms.md`).
//!
//! Each room of a rooms level (but the arrival room and vaults), and each of [`CAVE_CHAMBERS`]
//! chambers a cave level tries, is special with the branch's `chance` (its `room_effects` table,
//! else the shared one: `Canvas::room_odds`), until the level has `per_level` of them. Its effect
//! is then drawn ([`draw`]) among those the room is eligible for and the branch does not ban:
//! a favoured effect by its fixed share, the rest by their own `weight`. An effect with a
//! `level_chance` covers the whole level instead with that chance (one more draw; the room
//! stays plain, and it counts toward `per_level`). A rooms level's special room is the room with
//! its walls and doors; a cave's is a blob of its floor ([`chamber`]) with the rock around it.
//! The effect's floor, if it has one (a flooded room's shallow water), is laid over the plain
//! floor in it (doorways too); a collapsed floor's cracks only over its middle ([`middle`]). A room whose effect creatures shun (a sanctuary), or a teeming
//! den, is never one a creature patrols through. Both generators decide this last, from the same
//! stream, so a level without a special room is exactly what it would be without them.

use super::Canvas;
use crate::alignment::Alignment;
use crate::content::{RoomEffectDef, RoomEffectId, RoomEffects, RoomOdds, SpecialRoom};
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

/// Rolls whether the next room or chamber is special: false when the level has as many special
/// rooms as it may get (one covering the level counts), or the roll fails.
pub(super) fn roll(canvas: &Canvas, rng: &mut Rng) -> bool {
    let odds = canvas.room_odds;
    let taken = canvas.rooms.len() + usize::from(canvas.pervading.is_some());
    if !odds.generated() || canvas.content.room_effects().defs().is_empty() {
        return false;
    }
    taken < usize::from(odds.per_level) && rng.chance(u32::from(odds.chance))
}

/// Draws the effect of a special room among the effects not banned that `eligible` allows: by
/// the favoured ones' shares first (one draw in 0..100, walking them in effect order), else, or
/// when that draw lands past their shares, by the weights of the others (or, when every eligible
/// effect is favoured, by the favoured ones' shares). `None` when no effect is eligible.
pub(crate) fn draw(
    effects: &RoomEffects,
    odds: &RoomOdds,
    rng: &mut Rng,
    eligible: impl Fn(&RoomEffectDef) -> bool,
) -> Option<RoomEffectId> {
    let open: Vec<RoomEffectId> = (0..effects.defs().len())
        .filter_map(|i| u8::try_from(i).ok().map(RoomEffectId))
        .filter(|&id| !odds.bans(id) && eligible(effects.get(id)))
        .collect();
    let favoured: Vec<(RoomEffectId, u32)> = (open.iter())
        .filter_map(|&id| odds.share(id).map(|s| (id, u32::from(s))))
        .collect();
    if !favoured.is_empty() {
        let at = rng.below(100);
        let mut sum = 0;
        for &(id, share) in &favoured {
            sum += share;
            if sum > at {
                return Some(id);
            }
        }
    }
    let rest: Vec<(RoomEffectId, u32)> = (open.iter())
        .filter(|&&id| odds.share(id).is_none())
        .map(|&id| (id, u32::from(effects.get(id).weight)))
        .collect();
    let pool = if rest.is_empty() { &favoured } else { &rest };
    let total: u32 = pool.iter().map(|&(_, w)| w).sum();
    let mut at = rng.below(total);
    for &(id, weight) in pool {
        if at < weight {
            return Some(id);
        }
        at -= weight;
    }
    None
}

/// The effect of a special room of `cells` floor cells on this level, drawn ([`draw`]) among
/// those it is eligible for; and whether it covers the whole level instead (its `level_chance`,
/// one more draw, only for an effect that has one and while no effect covers the level yet).
pub(super) fn pick(canvas: &Canvas, rng: &mut Rng, cells: usize) -> Option<(RoomEffectId, bool)> {
    let effects = canvas.content.room_effects();
    let (danger, below) = (canvas.danger, canvas.below);
    let peopled = |e: &RoomEffectDef| {
        (e.swarm.as_ref())
            .is_none_or(|s| super::den::peopled(canvas.content, s, canvas.population, canvas.least))
    };
    let id = draw(effects, canvas.room_odds, rng, |e| {
        e.eligible(cells, danger, below) && peopled(e)
    })?;
    let chance = effects.get(id).level_chance;
    let whole = chance > 0 && canvas.pervading.is_none() && rng.chance(u32::from(chance));
    Some((id, whole))
}

/// Makes `room` special as [`pick`] decided: it covers the level when `whole`, else it is
/// [`make`], with its hidden side drawn first if its effect has `sides` (one good to two neutral
/// to one evil; one more draw, only for such an effect; a shrine's good or evil, one to one), and
/// its dressing stood along its walls after if it has `dressing` (`dress.rs`; more draws, only for
/// such an effect).
pub(super) fn place(
    canvas: &mut Canvas,
    rng: &mut Rng,
    mut room: SpecialRoom,
    whole: bool,
) -> bool {
    if whole {
        canvas.pervading = Some(room.effect);
        return true;
    }
    let effect = canvas.content.room_effect(room.effect);
    if effect.shrine.is_some() {
        // A shrine of a side is good or evil, one to one.
        room.side = Some(match rng.below(2) {
            0 => Alignment::Good,
            _ => Alignment::Evil,
        });
    } else if effect.sides.is_some() {
        room.side = Some(match rng.below(4) {
            0 => Alignment::Good,
            3 => Alignment::Evil,
            _ => Alignment::Neutral,
        });
    }
    let dressing = canvas.content.room_effect(room.effect).dressing;
    if !make(canvas, room) {
        return false;
    }
    if let Some(dressing) = dressing
        && let Some(room) = canvas.rooms.last().cloned()
    {
        super::dress::dress(canvas, rng, &room, dressing);
    }
    true
}

/// Makes `room` special, laying its effect's floor over the plain floor in it: unless it
/// overlaps another special room, or creatures shun it (or it is a teeming den or a tempting hoard)
/// and one patrols through it. Returns whether it did.
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
    if canvas.rooms.iter().any(|r| r.overlaps(&room))
        || ((effect.shunned || effect.swarm.is_some() || effect.hoard.is_some()) && patrolled())
    {
        return false;
    }
    let feature = match effect.feature {
        Some(terrain) => match feature_cell(canvas, &room) {
            Some(at) => Some((at, terrain)),
            None => return false,
        },
        None => None,
    };
    if let Some(floor) = effect.floor {
        let rim = effect.collapse.is_some();
        let cells: Vec<Pos> = (room.cells().into_iter())
            .filter(|&p| canvas.get(p) == canvas.pal.floor && !canvas.is_stamped(p))
            .filter(|&p| !rim || middle(canvas, &room, p))
            .collect();
        for p in cells {
            canvas.set(p, floor);
        }
    }
    if let Some((at, terrain)) = feature {
        canvas.set(at, terrain);
        let light = room.side.and_then(|side| effect.side_light(side));
        if let Some(light) = light {
            canvas.lights.insert(at, light);
        }
    }
    canvas.rooms.push(room);
    true
}

/// Where a room's feature goes (spec §7 "Special rooms"): the cell of plain floor nearest the
/// middle of its box (ties by cell order) that nothing else holds (no stamp, light, creature or
/// patrol, trap, or object), that is no door's neighbour, and without which the room's walkable
/// cells stay joined; `None` when there is none.
fn feature_cell(canvas: &Canvas, room: &SpecialRoom) -> Option<Pos> {
    let content = canvas.content;
    let cells = room.cells();
    let middle = Pos::new((room.min.x + room.max.x) / 2, (room.min.y + room.max.y) / 2);
    let walked: Vec<Pos> = (canvas.creatures.iter())
        .flat_map(|c| {
            let mut p = c.pos;
            std::iter::once(p).chain(c.route.iter().map(move |d| {
                p = p.step(*d);
                p
            }))
        })
        .collect();
    let by_door = |p: Pos| {
        (Dir::ALL.iter()).any(|&d| {
            let q = p.step(d);
            canvas.terrain.in_bounds(q) && content.terrain(canvas.get(q)).is_door()
        })
    };
    let mut free: Vec<Pos> = (cells.iter().copied())
        .filter(|&p| canvas.is_free_floor(p) && !walked.contains(&p))
        .filter(|p| !canvas.traps.contains_key(p) && !canvas.objects.contains_key(p))
        .filter(|&p| !by_door(p))
        .collect();
    free.sort_by_key(|p| (p.dist2(middle), *p));
    let walkable: Vec<Pos> = (cells.iter().copied())
        .filter(|&p| canvas.is_walkable(p))
        .collect();
    free.into_iter().find(|&at| joined(&walkable, at))
}

/// Whether `cells` less `without` are all joined to each other, 4-connected.
pub(super) fn joined(cells: &[Pos], without: Pos) -> bool {
    let open: Vec<Pos> = cells.iter().copied().filter(|&p| p != without).collect();
    let Some(&first) = open.first() else {
        return true;
    };
    let mut found = vec![first];
    let mut queue = VecDeque::from([first]);
    while let Some(p) = queue.pop_front() {
        for q in super::CARDINALS.map(|d| p.step(d)) {
            if open.contains(&q) && !found.contains(&q) {
                found.push(q);
                queue.push_back(q);
            }
        }
    }
    found.len() == open.len()
}

/// Whether `p` lies in the middle of `room`, away from its rim: each of its eight neighbours is a
/// walkable cell of the room and no door. A collapsed floor cracks only there, so a ring of sound
/// floor runs from every door round the room (a room too narrow to have a middle gets no cracks).
fn middle(canvas: &Canvas, room: &SpecialRoom, p: Pos) -> bool {
    Dir::ALL.into_iter().map(|d| p.step(d)).all(|q| {
        room.contains(q)
            && canvas.is_walkable(q)
            && canvas.terrain.get(q).is_some_and(|&t| {
                let def = canvas.content.terrain(t);
                !def.is_closed_door() && !def.is_open_door()
            })
    })
}

/// A cave's special room: a few of its chambers may be special (see the module docs).
pub(super) fn cave_rooms(canvas: &mut Canvas, rng: &mut Rng, up: Pos) {
    for _ in 0..CAVE_CHAMBERS {
        if !roll(canvas, rng) {
            continue;
        }
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
        let Some((effect, whole)) = pick(canvas, rng, floor.len()) else {
            continue;
        };
        // The rock around the floor belongs to it too, so its walls look as its floor does.
        let mut cells = floor.clone();
        for &p in &floor {
            for q in Dir::ALL.map(|d| p.step(d)) {
                if canvas.terrain.in_bounds(q) && !canvas.is_walkable(q) {
                    cells.push(q);
                }
            }
        }
        place(canvas, rng, SpecialRoom::blob(effect, cells), whole);
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
