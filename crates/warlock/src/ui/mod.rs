//! Glyph UI: the map viewport, message line, status bar, and the modal screens (spec §6).
//!
//! Everything shown about the world comes from `World::perception()`. Terrain and the top object
//! of each pile go into the map grid layer, creatures and glyphs that outgrow their cell into the
//! free-glyph layer ([`glyphs`]), and text and panels into the UI console on top. [`motion`]
//! moves the camera and slides creatures between cells; [`effects`] adds particles, glow, heat
//! haze, and fades; [`missiles`] flies shots and throws; [`rain`] lets it rain under the open
//! sky; [`sounds`] marks where unseen sounds were heard; [`target`] aims shots and throws (the
//! line of fire and its cursor). Terrain and objects are drawn
//! in their base colors: the map grid's shader lights them with the visual light
//! (`render::lighting`), composited after the map is placed and before free glyphs are lit.
//! [`dev`] adds the developer overlays (reveal mode is the one exception that reads ground truth).
//!
//! Menus, character creation ([`chargen_screen`]), settings, controls, the `?` help, the
//! inventory, choosing what to pick up or drop ([`object_screen`]), the equipment screen and its
//! choice of what goes in a slot ([`equipment_screen`]), the character screen
//! ([`character_screen`]), the weapon skills screen ([`weapon_skills_screen`]), and the death
//! screen ([`death_screen`]) are screens on one stack
//! ([`screens`]): opened by [`Screens::open`], fed key presses by `input::route_keys`, and drawn
//! by [`draw`], the one system that writes the UI console.

mod character_screen;
mod chargen_screen;
mod controls_screen;
mod death_screen;
mod dev;
mod effects;
mod equipment_screen;
mod glyphs;
mod info_tabs;
mod look;
mod menu;
mod missiles;
mod motion;
mod new_run_screen;
mod object_screen;
mod rain;
mod screens;
mod settings_screen;
mod skills_screen;
mod sounds;
mod spells;
mod target;
mod weapon_skills_screen;

pub use menu::Press;
pub use screens::{MenuChoice, ScreenAction, ScreenName, ScreenStack, Screens};

pub use dev::DevOverlays;
pub use target::{Aim, AimKind, LastTarget};

use crate::game::{Game, SimSet};
use crate::input::{Bindings, InputMode, route_keys};
use crate::render::anim::{AnimTable, TerrainAnim};
use crate::render::console::{Cell, Console};
use crate::render::grid::MapGrid;
use crate::render::{ConsoleLayout, Effects, VisualTime, lighting, shadows};
use crate::settings::Settings;
use bevy::prelude::*;
use missiles::Missiles;
use motion::Motion;
use spells::Spells;
use warlock_core::Pos;
use warlock_core::World;
use warlock_core::burden::Burden;
use warlock_core::color::Rgb;
use warlock_core::content::{MoonDef, ObjectId};
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
                    (screens::refresh, screens::on_death)
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
        draw_game(&mut console, game, *mode, &grid, *overlays, &bindings);
    }
    stack.draw(&mut console, &settings, &bindings);
}

