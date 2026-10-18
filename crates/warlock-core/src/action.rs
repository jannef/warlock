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
    /// Pick up the stacks with these letters from the pile on the player's cell (`a` is the top
    /// of the pile), or everything there with no letters. Takes the level's turn.
    PickUp(Letters),
    /// Drop the inventory stacks with these letters onto the player's cell, or everything
    /// carried with no letters. Takes the level's turn.
    Drop(Letters),
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

/// A set of letters `a` to `z` naming stacks of a list the player sees (a pile, the inventory)
/// by position: `a` is the first. Written as the letters in order, e.g. `ac`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Letters(u32);

impl Letters {
    /// How many letters there are.
    pub const COUNT: usize = 26;

    /// The letters of these positions; positions past `z` are left out.
    pub fn of(positions: impl IntoIterator<Item = usize>) -> Self {
        let mut letters = Self::default();
        for i in positions {
            letters.insert(i);
        }
        letters
    }

    pub fn insert(&mut self, i: usize) {
        if i < Self::COUNT {
            self.0 |= 1 << i;
        }
    }

    pub fn remove(&mut self, i: usize) {
        if i < Self::COUNT {
            self.0 &= !(1 << i);
        }
    }

    pub fn contains(self, i: usize) -> bool {
        i < Self::COUNT && self.0 & (1 << i) != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// The positions, in order.
    pub fn iter(self) -> impl Iterator<Item = usize> {
        (0..Self::COUNT).filter(move |i| self.contains(*i))
    }
}

/// The letter of list position `i`: `a` for 0. Positions past `z` have none.
pub fn letter(i: usize) -> Option<char> {
    (i < Letters::COUNT).then(|| char::from(b'a' + i as u8))
}

impl fmt::Display for Letters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter()
            .filter_map(letter)
            .try_for_each(|c| write!(f, "{c}"))
    }
}

impl FromStr for Letters {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut letters = Letters::default();
        for c in s.chars() {
            if !c.is_ascii_lowercase() {
                return Err(format!("`{c}` is not a letter from a to z"));
            }
            letters.insert(usize::from(c as u8 - b'a'));
        }
        Ok(letters)
    }
}

/// Something the player perceived happening during a turn. Presentation turns these into
/// animations; they never carry information about things the player could not perceive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    ActorMoved {
        actor: ActorId,
        from: Pos,
        to: Pos,
    },
    DoorOpened {
        pos: Pos,
    },
    DoorClosed {
        pos: Pos,
    },
    LevelChanged {
        from: String,
        to: String,
    },
    /// An actor attacked another: an intent only, nothing is damaged (spec §8 "Creature
    /// behavior"). `from` and `to` are where they stood at that moment. Kept only when the
    /// player saw both of them (or is one of them); the message line reports more (an attack on
    /// the player by something unseen, an attack the player saw only one end of).
    Attacked {
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        from: Pos,
        to: Pos,
    },
}

/// How an attack is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackKind {
    /// From the next cell.
    Melee,
    /// A shot from a distance.
    Ranged,
    /// A spell cast from a distance.
    Spell,
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
/// rest:dawn rest:dusk pickup pickup:ac drop:b vision:ID skip:1h` (durations: `30s`, `10m`,
/// `2h`; letters name stacks, `a` first; `pickup` and `drop` without letters take everything).
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
            Action::PickUp(l) if l.is_empty() => f.write_str("pickup"),
            Action::PickUp(l) => write!(f, "pickup:{l}"),
            Action::Drop(l) if l.is_empty() => f.write_str("drop"),
            Action::Drop(l) => write!(f, "drop:{l}"),
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
            "pickup" | "drop" => {
                let letters = match arg {
                    None => Letters::default(),
                    Some("") => return Err(format!("`{verb}:` needs letters, e.g. `{verb}:ab`")),
                    Some(text) => text.parse().map_err(|e| format!("`{s}`: {e}"))?,
                };
                Ok(if verb == "pickup" {
                    Action::PickUp(letters)
                } else {
                    Action::Drop(letters)
                })
            }
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
                 descend, lantern, rest:1h|dawn|dusk, pickup[:LETTERS], drop[:LETTERS], \
                 vision:ID, skip:DURATION)"
            )),
        }
    }
}

/// Parses whitespace-separated actions, e.g. `"move:E move:SE wait"`; `ACTION*N` repeats an
/// action, e.g. `"move:N*3 wait*10"`.
pub fn parse_actions(text: &str) -> Result<Vec<Action>, String> {
    let mut actions = Vec::new();
    for word in text.split_whitespace() {
        let (action, count) = match word.split_once('*') {
            Some((action, n)) => match n.parse::<usize>() {
                Ok(n) => (action, n),
                Err(e) => return Err(format!("`{word}`: {e}")),
            },
            None => (word, 1),
        };
        let action: Action = action.parse()?;
        actions.extend(std::iter::repeat_n(action, count));
    }
    Ok(actions)
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
    fn letters_are_a_set_of_positions() {
        let mut l = Letters::of([3, 0, 3, 30]);
        assert_eq!((l.len(), l.to_string()), (2, "ad".to_string()));
        assert!(l.contains(3) && !l.contains(1) && !l.contains(30));
        l.remove(3);
        l.insert(25);
        assert_eq!(l.iter().collect::<Vec<_>>(), [0, 25]);
        assert_eq!("za".parse::<Letters>(), Ok(Letters::of([0, 25])));
        assert_eq!(letter(0), Some('a'));
        assert_eq!(letter(25), Some('z'));
        assert_eq!(letter(26), None);
        assert!(Letters::default().is_empty());
    }

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
            Action::PickUp(Letters::default()),
            Action::PickUp(Letters::of([0])),
            Action::PickUp(Letters::of([0, 2, 25])),
            Action::Drop(Letters::default()),
            Action::Drop(Letters::of([1])),
            Action::Drop(Letters::of(0..Letters::COUNT)),
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
            parse_actions("move:N*3 wait*0 wait"),
            Ok(vec![
                Action::Move(Dir::N),
                Action::Move(Dir::N),
                Action::Move(Dir::N),
                Action::Wait
            ])
        );
        assert!(parse_actions("move:N*x").is_err());
        assert!(parse_actions("wait*").is_err());
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
        assert_eq!(
            parse_actions("PICKUP:CA drop pickup drop:b"),
            Ok(vec![
                Action::PickUp(Letters::of([0, 2])),
                Action::Drop(Letters::default()),
                Action::PickUp(Letters::default()),
                Action::Drop(Letters::of([1])),
            ])
        );
        assert_eq!(Action::PickUp(Letters::of([2, 0])).to_string(), "pickup:ac");
        for bad in [
            "pickup:",
            "pickup:a1",
            "drop:A-",
            "drop:",
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
