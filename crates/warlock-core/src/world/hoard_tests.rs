//! Tempting hoards (spec §7 "Special rooms", design §9.7; `world/hoard.rs`, `mapgen/hoard.rs`):
//! the first stack lifted inside seals the room (its doors shut and lock, guardians come at its
//! edges, out of view where they can be), Hoard-wise lets one go first, a room seals once, a cave
//! blob gets only guardians, the dumps, saving, and a generated hoard's items.

use super::*;
use crate::action::Positions;
use crate::content::{ContentFiles, SpecialRoom};

/// A lit hall, its east part (x 7 on, walls included) a tempting hoard with one door, closed
/// (`+`, at (7, 3)), and an open one (`'`, at (7, 1)). Daggers lie at `a` and `b` inside. A wall
/// at x 11, open only at its foot, hides the room's east end from the daggers.
const HALL: &str = "\
###############
#......'...#..#
#.........a#..#
#@.....+..b#..#
#......#...#..#
#......#......#
###############
";

/// The closed door.
const DOOR: Pos = Pos::new(7, 3);
/// The open door.
const OPEN: Pos = Pos::new(7, 1);
/// The daggers inside.
const A: Pos = Pos::new(10, 2);
const B: Pos = Pos::new(10, 3);

/// The built-in content with [`HALL`] (`header` its level header, `map` its map) and its treasure
/// room over `area`.
fn world_of(header: &str, map: &str, area: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_hoard.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\n{header}map = '''\n{map}'''\n\
             [[room]]\neffect = \"hoard\"\narea = {area}\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"+\" = \"door_closed\"\n\"'\" = \"door_open\"\n\
             \"a\" = {{ terrain = \"floor\", objects = [\"dagger\"] }}\n\
             \"b\" = {{ terrain = \"floor\", objects = [\"dagger\"] }}\n"
        ),
    );
    files.insert(
        "scenarios/test_hoard.toml",
        "name = \"T\"\nstart_level = \"test_hoard\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_hoard", None).expect("world")
}

/// [`HALL`] on a level of danger 3 where kobolds live.
fn world() -> World {
    world_of(
        "danger = 3\npopulation = [\"kobold\"]\n",
        HALL,
        "[7, 0, 14, 6]",
    )
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// The player stands at `at`, looks about, and picks up everything there.
fn lift(w: &mut World, at: Pos) -> Outcome {
    w.player_mut().pos = at;
    w.perceive(&mut Vec::new());
    w.apply(&Action::PickUp(Positions::default()))
}

fn room(w: &World) -> SpecialRoom {
    w.state.current().rooms[0].clone()
}

fn creatures(w: &World) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .collect()
}

#[test]
fn the_tempting_hoard_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let hoard = effects.get(effects.id("hoard").expect("hoard"));
    let numbers = hoard.hoard.expect("a hoard");
    assert_eq!(
        (
            hoard.name.as_str(),
            hoard.mark,
            numbers.items,
            numbers.loot_level,
            numbers.guardians,
            numbers.danger,
            hoard.needs_danger,
            hoard.weight
        ),
        ("tempting hoard", 'T', 3, 3, [2, 4], 1, true, 5)
    );
    assert!(!hoard.can_pervade(), "never a whole level");
}

#[test]
fn a_hoard_needs_a_danger_and_sane_numbers() {
    let shipped = include_str!("../../../../assets/data/room_effects.toml");
    let line = "hoard = { items = 3, loot_level = 3, guardians = [2, 4], danger = 1 }";
    assert!(shipped.contains(line));
    let without_danger = shipped.replace(&format!("needs_danger = true\n{line}"), line);
    assert_ne!(without_danger, shipped);
    for (what, text) in [
        ("no needs_danger", without_danger),
        (
            "no items",
            shipped.replace(line, &line.replace("items = 3", "items = 0")),
        ),
        (
            "least above most",
            shipped.replace(line, &line.replace("[2, 4]", "[5, 4]")),
        ),
        (
            "too many guardians",
            shipped.replace(line, &line.replace("[2, 4]", "[2, 17]")),
        ),
        (
            "over a whole level",
            shipped.replace(
                line,
                &format!("{line}\nlevel_chance = 5\npervade = \"Gold!\""),
            ),
        ),
    ] {
        let mut files = ContentFiles::builtin();
        files.insert("data/room_effects.toml", &text);
        assert!(Content::load(&files).is_err(), "{what} is refused");
    }
}

