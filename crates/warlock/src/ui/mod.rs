//! Glyph UI: the map viewport, message line, status bar, and controls overlay (spec §6).
//!
//! Everything shown about the world comes from `World::perception()`.

use crate::game::{Game, SimSet};
use crate::input::{Bindings, InputMode};
use crate::render::ConsoleLayout;
use crate::render::console::{Cell, Console};
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::perception::{CellView, PerceptionView};

const MESSAGE_ROWS: i32 = 2;
const STATUS_ROWS: i32 = 1;

const TEXT: Rgb = Rgb::new(0xd8, 0xd4, 0xc8);
const TEXT_DIM: Rgb = Rgb::new(0x7c, 0x78, 0x70);
const ACCENT: Rgb = Rgb::new(0xe0, 0xb0, 0x50);
const PANEL_BG: Rgb = Rgb::new(0x12, 0x12, 0x18);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw.after(SimSet));
    }
}

fn draw(
    game: Res<Game>,
    mode: Res<InputMode>,
    layout: Res<ConsoleLayout>,
    bindings: Res<Bindings>,
    mut console: ResMut<Console>,
) {
    if !(game.is_changed() || mode.is_changed() || layout.is_changed() || bindings.is_changed()) {
        return;
    }
    console.clear();
    let cols = console.cols() as i32;
    let rows = console.rows() as i32;
    let map_rows = rows - MESSAGE_ROWS - STATUS_ROWS;
    if map_rows > 0 {
        draw_map(
            &mut console,
            &game.world.perception(),
            Rect2 {
                x: 0,
                y: MESSAGE_ROWS,
                w: cols,
                h: map_rows,
            },
        );
    }
    draw_messages(&mut console, &game, *mode);
    draw_status(&mut console, &game, rows - 1);
    if *mode == InputMode::Help {
        draw_help(&mut console, &bindings);
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect2 {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// World position shown at the viewport's top-left. Levels smaller than the viewport are
/// centered; larger ones scroll to keep the viewer in view.
fn camera_origin(view: i32, level: i32, viewer: i32) -> i32 {
    if level <= view {
        -(view - level) / 2
    } else {
        (viewer - view / 2).clamp(0, level - view)
    }
}

fn draw_map(console: &mut Console, view: &PerceptionView<'_>, area: Rect2) {
    let content = view.content();
    let viewer = view.viewer();
    let ox = camera_origin(area.w, view.width(), viewer.x);
    let oy = camera_origin(area.h, view.height(), viewer.y);
    for sy in 0..area.h {
        for sx in 0..area.w {
            let p = Pos::new(ox + sx, oy + sy);
            let cell = match view.cell(p) {
                CellView::Unknown => continue,
                CellView::Remembered(t) => {
                    let def = content.terrain(t);
                    Cell {
                        ch: def.glyph,
                        fg: remembered(def.fg),
                        bg: remembered(def.bg),
                    }
                }
                CellView::Visible(t) => {
                    let def = content.terrain(t);
                    Cell {
                        ch: def.glyph,
                        fg: def.fg,
                        bg: def.bg,
                    }
                }
            };
            console.put(area.x + sx, area.y + sy, cell);
        }
    }
    for actor in view.actors() {
        let (sx, sy) = (actor.pos.x - ox, actor.pos.y - oy);
        if sx < 0 || sy < 0 || sx >= area.w || sy >= area.h {
            continue;
        }
        let def = content.creature(actor.creature);
        let (x, y) = (area.x + sx, area.y + sy);
        let bg = console.get(x, y).map_or(Rgb::BLACK, |c| c.bg);
        console.put(
            x,
            y,
            Cell {
                ch: def.glyph,
                fg: def.fg,
                bg,
            },
        );
    }
}

/// Remembered terrain: dim and desaturated, so it never reads as "in view".
fn remembered(c: Rgb) -> Rgb {
    let luma = (u32::from(c.r) * 54 + u32::from(c.g) * 183 + u32::from(c.b) * 19) / 256;
    let f = |v: u8| ((u32::from(v) + luma) / 2 * 45 / 100) as u8;
    Rgb::new(f(c.r), f(c.g), f(c.b))
}

fn draw_messages(console: &mut Console, game: &Game, mode: InputMode) {
    if let InputMode::Direction(_) = mode {
        console.print(1, 0, "In which direction? (Esc to cancel)", ACCENT);
        return;
    }
    let turn = game.world.turn();
    let recent: Vec<_> = game
        .world
        .messages()
        .iter()
        .rev()
        .take(MESSAGE_ROWS as usize)
        .collect();
    for (row, msg) in recent.iter().rev().enumerate() {
        let color = if msg.turn + 1 >= turn { TEXT } else { TEXT_DIM };
        console.print(1, row as i32, &msg.text, color);
    }
}

fn draw_status(console: &mut Console, game: &Game, y: i32) {
    let cols = console.cols() as i32;
    console.fill(
        0,
        y,
        cols,
        1,
        Cell {
            ch: ' ',
            fg: TEXT,
            bg: PANEL_BG,
        },
    );
    let world = &game.world;
    let view = world.perception();
    let pos = world.player_pos();
    let status = format!(
        "{}  ·  Turn {}  ·  {},{}  ·  Seed {}",
        view.level_name(),
        world.turn(),
        pos.x,
        pos.y,
        world.seed()
    );
    console.print(1, y, &status, TEXT);
    let hint = "? controls";
    console.print(cols - hint.chars().count() as i32 - 1, y, hint, TEXT_DIM);
}

fn draw_help(console: &mut Console, bindings: &Bindings) {
    let lines: Vec<(String, String)> = bindings
        .labels()
        .into_iter()
        .map(|label| (label.to_string(), bindings.keys_for(label).join(" ")))
        .collect();
    let label_w = lines
        .iter()
        .map(|(l, _)| l.chars().count())
        .max()
        .unwrap_or(0) as i32;
    let keys_w = lines
        .iter()
        .map(|(_, k)| k.chars().count())
        .max()
        .unwrap_or(0) as i32;
    let w = (label_w + keys_w + 7).min(console.cols() as i32);
    let h = (lines.len() as i32 + 4).min(console.rows() as i32);
    let x = (console.cols() as i32 - w) / 2;
    let y = (console.rows() as i32 - h) / 2;
    console.frame(x, y, w, h, TEXT_DIM, PANEL_BG);
    console.print(x + 2, y, " Controls ", ACCENT);
    for (i, (label, keys)) in lines.iter().enumerate() {
        let row = y + 2 + i as i32;
        console.print(x + 2, row, label, TEXT);
        console.print(x + 4 + label_w, row, keys, ACCENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_centers_small_levels_and_clamps_large_ones() {
        assert_eq!(camera_origin(100, 80, 10), -10);
        assert_eq!(camera_origin(50, 400, 10), 0);
        assert_eq!(camera_origin(50, 400, 200), 175);
        assert_eq!(camera_origin(50, 400, 399), 350);
    }

    #[test]
    fn remembered_colors_are_darker() {
        let c = Rgb::new(200, 100, 50);
        let r = remembered(c);
        assert!(r.r < c.r && r.g < c.g);
    }
}
