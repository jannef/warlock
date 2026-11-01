//! The collapsed floor (spec §7 "Special rooms", design §9.13; `world/collapse.rs`): a step of the
//! player onto its cracked floor gives way with 15 % (doubled while Burdened), and they fall to
//! the level below, hurt (a fall can kill) and Staggered; Old delver never falls; creatures
//! never do; no draw anywhere else. A generated one keeps a sound rim round its cracks.

use super::*;
use crate::burden::Burden;
use crate::content::ContentFiles;
use crate::object::Stack;

/// A hall with a collapsed floor over x 10 to 19 (`:` its cracked floor, `.` one plain cell of
/// it) and a way down to the cellar.
const TOP: &str = "\
##############################
#@........::::::::::........>#
#.........::::.:::::.........#
#.........::::::::::.....r...#
##############################
";

/// The cellar: stairs up, open floor, and a vault behind a locked door.
const CELLAR: &str = "\
############
#<.....#VVV#
#......+VVV#
#......#VVV#
############
";

/// A cracked cell of the room.
const CRACKED: Pos = Pos::new(11, 2);
/// A cell of the room with no cracks.
const SOUND: Pos = Pos::new(14, 2);
/// A cell of the hall outside the room.
const OUTSIDE: Pos = Pos::new(5, 2);

fn world_with(down: &str, rat: bool) -> World {
    let top = if rat {
        TOP.to_string()
    } else {
        TOP.replace('r', ".")
    };
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_top.toml",
        &format!(
            "name = \"Top\"\nambient = [120, 120, 120, 0]\nmap = '''\n{top}'''\n\
             [[room]]\neffect = \"collapsed\"\narea = [10, 0, 19, 4]\n[legend]\n\
             \":\" = \"cracked_floor\"\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \">\" = {{ terrain = \"stairs_down\"{down} }}\n"
        ),
    );
    files.insert(
        "levels/test_cellar.toml",
        &format!(
            "name = \"Cellar\"\nambient = [120, 120, 120, 0]\nmap = '''\n{CELLAR}'''\n\
             [legend]\n\"V\" = \"floor\"\n\
             \"+\" = {{ terrain = \"door_closed\", lock = \"vault\" }}\n\
             \"<\" = {{ terrain = \"stairs_up\", to = \"test_top\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_collapse.toml",
        "name = \"T\"\nstart_level = \"test_top\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_collapse", None).expect("world")
}

fn world() -> World {
    world_with(", to = \"test_cellar\"", false)
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// Puts the player back on the top level at `at`, whole.
fn back(w: &mut World, at: Pos) {
    w.state.current_level = "test_top".into();
    let content = w.content.clone();
    let player = w.player_mut();
    player.level = "test_top".into();
    player.pos = at;
    player.hp = player.max_hp(&content);
}

/// The player steps onto `at` `times` times (put back after each fall); how many fell.
fn falls(w: &mut World, at: Pos, times: u32) -> u32 {
    let mut fell = 0;
    for _ in 0..times {
        back(w, at);
        if w.collapse_step(at, &mut Vec::new()) {
            fell += 1;
        }
    }
    fell
}

#[test]
fn the_collapsed_floor_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let def = effects.get(effects.id("collapsed").expect("collapsed"));
    assert_eq!(
        (def.name.as_str(), def.mark, def.weight, def.needs_below),
        ("collapsed floor", 'k', 3, true)
    );
    assert_eq!(def.floor, content.terrain_id("cracked_floor"));
    assert_eq!(
        content.terrain(def.floor.expect("floor")).name,
        "cracked floor"
    );
    let collapse = def.collapse.expect("collapse");
    assert_eq!(
        (collapse.chance, collapse.damage.to_string()),
        (15, "2d6".into())
    );
    assert_eq!(def.enter, "The floor creaks under your weight.");
    assert_eq!(
        def.near.as_deref(),
        Some("The floor ahead is cracked and sagging.")
    );
    // Only on a level with another below it, and never the whole level.
    assert!(def.eligible(40, false, true) && !def.eligible(40, false, false));
    assert_eq!(def.level_chance, 0);
    for e in effects.defs().iter().filter(|e| e.id != "collapsed") {
        assert!(e.collapse.is_none(), "{}", e.id);
    }
}

#[test]
fn about_one_step_in_seven_gives_way_and_none_on_sound_floor_or_outside() {
    let mut w = world();
    let fell = falls(&mut w, CRACKED, 2000);
    assert!((240..=360).contains(&fell), "{fell} falls in 2000 steps");
    assert_eq!(falls(&mut w, SOUND, 300), 0);
    assert_eq!(falls(&mut w, OUTSIDE, 300), 0);
}

