//! Action costs (spec §8 "Time"): an action's cost and the actor's speed decide how long it
//! takes, `cost / 1000 × 1000 / speed × 6 s` (an overworld step: its travel time instead of
//! 6 s), exactly over any run of actions; resting ends exactly at its goal at any speed.

use super::*;
use crate::action::parse_actions;
use crate::content::{ActionCosts, ContentFiles, NORMAL_COST};
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
id = "packed_earth"
name = "packed earth"
glyph = ","
fg = "#806040"
move_cost = 90

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
id = "tower"
name = "tower"
glyph = "T"
fg = "#c0c0c0"
stairs = "down"
travel_minutes = 30
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "foes"
name = "foes"
hostile = ["player"]

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// A melee attack costs 1400, as two-weapon fighting might.
const ACTIONS: &str = r##"
melee = 1400
"##;

// The player's speed is filled in per test. The foes stand their ground next to the player and
// attack: the grunt with the melee cost, the brute with its own (12 s), the archer with the
// shooting cost, the shaman with its own (3 s).
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
speed = PLAYER_SPEED

[[creature]]
id = "walker"
name = "walker"
glyph = "w"
fg = "#806040"
weight = 1000
base_hp = 1000
faction = "wildlife"

[[creature]]
id = "grunt"
name = "grunt"
glyph = "g"
fg = "#60a040"
weight = 1000
base_hp = 1000
faction = "foes"

[[creature]]
id = "brute"
name = "brute"
glyph = "B"
fg = "#60a040"
weight = 1000
base_hp = 1000
faction = "foes"
attack_cost = 2000

[[creature]]
id = "archer"
name = "archer"
glyph = "a"
fg = "#a0a040"
weight = 1000
base_hp = 1000
faction = "foes"
combat = "ranged"
range = 3

[[creature]]
id = "shaman"
name = "shaman"
glyph = "s"
fg = "#a040a0"
weight = 1000
base_hp = 1000
faction = "foes"
combat = "caster"
range = 3
attack_cost = 500
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"," = "packed_earth"
"+" = "door_closed"
"'" = "door_open"
"##;

// A closed door east of the player, stairs down below them, a row of packed earth (a step takes
// 90 % of a normal one), and a walker of normal speed pacing its own row.
const YARD: &str = r##"
name = "Yard"
map = '''
############
#@......+..#
#>.........#
#,,,,,,,,,,#
#w.........#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
">" = { terrain = "stairs_down", to = "cellar" }
"w" = { terrain = "floor", creature = "walker", route = "E9 W9" }
"##;

const CELLAR: &str = r##"
name = "Cellar"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "yard" }
"##;

// The player surrounded by foes (ids 1 to 4: grunt, brute, archer, shaman).
const PIT: &str = r##"
name = "Pit"
map = '''
#######
#..g..#
#.B@a.#
#..s..#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"g" = { terrain = "floor", creature = "grunt" }
"B" = { terrain = "floor", creature = "brute" }
"a" = { terrain = "floor", creature = "archer", carry = ["bow", "arrow*30"], wield = ["bow", "arrow"] }
"s" = { terrain = "floor", creature = "shaman" }
"##;

// The archer's bow and arrows, and where they are readied.
const OBJECTS: &str = r##"
[[object]]
id = "bow"
name = "bow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
materials = ["wood"]
weight = 700
value = 0
slot = "missile_weapon"
launches = "arrow"
damage = "1d6"
range = 8

[[object]]
id = "arrow"
name = "arrow"
glyph = "/"
fg = "#c0c0c0"
class = "missiles"
materials = ["wood"]
weight = 30
value = 0
stackable = true
slot = "missiles"
ammo = "arrow"
"##;

const SLOTS: &str = r##"
[[slot]]
id = "missile_weapon"
name = "Missile weapon"
action = "ready"
mark = "readied"

[[slot]]
id = "missiles"
name = "Missiles"
action = "ready"
mark = "quivered"
"##;

