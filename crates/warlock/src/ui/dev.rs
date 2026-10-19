//! Developer overlays (spec §9), toggled by dev keys (debug builds or `--dev`; see the bindings
//! table, so the `?` help lists them):
//! - **Reveal map** draws ground truth — terrain, objects, and creatures the player does not
//!   see — in a cold violet nothing else uses. It is the one place presentation reads ground
//!   truth (`World::ground_truth`), and is clearly dev-only.
//! - **Light values** replaces each visible cell's glyph with its perceived brightness as a digit,
//!   like the dump's `light` layer (`0` dark to `9` fully lit); the shader still lights the cell,
//!   so the digit shows the light's color too.

use crate::render::console::{Cell, Console};
use crate::render::grid::MapGrid;
use bevy::prelude::*;
use warlock_core::color::Rgb;
use warlock_core::dump::light_digit;
use warlock_core::perception::CellView;
use warlock_core::{Pos, World};

/// Which developer overlays are on.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DevOverlays {
    /// Draw ground truth where the player does not see. Dev only.
    pub reveal: bool,
    /// Show perceived brightness digits on visible cells.
    pub light_values: bool,
}

impl DevOverlays {
    fn names(self) -> Vec<&'static str> {
        [(self.reveal, "reveal"), (self.light_values, "light values")]
            .into_iter()
            .filter_map(|(on, name)| on.then_some(name))
            .collect()
    }
}

/// Color of the "DEV" indicator and of revealed creatures.
const DEV_ACCENT: Rgb = Rgb::new(0xe0, 0x70, 0xe0);
/// Digits of the light-value overlay (lit by the shader like terrain).
const DIGIT: Rgb = Rgb::new(0xf0, 0xf0, 0xf0);

/// Revealed terrain: dim, in a cold violet unlike anything in view or remembered.
fn revealed(c: Rgb) -> Rgb {
    let luma = (u32::from(c.r) * 54 + u32::from(c.g) * 183 + u32::from(c.b) * 19) / 256;
    let ch = |weight: u32, base: u32| (luma * weight / 100 + base).min(255) as u8;
    Rgb::new(ch(40, 18), ch(30, 14), ch(60, 36))
}

/// Draws the enabled overlays into the map grid, over what the player perceives.
pub(super) fn draw_map_overlays(grid: &mut MapGrid, world: &World, overlays: DevOverlays) {
    if overlays == DevOverlays::default() {
        return;
    }
    let view = world.perception();
    let content = world.content();
    let (cols, rows) = (grid.console.cols() as i32, grid.console.rows() as i32);
    let cells = (0..rows).flat_map(|y| (0..cols).map(move |x| (x, y)));
    let level_pos = |x: i32, y: i32| Pos::new(grid.level_origin.x + x, grid.level_origin.y + y);
    let positions: Vec<_> = cells.map(|(x, y)| ((x, y), level_pos(x, y))).collect();

    if overlays.reveal {
        let (level, actors) = world.ground_truth();
        for &((x, y), p) in &positions {
            if !view.is_visible(p)
                && let Some(t) = level.terrain_at(p)
            {
                let def = content.terrain(t);
                let (ch, fg) = match level.objects_at(p).first() {
                    Some(top) => {
                        let object = content.object(top.object);
                        (object.glyph, object.fg)
                    }
                    None => (def.glyph_at(p), def.fg),
                };
                grid.console.put(
                    x,
                    y,
                    Cell {
                        ch,
                        fg: revealed(fg),
                        bg: revealed(def.bg),
                    },
                );
            }
        }
        // Traps the player does not know, in view too (a hidden trap is hidden there).
        for &((x, y), p) in &positions {
            let Some(trap) = level.trap_at(p) else {
                continue;
            };
            if view.trap(p).is_some() || view.actor_at(p).is_some() {
                continue;
            }
            let bg = level.terrain_at(p).map_or(Rgb::BLACK, |t| {
                let bg = content.terrain(t).bg;
                if view.is_visible(p) { bg } else { revealed(bg) }
            });
            grid.free_terrain.retain(|(q, _)| *q != p);
            let ch = content.trap(trap.kind).glyph;
            grid.console.put(
                x,
                y,
                Cell {
                    ch,
                    fg: DEV_ACCENT,
                    bg,
                },
            );
        }
        for actor in actors.filter(|a| view.actor_at(a.pos).is_none()) {
            let at = grid.to_console(actor.pos);
            let bg = level
                .terrain_at(actor.pos)
                .map_or(Rgb::BLACK, |t| revealed(content.terrain(t).bg));
            let ch = content.creature(actor.creature).glyph;
            grid.console.put(
                at.x,
                at.y,
                Cell {
                    ch,
                    fg: DEV_ACCENT,
                    bg,
                },
            );
        }
    }

    if overlays.light_values {
        for &((x, y), p) in &positions {
            if let (Some(light), CellView::Visible(t)) = (view.light(p), view.cell(p))
                && view.actor_at(p).is_none()
            {
                grid.free_terrain.retain(|(q, _)| *q != p);
                grid.console.put(
                    x,
                    y,
                    Cell {
                        ch: light_digit(light),
                        fg: DIGIT,
                        bg: content.terrain(t).bg,
                    },
                );
            }
        }
    }
}

