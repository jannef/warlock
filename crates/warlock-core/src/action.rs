//! Player actions and the events a turn produces.

use crate::combat::Strike;
use crate::content::{
    ActionKind, CombatStance, DamageTypeId, DiseaseId, ObjectId, StatusId, TrapId,
};
use crate::geom::{Dir, Pos};
use crate::time::{DAY, Duration};
use crate::weather::Weather;
use crate::world::{ActorId, Conversation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

/// Something the player asks to do. Actions are the unit of replays (spec §9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Step in a direction; bumping a closed door opens it (unlocking it first with a key that
    /// fits, if it is locked).
    Move(Dir),
    /// Let a turn pass.
    Wait,
    /// Open the door in a direction; a locked one only with a key that fits, which unlocks it
    /// first (spec §8 "Doors").
    Open(Dir),
    Close(Dir),
    /// Kick in a direction (spec §8 "Doors"): a closed door, to break it (a chance of Strength in
    /// its sturdiness), or a creature, an unarmed blow. Takes the time of `kick`; kicking at
    /// empty space or something solid takes none.
    Kick(Dir),
    /// Pick the lock of the closed door in a direction (spec §8 "Doors"; the Pick lock ability,
    /// the Open lock skill's use): a Dexterity check plus what Open lock gives against the lock's
    /// difficulty. Picked, the door opens. Takes the time of `pick_lock` whatever comes of it
    /// (plus `open_door` when it opens); refused, taking none, without the skill or a lock there.
    PickLock(Dir),
    /// Take stairs up.
    Ascend,
    /// Take stairs down.
    Descend,
    /// Light or douse the lantern the scenario gives the player. Takes a turn.
    ToggleLantern,
    /// Lower or raise the hood of the player's lantern (spec §5 "Light sources"): hooded, it
    /// burns dim and hollow, lighting the cells around the player but not them. A lantern that
    /// is out is lit with its hood lowered. Takes the time of `hood` (and of `lantern` when it
    /// lights it); refused, no time, without a lantern with a hood.
    HoodLantern,
    /// Pick up the stacks at these positions in the pile on the player's cell (0 is the top of
    /// the pile), or everything there with none. Takes a turn (its cost: `pick_up`).
    PickUp(Positions),
    /// Drop the inventory stacks at these positions onto the player's cell, or everything
    /// carried with none. Takes a turn (its cost: `drop`, plus taking off what of it is worn or
    /// wielded). In a shop (spec §7 "Shops") what of them is unpaid goes back; the player's own
    /// things stay, and the shop says what it would pay for those it buys.
    Drop(Positions),
    /// Sell the inventory stacks at these positions (everything with none) to the shop the
    /// player stands in, for what it offers (spec §7 "Shops"): like [`Action::Drop`] there, but
    /// what the shop buys is sold, paid from the keeper's purse. Takes the time of `drop`.
    Sell(Positions),
    /// Pay for everything unpaid in the shop the player stands in (spec §7 "Shops"), from the
    /// gold they carry, which takes no game time; then, with a direction, step that way (out of
    /// the shop, as [`Action::Move`] would, taking its time): paying and leaving in one action.
    /// Refused, with no step, if the player cannot pay.
    Pay(Option<Dir>),
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
    /// Chat with the creature in a direction (spec §8 "Dialogue"): it answers with a line, and
    /// one that has topics opens the dialogue screen. Takes the time of `chat`; refused (no
    /// time) when the player sees no one there.
    Chat(Dir),
    /// Ask the creature in a direction about one of its topics, by id. Takes no time. Quests
    /// (spec §8 "Quests") are asked about with topics of their own: `quest-ID` (hear the offer,
    /// be reminded, or hand it in), `accept-ID`, and `decline-ID`.
    Ask {
        dir: Dir,
        topic: String,
    },
    /// Try to steal from the creature in a direction (spec §8 "Stealing"): one stack of what it
    /// carries and neither wears nor wields, if the player's Dexterity and Steal skill beat its
    /// Willpower; caught, it turns on them. Takes the time of `steal`; refused (no time) when the
    /// player sees no one there, or someone hostile.
    Steal(Dir),
    /// Throw the player's voice to a cell they see within [`crate::world::VOICE_RANGE`] (spec §5
    /// "Hearing"): a voice sounds there, and creatures that hear it and do not see its cell go
    /// and look. Takes the time of `throw_voice`; refused (no time) at a cell they do not see,
    /// out of reach, solid, or their own.
    ThrowVoice(Pos),
    /// Eat one of a stack, from the pack or the floor (spec §8 "Hunger"): a meal of as many
    /// turns (each the `eat` action) as the food takes, stopping early like a rest; the food is
    /// eaten and feeds at the end.
    Eat(Meal),
    /// Read one of inventory stack `item` (a position, `a` = 0), a scroll (spec §8 "Scrolls"),
    /// using it up: a spell scroll at the cell `target` names (it flies or runs there from the
    /// player, with the traps' rules), the scroll of identify on the inventory stack it names.
    /// Takes the time of `read`; refused (no time, the scroll kept) without a fitting target.
    Read {
        item: usize,
        target: Option<ReadTarget>,
    },
    /// Take a combat stance (spec §8 "Stances"). Takes no game time, but changes state, so
    /// replays record it.
    SetStance(CombatStance),
    /// Dig into the wall in this direction with the digging tool the player has ready (spec §8
    /// "Digging"): turn after turn (each the `dig` action's time) until the wall gives way,
    /// stopping early like a rest. What was dug stays dug.
    Dig(Dir),
    /// Search for hidden features (secret doors, hidden traps; spec §8 "Searching") for up to
    /// this many turns (each the `search` action's time, a fresh check every turn), stopping at
    /// once when something is found or something interrupts, like a rest. 1 is one search.
    Search(u8),
    /// Try to disarm the trap the player knows of in a direction (spec §8 "Disarming traps"; the
    /// Disarm ability): a Dexterity check plus Disable device against the trap's difficulty takes
    /// it away; a bad failure sets it off at the player. Takes the time of `disarm`, whatever
    /// comes of it; refused (no time) without the skill or a known trap there.
    Disarm(Dir),
    /// Catch a second wind (spec §8 "Talents"; the Second wind ability): hit points back at
    /// once, then Winded for a while. Takes the time of `wait`; refused (no time) without the
    /// talent or while winded.
    SecondWind,
    /// Take talent `id` (an id in `data/talents/`), for `choice` if it takes one (spec §8
    /// "Talents"): one of the picks a level brings, made at once (nothing is banked). Refused
    /// with the reason when none is due or a prerequisite fails. Takes no game time; recorded in
    /// replays.
    Talent {
        id: String,
        choice: Option<String>,
    },
    /// Spend one of a level's skill rolls on trained skill `skill` (spec §8 "Talents"). No
    /// level brings rolls until the skill ladder lands (#230), so it is refused. Takes no game
    /// time; recorded in replays.
    Advance(String),
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
    /// Dev command: identify the inventory stacks at these positions, or everything carried
    /// with none (spec §8 "Magic items"; the scroll of identify does it in play). Takes no
    /// game time; recorded in replays.
    Identify(Positions),
    /// Dev command: the player suffers one degree more of a status effect (an id in
    /// `data/status.toml`), as a wound or a bite would give it: for trying statuses out and for
    /// captures. Takes no game time; recorded in replays.
    Afflict(String),
    /// Dev command: the player catches a disease (an id in `data/diseases.toml`) as if they
    /// had failed to resist it: it incubates, then shows. Takes no game time; recorded in
    /// replays.
    Infect(String),
    /// Dev command: spend every choice a level still asks for with the first the player may
    /// take (spec §8 "Talents"), for tests and benchmarks. Takes no game time; recorded in
    /// replays.
    LevelUpAuto,
    /// Dev command: the player gains this much experience, and any levels it brings (spec §8
    /// "Experience"), for tests and captures. Takes no game time; recorded in replays.
    GrantXp(u32),
    /// Dev command: an instant frost (spec §5 "Ice"): the water within `radius` of cell `at`, in
    /// its line of sight, freezes at once to ice that holds a laden person
    /// ([`crate::ice::FROST`]). Takes no game time; recorded in replays.
    Freeze {
        at: Pos,
        radius: i32,
    },
    /// Dev command: a burst of heat (standing in for a fireball): the ice within `radius` of
    /// cell `at`, in its line of sight, melts at once, and whoever stood on it falls in. Takes no
    /// game time; recorded in replays.
    Thaw {
        at: Pos,
        radius: i32,
    },
}