#[test]
fn a_burdened_step_gives_way_twice_as_often() {
    let mut w = world();
    let plate = w.content().object_id("plate_armour").expect("plate armour");
    while w.load_of(w.state.player).penalty() < Burden::Burdened {
        w.player_mut().inventory.push(Stack::one(plate));
    }
    assert_eq!(w.load_of(w.state.player).penalty(), Burden::Burdened);
    let fell = falls(&mut w, CRACKED, 2000);
    assert!((520..=680).contains(&fell), "{fell} falls in 2000 steps");
}

#[test]
fn old_delver_never_falls_and_draws_nothing() {
    let mut w = world();
    w.player_mut().talents.take("old_delver", None);
    let rng = w.state.rng.clone();
    assert_eq!(falls(&mut w, CRACKED, 500), 0);
    assert_eq!(w.state.rng, rng);
}

#[test]
fn steps_where_nothing_can_give_way_draw_nothing() {
    let mut w = world();
    let rng = w.state.rng.clone();
    falls(&mut w, SOUND, 50);
    falls(&mut w, OUTSIDE, 50);
    assert_eq!(w.state.rng, rng);
    // A way down that leads nowhere yet: nothing to fall to.
    let mut w = world_with("", false);
    let rng = w.state.rng.clone();
    assert_eq!(falls(&mut w, CRACKED, 100), 0);
    assert_eq!(w.state.rng, rng);
}

#[test]
fn a_creature_never_falls() {
    let mut w = world_with(", to = \"test_cellar\"", true);
    let player = w.state.player;
    let id = *(w.state.actors.keys())
        .find(|&&a| a != player)
        .expect("the rat");
    let rng = w.state.rng.clone();
    for _ in 0..200 {
        w.state.actors.get_mut(&id).expect("rat").pos = CRACKED;
        w.step_onto_trap(CRACKED, id, &mut Vec::new());
    }
    assert_eq!(w.state.actors[&id].level, "test_top");
    assert_eq!(w.state.rng, rng);
}

#[test]
fn a_fall_lands_on_open_floor_below_hurt_and_staggered() {
    let mut w = world();
    let mut events = Vec::new();
    let mut tries = 0;
    loop {
        back(&mut w, CRACKED);
        events.clear();
        if w.collapse_step(CRACKED, &mut events) {
            break;
        }
        tries += 1;
        assert!(tries < 200, "no fall in 200 steps");
    }
    assert_eq!(w.state.current_level, "test_cellar");
    let player = w.state.player();
    assert_eq!(player.level, "test_cellar");
    // Never in the vault behind the locked door, never on the stairs.
    assert!(player.pos.x < 7, "{:?}", player.pos);
    assert_ne!(player.pos, Pos::new(1, 1));
    assert!(player.hp < player.max_hp(&w.content) && player.hp >= 1);
    let staggered = w.content().status_id("staggered").expect("staggered");
    assert!(player.effects.has(staggered));
    assert!(events.contains(&Event::LevelChanged {
        from: "test_top".into(),
        to: "test_cellar".into()
    }));
    assert!(said(&w, "The floor gives way beneath you!"));
    assert!(said(&w, "You fall to Cellar."));
}

#[test]
fn a_fall_can_kill_and_records_its_cause() {
    let mut w = world();
    for _ in 0..400 {
        back(&mut w, CRACKED);
        w.player_mut().hp = 1;
        if w.collapse_step(CRACKED, &mut Vec::new()) {
            // The 2d6 is at least 2: the last hit point is not spared.
            let death = w.state.death.as_ref().expect("dead");
            assert_eq!(death.cause, Some(Cause::Fall));
            assert_eq!((death.by, death.of), (None, None));
            assert_eq!(death.place, "Cellar");
            assert_eq!(
                Cause::Fall.epitaph(w.content()),
                "Fell through a collapsed floor"
            );
            let staggered = w.content().status_id("staggered").expect("staggered");
            assert!(!w.state.player().effects.has(staggered));
            return;
        }
    }
    panic!("no fall in 400 steps");
}

/// A branch whose every special room is a collapsed floor where one fits: rooms on top, caves
/// below, and a last level under them both.
const CRUMBLING: &str = r#"
[[dungeon]]
id = "test_crumbling"
name = "Crumbling"
generator = "rooms"
depth = 3
size = [80, 21]
room_effects = { chance = 100, per_level = 3, favoured = { collapsed = 100 } }
[[dungeon.levels]]
from = 2
generator = "caves"
size = [100, 40]
"#;

