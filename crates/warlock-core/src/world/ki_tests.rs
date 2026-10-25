//! The monk's Ki and the bard's Inspiration (spec §8 "Resources"; the progression design §4.18,
//! R4, #252): Flurry grants Ki and each second blow spends a point of it, Quivering palm costs
//! five instead of a cooldown, Inspire grants Inspiration and spends three of it in place of its
//! mana when they are held, and Virtuoso's song costs nothing.

use super::class_active_tests::{beside, last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{Build, ClassActive, ResourceId};
use crate::equipment::{free_slot, slots_taken};
use crate::object::Stack;
use crate::replay::Replay;

fn resource(w: &World, id: &str) -> ResourceId {
    w.content().resource_id(id).expect(id)
}

fn held(w: &World, id: &str) -> Option<u16> {
    (w.state.player().resources.get(resource(w, id))).map(|p| p.value)
}

fn set(w: &mut World, id: &str, value: u16) {
    let r = resource(w, id);
    w.player_mut().resources.set(r, value);
}

/// Why the player cannot use `ability` now, as the abilities menu has it; `None` without it.
fn own(w: &World, ability: Ability) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == ability)
        .map(|a| a.unavailable)
}

/// A human of `class` as a run starts them in the showcase (seed 7), with their kit and the
/// creation picks of chance.
fn fresh(class: &str) -> World {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world")
}

/// A monk with nothing in hand at `level`.
fn monk(level: u8) -> World {
    let mut w = world("monk");
    w.player_mut().equipment = Default::default();
    reach(&mut w, level);
    w
}

#[test]
fn flurry_grants_ki_and_each_second_blow_spends_a_point() {
    let w = monk(17);
    assert_eq!(held(&w, "ki"), None, "no ki before Flurry");
    let mut w = monk(18);
    assert_eq!(held(&w, "ki"), Some(5), "Flurry: Ki, full");
    let kobold = beside(&mut w, "kobold", 500);
    let player = w.state.player;
    assert!(w.flurry(player, kobold, &mut Vec::new()).is_some());
    assert_eq!(held(&w, "ki"), Some(4), "a point a second blow");
    // No ki, no flurry.
    set(&mut w, "ki", 0);
    assert!(w.flurry(player, kobold, &mut Vec::new()).is_none());
    assert_eq!(held(&w, "ki"), Some(0));
    // It comes back a point every three base turns, and stays at 0 meanwhile.
    run(&mut w, "wait*3");
    assert_eq!(held(&w, "ki"), Some(1));
}

#[test]
fn an_attack_with_ki_strikes_twice_and_without_once() {
    let mut w = monk(18);
    let kobold = beside(&mut w, "kobold", 500);
    run(&mut w, "move:E");
    assert_eq!(
        held(&w, "ki"),
        Some(4),
        "the flurry's second blow spent one"
    );
    set(&mut w, "ki", 0);
    let before = w.state.turn;
    run(&mut w, "move:E");
    assert!(w.state.turn > before);
    assert!(w.state.actors.contains_key(&kobold));
    // The point that came back during the attack's time is all it holds: nothing was spent.
    assert!(
        held(&w, "ki").is_some_and(|v| v <= 1),
        "{:?}",
        held(&w, "ki")
    );
}

#[test]
fn quivering_palm_costs_five_ki_and_no_cooldown() {
    let mut w = monk(50);
    let palm = Ability::Class(ClassActive::QuiveringPalm);
    let kobold = beside(&mut w, "kobold", 500);
    let ki = resource(&w, "ki");
    // Short: refused, no time, nothing spent.
    set(&mut w, "ki", 4);
    assert_eq!(own(&w, palm), Some(Some(Unavailable::Short(ki))));
    let turn = w.state.turn;
    run(&mut w, "use:quivering_palm:E");
    assert_eq!(last(&w), "You have too little ki.");
    assert_eq!((w.state.turn, held(&w, "ki")), (turn, Some(4)));
    // A weapon in hand: refused before the ki is spent.
    set(&mut w, "ki", 5);
    let content = w.content().clone();
    let dagger = content.object_id("dagger").expect("a dagger");
    let me = w.player_mut();
    me.inventory.push(Stack::new(dagger, 1));
    let item = me.inventory.len() - 1;
    let slot = free_slot(&content, &me.equipment, dagger).expect("a free slot");
    me.equipment.put(&slots_taken(&content, dagger, slot), item);
    run(&mut w, "use:quivering_palm:E");
    assert_eq!(last(&w), "Your hands are not free.");
    assert_eq!((w.state.turn, held(&w, "ki")), (turn, Some(5)));
    // Bare-handed: the blow, for all five, and no Spent after it.
    w.player_mut().equipment = Default::default();
    assert_eq!(own(&w, palm), Some(None));
    run(&mut w, "use:quivering_palm:E");
    assert!(w.state.turn > turn);
    assert_eq!(held(&w, "ki"), Some(0));
    assert!(!suffers(&w, w.state.player, "spent"));
    assert!(w.state.actors.contains_key(&kobold));
}

#[test]
fn inspire_spends_inspiration_in_place_of_mana() {
    let mut w = fresh("bard");
    reach(&mut w, 17);
    assert_eq!(
        held(&w, "inspiration"),
        None,
        "no inspiration before Inspire"
    );
    reach(&mut w, 18);
    assert_eq!(
        held(&w, "inspiration"),
        Some(0),
        "Inspire: Inspiration, empty"
    );
    let inspire = Ability::Class(ClassActive::Inspire);
    // Without enough inspiration it costs mana, as before.
    w.player_mut().mana = 10;
    run(&mut w, "use:inspire");
    assert_eq!(w.state.player().mana, 7);
    assert!(suffers(&w, w.state.player, "inspired"));
    // A kill of the bard's own builds a point.
    let player = w.state.player;
    w.gain_resource(player, Build::Kill);
    assert_eq!(held(&w, "inspiration"), Some(1));
    // Holding three, those are spent and the mana is left alone, even with none.
    set(&mut w, "inspiration", 3);
    w.player_mut().mana = 0;
    assert_eq!(own(&w, inspire), Some(None));
    let turn = w.state.turn;
    run(&mut w, "use:inspire");
    assert!(w.state.turn > turn, "a chat's time");
    assert_eq!(
        (held(&w, "inspiration"), w.state.player().mana),
        (Some(0), 0)
    );
    // Two inspiration and too little mana: neither pays.
    set(&mut w, "inspiration", 2);
    w.player_mut().mana = 2;
    assert_eq!(own(&w, inspire), Some(Some(Unavailable::NoMana)));
    // Virtuoso: a greater song for nothing, the inspiration kept.
    reach(&mut w, 60);
    set(&mut w, "inspiration", 5);
    w.player_mut().mana = 0;
    run(&mut w, "use:inspire");
    assert!(suffers(&w, w.state.player, "inspired_greatly"));
    assert_eq!(held(&w, "inspiration"), Some(5));
}

#[test]
fn ki_and_inspiration_replay() {
    for (class, text) in [
        (
            "monk",
            "xp:3000000 levelup:auto wait*4 use:quivering_palm:E wait",
        ),
        (
            "bard",
            "xp:3000000 levelup:auto use:inspire wait*40 use:inspire",
        ),
    ] {
        let mut w = fresh(class);
        let mut replay = Replay::new(&w);
        for action in parse_actions(text).expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        let verified = Replay::from_json(&replay.to_json())
            .expect("json")
            .run(w.content().clone())
            .expect("the replay verifies");
        assert_eq!(verified.state_hash(), w.state_hash(), "{class}");
    }
}
