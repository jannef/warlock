//! The simulation: world state and the turn pipeline (spec §8).
//!
//! Turn phases, in order: player action → the other actors on the level act in time order until
//! the player may act again, the clock advancing to the moment of each action (the energy
//! scheduler, `schedule.rs`; [`World::creature_turn`] is one creature's action) → environment →
//! perception (sight, light, memory) → perceived events are returned to the caller.
//!
//! Time (spec §8): every action that takes time has a cost in energy (`data/actions.toml`, 1000
//! for a normal action; [`ActionKind`]) and a base time: 6 s ([`crate::content::BASE_TURN`])
//! for everything but a step on the overworld, whose base is the terrain's travel time. Its
//! energy price is the cost times the base in seconds ([`crate::content::price`]; for a step,
//! [`LevelState::step_price`]), and the actor's speed decides how long that takes:
//! `cost / 1000 × 1000 / speed × base`, so a normal action takes the player (normal speed) 6 s,
//! and twice the speed takes half of it. Resting ([`Action::Rest`]) passes many turns in one
//! action.
//!
//! Hearing (spec §5 "Hearing", [`crate::noise`]): steps and doors make noise as they happen, and
//! every other actor in earshot on the level hears it at once ([`Actor::heard`], until it next
//! acts). Perception turns what the player heard from unseen sources into perceived sounds, and
//! news among them into messages ("You hear footsteps to the north-east."), which stop a rest.
//!
//! Weather (spec §5 "Weather", [`crate::weather`]) is a pure function of the seed and the time;
//! the world only keeps a dev override. Perception knows it under the open sky, and a change seen
//! there is a message ("It starts to rain."). Rain starting or growing heavier stops a rest; the
//! rest of the news does not.
//!
//! The overworld (spec §3): stairs down on it are locations leading into other levels; `>` on
//! any other cell enters that cell's local area (`local:x,y`, generated on the first visit) on
//! the side the player walked onto the cell from ([`WorldState`]'s last overworld step), and
//! walking off a local area's edge leads back to its overworld cell (the level state's edge exit).
//!
//! Objects (spec §6): piles lie on cells of a level ([`LevelState`]), actors carry inventories.
//! Picking up and dropping are actions with their own costs; stepping onto a pile says what lies
//! there. An actor's weight is its creature's plus what it carries ([`Actor::weight`]).

mod actor;
mod ai;
mod hash;
mod level;
mod path;
mod schedule;

pub use actor::{Actor, ActorId, ActorKind, Mind, Patrol};
pub(crate) use hash::fnv1a;
pub use level::LevelState;

use crate::action::{Action, AttackKind, Event, Letters, MAX_REST, Outcome, Rest, letter};
use crate::content::{
    ActionKind, Content, EdgeExit, LevelKind, LevelTemplate, StairsDir, Stance, TerrainId,
    local_area_id,
};
use crate::geom::{Dir, Pos};
use crate::light::{LightCache, Sky};
use crate::mapgen::local::arrival;
use crate::messages::MessageLog;
use crate::noise::{self, Act, Recent};
use crate::object::{self, Stack, Weight};
use crate::perception::{Perception, PerceptionView};
use crate::rng::Rng;
use crate::time::{self, Duration, GameTime};
use crate::weather::{self, Weather};
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
    /// Dev: the weather set by hand (`weather:rain`), until it runs out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) weather_override: Option<weather::Override>,
}

impl WorldState {
    pub(crate) fn player(&self) -> &Actor {
        &self.actors[&self.player]
    }

