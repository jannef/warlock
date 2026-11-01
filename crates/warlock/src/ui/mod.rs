//! Glyph UI: the map viewport, message line, status bar, and the modal screens (spec §6).
//!
//! Everything shown about the world comes from `World::perception()`. Terrain and the top object
//! of each pile go into the map grid layer, creatures and glyphs that outgrow their cell into the
//! free-glyph layer ([`glyphs`]), and text and panels into the UI console on top. [`motion`]
//! moves the camera and slides creatures between cells; [`effects`] adds particles, glow, heat
//! haze, and fades; [`missiles`] flies shots and throws; [`rain`] lets it rain under the open
//! sky; [`sounds`] marks where unseen sounds were heard; [`target`] aims shots and throws (the
//! line of fire and its cursor); [`blood`] stains the cells blood lies on. Terrain and objects are
//! drawn in their base colors: the map grid's shader lights them with the visual light
//! (`render::lighting`), composited after the map is placed and before free glyphs are lit.
//! [`dev`] adds the developer overlays (reveal mode is the one exception that reads ground truth).
//!
//! The message line shows this turn's messages, a page at a time at `--more--` ([`more`]).
//!
//! Menus, character creation ([`chargen_screen`]), settings, controls, the `?` help, the
//! inventory, choosing what to pick up or drop ([`object_screen`]), the equipment screen and its
//! choice of what goes in a slot ([`equipment_screen`]), the character screen
//! ([`character_screen`]), the weapon skills screen ([`weapon_skills_screen`]), the message
//! history ([`messages_screen`]), the dialogue screen ([`dialogue_screen`]), and the death screen
//! ([`death_screen`]) are screens on one stack
//! ([`screens`]): opened by [`Screens::open`], fed key presses by `input::route_keys`, and drawn
//! by [`draw`], the one system that writes the UI console.

mod abilities_screen;
mod blood;
mod character_screen;
mod characters_screen;
mod chargen_screen;
mod chargen_talents_screen;
mod controls_screen;
mod death_screen;
mod describe_screen;
mod dev;
mod dialog_screen;
mod dialogue_screen;
mod effects;
mod equipment_screen;
mod files_screen;
mod glyphs;
mod info_tabs;
mod level_up_screen;
mod look;
mod magic_screen;
mod menu;
mod messages_screen;
mod missiles;
pub(crate) mod more;
mod motion;
mod name_screen;
mod new_run_screen;
mod object_screen;
mod paged_list;
mod quest_screen;
mod rain;
#[cfg(test)]
mod screen_snapshots;
mod screens;
mod settings_screen;
mod sign_screen;
mod skills_screen;
mod sounds;
mod spells;
#[cfg(test)]
mod stale_list_tests;
mod story_screen;
mod talents_screen;
mod target;
mod weapon_skills_screen;

pub use dialog_screen::Dialog;
pub use menu::{Press, quit_dialog};
pub use screens::{MenuChoice, ScreenAction, ScreenName, ScreenStack, Screens};

pub use dev::DevOverlays;
pub use target::{Aim, AimKind, LastTarget};

use crate::game::{Game, SimSet};
use crate::input::{Bindings, InputMode, KeyCommand, route_keys};
use crate::render::anim::{AnimTable, TerrainAnim};
use crate::render::console::{Cell, Console};
use crate::render::grid::MapGrid;
use crate::render::{ConsoleLayout, Effects, VisualTime, lighting, shadows};
use crate::settings::{Clock, Settings, TemperatureUnit};
use bevy::prelude::*;
use missiles::Missiles;
use motion::Motion;
use spells::Spells;
use warlock_core::Pos;
use warlock_core::World;
use warlock_core::ability::Ability;
use warlock_core::burden::Burden;
use warlock_core::color::Rgb;
use warlock_core::content::{MoonDef, ObjectId};
use warlock_core::hunger::Hunger;
use warlock_core::light::MoonPhase;
use warlock_core::perception::{CellView, PerceptionView};
use warlock_core::time::GameTime;

const MESSAGE_ROWS: i32 = 2;
const STATUS_ROWS: i32 = 1;

const TEXT: Rgb = Rgb::new(0xd8, 0xd4, 0xc8);
/// Secondary text. Every UI text color keeps at least 4.5:1 contrast on the panel and on black.
const TEXT_DIM: Rgb = Rgb::new(0x8e, 0x8a, 0x82);
const ACCENT: Rgb = Rgb::new(0xe0, 0xb0, 0x50);
const PANEL_BG: Rgb = Rgb::new(0x12, 0x12, 0x18);
/// The names of magic items (spec §8 "Magic items"): a soft blue, as in Path of Exile...
const MAGIC: Rgb = Rgb::new(0x96, 0xaa, 0xe6);
/// ...and of rare ones, a pale yellow (paler than the accent, so a selection still stands out).
const RARE: Rgb = Rgb::new(0xe6, 0xd8, 0x8c);

/// The color of an item's name by its rarity, which the player always sees: `normal` for a
/// normal one. The object's glyph keeps its own color.
fn rarity_color(rarity: warlock_core::content::Rarity, normal: Rgb) -> Rgb {
    match rarity {
        warlock_core::content::Rarity::Normal => normal,
        warlock_core::content::Rarity::Magic => MAGIC,
        warlock_core::content::Rarity::Rare => RARE,
    }
}
/// The status bar's temperature below freezing: a cool hint (the number says it all).
const COLD: Rgb = Rgb::new(0xa8, 0xc8, 0xe8);
/// The status bar's temperature from [`HOT_TENTHS`] up: a warm hint.
const HOT: Rgb = Rgb::new(0xe8, 0xb8, 0x94);
/// Hot enough for the warm hint: 30 °C, in tenths.
const HOT_TENTHS: i32 = 300;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motion>()
            .init_resource::<Missiles>()
            .init_resource::<Spells>()
            .init_resource::<DevOverlays>()
            .init_resource::<ScreenStack>()
            .add_message::<MenuChoice>()
            .init_resource::<effects::Bursts>()
            .init_resource::<sounds::SoundMarkers>()
            .add_systems(Startup, screens::open_startup_screens)
            .add_systems(
                Update,
                (
                    screens::apply_choice.after(route_keys).before(SimSet),
                    controls_screen::receive_keymap.before(draw),
                    (
                        (
                            motion::track,
                            effects::track,
                            missiles::track,
                            spells::track,
                            sounds::track,
                        ),
                        draw_map_layer,
                        (missiles::lights, spells::lights),
                        shadows::update,
                        lighting::update,
                        effects::emit,
                        rain::emit,
                        sounds::emit,
                        glyphs::emit,
                        (missiles::emit, target::emit_path),
                        // Spell effects are light themselves: on top of everything on the map.
                        spells::emit,
                    )
                        .chain()
                        .after(SimSet)
                        .run_if(resource_exists::<Game>),
                    (
                        screens::refresh,
                        screens::on_run_start,
                        screens::converse,
                        screens::on_death,
                        screens::on_level_up,
                    )
                        .chain()
                        .after(SimSet)
                        .before(draw)
                        .run_if(resource_exists::<Game>),
                    more::wait
                        .after(SimSet)
                        .before(draw)
                        .run_if(resource_exists::<Game>),
                    draw.after(draw_map_layer).after(screens::apply_choice),
                ),
            );
    }
}

