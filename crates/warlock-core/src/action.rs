//! Player actions and the events a turn produces.

use crate::combat::Strike;
use crate::content::{ActionKind, CombatStance, DiseaseId, ObjectId, StatusId};
use crate::geom::{Dir, Pos};
use crate::time::{DAY, Duration};
use crate::weather::Weather;
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
    /// of the pile), or everything there with no letters. Takes a turn (its cost: `pick_up`).
    PickUp(Letters),
    /// Drop the inventory stacks with these letters onto the player's cell, or everything
    /// carried with no letters. Takes a turn (its cost: `drop`, plus taking off what of it is
    /// worn or wielded).
    Drop(Letters),
    /// Wear or wield inventory stack `item` (a position, `a` = 0) in `slot` (an id in
    /// `data/slots.toml`), taking off what is there first (a swap). Takes the time of the slot's
    /// action (`wield`, `wear`, `body_armor`, `ready`) for each thing put on or taken off.
    Equip {
        slot: String,
        item: usize,
    },
    /// Take off what is in a slot (an id in `data/slots.toml`). Takes the time of its action.
    Remove(String),
    /// Bash whoever stands in a direction with the shield the player holds (spec §8 "Weapon
    /// skills"): a melee blow with the shield's own damage, drawing on and training shield
    /// bashing. Takes the time of `melee`.
    Bash(Dir),
    /// Shoot the readied missile weapon's ammunition at this cell (spec §8 "Ranged combat"):
    /// one shot flies along the line through it, as far as the weapon shoots. Takes the time of
    /// `shoot` (plus `reload` for a missile weapon that needs it).
    Fire(Pos),
    /// Throw one of inventory stack `item` (a position, `a` = 0), a throwing weapon, at cell
    /// `at`. Takes the time of `throw`.
    Throw {
        item: usize,
        at: Pos,
    },
    /// Wait turn after turn (creatures act) until the goal, stopping early when a creature comes
    /// into view or a message is logged.
    Rest(Rest),
    /// Take a combat stance (spec §8 "Stances"). Takes no game time, but changes state, so
    /// replays record it.
    SetStance(CombatStance),
    /// Dig into the wall in this direction with the digging tool the player has ready (spec §8
    /// "Digging"): turn after turn (each the `dig` action's time) until the wall gives way,
    /// stopping early like a rest. What was dug stays dug.
    Dig(Dir),
    /// Dev command: switch the player's vision profile (an id in `data/vision.toml`). Takes no
    /// game time, but changes state and perception, so replays record it like any action.
    SetVision(String),
    /// Dev command: move the clock ahead at once; nothing else acts.
    SkipTime(Duration),
    /// Dev command: set the weather for three hours of game time ([`crate::weather::OVERRIDE`]),
    /// then the weather of the day returns. Takes no game time; recorded in replays.
    SetWeather(Weather),
    /// Dev command: show spell effect `effect` (an id in `data/effects.toml`) at level cell `at`,
    /// or from `at` toward `to`: a burst around `at` (around `to`, flying there from `at`), or a
    /// bolt from `at` to `to` (cut short by the first opaque cell). It only shows: it hurts no
    /// one and takes no game time, but it is recorded in replays, so captures can show any moment
    /// of an effect (spec §4.8 "Spell effects").
    Effect {
        effect: String,
        at: Pos,
        to: Option<Pos>,
    },
    /// Dev command: the player suffers one degree more of a status effect (an id in
    /// `data/status.toml`), as a wound or a bite would give it: for trying statuses out and for
    /// captures. Takes no game time; recorded in replays.
    Afflict(String),
    /// Dev command: the player catches a disease (an id in `data/diseases.toml`) as if they
    /// had failed to resist it: it incubates, then shows. Takes no game time; recorded in
    /// replays.
    Infect(String),
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
    /// An actor attacked another (spec §8 "Combat"). `from` and `to` are where they stood at
    /// that moment; `strike` is what came of it: a hit (a critical, perhaps) or a miss and why,
    /// or `None` for an intent only (a spell). Kept only when the player saw both of them (or
    /// is one of them); the message line reports more (an attack on the player by something
    /// unseen, an attack the player saw only one end of).
    Attacked {
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        from: Pos,
        to: Pos,
        strike: Option<Strike>,
        /// What a shot or a throw was made with: the missile (the ammunition as it flew, the
        /// thrown thing); for a shield bash, the shield; `None` for other blows and spells.
        with: Option<ObjectId>,
    },
    /// A missile flew (spec §8 "Ranged combat"): a shot or a throw. As the player perceives it
    /// (the rest of the turn's events are kept only when seen; this one is cut to what was
    /// seen): `from` is where it was loosed if the player saw that cell, `path` the cells it
    /// flew over, in order, from the first the player saw to the last, and `end` how its flight
    /// ended on the last of them. Kept when the player saw any of its path.
    Missile {
        /// What flew: the object as it flies and lands (a flaming arrow flies burning).
        object: ObjectId,
        from: Option<Pos>,
        path: Vec<Pos>,
        end: MissileEnd,
    },
    /// An actor died where it stood, killed by the attack before it, or of status effect `of`
    /// (it bled to death). Kept when the player saw it (their own death always).
    Died {
        actor: ActorId,
        at: Pos,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        of: Option<StatusId>,
    },
    /// A status effect began (`began`) or ended on an actor (spec §8 "Status effects"). Only the
    /// player's own, which they always perceive: said in the message line ("You are bleeding.",
    /// "The bleeding stops.").
    Status {
        actor: ActorId,
        status: StatusId,
        began: bool,
    },
    /// A disease of the player's showed itself, went badly at a progression, or left them (spec
    /// §8 "Diseases"). Only the player's own, which they always perceive: said in the message
    /// line in the disease's own words (its `onset`, a symptom, `recovered`).
    Disease {
        actor: ActorId,
        disease: DiseaseId,
        news: DiseaseNews,
    },
    /// A spell-like effect the player perceives (spec §4.8 "Spell effects"): traps now, spells
    /// later. As the player perceives it: it is light itself, so its cells are cut to those the
    /// player sees or has in line of sight (in the dark too), and a bolt's path to the stretch
    /// from the first of them to the last. Kept when the player saw any of its cells.
    Spell {
        /// An id in `data/effects.toml` (`fireball`, `lightning_bolt`): what it looks like.
        effect: String,
        /// Where it comes from: the trap's cell, later a caster.
        origin: Pos,
        shape: SpellShape,
        /// The affected cells the player can see; perception-filtered, never ground truth.
        cells: Vec<Pos>,
    },
    /// A wall gave way (spec §8 "Digging") and is rubble now. Kept when the player sees the cell
    /// afterwards.
    WallBroken {
        pos: Pos,
        cause: Breach,
    },
}

