//! A creature's presence (`aura.rs`, #1128): its aura's reach by sight and by hearing, its check
//! and its `every`; innate statuses refreshed and kept out of the look line; a cry heard by the
//! hearing rules, through a door and at Listen's threshold, and drawn on the crier's turn only.

use super::*;
use crate::content::ContentFiles;
use crate::noise::{Listening, SoundKind};

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"
toggle_noise = 60
muffle = 30

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#a06030"
closes_to = "door_closed"
toggle_noise = 70
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "undead"
name = "undead"
hostile = ["player"]

[[faction]]
id = "wild"
name = "wildlife"
"##;

const STATUS: &str = r##"
[[status]]
id = "aura"
name = "Aura spent"
max = 100
stacking = "refresh"
eases = 6

[[status]]
id = "chilled"
name = "Chilled"
seen = "chilled"
max = 10
stacking = "add"
eases = 600
start = "You feel the cold."

[[status]]
id = "cloaked"
name = "Cloaked"
seen = "cloaked in flame"
max = 1
stacking = "refresh"
eases = 60
"##;

/// `{aura}` is the haunt's `[creature.aura]` body, `{speed}` the crier's speed.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
hearing = 10

[[creature]]
id = "haunt"
name = "wraith"
glyph = "W"
fg = "#6a7a90"
weight = 100
faction = "undead"
lurks = true
base_hp = 1000
[creature.aura]
{aura}

[[creature]]
id = "salamander"
name = "salamander"
glyph = "S"
fg = "#e06020"
weight = 1000
faction = "undead"
lurks = true
innate = ["cloaked"]

[[creature]]
id = "crier"
name = "snake"
glyph = "s"
fg = "#60a040"
weight = 1000
faction = "wild"
lurks = true
speed = {speed}
cry = { kind = "hiss", loudness = 75, every = 6 }
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"@" = { terrain = "floor", spawn = "player" }
"W" = { terrain = "floor", creature = "haunt" }
"S" = { terrain = "floor", creature = "salamander" }
"s" = { terrain = "floor", creature = "crier" }
"##;

/// The wraith two cells east of the player, in plain sight.
const NEAR: &str = "########\n#@.W...#\n########\n";
/// The wraith three cells east of the player, past a closed door.
const DOOR: &str = "#######\n#@+.W.#\n#######\n";

const SIGHT: &str =
    "status = \"chilled\"\nradius = 2\nevery = 30\ntouch = \"A chill comes off {the}.\"";
const HEARD: &str = "status = \"chilled\"\nradius = 3\nheard = true\nloudness = 100\n\
                     touch = \"{the} wails.\"\nunseen = \"A wail rises through the walls.\"";

fn files(aura: &str, speed: u32, map: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    let creatures = CREATURES
        .replace("{aura}", aura)
        .replace("{speed}", &speed.to_string());
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/status.toml", STATUS),
        ("data/creatures.toml", creatures.as_str()),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    files
}

fn world_with(aura: &str, map: &str) -> World {
    let content = Arc::new(Content::load(&files(aura, 1000, map)).expect("valid content"));
    World::new(content, "hall", None).expect("world")
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

fn first_of<'w>(w: &'w World, kind: &str) -> &'w Actor {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == id)
        .expect("one of it")
}

