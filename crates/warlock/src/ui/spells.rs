//! Spell effects on screen (spec §4.8 "Spell effects"): each perceived spell event
//! ([`Event::Spell`]: a trap's fireball or lightning bolt, later spells) plays its effect from
//! `data/effects.toml` over the cells the event says the player saw, never more.
//!
//! - A **burst** (a fireball) flies from its origin to its center when they differ (its
//!   `projectile`, trailing its `flight` particles), then grows from its core to its edge, white-
//!   hot at the core and cooling outward (its colors, hottest first, and its glyphs by heat,
//!   flickering), burns, and cools from the edge in; embers fly at the impact and a faint smoke
//!   rises as it fades (its `impact` and `linger` particles).
//! - A **bolt** (a lightning bolt) draws out along its path and flashes `flashes` times: a jagged
//!   line of line glyphs that changes shape with every flash and flicker, forking now and then,
//!   then a dying afterglow.
//! - Both are light themselves (drawn unlit, glowing, on top of everything else on the map), light
//!   the scene ([`SpellLights`], visual only), and while they peak darken everything else
//!   (`darken`, with the `screen_fades` effect). Landing near the player they shake the view
//!   (`shake`, with the `screen_shake` effect; [`Spells::shake`] moves the map camera by whole
//!   pixels).
//!
//! Everything is a function of [`VisualTime`] (`render::spells`). The effects of one turn play
//! one after another. New input does not cut them short (they hold nothing back); a level change
//! does. With the `motion` effect off (or reduced effects) an effect is a brief still flash of its
//! cells: no travel, light, darkening, or shake. With frozen visual time (`--freeze-time T`) each
//! effect of the last turn shows its moment `T` seconds after it began.

use super::glyphs;
use crate::game::Game;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::lighting::{SpellLight, SpellLights, TERRAIN_FLOOR, VisualLight};
use crate::render::particles::{self, Particle, Source, gradient};
use crate::render::post::emissive;
use crate::render::spells::{
    self as model, JAG, Phase, Stroke, Timeline, bolt_strokes, burst_distance, burst_heat,
    heat_glyph, shake_amplitude, shake_offset, shape_slot, still_heat, strength,
};
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::color::Rgb;
use warlock_core::content::{Content, EffectDef, FILL_GLYPH, ParticleTrigger};
use warlock_core::perception::CellView;
use warlock_core::{Event, Pos, SpellShape};

/// The pause between two effects of one turn, in seconds.
const GAP: f32 = 0.08;
/// Seed bit that keeps spell effects' randomness apart from other sources'.
const SPELL_KEY: u64 = 1 << 60;
/// How bright a bolt's forks are, relative to its line.
const FORK: f32 = 0.55;
/// Below this alpha a cooling cell's glyph is not drawn.
const FAINT: f32 = 0.02;
/// A burst's body is as opaque as its `fill` from this heat up, thinner below.
const FILL_FULL: f32 = 0.5;
/// A burst's body is drawn a little larger than its cell, so neighbors blend without seams.
const FILL_SCALE: f32 = 1.1;
/// How bright a bolt's still flash is (the `motion` effect off), as a strength.
const STILL_BOLT: f32 = 0.8;

fn cell(p: Pos) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// One perceived effect playing.
#[derive(Clone, Debug, PartialEq)]
struct Cast {
    /// Its effect: an index into [`Content::effects`].
    effect: usize,
    origin: Pos,
    shape: SpellShape,
    /// The cells the player saw it on, sorted.
    cells: Vec<Pos>,
    /// When it began, in visual time.
    start: f32,
    timeline: Timeline,
    /// Until when (after `start`) anything of it shows: its particles may outlive it.
    lasts: f32,
    key: u64,
    /// Whether it plays animated (the `motion` effect): else a still flash of its cells.
    animated: bool,
    /// Its shake where the player is: amplitude in logical pixels and duration.
    shake: Option<(f32, f32)>,
}

impl Cast {
    fn def<'a>(&self, content: &'a Content) -> &'a EffectDef {
        &content.effects()[self.effect]
    }

    fn shows(&self, p: Pos) -> bool {
        self.cells.binary_search(&p).is_ok()
    }

    /// Where a burst's projectile flies from and to (cell top-lefts, in level cells).
    fn flight(&self) -> Option<(Vec2, Vec2)> {
        let SpellShape::Burst { center, .. } = self.shape else {
            return None;
        };
        let (from, to) = (cell(self.origin), cell(center));
        let way = (to - from).normalize_or_zero();
        (way != Vec2::ZERO).then(|| (from + way * 0.5, to))
    }

    /// Where its projectile is `x` (0 to 1) through its travel.
    fn projectile_at(&self, x: f32) -> Option<Vec2> {
        self.flight()
            .map(|(from, to)| from.lerp(to, x.clamp(0.0, 1.0)))
    }
}