#[test]
fn the_first_stack_lifted_seals_the_room() {
    let mut w = world();
    assert!(creatures(&w).is_empty());
    let out = lift(&mut w, A);
    assert!(out.took_turn);
    let room = room(&w);
    assert!(room.sealed && room.lifted == 1);
    assert!(said(&w, super::hoard::SLAM));
    // Both doors shut and locked, with a lock no key but a skeleton one opens.
    let level = w.state.current();
    for door in [DOOR, OPEN] {
        assert!(
            w.content.terrain(level.terrain[door]).is_closed_door(),
            "{door}"
        );
        assert_eq!(level.locks.get(&door), Some(&crate::door::Lock::new(None)));
    }
    // Guardians, hostile and awake, on the room's walkable cells.
    let guards = creatures(&w);
    assert!(
        (2..=4).contains(&guards.len()),
        "{} guardians",
        guards.len()
    );
    for g in guards {
        assert!(room.contains(g.pos), "{} inside", g.pos);
        assert!(level.is_walkable(&w.content, g.pos));
        assert!(!w.asleep(g.id));
        assert_eq!(w.stance(w.state.player, g.id), Stance::Hostile);
    }
}

#[test]
fn guardians_come_at_the_edges_out_of_view_where_they_can() {
    for seed in 0..8 {
        let mut w = world();
        w.state.rng = crate::rng::Rng::stream(seed, "gameplay");
        w.player_mut().pos = A;
        w.perceive(&mut Vec::new());
        let seen: Vec<Pos> = (room(&w).cells().into_iter())
            .filter(|&p| w.perception.sees_actors_at(p))
            .collect();
        lift(&mut w, A);
        let room = room(&w);
        let guards = creatures(&w);
        assert!(guards.len() >= 2, "seed {seed}");
        for g in guards {
            let edge = Dir::ALL.iter().any(|d| {
                let q = g.pos.step(*d);
                !room.contains(q) || !w.state.current().is_walkable(&w.content, q)
            });
            assert!(edge, "seed {seed}: {} on the edge", g.pos);
            assert!(!seen.contains(&g.pos), "seed {seed}: {} out of view", g.pos);
        }
    }
}

#[test]
fn nothing_lifted_outside_counts() {
    let mut w = world();
    let daggers = w.state.current().objects_at(A).to_vec();
    w.state.current_mut().set_pile(A, Vec::new());
    w.state.current_mut().set_pile(Pos::new(5, 3), daggers);
    lift(&mut w, Pos::new(5, 3));
    assert!(!room(&w).sealed);
    assert_eq!(room(&w).lifted, 0);
    assert!(creatures(&w).is_empty());
}

#[test]
fn hoard_wise_lifts_one_stack_before_it_seals() {
    let mut w = world();
    w.player_mut().talents.take("hoard_wise", None);
    lift(&mut w, A);
    let first = room(&w);
    assert!(!first.sealed && first.lifted == 1, "one is free");
    assert!(creatures(&w).is_empty());
    assert!(!said(&w, super::hoard::SLAM));
    lift(&mut w, B);
    assert!(room(&w).sealed, "the second seals it");
    assert!(!creatures(&w).is_empty());
}

#[test]
fn a_room_seals_once() {
    let mut w = world();
    lift(&mut w, A);
    let guards = creatures(&w).len();
    lift(&mut w, B);
    assert_eq!(creatures(&w).len(), guards, "no more come");
    assert_eq!(room(&w).lifted, 1, "the count stops once sealed");
}