// An overworld: a road east to a tower (a location leading into the keep), plains around.
const WILDS: &str = r##"
name = "Wilds"
kind = "overworld"
map = '''
......
.@==T.
......
'''
[legend]
"." = "plains"
"=" = "road"
"@" = { terrain = "road", spawn = "player" }
"T" = { terrain = "tower", to = "keep" }
"##;

const KEEP: &str = r##"
name = "Keep"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "wilds" }
"##;

fn files(player_speed: u32) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/actions.toml", ACTIONS);
    let creatures = CREATURES.replace("PLAYER_SPEED", &player_speed.to_string());
    files.insert("data/creatures.toml", &creatures);
    files.insert("data/legend.toml", LEGEND);
    files.insert("data/objects.toml", OBJECTS);
    files.insert("data/slots.toml", SLOTS);
    files.insert(
        "data/materials.toml",
        "[[material]]\nid = \"wood\"\nname = \"wood\"\n",
    );
    files.insert("levels/yard.toml", YARD);
    files.insert("levels/cellar.toml", CELLAR);
    files.insert("levels/pit.toml", PIT);
    files.insert("levels/wilds.toml", WILDS);
    files.insert("levels/keep.toml", KEEP);
    // A hall far longer than the view range; the walker starts 94 cells from the player and
    // walks towards them and back.
    let row = format!("#@{}w{}#", ".".repeat(93), ".".repeat(5));
    let wall = "#".repeat(row.len());
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nmap = '''\n{wall}\n{row}\n{wall}\n'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"w\" = {{ terrain = \"floor\", creature = \"walker\", route = \"W60 E60\" }}\n"
        ),
    );
    for name in ["yard", "pit", "wilds", "hall"] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\nhp = 1000\n"),
        );
    }
    files
}

fn content(player_speed: u32) -> Arc<Content> {
    Arc::new(Content::load(&files(player_speed)).expect("test content is valid"))
}

fn world(scenario: &str, player_speed: u32) -> World {
    World::new(content(player_speed), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| {
            let out = world.apply(a);
            assert!(out.took_turn, "{a}: {:?}", world.messages().iter().last());
            out
        })
        .collect()
}

fn elapsed(outs: &[Outcome]) -> Vec<u64> {
    outs.iter().map(|o| o.elapsed.as_seconds()).collect()
}

fn total(outs: &[Outcome]) -> u64 {
    elapsed(outs).iter().sum()
}

fn secs(s: u64) -> Duration {
    Duration::seconds(s)
}

fn player_energy(world: &World) -> i64 {
    world.state().player().energy
}

/// How often `id` did something that showed in the events: moved or attacked.
fn acts_of(outs: &[Outcome], id: ActorId) -> usize {
    let events = outs.iter().flat_map(|o| o.events.iter());
    events
        .filter(|e| match e {
            Event::ActorMoved { actor, .. } => *actor == id,
            Event::Attacked { attacker, .. } => *attacker == id,
            _ => false,
        })
        .count()
}

/// The attacks of actor `id` on the player among the outcomes' events: one per attack it made
/// on them (an arrow that misses the player flies on, and may strike someone behind them too).
fn attacks_on_player(outs: &[Outcome], id: ActorId) -> usize {
    let events = outs.iter().flat_map(|o| o.events.iter());
    events
        .filter(|e| {
            matches!(e, Event::Attacked { attacker, target, .. }
                if *attacker == id && *target == ActorId(0))
        })
        .count()
}

#[test]
fn a_normal_action_takes_six_seconds_at_normal_speed() {
    let mut w = world("yard", 1000);
    let start = w.time();
    // Waiting, steps, a door opened and closed and bumped open, the stairs down and up.
    let outs = run(
        &mut w,
        "wait move:E*6 open:E close:E move:E move:E move:W*7 move:S descend wait ascend",
    );
    assert!(
        elapsed(&outs).iter().all(|s| *s == 6),
        "{:?}",
        elapsed(&outs)
    );
    assert_eq!(w.time().since(start), secs(6 * 22));
    assert_eq!(player_energy(&w), 0, "nothing left over");
}

#[test]
fn an_action_of_cost_1400_takes_8_4_seconds_over_a_sequence() {
    // Attacking the grunt to the north, again and again: 8.4 s each, the clock in whole seconds,
    // the leftover carried, so five attacks take exactly 42 s and ten 84 s.
    let mut w = world("pit", 1000);
    let mut outs = run(&mut w, "move:N*5");
    assert_eq!(elapsed(&outs), [9, 8, 9, 8, 8]);
    assert_eq!(player_energy(&w), 0);
    let more = run(&mut w, "move:N*5");
    assert_eq!(total(&more), 42);
    outs.extend(more);
    assert_eq!(acts_of(&outs, w.player_id()), 10, "ten attacks");
    assert_eq!(w.player_pos(), Pos::new(3, 2), "attacks, not steps");
}

#[test]
fn steps_of_exactly_5_4_seconds_take_exactly_54_seconds_ten_at_a_time() {
    // Packed earth costs 90 % of a step: 5.4 s at normal speed.
    let mut w = world("yard", 1000);
    run(&mut w, "move:S");
    let outs = run(&mut w, "move:S move:E*9");
    assert_eq!(elapsed(&outs), [6, 5, 6, 5, 5, 6, 5, 6, 5, 5]);
    assert_eq!(total(&outs), 54);
    assert_eq!(player_energy(&w), 0, "no drift: nothing left over");
}

#[test]
fn a_step_at_speed_1111_takes_about_5_4_seconds_without_drift() {
    // 6000 / 1111 = 5.4005 s: on a clock of whole seconds a step takes 5 or 6 s, and a run of
    // steps exactly its total time, up to the second the player is ready again. Ten steps take
    // 54.005 s, so the player is ready after 55 s; 1111 steps take exactly 6000 s.
    let mut w = world("yard", 1111);
    let mut done = 0;
    for n in 1..=1111u64 {
        let dir = if n % 2 == 1 { Dir::E } else { Dir::W };
        let out = w.apply(&Action::Move(dir));
        assert!(out.took_turn);
        let step = out.elapsed.as_seconds();
        assert!((5..=6).contains(&step), "step {n} took {step} s");
        done += step;
        assert_eq!(done, (n * 6000).div_ceil(1111), "after {n} steps");
        let energy = player_energy(&w);
        assert!((0..1111).contains(&energy), "{energy}");
        if n == 10 {
            assert_eq!(done, 55, "54.005 s");
        }
    }
    assert_eq!(done, 6000);
    assert_eq!(player_energy(&w), 0);
}

#[test]
fn a_slower_player_takes_longer() {
    // At speed 900 a normal action takes 6.67 s: three take 20 s, nine a minute.
    let mut w = world("yard", 900);
    let outs = run(&mut w, "wait*3");
    assert_eq!(elapsed(&outs), [7, 7, 6]);
    let outs = run(&mut w, "wait*6");
    assert_eq!(total(&outs), 40);
    assert_eq!(player_energy(&w), 0);
}

#[test]
fn a_faster_player_gets_proportionally_more_done() {
    // At twice the speed the player acts twice for each action of the walker (normal speed).
    let walker = ActorId(1);
    let mut w = world("yard", 2000);
    let outs = run(&mut w, "wait*20");
    assert!(elapsed(&outs).iter().all(|s| *s == 3));
    assert_eq!(
        acts_of(&outs, walker),
        10,
        "a minute: 10 steps of the walker"
    );
    let mut w = world("yard", 1000);
    let outs = run(&mut w, "wait*10");
    assert_eq!(total(&outs), 60);
    assert_eq!(acts_of(&outs, walker), 10);
    // At 1500: three actions in the time of two.
    let mut w = world("yard", 1500);
    let outs = run(&mut w, "wait*30");
    assert_eq!(total(&outs), 120);
    assert_eq!(acts_of(&outs, walker), 20);
}

#[test]
fn creatures_attacks_use_their_costs() {
    // Waiting out 84 s among the foes: the grunt attacks every 8.4 s (melee costs 1400), the
    // brute every 12 s (its own 2000), the archer every 6 s (shooting costs 1000), the shaman
    // every 3 s (its own 500).
    let (grunt, brute, archer, shaman) = (ActorId(1), ActorId(2), ActorId(3), ActorId(4));
    let mut w = world("pit", 1000);
    let outs = run(&mut w, "wait*14");
    assert_eq!(total(&outs), 84);
    let counts: Vec<_> = [grunt, brute, archer, shaman]
        .iter()
        .map(|id| attacks_on_player(&outs, *id))
        .collect();
    assert_eq!(counts, [10, 7, 14, 28]);
    let content = w.content();
    let of = |id: &str| content.creature_id(id).expect("creature");
    assert_eq!(content.action_cost(of("grunt"), ActionKind::Melee), 1400);
    assert_eq!(content.action_cost(of("brute"), ActionKind::Melee), 2000);
    assert_eq!(content.action_cost(of("brute"), ActionKind::Wait), 1000);
    assert_eq!(content.action_cost(of("shaman"), ActionKind::Cast), 500);
    assert_eq!(content.action_cost(of("archer"), ActionKind::Shoot), 1000);
}

#[test]
fn overworld_steps_take_their_travel_time_scaled_by_speed_and_the_rest_six_seconds() {
    let minutes = |m| Duration::minutes(m);
    // Road, road, onto the tower (30 minutes), into the keep and out, a wait, onto the plains.
    let actions = "move:E move:E move:E descend ascend wait move:S";
    let mut w = world("wilds", 1000);
    let outs = run(&mut w, actions);
    let took: Vec<_> = outs.iter().map(|o| o.elapsed).collect();
    let expected = [20, 20, 30].map(minutes).to_vec();
    assert_eq!(took[..3], expected);
    assert_eq!(took[3..6], [secs(6); 3]);
    assert_eq!(took[6], minutes(45));
    // Twice as fast, everything takes half as long.
    let mut w = world("wilds", 2000);
    let took: Vec<_> = run(&mut w, actions).iter().map(|o| o.elapsed).collect();
    let expected = [10, 10, 15].map(minutes).to_vec();
    assert_eq!(took[..3], expected);
    assert_eq!(took[3..6], [secs(3); 3]);
    assert_eq!(took[6], secs(45 * 30));
}

#[test]
fn rest_ends_exactly_at_its_goal_at_any_player_speed() {
    for speed in [100, 900, 1000, 1111, 1500, 2000, 10_000] {
        // Steps on packed earth first, so the player may have energy left over.
        let mut w = world("yard", speed);
        run(&mut w, "move:S move:S move:E*3");
        let start = w.time();
        let out = w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        assert_eq!(out.elapsed, Duration::hours(1), "speed {speed}");
        assert_eq!(w.time(), start + Duration::hours(1), "speed {speed}");
        let energy = player_energy(&w);
        assert!(
            (0..i64::from(speed)).contains(&energy),
            "speed {speed}: ready then"
        );
        w.apply(&Action::Rest(Rest::Dusk));
        assert_eq!((w.time().hour(), w.time().minute()), (18, 0));
        assert_eq!(w.time().second(), 0, "speed {speed}");
        w.apply(&Action::Rest(Rest::Dawn));
        assert_eq!(w.time().second_of_day(), 6 * 3600, "speed {speed}");
        // Waiting goes on as before, with whatever energy is left over.
        let left = u64::try_from(player_energy(&w)).expect("ready");
        let out = w.apply(&Action::Wait);
        let wait = (6000 - left.min(6000)).div_ceil(u64::from(speed));
        assert_eq!(out.elapsed, secs(wait), "speed {speed}");
    }
}

#[test]
fn resting_stops_where_waiting_would_at_any_player_speed() {
    // In the hall the walker is far enough away for turns to pass between perceptions; at any
    // speed of the player the rest stops on the very turn it comes into view, in the same state.
    for speed in [900, 1111, 2000] {
        let mut waited = world("hall", speed);
        while waited.creatures_in_view().is_empty() {
            waited.apply(&Action::Wait);
            assert!(waited.turn() < 2000, "the walker never came into view");
        }
        let mut rested = world("hall", speed);
        let out = rested.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        assert_eq!(rested.turn(), waited.turn(), "speed {speed}");
        assert_eq!(rested.time(), waited.time(), "speed {speed}");
        assert_eq!(
            out.elapsed,
            waited.time().since(world("hall", speed).time())
        );
        assert_eq!(
            rested.state().actors,
            waited.state().actors,
            "speed {speed}"
        );
    }
}

#[test]
fn costs_and_speeds_are_deterministic_saved_and_replayed() {
    let mut w = world("yard", 1111);
    let mut replay = Replay::new(&w);
    let actions = "move:S move:S move:E*3 wait rest:10m move:W*3 move:N descend wait ascend move:E";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content(1111)).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let mut loaded = crate::save::decode(&crate::save::encode(&w), content(1111)).expect("loads");
    run(&mut w, "move:E*4 rest:1h");
    run(&mut loaded, "move:E*4 rest:1h");
    assert_eq!(loaded.state(), w.state());
}

#[test]
fn costs_come_from_data_and_are_validated() {
    // The built-in content: every action a normal one but strapping on body armor (3000), a chat
    // (500), the lantern's hood (500), climbing out of the water onto ice (2000), trying to
    // disarm a trap (2000), and a try at picking a lock (2000), and no
    // creature with an attack of its own cost but the marilith's six blades (500) and the
    // balor's sword and whip (700).
    let builtin = Content::builtin().expect("builtin content");
    for &kind in ActionKind::ALL {
        let expected = match kind {
            ActionKind::BodyArmor => 3 * NORMAL_COST,
            ActionKind::Chat | ActionKind::Hood => NORMAL_COST / 2,
            ActionKind::Climb | ActionKind::Disarm | ActionKind::PickLock => 2 * NORMAL_COST,
            _ => NORMAL_COST,
        };
        assert_eq!(builtin.action_costs().cost(kind), expected, "{kind:?}");
    }
    for c in builtin.creature_defs() {
        let want = match c.id.as_str() {
            "marilith" => Some(500),
            "balor" => Some(700),
            _ => None,
        };
        assert_eq!(c.attack_cost, want, "{}", c.id);
    }

    let load = |path: &str, text: &str| {
        let mut files = files(1000);
        files.insert(path, text);
        Content::load(&files)
    };
    let err = load("data/actions.toml", "sleep = 1000").unwrap_err();
    assert_eq!(err.file, "data/actions.toml");
    assert!(
        err.message.contains("unknown action `sleep`"),
        "{}",
        err.message
    );
    let err = load("data/actions.toml", "stairs = 50").unwrap_err();
    assert!(
        err.message.contains("cost 50 is out of range"),
        "{}",
        err.message
    );
    let creatures = CREATURES
        .replace("PLAYER_SPEED", "1000")
        .replace("attack_cost = 2000", "attack_cost = 20000");
    let err = load("data/creatures.toml", &creatures).unwrap_err();
    assert!(
        err.message
            .contains("creature `brute`: attack_cost: cost 20000 is out of range"),
        "{}",
        err.message
    );
    // Without the file every action costs 1000.
    let mut files = files(1000);
    files.remove("data/actions.toml");
    let content = Content::load(&files).expect("valid");
    assert_eq!(content.action_costs(), &ActionCosts::default());
}
