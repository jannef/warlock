//! Missiles in flight (spec §4.8 "Missiles"): every shot and throw the player perceives flies on
//! the free-glyph layer, from the turn's [`Event::Missile`]s, which carry the cells it flew over
//! as far as the player saw them (never more: no ground truth is read here).
//!
//! A missile flies from the edge of its shooter's cell (or from where it came into view) to
//! where it ends, at [`SPEED`] cells a second (at least [`MIN_SECONDS`]), easing out a little; the
//! missiles of one turn fly one after another, in the order they were loosed. Its glyph is its
//! object's `missile` glyph turned along the flight (an arrow `→`), tumbling if it `spins` (a
//! thrown dagger), else its own glyph upright (a sling stone `•`); it fades in and out at the
//! ends and leaves a short, faint trail. It is lit like any free glyph, but a burning one is a
//! light itself: it lights what it passes ([`Transients`]: a moving light of its object's light
//! preset, with its flicker) and sheds embers (its `flight` particles). Where it comes down or
//! strikes, a tiny puff of dust. What it lands as (the arrow on the floor, a burning arrow's
//! light) is held back on the map until it arrives ([`Missiles::arriving`]).
//!
//! Everything is a function of [`VisualTime`] and the turn's events, and drawn only over cells
//! the player sees, but for a burning missile, which is light itself: it is drawn over the cells
//! its event says the player saw it fly over (in line of sight, in the dark too). With the `motion` effect off (or reduced effects) nothing flies: what lands
//! is there at once; the embers and the puff run with `particles`. With frozen visual time a
//! missile is shown halfway, and new input completes every flight.

use super::effects::particle_color;
use super::glyphs;
use crate::game::Game;
use crate::render::free_glyph::{FreeGlyph, FreeGlyphs};
use crate::render::grid::MapGrid;
use crate::render::lighting::{ACTOR_FLOOR, TransientLight, Transients, VisualLight};
use crate::render::particles::{self, Particle, Source};
use crate::render::post::{emissive, in_range};
use crate::render::{Effects, VisualTime};
use crate::settings::{Role, Settings};
use bevy::prelude::*;
use warlock_core::color::Rgb;
use warlock_core::content::{ObjectId, ParticleEmitter, ParticleTrigger};
use warlock_core::{Event, MissileEnd, Pos};

/// How fast a missile flies, in cells per second.
pub const SPEED: f32 = 30.0;
/// The shortest flight, in seconds (a shot at a neighbor).
pub const MIN_SECONDS: f32 = 0.06;
/// The pause between two missiles of one turn, in seconds.
const GAP: f32 = 0.05;
/// How much a flight eases out: 0 is constant speed; it starts this much faster than average and
/// ends this much slower.
const EASE: f32 = 0.25;
/// The share of a flight spent fading out at its end (and a tenth fading in).
const FADE: f32 = 0.15;
/// How fast a tumbling missile turns, in radians per second.
const SPIN: f32 = 14.0;
/// The trail: faint dots behind a missile, in cells behind it and how opaque.
const TRAIL: [(f32, f32); 3] = [(0.35, 0.28), (0.6, 0.16), (0.85, 0.07)];
const TRAIL_GLYPH: char = '·';
/// Seed bit that keeps a flight's particles apart from other sources.
const FLIGHT_KEY: u64 = 1 << 62;

/// The puff of dust where a missile comes down or strikes: a few motes, short-lived.
fn puff() -> ParticleEmitter {
    ParticleEmitter {
        on: ParticleTrigger::Ambient,
        glyphs: "·.'".into(),
        colors: vec![Rgb::new(0xc8, 0xbc, 0xa4), Rgb::new(0x70, 0x66, 0x58)],
        rate: 0.0,
        count: 4,
        life: 0.35,
        velocity: [0.0, -0.5],
        spread: [0.9, 0.5],
        gravity: 2.2,
        offset: [0.0, 0.2],
        area: [0.12, 0.06],
        sway: 0.0,
        scale: 0.6,
        fade: 0.6,
        glow: 0.0,
        opacity: 0.7,
    }
}

fn cell(p: Pos) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// The cell a glyph at `at` (its top-left, in level cells) is over.
fn cell_under(at: Vec2) -> Pos {
    Pos::new((at.x + 0.5).floor() as i32, (at.y + 0.5).floor() as i32)
}

