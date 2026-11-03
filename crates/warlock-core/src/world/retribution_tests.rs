//! Retribution (`world/smite.rs::retribution`; spec §8 "Class abilities", "Free blows"; the
//! paladin design P3, #912): a hostile's blow that lands is answered by the paladin's smite for
//! four zeal, never as well as a riposte, not outside the bands, and deterministically.

use super::class_active_tests::{beside, reach, world};
use super::free_blow::{Cue, Free};
use super::*;
use crate::action::AttackKind;
use crate::content::ResourceId;

fn zeal(w: &World) -> ResourceId {
    w.content().resource_id("zeal").expect("zeal")
}

fn held(w: &World) -> u16 {
    (w.state.player().resources.get(zeal(w))).map_or(0, |p| p.value)
}

fn set(w: &mut World, value: u16) {
    let id = zeal(w);
    w.player_mut().resources.set(id, value);
}

/// A level-25 paladin with a kobold beside them (east) that does not die of a smite.
fn paladin_and_kobold() -> (World, ActorId) {
    let mut w = world("paladin");
    reach(&mut w, 25);
    assert!(w.state.player().talents.has("retribution", 1), "at 25");
    let kobold = beside(&mut w, "kobold", 5000);
    // The player has noticed it: its attacks never catch them unawares (Surprised).
    let turn = w.state.turn;
    w.state
        .actors
        .get_mut(&kobold)
        .expect("kobold")
        .mind
        .noticed = Some(turn);
    (w, kobold)
}

/// The kobold's melee attack on the player: whether a blow of it hit, and whether the player
/// struck back.
fn attacked(w: &mut World, kobold: ActorId) -> (bool, bool) {
    let player = w.state.player;
    let hp = w.state.player().hp;
    let mut events = Vec::new();
    w.attack(kobold, player, AttackKind::Melee, &mut events);
    w.player_mut().hp = hp;
    let hit = events.iter().any(|e| {
        matches!(e, Event::Attacked { attacker, strike: Some(s), .. }
            if *attacker == kobold && s.is_hit())
    });
    let back = (events.iter())
        .any(|e| matches!(e, Event::Attacked { attacker, .. } if *attacker == player));
    (hit, back)
}

const ANSWER: &str = "You answer the blow with holy wrath!";

#[test]
fn a_hit_is_answered_for_four_zeal_and_a_miss_is_not() {
    let (mut w, kobold) = paladin_and_kobold();
    let (mut answered, mut short, mut missed) = (0, 0, 0);
    for round in 0..300 {
        // At 3 the blow taken builds the fourth point first; at 2 it leaves three: too few.
        let start = if round % 2 == 0 { 3 } else { 2 };
        set(&mut w, start);
        w.news.clear();
        let (hit, back) = attacked(&mut w, kobold);
        let said = w.news.iter().any(|n| n == ANSWER);
        assert_eq!(back, said, "the line comes with the blow");
        match (hit, start) {
            (false, _) => {
                assert!(!back, "a miss is never answered");
                assert_eq!(held(&w), start);
                missed += 1;
            }
            (true, 3) => {
                assert!(
                    back,
                    "struck to four, it answers: {} {:?}",
                    held(&w),
                    w.news
                );
                // Four spent; the smite back may have hit (a point back).
                assert!(held(&w) <= 1, "{}", held(&w));
                answered += 1;
            }
            (true, _) => {
                assert!(!back, "struck to three: too few");
                assert_eq!(held(&w), 3, "only the point the blow taken built");
                short += 1;
            }
        }
    }
    assert!(
        answered > 0 && short > 0 && missed > 0,
        "{answered} {short} {missed}"
    );
}

#[test]
fn one_free_blow_an_attack_retribution_before_the_riposte() {
    let (mut w, kobold) = paladin_and_kobold();
    let player = w.state.player;
    w.player_mut().free_blows.riposte = true;
    set(&mut w, 10);
    let mut events = Vec::new();
    // A parried blow and one that hit: Retribution's smite alone.
    let both = Cue::Attacked {
        parried: true,
        dodged: false,
        hit: true,
    };
    let blow = w.second_blow(kobold, player, both, &mut events);
    assert_eq!(blow.map(|b| b.kind), Some(Free::Retribution));
    assert_eq!(held(&w), 6, "four spent");
    // Parried and nothing hit: the riposte, nothing spent.
    let parried = Cue::Attacked {
        parried: true,
        dodged: false,
        hit: false,
    };
    let blow = w.second_blow(kobold, player, parried, &mut events);
    assert_eq!(blow.map(|b| b.kind), Some(Free::Riposte));
    assert_eq!(held(&w), 6);
    // Short of zeal, a parried hit is riposted instead.
    set(&mut w, 3);
    let blow = w.second_blow(kobold, player, both, &mut events);
    assert_eq!(blow.map(|b| b.kind), Some(Free::Riposte));
    assert_eq!(held(&w), 3);
}

#[test]
fn not_for_a_neutral_paladin_and_not_said() {
    let (mut w, kobold) = paladin_and_kobold();
    let player = w.state.player;
    w.player_mut().alignment = Some(crate::alignment::Score::new(0));
    set(&mut w, 10);
    w.news.clear();
    let hit = Cue::Attacked {
        parried: false,
        dodged: false,
        hit: true,
    };
    let mut events = Vec::new();
    assert!(w.second_blow(kobold, player, hit, &mut events).is_none());
    assert_eq!(held(&w), 10, "nothing spent");
    assert!(w.news.is_empty() && events.is_empty(), "no line");
    // Kind again, it answers.
    w.player_mut().alignment = Some(crate::alignment::Score::new(200));
    let blow = w.second_blow(kobold, player, hit, &mut events);
    assert_eq!(blow.map(|b| b.kind), Some(Free::Retribution));
}

#[test]
fn creatures_and_those_without_it_never_retribute() {
    let (mut w, kobold) = paladin_and_kobold();
    let player = w.state.player;
    set(&mut w, 10);
    let hit = Cue::Attacked {
        parried: false,
        dodged: false,
        hit: true,
    };
    let mut events = Vec::new();
    // The kobold struck by the player: creatures hold no zeal and no Retribution.
    assert!(w.second_blow(player, kobold, hit, &mut events).is_none());
    // Surprised, the paladin strikes nothing out of their turn, as a riposte.
    let surprised = w.content().status_id("surprised").expect("surprised");
    w.afflict(player, surprised, 1, None, &mut events);
    assert!(w.second_blow(kobold, player, hit, &mut events).is_none());
    assert_eq!(held(&w), 10);
    // A paladin below 25 has it not.
    let mut young = world("paladin");
    let kobold = beside(&mut young, "kobold", 5000);
    set(&mut young, 10);
    let player = young.state.player;
    assert!(
        young
            .second_blow(kobold, player, hit, &mut events)
            .is_none()
    );
    assert_eq!(held(&young), 10);
}

#[test]
fn the_answering_blows_draw_the_same_every_time() {
    let (mut w, kobold) = paladin_and_kobold();
    let (mut again, other) = paladin_and_kobold();
    assert_eq!(kobold, other);
    for _ in 0..40 {
        set(&mut w, 8);
        set(&mut again, 8);
        assert_eq!(attacked(&mut w, kobold), attacked(&mut again, kobold));
    }
    assert_eq!(w.state_hash(), again.state_hash());
}
