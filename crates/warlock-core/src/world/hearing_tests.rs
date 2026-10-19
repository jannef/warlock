//! Hearing in the turn pipeline (spec §5 "Hearing"): who hears what, what the player perceives,
//! messages, and resting.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::noise::{Act, Heard, RECALL_TURNS, SoundKind};
use crate::replay::Replay;

const TERRAIN: &str = r##"
[[terrain]]
id = "void"
name = "rock"
glyph = " "
fg = "#000000"
walkable = false
transparent = false

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

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
footsteps = 50
hearing = 10

[[creature]]
id = "rat"
name = "rat"
glyph = "r"
fg = "#806040"
weight = 4000
footsteps = 80
hearing = 5

[[creature]]
id = "mole"
name = "mole"
glyph = "m"
fg = "#806040"
hearing = 5
weight = 1000
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"'" = "door_open"
"##;

// The player in a closet behind a closed door; a rat walks the hall outside, back and forth.
const CLOSET: &str = r##"
name = "Closet"
map = '''
##############################
#...........................r#
#############+################
            #@#
            ###
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat", route = "W26 E26" }
"##;

// A lit yard: a mole sits still, two rats walk side by side.
const YARD: &str = r##"
name = "Yard"
map = '''
###########
#@....m...#
#.........#
#..r......#
#..r......#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"m" = { terrain = "floor", creature = "mole" }
"r" = { terrain = "floor", creature = "rat", route = "E4 W4" }
"##;

fn files(creatures: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", creatures);
    files.insert("data/legend.toml", LEGEND);
    files.insert("levels/closet.toml", CLOSET);
    files.insert("levels/yard.toml", YARD);
    files.insert(
        "scenarios/closet.toml",
        "name = \"Closet\"\nstart_level = \"closet\"\n",
    );
    files.insert(
        "scenarios/yard.toml",
        "name = \"Yard\"\nstart_level = \"yard\"\n",
    );
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files(CREATURES)).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn heard(world: &World, id: u32) -> &[Heard] {
    &world.state.actors[&ActorId(id)].heard
}

fn hearing_messages(world: &World) -> Vec<String> {
    world
        .messages()
        .iter()
        .filter(|m| m.text.contains(" hear "))
        .map(|m| m.text.clone())
        .collect()
}

/// Turns (after each of `turns` waits) on which the player perceived a sound.
fn audible_turns(world: &mut World, turns: u64) -> Vec<u64> {
    let mut audible = Vec::new();
    for _ in 0..turns {
        world.apply(&Action::Wait);
        if !world.perception().sounds().is_empty() {
            audible.push(world.turn());
        }
    }
    audible
}

/// Runs of audible turns no more than [`RECALL_TURNS`] apart: what one would call one sound.
fn stretches(audible: &[u64]) -> Vec<u64> {
    let mut starts = Vec::new();
    let mut last = None;
    for &turn in audible {
        if last.is_none_or(|l| turn - l > RECALL_TURNS) {
            starts.push(turn);
        }
        last = Some(turn);
    }
    starts
}

#[test]
fn creatures_hear_noises_until_they_act() {
    let mut w = world("yard");
    // The player's step at (2, 1) is 4 cells from the mole (#1): 50 - 40 = 10 there.
    w.make_noise(w.player_id(), Act::Step(Pos::new(2, 1)));
    assert_eq!(
        heard(&w, 1),
        [Heard {
            kind: SoundKind::Footsteps,
            at: Pos::new(2, 1),
            loudness: 10
        }]
    );
    // The maker does not hear itself.
    assert!(heard(&w, 0).is_empty());
    // As the mole acts (it only sits), what it heard is spent; the rats' steps after it are new.
    w.apply(&Action::Wait);
    assert!(heard(&w, 1).iter().all(|h| h.at != Pos::new(2, 1)));
}

#[test]
fn a_listener_hears_what_reaches_its_threshold_and_nothing_fainter() {
    for (hearing, hears) in [(10, true), (11, false)] {
        let mole = CREATURES.replace(
            "glyph = \"m\"\nfg = \"#806040\"\nhearing = 5",
            &format!("glyph = \"m\"\nfg = \"#806040\"\nhearing = {hearing}"),
        );
        let content = Arc::new(Content::load(&files(&mole)).expect("content"));
        let mut w = World::new(content, "yard", None).expect("world");
        // 10 at the mole, as above.
        w.make_noise(w.player_id(), Act::Step(Pos::new(2, 1)));
        assert_eq!(!heard(&w, 1).is_empty(), hears, "hearing {hearing}");
    }
}

#[test]
fn creatures_hear_those_that_act_after_them() {
    let mut w = world("yard");
    w.apply(&Action::Wait);
    // Ids go by spawn order, row by row: #2 is the upper rat, #3 the lower. #2 acted first, so
    // it heard #3's step after it; #3 heard #2's step before its own action, which spent it.
    let (upper, lower) = (
        w.state.actors[&ActorId(2)].pos,
        w.state.actors[&ActorId(3)].pos,
    );
    assert_eq!((upper, lower), (Pos::new(4, 3), Pos::new(4, 4)));
    assert_eq!(
        heard(&w, 2),
        [Heard {
            kind: SoundKind::Footsteps,
            at: lower,
            loudness: 70
        }]
    );
    assert!(heard(&w, 3).is_empty());
    // The mole heard both, the player too (but sees them, so perceives none).
    assert_eq!(heard(&w, 1).len(), 2);
    assert_eq!(heard(&w, 0).len(), 2);
    assert!(w.perception().sounds().is_empty(), "seen sources are seen");
}

