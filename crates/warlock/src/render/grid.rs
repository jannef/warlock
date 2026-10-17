//! The map grid layer (spec §4.2, layer 1): a console the map is drawn into, shown behind a clip
//! rectangle (the map viewport) with a sub-cell pixel offset, so the camera can scroll smoothly.
//!
//! The grid is one cell larger than the viewport in each direction: whatever the offset, the
//! viewport is fully covered and its edges never show gaps.

use super::console::Console;
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::content::TerrainId;

/// The map grid and where it sits on screen.
#[derive(Resource, Debug, Default)]
pub struct MapGrid {
    /// Terrain cells; cell (0, 0) shows level position [`MapGrid::level_origin`].
    pub console: Console,
    /// Level position drawn in console cell (0, 0).
    pub level_origin: IVec2,
    /// Framebuffer position (physical pixels) of console cell (0, 0)'s top-left corner.
    pub origin: Vec2,
    /// Cell size in physical pixels.
    pub cell: Vec2,
    /// The map viewport in framebuffer pixels; nothing of the map is drawn outside it.
    pub clip: Rect,
    /// Visible terrain drawn on the free-glyph layer instead of the grid (it outgrows its cell).
    pub free_terrain: Vec<(Pos, TerrainId)>,
}

impl MapGrid {
    /// Framebuffer position of the top-left corner of level position `p` (may be fractional).
    pub fn to_screen(&self, p: Vec2) -> Vec2 {
        self.origin + (p - self.level_origin.as_vec2()) * self.cell
    }

    /// Console coordinates of level position `p`.
    pub fn to_console(&self, p: Pos) -> IVec2 {
        IVec2::new(p.x, p.y) - self.level_origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_level_positions_to_pixels_and_cells() {
        let grid = MapGrid {
            level_origin: IVec2::new(10, 5),
            origin: Vec2::new(4.0, 40.0),
            cell: Vec2::new(11.0, 22.0),
            ..default()
        };
        assert_eq!(grid.to_screen(Vec2::new(10.0, 5.0)), Vec2::new(4.0, 40.0));
        assert_eq!(grid.to_screen(Vec2::new(12.5, 6.0)), Vec2::new(31.5, 62.0));
        assert_eq!(grid.to_console(Pos::new(13, 5)), IVec2::new(3, 0));
    }
}
