//! The disarming blow (#1166): `knock_away.rs`, with the bearers' rows.

use super::*;
use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::content::{Content, CreatureSpawn};
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A kitted human warrior in the showcase (seed `seed`), alone on the level, in an open room of
/// floor walled round, its west end at the player's cell, wielding a long sword and nothing
/// else in the hands.
fn arena(seed: u64) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    let sword = object(&w, "long_sword");
    wield(&mut w, Stack::new(sword, 1));
    let (floor, wall) = (
        w.content().terrain_id("floor").expect("floor"),
        w.content().terrain_id("wall").expect("wall"),
    );
    let me = w.player_pos();
    for dy in -6i32..=6 {
        for dx in -1..=21 {
            let edge = dy.abs() == 6 || dx == -1 || dx == 21;
            let p = Pos::new(me.x + dx, me.y + dy);
            let level = w.state.current_mut();
            level.terrain.set(p, if edge { wall } else { floor });
            level.traps.remove(&p);
            level.set_pile(p, Vec::new());
        }
    }
    w
}

fn object(w: &World, id: &str) -> crate::content::ObjectId {
    w.content().object_id(id).expect("an object")
}

/// Puts `stack` in the player's pack and in their main hand (both hands for a two-handed one).
fn wield(w: &mut World, stack: Stack) {
    let content = w.content.clone();
    let hand = content.hand_slots().next().expect("a hand");
    let slots = crate::equipment::slots_taken(&content, stack.object, hand);
    let me = w.player_mut();
    me.inventory.push(stack);
    let item = me.inventory.len() - 1;
    me.equipment.put(&slots, item);
}

