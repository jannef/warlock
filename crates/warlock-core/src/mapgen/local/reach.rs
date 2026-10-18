//! Getting around a local area: game trails, joining every walkable pocket to the rest (small
//! ones are grown over instead), ways out on every side, and the checks every area must pass.

use super::super::connect::{cheapest_path, flood};
use super::land::{Claim, Land};
use super::water::{course, line4, point_on};
use crate::content::{Content, LevelTemplate, validate_route};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::rng::Rng;
use std::collections::BTreeMap;

/// Walkable cells every side must have at least, so the player can always walk off it.
pub(super) const MIN_WAYS_OUT: usize = 3;

/// Pockets this small are grown over rather than joined to the rest.
const SMALL_POCKET: usize = 12;

/// How hard it is to clear a way through `p` (`None`: never, e.g. ruins). Walkable cells cost 1.
fn clearing_cost(land: &Land, p: Pos) -> Option<u32> {
    if land.is_walkable(p) {
        return Some(1);
    }
    if land.claim(p) == Claim::Fixed {
        return None;
    }
    let t = land.get(p)?;
    let k = land.kit;
    Some(if t == k.crag || t == k.standing_stone {
        14
    } else if t == k.boulder {
        8
    } else if t == k.pine {
        7
    } else {
        5
    })
}

/// Clears `path` down to the area's ground where it is blocked.
fn clear(land: &mut Land, path: &[Pos]) {
    for &p in path {
        if !land.is_walkable(p) {
            land.set(p, land.ground);
        }
    }
}

/// A game trail from `from` out to side `side`, one cell wide and winding, cleared of trees and
/// thickets down to the ground (water, roads, and ruins are laid for good: it crosses them).
pub(super) fn trail(land: &mut Land, rng: &mut Rng, from: Pos, side: Dir) {
    let target = point_on(land, rng, side, 10);
    let points = course(rng, from, target, 3, 8);
    for pair in points.windows(2) {
        for p in line4(pair[0], pair[1]) {
            if land.in_bounds(p) && land.claim(p) != Claim::Fixed && !land.is_dry_footing(p) {
                land.set(p, land.ground);
            }
        }
    }
}

/// The walkable cell nearest the middle, on dry footing if any (rows count double).
pub(super) fn start(land: &Land) -> Option<Pos> {
    let c = land.center();
    land.cells()
        .filter(|p| land.is_walkable(*p) && !land.is_water(*p) && !land.on_edge(*p))
        .min_by_key(|p| {
            let (dx, dy) = (p.x - c.x, 2 * (p.y - c.y));
            (!land.is_dry_footing(*p), dx * dx + dy * dy, p.y, p.x)
        })
}

/// Makes every walkable cell reachable from `start` (4-connected, stricter than the player's
/// 8-way steps): small pockets are grown over with what surrounds them, larger ones get a way
/// cleared to the rest. Then every side gets ways out. Fails if a pocket cannot be joined.
pub(super) fn join(land: &mut Land, start: Pos) -> Option<()> {
    let (w, h) = (land.width(), land.height());
    for _ in 0..200 {
        let main = flood(w, h, start, |p| land.is_walkable(p));
        let Some(stray) = land.cells().find(|p| land.is_walkable(*p) && !main[*p]) else {
            break;
        };
        let pocket = flood(w, h, stray, |p| land.is_walkable(p));
        let cells: Vec<Pos> = land.cells().filter(|p| pocket[*p]).collect();
        let fixed = cells.iter().any(|p| land.claim(*p) == Claim::Fixed);
        if cells.len() <= SMALL_POCKET && !fixed {
            let filler = surrounding(land, &pocket, &cells);
            for p in cells {
                land.set(p, filler);
            }
            continue;
        }
        let starts: Vec<(Pos, u32)> = cells.iter().map(|p| (*p, 0)).collect();
        let path = cheapest_path(w, h, &starts, |p| clearing_cost(land, p), |p| main[p], 2)?;
        clear(land, &path);
    }
    for side in [Dir::N, Dir::E, Dir::S, Dir::W] {
        for _ in 0..MIN_WAYS_OUT {
            if ways_out(w, h, side, |p| land.is_walkable(p)) >= MIN_WAYS_OUT {
                break;
            }
            let main = flood(w, h, start, |p| land.is_walkable(p));
            let starts: Vec<(Pos, u32)> =
                land.cells().filter(|p| main[*p]).map(|p| (p, 0)).collect();
            let blocked_side = |p: Pos| on_side(w, h, p, side) && !land.is_walkable(p);
            let path = cheapest_path(w, h, &starts, |p| clearing_cost(land, p), blocked_side, 2)?;
            clear(land, &path);
        }
    }
    Some(())
}