/// How far an effect travels before its impact, in cells: a burst's projectile from the edge of
/// its origin to its center, a bolt drawing out along its path.
fn travel_distance(origin: Pos, shape: &SpellShape) -> f32 {
    match shape {
        SpellShape::Burst { center, .. } if *center != origin => {
            (cell(*center) - cell(origin)).length() - 0.5
        }
        SpellShape::Burst { .. } => 0.0,
        SpellShape::Bolt { path } => path.len() as f32,
    }
}

/// The longest life of `def`'s particles of trigger `on`.
fn particle_life(def: &EffectDef, on: ParticleTrigger) -> f32 {
    (def.particles.iter())
        .filter(|e| e.on == on)
        .map(|e| e.life)
        .fold(0.0, f32::max)
}

/// The effects of the last perceived spell events, playing or just over.
#[derive(Resource, Debug, Default)]
pub struct Spells {
    casts: Vec<Cast>,
    /// The game's applied-action count whose outcome was read last.
    read: Option<u64>,
}

impl Spells {
    /// Casts for the perceived spell `events` of one outcome, the first beginning at `now`, each
    /// after the one before has struck; with frozen visual time all at visual time 0, so the
    /// frozen moment is how long after they began; without `motion`, still flashes. `viewer`
    /// decides how hard each shakes the view.
    fn start(
        &mut self,
        content: &Content,
        events: &[Event],
        now: f32,
        frozen: bool,
        motion: bool,
        viewer: Pos,
        key: u64,
    ) {
        let mut next = now;
        for (i, event) in events.iter().enumerate() {
            let Event::Spell {
                effect,
                origin,
                shape,
                cells,
            } = event
            else {
                continue;
            };
            let Some(index) = content.effects().iter().position(|e| e.id == *effect) else {
                continue;
            };
            let def = &content.effects()[index];
            let timeline = if motion {
                Timeline::new(def, travel_distance(*origin, shape))
            } else {
                Timeline::still()
            };
            let start = if frozen { 0.0 } else { next };
            next = start + timeline.linger_start() + GAP;
            let mut sorted = cells.clone();
            sorted.sort();
            sorted.dedup();
            let near = (cells.iter())
                .map(|p| (p.dist2(viewer) as f32).sqrt())
                .fold(f32::MAX, f32::min);
            let shake = (def.shake)
                .map(|s| (shake_amplitude(&s, near), s.seconds))
                .filter(|(amplitude, _)| *amplitude > 0.0);
            let lasts = [
                timeline.end(),
                timeline.impact_start() + def.grow + particle_life(def, ParticleTrigger::Impact),
                timeline.linger_start() + particle_life(def, ParticleTrigger::Linger),
                timeline.impact_start() + def.darken.map_or(0.0, |d| d.seconds),
                timeline.impact_start() + shake.map_or(0.0, |s| s.1),
            ]
            .into_iter()
            .fold(0.0, f32::max);
            self.casts.push(Cast {
                effect: index,
                origin: *origin,
                shape: shape.clone(),
                cells: sorted,
                start,
                timeline,
                lasts,
                key: SPELL_KEY | key.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ ((i as u64) << 40),
                animated: motion,
                shake,
            });
        }
    }

    /// The view's shake at visual `time`, in whole physical pixels (`scale` of them to a logical
    /// pixel): every playing effect that landed near the player, added up, at most the largest
    /// of them on each axis.
    pub fn shake(&self, time: f32, scale: f32) -> Vec2 {
        let mut sum = Vec2::ZERO;
        let mut most = 0.0f32;
        for c in self.casts.iter().filter(|c| c.animated) {
            if let Some((amplitude, seconds)) = c.shake {
                let since = time - c.start - c.timeline.impact_start();
                sum += shake_offset(since, seconds, amplitude, c.key);
                most = most.max(amplitude);
            }
        }
        (sum.clamp(Vec2::splat(-most), Vec2::splat(most)) * scale).round()
    }

