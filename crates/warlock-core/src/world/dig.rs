//! Digging and breaking walls (spec §8 "Digging").
//!
//! Terrain that `breaks` (`data/terrain.toml`: a `hardness` and what it `becomes`) gives way to
//! digging and to explosions; a cell on a level's edge never does, whatever its terrain.
//!
//! **Digging** ([`crate::Action::Dig`]) takes the digging tool the player has ready (an object
//! with `dig` power, in the Tool slot) to the wall next to them, turn after turn like a rest
//! ([`World::keep_at`]): each turn is a `dig` action (its time, slowed by burden as every price
//! is), does [`dig_work`] against the wall's hardness, and is loud ([`crate::noise::DIG_NOISE`]).
//! It stops when the wall gives way, a creature comes into view, or a message is logged, and at
//! the latest after a day; the work done stays on the wall ([`LevelState::dug`], saved) for the
//! next dig. Creatures do not dig.
//!
//! **Explosions** ([`World::blast`]) break every wall in their burst whose hardness is below
//! their blast, at once and deterministically: no roll.
//!
//! A broken wall becomes its rubble, and solid rock next to it shows its face (a wall), as
//! generated levels turn rock that touches open ground into walls. Light, sight, and memory
//! follow the terrain as they do a door's: [`World::set_terrain`] drops the light masks the cell
//! could change, and perception sees and remembers the rubble like any terrain.

use super::{Ended, LevelState, Spent, World, article};
use crate::action::{Breach, Event, MAX_REST};
use crate::content::{ActionKind, LevelKind};
use crate::geom::{Dir, Pos};
use crate::noise::Act;
use crate::sight::LineOfSight;
use crate::stats::Stat;
use crate::time::Duration;

/// The work done on a wall is kept in twentieths of a point of hardness, so a dig's work,
/// power × (Strength + 10) / 20, needs no rounding.
pub const WORK_SCALE: u32 = 20;

/// The longest one dig goes on: a day, like the longest rest.
pub const MAX_DIG: Duration = MAX_REST;

/// The work one turn of digging does with a tool of `power` at Strength `strength` (the score),
/// in [`WORK_SCALE`]ths of a point of hardness: power × (Strength + 10). A pick-axe (power 10)
/// at Strength 10 does 200, a full 10 points: a wall of hardness 600 takes it 60 turns.
pub fn dig_work(power: u16, strength: u8) -> u32 {
    u32::from(power) * (u32::from(strength) + 10)
}

/// How many turns of digging a wall of `hardness` takes with a tool of `power` at Strength
/// `strength`.
pub fn digs_to_break(hardness: u16, power: u16, strength: u8) -> u32 {
    (u32::from(hardness) * WORK_SCALE).div_ceil(dig_work(power, strength).max(1))
}

impl LevelState {
    /// Whether `p` is on the level's edge, where walls never break.
    pub(crate) fn on_edge(&self, p: Pos) -> bool {
        p.x == 0 || p.y == 0 || p.x == self.width() - 1 || p.y == self.height() - 1
    }

    /// Whether the wall at `p` breaks: its terrain does and it is not on the edge.
    pub(crate) fn breaks_at(&self, content: &crate::content::Content, p: Pos) -> bool {
        !self.on_edge(p)
            && (self.terrain.get(p)).is_some_and(|t| content.terrain(*t).breaks.is_some())
    }
}

impl World {
    /// Starts digging toward `dir` (spec §8 "Digging"): refused, taking no time, without a
    /// digging tool ready, on the overworld, and toward anything that is no wall that breaks.
    pub(super) fn start_dig(&mut self, dir: Dir) -> Spent {
        let content = self.content.clone();
        let player = self.state.player();
        if player.dig_tool(&content).is_none() {
            self.say("You have no digging tool ready.");
            return Spent::Nothing;
        }
        let level = self.state.current();
        if level.kind == LevelKind::Overworld {
            self.say("You cannot dig here.");
            return Spent::Nothing;
        }
        let at = player.pos.step(dir);
        let def = match level.terrain.get(at) {
            Some(t) if !content.terrain(*t).walkable => content.terrain(*t),
            _ => {
                self.say("There is nothing to dig there.");
                return Spent::Nothing;
            }
        };
        if !level.breaks_at(&content, at) {
            self.say(format!("You cannot dig through the {}.", def.name));
            return Spent::Nothing;
        }
        let text = if level.dug.contains_key(&at) {
            format!("You go on digging at the {}.", def.name)
        } else {
            format!("You dig into the {}.", def.name)
        };
        self.say(text);
        Spent::Dig(at)
    }

