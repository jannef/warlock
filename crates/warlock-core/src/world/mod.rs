//! The simulation: world state and the turn pipeline (spec §8).
//!
//! Turn phases, in order: player action → creatures (ascending id) → environment → perception
//! (sight, light, memory) → perceived events are returned to the caller.

mod actor;
mod hash;
mod level;

pub use actor::{Actor, ActorId, ActorKind, Patrol};
pub(crate) use hash::fnv1a;
pub use level::LevelState;

use crate::action::{Action, Event, Outcome};
use crate::content::{Content, LevelTemplate, StairsDir, TerrainId};
use crate::geom::{Dir, Pos};
use crate::light::LightCache;
use crate::messages::MessageLog;
use crate::perception::{Perception, PerceptionView};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Why a world could not be created or restored.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorldError {
    #[error("unknown scenario `{0}`")]
    UnknownScenario(String),
    #[error("unknown level `{0}`")]
    UnknownLevel(String),
    #[error("level `{0}` has no player start")]
    NoPlayerStart(String),
    #[error("cannot generate level `{0}`: {1}")]
    Generation(String, String),
    #[error("saved state is inconsistent: {0}")]
    InconsistentState(String),
}

/// All persistent simulation state. This is what saves contain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub(crate) scenario: String,
    pub(crate) seed: u64,
    /// The gameplay random stream (spec §8), continued across saves. Map generation never draws
    /// from it: each generated level has its own stream derived from the seed and its id.
    pub(crate) rng: Rng,
    pub(crate) turn: u64,
    pub(crate) current_level: String,
    pub(crate) levels: BTreeMap<String, LevelState>,
    pub(crate) actors: BTreeMap<ActorId, Actor>,
    pub(crate) player: ActorId,
    pub(crate) next_actor_id: u32,
    pub(crate) messages: MessageLog,
    /// The player's vision profile (an id in `data/vision.toml`).
    pub(crate) vision: String,
    /// Whether the player's lantern (the scenario's `lantern`) is lit.
    pub(crate) lantern_lit: bool,
}

impl WorldState {
    pub(crate) fn player(&self) -> &Actor {
        &self.actors[&self.player]
    }

    pub(crate) fn current(&self) -> &LevelState {
        &self.levels[&self.current_level]
    }

    fn current_mut(&mut self) -> &mut LevelState {
        self.levels
            .get_mut(&self.current_level)
            .expect("current level is instantiated")
    }

    fn actor_at(&self, level: &str, p: Pos) -> Option<&Actor> {
        self.actors
            .values()
            .find(|a| a.level == level && a.pos == p)
    }

    fn check(&self) -> Result<(), WorldError> {
        let bad = |m: String| Err(WorldError::InconsistentState(m));
        if !self.levels.contains_key(&self.current_level) {
            return bad(format!("current level `{}` missing", self.current_level));
        }
        match self.actors.get(&self.player) {
            Some(p) if p.kind == ActorKind::Player && p.level == self.current_level => {}
            _ => return bad("player actor missing or misplaced".into()),
        }
        for actor in self.actors.values() {
            let Some(level) = self.levels.get(&actor.level) else {
                return bad(format!("actor {} is on missing level", actor.id));
            };
            if !level.terrain.in_bounds(actor.pos) || actor.id.0 >= self.next_actor_id {
                return bad(format!(
                    "actor {} is out of bounds or has a future id",
                    actor.id
                ));
            }
        }
        Ok(())
    }
}

/// A running game: state plus the content it refers to and the current perception.
#[derive(Clone, Debug)]
pub struct World {
    content: Arc<Content>,
    /// Boxed: the state (with its random stream) is large, and worlds are moved around whole.
    state: Box<WorldState>,
    /// Boxed: the snapshot is large and replaced wholesale every turn.
    perception: Box<Perception>,
    /// Cached light masks; derived, never saved.
    lights: LightCache,
}

