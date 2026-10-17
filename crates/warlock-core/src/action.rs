//! Player actions and the events a turn produces.

use crate::geom::{Dir, Pos};
use crate::world::ActorId;
use serde::{Deserialize, Serialize};

/// Something the player asks to do. Actions are the unit of replays (spec §9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Step in a direction; bumping a closed door opens it.
    Move(Dir),
    /// Let a turn pass.
    Wait,
    Open(Dir),
    Close(Dir),
    /// Take stairs up.
    Ascend,
    /// Take stairs down.
    Descend,
}

/// Something the player perceived happening during a turn. Presentation turns these into
/// animations; they never carry information about things the player could not perceive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    ActorMoved { actor: ActorId, from: Pos, to: Pos },
    DoorOpened { pos: Pos },
    DoorClosed { pos: Pos },
    LevelChanged { from: String, to: String },
}

/// The result of applying an action.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Whether game time advanced. Refused actions (walking into a wall) take no time.
    pub took_turn: bool,
    /// Perceived events, in the order they happened.
    pub events: Vec<Event>,
}