/// The most common obstacle bordering `pocket` (the area's leafy tree if none).
fn surrounding(land: &Land, pocket: &Grid<bool>, cells: &[Pos]) -> crate::content::TerrainId {
    let mut counts: BTreeMap<crate::content::TerrainId, usize> = BTreeMap::new();
    for p in cells {
        for d in [Dir::N, Dir::E, Dir::S, Dir::W] {
            let q = p.step(d);
            if let Some(t) = land.get(q)
                && !pocket[q]
                && !land.is_walkable(q)
            {
                *counts.entry(t).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(t, n)| (*n, std::cmp::Reverse(*t)))
        .map_or(land.kit.tree, |(t, _)| t)
}

/// Whether `p` lies on side `side` (N, E, S, or W) of a `w`×`h` map.
fn on_side(w: i32, h: i32, p: Pos, side: Dir) -> bool {
    match side {
        Dir::N => p.y == 0,
        Dir::S => p.y == h - 1,
        Dir::W => p.x == 0,
        _ => p.x == w - 1,
    }
}

/// Walkable cells on side `side`.
fn ways_out(w: i32, h: i32, side: Dir, walkable: impl Fn(Pos) -> bool) -> usize {
    let cells: Vec<Pos> = match side {
        Dir::N | Dir::S => (0..w)
            .map(|x| Pos::new(x, if side == Dir::N { 0 } else { h - 1 }))
            .collect(),
        _ => (0..h)
            .map(|y| Pos::new(if side == Dir::W { 0 } else { w - 1 }, y))
            .collect(),
    };
    cells.into_iter().filter(|p| walkable(*p)).count()
}

/// Why a generated local area is not acceptable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Invalid {
    NoStart,
    Unreachable(Pos),
    NoWayOut(Dir),
    BadFlow(Pos),
    BadPatrol(String),
}

/// Checks a generated local area: the start is walkable; every walkable cell is reachable from it
/// (4-connected); every side has ways out; running water and only running water has a flow; and
/// patrols stay on walkable terrain and loop.
pub(super) fn validate(level: &LevelTemplate, content: &Content) -> Result<(), Invalid> {
    let terrain = &level.terrain;
    let (w, h) = (terrain.width(), terrain.height());
    let walkable = |p: Pos| terrain.get(p).is_some_and(|t| content.terrain(*t).walkable);
    let start = level
        .player_start
        .filter(|p| walkable(*p))
        .ok_or(Invalid::NoStart)?;
    let reached = flood(w, h, start, walkable);
    if let Some((p, _)) = terrain.iter().find(|(p, _)| walkable(*p) && !reached[*p]) {
        return Err(Invalid::Unreachable(p));
    }
    for side in [Dir::N, Dir::E, Dir::S, Dir::W] {
        if ways_out(w, h, side, walkable) < MIN_WAYS_OUT {
            return Err(Invalid::NoWayOut(side));
        }
    }
    for (p, t) in terrain.iter() {
        if content.terrain(*t).flows() != level.flow.contains_key(&p) {
            return Err(Invalid::BadFlow(p));
        }
    }
    for spawn in &level.creatures {
        validate_route(level, content, spawn).map_err(Invalid::BadPatrol)?;
    }
    Ok(())
}

/// Where a player walking in from side `side` (any of the 8 directions; a corner allows either
/// of its sides) arrives: a walkable edge cell near the middle of that side (or the corner):
/// on the road if one leaves the area there (it comes from a road on the overworld), else on
/// dry footing if there is any, trading a few steps along the side for open ground around it
/// (rather than a thicket).
pub fn arrival(
    terrain: &Grid<crate::content::TerrainId>,
    content: &Content,
    side: Dir,
) -> Option<Pos> {
    let road = super::Kit::resolve(content)
        .ok()
        .map(|k| [k.road, k.bridge]);
    let on_road = |t: &crate::content::TerrainId| road.is_some_and(|r| r.contains(t));
    let (w, h) = (terrain.width(), terrain.height());
    let (dx, dy) = side.delta();
    let target = Pos::new(
        match dx {
            -1 => 0,
            1 => w - 1,
            _ => w / 2,
        },
        match dy {
            -1 => 0,
            1 => h - 1,
            _ => h / 2,
        },
    );
    let on = |p: Pos| {
        (dx == -1 && p.x == 0)
            || (dx == 1 && p.x == w - 1)
            || (dy == -1 && p.y == 0)
            || (dy == 1 && p.y == h - 1)
    };
    let walkable = |p: Pos| terrain.get(p).is_some_and(|t| content.terrain(*t).walkable);
    // Blocked cells (or none: off the map) within 3 columns and 2 rows.
    let closed = |p: Pos| {
        (-2..=2)
            .flat_map(|dy| (-3..=3).map(move |dx| Pos::new(p.x + dx, p.y + dy)))
            .filter(|q| !walkable(*q))
            .count() as i32
    };
    terrain
        .iter()
        .filter(|(p, _)| on(*p) && walkable(*p))
        .min_by_key(|(p, t)| {
            let wet = content.terrain(**t).move_cost > 100;
            let (ex, ey) = (p.x - target.x, 2 * (p.y - target.y));
            (
                !on_road(t),
                wet,
                ex * ex + ey * ey + 40 * closed(*p),
                p.y,
                p.x,
            )
        })
        .map(|(p, _)| p)
}
