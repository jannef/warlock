//! War cry (spec §8 "Class abilities"; #844; `world/war_cry.rs`): the berserker's roar at 25,
//! for 4 rage and a wait; the hostiles that hear and see the berserker and have vitals check
//! their nerve against Strength and Intimidate, and the failed flee; never refused but for want
//! of rage; each cry a mark of Intimidate.

use super::class_active_tests::{last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::content::ClassActive;
use crate::noise::SoundKind;
use crate::skills::{Effect, Trained, Use};
use crate::stats::{PerStat, Stat, Stats};

/// A creature of type `creature` placed at `at`, every score `score`, awake and hostile; its id.
pub(super) fn foe_at(w: &mut World, creature: &str, at: Pos, score: u8) -> ActorId {
    let content = w.content().clone();
    // The level is emptied of its creatures (`crier`): a fresh showcase lends one to copy.
    let fresh = world("berserker");
    let template = (fresh.state.actors.values())
        .find(|a| a.id != fresh.state.player)
        .cloned()
        .expect("a creature in the showcase");
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

/// A raging berserker with War cry, alone on the level, Strength `strength` (the rest 14).
fn crier(strength: u8) -> World {
    let mut w = world("berserker");
    w.player_mut().talents.take("war_cry", None);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let mut scores = PerStat::splat(14);
    scores[Stat::Strength] = strength;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
    run(&mut w, "use:rage");
    w
}

/// The player's rage, if they hold it.
fn rage(w: &World) -> Option<u16> {
    let rage = w.content().resource_id("rage").expect("rage");
    w.state.player().resources.get(rage).map(|p| p.value)
}

fn own(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Class(ClassActive::WarCry))
        .map(|a| a.unavailable)
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

fn cry(w: &mut World) -> Outcome {
    w.apply(&Action::Use(ClassActive::WarCry, None))
}

/// Two cells east of the player.
fn near(w: &World) -> Pos {
    let me = w.player_pos();
    Pos::new(me.x + 2, me.y)
}

#[test]
fn war_cry_comes_at_25_and_only_the_rage_refuses_it() {
    let mut w = world("berserker");
    reach(&mut w, 24);
    assert_eq!(own(&w), None, "not before 25");
    reach(&mut w, 25);
    assert!(said(&w, "You gain War cry."), "{}", last(&w));
    let rage_id = w.content().resource_id("rage").expect("rage");
    assert_eq!(own(&w), Some(Some(Unavailable::Short(rage_id))));
    assert_eq!(
        Unavailable::Short(rage_id).reason(w.content()),
        "you have not the rage for it"
    );
    let time = w.state.time;
    let out = cry(&mut w);
    assert!(!out.took_turn);
    assert_eq!(w.state.time, time, "no time");
    assert_eq!(last(&w), "You have not the rage for it.");
    run(&mut w, "use:rage");
    assert_eq!(own(&w), Some(None), "raging, it may roar");
    let rows: Vec<_> = (w.perception().abilities().into_iter())
        .filter_map(|a| match a.ability {
            Ability::Class(active) => Some(active),
            _ => None,
        })
        .collect();
    assert_eq!(
        rows,
        [ClassActive::Rage, ClassActive::WarCry],
        "under Class"
    );
}

#[test]
fn a_roar_at_an_empty_room_spends_its_turn_and_four_rage() {
    let mut w = crier(14);
    let mut waited = w.clone();
    run(&mut waited, "wait");
    let out = cry(&mut w);
    assert!(out.took_turn);
    assert_eq!(w.state.time, waited.state.time, "the time of a wait");
    assert_eq!(
        rage(&w).map(|r| r + 4),
        rage(&waited),
        "four rage more than a wait"
    );
    assert_eq!(last(&w), "You roar your fury!");
}

#[test]
fn the_roar_is_a_howl_heard_far() {
    let mut w = crier(30);
    let player = w.state.player;
    // Out of its sight, a kobold hears a howl from where the player stands, and, unseeing, it
    // does not quail however mighty the roar.
    let me = w.player_pos();
    let level = w.state.current().clone();
    let heard_howl = |w: &World, id: ActorId| {
        (w.state.actors[&id].heard.iter()).any(|h| h.kind == SoundKind::Howl && h.at == me)
    };
    let far = (-30..30)
        .flat_map(|y| (-30..30).map(move |x| Pos::new(me.x + x, me.y + y)))
        .filter(|&p| p.chebyshev(me) > 1 && level.is_walkable(w.content(), p))
        .find(|&p| {
            let mut probing = w.clone();
            let probe = foe_at(&mut probing, "kobold", p, 3);
            let unseeing = !probing.sees_now(probe, player);
            let _ = probing.war_cry(&mut Vec::new());
            unseeing && heard_howl(&probing, probe)
        })
        .expect("a cell out of sight and within earshot");
    let kobold = foe_at(&mut w, "kobold", far, 3);
    let _ = w.war_cry(&mut Vec::new());
    assert!(heard_howl(&w, kobold));
    assert!(
        !suffers(&w, kobold, "frightened"),
        "it never saw the roarer"
    );
}

#[test]
fn the_faint_hearted_quail_and_the_steady_stand() {
    // A mighty berserker (St 30) against a kobold of Willpower 3: it quails, unless a natural
    // 24.
    let mut w = crier(30);
    let player = w.state.player;
    let at = near(&w);
    let kobold = foe_at(&mut w, "kobold", at, 3);
    w.perceive(&mut Vec::new());
    let out = cry(&mut w);
    assert!(out.took_turn);
    assert!(said(&w, "The kobold quails and flees from you."));
    let fear = w.content().status_id("frightened").expect("frightened");
    let effect = w.state.actors[&kobold].effects.get(fear).copied();
    assert_eq!(
        effect.and_then(|e| e.by),
        Some(player),
        "caused by the player"
    );
    assert!(
        w.state.actors[&kobold].pos.chebyshev(w.player_pos()) > 2,
        "it ran"
    );
    // A feeble one (St 3) against a kobold of Willpower 30: it stands, unless a natural 2.
    let mut w = crier(3);
    let at = near(&w);
    let kobold = foe_at(&mut w, "kobold", at, 30);
    w.perceive(&mut Vec::new());
    cry(&mut w);
    assert!(said(&w, "The kobold does not flinch."));
    assert!(!suffers(&w, kobold, "frightened"));
}

#[test]
fn no_level_cut_and_two_draws_a_hearer() {
    // A troll far above a level-1 berserker still checks, and a mighty roar sends it running.
    let mut w = crier(30);
    let at = near(&w);
    let troll = foe_at(&mut w, "troll", at, 3);
    assert!(w.content().creature(w.state.actors[&troll].creature).level > w.player_level());
    w.perceive(&mut Vec::new());
    let mut rng = w.state.rng.clone();
    let mut events = Vec::new();
    let _ = w.war_cry(&mut events);
    assert!(suffers(&w, troll, "frightened"));
    crate::combat::roll_natural(&mut rng);
    assert_eq!(w.state.rng, rng, "two gameplay draws for the one hearer");
}

#[test]
fn the_fearless_and_the_peaceful_draw_nothing() {
    // A skeleton has no heart to quail; a peaceful creature is not a foe.
    let mut w = crier(30);
    let at = near(&w);
    let bones = foe_at(&mut w, "skeleton_warrior", at, 3);
    let other = Pos::new(at.x, at.y + 1);
    let friend = foe_at(&mut w, "kobold", other, 3);
    let peaceful = w.content().factions().id("peaceful").expect("peaceful");
    w.state
        .actors
        .get_mut(&friend)
        .expect("there")
        .faction_override = Some(peaceful);
    assert_ne!(w.stance(w.state.player, friend), Stance::Hostile);
    w.perceive(&mut Vec::new());
    let rng = w.state.rng.clone();
    let mut events = Vec::new();
    let _ = w.war_cry(&mut events);
    assert_eq!(w.state.rng, rng, "no draw");
    assert!(!suffers(&w, bones, "frightened"));
}

#[test]
fn intimidate_hardens_the_check_and_each_cry_marks_it() {
    let mut w = crier(14);
    let content = w.content().clone();
    let intimidate = content.skill_id("intimidate").expect("intimidate");
    let value = |w: &mut World, v: u16| {
        let skills = &mut w.player_mut().skills;
        let mut all: Vec<_> = skills.iter().filter(|(s, _)| *s != intimidate).collect();
        all.push((intimidate, Trained { value: v, marks: 0 }));
        *skills = all.into_iter().collect();
    };
    value(&mut w, 100);
    assert_eq!(
        w.state.player().skill_effect(&content, Effect::Intimidate),
        8
    );
    value(&mut w, 200);
    assert_eq!(
        w.state.player().skill_effect(&content, Effect::Intimidate),
        12
    );
    assert!(w.trains(Use::Intimidate));
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(intimidate)
            .map_or(0, |t| t.marks)
    };
    let before = marks(&w);
    cry(&mut w);
    assert_eq!(marks(&w), before + 1, "a mark a cry, nobody there");
}