/// How much time an action took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spent {
    /// Refused or failed: nothing changed but the message log.
    Nothing,
    /// Changed state without game time passing; perception must be recomputed.
    NoTime,
    /// A full turn: creatures act, then perception is recomputed.
    Turn,
}

impl From<bool> for Spent {
    fn from(took_turn: bool) -> Self {
        if took_turn {
            Spent::Turn
        } else {
            Spent::Nothing
        }
    }
}

impl World {
    /// Starts a new run of `scenario`. `seed` defaults to the scenario's seed.
    pub fn new(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
    ) -> Result<Self, WorldError> {
        let def = content
            .scenario(scenario)
            .ok_or_else(|| WorldError::UnknownScenario(scenario.to_string()))?;
        let seed = seed.unwrap_or(def.seed);
        let start = level_template(&content, seed, &def.start_level, None)?;
        let player_pos = start
            .player_start
            .ok_or_else(|| WorldError::NoPlayerStart(start.id.clone()))?;
        let player_creature = content
            .creature_id("player")
            .expect("content validation guarantees a player creature");

        let mut state = WorldState {
            scenario: def.id.clone(),
            seed,
            rng: Rng::gameplay(seed),
            turn: 0,
            current_level: start.id.clone(),
            levels: BTreeMap::new(),
            actors: BTreeMap::new(),
            player: ActorId(0),
            next_actor_id: 1,
            messages: MessageLog::default(),
            vision: def.vision.clone(),
            lantern_lit: false,
        };
        state.actors.insert(
            ActorId(0),
            Actor {
                id: ActorId(0),
                kind: ActorKind::Player,
                creature: player_creature,
                level: start.id.clone(),
                pos: player_pos,
                patrol: None,
            },
        );
        instantiate_level(&mut state, &start);
        drop(start); // It may borrow `content`, which `assemble` takes.
        state
            .messages
            .push(0, format!("Welcome to {}. Press ? for controls.", def.name));
        Ok(Self::assemble(content, state))
    }

    /// Restores a world from saved state.
    pub fn from_state(content: Arc<Content>, state: WorldState) -> Result<Self, WorldError> {
        state.check()?;
        if content.scenario(&state.scenario).is_none() {
            return Err(WorldError::UnknownScenario(state.scenario.clone()));
        }
        if content.vision(&state.vision).is_none() {
            return Err(WorldError::InconsistentState(format!(
                "unknown vision profile `{}`",
                state.vision
            )));
        }
        if let Some(level) = state.levels.values().find(|l| !l.ids_valid_for(&content)) {
            return Err(WorldError::InconsistentState(format!(
                "level `{}` uses terrain or lights this content does not have",
                level.id
            )));
        }
        Ok(Self::assemble(content, state))
    }

    fn assemble(content: Arc<Content>, mut state: WorldState) -> Self {
        let mut lights = LightCache::default();
        let perception = Box::new(Perception::compute(&state, &content, &mut lights));
        perception.remember(state.current_mut());
        Self {
            content,
            state: Box::new(state),
            perception,
            lights,
        }
    }

    pub fn content(&self) -> &Arc<Content> {
        &self.content
    }

    pub fn state(&self) -> &WorldState {
        &self.state
    }

    pub fn scenario(&self) -> &str {
        &self.state.scenario
    }

    pub fn seed(&self) -> u64 {
        self.state.seed
    }

    pub fn turn(&self) -> u64 {
        self.state.turn
    }

    pub fn player_id(&self) -> ActorId {
        self.state.player
    }

    pub fn player_pos(&self) -> Pos {
        self.state.player().pos
    }

    pub fn messages(&self) -> &MessageLog {
        &self.state.messages
    }

    /// The player's lantern: `None` if the scenario gives none, else whether it is lit.
    pub fn lantern(&self) -> Option<bool> {
        self.content
            .scenario(&self.state.scenario)
            .and_then(|s| s.lantern.as_ref())
            .map(|_| self.state.lantern_lit)
    }