/// A creature of type `kind` at `p`, of the undead (hostile to the player).
fn put(w: &mut World, kind: &str, p: Pos) -> ActorId {
    let undead = w.content().factions().id("undead").expect("undead");
    let spawn = CreatureSpawn {
        creature: w.content().creature_id(kind).expect("a creature"),
        faction: Some(undead),
        alignment: None,
        npc: None,
        pos: p,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let (content, level) = (w.content.clone(), w.state.current_level.clone());
    travel::make_creature(&mut w.state, &content, &level, &spawn)
}

fn main_hand(w: &World) -> Option<usize> {
    let hand = w.content().hand_slots().next().expect("a hand");
    w.state.player().equipment.get(hand)
}

/// Lets `foe` strike until the player's main hand is empty (at most `tries` hits); how many it
/// took.
fn knock_until(w: &mut World, foe: ActorId, tries: u32) -> Option<u32> {
    let player = w.state.player;
    for n in 1..=tries {
        w.knock_away(foe, player, AttackKind::Melee);
        if main_hand(w).is_none() {
            return Some(n);
        }
    }
    None
}

/// Where on the level a long sword lies, as (cell, count).
fn swords_on_floor(w: &World) -> Vec<Pos> {
    let sword = object(w, "long_sword");
    let level = w.state.current();
    (level.objects.iter())
        .filter(|(_, pile)| pile.iter().any(|s| s.object == sword))
        .map(|(&p, _)| p)
        .collect()
}

fn last(w: &World) -> Vec<String> {
    let all: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    all[all.len().saturating_sub(2)..].to_vec()
}

#[test]
fn the_duellists_carry_the_blow_and_no_one_else() {
    let content = &*BUILTIN;
    let bearers: Vec<(&str, u8, i32)> = (content.creature_defs().iter())
        .filter_map(|c| c.disarms.map(|d| (c.id.as_str(), d.chance, d.difficulty)))
        .collect();
    assert_eq!(
        bearers,
        [
            ("death_knight", 15, 21),
            ("drow_blademaster", 15, 18),
            ("drow_captain", 15, 19),
            ("ultroloth", 20, 24),
            ("waelcyrge", 20, 28),
        ]
    );
}

#[test]
fn a_failed_hold_drops_the_weapon_on_an_open_cell_beside_the_player() {
    let mut w = arena(7);
    let me = w.player_pos();
    let foe = put(&mut w, "ultroloth", me.step(Dir::E));
    w.perceive(&mut Vec::new());
    let tries = knock_until(&mut w, foe, 200).expect("it knocks the sword away in the end");
    assert!(tries > 1, "not on every hit: the chance and the check");
    let lying = swords_on_floor(&w);
    assert_eq!(lying.len(), 1);
    let at = lying[0];
    assert_eq!(at.chebyshev(me), 1, "beside the player");
    assert_ne!(at, me.step(Dir::E), "never under the ultroloth");
    assert!(w.state.current().is_walkable(w.content(), at));
    assert!(w.state.player().inventory.is_empty(), "out of the pack");
    assert!(w.state.player().equipment.is_empty());
    assert_eq!(
        last(&w),
        [
            "The ultroloth knocks your long sword from your hand!",
            "It lands beside you."
        ]
    );
    // With the hand empty, its hits knock nothing more away.
    let player = w.state.player;
    for _ in 0..50 {
        w.knock_away(foe, player, AttackKind::Melee);
    }
    assert_eq!(swords_on_floor(&w), [at]);
}

#[test]
fn the_roll_is_the_same_every_time() {
    let run = || {
        let mut w = arena(11);
        let me = w.player_pos();
        let foe = put(&mut w, "ultroloth", me.step(Dir::E));
        w.perceive(&mut Vec::new());
        let tries = knock_until(&mut w, foe, 200);
        (tries, swords_on_floor(&w), w.state_hash())
    };
    assert_eq!(run(), run());
}

#[test]
fn with_no_open_cell_beside_the_player_the_weapon_stays_in_hand() {
    let mut w = arena(7);
    let me = w.player_pos();
    let foe = put(&mut w, "ultroloth", me.step(Dir::E));
    let wall = w.content().terrain_id("wall").expect("wall");
    for dir in Dir::ALL {
        if dir != Dir::E {
            w.state.current_mut().terrain.set(me.step(dir), wall);
        }
    }
    w.perceive(&mut Vec::new());
    assert_eq!(knock_until(&mut w, foe, 200), None);
    assert!(swords_on_floor(&w).is_empty());
    assert!(main_hand(&w).is_some());
}

#[test]
fn a_spirit_weapon_knocked_away_fades() {
    let mut w = arena(7);
    w.player_mut().inventory.clear();
    w.player_mut().equipment = Default::default();
    let sword = object(&w, "long_sword");
    wield(&mut w, Stack::spirit(sword));
    let me = w.player_pos();
    let foe = put(&mut w, "ultroloth", me.step(Dir::E));
    w.perceive(&mut Vec::new());
    knock_until(&mut w, foe, 200).expect("knocked from the hand");
    assert!(swords_on_floor(&w).is_empty(), "nothing lies anywhere");
    assert!(w.state.player().inventory.is_empty());
    assert_eq!(
        last(&w),
        [
            "The ultroloth knocks your long sword from your hand!",
            "The long sword fades from your hand."
        ]
    );
}

#[test]
fn only_a_bearer_s_melee_hit_disarms() {
    let mut w = arena(7);
    let me = w.player_pos();
    let kobold = put(&mut w, "kobold", me.step(Dir::E));
    let foe = put(&mut w, "ultroloth", me.step(Dir::SE));
    w.perceive(&mut Vec::new());
    let player = w.state.player;
    for _ in 0..200 {
        w.knock_away(kobold, player, AttackKind::Melee);
        w.knock_away(foe, player, AttackKind::Ranged);
        w.knock_away(player, foe, AttackKind::Melee);
    }
    assert!(main_hand(&w).is_some());
}

#[test]
fn a_two_handed_grip_holds_better_than_one_hand() {
    let content = &*BUILTIN;
    let mut w = arena(7);
    let rank = |w: &World, skill: Option<_>| {
        skill.map_or(0, |s| i32::from(w.state.player().weapon_skills.rank(s)))
    };
    let sword = object(&w, "long_sword");
    let me = w.state.player();
    let lone = rank(&w, me.style(content, AttackKind::Melee, None));
    let one = super::knock_away::hold(content, me, sword) - rank(&w, content.object(sword).skill);
    w.player_mut().inventory.clear();
    w.player_mut().equipment = Default::default();
    let great = object(&w, "great_sword");
    wield(&mut w, Stack::new(great, 1));
    let two = super::knock_away::hold(content, w.state.player(), great)
        - rank(&w, content.object(great).skill);
    assert_eq!(two, one - lone + w.content().tuning().knock_away.two_handed);
}