/// A missile on its way.
#[derive(Clone, Debug, PartialEq)]
struct Flight {
    object: ObjectId,
    /// Where its glyph flies from and to (its top-left, in level cells).
    from: Vec2,
    to: Vec2,
    /// The way it flies, a unit vector in cells (zero if the player saw too little to tell).
    way: Vec2,
    end: MissileEnd,
    /// The cell it came down on (or struck someone on), where what it lands as lies.
    lands: Option<Pos>,
    /// The cells it was seen flying over, and the one it was loosed from if seen.
    seen: Vec<Pos>,
    start: f32,
    seconds: f32,
    /// Whether it is drawn flying (the `motion` effect): else it arrives at once.
    flies: bool,
    key: u64,
}

impl Flight {
    /// The flight of a perceived missile: `from` the cell it was loosed from if seen, `path` the
    /// cells it was seen flying over, `end` how it ended on the last of them.
    fn new(object: ObjectId, from: Option<Pos>, path: &[Pos], end: MissileEnd) -> Option<Self> {
        let (&first, &last) = (path.first()?, path.last()?);
        let back = from.unwrap_or(first);
        let way = (cell(last) - cell(back)).normalize_or_zero();
        let start_at = match from {
            Some(f) => cell(f) + way * 0.5,
            None => cell(first) - way * 0.5,
        };
        let (to, lands) = match end {
            MissileEnd::Fell => (cell(last), Some(last)),
            MissileEnd::Struck => (cell(last) - way * 0.3, Some(last)),
            MissileEnd::Unseen => (cell(last) + way * 0.5, None),
        };
        Some(Self {
            object,
            from: start_at,
            to,
            way,
            end,
            lands,
            seen: from.into_iter().chain(path.iter().copied()).collect(),
            start: 0.0,
            seconds: ((to - start_at).length() / SPEED).max(MIN_SECONDS),
            flies: true,
            key: 0,
        })
    }

    /// When it arrives where it ends.
    fn arrival(&self) -> f32 {
        if self.flies {
            self.start + self.seconds
        } else {
            self.start
        }
    }

    /// Where its glyph is at `time`, whether it flies then or not (for the embers it shed).
    fn along(&self, time: f32) -> Vec2 {
        let x = ((time - self.start) / self.seconds).clamp(0.0, 1.0);
        self.from.lerp(self.to, x + EASE * x * (1.0 - x))
    }

    /// Where its glyph is at `time` while it flies; `None` before and after.
    fn at(&self, time: f32) -> Option<Vec2> {
        let x = (time - self.start) / self.seconds;
        (self.flies && (0.0..1.0).contains(&x)).then(|| self.along(time))
    }
}

/// The missiles of the last turn: in flight, or just landed (their puff still settling).
#[derive(Resource, Debug, Default)]
pub struct Missiles {
    flights: Vec<Flight>,
    /// The turn whose events they come from.
    turn: Option<u64>,
}

impl Missiles {
    /// Where missiles still on their way at `time` will lie, and what lies there: the map shows
    /// neither that object nor its light until they arrive.
    pub fn arriving(&self, time: f32) -> Vec<(Pos, ObjectId)> {
        (self.flights.iter())
            .filter(|f| time < f.arrival())
            .filter_map(|f| f.lands.map(|p| (p, f.object)))
            .collect()
    }

    /// Flights for a turn's perceived `events`, the first leaving at `now`, each after the one
    /// before; with frozen visual time all of them halfway; without `motion`, none flying.
    fn start(&mut self, events: &[Event], now: f32, frozen: bool, motion: bool, turn: u64) {
        let mut next = now;
        for (i, event) in events.iter().enumerate() {
            let Event::Missile {
                object,
                from,
                path,
                end,
            } = event
            else {
                continue;
            };
            let Some(mut flight) = Flight::new(*object, *from, path, *end) else {
                continue;
            };
            flight.flies = motion;
            flight.key =
                FLIGHT_KEY | (turn.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ ((i as u64) << 40));
            flight.start = if frozen {
                now - flight.seconds * 0.5
            } else {
                next
            };
            if motion {
                next += flight.seconds + GAP;
            }
            self.flights.push(flight);
        }
    }
}

/// Reads a new turn's missiles; new input completes the ones still flying.
pub(super) fn track(
    game: Res<Game>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    mut missiles: ResMut<Missiles>,
) {
    let now = visual.seconds;
    let settled = |f: &Flight| now >= f.arrival() + puff().life;
    if missiles.flights.iter().any(settled) {
        missiles.flights.retain(|f| !settled(f));
    }
    if !game.is_changed() {
        return;
    }
    missiles.flights.clear();
    let turn = game.world.turn();
    if missiles.turn.replace(turn) == Some(turn) {
        return;
    }
    let events = &game.last_outcome.events;
    missiles.start(events, now, visual.frozen, effects.motion, turn);
}

