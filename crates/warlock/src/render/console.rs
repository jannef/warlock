//! The CPU-side console: a window-sized grid of glyph cells that UI and map drawing write into.
//! The renderer uploads it to textures when it changes.

use bevy::prelude::*;
use warlock_core::color::Rgb;

/// One console cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgb,
    pub bg: Rgb,
}

impl Cell {
    pub const BLANK: Cell = Cell {
        ch: ' ',
        fg: Rgb::BLACK,
        bg: Rgb::BLACK,
    };
}

/// A grid of cells sized to the window.
#[derive(Resource, Debug, Default)]
pub struct Console {
    cols: u32,
    rows: u32,
    cells: Vec<Cell>,
    /// Set when cells change; cleared by the upload.
    pub dirty: bool,
}

impl Console {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn resize(&mut self, cols: u32, rows: u32) {
        if (cols, rows) != (self.cols, self.rows) {
            self.cols = cols;
            self.rows = rows;
            self.cells = vec![Cell::BLANK; (cols * rows) as usize];
            self.dirty = true;
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::BLANK);
        self.dirty = true;
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as u32) < self.cols && (y as u32) < self.rows)
            .then(|| (y as u32 * self.cols + x as u32) as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Cell> {
        self.index(x, y).map(|i| self.cells[i])
    }

    /// Writes a cell; out-of-range positions are ignored.
    pub fn put(&mut self, x: i32, y: i32, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
            self.dirty = true;
        }
    }

    /// Writes `text` starting at `(x, y)` keeping each cell's background. Returns the number of
    /// cells written (clipped at the right edge).
    pub fn print(&mut self, x: i32, y: i32, text: &str, fg: Rgb) -> i32 {
        let mut written = 0;
        for (i, ch) in text.chars().enumerate() {
            let cx = x + i as i32;
            if let Some(idx) = self.index(cx, y) {
                self.cells[idx].ch = ch;
                self.cells[idx].fg = fg;
                written += 1;
            }
        }
        self.dirty = true;
        written
    }

    /// Fills a rectangle with `cell`.
    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, cell: Cell) {
        for cy in y..y + h {
            for cx in x..x + w {
                self.put(cx, cy, cell);
            }
        }
    }

    /// Draws a single-line box border with `bg` interior.
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, fg: Rgb, bg: Rgb) {
        if w < 2 || h < 2 {
            return;
        }
        self.fill(x, y, w, h, Cell { ch: ' ', fg, bg });
        let mut edge = |cx, cy, ch| self.put(cx, cy, Cell { ch, fg, bg });
        for cx in x + 1..x + w - 1 {
            edge(cx, y, '─');
            edge(cx, y + h - 1, '─');
        }
        for cy in y + 1..y + h - 1 {
            edge(x, cy, '│');
            edge(x + w - 1, cy, '│');
        }
        edge(x, y, '┌');
        edge(x + w - 1, y, '┐');
        edge(x, y + h - 1, '└');
        edge(x + w - 1, y + h - 1, '┘');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_clips_at_edges() {
        let mut c = Console::default();
        c.resize(5, 2);
        assert_eq!(c.print(3, 0, "hello", Rgb::WHITE), 2);
        assert_eq!(c.get(4, 0).map(|cell| cell.ch), Some('e'));
        assert_eq!(c.print(0, 5, "x", Rgb::WHITE), 0);
    }

    #[test]
    fn frame_draws_corners() {
        let mut c = Console::default();
        c.resize(4, 3);
        c.frame(0, 0, 4, 3, Rgb::WHITE, Rgb::BLACK);
        assert_eq!(c.get(0, 0).map(|cell| cell.ch), Some('┌'));
        assert_eq!(c.get(3, 2).map(|cell| cell.ch), Some('┘'));
        assert_eq!(c.get(1, 1).map(|cell| cell.ch), Some(' '));
    }
}
