//! The heavy hall (spec §7 "Special rooms"; `WorldState::load`, the one burden hook): carrying
//! limits are halved inside, for the player and creatures alike; the player is told as their
//! burden changes, cannot step in where they would be overloaded, and can always step out.

use super::*;
use crate::burden::{self, Burden};
use crate::content::ContentFiles;
use crate::object::Stack;

/// A hall, 30 wide, with a heavy hall over x 10 to 19.
const HALL: &str = "\
##############################
#@...........................#
#............................#
#............................#
##############################
";

/// The first cell of the heavy hall on the player's row, and the one before it.
const INSIDE: Pos = Pos::new(10, 1);
const OUTSIDE: Pos = Pos::new(9, 1);

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_heavy.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n\
             [[room]]\neffect = \"heavy\"\narea = [10, 0, 19, 4]\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_heavy.toml",
        "name = \"T\"\nstart_level = \"test_heavy\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_heavy", None).expect("world")
}

fn run(w: &mut World, text: &str) {
    for action in crate::action::parse_actions(text).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn load(w: &World) -> burden::Load {
    w.actor_load(w.state.player).expect("the player")
}

/// Rocks into the player's pack until they carry at least `percent` of their limit outside.
fn carry(w: &mut World, percent: u64) {
    let rock = w.content().object_id("rock").expect("rock");
    let limit = load(w).limit;
    let want = limit * percent / 100;
    let have = load(w).carried;
    let rocks = want.saturating_sub(have).div_ceil(800);
    let me = w.player_mut();
    me.inventory
        .push(Stack::new(rock, u32::try_from(rocks).expect("few")));
}

#[test]
fn the_heavy_hall_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let heavy = effects.get(effects.id("heavy").expect("heavy"));
    assert_eq!(
        (heavy.name.as_str(), heavy.mark, heavy.burden, heavy.weight),
        ("heavy hall", 'w', 50, 4)
    );
    // Only the burden: no status, floor, temperature, or light, and never the whole level.
    assert!(heavy.status.is_none() && heavy.floor.is_none() && heavy.level_chance == 0);
    assert!(!heavy.shunned && heavy.temperature == 0);
    assert_eq!(
        (heavy.regen, heavy.mana_regen, heavy.noise, heavy.light),
        (100, 100, 100, 100)
    );
    // Every other effect leaves the limit as it is.
    let others = effects.defs().iter().filter(|e| e.id != "heavy");
    assert!(others.into_iter().all(|e| e.burden == 100));
}

#[test]
fn the_limit_is_halved_inside_and_the_player_is_told() {
    let mut w = world();
    let outside = load(&w).limit;
    carry(&mut w, 25);
    assert_eq!(load(&w).burden(), Burden::Unburdened);
    let player = w.state.player().clone();
    let inside = w.state.load_at(w.content(), &player, INSIDE).limit;
    assert_eq!(inside, burden::in_room(outside, 50));
    assert!(inside.abs_diff(outside / 2) < 3 && inside.is_multiple_of(3));
    // Up to the door: as before.
    run(&mut w, "move:E*8");
    assert_eq!(w.state.player().pos, OUTSIDE);
    assert_eq!(load(&w).limit, outside);
    // In: the burden's change and the room's line; the status bar (perception) agrees.
    run(&mut w, "move:E");
    assert_eq!(w.state.player().pos, INSIDE);
    assert!(said(
        &w,
        "Your pack drags at you, as if the stones pulled it down."
    ));
    assert!(said(&w, Burden::Burdened.news()));
    assert_eq!(load(&w).limit, inside);
    assert_eq!(load(&w).burden(), Burden::Burdened);
    assert_eq!(w.perception().own_load(), load(&w));
    // Out again: lighter at once.
    run(&mut w, "move:W");
    assert!(said(&w, Burden::Unburdened.news()));
    assert_eq!(load(&w).limit, outside);
    assert_eq!(w.perception().own_load(), load(&w));
}

#[test]
fn no_step_in_where_the_load_would_be_too_much_but_always_out() {
    let mut w = world();
    carry(&mut w, 60);
    assert_eq!(load(&w).burden(), Burden::Burdened);
    run(&mut w, "move:E*8");
    assert_eq!(w.state.player().pos, OUTSIDE);
    let time = w.state.time;
    run(&mut w, "move:E");
    assert_eq!(w.state.player().pos, OUTSIDE, "refused");
    assert_eq!(w.state.time, time, "no time taken");
    assert!(said(&w, burden::HEAVY_REFUSED));
    // Inside, with the same load (picked up there, say): overloaded, so no step deeper in, but
    // a step out is to where they can move.
    w.player_mut().pos = INSIDE;
    assert_eq!(load(&w).burden(), Burden::Overloaded);
    run(&mut w, "move:E");
    assert_eq!(w.state.player().pos, INSIDE);
    assert!(said(&w, burden::MOVE_REFUSED));
    run(&mut w, "move:W");
    assert_eq!(w.state.player().pos, OUTSIDE);
    assert_eq!(load(&w).burden(), Burden::Burdened);
}

#[test]
fn creatures_carry_less_inside_too() {
    let w = world();
    let rat = w.content().creature_id("rat").expect("rat");
    let mut actor = w.state.player().clone();
    actor.kind = ActorKind::Creature;
    actor.creature = rat;
    let state = &w.state;
    let outside = state.load_at(w.content(), &actor, OUTSIDE).limit;
    let inside = state.load_at(w.content(), &actor, INSIDE).limit;
    assert_eq!(inside, burden::in_room(outside, 50));
    assert!(inside < outside);
}
