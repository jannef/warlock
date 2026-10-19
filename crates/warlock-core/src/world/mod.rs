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
//! there. An actor's weight is its creature's plus what it carries ([`Actor::weight`]), worn and
//! wielded things included. Wearing and wielding (`equip.rs`) take the time of each slot's
//! action; dropping something worn takes it off first.
//!
//! Combat (spec §8 "Combat", `fight.rs`): attacks are rolled against Defence and take hit
//! points; a creature at 0 dies and drops what it carried, and the player's death ends the run
//! ([`World::death`]): every action after it is refused. Hit points come back with game time.
//!
//! Burden (spec §8 "Burden", [`crate::burden`]): what an actor carries against what its
//! Strength, Toughness, and size let it carry slows every action it takes (in the prices,
//! `schedule.rs`), caps its Dexterity in Defence, makes its steps louder, and, overloaded, keeps
//! it from moving. The player is told when their burden changes.
//!
//! Ranged combat (`shoot.rs`): shots and throws fly along a line, attack whoever stands in their
//! way, and come down to be picked up again; burning missiles are fires that light the level
//! until they burn out.
//!
//! Special rooms (spec §7 "Special rooms", [`crate::content::RoomEffectDef`]): a level's special
//! rooms change the rules inside them, through one question, [`World::room_effect_at`]: the
//! light (and so what the player and creatures see), the noise made there, regeneration, and
//! whether creatures hostile to the player step in. The first time the player is inside one they
//! are told what it is.
//! Weapon skills (spec §8 "Weapon skills", [`crate::weapon_skills`], `fight.rs`): every attack
//! draws on a skill (the weapon's, unarmed, thrown, shield bashing) for its to-hit, damage,
//! threat, and cost, and the player's attacks and shield blocks earn marks toward its next rank
//! ("You feel more skilled with daggers."). A shield bash ([`Action::Bash`]) is a blow with a
//! held shield.
//! Digging (`dig.rs`): walls that break give way to a digging tool, turn after turn like a rest,
//! and to explosions stronger than they are hard; they become rubble.
//! Blood (`bleed.rs`, [`crate::blood`]): big hits and kills stain the floor and the walls around
//! the victim. Presentation state kept with the level; no rule reads it.
//! Status effects and healing (spec §8 "Status effects", "Healing", `status.rs`): big hits and
//! criticals make their targets bleed, a giant rat's bite may poison; effects cost hit points
//! and ease as the clock moves, slow healing, and can kill. Hit points come back with game
//! time, twice as fast while the player rests.
//! Hunger (spec §8 "Hunger", `hunger.rs`, `eat.rs`): game time burns the player's nutrition, and
//! the state it leaves them in takes effect (Weak, Fainting, Starving, Bloated); meals pass turns
//! like a rest and feed at their end. Creatures that die leave corpses, which rot away.
//! Temperature and ice (spec §5 "Temperature", "Ice"; `ice.rs`): water freezes in the cold and
//! thaws in the warmth, brought up to the clock as it moves on the player's level (and on
//! arrival), never ticking while nobody is there. Ice holds actors by their weight; one that
//! breaks through plunges into the freezing water and climbs out onto the ice in a few turns.
//!
//! Traps (spec §8 "Traps", `trap.rs`): whoever steps onto a trap sets it off, the player and
//! creatures alike: arrows, spears, fireballs, lightning, pits, alarms, and teleports. Traps are
//! hidden until the player sets one off or sees a creature do it; then they are known and
//! remembered.

mod actor;
mod ai;
mod bleed;
mod dig;
mod disease;
mod door;
mod eat;
mod equip;
mod experience;
mod fight;
mod hash;
mod hunger;
mod ice;
mod level;
mod loot;
mod path;
mod quest;
mod schedule;
mod shoot;
mod spell;
mod status;
mod talk;
mod train;
mod trap;

pub use actor::{Actor, ActorId, ActorKind, Mind, Patrol};
pub use dig::{MAX_DIG, WORK_SCALE, dig_work, digs_to_break};
pub use eat::Rot;
pub(crate) use eat::rot_at;
pub use fight::{Cause, Death};
pub(crate) use hash::fnv1a;
pub use level::{Fire, LevelState};
pub use loot::Identified;
pub use quest::{QuestRecord, QuestView, Quests, Stage};
pub use talk::{Conversation, Talk, TopicView};

