//! Visual effects from the perception snapshot (spec §4.8): glyph particles, per-cell glow and heat
//! haze in the map grid, and the screen fade on level change.
//!
//! Only perceived things produce effects. Ambient emitters run on visible cells and seen
//! creatures. Bursts come from the turn's events (`Outcome.events`, already filtered by
//! perception) at visible cells: a splash where a creature steps into water, dust where a door
//! opens or closes. Particles over cells that are not visible are not drawn, and remembered cells
//! never glow, haze, or emit. Each part runs only with its effect on.

use super::glyphs;
use super::motion::Motion;
use crate::game::Game;
use crate::render::console::CellFx;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::lighting::{TERRAIN_FLOOR, VisualLight};
use crate::render::particles::{self, Particle, Source};
use crate::render::post::{ScreenTints, in_range};
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::color::Rgb;
use warlock_core::content::{ParticleEmitter, ParticleTrigger, TerrainId};
use warlock_core::perception::{CellView, PerceptionView};
use warlock_core::{Event, Pos};

/// How long the map takes to fade in from black after a level change, in seconds.
pub const FADE_SECONDS: f32 = 0.25;

/// With frozen visual time (`--freeze-time`), bursts are shown this long after they went off, so
/// captures show them in flight.
const FROZEN_BURST_AGE: f32 = 0.15;

/// Heat haze rising from a hot cell: `(dx, dy, strength)` relative to the source (y down).
const HAZE_PLUME: [(i32, i32, f32); 8] = [
    (0, 0, 0.5),
    (0, -1, 1.0),
    (-1, -1, 0.45),
    (1, -1, 0.45),
    (0, -2, 0.75),
    (-1, -2, 0.3),
    (1, -2, 0.3),
    (0, -3, 0.35),
];

/// Seed bit that keeps creature particle keys apart from cell keys.
const CREATURE_KEY: u64 = 1 << 63;

/// A burst of particles set off by a perceived event.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Burst {
    terrain: TerrainId,
    /// Index into the terrain's emitters.
    emitter: usize,
    pos: Pos,
    seed: u64,
    start: f32,
}

/// Running particle bursts.
#[derive(Resource, Debug, Default)]
pub struct Bursts {
    list: Vec<Burst>,
    /// The turn whose events were read last.
    turn: Option<u64>,
}

impl Bursts {
    /// Sets off the `trigger` emitters of the terrain seen at `pos`, if it is visible.
    fn trigger(
        &mut self,
        view: &PerceptionView<'_>,
        pos: Pos,
        trigger: ParticleTrigger,
        seed: u64,
        start: f32,
    ) {
        let CellView::Visible(terrain) = view.cell(pos) else {
            return;
        };
        let emitters = &view.content().terrain(terrain).particles;
        for (emitter, e) in emitters.iter().enumerate() {
            if e.on == trigger {
                self.list.push(Burst {
                    terrain,
                    emitter,
                    pos,
                    seed: seed ^ emitter as u64,
                    start,
                });
            }
        }
    }
}

/// Reads a new turn's perceived events: bursts for steps and doors, a fade on level change.
pub(super) fn track(
    game: Res<Game>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    mut bursts: ResMut<Bursts>,
    mut tints: ResMut<ScreenTints>,
) {
    let now = visual.seconds;
    let content = game.world.content();
    let life = |b: &Burst| {
        let def = content.terrain(b.terrain);
        def.particles.get(b.emitter).map_or(0.0, |e| e.life)
    };
    if bursts.list.iter().any(|b| now - b.start >= life(b)) {
        bursts.list.retain(|b| now - b.start < life(b));
    }
    if !game.is_changed() {
        return;
    }
    let turn = game.world.turn();
    if bursts.turn.replace(turn) == Some(turn) || !effects.particles {
        return;
    }
    let view = game.world.perception();
    let start = if visual.frozen {
        now - FROZEN_BURST_AGE
    } else {
        now
    };
    for (i, event) in game.last_outcome.events.iter().enumerate() {
        let seed = turn.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ ((i as u64) << 40);
        match event {
            Event::ActorMoved { to, .. } => {
                bursts.trigger(&view, *to, ParticleTrigger::Step, seed, start);
            }
            Event::DoorOpened { pos } | Event::DoorClosed { pos } => {
                bursts.trigger(&view, *pos, ParticleTrigger::Toggle, seed, start);
            }
            Event::LevelChanged { .. } => {
                bursts.list.clear();
                tints.clear();
                if !visual.frozen && effects.screen_fades {
                    tints.flash(LinearRgba::BLACK, FADE_SECONDS, now);
                }
            }
        }
    }
}

/// Visible cells in the map grid with ambient emitters, kept until the view changes.
#[derive(Debug, Default)]
pub(super) struct Sources {
    key: Option<(IVec2, UVec2)>,
    cells: Vec<(Pos, TerrainId)>,
}

