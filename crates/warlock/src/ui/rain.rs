//! Rain on the free-glyph layer (spec §4.8 "Weather"): faint drops falling on every visible cell
//! under the open sky, small splashes on open ground, and rings on still water, while it rains
//! where the player is.
//!
//! The weather comes from the perception snapshot ([`PerceptionView::weather`], known only under
//! the open sky). A cell rains iff it is visible and its terrain is not roofed; a drop is drawn
//! only while it is over a cell that rains, so rain never falls under a roof or where the player
//! does not see. The emitters are the weather's (`data/weather.toml`): ambient emitters per cell,
//! keyed by the cell's level position (so the rain stays put while the camera scrolls), stateless
//! like every particle. Drops are lit like other particles: faint in the dark, catching a
//! lantern's light. Off with the `weather` effect switch; the darker light of a rainy day is
//! gameplay light and stays.

use super::effects::particle_color;
use crate::game::Game;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::lighting::VisualLight;
use crate::render::particles::{self, Particle, Source};
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::Pos;
use warlock_core::color::Rgb;
use warlock_core::content::{GlyphAnim, ParticleEmitter, WeatherDef};
use warlock_core::perception::{CellView, PerceptionView};

/// Most rain particles in a frame, whatever the size of the view.
const MAX_PARTICLES: usize = 3000;

/// Rows below the map grid whose drops can still be seen falling into its bottom rows.
const BELOW: i32 = 4;

/// Seed salts that keep the rain's emitters apart from each other and from a cell's own
/// particles.
const DROPS: u64 = 0x7261_696e_0000_0001;
const SPLASH: u64 = 0x7261_696e_0000_0002;
const RIPPLES: u64 = 0x7261_696e_0000_0003;

/// What rain lands on in a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ground {
    /// Open ground: drops splash.
    Open,
    /// Still water: rings.
    Water,
    /// Anything else (trees, walls, running water): drops only.
    Other,
}

/// The cells it rains on around the map grid, kept until the view changes.
#[derive(Debug, Default)]
pub(super) struct Wet {
    key: Option<(IVec2, UVec2)>,
    origin: IVec2,
    size: UVec2,
    /// Row-major over `origin` and `size`: whether it rains on the cell.
    wet: Vec<bool>,
    cells: Vec<(Pos, Ground)>,
}

impl Wet {
    /// The visible, unroofed cells of `size` cells from `origin`, and [`BELOW`] rows more.
    fn update(&mut self, view: &PerceptionView<'_>, origin: IVec2, size: UVec2) {
        let content = view.content();
        let size = UVec2::new(size.x, size.y + BELOW as u32);
        self.origin = origin;
        self.size = size;
        self.wet.clear();
        self.cells.clear();
        for y in origin.y..origin.y + size.y as i32 {
            for x in origin.x..origin.x + size.x as i32 {
                let p = Pos::new(x, y);
                let CellView::Visible(t) = view.cell(p) else {
                    self.wet.push(false);
                    continue;
                };
                let def = content.terrain(t);
                self.wet.push(!def.roofed);
                if def.roofed {
                    continue;
                }
                let ground = match def.anim {
                    Some(GlyphAnim::Wave { .. }) => Ground::Water,
                    _ if def.walkable && def.transparent && !def.flows() => Ground::Open,
                    _ => Ground::Other,
                };
                self.cells.push((p, ground));
            }
        }
    }

    fn is_wet(&self, p: Pos) -> bool {
        let (x, y) = (p.x - self.origin.x, p.y - self.origin.y);
        if x < 0 || y < 0 || x >= self.size.x as i32 || y >= self.size.y as i32 {
            return false;
        }
        self.wet[(y * self.size.x as i32 + x) as usize]
    }
}

/// The rain's particles at `time` over the cells in `wet`, at most [`MAX_PARTICLES`], and for
/// each how it leans: cells across per cell down, along its emitter's velocity (drops lean the
/// way they fall; splashes and rings stand upright).
fn rain(
    def: &WeatherDef,
    wet: &Wet,
    time: f32,
    paint: impl Fn(Rgb) -> Rgb + Copy,
    out: &mut Vec<Particle>,
    leans: &mut Vec<f32>,
) {
    let lean = |e: &ParticleEmitter| match e.velocity {
        [x, y] if y != 0.0 => x / y,
        _ => 0.0,
    };
    for &(p, ground) in &wet.cells {
        if out.len() >= MAX_PARTICLES {
            break;
        }
        let source = Source {
            origin: Vec2::new(p.x as f32, p.y as f32),
            down: Source::UPRIGHT,
            paint,
        };
        let key = particles::cell_key(p.x, p.y);
        if let Some(e) = &def.drops {
            particles::ambient(e, key ^ DROPS, source, time, out);
            leans.resize(out.len(), lean(e));
        }
        let (landing, salt) = match ground {
            Ground::Open => (&def.splash, SPLASH),
            Ground::Water => (&def.ripples, RIPPLES),
            Ground::Other => (&None, 0),
        };
        if let Some(e) = landing {
            particles::ambient(e, key ^ salt, source, time, out);
            leans.resize(out.len(), 0.0);
        }
    }
    out.truncate(MAX_PARTICLES);
    leans.truncate(MAX_PARTICLES);
}