/// The one draw pass of the UI console: clears it, draws the gameplay UI when a run exists, then
/// the top screen of the stack over it. It redraws only when something it draws changed, and
/// nothing else writes the console, so nothing drawn here can be wiped by another system.
fn draw(
    game: Option<Res<Game>>,
    stack: Res<ScreenStack>,
    mode: Res<InputMode>,
    layout: Res<ConsoleLayout>,
    settings: Res<Settings>,
    bindings: Res<Bindings>,
    overlays: Res<DevOverlays>,
    grid: Res<MapGrid>,
    mut drawn: Local<Option<(bool, Option<IVec2>)>>,
    mut console: ResMut<Console>,
) {
    // Whether a run exists, and in look mode (or aiming) the map origin the cursor is drawn
    // relative to. The map grid is rewritten every frame, so only that origin is watched (#56).
    let look = matches!(*mode, InputMode::Look(_) | InputMode::Aim(_)).then_some(grid.level_origin);
    let key = (game.is_some(), look);
    let changed = drawn.replace(key) != Some(key)
        || game.as_ref().is_some_and(|game| game.is_changed())
        || stack.is_changed()
        || mode.is_changed()
        || layout.is_changed()
        || settings.is_changed()
        || bindings.is_changed()
        || overlays.is_changed();
    if !changed {
        return;
    }
    console.clear();
    if let Some(game) = &game {
        draw_game(
            &mut console,
            game,
            *mode,
            &grid,
            *overlays,
            &bindings,
            &settings,
        );
    }
    stack.draw(&mut console, &settings, &bindings);
}