use crate::action::{
    Action, AttackKind, Breach, DiseaseNews, Event, Letters, MAX_REST, MissileEnd, Outcome, Rest,
    letter,
};
use crate::burden::{self, Burden, Load};
use crate::character::{Character, CharacterSpec};
use crate::content::{
    ActionKind, CombatStance, Content, CreatureId, EdgeExit, LevelKind, LevelTemplate,
    RoomEffectDef, StairsDir, Stance, TerrainId, TrapEffect, local_area_id,
};
use crate::geom::{Dir, Pos};
use crate::hunger::Satiation;
use crate::light::{LightCache, Sky};
use crate::mapgen::local::arrival;
use crate::messages::MessageLog;
use crate::noise::{self, Act, Recent};
use crate::object::{self, Stack, Weight};
use crate::perception::VIEW_RADIUS;
use crate::perception::{Perception, PerceptionView};
use crate::rng::Rng;
use crate::sight::LineOfSight;
use crate::skills::{Skills, Use};
use crate::time::{self, Duration, GameTime};
use crate::weapon_skills::WeaponSkills;
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
    #[error("cannot create the character: {0}")]
    Character(String),
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
    /// The player's character (spec §8 "Character creation"): race, gender, and class, the stat
    /// the gender raised, birth date, and birth sign; none when the scenario gives the player's
    /// eyes and stats itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) character: Option<Character>,
    /// Whether the player's lantern (the scenario's `lantern`) is lit.
    pub(crate) lantern_lit: bool,
    /// The step that brought the player onto their overworld cell, if they walked there: `>`
    /// then arrives on the side of the local area they came from. Cleared on any level change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) overworld_step: Option<Dir>,
    /// Sounds the player perceived lately: those like them are no news ([`noise::news`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) recent_sounds: Vec<Recent>,
    /// The training random stream (spec §8 "Skills"): the chances of training marks, apart from
    /// the gameplay stream so training never changes a fight's rolls.
    pub(crate) training: Rng,
    /// Dev: the weather set by hand (`weather:rain`), until it runs out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) weather_override: Option<weather::Override>,
    /// How the run ended, once the player has died: nothing more happens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) death: Option<Death>,
    /// The player's experience total (spec §8 "Experience"); their level follows from it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) experience: u32,
    /// The player's quests and the kills they count (spec §8 "Quests").
    #[serde(default, skip_serializing_if = "Quests::is_empty")]
    pub(crate) quests: Quests,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
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

    /// What `actor` carries against what it can (spec §8 "Burden"): the player by their race's
    /// size (and whether it ignores Burdened) when the run has a character, anyone else by its
    /// creature type's size.
    pub(crate) fn load(&self, content: &Content, actor: &Actor) -> Load {
        let race = (self.character.as_ref())
            .filter(|_| actor.kind == ActorKind::Player)
            .and_then(|c| c.spec.race(content).ok());
        burden::load(actor, race, content)
    }

    /// The size of `actor`, an index into [`Content::sizes`]: the player's race's when the run
    /// has a character, else its creature type's.
    pub(crate) fn size_of(&self, content: &Content, actor: &Actor) -> usize {
        let race = (self.character.as_ref())
            .filter(|_| actor.kind == ActorKind::Player)
            .and_then(|c| c.spec.race(content).ok());
        race.map_or(content.creature(actor.creature).size, |r| r.size)
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
        // A dead player stopped where they fell, whether or not they were ready to act.
        match self.actors.get(&self.player) {
            Some(p)
                if p.kind == ActorKind::Player
                    && p.level == self.current_level
                    && (p.energy >= 0 || self.death.is_some()) => {}
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
    /// Creatures that died during the current action, as they were (not saved): what they did
    /// that turn is said when it is perceived.
    fallen: Vec<Actor>,
    /// What the player's kills earned them during the current action (not saved): said after
    /// the attack's own messages.
    news: Vec<String>,
    /// Traps sprung since this world was created or loaded (not saved): a rest perceives every
    /// turn in which one went off (a teleport moves someone farther than a step).
    sprung: u64,
    /// Lines muttered during the current turn (not saved): said when perception runs, if the
    /// player heard them.
    mutters: Vec<talk::Mutter>,
    /// Whether the player is resting ([`World::rest`]; not saved: a rest is one action): they
    /// heal faster.
    resting: bool,
}

/// How a long action (a rest, digging; [`World::keep_at`]) ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ended {
    /// There was nothing more to do.
    Done,
    /// This creature came into view.
    Came(ActorId),
    /// A message was logged that stops it.
    Said,
    /// The player died.
    Died,
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
    /// A meal (see [`World::dine`]).
    Meal(eat::Dish),
    /// Digging into the wall at this cell (see [`World::dig`]).
    Dig(Pos),
    /// Dev: the clock jumps ahead; nobody acts, then perception is recomputed.
    Skip(Duration),
}

impl World {
    /// Starts a new run of `scenario` with the scenario's own character. `seed` defaults to the
    /// scenario's seed.
    pub fn new(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
    ) -> Result<Self, WorldError> {
        Self::with_character(content, scenario, seed, None)
    }

