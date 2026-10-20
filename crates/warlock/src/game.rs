//! The running game: owns the simulation and applies player actions.

use crate::cli::Args;
use crate::save::{Resume, SaveStore};
use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use warlock_core::character::CharacterSpec;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::replay::{Divergence, Replay};
use warlock_core::world::WorldError;
use warlock_core::{Action, Outcome, World};

/// Loaded content, shared by the menu (scenario list) and every run.
#[derive(Resource, Debug, Clone)]
pub struct GameContent(pub Arc<Content>);

/// Loads the content named by the command line (the built-in copy unless `--content-dir`).
pub fn load_content(args: &Args) -> Result<Arc<Content>, String> {
    let files = match &args.content_dir {
        Some(dir) => ContentFiles::from_dir(dir)
            .map_err(|e| format!("cannot read content from {}: {e}", dir.display()))?,
        None => ContentFiles::builtin(),
    };
    Content::load(&files)
        .map(Arc::new)
        .map_err(|e| format!("invalid content: {e}"))
}

/// The simulation as an ECS resource. It exists only while a run is in progress: the main menu
/// runs without it, and everything that needs a run is gated on `resource_exists::<Game>`.
#[derive(Resource)]
pub struct Game {
    pub world: World,
    /// Events from the most recent turn, for presentation.
    pub last_outcome: Outcome,
    /// How many actions this session applied: presentation that must read each outcome once,
    /// even one that took no turn (a dev spell effect), watches it.
    pub applied: u64,
    /// The character's save id (`save::SaveStore`): every way of saving writes its files.
    pub save_id: String,
    /// Present with `--record`: the run so far, saved after every action.
    recorder: Option<(Replay, PathBuf)>,
}

impl Game {
    /// A fresh run of `scenario` as `character` (the scenario's if `None`) named `name` (the
    /// run's default if `None`), e.g. from the main menu, with a new save id from `store`: it
    /// never touches another character's save.
    pub fn new_run(
        content: Arc<Content>,
        store: &SaveStore,
        scenario: &str,
        seed: Option<u64>,
        character: Option<&CharacterSpec>,
        name: Option<&str>,
    ) -> Result<Self, String> {
        let world = World::with_character_named(content, scenario, seed, character, name)
            .map_err(|e| format!("cannot start scenario `{scenario}`: {e}"))?;
        Ok(Self::new_character(world, store))
    }

    /// A game around a world loaded elsewhere (a saved character's run), saved as `save_id`.
    pub fn from_world(world: World, save_id: String) -> Self {
        Self {
            world,
            last_outcome: Outcome::default(),
            applied: 0,
            save_id,
            recorder: None,
        }
    }

    /// A game around a fresh run, with a new save id from `store`.
    fn new_character(world: World, store: &SaveStore) -> Self {
        let name = world.character().map_or("", |c| c.name.as_str());
        let id = store.new_id(name);
        Self::from_world(world, id)
    }

    /// Character `id`'s saved run, if it is usable.
    pub fn resumed(store: &SaveStore, id: &str, content: &Arc<Content>) -> Resume {
        match store.resume(id, content) {
            Resume::Loaded(mut world) => {
                info!("resumed {id} at turn {}", world.turn());
                world.notice("Welcome back.");
                Resume::Loaded(world)
            }
            other => other,
        }
    }