    /// Digs into the wall at `at` turn after turn until it gives way, something interrupts, or a
    /// day has passed ([`World::keep_at`]), then says how it went.
    pub(super) fn dig(&mut self, at: Pos, events: &mut Vec<Event>) {
        let start = self.state.time;
        let wall = self.state.current().terrain.get(at).copied();
        let name = wall.map_or_else(String::new, |t| self.content.terrain(t).name.clone());
        let until = start + MAX_DIG;
        let standing = move |w: &World| w.state.current().terrain.get(at).copied() == wall;
        let ended = self.keep_at(
            events,
            |w| {
                standing(w)
                    && w.state.time < until
                    && w.state.player().dig_tool(&w.content).is_some()
            },
            |w, events| {
                let price = w.price_of(w.state.player, ActionKind::Dig);
                w.dig_turn(at, events);
                price
            },
        );
        let took = self.state.time.since(start).spoken();
        let text = match (standing(self), ended) {
            (_, Ended::Died) => return,
            (false, _) => format!("After {took}, the {name} crumbles."),
            (true, Ended::Came(id)) => format!(
                "{} comes into view; you stop digging after {took}.",
                self.creature_called(id)
            ),
            (true, Ended::Said | Ended::Done) => format!("You stop digging after {took}."),
        };
        self.say(text);
    }

    /// One turn of the player's digging into the wall at `at`: the noise, the work, and the wall
    /// giving way once the work done reaches its hardness.
    fn dig_turn(&mut self, at: Pos, events: &mut Vec<Event>) {
        let player = self.state.player();
        let Some((_, power)) = player.dig_tool(&self.content) else {
            return;
        };
        let work = dig_work(power, player.stats.score(Stat::Strength));
        self.make_noise(self.state.player, Act::Dig);
        let level = self.state.current();
        let Some(breaks) = (level.terrain.get(at)).and_then(|t| self.content.terrain(*t).breaks)
        else {
            return;
        };
        let level = self.state.current_mut();
        let done = level.dug.entry(at).or_insert(0);
        *done = done.saturating_add(work);
        if *done >= u32::from(breaks.hardness) * WORK_SCALE {
            self.break_wall(at, Breach::Dug, events);
        }
    }

    /// An explosion's effect on walls (spec §8 "Digging"): every wall in its burst (the cells in
    /// line of sight of `center` within `radius`, the walls bounding it included) whose hardness
    /// is below `blast` breaks, all at once, so a blast never reaches through a wall it breaks.
    /// Walls on the level's edge never break. Returns the cells that broke, in order.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn blast(
        &mut self,
        center: Pos,
        radius: i32,
        blast: u16,
        events: &mut Vec<Event>,
    ) -> Vec<Pos> {
        let content = self.content.clone();
        let level = self.state.current();
        let burst = LineOfSight::compute(center, radius, |p| level.is_opaque(&content, p));
        let mut broken = Vec::new();
        for y in center.y - radius..=center.y + radius {
            for x in center.x - radius..=center.x + radius {
                let p = Pos::new(x, y);
                let hardness = (level.terrain.get(p))
                    .and_then(|t| content.terrain(*t).breaks)
                    .map(|b| b.hardness);
                if burst.contains(p)
                    && level.breaks_at(&content, p)
                    && hardness.is_some_and(|h| h < blast)
                {
                    broken.push(p);
                }
            }
        }
        for &p in &broken {
            self.break_wall(p, Breach::Blast, events);
        }
        broken
    }

    /// Breaks the wall at `p` on the current level into what it becomes: the work done on it is
    /// gone, solid rock around it shows its face, and light follows ([`World::set_terrain`]).
    pub(super) fn break_wall(&mut self, p: Pos, cause: Breach, events: &mut Vec<Event>) {
        let level = self.state.current();
        let Some(breaks) = (level.terrain.get(p)).and_then(|t| self.content.terrain(*t).breaks)
        else {
            return;
        };
        self.state.current_mut().dug.remove(&p);
        self.set_terrain(p, breaks.becomes);
        for dir in Dir::ALL {
            let q = p.step(dir);
            let level = self.state.current();
            if let Some(face) = (level.terrain.get(q)).and_then(|t| self.content.terrain(*t).face) {
                self.set_terrain(q, face);
            }
        }
        events.push(Event::WallBroken { pos: p, cause });
    }

    /// What a creature is called at the start of a sentence: "A kobold", "An elf".
    pub(super) fn creature_called(&self, id: super::ActorId) -> String {
        let creature = self.state.actors[&id].creature;
        let name = &self.content.creature(creature).name;
        format!("{} {name}", article(name))
    }
}
