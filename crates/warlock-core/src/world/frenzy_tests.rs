//! The frenzy stance (spec §8 "Stances"; #680): rage puts a berserker in it and holds them
//! there, every end of the rage leaves them aggressive, and nothing else ever gives it.

use super::class_active_tests::world;
use super::*;
use crate::content::CombatStance::{self, *};
use crate::test_support::{builtin, last, run};

fn stance(w: &World) -> CombatStance {
    w.perception().own_stance()
}

fn raging(w: &World) -> bool {
    let raging = w.content().status_id("raging").expect("raging");
    w.state.player().effects.has(raging)
}

#[test]
fn rage_forces_the_frenzy_stance_from_every_other() {
    for from in CombatStance::CHOSEN {
        let mut w = world("berserker");
        run(&mut w, &format!("stance:{from}"));
        assert_eq!(stance(&w), from);
        run(&mut w, "use:rage");
        assert!(raging(&w));
        assert_eq!(stance(&w), Frenzy, "from {from}");
        assert_eq!(w.state.player().stance, Frenzy);
    }
    // The dev grant starts Raging too, and with it the stance.
    let mut w = world("berserker");
    run(&mut w, "stance:coward resource:rage");
    assert_eq!(stance(&w), Frenzy);
}

#[test]
fn rage_holds_the_stance_and_nothing_else_gives_it() {
    let mut w = world("berserker");
    // Never chosen: refused, no time, with a line.
    let out = w.apply(&Action::SetStance(Frenzy));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last(&w), "Only rage brings the frenzy.");
    assert_eq!(stance(&w), Neutral);
    run(&mut w, "use:rage");
    let told = w.messages().len();
    for to in CombatStance::ALL {
        let out = w.apply(&Action::SetStance(to));
        assert!(!out.took_turn && out.elapsed.is_zero(), "{to}");
        assert!(last(&w).starts_with("Your rage holds you in the frenzy."));
        assert_eq!(stance(&w), Frenzy);
    }
    assert!(w.messages().len() > told);
    // The keys step nowhere from it.
    assert_eq!(Frenzy.more_aggressive(), Frenzy);
    assert_eq!(Frenzy.more_defensive(), Frenzy);
}

#[test]
fn only_a_berserkers_own_rage_brings_the_frenzy() {
    // A warrior granted rage (the dev grant: Raging with it) is not a berserker.
    let mut w = world("warrior");
    run(&mut w, "stance:defensive resource:rage");
    assert!(raging(&w));
    assert_eq!(stance(&w), Defensive);
    run(&mut w, "stance:aggressive");
    assert_eq!(stance(&w), Aggressive, "not held");
    run(&mut w, "stance:frenzy");
    assert_eq!(stance(&w), Aggressive);
    // A creature that suffers Raging stays neutral.
    let raging_id = w.content().status_id("raging").expect("raging");
    let other = (w.state.actors.keys())
        .copied()
        .find(|&id| id != w.state.player)
        .expect("a creature in the showcase");
    w.afflict(other, raging_id, 1, None, &mut Vec::new());
    assert_eq!(w.state.actors[&other].stance, Neutral);
}

#[test]
fn every_end_of_the_rage_leaves_the_berserker_aggressive() {
    // The rage running out.
    let mut w = world("berserker");
    run(&mut w, "stance:coward use:rage wait*30");
    assert!(!raging(&w));
    assert_eq!(stance(&w), Aggressive, "not the coward's from before");
    assert!(
        (w.messages().iter()).any(|m| m.display() == "You fall back into an aggressive stance.")
    );
    run(&mut w, "stance:defensive");
    assert_eq!(stance(&w), Defensive, "free to choose again");
    // Raging ended otherwise (cured, `World::ended`).
    let mut w = world("berserker");
    run(&mut w, "use:rage");
    let raging_id = w.content().status_id("raging").expect("raging");
    let effect = *w.state.player().effects.get(raging_id).expect("raging");
    w.player_mut().effects.remove(raging_id);
    let player = w.state.player;
    w.ended(player, &[effect], &mut Vec::new());
    assert_eq!(stance(&w), Aggressive);
    // The rage spent to nothing (Unstoppable's cost, any spending).
    let mut w = world("berserker");
    run(&mut w, "use:rage");
    let rage = w.content().resource_id("rage").expect("rage");
    let held = w.state.player().resources.get(rage).expect("held").value;
    assert!(w.spend_resource(player, rage, held, &mut Vec::new()));
    assert!(!raging(&w));
    assert_eq!(stance(&w), Aggressive);
    // Rest through it: the rage decays while resting too.
    let mut w = world("berserker");
    run(&mut w, "use:rage rest:1h");
    assert!(!raging(&w));
    assert_eq!(stance(&w), Aggressive);
}

#[test]
fn a_save_mid_rage_comes_back_raging() {
    let mut w = world("berserker");
    run(&mut w, "use:rage");
    let content = w.content().clone();
    let back = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.perception().own_stance(), Frenzy);
}

#[test]
fn the_frenzy_hits_harder_and_more_often_and_guards_worse_than_the_aggressive() {
    let mut w = world("berserker");
    let content = w.content().clone();
    let (aggressive, frenzy) = (content.stance_def(Aggressive), content.stance_def(Frenzy));
    assert!(frenzy.to_hit > aggressive.to_hit);
    assert!(frenzy.damage > aggressive.damage);
    assert!(frenzy.dodge < aggressive.dodge);
    let player = w.state.player;
    let measure = |w: &mut World, s: CombatStance| {
        w.player_mut().stance = s;
        let profile = w.state.player().melee(&content);
        (profile.to_hit, profile.bonus, w.defence_of(player).total())
    };
    let (hit_a, damage_a, dv_a) = measure(&mut w, Aggressive);
    let (hit_b, damage_b, dv_b) = measure(&mut w, Frenzy);
    assert_eq!(hit_b - hit_a, frenzy.to_hit - aggressive.to_hit);
    assert_eq!(damage_b - damage_a, frenzy.damage - aggressive.damage);
    assert_eq!(dv_b - dv_a, frenzy.dodge - aggressive.dodge);
    // Wild guard halves the frenzy's Dodge penalty.
    w.player_mut().talents.take("wild_guard", None);
    let (_, _, dv_wild) = measure(&mut w, Frenzy);
    assert_eq!(dv_wild - dv_b, -frenzy.dodge - (-frenzy.dodge) / 2);
}

#[test]
fn rage_is_all_a_replay_needs() {
    let content = builtin();
    let spec: crate::character::CharacterSpec = "human,male,berserker".parse().expect("spec");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = crate::replay::Replay::new(&w);
    for action in crate::action::parse_actions("stance:coward use:rage stance:neutral wait*20")
        .expect("actions")
    {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = crate::replay::Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    assert_eq!(verified.perception().own_stance(), Aggressive);
}