    /// This frame's light of the playing effects and how much they darken the rest of the view
    /// (with `darken` on: the `screen_fades` effect).
    fn lights(&self, content: &Content, time: f32, darken: bool) -> SpellLights {
        let mut out = SpellLights::default();
        for c in self.casts.iter().filter(|c| c.animated) {
            let def = c.def(content);
            let age = time - c.start;
            let phase = c.timeline.phase(age);
            if darken && let Some(d) = def.darken {
                // From the moment it is at its brightest: a bolt as it strikes, a burst as it
                // bursts (its projectile is dim).
                let from = match c.shape {
                    SpellShape::Bolt { .. } => 0.0,
                    SpellShape::Burst { .. } => c.timeline.impact_start(),
                };
                let since = age - from;
                out.dim = out
                    .dim
                    .max(d.amount * model::darken_curve(since, d.seconds));
            }
            let Some(light) = def.light else {
                continue;
            };
            let k = strength(def, phase);
            if k <= 0.0 {
                continue;
            }
            out.open.extend(&c.cells);
            let rgb = Vec3::from_array(light.color.map(f32::from)) / 255.0 * k;
            let mut add = |at: Vec2| {
                out.lights.push(SpellLight {
                    at: at + Vec2::splat(0.5),
                    rgb,
                    radius: light.radius,
                })
            };
            match (&c.shape, phase) {
                (SpellShape::Burst { .. }, Phase::Travel(x)) => {
                    if let Some(at) = c.projectile_at(x) {
                        add(at);
                    }
                }
                (SpellShape::Burst { center, .. }, _) => add(cell(*center)),
                (SpellShape::Bolt { path }, phase) => {
                    let shown = match phase {
                        Phase::Travel(x) => ((x * path.len() as f32).ceil() as usize).max(1),
                        _ => path.len(),
                    };
                    // Every other cell, and the last one drawn.
                    let lit = path[..shown.min(path.len())].iter().enumerate();
                    for (i, &p) in lit {
                        if i % 2 == 1 || i + 1 == shown {
                            add(cell(p));
                        }
                    }
                }
            }
        }
        out.open.sort();
        out.open.dedup();
        out
    }
}

/// Reads the spell events of each new outcome (once, even one that took no turn: the dev
/// action), and drops effects that are over; a level change ends them all.
pub(super) fn track(
    game: Res<Game>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    mut spells: ResMut<Spells>,
) {
    let now = visual.seconds;
    let over = |c: &Cast| now - c.start >= c.lasts;
    if spells.casts.iter().any(over) {
        spells.casts.retain(|c| !over(c));
    }
    if game.is_added() {
        // A new run counts its actions from the start.
        *spells = Spells::default();
    }
    if !game.is_changed() || spells.read.replace(game.applied) == Some(game.applied) {
        return;
    }
    let events = &game.last_outcome.events;
    if events
        .iter()
        .any(|e| matches!(e, Event::LevelChanged { .. }))
    {
        spells.casts.clear();
    }
    let content = game.world.content();
    let viewer = game.world.perception().viewer();
    let (frozen, motion) = (visual.frozen, effects.motion);
    spells.start(content, events, now, frozen, motion, viewer, game.applied);
}

/// This frame's light of the playing effects and their darkening, for the visual lighting.
pub(super) fn lights(
    game: Res<Game>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    spells: Res<Spells>,
    mut out: ResMut<SpellLights>,
) {
    let now = spells.lights(game.world.content(), visual.seconds, effects.screen_fades);
    if *out != now {
        *out = now;
    }
}

