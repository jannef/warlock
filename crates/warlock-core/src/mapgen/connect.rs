//! Connectivity: flood fill, cheapest-path digging for corridors and tunnels, and the validation
//! every generated level must pass.

use super::CARDINALS;
use crate::content::{Content, LevelTemplate, StairsDir, breach_fault, validate_route};
use crate::door::Lock;
use crate::geom::Pos;
use crate::grid::Grid;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Cells 4-connected to `from` through cells where `passable` holds (`from` included if
/// passable). Movement is 8-way, so this is stricter than what the player needs.
pub(super) fn flood(
    width: i32,
    height: i32,
    from: Pos,
    passable: impl Fn(Pos) -> bool,
) -> Grid<bool> {
    let mut seen = Grid::new(width, height, false);
    if !passable(from) || !seen.in_bounds(from) {
        return seen;
    }
    seen[from] = true;
    let mut stack = vec![from];
    while let Some(p) = stack.pop() {
        for dir in CARDINALS {
            let q = p.step(dir);
            if seen.get(q) == Some(&false) && passable(q) {
                seen[q] = true;
                stack.push(q);
            }
        }
    }
    seen
}

/// Breadth-first step counts from `from` over passable cells (4-connected); `u32::MAX` where
/// unreachable.
pub(super) fn distances(
    width: i32,
    height: i32,
    from: Pos,
    passable: impl Fn(Pos) -> bool,
) -> Grid<u32> {
    let mut dist = Grid::new(width, height, u32::MAX);
    if !dist.in_bounds(from) || !passable(from) {
        return dist;
    }
    dist[from] = 0;
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        let d = dist[p];
        for dir in CARDINALS {
            let q = p.step(dir);
            if dist.get(q) == Some(&u32::MAX) && passable(q) {
                dist[q] = d + 1;
                queue.push_back(q);
            }
        }
    }
    dist
}

/// The cheapest 4-connected path from one of `starts` (each with an initial cost) to a cell where
/// `is_goal` holds, as cells from start to goal inclusive. `cost(p)` is the price of entering `p`
/// (`None` where digging is forbidden; starts must be enterable too) and every change of direction
/// adds `turn`, so paths run straight with few bends. Ties break by insertion order, so the result
/// is deterministic.
pub(super) fn cheapest_path(
    width: i32,
    height: i32,
    starts: &[(Pos, u32)],
    cost: impl Fn(Pos) -> Option<u32>,
    is_goal: impl Fn(Pos) -> bool,
    turn: u32,
) -> Option<Vec<Pos>> {
    // State: cell × the direction we entered it by (4 = a start, no direction yet).
    const DIRS: usize = 5;
    let cells = (width.max(0) * height.max(0)) as usize;
    let index = |p: Pos| (p.y * width + p.x) as usize;
    let pos_of = |cell: usize| Pos::new(cell as i32 % width, cell as i32 / width);
    let in_bounds = |p: Pos| p.x >= 0 && p.y >= 0 && p.x < width && p.y < height;
    let mut best = vec![u32::MAX; cells * DIRS];
    let mut parent = vec![usize::MAX; cells * DIRS];
    let mut heap = BinaryHeap::new();
    let mut seq: u64 = 0;
    for &(p, initial) in starts {
        if !in_bounds(p) || cost(p).is_none() {
            continue;
        }
        let s = index(p) * DIRS + 4;
        if initial < best[s] {
            best[s] = initial;
            heap.push(Reverse((initial, seq, s)));
            seq += 1;
        }
    }
    while let Some(Reverse((c, _, s))) = heap.pop() {
        if c > best[s] {
            continue;
        }
        let (cell, from_dir) = (s / DIRS, s % DIRS);
        let p = pos_of(cell);
        if is_goal(p) {
            let mut path = vec![p];
            let mut at = s;
            while parent[at] != usize::MAX {
                at = parent[at];
                path.push(pos_of(at / DIRS));
            }
            path.reverse();
            return Some(path);
        }
        for (d, dir) in CARDINALS.iter().enumerate() {
            let q = p.step(*dir);
            if !in_bounds(q) {
                continue;
            }
            let Some(step) = cost(q) else { continue };
            let bend = if from_dir != 4 && from_dir != d {
                turn
            } else {
                0
            };
            let next = c.saturating_add(step).saturating_add(bend);
            let t = index(q) * DIRS + d;
            if next < best[t] {
                best[t] = next;
                parent[t] = s;
                heap.push(Reverse((next, seq, t)));
                seq += 1;
            }
        }
    }
    None
}

/// Why a generated level is not acceptable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Invalid {
    NoUpStairs,
    Unreachable(Pos),
    /// Reached only through a locked door or a secret one.
    LockedAway(Pos),
    /// A lock on something other than a closed door.
    BadLock(Pos),
    /// No key to the lock here within reach (without unlocking or finding a door).
    NoKey(Pos),
    BadPatrol(String),
    Dark,
    /// A sealed room (`sealed = "dig"`) with no breach, at its corner.
    NoBreach(Pos),
    /// A sealed room's breach that does not open it ([`breach_fault`]).
    BadBreach(Pos),
}

