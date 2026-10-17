//! The running game: owns the simulation and applies player actions.

use crate::cli::Args;
use crate::save::{Resume, SaveStore};
use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::replay::Replay;
use warlock_core::{Action, Outcome, World};

/// The simulation as an ECS resource.
#[derive(Resource)]
pub struct Game {
    pub world: World,
    /// Events from the most recent turn, for presentation.
    pub last_outcome: Outcome,
    /// Present with `--record`: the run so far, saved after every action.
    recorder: Option<(Replay, PathBuf)>,
}

impl Game {
    /// Loads content and starts the scenario named on the command line.
    pub fn start(args: &Args) -> Result<Self, String> {
        let files = match &args.content_dir {
            Some(dir) => ContentFiles::from_dir(dir)
                .map_err(|e| format!("cannot read content from {}: {e}", dir.display()))?,
            None => ContentFiles::builtin(),
        };
        let content = Content::load(&files).map_err(|e| format!("invalid content: {e}"))?;
        let content = Arc::new(content);
        let mut failure = None;
        if SaveStore::resumes(args) {
            match SaveStore::from_args(args).resume(&content) {
                Resume::Loaded(mut world) => {
                    world.notice("Welcome back.");
                    return Ok(Self {
                        world,
                        last_outcome: Outcome::default(),
                        recorder: None,
                    });
                }
                Resume::Failed(message) => failure = Some(message),
                Resume::NoSave => {}
            }
        }
        let (world, replay) = match &args.replay {
            Some(path) => {
                let text = std::fs::read_to_string(path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                let replay =
                    Replay::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))?;
                let (world, divergence) = replay
                    .play(content)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                if let Some(divergence) = divergence {
                    eprintln!("warlock: warning: {}: {divergence}", path.display());
                }
                (world, Some(replay))
            }
            None => {
                let mut world = World::new(content, &args.scenario, args.seed)
                    .map_err(|e| format!("cannot start scenario `{}`: {e}", args.scenario))?;
                if let Some(message) = failure {
                    world.notice(message);
                }
                let replay = args.record.is_some().then(|| Replay::new(&world));
                (world, replay)
            }
        };
        let recorder = args.record.clone().zip(replay).map(|(path, r)| (r, path));
        let game = Self {
            world,
            last_outcome: Outcome::default(),
            recorder,
        };
        game.save_recording();
        Ok(game)
    }

    /// Applies `action`, recording it if `--record` is on.
    pub fn apply(&mut self, action: &Action) {
        self.last_outcome = self.world.apply(action);
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

/// A gameplay action requested by input (or a replay).
#[derive(Message, Debug, Clone)]
pub struct PlayerAction(pub Action);

/// Systems that advance the simulation. Input runs before it, presentation after.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimSet;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerAction>()
            .add_systems(Update, apply_actions.in_set(SimSet));
    }
}

fn apply_actions(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
    for PlayerAction(action) in actions.read() {
        game.apply(action);
    }
}
