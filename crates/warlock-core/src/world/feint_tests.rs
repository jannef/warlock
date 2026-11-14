//! Feint (spec §8 "Skills": Bluff; #1016; `world/feint.rs`): Bluff's use; made and failed; its
//! refusals, with no time and no draw; a blow after it finds the foe flat-footed, though the foe
//! acted between; Sneak attack's dice for a thief; marks on the training stream.

use super::class_active_tests::world;
use super::intimidate_tests::foe_at;
use super::*;
use crate::ability::Ability;
use crate::rng::Rng;
use crate::skills::{Effect, Trained};
use crate::stats::{PerStat, Stat, Stats};
use crate::test_support::{builtin, last};

/// The player's skill `id` set to `value` (trained), the rest of their skills as they are.
fn train(w: &mut World, id: &str, value: Option<u16>) {
    let skill = w.content().skill_id(id).expect("a skill");
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<_> = skills.iter().filter(|(s, _)| *s != skill).collect();
    if let Some(value) = value {
        all.push((skill, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

/// A `class` alone on the level, Presence `presence` (the rest 14), Bluff `bluff`, plenty of hit
/// points to take blows with.
fn liar(class: &str, presence: u8, bluff: Option<u16>) -> World {
    let mut w = world(class);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let mut scores = PerStat::splat(14);
    scores[Stat::Presence] = presence;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
    w.player_mut().hp_base = 500;
    w.player_mut().hp = 500;
    train(&mut w, "bluff", bluff);
    w
}

/// The cell east of the player.
fn beside(w: &World) -> Pos {
    w.player_pos().step(Dir::E)
}

/// A kobold of every score `score` east of the player.
fn kobold_beside(w: &mut World, score: u8) -> ActorId {
    let at = beside(w);
    foe_at(w, "kobold", at, score)
}

fn feint(w: &mut World) -> Outcome {
    w.perceive(&mut Vec::new());
    w.apply(&Action::Feint(Dir::E))
}

/// Whether `id` suffers Surprised.
fn fooled(w: &World, id: ActorId) -> bool {
    let status = w.content().feint().expect("feinting").status;
    w.state.actors[&id].effects.has(status)
}

/// One feint at a kobold of every score `will`, on gameplay stream `seed`: whether it was fooled.
/// The foe does not act.
fn outcome(presence: u8, bluff: u16, will: u8, seed: u64) -> bool {
    let mut w = liar("thief", presence, Some(bluff));
    let kobold = kobold_beside(&mut w, will);
    w.state.actors.get_mut(&kobold).expect("there").energy = -100_000;
    w.state.rng = Rng::gameplay(seed);
    assert!(feint(&mut w).took_turn);
    let made = fooled(&w, kobold);
    let word = if made {
        "falls for it."
    } else {
        "is not fooled."
    };
    assert_eq!(last(&w), format!("The kobold {word}"));
    made
}

#[test]
fn bluff_is_feint_in_the_menu_aimed_in_a_direction() {
    let mut w = liar("thief", 14, Some(10));
    let content = w.content().clone();
    let rows = w.perception().abilities();
    let row = rows.iter().find(|a| a.ability == Ability::Feint);
    // Nobody beside to fool (#1327); a kobold there, it is usable.
    let none = crate::ability::Unavailable::NoTarget(crate::beside::Target::Fool);
    assert_eq!(row.map(|r| r.unavailable), Some(Some(none)));
    let mut kobolded = liar("thief", 14, Some(10));
    kobold_beside(&mut kobolded, 10);
    kobolded.perceive(&mut Vec::new());
    let usable = (kobolded.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Feint)
        .map(|r| r.unavailable);
    assert_eq!(usable, Some(None), "usable");
    let skill = content.skill_id("bluff").expect("bluff");
    assert!(
        !rows.iter().any(|a| a.ability == Ability::Skill(skill)),
        "no \"not usable yet\" row"
    );
    let it = Ability::Feint;
    assert_eq!(it.aim(&content), crate::ability::Aim::Direction);
    assert_eq!(it.source(), crate::ability::Source::Skill);
    assert_eq!(it.cost(), Some(ActionKind::Feint));
    assert_eq!(it.name(&content), "Feint");
    assert_eq!(
        it.action(&content, Some(Dir::N)),
        Some(Action::Feint(Dir::N))
    );
    assert_eq!(it.action(&content, None), None, "not without a direction");
    train(&mut w, "bluff", None);
    assert!(!(w.perception().abilities().iter()).any(|a| a.ability == Ability::Feint));
}

#[test]
fn refusals_take_no_time_and_no_draw() {
    let mut w = liar("thief", 30, Some(10));
    let at = beside(&w);
    let refused = |w: &mut World, why: &str| {
        let (time, rng, training) = (w.state.time, w.state.rng.clone(), w.state.training.clone());
        let out = feint(w);
        assert!(!out.took_turn, "{why}");
        assert_eq!(
            (w.state.time, &w.state.rng, &w.state.training),
            (time, &rng, &training),
            "{why}"
        );
        assert_eq!(last(w), why);
    };
    refused(&mut w, "You see no one there to feint at.");
    // The animated and the bones have no mind to fool.
    let bones = foe_at(&mut w, "skeleton_warrior", at, 3);
    assert!(!w.has_vitals(bones));
    refused(&mut w, "The skeleton warrior has no mind to fool.");
    w.state.actors.remove(&bones);
    let kobold = foe_at(&mut w, "kobold", at, 3);
    let peaceful = w.content().factions().id("peaceful").expect("peaceful");
    let them = w.state.actors.get_mut(&kobold).expect("there");
    them.faction_override = Some(peaceful);
    refused(&mut w, "The kobold is no foe of yours.");
    let them = w.state.actors.get_mut(&kobold).expect("there");
    them.faction_override = None;
    train(&mut w, "bluff", None);
    refused(&mut w, "You do not know how to feint.");
}

#[test]
fn the_check_is_presence_and_bluff_against_willpower() {
    let fooled_of = |presence, bluff, will| {
        (0..200)
            .filter(|&seed| outcome(presence, bluff, will, seed))
            .count()
    };
    // An average liar, an average foe: 2d12 + 0 against 10 + 0 + 0, 36 ways short of 144.
    let even = fooled_of(10, 1, 10);
    assert!((30..=95).contains(&even), "{even} of 200");
    // Bluff's points and Presence each make it harder to see through.
    let skilled = fooled_of(10, 100, 10);
    assert!(skilled > even + 30, "{skilled} against {even}");
    let bold = fooled_of(24, 1, 10);
    assert!(bold > even + 30, "{bold} against {even}");
    // A foe of iron will is seldom fooled.
    let stubborn = fooled_of(10, 1, 30);
    assert!(stubborn < even, "{stubborn} against {even}");
    // The ladder of the curve, at most 20.
    let content = builtin();
    let bluff = (content.skills().iter())
        .find(|s| s.id == "bluff")
        .expect("bluff");
    let benefit = (bluff.benefits.iter())
        .find(|b| b.effect == Effect::Feint)
        .expect("the feint benefit");
    assert_eq!(
        [0, 100, 200].map(|v| benefit.at(v)),
        [0, 8, 12],
        "the design's curve"
    );
}

#[test]
fn a_feint_takes_a_turn_whatever_comes_of_it_and_draws_two_dice() {
    let mut w = liar("thief", 10, Some(1));
    let kobold = kobold_beside(&mut w, 10);
    w.state.actors.get_mut(&kobold).expect("there").energy = -100_000;
    let mut waited = w.clone();
    waited.apply(&Action::Wait);
    let mut expected = w.state.rng.clone();
    assert!(feint(&mut w).took_turn);
    assert_eq!(w.state.time, waited.state.time, "a turn, as a wait");
    crate::combat::roll_natural(&mut expected);
    assert_eq!(w.state.rng, expected, "the check's two gameplay draws");
}

#[test]
fn the_next_blow_finds_it_flat_footed_though_it_acted_between() {
    // A foe at the player's pace, ready to act in the feint's own turn.
    let mut w = liar("thief", 30, Some(200));
    let player = w.state.player;
    train(&mut w, "sneak_attack", Some(100));
    let kobold = kobold_beside(&mut w, 3);
    w.state.rng = Rng::gameplay(1);
    w.perceive(&mut Vec::new());
    // Noticed: this is a fight, so nothing here is by the foe being unaware.
    w.state
        .actors
        .get_mut(&kobold)
        .expect("there")
        .mind
        .last_seen = Some(w.player_pos());
    assert!(!w.sneaking(player, kobold, false), "no dice before the lie");
    let (hp, energy) = (w.state.player().hp, w.state.actors[&kobold].energy);
    assert!(feint(&mut w).took_turn);
    assert_eq!(last_of_feint(&w), "The kobold falls for it.");
    let dear = w.state.actors[&kobold].energy < energy;
    assert!(w.state.player().hp < hp || dear, "it acted");
    assert!(fooled(&w, kobold), "its action did not ease it");
    assert!(w.off_guard(kobold), "flat-footed for the blow");
    assert!(w.sneaking(player, kobold, false), "Sneak attack's dice");
    // Its Defence as a blow finds it has no Dexterity or Dodge in it, as anyone's surprise.
    let target = w.target_of(kobold);
    assert_eq!(
        w.flat_footed(kobold, false, target.clone()),
        target.without([
            crate::combat::Category::Dexterity,
            crate::combat::Category::Dodge
        ]),
    );
    // The foe's next action is the one that ends it: one blow's worth.
    run_until_it_acts(&mut w, kobold);
    assert!(!fooled(&w, kobold), "its wits recovered");
    assert!(!w.sneaking(player, kobold, false), "no dice any more");
}

/// The line that said what came of the feint (the foe's own turn may have said more after it).
fn last_of_feint(w: &World) -> String {
    (w.messages().iter().map(|m| m.display()))
        .find(|m| m.ends_with("falls for it.") || m.ends_with("is not fooled."))
        .unwrap_or_default()
}

/// The player waits until `id` has recovered from its surprise.
fn run_until_it_acts(w: &mut World, id: ActorId) {
    for _ in 0..5 {
        if !fooled(w, id) {
            return;
        }
        w.apply(&Action::Wait);
    }
}

#[test]
fn a_feint_may_mark_bluff_on_the_training_stream_whatever_came_of_it() {
    let marks_of = |seed: u64| {
        let mut w = liar("thief", 3, Some(1));
        let kobold = kobold_beside(&mut w, 30);
        w.state.actors.get_mut(&kobold).expect("there").energy = -100_000;
        w.state.training = Rng::training(seed);
        let gameplay = w.state.rng.clone();
        assert!(feint(&mut w).took_turn);
        let skill = w.content().skill_id("bluff").expect("bluff");
        let marks = (w.state.player().skills.get(skill)).map_or(0, |t| t.marks);
        assert_ne!(w.state.training, Rng::training(seed), "a training draw");
        let mut expected = gameplay;
        crate::combat::roll_natural(&mut expected);
        assert_eq!(
            w.state.rng, expected,
            "marks draw nothing from the gameplay"
        );
        marks
    };
    let marks: u32 = (0..200).map(marks_of).sum();
    assert!((70..=130).contains(&marks), "half the time: {marks} of 200");
}

#[test]
fn a_feint_replays() {
    let mut w = liar("thief", 30, Some(100));
    kobold_beside(&mut w, 3);
    let mut twin = w.clone();
    w.perceive(&mut Vec::new());
    twin.perceive(&mut Vec::new());
    for _ in 0..3 {
        w.apply(&Action::Feint(Dir::E));
        twin.apply(&Action::Feint(Dir::E));
    }
    assert_eq!(w.state_hash(), twin.state_hash());
}

/// Openings (`mark.rs`; #1462): the player's pool, held at `value`.
fn hold_openings(w: &mut World, value: u16) {
    let id = w.content().resource_id("openings").expect("openings");
    w.player_mut().resources.set(id, value);
}

fn openings_held(w: &World) -> u16 {
    let id = w.content().resource_id("openings").expect("openings");
    (w.state.player().resources.get(id)).map_or(0, |p| p.value)
}

/// A thief holding `held` openings, kobolds east and west of them, their mark the one `mark`
/// of them (if any), feints east on the first gameplay stream from 0 whose feint is `made` (or
/// not). The foes do not act. The world after, and the kobold feinted at.
fn feint_openings(held: u16, mark: Option<Dir>, made: bool) -> (World, ActorId) {
    for seed in 0..200 {
        let mut w = liar("thief", 14, Some(50));
        hold_openings(&mut w, held);
        let kobold = kobold_beside(&mut w, 10);
        let west = w.player_pos().step(Dir::W);
        let other = foe_at(&mut w, "kobold", west, 10);
        for id in [kobold, other] {
            w.state.actors.get_mut(&id).expect("there").energy = -100_000;
        }
        w.player_mut().mark = match mark {
            Some(Dir::E) => Some(kobold),
            Some(_) => Some(other),
            None => None,
        };
        w.state.rng = Rng::gameplay(seed);
        assert!(feint(&mut w).took_turn);
        if fooled(&w, kobold) == made {
            return (w, kobold);
        }
    }
    panic!("no feint came out made = {made}");
}

/// How many times the player was told `text`.
fn told(w: &World, text: &str) -> usize {
    (w.messages().iter())
        .filter(|m| m.display().contains(text))
        .count()
}

#[test]
fn a_feint_made_builds_an_opening_in_the_foes_guard() {
    let (w, kobold) = feint_openings(0, None, true);
    assert_eq!(openings_held(&w), 1, "one opening");
    assert_eq!(w.state.player().mark, Some(kobold), "the foe is the mark");
    // At the mark itself the pool grows, nothing lost.
    let (w, kobold) = feint_openings(2, Some(Dir::E), true);
    assert_eq!(openings_held(&w), 3);
    assert_eq!(w.state.player().mark, Some(kobold));
    assert_eq!(told(&w, "are lost"), 0);
    // Full, it stays full.
    let (w, _) = feint_openings(5, Some(Dir::E), true);
    assert_eq!(openings_held(&w), 5);
}

#[test]
fn a_failed_feint_builds_nothing_and_still_moves_the_mark() {
    let (w, kobold) = feint_openings(1, None, false);
    assert_eq!(openings_held(&w), 1, "nothing built");
    assert_eq!(w.state.player().mark, Some(kobold), "the mark set");
    // Over an empty pool the mark set fades with the turn's time (`mark::faded`).
    let (w, _) = feint_openings(0, None, false);
    assert_eq!((openings_held(&w), w.state.player().mark), (0, None));
    // Marked elsewhere, the old mark's openings are lost (and the new mark over the emptied
    // pool fades with the turn).
    let (w, _) = feint_openings(2, Some(Dir::W), false);
    assert_eq!(openings_held(&w), 0, "the old mark's openings lost");
    assert_eq!(w.state.player().mark, None, "the old mark is gone");
    assert_eq!(told(&w, "Your openings on the kobold are lost."), 1);
}

#[test]
fn a_feint_at_a_new_foe_empties_the_pool_first_then_builds() {
    let (w, kobold) = feint_openings(3, Some(Dir::W), true);
    assert_eq!(openings_held(&w), 1, "emptied, then one");
    assert_eq!(w.state.player().mark, Some(kobold));
    assert_eq!(told(&w, "Your openings on the kobold are lost."), 1);
    let lines: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    let lost = lines.iter().position(|m| m.contains("are lost"));
    let fell = lines.iter().position(|m| m.ends_with("falls for it."));
    assert!(lost.is_some() && lost < fell, "lost before the check");
}

#[test]
fn a_refused_feint_moves_no_mark() {
    let mut w = liar("thief", 14, Some(50));
    hold_openings(&mut w, 2);
    let west = w.player_pos().step(Dir::W);
    let other = foe_at(&mut w, "kobold", west, 10);
    w.player_mut().mark = Some(other);
    let at = beside(&w);
    foe_at(&mut w, "skeleton_warrior", at, 3);
    assert!(!feint(&mut w).took_turn);
    assert_eq!((openings_held(&w), w.state.player().mark), (2, Some(other)));
}
