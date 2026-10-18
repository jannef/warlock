//! The speed system (spec §8 "Speed"): the energy scheduler decides who acts when, in time
//! order, so faster creatures act more often in the same game time.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use std::sync::LazyLock;

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

[[terrain]]
id = "plains"
name = "plains"
glyph = "."
fg = "#a0a060"
travel_minutes = 45

[[terrain]]
id = "road"
name = "road"
glyph = "="
fg = "#a08060"
travel_minutes = 20

[[terrain]]
id = "forest"
name = "forest"
glyph = "&"
fg = "#40a040"
travel_minutes = 90
"##;

// The player and "normal" have the default speed, 1000.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "normal"
name = "normal creature"
glyph = "n"
fg = "#806040"
weight = 1000

[[creature]]
id = "quick"
name = "quick creature"
glyph = "q"
fg = "#806040"
weight = 1000
speed = 1500

[[creature]]
id = "fast"
name = "fast creature"
glyph = "f"
fg = "#806040"
weight = 1000
speed = 2000

[[creature]]
id = "slow"
name = "slow creature"
glyph = "s"
fg = "#806040"
weight = 1000
speed = 900
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"~" = "water"
"##;

// Four creatures of different speeds on parallel lanes (ids 1 to 4, top to bottom), out of the
// player's way; a pool to wade through (steps take twice as long) and stairs down to a cellar.
// No vision file, so everything in line of sight is seen.
const TRACK: &str = r##"
name = "Track"
map = '''
##########################
#@.~~~...................#
#>.......................#
#n.......................#
#q.......................#
#f.......................#
#s.......................#
##########################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
">" = { terrain = "stairs_down", to = "cellar" }
"n" = { terrain = "floor", creature = "normal", route = "E20 W20" }
"q" = { terrain = "floor", creature = "quick", route = "E20 W20" }
"f" = { terrain = "floor", creature = "fast", route = "E20 W20" }
"s" = { terrain = "floor", creature = "slow", route = "E20 W20" }
"##;

const CELLAR: &str = r##"
name = "Cellar"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "track" }
"##;

// Two creatures of normal speed want the same cell, (3, 2), and so does the player below it.
// `a` comes first in the map, so it has the lower id although it stands to the right.
const CONTEST: &str = r##"
name = "Contest"
map = '''
#######
#...a.#
#.b...#
#..@..#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "normal", route = "SW1 NE1" }
"b" = { terrain = "floor", creature = "normal", route = "E1 W1" }
"##;

