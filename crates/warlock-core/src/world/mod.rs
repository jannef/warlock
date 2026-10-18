//! The simulation: world state and the turn pipeline (spec §8).
//!
//! Turn phases, in order: player action → the other actors on the level act in time order until
//! the player may act again, the clock advancing to the moment of each action (the energy
//! scheduler, `schedule.rs`; [`World::creature_turn`] is one creature's action) → environment →
//! perception (sight, light, memory) → perceived events are returned to the caller.
//!
//! Time (spec §8): every action that takes time has a duration: the level's turn
//! ([`LevelState::turn`], 6 s on local levels, 10 min on the overworld), or for a step the level's
//! [`LevelState::step_time`] of the destination terrain. An actor's speed decides how soon it may
//! act again: at normal speed (the player's) after exactly that long, at twice the speed after
//! half of it. Resting ([`Action::Rest`]) passes many turns in one action.
//!
//! Hearing (spec §5 "Hearing", [`crate::noise`]): steps and doors make noise as they happen, and
//! every other actor in earshot on the level hears it at once ([`Actor::heard`], until it next
//! acts). Perception turns what the player heard from unseen sources into perceived sounds, and
//! news among them into messages ("You hear footsteps to the north-east."), which stop a rest.
//!
//! The overworld (spec §3): stairs down on it are locations leading into other levels; `>` on
//! any other cell enters that cell's local area (`local:x,y`, generated on the first visit) on
//! the side the player walked onto the cell from ([`WorldState`]'s last overworld step), and
//! walking off a local area's edge leads back to its overworld cell (the level state's edge exit).
//!
//! Objects (spec §6): piles lie on cells of a level ([`LevelState`]), actors carry inventories.
//! Picking up and dropping take the level's turn; stepping onto a pile says what lies there. An
//! actor's weight is its creature's plus what it carries ([`Actor::weight`]).

mod actor;
mod hash;
mod level;
mod schedule;

pub use actor::{Actor, ActorId, ActorKind, Patrol};
pub(crate) use hash::fnv1a;
pub use level::LevelState;

use crate::action::{Action, Event, Letters, MAX_REST, Outcome, Rest, letter};
use crate::content::{
    Content, EdgeExit, LevelKind, LevelTemplate, StairsDir, TerrainId, local_area_id,
};
use crate::geom::{Dir, Pos};
use crate::light::LightCache;
use crate::mapgen::local::arrival;
use crate::messages::MessageLog;
use crate::noise::{self, Act, Recent};
use crate::object::{self, Stack, Weight};
use crate::perception::{Perception, PerceptionView};
use crate::rng::Rng;
use crate::time::{self, Duration, GameTime};
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
    /// The game clock.
    pub(crate) time: GameTime,
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
    /// The step that brought the player onto their overworld cell, if they walked there: `>`
    /// then arrives on the side of the local area they came from. Cleared on any level change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) overworld_step: Option<Dir>,
    /// Sounds the player perceived lately: those like them are no news ([`noise::news`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) recent_sounds: Vec<Recent>,
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
            Some(p)
                if p.kind == ActorKind::Player
                    && p.level == self.current_level
                    && p.energy >= 0 => {}
            _ => return bad("player actor missing, misplaced, or not ready to act".into()),
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

    /// The player's inventory.
    fn inventory_mut(&mut self) -> &mut Vec<Stack> {
        let player = self.player;
        &mut self
            .actors
            .get_mut(&player)
            .expect("the player actor exists")
            .inventory
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
    /// Messages logged since this world was created or loaded (not saved): resting stops when it
    /// changes.
    said: u64,
}