/// Draws the playing effects on top of everything else on the map: their glyphs, light
/// themselves, and their particles.
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    spells: Res<Spells>,
    mut free: ResMut<FreeGlyphs>,
    mut scratch: Local<Vec<Particle>>,
) {
    if spells.casts.is_empty() {
        return;
    }
    let view = game.world.perception();
    let content = view.content();
    let time = visual.seconds;
    let paint = |c: Rgb| settings.palette.apply(c, Role::Fg);
    let aspect = if grid.cell.x > 0.0 {
        grid.cell.y / grid.cell.x
    } else {
        2.0
    };
    let out = &mut *scratch;
    for c in &spells.casts {
        let def = c.def(content);
        let age = time - c.start;
        let glow = |color: LinearRgba, amount: f32| {
            if effects.glow {
                emissive(color, def.glow * amount)
            } else {
                color
            }
        };
        let mut put = |at: Vec2, ch: char, color: LinearRgba, scale: f32, turn: f32| {
            free.push(FreeGlyph {
                pos: grid.to_screen(at),
                ch,
                color,
                scale,
                slant: 0.0,
                turn,
            });
        };
        // Particles first: embers fly out from behind the glyphs.
        if effects.particles {
            out.clear();
            particles_of(c, def, age, aspect, paint, out);
            for p in out.iter() {
                let home = Pos::new(
                    (p.pos.x + 0.5).floor() as i32,
                    (p.pos.y + 0.5).floor() as i32,
                );
                if view.is_visible(home) || c.shows(home) {
                    let color = if p.emissive {
                        p.color
                    } else {
                        glyphs::lit(p.color, &light, p.pos, cell(home), TERRAIN_FLOOR)
                    };
                    put(p.pos, p.ch, color, p.scale, 0.0);
                }
            }
        }
        let phase = c.timeline.phase(age);
        let glyphs_of: Vec<char> = def.glyphs.chars().collect();
        match (&c.shape, c.animated) {
            (SpellShape::Burst { center, radius }, true) => {
                if let Phase::Travel(x) = phase
                    && let (Some(at), Some(ch)) = (c.projectile_at(x), def.projectile)
                {
                    let home = Pos::new((at.x + 0.5).floor() as i32, (at.y + 0.5).floor() as i32);
                    if view.is_visible(home) || c.shows(home) {
                        let color = gradient(&def.colors, 0.15, paint);
                        put(at, ch, glow(color, 1.0), 1.0, 0.0);
                    }
                }
                let slot = shape_slot(age, def.flicker);
                for &p in &c.cells {
                    let d = burst_distance(p.x - center.x, p.y - center.y, aspect);
                    let mut rng = particles::Rng::new(c.key ^ particles::cell_key(p.x, p.y), slot);
                    let Some(heat) = burst_heat(d, *radius, def.grow, phase, rng.signed()) else {
                        continue;
                    };
                    let alpha = (heat / 0.12).min(1.0);
                    if alpha < FAINT {
                        continue;
                    }
                    burst_cell(&mut put, p, heat, alpha, def, &glyphs_of, &glow, paint);
                }
            }
            (SpellShape::Bolt { path }, true) => {
                let k = strength(def, phase);
                if k <= 0.0 {
                    continue;
                }
                let shown = match phase {
                    Phase::Travel(x) => ((x * path.len() as f32).ceil() as usize).max(1),
                    _ => path.len(),
                };
                let slot = shape_slot(age, def.flicker);
                let open = |q: Pos| {
                    c.shows(q)
                        || matches!(view.cell(q), CellView::Visible(t)
                            if content.terrain(t).transparent)
                };
                let branches = if matches!(phase, Phase::Impact(_)) {
                    def.branches
                } else {
                    0.0
                };
                let strokes =
                    bolt_strokes(c.origin, path, aspect, JAG, branches, c.key, slot, open);
                for s in strokes {
                    if s.index < shown && (s.fork || c.shows(path[s.index])) {
                        bolt_stroke(&mut put, s, k, def, &glyphs_of, &glow, paint, aspect);
                    }
                }
            }
            (shape, false) => {
                // Still: a brief flash of its cells, no motion.
                if !matches!(phase, Phase::Impact(_)) {
                    continue;
                }
                match shape {
                    SpellShape::Burst { center, radius } => {
                        for &p in &c.cells {
                            let d = burst_distance(p.x - center.x, p.y - center.y, aspect);
                            let heat = still_heat(d, *radius);
                            burst_cell(&mut put, p, heat, 1.0, def, &glyphs_of, &glow, paint);
                        }
                    }
                    SpellShape::Bolt { path } => {
                        let never = |_: Pos| false;
                        for s in bolt_strokes(c.origin, path, aspect, 0.0, 0.0, c.key, 0, never) {
                            if c.shows(path[s.index]) {
                                let k = STILL_BOLT;
                                bolt_stroke(&mut put, s, k, def, &glyphs_of, &glow, paint, aspect);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Draws one cell of a burst at `heat` (1 white-hot, toward 0 as it cools) and opacity `alpha`:
/// its glowing body (`fill`, a shade cooler and thinner toward the edge), then its glyph by heat.
fn burst_cell(
    put: &mut impl FnMut(Vec2, char, LinearRgba, f32, f32),
    p: Pos,
    heat: f32,
    alpha: f32,
    def: &EffectDef,
    glyphs: &[char],
    glow: &impl Fn(LinearRgba, f32) -> LinearRgba,
    paint: impl Fn(Rgb) -> Rgb + Copy,
) {
    if def.fill > 0.0 {
        let mut body = glow(gradient(&def.colors, 1.25 - heat, paint), heat * 0.5);
        body.alpha = alpha * def.fill * (heat / FILL_FULL).min(1.0);
        put(cell(p), FILL_GLYPH, body, FILL_SCALE, 0.0);
    }
    let mut color = glow(gradient(&def.colors, 1.0 - heat, paint), heat);
    color.alpha = alpha;
    put(
        cell(p),
        glyphs[heat_glyph(heat, glyphs.len())],
        color,
        1.0,
        0.0,
    );
}

/// Draws one stroke of a bolt at strength `k`: its line glyph (its forks' for a fork) turned along
/// it and stretched to its length, white at its brightest, through blue to violet as it dims;
/// forks dimmer. Cells are `aspect` times taller than wide.
fn bolt_stroke(
    put: &mut impl FnMut(Vec2, char, LinearRgba, f32, f32),
    s: Stroke,
    k: f32,
    def: &EffectDef,
    glyphs: &[char],
    glow: &impl Fn(LinearRgba, f32) -> LinearRgba,
    paint: impl Fn(Rgb) -> Rgb,
    aspect: f32,
) {
    let k = if s.fork { k * FORK } else { k };
    let mut color = glow(gradient(&def.colors, 1.0 - k, paint), k);
    color.alpha = (k / 0.1).min(1.0);
    let ch = match (s.fork, glyphs.last()) {
        (true, Some(&fork)) => fork,
        _ => glyphs[0],
    };
    // In cell widths on screen: a glyph is one cell width across at scale 1.
    let run = (s.to - s.from) * Vec2::new(1.0, aspect);
    let mid = (s.from + s.to) * 0.5;
    put(mid - 0.5, ch, color, run.length(), run.y.atan2(run.x));
}

/// The particles of cast `c` (effect `def`) `age` seconds after it began: its projectile's
/// `flight` trail, `impact` bursts from each cell as the effect reaches it, `linger` bursts from
/// each cell as it starts to fade. Cells are `aspect` times taller than wide.
fn particles_of(
    c: &Cast,
    def: &EffectDef,
    age: f32,
    aspect: f32,
    paint: impl Fn(Rgb) -> Rgb + Copy,
    out: &mut Vec<Particle>,
) {
    let source = |origin: Vec2| Source {
        origin,
        down: Source::UPRIGHT,
        paint,
    };
    let edge = match c.shape {
        SpellShape::Burst { radius, .. } => radius.max(0) as f32 + 0.5,
        SpellShape::Bolt { .. } => 1.0,
    };
    let center = match c.shape {
        SpellShape::Burst { center, .. } => Some(center),
        SpellShape::Bolt { .. } => None,
    };
    for (n, e) in def.particles.iter().enumerate() {
        let key = c.key ^ ((n as u64) << 32);
        match e.on {
            ParticleTrigger::Flight if c.animated => {
                let span = (0.0, c.timeline.travel);
                let along = |born: f32| c.projectile_at(born / c.timeline.travel.max(1e-6));
                if c.timeline.travel > 0.0 {
                    let origin = |born: f32| along(born).unwrap_or(Vec2::ZERO);
                    particles::trail(e, key, span, origin, source(Vec2::ZERO), age, out);
                }
            }
            ParticleTrigger::Impact | ParticleTrigger::Linger => {
                for &p in &c.cells {
                    // A burst reaches its cells as it grows; a bolt strikes them all at once.
                    let reach = center.map_or(0.0, |o| {
                        let d = burst_distance(p.x - o.x, p.y - o.y, aspect);
                        def.grow * (d / edge).min(1.0)
                    });
                    let at = match e.on {
                        ParticleTrigger::Impact if c.animated => c.timeline.impact_start() + reach,
                        ParticleTrigger::Impact => 0.0,
                        _ => c.timeline.linger_start(),
                    };
                    let seed = key ^ particles::cell_key(p.x, p.y);
                    particles::burst(e, seed, source(cell(p)), age - at, out);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;

    fn arena() -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        World::new(content, "arena", None).expect("arena")
    }

    fn cast_of(world: &mut World, action: &str, now: f32, frozen: bool, motion: bool) -> Spells {
        let out = world.apply(&action.parse().expect("parses"));
        let mut spells = Spells::default();
        let viewer = world.perception().viewer();
        spells.start(world.content(), &out.events, now, frozen, motion, viewer, 1);
        spells
    }

    #[test]
    fn a_fireball_at_the_feet_shakes_and_darkens_a_far_one_only_darkens() {
        let mut world = arena();
        let player = world.player_pos();
        let content = world.content().clone();
        let fireball = content.effect("fireball").expect("fireball").clone();
        let near = format!("effect:fireball@{},{}", player.x, player.y);
        let spells = cast_of(&mut world, &near, 10.0, false, true);
        let c = &spells.casts[0];
        assert_eq!(
            (c.start, c.timeline.travel),
            (10.0, 0.0),
            "struck where it began"
        );
        let (amplitude, _) = c.shake.expect("it shakes the view");
        assert_eq!(amplitude, fireball.shake.expect("shake").pixels);
        let shaken = (0..30).any(|i| spells.shake(10.0 + i as f32 * 0.01, 1.0) != Vec2::ZERO);
        assert!(shaken);
        assert_eq!(spells.shake(20.0, 1.0), Vec2::ZERO, "and stops");
        let peak = spells.lights(&content, 10.1, true);
        assert!(peak.dim > 0.5 && !peak.lights.is_empty());
        assert_eq!(
            spells.lights(&content, 10.1, false).dim,
            0.0,
            "screen fades off"
        );
        let after = spells.lights(&content, 10.0 + c.lasts, true);
        assert_eq!((after.dim, after.lights.len()), (0.0, 0));

        // Across the pit: lit and darkening, no shake.
        let far = cast_of(&mut world, "effect:fireball@40,10", 0.0, false, true);
        assert_eq!(far.casts[0].shake, None);
    }

    #[test]
    fn without_motion_an_effect_is_a_still_flash() {
        let mut world = arena();
        let player = world.player_pos();
        let action = format!("effect:fireball@{},{}", player.x, player.y);
        let spells = cast_of(&mut world, &action, 0.0, false, false);
        let content = world.content().clone();
        assert_eq!(spells.casts[0].timeline, Timeline::still());
        assert!(!spells.casts[0].animated);
        let lights = spells.lights(&content, 0.1, true);
        assert_eq!(
            (lights.dim, lights.lights.len()),
            (0.0, 0),
            "no light, no darkening"
        );
        assert_eq!(spells.shake(0.1, 1.0), Vec2::ZERO);
    }

    #[test]
    fn effects_of_a_turn_play_one_after_another_or_all_at_the_frozen_moment() {
        let mut world = arena();
        let mut events = world
            .apply(&"effect:fireball@40,10".parse().expect("parses"))
            .events;
        events.extend(
            world
                .apply(&"effect:lightning_bolt@30,12>50,12".parse().expect("parses"))
                .events,
        );
        assert_eq!(events.len(), 2);
        let content = world.content().clone();
        let viewer = world.perception().viewer();
        let mut spells = Spells::default();
        spells.start(&content, &events, 5.0, false, true, viewer, 1);
        let (a, b) = (&spells.casts[0], &spells.casts[1]);
        assert!((b.start - (a.start + a.timeline.linger_start() + GAP)).abs() < 1e-6);
        assert_ne!(a.key, b.key);
        let mut frozen = Spells::default();
        frozen.start(&content, &events, 0.12, true, true, viewer, 1);
        assert!(frozen.casts.iter().all(|c| c.start == 0.0));
        // The bolt draws out along its path, then lights it every other cell.
        let bolt = &frozen.casts[1];
        let SpellShape::Bolt { path } = &bolt.shape else {
            panic!("a bolt");
        };
        assert!(
            (bolt.timeline.travel - path.len() as f32 / content.effects()[bolt.effect].speed).abs()
                < 1e-6
        );
        let mut alone = Spells::default();
        alone.casts.push(bolt.clone());
        let lit = alone.lights(&content, bolt.timeline.impact_start() + 0.01, true);
        assert_eq!(lit.lights.len(), path.len() / 2 + path.len() % 2);
    }

    #[test]
    fn a_burst_that_flies_lights_its_way_from_its_origin() {
        let mut world = arena();
        let spells = cast_of(&mut world, "effect:fireball@20,10>40,10", 0.0, false, true);
        let content = world.content().clone();
        let c = &spells.casts[0];
        assert!(c.timeline.travel > 0.5, "twenty cells at its speed");
        let early = spells.lights(&content, 0.01, true);
        assert_eq!(early.dim, 0.0, "no darkening before the impact");
        assert!(early.lights[0].at.x < 22.0, "near the origin");
        let late = spells.lights(&content, c.timeline.travel * 0.9, true);
        assert!(late.lights[0].at.x > 37.0, "near the center");
    }
}
