//! Smooth integer value noise for the outdoor generators: random values on a coarse lattice,
//! eased between lattice points, so woods, meadows, and pools come in natural blobs rather than
//! speckle. Integer-only, like all generation.

use crate::geom::Pos;
use crate::grid::Grid;
use crate::rng::Rng;

/// Noise values run from 0 to this.
pub(super) const MAX: i32 = 1000;

/// Fixed-point one for the easing weights.
const ONE: i32 = 1024;

/// One octave: lattice values every `step` rows and `2 · step` columns (a cell is about twice as
/// tall as it is wide on screen, so blobs come out round).
struct Octave {
    step: i32,
    cols: i32,
    values: Vec<i32>,
}

impl Octave {
    fn new(rng: &mut Rng, width: i32, height: i32, step: i32) -> Self {
        let step = step.max(1);
        let cols = width / (2 * step) + 2;
        let rows = height / step + 2;
        let values = (0..cols * rows)
            .map(|_| rng.below(MAX as u32 + 1) as i32)
            .collect();
        Self { step, cols, values }
    }

    fn at(&self, p: Pos) -> i32 {
        let (sx, sy) = (2 * self.step, self.step);
        let (gx, gy) = (p.x.max(0) / sx, p.y.max(0) / sy);
        let ex = ease(p.x.max(0) % sx, sx);
        let ey = ease(p.y.max(0) % sy, sy);
        let v = |x: i32, y: i32| self.values[(y * self.cols + x) as usize];
        let top = lerp(v(gx, gy), v(gx + 1, gy), ex);
        let bottom = lerp(v(gx, gy + 1), v(gx + 1, gy + 1), ex);
        lerp(top, bottom, ey)
    }
}

/// Smoothstep of `t / span` in [`ONE`]ths.
fn ease(t: i32, span: i32) -> i32 {
    let t = t * ONE / span;
    t * t / ONE * (3 * ONE - 2 * t) / ONE
}

fn lerp(a: i32, b: i32, weight: i32) -> i32 {
    a + (b - a) * weight / ONE
}

/// A field of noise over a `width`×`height` map: octaves with lattice steps (rows) and weights,
/// summed and scaled back to 0..=[`MAX`].
pub(super) fn field(rng: &mut Rng, width: i32, height: i32, octaves: &[(i32, i32)]) -> Grid<i32> {
    let layers: Vec<(Octave, i32)> = octaves
        .iter()
        .map(|&(step, weight)| (Octave::new(rng, width, height, step), weight))
        .collect();
    let total: i32 = layers.iter().map(|(_, w)| w).sum::<i32>().max(1);
    let mut grid = Grid::new(width, height, 0);
    for y in 0..height {
        for x in 0..width {
            let p = Pos::new(x, y);
            let sum: i32 = layers.iter().map(|(o, w)| o.at(p) * w).sum();
            grid[p] = sum / total;
        }
    }
    grid
}

/// The values of a field in ascending order, to turn a share of the map into a threshold.
pub(super) struct Quantiles(Vec<i32>);

impl Quantiles {
    pub fn of(field: &Grid<i32>) -> Self {
        let mut values = field.cells().to_vec();
        values.sort_unstable();
        Self(values)
    }

    /// The value that `share` permille of the field lies above (none for 0, all for 1000).
    pub fn cut(&self, share: i32) -> i32 {
        let n = self.0.len() as i32;
        if n == 0 || share <= 0 {
            return MAX + 1;
        }
        if share >= 1000 {
            return -1;
        }
        let i = (n - n * share / 1000).clamp(0, n - 1);
        self.0[i as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_smooth_and_in_range() {
        let mut rng = Rng::stream(1, "noise");
        let f = field(&mut rng, 80, 40, &[(6, 2), (2, 1)]);
        for (p, v) in f.iter() {
            assert!((0..=MAX).contains(v), "{p}: {v}");
            let right = Pos::new(p.x + 1, p.y);
            if let Some(r) = f.get(right) {
                assert!((v - r).abs() <= 300, "{p}: jump from {v} to {r}");
            }
        }
    }

    #[test]
    fn quantiles_cut_the_asked_share() {
        let mut rng = Rng::stream(2, "noise");
        let f = field(&mut rng, 80, 40, &[(5, 1)]);
        let q = Quantiles::of(&f);
        for share in [100, 300, 700] {
            let cut = q.cut(share);
            let above = f.cells().iter().filter(|v| **v > cut).count() as i32;
            assert!(
                (above - 3200 * share / 1000).abs() <= 80,
                "{share}: {above}"
            );
        }
        assert!(f.cells().iter().all(|v| *v < q.cut(0)));
        assert!(f.cells().iter().all(|v| *v > q.cut(1000)));
    }
}
