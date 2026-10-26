//! The quick leave (spec §7 "Leaving an outdoor area", #474): `<` off any stairs in an outdoor
//! area with an overworld cell leaves it as walking off the edge does, unless a hostile creature
//! is seen, or unseen within 10 cells (Chebyshev); refused, it takes no time and names nobody.
//! The fixture is Cēaptūn (no stairs, peaceful townsfolk) with a kobold placed where each case
//! needs it; dungeons, an outdoor level without an overworld cell, and the overworld are left as
//! they were.

use super::caster_tests::kobold_at;
use super::quick_leave::{QUICK_LEAVE_REACH, QUICK_LEAVE_REFUSED};
use super::*;
use crate::action::parse_actions;
use crate::content::{LevelKind, Stance};
use crate::replay::Replay;

/// The town's square, where the dev scenario starts, and its cell on the overworld.
const SQUARE: Pos = Pos::new(46, 17);
const TOWN_CELL: Pos = Pos::new(18, 28);

fn content() -> Arc<Content> {
    Arc::new(Content::load(&crate::content::ContentFiles::builtin()).expect("builtin content"))
}

fn town() -> World {
    let mut w = World::new(content(), "ceaptun", None).expect("a run in Cēaptūn");
    assert_eq!(w.player_pos(), SQUARE);
    w.perceive(&mut Vec::new());
    w
}

fn act(world: &mut World, actions: &str) -> Vec<bool> {
    let actions = parse_actions(actions).expect("actions");
    actions.iter().map(|a| world.apply(a).took_turn).collect()
}

fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

/// `<` is refused: no time passes, the player stays, and the line names nobody.
fn refused(w: &mut World) {
    let before = w.state.time;
    assert_eq!(act(w, "ascend"), [false]);
    assert_eq!(w.state.current_level, "ceaptun");
    assert_eq!(w.state.time, before, "no time");
    assert_eq!(last(w), QUICK_LEAVE_REFUSED);
}

/// `<` leaves the town for its overworld cell.
fn leaves(w: &mut World) {
    assert_eq!(act(w, "ascend"), [true]);
    assert_eq!(w.state.current_level, "ascmearc");
    assert_eq!(w.player_pos(), TOWN_CELL);
    assert_eq!(last(w), "You return to The Ascmearc.");
}

#[test]
fn the_town_is_left_at_once_when_no_enemy_is_about() {
    let mut w = town();
    // The townsfolk are about, peaceful: they do not stop it.
    let here = w.player_pos();
    let player = w.state.player;
    let near = (w.state.actors.values())
        .filter(|a| a.id != player && a.level == "ceaptun")
        .filter(|a| a.pos.chebyshev(here) <= QUICK_LEAVE_REACH)
        .map(|a| a.id)
        .collect::<Vec<_>>();
    assert!(!near.is_empty(), "someone near the square");
    assert!(near.iter().all(|&a| w.stance(player, a) != Stance::Hostile));
    let before = w.state.time;
    leaves(&mut w);
    assert!(w.state.time > before, "it takes the time of stairs");
}

#[test]
fn a_hostile_creature_in_sight_stops_it_however_far() {
    let mut w = town();
    let kobold = kobold_at(&mut w, (14, 0), 10);
    let at = w.state.actors[&kobold].pos;
    assert!(w.perception.sees_actors_at(at), "in sight on the square");
    assert!(at.chebyshev(w.player_pos()) > QUICK_LEAVE_REACH);
    refused(&mut w);
    // Gone, the way is clear again.
    w.state.actors.remove(&kobold);
    leaves(&mut w);
}

#[test]
fn a_hidden_hostile_creature_within_ten_cells_stops_it_and_one_farther_does_not() {
    let mut w = town();
    // In the Gæsthūs behind its closed door, ten rows south of the square: unseen.
    let kobold = kobold_at(&mut w, (-6, 10), 10);
    let at = w.state.actors[&kobold].pos;
    assert!(!w.perception.sees_actors_at(at), "unseen indoors");
    assert_eq!(at.chebyshev(w.player_pos()), QUICK_LEAVE_REACH);
    refused(&mut w);
    // One row farther, eleven away: it does not count.
    w.state.actors.get_mut(&kobold).expect("the kobold").pos = Pos::new(at.x, at.y + 1);
    w.perceive(&mut Vec::new());
    assert!(!w.perception.sees_actors_at(Pos::new(at.x, at.y + 1)));
    leaves(&mut w);
}

#[test]
fn stairs_under_the_player_are_taken_as_ever_and_other_levels_need_them() {
    // A dungeon level: `<` off the stairs is no way up.
    let mut w = World::new(content(), "showcase", None).expect("the showcase");
    let level = w.state.current_level.clone();
    assert_eq!(w.state.current().kind, LevelKind::Dungeon);
    assert_eq!(act(&mut w, "ascend"), [false]);
    assert_eq!(last(&w), "You see no way up here.");
    assert_eq!(w.state.current_level, level);
    // The showcase's courtyard is outdoors but no overworld cell leads into it: stairs only.
    let mut w = World::new(content(), "courtyard", None).expect("the courtyard");
    assert_eq!(w.state.current().kind, LevelKind::Outdoor);
    assert!(w.state.current().exit.is_none());
    assert_eq!(act(&mut w, "ascend"), [false]);
    assert_eq!(last(&w), "You see no way up here.");
    // The overworld itself has nowhere further up.
    let mut w = World::new(content(), "ascmearc", None).expect("the Ascmearc");
    assert_eq!(act(&mut w, "ascend"), [false]);
    assert_eq!(last(&w), "You see no way up here.");
    // A local area: left the quick way, to its cell.
    let mut w = World::new(content(), "forest", None).expect("the forest");
    let exit = w.state.current().exit.clone().expect("an edge exit");
    assert_eq!(act(&mut w, "ascend"), [true]);
    assert_eq!(w.state.current_level, exit.to);
    assert_eq!(w.player_pos(), exit.pos);
}

#[test]
fn a_quick_leave_replays_exactly() {
    let content = content();
    let mut w = World::new(content.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E*2 ascend move:E descend ascend").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.state.current_level, "ascmearc");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
