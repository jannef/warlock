//! The stamina pool (spec §8 "Resources"; the combat progression design §4.5, #560): the
//! fighting classes hold it from level 1 by the granted talent Trained wind, its maximum grows
//! with Toughness and the level (`max_from`, read in `Actor::resource_max`), it comes back a
//! point every two base turns, resting too, and Deep lungs, Steady breathing, and Second breath
//! change it through the existing resource hooks.

use super::class_active_tests::{reach, run, world};
use super::*;
use crate::character::CharacterSpec;
use crate::content::{Build, ResourceId};
use crate::stats::{PerStat, Stat, Stats, modifier};

fn stamina(w: &World) -> ResourceId {
    w.content().resource_id("stamina").expect("stamina")
}

fn held(w: &World) -> Option<u16> {
    (w.state.player().resources.get(stamina(w))).map(|p| p.value)
}

fn max(w: &World) -> u16 {
    let level = w.player_level();
    (w.state.player()).resource_max(w.content(), stamina(w), level)
}

fn set(w: &mut World, value: u16) {
    let id = stamina(w);
    w.player_mut().resources.set(id, value);
}

/// The player takes talent `id` (one rank more), without a level to pick it at.
fn give(w: &mut World, talent: &str) {
    assert!(w.content().talents().id(talent).is_some(), "{talent}");
    w.player_mut().talents.take(talent, None);
}

/// Every score `score` (potentials 40).
fn scores(w: &mut World, score: u8) {
    w.player_mut().stats = Stats::new(PerStat::splat(score), PerStat::splat(40));
}

#[test]
fn the_fighting_classes_hold_it_from_level_one_and_the_others_do_not() {
    let content = world("warrior").content.clone();
    let start = |class: &str| {
        let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world")
    };
    for class in [
        "warrior",
        "barbarian",
        "paladin",
        "ranger",
        "monk",
        "thief",
        "bard",
    ] {
        let w = start(class);
        assert_eq!(w.player_level(), 1);
        assert!(
            w.state.player().talents.rank_of("trained_wind", None) > 0,
            "{class} is given Trained wind"
        );
        assert_eq!(held(&w), Some(max(&w)), "{class} holds stamina, full");
    }
    for class in ["wizard", "priest", "witch", "druid", "commoner", "merchant"] {
        let w = start(class);
        assert!(w.state.character.is_some());
        assert_eq!(held(&w), None, "{class} holds none");
    }
}

#[test]
fn a_barbarian_holds_rage_and_stamina_at_once() {
    let mut w = world("barbarian");
    run(&mut w, "use:rage");
    let rage = w.content().resource_id("rage").expect("rage");
    assert!(w.state.player().resources.get(rage).is_some(), "raging");
    assert!(held(&w).is_some(), "and still holding stamina");
}

#[test]
fn the_maximum_grows_with_toughness_and_the_level_and_never_falls_below_five() {
    let mut w = world("warrior");
    scores(&mut w, 14);
    assert_eq!(max(&w), 10 + u16::try_from(modifier(14)).unwrap());
    assert_eq!(max(&w), 12, "Toughness 14 at level 1");
    scores(&mut w, 18);
    assert_eq!(max(&w), 14, "Toughness 18 at level 1");
    scores(&mut w, 1);
    assert!(modifier(1) <= -5);
    assert_eq!(max(&w), 5, "at least 5");
    scores(&mut w, 14);
    reach(&mut w, 20);
    scores(&mut w, 14);
    assert_eq!(max(&w), 14, "a point per 10 levels");
    // Toughness damage lowers it, as it lowers hit points.
    let full = max(&w);
    w.player_mut().damage.take(Stat::Toughness, 6, 10);
    assert!(max(&w) < full);
}

#[test]
fn it_is_granted_full_and_comes_back_a_point_every_two_base_turns_resting_too() {
    let mut w = world("warrior");
    let id = stamina(&w);
    let level = w.player_level();
    let start = (w.state.player()).resource_start(w.content(), id, level);
    assert_eq!(start, max(&w), "it starts full");
    set(&mut w, 0);
    run(&mut w, "wait*2");
    assert_eq!(held(&w), Some(1));
    run(&mut w, "wait*4");
    assert_eq!(held(&w), Some(3));
    set(&mut w, 0);
    run(&mut w, "rest:1m");
    assert_eq!(held(&w), Some(5), "a rest brings it back too");
    run(&mut w, "rest:10m");
    assert_eq!(held(&w), Some(max(&w)), "up to its maximum and no further");
}

#[test]
fn deep_lungs_raises_the_maximum() {
    let mut w = world("warrior");
    let before = max(&w);
    give(&mut w, "deep_lungs");
    assert_eq!(max(&w), before + 4);
}

#[test]
fn steady_breathing_brings_it_back_half_again_as_fast() {
    let mut w = world("warrior");
    let mut twin = World::from_state(w.content.clone(), w.state().clone()).expect("restores");
    give(&mut w, "steady_breathing");
    set(&mut w, 0);
    set(&mut twin, 0);
    run(&mut w, "wait*4");
    run(&mut twin, "wait*4");
    assert_eq!(held(&twin), Some(2));
    assert_eq!(held(&w), Some(3));
}

#[test]
fn second_breath_gives_a_point_back_for_a_kill() {
    let mut w = world("warrior");
    let player = w.state.player;
    set(&mut w, 3);
    w.gain_resource(player, Build::Kill);
    assert_eq!(held(&w), Some(3), "a kill alone gives none");
    give(&mut w, "second_breath");
    w.gain_resource(player, Build::Kill);
    assert_eq!(held(&w), Some(4));
}

#[test]
fn an_old_save_of_a_fighter_is_granted_the_pool_on_load() {
    let mut w = world("ranger");
    reach(&mut w, 4);
    let mut state = w.state().clone();
    let id = stamina(&w);
    let player = state.player;
    let actor = state.actors.get_mut(&player).expect("player");
    assert!(actor.resources.remove(id));
    let restored = World::from_state(w.content.clone(), state).expect("restores");
    // At its start: full by the data (a magic item's more, a story's ring, is not filled).
    let level = restored.player_level();
    let start = (restored.state.player()).resource_start(restored.content(), id, level);
    assert_eq!(held(&restored), Some(start), "granted on load");
    assert!(start <= max(&restored));
    let bytes = crate::save::encode(&restored);
    let loaded = crate::save::decode(&bytes, w.content.clone()).expect("loads");
    assert_eq!(loaded.state(), restored.state());
}

#[test]
fn creatures_never_hold_it() {
    let mut w = world("warrior");
    run(&mut w, "wait*10");
    let player = w.state.player;
    for (id, actor) in &w.state.actors {
        if *id != player {
            assert!(actor.resources.is_empty(), "{id:?} holds a pool");
        }
    }
}
