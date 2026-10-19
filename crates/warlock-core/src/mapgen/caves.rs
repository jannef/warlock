//! Caves: cellular-automata caverns.
//!
//! 1. Random noise (55 % floor), smoothed five times with the 4-5 rule: a cell becomes rock with
//!    at least five rock neighbors and stays rock with at least four.
//! 2. Keep the largest 4-connected cave; everything else is rock. Too small a cave is a failed
//!    attempt.
//! 3. Stamp one or two vaults into solid rock (a random spot among all that fit) and tunnel from
//!    each entrance to the cave.
//! 4. A few pools of shallow water.
//! 5. The up staircase in a roomy spot; the down staircase in a roomy spot far from it (by walking
//!    distance).
//! 6. Light: a lamp beside the up staircase, lamps scattered over about half of the 16×16 blocks,
//!    and patches of faintly glowing fungus.
//! 7. A few creatures patrolling out and back along random walks.
//! 8. Now and then a chamber away from the arrival is special (`special.rs`): a blob of floor.

use super::connect::{cheapest_path, distances, flood};
use super::{Canvas, Needs, out_and_back, special, vault};
use crate::content::StairsDir;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::rng::Rng;

/// Percent of inner cells that start as rock.
const ROCK: u32 = 45;
const SMOOTHING: usize = 5;
/// The kept cave must cover at least this percent of the inner area.
const MIN_CAVE: i32 = 30;
/// Side of the blocks lamps are scattered over.
const LAMP_BLOCK: i32 = 16;
/// Longest tunnel from a vault entrance to the cave.
const MAX_TUNNEL: usize = 40;

pub(super) fn build(canvas: &mut Canvas, rng: &mut Rng, needs: Needs) -> Option<()> {
    noise(canvas, rng);
    for _ in 0..SMOOTHING {
        smooth(canvas);
    }
    keep_largest_cave(canvas)?;
    let vaults = 1 + usize::from(rng.chance(30));
    place_vaults(canvas, rng, vaults);
    pools(canvas, rng);
    let up = place_stairs(canvas, rng, needs)?;
    light(canvas, rng, up);
    patrols(canvas, rng, up, needs);
    special::cave_rooms(canvas, rng, up);
    Some(())
}

fn noise(canvas: &mut Canvas, rng: &mut Rng) {
    for p in canvas.cells() {
        if canvas.is_inner(p) && !rng.chance(ROCK) {
            canvas.set(p, canvas.pal.floor);
        }
    }
}

fn smooth(canvas: &mut Canvas) {
    let next: Vec<(Pos, bool)> = canvas
        .cells()
        .filter(|p| canvas.is_inner(*p))
        .map(|p| {
            let rock = Dir::ALL
                .iter()
                .filter(|d| canvas.is_rock(p.step(**d)))
                .count();
            (p, rock >= 5 || (canvas.is_rock(p) && rock >= 4))
        })
        .collect();
    for (p, rock) in next {
        let t = if rock {
            canvas.pal.void
        } else {
            canvas.pal.floor
        };
        canvas.set(p, t);
    }
}

/// Fills every cave but the largest; `None` if even that one is too small.
fn keep_largest_cave(canvas: &mut Canvas) -> Option<()> {
    let (w, h) = (canvas.width(), canvas.height());
    let floor = canvas.pal.floor;
    let mut claimed = Grid::new(w, h, false);
    let mut best: Option<(usize, Grid<bool>)> = None;
    for p in canvas.cells() {
        if claimed[p] || canvas.get(p) != floor {
            continue;
        }
        let region = flood(w, h, p, |q| canvas.get(q) == floor);
        let size = region.cells().iter().filter(|r| **r).count();
        for (q, r) in region.iter() {
            if *r {
                claimed[q] = true;
            }
        }
        if best.as_ref().is_none_or(|(s, _)| size > *s) {
            best = Some((size, region));
        }
    }
    let (size, keep) = best?;
    if (size as i32) * 100 < (w - 2) * (h - 2) * MIN_CAVE {
        return None;
    }
    for p in canvas.cells() {
        if canvas.get(p) == floor && !keep[p] {
            canvas.set(p, canvas.pal.void);
        }
    }
    Some(())
}

/// Stamps up to `count` vaults into solid rock, each joined to the cave by a tunnel from every
/// entrance.
fn place_vaults(canvas: &mut Canvas, rng: &mut Rng, count: usize) {
    let mut placed = 0;
    for v in vault::shuffled(canvas, rng) {
        if placed == count {
            break;
        }
        let (max_x, max_y) = (
            canvas.width() - 2 - v.width(),
            canvas.height() - 2 - v.height(),
        );
        let spots: Vec<Pos> = (2..=max_y)
            .flat_map(|y| (2..=max_x).map(move |x| Pos::new(x, y)))
            .filter(|at| vault::fits(canvas, v, *at))
            .collect();
        for _ in 0..8 {
            let Some(&at) = rng.pick(&spots) else { break };
            let inside = |p: Pos| {
                p.x >= at.x && p.y >= at.y && p.x < at.x + v.width() && p.y < at.y + v.height()
            };
            let tunnels: Option<Vec<Vec<Pos>>> = v
                .entrances
                .iter()
                .map(|(door, out)| {
                    let start = Pos::new(at.x + door.x, at.y + door.y).step(*out);
                    tunnel(canvas, start, inside)
                })
                .collect();
            let Some(tunnels) = tunnels else { continue };
            vault::stamp(canvas, v, at);
            for p in tunnels.into_iter().flatten() {
                if canvas.is_rock(p) {
                    canvas.set(p, canvas.pal.floor);
                }
            }
            placed += 1;
            break;
        }
    }
}

