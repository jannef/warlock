//! Replays: a scenario, a seed, and the actions taken, with the state hash after every step
//! (spec §9). Replaying verifies the hashes and reports the first step that diverges.
//!
//! JSON shape (hashes are 16-digit hex strings, actions use the text syntax of [`Action`]):
//!
//! ```json
//! { "game_version": "0.1.0", "scenario": "showcase", "seed": 1,
//!   "steps": [ { "action": "move:N", "hash": "0123456789abcdef" } ] }
//! ```

use crate::action::Action;
use crate::content::Content;
use crate::world::{World, WorldError};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::sync::Arc;

/// Why a replay could not be loaded or did not reproduce.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplayError {
    #[error("invalid replay: {0}")]
    Parse(String),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error("{0}")]
    Diverged(Divergence),
}

/// The first step whose resulting state hash differs from the recorded one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence {
    /// Zero-based index into [`Replay::steps`].
    pub step: usize,
    pub action: Action,
    pub expected: u64,
    pub actual: u64,
}

impl fmt::Display for Divergence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "replay diverged at step {} (`{}`): expected state hash {:016x}, got {:016x}",
            self.step, self.action, self.expected, self.actual
        )
    }
}

/// One applied action and the state hash after it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    #[serde(with = "action_text")]
    pub action: Action,
    #[serde(with = "hash_hex")]
    pub hash: u64,
}

/// A recorded run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub game_version: String,
    pub scenario: String,
    pub seed: u64,
    pub steps: Vec<Step>,
}

impl Replay {
    /// An empty replay starting from the scenario and seed of `world` (which should be fresh).
    pub fn new(world: &World) -> Self {
        Self {
            game_version: crate::GAME_VERSION.to_string(),
            scenario: world.scenario().to_string(),
            seed: world.seed(),
            steps: Vec::new(),
        }
    }

    /// Records `action`, which has just been applied to `world`.
    pub fn push(&mut self, action: &Action, world: &World) {
        self.steps.push(Step {
            action: action.clone(),
            hash: world.state_hash(),
        });
    }

    /// Parses a replay from JSON.
    pub fn from_json(text: &str) -> Result<Self, ReplayError> {
        serde_json::from_str(text).map_err(|e| ReplayError::Parse(e.to_string()))
    }

    /// Pretty-printed JSON with a trailing newline.
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("replay serializes to JSON");
        text.push('\n');
        text
    }

    /// A fresh world at the start of this replay.
    pub fn start(&self, content: Arc<Content>) -> Result<World, WorldError> {
        World::new(content, &self.scenario, Some(self.seed))
    }

    /// Plays every step, returning the final world and the first divergence, if any. Playing
    /// continues past a divergence so the state can still be inspected.
    pub fn play(&self, content: Arc<Content>) -> Result<(World, Option<Divergence>), WorldError> {
        let mut world = self.start(content)?;
        let mut divergence = None;
        for (step, recorded) in self.steps.iter().enumerate() {
            world.apply(&recorded.action);
            let actual = world.state_hash();
            if actual != recorded.hash && divergence.is_none() {
                divergence = Some(Divergence {
                    step,
                    action: recorded.action.clone(),
                    expected: recorded.hash,
                    actual,
                });
            }
        }
        Ok((world, divergence))
    }

    /// Plays the replay and verifies every hash.
    pub fn run(&self, content: Arc<Content>) -> Result<World, ReplayError> {
        match self.play(content)? {
            (world, None) => Ok(world),
            (_, Some(divergence)) => Err(ReplayError::Diverged(divergence)),
        }
    }
}

/// Serde adapter: actions as their text syntax.
mod action_text {
    use super::*;

    pub fn serialize<S: Serializer>(action: &Action, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(action)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Action, D::Error> {
        let text = String::deserialize(d)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Serde adapter: hashes as 16-digit hex (JSON numbers lose precision above 2^53 in many tools).
mod hash_hex {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&format_args!("{hash:016x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        let text = String::deserialize(d)?;
        u64::from_str_radix(&text, 16).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::parse_actions;

    fn content() -> Arc<Content> {
        Arc::new(Content::builtin().expect("builtin content is valid"))
    }

    fn recorded(actions: &str) -> Replay {
        let mut world = World::new(content(), "showcase", None).expect("world");
        let mut replay = Replay::new(&world);
        for action in parse_actions(actions).expect("actions") {
            world.apply(&action);
            replay.push(&action, &world);
        }
        replay
    }

    #[test]
    fn replay_reproduces_and_round_trips_through_json() {
        let replay = recorded("move:N move:N wait move:E");
        let parsed = Replay::from_json(&replay.to_json()).expect("parse");
        assert_eq!(parsed, replay);
        let world = parsed.run(content()).expect("verifies");
        assert_eq!(world.state_hash(), replay.steps[3].hash);
    }

    #[test]
    fn json_shape_is_documented_shape() {
        let json: serde_json::Value =
            serde_json::from_str(&recorded("wait").to_json()).expect("json");
        assert_eq!(json["scenario"], "showcase");
        assert_eq!(json["steps"][0]["action"], "wait");
        assert_eq!(json["steps"][0]["hash"].as_str().map(str::len), Some(16));
    }

    #[test]
    fn reports_the_first_diverging_step() {
        let mut replay = recorded("wait wait wait");
        replay.steps[1].hash ^= 1;
        replay.steps[2].hash ^= 1;
        match replay.run(content()) {
            Err(ReplayError::Diverged(d)) => {
                assert_eq!(d.step, 1);
                assert_eq!(d.action, Action::Wait);
            }
            other => panic!("expected divergence, got {other:?}"),
        }
    }

    #[test]
    fn unknown_scenario_and_bad_json_are_errors() {
        let mut replay = recorded("wait");
        replay.scenario = "nope".into();
        assert!(matches!(replay.run(content()), Err(ReplayError::World(_))));
        assert!(matches!(
            Replay::from_json("{ \"scenario\": 1 }"),
            Err(ReplayError::Parse(_))
        ));
        let bad_action = r#"{"game_version":"x","scenario":"showcase","seed":1,
            "steps":[{"action":"dance","hash":"00"}]}"#;
        assert!(Replay::from_json(bad_action).is_err());
    }
}
