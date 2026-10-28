//! Concentration's hold on mana (#668): the pace and the pool by value, nothing untrained, the
//! pool's fifth, the maximum falling with the skill, and the `cast` marks.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::skills::Skills;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// An elf wizard in the showcase, every score 14, trained in nothing but Concentration at
/// `value` (untrained with `None`), with a pool of `base` before Concentration, full.
fn wizard(value: Option<u16>, base: i32) -> World {
    let spec: CharacterSpec = "elf,female,wizard".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let concentration = w
        .content()
        .skill_id("concentration")
        .expect("concentration");
    let player = w.player_mut();
    player.stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    player.skills = Skills::default();
    if let Some(value) = value {
        player.skills.add(concentration, i32::from(value));
    }
    // Every score 14 gives the three mental modifiers +2 each.
    player.mana_base = base - 6;
    let max = w.state.player().max_mana(&w.content);
    w.player_mut().mana = max;
    w
}

fn max(w: &World) -> i32 {
    w.state.player().max_mana(&w.content)
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn marks(w: &World) -> u32 {
    let id = w
        .content()
        .skill_id("concentration")
        .expect("concentration");
    w.state.player().skills.get(id).map_or(0, |t| t.marks)
}

#[test]
fn untrained_or_at_zero_it_gives_nothing() {
    for value in [None, Some(0)] {
        let w = wizard(value, 20);
        assert_eq!(max(&w), 20, "{value:?}");
        assert_eq!(w.mana_rate(w.player_id()), 100, "{value:?}");
    }
}

#[test]
fn the_pace_grows_with_the_value() {
    for (value, rate) in [(20, 105), (60, 122), (100, 140), (200, 160)] {
        let w = wizard(Some(value), 20);
        assert_eq!(w.mana_rate(w.player_id()), rate, "at {value}");
    }
}

#[test]
fn the_pool_grows_a_little_and_at_most_a_fifth() {
    for (value, base, deeper) in [
        (24, 20, 20),
        (25, 20, 21),
        (100, 30, 35),
        (100, 20, 24),
        (200, 30, 36),
        (100, 4, 4),
        (100, 0, 0),
    ] {
        let w = wizard(Some(value), base);
        assert_eq!(max(&w), deeper, "{base} at {value}");
        assert_eq!(w.state.player().lasting_max_mana(&w.content), deeper);
    }
}

#[test]
fn mana_follows_the_maximum_down_when_the_skill_goes() {
    let mut w = wizard(Some(100), 30);
    assert_eq!(w.state.player().mana, 35);
    w.player_mut().skills = Skills::default();
    run(&mut w, "wait");
    assert_eq!(w.state.player().mana, 30);
}

#[test]
fn spending_mana_earns_marks_only_when_trained() {
    let mut w = wizard(Some(10), 40);
    run(&mut w, "drain:1*40");
    let got = marks(&w);
    assert!(
        (3..=20).contains(&got),
        "{got} marks from 40 spends at 25 %"
    );

    let mut w = wizard(None, 40);
    run(&mut w, "drain:1*40");
    assert_eq!(marks(&w), 0);
}

#[test]
fn faster_mana_comes_back_sooner() {
    let mut slow = wizard(None, 30);
    let mut fast = wizard(Some(100), 30);
    for w in [&mut slow, &mut fast] {
        w.player_mut().mana = 0;
        run(w, "wait*600");
    }
    assert!(
        fast.state.player().mana > slow.state.player().mana,
        "{} vs {}",
        fast.state.player().mana,
        slow.state.player().mana
    );
}

#[test]
fn a_level_fills_what_it_deepens_too() {
    let mut w = wizard(Some(100), 20);
    assert_eq!((w.state.player().mana, max(&w)), (24, 24));
    let content = w.content.clone();
    w.player_mut().grow_mana_base(&content, 10);
    // A pool of 30, deepened by Concentration's 5: eleven more, all filled.
    assert_eq!((w.state.player().mana, max(&w)), (35, 35));
}