#[test]
fn the_player_perceives_an_unseen_patrol_roughly_and_never_who() {
    let mut w = world("closet");
    let audible = audible_turns(&mut w, 60);
    assert!(!audible.is_empty(), "the rat passes the door");
    let mut w = world("closet");
    for _ in 0..audible[0] {
        w.apply(&Action::Wait);
    }
    let view = w.perception();
    let rat = w.state.actors[&ActorId(1)].pos;
    let [sound] = view.sounds() else {
        panic!("one sound: {:?}", view.sounds());
    };
    assert_eq!(sound.kind, SoundKind::Footsteps);
    assert!(sound.pos.chebyshev(rat) <= 2, "{} vs {rat}", sound.pos);
    assert!(!view.is_visible(sound.pos));
    assert_eq!(view.sound_at(sound.pos), Some(sound));
    // What the snapshot carries is the kind, a place, and a volume: no actor.
    let json = serde_json::to_value(sound).expect("serializes");
    let keys: Vec<_> = json
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect();
    assert_eq!(keys, ["kind", "pos", "volume"]);
}

#[test]
fn a_deaf_player_hears_nothing() {
    let deaf = CREATURES.replacen("hearing = 10\n", "", 1);
    let content = Arc::new(Content::load(&files(&deaf)).expect("content"));
    let mut w = World::new(content, "closet", None).expect("world");
    assert!(audible_turns(&mut w, 60).is_empty());
    assert!(hearing_messages(&w).is_empty());
}

#[test]
fn a_patrol_heard_turn_after_turn_is_said_once_per_pass() {
    let mut w = world("closet");
    let audible = audible_turns(&mut w, 200);
    let passes = stretches(&audible);
    let said = hearing_messages(&w);
    assert!(passes.len() >= 4, "{audible:?}");
    assert_eq!(said.len(), passes.len(), "{said:?} for {audible:?}");
    assert!(audible.len() >= 3 * said.len(), "not one message a turn");
    for text in &said {
        assert!(
            text.ends_with("to the north-east.")
                || text.ends_with("to the north-west.")
                || text.ends_with("to the north."),
            "{text}"
        );
    }
}

#[test]
fn a_new_sound_stops_a_rest_and_a_known_one_does_not() {
    let mut waited = world("closet");
    let passes = stretches(&audible_turns(&mut waited, 200));

    let mut rested = world("closet");
    rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(
        rested.turn(),
        passes[0],
        "stops on the turn it is first heard"
    );
    let said: Vec<_> = rested.messages().iter().map(|m| m.text.clone()).collect();
    let n = said.len();
    assert!(said[n - 2].contains(" hear footsteps"), "{said:?}");
    assert!(
        said[n - 1].starts_with("You stop resting after"),
        "{said:?}"
    );
    // Resting on while the same rat is heard goes on until it comes back from afar.
    rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(rested.turn(), passes[1]);
}

#[test]
fn hearing_is_part_of_the_state_and_replays_reproduce_it() {
    let actions = "wait wait wait wait wait wait wait wait wait wait wait wait wait wait \
                   rest:1h open:N close:N wait";
    let mut a = world("closet");
    let mut replay = Replay::new(&a);
    for action in parse_actions(actions).expect("actions") {
        a.apply(&action);
        replay.push(&action, &a);
    }
    assert!(!a.state.recent_sounds.is_empty());
    let played = replay.run(content()).expect("replay verifies");
    assert_eq!(played.state(), a.state());
    // A restored world perceives the same sounds and goes on identically.
    let mut b = World::from_state(content(), a.state().clone()).expect("restores");
    assert_eq!(b.perception().sounds(), a.perception().sounds());
    for _ in 0..30 {
        a.apply(&Action::Wait);
        b.apply(&Action::Wait);
    }
    assert_eq!(a.state(), b.state());
}

#[test]
fn doors_are_heard_by_creatures() {
    let mut w = world("closet");
    // The rat walks west along the hall; open the door while it is near.
    let audible = audible_turns(&mut w, 60);
    let mut w = world("closet");
    for _ in 1..audible[0] {
        w.apply(&Action::Wait);
    }
    w.make_noise(w.player_id(), Act::OpenDoor(Pos::new(13, 2)));
    let rat = &w.state.actors[&ActorId(1)];
    assert!(
        rat.heard
            .iter()
            .any(|h| h.kind == SoundKind::DoorOpened && h.at == Pos::new(13, 2)),
        "{:?}",
        rat.heard
    );
}

#[test]
fn the_hub_s_gallery_patrol_is_heard_through_its_door() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let mut w = World::new(content, "showcase", None).expect("world");
    // Out of the start room and along the corridor to the patrol gallery's closed door (43, 11).
    let walk = format!(
        "move:N move:N move:N open:N move:N move:N {}",
        "move:E ".repeat(36)
    );
    for action in parse_actions(&walk).expect("actions") {
        w.apply(&action);
    }
    assert_eq!(w.player_pos(), Pos::new(43, 10));
    // The kobold comes along the gallery's north side, behind the door.
    let first = audible_turns(&mut w, 70)[0];
    let mut w2 = World::new(w.content().clone(), "showcase", None).expect("world");
    for action in parse_actions(&walk).expect("actions") {
        w2.apply(&action);
    }
    while w2.turn() < first {
        w2.apply(&Action::Wait);
    }
    let said = hearing_messages(&w2);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("footsteps to the south"), "{said:?}");
    let view = w2.perception();
    assert!(!view.sounds().is_empty());
    assert!(
        view.sounds().iter().all(|s| s.pos.y >= 12),
        "beyond the door"
    );
}
