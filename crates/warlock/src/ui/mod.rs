//! Glyph UI: the map viewport, message line, status bar, and controls overlay (spec §6).
//!
//! Everything shown about the world comes from `World::perception()`. Terrain goes into the map
//! grid layer, creatures and glyphs that outgrow their cell into the free-glyph layer
//! ([`glyphs`]), and text and panels into the UI console on top. [`motion`] moves the camera and
//! slides creatures between cells; [`effects`] adds particles, glow, heat haze, and fades. Terrain
//! is drawn in its base colors: the map grid's shader lights it with the visual light
//! (`render::lighting`), composited after the map is placed and before free glyphs are lit.
//! [`dev`] adds the developer overlays (reveal mode is the one exception that reads ground truth).

mod controls_screen;
mod dev;
mod effects;
mod glyphs;
mod look;
mod menu;
mod motion;
mod settings_screen;

pub use menu::{Menu, OpenPause, ScreenName};

pub use dev::DevOverlays;

use crate::game::{Game, SimSet};
use crate::input::{Bindings, InputMode};
use crate::render::anim::{AnimTable, TerrainAnim};
use crate::render::console::{Cell, Console};
use crate::render::grid::MapGrid;
use crate::render::{ConsoleLayout, Effects, VisualTime, lighting};
use bevy::prelude::*;
use motion::Motion;
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::perception::{CellView, PerceptionView};

const MESSAGE_ROWS: i32 = 2;
const STATUS_ROWS: i32 = 1;

const TEXT: Rgb = Rgb::new(0xd8, 0xd4, 0xc8);
/// Secondary text. Every UI text color keeps at least 4.5:1 contrast on the panel and on black.
const TEXT_DIM: Rgb = Rgb::new(0x8e, 0x8a, 0x82);
const ACCENT: Rgb = Rgb::new(0xe0, 0xb0, 0x50);
const PANEL_BG: Rgb = Rgb::new(0x12, 0x12, 0x18);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motion>()
            .init_resource::<DevOverlays>()
            .add_message::<OpenPause>()
            .init_resource::<effects::Bursts>()
            .add_systems(
                Update,
                (
                    (
                        (motion::track, effects::track),
                        (draw, draw_map_layer),
                        lighting::update,
                        effects::emit,
                        glyphs::emit,
                    )
                        .chain()
                        .after(SimSet)
                        .run_if(resource_exists::<Game>),
                    menu::run.run_if(resource_exists::<Menu>).after(draw),
                    menu::open_pause.run_if(resource_exists::<Game>),
                ),
            );
    }
}

