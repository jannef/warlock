//! Water in local areas: a river or a lake shore along the sides that border one on the
//! overworld, ponds, and brooks. Running water gets a flow on every cell; along a stretch of
//! river or brook it runs one way at one speed, turning only where the water bends.

use super::Setting;
use super::land::{Claim, Land};
use super::noise;
use crate::content::Flow;
use crate::geom::{Dir, Pos};
use crate::rng::Rng;

/// Flow speed of rivers and of brooks, in percent (brooks are narrower and quicker).
const RIVER_SPEED: u16 = 100;
const BROOK_SPEED: u16 = 150;

/// How far `p` lies from side `side` of the map, in columns (a row counts as two: cells are
/// about twice as tall as wide); corners measure from the corner across the diagonal.
fn from_side(land: &Land, p: Pos, side: Dir) -> i32 {
    let (w, h) = (land.width() - 1, land.height() - 1);
    let (dx, dy) = side.delta();
    let across_x = match dx {
        -1 => p.x,
        1 => w - p.x,
        _ => 0,
    };
    let across_y = match dy {
        -1 => 2 * p.y,
        1 => 2 * (h - p.y),
        _ => 0,
    };
    if dx != 0 && dy != 0 {
        // Distance to the corner's diagonal, scaled by 1/√2 ≈ 7/10.
        (across_x + across_y) * 7 / 10
    } else {
        across_x + across_y
    }
}

/// The two ways water can run along side `side`: parallel to it.
fn along(side: Dir) -> [Dir; 2] {
    match side {
        Dir::N | Dir::S => [Dir::E, Dir::W],
        Dir::E | Dir::W => [Dir::S, Dir::N],
        // The NW and SE corners' diagonal runs from bottom-left to top-right, NE and SW's from
        // top-left to bottom-right.
        Dir::NW | Dir::SE => [Dir::NE, Dir::SW],
        Dir::NE | Dir::SW => [Dir::SE, Dir::NW],
    }
}

fn dot(a: Dir, b: (i32, i32)) -> i32 {
    let (ax, ay) = a.delta();
    ax * b.0 + ay * b.1
}

/// Which way a river along `side` runs: the way along the side closest to its own flow on the
/// overworld, then to all the neighboring river's flows.
fn river_way(side: Dir, flow: Dir, all: (i32, i32)) -> Dir {
    let [a, b] = along(side);
    let key = |d: Dir| (dot(d, flow.delta()), dot(d, all));
    if key(b) > key(a) { b } else { a }
}

/// Rivers and lake shores along the sides the overworld puts them on. Banks are kept clear.
pub(super) fn sides(land: &mut Land, setting: &Setting, rng: &mut Rng) {
    if setting.rivers.is_empty() && setting.lakes.is_empty() {
        return;
    }
    let (w, h) = (land.width(), land.height());
    let meander = noise::field(rng, w, h, &[(8, 2), (3, 1)]);
    let river_width = rng.range(11, 15);
    let lake_width = rng.range(16, 22);
    let all = setting.rivers.iter().fold((0, 0), |(x, y), (_, f)| {
        let (dx, dy) = f.delta();
        (x + dx, y + dy)
    });
    for p in land.cells() {
        // Banks wander up to 8 columns in or out; the water never leaves its side.
        let bend = ((meander[p] - noise::MAX / 2) * 30 / noise::MAX).clamp(-8, 8);
        // How deep into the water `p` lies (positive: in it), and the flow if running.
        let mut best: Option<(i32, Option<Dir>)> = None;
        for &(side, flow) in &setting.rivers {
            let depth = river_width + bend - from_side(land, p, side);
            if best.is_none_or(|(d, _)| depth > d) {
                best = Some((depth, Some(river_way(side, flow, all))));
            }
        }
        for &side in &setting.lakes {
            let depth = lake_width + bend - from_side(land, p, side);
            if best.is_none_or(|(d, _)| depth > d) {
                best = Some((depth, None));
            }
        }
        let Some((depth, way)) = best else { continue };
        if depth > 0 {
            match way {
                Some(dir) => land.set_running(p, Flow::new(dir, RIVER_SPEED)),
                None => land.set(p, land.kit.water),
            }
            land.set_claim(p, Claim::Fixed);
        } else if depth > -3 {
            land.claim_at_least(p, Claim::Clear);
        }
    }
}