/// The largest radius of the dev frost and heat bursts.
pub const MAX_BURST: i32 = 20;

/// The most turns one search action may last (`search:N`).
pub const MAX_SEARCH: u8 = 100;

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

/// Where the food of a meal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Meal {
    /// Inventory stack at this position (`a` = 0): `eat:b`.
    Pack(usize),
    /// The stack at this position in the pile on the player's cell (`a` = the top):
    /// `eat:floor:a`.
    Floor(usize),
}

/// What a scroll is read at ([`Action::Read`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadTarget {
    /// A level cell, aimed at: `read:b:12,5`.
    Cell(Pos),
    /// An inventory stack, by position: `read:b:c` (the scroll of identify).
    Item(usize),
}

/// A set of stacks of a list (a pile, the inventory) by position, `0` the first: the rules name
/// stacks by where they are, never by the letter a screen shows (letters restart on every page
/// of a list, spec §6). Action text writes the set as letters while every position is below 26
/// (`a` = 0: `ac`), else as numbers (`0,2,30`); both forms parse, mixed too (`ac,30`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Positions(BTreeSet<usize>);

/// The letters naming list rows, `a` to `z`: action text writes the positions below this as
/// letters, and a page of a list holds at most this many stacks.
pub const LETTERS: usize = 26;

