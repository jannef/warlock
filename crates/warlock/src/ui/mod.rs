//! Glyph UI: the map viewport, message line, status bar, and controls overlay (spec §6).
//!
//! Everything shown about the world comes from `World::perception()`. Terrain goes into the map
//! grid layer, creatures and glyphs that outgrow their cell into the free-glyph layer
//! ([`glyphs`]), and text and panels into the UI console on top. [`motion`] moves the camera and
//! slides creatures between cells.

mod glyphs;
mod motion;

use crate::game::{Game, SimSet};
use crate::input::{Bindings, InputMode};
use crate::render::anim::{AnimTable, TerrainAnim};
use crate::render::console::{Cell, Console};
use crate::render::grid::MapGrid;
use crate::render::{ConsoleLayout, VisualTime};
use bevy::prelude::*;
use motion::Motion;
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::perception::{CellView, PerceivedLight, PerceptionView};

const MESSAGE_ROWS: i32 = 2;
const STATUS_ROWS: i32 = 1;

const TEXT: Rgb = Rgb::new(0xd8, 0xd4, 0xc8);
const TEXT_DIM: Rgb = Rgb::new(0x7c, 0x78, 0x70);
const ACCENT: Rgb = Rgb::new(0xe0, 0xb0, 0x50);
const PANEL_BG: Rgb = Rgb::new(0x12, 0x12, 0x18);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motion>().add_systems(
            Update,
            (motion::track, (draw, draw_map_layer), glyphs::emit)
                .chain()
                .after(SimSet),
        );
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
    let rows = console.rows() as i32;
    draw_messages(&mut console, &game, *mode);
    draw_status(&mut console, &game, rows - 1);
    if *mode == InputMode::Help {
        draw_help(&mut console, &bindings);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect2 {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// The map viewport in UI console cells: everything between the message line and the status bar.
fn map_area(cols: i32, rows: i32) -> Option<Rect2> {
    let h = rows - MESSAGE_ROWS - STATUS_ROWS;
    (h > 0 && cols > 0).then_some(Rect2 {
        x: 0,
        y: MESSAGE_ROWS,
        w: cols,
        h,
    })
}

/// What the map grid was last drawn for.
#[derive(Debug, PartialEq, Eq)]
struct MapKey {
    level_origin: IVec2,
    area: Rect2,
    covered: Vec<Pos>,
}

/// Places the map grid under the camera and redraws it when the view changed.
fn draw_map_layer(
    game: Res<Game>,
    console: Res<Console>,
    layout: Res<ConsoleLayout>,
    motion: Res<Motion>,
    visual: Res<VisualTime>,
    anims: Res<AnimTable>,
    mut grid: ResMut<MapGrid>,
    mut drawn: Local<Option<MapKey>>,
) {
    let Some(area) = map_area(console.cols() as i32, console.rows() as i32) else {
        grid.clip = Rect::default();
        return;
    };
    let cell = layout.cell.as_vec2();
    let (level_origin, offset) = motion::camera_split(motion.camera(visual.seconds), cell);
    let top_left = layout.origin + Vec2::new(area.x as f32, area.y as f32) * cell;
    grid.cell = cell;
    grid.clip = Rect::from_corners(
        top_left,
        top_left + Vec2::new(area.w as f32, area.h as f32) * cell,
    );
    grid.origin = top_left - offset;
    grid.level_origin = level_origin;

    let view = game.world.perception();
    let key = MapKey {
        level_origin,
        area,
        covered: motion.occupied(view.actors().iter().map(|a| a.pos)),
    };
    let changed = game.is_changed() || anims.is_changed() || layout.is_changed();
    if !changed && drawn.as_ref() == Some(&key) {
        return;
    }
    // One margin cell right and below, so a sub-cell offset never shows a gap.
    grid.console.resize(area.w as u32 + 1, area.h as u32 + 1);
    grid.console.clear();
    draw_map(&mut grid, &view, &anims, &key.covered);
    *drawn = Some(key);
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

/// Draws terrain into the map grid. Creatures are on the free-glyph layer: the grid keeps the
/// terrain under them but not its glyph (`covered`). Only visible terrain animates.
fn draw_map(grid: &mut MapGrid, view: &PerceptionView<'_>, anims: &AnimTable, covered: &[Pos]) {
    let content = view.content();
    let (ox, oy) = (grid.level_origin.x, grid.level_origin.y);
    grid.free_terrain.clear();
    for sy in 0..grid.console.rows() as i32 {
        for sx in 0..grid.console.cols() as i32 {
            let p = Pos::new(ox + sx, oy + sy);
            let seen = view.cell(p);
            let cell = match seen {
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
                    let light = view.light(p);
                    Cell {
                        ch: def.glyph,
                        fg: lit(def.fg, light, TERRAIN_FLOOR),
                        bg: lit(def.bg, light, TERRAIN_FLOOR),
                    }
                }
            };
            grid.console.put(sx, sy, cell);
            if let CellView::Visible(t) = seen {
                match anims.terrain(t) {
                    Some(TerrainAnim::Grid { .. }) => {
                        grid.console.animate(sx, sy, anims.cell(t, p))
                    }
                    Some(TerrainAnim::Free) => {
                        grid.console.clear_glyph(sx, sy);
                        if covered.binary_search(&p).is_err() {
                            grid.free_terrain.push((p, t));
                        }
                    }
                    None => {}
                }
            }
        }
    }
    for &p in covered {
        let at = grid.to_console(p);
        grid.console.clear_glyph(at.x, at.y);
    }
}

/// Lowest light factor for visible terrain and actors, so dim cells stay readable.
const TERRAIN_FLOOR: f32 = 0.3;
/// Lowest light factor for creatures (drawn on the free-glyph layer by [`glyphs`]).
const ACTOR_FLOOR: f32 = 0.6;

/// Simple per-cell lighting until visual light compositing lands (#19): each channel is scaled by
/// the perceived light of that channel (a square-root curve, with a readability floor). UV, when
/// perceived, tints violet; cells seen only through darkvision are drawn in dim grey.
pub(super) fn lit(c: Rgb, light: Option<PerceivedLight>, floor: f32) -> Rgb {
    let Some(light) = light else {
        return c;
    };
    if light.darkvision {
        let luma = (u32::from(c.r) * 54 + u32::from(c.g) * 183 + u32::from(c.b) * 19) / 256;
        let grey = (luma as f32 * (floor + 0.15)).round() as u8;
        return Rgb::new(grey, grey, grey);
    }
    let [r, g, b] = light.rgb.map(f32::from);
    let uv = f32::from(light.uv);
    let scale = |v: u8, l: f32| {
        let t = (l / 255.0).clamp(0.0, 1.0).sqrt();
        (f32::from(v) * (floor + (1.0 - floor) * t)).round() as u8
    };
    Rgb::new(scale(c.r, r + uv * 0.6), scale(c.g, g), scale(c.b, b + uv))
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
    let light = view.light(pos).map_or(0, |l| l.brightness());
    let lantern = match world.lantern() {
        Some(true) => "  ·  Lantern lit",
        Some(false) => "  ·  Lantern out",
        None => "",
    };
    let status = format!(
        "{}  ·  Turn {}  ·  {},{}  ·  Light {}  ·  {} vision{}  ·  Seed {}",
        view.level_name(),
        world.turn(),
        pos.x,
        pos.y,
        light,
        view.vision().name,
        lantern,
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
    use crate::render::console::CellAnim;
    use std::sync::Arc;
    use warlock_core::content::{Content, ContentFiles};
    use warlock_core::{Action, Dir, World};

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

    #[test]
    fn map_area_lies_between_messages_and_status() {
        assert_eq!(
            map_area(80, 24),
            Some(Rect2 {
                x: 0,
                y: MESSAGE_ROWS,
                w: 80,
                h: 24 - MESSAGE_ROWS - STATUS_ROWS
            })
        );
        assert_eq!(map_area(80, MESSAGE_ROWS + STATUS_ROWS), None);
    }

    /// Water and an altar on both sides of a door; the player looks, then closes the door.
    fn pool_world() -> World {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/pool.toml",
            "name = \"Pool\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#~@+~_#\n#######\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/pool.toml",
            "name = \"P\"\nstart_level = \"pool\"\n",
        );
        let content = Content::load(&files).expect("test content loads");
        let mut world = World::new(Arc::new(content), "pool", None).expect("scenario starts");
        world.apply(&Action::Open(Dir::E));
        world.apply(&Action::Close(Dir::E));
        world
    }

    #[test]
    fn map_animates_only_visible_terrain_and_hands_pulses_to_free_glyphs() {
        let mut world = pool_world();
        let view = world.perception();
        let content = view.content();
        let anims = AnimTable::new(content, |_| 0, true);
        let mut grid = MapGrid::default();
        grid.console.resize(7, 3);
        let player = view.viewer();
        draw_map(&mut grid, &view, &anims, &[player]);

        let at = |x: i32, y: i32| (y * 7 + x) as usize;
        let cell = |x, y| grid.console.cells()[at(x, y)];
        // The player's cell keeps its floor background but not the floor glyph.
        assert_eq!(cell(2, 1).map(|c| c.ch), Some(' '));
        // Visible water animates; remembered water behind the closed door does not.
        assert!(grid.console.anims()[at(1, 1)].entry > 0);
        assert_eq!(cell(4, 1).map(|c| c.ch), Some('~'));
        assert_eq!(grid.console.anims()[at(4, 1)], CellAnim::STILL);
        // The remembered altar is a still glyph in the grid, not a pulse.
        assert_eq!(cell(5, 1).map(|c| c.ch), Some('_'));
        assert!(grid.free_terrain.is_empty());

        // With the door open the altar is visible: it pulses on the free-glyph layer.
        let altar = content.terrain_id("altar").expect("altar exists");
        world.apply(&Action::Open(Dir::E));
        let view = world.perception();
        draw_map(&mut grid, &view, &anims, &[player]);
        assert_eq!(grid.free_terrain, vec![(Pos::new(5, 1), altar)]);
        assert_eq!(grid.console.cells()[at(5, 1)].map(|c| c.ch), Some(' '));
    }

    fn perceived(rgb: [u16; 3], uv: u16, darkvision: bool) -> Option<PerceivedLight> {
        Some(PerceivedLight {
            rgb,
            uv,
            darkvision,
            dist2: 4,
        })
    }

    #[test]
    fn light_scales_colors_with_a_readable_floor() {
        let c = Rgb::new(200, 200, 200);
        assert_eq!(lit(c, perceived([255, 255, 255], 0, false), 0.3), c);
        assert_eq!(lit(c, perceived([600, 600, 600], 0, false), 0.3), c);
        let dark = lit(c, perceived([0, 0, 0], 0, false), 0.3);
        assert_eq!(dark, Rgb::new(60, 60, 60));
        let red = lit(c, perceived([255, 0, 0], 0, false), 0.3);
        assert!(red.r > red.g && red.g == red.b);
        let dv = lit(Rgb::new(200, 100, 50), perceived([0, 0, 0], 0, true), 0.3);
        assert!(dv.r == dv.g && dv.g == dv.b, "darkvision is monochrome");
        let uv = lit(c, perceived([0, 0, 0], 200, false), 0.3);
        assert!(uv.b > uv.r && uv.r > uv.g, "UV tints violet");
    }
}