    /// What the player perceives right now. Rendering and dumps must go through this.
    pub fn perception(&self) -> PerceptionView<'_> {
        PerceptionView {
            content: &self.content,
            level: self.state.current(),
            snapshot: &self.perception,
        }
    }

    /// Ground truth for the current level, **for developer tools only** (reveal mode, debug
    /// dumps). Gameplay presentation must use [`World::perception`].
    pub fn ground_truth(&self) -> (&LevelState, impl Iterator<Item = &Actor>) {
        let level = self.state.current();
        let actors = self
            .state
            .actors
            .values()
            .filter(move |a| a.level == level.id);
        (level, actors)
    }

    /// Applies one player action and runs the rest of the turn if it took time.
    pub fn apply(&mut self, action: &Action) -> Outcome {
        let mut events = Vec::new();
        let spent = self.player_acts(action, &mut events);
        if spent == Spent::Turn {
            self.state.turn += 1;
            self.creatures_act(&mut events);
        }
        if spent != Spent::Nothing {
            self.perceive(&mut events);
        }
        Outcome {
            took_turn: spent == Spent::Turn,
            events,
        }
    }

    /// Phase 4: recomputes perception, updates memory, and drops events the player did not see.
    fn perceive(&mut self, events: &mut Vec<Event>) {
        *self.perception = Perception::compute(&self.state, &self.content, &mut self.lights);
        let current = self.state.current_level.clone();
        if let Some(level) = self.state.levels.get_mut(&current) {
            self.perception.remember(level);
        }
        let player = self.state.player;
        events.retain(|e| match e {
            Event::ActorMoved { actor, from, to } => {
                *actor == player
                    || self.perception.sees_actors_at(*from)
                    || self.perception.sees_actors_at(*to)
            }
            Event::DoorOpened { pos } | Event::DoorClosed { pos } => {
                self.perception.is_visible(*pos)
            }
            Event::LevelChanged { .. } => true,
        });
    }

    /// Appends a system notice (for example "Game saved.") to the message log.
    pub fn notice(&mut self, text: impl Into<String>) {
        self.say(text);
    }

    fn say(&mut self, text: impl Into<String>) {
        let turn = self.state.turn;
        self.state.messages.push(turn, text);
    }

    /// Phase 1. Returns how much time the action took.
    fn player_acts(&mut self, action: &Action, events: &mut Vec<Event>) -> Spent {
        let here = self.player_pos();
        match action {
            Action::Move(dir) => self.player_moves(*dir, events).into(),
            Action::Wait => Spent::Turn,
            Action::Open(dir) => self.open_door(here.step(*dir), events).into(),
            Action::Close(dir) => self.close_door(here.step(*dir), events).into(),
            Action::Descend => self.take_stairs(StairsDir::Down, events).into(),
            Action::Ascend => self.take_stairs(StairsDir::Up, events).into(),
            Action::ToggleLantern => self.toggle_lantern().into(),
            Action::SetVision(id) => self.set_vision(id),
        }
    }

    fn toggle_lantern(&mut self) -> bool {
        if self.lantern().is_none() {
            self.say("You have no lantern.");
            return false;
        }
        self.state.lantern_lit = !self.state.lantern_lit;
        if self.state.lantern_lit {
            self.say("You light your lantern.");
        } else {
            self.say("You douse your lantern.");
        }
        true
    }

    fn set_vision(&mut self, id: &str) -> Spent {
        let Some(profile) = self.content.vision(id) else {
            self.say(format!("There is no vision profile `{id}`."));
            return Spent::Nothing;
        };
        let name = profile.name.clone();
        self.state.vision = id.to_string();
        self.say(format!("Vision: {name}."));
        Spent::NoTime
    }

    /// Changes terrain on the current level and invalidates light masks it may affect.
    fn set_terrain(&mut self, p: Pos, terrain: TerrainId) {
        let level = self.state.current_mut();
        level.terrain.set(p, terrain);
        let level = self.state.current();
        self.lights.terrain_changed(level, &self.content, p);
    }

    fn player_moves(&mut self, dir: Dir, events: &mut Vec<Event>) -> bool {
        let from = self.player_pos();
        let to = from.step(dir);
        let level = self.state.current();
        let Some(&terrain) = level.terrain.get(to) else {
            self.say("You can't go that way.");
            return false;
        };
        let def = self.content.terrain(terrain);
        if def.is_closed_door() {
            return self.open_door(to, events);
        }
        if !def.walkable {
            let name = def.name.clone();
            self.say(format!("The {name} blocks your way."));
            return false;
        }
        if let Some(other) = self.state.actor_at(&level.id, to) {
            let name = self.content.creature(other.creature).name.clone();
            self.say(format!("The {name} is in the way."));
            return false;
        }
        let player = self.state.player;
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.pos = to;
        }
        events.push(Event::ActorMoved {
            actor: player,
            from,
            to,
        });
        if let Some(stairs) = self.state.current().stairs_at(to) {
            let way = match stairs.dir {
                StairsDir::Down => "down",
                StairsDir::Up => "up",
            };
            self.say(format!("There is a staircase {way} here."));
        }
        true
    }

    fn open_door(&mut self, p: Pos, events: &mut Vec<Event>) -> bool {
        let terrain = self.state.current().terrain.get(p).copied();
        let def = terrain.map(|t| self.content.terrain(t));
        match def {
            Some(def) if def.is_closed_door() => {
                let opened = def.opens_to.expect("closed doors have opens_to");
                self.set_terrain(p, opened);
                events.push(Event::DoorOpened { pos: p });
                self.say("You open the door.");
                true
            }
            Some(def) if def.is_open_door() => {
                self.say("That door is already open.");
                false
            }
            _ => {
                self.say("There is nothing to open there.");
                false
            }
        }
    }

    fn close_door(&mut self, p: Pos, events: &mut Vec<Event>) -> bool {
        let terrain = self.state.current().terrain.get(p).copied();
        let def = terrain.map(|t| self.content.terrain(t));
        match def {
            Some(def) if def.is_open_door() => {
                let closed = def.closes_to.expect("open doors have closes_to");
                if self.state.actor_at(&self.state.current_level, p).is_some() {
                    self.say("Something is in the way.");
                    return false;
                }
                self.set_terrain(p, closed);
                events.push(Event::DoorClosed { pos: p });
                self.say("You close the door.");
                true
            }
            Some(def) if def.is_closed_door() => {
                self.say("That door is already closed.");
                false
            }
            _ => {
                self.say("There is nothing to close there.");
                false
            }
        }
    }

    fn take_stairs(&mut self, dir: StairsDir, events: &mut Vec<Event>) -> bool {
        let here = self.player_pos();
        let link = self
            .state
            .current()
            .stairs_at(here)
            .filter(|s| s.dir == dir)
            .cloned();
        let way = match dir {
            StairsDir::Down => "down",
            StairsDir::Up => "up",
        };
        let Some(link) = link else {
            self.say(format!("You see no way {way} here."));
            return false;
        };
        let Some(to) = link.to else {
            self.say("These stairs lead nowhere yet.");
            return false;
        };
        let from = self.state.current_level.clone();
        if let Err(err) = self.travel(&to) {
            self.say(format!("The stairs are blocked ({err})."));
            return false;
        }
        events.push(Event::LevelChanged { from, to });
        let name = self.state.current().name.clone();
        let verb = match dir {
            StairsDir::Down => "descend",
            StairsDir::Up => "climb",
        };
        self.say(format!("You {verb} the stairs to {name}."));
        true
    }

    /// Moves the player to level `to` (generating it on the first visit), arriving on the stairs
    /// that lead back, else the level's player start, else its first staircase.
    fn travel(&mut self, to: &str) -> Result<(), WorldError> {
        let from = self.state.current_level.clone();
        let mut start = None;
        if !self.state.levels.contains_key(to) {
            let template = level_template(&self.content, self.state.seed, to, Some(&from))?;
            start = template.player_start;
            instantiate_level(&mut self.state, &template);
        }
        let target = &self.state.levels[to];
        let arrival = target
            .stairs
            .iter()
            .find(|s| s.to.as_deref() == Some(from.as_str()))
            .map(|s| s.pos)
            .or(start)
            .or_else(|| self.content.level(to).and_then(|t| t.player_start))
            .or_else(|| target.stairs.first().map(|s| s.pos))
            .ok_or_else(|| WorldError::NoPlayerStart(to.to_string()))?;
        let arrival = self.free_cell_near(to, arrival);
        let player = self.state.player;
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.level = to.to_string();
            actor.pos = arrival;
        }
        self.state.current_level = to.to_string();
        Ok(())
    }

    /// `p` if no actor stands there, else the nearest walkable unoccupied cell (ring search).
    fn free_cell_near(&self, level_id: &str, p: Pos) -> Pos {
        let level = &self.state.levels[level_id];
        let free = |q: Pos| {
            level.is_walkable(&self.content, q) && self.state.actor_at(level_id, q).is_none()
        };
        if self.state.actor_at(level_id, p).is_none() {
            return p;
        }
        for r in 1..level.width().max(level.height()) {
            for dy in -r..=r {
                for dx in -r..=r {
                    let q = Pos::new(p.x + dx, p.y + dy);
                    if q.chebyshev(p) == r && free(q) {
                        return q;
                    }
                }
            }
        }
        p
    }

    /// Phase 2: creatures follow their patrol routes, waiting when blocked.
    fn creatures_act(&mut self, events: &mut Vec<Event>) {
        let current = self.state.current_level.clone();
        let movers: Vec<ActorId> = self
            .state
            .actors
            .values()
            .filter(|a| a.level == current && a.patrol.is_some())
            .map(|a| a.id)
            .collect();
        for id in movers {
            let actor = &self.state.actors[&id];
            let Some(patrol) = &actor.patrol else {
                continue;
            };
            let from = actor.pos;
            let to = from.step(patrol.route[patrol.step]);
            let blocked = !self.state.current().is_walkable(&self.content, to)
                || self.state.actor_at(&current, to).is_some();
            if blocked {
                continue;
            }
            if let Some(actor) = self.state.actors.get_mut(&id) {
                actor.pos = to;
                if let Some(patrol) = &mut actor.patrol {
                    patrol.step = (patrol.step + 1) % patrol.route.len();
                }
            }
            events.push(Event::ActorMoved {
                actor: id,
                from,
                to,
            });
        }
    }
}