/// The gameplay UI: message line (or the current prompt), status bar, dev indicator, and the
/// look cursor.
fn draw_game(
    console: &mut Console,
    game: &Game,
    mode: InputMode,
    grid: &MapGrid,
    overlays: DevOverlays,
    bindings: &Bindings,
) {
    let rows = console.rows() as i32;
    draw_messages(console, game, mode, bindings);
    draw_status(console, game, rows - 1);
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
                    let line = target::line(&view, view.viewer(), p, aim.range(&view));
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
    draw_map(&mut grid, &view, &anims, &key);
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

/// Draws terrain into the map grid, with the top object of each known pile in place of the
/// terrain's glyph (over its background, lit like terrain). Creatures are on the free-glyph
/// layer: the grid keeps the terrain under them but not its glyph (`covered`); so do cells the
/// line of fire is drawn over while aiming (`path`, its dots are free glyphs too), unless an
/// object lies there. What a missile still in flight lands as (`arriving`) is not drawn until it
/// arrives. Only visible terrain without objects animates. Visible cells keep their base colors
/// (the shader lights them); remembered ones, objects included, are dimmed here.
fn draw_map(grid: &mut MapGrid, view: &PerceptionView<'_>, anims: &AnimTable, key: &MapKey) {
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
            let top = pile.first().map(|s| content.object(s.object));
            let glyph = |t| {
                let def = content.terrain(t);
                let (ch, fg) = top.map_or((def.glyph, def.fg), |o| (o.glyph, o.fg));
                (ch, fg, def.bg)
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
        if view.objects(p).is_empty() {
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

fn draw_messages(console: &mut Console, game: &Game, mode: InputMode, bindings: &Bindings) {
    if let InputMode::Direction(command) = mode {
        let prompt = match command {
            crate::input::KeyCommand::UseTool => "Dig in which direction? (Esc to cancel)",
            _ => "In which direction? (Esc to cancel)",
        };
        console.print(1, 0, prompt, ACCENT);
        return;
    }
    if mode == InputMode::Rest {
        console.print(1, 0, REST_PROMPT, ACCENT);
        return;
    }
    if let InputMode::Aim(aim) = mode {
        let view = game.world.perception();
        let line = target::line(&view, view.viewer(), aim.at, aim.range(&view));
        let mut about = look::describe(&view, aim.at);
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
                (crate::input::KeyCommand::Fire, ammo)
            }
            AimKind::Throw(item) => (crate::input::KeyCommand::Throw, Some(item)),
        };
        let content = view.content();
        let ready = (missile.and_then(|i| view.inventory().get(i)))
            .map_or_else(String::new, |s| content.object(s.object).describe(1));
        let next = key(crate::input::KeyCommand::NextTarget);
        let prompt = aim.prompt(&ready, (&key(confirm), &next));
        console.print(1, 1, &prompt, ACCENT);
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

/// The rest prompt; its letters are [`crate::input::REST_CHOICES`].
const REST_PROMPT: &str = "Rest: (h)our, until (d)awn, until (n)ightfall? (Esc to cancel)";

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
    let segments = status_segments(&game.world);
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

/// The status bar's segments for `world`, in display order.
fn status_segments(world: &World) -> Vec<Segment> {
    let view = world.perception();
    let pos = world.player_pos();
    let time = world.time();
    let light = view.light(pos).map_or(0, |l| l.brightness());
    let lantern = match world.lantern() {
        Some(true) => Some("Lantern lit".to_string()),
        Some(false) => Some("Lantern out".to_string()),
        None => None,
    };
    let content = view.content();
    // Display order; the number is the drop priority (higher drops first when space runs out;
    // of equal ones the later first). The moon phase is cut down to its glyph and the date to
    // its day and month before they are dropped. The weather shows only under the open sky,
    // where the player knows it, and there it matters more than the date: it changes the light
    // and how far one sees. Hit points matter most after the level: they outlast the time, and
    // so does the burden (shown when there is one), which slows everything the player does.
    // The character level (`Lv 3`) comes after them and goes before them, but after the time.
    let calendar = content.calendar();
    let (hp, max_hp) = view.own_hp();
    let burden = view.own_load().burden();
    [
        Some(Segment::new(0, view.level_name())),
        Some(Segment::new(1, format!("HP {hp}/{max_hp}"))),
        (burden != Burden::Unburdened).then(|| Segment::new(1, burden.name())),
        Some(Segment::new(
            1,
            format!("Lv {}", view.own_experience().level),
        )),
        view.region().map(|r| Segment::new(4, r)),
        Some(Segment::new(
            1,
            format!("{:02}:{:02}", time.hour(), time.minute()),
        )),
        Some(Segment::new(3, calendar.full_date(time)).or_short(calendar.date(time))),
        moon_status(content.moons(), time)
            .map(|(text, glyphs)| Segment::new(5, text).or_short(glyphs)),
        view.weather().map(|w| Segment::new(2, w.name())),
        Some(Segment::new(6, format!("Turn {}", world.turn()))),
        Some(Segment::new(10, format!("{},{}", pos.x, pos.y))),
        Some(Segment::new(7, format!("Light {light}"))),
        Some(Segment::new(8, format!("{} vision", view.vision().name))),
        lantern.map(|l| Segment::new(9, l)),
        Some(Segment::new(11, format!("Seed {}", world.seed()))),
    ]
    .into_iter()
    .flatten()
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
    use warlock_core::content::{Content, ContentFiles};
    use warlock_core::{Action, Dir, Letters, World};

    #[test]
    fn the_status_bar_shows_the_burden_after_the_hit_points_and_keeps_it_late() {
        let mut world = hoard();
        let texts =
            |w: &World| -> Vec<String> { status_segments(w).into_iter().map(|s| s.text).collect() };
        assert!(
            !texts(&world).iter().any(|t| t.contains("urdened")),
            "unburdened: nothing"
        );
        world.apply(&Action::PickUp(Letters::default()));
        let segments = status_segments(&world);
        assert_eq!(segments[1].text, "HP 10/10");
        assert_eq!(segments[2].text, "Burdened", "right after the hit points");
        // Narrow: the time gives way first, then the burden, before the hit points.
        let line = "Hoard  ·  HP 10/10  ·  Burdened";
        assert_eq!(fit_segments(&segments, line.chars().count()), line);
        assert_eq!(
            fit_segments(&segments, line.chars().count() - 1),
            "Hoard  ·  HP 10/10"
        );
        assert!(fit_segments(&segments, 55).contains("Burdened  ·  Lv 1  ·  08:"));
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
            status_segments(world).into_iter().map(|s| s.text).collect()
        };
        let mut world = World::new(content.clone(), "showcase", None).expect("starts");
        world.apply(&Action::SetWeather(warlock_core::weather::Weather::Rain));
        let names = ["Clear", "Overcast", "Rain", "Heavy rain"];
        assert!(!texts(&world).iter().any(|t| names.contains(&t.as_str())));
        let mut world = World::new(content, "courtyard", None).expect("starts");
        world.apply(&Action::SetWeather(
            warlock_core::weather::Weather::HeavyRain,
        ));
        let segments = status_segments(&world);
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
        let line = fit_segments(&segments, 80);
        assert!(
            line.contains("Heavy rain") && line.contains("11 Darkening"),
            "{line}"
        );
    }

    #[test]
    fn the_status_bar_shows_hit_points_and_keeps_them_late() {
        let content = Arc::new(Content::builtin().expect("content"));
        let world = World::new(content, "showcase", None).expect("starts");
        let segments = status_segments(&world);
        assert_eq!(segments[1].text, "HP 17/17", "right after the level");
        // Narrow: everything but the level gives way first, the time included.
        let line = fit_segments(&segments, 25);
        assert!(line.starts_with("The Hub  ·  HP 17/17"), "{line}");
        assert!(!line.contains(':'), "{line}");
        assert_eq!(fit_segments(&segments, 20), "The Hub  ·  HP 17/17");
    }

    #[test]
    fn rest_prompt_names_every_choice() {
        for (key, _) in crate::input::REST_CHOICES {
            assert!(REST_PROMPT.contains(&format!("({key})")), "{key}");
        }
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
        draw_map(&mut grid, &view, &anims, &key);

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
        draw_map(&mut grid, &view, &anims, &key);
        assert_eq!(grid.free_terrain, vec![(Pos::new(5, 1), altar)]);
        assert_eq!(grid.console.cells()[at(5, 1)].map(|c| c.ch), Some(' '));
    }
}