/// Checks a generated level: it has an up staircase (the arrival point and player start), and
/// every stair, door (secret ones too), and walkable cell is reachable from it; locks lie only on
/// closed doors and shut chests, and the stairs and a key to every door's lock (lying, or carried by a creature) are
/// reachable without unlocking any door (spec §8 "Doors") or finding any secret door (spec §8
/// "Searching"), so no level needs a key it does not give or a door the player may never find;
/// every patrol stays on walkable terrain and loops; and at least one cell emits light. The one
/// exception to reaching every cell: a sealed room (a greater treasure room, `sealed = "dig"`),
/// whose cells are reached by breaking its breach, provided the breach opens it
/// ([`breach_fault`]); nothing the level needs (stairs, a key, a creature) is ever inside one.
pub(super) fn validate(level: &LevelTemplate, content: &Content) -> Result<(), Invalid> {
    let terrain = &level.terrain;
    let up = level
        .stairs
        .iter()
        .find(|s| s.dir == StairsDir::Up)
        .map(|s| s.pos)
        .filter(|p| level.player_start == Some(*p))
        .ok_or(Invalid::NoUpStairs)?;
    let passable = |p: Pos| {
        level.secrets.contains_key(&p)
            || terrain.get(p).is_some_and(|t| {
                let def = content.terrain(*t);
                def.walkable || def.is_closed_door()
            })
    };
    let reached = flood(terrain.width(), terrain.height(), up, passable);
    let mut sealed = Vec::new();
    for room in &level.rooms {
        if content.room_effect(room.effect).sealed.is_none() {
            continue;
        }
        let breach = room.breach.ok_or(Invalid::NoBreach(room.min))?;
        let reach = |p: Pos| reached.get(p) == Some(&true);
        if breach_fault(terrain, content, room, breach, reach).is_some() {
            return Err(Invalid::BadBreach(breach));
        }
        sealed.push(room);
    }
    let walled = |p: Pos| sealed.iter().any(|r| r.contains(p));
    if let Some((p, _)) =
        (terrain.iter()).find(|(p, _)| passable(*p) && !reached[*p] && !walled(*p))
    {
        return Err(Invalid::Unreachable(p));
    }
    if let Some(s) = level.stairs.iter().find(|s| !reached[s.pos]) {
        return Err(Invalid::Unreachable(s.pos));
    }
    if !level.locks.is_empty() || !level.secrets.is_empty() {
        let chest = |p: Pos| (terrain.get(p)).is_some_and(|t| content.terrain(*t).is_shut_chest());
        let closed = |p: Pos| {
            terrain
                .get(p)
                .is_some_and(|t| content.terrain(*t).is_closed_door())
        };
        if let Some(p) = level.locks.keys().find(|p| !closed(**p) && !chest(**p)) {
            return Err(Invalid::BadLock(*p));
        }
        let unlocked = |p: Pos| {
            passable(p) && !level.locks.contains_key(&p) && !level.secrets.contains_key(&p)
        };
        let open = flood(terrain.width(), terrain.height(), up, unlocked);
        if let Some(s) = level.stairs.iter().find(|s| !open[s.pos]) {
            return Err(Invalid::LockedAway(s.pos));
        }
        let keyless = |lock: &Lock| !level.has_key_for(content, lock, |p| open[p]);
        // A generated chest's lock wants no key: Pick lock, Knock, or a kick open it, and a
        // chest is never in the way (spec §8 "Chests").
        let needs_key = |(p, lock): (&Pos, &Lock)| !chest(*p) && keyless(lock);
        if let Some((p, _)) = level.locks.iter().find(|&entry| needs_key(entry)) {
            return Err(Invalid::NoKey(*p));
        }
    }
    for spawn in &level.creatures {
        validate_route(level, content, spawn).map_err(Invalid::BadPatrol)?;
        if !reached[spawn.pos] {
            return Err(Invalid::Unreachable(spawn.pos));
        }
    }
    let emits = terrain
        .cells()
        .iter()
        .any(|t| content.terrain(*t).light.is_some());
    if level.lights.is_empty() && !emits {
        return Err(Invalid::Dark);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheapest_path_prefers_straight_lines_and_respects_walls() {
        // A wall at x = 5 except a gap at y = 8.
        let cost = |p: Pos| (p.x != 5 || p.y == 8).then_some(1);
        let path = cheapest_path(
            10,
            10,
            &[(Pos::new(1, 1), 0)],
            cost,
            |p| p == Pos::new(8, 1),
            3,
        )
        .expect("a path through the gap");
        assert_eq!(path.first(), Some(&Pos::new(1, 1)));
        assert_eq!(path.last(), Some(&Pos::new(8, 1)));
        assert!(path.contains(&Pos::new(5, 8)));
        for pair in path.windows(2) {
            assert_eq!(pair[0].dist2(pair[1]), 1, "4-connected steps");
        }
        let blocked = |p: Pos| (p.x != 5).then_some(1);
        assert!(cheapest_path(10, 10, &[(Pos::new(1, 1), 0)], blocked, |p| p.x == 8, 3).is_none());
    }

    #[test]
    fn flood_and_distances_agree() {
        let passable = |p: Pos| p.y != 2;
        let seen = flood(4, 4, Pos::new(0, 0), passable);
        let dist = distances(4, 4, Pos::new(0, 0), passable);
        for (p, s) in seen.iter() {
            assert_eq!(*s, dist[p] != u32::MAX, "{p}");
        }
        assert_eq!(dist[Pos::new(3, 1)], 4);
        assert!(!seen[Pos::new(0, 3)]);
    }
}
