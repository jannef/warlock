//! The game clock in the turn pipeline (spec §8 "Time"): how long actions take, resting,
//! skipping. What actions cost and how speed scales it: `cost_tests.rs`.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
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
id = "water"
name = "shallow water"
glyph = "~"
fg = "#4060c0"
move_cost = 200

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#a06030"
closes_to = "door_closed"

[[terrain]]
id = "stairs_down"
name = "staircase down"
glyph = ">"
fg = "#ffffff"
stairs = "down"

[[terrain]]
id = "stairs_up"
name = "staircase up"
glyph = "<"
fg = "#ffffff"
stairs = "up"
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "rat"
name = "rat"
glyph = "r"
fg = "#806040"
weight = 4000
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"~" = "water"
"+" = "door_closed"
"'" = "door_open"
"##;

// A room with a pool, a door in its east wall, and stairs down to a cellar. Below the room, out
// of sight, a rat walks up a corridor into the room and back.
const CAMP: &str = r##"
name = "Camp"
map = '''
##########
#@.~...>.#
#........+
######.###
     #.#
     #.#
     #.#
     #.#
     #r#
     ###
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat", route = "N6 S6" }
">" = { terrain = "stairs_down", to = "cellar" }
"##;

const CELLAR: &str = r##"
name = "Cellar"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "camp" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    files.insert("levels/camp.toml", CAMP);
    files.insert("levels/cellar.toml", CELLAR);
    // A hall far longer than the view range; the rat starts 94 cells from the player.
    let row = format!("#@{}r{}#", ".".repeat(93), ".".repeat(5));
    let wall = "#".repeat(row.len());
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nmap = '''\n{wall}\n{row}\n{wall}\n'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\", route = \"W60 E60\" }}\n"
        ),
    );
    files.insert(
        "scenarios/camp.toml",
        "name = \"Camp\"\nstart_level = \"camp\"\nstart_date = [1, 4, 12]\nstart_time = \"14:30\"\n",
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn last_message(world: &World) -> String {
    world
        .messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

/// A time on the camp scenario's start month (Rainmoon, year 1).
fn at(day: u32, hour: u32, minute: u32) -> GameTime {
    GameTime::from_calendar(1, 4, day, hour, minute).expect("valid date")
}

fn secs(s: u64) -> Duration {
    Duration::seconds(s)
}

/// Waits until a creature comes into view; returns how many turns that took.
fn waits_until_a_creature_shows(world: &mut World) -> u64 {
    while world.creatures_in_view().is_empty() {
        world.apply(&Action::Wait);
        assert!(world.turn() < 1000, "no creature ever came into view");
    }
    world.turn()
}

#[test]
fn the_clock_starts_at_the_scenario_start() {
    assert_eq!(world("camp").time(), at(12, 14, 30));
    assert_eq!(world("hall").time().to_string(), "0001-01-01 08:00:00");
}

#[test]
fn actions_take_six_seconds_and_steps_the_terrain_move_cost() {
    let mut w = world("camp");
    let out = run(&mut w, "move:E move:E move:E wait");
    let elapsed: Vec<_> = out.iter().map(|o| o.elapsed).collect();
    // Floor, shallow water (twice as long), floor, a wait.
    assert_eq!(elapsed, [secs(6), secs(12), secs(6), secs(6)]);
    assert!(out.iter().all(|o| o.took_turn));
    assert_eq!(w.turn(), 4, "a turn is one turn however long it takes");
    assert_eq!(w.time(), at(12, 14, 30) + secs(30));
}

#[test]
fn refused_and_free_actions_take_no_time() {
    let mut w = world("camp");
    for out in run(&mut w, "move:N open:W close:W ascend lantern vision:normal") {
        assert!(!out.took_turn);
        assert_eq!(out.elapsed, Duration::ZERO);
    }
    assert_eq!((w.turn(), w.time()), (0, at(12, 14, 30)));
}

#[test]
fn doors_and_stairs_take_a_normal_action_on_any_level() {
    let mut w = world("camp");
    run(&mut w, &format!("move:SE {}", "move:E ".repeat(6))); // (8, 2), west of the door
    let out = run(&mut w, "move:E close:E open:E");
    assert_eq!(w.player_pos(), Pos::new(8, 2), "bumping the door opens it");
    assert!(out.iter().all(|o| o.elapsed == secs(6)));

    let mut w = world("camp");
    run(&mut w, &"move:E ".repeat(6)); // onto the stairs at (7, 1)
    let out = run(&mut w, "descend wait move:E move:W ascend");
    assert!(out.iter().all(|o| o.elapsed == secs(6)));
    assert_eq!(w.state().current_level, "camp");
}

#[test]
fn resting_passes_turns_until_the_goal() {
    let mut w = world("camp");
    run(&mut w, &"move:E ".repeat(6));
    run(&mut w, "descend"); // The cellar: no creatures.
    let (turn, time) = (w.turn(), w.time());
    let rat = w.state().actors[&ActorId(1)].clone();

    let out = w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert!(out.took_turn);
    assert_eq!(out.elapsed, Duration::hours(1));
    assert_eq!(w.turn(), turn + 600, "one turn per six seconds");
    assert_eq!(last_message(&w), "You rest for 1 hour.");
    assert_eq!(
        w.state().actors[&ActorId(1)],
        rat,
        "other levels stand still"
    );

    let out = w.apply(&Action::Rest(Rest::Dusk));
    assert_eq!(w.time(), at(12, 18, 0));
    assert_eq!(out.elapsed, at(12, 18, 0).since(time + Duration::hours(1)));
    assert_eq!(last_message(&w), "You rest until dusk.");
    w.apply(&Action::Rest(Rest::Dawn));
    assert_eq!(w.time(), at(13, 6, 0), "dawn is the next morning");
    assert_eq!(last_message(&w), "You rest until dawn.");
    w.apply(&Action::Rest(Rest::Dawn));
    assert_eq!(
        w.time(),
        at(14, 6, 0),
        "at dawn, the next dawn is a day away"
    );
}

#[test]
fn resting_stops_on_the_turn_a_creature_comes_into_view() {
    // In the hall the rat is far enough away for turns to pass between perceptions; the rest
    // must still stop on the very turn it comes into view.
    assert_eq!(world("hall").out_of_view_for(), secs(6 * (94 - 41)));
    assert_eq!(world("camp").out_of_view_for(), Duration::ZERO);
    for scenario in ["camp", "hall"] {
        let mut waited = world(scenario);
        let turns = waits_until_a_creature_shows(&mut waited);
        assert!(turns > 1, "{scenario}: the rat starts out of view");

        let mut rested = world(scenario);
        let out = rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        assert_eq!(rested.turn(), turns, "{scenario}");
        assert_eq!(out.elapsed, secs(6 * turns));
        assert_eq!(rested.state().actors, waited.state().actors, "{scenario}");
        let rested_for = secs(6 * turns).spoken();
        assert_eq!(
            last_message(&rested),
            format!("A rat comes into view; you stop resting after {rested_for}.")
        );
        // Resting on with the rat in view goes on until it leaves and comes back.
        let before = rested.turn();
        rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        assert!(rested.turn() > before + 1, "{scenario}");
    }
}

#[test]
fn skipping_time_moves_only_the_clock() {
    let mut w = world("camp");
    let before = w.state().clone();
    let out = w.apply(&Action::SkipTime(Duration::hours(1)));
    assert!(!out.took_turn);
    assert_eq!(out.elapsed, Duration::hours(1));
    assert_eq!(w.time(), at(12, 15, 30));
    assert_eq!(w.turn(), 0);
    assert_eq!(w.state().actors, before.actors, "nobody acts");
    assert_eq!(last_message(&w), "Time skips ahead 1 hour (dev).");
}

#[test]
fn time_is_part_of_the_state_and_replays_reproduce_it() {
    let actions = "move:E move:E rest:1h skip:2h move:E rest:dawn wait";
    let mut a = world("camp");
    let mut replay = Replay::new(&a);
    for action in parse_actions(actions).expect("actions") {
        a.apply(&action);
        replay.push(&action, &a);
    }
    let played = replay.run(content()).expect("replay verifies");
    assert_eq!(played.time(), a.time());
    assert_eq!(played.state(), a.state());

    // The state hash covers the clock.
    let mut b = World::from_state(content(), a.state().clone()).expect("restores");
    assert_eq!(b.time(), a.time());
    let mut state = a.state().clone();
    state.time += secs(1);
    assert_ne!(state.hash(), a.state_hash());
    a.apply(&Action::Wait);
    b.apply(&Action::Wait);
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn move_costs_are_validated_and_levels_set_no_turn_length() {
    let with = |path: &str, from: &str, to: &str| {
        let mut files = files();
        let text = match path {
            "data/terrain.toml" => TERRAIN,
            _ => CELLAR,
        };
        files.insert(path, &text.replace(from, to));
        Content::load(&files)
    };
    let err = with("data/terrain.toml", "move_cost = 200", "move_cost = 0").unwrap_err();
    assert!(err.message.contains("move_cost 0"), "{}", err.message);
    // Six seconds is the one base: a level cannot have turns of its own any more.
    let err = with(
        "levels/cellar.toml",
        "name = \"Cellar\"",
        "name = \"Cellar\"\nturn_seconds = 10",
    )
    .unwrap_err();
    assert!(
        err.message.contains("unknown field `turn_seconds`"),
        "{}",
        err.message
    );
}
