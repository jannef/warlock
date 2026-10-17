//! Symmetric shadowcasting (Albert Ford, <https://www.albertford.com/shadowcasting/>).
//!
//! Properties relied on by the rest of the game:
//! - **Symmetry**: if transparent cell A sees transparent cell B, then B sees A.
//! - **Expansive walls**: opaque cells bordering visible space are revealed.
//! - **Determinism**: all slope math is exact integer arithmetic.
//!
//! The same routine computes player sight and light shadows.

use crate::geom::Pos;

/// Calls `reveal` for every cell visible from `origin` within `radius` (Euclidean).
///
/// `is_opaque` must return `true` for out-of-bounds positions. `reveal` may be called more than
/// once for the same cell and may receive out-of-bounds positions (treated as opaque walls), which
/// the caller should ignore.
pub fn compute_fov(
    origin: Pos,
    radius: i32,
    is_opaque: impl Fn(Pos) -> bool,
    mut reveal: impl FnMut(Pos),
) {
    reveal(origin);
    if radius <= 0 {
        return;
    }
    // Slightly rounder circles than a strict r² cutoff.
    let radius2 = radius * radius + radius;
    let in_range = |depth: i32, col: i32| depth * depth + col * col <= radius2;

    for quadrant in [
        Quadrant::North,
        Quadrant::East,
        Quadrant::South,
        Quadrant::West,
    ] {
        let to_pos = |depth: i32, col: i32| quadrant.transform(origin, depth, col);
        let mut stack = vec![Row {
            depth: 1,
            start: Slope::new(-1, 1),
            end: Slope::new(1, 1),
        }];
        while let Some(mut row) = stack.pop() {
            if row.depth > radius {
                continue;
            }
            // `None` before the first tile, then whether the previous tile was opaque.
            let mut prev_opaque: Option<bool> = None;
            for col in row.min_col()..=row.max_col() {
                let opaque = is_opaque(to_pos(row.depth, col));
                if (opaque || row.is_symmetric(col)) && in_range(row.depth, col) {
                    reveal(to_pos(row.depth, col));
                }
                if prev_opaque == Some(true) && !opaque {
                    row.start = Slope::tile_edge(row.depth, col);
                }
                if prev_opaque == Some(false) && opaque {
                    stack.push(Row {
                        depth: row.depth + 1,
                        start: row.start,
                        end: Slope::tile_edge(row.depth, col),
                    });
                }
                prev_opaque = Some(opaque);
            }
            if prev_opaque == Some(false) {
                stack.push(Row {
                    depth: row.depth + 1,
                    start: row.start,
                    end: row.end,
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Quadrant {
    North,
    East,
    South,
    West,
}

impl Quadrant {
    fn transform(self, origin: Pos, depth: i32, col: i32) -> Pos {
        match self {
            Quadrant::North => Pos::new(origin.x + col, origin.y - depth),
            Quadrant::South => Pos::new(origin.x + col, origin.y + depth),
            Quadrant::East => Pos::new(origin.x + depth, origin.y + col),
            Quadrant::West => Pos::new(origin.x - depth, origin.y + col),
        }
    }
}

/// An exact rational slope `num / den` with `den > 0`.
#[derive(Clone, Copy, Debug)]
struct Slope {
    num: i64,
    den: i64,
}

impl Slope {
    fn new(num: i64, den: i64) -> Self {
        debug_assert!(den > 0);
        Self { num, den }
    }

    /// Slope of the left edge of the tile at `(depth, col)`: `(2·col − 1) / (2·depth)`.
    fn tile_edge(depth: i32, col: i32) -> Self {
        Self::new(2 * i64::from(col) - 1, 2 * i64::from(depth))
    }
}

#[derive(Clone, Copy, Debug)]
struct Row {
    depth: i32,
    start: Slope,
    end: Slope,
}

impl Row {
    /// `round_ties_up(depth · start)` = ⌊depth·num/den + ½⌋.
    fn min_col(&self) -> i32 {
        let d = i64::from(self.depth);
        (2 * d * self.start.num + self.start.den).div_euclid(2 * self.start.den) as i32
    }

    /// `round_ties_down(depth · end)` = ⌈depth·num/den − ½⌉.
    fn max_col(&self) -> i32 {
        let d = i64::from(self.depth);
        -((self.end.den - 2 * d * self.end.num).div_euclid(2 * self.end.den)) as i32
    }

    /// `depth·start ≤ col ≤ depth·end`, exactly.
    fn is_symmetric(&self, col: i32) -> bool {
        let d = i64::from(self.depth);
        let c = i64::from(col);
        c * self.start.den >= d * self.start.num && c * self.end.den <= d * self.end.num
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Parses an ASCII map: `#` opaque, anything else transparent.
    fn parse(map: &str) -> (Vec<Vec<bool>>, Pos) {
        let mut origin = Pos::new(0, 0);
        let rows = map
            .lines()
            .enumerate()
            .map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(|(x, c)| {
                        if c == '@' {
                            origin = Pos::new(x as i32, y as i32);
                        }
                        c == '#'
                    })
                    .collect()
            })
            .collect();
        (rows, origin)
    }

    fn opaque_fn(rows: &[Vec<bool>]) -> impl Fn(Pos) -> bool + '_ {
        move |p: Pos| {
            p.y < 0
                || p.x < 0
                || rows
                    .get(p.y as usize)
                    .and_then(|r| r.get(p.x as usize))
                    .copied()
                    .unwrap_or(true)
        }
    }

    fn visible(rows: &[Vec<bool>], origin: Pos, radius: i32) -> BTreeSet<Pos> {
        let mut seen = BTreeSet::new();
        compute_fov(origin, radius, opaque_fn(rows), |p| {
            seen.insert(p);
        });
        seen
    }

    #[test]
    fn open_room_is_fully_visible() {
        let (rows, origin) = parse("#####\n#...#\n#.@.#\n#...#\n#####");
        let seen = visible(&rows, origin, 10);
        for y in 0..5 {
            for x in 0..5 {
                assert!(
                    seen.contains(&Pos::new(x, y)),
                    "({x},{y}) should be visible"
                );
            }
        }
    }

    #[test]
    fn walls_block_sight() {
        let (rows, origin) = parse("#######\n#@.#..#\n#######");
        let seen = visible(&rows, origin, 10);
        assert!(seen.contains(&Pos::new(3, 1)), "the wall itself is visible");
        assert!(
            !seen.contains(&Pos::new(4, 1)),
            "cells behind the wall are hidden"
        );
    }

    #[test]
    fn radius_limits_sight() {
        let row = format!("@{}", ".".repeat(20));
        let (rows, origin) = parse(&row);
        let seen = visible(&rows, origin, 5);
        assert!(seen.contains(&Pos::new(5, 0)));
        assert!(!seen.contains(&Pos::new(6, 0)));
    }

    #[test]
    fn pillar_casts_shadow() {
        let (rows, origin) = parse(".......\n.......\n...#...\n.......\n...@...");
        let seen = visible(&rows, origin, 10);
        assert!(seen.contains(&Pos::new(3, 2)), "pillar visible");
        assert!(
            !seen.contains(&Pos::new(3, 0)),
            "directly behind pillar is in shadow"
        );
        assert!(
            !seen.contains(&Pos::new(3, 1)),
            "directly behind pillar is in shadow"
        );
    }

    mod props {
        use super::*;
        use proptest::prelude::*;

        fn arb_map() -> impl Strategy<Value = Vec<Vec<bool>>> {
            prop::collection::vec(prop::collection::vec(prop::bool::weighted(0.3), 12), 12)
        }

        proptest! {
            #[test]
            fn sight_is_symmetric_between_transparent_cells(
                map in arb_map(),
                ax in 0i32..12, ay in 0i32..12, bx in 0i32..12, by in 0i32..12,
            ) {
                let a = Pos::new(ax, ay);
                let b = Pos::new(bx, by);
                let opaque = opaque_fn(&map);
                prop_assume!(!opaque(a) && !opaque(b));
                let a_sees_b = visible(&map, a, 30).contains(&b);
                let b_sees_a = visible(&map, b, 30).contains(&a);
                prop_assert_eq!(a_sees_b, b_sees_a);
            }

            #[test]
            fn enclosed_cells_are_never_seen(map in arb_map(), ox in 1i32..11, oy in 1i32..11) {
                // Wall in a ring around the origin's 3x3 neighborhood; nothing beyond is visible.
                let mut map = map;
                let origin = Pos::new(ox, oy);
                for y in 0..12 {
                    for x in 0..12 {
                        let d = Pos::new(x, y).chebyshev(origin);
                        if d == 2 { map[y as usize][x as usize] = true; }
                        if d < 2 { map[y as usize][x as usize] = false; }
                    }
                }
                for p in visible(&map, origin, 30) {
                    prop_assert!(p.chebyshev(origin) <= 2, "saw {p} through the ring");
                }
            }
        }
    }
}
