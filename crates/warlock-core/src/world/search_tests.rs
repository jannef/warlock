//! Searching in the world (spec §8 "Searching"): secret doors that look like walls until found,
//! hidden traps found and known, the active and the passive check, who notices in passing, what
//! draws from the gameplay stream, searching for a while, and saves and replays.

use super::search::{Hidden, Look};
use super::*;
use crate::action::parse_actions;
use crate::combat;
use crate::content::{ContentFiles, TerrainId};
use crate::perception::CellView;
use crate::replay::Replay;

/// A hall on the built-in content, the player at `@` as `character` (`RACE,GENDER,CLASS`); `S` is
/// a secret door anyone finds (difficulty 1: all but a natural 2), `H` one nobody does (50: only a
/// natural 24), `T` timber wall (not stone), `t` a secret door in it, and `a` a hidden alarm trap.
fn world(map: &str, character: &str) -> World {
    let mut files = ContentFiles::builtin();
    // Long tests rest: nobody starves meanwhile.
    files.remove("data/hunger.toml");
    let legend = r#"
[legend]
"@" = { terrain = "floor", spawn = "player" }
"S" = { terrain = "door_closed", secret = 1 }
"H" = { terrain = "door_closed", secret = 50 }
"T" = "timber_wall"
"t" = { terrain = "door_closed", secret = 1 }
"a" = { terrain = "floor", trap = "alarm" }
"#;
    files.insert(
        "levels/search_hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{legend}"),
    );
    let (race, rest) = character.split_once(',').expect("race");
    let (gender, class) = rest.split_once(',').expect("gender and class");
    files.insert(
        "scenarios/search_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"search_hall\"\nseed = 7\ncharacter = {{ race = \
             \"{race}\", gender = \"{gender}\", class = \"{class}\" }}\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "search_hall", None).expect("world")
}

const WARRIOR: &str = "human,male,warrior";
const THIEF: &str = "human,female,thief";
const ELF: &str = "elf,female,warrior";
const DWARF: &str = "dwarf,male,warrior";

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn id(world: &World, terrain: &str) -> TerrainId {
    world.content().terrain_id(terrain).expect(terrain)
}

/// The gameplay stream after `checks` checks from where `world`'s is now.
fn after_checks(world: &World, checks: usize) -> crate::rng::Rng {
    let mut rng = world.state.rng.clone();
    for _ in 0..checks {
        combat::roll_natural(&mut rng);
    }
    rng
}

// A secret door in the wall between two rooms; the player in the west room, beside it.
const HALL: &str = "#######\n#@#...#\n#.S...#\n#.#...#\n#######\n";
const DOOR: Pos = Pos::new(2, 2);

#[test]
fn a_secret_door_is_a_wall_until_it_is_found() {
    let mut w = world(HALL, WARRIOR);
    let wall = id(&w, "wall");
    let door = id(&w, "door_closed");
    // It looks, blocks, and is remembered like the wall around it; only ground truth knows.
    assert_eq!(w.perception().cell(DOOR), CellView::Visible(wall));
    assert_eq!(w.ground_truth().0.terrain_at(DOOR), Some(wall));
    assert!(w.ground_truth().0.secret_at(DOOR).is_some());
    let glyph = |layer| crate::dump::layer_rows(&w, layer)[2].chars().nth(2);
    assert_eq!(glyph(crate::dump::Layer::Truth), Some('S'));
    assert_eq!(glyph(crate::dump::Layer::View), Some('#'));
    run(&mut w, "move:S move:E");
    assert_eq!(w.player_pos(), Pos::new(1, 2));
    assert_eq!(
        texts(&w).last().map(String::as_str),
        Some("The stone wall blocks your way.")
    );
    // A search finds it: it is its door now, remembered, and said.
    let out = run(&mut w, "search");
    assert!(out[0].took_turn);
    assert_eq!(
        texts(&w).last().map(String::as_str),
        Some("You find a secret door!")
    );
    let level = w.state.current();
    assert_eq!(
        (level.terrain[DOOR], level.memory[DOOR]),
        (door, Some(door))
    );
    assert!(level.secrets.is_empty());
    run(&mut w, "move:E move:E");
    assert_eq!(
        w.player_pos(),
        Pos::new(2, 2),
        "opened, then walked through"
    );
}

#[test]
fn checks_draw_only_when_something_hidden_is_in_reach() {
    // The warrior notices nothing in passing: waiting beside the secret door draws nothing.
    let mut w = world(
        "#######\n#@#...#\n#.H...#\n#.#...#\n#.#...#\n#######\n",
        WARRIOR,
    );
    let before = w.state.rng.clone();
    run(&mut w, "wait*3");
    assert_eq!(w.state.rng, before, "no passive check for the warrior");
    // A search checks it once a turn: two draws (the natural roll).
    let expected = after_checks(&w, 1);
    run(&mut w, "search");
    assert_eq!(w.state.rng, expected);
    // Out of reach, a search draws nothing.
    run(&mut w, "move:S*3");
    assert_eq!(w.player_pos(), Pos::new(1, 4));
    let before = w.state.rng.clone();
    run(&mut w, "search wait");
    assert_eq!(w.state.rng, before);
}

