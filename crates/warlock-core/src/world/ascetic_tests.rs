//! The ascetic's ki tricks (spec §8 "Class abilities"; the ki tricks design §6.2, §9): each
//! refused before its ki is spent, the spend, and what it gives.

use super::class_active_tests::{at_normal_speed, beside, last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Category, Strike};
use crate::content::{ClassActive, ContentFiles, ResourceId};
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

/// The melee blows in `outcome`: who struck whom, and what came of it, in order.
fn strikes(outcome: &Outcome) -> Vec<(ActorId, ActorId, Option<Strike>)> {
    (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Melee,
                strike,
                ..
            } => Some((*attacker, *target, *strike)),
            _ => None,
        })
        .collect()
}

// Patient defence (#849).

const PATIENT: ClassActive = ClassActive::PatientDefence;

#[test]
fn patient_defence_comes_at_level_eighteen() {
    let w = ascetic(17);
    assert_eq!(own(&w, PATIENT), None, "not before level 18");
    let w = ascetic(18);
    assert_eq!(own(&w, PATIENT), Some(None), "in the menu, ready");
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(
        rows.iter().any(|r| r == "Patient defence (class)"),
        "{rows:?}"
    );
}

#[test]
fn patient_defence_is_refused_before_the_spend_without_ki() {
    let mut w = ascetic(18);
    let player = w.state.player;
    let (turn, time) = (w.state.turn, w.state.time);
    set_ki(&mut w, 0);
    assert_eq!(own(&w, PATIENT), Some(Some(Unavailable::Short(ki(&w)))));
    run(&mut w, "use:patient_defence");
    assert_eq!(last(&w), "Your ki is spent.");
    assert!(!suffers(&w, player, "poised"));
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time");
}

#[test]
fn patient_defence_is_a_wait_for_ki_and_poised_lasts_until_the_next_action() {
    let mut w = ascetic(18);
    let player = w.state.player;
    let dodge = w.defence_of(player).bonus(Category::Dodge);
    let ki_before = held_ki(&w).expect("ki");
    let wait = {
        let mut other = ascetic(18);
        other.apply(&Action::Wait).elapsed
    };
    let outcome = w.apply(&parse_actions("use:patient_defence").expect("an action")[0]);
    assert_eq!(outcome.elapsed, wait, "a wait's time");
    assert_eq!(held_ki(&w), Some(ki_before - 1), "a point of ki");
    assert_eq!(last(&w), "You settle into your guard.");
    // The creatures' turns after it meet it: it outlived the action that gave it.
    assert!(suffers(&w, player, "poised"));
    assert_eq!(w.defence_of(player).bonus(Category::Dodge), dodge + 4);
    assert!(w.free_blows_of(w.state.player()).riposte, "the riposte on");
    // The next action, whatever it is, ends it, silently.
    w.apply(&Action::Wait);
    assert!(!suffers(&w, player, "poised"));
    assert_eq!(w.defence_of(player).bonus(Category::Dodge), dodge);
    assert!(
        !w.free_blows_of(w.state.player()).riposte,
        "the riposte off"
    );
    // Taken again and again, it holds through each.
    run(&mut w, "use:patient_defence use:patient_defence");
    assert!(suffers(&w, player, "poised"));
}

#[test]
fn poised_a_dodged_or_parried_blow_is_answered_at_once() {
    let mut w = ascetic(18);
    let player = w.state.player;
    let kobold = beside(&mut w, "kobold", 5000);
    let dodged = |s: Option<Strike>| {
        matches!(
            s,
            Some(Strike::Missed {
                by: Some(Category::Dexterity | Category::Dodge | Category::Parry)
            })
        )
    };
    // Not Poised, a dodged blow goes unanswered (an ascetic has no riposte of their own).
    let mut unanswered = 0;
    for _ in 0..40 {
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        let outcome = w.apply(&Action::Wait);
        let struck = strikes(&outcome);
        assert!(struck.iter().all(|&(by, ..)| by == kobold), "{struck:?}");
        unanswered += struck.iter().filter(|&&(.., s)| dodged(s)).count();
    }
    assert!(unanswered > 0, "the kobold missed some");
    // Poised, every blow dodged is answered by one back, and only those.
    let mut answered = 0;
    for _ in 0..60 {
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        set_ki(&mut w, 5);
        let outcome = w.apply(&parse_actions("use:patient_defence").expect("an action")[0]);
        let struck = strikes(&outcome);
        let Some(&(by, at, strike)) = struck.first() else {
            continue;
        };
        assert_eq!((by, at), (kobold, player));
        if dodged(strike) {
            assert_eq!(
                struck.get(1).map(|&(b, a, _)| (b, a)),
                Some((player, kobold))
            );
            answered += 1;
        } else {
            assert_eq!(struck.len(), 1, "a hit is not answered: {struck:?}");
        }
    }
    assert!(answered > 0, "the kobold missed some");
}

#[test]
fn patient_defence_replays() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:200000 levelup:auto use:patient_defence wait use:patient_defence";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 18, "level {}", w.player_level());
    assert!(suffers(&w, w.state.player, "poised"));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
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

// Stillness (#850).

const STILL: ClassActive = ClassActive::Stillness;