/// The template of level `id`: a hand-made level, or a level of a dungeon branch generated from
/// the run seed (`entered_from` is where the up staircase of a branch's first level leads).
fn level_template<'a>(
    content: &'a Content,
    seed: u64,
    id: &str,
    entered_from: Option<&str>,
) -> Result<Cow<'a, LevelTemplate>, WorldError> {
    if let Some(template) = content.level(id) {
        return Ok(Cow::Borrowed(template));
    }
    if content.branch_level(id).is_none() {
        return Err(WorldError::UnknownLevel(id.to_string()));
    }
    crate::mapgen::generate(content, seed, id, entered_from)
        .map(Cow::Owned)
        .map_err(|e| WorldError::Generation(id.to_string(), e.to_string()))
}

/// Creates the runtime state for a level and spawns its creatures.
fn instantiate_level(state: &mut WorldState, template: &LevelTemplate) {
    let id = template.id.as_str();
    state
        .levels
        .insert(id.to_string(), LevelState::from_template(template));
    for spawn in &template.creatures {
        let actor_id = ActorId(state.next_actor_id);
        state.next_actor_id += 1;
        state.actors.insert(
            actor_id,
            Actor {
                id: actor_id,
                kind: ActorKind::Creature,
                creature: spawn.creature,
                level: id.to_string(),
                pos: spawn.pos,
                patrol: (!spawn.route.is_empty()).then(|| Patrol {
                    route: spawn.route.clone(),
                    step: 0,
                }),
            },
        );
    }
}

#[cfg(test)]
mod branch_tests;
#[cfg(test)]
mod light_tests;
#[cfg(test)]
mod tests;
