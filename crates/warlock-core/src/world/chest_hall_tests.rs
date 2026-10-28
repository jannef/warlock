//! The showcase's `chests` scenario (#619; docs/design/chests-and-treasure.md, C7): the chest
//! hall's coffer, locked chest, trapped chest, and mimic all look like chests; both treasure
//! rooms are hinted on arrival; the secret door leads into the treasure room, and the pick-axe
//! beside the breach digs into the greater one.

use super::*;
use crate::action::parse_actions;
use crate::chest::ChestState;

const COFFER: Pos = Pos::new(4, 4);
const LOCKED: Pos = Pos::new(8, 4);
const TRAPPED: Pos = Pos::new(12, 4);
const MIMIC: Pos = Pos::new(16, 4);

fn world() -> World {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    World::new(content, "chests", None).expect("the chest hall")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

#[test]
fn the_chest_hall_holds_every_kind_of_chest() {
    let w = world();
    let view = w.perception();
    for p in [COFFER, LOCKED, TRAPPED, MIMIC] {
        let look = view.chest_at(p).expect("a chest in sight");
        assert_eq!(look.state, ChestState::Shut, "{p}");
    }
    let band = |p: Pos| view.chest_at(p).map(|c| c.band);
    assert_eq!(band(COFFER), Some(0), "a coffer");
    assert_eq!(band(LOCKED), Some(1), "a chest of the hall's level");
    let level = w.state.current();
    assert!(level.locks.contains_key(&LOCKED));
    assert!(level.traps.contains_key(&TRAPPED));
    assert!(!level.chests.contains_key(&MIMIC), "the mimic is no chest");
    let mimic = (w.state.actors.values())
        .find(|a| a.pos == MIMIC)
        .expect("the mimic");
    assert!(w.content().is_mimic(mimic.creature));
    assert!(w.disguised(mimic.id));
    // Both rooms, hinted on arrival.
    let effect = |id: &str| w.content().room_effects().id(id).expect(id);
    let effects: Vec<_> = level.rooms.iter().map(|r| r.effect).collect();
    assert_eq!(effects, [effect("treasury"), effect("great_treasury")]);
    assert!(said(
        &w,
        "Something tells you there is wealth hidden on this level."
    ));
    assert!(said(
        &w,
        "Something tells you a fortune lies walled up on this level."
    ));
}

#[test]
fn the_thief_picks_the_locked_chest() {
    let mut w = world();
    run(&mut w, "move:SE move:E*4 open:S");
    assert!(said(&w, "The chest is locked."));
    run(&mut w, "pick:S");
    assert!(said(&w, "Inside you find a short sword and 15 gold coins."));
}

#[test]
fn the_secret_door_leads_into_the_treasure_room() {
    let mut w = world();
    run(&mut w, "move:E*18 move:SE search:100 open:E move:E*2");
    assert!(said(
        &w,
        "Gold glints in the dust. No one has been here in a long time."
    ));
}

#[test]
fn the_pick_axe_digs_into_the_greater_treasure_room() {
    let mut w = world();
    run(
        &mut w,
        "move:SE*6 move:E*15 pickup equip:tool:a dig:E move:E*2",
    );
    assert_eq!(w.player_pos(), Pos::new(26, 8));
    assert!(said(
        &w,
        "The air is stale and heavy with the smell of old gold."
    ));
}
