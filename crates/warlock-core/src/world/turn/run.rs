//! Running (#1325; spec §6 "Running"): [`crate::action::Action::Run`], step after step in a
//! direction until something worth stopping for. A long action through [`World::keep_at`], so it
//! stops for whatever a rest stops for (a hostile coming within [`World::stop_within`] in view,
//! an attack, a shot or a spell near, a message said, death); the step is its `act`, and whether
//! there is a next step, and which, is its `more`.
//!
//! The next step is a pure function of the level as the player knows it (the perception: what
//! they see and remember, never ground truth), worked out in core turn by turn after each step's
//! perception, so a run is one recorded action and replays exactly:
//! - **In the open** (the first step does not land in a corridor) the run holds its direction
//!   until the next cell cannot be stepped onto ([`steppable`]).
//! - **Along a corridor** (the first step lands in one, [`way_on`]) it follows the corridor
//!   through its bends, and stops where there is more than one way on (an intersection; a closed
//!   door counts as a way, so a side door stops it), none (a dead end), or a wide one (the
//!   corridor opens into a room).
//!
//! `world/mod.rs` has only the dispatch lines.

use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::burden::Burden;
use crate::geom::{Dir, Pos};
use crate::perception::{CellView, PerceptionView};
use crate::skills::Use;
use std::cell::Cell;

/// How the run goes on: holding its direction, or following a corridor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Open,
    Corridor,
}

/// Whether the player knows `p` as open space for finding the ways on: ground they can walk, or
/// a closed door (a way on, though the run does not open it).
fn open(view: &PerceptionView, p: Pos) -> bool {
    terrain_at(view, p).is_some_and(|t| {
        let def = view.content().terrain(t);
        def.walkable || def.is_closed_door()
    })
}

/// The terrain the player knows at `p`: inside the level, seen or remembered.
fn terrain_at(view: &PerceptionView, p: Pos) -> Option<crate::content::TerrainId> {
    if p.x < 0 || p.y < 0 || p.x >= view.width() || p.y >= view.height() {
        return None;
    }
    match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => Some(t),
        CellView::Unknown => None,
    }
}

/// Whether a run may step onto `p` as the player knows it: walkable ground (not deep water, a
/// closed door, or a shut chest), no creature seen there, and no trap known there.
pub(crate) fn steppable(view: &PerceptionView, p: Pos) -> bool {
    let Some(t) = terrain_at(view, p) else {
        return false;
    };
    let def = view.content().terrain(t);
    def.walkable
        && !def.swim
        && !def.is_closed_door()
        && !def.is_shut_chest()
        && view.actor_at(p).is_none()
        && view.trap(p).is_none()
}