/// The cells from `start` through rock to the nearest open cave cell (at most `MAX_TUNNEL` long),
/// never entering cells `avoid` accepts or stamped ones.
fn tunnel(canvas: &Canvas, start: Pos, avoid: impl Fn(Pos) -> bool) -> Option<Vec<Pos>> {
    let open = |p: Pos| canvas.is_walkable(p) && !canvas.is_stamped(p);
    let cost = |p: Pos| {
        if !canvas.is_inner(p) || canvas.is_stamped(p) || avoid(p) {
            None
        } else if canvas.is_rock(p) {
            Some(2)
        } else {
            open(p).then_some(1)
        }
    };
    let path = cheapest_path(
        canvas.width(),
        canvas.height(),
        &[(start, 0)],
        cost,
        open,
        1,
    )?;
    (path.len() <= MAX_TUNNEL).then_some(path)
}

/// One to three pools of shallow water grown from random floor cells.
fn pools(canvas: &mut Canvas, rng: &mut Rng) {
    let (w, h) = (canvas.width(), canvas.height());
    let count = rng.range(1, 3) * ((w * h) / 7200).max(1);
    for _ in 0..count {
        let seed = (0..30)
            .map(|_| Pos::new(rng.range(1, w - 2), rng.range(1, h - 2)))
            .find(|p| canvas.is_free_floor(*p));
        let Some(seed) = seed else { continue };
        let size = rng.range(5, 18);
        let mut frontier = vec![seed];
        let mut filled = 0;
        while filled < size && !frontier.is_empty() {
            let p = frontier.swap_remove(rng.index(frontier.len()));
            if !canvas.is_free_floor(p) {
                continue;
            }
            canvas.set(p, canvas.pal.water);
            filled += 1;
            frontier.extend(super::CARDINALS.map(|d| p.step(d)));
        }
    }
}

/// The up staircase in a random roomy spot; the down staircase (if needed) in a roomy spot among
/// the farthest by walking distance. Returns the up staircase.
fn place_stairs(canvas: &mut Canvas, rng: &mut Rng, needs: Needs) -> Option<Pos> {
    let roomy: Vec<Pos> = canvas.cells().filter(|p| canvas.is_open_area(*p)).collect();
    let up = *rng.pick(&roomy)?;
    canvas.place_stairs(up, StairsDir::Up);
    if !needs.down {
        return Some(up);
    }
    let dist = distances(canvas.width(), canvas.height(), up, |p| {
        canvas.is_passable(p)
    });
    let roomy: Vec<(Pos, u32)> = canvas
        .cells()
        .filter(|p| canvas.is_open_area(*p) && dist[*p] != u32::MAX)
        .map(|p| (p, dist[p]))
        .collect();
    let far = roomy.iter().map(|(_, d)| *d).max()?;
    let candidates: Vec<Pos> = roomy
        .into_iter()
        .filter(|(_, d)| u64::from(*d) * 10 >= u64::from(far) * 7)
        .map(|(p, _)| p)
        .collect();
    let down = *rng.pick(&candidates)?;
    canvas.place_stairs(down, StairsDir::Down);
    Some(up)
}

fn light(canvas: &mut Canvas, rng: &mut Rng, up: Pos) {
    let lamp = canvas.pal.lamp;
    let mut beside: Vec<Pos> = Dir::ALL.iter().map(|d| up.step(*d)).collect();
    rng.shuffle(&mut beside);
    if let Some(p) = beside.into_iter().find(|p| canvas.is_free_floor(*p)) {
        canvas.lights.insert(p, lamp);
    }
    let (w, h) = (canvas.width(), canvas.height());
    for by in (0..h).step_by(LAMP_BLOCK as usize) {
        for bx in (0..w).step_by(LAMP_BLOCK as usize) {
            if !rng.chance(50) {
                continue;
            }
            for _ in 0..8 {
                let p = Pos::new(
                    (bx + rng.below(LAMP_BLOCK as u32) as i32).min(w - 1),
                    (by + rng.below(LAMP_BLOCK as u32) as i32).min(h - 1),
                );
                if canvas.is_free_floor(p) {
                    canvas.lights.insert(p, lamp);
                    break;
                }
            }
        }
    }
    let fungus = canvas.pal.fungus;
    for _ in 0..(w * h) / 600 {
        let p = Pos::new(rng.range(1, w - 2), rng.range(1, h - 2));
        if !canvas.is_free_floor(p) {
            continue;
        }
        canvas.set(p, fungus);
        for _ in 0..rng.below(4) {
            let q = p.step(Dir::ALL[rng.index(Dir::ALL.len())]);
            if canvas.is_free_floor(q) {
                canvas.set(q, fungus);
            }
        }
    }
}

/// Two to four creatures, away from the arrival point, walking out and back.
fn patrols(canvas: &mut Canvas, rng: &mut Rng, up: Pos, needs: Needs) {
    if needs.creatures.is_empty() {
        return;
    }
    let (w, h) = (canvas.width(), canvas.height());
    let dist = distances(w, h, up, |p| canvas.is_passable(p));
    let want = rng.range(2, 4);
    let mut placed = 0;
    for _ in 0..60 {
        if placed == want {
            break;
        }
        let start = Pos::new(rng.range(1, w - 2), rng.range(1, h - 2));
        if !canvas.is_free_floor(start) || dist[start] == u32::MAX || dist[start] < 12 {
            continue;
        }
        let len = rng.range(4, 10);
        let Some(route) = out_and_back(rng, start, len, |p| canvas.is_walkable(p)) else {
            continue;
        };
        let Some(&creature) = rng.pick(needs.creatures) else {
            return;
        };
        if canvas.add_patrol(creature, start, route) {
            placed += 1;
        }
    }
}