/// Puts this frame's particles on the free-glyph layer, below creatures.
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    motion: Res<Motion>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    bursts: Res<Bursts>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    mut free: ResMut<FreeGlyphs>,
    mut sources: Local<Sources>,
    mut scratch: Local<Vec<Particle>>,
) {
    if !effects.particles {
        return;
    }
    let view = game.world.perception();
    let content = view.content();
    let time = visual.seconds;
    let size = UVec2::new(grid.console.cols(), grid.console.rows());
    let key = (grid.level_origin, size);
    if game.is_changed() || sources.key != Some(key) {
        sources.key = Some(key);
        sources.cells = ambient_sources(&view, grid.level_origin, size);
    }

    let out = &mut *scratch;
    out.clear();
    let paint = |c: Rgb| settings.palette.apply(c, Role::Fg);
    let from = |origin: Vec2| Source { origin, paint };
    let ambient = |e: &&ParticleEmitter| e.on == ParticleTrigger::Ambient;
    for &(p, terrain) in &sources.cells {
        for e in content.terrain(terrain).particles.iter().filter(ambient) {
            particles::ambient(e, particles::cell_key(p.x, p.y), from(cell(p)), time, out);
        }
    }
    for actor in view.actors() {
        let def = content.creature(actor.creature);
        if def.particles.is_empty() {
            continue;
        }
        // Creature particles move with it (including while it slides).
        let at = motion.actor_pos(actor.id, actor.pos, time);
        for e in def.particles.iter().filter(ambient) {
            particles::ambient(e, CREATURE_KEY | u64::from(actor.id.0), from(at), time, out);
        }
    }
    for b in &bursts.list {
        if let Some(e) = content.terrain(b.terrain).particles.get(b.emitter) {
            particles::burst(e, b.seed, from(cell(b.pos)), time - b.start, out);
        }
    }

    for p in out.iter() {
        let at = Pos::new(
            (p.pos.x + 0.5).floor() as i32,
            (p.pos.y + 0.5).floor() as i32,
        );
        if !view.is_visible(at) {
            continue;
        }
        free.push(FreeGlyph {
            pos: grid.to_screen(p.pos),
            ch: p.ch,
            color: particle_color(p, at, &light),
            scale: p.scale,
        });
    }
}

fn cell(p: Pos) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// Visible cells of the `size` cells from `origin` whose terrain has ambient emitters.
fn ambient_sources(view: &PerceptionView<'_>, origin: IVec2, size: UVec2) -> Vec<(Pos, TerrainId)> {
    let content = view.content();
    let mut cells = Vec::new();
    for y in origin.y..origin.y + size.y as i32 {
        for x in origin.x..origin.x + size.x as i32 {
            let p = Pos::new(x, y);
            if let CellView::Visible(t) = view.cell(p)
                && content
                    .terrain(t)
                    .particles
                    .iter()
                    .any(|e| e.on == ParticleTrigger::Ambient)
            {
                cells.push((p, t));
            }
        }
    }
    cells
}

/// A particle's color on screen: glowing particles are light themselves; the rest are lit like
/// free glyphs (sampled at the particle, else at its cell `home`) and kept within display range.
fn particle_color(p: &Particle, home: Pos, light: &VisualLight) -> LinearRgba {
    if p.emissive {
        return p.color;
    }
    let home = cell(home);
    in_range(glyphs::lit(p.color, light, p.pos, home, TERRAIN_FLOOR))
}

