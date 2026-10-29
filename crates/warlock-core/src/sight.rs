//! Creature sight (spec §5 "Creature sight"): what a creature sees, by the player's rules.
//!
//! A creature sees an actor iff the actor's cell is visible to it as the player's rules would
//! have it (see [`crate::perception`]): in line of sight (symmetric shadowcasting from the
//! creature's cell) within its sight radius, **and** lit at least to its eye's threshold by the
//! same gameplay light the player sees (ambient, sky, every source, the sun and the moons through
//! the weather's clouds, bounce, and what a special room does to it: a shadowed room's darkness, a
//! sanctuary's glow) or within its darkvision radius. On the overworld, sight is at map
//! scale: line of sight within the daylight (and moonlight, and rain) radius and the creature's
//! own sight, whatever the light. An actor on an opaque cell is seen
//! only within darkvision (off the overworld; no such terrain exists there yet).
//!
//! Creatures never read the player's perception. They compute their own, and only for the cells
//! where a hostile stands: a line of sight within their radius, and the light of those cells
//! ([`light`]), so the cost follows the hostiles nearby, not the size of the level.

use crate::content::{Content, LevelKind, VisionProfile};
use crate::daylight;
use crate::fov::compute_fov;
use crate::geom::Pos;
use crate::light::{Area, BOUNCE_STEPS, Bounce, Light, LightCache, Sunlight};
use crate::world::WorldState;

/// The cells in line of sight from one cell within a radius (symmetric shadowcasting).
#[derive(Clone, Debug)]
pub(crate) struct LineOfSight {
    origin: Pos,
    radius: i32,
    /// Row-major over the square of side `2 · radius + 1` around the origin.
    seen: Vec<bool>,
}

impl LineOfSight {
    pub(crate) fn compute(origin: Pos, radius: i32, is_opaque: impl Fn(Pos) -> bool) -> Self {
        let radius = radius.max(0);
        let side = 2 * radius + 1;
        let mut los = Self {
            origin,
            radius,
            seen: vec![false; (side * side) as usize],
        };
        compute_fov(origin, radius, is_opaque, |p| {
            if let Some(i) = los.index(p) {
                los.seen[i] = true;
            }
        });
        los
    }

    fn index(&self, p: Pos) -> Option<usize> {
        let side = 2 * self.radius + 1;
        let (dx, dy) = (
            p.x - self.origin.x + self.radius,
            p.y - self.origin.y + self.radius,
        );
        ((0..side).contains(&dx) && (0..side).contains(&dy)).then_some((dy * side + dx) as usize)
    }

    pub(crate) fn contains(&self, p: Pos) -> bool {
        self.index(p).is_some_and(|i| self.seen[i])
    }

    pub(crate) fn radius(&self) -> i32 {
        self.radius
    }

    /// The smallest radius that reaches squared distance `d2` (`r² + r ≥ d2`). Line of sight
    /// within a radius is the same whatever the radius it was computed with, as long as it
    /// reaches: shadows are cast only by nearer cells.
    pub(crate) fn radius_for(d2: i32) -> i32 {
        let mut r = d2.max(0).isqrt();
        while r * r + r < d2 {
            r += 1;
        }
        r
    }
}

/// How far a viewer whose eyes reach `sight` cells sees on the current level right now: on the
/// overworld no farther than daylight (and moonlight, and rain) allows.
pub(crate) fn sight_radius(state: &WorldState, content: &Content, sight: i32) -> i32 {
    match (state.current().kind, state.sky(content)) {
        (LevelKind::Overworld, Some(sky)) => {
            sight.min(daylight::sight_under(state.time, &sky, content.climate()))
        }
        _ => sight,
    }
}

/// Whether a viewer with `eye` and line of sight `los` (from [`sight_radius`]) sees actors on
/// cell `p` of the current level: the viewer's own cell always, else a cell in line of sight
/// that is lit enough for the eye or within its darkvision; on the overworld any cell in line of
/// sight.
pub(crate) fn sees(
    state: &WorldState,
    content: &Content,
    lights: &mut LightCache,
    los: &LineOfSight,
    eye: &VisionProfile,
    p: Pos,
) -> bool {
    sees_by(state, content, lights, los, eye, p).is_some()
}

/// What settled that a viewer sees a cell ([`sees_by`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum By {
    /// Its own cell, the overworld's line of sight, or light enough for its eyes.
    Light,
    /// Its darkvision, before the light was looked at: the cell may be lit enough too
    /// ([`lit_for`] says).
    Darkvision,
}

/// [`sees`], and what settled it: darkvision is asked first, and within it the light is not
/// computed (slipping past, `world/slip.rs`, asks [`lit_for`] when it matters).
pub(crate) fn sees_by(
    state: &WorldState,
    content: &Content,
    lights: &mut LightCache,
    los: &LineOfSight,
    eye: &VisionProfile,
    p: Pos,
) -> Option<By> {
    if p == los.origin {
        return Some(By::Light);
    }
    if !los.contains(p) {
        return None;
    }
    let level = state.current();
    if level.kind == LevelKind::Overworld {
        return Some(By::Light);
    }
    // Darkvision first: within it the light does not matter (and need not be computed).
    let dist2 = p.dist2(los.origin);
    if eye.in_darkvision(dist2) {
        return Some(By::Darkvision);
    }
    lit_for(state, content, lights, eye, p).then_some(By::Light)
}