/// Whether `p` is a door, open or closed.
fn door(content: &Content, level: &crate::content::LevelTemplate, p: Pos) -> bool {
    let def = content.terrain(level.terrain[p]);
    def.is_closed_door() || def.is_open_door()
}

/// Every generated collapsed floor cracks only in its middle: no cell next to a wall, a door, or
/// the room's edge cracks, nor a door; and its ways in (doors, doorways, a cave chamber's open
/// edge) are joined to each other across sound floor.
#[test]
fn a_generated_collapsed_floor_keeps_a_sound_rim_and_a_way_across() {
    let mut files = ContentFiles::builtin();
    files.insert("data/dungeons/test_crumbling.toml", CRUMBLING);
    let content = Content::load(&files).expect("content");
    let collapsed = content.room_effects().id("collapsed").expect("collapsed");
    let cracked = content.terrain_id("cracked_floor").expect("cracked floor");
    let (mut rooms, mut with_cracks, mut by_depth) = (0, 0, [0; 2]);
    for seed in 0..60 {
        for depth in 1..=2 {
            let id = format!("test_crumbling:{depth}");
            let level =
                crate::mapgen::generate(&content, ("showcase", seed), &id, None).expect("level");
            let walkable =
                |p: Pos| level.terrain.in_bounds(p) && content.terrain(level.terrain[p]).walkable;
            let passable =
                |p: Pos| level.terrain.in_bounds(p) && (walkable(p) || door(&content, &level, p));
            for room in level.rooms.iter().filter(|r| r.effect == collapsed) {
                rooms += 1;
                by_depth[depth as usize - 1] += 1;
                let cells = room.cells();
                let crack = |p: Pos| level.terrain[p] == cracked;
                if cells.iter().any(|&p| crack(p)) {
                    with_cracks += 1;
                }
                for &p in cells.iter().filter(|&&p| crack(p)) {
                    for q in Dir::ALL.map(|d| p.step(d)) {
                        assert!(
                            room.contains(q) && walkable(q) && !door(&content, &level, q),
                            "seed {seed} {id}: {p:?} cracks beside {q:?}"
                        );
                    }
                }
                // The ways in: cells of the room one can pass with a passable cell outside it.
                let ways: Vec<Pos> = (cells.iter().copied())
                    .filter(|&p| passable(p))
                    .filter(|&p| {
                        (Dir::ALL.map(|d| p.step(d)).into_iter())
                            .any(|q| !room.contains(q) && passable(q))
                    })
                    .collect();
                let Some(&first) = ways.first() else {
                    continue;
                };
                let sound = |p: Pos| room.contains(p) && passable(p) && !crack(p);
                let mut seen = std::collections::BTreeSet::from([first]);
                let mut next = vec![first];
                while let Some(p) = next.pop() {
                    for q in [Dir::N, Dir::E, Dir::S, Dir::W].map(|d| p.step(d)) {
                        if sound(q) && seen.insert(q) {
                            next.push(q);
                        }
                    }
                }
                for way in &ways {
                    assert!(
                        seen.contains(way),
                        "seed {seed} {id}: no sound way from {first:?} to {way:?}"
                    );
                }
            }
        }
    }
    assert!(rooms > 60, "{rooms} collapsed floors");
    assert!(with_cracks * 2 > rooms, "{with_cracks} of {rooms} crack");
    assert!(
        by_depth.iter().all(|n| *n > 10),
        "rooms and caves: {by_depth:?}"
    );
}

#[test]
fn walking_the_cracked_floor_drops_the_player_through() {
    let mut w = world();
    back(&mut w, Pos::new(10, 1));
    let mut east = true;
    for _ in 0..300 {
        let dir = if east { Dir::E } else { Dir::W };
        east = !east;
        let out = w.apply(&Action::Move(dir));
        assert!(out.took_turn);
        if w.state.current_level == "test_cellar" {
            assert!(said(&w, "The floor creaks under your weight."));
            // It replays the same.
            return;
        }
    }
    panic!("no fall in 300 steps over cracked floor");
}

#[test]
fn landing_cells_leave_out_locked_vaults_stairs_and_actors() {
    let mut w = world();
    w.visit("test_cellar", Some("test_top")).expect("cellar");
    let cells = super::collapse::landing(&w.state.levels["test_cellar"], &w.state, &w.content);
    assert!(!cells.is_empty());
    assert!(
        cells.iter().all(|p| p.x < 7 && *p != Pos::new(1, 1)),
        "{cells:?}"
    );
}
