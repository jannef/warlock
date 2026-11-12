//! The ascetic's Ki and the bard's Inspiration (spec §8 "Resources"; the progression design §4.18,
//! R4, #252; the ki tricks design, #846): Martial arts grants Ki at level 1, its maximum grows
//! with Willpower and the level, Flurry's second blows spend a point each from level 1, Quivering
//! palm costs five instead of a cooldown, Perfect self brings it back twice as fast, Inspire
//! grants Inspiration and spends three of it (the songs themselves are `song_tests.rs`'s), and
//! Virtuoso's song costs nothing.

use super::class_active_tests::{at_normal_speed, beside, last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{Build, ClassActive, ResourceId};
use crate::equipment::{free_slot, slots_taken};
use crate::object::Stack;
use crate::replay::Replay;
use crate::stats::{PerStat, Stats, modifier};

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

/// An ascetic with nothing in hand at `level`.
fn ascetic(level: u8) -> World {
    let mut w = world("ascetic");
    w.player_mut().equipment = Default::default();
    if level > 1 {
        reach(&mut w, level);
    }
    at_normal_speed(&mut w);
    w
}

/// The most ki the player may hold now.
fn ki_max(w: &World) -> u16 {
    let level = w.player_level();
    (w.state.player()).resource_max(w.content(), resource(w, "ki"), level)
}

/// Every score of the player at `score` (Willpower among them).
fn scores(w: &mut World, score: u8) {
    w.player_mut().stats = Stats::new(PerStat::splat(score), PerStat::splat(40));
}

#[test]
fn martial_arts_grants_ki_full_at_level_one_and_no_other_class_holds_it() {
    let w = fresh("ascetic");
    assert_eq!(w.player_level(), 1);
    assert_eq!(held(&w, "ki"), Some(ki_max(&w)), "Martial arts: Ki, full");
    for class in ["berserker", "thief", "warrior", "wizard"] {
        assert_eq!(held(&fresh(class), "ki"), None, "{class}");
    }
}

#[test]
fn the_maximum_grows_with_willpower_and_the_level_and_never_falls_below_three() {
    let mut w = ascetic(1);
    // 4, the Willpower modifier, a point per 6 levels: the design's three samples.
    scores(&mut w, 12);
    assert_eq!(ki_max(&w), 4 + u16::try_from(modifier(12)).unwrap());
    assert_eq!(ki_max(&w), 5, "Willpower 12 at level 1");
    scores(&mut w, 1);
    assert!(modifier(1) <= -2);
    assert_eq!(ki_max(&w), 3, "at least 3");
    reach(&mut w, 25);
    // The picks may have taken Inner calm: this is the pool's own maximum.
    w.player_mut().talents.forget("inner_calm");
    scores(&mut w, 14);
    assert_eq!(ki_max(&w), 10, "Willpower 14 at level 25");
    reach(&mut w, 60);
    w.player_mut().talents.forget("inner_calm");
    scores(&mut w, 16);
    assert_eq!(ki_max(&w), 17, "Willpower 16 at level 60");
}

#[test]
fn perfect_self_brings_ki_back_twice_as_fast() {
    let mut w = ascetic(59);
    w.player_mut().talents.forget("inner_calm");
    let player = w.state.player;
    let ki = resource(&w, "ki");
    assert_eq!(w.resource_rate(player, ki), 100);
    reach(&mut w, 60);
    w.player_mut().talents.forget("inner_calm");
    assert_eq!(w.resource_rate(player, ki), 200, "Perfect self");
    set(&mut w, "ki", 0);
    run(&mut w, "wait*3");
    assert_eq!(
        held(&w, "ki"),
        Some(2),
        "a point every base turn and a half"
    );
}

#[test]
fn a_level_one_ascetic_flurries_and_each_second_blow_spends_a_point() {
    let mut w = ascetic(1);
    set(&mut w, "ki", 5);
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
    let mut w = ascetic(1);
    set(&mut w, "ki", 5);
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
    let mut w = ascetic(50);
    let palm = Ability::Class(ClassActive::QuiveringPalm);
    let kobold = beside(&mut w, "kobold", 500);
    let ki = resource(&w, "ki");
    // Short: refused, no time, nothing spent.
    set(&mut w, "ki", 4);
    assert_eq!(own(&w, palm), Some(Some(Unavailable::Short(ki))));
    let turn = w.state.turn;
    run(&mut w, "use:quivering_palm:E");
    assert_eq!(last(&w), "Your ki is spent.");
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
    w.perceive(&mut Vec::new());
    assert_eq!(own(&w, palm), Some(None));
    run(&mut w, "use:quivering_palm:E");
    assert!(w.state.turn > turn);
    assert_eq!(held(&w, "ki"), Some(0));
    assert!(w.state.actors.contains_key(&kobold));
}

#[test]
fn inspire_spends_inspiration() {
    let mut w = fresh("bard");
    reach(&mut w, 8);
    assert_eq!(
        held(&w, "inspiration"),
        None,
        "no inspiration before Inspire"
    );
    reach(&mut w, 9);
    assert_eq!(
        held(&w, "inspiration"),
        Some(0),
        "Inspire: Inspiration, empty"
    );
    let inspire = Ability::Class(ClassActive::Inspire);
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
    assert!(suffers(&w, w.state.player, "inspired"));
    // Two inspiration and plenty of mana: no song (one cost, no fallback).
    set(&mut w, "inspiration", 2);
    w.player_mut().mana = 20;
    let short = Unavailable::Short(resource(&w, "inspiration"));
    assert_eq!(own(&w, inspire), Some(Some(short)));
    // Virtuoso: a greater song for nothing, the inspiration kept.
    reach(&mut w, 30);
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
            "ascetic",
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
