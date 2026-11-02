//! The ascetic's ki tricks (spec §8 "Class abilities"; the ki tricks design §6.2, §9): each
//! refused before its ki is spent, the spend, and what it gives.

use super::class_active_tests::{at_normal_speed, last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::content::{ClassActive, ResourceId};
use crate::dump::{self, Layer};
use crate::replay::Replay;

fn ki(w: &World) -> ResourceId {
    w.content().resource_id("ki").expect("ki")
}

fn held_ki(w: &World) -> Option<u16> {
    (w.state.player().resources.get(ki(w))).map(|p| p.value)
}

fn set_ki(w: &mut World, value: u16) {
    let ki = ki(w);
    w.player_mut().resources.set(ki, value);
}

/// Why the player cannot use `active` now, as the abilities menu has it; `None` without it.
fn own(w: &World, active: ClassActive) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Class(active))
        .map(|a| a.unavailable)
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

// Step of the wind (#847).

const STEP: ClassActive = ClassActive::StepOfTheWind;

#[test]
fn step_of_the_wind_comes_at_level_six_with_fleets_quicker_steps() {
    let w = ascetic(5);
    assert_eq!(own(&w, STEP), None, "not before level 6");
    let w = ascetic(6);
    assert_eq!(own(&w, STEP), Some(None), "in the menu, ready");
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(
        rows.iter().any(|r| r == "Step of the wind (class)"),
        "{rows:?}"
    );
    assert!(
        w.content().talents().id("fleet").is_none(),
        "Fleet is folded in"
    );
    let mods = w.state.player().talent_mods(w.content());
    assert_eq!(mods.step_percent(false, false, false), -10, "unarmoured");
    assert_eq!(mods.step_percent(true, false, false), 0, "armoured");
    assert_eq!(mods.step_percent(false, false, true), 0, "with a shield");
}

#[test]
fn step_of_the_wind_is_refused_before_the_spend() {
    let mut w = ascetic(6);
    let player = w.state.player;
    let (turn, time) = (w.state.turn, w.state.time);
    // No ki: greyed in the menu, refused, nothing given.
    set_ki(&mut w, 0);
    assert_eq!(own(&w, STEP), Some(Some(Unavailable::Short(ki(&w)))));
    run(&mut w, "use:step_of_the_wind");
    assert_eq!(last(&w), "Your ki is spent.");
    assert!(!suffers(&w, player, "light_footed"));
    // Light-footed already: refused, the ki kept.
    set_ki(&mut w, 3);
    run(&mut w, "use:step_of_the_wind");
    assert!(suffers(&w, player, "light_footed"));
    assert_eq!(held_ki(&w), Some(2));
    run(&mut w, "use:step_of_the_wind");
    assert_eq!(last(&w), "You are already light on your feet.");
    assert_eq!(held_ki(&w), Some(2), "nothing spent");
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time at all");
}

#[test]
fn step_of_the_wind_takes_no_time_and_halves_the_next_action_once() {
    let mut w = ascetic(6);
    let player = w.state.player;
    let ki_before = held_ki(&w).expect("ki");
    let outcome = w.apply(&parse_actions("use:step_of_the_wind").expect("an action")[0]);
    assert_eq!(outcome.elapsed, crate::time::Duration::ZERO, "no time");
    assert_eq!(held_ki(&w), Some(ki_before - 1), "a point of ki");
    assert!(suffers(&w, player, "light_footed"));
    // The next action, whatever it is, costs half, and ends it.
    let quick = w.apply(&Action::Wait).elapsed;
    assert!(!suffers(&w, player, "light_footed"), "spent by one action");
    let slow = w.apply(&Action::Wait).elapsed;
    assert_eq!(quick * 2, slow, "the next action costs half");
}

#[test]
fn step_of_the_wind_replays() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:20000 levelup:auto use:step_of_the_wind move:E use:step_of_the_wind wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 6, "level {}", w.player_level());
    let ki_max = (w.state.player()).resource_max(w.content(), ki(&w), w.player_level());
    assert!(held_ki(&w) < Some(ki_max), "ki spent");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

// Diamond soul's catch (#851).

/// The arrow trap's attack, made sure to hit but for a natural 2 (one roll in 144).
fn sure_shot(w: &World) -> (crate::content::TrapId, crate::combat::Profile) {
    let content = w.content();
    let trap = content.trap_id("arrow").expect("the arrow trap");
    let crate::content::TrapEffect::Shot { attack, .. } = &content.trap(trap).effect else {
        panic!("the arrow trap shoots");
    };
    let mut profile = crate::combat::Profile::bare(attack);
    profile.to_hit += 60;
    (trap, profile)
}

/// Arrows lying on cell `at`.
fn arrows_at(w: &World, at: Pos) -> u32 {
    let arrow = w.content().object_id("arrow").expect("arrows");
    (w.state.current().objects_at(at).iter())
        .filter(|s| s.object == arrow)
        .map(|s| s.count)
        .sum()
}

