//! The running game: owns the simulation and applies player actions.

use crate::cli::Args;
use bevy::prelude::*;
use std::sync::Arc;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::{Action, Outcome, World};

/// The simulation as an ECS resource.
#[derive(Resource)]
pub struct Game {
    pub world: World,
    /// Events from the most recent turn, for presentation.
    pub last_outcome: Outcome,
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
        let world = World::new(Arc::new(content), &args.scenario, args.seed)
            .map_err(|e| format!("cannot start scenario `{}`: {e}", args.scenario))?;
        Ok(Self {
            world,
            last_outcome: Outcome::default(),
        })
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
        let outcome = game.world.apply(action);
        game.last_outcome = outcome;
    }
}
