//! Player actions and the events a turn produces.

use crate::geom::{Dir, Pos};
use crate::time::{DAY, Duration};
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
    /// Wait turn after turn (creatures act) until the goal, stopping early when a creature comes
    /// into view or a message is logged.
    Rest(Rest),
    /// Dev command: switch the player's vision profile (an id in `data/vision.toml`). Takes no
    /// game time, but changes state and perception, so replays record it like any action.
    SetVision(String),
    /// Dev command: move the clock ahead at once; nothing else acts.
    SkipTime(Duration),
}

/// How long to rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rest {
    /// For this long (at most a day).
    For(Duration),
    /// Until the next dawn, 06:00.
    Dawn,
    /// Until the next dusk, 18:00.
    Dusk,
}

/// The longest rest; resting until dawn or dusk is never longer either.
pub const MAX_REST: Duration = Duration::seconds(DAY);

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
    /// Whether at least one turn passed (creatures acted). Refused actions (walking into a wall)
    /// and free actions take none.
    pub took_turn: bool,
    /// Game time that passed: a step, a rest, a skip; zero for refused and free actions.
    pub elapsed: Duration,
    /// Perceived events, in the order they happened.
    pub events: Vec<Event>,
}

/// Text syntax for scripting: `move:E wait open:E close:W ascend descend lantern rest:1h
/// rest:dawn rest:dusk vision:ID skip:1h` (durations: `30s`, `10m`, `2h`).
///
/// `Display` and `FromStr` round-trip; action names are case-insensitive when parsing (vision
/// profile ids keep their case).
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Move(d) => write!(f, "move:{d}"),
            Action::Wait => f.write_str("wait"),
            Action::Open(d) => write!(f, "open:{d}"),
            Action::Close(d) => write!(f, "close:{d}"),
            Action::Ascend => f.write_str("ascend"),
            Action::Descend => f.write_str("descend"),
            Action::ToggleLantern => f.write_str("lantern"),
            Action::Rest(Rest::For(d)) => write!(f, "rest:{d}"),
            Action::Rest(Rest::Dawn) => f.write_str("rest:dawn"),
            Action::Rest(Rest::Dusk) => f.write_str("rest:dusk"),
            Action::SetVision(id) => write!(f, "vision:{id}"),
            Action::SkipTime(d) => write!(f, "skip:{d}"),
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
            "lantern" => bare(Action::ToggleLantern),
            "rest" => match arg {
                Some("dawn") => Ok(Action::Rest(Rest::Dawn)),
                Some("dusk") => Ok(Action::Rest(Rest::Dusk)),
                Some(d) => match d.parse::<Duration>()? {
                    d if d.is_zero() || d > MAX_REST => {
                        Err(format!("`rest:{d}`: rest from 1s up to 24h"))
                    }
                    d => Ok(Action::Rest(Rest::For(d))),
                },
                None => Err("`rest` needs a goal: `rest:1h`, `rest:dawn`, or `rest:dusk`".into()),
            },
            "skip" => match arg.map(str::parse::<Duration>).transpose()? {
                Some(d) if !d.is_zero() => Ok(Action::SkipTime(d)),
                _ => Err("`skip` needs a positive duration, e.g. `skip:1h`".into()),
            },
            "vision" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() => Ok(Action::SetVision(id.to_string())),
                _ => Err("`vision` needs a profile id, e.g. `vision:darkvision`".into()),
            },
            _ => Err(format!(
                "unknown action `{s}` (expected move:DIR, wait, open:DIR, close:DIR, ascend, \
                 descend, lantern, rest:1h|dawn|dusk, vision:ID, skip:DURATION)"
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
        let mut all = vec![
            Action::Wait,
            Action::Ascend,
            Action::Descend,
            Action::ToggleLantern,
            Action::SetVision("low_light".into()),
            Action::SetVision("Mixed_Case".into()),
            Action::Rest(Rest::For(Duration::hours(1))),
            Action::Rest(Rest::For(Duration::minutes(90))),
            Action::Rest(Rest::Dawn),
            Action::Rest(Rest::Dusk),
            Action::SkipTime(Duration::hours(1)),
            Action::SkipTime(Duration::seconds(30)),
        ];
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
        assert_eq!(
            parse_actions("LANTERN vision:uv_sight"),
            Ok(vec![
                Action::ToggleLantern,
                Action::SetVision("uv_sight".into())
            ])
        );
        assert!(parse_actions("vision").is_err());
        assert!(parse_actions("vision:").is_err());
        assert!(parse_actions("lantern:E").is_err());
        assert_eq!(
            parse_actions("REST:Dawn rest:1h skip:2H"),
            Ok(vec![
                Action::Rest(Rest::Dawn),
                Action::Rest(Rest::For(Duration::hours(1))),
                Action::SkipTime(Duration::hours(2)),
            ])
        );
        for bad in [
            "rest",
            "rest:noon",
            "rest:0m",
            "rest:25h",
            "skip",
            "skip:0s",
            "skip:x",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
    }
}