/// A kobold east of the player shoots `shots` sure arrows at them; the attacks' strikes.
fn shot_at(w: &mut World, shots: usize) -> Vec<Option<crate::combat::Strike>> {
    let shooter = super::class_active_tests::beside(w, "kobold", 50);
    let (_, profile) = sure_shot(w);
    let arrow = w.content().object_id("arrow").expect("arrows");
    let at = w.player_pos();
    let mut strikes = Vec::new();
    for _ in 0..shots {
        let flight = super::shoot::Flight {
            kind: AttackKind::Ranged,
            with: arrow,
            flies: arrow,
            profile: profile.clone(),
            range: 10,
            lands: Some(crate::object::Stack::one(arrow)),
            breaks: 0,
        };
        let mut events = Vec::new();
        w.fly(shooter, at, flight, &mut events);
        strikes.extend(events.iter().filter_map(|e| match e {
            Event::Attacked { target, strike, .. } if *target == w.state.player => Some(*strike),
            _ => None,
        }));
    }
    strikes
}

#[test]
fn diamond_soul_plucks_a_shot_from_the_air_for_a_point_of_ki() {
    let mut w = ascetic(40);
    let mods = w.state.player().talent_mods(w.content());
    assert!(
        mods.catch_missiles.is_some(),
        "the catch comes with Diamond soul"
    );
    assert_eq!(mods.deflection, 2, "beside its Deflection");
    set_ki(&mut w, 10);
    let (here, hp) = (w.player_pos(), w.state.player().hp);
    let strikes = shot_at(&mut w, 5);
    assert_eq!(strikes.len(), 5, "every arrow reached them");
    let caught = 10 - held_ki(&w).expect("ki");
    assert!(caught >= 4, "the sure hits were caught: {strikes:?}");
    assert!(
        strikes.iter().all(|s| !s.is_some_and(|s| s.is_hit())),
        "a caught hit is a miss: {strikes:?}"
    );
    assert_eq!(w.state.player().hp, hp, "nothing struck home");
    assert_eq!(
        arrows_at(&w, here),
        u32::from(caught),
        "each fell at their feet"
    );
    let plucked = (w.news.iter()).filter(|n| *n == "You pluck the arrow from the air.");
    assert_eq!(plucked.count(), usize::from(caught), "{:?}", w.news);
}

#[test]
fn the_catch_needs_ki_and_empty_hands() {
    // No ki: the hits stand.
    let mut w = ascetic(40);
    set_ki(&mut w, 0);
    let hp = w.state.player().hp;
    let strikes = shot_at(&mut w, 3);
    assert!(
        strikes.iter().any(|s| s.is_some_and(|s| s.is_hit())),
        "{strikes:?}"
    );
    assert!(w.state.player().hp < hp, "struck");
    assert!(w.news.is_empty(), "{:?}", w.news);
    // A weapon in hand: the hits stand, the ki kept.
    let mut w = ascetic(40);
    let dagger = w.content().object_id("dagger").expect("a dagger");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::one(dagger));
    let item = w.state.player().inventory.len() - 1;
    run(&mut w, &format!("equip:main_hand:{item}"));
    assert!(w.state.player().weapon_item(w.content()).is_some());
    set_ki(&mut w, 10);
    let strikes = shot_at(&mut w, 3);
    assert!(
        strikes.iter().any(|s| s.is_some_and(|s| s.is_hit())),
        "{strikes:?}"
    );
    assert_eq!(held_ki(&w), Some(10), "nothing spent");
    // Before Diamond soul: no catch.
    let mut w = ascetic(39);
    set_ki(&mut w, 10);
    let strikes = shot_at(&mut w, 3);
    assert!(
        strikes.iter().any(|s| s.is_some_and(|s| s.is_hit())),
        "{strikes:?}"
    );
    assert_eq!(held_ki(&w), Some(10), "nothing spent");
}

#[test]
fn diamond_soul_catches_a_traps_arrow_too() {
    let mut w = ascetic(40);
    let player = w.state.player;
    let here = w.player_pos();
    set_ki(&mut w, 20);
    let hp = w.state.player().hp;
    let (trap, _) = sure_shot(&w);
    let content = w.content().clone();
    let mut events = Vec::new();
    for _ in 0..12 {
        let set = crate::content::Trap::new(trap, content.trap(trap), false, None);
        w.state.current_mut().traps.insert(here, set);
        w.spring_trap(here, player, &mut events);
    }
    let caught = 20 - held_ki(&w).expect("ki");
    assert!(caught > 0, "some arrow of twelve hit and was caught");
    let struck = (events.iter())
        .filter(|e| matches!(e, Event::TrapStruck { strike, .. } if strike.is_hit()));
    assert_eq!(struck.count(), 0, "every hit was caught");
    assert_eq!(w.state.player().hp, hp, "nothing struck home");
    assert_eq!(
        arrows_at(&w, here),
        u32::from(caught),
        "each fell at their feet"
    );
    let plucked = (w.news.iter()).filter(|n| *n == "You pluck the arrow from the air.");
    assert_eq!(plucked.count(), usize::from(caught), "{:?}", w.news);
}

#[test]
fn the_catch_replays_in_the_trap_hall() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w = World::start_run(content, "traps", None, Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    // Bare-handed at level 40 and more, on and off the proving row's arrow trap.
    let text = "remove:main_hand xp:5000000 levelup:auto move:N*14 move:W*21 move:N*3 \
                move:S move:N move:S move:N move:S move:N move:S move:N move:S move:N";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "You pluck the arrow from the air."),
        "{said:?}"
    );
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
