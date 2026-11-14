//! Light, the second step of [`Perception::compute`](super::Perception::compute) (the module
//! docs): base light (ambient, sky, bounce, a room's glow) and every source's light on the cells
//! in line of sight ([`light`]), then on the wall faces the player sees
//! ([`Lighting::light_faces`]), and what of each source they see ([`Lighting::sources`]).

use super::seeing::{Los, Seeing};
use super::{Emitter, SeenLight, Sight, Viewing};
use crate::daylight;
use crate::geom::{Dir, Pos};
use crate::grid::Window;
use crate::light::{
    Area, BOUNCE_STEPS, Bounce, Light, LightCache, SUN_KEY, Sunlight, falloff, lit_in_sight,
    moon_key, wall_faces,
};

/// The light the player perceives on the cells in view.
pub(super) struct Lighting {
    /// Perceived ambient light.
    pub(super) ambient: Light,
    /// Ambient plus sky plus bounce light; meaningful on visible cells only.
    pub(super) base: Window<Light>,
    /// Perceived light; meaningful on visible cells only.
    pub(super) light: Window<Light>,
    indirect: Indirect,
    /// The sun's and the moons' light, by source key.
    beams: Vec<(u64, Emitter, Sunlight)>,
    /// Each source's light on the cells in line of sight, and then on wall faces.
    lit: Vec<Lit>,
}

/// One source's light in view: on the cells in line of sight, and on the wall faces seen.
struct Lit {
    key: u64,
    emitter: Emitter,
    cells: Vec<(Pos, Light)>,
    faces: Vec<(Pos, Light)>,
}

/// Steady light on a transparent cell beyond the ambient light: sky, bounce, and a special
/// room's glow (a sanctuary's).
struct Indirect {
    /// The sky's light, perceived; dark off the open sky.
    glow: Light,
    bounce: Option<Bounce>,
}

impl Indirect {
    fn at(&self, v: &Viewing, p: Pos) -> Light {
        let bounce = self.bounce.as_ref().map_or(Light::DARK, |b| b.at(p));
        let bounce = bounce.saturating_add(v.eye.perceive(v.level.room_glow(v.content, p)));
        if !self.glow.is_dark() && v.exposed(p) {
            bounce.saturating_add(self.glow)
        } else {
            bounce
        }
    }
}

/// Light on the cells in line of sight (the player's and the bond's): base light plus every
/// source reaching them, the sun and the moons last. Light masks come from (and are added to)
/// `lights`.
pub(super) fn light(v: &Viewing, los: &Los, lights: &mut LightCache) -> Lighting {
    let (state, content, level, eye) = (v.state, v.content, v.level, &v.eye);
    let (width, height) = (level.width(), level.height());
    let is_opaque = |p: Pos| v.is_opaque(p);
    // Sources that can light a cell in view, or a surface whose bounce light reaches one (a
    // bounce step moves at most √2 < 2 cells; wall faces are within a cell of view).
    let reach = los.light_reach(v.viewer) + 1 + 2 * BOUNCE_STEPS;
    let near = lights.near(state, content, v.viewer, reach);
    // Direct light is needed on everything in sight and within bounce reach of it.
    let area = Area::around(los.lit_cells().iter().copied(), BOUNCE_STEPS, width, height);

    // Under the open sky: the sun and each moon that shines where no taller terrain shades a
    // cell, the sky everywhere (none of them under a roof).
    let mut bodies: Vec<(u64, Emitter, [i16; 2], Light)> = Vec::new();
    if let Some(sky) = &v.sky {
        if sky.sun.is_up() {
            let sun = sky.sun;
            bodies.push((SUN_KEY, Emitter::Sun(sun), sun.shadow, sun.light));
        }
        for moon in sky.moons.iter().filter(|m| m.shines()) {
            let emitter = Emitter::Moon(*moon);
            bodies.push((moon_key(moon.index), emitter, moon.shadow, moon.light));
        }
    }
    let beams: Vec<(u64, Emitter, Sunlight)> = match area {
        Some(area) => (bodies.into_iter())
            .map(|(key, emitter, shadow, light)| {
                let height = |p: Pos| v.terrain(p).map_or(0, |t| t.height);
                let tallest = content.tallest_terrain();
                let exposed = |p: Pos| v.exposed(p);
                let cells =
                    Sunlight::compute(shadow, light, area, tallest, height, exposed, is_opaque);
                (key, emitter, cells)
            })
            .collect(),
        None => Vec::new(),
    };
    let glow = v
        .sky
        .as_ref()
        .map_or(Light::DARK, |s| eye.perceive(s.glow()));

    // Base light on cells in line of sight: ambient, sky, and bounce (on transparent cells;
    // RGB only, so every eye perceives all of it). The overworld never gets darker than its
    // faint night light (moonlight comes on top).
    let night = match &v.sky {
        Some(sky) if v.overworld => daylight::overworld_night(sky.sun.sky),
        _ => Light::DARK,
    };
    let ambient = eye.perceive(level.ambient.saturating_add(night));
    let bounce = area.map(|area| {
        let tint = |p: Pos| {
            let rgb = v.terrain(p).map(|t| t.bounce);
            rgb.map_or([0; 3], |c| [c.r, c.g, c.b])
        };
        let masks = (near.iter())
            .filter(|(_, mask)| mask.overlaps(area.min, area.max))
            .map(|(_, mask)| mask.cells.as_slice());
        let beams = beams.iter().map(|(_, _, s)| s.cells.as_slice());
        Bounce::compute(area, masks.chain(beams), is_opaque, tint)
    });
    let indirect = Indirect { glow, bounce };
    let (lo, hi) = los.bounds;
    let mut base = Window::new(width, height, lo, hi, Light::DARK);
    for &p in los.lit_cells() {
        base[p] = v.shade(p, ambient.saturating_add(indirect.at(v, p)));
    }

    // Light on cells in line of sight: base light plus every source reaching them, the sun
    // and the moons last.
    let mut light = base.clone();
    let mut lit: Vec<_> = near
        .into_iter()
        .map(|(source, mask)| {
            let emitter = Emitter::Point {
                pos: source.pos,
                preset: source.preset,
            };
            let cells = lit_in_sight(
                &mask.cells,
                |q| los.lit(q),
                |q, l| v.shade(q, eye.perceive(l)),
            );
            Lit::new(source.key, emitter, cells)
        })
        .collect();
    for (key, emitter, beam) in &beams {
        let cells = lit_in_sight(
            &beam.cells,
            |q| los.lit(q),
            |q, l| v.shade(q, eye.perceive(l)),
        );
        lit.push(Lit::new(*key, *emitter, cells));
    }
    for source in &lit {
        for &(p, c) in &source.cells {
            light[p] = light[p].saturating_add(c);
        }
    }
    Lighting {
        ambient,
        base,
        light,
        indirect,
        beams,
        lit,
    }
}