    /// Starts a new run of `scenario` as `character` (spec §8 "Character creation"); without
    /// one, as the scenario's character, or if it names none, with the eyes and stats it gives.
    /// `seed` defaults to the scenario's seed.
    pub fn with_character(
        content: Arc<Content>,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
    ) -> Result<Self, WorldError> {
        let def = content
            .scenario(scenario)
            .ok_or_else(|| WorldError::UnknownScenario(scenario.to_string()))?;
        let seed = seed.unwrap_or(def.seed);
        // The player's hit points beyond their Toughness: what their race and class give, or
        // without a character what the scenario gives. Without a character the player is trained
        // in nothing and has no weapon skills.
        let chosen = character.or(def.character.as_ref());
        let (stats, vision, hp_base, skills, character, weapon_skills) = match chosen {
            Some(spec) => {
                let s = (spec.start(&content, seed, def.start)).map_err(WorldError::Character)?;
                let character = Some(s.character);
                (
                    s.stats,
                    s.vision,
                    s.hp,
                    s.skills,
                    character,
                    s.weapon_skills,
                )
            }
            None => (
                def.player_stats,
                def.vision().to_string(),
                i32::from(def.hp),
                Skills::default(),
                None,
                WeaponSkills::default(),
            ),
        };
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
            training: Rng::training(seed),
            turn: 0,
            time: def.start,
            current_level: start.id.clone(),
            levels: BTreeMap::new(),
            actors: BTreeMap::new(),
            player: ActorId(0),
            next_actor_id: 1,
            messages: MessageLog::default(),
            vision,
            character,
            lantern_lit: false,
            overworld_step: None,
            recent_sounds: Vec::new(),
            weather_override: None,
            death: None,
            experience: 0,
            quests: Quests::default(),
        };
        let mut player = Actor {
            id: ActorId(0),
            kind: ActorKind::Player,
            creature: player_creature,
            level: start.id.clone(),
            pos: player_pos,
            patrol: None,
            energy: 0,
            heard: Vec::new(),
            mind: Mind::default(),
            inventory: def.carry.clone(),
            equipment: Default::default(),
            stats,
            hp: 0,
            hp_base,
            regen: 0,
            skills,
            effects: Default::default(),
            diseases: Default::default(),
            damage: Default::default(),
            weapon_skills,
            faction_override: None,
            npc: None,
            talk: Talk::default(),
            satiation: content.hunger().map(|h| Satiation::new(h.start)),
            stance: CombatStance::Neutral,
        };
        player.hp = player.max_hp(&content);
        state.actors.insert(ActorId(0), player);
        instantiate_level(&mut state, &content, &start);
        drop(start); // It may borrow `content`, which `assemble` takes.
        state
            .messages
            .push(0, format!("Welcome to {}. Press ? for controls.", def.name));
        if let Some(text) = enter_room(&mut state, &content) {
            state.messages.push(0, text);
        }
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
        if let Some(character) = &state.character {
            character
                .check(&content)
                .map_err(|m| WorldError::InconsistentState(format!("character: {m}")))?;
        }
        if let Some(id) = state.quests.unknown_to(&content) {
            return Err(WorldError::InconsistentState(format!(
                "the quests know `{id}`, which this content does not have"
            )));
        }
        if let Some(level) = state.levels.values().find(|l| !l.ids_valid_for(&content)) {
            return Err(WorldError::InconsistentState(format!(
                "level `{}` uses terrain, lights, or objects this content does not have",
                level.id
            )));
        }
        let creatures = content.creature_defs().len();
        let skills = content.skills().len();
        if let Some(actor) = state.actors.values().find(|a| {
            usize::from(a.creature.0) >= creatures
                || !a.inventory.iter().all(|s| s.is_valid_for(&content))
                || a.inventory.len() > object::INVENTORY_SLOTS
                || !a.equipment.is_valid_for(&a.inventory, &content)
                || a.skills.iter().any(|(id, _)| usize::from(id.0) >= skills)
                || !a.weapon_skills.is_valid_for(content.weapon_skills())
        }) {
            return Err(WorldError::InconsistentState(format!(
                "actor {} is a creature, carries objects, wears them in slots, or knows skills or \
                 weapon skills this content does not have",
                actor.id
            )));
        }
        if let Some(actor) = state
            .actors
            .values()
            .find(|a| !a.effects.is_valid_for(&content))
        {
            return Err(WorldError::InconsistentState(format!(
                "actor {} suffers status effects this content does not have",
                actor.id
            )));
        }
        // Diseased exactly while a disease shows; stat damage no score could carry is refused.
        let diseased = |a: &Actor| content.disease_status().is_some_and(|s| a.effects.has(s));
        if let Some(actor) = state.actors.values().find(|a| {
            !a.diseases.is_valid_for(&content)
                || diseased(a) != a.diseases.shows()
                || !a.damage.is_valid_for(&a.stats)
        }) {
            return Err(WorldError::InconsistentState(format!(
                "actor {} carries diseases or stat damage this content does not allow",
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
            fallen: Vec::new(),
            news: Vec::new(),
            sprung: 0,
            mutters: Vec::new(),
            resting: false,
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

    /// The player's quests and the kills they count (spec §8 "Quests"). For dev tools; the
    /// player's quest log reads [`PerceptionView::quests`].
    pub fn quests(&self) -> &Quests {
        &self.state.quests
    }

    /// The player's character, if the run has one (spec §8 "Character creation").
    pub fn character(&self) -> Option<&Character> {
        self.state.character.as_ref()
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
    /// "Factions"): their types' unless a level placed them elsewhere; [`Stance::Neutral`] if either is unknown.
    pub fn stance(&self, a: ActorId, b: ActorId) -> Stance {
        match (self.state.actors.get(&a), self.state.actors.get(&b)) {
            (Some(a), Some(b)) => self
                .content
                .stance_of(a.faction(&self.content), b.faction(&self.content)),
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
            character: self.state.character.as_ref(),
            time: self.state.time,
            death: self.state.death.as_ref(),
            experience: self.state.experience,
            quests: &self.state.quests,
            snapshot: &self.perception,
        }
    }

    /// The weight of actor `id`, on any level: its creature's base weight plus everything it
    /// carries. For the rules and dev tools; the player is shown only their own
    /// ([`PerceptionView::own_weight`]).
    pub fn actor_weight(&self, id: ActorId) -> Option<Weight> {
        self.state.actors.get(&id).map(|a| a.weight(&self.content))
    }

    /// What actor `id`, on any level, carries against what it can (spec §8 "Burden"). For the
    /// rules and dev tools; the player is shown only their own ([`PerceptionView::own_load`]).
    pub fn actor_load(&self, id: ActorId) -> Option<Load> {
        (self.state.actors.get(&id)).map(|a| self.state.load(&self.content, a))
    }

    /// What actor `id` carries against what it can.
    fn load_of(&self, id: ActorId) -> Load {
        self.state.load(&self.content, &self.state.actors[&id])
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

    /// The effect of the special room at `pos` on level `level`, if that level has been visited
    /// and a special room lies there (spec §7 "Special rooms"). This is the one question the rules
    /// ask of special rooms (through the level's state inside the core); ground truth, **for the
    /// rules, tests, and developer tools**: the player knows only the rooms they have been in
    /// ([`PerceptionView::room`]).
    pub fn room_effect_at(&self, level: &str, pos: Pos) -> Option<&RoomEffectDef> {
        (self.state.levels.get(level)?).room_effect_at(&self.content, pos)
    }

    /// Applies one player action and runs the rest of the turn if it took time. Once the player
    /// has died, nothing more happens: every action is refused.
    pub fn apply(&mut self, action: &Action) -> Outcome {
        if self.state.death.is_some() {
            return Outcome::default();
        }
        self.fallen.clear();
        let (turn, time) = (self.state.turn, self.state.time);
        let mut events = Vec::new();
        let load = self.load_of(self.state.player);
        let (burden, slowed) = (load.burden(), load.penalty() != Burden::Unburdened);
        let quests = self.quest_progress();
        let spent = self.player_acts(action, &mut events);
        // Taking off what raised them lowers the player's maximum hit points.
        self.keep_hp_within_max(self.state.player);
        self.burden_news(burden);
        self.quest_news(&quests);
        // Acting (not waiting) under a load that slows the player trains balance.
        let acted = matches!(spent, Spent::Turn(_)) && !matches!(action, Action::Wait);
        if acted && slowed && self.trains(Use::Burdened) {
            self.train(Use::Burdened);
        }
        // A status effect that lowers Strength during the turn may change it too, and a wound
        // the player gave may kill what a quest asks for.
        let burden = self.load_of(self.state.player).burden();
        let quests = self.quest_progress();
        match spent {
            Spent::Nothing => {}
            Spent::NoTime => self.perceive(&mut events),
            Spent::Turn(price) => {
                self.run_turn(price, &mut events);
                self.perceive(&mut events);
            }
            Spent::Rest(until, goal) => self.rest(until, goal, &mut events),
            Spent::Meal(dish) => self.dine(dish, &mut events),
            Spent::Dig(at) => self.dig(at, &mut events),
            Spent::Skip(duration) => {
                // Nobody acts or gains energy.
                self.state.time += duration;
                self.player_mut().heard.clear();
                self.perceive(&mut events);
            }
        }
        if self.state.death.is_none() {
            self.burden_news(burden);
            self.quest_news(&quests);
        }
        Outcome {
            took_turn: self.state.turn != turn,
            elapsed: self.state.time.since(time),
            events,
        }
    }

    /// Rests until `until`: turn after turn of waiting ([`World::keep_at`]), the last cut short to
    /// end exactly at `until`, whatever the player's speed ([`World::price_until`]); then says
    /// how long the player rested. The player heals faster all the while.
    fn rest(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        self.resting = true;
        self.rest_until(until, goal, events);
        self.resting = false;
    }

    fn rest_until(&mut self, until: GameTime, goal: Rest, events: &mut Vec<Event>) {
        let start = self.state.time;
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| wait.min(w.price_until(until)),
        );
        let rested = self.state.time.since(start).spoken();
        let text = match (ended, goal) {
            (Ended::Died, _) => return,
            (Ended::Came(id), _) => format!(
                "{} comes into view; you stop resting after {rested}.",
                self.creature_called(id)
            ),
            (Ended::Said, _) => format!("You stop resting after {rested}."),
            (Ended::Done, Rest::For(_)) => format!("You rest for {rested}."),
            (Ended::Done, Rest::Dawn) => "You rest until dawn.".into(),
            (Ended::Done, Rest::Dusk) => "You rest until dusk.".into(),
        };
        self.say(text);
    }

    /// Passes turn after turn of a long action (a rest, digging) through the normal pipeline
    /// while `more` says there is more to do: the player does `act` (which returns what it
    /// costs), the others act, perception. It stops after a turn in which a creature came into
    /// view (one not seen the turn before; but for one who talks and fights no one,
    /// [`World::disturbs_rest`]: the player sees who it is) or a message was logged that stops a
    /// rest, and when the player dies. Turns that end before any creature that disturbs a rest
    /// could come into view ([`World::out_of_view_for`]), in which the player heard nothing, in
    /// which the weather the player sees did not change, in which none of the player's diseases
    /// showed or went badly (news that stops a rest), and in which no trap went off (a teleport
    /// moves a creature farther than its steps) pass without perception, which cannot change the
    /// outcome; only the last turn's events are kept, and the player's status effects beginning
    /// and ending in the others, and their diseases' news (so they are said).
    fn keep_at(
        &mut self,
        events: &mut Vec<Event>,
        more: impl Fn(&Self) -> bool,
        mut act: impl FnMut(&mut Self, &mut Vec<Event>) -> i64,
    ) -> Ended {
        let said = self.said;
        let mut in_view = self.watched_in_view();
        let mut came = None;
        let alive = |w: &Self| w.state.death.is_none();
        while more(self) && came.is_none() && self.said == said && alive(self) {
            // What was perceived was said.
            events.clear();
            let unseen_until = self.state.time + self.out_of_view_for();
            let sprung = self.sprung;
            loop {
                events.retain(|e| matches!(e, Event::Status { .. } | Event::Disease { .. }));
                let price = act(self, events);
                self.run_turn(price, events);
                let done = !more(self) || self.said != said || !alive(self);
                // Something heard may be news (or someone muttering): perception says. So does a
                // disease showing or going badly (news that stops a rest), and a trap going off
                // (a teleport moves someone farther than its steps).
                let heard = !self.state.player().heard.is_empty();
                let ill = events.iter().any(
                    |e| matches!(e, Event::Disease { news, .. } if *news != DiseaseNews::Recovered),
                );
                let trapped = self.sprung != sprung;
                if done
                    || heard
                    || ill
                    || trapped
                    || self.state.time > unseen_until
                    || self.weather_turned()
                {
                    break;
                }
            }
            self.perceive(events);
            let now_in_view = self.watched_in_view();
            came = now_in_view.iter().find(|a| !in_view.contains(a)).copied();
            in_view = now_in_view;
        }
        match came {
            _ if !alive(self) => Ended::Died,
            Some(id) => Ended::Came(id),
            None if self.said != said => Ended::Said,
            None => Ended::Done,
        }
    }

    /// A message when the player's burden is no longer `before` (spec §8 "Burden"): picking
    /// something up or dropping it changed it. Said right after the action that did it.
    fn burden_news(&mut self, before: Burden) {
        let now = self.load_of(self.state.player).burden();
        if now != before {
            self.say(now.news());
        }
    }

    /// Whether the player can move under their load; if not, says so (spec §8 "Burden").
    fn can_move(&mut self) -> bool {
        let movable = self.load_of(self.state.player).penalty().can_move();
        if !movable {
            self.say(burden::MOVE_REFUSED);
        }
        movable
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

    /// Creatures the player sees whose coming into view stops a rest ([`World::disturbs_rest`]).
    fn watched_in_view(&self) -> Vec<ActorId> {
        let mut seen = self.creatures_in_view();
        seen.retain(|id| self.disturbs_rest(&self.state.actors[id]));
        seen
    }

    /// Phase 4: recomputes perception, updates memory, tells the player about a special room they
    /// are in for the first time, says what news they heard and how the weather changed, reports
    /// the attacks they perceived, and drops events they did not see.
    fn perceive(&mut self, events: &mut Vec<Event>) {
        let now = self.state.time;
        // The ice of the moment (after a skip or on arrival), under the weather it had.
        self.update_ice();
        if self.state.weather_override.is_some_and(|o| o.until <= now) {
            self.state.weather_override = None;
        }
        self.burn_out();
        self.rot_away();
        let before = self.perception.weather();
        *self.perception = Perception::compute(&self.state, &self.content, &mut self.lights);
        self.learn_traps(events);
        let current = self.state.current_level.clone();
        if let Some(level) = self.state.levels.get_mut(&current) {
            self.perception.remember(level);
        }
        if let Some(text) = enter_room(&mut self.state, &self.content) {
            self.say(text);
        }
        self.hear();
        self.say_mutters();
        if let (Some(from), Some(to)) = (before, self.perception.weather()) {
            self.weather_news(from, to);
        }
        let reports = self.perceive_events(events);
        for (text, stops_rest) in reports {
            if stops_rest {
                self.say(text);
            } else {
                self.note(text);
            }
        }
        // What kills earned the player meanwhile (a creature their wound bled to death), and a
        // weapon skill a blocked blow raised (`fight.rs`).
        self.say_news();
    }

    /// Traps the player saw go off are known from now on (spec §8 "Traps"): those they set off
    /// themselves, and those set off where they see (a creature they see stepping onto one).
    fn learn_traps(&mut self, events: &[Event]) {
        let player = self.state.player;
        let current = self.state.current_level.clone();
        for event in events {
            if let Event::TrapSprung { at, victim, .. } = *event
                && (victim == player || self.perception.sees_actors_at(at))
                && let Some(trap) =
                    (self.state.levels.get_mut(&current)).and_then(|l| l.traps.get_mut(&at))
            {
                trap.known = true;
            }
        }
    }

    /// Keeps the events the player perceived (dropping the rest) and returns the messages for
    /// the attacks, traps, deaths, status effects, diseases, and walls giving way among them,
    /// each with whether it stops a rest. A creature's attack is said if the player saw the
    /// attacker or the target, or is the target, and kept for presentation if they saw both (the
    /// player's own attacks were said, and only seen ones recorded, when made); a death is said
    /// and kept with the attack that caused it, if the player saw who died. The player's own
    /// death is always said, and settles whether they saw what killed them. A death of a status
    /// effect is said if the player sees where it happened ("The giant rat bleeds to death.").
    /// The player's own status effects are said as they begin (stopping a rest) and end (not
    /// stopping one); their diseases as they show and worsen (stopping a rest) and leave (not
    /// stopping one). A trap going off is said and kept if the player set it off, saw its cell,
    /// or saw any of its blast; what it did to someone if they are the player or the player saw
    /// them (each stopping a rest). A wall that gave way is kept if the player sees its cell, and
    /// one blasted open is said ("The wall collapses!"; digging says its own).
    fn perceive_events(&mut self, events: &mut Vec<Event>) -> Vec<(String, bool)> {
        use fight::{End, Words, attack_message, death_message};
        let player = self.state.player;
        let content = self.content.clone();
        let mut reports: Vec<(String, bool)> = Vec::new();
        // What came last: an attack (its attacker, its target, whether the player saw the
        // target, and the attacker's creature type if the player saw that), or a trap (its
        // target and whether the player saw it).
        enum Last {
            Attack(ActorId, ActorId, bool, Option<CreatureId>),
            Trap(ActorId, bool),
        }
        let mut last: Option<Last> = None;
        let mut killer_seen: Option<Option<CreatureId>> = None;
        let mut collapsed = 0;
        let perception = &self.perception;
        let end = |id: ActorId, seen: bool| match self.actor_or_fallen(id) {
            _ if id == player => End::You,
            Some(a) if seen => End::Seen(content.creature(a.creature).name.as_str()),
            _ => End::Unseen,
        };
        // A missile is seen where it flies over cells the player sees: its path is cut to the
        // stretch from the first of them to the last, and where it was loosed is known only if
        // the player saw it leave. A burning one is light itself: it is seen wherever it flies
        // in the player's line of sight, in the dark too.
        let mut los: Option<LineOfSight> = None;
        let line_of_sight = |los: &mut Option<LineOfSight>| {
            if los.is_none() {
                let level = self.state.current();
                let viewer = self.state.player().pos;
                *los = Some(LineOfSight::compute(viewer, VIEW_RADIUS, |p| {
                    level.is_opaque(&content, p)
                }));
            }
        };
        for event in events.iter_mut() {
            // A spell effect is light itself too (`spell.rs`).
            if let Event::Spell { .. } = event {
                line_of_sight(&mut los);
                let los = los.as_ref().expect("computed");
                spell::perceive_spell(event, |p| perception.is_visible(p) || los.contains(p));
                continue;
            }
            if let Event::Missile {
                object,
                from,
                path,
                end,
            } = event
            {
                let burning = content.object(*object).burning.is_some();
                if burning {
                    line_of_sight(&mut los);
                }
                let in_sight = |p: Pos| burning && los.as_ref().is_some_and(|l| l.contains(p));
                let seen = |p: &Pos| perception.is_visible(*p) || in_sight(*p);
                let (Some(first), Some(last)) =
                    (path.iter().position(seen), path.iter().rposition(seen))
                else {
                    path.clear();
                    continue;
                };
                if last + 1 < path.len() {
                    *end = MissileEnd::Unseen;
                }
                path.truncate(last + 1);
                path.drain(..first);
                if first > 0 || from.is_some_and(|f| !perception.sees_actors_at(f)) {
                    *from = None;
                }
            }
        }
        // Whether the player perceived each trap that went off, in order: they set it off, see
        // its cell, or see any of its blast (a spell before the next trap).
        let mut sprung_seen = Vec::new();
        for (i, event) in events.iter().enumerate() {
            if let Event::TrapSprung { at, victim, .. } = *event {
                let blast = (events[i + 1..].iter())
                    .take_while(|e| !matches!(e, Event::TrapSprung { .. }))
                    .any(|e| matches!(e, Event::Spell { cells, .. } if !cells.is_empty()));
                sprung_seen.push(victim == player || perception.sees_actors_at(at) || blast);
            }
        }
        let mut sprung_seen = sprung_seen.into_iter();
        events.retain(|e| match *e {
            Event::ActorMoved { actor, from, to } => {
                actor == player || perception.sees_actors_at(from) || perception.sees_actors_at(to)
            }
            Event::DoorOpened { pos }
            | Event::DoorClosed { pos }
            | Event::DoorKicked { pos }
            | Event::DoorBroken { pos } => perception.is_visible(pos),
            Event::WallBroken { pos, cause } => {
                let seen = perception.is_visible(pos);
                if seen && cause == Breach::Blast {
                    collapsed += 1;
                }
                seen
            }
            Event::LevelChanged { .. } | Event::Talked(_) => true,
            Event::Missile { ref path, .. } => !path.is_empty(),
            Event::Spell { ref cells, .. } => !cells.is_empty(),
            Event::Attacked {
                attacker,
                target,
                kind,
                from,
                to,
                strike,
                with,
            } => {
                let sees_attacker = attacker == player || perception.sees_actors_at(from);
                let sees_target = target == player || perception.sees_actors_at(to);
                let creature = (self.actor_or_fallen(attacker)).map(|a| a.creature);
                let by = creature.filter(|_| sees_attacker);
                last = Some(Last::Attack(attacker, target, sees_target, by));
                if attacker == player {
                    return true;
                }
                if let Some(me) = self.actor_or_fallen(attacker) {
                    let words = Words::of(me, kind, with, &content);
                    let hide = (self.actor_or_fallen(target))
                        .and_then(|t| content.creature(t.creature).natural.as_ref())
                        .map(|h| h.name.as_str());
                    let (a, t) = (end(attacker, sees_attacker), end(target, sees_target));
                    let said = attack_message(a, t, &words, kind, strike, hide);
                    reports.extend(said.map(|text| (text, true)));
                }
                sees_attacker && sees_target
            }
            Event::FellThroughIce { actor, at } => {
                // The player's own fall was said as it happened.
                if actor == player {
                    return true;
                }
                let seen = perception.sees_actors_at(at);
                if let Some(me) = self.actor_or_fallen(actor).filter(|_| seen) {
                    let name = &content.creature(me.creature).name;
                    reports.push((format!("The {name} falls through the ice!"), true));
                }
                seen
            }
            Event::Status {
                actor,
                status,
                began,
            } => {
                let def = content.status(status);
                let text = if began { &def.start } else { &def.end };
                if let Some(text) = text.clone().filter(|_| actor == player) {
                    reports.push((text, began));
                }
                actor == player
            }
            Event::Disease {
                actor,
                disease,
                news,
            } => {
                let def = content.disease(disease);
                let (text, stops) = match news {
                    DiseaseNews::Onset => (def.onset.as_str(), true),
                    DiseaseNews::Worse(n) => (def.symptom(n), true),
                    DiseaseNews::Recovered => (def.recovered.as_str(), false),
                };
                if actor == player {
                    reports.push((text.to_string(), stops));
                }
                actor == player
            }
            Event::Died {
                actor,
                at,
                of: Some(status),
            } => {
                if actor == player {
                    reports.push((death_message(End::You), true));
                    return true;
                }
                let seen = perception.sees_actors_at(at);
                if let (true, Some(dead)) = (seen, self.actor_or_fallen(actor)) {
                    let name = &content.creature(dead.creature).name;
                    let dies = content.status(status).dies.as_deref().unwrap_or("dies");
                    reports.push((format!("The {name} {dies}."), true));
                }
                seen
            }
            Event::Died { actor, .. } => match last {
                None => actor == player,
                Some(Last::Attack(_, _, _, by)) if actor == player => {
                    killer_seen = Some(by);
                    reports.push((death_message(End::You), true));
                    true
                }
                // A trap's victim: the death records the trap, not who was seen.
                Some(Last::Trap(..)) if actor == player => {
                    reports.push((death_message(End::You), true));
                    true
                }
                // A death by the player's hand was said with their blow, and recorded only if
                // they saw it.
                Some(Last::Attack(attacker, ..)) if attacker == player => true,
                Some(Last::Attack(_, target, seen, _) | Last::Trap(target, seen)) => {
                    let seen = seen && target == actor;
                    if seen {
                        reports.push((death_message(end(actor, true)), true));
                    }
                    seen
                }
            },
            Event::TrapSprung { trap, at, victim } => {
                last = None;
                let seen = sprung_seen.next().unwrap_or(false);
                if seen {
                    let def = content.trap(trap);
                    reports.extend(def.message.clone().map(|text| (text, true)));
                    let name = (self.actor_or_fallen(victim))
                        .map(|a| content.creature(a.creature).name.as_str());
                    let creature = name.filter(|_| perception.sees_actors_at(at));
                    let text = match (&def.effect, victim == player, creature) {
                        (TrapEffect::Pit { .. }, true, _) => Some("You fall into a pit.".into()),
                        (TrapEffect::Pit { .. }, false, Some(name)) => {
                            Some(format!("The {name} falls into a pit."))
                        }
                        (TrapEffect::Teleport, true, _) => {
                            Some("You are wrenched elsewhere.".into())
                        }
                        (TrapEffect::Teleport, false, Some(name)) => {
                            Some(format!("The {name} vanishes!"))
                        }
                        _ => None,
                    };
                    reports.extend(text.map(|text| (text, true)));
                }
                seen
            }
            Event::TrapStruck {
                trap,
                target,
                to,
                strike,
            } => {
                let seen = target == player || perception.sees_actors_at(to);
                last = Some(Last::Trap(target, seen));
                let def = content.trap(trap);
                // A fall is said with the trap ("You fall into a pit.").
                if seen && !matches!(def.effect, TrapEffect::Pit { .. }) {
                    let words = Words::new(&def.noun, &def.verb);
                    let hide = (self.actor_or_fallen(target))
                        .and_then(|t| content.creature(t.creature).natural.as_ref())
                        .map(|h| h.name.as_str());
                    let said = attack_message(
                        End::Trap(&def.noun),
                        end(target, true),
                        &words,
                        AttackKind::Melee,
                        Some(strike),
                        hide,
                    );
                    reports.extend(said.map(|text| (text, true)));
                }
                seen
            }
        });
        if let (Some(by), Some(death)) = (killer_seen, self.state.death.as_mut())
            && death.cause.is_none()
        {
            death.by = by;
        }
        match collapsed {
            0 => {}
            1 => reports.push(("The wall collapses!".into(), true)),
            _ => reports.push(("The walls collapse!".into(), true)),
        }
        reports
    }

    /// A message for each sound the player perceives that is news ([`noise::news`]): a patrol
    /// heard turn after turn is said once, not every turn. Messages stop a rest. A voice is said
    /// with its words, if any ([`World::say_mutters`]).
    fn hear(&mut self) {
        let turn = self.state.turn;
        let news = noise::news(
            &mut self.state.recent_sounds,
            self.perception.sounds(),
            turn,
        );
        let viewer = self.player_pos();
        let mut said: Vec<String> = Vec::new();
        for sound in news.iter().filter(|s| s.kind != noise::SoundKind::Voice) {
            let text = sound.message(viewer);
            if !said.contains(&text) {
                self.say(text.clone());
                said.push(text);
            }
        }
        // Hearing something new trains listening.
        if !news.is_empty() && self.trains(Use::Hear) {
            self.train(Use::Hear);
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

    /// The noise of `act` by `actor` on the current level (under its burden and its skills),
    /// heard by those it reaches. A step of the player's that a creature could have heard, but
    /// for their stealth, trains stealth.
    fn make_noise(&mut self, actor: ActorId, act: Act) {
        self.make_noise_at(actor, act, 100);
    }

    /// The noise of `actor` stepping onto `to` (it has just moved), `fleeing` or not
    /// ([`World::flees`]): a stance may quieten a flight.
    fn make_step_noise(&mut self, actor: ActorId, to: Pos, fleeing: bool) {
        let percent = match self.state.actors.get(&actor) {
            Some(a) if fleeing => self.content.stance_def(a.stance).flee_noise,
            _ => 100,
        };
        self.make_noise_at(actor, Act::Step(to), percent);
    }

    /// [`World::make_noise`] with the noise's loudness scaled to `percent`.
    fn make_noise_at(&mut self, actor: ActorId, act: Act, percent: u32) {
        let (state, content) = (&self.state, &self.content);
        let Some(a) = state.actors.get(&actor) else {
            return;
        };
        let burden = state.load(content, a).penalty();
        let made = noise::noise(content, state.current(), a, burden, act);
        let made = made.map(|mut n| {
            n.loudness = u16::try_from(u32::from(n.loudness) * percent / 100).unwrap_or(u16::MAX);
            n
        });
        let made = made.filter(|n| n.loudness > 0);
        let sneaking = actor == state.player
            && matches!(act, Act::Step(_))
            && self.trains(Use::Sneak)
            && noise::plain_noise(content, state.current(), a, burden, act)
                .is_some_and(|plain| noise::heard_by_anyone(state, content, plain));
        if let Some(made) = made {
            noise::make(&mut self.state, &self.content, made);
        }
        if sneaking {
            self.train(Use::Sneak);
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
            Action::Open(dir) => return self.open_door(here.step(*dir), events),
            Action::Kick(dir) => return self.kick(*dir, events),
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
            Action::PickUp(letters) => {
                // Priced at the load the player has as they begin.
                let price = self.price_of(self.state.player, ActionKind::PickUp);
                return if self.pick_up(*letters) {
                    Spent::Turn(price)
                } else {
                    Spent::Nothing
                };
            }
            Action::Drop(letters) => return self.drop_objects(*letters),
            Action::Equip { slot, item } => return self.equip(slot, *item),
            Action::Remove(slot) => return self.remove(slot),
            Action::Bash(dir) => return self.bash(*dir, events),
            Action::Fire(at) => return self.fire(*at, events),
            Action::Throw { item, at } => return self.throw(*item, *at, events),
            Action::Rest(goal) => return self.start_rest(*goal),
            Action::Chat(dir) => return self.chat(*dir, events),
            Action::Ask { dir, topic } => return self.ask(*dir, topic, events),
            Action::Eat(meal) => return self.eat(*meal),
            Action::Dig(dir) => return self.start_dig(*dir),
            Action::SetVision(id) => return self.set_vision(id),
            Action::SetStance(stance) => return self.set_stance(*stance),
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
            Action::Effect { effect, at, to } => {
                return self.show_effect(effect, *at, *to, events);
            }
            Action::Identify(letters) => return self.identify_dev(*letters),
            Action::Freeze { at, radius } => {
                let froze = self.frost(*at, *radius, crate::ice::FROST);
                self.say(format!(
                    "Frost seizes the water around {},{} ({froze} cells, dev).",
                    at.x, at.y
                ));
                self.frost_look(*at, *radius, events);
                return Spent::NoTime;
            }
            Action::Thaw { at, radius } => {
                self.say(format!(
                    "Heat melts the ice around {},{} (dev).",
                    at.x, at.y
                ));
                self.shock(*at, *radius, crate::ice::Shock::Heat, events);
                return Spent::NoTime;
            }
            Action::Afflict(id) => {
                let Some(status) = self.content.status_id(id) else {
                    self.say(format!("There is no status `{id}`."));
                    return Spent::Nothing;
                };
                if self.content.disease_status() == Some(status) {
                    self.say(format!("`{id}` comes of a disease: try `infect:DISEASE`."));
                    return Spent::Nothing;
                }
                let player = self.state.player;
                self.afflict(player, status, 1, None, events);
                return Spent::NoTime;
            }
            Action::Infect(id) => {
                let Some(disease) = self.content.disease_id(id) else {
                    self.say(format!("There is no disease `{id}`."));
                    return Spent::Nothing;
                };
                let player = self.state.player;
                let name = self.content.disease(disease).name.clone();
                if !self.infect(player, disease, None) {
                    self.say(format!("You carry {name} already (dev)."));
                    return Spent::Nothing;
                }
                self.say(format!("You catch {name} (dev)."));
                return Spent::NoTime;
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
        let Some(mut chosen) =
            self.chosen(letters, pile.len(), |c| format!("There is no {c} here."))
        else {
            return false;
        };
        let content = self.content.clone();
        // What burns stays where it lies.
        let burning: Vec<usize> = (chosen.iter())
            .filter(|&i| content.object(pile[i].object).burning.is_some())
            .collect();
        for &i in &burning {
            chosen.remove(i);
        }
        if chosen.is_empty() {
            let what = burning.first().map(|&i| pile[i].describe(&content));
            self.say(format!(
                "You cannot pick up {} while it burns.",
                what.unwrap_or_default()
            ));
            return false;
        }
        let inventory = self.state.inventory_mut();
        let mut taken = Letters::default();
        for i in chosen.iter() {
            if object::add_to_inventory(inventory, pile[i].clone(), &content) {
                taken.insert(i);
            }
        }
        if taken.is_empty() {
            self.say("You have no room for more.");
            return false;
        }
        // Everything was asked for, but the pile has more stacks than there are letters.
        let more = letters.is_empty() && pile.len() > taken.len() + burning.len();
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
    /// on the player's cell, taking off first what of them is worn or wielded (which takes its
    /// time too).
    fn drop_objects(&mut self, letters: Letters) -> Spent {
        let inventory = self.state.player().inventory.clone();
        if inventory.is_empty() {
            self.say("You are not carrying anything.");
            return Spent::Nothing;
        }
        let Some(chosen) = self.chosen(letters, inventory.len(), |c| format!("You carry no {c}."))
        else {
            return Spent::Nothing;
        };
        let mut kinds = self.take_off_before_dropping(chosen.iter());
        kinds.push(ActionKind::Drop);
        // Priced at the load the player has as they begin.
        let spent = self.price_for(&kinds);
        let (dropped, kept) = split(&inventory, chosen);
        *self.state.inventory_mut() = kept;
        self.player_mut().equipment.after_removal(chosen);
        let here = self.player_pos();
        let content = self.content.clone();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        object::add_to_pile(&mut pile, &dropped, &content);
        level.set_pile(here, pile);
        self.say(format!("You drop {}.", object::phrase(&dropped, &content)));
        spent
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

    /// The player takes combat stance `stance` (spec §8 "Stances"). Takes no time.
    fn set_stance(&mut self, stance: CombatStance) -> Spent {
        self.player_mut().stance = stance;
        self.say(format!("You take {} stance.", stance.phrase()));
        Spent::NoTime
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

    /// Changes terrain on the current level and invalidates light masks it may affect. A lock
    /// stays only on a closed door.
    fn set_terrain(&mut self, p: Pos, terrain: TerrainId) {
        let closed_door = self.content.terrain(terrain).is_closed_door();
        let level = self.state.current_mut();
        level.terrain.set(p, terrain);
        if !closed_door {
            level.locks.remove(&p);
        }
        let level = self.state.current();
        self.lights.terrain_changed(level, &self.content, p);
    }

    fn player_moves(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let from = self.player_pos();
        let to = from.step(dir);
        let level = self.state.current();
        let Some(&terrain) = level.terrain.get(to) else {
            if let Some(exit) = level.exit.clone() {
                if !self.can_move() {
                    return Spent::Nothing;
                }
                return self.leave_by_edge(exit, events);
            }
            self.say("You can't go that way.");
            return Spent::Nothing;
        };
        let def = self.content.terrain(terrain);
        if def.is_closed_door() {
            return self.open_door(to, events);
        }
        if !def.walkable {
            let name = def.name.clone();
            self.say(format!("The {name} blocks your way."));
            return Spent::Nothing;
        }
        // Someone there: the player attacks them if they are hostile (by their stance toward the
        // player, which the player knows from what they are), else swaps places with them
        // unless they will not make way. Only what the player sees of them is said.
        let mut swap = None;
        if let Some(other) = self.state.actor_at(&level.id, to) {
            let target = other.id;
            let seen = self.perception.sees_actors_at(to);
            let player = self.state.player;
            if self.stance(player, target) == Stance::Hostile {
                let skill = self.state.player().melee_skill(&self.content);
                self.player_attacks(target, seen, events);
                // Priced before the exertion, which may burn enough to leave the player
                // hungrier (no longer Bloated).
                let price = self.attack_price(player, ActionKind::Melee, skill);
                self.exert(ActionKind::Melee);
                return Spent::Turn(price);
            }
            if let Some(why) = self.refusal(target) {
                self.refused(target, why, seen);
                return Spent::Nothing;
            }
            swap = Some((target, seen));
        }
        if !self.can_move() {
            return Spent::Nothing;
        }
        let player = self.state.player;
        let fleeing = self.flees(player, to);
        // Ice there may give way (then it is a step into the water); climbing out onto it from
        // the water takes longer.
        let footing = self.tread(player, from, to);
        let onto = self.state.current().terrain[to];
        let step = self.step_price(player, self.content.terrain(onto), fleeing)
            + self.tread_price(player, footing);
        let overworld = self.state.current().kind == LevelKind::Overworld;
        self.state.overworld_step = overworld.then_some(dir);
        if let Some(actor) = self.state.actors.get_mut(&player) {
            actor.pos = to;
        }
        events.push(Event::ActorMoved {
            actor: player,
            from,
            to,
        });
        self.after_tread(player, to, footing, events);
        self.make_step_noise(player, to, fleeing);
        // Wading through a marsh may give a disease, by how long the step takes.
        let speed = self.content.creature(self.state.player().creature).speed;
        self.wade(
            onto,
            u64::try_from(step).unwrap_or(0) / u64::from(speed.max(1)),
        );
        if let Some((other, seen)) = swap {
            self.swap_places(other, to, from, seen, events);
        }
        // A journey on the overworld trains survival.
        if overworld && self.trains(Use::Travel) {
            self.train(Use::Travel);
        }
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
        // A trap there goes off (spec §8 "Traps"); a pit costs the next action too.
        let climb = self.spring_trap(to, player, events);
        Spent::Turn(step + climb)
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
        if !self.can_move() {
            return false;
        }
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
        if !self.can_move() {
            return false;
        }
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
        self.reach(to);
        // Sounds remembered on the level left say nothing about this one.
        self.state.recent_sounds.clear();
        if let Some(line) = self.content.level(to).and_then(|t| t.arrival.clone()) {
            self.say(line);
        }
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

/// The stacks of `stacks` that `chosen` names, and the others, each in order.
fn split(stacks: &[Stack], chosen: Letters) -> (Vec<Stack>, Vec<Stack>) {
    let (picked, rest): (Vec<_>, Vec<_>) = stacks
        .iter()
        .enumerate()
        .partition(|(i, _)| chosen.contains(*i));
    let unzip = |v: Vec<(usize, &Stack)>| v.into_iter().map(|(_, s)| s.clone()).collect();
    (unzip(picked), unzip(rest))
}

/// The first time the player is inside a special room (spec §7 "Special rooms"): marks it
/// entered and returns what they are told; from then on they know it (look mode names it).
fn enter_room(state: &mut WorldState, content: &Content) -> Option<String> {
    let here = state.player().pos;
    let room = (state.current_mut().rooms.iter_mut()).find(|r| r.contains(here))?;
    if room.entered {
        return None;
    }
    room.entered = true;
    Some(content.room_effect(room.effect).enter.clone())
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

/// Creates the runtime state for a level and spawns its creatures, each with its type's stats,
/// carrying, wearing, and wielding what the level gives it.
fn instantiate_level(state: &mut WorldState, content: &Content, template: &LevelTemplate) {
    let id = template.id.as_str();
    let level = LevelState::from_template(template, content, state.time);
    state.levels.insert(id.to_string(), level);
    // Under the open sky its water has frozen and thawed with the weather of the last weeks.
    state.pre_roll_ice(content, id);
    for spawn in &template.creatures {
        let actor_id = ActorId(state.next_actor_id);
        state.next_actor_id += 1;
        let def = content.creature(spawn.creature);
        let mut actor = Actor {
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
            equipment: spawn.equipment.clone(),
            stats: def.stats,
            hp: 0,
            hp_base: def.base_hp,
            regen: 0,
            skills: def.skills.clone(),
            effects: Default::default(),
            diseases: Default::default(),
            damage: Default::default(),
            weapon_skills: def.weapon_skills.clone(),
            faction_override: spawn.faction,
            npc: spawn.npc.clone(),
            talk: Talk::default(),
            satiation: None,
            stance: CombatStance::Neutral,
        };
        actor.hp = actor.max_hp(content);
        state.actors.insert(actor_id, actor);
    }
}

#[cfg(test)]
mod ai_tests;
#[cfg(test)]
mod blood_tests;
#[cfg(test)]
mod branch_tests;
#[cfg(test)]
mod burden_tests;
#[cfg(test)]
mod character_tests;
#[cfg(test)]
mod cost_tests;
#[cfg(test)]
mod dig_tests;
#[cfg(test)]
mod disease_tests;
#[cfg(test)]
mod door_tests;
#[cfg(test)]
mod equip_tests;
#[cfg(test)]
mod experience_tests;
#[cfg(test)]
mod fight_tests;
#[cfg(test)]
mod hearing_tests;
#[cfg(test)]
mod hunger_tests;
#[cfg(test)]
mod ice_tests;
#[cfg(test)]
mod light_tests;
#[cfg(test)]
mod magic_tests;
#[cfg(test)]
mod moon_tests;
#[cfg(test)]
mod object_tests;
#[cfg(test)]
mod overworld_tests;
#[cfg(test)]
mod quest_tests;
#[cfg(test)]
mod room_tests;
#[cfg(test)]
mod shoot_tests;
#[cfg(test)]
mod skills_tests;
#[cfg(test)]
mod speed_tests;
#[cfg(test)]
mod stats_tests;
#[cfg(test)]
mod status_tests;
#[cfg(test)]
mod sun_tests;
#[cfg(test)]
mod talk_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod time_tests;
#[cfg(test)]
mod trap_tests;
#[cfg(test)]
mod weapon_skill_tests;
#[cfg(test)]
mod weather_tests;
