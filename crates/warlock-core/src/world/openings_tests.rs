//! The thief's Openings (spec §8 "Resources" and "Class abilities"; #251): Improved sneak attack
//! grants the pool, a sneak attack that hits builds it, it slips away with time, and Master
//! thief's finisher spends five for a blow that is a critical whenever it hits.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{Event, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::{ClassActive, Factor, ResourceId};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human thief in the showcase (seed 7), with their kit, born under no sign, every score 14.
fn thief() -> World {
    let spec: CharacterSpec = "human,male,thief".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "healthy".into()]);
    let w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn last(world: &World) -> String {
    (world.messages().iter().next_back()).map_or_else(String::new, |m| m.display())
}

/// Grants the XP that takes the player to `level` and spends every choice it brings.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
    if world.due().is_some() {
        run(world, "levelup:auto");
    }
    assert_eq!(world.due(), None, "every choice spent");
}

fn openings(world: &World) -> ResourceId {
    world.content().resource_id("openings").expect("openings")
}

fn held(world: &World) -> Option<u16> {
    let id = openings(world);
    world.state.player().resources.get(id).map(|p| p.value)
}

fn set(world: &mut World, value: u16) {
    let id = openings(world);
    world.player_mut().resources.set(id, value);
}

/// A creature of type `creature` placed `dir` of the player, with `hp` hit points, hostile; its
/// id.
fn beside(world: &mut World, creature: &str, hp: i32, dir: Dir) -> ActorId {
    let content = world.content().clone();
    let at = world.player_pos().step(dir);
    let template = (world.state.actors.values())
        .find(|a| a.id != world.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(world.state.next_actor_id);
    world.state.next_actor_id += 1;
    let creature = content.creature_id(creature).expect("a creature type");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: hp,
        hp,
        level: world.state.current_level.clone(),
        ..template
    };
    world.state.actors.insert(id, actor);
    id
}

#[test]
fn improved_sneak_attack_grants_openings_at_zero() {
    let mut w = thief();
    reach(&mut w, 31);
    assert_eq!(held(&w), None, "no openings before level 32");
    reach(&mut w, 32);
    assert_eq!(held(&w), Some(0), "held from level 32, empty");
    let header = dump::ascii(&w, &[]);
    let header = header.lines().next().unwrap_or_default();
    assert!(header.contains("openings 0/5"), "{header}");
    // A saved thief from before the pool (the talent held, no openings) gets it on load, and a
    // pool already held is never refilled.
    let id = openings(&w);
    let mut state = w.state().clone();
    state
        .actors
        .get_mut(&state.player)
        .expect("player")
        .resources
        .remove(id);
    let restored = World::from_state(BUILTIN.clone(), state).expect("restores");
    assert_eq!(held(&restored), Some(0));
    set(&mut w, 3);
    reach(&mut w, 33);
    assert_eq!(held(&w), Some(3), "a level does not refill it");
}

#[test]
fn a_sneak_attack_that_hits_builds_an_opening_up_to_five() {
    let mut w = thief();
    reach(&mut w, 32);
    let player = w.state.player;
    let hit = Blow {
        strike: Strike::Hit { critical: false },
        damage: 2,
        kind: None,
    };
    w.ambushed(player, false, hit);
    assert_eq!(held(&w), Some(0), "a seen blow builds nothing");
    let missed = Blow::missed(None);
    w.ambushed(player, true, missed);
    assert_eq!(held(&w), Some(0), "a miss builds nothing");
    for n in 1..=7u16 {
        w.ambushed(player, true, hit);
        assert_eq!(held(&w), Some(n.min(5)));
    }
}

#[test]
fn openings_slip_away_a_point_every_twenty_base_turns_and_stay_at_zero() {
    let mut w = thief();
    reach(&mut w, 32);
    set(&mut w, 2);
    run(&mut w, "wait*19");
    assert_eq!(held(&w), Some(2), "not yet");
    run(&mut w, "wait*6");
    assert_eq!(held(&w), Some(1), "a point gone after twenty base turns");
    run(&mut w, "wait*30");
    assert_eq!(held(&w), Some(0), "never ends: it sits at 0");
}

#[test]
fn the_finisher_needs_five_openings_and_a_hostile() {
    let mut w = thief();
    let finisher = Ability::Class(ClassActive::MasterThief);
    reach(&mut w, 59);
    assert!(
        !finisher.had_by(w.state.player(), w.content()),
        "not before 60"
    );
    reach(&mut w, 60);
    let id = openings(&w);
    let me = w.state.player();
    assert!(finisher.had_by(me, w.content()));
    assert_eq!(
        finisher.unavailable(me, w.content()),
        Some(Unavailable::Short(id))
    );
    let row = (dump::layer_rows(&w, Layer::Abilities).into_iter())
        .find(|r| r.starts_with("Master thief"))
        .expect("in the abilities menu");
    assert_eq!(row, "Master thief (class): not enough openings");
    let outcome = w.apply(&"use:master_thief:E".parse().expect("action"));
    assert!(!outcome.took_turn);
    assert_eq!(last(&w), "You cannot: not enough openings.");
    // With five, but nobody there: refused, and nothing spent.
    set(&mut w, 5);
    assert_eq!(finisher.unavailable(w.state.player(), w.content()), None);
    let outcome = w.apply(&"use:master_thief:E".parse().expect("action"));
    assert!(!outcome.took_turn);
    assert_eq!(held(&w), Some(5), "nothing spent on a refusal");
}

#[test]
fn every_hit_of_the_finisher_is_a_critical() {
    let mut w = thief();
    reach(&mut w, 60);
    let player = w.state.player;
    let target = beside(&mut w, "kobold", 10_000, Dir::E);
    let (mut hits, mut tries) = (0, 0);
    while hits < 5 && tries < 60 {
        tries += 1;
        set(&mut w, 5);
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        let outcome = w.apply(&"use:master_thief:E".parse().expect("action"));
        assert!(outcome.took_turn, "used");
        assert_eq!(held(&w), Some(0), "five openings spent, hit or miss");
        // The finisher's own blow is the player's first attack of the turn.
        let first = (outcome.events.iter()).find_map(|e| match e {
            Event::Attacked {
                attacker, strike, ..
            } if *attacker == player => *strike,
            _ => None,
        });
        if let Some(Strike::Hit { critical }) = first {
            assert!(critical, "a hit of the finisher is a critical");
            hits += 1;
        }
        assert!(w.state.actors.contains_key(&target), "it stands");
    }
    assert_eq!(hits, 5, "five hits in {tries} tries");
}

#[test]
fn the_finisher_replays() {
    let spec: CharacterSpec = "human,male,thief".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let text = "xp:3000000 levelup:auto use:master_thief:E wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(held(&w), Some(0), "granted on the way to 60");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    assert_eq!(
        Action::Use(ClassActive::MasterThief, Some(Dir::E)).to_string(),
        "use:master_thief:E"
    );
}