/// A pond of still water away from the middle of the area, if one fits on open land.
pub(super) fn pond(land: &mut Land, rng: &mut Rng, size: (i32, i32)) {
    let (w, h) = (land.width(), land.height());
    let center = land.center();
    for _ in 0..30 {
        let rx = rng.range(size.0, size.1);
        let ry = (rx / 2).max(2);
        let c = Pos::new(rng.range(rx + 3, w - rx - 4), rng.range(ry + 3, h - ry - 4));
        let (dx, dy) = ((c.x - center.x).abs(), 2 * (c.y - center.y).abs());
        if dx.max(dy) < rx + 14 {
            continue;
        }
        let cells = land.blob(rng, c, rx, ry, 250);
        let margin = |p: Pos| {
            Dir::ALL
                .iter()
                .all(|d| land.claim(p.step(*d)) == Claim::Free)
        };
        if cells.is_empty() || !cells.iter().all(|p| margin(*p) && !land.on_edge(*p)) {
            continue;
        }
        for p in &cells {
            land.set(*p, land.kit.water);
            land.set_claim(*p, Claim::Fixed);
        }
        return;
    }
}

/// The 8-way direction closest to the vector `(dx, dy)` (in cells).
fn heading(dx: i32, dy: i32) -> Dir {
    let (ax, ay) = (dx.abs(), dy.abs());
    // tan(22.5°) ≈ 5/12.
    let sx = if ax * 12 > ay * 5 { dx.signum() } else { 0 };
    let sy = if ay * 12 > ax * 5 { dy.signum() } else { 0 };
    Dir::from_delta(sx, sy).unwrap_or(Dir::S)
}

/// Cells of the line from `a` to `b` (8-connected, both ends included).
pub(super) fn line(a: Pos, b: Pos) -> Vec<Pos> {
    let (dx, dy) = ((b.x - a.x).abs(), -(b.y - a.y).abs());
    let (sx, sy) = ((b.x - a.x).signum(), (b.y - a.y).signum());
    let mut err = dx + dy;
    let mut p = a;
    let mut out = vec![p];
    while p != b {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            p.x += sx;
        }
        if e2 <= dx {
            err += dx;
            p.y += sy;
        }
        out.push(p);
    }
    out
}

/// Cells of the line from `a` to `b` with no diagonal steps (each one goes across first, then
/// along), so the cells are 4-connected.
pub(super) fn line4(a: Pos, b: Pos) -> Vec<Pos> {
    let mut out: Vec<Pos> = Vec::new();
    for p in line(a, b) {
        if let Some(&last) = out.last()
            && last.x != p.x
            && last.y != p.y
        {
            out.push(Pos::new(p.x, last.y));
        }
        out.push(p);
    }
    out
}

/// A winding course from `from` to `to`: `bends` points between them, each pushed sideways by up
/// to `sway` (columns across a vertical course, half as many rows across a horizontal one).
pub(super) fn course(rng: &mut Rng, from: Pos, to: Pos, bends: i32, sway: i32) -> Vec<Pos> {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let horizontal = dx.abs() >= 2 * dy.abs();
    let mut points = vec![from];
    for i in 1..=bends {
        let base = Pos::new(from.x + dx * i / (bends + 1), from.y + dy * i / (bends + 1));
        let push = rng.range(-sway, sway);
        points.push(if horizontal {
            Pos::new(base.x, base.y + push / 2)
        } else {
            Pos::new(base.x + push, base.y)
        });
    }
    points.push(to);
    points
}