fn draw(
    game: Res<Game>,
    mode: Res<InputMode>,
    layout: Res<ConsoleLayout>,
    bindings: Res<Bindings>,
    overlays: Res<DevOverlays>,
    grid: Res<MapGrid>,
    mut cursor_origin: Local<Option<IVec2>>,
    mut console: ResMut<Console>,
) {
    // The look cursor moves with the map, but MapGrid is rewritten every frame, so watch only the
    // origin the cursor depends on. Redrawing on every frame would wipe overlays drawn after this
    // system, such as the pause menu (#56).
    let cursor_moved =
        matches!(*mode, InputMode::Look(_)) && *cursor_origin != Some(grid.level_origin);
    *cursor_origin = Some(grid.level_origin);
    if !(game.is_changed()
        || cursor_moved
        || mode.is_changed()
        || layout.is_changed()
        || bindings.is_changed()
        || overlays.is_changed())
    {
        return;
    }
    console.clear();
    let rows = console.rows() as i32;
    draw_messages(&mut console, &game, *mode);
    draw_status(&mut console, &game, rows - 1);
    dev::draw_indicator(&mut console, *overlays);
    if let InputMode::Look(p) = *mode
        && let Some(area) = map_area(console.cols() as i32, console.rows() as i32)
    {
        let (x, y) = (p.x - grid.level_origin.x, p.y - grid.level_origin.y);
        if (0..area.w).contains(&x) && (0..area.h).contains(&y) {
            look::draw_cursor(
                &mut console,
                area.x + x,
                area.y + y,
                &game.world.perception(),
                p,
            );
        }
    }
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
    overlays: Res<DevOverlays>,
    effects: Res<Effects>,
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
    let changed = game.is_changed()
        || anims.is_changed()
        || layout.is_changed()
        || overlays.is_changed()
        || effects.is_changed();
    if !changed && drawn.as_ref() == Some(&key) {
        return;
    }
    // One margin cell right and below, so a sub-cell offset never shows a gap.
    grid.console.resize(area.w as u32 + 1, area.h as u32 + 1);
    grid.console.clear();
    draw_map(&mut grid, &view, &anims, &key.covered);
    if effects.glow || effects.distortion {
        effects::draw_cell_fx(&mut grid, &view);
    }
    dev::draw_map_overlays(&mut grid, &game.world, *overlays);
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
/// terrain under them but not its glyph (`covered`). Only visible terrain animates. Visible
/// terrain keeps its base colors (the shader lights it); remembered terrain is dimmed here.
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
                    Cell {
                        ch: def.glyph,
                        fg: def.fg,
                        bg: def.bg,
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

/// Remembered terrain: dim and desaturated, so it never reads as "in view" (visible terrain in
/// the dark stays a little brighter; see `render::lighting::TERRAIN_FLOOR`).
fn remembered(c: Rgb) -> Rgb {
    let luma = (u32::from(c.r) * 54 + u32::from(c.g) * 183 + u32::from(c.b) * 19) / 256;
    let f = |v: u8| ((u32::from(v) + luma) / 2 * 35 / 100) as u8;
    Rgb::new(f(c.r), f(c.g), f(c.b))
}

fn draw_messages(console: &mut Console, game: &Game, mode: InputMode) {
    if let InputMode::Direction(_) = mode {
        console.print(1, 0, "In which direction? (Esc to cancel)", ACCENT);
        return;
    }
    if let InputMode::Look(p) = mode {
        console.print(1, 0, &look::describe(&game.world.perception(), p), TEXT);
        console.print(
            1,
            1,
            "Looking: direction keys move, Esc or x to stop.",
            TEXT_DIM,
        );
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
        console.print(1, row as i32, &msg.display(), color);
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
        Some(true) => Some("Lantern lit".to_string()),
        Some(false) => Some("Lantern out".to_string()),
        None => None,
    };
    // Display order; the number is the drop priority (higher drops first when space runs out).
    let segments: Vec<(u8, String)> = [
        Some((0, view.level_name().to_string())),
        Some((1, format!("Turn {}", world.turn()))),
        Some((5, format!("{},{}", pos.x, pos.y))),
        Some((2, format!("Light {light}"))),
        Some((3, format!("{} vision", view.vision().name))),
        lantern.map(|l| (4, l)),
        Some((6, format!("Seed {}", world.seed()))),
    ]
    .into_iter()
    .flatten()
    .collect();
    // The controls hint matters most to new players, so it keeps its place unless the window is
    // very narrow; status segments give way around it.
    let hint = "? controls";
    let hint_w = hint.chars().count() as i32;
    let show_hint = cols >= 50;
    let reserved = if show_hint { hint_w + 3 } else { 0 };
    let width = (cols - 2 - reserved).max(0) as usize;
    console.print(1, y, &fit_segments(&segments, width), TEXT);
    if show_hint {
        console.print(cols - hint_w - 1, y, hint, TEXT_DIM);
    }
}

/// Joins `segments` with separators, dropping the highest-priority-number segments first until
/// the line fits in `width` characters. Kept segments stay in their original order.
fn fit_segments(segments: &[(u8, String)], width: usize) -> String {
    const SEP: &str = "  ·  ";
    let mut keep: Vec<bool> = vec![true; segments.len()];
    loop {
        let line = segments
            .iter()
            .zip(&keep)
            .filter(|(_, k)| **k)
            .map(|((_, text), _)| text.as_str())
            .collect::<Vec<_>>()
            .join(SEP);
        if line.chars().count() <= width {
            return line;
        }
        let drop = segments
            .iter()
            .enumerate()
            .filter(|(i, _)| keep[*i])
            .max_by_key(|(i, (priority, _))| (*priority, *i))
            .map(|(i, _)| i);
        match drop {
            Some(i) if keep.iter().filter(|k| **k).count() > 1 => keep[i] = false,
            // A single segment that is still too long is cut at the edge.
            _ => return line.chars().take(width).collect(),
        }
    }
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
    fn ui_text_has_readable_contrast() {
        use crate::settings::palette::contrast_ratio;
        for (name, fg) in [("TEXT", TEXT), ("TEXT_DIM", TEXT_DIM), ("ACCENT", ACCENT)] {
            for bg in [PANEL_BG, Rgb::BLACK] {
                let ratio = contrast_ratio(fg, bg);
                assert!(ratio >= 4.5, "{name} on {bg:?} is only {ratio:.2}:1");
            }
        }
    }

    #[test]
    fn status_segments_drop_by_priority_and_keep_order() {
        let segs: Vec<(u8, String)> = vec![
            (0, "The Hub".into()),
            (1, "Turn 12".into()),
            (5, "7,15".into()),
            (2, "Light 366".into()),
            (6, "Seed 1".into()),
        ];
        let full = fit_segments(&segs, 200);
        assert_eq!(
            full,
            "The Hub  ·  Turn 12  ·  7,15  ·  Light 366  ·  Seed 1"
        );
        let narrow = fit_segments(&segs, 34);
        assert_eq!(narrow, "The Hub  ·  Turn 12  ·  Light 366");
        assert_eq!(fit_segments(&segs, 4), "The ");
    }

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
        let anims = AnimTable::new(content, |_| 0, true, Default::default());
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
}
