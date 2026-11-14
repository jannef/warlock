//! Smite and zeal (`world/classes/smite.rs`; spec §8 "Class abilities": "Smite", "Resources"; the
//! paladin design P1, #910): the pool, the blow's dice and aim, its cost, the sin that empties
//! it, and a save without it.

use super::class_active_tests::{at_normal_speed, beside, reach, world};
use crate::content::{Build, ResourceId};
use crate::replay::Replay;
use crate::stats::Stat;
use crate::test_support::{builtin, last, run};
use crate::world::character::alignment::Deed;
use crate::world::*;

fn zeal(w: &World) -> ResourceId {
    w.content().resource_id("zeal").expect("zeal")
}

fn held(w: &World) -> Option<u16> {
    w.state.player().resources.get(zeal(w)).map(|p| p.value)
}

fn set(w: &mut World, value: u16) {
    let id = zeal(w);
    w.player_mut().resources.set(id, value);
}

#[test]
fn a_new_paladin_smites_and_holds_zeal_at_five() {
    let w = world("paladin");
    assert_eq!(held(&w), Some(5), "granted at its start");
    assert!(w.state.player().talents.has("smite"), "Smite at level 1");
    // Every score 14 (+2): 10 + 2, at level 1.
    let max = w.resource_max(w.state.player, zeal(&w));
    assert_eq!(max, 12);
    // Nobody else holds it: the priest's table has no Smite.
    let priest = world("priest");
    assert_eq!(held(&priest), None);
}

#[test]
fn the_smite_aims_with_presence_and_falls_harder_on_the_unholy() {
    let mut w = world("paladin");
    let content = w.content().clone();
    let goblin = beside(&mut w, "goblin", 500);
    let plain = w.state.player().melee(&content);
    let smite = w.smite_profile(goblin).expect("a smite");
    let presence = (w.state.player().effective_stats(&content)).modifier(Stat::Presence);
    assert_eq!(smite.to_hit, plain.to_hit + presence, "Presence to hit");
    assert_eq!(smite.extra.map(|d| d.to_string()), Some("1d6".into()));
    w.state.actors.remove(&goblin);
    let skeleton = beside(&mut w, "skeleton", 500);
    let smite = w.smite_profile(skeleton).expect("a smite");
    assert_eq!(
        smite.extra.map(|d| d.to_string()),
        Some("2d6".into()),
        "a die more on the undead"
    );
    reach(&mut w, 20);
    let smite = w.smite_profile(skeleton).expect("a smite");
    assert_eq!(smite.extra.map(|d| d.to_string()), Some("3d6".into()));
}

#[test]
fn a_smite_costs_three_zeal_and_nothing_when_refused() {
    let mut w = world("paladin");
    at_normal_speed(&mut w);
    set(&mut w, 5);
    let turn = w.state.turn;
    run(&mut w, "use:smite:E");
    assert_eq!(last(&w), "There is nobody there to strike.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(5)), "refused, free");
    beside(&mut w, "kobold", 500);
    run(&mut w, "use:smite:E");
    assert_eq!(w.state.turn, turn + 1, "a melee attack's time");
    // Three spent; the blow may have hit (one back) and the kobold may have hit back (one).
    let after = held(&w).expect("held");
    assert!((2..=4).contains(&after), "{after}");
    set(&mut w, 2);
    let turn = w.state.turn;
    run(&mut w, "use:smite:E");
    assert_eq!(last(&w), "Your zeal is not enough.");
    assert_eq!(w.state.turn, turn, "no time");
}

#[test]
fn zeal_builds_by_blows_given_and_taken_and_comes_back_with_time() {
    let mut w = world("paladin");
    at_normal_speed(&mut w);
    let player = w.state.player;
    set(&mut w, 0);
    w.gain_resource(player, Build::Hit);
    assert_eq!(held(&w), Some(1), "a blow that hits");
    w.gain_resource(player, Build::Struck);
    assert_eq!(held(&w), Some(2), "a blow taken");
    w.gain_resource(player, Build::Kill);
    assert_eq!(held(&w), Some(2), "kills build nothing");
    // A point a minute (ten base turns), resting too.
    set(&mut w, 0);
    run(&mut w, "wait*10");
    assert_eq!(held(&w), Some(1));
    run(&mut w, "rest:5m");
    assert_eq!(held(&w), Some(6), "resting too");
}

#[test]
fn a_sin_empties_zeal() {
    let mut w = world("paladin");
    let player = w.state.player;
    set(&mut w, 7);
    w.shift_alignment(player, 20, Deed::PaidDebt);
    assert_eq!(held(&w), Some(7), "a good deed leaves it");
    w.shift_alignment(player, -20, Deed::Theft);
    assert_eq!(held(&w), Some(0));
    assert!(w.notes.iter().any(|n| n == "Your zeal deserts you."));
    // It bars nothing: the pool fills again.
    assert!(w.state.player().resources.has(zeal(&w)));
}

#[test]
fn a_saved_paladin_without_the_pool_holds_it_on_load() {
    let mut w = world("paladin");
    reach(&mut w, 12);
    let id = zeal(&w);
    w.player_mut().resources.remove(id);
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    assert_eq!(held(&restored), Some(5));
}

#[test]
fn a_replay_of_a_smite_verifies() {
    // A paladin of the showcase from the start, as a replay records it: a smite at nobody
    // (refused, free), waits that bring zeal back, and Lay on hands for six of it.
    let content = builtin();
    let spec: crate::character::CharacterSpec = "human,male,paladin".parse().expect("spec");
    let mut w = World::start_run(
        content.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let actions = "use:smite:E wait*12 use:lay_on_hands wait";
    for action in crate::action::parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(held(&w) < Some(2), "5, one back with time, six spent");
    Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
}