/// The gameplay UI: message line (or the current prompt), status bar (the time on the
/// settings' clock, the temperature in their unit), dev indicator, and the look cursor.
fn draw_game(
    console: &mut Console,
    game: &Game,
    mode: InputMode,
    grid: &MapGrid,
    overlays: DevOverlays,
    bindings: &Bindings,
    settings: &Settings,
) {
    let rows = console.rows() as i32;
    draw_messages(console, game, mode, bindings, settings.effects.blood);
    let hint = controls_hint(bindings);
    let (clock, unit) = (settings.clock, settings.temperature);
    draw_status(console, game, rows - 1, (clock, unit), &hint);
    dev::draw_indicator(console, overlays);
    let cursor = match mode {
        InputMode::Look(p) => Some(p),
        InputMode::Aim(aim) => Some(aim.at),
        _ => None,
    };
    if let Some(p) = cursor
        && let Some(area) = map_area(console.cols() as i32, console.rows() as i32)
    {
        let (x, y) = (p.x - grid.level_origin.x, p.y - grid.level_origin.y);
        if (0..area.w).contains(&x) && (0..area.h).contains(&y) {
            let view = game.world.perception();
            let at = (area.x + x, area.y + y);
            match mode {
                InputMode::Aim(aim) => {
                    let line = aim.line(&view);
                    target::draw_cursor(console, at, &view, p, &line);
                }
                _ => look::draw_cursor(console, at.0, at.1, &view, p),
            }
        }
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
    /// What missiles still in flight will lie as, where: not drawn yet.
    arriving: Vec<(Pos, ObjectId)>,
    /// The line of fire while aiming: cells whose glyph gives way to its dots.
    path: Vec<Pos>,
}

/// Places the map grid under the camera and redraws it when the view changed.
fn draw_map_layer(
    game: Res<Game>,
    console: Res<Console>,
    layout: Res<ConsoleLayout>,
    motion: Res<Motion>,
    missiles: Res<Missiles>,
    spells: Res<Spells>,
    mode: Res<InputMode>,
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
    // A spell effect landing near the player shakes the camera by whole pixels.
    let shake = if effects.screen_shake {
        spells.shake(visual.seconds, layout.scale)
    } else {
        Vec2::ZERO
    };
    let camera = motion.camera(visual.seconds) + shake / cell;
    let (level_origin, offset) = motion::camera_split(camera, cell);
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
        arriving: missiles.arriving(visual.seconds),
        path: target::path_cells(&view, *mode),
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
    draw_map(&mut grid, &view, &anims, &key, effects.blood);
    if effects.glow || effects.distortion {
        effects::draw_cell_fx(&mut grid, &view, &key.arriving);
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

/// Draws terrain into the map grid, with each known trap, else the top object of each known
/// pile, in place of the terrain's glyph (over its background, lit like terrain): a known trap
/// is a danger, so it shows over what lies on it. Creatures are on the free-glyph
/// layer: the grid keeps the terrain under them but not its glyph (`covered`); so do cells the
/// line of fire is drawn over while aiming (`path`, its dots are free glyphs too), unless an
/// object or a trap lies there. What a missile still in flight lands as (`arriving`) is not drawn
/// until it arrives. Only visible terrain without objects or traps animates. Visible cells keep
/// their base colors (the shader lights them); remembered ones, objects and traps included, are
/// dimmed here. Blood the player knows of tints its cell first ([`blood::stained`]), unless
/// `show_blood` is off.
fn draw_map(
    grid: &mut MapGrid,
    view: &PerceptionView<'_>,
    anims: &AnimTable,
    key: &MapKey,
    show_blood: bool,
) {
    let covered = &key.covered;
    let content = view.content();
    let (ox, oy) = (grid.level_origin.x, grid.level_origin.y);
    grid.free_terrain.clear();
    for sy in 0..grid.console.rows() as i32 {
        for sx in 0..grid.console.cols() as i32 {
            let p = Pos::new(ox + sx, oy + sy);
            let seen = view.cell(p);
            let pile = match view.objects(p) {
                [top, rest @ ..] if top.count == 1 && key.arriving.contains(&(p, top.object)) => {
                    rest
                }
                pile => pile,
            };
            let top = pile
                .first()
                .map(|s| (content.object(s.object).glyph, view.lore().fg(s)));
            // A chest takes its band's colour, and keeps its glyph over a known trap (spec §8
            // "Chests").
            let chest = view.chest_at(p);
            let band = chest.and_then(|c| content.chests().bands.get(c.band));
            let trap = (view.trap(p).filter(|_| chest.is_none())).map(|t| content.trap(t));
            let stain = view.blood(p).filter(|_| show_blood);
            let glyph = |t| {
                let def = content.terrain(t);
                let (ch, fg) = match (trap, top) {
                    (Some(trap), _) => (trap.glyph, trap.fg),
                    (None, Some(o)) => o,
                    (None, None) => (def.glyph_at(p), band.map_or(def.fg, |b| b.fg)),
                };
                // An object or a trap lies in the blood: only the background takes it.
                let over = top.is_some() || trap.is_some();
                let (fg, bg) = match stain {
                    Some(stain) => {
                        let wall = !def.transparent;
                        blood::stained((fg, def.bg), stain, view.time(), wall, over)
                    }
                    None => (fg, def.bg),
                };
                (ch, fg, bg)
            };
            let cell = match seen {
                CellView::Unknown => continue,
                CellView::Remembered(t) => {
                    let (ch, fg, bg) = glyph(t);
                    Cell {
                        ch,
                        fg: remembered(fg),
                        bg: remembered(bg),
                    }
                }
                CellView::Visible(t) => {
                    let (ch, fg, bg) = glyph(t);
                    Cell { ch, fg, bg }
                }
            };
            grid.console.put(sx, sy, cell);
            if let CellView::Visible(t) = seen
                && top.is_none()
                && trap.is_none()
            {
                match anims.terrain(t) {
                    Some(TerrainAnim::Grid { .. }) => {
                        grid.console.animate(sx, sy, anims.cell(t, p, view.flow(p)))
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
    for &p in &key.path {
        if view.objects(p).is_empty() && view.trap(p).is_none() {
            let at = grid.to_console(p);
            grid.console.clear_glyph(at.x, at.y);
        }
    }
}

/// Remembered terrain: dim and desaturated, so it never reads as "in view" (visible terrain in
/// the dark stays a little brighter; see `render::lighting::TERRAIN_FLOOR`).
fn remembered(c: Rgb) -> Rgb {
    let luma = (u32::from(c.r) * 54 + u32::from(c.g) * 183 + u32::from(c.b) * 19) / 256;
    let f = |v: u8| ((u32::from(v) + luma) / 2 * 35 / 100) as u8;
    Rgb::new(f(c.r), f(c.g), f(c.b))
}

/// The message line, or the prompt of the input mode; look mode and aiming describe the cell
/// under the cursor (its blood only if `blood` is shown). Outside the prompts it shows this
/// turn's messages, a page at a time at `--more--` ([`more`]).
fn draw_messages(
    console: &mut Console,
    game: &Game,
    mode: InputMode,
    bindings: &Bindings,
    blood: bool,
) {
    if let InputMode::Direction(command) = mode {
        let prompt = match command {
            KeyCommand::UseTool => "Dig in which direction? (Esc to cancel)",
            KeyCommand::Chat => "Talk in which direction? (Esc to cancel)",
            _ => "In which direction? (Esc to cancel)",
        };
        console.print(1, 0, prompt, ACCENT);
        return;
    }
    if let InputMode::Ability(ability) = mode {
        let prompt = match ability {
            Ability::Disarm => "Disarm in which direction? (Esc to cancel)",
            _ => "In which direction? (Esc to cancel)",
        };
        console.print(1, 0, prompt, ACCENT);
        return;
    }
    if mode == InputMode::Rest {
        console.print(1, 0, &rest_prompt(bindings), ACCENT);
        return;
    }
    if let InputMode::Aim(aim) = mode {
        let view = game.world.perception();
        let line = aim.line(&view);
        let mut about = look::describe(&view, aim.at, blood);
        if let Some(status) = line.status() {
            about = format!("{about} {status}");
        }
        console.print(1, 0, &about, TEXT);
        let key = |command| {
            (bindings.keys_of(command).first()).map_or_else(|| "?".to_string(), |k| k.display())
        };
        let (confirm, missile) = match aim.kind {
            AimKind::Fire => {
                let ammo = view.ready_to_fire().map(|(_, item, _)| item);
                (KeyCommand::Missile, ammo)
            }
            AimKind::Throw(item) | AimKind::Zap(item) => (KeyCommand::Missile, Some(item)),
            AimKind::Read(item) => (KeyCommand::Read, Some(item)),
            AimKind::Ability(_) => (KeyCommand::Loose, None),
        };
        let content = view.content();
        let ready = match aim.kind {
            AimKind::Ability(ability) => ability.name(content).to_string(),
            _ => (missile.and_then(|i| view.inventory().get(i))).map_or_else(String::new, |s| {
                let named = view.lore().describe_object(s.object, 1);
                // A wand the player knows says what it does in their hands (its rung).
                match view
                    .wand_does(s)
                    .filter(|_| matches!(aim.kind, AimKind::Zap(_)))
                {
                    Some(does) => format!("{named} ({does})"),
                    None => named,
                }
            }),
        };
        let next = key(KeyCommand::NextTarget);
        let prompt = aim.prompt(&ready, (&key(KeyCommand::Loose), &key(confirm), &next));
        console.print(1, 1, &prompt, ACCENT);
        return;
    }
    if let InputMode::Look(p) = mode {
        let about = look::describe(&game.world.perception(), p, blood);
        console.print(1, 0, &about, TEXT);
        let stop = match bindings.key_name(KeyCommand::Look) {
            Some(key) => format!("Esc or {key}"),
            None => "Esc".to_string(),
        };
        let prompt = format!("Looking: direction keys move, {stop} to stop.");
        console.print(1, 1, &prompt, TEXT_DIM);
        return;
    }
    more::draw(console, game, mode, bindings);
}

/// A key a message names by its command's id ([`warlock_core::messages::key`]), as bound: `?`
/// for `help` with the default keys (`Welcome to Showcase. Press ? for controls.`). Without a key,
/// help points at Esc, whose pause menu holds Controls (as [`controls_hint`] does), and any
/// other command is named.
fn message_key(bindings: &Bindings, id: &str) -> String {
    let Some(command) = KeyCommand::from_id(id) else {
        return id.to_string();
    };
    bindings.key_name(command).unwrap_or_else(|| match command {
        KeyCommand::Help => "Esc".to_string(),
        other => other.name(),
    })
}

/// The rest prompt, naming each choice's key as bound ([`crate::input::REST_CHOICES`]):
/// `Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)` with the default keys, the
/// key in parentheses where a word starts with it, else after the words (`hour (F1)`). A choice
/// without a key is left out.
fn rest_prompt(bindings: &Bindings) -> String {
    let choices: Vec<String> = crate::input::REST_CHOICES
        .iter()
        .filter_map(|(command, _, words)| {
            let key = bindings.key_name(*command)?;
            let starts = std::iter::once(0).chain(words.match_indices(' ').map(|(i, _)| i + 1));
            let single = key.chars().count() == 1;
            let start = starts
                .filter(|_| single)
                .find(|&at| words[at..].starts_with(&key));
            Some(match start {
                Some(at) => {
                    let rest = &words[at + key.len()..];
                    format!("{}({key}){rest}", &words[..at])
                }
                None => format!("{words} ({key})"),
            })
        })
        .collect();
    format!("Rest: {}? (Esc to cancel)", choices.join(", "))
}

/// The status bar's reminder of where the keys are: the help key, else the pause menu (which
/// holds Controls).
fn controls_hint(bindings: &Bindings) -> String {
    match bindings.key_name(KeyCommand::Help) {
        Some(key) => format!("{key} controls"),
        None => "Esc menu".to_string(),
    }
}

fn draw_status(
    console: &mut Console,
    game: &Game,
    y: i32,
    (clock, unit): (Clock, TemperatureUnit),
    hint: &str,
) {
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
    let segments = status_segments(&game.world, clock, unit);
    // The controls hint matters most to new players, so it keeps its place unless the window is
    // very narrow; status segments give way around it.
    let hint_w = hint.chars().count() as i32;
    let show_hint = cols >= 50;
    let reserved = if show_hint { hint_w + 3 } else { 0 };
    let width = (cols - 2 - reserved).max(0) as usize;
    let line = fit_segments(&segments, width);
    console.print(1, y, &line, TEXT);
    // A subtle hint of cold or heat on the temperature, when it is shown: the number says it.
    let temperature = game.world.perception().temperature();
    let tint = match temperature.tenths() {
        t if t < 0 => Some(COLD),
        t if t >= HOT_TENTHS => Some(HOT),
        _ => None,
    };
    let text = unit.show(temperature);
    if let (Some(tint), Some(at)) = (tint, line.find(&text)) {
        let x = line[..at].chars().count() as i32;
        console.print(1 + x, y, &text, tint);
    }
    if show_hint {
        console.print(cols - hint_w - 1, y, hint, TEXT_DIM);
    }
}

/// The status bar's segments for `world`, in display order, the time on `clock` and the
/// temperature in `unit`.
fn status_segments(world: &World, clock: Clock, unit: TemperatureUnit) -> Vec<Segment> {
    let view = world.perception();
    let pos = world.player_pos();
    let time = world.time();
    let light = view.light(pos).map_or(0, |l| l.brightness());
    let content = view.content();
    // Display order; the number is the drop priority (higher drops first when space runs out;
    // of equal ones the later first). The moon phase is cut down to its glyph and the date to
    // its day and month before they are dropped. The weather shows only under the open sky,
    // where the player knows it, and there it matters more than the date: it changes the light
    // and how far one sees. Hit points matter most after the level, and mana right after them,
    // then each alternative resource the player holds (`Rage 7`, `Focus 7/10`; spec §8
    // "Resources"), giving way before mana: they outlast the time, and so do the status effects the player suffers (by name, the most
    // important first: the last gives way first), Dazzled (strong light on light-blind eyes,
    // spec §8 "Character creation"), hunger (shown unless the player is Not hungry), and the burden (shown
    // when there is one), which slows everything the player does; hunger outlasts the burden. The
    // character level (`Lv 3`) comes after them and goes before them, but after the time. The
    // temperature where the player is (always, underground too) sits after the weather; it
    // outlasts the turn, the position, the seed, and the moon, and gives way before the date.
    let calendar = content.calendar();
    let (hp, max_hp) = view.own_hp();
    let (mp, max_mp) = view.own_mana();
    let burden = view.own_load().burden();
    let hunger = view.own_hunger().filter(|h| *h != Hunger::NotHungry);
    let resources = (view.own_resources().into_iter()).map(|r| {
        let def = content.resource(r.resource);
        let text = match def.show_max {
            true => format!("{} {}/{}", def.short, r.value, r.max),
            false => format!("{} {}", def.short, r.value),
        };
        Segment::new(1, text)
    });
    let effects = (view.own_effects()).map(|s| Segment::new(1, content.status(s).name.clone()));
    let stance = view.own_stance();
    let head = [
        Segment::new(0, view.level_name()),
        Segment::new(1, format!("HP {hp}/{max_hp}")),
        Segment::new(1, format!("MP {mp}/{max_mp}")),
    ];
    let rest = [
        (view.dazzled()).then(|| Segment::new(1, "Dazzled")),
        hunger.map(|h| Segment::new(1, h.name())),
        (burden != Burden::Unburdened).then(|| Segment::new(1, burden.name())),
        Some(Segment::new(
            1,
            format!("Lv {}", view.own_experience().level),
        )),
        (!stance.is_neutral()).then(|| Segment::new(1, content.stance_def(stance).name.clone())),
        view.region().map(|r| Segment::new(4, r)),
        Some(Segment::new(1, clock.format(time))),
        Some(Segment::new(3, calendar.full_date(time)).or_short(calendar.date(time))),
        moon_status(content.moons(), time)
            .map(|(text, glyphs)| Segment::new(5, text).or_short(glyphs)),
        view.weather().map(|w| Segment::new(2, w.name())),
        Some(Segment::new(4, unit.show(view.temperature()))),
        Some(Segment::new(6, format!("Turn {}", world.turn()))),
        Some(Segment::new(10, format!("{},{}", pos.x, pos.y))),
        Some(Segment::new(7, format!("Light {light}"))),
        Some(Segment::new(8, format!("{} vision", view.vision().name))),
        Some(Segment::new(11, format!("Seed {}", world.seed()))),
    ];
    (head.into_iter())
        .chain(resources)
        .chain(effects)
        .chain(rest.into_iter().flatten())
        .collect()
}

/// The moons' phases for the status bar, and just their glyphs for a narrow window: the glyph
/// and name of a single moon's phase (`◑ First quarter`), each moon's name and glyph when there
/// are several (`Moon ●  Ilse ☽`); `None` without moons.
fn moon_status(moons: &[MoonDef], time: GameTime) -> Option<(String, String)> {
    let glyph = |m: &MoonDef| phase_glyph(MoonPhase::of(m.phase_at(time)));
    let glyphs = moons.iter().map(glyph).collect::<String>();
    let text = match moons {
        [] => return None,
        [moon] => format!(
            "{} {}",
            glyph(moon),
            MoonPhase::of(moon.phase_at(time)).name()
        ),
        _ => (moons.iter())
            .map(|m| format!("{} {}", m.name, glyph(m)))
            .collect::<Vec<_>>()
            .join("  "),
    };
    Some((text, glyphs))
}

/// A glyph for a moon phase, its lit part drawn in ink (UI text is light on dark) and lit on the
/// right while waxing: `○` new, `☽` waxing crescent, `◑` first quarter and waxing gibbous, `●`
/// full, `◐` waning gibbous and last quarter, `☾` waning crescent.
fn phase_glyph(phase: MoonPhase) -> char {
    match phase {
        MoonPhase::New => '○',
        MoonPhase::WaxingCrescent => '☽',
        MoonPhase::FirstQuarter | MoonPhase::WaxingGibbous => '◑',
        MoonPhase::Full => '●',
        MoonPhase::WaningGibbous | MoonPhase::LastQuarter => '◐',
        MoonPhase::WaningCrescent => '☾',
    }
}

/// One segment of the status bar.
struct Segment {
    /// Drop priority: higher gives way first when space runs out.
    priority: u8,
    text: String,
    /// What it is cut down to before it is dropped, if anything.
    short: Option<String>,
}

impl Segment {
    fn new(priority: u8, text: impl Into<String>) -> Self {
        Self {
            priority,
            text: text.into(),
            short: None,
        }
    }

    fn or_short(self, short: impl Into<String>) -> Self {
        Self {
            short: Some(short.into()),
            ..self
        }
    }

    /// The text in `form`: 0 dropped, 1 short, 2 full.
    fn at(&self, form: u8) -> Option<&str> {
        match form {
            2 => Some(&self.text),
            1 => self.short.as_deref(),
            _ => None,
        }
    }
}

/// Joins `segments` with separators so the line fits in `width` characters: the segment with
/// the highest priority number gives way first, cut down to its short form if it has one, else
/// dropped. Then, most important first, segments take back the room that is left, until one
/// that was dropped does not fit. Kept segments stay in their original order.
fn fit_segments(segments: &[Segment], width: usize) -> String {
    const SEP: &str = "  ·  ";
    let line = |forms: &[u8]| {
        (segments.iter().zip(forms))
            .filter_map(|(s, &form)| s.at(form))
            .collect::<Vec<_>>()
            .join(SEP)
    };
    let fits = |forms: &[u8]| line(forms).chars().count() <= width;
    // Shorter forms: a short form if the segment has one, else dropped.
    let shorter = |s: &Segment, form: u8| if form == 2 && s.short.is_some() { 1 } else { 0 };
    let longer = |s: &Segment, form: u8| if form == 0 && s.short.is_some() { 1 } else { 2 };
    let mut forms: Vec<u8> = vec![2; segments.len()];
    while !fits(&forms) {
        let shown = forms.iter().filter(|f| **f > 0).count();
        let give_way = (segments.iter().enumerate())
            .filter(|(i, _)| forms[*i] > 0)
            .max_by_key(|(i, s)| (s.priority, *i))
            .map(|(i, s)| (i, shorter(s, forms[i])));
        match give_way {
            Some((i, form)) if form > 0 || shown > 1 => forms[i] = form,
            // A single segment that is still too long is cut at the edge.
            _ => return line(&forms).chars().take(width).collect(),
        }
    }
    let mut order: Vec<usize> = (0..segments.len()).collect();
    order.sort_by_key(|&i| (segments[i].priority, i));
    for i in order {
        while forms[i] < 2 {
            let was = forms[i];
            forms[i] = longer(&segments[i], was);
            if !fits(&forms) {
                forms[i] = was;
                break;
            }
        }
        if forms[i] == 0 {
            break;
        }
    }
    line(&forms)
}

/// A test world: average scores (St 10 To 10, so burdened over 15 kg), standing on two chain
/// mails (22 kg).
#[cfg(test)]
pub(crate) fn hoard() -> World {
    let mut files = warlock_core::content::ContentFiles::builtin();
    files.insert(
        "levels/hoard.toml",
        "name = \"Hoard\"\nmap = '''\n#####\n#@..#\n#####\n'''\n[legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\", \
         objects = [\"chain_mail\", \"chain_mail\"] }\n",
    );
    files.insert(
        "scenarios/hoard.toml",
        "name = \"Hoard\"\nstart_level = \"hoard\"\n",
    );
    let content = warlock_core::content::Content::load(&files).expect("valid");
    World::new(std::sync::Arc::new(content), "hoard", None).expect("world")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::console::CellAnim;
    use std::sync::Arc;
    use warlock_core::content::CombatStance;
    use warlock_core::content::{Content, ContentFiles};
    use warlock_core::{Action, Dir, Positions, World};

    #[test]
    fn the_status_bar_shows_the_burden_after_the_hit_points_and_mana_and_keeps_it_late() {
        let mut world = hoard();
        let texts = |w: &World| -> Vec<String> {
            status_segments(w, Clock::H24, TemperatureUnit::Celsius)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(
            !texts(&world).iter().any(|t| t.contains("urdened")),
            "unburdened: nothing"
        );
        world.apply(&Action::PickUp(Positions::default()));
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        assert_eq!(segments[1].text, "HP 10/10");
        assert_eq!(
            segments[2].text, "MP 0/0",
            "mana right after the hit points"
        );
        assert_eq!(segments[3].text, "Burdened", "then the burden");
        // Narrow: the time gives way first, then the burden, before the mana and hit points.
        let line = "Hoard  ·  HP 10/10  ·  MP 0/0  ·  Burdened";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  ·  HP 10/10  ·  MP 0/0"
        );
        assert!(fit_segments(&segments, 66).contains("Burdened  ·  Lv 1  ·  08:"));
    }

    #[test]
    fn the_status_bar_says_dazzled_while_strong_light_dazzles_light_blind_eyes() {
        let world_as = |race: &str| {
            let mut files = ContentFiles::builtin();
            files.edit_table("races", |text| {
                format!(
                    "{text}\n[[race]]\nid = \"blind\"\nname = \"Blind\"\ndescription = \"Blind.\"\n\
                     vision = \"normal\"\nage = [20, 30]\n\
                     light_blind = {{ glare = 200, sight = 5, to_hit = -1 }}\n"
                )
            });
            files.insert(
                "levels/glare.toml",
                "name = \"Glare\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
                 [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
            );
            files.insert(
                "scenarios/glare.toml",
                "name = \"Glare\"\nstart_level = \"glare\"\n",
            );
            let content = Arc::new(Content::load(&files).expect("valid"));
            let spec = format!("{race},male,warrior").parse().expect("spec");
            World::with_character(content, "glare", None, Some(&spec)).expect("world")
        };
        let texts = |w: &World| -> Vec<String> {
            status_segments(w, Clock::H24, TemperatureUnit::Celsius)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert_eq!(
            texts(&world_as("blind"))[4],
            "Dazzled",
            "after the hit points, the mana, and a warrior's stamina"
        );
        assert!(!texts(&world_as("human")).contains(&"Dazzled".to_string()));
    }

    #[test]
    fn the_status_bar_shows_hunger_unless_not_hungry_and_keeps_it_later_than_the_burden() {
        let world_starting_at = |nutrition: i32| {
            let mut files = ContentFiles::builtin();
            let hunger = include_str!("../../../../assets/data/hunger.toml")
                .replace("start = 3600", &format!("start = {nutrition}"));
            files.insert("data/hunger.toml", &hunger);
            files.insert(
                "levels/hoard.toml",
                "name = \"Hoard\"\nmap = '''\n#####\n#@..#\n#####\n'''\n[legend]\n\
                 \"@\" = { terrain = \"floor\", spawn = \"player\", \
                 objects = [\"chain_mail\", \"chain_mail\"] }\n",
            );
            files.insert(
                "scenarios/hoard.toml",
                "name = \"Hoard\"\nstart_level = \"hoard\"\n",
            );
            let content = Arc::new(Content::load(&files).expect("valid"));
            World::new(content, "hoard", None).expect("world")
        };
        let texts = |w: &World| -> Vec<String> {
            status_segments(w, Clock::H24, TemperatureUnit::Celsius)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(
            !texts(&world_starting_at(3600))
                .iter()
                .any(|t| t.contains("ungry")),
            "not hungry: nothing"
        );
        for (nutrition, shown) in [
            (6000, "Bloated"),
            (5000, "Satiated"),
            (1000, "Hungry"),
            (500, "Weak"),
            (100, "Fainting"),
        ] {
            assert_eq!(
                texts(&world_starting_at(nutrition))[3],
                shown,
                "{nutrition}"
            );
        }
        let mut world = world_starting_at(1000);
        world.apply(&Action::PickUp(Positions::default()));
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        assert_eq!(
            texts(&world)[..5],
            ["Hoard", "HP 10/10", "MP 0/0", "Hungry", "Burdened"]
        );
        // Narrow: the time gives way first, then the burden, then hunger, before the mana, then
        // the hit points.
        let line = "Hoard  ·  HP 10/10  ·  MP 0/0  ·  Hungry  ·  Burdened";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  ·  HP 10/10  ·  MP 0/0  ·  Hungry"
        );
        assert_eq!(fit_segments(&segments, 36), "Hoard  ·  HP 10/10  ·  MP 0/0");
        assert_eq!(fit_segments(&segments, 25), "Hoard  ·  HP 10/10");
    }

    #[test]
    fn the_status_bar_shows_status_effects_after_the_hit_points_most_important_first() {
        let mut world = hoard();
        world.apply(&Action::Afflict("poisoned".into()));
        world.apply(&Action::Afflict("bleeding".into()));
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        let texts: Vec<&str> = segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(
            texts[..5],
            ["Hoard", "HP 10/10", "MP 0/0", "Bleeding", "Poisoned"]
        );
        // They give way late, the least important first, but before the mana and hit points.
        let line = "Hoard  ·  HP 10/10  ·  MP 0/0  ·  Bleeding  ·  Poisoned";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        let shorter = "Hoard  ·  HP 10/10  ·  MP 0/0  ·  Bleeding";
        assert_eq!(fit_segments(&segments, line.chars().count() - 1), shorter);
        assert_eq!(fit_segments(&segments, 20), "Hoard  ·  HP 10/10");
        assert!(fit_segments(&segments, 200).contains("Poisoned  ·  "));
    }

    #[test]
    fn the_status_bar_shows_diseased_once_a_disease_shows_never_which() {
        let mut world = hoard();
        world.apply(&Action::Infect("the_shakes".into()));
        let texts = |w: &World| -> Vec<String> {
            status_segments(w, Clock::H24, TemperatureUnit::Celsius)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(
            !texts(&world).iter().any(|t| t.contains("isease")),
            "incubating"
        );
        // The shakes show within a day, and are known a day later: still just "Diseased".
        for _ in 0..2 {
            world.apply(&Action::Rest(warlock_core::Rest::For(
                warlock_core::time::Duration::hours(24),
            )));
            assert_eq!(
                texts(&world)[..4],
                ["Hoard", "HP 10/10", "MP 0/0", "Diseased"]
            );
        }
        assert!(!texts(&world).iter().any(|t| t.contains("shakes")));
    }

    #[test]
    fn the_status_bar_shows_the_mana_left_after_the_hit_points() {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/cell.toml",
            "name = \"Cell\"\nmap = '''\n###\n#@#\n###\n'''\n[legend]\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/cell.toml",
            "name = \"Cell\"\nstart_level = \"cell\"\nmana = 12\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let mut world = World::new(content, "cell", None).expect("world");
        world.apply(&Action::Drain(7));
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        assert_eq!(segments[2].text, "MP 5/12");
    }

    #[test]
    fn the_status_bar_shows_each_resource_held_after_the_mana() {
        let mut world = hoard();
        world.apply(&Action::Resource("rage".into()));
        world.apply(&Action::Resource("focus".into()));
        world.apply(&Action::Wait);
        let texts: Vec<String> = status_segments(&world, Clock::H24, TemperatureUnit::Celsius)
            .into_iter()
            .map(|s| s.text)
            .collect();
        // Rage shows its value alone, focus its maximum too.
        assert_eq!(texts[2..5], ["MP 0/0", "Rage 11", "Focus 10/10"]);
        // Raging (the rage brings it) raises Toughness, and the maximum with it.
        let line = "Hoard  ·  HP 10/12  ·  MP 0/0  ·  Rage 11  ·  Focus 10/10";
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  ·  HP 10/12  ·  MP 0/0  ·  Rage 11",
            "the last resource gives way first, before the mana"
        );
    }

    #[test]
    fn the_status_bar_names_a_stance_that_is_not_neutral() {
        let mut world = hoard();
        let texts = |w: &World| -> Vec<String> {
            status_segments(w, Clock::H24, TemperatureUnit::Celsius)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        assert!(!texts(&world).iter().any(|t| t.contains("Neutral")));
        world.apply(&Action::SetStance(CombatStance::Defensive));
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        let at = |t: &str| segments.iter().position(|s| s.text == t);
        assert_eq!(
            at("Defensive"),
            at("Lv 1").map(|i| i + 1),
            "right after the level"
        );
        world.apply(&Action::SetStance(CombatStance::Neutral));
        assert!(!texts(&world).iter().any(|t| t == "Neutral"));
    }

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
    fn the_status_bar_shows_the_moon_phase() {
        let content = Content::builtin().expect("builtin content");
        let at = |day| GameTime::from_calendar(1, 4, day, 0, 0).expect("valid");
        let moons = content.moons();
        let text = |moons: &[MoonDef], day| moon_status(moons, at(day)).map(|(text, _)| text);
        assert_eq!(text(moons, 1).as_deref(), Some("○ New moon"));
        assert_eq!(text(moons, 8).as_deref(), Some("◑ First quarter"));
        assert_eq!(text(moons, 16).as_deref(), Some("● Full moon"));
        assert_eq!(text(moons, 27).as_deref(), Some("☾ Waning crescent"));
        assert_eq!(text(moons, 20).as_deref(), Some("◐ Waning gibbous"));
        assert_eq!(moon_status(&[], at(1)), None);
        let ilse = MoonDef {
            name: "Ilse".into(),
            period_days: 7,
            ..moons[0].clone()
        };
        let two = [moons[0].clone(), ilse];
        assert_eq!(
            moon_status(&two, at(16)),
            Some(("Moon ●  Ilse ○".into(), "●○".into()))
        );
        assert_eq!(content.calendar().full_date(at(12)), "12 Rainmoon, Year 1");
    }

    #[test]
    fn status_segments_drop_by_priority_and_keep_order() {
        let segs = vec![
            Segment::new(0, "The Hub"),
            Segment::new(1, "Turn 12"),
            Segment::new(5, "7,15"),
            Segment::new(2, "Light 366"),
            Segment::new(6, "Seed 1"),
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
    fn status_segments_shorten_before_they_drop_and_take_back_room() {
        let segs = vec![
            Segment::new(0, "Wilds"),
            Segment::new(1, "22:20"),
            Segment::new(2, "Moon ● Full moon").or_short("●"),
            Segment::new(3, "Turn 13"),
            Segment::new(4, "Seed 1"),
        ];
        let all = "Wilds  ·  22:20  ·  Moon ● Full moon  ·  Turn 13  ·  Seed 1";
        assert_eq!(fit_segments(&segs, 200), all);
        // Seed and turn give way, then the moon is cut to its glyph: room for the turn again.
        assert_eq!(fit_segments(&segs, 33), "Wilds  ·  22:20  ·  ●  ·  Turn 13");
        assert_eq!(fit_segments(&segs, 22), "Wilds  ·  22:20  ·  ●");
        assert_eq!(fit_segments(&segs, 15), "Wilds  ·  22:20");
        // The seed is short enough to fit where the turn does not, but never shows without it.
        assert_eq!(fit_segments(&segs, 32), "Wilds  ·  22:20  ·  ●");
    }

    #[test]
    fn the_status_bar_shows_the_weather_only_under_the_open_sky() {
        let content = Arc::new(Content::builtin().expect("content"));
        let texts = |world: &World| -> Vec<String> {
            status_segments(world, Clock::H24, TemperatureUnit::Celsius)
                .into_iter()
                .map(|s| s.text)
                .collect()
        };
        let mut world = World::new(content.clone(), "showcase", None).expect("starts");
        world.apply(&Action::SetWeather(warlock_core::weather::Weather::Rain));
        let names = ["Clear", "Overcast", "Rain", "Heavy rain"];
        assert!(!texts(&world).iter().any(|t| names.contains(&t.as_str())));
        let mut world = World::new(content, "courtyard", None).expect("starts");
        world.apply(&Action::SetWeather(
            warlock_core::weather::Weather::HeavyRain,
        ));
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        let at = |text: &str| segments.iter().position(|s| s.text.contains(text));
        let (weather, moon, date) = (at("Heavy rain"), at("Waxing gibbous"), at("Darkening"));
        assert_eq!(weather, moon.map(|m| m + 1), "shown after the moon");
        // It gives way after the time, before the date (cut to day and month first).
        let priority = |i: Option<usize>| segments[i.expect("shown")].priority;
        assert!(priority(weather) > 1 && priority(weather) < priority(date));
        assert_eq!(
            segments[date.expect("date")].short.as_deref(),
            Some("11 Darkening")
        );
        let line = fit_segments(&segments, 96);
        assert!(
            line.contains("Heavy rain") && line.contains("11 Darkening"),
            "{line}"
        );
    }

    #[test]
    fn the_status_bar_always_shows_the_temperature_in_the_chosen_unit() {
        let content = Arc::new(Content::builtin().expect("content"));
        // Underground too: the hub is 10 °C, after the moon (there is no weather there).
        let hub = World::new(content.clone(), "showcase", None).expect("starts");
        let segments = status_segments(&hub, Clock::H24, TemperatureUnit::Celsius);
        let at = |segments: &[Segment], text: &str| segments.iter().position(|s| s.text == text);
        let moon = segments.iter().position(|s| s.text.contains("gibbous"));
        assert_eq!(at(&segments, "10 °C"), moon.map(|m| m + 1));
        let fahrenheit = status_segments(&hub, Clock::H24, TemperatureUnit::Fahrenheit);
        assert!(at(&fahrenheit, "50 °F").is_some());
        // Outdoors, the weather's, right after the weather.
        let yard = World::new(content, "courtyard", None).expect("starts");
        let segments = status_segments(&yard, Clock::H24, TemperatureUnit::Celsius);
        let shown = TemperatureUnit::Celsius.show(yard.perception().temperature());
        let weather = segments.iter().position(|s| s.text == "Clear");
        assert_eq!(at(&segments, &shown), weather.map(|w| w + 1));
        // It outlasts the seed, the position, the turn, and the moon, and gives way before the
        // date, the time, the hit points, and the level.
        let priority = |text: &str| segments[at(&segments, text).expect(text)].priority;
        let temperature = priority(&shown);
        for later in ["Turn 0", "6,9", "Seed 1"] {
            assert!(priority(later) > temperature, "{later}");
        }
        let moon = segments
            .iter()
            .find(|s| s.text.contains("gibbous"))
            .expect("moon");
        assert!(moon.priority > temperature);
        let date = segments
            .iter()
            .find(|s| s.text.contains("Darkening"))
            .expect("date");
        assert!(date.priority < temperature);
        assert!(priority("06:30") < temperature && priority("HP 19/19") < temperature);
        // Narrow: the moon is gone before the temperature, the temperature before the date.
        let full = fit_segments(&segments, 200);
        let without = |gone: &str| {
            let width = full.chars().count() - 1;
            (5..=width)
                .rev()
                .map(|w| fit_segments(&segments, w))
                .find(|line| !line.contains(gone))
                .expect("it drops at some width")
        };
        let moonless = without("gibbous");
        assert!(moonless.contains(&shown), "{moonless}");
        let cold = without(&shown);
        assert!(cold.contains("Darkening"), "{cold}");
    }

    #[test]
    fn the_twelve_hour_clock_is_longer_but_the_status_bar_still_fits() {
        let content = Arc::new(Content::builtin().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let segments = status_segments(&world, Clock::H12, TemperatureUnit::Celsius);
        let time = (segments.iter())
            .map(|s| &s.text)
            .find(|t| t.ends_with(" AM") || t.ends_with(" PM"))
            .expect("the time");
        for width in [0, 10, 20, 25, 40, 55, 80, 120] {
            let line = fit_segments(&segments, width);
            assert!(line.chars().count() <= width, "{width}: {line}");
        }
        let line = fit_segments(&segments, 120);
        assert!(line.contains(time.as_str()), "{line}");
        // Narrow: the time gives way first, as in 24-hour form.
        assert_eq!(fit_segments(&segments, 20), "The Hub  ·  HP 18/18");
    }

    #[test]
    fn the_status_bar_shows_hit_points_and_keeps_them_late() {
        let content = Arc::new(Content::builtin().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let segments = status_segments(&world, Clock::H24, TemperatureUnit::Celsius);
        assert_eq!(segments[1].text, "HP 18/18", "right after the level");
        // Narrow: everything but the level gives way first, the time included.
        let line = fit_segments(&segments, 25);
        assert!(line.starts_with("The Hub  ·  HP 18/18"), "{line}");
        assert!(!line.contains(':'), "{line}");
        assert_eq!(fit_segments(&segments, 20), "The Hub  ·  HP 18/18");
    }

    #[test]
    fn prompts_name_the_keys_as_bound() {
        let mut b = Bindings::default();
        assert_eq!(
            rest_prompt(&b),
            "Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)"
        );
        assert_eq!(controls_hint(&b), "? controls");
        let key = |text: &str| crate::input::KeyBinding::from_text(text).unwrap();
        b.bind_primary(KeyCommand::RestHour, key("F1")).unwrap();
        b.bind_primary(KeyCommand::RestDusk, key("i")).unwrap();
        b.bind_primary(KeyCommand::RestDawn, key("u")).unwrap();
        assert_eq!(
            rest_prompt(&b),
            "Rest: hour (F1), (u)ntil dawn, until nightfall (i)? (Esc to cancel)"
        );
        b.0.retain(|(_, c)| *c != KeyCommand::Help);
        assert_eq!(controls_hint(&b), "Esc menu");
    }

    #[test]
    fn the_welcome_line_names_the_help_key_as_bound() {
        let content = Arc::new(Content::builtin().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let welcome = world.messages().iter().next().expect("a welcome");
        let shown = |b: &Bindings| welcome.display_with(|id| message_key(b, id));
        let mut b = Bindings::default();
        // The text it always had, with the default keys.
        assert_eq!(shown(&b), "Welcome to Showcase. Press ? for controls.");
        let key = crate::input::KeyBinding::from_text("F1").unwrap();
        b.bind_primary(KeyCommand::Help, key).unwrap();
        assert_eq!(shown(&b), "Welcome to Showcase. Press F1 for controls.");
        b.0.retain(|(_, c)| *c != KeyCommand::Help);
        assert_eq!(shown(&b), "Welcome to Showcase. Press Esc for controls.");
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
        let key = MapKey {
            level_origin: IVec2::ZERO,
            area: Rect2 {
                x: 0,
                y: 0,
                w: 7,
                h: 3,
            },
            covered: vec![player],
            arriving: Vec::new(),
            path: Vec::new(),
        };
        draw_map(&mut grid, &view, &anims, &key, true);

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
        draw_map(&mut grid, &view, &anims, &key, true);
        assert_eq!(grid.free_terrain, vec![(Pos::new(5, 1), altar)]);
        assert_eq!(grid.console.cells()[at(5, 1)].map(|c| c.ch), Some(' '));
    }
}