/// What became of a disease ([`Event::Disease`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiseaseNews {
    /// Its incubation is over: it shows.
    Onset,
    /// Its progression number `n` (1 for the first) went badly: a symptom.
    Worse(u16),
    /// It is gone.
    Recovered,
}

/// The shape of a spell effect ([`Event::Spell`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellShape {
    /// A ball around `center` reaching `radius` cells (the rounded radius, like every radius:
    /// cells at squared distance up to `radius² + radius`).
    Burst { center: Pos, radius: i32 },
    /// A line along `path`, in order, from the cell next to its origin on.
    Bolt { path: Vec<Pos> },
}

/// What broke a wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Breach {
    /// Dug through with a tool.
    Dug,
    /// Blasted open by an explosion.
    Blast,
}

/// How a missile's flight ended, on the last cell of its path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissileEnd {
    /// It struck someone there.
    Struck,
    /// It came down there: it stopped against something solid, at the end of its range, or at
    /// the cell it was aimed at.
    Fell,
    /// It flew on out of the player's sight.
    Unseen,
}

/// How an attack is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackKind {
    /// From the next cell.
    Melee,
    /// A shot from a distance, with a missile weapon.
    Ranged,
    /// A throwing weapon thrown from a distance.
    Thrown,
    /// A spell cast from a distance.
    Spell,
}

