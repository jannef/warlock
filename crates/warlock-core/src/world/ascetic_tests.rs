//! The ascetic's ki tricks (spec §8 "Class abilities"; the ki tricks design §6.2, §9): each
//! refused before its ki is spent, the spend, and what it gives.

use super::class_active_tests::{at_normal_speed, beside, last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Category, Strike};
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