    /// The sky over the current level now, under the weather of the moment (the dev override
    /// while it lasts): the sun, the moons, and their light through the clouds; `None` on a level
    /// not under the open sky. The player's perception and creature sight both see this sky.
    pub(crate) fn sky(&self, content: &Content) -> Option<Sky> {
        self.current().kind.is_sky_exposed().then(|| {
            let over = self.weather_override.as_ref();
            let now = weather::conditions(self.seed, self.time, content.climate(), over);
            Sky::at(self.time, content.moons(), now)
        })
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
    /// A turn: the player's action costs this energy price; the others act until the player may
    /// act again (the clock advances), then perception is recomputed.
    Turn(i64),
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
            weather_override: None,
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
                mind: Mind::default(),
                inventory: Vec::new(),
                stats: def.player_stats,
            },
        );
        instantiate_level(&mut state, &content, &start);
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

    /// How actors `a` and `b` stand toward each other: the stance of their factions (spec §7
    /// "Factions"), so it follows from their types; [`Stance::Neutral`] if either is unknown.
    pub fn stance(&self, a: ActorId, b: ActorId) -> Stance {
        match (self.state.actors.get(&a), self.state.actors.get(&b)) {
            (Some(a), Some(b)) => self.content.stance(a.creature, b.creature),
            _ => Stance::Neutral,
        }
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
            Spent::Turn(price) => {
                self.run_turn(price, &mut events);
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
    /// message was logged that stops a rest. The last turn is cut short to end exactly at
    /// `until`, whatever the player's speed ([`World::price_until`]). Turns that end before any
    /// creature could come into view ([`World::out_of_view_for`]) and in which the weather the
    /// player sees did not change pass without perception, which cannot change the outcome; only
    /// the last turn's events are kept.
    fn rest(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        let start = self.state.time;
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        let said = self.said;
        let mut in_view = self.creatures_in_view();
        let mut came = None;
        while self.state.time < until && came.is_none() && self.said == said {
            let unseen_until = self.state.time + self.out_of_view_for();
            loop {
                events.clear();
                self.run_turn(wait.min(self.price_until(until)), events);
                let done = self.state.time >= until || self.said != said;
                if done || self.state.time > unseen_until || self.weather_turned() {
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

    /// The weather now, which the player would see: on a level under the open sky.
    fn weather_now(&self) -> Option<Weather> {
        let state = &self.state;
        state.current().kind.is_sky_exposed().then(|| {
            let over = state.weather_override.as_ref();
            weather::weather_at(state.seed, state.time, self.content.climate(), over)
        })
    }

    /// Whether the weather the player would see differs from what they saw at the last
    /// perception (so a rest perceives the change and says so).
    fn weather_turned(&self) -> bool {
        self.weather_now() != self.perception.weather()
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

    /// Phase 4: recomputes perception, updates memory, says what news the player heard and how
    /// the weather changed, reports the attacks the player perceived, and drops events the player
    /// did not see.
    fn perceive(&mut self, events: &mut Vec<Event>) {
        let now = self.state.time;
        if self.state.weather_override.is_some_and(|o| o.until <= now) {
            self.state.weather_override = None;
        }
        let before = self.perception.weather();
        *self.perception = Perception::compute(&self.state, &self.content, &mut self.lights);
        let current = self.state.current_level.clone();
        if let Some(level) = self.state.levels.get_mut(&current) {
            self.perception.remember(level);
        }
        self.hear();
        if let (Some(from), Some(to)) = (before, self.perception.weather()) {
            self.weather_news(from, to);
        }
        let player = self.state.player;
        let mut reports = Vec::new();
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
            &Event::Attacked {
                attacker,
                target,
                kind,
                from,
                to,
            } => {
                // Reported if the player saw the attacker or the target, or is the target (the
                // player's own attacks are reported, and only seen ones recorded, when made);
                // kept for presentation only if the player saw both.
                let sees_attacker = self.perception.sees_actors_at(from);
                let sees_target = target == player || self.perception.sees_actors_at(to);
                if attacker != player {
                    let name = |id: ActorId, seen: bool| {
                        let creature = self.state.actors[&id].creature;
                        seen.then(|| self.content.creature(creature).name.as_str())
                    };
                    let who = name(attacker, sees_attacker);
                    let whom = (target != player).then(|| name(target, sees_target));
                    reports.extend(attack_report(who, whom, kind, sees_target));
                }
                attacker == player || (sees_attacker && sees_target)
            }
        });
        for text in reports {
            self.say(text);
        }
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

    /// A message when the weather the player sees changes under the open sky (never on
    /// arriving there: the status bar shows it). Rain starting or growing heavier stops a rest;
    /// clouds gathering or parting and rain easing or stopping are only logged.
    fn weather_news(&mut self, from: Weather, to: Weather) {
        use Weather::{Clear, HeavyRain, Overcast, Rain};
        let text = match (from, to) {
            (Clear, Overcast) => "Clouds gather.",
            (Overcast, Clear) => "The sky clears.",
            (Clear | Overcast, Rain) => "It starts to rain.",
            (Clear | Overcast, HeavyRain) => "It starts to pour.",
            (Rain, HeavyRain) => "The rain grows heavy.",
            (HeavyRain, Rain) => "The rain eases.",
            (Rain | HeavyRain, Overcast) => "The rain stops.",
            (Rain | HeavyRain, Clear) => "The rain stops and the sky clears.",
            _ => return,
        };
        if to.is_rain() && to > from {
            self.say(text);
        } else {
            self.note(text);
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

    /// Logs a message; it stops a rest.
    fn say(&mut self, text: impl Into<String>) {
        self.note(text);
        self.said += 1;
    }

    /// Logs a message that does not stop a rest.
    fn note(&mut self, text: impl Into<String>) {
        let turn = self.state.turn;
        self.state.messages.push(turn, text);
    }

    /// A turn of the player's action `kind` if they did it, else nothing.
    fn took(&self, done: bool, kind: ActionKind) -> Spent {
        if done {
            Spent::Turn(self.price_of(self.state.player, kind))
        } else {
            Spent::Nothing
        }
    }

    /// Phase 1. Returns what the action cost.
    fn player_acts(&mut self, action: &Action, events: &mut Vec<Event>) -> Spent {
        let here = self.player_pos();
        // Whether the action was done, and what kind of action it was for its cost (the same on
        // any level: taking stairs ends on another).
        let (done, kind) = match action {
            Action::Move(dir) => return self.player_moves(*dir, events),
            Action::Wait => (true, ActionKind::Wait),
            Action::Open(dir) => (
                self.open_door(here.step(*dir), events),
                ActionKind::OpenDoor,
            ),
            Action::Close(dir) => (
                self.close_door(here.step(*dir), events),
                ActionKind::CloseDoor,
            ),
            Action::Descend => (
                self.take_stairs(StairsDir::Down, events),
                ActionKind::Stairs,
            ),
            Action::Ascend => (self.take_stairs(StairsDir::Up, events), ActionKind::Stairs),
            Action::ToggleLantern => (self.toggle_lantern(), ActionKind::Lantern),
            Action::PickUp(letters) => (self.pick_up(*letters), ActionKind::PickUp),
            Action::Drop(letters) => (self.drop_objects(*letters), ActionKind::Drop),
            Action::Rest(goal) => return self.start_rest(*goal),
            Action::SetVision(id) => return self.set_vision(id),
            Action::SetWeather(weather) => {
                let over = weather::Override::new(*weather, self.state.time);
                self.state.weather_override = Some(over);
                self.say(format!(
                    "Weather: {} for {} (dev).",
                    weather.name().to_lowercase(),
                    weather::OVERRIDE.spoken()
                ));
                return Spent::NoTime;
            }
            Action::SkipTime(duration) => {
                self.say(format!("Time skips ahead {} (dev).", duration.spoken()));
                return Spent::Skip(*duration);
            }
        };
        self.took(done, kind)
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
            let opened = self.open_door(to, events);
            return self.took(opened, ActionKind::OpenDoor);
        }
        if !def.walkable {
            let name = def.name.clone();
            self.say(format!("The {name} blocks your way."));
            return Spent::Nothing;
        }
        if let Some(other) = self.state.actor_at(&level.id, to) {
            // What the player knows of it: its name if they see it, else just that something is
            // there; and its stance toward them (from its type), which decides between attacking
            // and stopping.
            let target = other.id;
            let seen = self.perception.sees_actors_at(to);
            let name = self.content.creature(other.creature).name.clone();
            let what = if seen {
                format!("the {name}")
            } else {
                "something".into()
            };
            let player = self.state.player;
            if self.stance(player, target) == Stance::Hostile {
                if seen {
                    events.push(Event::Attacked {
                        attacker: player,
                        target,
                        kind: AttackKind::Melee,
                        from,
                        to,
                    });
                }
                self.say(format!("You attack {what}."));
                self.make_noise(player, Act::Attack(AttackKind::Melee));
                return self.took(true, AttackKind::Melee.action());
            }
            let what = if seen {
                format!("The {name}")
            } else {
                "Something".into()
            };
            self.say(format!("{what} is in the way."));
            return Spent::Nothing;
        }
        let player = self.state.player;
        let step = self.step_price(player, def);
        self.state.overworld_step = (level.kind == LevelKind::Overworld).then_some(dir);
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

    /// Walks off the edge of a local area, back to its overworld cell: like taking stairs.
    fn leave_by_edge(&mut self, exit: EdgeExit, events: &mut Vec<Event>) -> Spent {
        let from = self.state.current_level.clone();
        if let Err(err) = self.travel(&exit.to, Some(exit.pos)) {
            self.say(format!("You cannot leave this way ({err})."));
            return Spent::Nothing;
        }
        events.push(Event::LevelChanged { from, to: exit.to });
        let name = self.state.current().name.clone();
        self.say(format!("You return to {name}."));
        self.took(true, ActionKind::Stairs)
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
            instantiate_level(&mut self.state, &self.content, &template);
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
}

/// The message for a creature's attack the player perceived: by `who` (a creature type's name,
/// `None` if the player did not see the attacker), on `whom` (`None`: the player; `Some(None)`: a
/// creature the player did not see). Nothing if the player saw neither end and was not the
/// target (`target_seen` is whether the target was seen, or is the player).
fn attack_report(
    who: Option<&str>,
    whom: Option<Option<&str>>,
    kind: AttackKind,
    target_seen: bool,
) -> Option<String> {
    if who.is_none() && !target_seen {
        return None;
    }
    let who = who.map_or_else(|| "Something".to_string(), |n| format!("The {n}"));
    let whom = match whom {
        None => "you".to_string(),
        Some(Some(name)) => format!("the {name}"),
        Some(None) => "something".to_string(),
    };
    let verb = match kind {
        AttackKind::Melee => "attacks",
        AttackKind::Ranged => "shoots at",
        AttackKind::Spell => "casts a spell at",
    };
    Some(format!("{who} {verb} {whom}."))
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

/// Creates the runtime state for a level and spawns its creatures, each with its type's stats.
fn instantiate_level(state: &mut WorldState, content: &Content, template: &LevelTemplate) {
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
                mind: Mind::default(),
                inventory: spawn.carry.clone(),
                stats: content.creature(spawn.creature).stats,
            },
        );
    }
}

#[cfg(test)]
mod ai_tests;
#[cfg(test)]
mod branch_tests;
#[cfg(test)]
mod cost_tests;
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
mod stats_tests;
#[cfg(test)]
mod sun_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod time_tests;
#[cfg(test)]
mod weather_tests;
