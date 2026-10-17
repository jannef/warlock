//! Bounce light (spec §5, "Bounce light"): one weak bounce of source light off the surfaces it
//! falls on, as gameplay light.
//!
//! 1. **Direct light**: the sum of every source's light (its cached [`Mask`]) on each transparent
//!    cell of an area, red, green, and blue only — UV light does not bounce.
//! 2. **Reflection**: each lit transparent cell reflects its direct light, tinted by the mean
//!    `bounce` color of the surfaces there: its own terrain (the floor) and the wall faces that
//!    bound it (its opaque orthogonal neighbors, lit from this side).
//! 3. **Spread**: [`BOUNCE_STEPS`] times, each transparent cell takes the mean of itself and its
//!    linked neighbors: transparent orthogonal neighbors, and transparent diagonal ones with a
//!    transparent orthogonal cell between (light never squeezes through a wall corner). Opaque
//!    cells neither take nor pass on light, so bounce light never crosses a wall, and it travels at
//!    most `BOUNCE_STEPS` cells from where it was reflected, fading with distance.
//! 4. **Strength**: the indirect light is [`BOUNCE`]`/256` of the spread reflected light. Every
//!    step is a mean, so it is never more than that fraction of the brightest reflected light
//!    within reach, and in an evenly lit area it is exactly that fraction.
//!
//! Integer math only (fixed point: reflected light is kept in units of 1/255), with a fixed order,
//! so it is deterministic. Indirect light is a property of the level and its lights, not of the
//! viewer: the area only has to cover every cell within `BOUNCE_STEPS` of the cells asked about.

use super::{Light, Mask};
use crate::geom::{Dir, Pos};

/// Indirect light is this many 256ths of the reflected light around a cell (12.5 %).
pub const BOUNCE: u32 = 32;

/// How far bounce light spreads from the surface that reflected it, in cells (one step per cell,
/// diagonals included).
pub const BOUNCE_STEPS: i32 = 3;

/// The orthogonal directions, in [`Dir::ALL`] order.
const ORTHOGONAL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

/// A rectangle of cells, corners included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Area {
    pub min: Pos,
    pub max: Pos,
}

impl Area {
    /// The smallest area holding every cell of `cells` and `margin` more cells on each side,
    /// clipped to `width × height`; `None` if `cells` is empty.
    pub fn around(
        cells: impl IntoIterator<Item = Pos>,
        margin: i32,
        width: i32,
        height: i32,
    ) -> Option<Self> {
        let mut cells = cells.into_iter();
        let first = cells.next()?;
        let (min, max) = cells.fold((first, first), |(lo, hi), p| {
            (
                Pos::new(lo.x.min(p.x), lo.y.min(p.y)),
                Pos::new(hi.x.max(p.x), hi.y.max(p.y)),
            )
        });
        let min = Pos::new((min.x - margin).max(0), (min.y - margin).max(0));
        let max = Pos::new(
            (max.x + margin).min(width - 1),
            (max.y + margin).min(height - 1),
        );
        (min.x <= max.x && min.y <= max.y).then_some(Self { min, max })
    }

    fn width(self) -> i32 {
        self.max.x - self.min.x + 1
    }

    fn len(self) -> usize {
        (self.width() * (self.max.y - self.min.y + 1)) as usize
    }

    fn index(self, p: Pos) -> Option<usize> {
        (p.x >= self.min.x && p.y >= self.min.y && p.x <= self.max.x && p.y <= self.max.y)
            .then(|| ((p.y - self.min.y) * self.width() + (p.x - self.min.x)) as usize)
    }

    fn pos(self, i: usize) -> Pos {
        let i = i as i32;
        Pos::new(self.min.x + i % self.width(), self.min.y + i / self.width())
    }
}

/// Indirect light on the transparent cells of an area (dark on opaque cells and outside it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Bounce {
    area: Area,
    cells: Vec<Light>,
}

