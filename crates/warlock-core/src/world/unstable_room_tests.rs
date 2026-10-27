//! The unstable room (spec §7 "Special rooms", design §9.6; `world/warp.rs`): every step inside,
//! the player's or a creature's, has a tenth's chance to move the stepper elsewhere on the level
//! as a teleport trap does, said with the trap's own lines; steps outside never do.

use super::*;
use crate::content::ContentFiles;

/// A hall, 30 wide, with an unstable room over x 10 to 19; `r` is a giant rat inside it.
const HALL: &str = "\
##############################
#@...........................#
#..........r.................#
#............................#
##############################
";

/// The rat's cell, inside the room.
const INSIDE: Pos = Pos::new(11, 2);
/// A cell of the hall outside the room.
const OUTSIDE: Pos = Pos::new(5, 2);

fn world(rat: bool) -> World {
    let map = if rat {
        HALL.to_string()
    } else {
        HALL.replace('r', ".")
    };
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_unstable.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{map}'''\n\
             [[room]]\neffect = \"unstable\"\narea = [10, 0, 19, 4]\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_unstable.toml",
        "name = \"T\"\nstart_level = \"test_unstable\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_unstable", None).expect("world")
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// Actor `who` steps onto `at` `times` times (put back there after each warp); how many of the
/// steps moved it elsewhere.
fn warps(w: &mut World, who: ActorId, at: Pos, times: u32) -> u32 {
    let mut moved = 0;
    for _ in 0..times {
        w.state.actors.get_mut(&who).expect("actor").pos = at;
        let mut events = Vec::new();
        w.step_onto_trap(at, who, &mut events);
        let to = w.state.actors[&who].pos;
        if to != at {
            moved += 1;
            assert!(
                events.contains(&Event::ActorMoved {
                    actor: who,
                    from: at,
                    to
                }),
                "a warp is a move: {events:?}"
            );
        }
    }
    moved
}

#[test]
fn the_unstable_room_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let unstable = effects.get(effects.id("unstable").expect("unstable"));
    assert_eq!(
        (
            unstable.name.as_str(),
            unstable.mark,
            unstable.warp,
            unstable.weight,
            unstable.max_cells
        ),
        ("unstable room", 'u', 10, 4, Some(60))
    );
    assert_eq!(unstable.enter, "The floor shimmers under your feet.");
    // Only the warp: never the whole level, and only for small rooms.
    assert!(unstable.level_chance == 0 && unstable.status.is_none() && !unstable.shunned);
    assert!(unstable.eligible(60, false, false) && !unstable.eligible(61, false, false));
    // No other effect warps.
    for e in effects.defs().iter().filter(|e| e.id != "unstable") {
        assert_eq!(e.warp, 0, "{}", e.id);
    }
}

#[test]
fn about_one_step_in_ten_inside_warps_the_player_and_none_outside() {
    let mut w = world(false);
    let player = w.state.player;
    let inside = warps(&mut w, player, INSIDE, 2000);
    assert!(
        (150..=250).contains(&inside),
        "{inside} warps in 2000 steps"
    );
    assert!(said(&w, "You are wrenched elsewhere."));
    assert_eq!(warps(&mut w, player, OUTSIDE, 500), 0);
}

#[test]
fn steps_outside_draw_nothing() {
    // The same walk outside the room, with and without the room: the same game.
    let mut plain = world(false);
    let level = plain.state.current_mut();
    level.rooms.clear();
    let mut unstable = world(false);
    for w in [&mut plain, &mut unstable] {
        for action in crate::action::parse_actions("move:SE move:E*3 move:W*3").expect("actions") {
            w.apply(&action);
        }
    }
    assert_eq!(
        plain.state.rng.below(1 << 30),
        unstable.state.rng.below(1 << 30)
    );
}

#[test]
fn a_walk_inside_warps_the_player_somewhere_free() {
    let mut w = world(false);
    for action in crate::action::parse_actions("move:SE move:E*9").expect("actions") {
        w.apply(&action);
    }
    let mut warped = None;
    for i in 0..200 {
        let before = w.player_pos();
        let dir = if before.x < 15 { Dir::E } else { Dir::W };
        w.apply(&Action::Move(dir));
        if w.player_pos() != before.step(dir) {
            warped = Some(i);
            break;
        }
    }
    assert!(warped.is_some(), "200 steps inside and never warped");
    assert!(said(&w, "You are wrenched elsewhere."));
    let level = w.state.current();
    let here = w.player_pos();
    assert!(level.is_walkable(&w.content, here));
}

#[test]
fn creatures_warp_too_and_the_player_sees_them_go() {
    let mut w = world(true);
    w.apply(&Action::Wait);
    let rat = (w.state.actors.values())
        .find(|a| a.pos == INSIDE)
        .map(|a| a.id)
        .expect("the rat");
    let inside = warps(&mut w, rat, INSIDE, 2000);
    assert!(
        (150..=250).contains(&inside),
        "{inside} warps in 2000 steps"
    );
    assert!(said(&w, "The giant rat vanishes!"));
}

#[test]
fn a_warp_is_the_same_game_after_a_save() {
    let mut w = world(false);
    for action in crate::action::parse_actions("move:SE move:E*9").expect("actions") {
        w.apply(&action);
    }
    let bytes = crate::save::encode(&w);
    let mut copy = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    for action in
        crate::action::parse_actions("move:E*4 move:W*4 move:E*4 move:W*4").expect("actions")
    {
        w.apply(&action);
        copy.apply(&action);
    }
    assert_eq!(w.state_hash(), copy.state_hash());
}