#[test]
fn an_open_door_with_something_in_it_stays_open() {
    let mut w = world();
    let daggers = w.state.current().objects_at(B).to_vec();
    w.state.current_mut().set_pile(OPEN, daggers);
    lift(&mut w, A);
    let level = w.state.current();
    assert!(w.content.terrain(level.terrain[OPEN]).is_open_door());
    assert!(!level.locks.contains_key(&OPEN));
    assert!(level.locks.contains_key(&DOOR), "the other still locks");
}

#[test]
fn a_blob_without_doors_gets_only_guardians() {
    let map = HALL.replace(['+', '\''], ".");
    let mut w = world_of(
        "danger = 3\npopulation = [\"kobold\"]\n",
        &map,
        "[7, 0, 14, 6]",
    );
    lift(&mut w, A);
    assert!(room(&w).sealed);
    assert!(!said(&w, super::hoard::SLAM), "no doors to slam");
    assert!(w.state.current().locks.is_empty());
    assert!(creatures(&w).len() >= 2);
}

#[test]
fn without_a_danger_only_the_doors_lock() {
    let mut w = world_of("", HALL, "[7, 0, 14, 6]");
    lift(&mut w, A);
    assert!(said(&w, super::hoard::SLAM));
    assert!(creatures(&w).is_empty());
}

#[test]
fn the_rooms_dump_says_lifted_and_sealed() {
    let mut w = world();
    lift(&mut w, A);
    let text = crate::dump::ascii(&w, &[crate::dump::Layer::Rooms]);
    assert!(text.contains("tempting hoard at"), "{text}");
    assert!(text.contains(", lifted 1, sealed"), "{text}");
}

#[test]
fn a_sealed_hoard_is_the_same_game_after_a_save() {
    let mut w = world();
    lift(&mut w, A);
    let bytes = crate::save::encode(&w);
    let copy = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(copy.state_hash(), w.state_hash());
    assert!(copy.state.current().rooms[0].sealed);
    // A room never lifted from hashes as before: nothing about it is saved.
    let fresh = world();
    let json = serde_json::to_string(&fresh.state.current().rooms[0]).expect("json");
    assert!(
        !json.contains("lifted") && !json.contains("sealed"),
        "{json}"
    );
}

/// A rooms branch whose every special room is a tempting hoard.
const HOARDS: &str = r#"
[[dungeon]]
id = "hoards"
name = "Hoards"
generator = "rooms"
depth = 1
size = [80, 21]
danger = 3
population = ["kobold", "rat"]
room_effects = { chance = 100, per_level = 1, favoured = { hoard = 100 } }
"#;

#[test]
fn a_generated_hoard_holds_its_items_and_no_one() {
    let mut files = ContentFiles::builtin();
    let dungeons = format!(
        "{}{HOARDS}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    let content = Content::load(&files).expect("content");
    let hoard = content.room_effects().id("hoard").expect("hoard");
    let mut found = 0;
    for seed in 0..20 {
        let make = |s| crate::mapgen::generate(&content, ("showcase", s), "hoards:1", None);
        let level = make(seed).expect("hoards:1");
        assert_eq!(make(seed).expect("again"), level, "deterministic");
        let Some(room) = level.rooms.iter().find(|r| r.effect == hoard) else {
            continue;
        };
        found += 1;
        let items: usize = (level.objects.iter())
            .filter(|(p, _)| room.contains(**p))
            .map(|(_, pile)| pile.len())
            .sum();
        assert!((1..=3).contains(&items), "seed {seed}: {items} items");
        assert!(
            !level.creatures.iter().any(|c| room.contains(c.pos)),
            "seed {seed}: unguarded"
        );
    }
    assert!(found >= 10, "{found} hoards in 20 floors");
}
