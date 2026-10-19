//! Motion and time (spec §4.6): creatures slide between cells, lunge at what they attack in
//! melee, and the map camera eases toward its target.
//!
//! Motion starts from the turn's perceived events (`Outcome.events`) and is a pure function of
//! visual time. A creature quicker than the player can step several times in one turn; its steps
//! make one slide along the cells it passed. A melee attack is a short lunge toward the target
//! and back. Any new input completes running motion instantly, so fast play never waits. With the
//! `motion` effect off or frozen visual time (`--freeze-time`), all motion is instant (no lunges).

use super::{camera_origin, map_area};
use crate::game::Game;
use crate::render::console::Console;
use crate::render::{ConsoleLayout, Effects, VisualTime};
use bevy::prelude::*;
use warlock_core::world::ActorId;
use warlock_core::{AttackKind, Event, Pos};

/// How long a slide (and a camera move) takes, in seconds of visual time. The camera uses the
/// same duration and curve, so a sliding player stays put on screen while the map scrolls.
pub const SLIDE_SECONDS: f32 = 0.08;

/// How long a lunge takes, out and back, in seconds of visual time.
pub const LUNGE_SECONDS: f32 = 0.16;
/// How far toward its target a lunging creature reaches, in cells.
const LUNGE_REACH: f32 = 0.22;
/// The share of a lunge spent going out; the rest eases back.
const LUNGE_OUT: f32 = 0.35;

/// Ease-out cubic over `[0, 1]`: a fast start and a gentle stop.
pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

/// A timed camera move between two positions (in level cells).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ease {
    from: Vec2,
    to: Vec2,
    start: f32,
}

impl Ease {
    fn still(at: Vec2) -> Self {
        Self {
            from: at,
            to: at,
            start: 0.0,
        }
    }

    fn at(&self, time: f32) -> Vec2 {
        if self.from == self.to {
            return self.to;
        }
        self.from + (self.to - self.from) * ease_out((time - self.start) / SLIDE_SECONDS)
    }
}

fn cell(p: Pos) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// A creature sliding along the cells it stepped through this turn into the one it now occupies.
#[derive(Clone, Debug, PartialEq)]
struct Slide {
    actor: ActorId,
    /// Where it started, then each cell it stepped onto, one cell apart; at least two cells.
    path: Vec<Pos>,
    start: f32,
}

impl Slide {
    fn steps(&self) -> usize {
        self.path.len() - 1
    }

    fn to(&self) -> Pos {
        self.path[self.steps()]
    }

    /// How long the slide takes: [`SLIDE_SECONDS`] for one step, √steps times that for several,
    /// so a quick creature visibly covers more ground without its slide dragging on.
    fn seconds(&self) -> f32 {
        SLIDE_SECONDS * (self.steps() as f32).sqrt()
    }

    /// Where the creature is drawn at `time`: eased along the whole path, one unit per step.
    fn at(&self, time: f32) -> Vec2 {
        let along = ease_out((time - self.start) / self.seconds()) * self.steps() as f32;
        let i = (along.floor() as usize).min(self.steps() - 1);
        let (a, b) = (cell(self.path[i]), cell(self.path[i + 1]));
        a + (b - a) * (along - i as f32)
    }

    fn is_done(&self, time: f32) -> bool {
        time - self.start >= self.seconds()
    }
}

/// A creature lunging at what it attacks in melee: a short reach toward it and back.
#[derive(Clone, Debug, PartialEq)]
struct Lunge {
    actor: ActorId,
    /// Where it attacked from; the lunge shows only while it stands there.
    from: Pos,
    /// The way to its target, [`LUNGE_REACH`] long.
    reach: Vec2,
    start: f32,
}

impl Lunge {
    /// How far out it is at `time`: quickly out, then eased back.
    fn offset(&self, time: f32) -> Vec2 {
        let x = (time - self.start) / LUNGE_SECONDS;
        let out = match x {
            x if !(0.0..1.0).contains(&x) => 0.0,
            x if x < LUNGE_OUT => ease_out(x / LUNGE_OUT),
            x => 1.0 - ease_out((x - LUNGE_OUT) / (1.0 - LUNGE_OUT)),
        };
        self.reach * out
    }