/// The ways on from `at` for one who came from `from`: the open cells around `at` ([`open`]) in
/// groups of neighbours (consecutive around the ring, so each group is one way), less the group
/// that holds `from` (the way back). Each way's directions in ring order, clockwise from north.
pub(crate) fn ways_on(view: &PerceptionView, at: Pos, from: Pos) -> Vec<Vec<Dir>> {
    let ring = Dir::ALL.map(|d| open(view, at.step(d)));
    // Start the walk round the ring just after a closed cell, so no group is cut in two.
    let Some(start) = (0..8).find(|&i| !ring[i]) else {
        // Open all round: one group, which holds the way back.
        return Vec::new();
    };
    let mut ways: Vec<Vec<Dir>> = Vec::new();
    let mut current: Vec<Dir> = Vec::new();
    for k in 1..=8 {
        let i = (start + k) % 8;
        if ring[i] {
            current.push(Dir::ALL[i]);
        } else if !current.is_empty() {
            ways.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        ways.push(current);
    }
    ways.retain(|way| !way.iter().any(|&d| at.step(d) == from));
    ways
}

/// The one way on along a corridor from `at`, come from `from` ([`ways_on`]): exactly one way,
/// at most two cells wide (a corridor's bend or a diagonal stretch; wider is a room). `None` at
/// an intersection, a dead end, or a room.
pub(crate) fn way_on(view: &PerceptionView, at: Pos, from: Pos) -> Option<Vec<Dir>> {
    let mut ways = ways_on(view, at, from);
    match (ways.len(), ways.first().map(Vec::len)) {
        (1, Some(1 | 2)) => ways.pop(),
        _ => None,
    }
}

/// The next step along a corridor from `at`, come from `from` heading `heading`: the cell of the
/// one way on ([`way_on`]) that can be stepped onto, the one nearest the heading first (an
/// orthogonal step on a tie). `None` where the corridor gives no such step.
fn corridor_step(view: &PerceptionView, at: Pos, from: Pos, heading: Dir) -> Option<Dir> {
    let way = way_on(view, at, from)?;
    let turn = |d: Dir| {
        let (a, b) = (index(d), index(heading));
        let diff = (a + 8 - b) % 8;
        let diagonal = matches!(d, Dir::NE | Dir::SE | Dir::SW | Dir::NW);
        (diff.min(8 - diff), diagonal)
    };
    let mut way = way;
    way.sort_by_key(|&d| turn(d));
    way.into_iter().find(|&d| steppable(view, at.step(d)))
}

fn index(d: Dir) -> usize {
    Dir::ALL.iter().position(|&x| x == d).unwrap_or(0)
}

impl World {
    /// The player's run in `dir`: refused for what refuses a step (a status, a hold) by the
    /// dispatch; a first step that cannot be a run's ([`steppable`]: a wall, a door, a creature,
    /// a known trap, deep water, the unknown) is a plain step that way instead (it opens the
    /// door, strikes the creature, or says what is in the way).
    pub(super) fn start_running(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let to = self.player_pos().step(dir);
        if !steppable(&self.perception(), to) {
            return self.player_moves(dir, events);
        }
        Spent::Run(dir)
    }

    /// Runs in `dir` (see the module): step after step through [`World::keep_at`]. Ended by a
    /// creature coming close, it says so ("A kobold comes into view; you stop running."); ended
    /// by a message, that message says why; ended where the way gives out, nothing more is said.
    pub(super) fn run(&mut self, dir: Dir, events: &mut Vec<Event>) {
        let player = self.state.player;
        let (steps, stuck) = (Cell::new(0u32), Cell::new(false));
        let (mode, next) = (Cell::new(None::<Mode>), Cell::new(None::<Dir>));
        // Where the player stood before their last step, and the way it went.
        let (from, heading) = (Cell::new(self.player_pos()), Cell::new(dir));
        self.running = true;
        let ended = self.keep_at(
            events,
            |w| {
                if stuck.get() {
                    return false;
                }
                let view = w.perception();
                let at = w.player_pos();
                if steps.get() == 0 {
                    next.set(Some(dir));
                    return true;
                }
                // Within a stretch of turns the perception is the one before the last step: it
                // is looked at again once that step is perceived (`keep_at`).
                if view.viewer() != at {
                    return true;
                }
                let mode = match mode.get() {
                    Some(mode) => mode,
                    None => {
                        let found = match way_on(&view, at, from.get()) {
                            Some(_) => Mode::Corridor,
                            None => Mode::Open,
                        };
                        mode.set(Some(found));
                        found
                    }
                };
                let step = match mode {
                    Mode::Open => Some(dir).filter(|&d| steppable(&view, at.step(d))),
                    Mode::Corridor => corridor_step(&view, at, from.get(), heading.get()),
                };
                next.set(step);
                step.is_some()
            },
            |w, events| {
                let Some(dir) = next.get() else {
                    stuck.set(true);
                    return 0;
                };
                let load = w.load_of(player);
                let (burden, slowed) = (load.burden(), load.penalty() != Burden::Unburdened);
                from.set(w.player_pos());
                heading.set(dir);
                match w.player_moves(dir, events) {
                    Spent::Turn(price) => {
                        steps.set(steps.get() + 1);
                        w.ran_a_step(burden, slowed);
                        price
                    }
                    // Refused after all (a load too heavy there, someone who will not make way):
                    // what refused it was said, and the run is over.
                    _ => {
                        stuck.set(true);
                        0
                    }
                }
            },
        );
        self.running = false;
        if let Ended::Came(came) = ended {
            let text = format!("{}; you stop running.", self.came_line(came));
            self.say(text);
        }
    }

    /// What a step of a run trains, as a step taken alone does (`World::apply`): balance under a
    /// load that slows the player, Dexterity's loss while Overloaded, a hungry step.
    fn ran_a_step(&mut self, burden: Burden, slowed: bool) {
        if slowed && self.trains(Use::Burdened) {
            self.train(Use::Burdened);
        }
        if burden == Burden::Overloaded {
            self.stat_deed(crate::content::StatDeed::Overloaded);
        }
        self.train_hungry_step();
    }
}