/// Degrees of `id` on the player.
fn on_player(w: &World, id: &str) -> u16 {
    let s = status(w, id);
    w.state.player().effects.get(s).map_or(0, |e| e.severity)
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

#[test]
fn a_sight_aura_touches_the_hostile_within_its_radius_and_names_its_bearer() {
    let mut w = world_with(SIGHT, NEAR);
    w.apply(&Action::Wait);
    assert_eq!(on_player(&w, "chilled"), 1);
    let said = said(&w);
    let touch = said
        .iter()
        .position(|m| m == "A chill comes off the wraith.");
    let start = said.iter().position(|m| m == "You feel the cold.");
    assert!(touch.is_some() && touch < start, "{said:?}");
    // 30 s between touches at 6 s a degree: five degrees of the cooldown, one perhaps eased.
    let spent = first_of(&w, "haunt").effects.get(status(&w, "aura"));
    assert!(spent.is_some_and(|e| (4..=5).contains(&e.severity)));
}

#[test]
fn an_aura_touches_once_every_every() {
    let mut w = world_with(SIGHT, NEAR);
    w.apply(&Action::Wait);
    let start = w.time();
    let mut touches = vec![(0, on_player(&w, "chilled"))];
    while w.time().since(start).as_seconds() < 100 {
        w.apply(&Action::Wait);
        touches.push((w.time().since(start).as_seconds(), on_player(&w, "chilled")));
    }
    // Chilled adds up and lasts: a degree for each touch, one every 30 s.
    let (elapsed, degrees) = *touches.last().expect("waited");
    assert_eq!(u64::from(degrees), elapsed / 30 + 1, "{touches:?}");
    // No second degree before its 30 s were up.
    assert!(
        touches.iter().all(|&(t, d)| t >= 24 || d == 1),
        "{touches:?}"
    );
}

#[test]
fn out_of_its_radius_or_its_sight_it_touches_no_one() {
    let mut w = world_with(SIGHT, "#########\n#@...W..#\n#########\n");
    w.apply(&Action::Wait);
    assert_eq!(on_player(&w, "chilled"), 0, "three cells off");
    let mut w = world_with(&SIGHT.replace("radius = 2", "radius = 3"), DOOR);
    w.apply(&Action::Wait);
    assert_eq!(
        on_player(&w, "chilled"),
        0,
        "a closed door blocks the sight"
    );
    assert!(
        first_of(&w, "haunt").effects.is_empty(),
        "no one reached: no cooldown"
    );
}

#[test]
fn a_heard_aura_reaches_through_a_door_and_says_so_unseen() {
    let mut w = world_with(HEARD, DOOR);
    w.apply(&Action::Wait);
    assert_eq!(on_player(&w, "chilled"), 1);
    assert!(said(&w).contains(&"A wail rises through the walls.".to_string()));
    // Too quiet to carry through the door, it does not.
    let mut w = world_with(&HEARD.replace("loudness = 100", "loudness = 60"), DOOR);
    w.apply(&Action::Wait);
    assert_eq!(on_player(&w, "chilled"), 0);
}

#[test]
fn a_check_made_keeps_it_off() {
    let easy = format!("{SIGHT}\ncheck = \"willpower\"\ndifficulty = 1");
    let hard = format!("{SIGHT}\ncheck = \"willpower\"\ndifficulty = 40");
    let (mut kept, mut took) = (0, 0);
    for seed in 0..20 {
        for (aura, count) in [(&easy, &mut kept), (&hard, &mut took)] {
            let content = Arc::new(Content::load(&files(aura, 1000, NEAR)).expect("content"));
            let mut w = World::new(content, "hall", Some(seed)).expect("world");
            w.apply(&Action::Wait);
            *count += u32::from(on_player(&w, "chilled") == 0);
        }
    }
    // Only a natural 2 fails an easy check, only a natural 24 makes a hard one.
    assert!(kept >= 18, "kept off {kept} of 20");
    assert!(took <= 2, "kept off {took} of 20");
}

#[test]
fn innate_statuses_come_back_each_turn_and_never_show() {
    let mut w = world_with(SIGHT, "########\n#@..S..#\n########\n");
    let cloaked = status(&w, "cloaked");
    let id = first_of(&w, "salamander").id;
    assert!(
        first_of(&w, "salamander").effects.get(cloaked).is_some(),
        "made with it"
    );
    w.state
        .actors
        .get_mut(&id)
        .expect("it")
        .effects
        .remove(cloaked);
    w.apply(&Action::Wait);
    assert!(
        first_of(&w, "salamander").effects.get(cloaked).is_some(),
        "put back"
    );
    let view = w.perception();
    assert!(
        view.effects_of(id).next().is_none(),
        "its kind always has it"
    );
}

/// A world with the snake (at `speed`) behind a closed door: its hiss 75 loud there comes to
/// the player at 5, under their hearing of 10.
fn snake_behind_a_door(speed: u32) -> World {
    let map = "#######\n#@+..s#\n#######\n";
    let content = Arc::new(Content::load(&files(SIGHT, speed, map)).expect("content"));
    World::new(content, "hall", None).expect("world")
}

fn hears_a_hiss(w: &World) -> bool {
    (w.perception().sounds().iter()).any(|s| s.kind == SoundKind::Hiss)
}

#[test]
fn a_cry_is_heard_by_the_hearing_rules_through_a_door_and_at_listen() {
    let mut w = snake_behind_a_door(1000);
    w.apply(&Action::Wait);
    assert!(!hears_a_hiss(&w), "too faint through the door");
    // Pressed to the door: the threshold 6 lower for what comes by it.
    let mut w = snake_behind_a_door(1000);
    w.state.listening = Some(Listening {
        at: Pos::new(2, 1),
        keener: 6,
    });
    w.apply(&Action::Wait);
    assert!(hears_a_hiss(&w));
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "You faintly hear a hiss to the east."),
        "{:?}",
        said(&w)
    );
}

#[test]
fn a_cry_is_drawn_on_the_criers_turn_only() {
    // Every action of its an idle one, a slow snake (a tenth of the player's pace) hisses on its
    // own turns alone: never two turns running.
    let mut w = snake_behind_a_door(100);
    let mut heard = Vec::new();
    for turn in 0..40 {
        w.state.listening = Some(Listening {
            at: Pos::new(2, 1),
            keener: 6,
        });
        w.apply(&Action::Wait);
        if hears_a_hiss(&w) {
            heard.push(turn);
        }
    }
    assert!((3..=5).contains(&heard.len()), "{heard:?}");
    assert!(heard.windows(2).all(|p| p[1] - p[0] >= 5), "{heard:?}");
}