impl Bounce {
    /// Bounces the light of `masks` within `area`. `is_opaque` says which cells block light (out
    /// of bounds counts as opaque); `tint` gives the `bounce` color of the surface at a cell, each
    /// channel 0 to 255. Correct on cells at least [`BOUNCE_STEPS`] inside the area's edges (or
    /// at the level's edges): the area must cover everything within reach of the cells asked about.
    pub fn compute<'a>(
        area: Area,
        masks: impl IntoIterator<Item = &'a Mask>,
        is_opaque: impl Fn(Pos) -> bool,
        tint: impl Fn(Pos) -> [u8; 3],
    ) -> Self {
        // Work on the area plus a closed one-cell border, so neighbors never fall outside.
        let padded = Area {
            min: Pos::new(area.min.x - 1, area.min.y - 1),
            max: Pos::new(area.max.x + 1, area.max.y + 1),
        };
        let (n, width) = (padded.len(), padded.width() as usize);
        let inner = |i: usize| {
            let (x, y) = (i % width, i / width);
            x > 0 && y > 0 && x < width - 1 && i < n - width
        };
        let open: Vec<bool> = (0..n)
            .map(|i| inner(i) && !is_opaque(padded.pos(i)))
            .collect();

        // Direct light (red, green, blue) on transparent cells, saturating like gameplay light.
        let mut direct = vec![[0u32; 3]; n];
        for mask in masks.into_iter().filter(|m| m.overlaps(area.min, area.max)) {
            for &(p, light) in &mask.cells {
                if let Some(i) = area.index(p).and_then(|_| padded.index(p)) {
                    for (sum, c) in direct[i].iter_mut().zip(light.rgb) {
                        *sum = (*sum + u32::from(c)).min(u32::from(u16::MAX));
                    }
                }
            }
        }

        // Reflected light, in 1/255ths: direct light times the mean tint of the floor and the wall
        // faces bounding the cell.
        let mut light = vec![[0u32; 3]; n];
        let mut lit = false;
        for i in (0..n).filter(|&i| open[i] && direct[i] != [0; 3]) {
            let p = padded.pos(i);
            let surfaces = std::iter::once(p).chain(
                ORTHOGONAL
                    .iter()
                    .map(|d| p.step(*d))
                    .filter(|q| is_opaque(*q)),
            );
            let (mut tints, mut count) = ([0u32; 3], 0u32);
            for q in surfaces {
                for (sum, c) in tints.iter_mut().zip(tint(q)) {
                    *sum += u32::from(c);
                }
                count += 1;
            }
            light[i] = std::array::from_fn(|c| direct[i][c] * tints[c] / count);
            lit |= light[i] != [0; 3];
        }
        if !lit {
            let cells = vec![Light::DARK; area.len()];
            return Self { area, cells };
        }

        // Spread: each step, every transparent cell takes the mean of itself and its linked
        // neighbors (bit `k` of `links` for direction `Dir::ALL[k]`).
        let offset = |(dx, dy): (i32, i32)| dy as isize * width as isize + dx as isize;
        let offsets = Dir::ALL.map(|d| offset(d.delta()));
        let links: Vec<u8> = (0..n)
            .map(|i| {
                if !open[i] {
                    return 0;
                }
                let at = |o: isize| open[(i as isize + o) as usize];
                Dir::ALL.iter().enumerate().fold(0, |links, (k, d)| {
                    let (dx, dy) = d.delta();
                    let linked = at(offsets[k])
                        && (dx == 0 || dy == 0 || at(offset((dx, 0))) || at(offset((0, dy))));
                    if linked { links | 1 << k } else { links }
                })
            })
            .collect();
        let mut next = vec![[0u32; 3]; n];
        for _ in 0..BOUNCE_STEPS {
            for i in (0..n).filter(|&i| open[i]) {
                let (mut sum, mut count) = (light[i], 1);
                for (k, offset) in offsets.iter().enumerate() {
                    if links[i] & 1 << k != 0 {
                        let j = (i as isize + offset) as usize;
                        for (s, c) in sum.iter_mut().zip(light[j]) {
                            *s += c;
                        }
                        count += 1;
                    }
                }
                next[i] = sum.map(|s| s / count);
            }
            std::mem::swap(&mut light, &mut next);
        }

        let cells = (0..area.len())
            .map(|i| {
                let rgb = padded.index(area.pos(i)).map_or([0; 3], |j| light[j]);
                let [r, g, b] = rgb.map(|c| {
                    u16::try_from(u64::from(c) * u64::from(BOUNCE) / (255 * 256))
                        .unwrap_or(u16::MAX)
                });
                Light::new(r, g, b, 0)
            })
            .collect();
        Self { area, cells }
    }

    /// Indirect light at `p`: dark on opaque cells and outside the area.
    pub fn at(&self, p: Pos) -> Light {
        self.area.index(p).map_or(Light::DARK, |i| self.cells[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{LightId, LightPreset};
    use proptest::prelude::*;

    const SIZE: i32 = 14;
    const STONE: [u8; 3] = [255; 3];

    fn preset(color: Light, radius: i32) -> LightPreset {
        LightPreset {
            id: "test".into(),
            color,
            radius,
            stability: Default::default(),
        }
    }

    fn whole(width: i32, height: i32) -> Area {
        Area {
            min: Pos::new(0, 0),
            max: Pos::new(width - 1, height - 1),
        }
    }

    fn arb_map() -> impl Strategy<Value = Vec<Vec<bool>>> {
        let row = prop::collection::vec(prop::bool::weighted(0.3), SIZE as usize);
        prop::collection::vec(row, SIZE as usize)
    }

    fn opaque_fn(map: &[Vec<bool>]) -> impl Fn(Pos) -> bool + '_ {
        move |p: Pos| {
            !(0..SIZE).contains(&p.x)
                || !(0..SIZE).contains(&p.y)
                || map[p.y as usize][p.x as usize]
        }
    }

    /// Cells reachable from `from` in at most `steps` linked steps (spread rules above).
    fn reachable(from: Pos, steps: i32, opaque: impl Fn(Pos) -> bool) -> Vec<Pos> {
        let mut seen = vec![from];
        let mut frontier = vec![from];
        for _ in 0..steps {
            let mut next = Vec::new();
            for p in frontier {
                for d in Dir::ALL {
                    let q = p.step(d);
                    let (dx, dy) = d.delta();
                    let between = dx == 0
                        || dy == 0
                        || !opaque(Pos::new(p.x + dx, p.y))
                        || !opaque(Pos::new(p.x, p.y + dy));
                    if !opaque(q) && between && !seen.contains(&q) {
                        seen.push(q);
                        next.push(q);
                    }
                }
            }
            frontier = next;
        }
        seen
    }

    #[test]
    fn an_evenly_lit_area_gets_exactly_the_bounce_fraction() {
        // A radius-0 light on every cell of an open field (no walls in reach of the middle).
        let color = Light::new(256, 512, 1024, 300);
        let masks: Vec<Mask> = (0..9 * 9)
            .map(|i| Pos::new(i % 9, i / 9))
            .map(|p| Mask::compute(p, LightId(0), &preset(color, 0), |_| false))
            .collect();
        let bounce = Bounce::compute(whole(9, 9), &masks, |_| false, |_| STONE);
        let mid = bounce.at(Pos::new(4, 4));
        assert_eq!(
            mid,
            Light::new(32, 64, 128, 0),
            "12.5 %, and UV does not bounce"
        );
    }

    #[test]
    fn bounce_spreads_around_a_corner_and_fades() {
        // A lit room (x 0..4) opens onto a dark corridor (y 1) through a gap at x = 5.
        let map = [
            ".....######",
            ".....######",
            "...........",
            ".....######",
            ".....######",
        ];
        let opaque = |p: Pos| {
            map.get(p.y as usize)
                .and_then(|row| row.as_bytes().get(p.x as usize))
                .is_none_or(|c| *c == b'#')
        };
        let masks: Vec<Mask> = (0..5 * 5)
            .map(|i| Pos::new(i % 5, i / 5))
            .map(|p| {
                Mask::compute(
                    p,
                    LightId(0),
                    &preset(Light::new(255, 255, 255, 0), 0),
                    opaque,
                )
            })
            .collect();
        let bounce = Bounce::compute(whole(11, 5), &masks, opaque, |_| STONE);
        let at = |x: i32| bounce.at(Pos::new(x, 2)).rgb[0];
        assert!(
            at(5) > at(6) && at(6) > at(7) && at(7) > 0,
            "fades along the corridor"
        );
        assert_eq!(at(8), 0, "reaches three cells past the last lit one");
        assert_eq!(
            bounce.at(Pos::new(6, 1)),
            Light::DARK,
            "walls take no bounce light"
        );
    }

    #[test]
    fn tint_follows_the_surfaces() {
        // White light in a small room: floor and walls in turn tinted.
        let map = ["#####", "#...#", "#...#", "#...#", "#####"];
        let opaque = |p: Pos| {
            map.get(p.y as usize)
                .and_then(|row| row.as_bytes().get(p.x as usize))
                .is_none_or(|c| *c == b'#')
        };
        let white = preset(Light::new(255, 255, 255, 0), 3);
        let mask = Mask::compute(Pos::new(2, 2), LightId(0), &white, opaque);
        let rgb = |floor: [u8; 3], wall: [u8; 3]| {
            let tint = |p: Pos| if opaque(p) { wall } else { floor };
            Bounce::compute(whole(5, 5), [&mask], opaque, tint)
                .at(Pos::new(2, 2))
                .rgb
        };
        let [r, g, b] = rgb([64, 255, 64], STONE);
        assert!(g > r && r == b, "a green floor: {r} {g} {b}");
        let [r, g, b] = rgb(STONE, [255, 64, 64]);
        assert!(r > g && g == b, "red walls: {r} {g} {b}");
        let [r, g, b] = rgb(STONE, STONE);
        assert!(
            r == g && g == b && r > 0,
            "neutral stone keeps white light white"
        );
    }

    #[test]
    fn areas_are_fitted_to_cells_and_clipped() {
        let cells = [Pos::new(5, 6), Pos::new(9, 2)];
        let area = Area::around(cells, 3, 11, 20).expect("cells");
        assert_eq!((area.min, area.max), (Pos::new(2, 0), Pos::new(10, 9)));
        assert_eq!(Area::around([], 3, 11, 20), None);
    }

    proptest! {
        #[test]
        fn bounce_never_crosses_walls_and_stays_a_small_fraction(
            map in arb_map(),
            lights in prop::collection::vec((0i32..SIZE, 0i32..SIZE, 0u16..600, 0u16..600, 1i32..6), 1..5),
        ) {
            let opaque = opaque_fn(&map);
            let masks: Vec<Mask> = lights
                .iter()
                .filter(|(x, y, ..)| !opaque(Pos::new(*x, *y)))
                .map(|&(x, y, r, g, radius)| {
                    let color = Light::new(r, g, 100, 50);
                    Mask::compute(Pos::new(x, y), LightId(0), &preset(color, radius), &opaque)
                })
                .collect();
            let mut direct = std::collections::BTreeMap::<Pos, Light>::new();
            for mask in &masks {
                for &(p, l) in &mask.cells {
                    let acc = direct.entry(p).or_insert(Light::DARK);
                    *acc = acc.saturating_add(l);
                }
            }
            let bounce = Bounce::compute(whole(SIZE, SIZE), &masks, &opaque, |_| STONE);
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let p = Pos::new(x, y);
                    let here = bounce.at(p);
                    prop_assert_eq!(here.uv, 0);
                    if opaque(p) {
                        prop_assert!(here.is_dark());
                        continue;
                    }
                    // Only light reflected within reach, through open cells, arrives; at most
                    // BOUNCE/256 of the brightest of it.
                    let brightest = reachable(p, BOUNCE_STEPS, &opaque)
                        .into_iter()
                        .filter_map(|q| direct.get(&q).copied())
                        .fold(Light::DARK, Light::max);
                    for c in 0..3 {
                        let bound = u32::from(brightest.rgb[c]) * BOUNCE / 256;
                        prop_assert!(u32::from(here.rgb[c]) <= bound, "{} at {}", here.rgb[c], p);
                    }
                }
            }
        }

        #[test]
        fn bounce_does_not_depend_on_the_area_beyond_its_reach(
            map in arb_map(), x in 0i32..SIZE, y in 0i32..SIZE, lx in 0i32..SIZE, ly in 0i32..SIZE,
        ) {
            let opaque = opaque_fn(&map);
            let p = Pos::new(x, y);
            let light = preset(Light::new(300, 200, 100, 0), 5);
            let mask = Mask::compute(Pos::new(lx, ly), LightId(0), &light, &opaque);
            let small = Area::around([p], BOUNCE_STEPS, SIZE, SIZE).expect("area");
            let a = Bounce::compute(small, [&mask], &opaque, |_| STONE);
            let b = Bounce::compute(whole(SIZE, SIZE), [&mask], &opaque, |_| STONE);
            prop_assert_eq!(a.at(p), b.at(p));
        }
    }
}