    fn is_done(&self, time: f32) -> bool {
        time - self.start >= LUNGE_SECONDS
    }
}

/// The lunges a turn's `events` start at `now`: one for each melee attack (the last per actor).
fn lunges(events: &[Event], now: f32) -> Vec<Lunge> {
    let mut lunges: Vec<Lunge> = Vec::new();
    for event in events {
        if let &Event::Attacked {
            attacker,
            kind: AttackKind::Melee,
            from,
            to,
            ..
        } = event
        {
            lunges.retain(|l| l.actor != attacker);
            let way = Vec2::new((to.x - from.x) as f32, (to.y - from.y) as f32);
            lunges.push(Lunge {
                actor: attacker,
                from,
                reach: way.normalize_or_zero() * LUNGE_REACH,
                start: now,
            });
        }
    }
    lunges
}

/// The slides a turn's `events` start at `now`: for each actor, its last run of one-cell steps,
/// if it ends where the actor now stands (`standing`; `None` if not seen). Several steps in a row
/// make one slide; a jump or a level change starts over.
fn slides(events: &[Event], now: f32, standing: impl Fn(ActorId) -> Option<Pos>) -> Vec<Slide> {
    let mut paths: Vec<(ActorId, Vec<Pos>)> = Vec::new();
    for event in events {
        match *event {
            Event::ActorMoved { actor, from, to } => {
                let i = paths
                    .iter()
                    .position(|(a, _)| *a == actor)
                    .unwrap_or_else(|| {
                        paths.push((actor, Vec::new()));
                        paths.len() - 1
                    });
                let path = &mut paths[i].1;
                let step = from.chebyshev(to) == 1;
                if !step || path.last() != Some(&from) {
                    path.clear();
                    if step {
                        path.push(from);
                    }
                }
                path.push(to);
            }
            Event::LevelChanged { .. } => paths.clear(),
            _ => {}
        }
    }
    paths
        .into_iter()
        .filter(|(actor, path)| path.len() > 1 && standing(*actor) == path.last().copied())
        .map(|(actor, path)| Slide {
            actor,
            path,
            start: now,
        })
        .collect()
}

/// Running motion: creature slides and lunges, and the camera.
#[derive(Resource, Debug, Default)]
pub struct Motion {
    slides: Vec<Slide>,
    lunges: Vec<Lunge>,
    camera: Option<Ease>,
    /// The turn whose events started the current slides.
    turn: Option<u64>,
}

impl Motion {
    /// Where a creature standing at `pos` is drawn at `time`, in level cells.
    pub fn actor_pos(&self, actor: ActorId, pos: Pos, time: f32) -> Vec2 {
        let at = self
            .slides
            .iter()
            .find(|s| s.actor == actor && s.to() == pos)
            .map_or(cell(pos), |s| s.at(time));
        let lunge = self
            .lunges
            .iter()
            .find(|l| l.actor == actor && l.from == pos);
        at + lunge.map_or(Vec2::ZERO, |l| l.offset(time))
    }

    /// The level position (in cells, fractional while easing) at the map viewport's top-left.
    pub fn camera(&self, time: f32) -> Vec2 {
        self.camera.map_or(Vec2::ZERO, |c| c.at(time))
    }

    /// Cells whose terrain glyph a creature covers: where creatures stand, and the cells running
    /// slides started from or pass through.
    pub fn occupied(&self, standing: impl IntoIterator<Item = Pos>) -> Vec<Pos> {
        let mut cells: Vec<Pos> = standing.into_iter().collect();
        let passed = self.slides.iter().flat_map(|s| &s.path[..s.steps()]);
        cells.extend(passed.copied());
        cells.sort();
        cells.dedup();
        cells
    }

    /// Completes all running motion instantly.
    fn complete(&mut self) {
        self.slides.clear();
        self.lunges.clear();
        if let Some(camera) = &mut self.camera {
            *camera = Ease::still(camera.to);
        }
    }