impl Lit {
    fn new(key: u64, emitter: Emitter, cells: Vec<(Pos, Light)>) -> Self {
        Lit {
            key,
            emitter,
            cells,
            faces: Vec::new(),
        }
    }
}

impl Lighting {
    /// Wall faces: opaque cells in line of sight or completed take the brightest sky and bounce
    /// light of their visible transparent neighbors, then each source's light at their own
    /// distance (the sun's or a moon's where it reaches them) if they face a visible cell it
    /// lights.
    pub(super) fn light_faces(&mut self, v: &Viewing, los: &Los, seeing: &Seeing) {
        let sight = &seeing.sight;
        let is_seen = |q: Pos| sight.get(q) == Some(&Sight::Seen);
        let is_face = |w: Pos| v.is_opaque(w) && (los.lit(w) || sight.get(w) == Some(&Sight::Edge));
        for &w in los.lit_cells().iter().chain(&seeing.visible_cells) {
            if is_face(w) {
                let from_neighbors = Dir::ALL
                    .map(|d| w.step(d))
                    .into_iter()
                    .filter(|q| is_seen(*q) && !v.is_opaque(*q))
                    .fold(Light::DARK, |acc, q| acc.max(self.indirect.at(v, q)));
                self.base[w] = v.shade(w, self.ambient.saturating_add(from_neighbors));
                self.light[w] = self.base[w];
            }
        }
        let beams = &self.beams;
        let light_at = |key: u64, emitter: Emitter, w: Pos| {
            let light = match emitter {
                Emitter::Point { pos, preset } => {
                    let preset = v.content.light(preset);
                    falloff(preset.color, preset.radius.max(0), w.dist2(pos))
                }
                Emitter::Sun(_) | Emitter::Moon(_) => (beams.iter())
                    .find(|(k, _, _)| *k == key)
                    .map_or(Light::DARK, |(_, _, beam)| beam.at(w)),
            };
            v.shade(w, v.eye.perceive(light))
        };
        for source in &mut self.lit {
            let (key, emitter) = (source.key, source.emitter);
            source.faces = wall_faces(
                &source.cells,
                |w| light_at(key, emitter, w),
                is_seen,
                is_face,
            );
        }
        for source in &self.lit {
            for &(p, c) in &source.faces {
                self.light[p] = self.light[p].saturating_add(c);
            }
        }
    }

    /// The sources whose light the player sees on a visible cell, with what they see of it.
    pub(super) fn sources(&mut self, seeing: &Seeing) -> Vec<SeenLight> {
        std::mem::take(&mut self.lit)
            .into_iter()
            .filter_map(|source| {
                let contributions: Vec<_> = (source.cells.into_iter())
                    .chain(source.faces)
                    .filter(|(p, _)| seeing.is_visible(*p))
                    .collect();
                (!contributions.is_empty()).then_some(SeenLight {
                    key: source.key,
                    emitter: source.emitter,
                    contributions,
                })
            })
            .collect()
    }
}