/// The statuses Stillness ends, in its data's order.
const STILLED: [&str; 9] = [
    "held",
    "pinned",
    "staggered",
    "stunned",
    "surprised",
    "transfixed",
    "frightened",
    "drowsy",
    "webbed",
];

/// Everything said since `from` messages were in the log.
fn said_since(w: &World, from: usize) -> Vec<String> {
    (w.messages().iter().skip(from))
        .map(|m| m.display())
        .collect()
}

#[test]
fn stillness_comes_with_still_mind_at_level_twenty_five() {
    let w = ascetic(24);
    assert_eq!(own(&w, STILL), None, "not before level 25");
    let mut w = ascetic(25);
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(rows.iter().any(|r| r == "Still mind (class)"), "{rows:?}");
    let row = (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Class(STILL))
        .expect("in the menu");
    assert_eq!(row.ability.name(w.content()), "Stillness");
    assert_eq!(row.unavailable, Some(Unavailable::Still), "nothing to end");
    run(&mut w, "afflict:stunned");
    assert_eq!(own(&w, STILL), Some(None), "ready with a stun to end");
    let mods = w.state.player().talent_mods(w.content());
    let ends: Vec<&str> = (mods.stillness.as_ref().expect("stillness").iter())
        .map(|&s| w.content().status(s).id.as_str())
        .collect();
    assert_eq!(ends, STILLED);
}

#[test]
fn stillness_is_refused_before_the_spend() {
    let mut w = ascetic(25);
    let player = w.state.player;
    let (turn, time) = (w.state.turn, w.state.time);
    // Nothing to end: refused, the ki kept, no time.
    set_ki(&mut w, 5);
    run(&mut w, "use:stillness");
    assert_eq!(last(&w), "Your mind is already still.");
    assert_eq!(held_ki(&w), Some(5), "nothing spent");
    // Suffering only what it does not end (poison) is nothing to end either.
    run(&mut w, "afflict:poisoned use:stillness");
    assert_eq!(last(&w), "Your mind is already still.");
    assert_eq!(held_ki(&w), Some(5));
    // One point of ki is not enough for two: greyed and refused, the stun kept.
    set_ki(&mut w, 1);
    run(&mut w, "afflict:stunned");
    assert_eq!(own(&w, STILL), Some(Some(Unavailable::Short(ki(&w)))));
    run(&mut w, "use:stillness");
    assert_eq!(last(&w), "Your ki is spent.");
    assert!(suffers(&w, player, "stunned"));
    assert_eq!(held_ki(&w), Some(1));
    assert_eq!((w.state.turn, w.state.time), (turn, time), "no time at all");
}

#[test]
fn stillness_ends_what_holds_and_clouds_at_once_and_no_more() {
    let mut w = ascetic(25);
    let player = w.state.player;
    set_ki(&mut w, 5);
    run(
        &mut w,
        "afflict:held afflict:stunned afflict:frightened afflict:webbed afflict:poisoned",
    );
    for id in ["held", "stunned", "frightened", "webbed", "poisoned"] {
        assert!(suffers(&w, player, id), "{id}");
    }
    let from = w.messages().iter().count();
    let outcome = w.apply(&parse_actions("use:stillness").expect("an action")[0]);
    assert_eq!(outcome.elapsed, crate::time::Duration::ZERO, "no time");
    assert_eq!(held_ki(&w), Some(3), "two ki");
    for id in ["held", "stunned", "frightened", "webbed"] {
        assert!(!suffers(&w, player, id), "{id} ended");
    }
    assert!(suffers(&w, player, "poisoned"), "not in its list");
    let said = said_since(&w, from);
    assert_eq!(
        said.first().map(String::as_str),
        Some("You still your mind.")
    );
    for words in [
        "You are free to move.",
        "Your head clears.",
        "You master your fear.",
        "You tear free of the web.",
    ] {
        assert!(said.iter().any(|s| s.contains(words)), "{words}: {said:?}");
    }
    // The stun is gone before the next action: it costs its plain price.
    let freed = w.apply(&Action::Wait).elapsed;
    let plain = w.apply(&Action::Wait).elapsed;
    assert_eq!(freed, plain, "no stun left to slow it");
}

#[test]
fn stillness_names_only_statuses_the_content_has() {
    let mut files = ContentFiles::builtin();
    let still = include_str!("../../../../assets/data/talents/still_mind.toml");
    let bad = still.replace("\"webbed\",", "\"befuddled\",");
    files.insert("data/talents/still_mind.toml", &bad);
    let err = Content::load(&files)
        .expect_err("unknown status")
        .to_string();
    assert!(err.contains("unknown status `befuddled`"), "{err}");
    let mut files = ContentFiles::builtin();
    let empty = (still.split_once("statuses = [").map(|(head, tail)| {
        let (_, rest) = tail.split_once(']').expect("the list ends");
        format!("{head}statuses = []{rest}")
    }))
    .expect("a list");
    files.insert("data/talents/still_mind.toml", &empty);
    let err = Content::load(&files)
        .expect_err("an empty list")
        .to_string();
    assert!(err.contains("stillness"), "{err}");
}

#[test]
fn stillness_replays() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:3000000 levelup:auto afflict:held afflict:stunned use:stillness move:E \
                use:stillness wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.player_level() >= 25, "level {}", w.player_level());
    let player = w.state.player;
    assert!(!suffers(&w, player, "held") && !suffers(&w, player, "stunned"));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