/// A point on side `side` (N, E, S, or W) of the map, `margin` cells from its corners.
pub(super) fn point_on(land: &Land, rng: &mut Rng, side: Dir, margin: i32) -> Pos {
    let (w, h) = (land.width(), land.height());
    match side {
        Dir::N => Pos::new(rng.range(margin, w - 1 - margin), 0),
        Dir::S => Pos::new(rng.range(margin, w - 1 - margin), h - 1),
        Dir::W => Pos::new(0, rng.range(margin / 2, h - 1 - margin / 2)),
        _ => Pos::new(w - 1, rng.range(margin / 2, h - 1 - margin / 2)),
    }
}

/// A brook running across the area from one side to another, one or two cells wide.
pub(super) fn brook(land: &mut Land, rng: &mut Rng) {
    let sides = [Dir::N, Dir::E, Dir::S, Dir::W];
    let from = sides[rng.index(4)];
    // Mostly across to the far side, sometimes round a corner.
    let to = if rng.chance(70) {
        from.opposite()
    } else {
        let i = sides.iter().position(|d| *d == from).unwrap_or(0);
        sides[(i + if rng.chance(50) { 1 } else { 3 }) % 4]
    };
    let a = point_on(land, rng, from, 8);
    let b = point_on(land, rng, to, 8);
    let points = course(rng, a, b, 4, 10);
    for pair in points.windows(2) {
        let dir = heading(pair[1].x - pair[0].x, pair[1].y - pair[0].y);
        let flow = Flow::new(dir, BROOK_SPEED);
        // Two cells wide, the second beside the first across the way it runs, so the steps of
        // a slanting stretch overlap.
        let beside = if dir.delta().1 == 0 { (0, 1) } else { (1, 0) };
        for p in line(pair[0], pair[1]) {
            for q in [p, Pos::new(p.x + beside.0, p.y + beside.1)] {
                if land.in_bounds(q) && land.claim(q) != Claim::Fixed {
                    land.set_running(q, flow);
                    land.set_claim(q, Claim::Fixed);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_8_connected_and_end_to_end() {
        for (a, b) in [
            (Pos::new(0, 0), Pos::new(9, 3)),
            (Pos::new(5, 9), Pos::new(2, 0)),
            (Pos::new(3, 3), Pos::new(3, 3)),
        ] {
            let cells = line(a, b);
            assert_eq!(cells.first(), Some(&a));
            assert_eq!(cells.last(), Some(&b));
            assert!(cells.windows(2).all(|w| w[0].chebyshev(w[1]) == 1));
        }
    }

    #[test]
    fn trails_lines_take_no_diagonal_steps() {
        let cells = line4(Pos::new(2, 1), Pos::new(9, -6));
        assert_eq!(cells.first(), Some(&Pos::new(2, 1)));
        assert_eq!(cells.last(), Some(&Pos::new(9, -6)));
        assert!(cells.windows(2).all(|w| w[0].dist2(w[1]) == 1));
    }

    #[test]
    fn headings_round_to_the_nearest_of_eight() {
        assert_eq!(heading(10, 1), Dir::E);
        assert_eq!(heading(10, 8), Dir::SE);
        assert_eq!(heading(-1, -10), Dir::N);
        assert_eq!(heading(-6, 5), Dir::SW);
    }

    #[test]
    fn rivers_run_along_their_side_and_downstream() {
        assert_eq!(river_way(Dir::W, Dir::S, (0, 1)), Dir::S);
        assert_eq!(river_way(Dir::E, Dir::SE, (1, 1)), Dir::S);
        assert_eq!(river_way(Dir::N, Dir::E, (1, 0)), Dir::E);
        // Flowing straight at the side: the other river cells decide.
        assert_eq!(river_way(Dir::N, Dir::S, (1, 2)), Dir::E);
        assert_eq!(river_way(Dir::SW, Dir::SE, (1, 1)), Dir::SE);
    }
}