/// Puts this frame's rain on the free-glyph layer, below creatures.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    mut free: ResMut<FreeGlyphs>,
    mut wet: Local<Wet>,
    mut scratch: Local<(Vec<Particle>, Vec<f32>)>,
) {
    if !effects.weather {
        return;
    }
    let view = game.world.perception();
    let Some(weather) = view.weather() else {
        return;
    };
    let def = view.content().climate().def(weather);
    if def.emitters().next().is_none() {
        return;
    }
    let size = UVec2::new(grid.console.cols(), grid.console.rows());
    let key = (grid.level_origin, size);
    if game.is_changed() || wet.key != Some(key) {
        wet.key = Some(key);
        wet.update(&view, grid.level_origin, size);
    }
    let (out, leans) = &mut *scratch;
    out.clear();
    leans.clear();
    let paint = |c: Rgb| settings.palette.apply(c, Role::Fg);
    rain(def, &wet, visual.seconds, paint, out, leans);
    // A lean in cells becomes a slant in pixels.
    let aspect = grid.cell.x / grid.cell.y.max(1.0);
    for (p, lean) in out.iter().zip(leans.iter()) {
        let at = Pos::new(
            (p.pos.x + 0.5).floor() as i32,
            (p.pos.y + 0.5).floor() as i32,
        );
        if !wet.is_wet(at) {
            continue;
        }
        free.push(FreeGlyph {
            pos: grid.to_screen(p.pos),
            ch: p.ch,
            color: particle_color(p, at, &light),
            scale: p.scale,
            slant: lean * aspect,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::{Content, ContentFiles};
    use warlock_core::weather::Weather;
    use warlock_core::{Action, World};

    /// A walled garden under the sky: grass, a pond, a roofed porch, a wall, and a tree.
    fn garden() -> World {
        let mut files = ContentFiles::builtin();
        files.insert(
            "levels/garden.toml",
            "name = \"Garden\"\nkind = \"outdoor\"\nmap = '''\n\
             ##########\n#..~~..T.#\n#..~~....#\n#.@...===#\n##########\n'''\n\
             [legend]\n\"@\" = { terrain = \"grass\", spawn = \"player\" }\n\
             \".\" = \"grass\"\n\"~\" = \"water\"\n\"T\" = \"tree\"\n\"=\" = \"gazebo_floor\"\n",
        );
        files.insert(
            "scenarios/garden.toml",
            "name = \"G\"\nstart_level = \"garden\"\nstart_time = \"12:00\"\n",
        );
        let content = Content::load(&files).expect("test content loads");
        World::new(Arc::new(content), "garden", None).expect("scenario starts")
    }

    fn wet_cells(world: &World) -> Wet {
        let mut wet = Wet::default();
        wet.update(&world.perception(), IVec2::ZERO, UVec2::new(10, 5));
        wet
    }

    #[test]
    fn it_rains_on_visible_cells_under_the_sky_not_under_a_roof() {
        let world = garden();
        let wet = wet_cells(&world);
        let ground = |x, y| {
            wet.cells
                .iter()
                .find(|(p, _)| *p == Pos::new(x, y))
                .map(|(_, g)| *g)
        };
        assert_eq!(ground(1, 1), Some(Ground::Open), "grass");
        assert_eq!(ground(3, 1), Some(Ground::Water), "the pond");
        assert_eq!(ground(7, 1), Some(Ground::Other), "the tree");
        assert_eq!(ground(0, 0), Some(Ground::Other), "the wall");
        assert_eq!(ground(6, 3), None, "the porch is roofed");
        assert!(!wet.is_wet(Pos::new(6, 3)) && wet.is_wet(Pos::new(5, 3)));
        assert!(!wet.is_wet(Pos::new(-1, 0)) && !wet.is_wet(Pos::new(3, 9)));
    }

    #[test]
    fn rain_falls_only_while_it_rains_and_heavier_rain_is_denser() {
        let mut world = garden();
        let content = world.content().clone();
        let count = |w: Weather, wet: &Wet| {
            let (mut out, mut leans) = (Vec::new(), Vec::new());
            for i in 0..50 {
                let def = content.climate().def(w);
                rain(def, wet, i as f32 * 0.37, |c| c, &mut out, &mut leans);
                assert_eq!(out.len(), leans.len());
            }
            // Drops lean the way they fall (down and to the left); the rest stand upright.
            assert!(leans.iter().all(|l| *l <= 0.0));
            out
        };
        world.apply(&Action::SetWeather(Weather::Clear));
        let wet = wet_cells(&world);
        assert!(count(Weather::Clear, &wet).is_empty());
        assert!(count(Weather::Overcast, &wet).is_empty());
        let light = count(Weather::Rain, &wet);
        let heavy = count(Weather::HeavyRain, &wet);
        assert!(!light.is_empty() && heavy.len() > light.len() * 2);
        // Faint, and never glowing.
        assert!(heavy.iter().all(|p| p.color.alpha < 0.6 && !p.emissive));
        // Drops fall: their glyph is the drop's, and they move down and a little to the left.
        let drops = content
            .climate()
            .def(Weather::Rain)
            .drops
            .clone()
            .expect("drops");
        let mut a = Vec::new();
        let mut b = Vec::new();
        let source = Source {
            origin: Vec2::new(3.0, 3.0),
            down: Source::UPRIGHT,
            paint: |c| c,
        };
        // Find a moment a drop is in flight, then look a little later.
        let t = (0..2000)
            .map(|i| i as f32 * 0.01)
            .find(|t| {
                a.clear();
                particles::ambient(&drops, 9, source, *t, &mut a);
                !a.is_empty()
            })
            .expect("a drop falls");
        particles::ambient(&drops, 9, source, t + 0.03, &mut b);
        if let (Some(p), Some(q)) = (a.first(), b.first())
            && p.ch == q.ch
        {
            assert!(q.pos.y > p.pos.y && q.pos.x < p.pos.x, "{p:?} then {q:?}");
        }
    }
}