impl AttackKind {
    /// The action whose cost it has (`data/actions.toml`: `melee`, `shoot`, `throw`, `cast`).
    pub const fn action(self) -> ActionKind {
        match self {
            AttackKind::Melee => ActionKind::Melee,
            AttackKind::Ranged => ActionKind::Shoot,
            AttackKind::Thrown => ActionKind::Throw,
            AttackKind::Spell => ActionKind::Cast,
        }
    }
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

/// Text syntax for scripting: `move:E wait open:E close:W ascend descend lantern rest:1h rest:dawn
/// rest:dusk pickup pickup:ac drop:b equip:body:b remove:body bash:E fire:12,5 throw:c:12,5 dig:E
/// vision:ID skip:1h weather:rain effect:fireball@12,5 effect:lightning_bolt@3,4>12,4
/// afflict:bleeding infect:filth_fever` (durations: `30s`, `10m`, `2h`; letters name stacks, `a`
/// first; `pickup` and `drop` without letters take everything; `equip:SLOT:LETTER` puts inventory
/// stack LETTER on in slot SLOT, `remove:SLOT` takes off what is there; `bash:DIR` bashes with the
/// held shield; `dig:DIR` digs toward a wall; `fire:X,Y` shoots at the level cell (X, Y),
/// `throw:LETTER:X,Y` throws one of inventory stack LETTER there; weather: `clear`, `overcast`,
/// `rain`, `heavy`; `effect:ID@X,Y[>X,Y]` shows a spell effect at a cell, or from one cell toward
/// another).
///
/// `Display` and `FromStr` round-trip; action names are case-insensitive when parsing (vision
/// profile, slot, and effect ids keep their case).
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
            Action::Equip { slot, item } => {
                write!(f, "equip:{slot}:{}", letter(*item).unwrap_or('?'))
            }
            Action::Remove(slot) => write!(f, "remove:{slot}"),
            Action::Bash(d) => write!(f, "bash:{d}"),
            Action::Fire(at) => write!(f, "fire:{},{}", at.x, at.y),
            Action::Throw { item, at } => {
                let item = letter(*item).unwrap_or('?');
                write!(f, "throw:{item}:{},{}", at.x, at.y)
            }
            Action::Rest(Rest::For(d)) => write!(f, "rest:{d}"),
            Action::Rest(Rest::Dawn) => f.write_str("rest:dawn"),
            Action::Rest(Rest::Dusk) => f.write_str("rest:dusk"),
            Action::SetStance(stance) => write!(f, "stance:{stance}"),
            Action::Dig(d) => write!(f, "dig:{d}"),
            Action::SetVision(id) => write!(f, "vision:{id}"),
            Action::SkipTime(d) => write!(f, "skip:{d}"),
            Action::SetWeather(w) => write!(f, "weather:{}", w.id()),
            Action::Effect { effect, at, to } => {
                write!(f, "effect:{effect}@{},{}", at.x, at.y)?;
                match to {
                    Some(to) => write!(f, ">{},{}", to.x, to.y),
                    None => Ok(()),
                }
            }
            Action::Afflict(id) => write!(f, "afflict:{id}"),
            Action::Infect(id) => write!(f, "infect:{id}"),
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
            "bash" => dir().map(Action::Bash),
            "dig" => dir().map(Action::Dig),
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
            "weather" => match arg.and_then(Weather::from_id) {
                Some(w) => Ok(Action::SetWeather(w)),
                None => Err("`weather` needs `clear`, `overcast`, `rain`, or `heavy`".into()),
            },
            "equip" => {
                // The slot id keeps its case; the letter is a letter.
                let usage = || "`equip` needs a slot and a letter, e.g. `equip:body:b`".to_string();
                let (slot, item) = s
                    .split_once(':')
                    .and_then(|(_, rest)| rest.rsplit_once(':'))
                    .ok_or_else(usage)?;
                let mut chars = item.chars();
                match (chars.next().map(|c| c.to_ascii_lowercase()), chars.next()) {
                    (Some(c @ 'a'..='z'), None) if !slot.is_empty() && !slot.contains(':') => {
                        Ok(Action::Equip {
                            slot: slot.to_string(),
                            item: usize::from(c as u8 - b'a'),
                        })
                    }
                    _ => Err(usage()),
                }
            }
            "remove" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Remove(id.to_string()))
                }
                _ => Err("`remove` needs a slot, e.g. `remove:body`".into()),
            },
            "fire" => {
                let at = arg.ok_or_else(|| "`fire` needs a cell, e.g. `fire:12,5`".to_string())?;
                parse_cell(at).map(Action::Fire)
            }
            "throw" => {
                let usage = || "`throw` needs a letter and a cell, e.g. `throw:c:12,5`".to_string();
                let (item, at) = arg.and_then(|a| a.split_once(':')).ok_or_else(usage)?;
                let mut chars = item.chars();
                match (chars.next(), chars.next()) {
                    (Some(c @ 'a'..='z'), None) => Ok(Action::Throw {
                        item: usize::from(c as u8 - b'a'),
                        at: parse_cell(at).map_err(|_| usage())?,
                    }),
                    _ => Err(usage()),
                }
            }
            "stance" => arg
                .ok_or_else(|| "`stance` needs a stance, e.g. `stance:coward`".to_string())?
                .parse()
                .map(Action::SetStance),
            "vision" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() => Ok(Action::SetVision(id.to_string())),
                _ => Err("`vision` needs a profile id, e.g. `vision:darkvision`".into()),
            },
            "effect" => {
                // The effect id keeps its case.
                let usage = || {
                    "`effect` needs an effect and a cell, e.g. `effect:fireball@12,5` or \
                     `effect:lightning_bolt@3,4>12,4`"
                        .to_string()
                };
                let (effect, cells) = (s.split_once(':'))
                    .and_then(|(_, rest)| rest.split_once('@'))
                    .ok_or_else(usage)?;
                let bad = |c: char| matches!(c, ':' | '@' | '>') || c.is_whitespace();
                if effect.is_empty() || effect.contains(bad) {
                    return Err(usage());
                }
                let (at, to) = match cells.split_once('>') {
                    Some((at, to)) => (at, Some(to)),
                    None => (cells, None),
                };
                Ok(Action::Effect {
                    effect: effect.to_string(),
                    at: parse_cell(at).map_err(|_| usage())?,
                    to: to.map(parse_cell).transpose().map_err(|_| usage())?,
                })
            }
            "afflict" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Afflict(id.to_string()))
                }
                _ => Err("`afflict` needs a status id, e.g. `afflict:bleeding`".into()),
            },
            "infect" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Infect(id.to_string()))
                }
                _ => Err("`infect` needs a disease id, e.g. `infect:filth_fever`".into()),
            },
            _ => Err(format!(
                "unknown action `{s}` (expected move:DIR, wait, open:DIR, close:DIR, ascend, \
                 descend, lantern, rest:1h|dawn|dusk, pickup[:LETTERS], drop[:LETTERS], \
                 equip:SLOT:LETTER, remove:SLOT, bash:DIR, fire:X,Y, throw:LETTER:X,Y, dig:DIR, \
                 stance:NAME, vision:ID, skip:DURATION, weather:clear|overcast|rain|heavy, \
                 effect:ID@X,Y[>X,Y], afflict:STATUS, infect:DISEASE)"
            )),
        }
    }
}

