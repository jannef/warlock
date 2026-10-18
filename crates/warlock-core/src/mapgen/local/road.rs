//! The road through a local area of a road cell: from a junction near the middle out to the side
//! (or corner) of every neighboring road or location on the overworld, two cells wide, in gentle
//! bends. Where it crosses water it becomes a bridge.

use super::land::{Claim, Land};
use super::water::{course, line};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;

/// Lays the road towards `ways` (directions on the overworld). Returns its junction.
pub(super) fn lay(land: &mut Land, rng: &mut Rng, ways: &[Dir]) -> Option<Pos> {
    if ways.is_empty() {
        return None;
    }
    let (w, h) = (land.width(), land.height());
    let c = land.center();
    let junction = Pos::new(c.x + rng.range(-4, 4), c.y + rng.range(-2, 2));
    for &way in ways {
        let end = match way {
            Dir::N => Pos::new(junction.x + rng.range(-6, 6), 0),
            Dir::S => Pos::new(junction.x + rng.range(-6, 6), h - 1),
            Dir::W => Pos::new(0, junction.y + rng.range(-3, 3)),
            Dir::E => Pos::new(w - 1, junction.y + rng.range(-3, 3)),
            Dir::NE => Pos::new(w - 2, 0),
            Dir::SE => Pos::new(w - 2, h - 1),
            Dir::SW => Pos::new(0, h - 1),
            Dir::NW => Pos::new(0, 0),
        };
        let points = course(rng, junction, end, 1, 5);
        for pair in points.windows(2) {
            let (dx, dy) = (pair[1].x - pair[0].x, pair[1].y - pair[0].y);
            // Two cells wide: the second beside the first, across the way it runs.
            let side = if dx.abs() >= 2 * dy.abs() {
                (0, 1)
            } else {
                (1, 0)
            };
            for p in line(pair[0], pair[1]) {
                for q in [p, Pos::new(p.x + side.0, p.y + side.1)] {
                    pave(land, q);
                }
            }
        }
    }
    Some(junction)
}

fn pave(land: &mut Land, p: Pos) {
    if !land.in_bounds(p) {
        return;
    }
    let surface = if land.is_water(p) {
        land.kit.bridge
    } else {
        land.kit.road
    };
    land.set(p, surface);
    land.set_claim(p, Claim::Fixed);
    // Nothing grows right beside the road.
    for d in Dir::ALL {
        land.claim_at_least(p.step(d), Claim::Clear);
    }
}