/// Names the active overlays at the right end of the message line, so a revealed map is never
/// mistaken for what the player sees.
pub(super) fn draw_indicator(console: &mut Console, overlays: DevOverlays) {
    let names = overlays.names();
    if names.is_empty() {
        return;
    }
    let text = format!("DEV: {}", names.join(", "));
    let x = console.cols() as i32 - text.chars().count() as i32 - 1;
    console.print(x, 0, &text, DEV_ACCENT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::Content;

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn grid_over_level(world: &World) -> MapGrid {
        let view = world.perception();
        let mut grid = MapGrid::default();
        grid.console
            .resize(view.width() as u32, view.height() as u32);
        grid
    }

    #[test]
    fn reveal_shows_ground_truth_outside_the_view_only() {
        let world = showcase();
        let view = world.perception();
        let mut grid = grid_over_level(&world);
        draw_map_overlays(
            &mut grid,
            &world,
            DevOverlays {
                reveal: true,
                ..default()
            },
        );
        // The light hall's crystal, far out of view, is revealed in the dev tint.
        let crystal = Pos::new(6, 4);
        assert!(!view.is_visible(crystal));
        let cell = grid.console.get(crystal.x, crystal.y).expect("revealed");
        assert_eq!(cell.ch, '♦');
        // Hidden creatures are revealed too; cells in view are left to the normal map.
        let (_, actors) = world.ground_truth();
        let hidden: Vec<_> = actors.filter(|a| !view.is_visible(a.pos)).collect();
        assert!(!hidden.is_empty());
        for actor in hidden {
            let cell = grid
                .console
                .get(actor.pos.x, actor.pos.y)
                .expect("revealed");
            assert_eq!(cell.fg, DEV_ACCENT);
        }
        assert_eq!(grid.console.get(7, 15), None);
    }

    #[test]
    fn light_values_replace_visible_glyphs() {
        let world = showcase();
        let view = world.perception();
        let mut grid = grid_over_level(&world);
        grid.console
            .fill(0, 0, view.width(), view.height(), Cell::BLANK);
        draw_map_overlays(
            &mut grid,
            &world,
            DevOverlays {
                light_values: true,
                ..default()
            },
        );
        let lit = Pos::new(3, 15);
        let digit = light_digit(view.light(lit).expect("visible"));
        assert_eq!(grid.console.get(lit.x, lit.y).map(|c| c.ch), Some(digit));
        assert_eq!(
            grid.console.get(7, 15).map(|c| c.ch),
            Some(' '),
            "the player"
        );
        assert_eq!(
            grid.console.get(5, 4).map(|c| c.ch),
            Some(' '),
            "out of view"
        );
    }

    #[test]
    fn indicator_names_active_overlays() {
        assert!(DevOverlays::default().names().is_empty());
        let both = DevOverlays {
            reveal: true,
            light_values: true,
        };
        assert_eq!(both.names(), vec!["reveal", "light values"]);
        let mut console = Console::default();
        console.resize(40, 2);
        draw_indicator(&mut console, both);
        assert_eq!(console.get(39 - 1, 0).map(|c| c.ch), Some('s'));
    }
}
