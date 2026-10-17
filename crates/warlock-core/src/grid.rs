//! Dense 2D grids for per-cell level data (terrain, memory, visibility, light).

use crate::geom::Pos;
use serde::{Deserialize, Serialize};

/// A dense, row-major grid of `T` with fixed dimensions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid<T> {
    width: i32,
    height: i32,
    cells: Vec<T>,
}

impl<T: Clone> Grid<T> {
    /// A `width × height` grid filled with `fill`.
    ///
    /// # Panics
    /// If either dimension is negative.
    pub fn new(width: i32, height: i32, fill: T) -> Self {
        assert!(
            width >= 0 && height >= 0,
            "grid dimensions must be non-negative"
        );
        Self {
            width,
            height,
            cells: vec![fill; (width as usize) * (height as usize)],
        }
    }

    /// Resets every cell to `value`.
    pub fn fill(&mut self, value: T) {
        self.cells.fill(value);
    }
}

impl<T> Grid<T> {
    /// Builds a grid from row-major cells.
    ///
    /// # Panics
    /// If `cells.len() != width * height`.
    pub fn from_vec(width: i32, height: i32, cells: Vec<T>) -> Self {
        assert_eq!(
            cells.len(),
            (width.max(0) as usize) * (height.max(0) as usize),
            "cell count does not match dimensions"
        );
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn in_bounds(&self, p: Pos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }

    fn index(&self, p: Pos) -> Option<usize> {
        self.in_bounds(p)
            .then(|| (p.y as usize) * (self.width as usize) + (p.x as usize))
    }

    pub fn get(&self, p: Pos) -> Option<&T> {
        self.index(p).map(|i| &self.cells[i])
    }

    pub fn get_mut(&mut self, p: Pos) -> Option<&mut T> {
        self.index(p).map(|i| &mut self.cells[i])
    }

    /// Sets the cell at `p`; returns `false` (and does nothing) if `p` is out of bounds.
    pub fn set(&mut self, p: Pos, value: T) -> bool {
        match self.index(p) {
            Some(i) => {
                self.cells[i] = value;
                true
            }
            None => false,
        }
    }

    /// All cells in row-major order with their positions.
    pub fn iter(&self) -> impl Iterator<Item = (Pos, &T)> {
        let w = self.width.max(1);
        self.cells
            .iter()
            .enumerate()
            .map(move |(i, c)| (Pos::new(i as i32 % w, i as i32 / w), c))
    }

    /// Raw row-major cells.
    pub fn cells(&self) -> &[T] {
        &self.cells
    }
}

impl<T> std::ops::Index<Pos> for Grid<T> {
    type Output = T;

    /// # Panics
    /// If `p` is out of bounds.
    fn index(&self, p: Pos) -> &T {
        let i = Grid::index(self, p).expect("grid position out of bounds");
        &self.cells[i]
    }
}

impl<T> std::ops::IndexMut<Pos> for Grid<T> {
    /// # Panics
    /// If `p` is out of bounds.
    fn index_mut(&mut self, p: Pos) -> &mut T {
        let i = Grid::index(self, p).expect("grid position out of bounds");
        &mut self.cells[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_bounds() {
        let mut g = Grid::new(3, 2, 0u8);
        assert!(g.set(Pos::new(2, 1), 7));
        assert!(!g.set(Pos::new(3, 0), 1));
        assert_eq!(g.get(Pos::new(2, 1)), Some(&7));
        assert_eq!(g.get(Pos::new(-1, 0)), None);
        assert_eq!(g[Pos::new(2, 1)], 7);
    }

    #[test]
    fn iter_is_row_major() {
        let g = Grid::from_vec(2, 2, vec![1, 2, 3, 4]);
        let order: Vec<_> = g.iter().map(|(p, v)| (p.x, p.y, *v)).collect();
        assert_eq!(order, vec![(0, 0, 1), (1, 0, 2), (0, 1, 3), (1, 1, 4)]);
    }
}
