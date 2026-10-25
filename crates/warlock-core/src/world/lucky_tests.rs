//! The lucky days' gifts (spec §8 "Birth sign"; #269): the Phoenix rises once from the ashes,
//! the Firebird rises whole; the Iron Scourge cleaves and the Gate Warden strikes those who
//! leave, soaks more, and is less bowed by afflictions; the Deep One's melee hits hold a
//! creature fast until they miss it, strike another, or move. A character born on another day
//! has none of it.

use super::fight::Harm;
use super::stars::RISEN;
use super::*;
use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::{Sign, StatusId};
use crate::geom::{Dir, Pos};
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in the showcase (seed 7) born under the `element` `animal` on day `day`, every
/// score 14 and at full hit points; the run restored, so the gifts are granted as a load does.
fn born(element: &str, animal: &str, day: u32) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: animal.into(),
        element: element.into(),
    });
    character.born.day = day;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

fn suffers(w: &World, who: ActorId, id: &str) -> bool {
    w.state.actors[&who].effects.has(status(w, id))
}

fn last(w: &World) -> String {
    (w.messages().iter().map(|m| m.display()).next_back()).unwrap_or_default()
}

/// A free floor cell beside the player in direction `dir` or the next ones clockwise.
fn free_beside(w: &World, dir: Dir) -> Pos {
    let level = w.state.current();
    let me = w.player_pos();
    (0..8)
        .map(|i| Dir::ALL[(Dir::ALL.iter().position(|d| *d == dir).unwrap() + i) % 8])
        .map(|d| me.step(d))
        .find(|&p| {
            level
                .terrain
                .get(p)
                .is_some_and(|t| w.content().terrain(*t).walkable)
                && w.state.actor_at(&level.id, p).is_none()
        })
        .expect("a free cell beside the player")
}

/// A kobold (hostile to the player) placed at `at` on the player's level; its id.
fn foe_at(w: &mut World, at: Pos) -> ActorId {
    let content = w.content().clone();
    let template = (w.state.actors.values())
        .find(|a| a.id != w.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(w.state.next_actor_id);
    w.state.next_actor_id += 1;
    let creature = content.creature_id("kobold").expect("kobolds");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        effects: Default::default(),
        stats: def.stats,
        hp_base: 10,
        hp: 10,
        level: w.state.current_level.clone(),
        ..template
    };
    w.state.actors.insert(id, actor);
    id
}

/// A kobold placed on a free cell beside the player, `dir` or the next way clockwise.
fn foe_beside(w: &mut World, dir: Dir) -> ActorId {
    let at = free_beside(w, dir);
    foe_at(w, at)
}

fn hit() -> Blow {
    Blow {
        strike: Strike::Hit { critical: false },
        damage: 1,
        kind: None,
    }
}

#[test]
fn each_lucky_day_grants_its_gift_and_another_day_none() {
    let gift = |element: &str, animal: &str, day: u32| {
        let w = born(element, animal, day);
        w.state
            .player()
            .talents
            .iter()
            .map(|t| t.id.clone())
            .collect::<Vec<_>>()
    };
    let cases = [
        ("metal", "manticore", 3, "scourge"),
        ("fire", "phoenix", 10, "reborn_whole"),
        ("water", "kraken", 11, "drowning_grip"),
        ("earth", "barghest", 30, "warden_of_the_gate"),
    ];
    for (element, animal, day, id) in cases {
        let held = gift(element, animal, day);
        assert!(held.iter().any(|t| t == id), "{animal}: {held:?}");
        assert!(!gift(element, animal, day - 1).iter().any(|t| t == id));
    }
}

#[test]
fn the_scourge_cleaves_and_the_gate_warden_strikes_soaks_and_is_less_bowed() {
    let scourge = born("metal", "manticore", 3);
    let mods = scourge.state.player().talent_mods(scourge.content());
    assert!(mods.free_blows.cleave);
    let plain = born("metal", "manticore", 4);
    assert!(
        !plain
            .state
            .player()
            .talent_mods(plain.content())
            .free_blows
            .cleave
    );

    let warden = born("earth", "barghest", 30);
    let mods = warden.state.player().talent_mods(warden.content());
    let other = born("earth", "barghest", 29);
    let before = other.state.player().talent_mods(other.content());
    assert!(mods.free_blows.opportunist && !before.free_blows.opportunist);
    let poisoned = status(&warden, "poisoned");
    assert_eq!(before.penalty(poisoned, Stat::Strength, -2), -2);
    assert_eq!(mods.penalty(poisoned, Stat::Strength, -2), -1, "halved");
    assert_eq!(mods.penalty(poisoned, Stat::Strength, -1), 0, "toward 0");
    let soak = |w: &World| w.state.player().protection(&BUILTIN, None).against(None);
    assert_eq!(soak(&warden), soak(&other) + 1);
}

