//! Running (#1325, `run.rs`): in the open to the wall, along a corridor through its bends to a
//! room, stopping at an intersection and at a side door, and for whatever stops a rest (a hostile
//! coming within [`World::stop_within`] in view; never one farther off).

use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::test_support::last_text as last;
use crate::world::*;

/// The built-in content with `map`, lit: the player at `@`, a kobold at `k`, fences
/// (transparent, not walkable, never named as a statue is) at `F`.
fn world(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_run.toml",
        &format!(
            "name = \"Test\"\nambient = [200, 200, 200, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n\
             \"F\" = \"fence\"\n"
        ),
    );
    files.insert(
        "scenarios/test_run.toml",
        "name = \"T\"\nstart_level = \"test_run\"\nhp = 400\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_run", Some(1)).expect("world")
}

fn run(w: &mut World, dir: Dir) -> Outcome {
    w.apply(&Action::Run(dir))
}

#[test]
fn in_the_open_a_run_holds_its_direction_to_the_wall() {
    let mut w = world(
        "\
##########
#........#
#@.......#
#........#
##########
",
    );
    let said = w.messages().iter().count();
    let outcome = run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(8, 2), "beside the east wall");
    assert!(outcome.took_turn);
    assert_eq!(w.messages().iter().count(), said, "nothing to say");
    // Against the wall, a run is a plain step: the wall is in the way, and no time passes.
    let turn = w.state.turn;
    run(&mut w, Dir::E);
    assert_eq!(w.state.turn, turn);
    assert_eq!(last(&w), "The stone wall blocks your way.");
}

#[test]
fn a_diagonal_run_in_the_open_stops_at_the_wall_too() {
    let mut w = world(
        "\
##########
#@.......#
#........#
#........#
##########
",
    );
    run(&mut w, Dir::SE);
    assert_eq!(w.player_pos(), Pos::new(3, 3));
}

/// A corridor out of the player's cell east, bending south, east, south again, a stretch two
/// cells wide at a diagonal step, then into a room.
const BENDS: &str = "\
##############
#@...#########
####.#########
####....######
#######.######
#######.######
####.......###
####.......###
##############
";

#[test]
fn along_a_corridor_a_run_follows_its_bends_to_the_room() {
    let mut w = world(BENDS);
    run(&mut w, Dir::E);
    assert_eq!(
        w.player_pos(),
        Pos::new(7, 5),
        "the corridor's last cell, the room opening before it"
    );
}

#[test]
fn a_run_stops_where_a_corridor_branches() {
    // A branch north off the corridor.
    let mut w = world(
        "\
#########
###.#####
#@......#
#########
",
    );
    run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(3, 2), "at the junction");
    // A T: the corridor meets one across it, at its mouth.
    let mut w = world(
        "\
#########
######.##
#@.....##
######.##
#########
",
    );
    run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(5, 2), "at the T's mouth");
}

#[test]
fn a_closed_side_door_is_a_way_on_and_stops_a_run() {
    let mut w = world(
        "\
#########
####+####
#@......#
#########
",
    );
    run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(4, 2), "beside the door");
    // With no door there, the run goes on to the corridor's end.
    let mut w = world(
        "\
#########
#########
#@......#
#########
",
    );
    run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(7, 2));
}

#[test]
fn a_closed_door_ahead_ends_a_run_and_is_not_opened() {
    let mut w = world(
        "\
#########
#@...+..#
#########
",
    );
    run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(4, 1));
    let door = w.content.terrain_id("door_closed").expect("a door");
    assert_eq!(w.state.current().terrain[Pos::new(5, 1)], door);
}

#[test]
fn a_hostile_coming_close_stops_a_run_as_it_stops_a_rest() {
    // The kobold is seen from the start, penned behind a fence; the run comes to it.
    let mut w = world(
        "\
####################
#@.................#
#FFFFFFFFFFFFFFFFFF#
#.............k....#
####################
",
    );
    run(&mut w, Dir::E);
    let kobold = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .expect("the kobold");
    assert!(kobold.pos.chebyshev(w.player_pos()) <= w.stop_within());
    assert!(w.player_pos().x < 18, "stopped short of the wall");
    assert_eq!(last(&w), "The kobold comes close; you stop running.");
}

#[test]
fn a_hostile_farther_off_in_view_does_not_stop_a_run() {
    // Fences keep the kobold more than five cells off wherever it goes; the player sees it.
    let mut w = world(
        "\
################
#@...F.....F..k#
#....F.....F...#
################
",
    );
    let said = w.messages().iter().count();
    run(&mut w, Dir::E);
    assert_eq!(w.player_pos(), Pos::new(4, 1), "to the fence");
    assert_eq!(w.messages().iter().count(), said);
}

#[test]
fn a_run_records_and_replays() {
    let mut w = world(BENDS);
    let mut replay = Replay::new(&w);
    for action in parse_actions("run:E run:N").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.player_pos(), Pos::new(1, 1), "back along the corridor");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