    /// Points the camera at `target`, easing from where it is now unless `snap`.
    fn aim_camera(&mut self, target: Vec2, time: f32, snap: bool) {
        match &mut self.camera {
            Some(camera) if !snap => {
                if camera.to != target {
                    *camera = Ease {
                        from: camera.at(time),
                        to: target,
                        start: time,
                    };
                }
            }
            _ => self.camera = Some(Ease::still(target)),
        }
    }
}

/// Splits a camera position into the level cell at the viewport's top-left and the whole-pixel
/// offset of the view into that cell, so the grid keeps integer texel addressing.
pub fn camera_split(camera: Vec2, cell: Vec2) -> (IVec2, Vec2) {
    let floor = camera.floor();
    let mut base = floor.as_ivec2();
    let mut offset = ((camera - floor) * cell).round();
    for axis in 0..2 {
        if offset[axis] >= cell[axis] {
            offset[axis] = 0.0;
            base[axis] += 1;
        }
    }
    (base, offset)
}

/// Starts motion for a new turn, completes it on new input, and keeps the camera on target.
pub(super) fn track(
    game: Res<Game>,
    console: Res<Console>,
    layout: Res<ConsoleLayout>,
    visual: Res<VisualTime>,
    effects: Res<Effects>,
    mut motion: ResMut<Motion>,
) {
    let now = visual.seconds;
    let animate = effects.motion && !visual.frozen;
    let view = game.world.perception();
    let mut snap = !animate || layout.is_changed();
    if game.is_changed() {
        // New input (or anything else that changed the game) completes running motion.
        motion.complete();
        let turn = game.world.turn();
        if motion.turn.replace(turn) != Some(turn) {
            let events = &game.last_outcome.events;
            // A level change, or the player moved farther than a step (a teleport trap).
            let player = game.world.player_id();
            snap |= events.iter().any(|e| match *e {
                Event::LevelChanged { .. } => true,
                Event::ActorMoved { actor, from, to } => actor == player && from.chebyshev(to) > 1,
                _ => false,
            });
            if animate {
                // Only creatures seen where their last step took them slide.
                let standing = |id| view.actors().iter().find(|a| a.id == id).map(|a| a.pos);
                motion.slides = slides(events, now, standing);
                motion.lunges = lunges(events, now);
            }
        }
    }
    if motion.slides.iter().any(|s| s.is_done(now)) {
        motion.slides.retain(|s| !s.is_done(now));
    }
    if motion.lunges.iter().any(|l| l.is_done(now)) {
        motion.lunges.retain(|l| !l.is_done(now));
    }
    if let Some(area) = map_area(console.cols() as i32, console.rows() as i32) {
        let viewer = view.viewer();
        let target = Vec2::new(
            camera_origin(area.w, view.width(), viewer.x) as f32,
            camera_origin(area.h, view.height(), viewer.y) as f32,
        );
        if snap || motion.camera.map(|c| c.to) != Some(target) {
            motion.aim_camera(target, now, snap);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_out_starts_fast_and_lands_exactly() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert_eq!(ease_out(2.0), 1.0);
        assert_eq!(ease_out(-1.0), 0.0);
        assert!(ease_out(0.5) > 0.5, "more than half way at half time");
        let mut last = 0.0;
        for i in 1..=10 {
            let v = ease_out(i as f32 / 10.0);
            assert!(v > last);
            last = v;
        }
    }

    #[test]
    fn slides_move_from_the_old_cell_to_the_new_one() {
        let mut motion = Motion::default();
        let actor = ActorId(3);
        let (from, to) = (Pos::new(4, 4), Pos::new(5, 4));
        motion.slides.push(Slide {
            actor,
            path: vec![from, to],
            start: 1.0,
        });
        assert_eq!(motion.actor_pos(actor, to, 1.0), Vec2::new(4.0, 4.0));
        let mid = motion.actor_pos(actor, to, 1.0 + SLIDE_SECONDS / 2.0);
        assert!(mid.x > 4.5 && mid.x < 5.0 && mid.y == 4.0);
        assert_eq!(
            motion.actor_pos(actor, to, 1.0 + SLIDE_SECONDS),
            Vec2::new(5.0, 4.0)
        );
        // Another creature, or a stale slide whose target no longer matches, is not moved.
        assert_eq!(motion.actor_pos(ActorId(9), to, 1.0), Vec2::new(5.0, 4.0));
        assert_eq!(motion.actor_pos(actor, from, 1.0), Vec2::new(4.0, 4.0));
        assert_eq!(
            motion.occupied([to, Pos::new(1, 1)]),
            vec![Pos::new(1, 1), from, to]
        );
    }

    #[test]
    fn several_steps_in_a_turn_make_one_slide_along_the_path() {
        let moved = |actor, from: (i32, i32), to: (i32, i32)| Event::ActorMoved {
            actor: ActorId(actor),
            from: Pos::new(from.0, from.1),
            to: Pos::new(to.0, to.1),
        };
        let events = [
            moved(0, (1, 1), (2, 1)),
            moved(3, (5, 5), (6, 5)),
            moved(3, (6, 5), (7, 6)),
            // It moved on unseen, so it does not stand where these took it: no slide.
            moved(4, (9, 9), (9, 8)),
            moved(4, (9, 8), (9, 7)),
            // A jump starts over.
            moved(5, (1, 9), (8, 9)),
            moved(5, (8, 9), (7, 9)),
        ];
        let standing = |id: ActorId| match id.0 {
            0 => Some(Pos::new(2, 1)),
            3 => Some(Pos::new(7, 6)),
            4 => Some(Pos::new(9, 6)),
            5 => Some(Pos::new(7, 9)),
            _ => None,
        };
        let motion = Motion {
            slides: slides(&events, 1.0, standing),
            ..Motion::default()
        };
        let paths: Vec<_> = motion
            .slides
            .iter()
            .map(|s| (s.actor.0, s.path.len()))
            .collect();
        assert_eq!(paths, [(0, 2), (3, 3), (5, 2)]);

        let quick = ActorId(3);
        let to = Pos::new(7, 6);
        assert_eq!(motion.actor_pos(quick, to, 1.0), Vec2::new(5.0, 5.0));
        // Half way through the slide it is past the middle of its path (easing out), on its
        // second step, between (6, 5) and (7, 6).
        let mid = motion.actor_pos(quick, to, 1.0 + SLIDE_SECONDS / 2.0);
        assert!(
            mid.x > 6.0 && mid.x < 7.0 && mid.y > 5.0 && mid.y < 6.0,
            "{mid}"
        );
        let done = 1.0 + SLIDE_SECONDS * 2f32.sqrt();
        assert!(!motion.slides[1].is_done(done - 0.001) && motion.slides[1].is_done(done));
        assert_eq!(motion.actor_pos(quick, to, done), Vec2::new(7.0, 6.0));
        // The cells it started from and passed through are covered while it slides.
        let covered = motion.occupied([]);
        assert!(covered.contains(&Pos::new(5, 5)) && covered.contains(&Pos::new(6, 5)));

        // After a level change only later steps slide.
        let changed = Event::LevelChanged {
            from: "a".into(),
            to: "b".into(),
        };
        let events = [moved(3, (5, 5), (6, 5)), changed, moved(3, (6, 5), (7, 6))];
        let after = slides(&events, 0.0, standing);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].path, [Pos::new(6, 5), Pos::new(7, 6)]);
    }

    #[test]
    fn melee_attacks_lunge_toward_the_target_and_back() {
        let attack = |attacker, kind, from: (i32, i32), to: (i32, i32)| Event::Attacked {
            attacker: ActorId(attacker),
            target: ActorId(9),
            kind,
            from: Pos::new(from.0, from.1),
            to: Pos::new(to.0, to.1),
            strike: None,
            with: None,
        };
        let events = [
            attack(3, AttackKind::Melee, (4, 4), (5, 3)),
            // Shots and spells do not lunge.
            attack(4, AttackKind::Ranged, (1, 1), (6, 1)),
            attack(5, AttackKind::Spell, (2, 2), (6, 2)),
        ];
        let mut motion = Motion {
            lunges: lunges(&events, 1.0),
            ..Motion::default()
        };
        assert_eq!(motion.lunges.len(), 1);
        let at = Pos::new(4, 4);
        let actor = ActorId(3);
        assert_eq!(motion.actor_pos(actor, at, 1.0), Vec2::new(4.0, 4.0));
        // Out toward the north-east, a fraction of a cell, then back.
        let out = motion.actor_pos(actor, at, 1.0 + LUNGE_SECONDS * LUNGE_OUT);
        assert!(out.x > 4.1 && out.y < 3.9, "{out}");
        assert!((out - Vec2::new(4.0, 4.0)).length() <= LUNGE_REACH + 1e-4);
        let back = motion.actor_pos(actor, at, 1.0 + LUNGE_SECONDS * 0.8);
        assert!(back.x < out.x && back.x > 4.0, "{back}");
        assert_eq!(
            motion.actor_pos(actor, at, 1.0 + LUNGE_SECONDS),
            Vec2::new(4.0, 4.0)
        );
        // Only where it attacked from (if it moved on since, no lunge), and not the others.
        assert_eq!(
            motion.actor_pos(actor, Pos::new(5, 5), 1.05),
            Vec2::new(5.0, 5.0)
        );
        assert_eq!(
            motion.actor_pos(ActorId(4), Pos::new(1, 1), 1.05),
            Vec2::new(1.0, 1.0)
        );
        motion.complete();
        assert!(motion.lunges.is_empty());
    }

    #[test]
    fn new_input_completes_motion() {
        let mut motion = Motion::default();
        motion.aim_camera(Vec2::new(0.0, 0.0), 0.0, true);
        motion.aim_camera(Vec2::new(3.0, 0.0), 1.0, false);
        motion.slides.push(Slide {
            actor: ActorId(0),
            path: vec![Pos::new(0, 0), Pos::new(1, 0)],
            start: 1.0,
        });
        assert_eq!(motion.camera(1.0), Vec2::ZERO);
        motion.complete();
        assert!(motion.slides.is_empty());
        assert_eq!(motion.camera(1.0), Vec2::new(3.0, 0.0));
    }

    #[test]
    fn camera_eases_from_where_it_is() {
        let mut motion = Motion::default();
        motion.aim_camera(Vec2::new(10.0, 5.0), 0.0, false);
        assert_eq!(
            motion.camera(0.0),
            Vec2::new(10.0, 5.0),
            "the first aim snaps"
        );
        motion.aim_camera(Vec2::new(11.0, 5.0), 2.0, false);
        let mid = motion.camera(2.0 + SLIDE_SECONDS / 2.0);
        assert!(mid.x > 10.0 && mid.x < 11.0);
        // Re-aiming mid-move continues from the current position.
        motion.aim_camera(Vec2::new(12.0, 5.0), 2.0 + SLIDE_SECONDS / 2.0, false);
        assert_eq!(motion.camera(2.0 + SLIDE_SECONDS / 2.0), mid);
        assert_eq!(motion.camera(5.0), Vec2::new(12.0, 5.0));
        motion.aim_camera(Vec2::new(0.0, 0.0), 5.0, true);
        assert_eq!(motion.camera(5.0), Vec2::ZERO);
    }

    #[test]
    fn camera_splits_into_cell_and_whole_pixels() {
        let cell = Vec2::new(11.0, 22.0);
        assert_eq!(
            camera_split(Vec2::new(3.0, -2.0), cell),
            (IVec2::new(3, -2), Vec2::ZERO)
        );
        assert_eq!(
            camera_split(Vec2::new(3.5, 1.25), cell),
            (IVec2::new(3, 1), Vec2::new(6.0, 6.0))
        );
        // Rounding up to a whole cell carries into the next cell.
        assert_eq!(
            camera_split(Vec2::new(3.99, 0.0), cell),
            (IVec2::new(4, 0), Vec2::ZERO)
        );
        assert_eq!(
            camera_split(Vec2::new(-0.5, 0.0), cell),
            (IVec2::new(-1, 0), Vec2::new(6.0, 0.0))
        );
    }
}
