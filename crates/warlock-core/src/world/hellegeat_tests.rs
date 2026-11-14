//! A Pit's floors in a run (#1296; spec §7 "The Pits", `endless.rs`): forgotten when left as an
//! endless branch's are, made the same again from their id, paid once per Pit, and leaving the
//! endless branches' own count of floors made alone.

use super::*;
use crate::action::{Action, parse_actions};
use crate::test_support::CONTENT;

fn start(seed: u64) -> World {
    World::new(CONTENT.clone(), "grundleas", Some(seed)).expect("world")
}

/// The player arrives on level `id` as if by a staircase, and what they left is forgotten.
fn jump(world: &mut World, id: &str) {
    let from = world.state.current_level.clone();
    world.travel(id, None).expect("travel");
    world.forget_floors(&[Event::LevelChanged {
        from,
        to: id.to_string(),
    }]);
}

/// The player, set on the staircase `dir` whole and hale, takes it.
fn take(world: &mut World, dir: StairsDir) {
    let at = (world.state.current().stairs.iter())
        .find(|s| s.dir == dir)
        .expect("stairs")
        .pos;
    let player = world.state.player;
    let here = world.state.current_level.clone();
    (world.state.actors).retain(|id, a| *id == player || a.level != here || a.pos != at);
    for action in parse_actions("levelup:auto").expect("actions") {
        world.apply(&action);
    }
    let content = world.content.clone();
    let me = world.state.actors.get_mut(&player).expect("player");
    me.pos = at;
    me.energy = me.energy.max(0);
    me.hp = me.max_hp(&content);
    me.effects = Default::default();
    let action = match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    };
    let events = world.apply(&action).events;
    assert!((events.iter()).any(|e| matches!(e, Event::LevelChanged { .. })));
}

/// The Pits' floors that exist now.
fn pit_floors(world: &World) -> Vec<&String> {
    (world.state.levels.keys())
        .filter(|id| world.content.is_portal_floor(id))
        .collect()
}

#[test]
fn a_pits_floor_is_forgotten_when_left_and_made_the_same_again() {
    let mut w = start(3);
    let deeper = w.state.level_seed(&w.content, "grundleas:2");
    jump(&mut w, "hellegeat@7:1");
    assert_eq!(w.state.current_level, "hellegeat@7:1");
    assert!(
        !w.state.levels.contains_key("grundleas:1"),
        "the way in is gone"
    );
    let first = w.state.current().terrain.clone();
    take(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, "hellegeat@7:2");
    assert_eq!(
        pit_floors(&w),
        ["hellegeat@7:2"],
        "the first floor is forgotten"
    );
    take(&mut w, StairsDir::Up);
    assert_eq!(w.state.current_level, "hellegeat@7:1");
    assert_eq!(pit_floors(&w), ["hellegeat@7:1"]);
    assert_eq!(
        w.state.current().terrain,
        first,
        "the same map, from its id"
    );
    // A Pit's floors draw on no endless stream: Grundlēas's next floor is what it would have been.
    assert_eq!(w.state.level_seed(&w.content, "grundleas:2"), deeper);
}

#[test]
fn each_pit_pays_its_floors_once() {
    let mut w = start(4);
    jump(&mut w, "hellegeat@7:1");
    take(&mut w, StairsDir::Down);
    let (endless, content) = (&w.state.endless, &w.content);
    assert!(
        !endless.pays(content, "hellegeat@7:1"),
        "paid on the way down"
    );
    assert!(!endless.pays(content, "hellegeat@7:2"));
    assert!(
        endless.pays(content, "hellegeat@7:3"),
        "deeper than reached"
    );
    assert!(endless.pays(content, "hellegeat@8:1"), "another Pit");
}
