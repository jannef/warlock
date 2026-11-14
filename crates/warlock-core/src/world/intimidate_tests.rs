//! Intimidate (spec §8 "Skills": Intimidate; #868; `world/intimidate.rs`): the skill's use at one
//! foe, on War cry's check; its degrees by the ladder of `data/intimidate.toml`; its refusals,
//! with no time and no draw; a turn and a voice; each threat a mark of Intimidate.

use super::class_active_tests::{suffers, world};
use super::*;
use crate::ability::Ability;
use crate::noise::SoundKind;
use crate::skills::{Effect, Trained};
use crate::stats::{PerStat, Stat, Stats};
use crate::test_support::{last, run};

/// A creature of type `creature` placed at `at`, every score `score`, awake and hostile; its id.
pub(super) fn foe_at(w: &mut World, creature: &str, at: Pos, score: u8) -> ActorId {
    let content = w.content().clone();
    let template = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .cloned()
        .unwrap_or_else(|| {
            let fresh = world("warrior");
            (fresh.state.actors.values())
                .find(|a| a.id != fresh.state.player)
                .cloned()
                .expect("a creature in the showcase")
        });
    let id = ActorId(w.state.next_actor_id);
    w.state.next_actor_id += 1;
    let creature = content.creature_id(creature).expect("a creature type");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        summoner: None,
        ally: None,
        risen: None,
        mark: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: Stats::new(PerStat::splat(score), PerStat::splat(score)),
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        heard: Vec::new(),
        hp_base: 50,
        hp: 50,
        level: w.state.current_level.clone(),
        ..template
    };
    w.state.actors.insert(id, actor);
    id
}

