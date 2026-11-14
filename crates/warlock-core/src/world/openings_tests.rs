//! The thief's Openings (spec §8 "Resources" and "Class abilities"; #251, #1463, #1464): Sneak
//! attack (level 1) grants the pool, a sneak attack that hits builds it, it slips away with
//! time, Exploit (level 1) spends every one held on a sneak attack whatever the foe knows, its
//! dice by the openings spent (2d6 each from Improved sneak attack, 32), Low blow's stun and
//! to-hit added, and Master thief's threat by the openings and its sure critical from five (60).

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{Event, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::{ClassActive, Factor, ResourceId};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::stats::{PerStat, Stats};
use crate::test_support::CONTENT as BUILTIN;
use crate::test_support::last;
use crate::test_support::run;

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
    // The picks may have taken Patient blade: these tests are of the openings' own pace.
    world.player_mut().talents.forget("patient_blade");
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
        summoner: None,
        ally: None,
        risen: None,
        mark: None,
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
fn sneak_attack_grants_openings_at_zero_from_level_one() {
    let mut w = thief();
    assert_eq!(held(&w), Some(0), "held from level 1, empty");
    reach(&mut w, 32);
    assert_eq!(
        held(&w),
        Some(0),
        "Improved sneak attack grants nothing more"
    );
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
    w.ambushed((player, player), false, hit);
    assert_eq!(held(&w), Some(0), "a seen blow builds nothing");
    let missed = Blow::missed(None);
    w.ambushed((player, player), true, missed);
    assert_eq!(held(&w), Some(0), "a miss builds nothing");
    for n in 1..=7u16 {
        w.ambushed((player, player), true, hit);
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

/// The player's events of using Exploit `dir`, without the turn's end (no creature acts after).
fn exploit_now(w: &mut World, dir: Dir) -> (Spent, Vec<Event>) {
    let mut events = Vec::new();
    w.perceive(&mut events);
    let spent = w.use_class(ClassActive::Exploit, Some(dir), &mut events);
    (spent, events)
}

/// Whether `events` hold a hit of the player's on `target`.
fn hit_on(w: &World, events: &[Event], target: ActorId) -> bool {
    let player = w.state.player;
    (events.iter()).any(|e| {
        matches!(e, Event::Attacked { attacker, target: t, strike: Some(s), .. }
            if (*attacker, *t) == (player, target) && s.is_hit())
    })
}

fn exploit_row(w: &World) -> String {
    (dump::layer_rows(w, Layer::Abilities).into_iter())
        .find(|r| r.starts_with("Exploit"))
        .expect("in the abilities menu")
}

#[test]
fn exploit_comes_at_level_one_and_its_refusals_take_no_time_and_spend_nothing() {
    let mut w = thief();
    let exploit = Ability::Class(ClassActive::Exploit);
    let id = openings(&w);
    assert!(
        exploit.had_by(w.state.player(), w.content()),
        "from level 1"
    );
    assert_eq!(
        exploit.unavailable(w.state.player(), w.content()),
        Some(Unavailable::Short(id))
    );
    assert_eq!(exploit_row(&w), "Exploit (class): you have no opening");
    let turn = w.state.turn;
    let outcome = w.apply(&"use:exploit:E".parse().expect("action"));
    assert!(!outcome.took_turn);
    assert_eq!(last(&w), "You have no opening.");
    set(&mut w, 3);
    assert_eq!(exploit_row(&w), "Exploit (class): no foe within reach");
    // No direction, nobody there.
    let outcome = w.apply(&Action::Use(ClassActive::Exploit, None));
    assert!(!outcome.took_turn);
    assert_eq!(last(&w), "Which way?");
    let outcome = w.apply(&"use:exploit:E".parse().expect("action"));
    assert!(!outcome.took_turn);
    assert_eq!(last(&w), "There is no one there.");
    // A skeleton the player sees has no vitals: refused.
    let skeleton = beside(&mut w, "skeleton", 5000, Dir::E);
    let (spent, _) = exploit_now(&mut w, Dir::E);
    assert_eq!(exploit_row(&w), "Exploit (class): a direction");
    assert!(matches!(spent, Spent::Nothing));
    assert_eq!(last(&w), "It has no vital spot.");
    assert_eq!(
        (w.state.turn, held(&w)),
        (turn, Some(3)),
        "no time, nothing spent"
    );
    assert!(w.state.actors.contains_key(&skeleton));
}

#[test]
fn exploit_spends_every_opening_on_a_sneak_attack_whatever_the_foe_knows() {
    let mut w = thief();
    let content = w.content().clone();
    let kobold = beside(&mut w, "kobold", 5000, Dir::E);
    let surprised = content.status_id("surprised").expect("surprised");
    let mut hit = false;
    for n in (1..=5u16).cycle().take(40) {
        set(&mut w, n);
        let before = w.state.actors[&kobold].hp;
        let (spent, events) = exploit_now(&mut w, Dir::E);
        assert!(matches!(spent, Spent::Turn(_)), "used");
        assert_eq!(held(&w), Some(0), "every one spent, none built back");
        assert_eq!(w.state.player().mark, None, "the mark goes with the pool");
        assert!(!w.state.actors[&kobold].effects.has(surprised));
        let words = match n {
            1 => "You strike at the opening.",
            2..=4 => "You press your openings.",
            _ => "You tear into every opening!",
        };
        assert!(
            (w.messages().iter()).any(|m| m.display().contains(words)),
            "{words}"
        );
        if hit_on(&w, &events, kobold) {
            // The plain blow, the sneak dice, and n of the ladder's 1d6.
            let done = before - w.state.actors[&kobold].hp;
            assert!(done >= i32::from(n), "{done} for {n}");
            hit = true;
        }
    }
    assert!(hit, "one of forty exploits hit");
    assert_eq!(w.technique, super::weapon_active::Technique::default());
}

#[test]
fn the_ladders_dice_are_one_d6_an_opening_on_a_hit_only() {
    let mut w = thief();
    let hit = Blow {
        strike: Strike::Hit { critical: false },
        damage: 2,
        kind: None,
    };
    assert_eq!(w.land_exploit(0, hit), hit, "no openings, no dice");
    let missed = Blow::missed(None);
    assert_eq!(w.land_exploit(5, missed), missed, "a miss, no dice");
    for n in 1..=5u16 {
        for _ in 0..20 {
            let extra = w.land_exploit(n, hit).damage - 2;
            let n = i32::from(n);
            assert!((n..=6 * n).contains(&extra), "{extra} for {n}");
        }
    }
    // A critical never multiplies them: they are added to the blow's damage as it stands.
    let critical = Blow {
        strike: Strike::Hit { critical: true },
        ..hit
    };
    let extra = w.land_exploit(1, critical).damage - 2;
    assert!((1..=6).contains(&extra));
}

#[test]
fn low_blow_stuns_the_foe_and_gives_one_to_hit_an_opening() {
    let mut w = thief();
    let content = w.content().clone();
    let stunned = content.status_id("stunned").expect("stunned");
    let ladder = w.state.player().talent_mods(&content).stealth.exploit;
    assert_eq!((ladder.to_hit, ladder.status), (0, None), "not before 12");
    reach(&mut w, 12);
    let ladder = w.state.player().talent_mods(&content).stealth.exploit;
    assert_eq!((ladder.to_hit, ladder.status), (1, Some(stunned)));
    let kobold = beside(&mut w, "kobold", 5000, Dir::E);
    for _ in 0..40 {
        set(&mut w, 2);
        w.state.actors.get_mut(&kobold).expect("kobold").effects = Default::default();
        let (_, events) = exploit_now(&mut w, Dir::E);
        if hit_on(&w, &events, kobold) {
            assert!(
                w.state.actors[&kobold].effects.has(stunned),
                "stunned by the hit"
            );
            return;
        }
        assert!(
            !w.state.actors[&kobold].effects.has(stunned),
            "a miss stuns nothing"
        );
    }
    panic!("no exploit hit the kobold");
}

#[test]
fn a_killing_exploit_leaves_one_opening_and_no_mark() {
    let mut w = thief();
    let kobold = beside(&mut w, "kobold", 1, Dir::E);
    for _ in 0..40 {
        set(&mut w, 4);
        exploit_now(&mut w, Dir::E);
        if !w.state.actors.contains_key(&kobold) {
            assert_eq!(held(&w), Some(1), "the mark's kill gives one back");
            assert_eq!(w.state.player().mark, None);
            return;
        }
    }
    panic!("no exploit killed the kobold");
}

#[test]
fn exploit_replays() {
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
    let text = "use:exploit:E resource:openings use:exploit:E wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    assert_eq!(
        Action::Use(ClassActive::Exploit, Some(Dir::E)).to_string(),
        "use:exploit:E"
    );
    assert!(
        ClassActive::from_id("low_blow").is_none(),
        "no longer a class active"
    );
    // Master thief's finisher is folded into Exploit (#1464): no active, and its action text is
    // no class active's (an unknown `use:ID` is read as a spell's).
    assert!(ClassActive::from_id("master_thief").is_none());
    let finisher = "use:master_thief:E".parse::<Action>();
    assert!(!matches!(finisher, Ok(Action::Use(..))), "{finisher:?}");
}

/// A hit of `n` openings' exploit with nothing else on it.
const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 2,
    kind: None,
};

#[test]
fn improved_sneak_attack_makes_the_ladders_dice_two_d6_an_opening_at_32() {
    let mut w = thief();
    reach(&mut w, 31);
    let one = w
        .state
        .player()
        .talent_mods(w.content())
        .stealth
        .exploit
        .dice;
    assert_eq!(one.map(|d| d.to_string()), Some("1d6".into()), "before 32");
    reach(&mut w, 32);
    let two = w
        .state
        .player()
        .talent_mods(w.content())
        .stealth
        .exploit
        .dice;
    assert_eq!(two.map(|d| d.to_string()), Some("2d6".into()), "from 32");
    for n in 1..=5u16 {
        for _ in 0..20 {
            let extra = w.land_exploit(n, HIT).damage - 2;
            let n = i32::from(n);
            assert!((2 * n..=12 * n).contains(&extra), "{extra} for {n}");
        }
    }
}

#[test]
fn master_thief_lowers_an_exploits_threat_four_an_opening_and_is_sure_from_five() {
    let mut w = thief();
    reach(&mut w, 59);
    let ladder = w.state.player().talent_mods(w.content()).stealth.exploit;
    assert_eq!((ladder.threat, ladder.sure_at), (0, None), "not before 60");
    let melee = w.state.player().melee(w.content());
    assert_eq!(w.exploit_profile(3, melee.clone()).threat, melee.threat);
    reach(&mut w, 60);
    let ladder = w.state.player().talent_mods(w.content()).stealth.exploit;
    assert_eq!((ladder.threat, ladder.sure_at), (4, Some(5)));
    let melee = w.state.player().melee(w.content());
    for n in 0..=8u16 {
        let profile = w.exploit_profile(n, melee.clone());
        let lower = 4 * i32::from(n);
        let threat = (i32::from(melee.threat) - lower).max(2);
        assert_eq!(i32::from(profile.threat), threat, "threat at {n}");
        assert_eq!(profile.sure_critical, n >= 5, "sure at {n}");
        // Low blow's to-hit rides along.
        assert_eq!(profile.to_hit, melee.to_hit + i32::from(n), "to-hit at {n}");
    }
    // The sneak attack's own lowering never raises what the ladder brought to 2.
    let player = w.state.player;
    let low = w.exploit_profile(5, melee);
    assert_eq!(w.sneak_threat(player, true, low).threat, 2);
}

#[test]
fn every_hit_of_an_exploit_of_five_at_sixty_is_a_critical() {
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
        let (spent, events) = exploit_now(&mut w, Dir::E);
        assert!(matches!(spent, Spent::Turn(_)), "used");
        assert_eq!(held(&w), Some(0), "five openings spent, hit or miss");
        let first = (events.iter()).find_map(|e| match e {
            Event::Attacked {
                attacker, strike, ..
            } if *attacker == player => *strike,
            _ => None,
        });
        if let Some(Strike::Hit { critical }) = first {
            assert!(critical, "a hit of an exploit of five is a critical");
            hits += 1;
        }
        assert!(w.state.actors.contains_key(&target), "it stands");
    }
    assert_eq!(hits, 5, "five hits in {tries} tries");
    // The abilities menu has Exploit and no finisher.
    let rows = dump::layer_rows(&w, Layer::Abilities);
    assert!(rows.iter().any(|r| r.starts_with("Exploit")));
    assert!(
        !rows.iter().any(|r| r.starts_with("Master thief")),
        "{rows:?}"
    );
}

#[test]
fn a_pool_past_five_is_spent_whole_with_no_cap_on_the_ladder() {
    // Of Guile raises the pool's maximum; the ladder counts every opening spent.
    let mut w = thief();
    reach(&mut w, 60);
    let kobold = beside(&mut w, "kobold", 10_000, Dir::E);
    set(&mut w, 8);
    let (spent, _) = exploit_now(&mut w, Dir::E);
    assert!(matches!(spent, Spent::Turn(_)));
    assert_eq!(held(&w), Some(0), "all eight spent");
    assert!(w.state.actors.contains_key(&kobold));
    for _ in 0..20 {
        let extra = w.land_exploit(8, HIT).damage - 2;
        assert!((16..=96).contains(&extra), "8 × 2d6: {extra}");
    }
}