// An overworld: a road and a forest for the player to travel, and two creatures crossing the
// plains below them (45 minutes a step).
const WILDS: &str = r##"
name = "Wilds"
kind = "overworld"
map = '''
........
.@=====.
.&&&&&&.
........
q.......
f.......
'''
[legend]
"." = "plains"
"=" = "road"
"&" = "forest"
"@" = { terrain = "road", spawn = "player" }
"q" = { terrain = "plains", creature = "quick", route = "E7 W7" }
"f" = { terrain = "plains", creature = "fast", route = "E7 W7" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    files.insert("levels/track.toml", TRACK);
    files.insert("levels/cellar.toml", CELLAR);
    files.insert("levels/contest.toml", CONTEST);
    files.insert("levels/wilds.toml", WILDS);
    // A long hall: the fast creature starts 94 cells from the player in the first row, at
    // (95, 1), the slow one 59 cells away in the second, at (60, 2); both walk towards the
    // player and back.
    let fast = format!("#@{}f{}#", ".".repeat(93), ".".repeat(5));
    let slow = format!("#{}s{}#", ".".repeat(59), ".".repeat(40));
    let wall = "#".repeat(fast.len());
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nmap = '''\n{wall}\n{fast}\n{slow}\n{wall}\n'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"f\" = {{ terrain = \"floor\", creature = \"fast\", route = \"W60 E60\" }}\n\
             \"s\" = {{ terrain = \"floor\", creature = \"slow\", route = \"W55 E55\" }}\n"
        ),
    );
    for name in ["track", "contest", "wilds", "hall"] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\n"),
        );
    }
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world(scenario: &str) -> World {
    World::new(CONTENT.clone(), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

// The track's creatures, by speed.
const NORMAL: ActorId = ActorId(1);
const QUICK: ActorId = ActorId(2);
const FAST: ActorId = ActorId(3);
const SLOW: ActorId = ActorId(4);

/// Who moved, in order.
fn movers(out: &Outcome) -> Vec<ActorId> {
    out.events
        .iter()
        .filter_map(|e| match e {
            Event::ActorMoved { actor, .. } => Some(*actor),
            _ => None,
        })
        .collect()
}

fn moves(out: &Outcome, id: ActorId) -> usize {
    movers(out).iter().filter(|a| **a == id).count()
}

fn actor(world: &World, id: ActorId) -> &Actor {
    &world.state().actors[&id]
}

/// How far along its route a patrolling creature is.
fn patrol_step(world: &World, id: ActorId) -> usize {
    actor(world, id).patrol.as_ref().expect("patrols").step
}

fn speed(world: &World, id: ActorId) -> i64 {
    i64::from(world.content().creature(actor(world, id).creature).speed)
}

#[test]
fn speeds_come_from_content_and_the_player_has_normal_speed() {
    let w = world("track");
    assert_eq!(speed(&w, w.player_id()), 1000);
    let speeds: Vec<_> = [NORMAL, QUICK, FAST, SLOW]
        .iter()
        .map(|id| speed(&w, *id))
        .collect();
    assert_eq!(speeds, [1000, 1500, 2000, 900]);
    let builtin = Content::builtin().expect("builtin content");
    let of = |id: &str| builtin.creature(builtin.creature_id(id).unwrap()).speed;
    assert_eq!(of("player"), crate::content::NORMAL_SPEED);
    assert!(
        ["rat", "bat", "kobold", "deer"]
            .iter()
            .all(|c| of(c) != 1000)
    );
}

#[test]
fn speeds_are_validated() {
    for (speed, ok) in [
        (0, false),
        (99, false),
        (100, true),
        (10_000, true),
        (10_001, false),
    ] {
        let mut files = files();
        let creatures = CREATURES.replace("speed = 900", &format!("speed = {speed}"));
        files.insert("data/creatures.toml", &creatures);
        match Content::load(&files) {
            Ok(_) => assert!(ok, "speed {speed} accepted"),
            Err(e) => {
                assert!(!ok, "speed {speed} rejected: {}", e.message);
                assert!(
                    e.message
                        .contains(&format!("speed {speed} is out of range"))
                );
            }
        }
    }
}

#[test]
fn equal_speeds_act_as_before_player_first_then_by_id() {
    // Waiting: the creature with the lower id takes the cell, the other is blocked.
    let mut w = world("contest");
    let (a, b) = (ActorId(1), ActorId(2));
    let out = w.apply(&Action::Wait);
    assert_eq!(actor(&w, a).pos, Pos::new(3, 2));
    assert_eq!(actor(&w, b).pos, Pos::new(2, 2), "blocked: it waits");
    assert_eq!(movers(&out), [a]);
    // Stepping there first: the player acts before both creatures.
    let mut w = world("contest");
    let out = w.apply(&Action::Move(Dir::N));
    assert_eq!(w.player_pos(), Pos::new(3, 2));
    assert_eq!(actor(&w, a).pos, Pos::new(4, 1));
    assert_eq!(actor(&w, b).pos, Pos::new(2, 2));
    assert_eq!(movers(&out), [w.player_id()]);
    // Every turn each acts once (moving or blocked), in id order, and all are ready again when
    // the player is.
    let mut w = world("contest");
    for _ in 0..6 {
        let out = w.apply(&Action::Wait);
        assert!(movers(&out).windows(2).all(|p| p[0] < p[1]));
        assert!(w.state().actors.values().all(|a| a.energy == 0));
    }
}

#[test]
fn others_act_in_time_order_until_the_player_is_ready() {
    let mut w = world("track");
    // At the start everyone is ready: by id after the player; then the fast creature (ready
    // again after 3 s) and the quick one (after 4 s) before the player's turn is over at 6 s.
    let out = w.apply(&Action::Wait);
    assert_eq!(movers(&out), [NORMAL, QUICK, FAST, SLOW, FAST, QUICK]);
    assert_eq!(out.elapsed, Duration::seconds(6));
    // Next turn: the two ready at 6 s (after the player), then the slow one at 7 s, the quick
    // one at 8 s, and the fast one at 9 s.
    let out = w.apply(&Action::Wait);
    assert_eq!(movers(&out), [NORMAL, FAST, SLOW, QUICK, FAST]);
    assert_eq!(w.state().actors[&w.player_id()].energy, 0);
}

#[test]
fn twice_as_fast_acts_twice_a_turn_and_one_and_a_half_three_times_in_two() {
    let mut w = world("track");
    let outs = run(&mut w, &"wait ".repeat(12));
    for (turn, out) in outs.iter().enumerate() {
        assert_eq!(moves(out, NORMAL), 1, "turn {turn}");
        assert_eq!(moves(out, FAST), 2, "turn {turn}");
        let quick = if turn % 2 == 0 { 2 } else { 1 };
        assert_eq!(moves(out, QUICK), quick, "turn {turn}");
    }
    let slow: usize = outs.iter().map(|o| moves(o, SLOW)).sum();
    assert_eq!(slow, 11, "900 is 10.8 actions in 72 s");

    // A step into water takes 12 s: twice the actions.
    let mut w = world("track");
    let outs = run(&mut w, "move:E move:E");
    assert_eq!(outs[1].elapsed, Duration::seconds(12));
    let counts: Vec<_> = [NORMAL, QUICK, FAST, SLOW]
        .iter()
        .map(|id| moves(&outs[1], *id))
        .collect();
    assert_eq!(counts, [2, 3, 4, 2]);
}

/// Checks that each creature's actions are exactly accounted for by its speed and the time on
/// the level: its energy is all it gained less all it spent on steps of base time `step`, and
/// stays within one action of 0.
fn assert_proportional(w: &World, elapsed: Duration, id: ActorId, steps: i64, step: Duration) {
    let speed = speed(w, id);
    let gained = speed * i64::try_from(elapsed.as_seconds()).unwrap();
    let price = crate::content::price(crate::content::NORMAL_COST, step);
    let spent = steps * price;
    assert_eq!(actor(w, id).energy, gained - spent, "{id}: no drift");
    assert!(
        (-price..speed).contains(&(gained - spent)),
        "{id}: {steps} steps in {elapsed}"
    );
}

/// Plays `actions` `times` times, counting each patrolling creature's steps (they are never
/// blocked) from its route position.
fn play_counting_steps(w: &mut World, actions: &str, times: usize, ids: &[ActorId]) -> Vec<i64> {
    let actions = parse_actions(actions).expect("actions");
    let mut steps = vec![0; ids.len()];
    for _ in 0..times {
        for action in &actions {
            let before: Vec<_> = ids.iter().map(|id| patrol_step(w, *id)).collect();
            assert!(w.apply(action).took_turn, "{action}");
            for (i, id) in ids.iter().enumerate() {
                let len = actor(w, *id).patrol.as_ref().unwrap().route.len();
                steps[i] += ((patrol_step(w, *id) + len - before[i]) % len) as i64;
            }
        }
    }
    steps
}

#[test]
fn action_counts_stay_proportional_to_speed_over_long_spans() {
    // Steps of 6 and 12 seconds and waits, back and forth through the pool, 600 actions.
    let mut w = world("track");
    let start = w.time();
    let ids = [NORMAL, QUICK, FAST, SLOW];
    let walk = "move:E move:E move:E move:E wait move:W move:W move:W move:W wait";
    let steps = play_counting_steps(&mut w, walk, 60, &ids);
    let elapsed = w.time().since(start);
    assert_eq!(elapsed, Duration::seconds(60 * (6 * 5 + 12 * 5)));
    for (id, steps) in ids.iter().zip(steps) {
        assert_proportional(&w, elapsed, *id, steps, Duration::seconds(6));
    }
}

#[test]
fn action_counts_stay_proportional_on_the_overworld() {
    // Road (20 min) and forest (90 min) steps and 6-second waits; the creatures' steps across
    // the plains take 45 minutes.
    let mut w = world("wilds");
    let start = w.time();
    let ids = [ActorId(1), ActorId(2)];
    let journey = "move:E move:E move:E move:E move:E move:S move:W move:W move:W move:W move:W \
                   move:N wait";
    let steps = play_counting_steps(&mut w, journey, 20, &ids);
    let elapsed = w.time().since(start);
    assert_eq!(
        elapsed,
        Duration::minutes(20 * (6 * 20 + 6 * 90)) + Duration::seconds(20 * 6)
    );
    for (id, steps) in ids.iter().zip(steps) {
        assert_proportional(&w, elapsed, *id, steps, Duration::minutes(45));
    }
}

#[test]
fn creatures_on_other_levels_stay_frozen() {
    let mut w = world("track");
    run(&mut w, "wait move:S");
    let track = w.state().actors.clone();
    run(&mut w, "descend");
    assert_eq!(w.perception().level_id(), "cellar");
    run(&mut w, &"wait ".repeat(30));
    let others = |a: &BTreeMap<ActorId, Actor>| -> Vec<Actor> {
        a.values()
            .filter(|a| a.kind == ActorKind::Creature)
            .cloned()
            .collect()
    };
    assert_eq!(
        others(&w.state().actors),
        others(&track),
        "no energy, no moves"
    );
    // Back up, the climb (6 seconds) is all the time they get: no burst.
    let out = w.apply(&Action::Ascend);
    assert_eq!(out.elapsed, Duration::seconds(6));
    assert!((2..=3).contains(&moves(&out, FAST)), "{:?}", movers(&out));
    assert!((1..=2).contains(&moves(&out, NORMAL)), "{:?}", movers(&out));
}

#[test]
fn skipping_time_gives_nobody_energy() {
    let mut w = world("track");
    let before = w.state().actors.clone();
    w.apply(&Action::SkipTime(Duration::hours(1)));
    assert_eq!(w.state().actors, before);
    let out = w.apply(&Action::Wait);
    assert_eq!(moves(&out, FAST), 2, "no burst after the skip");
}

#[test]
fn resting_accounts_for_fast_creatures() {
    // The fast creature 94 cells away may take 53 steps before it could come into view, and has
    // the energy for them after 159 s; the slow one 59 cells away has 18 steps, 120 s.
    let w = world("hall");
    assert_eq!(w.out_of_view_for(), Duration::seconds(120));
    // Resting stops on the very turn waiting would see a creature come into view, in the same
    // state.
    let mut waited = world("hall");
    while waited.creatures_in_view().is_empty() {
        waited.apply(&Action::Wait);
        assert!(waited.turn() < 1000, "no creature ever came into view");
    }
    let mut rested = world("hall");
    let out = rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(rested.turn(), waited.turn());
    assert_eq!(out.elapsed, Duration::seconds(6 * waited.turn()));
    assert_eq!(rested.time(), waited.time());
    assert_eq!(rested.state().actors, waited.state().actors);
    assert_eq!(rested.creatures_in_view(), waited.creatures_in_view());
    // Resting on, they come and go; each rest stops like waiting would.
    for _ in 0..5 {
        let mut waited = rested.clone();
        let mut seen = waited.creatures_in_view();
        let start = waited.turn();
        loop {
            waited.apply(&Action::Wait);
            let now = waited.creatures_in_view();
            if now.iter().any(|a| !seen.contains(a)) || waited.turn() - start == 600 {
                break;
            }
            seen = now;
        }
        rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        assert_eq!(rested.state().actors, waited.state().actors);
        assert_eq!(rested.turn(), waited.turn());
    }
}

#[test]
fn energy_is_saved_hashed_and_replayed() {
    let mut a = world("track");
    run(&mut a, "wait move:E");
    assert!(a.state().actors.values().any(|a| a.energy < 0));
    let mut b = crate::save::decode(&crate::save::encode(&a), CONTENT.clone()).expect("loads");
    assert_eq!(b.state(), a.state());
    run(&mut a, "wait move:E move:E rest:1h");
    run(&mut b, "wait move:E move:E rest:1h");
    assert_eq!(b.state(), a.state());
    // The hash covers energy.
    let mut state = a.state().clone();
    state.actors.get_mut(&FAST).expect("fast").energy += 1;
    assert_ne!(state.hash(), a.state_hash());
    // A player who is not ready to act is not a state the game saves.
    let mut state = a.state().clone();
    let player = state.player;
    state.actors.get_mut(&player).expect("player").energy = -1;
    assert!(matches!(
        World::from_state(CONTENT.clone(), state),
        Err(WorldError::InconsistentState(_))
    ));

    // Replays reproduce every action of every creature.
    let mut w = world("track");
    let mut replay = Replay::new(&w);
    let actions = "move:E move:E wait rest:10m skip:1h move:W move:S move:W descend wait ascend";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(CONTENT.clone()).expect("replay verifies");
    assert_eq!(played.state(), w.state());
}

#[test]
fn speed_never_reaches_the_perception_snapshot() {
    let mut w = world("track");
    run(&mut w, "wait wait");
    let dump = crate::dump::json(&w, &[], false).to_string();
    assert!(dump.contains("seen_actors"));
    assert!(
        !dump.contains("speed") && !dump.contains("energy"),
        "{dump}"
    );
}
