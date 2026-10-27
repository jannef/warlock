//! The collapsed floor (spec §7 "Special rooms", design §9.13; `world/collapse.rs`): a step of the
//! player onto its cracked floor gives way with 15 % (doubled while Burdened), and they fall to
//! the level below, hurt but never killed, and Staggered; Light tread never falls; creatures
//! never do; no draw anywhere else.

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
fn light_tread_never_falls_and_draws_nothing() {
    let mut w = world();
    w.player_mut().talents.take("light_tread", None);
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
fn a_fall_never_kills() {
    let mut w = world();
    for _ in 0..400 {
        back(&mut w, CRACKED);
        w.player_mut().hp = 1;
        if w.collapse_step(CRACKED, &mut Vec::new()) {
            assert_eq!(w.state.player().hp, 1);
            assert!(w.state.death.is_none());
            return;
        }
    }
    panic!("no fall in 400 steps");
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