/// An effect strength in `[0, 1]` as a byte.
fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Writes the glow and heat haze of visible terrain into the map grid (after the map is drawn).
/// Haze rises from hot cells into the visible cells above them.
pub(super) fn draw_cell_fx(grid: &mut MapGrid, view: &PerceptionView<'_>) {
    let content = view.content();
    let (cols, rows) = (grid.console.cols() as i32, grid.console.rows() as i32);
    let origin = grid.level_origin;
    let at = |x: i32, y: i32| Pos::new(origin.x + x, origin.y + y);
    let mut haze = vec![0.0f32; (cols * rows).max(0) as usize];
    for y in 0..rows {
        for x in 0..cols {
            let CellView::Visible(t) = view.cell(at(x, y)) else {
                continue;
            };
            let def = content.terrain(t);
            if def.glow > 0.0 {
                grid.console.set_fx(
                    x,
                    y,
                    CellFx {
                        glow: byte(def.glow),
                        haze: 0,
                    },
                );
            }
            if def.haze > 0.0 {
                for (dx, dy, strength) in HAZE_PLUME {
                    let (hx, hy) = (x + dx, y + dy);
                    if (0..cols).contains(&hx) && (0..rows).contains(&hy) {
                        let h = &mut haze[(hy * cols + hx) as usize];
                        *h = h.max(def.haze * strength);
                    }
                }
            }
        }
    }
    for y in 0..rows {
        for x in 0..cols {
            let h = haze[(y * cols + x) as usize];
            if h > 0.0 && view.is_visible(at(x, y)) {
                let glow = grid.console.fx()[(y * cols + x) as usize].glow;
                grid.console.set_fx(
                    x,
                    y,
                    CellFx {
                        glow,
                        haze: byte(h),
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::{Content, ContentFiles};
    use warlock_core::{Action, Dir, World};

    /// A lit room with a brazier and water, and a door to a side room with the same.
    fn room() -> World {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/fx.toml",
            "name = \"Fx\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             #########\n#...#.~.#\n#.B.+.B.#\n#.~@#...#\n#########\n'''\n\
             [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\"B\" = \"brazier\"\n",
        );
        files.insert("scenarios/fx.toml", "name = \"Fx\"\nstart_level = \"fx\"\n");
        let content = Content::load(&files).expect("test content loads");
        World::new(Arc::new(content), "fx", None).expect("scenario starts")
    }

    fn grid_for(world: &World) -> MapGrid {
        let view = world.perception();
        let mut grid = MapGrid::default();
        grid.console
            .resize(view.width() as u32, view.height() as u32);
        grid
    }

    #[test]
    fn haze_rises_over_visible_braziers_and_glow_marks_them() {
        let world = room();
        let view = world.perception();
        let mut grid = grid_for(&world);
        draw_cell_fx(&mut grid, &view);
        let fx = |x: i32, y: i32| grid.console.fx()[(y * 9 + x) as usize];
        // The visible brazier at (2, 2) glows and heats the cells above it.
        assert!(fx(2, 2).glow > 0);
        assert_eq!(fx(2, 1).haze, 255);
        assert!(fx(2, 2).haze > 0 && fx(2, 2).haze < fx(2, 1).haze);
        assert!(fx(1, 1).haze > 0);
        // The brazier behind the closed door is not visible: no glow, no haze.
        assert_eq!(fx(6, 2), CellFx::NONE);
        assert_eq!(fx(5, 1), CellFx::NONE);
        assert_eq!(fx(6, 1), CellFx::NONE);
        // Far from any brazier nothing changes.
        assert_eq!(fx(3, 3), CellFx::NONE);
    }

    #[test]
    fn only_visible_cells_emit() {
        let world = room();
        let view = world.perception();
        let content = view.content();
        let brazier = content.terrain_id("brazier").expect("brazier exists");
        let sources = ambient_sources(&view, IVec2::ZERO, UVec2::new(9, 5));
        assert_eq!(sources, vec![(Pos::new(2, 2), brazier)]);
    }

    #[test]
    fn perceived_events_set_off_bursts() {
        let mut world = room();
        // Step into the water (W), then open the door (E of the start).
        let splash = world.apply(&Action::Move(Dir::W));
        assert!(!splash.events.is_empty());
        let view = world.perception();
        let mut bursts = Bursts::default();
        for event in &splash.events {
            if let Event::ActorMoved { to, .. } = event {
                bursts.trigger(&view, *to, ParticleTrigger::Step, 1, 0.0);
            }
        }
        assert!(!bursts.list.is_empty(), "water splashes");
        assert!(bursts.list.iter().all(|b| b.pos == Pos::new(2, 3)));

        let mut bursts = Bursts::default();
        bursts.trigger(&view, Pos::new(4, 2), ParticleTrigger::Toggle, 1, 0.0);
        assert!(!bursts.list.is_empty(), "the visible door puffs dust");
        let mut hidden = Bursts::default();
        hidden.trigger(&view, Pos::new(6, 1), ParticleTrigger::Step, 1, 0.0);
        assert!(
            hidden.list.is_empty(),
            "no splash in water that is not visible"
        );
    }

    #[test]
    fn particles_are_lit_unless_they_glow() {
        let world = room();
        let view = world.perception();
        let mut light = VisualLight::default();
        let size = UVec2::new(view.width() as u32, view.height() as u32);
        light.composite(&view, IVec2::ZERO, size, None);
        let mote = Particle {
            pos: Vec2::new(2.0, 1.0),
            ch: '.',
            color: LinearRgba::new(0.9, 0.9, 0.9, 0.5),
            scale: 1.0,
            emissive: false,
        };
        // Next to the brazier the light is overbright: lit, but kept within display range.
        let lit = particle_color(&mote, Pos::new(2, 1), &light);
        assert_ne!(lit, mote.color);
        assert!(lit.red <= 1.0 && lit.green <= 1.0 && lit.alpha == 0.5);
        // A spark glows: it is light, so it keeps its color.
        let spark = Particle {
            color: LinearRgba::new(1.8, 1.2, 0.3, 1.0),
            emissive: true,
            ..mote
        };
        assert_eq!(particle_color(&spark, Pos::new(2, 1), &light), spark.color);
    }
}