/// The player's Intimidate set to `value` (trained), the rest of their skills as they are.
fn intimidate_at(w: &mut World, value: Option<u16>) {
    let intimidate = w.content().skill_id("intimidate").expect("intimidate");
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<_> = skills.iter().filter(|(s, _)| *s != intimidate).collect();
    if let Some(value) = value {
        all.push((intimidate, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

/// A warrior alone on the level, Strength `strength` (the rest 14), Intimidate `value`.
fn bully(strength: u8, value: Option<u16>) -> World {
    let mut w = world("warrior");
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let mut scores = PerStat::splat(14);
    scores[Stat::Strength] = strength;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
    intimidate_at(&mut w, value);
    w
}

/// Two cells east of the player.
fn near(w: &World) -> Pos {
    let me = w.player_pos();
    Pos::new(me.x + 2, me.y)
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

fn threaten(w: &mut World, at: Pos) -> Outcome {
    w.perceive(&mut Vec::new());
    w.apply(&Action::Intimidate(at))
}

fn fear(w: &World, who: ActorId) -> Option<u16> {
    let fear = w.content().status_id("frightened").expect("frightened");
    w.state.actors[&who].effects.get(fear).map(|e| e.severity)
}

#[test]
fn the_trained_have_it_under_skills_aimed_at_a_place() {
    let mut w = bully(14, Some(10));
    let content = w.content().clone();
    let row = (w.perception().abilities().into_iter()).find(|a| a.ability == Ability::Intimidate);
    assert_eq!(
        row.map(|r| r.unavailable),
        Some(None),
        "usable, not \"not yet\""
    );
    let skill = content.skill_id("intimidate").expect("intimidate");
    assert!(
        !(w.perception().abilities().iter()).any(|a| a.ability == Ability::Skill(skill)),
        "no \"not usable yet\" row"
    );
    let it = Ability::Intimidate;
    assert_eq!(it.aim(&content), crate::ability::Aim::Cell);
    assert_eq!(it.source(), crate::ability::Source::Skill);
    assert_eq!(it.cost(), Some(ActionKind::Intimidate));
    assert_eq!(it.reach(&content), Some(6));
    let at = Pos::new(3, 4);
    assert_eq!(it.action_at(&content, at), Some(Action::Intimidate(at)));
    assert_eq!(it.action(&content, None), None);
    intimidate_at(&mut w, None);
    assert!(!(w.perception().abilities().iter()).any(|a| a.ability == Ability::Intimidate));
}

#[test]
fn refusals_take_no_time_and_no_draw() {
    let mut w = bully(30, Some(10));
    let me = w.player_pos();
    let at = near(&w);
    let refused = |w: &mut World, at: Pos, why: &str| {
        let (time, rng) = (w.state.time, w.state.rng.clone());
        let out = threaten(w, at);
        assert!(!out.took_turn, "{why}");
        assert_eq!((w.state.time, &w.state.rng), (time, &rng), "{why}");
        assert_eq!(last(w), why);
    };
    refused(&mut w, at, "You see no one there to intimidate.");
    refused(&mut w, me, "You cannot intimidate yourself.");
    refused(&mut w, Pos::new(me.x + 7, me.y), "That is out of range.");
    let bones = foe_at(&mut w, "skeleton_warrior", at, 3);
    refused(&mut w, at, "The skeleton warrior has no heart to quail.");
    w.state.actors.remove(&bones);
    let kobold = foe_at(&mut w, "kobold", at, 3);
    let peaceful = w.content().factions().id("peaceful").expect("peaceful");
    w.state
        .actors
        .get_mut(&kobold)
        .expect("there")
        .faction_override = Some(peaceful);
    refused(&mut w, at, "The kobold is no foe of yours.");
    w.state
        .actors
        .get_mut(&kobold)
        .expect("there")
        .faction_override = None;
    intimidate_at(&mut w, None);
    refused(&mut w, at, "You do not know how to intimidate.");
}

#[test]
fn a_threat_takes_a_turn_speaks_aloud_and_marks_the_skill() {
    let mut w = bully(30, Some(10));
    let at = near(&w);
    foe_at(&mut w, "kobold", at, 3);
    let mut waited = w.clone();
    run(&mut waited, "wait");
    let intimidate = w.content().skill_id("intimidate").expect("intimidate");
    let marks = |w: &World| (w.state.player().skills.get(intimidate)).map_or(0, |t| t.marks);
    let before = marks(&w);
    let me = w.player_pos();
    let out = threaten(&mut w, at);
    assert!(out.took_turn);
    assert_eq!(w.state.time, waited.state.time, "a turn, as a wait");
    assert!(said(&w, "You threaten the kobold."));
    assert_eq!(marks(&w), before + 1, "a mark a threat");
    // The kobold heard it as it was spoken (what it heard is forgotten as its turn is done).
    let mut w = bully(30, Some(10));
    let kobold = foe_at(&mut w, "kobold", at, 3);
    w.perceive(&mut Vec::new());
    let _ = w.intimidate(at, &mut Vec::new());
    let heard =
        (w.state.actors[&kobold].heard.iter()).any(|h| h.kind == SoundKind::Voice && h.at == me);
    assert!(heard, "a voice where the player stands");
}

#[test]
fn the_faint_hearted_quail_and_the_steady_stand() {
    // St 30 against a kobold of Willpower 3: it quails, unless a natural 24.
    let mut w = bully(30, Some(10));
    let player = w.state.player;
    let at = near(&w);
    let kobold = foe_at(&mut w, "kobold", at, 3);
    w.perceive(&mut Vec::new());
    let mut rng = w.state.rng.clone();
    let spent = w.intimidate(at, &mut Vec::new());
    assert!(matches!(spent, Spent::Turn(_)));
    crate::combat::roll_natural(&mut rng);
    assert_eq!(w.state.rng, rng, "two gameplay draws");
    assert!(said(&w, "The kobold quails and flees from you."));
    let fear_id = w.content().status_id("frightened").expect("frightened");
    let effect = w.state.actors[&kobold].effects.get(fear_id).copied();
    assert_eq!(
        effect.and_then(|e| e.by),
        Some(player),
        "caused by the player"
    );
    // St 3 against Willpower 30: it stands, unless a natural 2.
    let mut w = bully(3, Some(10));
    let at = near(&w);
    let kobold = foe_at(&mut w, "kobold", at, 30);
    threaten(&mut w, at);
    assert!(said(&w, "The kobold does not flinch."));
    assert!(!suffers(&w, kobold, "frightened"));
}

#[test]
fn the_degrees_rise_with_the_skill_up_to_war_crys_three() {
    // What Intimidate adds at each value (skills.toml: +8 at 100), and the degrees the ladder
    // gives: one for a little skill, War cry's three at the top.
    let mut seen = Vec::new();
    for value in [10, 40, 80, 200] {
        let mut w = bully(30, Some(value));
        let at = near(&w);
        let kobold = foe_at(&mut w, "kobold", at, 3);
        let points = (w.state.player()).skill_effect(w.content(), Effect::Intimidate);
        w.perceive(&mut Vec::new());
        let _ = w.intimidate(at, &mut Vec::new());
        seen.push((value, points, fear(&w, kobold)));
    }
    assert_eq!(
        seen,
        [
            (10, 0, Some(1)),
            (40, 3, Some(2)),
            (80, 6, Some(3)),
            (200, 12, Some(3))
        ]
    );
}

#[test]
fn a_threat_replays_alike() {
    let mut w = bully(30, Some(40));
    let at = near(&w);
    foe_at(&mut w, "kobold", at, 3);
    let mut again = w.clone();
    threaten(&mut w, at);
    threaten(&mut again, at);
    assert_eq!(w.state_hash(), again.state_hash());
    assert_eq!(
        Action::Intimidate(at).to_string(),
        format!("intimidate:{},{}", at.x, at.y)
    );
}
