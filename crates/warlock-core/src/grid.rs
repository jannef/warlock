//! Dense 2D grids for per-cell level data (terrain, memory, visibility, light).

use crate::geom::Pos;
use serde::{Deserialize, Serialize};

/// A dense, row-major grid of `T` with fixed dimensions. Read back (a save), its cells must
/// fill its dimensions, or it is an error, never a grid that panics when indexed (#1585).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawGrid<T>")]
pub struct Grid<T> {
    width: i32,
    height: i32,
    cells: Vec<T>,
}

/// A grid as written, before its size is checked.
#[derive(Deserialize)]
struct RawGrid<T> {
    width: i32,
    height: i32,
    cells: Vec<T>,
}

impl<T> TryFrom<RawGrid<T>> for Grid<T> {
    type Error = String;

    fn try_from(raw: RawGrid<T>) -> Result<Self, String> {
        let RawGrid {
            width,
            height,
            cells,
        } = raw;
        let fits = width >= 0
            && height >= 0
            && Some(cells.len()) == (width as usize).checked_mul(height as usize);
        if !fits {
            return Err(format!("a {width}×{height} grid has {} cells", cells.len()));
        }
        Ok(Self {
            width,
            height,
            cells,
        })
    }
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

/// A `width × height` grid that stores only a box of its cells: every cell outside the box
/// holds `fill`. It reads as a [`Grid`] of the whole would (`get` is `None` only off the grid),
/// and a write outside the box grows the box to take it in. For per-turn grids of a level of
/// which only a view's worth is touched (`Perception::compute`), so a turn does not fill and
/// copy the whole level's worth of cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Window<T> {
    width: i32,
    height: i32,
    /// The box's corners, inclusive; empty while `min.x > max.x`.
    min: Pos,
    max: Pos,
    cells: Vec<T>,
    fill: T,
}

impl<T: Copy> Window<T> {
    /// A `width × height` grid of `fill`, storing the box `min..=max` (clipped to the grid).
    pub fn new(width: i32, height: i32, min: Pos, max: Pos, fill: T) -> Self {
        let mut window = Self {
            width,
            height,
            min: Pos::new(0, 0),
            max: Pos::new(-1, -1),
            cells: Vec::new(),
            fill,
        };
        window.cover(min, max);
        window
    }

    fn in_bounds(&self, p: Pos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }

    /// The index of `p` in the box's cells, if the box holds it.
    fn slot(&self, p: Pos) -> Option<usize> {
        (p.x >= self.min.x && p.y >= self.min.y && p.x <= self.max.x && p.y <= self.max.y).then(
            || ((p.y - self.min.y) * (self.max.x - self.min.x + 1) + (p.x - self.min.x)) as usize,
        )
    }

    /// Grows the box to hold `min..=max` too (clipped to the grid), keeping every cell's value.
    pub fn cover(&mut self, min: Pos, max: Pos) {
        let min = Pos::new(min.x.max(0), min.y.max(0));
        let max = Pos::new(max.x.min(self.width - 1), max.y.min(self.height - 1));
        if min.x > max.x || min.y > max.y {
            return;
        }
        let (new_min, new_max) = if self.min.x > self.max.x {
            (min, max)
        } else {
            (
                Pos::new(self.min.x.min(min.x), self.min.y.min(min.y)),
                Pos::new(self.max.x.max(max.x), self.max.y.max(max.y)),
            )
        };
        if (new_min, new_max) == (self.min, self.max) {
            return;
        }
        let side = new_max.x - new_min.x + 1;
        let mut cells = vec![self.fill; (side * (new_max.y - new_min.y + 1)) as usize];
        if self.min.x <= self.max.x {
            let old = self.max.x - self.min.x + 1;
            for y in self.min.y..=self.max.y {
                let from = ((y - self.min.y) * old) as usize;
                let to = ((y - new_min.y) * side + (self.min.x - new_min.x)) as usize;
                cells[to..to + old as usize]
                    .copy_from_slice(&self.cells[from..from + old as usize]);
            }
        }
        (self.min, self.max, self.cells) = (new_min, new_max, cells);
    }

    pub fn get(&self, p: Pos) -> Option<&T> {
        self.in_bounds(p)
            .then(|| self.slot(p).map_or(&self.fill, |i| &self.cells[i]))
    }

    /// The cell at `p`, the box grown to hold it; `None` off the grid.
    pub fn get_mut(&mut self, p: Pos) -> Option<&mut T> {
        if !self.in_bounds(p) {
            return None;
        }
        self.cover(p, p);
        let i = self.slot(p).expect("the box holds a covered cell");
        Some(&mut self.cells[i])
    }
}

impl<T: Copy> std::ops::Index<Pos> for Window<T> {
    type Output = T;

    /// # Panics
    /// If `p` is off the grid.
    fn index(&self, p: Pos) -> &T {
        self.get(p).expect("grid position out of bounds")
    }
}

impl<T: Copy> std::ops::IndexMut<Pos> for Window<T> {
    /// # Panics
    /// If `p` is off the grid.
    fn index_mut(&mut self, p: Pos) -> &mut T {
        self.get_mut(p).expect("grid position out of bounds")
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
    fn a_grid_read_back_must_fill_its_size() {
        let g = Grid::from_vec(2, 2, vec![1u8, 2, 3, 4]);
        let json = serde_json::to_string(&g).expect("writes");
        assert_eq!(serde_json::from_str::<Grid<u8>>(&json).expect("reads"), g);
        let short = r#"{"width":2,"height":2,"cells":[1,2,3]}"#;
        assert!(serde_json::from_str::<Grid<u8>>(short).is_err());
        let negative = r#"{"width":-1,"height":2,"cells":[]}"#;
        assert!(serde_json::from_str::<Grid<u8>>(negative).is_err());
    }

    #[test]
    fn iter_is_row_major() {
        let g = Grid::from_vec(2, 2, vec![1, 2, 3, 4]);
        let order: Vec<_> = g.iter().map(|(p, v)| (p.x, p.y, *v)).collect();
        assert_eq!(order, vec![(0, 0, 1), (1, 0, 2), (0, 1, 3), (1, 1, 4)]);
    }

    #[test]
    fn window_reads_as_a_whole_grid_and_grows_on_write() {
        let mut w = Window::new(10, 8, Pos::new(2, 2), Pos::new(4, 3), 0u8);
        assert_eq!(w.get(Pos::new(0, 0)), Some(&0));
        assert_eq!(w.get(Pos::new(10, 0)), None);
        w[Pos::new(3, 2)] = 5;
        w[Pos::new(9, 7)] = 6;
        w[Pos::new(0, 0)] = 7;
        assert_eq!(
            (w[Pos::new(3, 2)], w[Pos::new(9, 7)], w[Pos::new(0, 0)]),
            (5, 6, 7)
        );
        assert_eq!(w[Pos::new(5, 5)], 0);
        assert!(w.get_mut(Pos::new(-1, 3)).is_none());
        let mut empty = Window::new(4, 4, Pos::new(9, 9), Pos::new(12, 12), false);
        assert_eq!(empty.get(Pos::new(1, 1)), Some(&false));
        empty[Pos::new(1, 1)] = true;
        assert_eq!(empty.get(Pos::new(1, 1)), Some(&true));
    }
}
