//! Coarse turns while the player rests (spec §8 "Rest", `stride.rs`): an idle creature away
//! from a resting player strides at its own pace, a busy one goes slower, one near the player
//! acts turn by turn, and outside a rest nothing changes; the rest's look at who came into view
//! sees what a perception would.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use std::sync::LazyLock;

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
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]
"##;

// At normal speed, so a creature takes a step a turn, and sight 10.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 1000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 1000
base_hp = 500
faction = "vermin"
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"##;

/// A hall the player is walled off from, too far away to hear: the rat at (40, 1) walks 30
/// cells east and back. Nearby: the same with the rat five cells from the player on a short
/// round, within its sight all along.
fn levels() -> [(&'static str, String); 2] {
    let level = |rat: usize, route: &str| {
        let row = format!("#@#{}r{}#", ".".repeat(rat - 3), ".".repeat(86 - rat));
        let wall = "#".repeat(row.len());
        format!(
            "name = \"Hall\"\nmap = '''\n{wall}\n{row}\n{wall}\n'''\n[legend]\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\", route = \"{route}\" }}\n"
        )
    };
    [("far", level(40, "E30 W30")), ("near", level(6, "E2 W2"))]
}

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    for (name, level) in levels() {
        files.insert(&format!("levels/{name}.toml"), &level);
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\n"),
        );
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(scenario: &str) -> World {
    World::new(CONTENT.clone(), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

const RAT: ActorId = ActorId(1);

fn rat(world: &World) -> &Actor {
    &world.state.actors[&RAT]
}

#[test]
fn an_idle_creature_away_from_a_resting_player_strides_at_its_own_pace() {
    // A minute's rest: its first action is a whole stride, 20 steps, and it waits them out.
    let mut rested = world("far");
    run(&mut rested, "rest:1m");
    let mut waited = world("far");
    run(&mut waited, "wait*10");
    assert_eq!((rested.turn(), waited.turn()), (10, 10));
    assert_eq!(rat(&rested).pos, Pos::new(60, 1), "a stride at once");
    assert_eq!(
        rat(&waited).pos,
        Pos::new(50, 1),
        "outside a rest, a step a turn"
    );
    // Ten minutes: five strides are a hundred steps, as a hundred turns are.
    let mut rested = world("far");
    run(&mut rested, "rest:10m");
    let mut waited = world("far");
    run(&mut waited, "wait*100");
    assert_eq!(rested.state().actors[&RAT], waited.state().actors[&RAT]);
}

#[test]
fn a_busy_creature_away_from_a_resting_player_goes_slower() {
    // Off to look into a noise far east of it: off its routine, it steps as usual, at twenty
    // times the price.
    let busy = |w: &mut World| {
        let rat = w.state.actors.get_mut(&RAT).expect("the rat");
        rat.mind.noise = Some(Pos::new(85, 1));
        rat.mind.post = Some(rat.pos);
    };
    let mut rested = world("far");
    busy(&mut rested);
    run(&mut rested, "rest:1m");
    let mut waited = world("far");
    busy(&mut waited);
    run(&mut waited, "wait*10");
    assert_eq!(rat(&rested).pos, Pos::new(41, 1));
    assert_eq!(rat(&waited).pos, Pos::new(50, 1));
}

#[test]
fn near_the_player_a_rest_goes_turn_by_turn() {
    // Within its sight of the player (who is walled off), the rat takes every turn as if the
    // player waited.
    let mut rested = world("near");
    run(&mut rested, "rest:10m");
    let turns = rested.turn();
    let mut waited = world("near");
    run(&mut waited, &format!("wait*{turns}"));
    assert_eq!(rested.state().actors, waited.state().actors);
}

#[test]
fn coarse_turns_replay() {
    let mut a = world("far");
    let mut replay = Replay::new(&a);
    for action in parse_actions("rest:10m wait rest:1h").expect("actions") {
        a.apply(&action);
        replay.push(&action, &a);
    }
    let played = replay.run(CONTENT.clone()).expect("replay verifies");
    assert_eq!(played.state(), a.state());
}

/// The rest's look at who is in view ([`World::watched_in_sight`]) finds whom a perception
/// finds, in the arena's fight and the depths, turn after turn, with the line of sight it keeps.
#[test]
fn a_rest_sees_whom_a_perception_sees() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    for scenario in ["arena", "depths", "showcase"] {
        let mut w = World::new(content.clone(), scenario, None).expect("world");
        let mut kept = None;
        for turn in 0..150 {
            let action = if turn % 3 == 0 { "move:E" } else { "wait" };
            run(&mut w, action);
            if w.death().is_some() {
                break;
            }
            let seen = w.watched_in_sight(&mut kept);
            assert_eq!(seen, Some(w.watched_in_view()), "{scenario}, turn {turn}");
        }
    }
}