impl Positions {
    pub fn of(positions: impl IntoIterator<Item = usize>) -> Self {
        Self(positions.into_iter().collect())
    }

    pub fn insert(&mut self, i: usize) {
        self.0.insert(i);
    }

    pub fn remove(&mut self, i: usize) {
        self.0.remove(&i);
    }

    pub fn contains(&self, i: usize) -> bool {
        self.0.contains(&i)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The positions, in order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().copied()
    }
}

/// The letter of list row `i`: `a` for 0. Rows past `z` have none.
pub fn letter(i: usize) -> Option<char> {
    (i < LETTERS).then(|| char::from(b'a' + i as u8))
}

/// Position `i` in action text: its letter below 26 (`c`), else its number (`30`).
pub fn position_text(i: usize) -> String {
    letter(i).map_or_else(|| i.to_string(), String::from)
}

/// One position in action text: a letter `a` to `z` (either case) or a number.
fn parse_position(text: &str) -> Option<usize> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => {
            Some(usize::from(c.to_ascii_lowercase() as u8 - b'a'))
        }
        _ if text.bytes().all(|b| b.is_ascii_digit()) => text.parse().ok(),
        _ => None,
    }
}

impl fmt::Display for Positions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.iter().all(|i| i < LETTERS) {
            return (self.iter().filter_map(letter)).try_for_each(|c| write!(f, "{c}"));
        }
        let numbers: Vec<String> = self.iter().map(|i| i.to_string()).collect();
        f.write_str(&numbers.join(","))
    }
}