/// How much time an action took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spent {
    /// Refused or failed: nothing changed but the message log.
    Nothing,
    /// Changed state without game time passing; perception must be recomputed.
    NoTime,
    /// A turn: the player's action took this long; the others act until the player may act again
    /// (the clock advances), then perception is recomputed.
    Turn(Duration),
    /// A rest until this moment (see [`World::rest`]).
    Rest(GameTime, Rest),
    /// Dev: the clock jumps ahead; nobody acts, then perception is recomputed.
    Skip(Duration),
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
            time: def.start,
            current_level: start.id.clone(),
            levels: BTreeMap::new(),
            actors: BTreeMap::new(),
            player: ActorId(0),
            next_actor_id: 1,
            messages: MessageLog::default(),
            vision: def.vision.clone(),
            lantern_lit: false,
            overworld_step: None,
            recent_sounds: Vec::new(),
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
                energy: 0,
                heard: Vec::new(),
                inventory: Vec::new(),
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
                "level `{}` uses terrain, lights, or objects this content does not have",
                level.id
            )));
        }
        let creatures = content.creature_defs().len();
        if let Some(actor) = state.actors.values().find(|a| {
            usize::from(a.creature.0) >= creatures
                || !a.inventory.iter().all(|s| s.is_valid_for(&content))
                || a.inventory.len() > object::INVENTORY_SLOTS
        }) {
            return Err(WorldError::InconsistentState(format!(
                "actor {} is a creature or carries objects this content does not have",
                actor.id
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
            said: 0,
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

    /// The game clock. Month names are in [`Content::calendar`].
    pub fn time(&self) -> GameTime {
        self.state.time
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
            player: self.state.player(),
            snapshot: &self.perception,
        }
    }

    /// The weight of actor `id`, on any level: its creature's base weight plus everything it
    /// carries. For the rules and dev tools; the player is shown only their own
    /// ([`PerceptionView::own_weight`]).
    pub fn actor_weight(&self, id: ActorId) -> Option<Weight> {
        self.state.actors.get(&id).map(|a| a.weight(&self.content))
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
        let (turn, time) = (self.state.turn, self.state.time);
        let mut events = Vec::new();
        match self.player_acts(action, &mut events) {
            Spent::Nothing => {}
            Spent::NoTime => self.perceive(&mut events),
            Spent::Turn(duration) => {
                self.run_turn(duration, &mut events);
                self.perceive(&mut events);
            }
            Spent::Rest(until, goal) => self.rest(until, goal, &mut events),
            Spent::Skip(duration) => {
                // Nobody acts or gains energy.
                self.state.time += duration;
                self.player_mut().heard.clear();
                self.perceive(&mut events);
            }
        }
        Outcome {
            took_turn: self.state.turn != turn,
            elapsed: self.state.time.since(time),
            events,
        }
    }

    /// Rests until `until`: turn after turn through the normal pipeline (the player waits, the
    /// others act, perception), stopping after a turn in which a creature came into view or a
    /// message was logged. The last turn is cut short to end exactly at `until`. Turns that end
    /// before any creature could come into view ([`World::out_of_view_for`]) pass without
    /// perception, which cannot change the outcome; only the last turn's events are kept.
    fn rest(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        let start = self.state.time;
        let turn = self.state.current().turn.max(Duration::seconds(1));
        let said = self.said;
        let mut in_view = self.creatures_in_view();
        let mut came = None;
        while self.state.time < until && came.is_none() && self.said == said {
            let unseen_until = self.state.time + self.out_of_view_for();
            loop {
                events.clear();
                self.run_turn(turn.min(until.since(self.state.time)), events);
                let done = self.state.time >= until || self.said != said;
                if done || self.state.time > unseen_until {
                    break;
                }
            }
            self.perceive(events);
            let now_in_view = self.creatures_in_view();
            came = now_in_view.iter().find(|a| !in_view.contains(a)).copied();
            in_view = now_in_view;
        }
        let rested = self.state.time.since(start).spoken();
        let text = match (came, self.said != said, goal) {
            (Some(id), _, _) => {
                let creature = self.state.actors[&id].creature;
                let name = &self.content.creature(creature).name;
                format!(
                    "{} {name} comes into view; you stop resting after {rested}.",
                    article(name)
                )
            }
            (None, true, _) => format!("You stop resting after {rested}."),
            (None, false, Rest::For(_)) => format!("You rest for {rested}."),
            (None, false, Rest::Dawn) => "You rest until dawn.".into(),
            (None, false, Rest::Dusk) => "You rest until dusk.".into(),
        };
        self.say(text);
    }

    /// Creatures (not the player) the player sees.
    fn creatures_in_view(&self) -> Vec<ActorId> {
        self.perception()
            .actors()
            .iter()
            .filter(|a| a.kind == ActorKind::Creature)
            .map(|a| a.id)
            .collect()
    }

    /// Phase 4: recomputes perception, updates memory, says what news the player heard, and
    /// drops events the player did not see.
    fn perceive(&mut self, events: &mut Vec<Event>) {
        *self.perception = Perception::compute(&self.state, &self.content, &mut self.lights);
        let current = self.state.current_level.clone();
        if let Some(level) = self.state.levels.get_mut(&current) {
            self.perception.remember(level);
        }
        self.hear();
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

    /// A message for each sound the player perceives that is news ([`noise::news`]): a patrol
    /// heard turn after turn is said once, not every turn. Messages stop a rest.
    fn hear(&mut self) {
        let turn = self.state.turn;
        let news = noise::news(
            &mut self.state.recent_sounds,
            self.perception.sounds(),
            turn,
        );
        let viewer = self.player_pos();
        let mut said: Vec<String> = Vec::new();
        for sound in news {
            let text = sound.message(viewer);
            if !said.contains(&text) {
                self.say(text.clone());
                said.push(text);
            }
        }
    }

    /// The noise of `act` by `actor` on the current level, heard by those it reaches.
    fn make_noise(&mut self, actor: ActorId, act: Act) {
        let made = self
            .state
            .actors
            .get(&actor)
            .and_then(|a| noise::noise(&self.content, self.state.current(), a, act));
        if let Some(made) = made {
            noise::make(&mut self.state, &self.content, made);
        }
    }

    fn player_mut(&mut self) -> &mut Actor {
        let player = self.state.player;
        self.state
            .actors
            .get_mut(&player)
            .expect("the player actor exists")
    }

    /// Appends a system notice (for example "Game saved.") to the message log.
    pub fn notice(&mut self, text: impl Into<String>) {
        self.say(text);
    }

    fn say(&mut self, text: impl Into<String>) {
        let turn = self.state.turn;
        self.state.messages.push(turn, text);
        self.said += 1;
    }

    /// Phase 1. Returns how much time the action took.
    fn player_acts(&mut self, action: &Action, events: &mut Vec<Event>) -> Spent {
        let here = self.player_pos();
        // One turn of the level the action starts on (taking stairs ends on another level).
        let turn = Spent::Turn(self.state.current().turn);
        let took = move |done: bool| if done { turn } else { Spent::Nothing };
        match action {
            Action::Move(dir) => self.player_moves(*dir, events),
            Action::Wait => turn,
            Action::Open(dir) => took(self.open_door(here.step(*dir), events)),
            Action::Close(dir) => took(self.close_door(here.step(*dir), events)),
            Action::Descend => took(self.take_stairs(StairsDir::Down, events)),
            Action::Ascend => took(self.take_stairs(StairsDir::Up, events)),
            Action::ToggleLantern => took(self.toggle_lantern()),
            Action::PickUp(letters) => took(self.pick_up(*letters)),
            Action::Drop(letters) => took(self.drop_objects(*letters)),
            Action::Rest(goal) => self.start_rest(*goal),
            Action::SetVision(id) => self.set_vision(id),
            Action::SkipTime(duration) => {
                self.say(format!("Time skips ahead {} (dev).", duration.spoken()));
                Spent::Skip(*duration)
            }
        }
    }

    /// When a rest toward `goal` ends: at most [`MAX_REST`] from now.
    fn start_rest(&mut self, goal: Rest) -> Spent {
        let now = self.state.time;
        let until = match goal {
            Rest::For(duration) => now + duration.min(MAX_REST),
            Rest::Dawn => now.next_at(time::DAWN),
            Rest::Dusk => now.next_at(time::DUSK),
        };
        if until == now {
            self.say("You do not rest.");
            return Spent::Nothing;
        }
        Spent::Rest(until, goal)
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

    /// Picks up the stacks `letters` names in the pile on the player's cell (everything there
    /// with no letters), as many as the inventory has letters for. Returns whether anything was
    /// taken.
    fn pick_up(&mut self, letters: Letters) -> bool {
        let here = self.player_pos();
        let pile = self.state.current().objects_at(here).to_vec();
        if pile.is_empty() {
            self.say("There is nothing here to pick up.");
            return false;
        }
        let Some(chosen) = self.chosen(letters, pile.len(), |c| format!("There is no {c} here."))
        else {
            return false;
        };
        let content = self.content.clone();
        let inventory = self.state.inventory_mut();
        let mut taken = Letters::default();
        for i in chosen.iter() {
            if object::add_to_inventory(inventory, pile[i], &content) {
                taken.insert(i);
            }
        }
        if taken.is_empty() {
            self.say("You have no room for more.");
            return false;
        }
        // Everything was asked for, but the pile has more stacks than there are letters.
        let more = letters.is_empty() && pile.len() > taken.len();
        let full = taken != chosen || (more && inventory.len() >= object::INVENTORY_SLOTS);
        let (got, left) = split(&pile, taken);
        self.state.current_mut().set_pile(here, left);
        let mut text = format!("You pick up {}.", object::phrase(&got, &content));
        if full {
            text.push_str(" You have no room for more.");
        }
        self.say(text);
        true
    }

    /// Drops the inventory stacks `letters` names (everything with no letters) on top of the pile
    /// on the player's cell. Returns whether anything was dropped.
    fn drop_objects(&mut self, letters: Letters) -> bool {
        let inventory = self.state.player().inventory.clone();
        if inventory.is_empty() {
            self.say("You are not carrying anything.");
            return false;
        }
        let Some(chosen) = self.chosen(letters, inventory.len(), |c| format!("You carry no {c}."))
        else {
            return false;
        };
        let (dropped, kept) = split(&inventory, chosen);
        *self.state.inventory_mut() = kept;
        let here = self.player_pos();
        let content = self.content.clone();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        object::add_to_pile(&mut pile, &dropped, &content);
        level.set_pile(here, pile);
        self.say(format!("You drop {}.", object::phrase(&dropped, &content)));
        true
    }

    /// The positions `letters` names in a list of `len` stacks, every position (up to `z`) for
    /// no letters. A letter past the end is refused with the message `missing` makes of it.
    fn chosen(
        &mut self,
        letters: Letters,
        len: usize,
        missing: impl Fn(char) -> String,
    ) -> Option<Letters> {
        if letters.is_empty() {
            return Some(Letters::of(0..len));
        }
        match letters.iter().find(|i| *i >= len).and_then(letter) {
            Some(c) => {
                self.say(missing(c));
                None
            }
            None => Some(letters),
        }
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

    fn player_moves(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let from = self.player_pos();
        let to = from.step(dir);
        let level = self.state.current();
        let Some(&terrain) = level.terrain.get(to) else {
            if let Some(exit) = level.exit.clone() {
                return self.leave_by_edge(exit, events);
            }
            self.say("You can't go that way.");
            return Spent::Nothing;
        };
        let def = self.content.terrain(terrain);
        if def.is_closed_door() {
            let turn = level.turn;
            let opened = self.open_door(to, events);
            return if opened {
                Spent::Turn(turn)
            } else {
                Spent::Nothing
            };
        }
        if !def.walkable {
            let name = def.name.clone();
            self.say(format!("The {name} blocks your way."));
            return Spent::Nothing;
        }
        if let Some(other) = self.state.actor_at(&level.id, to) {
            let name = self.content.creature(other.creature).name.clone();
            self.say(format!("The {name} is in the way."));
            return Spent::Nothing;
        }
        let step = level.step_time(def);
        self.state.overworld_step = (level.kind == LevelKind::Overworld).then_some(dir);
        let player = self.state.player;
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.pos = to;
        }
        events.push(Event::ActorMoved {
            actor: player,
            from,
            to,
        });
        self.make_noise(player, Act::Step(to));
        let level = self.state.current();
        if let Some(stairs) = level.stairs_at(to) {
            let text = match (level.kind, stairs.dir, &stairs.to) {
                (LevelKind::Overworld, StairsDir::Down, Some(target)) => {
                    format!("The way into {} is here.", self.place_name(target))
                }
                (_, StairsDir::Down, _) => "There is a staircase down here.".into(),
                (_, StairsDir::Up, _) => "There is a staircase up here.".into(),
            };
            self.say(text);
        }
        let text = match self.state.current().objects_at(to) {
            [] => None,
            [one] => Some(format!("You see here {}.", one.describe(&self.content))),
            _ => Some("There are several objects here.".to_string()),
        };
        if let Some(text) = text {
            self.say(text);
        }
        Spent::Turn(step)
    }

    /// What a level is called in messages; a dungeon branch goes by the branch's name.
    fn place_name(&self, id: &str) -> String {
        if let Some((branch, _)) = self.content.branch_level(id) {
            return branch.name.clone();
        }
        if let Some(level) = self.state.levels.get(id) {
            return level.name.clone();
        }
        self.content
            .level(id)
            .map_or_else(|| id.to_string(), |t| t.name.clone())
    }

    /// Walks off the edge of a local area, back to its overworld cell. Takes the level's turn.
    fn leave_by_edge(&mut self, exit: EdgeExit, events: &mut Vec<Event>) -> Spent {
        let turn = self.state.current().turn;
        let from = self.state.current_level.clone();
        if let Err(err) = self.travel(&exit.to, Some(exit.pos)) {
            self.say(format!("You cannot leave this way ({err})."));
            return Spent::Nothing;
        }
        events.push(Event::LevelChanged { from, to: exit.to });
        let name = self.state.current().name.clone();
        self.say(format!("You return to {name}."));
        Spent::Turn(turn)
    }

    /// `>` on an overworld cell that is not a location: enters the cell's local area, arriving
    /// on the side the player walked onto the cell from (stepping east, on its west side), or at
    /// its start (the middle) if they did not walk there.
    fn enter_local_area(&mut self, events: &mut Vec<Event>) -> bool {
        let id = local_area_id(self.player_pos());
        let Some(area) = self.content.local_area(&id) else {
            self.say("There is nothing to explore here.");
            return false;
        };
        let what = self.content.terrain(area.terrain).name.clone();
        let from = self.state.current_level.clone();
        let side = self.state.overworld_step.map(Dir::opposite);
        let content = self.content.clone();
        let arrival = match self.visit(&id, Some(&from)) {
            Ok(level) => side.and_then(|s| arrival(&level.terrain, &content, s)),
            Err(err) => {
                self.say(format!("You cannot go there ({err})."));
                return false;
            }
        };
        if let Err(err) = self.travel(&id, arrival) {
            self.say(format!("You cannot go there ({err})."));
            return false;
        }
        events.push(Event::LevelChanged { from, to: id });
        self.say(format!("You explore the {what}."));
        true
    }

    fn open_door(&mut self, p: Pos, events: &mut Vec<Event>) -> bool {
        let terrain = self.state.current().terrain.get(p).copied();
        let def = terrain.map(|t| self.content.terrain(t));
        match def {
            Some(def) if def.is_closed_door() => {
                let opened = def.opens_to.expect("closed doors have opens_to");
                self.make_noise(self.state.player, Act::OpenDoor(p));
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
                if self.state.actor_at(&self.state.current_level, p).is_some()
                    || !self.state.current().objects_at(p).is_empty()
                {
                    self.say("Something is in the way.");
                    return false;
                }
                self.make_noise(self.state.player, Act::CloseDoor(p));
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
        let kind = self.state.current().kind;
        let Some(link) = link else {
            if kind == LevelKind::Overworld && dir == StairsDir::Down {
                return self.enter_local_area(events);
            }
            self.say(format!("You see no way {way} here."));
            return false;
        };
        let Some(to) = link.to else {
            self.say("These stairs lead nowhere yet.");
            return false;
        };
        let from = self.state.current_level.clone();
        if let Err(err) = self.travel(&to, None) {
            self.say(format!("The stairs are blocked ({err})."));
            return false;
        }
        events.push(Event::LevelChanged { from, to });
        let level = self.state.current();
        let name = level.name.clone();
        let text = match (kind, level.kind, dir) {
            (LevelKind::Overworld, _, _) => format!("You enter {name}."),
            (_, LevelKind::Overworld, _) => format!("You climb out to {name}."),
            (_, _, StairsDir::Down) => format!("You descend the stairs to {name}."),
            (_, _, StairsDir::Up) => format!("You climb the stairs to {name}."),
        };
        self.say(text);
        true
    }

    /// Moves the player to level `to` (generating it on the first visit), arriving at `at` if
    /// given, else on the stairs that lead back, else at the level's start, else on its first
    /// staircase.
    fn travel(&mut self, to: &str, at: Option<Pos>) -> Result<(), WorldError> {
        let from = self.state.current_level.clone();
        self.visit(to, Some(&from))?;
        let target = &self.state.levels[to];
        let arrival = at
            .or_else(|| {
                target
                    .stairs
                    .iter()
                    .find(|s| s.to.as_deref() == Some(from.as_str()))
                    .map(|s| s.pos)
            })
            .or(target.start)
            .or_else(|| target.stairs.first().map(|s| s.pos))
            .ok_or_else(|| WorldError::NoPlayerStart(to.to_string()))?;
        let arrival = self.free_cell_near(to, arrival);
        let player = self.state.player;
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.level = to.to_string();
            actor.pos = arrival;
        }
        self.state.current_level = to.to_string();
        self.state.overworld_step = None;
        // Sounds remembered on the level left say nothing about this one.
        self.state.recent_sounds.clear();
        Ok(())
    }

    /// Level `id` as it is now, generating it on the first visit (entered from `from`).
    fn visit(&mut self, id: &str, from: Option<&str>) -> Result<&LevelState, WorldError> {
        if !self.state.levels.contains_key(id) {
            let template = level_template(&self.content, self.state.seed, id, from)?;
            instantiate_level(&mut self.state, &template);
        }
        Ok(&self.state.levels[id])
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

    /// One action of creature `id` on the current level, taken whenever the scheduler lets it act
    /// (phase 2; `schedule.rs` decides who acts when, and so how often). Returns how long the
    /// action took, which decides how soon the creature may act again: a step takes the level's
    /// step time onto the destination ([`LevelState::step_time`]), anything else the level's turn.
    ///
    /// This is the one place a creature decides and does something. Today it follows its patrol
    /// route, waiting when blocked or without one; creature behavior (AI) replaces that decision
    /// here, and whatever the action lets the player perceive (events, noise) starts here too.
    /// What the creature heard since it last acted ([`Actor::heard`]) is the AI's to read here;
    /// it is spent as the creature acts. Its steps make noise.
    fn creature_turn(&mut self, id: ActorId, events: &mut Vec<Event>) -> Duration {
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.heard.clear();
        }
        let level = self.state.current();
        let actor = &self.state.actors[&id];
        let Some(patrol) = &actor.patrol else {
            return level.turn;
        };
        let from = actor.pos;
        let to = from.step(patrol.route[patrol.step]);
        let step = match level.terrain.get(to).map(|t| self.content.terrain(*t)) {
            Some(def) if def.walkable && self.state.actor_at(&level.id, to).is_none() => {
                level.step_time(def)
            }
            _ => return level.turn,
        };
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
        self.make_noise(id, Act::Step(to));
        step
    }
}

/// The stacks of `stacks` that `chosen` names, and the others, each in order.
fn split(stacks: &[Stack], chosen: Letters) -> (Vec<Stack>, Vec<Stack>) {
    let (picked, rest): (Vec<_>, Vec<_>) = stacks
        .iter()
        .enumerate()
        .partition(|(i, _)| chosen.contains(*i));
    let unzip = |v: Vec<(usize, &Stack)>| v.into_iter().map(|(_, s)| *s).collect();
    (unzip(picked), unzip(rest))
}

/// `A` or `An` for a noun.
fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "An",
        _ => "A",
    }
}

/// The template of level `id`: a hand-made level, or one generated from the run seed: a level of
/// a dungeon branch (`entered_from` is where the up staircase of a branch's first level leads)
/// or a local area of the overworld.
fn level_template<'a>(
    content: &'a Content,
    seed: u64,
    id: &str,
    entered_from: Option<&str>,
) -> Result<Cow<'a, LevelTemplate>, WorldError> {
    if let Some(template) = content.level(id) {
        return Ok(Cow::Borrowed(template));
    }
    if !content.has_level(id) {
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
                energy: 0,
                heard: Vec::new(),
                inventory: spawn.carry.clone(),
            },
        );
    }
}

#[cfg(test)]
mod branch_tests;
#[cfg(test)]
mod hearing_tests;
#[cfg(test)]
mod light_tests;
#[cfg(test)]
mod moon_tests;
#[cfg(test)]
mod object_tests;
#[cfg(test)]
mod overworld_tests;
#[cfg(test)]
mod speed_tests;
#[cfg(test)]
mod sun_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod time_tests;
