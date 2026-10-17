//! Player actions and the events a turn produces.

use crate::geom::{Dir, Pos};
use crate::world::ActorId;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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
    /// Light or douse the lantern the scenario gives the player. Takes a turn.
    ToggleLantern,
    /// Dev command: switch the player's vision profile (an id in `data/vision.toml`). Takes no
    /// game time, but changes state and perception, so replays record it like any action.
    SetVision(String),
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

/// Text syntax for scripting: `move:E wait open:E close:W ascend descend`.
///
/// `Display` and `FromStr` round-trip; names are case-insensitive when parsing.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Move(d) => write!(f, "move:{d}"),
            Action::Wait => f.write_str("wait"),
            Action::Open(d) => write!(f, "open:{d}"),
            Action::Close(d) => write!(f, "close:{d}"),
            Action::Ascend => f.write_str("ascend"),
            Action::Descend => f.write_str("descend"),
        }
    }
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        let (verb, arg) = match lower.split_once(':') {
            Some((verb, arg)) => (verb, Some(arg)),
            None => (lower.as_str(), None),
        };
        let dir = || -> Result<Dir, String> {
            arg.ok_or_else(|| format!("`{verb}` needs a direction, e.g. `{verb}:E`"))?
                .parse()
        };
        let bare = |action: Action| match arg {
            None => Ok(action),
            Some(_) => Err(format!("`{verb}` takes no direction")),
        };
        match verb {
            "move" => dir().map(Action::Move),
            "open" => dir().map(Action::Open),
            "close" => dir().map(Action::Close),
            "wait" => bare(Action::Wait),
            "ascend" => bare(Action::Ascend),
            "descend" => bare(Action::Descend),
            _ => Err(format!(
                "unknown action `{s}` (expected move:DIR, wait, open:DIR, close:DIR, ascend, descend)"
            )),
        }
    }
}

/// Parses whitespace-separated actions, e.g. `"move:E move:SE wait"`.
pub fn parse_actions(text: &str) -> Result<Vec<Action>, String> {
    text.split_whitespace().map(str::parse).collect()
}

/// Formats actions as whitespace-separated text; inverse of [`parse_actions`].
pub fn format_actions(actions: &[Action]) -> String {
    let words: Vec<String> = actions.iter().map(ToString::to_string).collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trips() {
        let mut all = vec![Action::Wait, Action::Ascend, Action::Descend];
        for d in Dir::ALL {
            all.extend([Action::Move(d), Action::Open(d), Action::Close(d)]);
        }
        assert_eq!(parse_actions(&format_actions(&all)), Ok(all));
    }

    #[test]
    fn parses_case_insensitively_and_rejects_garbage() {
        assert_eq!(
            parse_actions("MOVE:ne  Wait"),
            Ok(vec![Action::Move(Dir::NE), Action::Wait])
        );
        assert_eq!(parse_actions(""), Ok(vec![]));
        assert!(parse_actions("move").is_err());
        assert!(parse_actions("move:up").is_err());
        assert!(parse_actions("wait:E").is_err());
        assert!(parse_actions("dance").is_err());
    }
}