/// Whether cell `p` of the current level is lit enough for `eye` to see actors on it by light
/// alone, whatever its darkvision: a transparent cell whose perceived light reaches the eye's
/// threshold.
pub(crate) fn lit_for(
    state: &WorldState,
    content: &Content,
    lights: &mut LightCache,
    eye: &VisionProfile,
    p: Pos,
) -> bool {
    if state.current().is_opaque(content, p) {
        return false;
    }
    // Bounce light only adds: if the light without it is enough, so is all of it.
    eye.lit_enough(light(state, content, lights, p, eye, false))
        || eye.lit_enough(light(state, content, lights, p, eye, true))
}

/// The light `eye` perceives on transparent cell `p` of the current level: exactly the light the
/// player's perception finds there with the same eye (ambient light, the overworld's night light,
/// sky light, bounce light, a special room's glow, every source's contribution, and the sun's,
/// each as a special room there leaves it: a shadowed room keeps part of each). Bounce light needs
/// the direct light within [`BOUNCE_STEPS`] of the cell, so the sources and the sunlight of that
/// small area are evaluated (source masks come from, and go into, `lights`). Without `bounce`,
/// the same light but for bounce light: no more than it, and much cheaper.
pub(crate) fn light(
    state: &WorldState,
    content: &Content,
    lights: &mut LightCache,
    p: Pos,
    eye: &VisionProfile,
    bounce: bool,
) -> Light {
    let level = state.current();
    let margin = if bounce { BOUNCE_STEPS } else { 0 };
    let Some(area) = Area::around([p], margin, level.width(), level.height()) else {
        return Light::DARK;
    };
    let is_opaque = |q: Pos| level.is_opaque(content, q);
    let terrain = |q: Pos| level.terrain.get(q).map(|t| content.terrain(*t));
    let exposed = |q: Pos| terrain(q).is_some_and(|t| !t.roofed);

    // The sources that light any cell of the area (a source lights cells within its radius, and
    // the area's cells lie within 2 · margin of `p`).
    let near = lights.near(state, content, p, 2 * margin);
    // Under the open sky: the sun and every moon that shines through the weather's clouds, where
    // nothing taller shades the area's cells (as the player's perception has them), and the
    // sky's glow.
    let sky = state.sky(content);
    let mut bodies: Vec<([i16; 2], Light)> = Vec::new();
    if let Some(sky) = &sky {
        if sky.sun.is_up() {
            bodies.push((sky.sun.shadow, sky.sun.light));
        }
        for moon in sky.moons.iter().filter(|m| m.shines()) {
            bodies.push((moon.shadow, moon.light));
        }
    }
    let beams: Vec<Sunlight> = (bodies.into_iter())
        .map(|(shadow, light)| {
            let height = |q: Pos| terrain(q).map_or(0, |t| t.height);
            let tallest = content.tallest_terrain();
            Sunlight::compute(shadow, light, area, tallest, height, exposed, is_opaque)
        })
        .collect();
    let glow = sky.as_ref().map_or(Light::DARK, |s| eye.perceive(s.glow()));
    let night = match &sky {
        Some(sky) if level.kind == LevelKind::Overworld => daylight::overworld_night(sky.sun.sky),
        _ => Light::DARK,
    };
    let ambient = eye.perceive(level.ambient.saturating_add(night));

    let bounce = if bounce {
        let tint = |q: Pos| {
            let rgb = terrain(q).map(|t| t.bounce);
            rgb.map_or([0; 3], |c| [c.r, c.g, c.b])
        };
        let masks = (near.iter())
            .filter(|(_, mask)| mask.overlaps(area.min, area.max))
            .map(|(_, mask)| mask.cells.as_slice());
        let beams = beams.iter().map(|s| s.cells.as_slice());
        Bounce::compute(area, masks.chain(beams), is_opaque, tint).at(p)
    } else {
        Light::DARK
    };
    let bounce = bounce.saturating_add(eye.perceive(level.room_glow(content, p)));
    let indirect = if !glow.is_dark() && exposed(p) {
        bounce.saturating_add(glow)
    } else {
        bounce
    };

    // In the order the player's perception adds them: base light, the sources, the sun and the
    // moons, each as a special room there leaves it (a shadowed room keeps part of each).
    let shade = |light: Light| level.room_shade(content, p, light);
    let mut light = shade(ambient.saturating_add(indirect));
    for (_, mask) in &near {
        if let Some(c) = mask.at(p) {
            light = light.saturating_add(shade(eye.perceive(c)));
        }
    }
    for beam in &beams {
        light = light.saturating_add(shade(eye.perceive(beam.at(p))));
    }
    light
}