impl FromStr for Positions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut positions = Positions::default();
        for part in s.split(',') {
            let letters = !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase());
            let number = !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
            if letters {
                part.bytes()
                    .for_each(|b| positions.insert(usize::from(b - b'a')));
            } else if let (true, Ok(i)) = (number, part.parse()) {
                positions.insert(i);
            } else {
                return Err(format!(
                    "`{part}` is neither letters a to z nor a position such as `30`"
                ));
            }
        }
        Ok(positions)
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
    /// A kick at a closed door that held: "WHAMM!" (spec §8 "Doors").
    DoorKicked {
        pos: Pos,
    },
    /// A door kicked in: it broke, and is open for good.
    DoorBroken {
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
        /// The kind of damage it did (spec §8 "Damage types": what a blow is said with, a
        /// sword's slashing or piercing by what was better); `None` for a miss, an intent, or
        /// an untyped attack.
        damage: Option<DamageTypeId>,
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
    /// An actor died where it stood, killed by the attack (or trap) before it, or of status
    /// effect `of` (it bled to death). Kept when the player saw it (their own death always).
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
    /// A trap went off (spec §8 "Traps"): `victim` set off trap `trap` lying on `at`. Kept when
    /// the player perceived it: they set it off, saw its cell, or saw any of its blast.
    TrapSprung {
        trap: TrapId,
        at: Pos,
        victim: ActorId,
    },
    /// What a trap did to `target`, standing on `to`: what came of its attack roll, or a hit for
    /// what is not rolled (a blast, a fall). Kept when the player is the target or saw them.
    TrapStruck {
        trap: TrapId,
        target: ActorId,
        to: Pos,
        strike: Strike,
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
    /// The player chatted with a creature, or asked it about a topic, and it answered (spec §8
    /// "Dialogue"): what it said and what the player may ask it about. One with topics opens
    /// the dialogue screen.
    Talked(Conversation),
    /// The player threw their voice to this cell (spec §5 "Hearing"; a cell they see): where it
    /// sounded.
    VoiceThrown {
        at: Pos,
    },
    /// A wall gave way (spec §8 "Digging") and is rubble now. Kept when the player sees the cell
    /// afterwards.
    WallBroken {
        pos: Pos,
        cause: Breach,
    },
    /// An actor broke through the ice and plunged into the water at `at` (spec §5 "Ice"). Kept
    /// when the player saw it (their own always).
    FellThroughIce {
        actor: ActorId,
        at: Pos,
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

/// Text syntax for scripting: `move:E wait open:E close:W kick:N pick:E ascend descend lantern hood
/// rest:1h rest:dawn rest:dusk pickup pickup:ac drop:b sell:b pay pay:W equip:body:b remove:body
/// bash:E fire:12,5 throw:c:12,5 dig:E search search:10 disarm:E eat:b eat:floor:a read:b
/// read:b:12,5 read:b:c
/// chat:E ask:E:TOPIC steal:E throw_voice:12,5 vision:ID
/// skip:1h
/// weather:rain effect:fireball@12,5 effect:lightning_bolt@3,4>12,4 identify:a afflict:bleeding
/// infect:filth_fever freeze:12,5,3 thaw:12,5,3` (durations: `30s`, `10m`, `2h`; LETTER names a
/// stack by its position, a letter for the first 26 (`a` = 0) or a number for any (`30`), and
/// LETTERS several, as letters (`ac`) or numbers (`0,2,30`); `pickup`, `drop`, `sell`, and
/// `identify` without letters take everything; `pay:DIR` pays and steps that way, out of the shop;
/// `equip:SLOT:LETTER` puts inventory stack LETTER on in slot SLOT, `remove:SLOT` takes off what
/// is there; `pick:DIR` picks the lock of the door that way; `bash:DIR` bashes with the held
/// shield; `dig:DIR` digs toward a wall; `search` searches once and `search:N` for up to N turns,
/// 1 to [`MAX_SEARCH`]; `disarm:DIR` tries to disarm the known trap that way; `fire:X,Y`
/// shoots at the level cell (X, Y), `throw:LETTER:X,Y` throws one of inventory stack LETTER
/// there; `eat:LETTER` eats one of inventory stack LETTER, `eat:floor:LETTER` one of the stack
/// LETTER of the pile underfoot; `read:LETTER:X,Y` reads inventory stack LETTER (a scroll) at
/// the level cell (X, Y), `read:LETTER:LETTER` on the second inventory stack; `chat:DIR` talks to the creature that way, `ask:DIR:TOPIC` asks
/// it about a topic by id, `steal:DIR` tries to steal from it; `hood` lowers or raises the
/// lantern's hood; `throw_voice:X,Y` throws
/// the player's voice to the level cell (X, Y); weather: `clear`, `overcast`, `rain`, `heavy`; `effect:ID@X,Y[>X,Y]`
/// shows a spell effect at a cell, or from one cell toward another; the dev bursts
/// `freeze:X,Y,R` and `thaw:X,Y,R` freeze the water and melt the ice within radius R, 0 to
/// [`MAX_BURST`], of cell (X, Y)).
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
            Action::Kick(d) => write!(f, "kick:{d}"),
            Action::PickLock(d) => write!(f, "pick:{d}"),
            Action::Ascend => f.write_str("ascend"),
            Action::Descend => f.write_str("descend"),
            Action::ToggleLantern => f.write_str("lantern"),
            Action::HoodLantern => f.write_str("hood"),
            Action::PickUp(l) if l.is_empty() => f.write_str("pickup"),
            Action::PickUp(l) => write!(f, "pickup:{l}"),
            Action::Drop(l) if l.is_empty() => f.write_str("drop"),
            Action::Drop(l) => write!(f, "drop:{l}"),
            Action::Sell(l) if l.is_empty() => f.write_str("sell"),
            Action::Sell(l) => write!(f, "sell:{l}"),
            Action::Pay(None) => f.write_str("pay"),
            Action::Pay(Some(d)) => write!(f, "pay:{d}"),
            Action::Equip { slot, item } => write!(f, "equip:{slot}:{}", position_text(*item)),
            Action::Remove(slot) => write!(f, "remove:{slot}"),
            Action::Bash(d) => write!(f, "bash:{d}"),
            Action::Fire(at) => write!(f, "fire:{},{}", at.x, at.y),
            Action::Throw { item, at } => {
                write!(f, "throw:{}:{},{}", position_text(*item), at.x, at.y)
            }
            Action::Eat(Meal::Pack(item)) => write!(f, "eat:{}", position_text(*item)),
            Action::Eat(Meal::Floor(item)) => write!(f, "eat:floor:{}", position_text(*item)),
            Action::Read { item, target } => {
                write!(f, "read:{}", position_text(*item))?;
                match target {
                    None => Ok(()),
                    Some(ReadTarget::Cell(at)) => write!(f, ":{},{}", at.x, at.y),
                    Some(ReadTarget::Item(i)) => write!(f, ":{}", position_text(*i)),
                }
            }
            Action::Rest(Rest::For(d)) => write!(f, "rest:{d}"),
            Action::Rest(Rest::Dawn) => f.write_str("rest:dawn"),
            Action::Rest(Rest::Dusk) => f.write_str("rest:dusk"),
            Action::Chat(d) => write!(f, "chat:{d}"),
            Action::Ask { dir, topic } => write!(f, "ask:{dir}:{topic}"),
            Action::Steal(d) => write!(f, "steal:{d}"),
            Action::ThrowVoice(at) => write!(f, "throw_voice:{},{}", at.x, at.y),
            Action::SetStance(stance) => write!(f, "stance:{stance}"),
            Action::Dig(d) => write!(f, "dig:{d}"),
            Action::Search(1) => f.write_str("search"),
            Action::Search(turns) => write!(f, "search:{turns}"),
            Action::Disarm(d) => write!(f, "disarm:{d}"),
            Action::SecondWind => f.write_str("second_wind"),
            Action::Talent { id, choice: None } => write!(f, "talent:{id}"),
            Action::Talent {
                id,
                choice: Some(choice),
            } => write!(f, "talent:{id}:{choice}"),
            Action::Advance(skill) => write!(f, "advance:{skill}"),
            Action::LevelUpAuto => f.write_str("levelup:auto"),
            Action::GrantXp(xp) => write!(f, "xp:{xp}"),
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
            Action::Identify(l) if l.is_empty() => f.write_str("identify"),
            Action::Identify(l) => write!(f, "identify:{l}"),
            Action::Afflict(id) => write!(f, "afflict:{id}"),
            Action::Infect(id) => write!(f, "infect:{id}"),
            Action::Freeze { at, radius } => write!(f, "freeze:{},{},{radius}", at.x, at.y),
            Action::Thaw { at, radius } => write!(f, "thaw:{},{},{radius}", at.x, at.y),
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
            "chat" => dir().map(Action::Chat),
            "ask" => {
                let usage =
                    || "`ask` needs a direction and a topic, e.g. `ask:E:rooms`".to_string();
                let (dir, topic) = arg.and_then(|a| a.split_once(':')).ok_or_else(usage)?;
                if topic.is_empty() || topic.contains(':') {
                    return Err(usage());
                }
                Ok(Action::Ask {
                    dir: dir.parse().map_err(|_| usage())?,
                    topic: topic.to_string(),
                })
            }
            "steal" => dir().map(Action::Steal),
            "kick" => dir().map(Action::Kick),
            "pick" => dir().map(Action::PickLock),
            "bash" => dir().map(Action::Bash),
            "dig" => dir().map(Action::Dig),
            "disarm" => dir().map(Action::Disarm),
            "search" => match arg.map(str::parse::<u8>) {
                None => Ok(Action::Search(1)),
                Some(Ok(turns)) if (1..=MAX_SEARCH).contains(&turns) => Ok(Action::Search(turns)),
                Some(_) => Err(format!(
                    "`search:N` searches for 1 to {MAX_SEARCH} turns, e.g. `search:10`"
                )),
            },
            "wait" => bare(Action::Wait),
            "second_wind" => bare(Action::SecondWind),
            "ascend" => bare(Action::Ascend),
            "descend" => bare(Action::Descend),
            "lantern" => bare(Action::ToggleLantern),
            "hood" => bare(Action::HoodLantern),
            "pay" => match arg {
                None => Ok(Action::Pay(None)),
                Some(_) => dir().map(|d| Action::Pay(Some(d))),
            },
            "pickup" | "drop" | "sell" | "identify" => {
                let positions = match arg {
                    None => Positions::default(),
                    Some("") => return Err(format!("`{verb}:` needs letters, e.g. `{verb}:ab`")),
                    Some(text) => text.parse().map_err(|e| format!("`{s}`: {e}"))?,
                };
                Ok(match verb {
                    "pickup" => Action::PickUp(positions),
                    "drop" => Action::Drop(positions),
                    "sell" => Action::Sell(positions),
                    _ => Action::Identify(positions),
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
                // The slot id keeps its case; the letter is a letter (or a number).
                let usage = || "`equip` needs a slot and a letter, e.g. `equip:body:b`".to_string();
                let (slot, item) = s
                    .split_once(':')
                    .and_then(|(_, rest)| rest.rsplit_once(':'))
                    .ok_or_else(usage)?;
                match parse_position(item) {
                    Some(item) if !slot.is_empty() && !slot.contains(':') => Ok(Action::Equip {
                        slot: slot.to_string(),
                        item,
                    }),
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
            "throw_voice" => {
                let usage = || "`throw_voice` needs a cell, e.g. `throw_voice:12,5`".to_string();
                parse_cell(arg.ok_or_else(usage)?).map(Action::ThrowVoice)
            }
            "eat" => {
                let usage = || "`eat` needs a letter, e.g. `eat:b` or `eat:floor:a`".to_string();
                let (floor, item) = match arg.map(|a| a.split_once(':')) {
                    Some(Some(("floor", item))) => (true, item),
                    Some(None) => (false, arg.unwrap_or_default()),
                    _ => return Err(usage()),
                };
                let i = parse_position(item).ok_or_else(usage)?;
                Ok(Action::Eat(if floor {
                    Meal::Floor(i)
                } else {
                    Meal::Pack(i)
                }))
            }
            "read" => {
                let usage = || {
                    "`read` needs a letter, then a cell to aim at or a letter to read it on, e.g. \
                     `read:b:12,5` or `read:b:c`"
                        .to_string()
                };
                let arg = arg.ok_or_else(usage)?;
                let (item, target) = match arg.split_once(':') {
                    Some((item, target)) => (item, Some(target)),
                    None => (arg, None),
                };
                let target = match target {
                    None => None,
                    Some(cell) if cell.contains(',') => {
                        Some(ReadTarget::Cell(parse_cell(cell).map_err(|_| usage())?))
                    }
                    Some(i) => Some(ReadTarget::Item(parse_position(i).ok_or_else(usage)?)),
                };
                Ok(Action::Read {
                    item: parse_position(item).ok_or_else(usage)?,
                    target,
                })
            }
            "throw" => {
                let usage = || "`throw` needs a letter and a cell, e.g. `throw:c:12,5`".to_string();
                let (item, at) = arg.and_then(|a| a.split_once(':')).ok_or_else(usage)?;
                Ok(Action::Throw {
                    item: parse_position(item).ok_or_else(usage)?,
                    at: parse_cell(at).map_err(|_| usage())?,
                })
            }
            "stance" => arg
                .ok_or_else(|| "`stance` needs a stance, e.g. `stance:coward`".to_string())?
                .parse()
                .map(Action::SetStance),
            "talent" => {
                // Ids keep their case.
                let usage = || {
                    "`talent` needs a talent id, e.g. `talent:hardy` or \
                     `talent:weapon_focus:long_blades`"
                        .to_string()
                };
                let rest = s.split_once(':').map(|(_, rest)| rest).ok_or_else(usage)?;
                let (id, choice) = match rest.split_once(':') {
                    Some((id, choice)) => (id, Some(choice)),
                    None => (rest, None),
                };
                let bad = |t: &str| {
                    t.is_empty() || t.contains([':', '@']) || t.contains(char::is_whitespace)
                };
                if bad(id) || choice.is_some_and(bad) {
                    return Err(usage());
                }
                Ok(Action::Talent {
                    id: id.to_string(),
                    choice: choice.map(str::to_string),
                })
            }
            "advance" => match s.split_once(':').map(|(_, id)| id) {
                Some(id) if !id.is_empty() && !id.contains(':') => {
                    Ok(Action::Advance(id.to_string()))
                }
                _ => Err("`advance` needs a skill id, e.g. `advance:stealth`".into()),
            },
            "levelup" => match arg {
                Some("auto") => Ok(Action::LevelUpAuto),
                _ => Err("`levelup` takes `auto`: `levelup:auto`".into()),
            },
            "xp" => match arg.and_then(|a| a.parse::<u32>().ok()) {
                Some(xp) if xp > 0 => Ok(Action::GrantXp(xp)),
                _ => Err("`xp` needs a number of experience points, e.g. `xp:2500`".into()),
            },
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
            "freeze" | "thaw" => {
                let usage = || format!("`{verb}` needs a cell and a radius, e.g. `{verb}:12,5,3`");
                let (cell, radius) = arg.and_then(|a| a.rsplit_once(',')).ok_or_else(usage)?;
                let at = parse_cell(cell).map_err(|_| usage())?;
                let radius = match radius.trim().parse::<i32>() {
                    Ok(r) if (0..=MAX_BURST).contains(&r) => r,
                    _ => return Err(format!("`{s}`: the radius is 0 to {MAX_BURST}")),
                };
                Ok(if verb == "freeze" {
                    Action::Freeze { at, radius }
                } else {
                    Action::Thaw { at, radius }
                })
            }
            _ => Err(format!(
                "unknown action `{s}` (expected move:DIR, wait, open:DIR, close:DIR, kick:DIR, \
                 pick:DIR, ascend, descend, lantern, hood, rest:1h|dawn|dusk, pickup[:LETTERS], \
                 drop[:LETTERS], \
                 sell[:LETTERS], pay[:DIR], equip:SLOT:LETTER, remove:SLOT, bash:DIR, fire:X,Y, \
                 throw:LETTER:X,Y, dig:DIR, eat:LETTER, eat:floor:LETTER, \
                 read:LETTER[:X,Y|:LETTER], chat:DIR, \
                 ask:DIR:TOPIC, steal:DIR, throw_voice:X,Y, stance:NAME, vision:ID, skip:DURATION, \
                 weather:clear|overcast|rain|heavy, effect:ID@X,Y[>X,Y], identify[:LETTERS], \
                 afflict:STATUS, infect:DISEASE, freeze:X,Y,R, thaw:X,Y,R)"
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
    fn positions_are_a_set_written_as_letters_or_numbers() {
        let mut l = Positions::of([3, 0, 3]);
        assert_eq!((l.len(), l.to_string()), (2, "ad".to_string()));
        assert!(l.contains(3) && !l.contains(1));
        l.remove(3);
        l.insert(25);
        assert_eq!(l.iter().collect::<Vec<_>>(), [0, 25]);
        assert_eq!("za".parse::<Positions>(), Ok(Positions::of([0, 25])));
        // Past `z` there are no letters: every position is a number then, and both forms (and a
        // mix of them) parse.
        l.insert(30);
        assert_eq!(l.to_string(), "0,25,30");
        for text in ["0,25,30", "az,30", "30,a,25", "za,30,30"] {
            assert_eq!(text.parse::<Positions>(), Ok(l.clone()), "{text}");
        }
        for bad in ["", "a,", ",b", "A", "-1", "+3", "3x", "a b"] {
            assert!(bad.parse::<Positions>().is_err(), "{bad}");
        }
        assert_eq!(letter(0), Some('a'));
        assert_eq!(letter(25), Some('z'));
        assert_eq!(letter(26), None);
        assert_eq!(
            (position_text(2), position_text(26)),
            ("c".into(), "26".into())
        );
        assert!(Positions::default().is_empty());
    }

    #[test]
    fn text_round_trips() {
        let mut all = vec![
            Action::Wait,
            Action::Ascend,
            Action::Descend,
            Action::ToggleLantern,
            Action::HoodLantern,
            Action::ThrowVoice(Pos::new(12, 5)),
            Action::ThrowVoice(Pos::new(0, 40)),
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
            Action::PickUp(Positions::default()),
            Action::PickUp(Positions::of([0])),
            Action::PickUp(Positions::of([0, 2, 25])),
            Action::Drop(Positions::default()),
            Action::Drop(Positions::of([1])),
            Action::Drop(Positions::of(0..LETTERS)),
            Action::Drop(Positions::of([0, 26, 140])),
            Action::PickUp(Positions::of([27])),
            Action::Sell(Positions::default()),
            Action::Sell(Positions::of([0, 3])),
            Action::Pay(None),
            Action::Pay(Some(Dir::W)),
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
            Action::Identify(Positions::default()),
            Action::Identify(Positions::of([0, 3])),
            Action::Ask {
                dir: Dir::NW,
                topic: "the_rooms".into(),
            },
            Action::Eat(Meal::Pack(1)),
            Action::Eat(Meal::Floor(0)),
            Action::Eat(Meal::Floor(25)),
            Action::Eat(Meal::Pack(26)),
            Action::Eat(Meal::Floor(31)),
            Action::Read {
                item: 1,
                target: None,
            },
            Action::Read {
                item: 0,
                target: Some(ReadTarget::Cell(Pos::new(12, 5))),
            },
            Action::Read {
                item: 3,
                target: Some(ReadTarget::Item(2)),
            },
            Action::Read {
                item: 27,
                target: Some(ReadTarget::Item(30)),
            },
            Action::Throw {
                item: 40,
                at: Pos::new(1, 2),
            },
            Action::Equip {
                slot: "body".into(),
                item: 26,
            },
            Action::Identify(Positions::of([26])),
            Action::Sell(Positions::of([3, 99])),
            Action::Afflict("bleeding".into()),
            Action::Afflict("Odd_Case".into()),
            Action::Infect("filth_fever".into()),
            Action::Infect("Odd_Case".into()),
            Action::Talent {
                id: "hardy".into(),
                choice: None,
            },
            Action::Talent {
                id: "weapon_focus".into(),
                choice: Some("long_blades".into()),
            },
            Action::Talent {
                id: "Odd_Case".into(),
                choice: Some("Mixed".into()),
            },
            Action::Advance("stealth".into()),
            Action::SecondWind,
            Action::LevelUpAuto,
            Action::GrantXp(2500),
            Action::Freeze {
                at: Pos::new(40, 12),
                radius: 3,
            },
            Action::Thaw {
                at: Pos::new(0, 7),
                radius: 0,
            },
            Action::Ask {
                dir: Dir::S,
                topic: "accept-rats".into(),
            },
            Action::Search(1),
            Action::Search(10),
            Action::Search(MAX_SEARCH),
        ];
        all.extend(Weather::ALL.map(Action::SetWeather));
        for d in Dir::ALL {
            all.extend([
                Action::Move(d),
                Action::Open(d),
                Action::Close(d),
                Action::Chat(d),
                Action::Steal(d),
                Action::Kick(d),
                Action::PickLock(d),
                Action::Bash(d),
                Action::Dig(d),
                Action::Disarm(d),
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
        assert_eq!(parse_actions("KICK:sw"), Ok(vec![Action::Kick(Dir::SW)]));
        assert!(parse_actions("kick").is_err());
        assert_eq!(parse_actions("PICK:e"), Ok(vec![Action::PickLock(Dir::E)]));
        assert!(parse_actions("pick").is_err());
        assert!(parse_actions("wait:E").is_err());
        assert_eq!(parse_actions("BASH:sw"), Ok(vec![Action::Bash(Dir::SW)]));
        assert!(parse_actions("bash").is_err());
        assert_eq!(parse_actions("Steal:N"), Ok(vec![Action::Steal(Dir::N)]));
        assert!(parse_actions("steal").is_err());
        assert_eq!(
            parse_actions("DISARM:ne"),
            Ok(vec![Action::Disarm(Dir::NE)])
        );
        assert!(parse_actions("disarm").is_err());
        assert_eq!(
            parse_actions("SEARCH search:1 search:10"),
            Ok(vec![
                Action::Search(1),
                Action::Search(1),
                Action::Search(10)
            ])
        );
        for bad in ["search:0", "search:101", "search:E", "search:"] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
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
            parse_actions("HOOD throw_voice:3,-4"),
            Ok(vec![
                Action::HoodLantern,
                Action::ThrowVoice(Pos::new(3, -4))
            ])
        );
        for bad in ["hood:E", "throw_voice", "throw_voice:E", "throw_voice:3"] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse_actions("FREEZE:12,5,4 thaw:3,4,0"),
            Ok(vec![
                Action::Freeze {
                    at: Pos::new(12, 5),
                    radius: 4
                },
                Action::Thaw {
                    at: Pos::new(3, 4),
                    radius: 0
                },
            ])
        );
        for bad in [
            "freeze",
            "freeze:12,5",
            "freeze:12,5,21",
            "thaw:1,2,-1",
            "thaw:a,b,c",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
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
                Action::PickUp(Positions::of([0, 2])),
                Action::Drop(Positions::default()),
                Action::PickUp(Positions::default()),
                Action::Drop(Positions::of([1])),
            ])
        );
        assert_eq!(
            Action::PickUp(Positions::of([2, 0])).to_string(),
            "pickup:ac"
        );
        // Any position as a number: the same stack as its letter, and past `z` too.
        assert_eq!(parse_actions("drop:2,0"), parse_actions("drop:ac"));
        assert_eq!(
            parse_actions("eat:1 equip:body:1"),
            parse_actions("eat:b equip:body:b")
        );
        assert_eq!(
            parse_actions("EAT:Floor:27 throw:30:4,5 drop:26"),
            Ok(vec![
                Action::Eat(Meal::Floor(27)),
                Action::Throw {
                    item: 30,
                    at: Pos::new(4, 5)
                },
                Action::Drop(Positions::of([26])),
            ])
        );
        for bad in [
            "eat:",
            "eat:ab",
            "throw:-1:4,5",
            "equip:body:",
            "drop:a,,b",
            "pickup:1x",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
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
            parse_actions("CHAT:ne ask:E:Rooms"),
            Ok(vec![
                Action::Chat(Dir::NE),
                Action::Ask {
                    dir: Dir::E,
                    topic: "rooms".into()
                }
            ])
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
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse_actions("EAT:B eat:Floor:a"),
            Ok(vec![
                Action::Eat(Meal::Pack(1)),
                Action::Eat(Meal::Floor(0))
            ])
        );
        for bad in [
            "eat",
            "eat:",
            "eat:ab",
            "eat:floor",
            "eat:floor:",
            "eat:floor:ab",
            "eat:pack:a",
            "read",
            "read:",
            "read:ab",
            "read:a:",
            "read:a:bc",
            "read:a:1,b",
            "read:a:1,2,3",
            "fire",
            "fire:",
            "fire:3",
            "fire:a,b",
            "throw",
            "throw:3,4",
            "throw:a",
            "throw:ab:3,4",
            "equip",
            "equip:body",
            "equip:body:",
            "equip::b",
            "equip:body:ab",
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
            "pay:Q",
            "sell:",
            "sell:A1",
            "drop:A-",
            "drop:",
            "rest",
            "rest:noon",
            "rest:0m",
            "rest:25h",
            "skip",
            "skip:0s",
            "skip:x",
            "chat",
            "chat:up",
            "ask",
            "ask:E",
            "ask:E:",
            "ask:up:rooms",
            "ask:E:a:b",
            "dig",
            "dig:",
            "dig:down",
        ] {
            assert!(parse_actions(bad).is_err(), "{bad}");
        }
    }
}