/// This frame's moving lights (burning missiles in flight) and the lights held back until a
/// burning missile arrives, for the visual lighting.
pub(super) fn lights(
    game: Res<Game>,
    visual: Res<VisualTime>,
    missiles: Res<Missiles>,
    mut transients: ResMut<Transients>,
) {
    let content = game.world.content();
    let time = visual.seconds;
    let mut now = Transients::default();
    for f in &missiles.flights {
        let Some(burning) = content.object(f.object).burning else {
            continue;
        };
        if let Some(at) = f.at(time) {
            now.lights.push(TransientLight {
                at: at + Vec2::splat(0.5),
                preset: burning.light,
                key: f.key,
            });
        }
        if time < f.arrival()
            && let Some(p) = f.lands
        {
            now.held.push((p, burning.light));
        }
    }
    if *transients != now {
        *transients = now;
    }
}

/// The turn on screen of a glyph drawn pointing right that flies `way` (in cells), on cells of
/// `size` pixels: clockwise, y down.
fn screen_turn(way: Vec2, size: Vec2) -> f32 {
    let px = way * size;
    px.y.atan2(px.x)
}

/// Draws the missiles in flight, their trails and embers, and the puffs where they ended, on
/// top of the creatures.
pub(super) fn emit(
    game: Res<Game>,
    grid: Res<MapGrid>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    settings: Res<Settings>,
    light: Res<VisualLight>,
    missiles: Res<Missiles>,
    mut free: ResMut<FreeGlyphs>,
    mut scratch: Local<Vec<Particle>>,
) {
    if missiles.flights.is_empty() {
        return;
    }
    let view = game.world.perception();
    let content = view.content();
    let time = visual.seconds;
    let paint = |c: Rgb| settings.palette.apply(c, Role::Fg);
    let source = |origin: Vec2| Source {
        origin,
        down: Source::UPRIGHT,
        paint,
    };
    let out = &mut *scratch;
    out.clear();
    let puff = puff();
    // The dark cells burning missiles were seen over: their embers show there too.
    let mut glowing: Vec<Pos> = Vec::new();
    for f in &missiles.flights {
        let def = content.object(f.object);
        if def.burning.is_some() {
            glowing.extend(&f.seen);
        }
        // Where it shows: over the cells the player sees, and a burning one over every cell it
        // was seen flying over.
        let shows = |p: Pos| view.is_visible(p) || (def.burning.is_some() && f.seen.contains(&p));
        if effects.particles {
            if f.flies {
                let flight = |e: &&ParticleEmitter| e.on == ParticleTrigger::Flight;
                for (i, e) in def.particles.iter().filter(flight).enumerate() {
                    let span = (f.start, f.start + f.seconds);
                    let key = f.key ^ i as u64;
                    let along = |born| f.along(born);
                    particles::trail(e, key, span, along, source(Vec2::ZERO), time, out);
                }
            }
            if f.end != MissileEnd::Unseen {
                particles::burst(&puff, f.key, source(f.to), time - f.arrival(), out);
            }
        }
        let Some(at) = f.at(time) else {
            continue;
        };
        let x = (time - f.start) / f.seconds;
        let alpha = particles::fade(x, FADE);
        let base = particles::linear(paint(def.fg));
        // A burning missile is light; anything else is lit where it flies.
        let shade = |color: LinearRgba, at: Vec2| {
            let lit = match def.burning {
                Some(_) => color,
                None => glyphs::lit(color, &light, at, at, ACTOR_FLOOR),
            };
            let lit = in_range(lit);
            if def.burning.is_some() && effects.glow {
                emissive(lit, def.glow)
            } else {
                lit
            }
        };
        for &(back, opacity) in &TRAIL {
            let behind = at - f.way * back;
            if f.way != Vec2::ZERO && shows(cell_under(behind)) {
                let mut color = shade(base, behind);
                color.alpha = alpha * opacity;
                free.push(FreeGlyph {
                    pos: grid.to_screen(behind),
                    ch: TRAIL_GLYPH,
                    color,
                    scale: 0.8,
                    slant: 0.0,
                    turn: 0.0,
                });
            }
        }
        if !shows(cell_under(at)) {
            continue;
        }
        let turn = if def.spins {
            let sense = if f.way.x < 0.0 { -1.0 } else { 1.0 };
            sense * SPIN * (time - f.start)
        } else if def.missile.is_some() {
            screen_turn(f.way, grid.cell)
        } else {
            0.0
        };
        let mut color = shade(base, at);
        color.alpha = alpha;
        free.push(FreeGlyph {
            pos: grid.to_screen(at),
            ch: def.missile.unwrap_or(def.glyph),
            color,
            scale: 1.0,
            slant: 0.0,
            turn,
        });
    }
    for p in out.iter() {
        let home = cell_under(p.pos);
        if view.is_visible(home) || glowing.contains(&home) {
            free.push(FreeGlyph {
                pos: grid.to_screen(p.pos),
                ch: p.ch,
                color: particle_color(p, home, &light),
                scale: p.scale,
                slant: 0.0,
                turn: 0.0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::content::ObjectId;

    fn flight(from: Option<(i32, i32)>, path: &[(i32, i32)], end: MissileEnd) -> Flight {
        let path: Vec<Pos> = path.iter().map(|&(x, y)| Pos::new(x, y)).collect();
        let from = from.map(|(x, y)| Pos::new(x, y));
        Flight::new(ObjectId(0), from, &path, end).expect("a flight")
    }

    #[test]
    fn missiles_fly_from_the_shooters_edge_to_where_they_end() {
        let f = flight(
            Some((2, 5)),
            &[(3, 5), (4, 5), (5, 5), (6, 5), (7, 5)],
            MissileEnd::Fell,
        );
        assert_eq!(
            f.from,
            Vec2::new(2.5, 5.0),
            "the edge of the shooter's cell"
        );
        assert_eq!(f.to, Vec2::new(7.0, 5.0), "comes to rest on the last cell");
        assert_eq!(f.way, Vec2::X);
        assert!((f.seconds - 4.5 / SPEED).abs() < 1e-6);
        assert_eq!(f.lands, Some(Pos::new(7, 5)));
        assert_eq!(f.at(-0.01), None, "not yet");
        assert_eq!(f.at(0.0), Some(f.from));
        let mid = f.at(f.seconds * 0.5).expect("in flight");
        assert!(
            mid.x > 4.75 && mid.x < 6.0,
            "eased out: past halfway at half time"
        );
        assert_eq!(f.at(f.seconds), None, "arrived");
        // Striking someone, it stops short of their center; flying out of sight, it leaves the
        // last cell seen. Seen only from the second cell on, it comes into view there.
        let struck = flight(Some((2, 5)), &[(3, 5), (4, 5)], MissileEnd::Struck);
        assert_eq!(struck.to, Vec2::new(3.7, 5.0));
        let gone = flight(None, &[(3, 5), (4, 6)], MissileEnd::Unseen);
        assert_eq!(gone.lands, None);
        let way = Vec2::new(1.0, 1.0).normalize();
        assert!((gone.from - (Vec2::new(3.0, 5.0) - way * 0.5)).length() < 1e-5);
        assert!((gone.to - (Vec2::new(4.0, 6.0) + way * 0.5)).length() < 1e-5);
        let short = flight(Some((1, 1)), &[(2, 1)], MissileEnd::Fell);
        assert_eq!(short.seconds, MIN_SECONDS);
    }

    #[test]
    fn a_turns_missiles_fly_one_after_another_and_hold_back_what_they_land_as() {
        let missile = |x: i32| Event::Missile {
            object: ObjectId(3),
            from: Some(Pos::new(0, 0)),
            path: (1..=x).map(|i| Pos::new(i, 0)).collect(),
            end: MissileEnd::Fell,
        };
        let mut m = Missiles::default();
        m.start(&[missile(6), missile(3)], 10.0, false, true, 1);
        let (a, b) = (&m.flights[0], &m.flights[1]);
        assert_eq!(a.start, 10.0);
        assert!((b.start - (a.arrival() + GAP)).abs() < 1e-6);
        assert_ne!(a.key, b.key);
        assert_eq!(m.arriving(10.0).len(), 2);
        assert_eq!(m.arriving(a.arrival()), [(Pos::new(3, 0), ObjectId(3))]);
        assert!(m.arriving(b.arrival()).is_empty());
        // Frozen: all halfway at once. Without motion: nothing flies, everything is there.
        let mut m = Missiles::default();
        m.start(&[missile(6), missile(3)], 10.0, true, true, 1);
        for f in &m.flights {
            assert!((f.start - (10.0 - f.seconds * 0.5)).abs() < 1e-6);
        }
        let mut m = Missiles::default();
        m.start(&[missile(6)], 10.0, false, false, 1);
        assert_eq!(m.flights[0].at(10.0), None);
        assert!(m.arriving(10.0).is_empty());
    }

    #[test]
    fn glyphs_turn_along_the_flight_on_screen() {
        let size = Vec2::new(10.0, 20.0);
        assert_eq!(screen_turn(Vec2::X, size), 0.0);
        assert!((screen_turn(Vec2::Y, size) - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // A diagonal step is steeper on screen than 45°: cells are taller than wide.
        let diagonal = screen_turn(Vec2::new(1.0, -1.0).normalize(), size);
        assert!((diagonal - (-2.0f32).atan()).abs() < 1e-6);
    }
}