#[test]
fn the_phoenix_rises_once_ashen_and_then_dies() {
    let mut w = born("fire", "phoenix", 2);
    let player = w.state.player;
    let kobold = foe_beside(&mut w, Dir::E);
    let mut events = Vec::new();
    let big = w.state.player().hp + 5;
    assert!(
        !w.hurt(player, big, Harm::Actor(kobold), &mut events),
        "rose"
    );
    assert!(w.death().is_none());
    assert!(w.state.progress.happened(RISEN));
    assert!(suffers(&w, player, "ashen"));
    let max = w.state.player().max_hp(w.content());
    assert_eq!(w.state.player().hp, (max / 3).max(1));
    assert!(
        w.messages()
            .iter()
            .any(|m| m.display().contains("rise from your ashes"))
    );
    // Once in a life: the next death is the last.
    let big = w.state.player().hp + 5;
    assert!(
        w.hurt(player, big, Harm::Actor(kobold), &mut events),
        "dead"
    );
}

#[test]
fn the_firebird_rises_whole() {
    let mut w = born("fire", "phoenix", 10);
    let player = w.state.player;
    let kobold = foe_beside(&mut w, Dir::E);
    let mut events = Vec::new();
    w.afflict(player, status(&w, "poisoned"), 3, None, &mut events);
    w.afflict(player, status(&w, "bleeding"), 2, None, &mut events);
    let disease = w.content().disease_id("the_shakes").expect("a disease");
    w.infect(player, disease, None);
    assert!(!w.state.player().diseases.is_empty());
    let big = w.state.player().hp + 5;
    assert!(
        !w.hurt(player, big, Harm::Actor(kobold), &mut events),
        "rose"
    );
    let me = w.state.player();
    assert_eq!(me.hp, me.max_hp(w.content()), "whole");
    assert!(me.effects.is_empty(), "nothing left: {:?}", me.effects);
    assert!(me.diseases.is_empty());
    assert!(!suffers(&w, player, "ashen"));
    assert!(w.state.progress.happened(RISEN), "and still once in a life");
}

#[test]
fn the_deep_ones_hit_holds_until_a_miss_another_target_or_a_step() {
    let mut w = born("water", "kraken", 11);
    let player = w.state.player;
    let east = free_beside(&w, Dir::E);
    let kobold = foe_at(&mut w, east);
    w.perceive(&mut Vec::new());
    let mut events = Vec::new();
    w.grip(player, kobold, AttackKind::Melee, Some(&hit()), &mut events);
    assert!(suffers(&w, kobold, "held"));
    w.say_news();
    assert_eq!(last(&w), "The kobold is caught in your grip.");
    // Held fast, it cannot step, and beside its holder it attacks instead.
    let away = Dir::ALL
        .into_iter()
        .find(|&d| east.step(d).chebyshev(w.player_pos()) > 1)
        .expect("a step away");
    let at = w.state.actors[&kobold].pos;
    w.creature_steps(kobold, away, &mut events);
    assert_eq!(w.state.actors[&kobold].pos, at, "it did not step");
    assert_eq!(
        w.held_back(kobold, ai::Plan::Step(away)),
        ai::Plan::Attack {
            target: player,
            kind: AttackKind::Melee
        }
    );
    // A shot is no grip; a miss lets go.
    w.grip(
        player,
        kobold,
        AttackKind::Ranged,
        Some(&Blow::missed(None)),
        &mut events,
    );
    assert!(suffers(&w, kobold, "held"));
    w.grip(
        player,
        kobold,
        AttackKind::Melee,
        Some(&Blow::missed(None)),
        &mut events,
    );
    assert!(!suffers(&w, kobold, "held"));
    assert_eq!(
        w.held_back(kobold, ai::Plan::Step(away)),
        ai::Plan::Step(away)
    );
    // Striking another lets go of the first.
    w.grip(player, kobold, AttackKind::Melee, Some(&hit()), &mut events);
    let other = foe_beside(&mut w, Dir::W);
    w.grip(player, other, AttackKind::Melee, Some(&hit()), &mut events);
    assert!(!suffers(&w, kobold, "held") && suffers(&w, other, "held"));
    // Moving lets go of everything.
    w.let_go(player, Some(other));
    assert!(suffers(&w, other, "held"));
    let dir = Dir::ALL
        .into_iter()
        .find(|&d| {
            let p = w.player_pos().step(d);
            let level = w.state.current();
            level
                .terrain
                .get(p)
                .is_some_and(|t| w.content().terrain(*t).walkable)
                && w.state.actor_at(&level.id, p).is_none()
        })
        .expect("somewhere to step");
    w.player_moves(dir, &mut events);
    assert!(!suffers(&w, other, "held"));
}

#[test]
fn without_the_grip_a_hit_holds_nothing() {
    let mut w = born("water", "kraken", 12);
    let player = w.state.player;
    let kobold = foe_beside(&mut w, Dir::E);
    w.grip(
        player,
        kobold,
        AttackKind::Melee,
        Some(&hit()),
        &mut Vec::new(),
    );
    assert!(!suffers(&w, kobold, "held"));
}