#[test]
fn nothing_hidden_nearby_means_nothing_drawn_however_long_one_searches() {
    let mut w = world("#######\n#@....#\n#.....#\n#######\n", THIEF);
    let before = w.state.rng.clone();
    let out = run(&mut w, "search:10 wait*5");
    assert_eq!(w.state.rng, before);
    assert_eq!(out[0].elapsed, crate::time::Duration::seconds(60));
    assert!(
        texts(&w).contains(&"You search for 1 minute and find nothing.".to_string()),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn who_notices_in_passing() {
    // Elves notice everything; dwarves what is on stone; the trained in Spot everything; the
    // rest nothing. The doors: S in a stone wall, t in a timber one.
    let stone = "#######\n#@#...#\n#.S...#\n#.#...#\n#######\n";
    let timber = "#TTTTTT\n#@T...T\n#.t...T\n#.T...T\nTTTTTTT\n";
    for (map, character, notices) in [
        (stone, ELF, true),
        (timber, ELF, true),
        (stone, DWARF, true),
        (timber, DWARF, false),
        (stone, THIEF, true),
        (timber, THIEF, true),
        (stone, WARRIOR, false),
    ] {
        let mut w = world(map, character);
        let before = w.state.rng.clone();
        run(&mut w, "wait");
        let found = w.state.current().secrets.is_empty();
        assert_eq!(found, notices, "{character} in {map}");
        assert_eq!(w.state.rng != before, notices, "{character}: drew a check");
        if notices {
            assert!(texts(&w).contains(&"You find a secret door!".to_string()));
        }
    }
}

#[test]
fn traps_are_detect_traps_to_find_and_the_rest_spot_and_search() {
    // A hidden alarm beside the player: Spot, however high, does not notice it in passing;
    // Detect traps, trained at all, does; the thief starts trained in it.
    let map = "#####\n#@a.#\n#...#\n#####\n";
    for (character, skill, notices) in [
        (WARRIOR, None, false),
        (WARRIOR, Some("spot"), false),
        (WARRIOR, Some("detect_traps"), true),
        (THIEF, None, true),
    ] {
        let mut w = world(map, character);
        if let Some(skill) = skill {
            let id = w.content().skill_id(skill).expect(skill);
            w.player_mut().skills.add(id, 1);
        }
        let before = w.state.rng.clone();
        run(&mut w, "wait");
        assert_eq!(w.state.rng != before, notices, "{character} with {skill:?}");
    }
    // The skill each check reads: Detect traps for a trap, both ways; Search and Spot for the
    // rest.
    let mut w = world(map, WARRIOR);
    let trap = Hidden::Trap(w.content().trap_id("alarm").expect("alarm"));
    let door = Hidden::Door(1);
    let all = |w: &World| {
        [Look::Active, Look::Passive]
            .map(|look| (w.search_modifier(look, trap), w.search_modifier(look, door)))
    };
    let raise = |w: &mut World, skill: &str| {
        let id = w.content().skill_id(skill).expect(skill);
        w.player_mut().skills.add(id, 200);
    };
    let [(active_trap, active_door), (passive_trap, passive_door)] = all(&w);
    raise(&mut w, "search");
    raise(&mut w, "spot");
    assert_eq!(
        all(&w),
        [
            (active_trap, active_door + 12),
            (passive_trap, passive_door + 12)
        ]
    );
    raise(&mut w, "detect_traps");
    assert_eq!(
        all(&w),
        [
            (active_trap + 12, active_door + 12),
            (passive_trap + 12, passive_door + 12)
        ]
    );
}

#[test]
fn a_found_trap_is_known_and_said() {
    let mut w = world("#####\n#@a.#\n#...#\n#####\n", WARRIOR);
    let at = Pos::new(2, 1);
    assert_eq!(w.perception().trap(at), None);
    run(&mut w, "search:10");
    let alarm = w.content().trap_id("alarm").expect("alarm");
    assert!(w.state.current().traps[&at].known);
    assert_eq!(w.state.current().trap_memory.get(&at), Some(&alarm));
    assert_eq!(w.perception().trap(at), Some(alarm));
    let said = texts(&w);
    assert!(
        said.contains(&"You notice a tripwire: an alarm trap.".to_string()),
        "{said:?}"
    );
    assert!(
        !said
            .iter()
            .any(|t| t.contains("find nothing") || t.contains("stop searching")),
        "a search that finds something says only that: {said:?}"
    );
    // Known, it asks no more checks.
    let before = w.state.rng.clone();
    run(&mut w, "search");
    assert_eq!(w.state.rng, before);
}

#[test]
fn searching_for_a_while_stops_at_once_when_something_is_found() {
    let mut w = world(HALL, WARRIOR);
    let out = run(&mut w, "search:10");
    assert!(w.state.current().secrets.is_empty());
    assert_eq!(
        out[0].elapsed,
        crate::time::Duration::seconds(6),
        "found on the first turn"
    );
}

#[test]
fn noticing_in_passing_stops_a_rest() {
    let mut w = world(HALL, ELF);
    // Found on the very first turn of the rest (a natural 2 aside).
    let out = run(&mut w, "rest:1h");
    assert!(w.state.current().secrets.is_empty());
    assert!(
        out[0].elapsed < crate::time::Duration::minutes(1),
        "{:?}",
        out[0].elapsed
    );
    assert!(texts(&w).contains(&"You find a secret door!".to_string()));
}

#[test]
fn the_cell_underfoot_is_never_checked() {
    // The elf steps onto the hidden alarm: it goes off, and is not "noticed" afterwards.
    let mut w = world("#####\n#@a.#\n#####\n", ELF);
    run(&mut w, "move:E");
    let said = texts(&w);
    assert!(said.contains(&"A loud bell rings!".to_string()), "{said:?}");
    assert!(
        !said.iter().any(|t| t.starts_with("You notice")),
        "{said:?}"
    );
}

#[test]
fn a_high_search_skill_reaches_farther_in_line_of_sight() {
    // Two cells east, in the open: a master searcher reaches it, a novice does not; behind a
    // wall, nobody does.
    let open = "#######\n#@.H..#\n#.....#\n#######\n";
    let walled = "#######\n#@#H..#\n#.#...#\n#######\n";
    let search = |w: &World| w.content().skill_id("search").expect("search");
    for (map, master, checks) in [(open, false, 0), (open, true, 1), (walled, true, 0)] {
        let mut w = world(map, WARRIOR);
        if master {
            let skill = search(&w);
            w.player_mut().skills.add(skill, 100);
        }
        let expected = after_checks(&w, checks);
        run(&mut w, "search");
        assert_eq!(w.state.rng, expected, "{map}: master {master}");
    }
}

#[test]
fn whatever_becomes_of_the_wall_takes_the_secret_with_it() {
    let mut w = world(HALL, WARRIOR);
    let rubble = id(&w, "rubble");
    w.set_terrain(DOOR, rubble);
    assert!(w.state.current().secrets.is_empty());
    // Found or dug through, nothing is left to find.
    let before = w.state.rng.clone();
    run(&mut w, "search");
    assert_eq!(w.state.rng, before);
}

#[test]
fn hidden_features_are_harder_deeper_down() {
    let content = Content::builtin().expect("content");
    let def = content.dungeon("depths").expect("depths");
    assert_eq!(def.hidden_at(1), 0);
    assert_eq!(def.hidden_at(8), 7 * i32::from(def.hidden_per_depth));
    let w = World::new(Arc::new(content), "depths", None).expect("world");
    assert_eq!(w.difficulty_of(super::search::Hidden::Door(14)), 14);
}

#[test]
fn secrets_save_and_replay() {
    let mut w = world(HALL, ELF);
    let content = w.content().clone();
    let mut replay = Replay::new(&w);
    let saved = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(saved.state(), w.state());
    assert_eq!(saved.state.current().secrets.len(), 1);
    for action in parse_actions("search move:S wait search:5").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn levels_without_secrets_save_as_before() {
    let w = world("#####\n#@..#\n#####\n", WARRIOR);
    let json = serde_json::to_string(w.state.current()).expect("serializes");
    assert!(!json.contains("secrets"), "{json}");
}

#[test]
fn levels_refuse_secrets_that_cannot_be() {
    for (legend, map, message) in [
        (
            r#""X" = { terrain = "floor", secret = true }"#,
            "#####\n#@X.#\n#####\n",
            "only an unlocked closed door",
        ),
        (
            r#""X" = { terrain = "door_closed", secret = true, locked = true }"#,
            "#####\n#@X.#\n#####\n",
            "only an unlocked closed door",
        ),
        (
            r#""X" = { terrain = "door_closed", secret = true }"#,
            ".....\n.@X..\n.....\n",
            "no wall beside it",
        ),
        (
            r#""X" = { terrain = "door_closed", secret = 51 }"#,
            "#####\n#@X.#\n#####\n",
            "secret difficulty 51",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        let spawn = r#""@" = { terrain = "floor", spawn = "player" }"#;
        files.insert(
            "levels/bad.toml",
            &format!("name = \"Bad\"\nmap = '''\n{map}'''\n[legend]\n{spawn}\n{legend}\n"),
        );
        let err = Content::load(&files).expect_err(legend);
        assert!(err.message.contains(message), "{legend}: {}", err.message);
    }
}
