//! Motion and time (spec §4.6): creatures slide between cells and the map camera eases toward
//! its target.
//!
//! Motion starts from the turn's perceived events (`Outcome.events`) and is a pure function of
//! visual time. Any new input completes running motion instantly, so fast play never waits. With
//! reduced effects or frozen visual time (`--freeze-time`), all motion is instant.

use super::{camera_origin, map_area};
use crate::game::Game;
use crate::render::console::Console;
use crate::render::{ConsoleLayout, Effects, VisualTime};
use bevy::prelude::*;
use warlock_core::world::ActorId;
use warlock_core::{Event, Pos};

/// How long a slide (and a camera move) takes, in seconds of visual time. The camera uses the
/// same duration and curve, so a sliding player stays put on screen while the map scrolls.
pub const SLIDE_SECONDS: f32 = 0.08;

/// Ease-out cubic over `[0, 1]`: a fast start and a gentle stop.
pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

/// A timed move between two positions (in level cells).
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

    fn is_done(&self, time: f32) -> bool {
        self.from == self.to || time - self.start >= SLIDE_SECONDS
    }
}

fn cell(p: Pos) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// A creature sliding into the cell it now occupies.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Slide {
    actor: ActorId,
    from: Pos,
    to: Pos,
    start: f32,
}

impl Slide {
    fn ease(&self) -> Ease {
        Ease {
            from: cell(self.from),
            to: cell(self.to),
            start: self.start,
        }
    }
}

/// Running motion: creature slides and the camera.
#[derive(Resource, Debug, Default)]
pub struct Motion {
    slides: Vec<Slide>,
    camera: Option<Ease>,
    /// The turn whose events started the current slides.
    turn: Option<u64>,
}

impl Motion {
    /// Where a creature standing at `pos` is drawn at `time`, in level cells.
    pub fn actor_pos(&self, actor: ActorId, pos: Pos, time: f32) -> Vec2 {
        self.slides
            .iter()
            .find(|s| s.actor == actor && s.to == pos)
            .map_or(cell(pos), |s| s.ease().at(time))
    }

    /// The level position (in cells, fractional while easing) at the map viewport's top-left.
    pub fn camera(&self, time: f32) -> Vec2 {
        self.camera.map_or(Vec2::ZERO, |c| c.at(time))
    }

    /// Cells whose terrain glyph a creature covers: where creatures stand, and where running
    /// slides started.
    pub fn occupied(&self, standing: impl IntoIterator<Item = Pos>) -> Vec<Pos> {
        let mut cells: Vec<Pos> = standing.into_iter().collect();
        cells.extend(self.slides.iter().map(|s| s.from));
        cells.sort();
        cells.dedup();
        cells
    }

    /// Completes all running motion instantly.
    fn complete(&mut self) {
        self.slides.clear();
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
    let animate = !effects.reduced && !visual.frozen;
    let view = game.world.perception();
    let mut snap = !animate || layout.is_changed();
    if game.is_changed() {
        // New input (or anything else that changed the game) completes running motion.
        motion.complete();
        let turn = game.world.turn();
        if motion.turn.replace(turn) != Some(turn) {
            for event in &game.last_outcome.events {
                match *event {
                    Event::ActorMoved { actor, from, to } => {
                        // Only slide creatures that are now where the event says.
                        let seen = view.actors().iter().any(|a| a.id == actor && a.pos == to);
                        if animate && seen && from.chebyshev(to) == 1 {
                            motion.slides.push(Slide {
                                actor,
                                from,
                                to,
                                start: now,
                            });
                        }
                    }
                    Event::LevelChanged { .. } => {
                        motion.slides.clear();
                        snap = true;
                    }
                    _ => {}
                }
            }
        }
    }
    if motion.slides.iter().any(|s| s.ease().is_done(now)) {
        motion.slides.retain(|s| !s.ease().is_done(now));
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
            from,
            to,
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
    fn new_input_completes_motion() {
        let mut motion = Motion::default();
        motion.aim_camera(Vec2::new(0.0, 0.0), 0.0, true);
        motion.aim_camera(Vec2::new(3.0, 0.0), 1.0, false);
        motion.slides.push(Slide {
            actor: ActorId(0),
            from: Pos::new(0, 0),
            to: Pos::new(1, 0),
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
