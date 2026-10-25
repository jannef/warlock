//! Picking locks (spec §8 "Doors", #233): only those trained in Open lock may try; a try is a
//! Dexterity check plus Open lock against the lock's difficulty, deeper locks harder; a picked
//! lock's door opens (and its trap goes off at the picker); every try takes its time and may be
//! tried again; refusals take none; a status may block it; replays verify.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::check;
use crate::content::ContentFiles;
use crate::door::{LOCK_DIFFICULTY, Lock};
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::Stat;
use crate::time::Duration;

/// A lit hall: the player west of a locked door (`L`, the usual difficulty), a hard one (`H`), an
/// easy one (`E`), a trapped locked door (`T`), an unlocked closed door (`+`), and an open one.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#@L.#...#
#.H.#...#
#.E.#...#
#.T.#...#
#.+.#...#
#.'.#...#
#########
'''
[legend]
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"'" = "door_open"
"@" = { terrain = "floor", spawn = "player" }
"L" = { terrain = "door_closed", locked = true }
"H" = { terrain = "door_closed", locked = true, lock_difficulty = 50 }
"E" = { terrain = "door_closed", lock = "pantry", lock_difficulty = 1 }
"T" = { terrain = "door_closed", locked = true, trap = "spear" }
"##;

/// A status for the tests that keeps the player from picking locks.
const SHAKING: &str = r#"
[[status]]
id = "shaking"
name = "Shaking"
max = 1
eases = 60
blocks = ["pick_lock"]
"#;

/// The built-in content plus the hall and a status that blocks picking, with scenarios: `hall`
/// (average stats, untrained), `deft` (Dexterity 18), `thief` (a human thief: trained in Open
/// lock), and `deep` (the Depths 2); with `eager`, Open lock earns a mark on every try.
fn files(eager: bool) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    let (_, statuses) = files.table("status").next().expect("data/status.toml");
    let statuses = format!("{statuses}{SHAKING}");
    files.insert("data/status.toml", &statuses);
    for (name, extra) in [
        ("hall", "start_level = \"hall\"\nhp = 1000\n"),
        (
            "deft",
            "start_level = \"hall\"\nhp = 1000\nstats = { dexterity = 18 }\n",
        ),
        (
            "thief",
            "start_level = \"hall\"\ncharacter = { race = \"human\", gender = \"male\", \
             class = \"thief\" }\n",
        ),
        ("deep", "start_level = \"depths:2\"\n"),
    ] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\n{extra}"),
        );
    }
    if eager {
        let text = include_str!("../../../../assets/data/skills.toml");
        let eager = text.replace(
            "mark = [{ on = \"pick\", chance = 20 }]",
            "mark = [{ on = \"pick\", chance = 100 }]",
        );
        assert_ne!(eager, text, "Open lock earns marks on a try");
        files.insert("data/skills.toml", &eager);
    }
    files
}

fn content(eager: bool) -> Arc<Content> {
    Arc::new(Content::load(&files(eager)).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(false), scenario, None).expect("world")
}

fn open_lock(w: &World) -> SkillId {
    w.content().skill_id("open_lock").expect("Open lock")
}

/// The player trained in Open lock at `value`.
fn train(w: &mut World, value: u16) {
    let id = open_lock(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    all.push((id, Trained { value, marks: 0 }));
    *skills = all.into_iter().collect();
}

/// What the player adds to picking a lock: the Dexterity modifier and what Open lock gives.
fn modifier(w: &World) -> i32 {
    let me = w.state.player();
    me.effective_stats(w.content()).modifier(Stat::Dexterity)
        + me.skill_effect(w.content(), Effect::Pick)
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn terrain(w: &World, p: Pos) -> String {
    let (level, _) = w.ground_truth();
    w.content().terrain(level.terrain[p]).id.clone()
}

fn locked(w: &World, p: Pos) -> bool {
    w.ground_truth().0.locks.contains_key(&p)
}

/// The player walks down the west aisle to row `row`, beside the door there.
fn go_to_row(w: &mut World, row: i32) {
    while w.player_pos().y < row {
        w.apply(&Action::Move(Dir::S));
    }
    assert_eq!(w.player_pos(), Pos::new(1, row));
}

fn pick_lock_in_menu(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::PickLock)
        .map(|a| a.unavailable)
}

#[test]
fn only_those_trained_in_open_lock_may_pick_locks() {
    let mut w = world("hall");
    assert_eq!(pick_lock_in_menu(&w), None, "untrained: not in the menu");
    let out = w.apply(&Action::PickLock(Dir::E));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last_message(&w), "You do not know how to pick locks.");
    assert!(locked(&w, Pos::new(2, 1)));
    train(&mut w, 0);
    assert_eq!(pick_lock_in_menu(&w), Some(None), "trained at 0: usable");
    let open_lock = open_lock(&w);
    assert!(
        !(w.perception().abilities().iter())
            .any(|a| matches!(a.ability, Ability::Skill(id) if id == open_lock)),
        "Open lock is listed as its use, not as a skill"
    );
    // A thief is trained in it from the start.
    assert_eq!(pick_lock_in_menu(&world("thief")), Some(None));
}

#[test]
fn picking_nothing_or_an_unlocked_door_takes_no_time() {
    let mut w = world("hall");
    train(&mut w, 20);
    for dir in [Dir::S, Dir::N] {
        let out = w.apply(&Action::PickLock(dir));
        assert!(!out.took_turn, "{dir}");
        assert_eq!(last_message(&w), "There is no lock there.");
    }
    go_to_row(&mut w, 5);
    let out = w.apply(&Action::PickLock(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "That door is not locked.");
    w.apply(&Action::Move(Dir::S));
    w.apply(&Action::PickLock(Dir::E));
    assert_eq!(last_message(&w), "There is no lock there.", "an open door");
}

#[test]
fn a_try_is_a_dexterity_check_plus_open_lock_against_the_locks_difficulty() {
    // Dexterity 18 (+4) and Open lock 100 (+8) against the usual lock (14) and the hard one (50:
    // only a natural 24 makes it); then average and barely trained against the usual one.
    for (scenario, value, row, difficulty, plus, at_least, at_most) in [
        ("deft", 100, 1, i32::from(LOCK_DIFFICULTY), 4 + 8, 36, 40),
        ("deft", 100, 2, 50, 4 + 8, 0, 3),
        ("hall", 1, 1, i32::from(LOCK_DIFFICULTY), 0, 5, 35),
    ] {
        let mut picked = 0;
        for seed in 0..40 {
            let mut w = World::new(content(false), scenario, Some(seed)).expect("world");
            train(&mut w, value);
            assert_eq!(modifier(&w), plus);
            go_to_row(&mut w, row);
            let door = Pos::new(2, row);
            assert_eq!(w.lock_difficulty(door), difficulty);
            let mut rng = w.state.rng.clone();
            let expected = check::check(&mut rng, plus, difficulty);
            let out = w.apply(&Action::PickLock(Dir::E));
            assert!(out.took_turn);
            assert_eq!(!locked(&w, door), expected, "{scenario} seed {seed}");
            if expected {
                picked += 1;
                assert_eq!(terrain(&w, door), "door_open");
                assert_eq!(last_message(&w), "You pick the lock and open the door.");
                assert!(out.events.contains(&Event::DoorOpened { pos: door }));
                assert_eq!(out.elapsed, Duration::seconds(18), "pick_lock + open_door");
            } else {
                assert_eq!(terrain(&w, door), "door_closed");
                assert_eq!(last_message(&w), "You fail to pick the lock.");
                assert_eq!(out.elapsed, Duration::seconds(12), "pick_lock");
            }
        }
        assert!(
            (at_least..=at_most).contains(&picked),
            "{scenario} against {difficulty}: {picked} of 40"
        );
    }
}

#[test]
fn a_lock_may_be_tried_again_and_again_until_it_gives() {
    let mut w = world("hall");
    train(&mut w, 0);
    let door = Pos::new(2, 1);
    let mut tries = 0;
    while locked(&w, door) {
        assert!(tries < 200, "a lock of 14 gives within 200 tries");
        let out = w.apply(&Action::PickLock(Dir::E));
        assert!(out.took_turn, "every try takes its time");
        tries += 1;
    }
    assert_eq!(terrain(&w, door), "door_open");
    // Unlocked for good: shut, it opens like any door.
    w.apply(&Action::Close(Dir::E));
    w.apply(&Action::Open(Dir::E));
    assert_eq!(last_message(&w), "You open the door.");
}

#[test]
fn a_trapped_doors_trap_goes_off_at_whoever_picks_its_lock() {
    let mut w = world("deft");
    train(&mut w, 200);
    go_to_row(&mut w, 4);
    let door = Pos::new(2, 4);
    let player = w.state.player;
    let mut sprung = Vec::new();
    while locked(&w, door) {
        let out = w.apply(&Action::PickLock(Dir::E));
        let springs: Vec<ActorId> = (out.events.iter())
            .filter_map(|e| match *e {
                Event::TrapSprung { at, victim, .. } if at == door => Some(victim),
                _ => None,
            })
            .collect();
        if locked(&w, door) {
            assert!(springs.is_empty(), "a failed try sets nothing off");
        }
        sprung.extend(springs);
    }
    assert_eq!(sprung, [player], "the trap went off once, at the picker");
    assert_eq!(terrain(&w, door), "door_open");
}

#[test]
fn trying_trains_open_lock() {
    let mut w = World::new(content(true), "hall", None).expect("world");
    train(&mut w, 30);
    let id = open_lock(&w);
    w.apply(&Action::PickLock(Dir::E));
    let trained = w.state.player().skills.get(id).expect("trained");
    assert!(
        trained.marks > 0 || trained.value > 30,
        "a try earns a mark: {trained:?}"
    );
    // A refused one does not.
    let before = w.state.player().skills.get(id);
    w.apply(&Action::PickLock(Dir::S));
    assert_eq!(w.state.player().skills.get(id), before);
}

#[test]
fn locks_are_harder_to_pick_deeper_in_a_branch() {
    let content = content(false);
    let depths = content.dungeon("depths").expect("the Depths");
    assert!(depths.locks_per_depth > 0, "the Depths' locks harden");
    assert_eq!(depths.locks_at(1), 0);
    assert_eq!(depths.locks_at(8), 7 * i32::from(depths.locks_per_depth));
    // A lock on the Depths 2: its own difficulty and one level's worth; the usual one too.
    let mut w = World::new(content.clone(), "deep", None).expect("world");
    let here = w.player_pos();
    let plus = i32::from(depths.locks_per_depth);
    let hard = Lock {
        difficulty: Some(20),
        ..Lock::new(None)
    };
    let plain = here.step(Dir::E);
    (w.state.current_mut().locks).insert(plain, Lock::new(Some("cellar")));
    w.state.current_mut().locks.insert(here, hard);
    assert_eq!(w.lock_difficulty(here), 20 + plus);
    assert_eq!(w.lock_difficulty(plain), i32::from(LOCK_DIFFICULTY) + plus);
    assert_eq!(w.lock_difficulty(here.step(Dir::W)), 0, "no lock there");
    // Off a branch the lock's own difficulty alone.
    let w = world("hall");
    assert_eq!(w.lock_difficulty(Pos::new(2, 2)), 50);
    assert_eq!(w.lock_difficulty(Pos::new(2, 3)), 1);
}

#[test]
fn a_status_that_blocks_picking_greys_it_out_and_refuses_it() {
    let mut w = world("hall");
    train(&mut w, 50);
    let out = w.apply(&Action::Afflict("shaking".into()));
    assert!(!out.took_turn);
    let shaking = w.content().status_id("shaking").expect("the test status");
    assert_eq!(
        pick_lock_in_menu(&w),
        Some(Some(Unavailable::While(shaking)))
    );
    let rng = w.state.rng.clone();
    let out = w.apply(&Action::PickLock(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You cannot pick locks while shaking.");
    assert_eq!(w.state.rng, rng, "no check rolled");
    assert!(locked(&w, Pos::new(2, 1)));
}

#[test]
fn picking_replays() {
    // A thief (trained in Open lock from the start) tries the usual lock until it gives, then the
    // easy one.
    let content = content(false);
    let mut w = World::new(content.clone(), "thief", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("pick:E*8 move:S*2 pick:E*8").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        !locked(&w, Pos::new(2, 1)),
        "eight tries open the usual lock"
    );
    assert!(!locked(&w, Pos::new(2, 3)), "and the easy one");
    let (verified, divergence) = replay.play(content.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
    // A lock's own difficulty is saved with it; the usual one is left out.
    let locks: Vec<&Lock> = w.state.current().locks.values().collect();
    let saved = serde_json::to_string(&locks).expect("json");
    assert!(saved.contains("\"difficulty\":50"), "{saved}");
    assert!(!saved.contains("\"difficulty\":14"), "{saved}");
}

#[test]
fn bad_lock_difficulties_are_reported() {
    let level = |legend: &str| {
        format!(
            "name = \"x\"\nmap = '''\n#####\n#@X.#\n#####\n'''\n[legend]\n\"X\" = {legend}\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        )
    };
    for (legend, expected) in [
        (
            "{ terrain = \"door_closed\", lock_difficulty = 20 }",
            "has a `lock_difficulty` but no lock",
        ),
        (
            "{ terrain = \"door_closed\", locked = true, lock_difficulty = 0 }",
            "`lock_difficulty` is 1 to 50, not 0",
        ),
        (
            "{ terrain = \"door_closed\", lock = \"a\", lock_difficulty = 51 }",
            "`lock_difficulty` is 1 to 50, not 51",
        ),
    ] {
        let mut files = files(false);
        files.insert("levels/hall.toml", &level(legend));
        let err = Content::load(&files).unwrap_err().message;
        assert!(err.contains(expected), "{legend}: {err}");
    }
    let mut files = files(false);
    let path = "campaigns/showcase/data/dungeons/showcase.toml";
    let (_, dungeons) = (files.table("dungeons").find(|(p, _)| *p == path)).expect("the depths");
    let steep = dungeons.replace("locks_per_depth = 1", "locks_per_depth = 6");
    assert_ne!(steep, dungeons);
    files.insert(path, &steep);
    let err = Content::load(&files).unwrap_err().message;
    assert!(err.contains("locks_per_depth must be 0 to 5"), "{err}");
}