    /// Starts the game the command line asks for: the character played last, a replay, or the
    /// scenario (as `--character` named `--name`, else the scenario's). Only used when the main
    /// menu is skipped.
    pub fn start(args: &Args, content: Arc<Content>) -> Result<Self, String> {
        let mut failure = None;
        let store = SaveStore::from_args(args);
        if SaveStore::resumes(args)
            && let Some(last) = store.last_played()
        {
            match Self::resumed(&store, &last.id, &content) {
                Resume::Loaded(world) => return Ok(Self::from_world(world, last.id)),
                Resume::Failed(message) => failure = Some(message),
                Resume::NoSave => {}
            }
        }
        let mut last_outcome = Outcome::default();
        let (world, replay) = match &args.replay {
            Some(path) => {
                let text = std::fs::read_to_string(path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                let replay =
                    Replay::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))?;
                let (world, divergence, outcome) = play_replay(&replay, content)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                if let Some(divergence) = divergence {
                    eprintln!("warlock: warning: {}: {divergence}", path.display());
                }
                last_outcome = outcome;
                (world, Some(replay))
            }
            None => {
                let scenario = args.scenario_name();
                let (character, name) = (args.character.as_ref(), args.name.as_deref());
                let mut world =
                    World::with_character_named(content, scenario, args.seed, character, name)
                        .map_err(|e| format!("cannot start scenario `{scenario}`: {e}"))?;
                if let Some(message) = failure {
                    world.notice(message);
                }
                let replay = args.record.is_some().then(|| Replay::new(&world));
                (world, replay)
            }
        };
        let recorder = args.record.clone().zip(replay).map(|(path, r)| (r, path));
        let game = Self {
            last_outcome,
            recorder,
            ..Self::new_character(world, &store)
        };
        game.save_recording();
        Ok(game)
    }

    /// Applies `action`, recording it if `--record` is on.
    pub fn apply(&mut self, action: &Action) {
        self.last_outcome = self.world.apply(action);
        self.applied += 1;
        if let Some((replay, _)) = &mut self.recorder {
            replay.push(action, &self.world);
        }
        self.save_recording();
    }

    /// Rewrites the `--record` file; a failure is reported but never stops the game.
    fn save_recording(&self) {
        if let Some((replay, path)) = &self.recorder
            && let Err(err) = std::fs::write(path, replay.to_json())
        {
            eprintln!("warlock: cannot write recording {}: {err}", path.display());
        }
    }
}

/// Plays `replay` like [`Replay::play`], also returning the outcome of its last step, so
/// presentation shows that turn's events (e.g. a splash in a `--capture`).
fn play_replay(
    replay: &Replay,
    content: Arc<Content>,
) -> Result<(World, Option<Divergence>, Outcome), WorldError> {
    let mut head = replay.clone();
    let Some(last) = head.steps.pop() else {
        let (world, divergence) = replay.play(content)?;
        return Ok((world, divergence, Outcome::default()));
    };
    let (mut world, mut divergence) = head.play(content)?;
    let outcome = world.apply(&last.action);
    let actual = world.state_hash();
    if divergence.is_none() && actual != last.hash {
        divergence = Some(Divergence {
            step: head.steps.len(),
            action: last.action,
            expected: last.hash,
            actual,
        });
    }
    Ok((world, divergence, outcome))
}

/// A gameplay action requested by input (or a replay).
#[derive(Message, Debug, Clone)]
pub struct PlayerAction(pub Action);

/// Systems that advance the simulation. Input runs before it, presentation after.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimSet;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerAction>().add_systems(
            Update,
            apply_actions.in_set(SimSet).run_if(resource_exists::<Game>),
        );
    }
}

fn apply_actions(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
    for PlayerAction(action) in actions.read() {
        game.apply(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::{Dir, Event};

    #[test]
    fn replays_keep_the_last_steps_events() {
        let content = Arc::new(Content::builtin().expect("built-in content loads"));
        let mut world = World::new(content.clone(), "showcase", None).expect("scenario starts");
        let mut replay = Replay::new(&world);
        for action in [Action::Move(Dir::N), Action::Move(Dir::N)] {
            world.apply(&action);
            replay.push(&action, &world);
        }
        let (played, divergence, outcome) =
            play_replay(&replay, content.clone()).expect("replay plays");
        assert_eq!(divergence, None);
        assert_eq!(played.state_hash(), world.state_hash());
        assert!(matches!(
            outcome.events.as_slice(),
            [Event::ActorMoved { .. }, ..]
        ));

        replay.steps.last_mut().expect("two steps").hash ^= 1;
        let (_, divergence, _) = play_replay(&replay, content.clone()).expect("replay plays");
        assert_eq!(divergence.map(|d| d.step), Some(1));
        replay.steps.clear();
        let (_, _, outcome) = play_replay(&replay, content).expect("empty replay plays");
        assert_eq!(outcome, Outcome::default());
    }
}