/// A level cell written `X,Y` (`12,5`).
fn parse_cell(text: &str) -> Result<Pos, String> {
    let bad = || format!("`{text}` is no cell; expected X,Y, e.g. `12,5`");
    let (x, y) = text.split_once(',').ok_or_else(bad)?;
    let coord = |v: &str| v.trim().parse::<i32>().map_err(|_| bad());
    Ok(Pos::new(coord(x)?, coord(y)?))
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
            Action::SetStance(CombatStance::Aggressive),
            Action::SetStance(CombatStance::Coward),
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
            Action::Equip {
                slot: "body".into(),
                item: 1,
            },
            Action::Equip {
                slot: "Left_Ring".into(),
                item: 25,
            },
            Action::Remove("main_hand".into()),
            Action::Fire(Pos::new(12, 5)),
            Action::Fire(Pos::new(0, 40)),
            Action::Throw {
                item: 2,
                at: Pos::new(7, 3),
            },
            Action::Effect {
                effect: "fireball".into(),
                at: Pos::new(12, 5),
                to: None,
            },
            Action::Effect {
                effect: "Lightning_Bolt".into(),
                at: Pos::new(-3, 4),
                to: Some(Pos::new(12, 0)),
            },
            Action::Afflict("bleeding".into()),
            Action::Afflict("Odd_Case".into()),
            Action::Infect("filth_fever".into()),
            Action::Infect("Odd_Case".into()),
        ];
        all.extend(Weather::ALL.map(Action::SetWeather));
        for d in Dir::ALL {
            all.extend([
                Action::Move(d),
                Action::Open(d),
                Action::Close(d),
                Action::Bash(d),
                Action::Dig(d),
            ]);
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
        assert_eq!(parse_actions("BASH:sw"), Ok(vec![Action::Bash(Dir::SW)]));
        assert!(parse_actions("bash").is_err());
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
        assert_eq!(
            parse_actions("WEATHER:Heavy weather:clear"),
            Ok(vec![
                Action::SetWeather(Weather::HeavyRain),
                Action::SetWeather(Weather::Clear)
            ])
        );
        assert_eq!(
            parse_actions("EQUIP:body:B remove:Missiles"),
            Ok(vec![
                Action::Equip {
                    slot: "body".into(),
                    item: 1
                },
                Action::Remove("Missiles".into()),
            ])
        );
        assert_eq!(
            Action::Equip {
                slot: "head".into(),
                item: 2
            }
            .to_string(),
            "equip:head:c"
        );
        assert_eq!(
            parse_actions("FIRE:3,4 throw:B:10,2"),
            Ok(vec![
                Action::Fire(Pos::new(3, 4)),
                Action::Throw {
                    item: 1,
                    at: Pos::new(10, 2)
                },
            ])
        );
        assert_eq!(
            parse_actions("EFFECT:Fireball@3,4 effect:lightning_bolt@1,2>9,2"),
            Ok(vec![
                Action::Effect {
                    effect: "Fireball".into(),
                    at: Pos::new(3, 4),
                    to: None
                },
                Action::Effect {
                    effect: "lightning_bolt".into(),
                    at: Pos::new(1, 2),
                    to: Some(Pos::new(9, 2))
                },
            ])
        );
        assert_eq!(
            Action::Effect {
                effect: "lightning_bolt".into(),
                at: Pos::new(1, 2),
                to: Some(Pos::new(9, 2)),
            }
            .to_string(),
            "effect:lightning_bolt@1,2>9,2"
        );
        for bad in [
            "effect",
            "effect:",
            "effect:fireball",
            "effect:fireball@",
            "effect:@3,4",
            "effect:fireball@3",
            "effect:fireball@3,4>",
            "effect:fireball@3,4>5",
            "effect:fire:ball@3,4",
            "effect:fireball@3,4>5,6>7,8",
            "fire",
            "fire:",
            "fire:3",
            "fire:a,b",
            "throw",
            "throw:3,4",
            "throw:a",
            "throw:ab:3,4",
            "throw:1:3,4",
            "equip",
            "equip:body",
            "equip:body:",
            "equip::b",
            "equip:body:ab",
            "equip:body:1",
            "equip:a:b:c",
            "remove",
            "remove:",
            "remove:a:b",
            "weather",
            "weather:snow",
            "afflict",
            "afflict:",
            "afflict:a:b",
            "infect",
            "infect:",
            "infect:a:b",
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
            "dig",
            "dig:",
            "dig:down",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
    }
}
