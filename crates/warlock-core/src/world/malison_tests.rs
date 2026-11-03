//! Malison (spec §8 "Class abilities"; #905; `world/malison.rs`, `caster.rs::lore_blow`): the
//! witch's at 18. The bite: a creature under a hex takes 3 more from her blows and her spells
//! (Poison lore's +2 on the poisoned lands with spells too); none on an unhexed one. The tithe:
//! a creature dying under a hex of hers, by any cause and anyone's hand, pays her half its level
//! in mana (at least 1, to her maximum), never for her own death; no draws.

use super::class_active_tests::{reach, run, world};
use super::war_cry_tests::foe_at;
use super::*;
use crate::combat::{Blow, Strike};
use crate::damage::Damage;

/// A witch with Malison (and nothing else of the witch's beyond her kit), alone on the level,
/// with mana to spare.
fn witch() -> World {
    let mut w = world("witch");
    w.player_mut().talents.take("malison", None);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    w
}

/// A `creature` `dx` cells east of the player (50 hit points), seen; its id.
fn foe(w: &mut World, creature: &str, dx: i32) -> ActorId {
    let me = w.player_pos();
    let id = foe_at(w, creature, Pos::new(me.x + dx, me.y), 10);
    w.perceive(&mut Vec::new());
    id
}

/// Lays a degree of status `id` on `on`, given by `by`.
fn lay(w: &mut World, on: ActorId, id: &str, by: Option<ActorId>) {
    let s = w.content().status_id(id).expect("a status");
    w.afflict(on, s, 1, by, &mut Vec::new());
}

fn hp(w: &World, id: ActorId) -> i32 {
    w.state.actors[&id].hp
}

/// The player's spell does `amount` (no damage type) to `target`, a sure hit.
fn spell(w: &mut World, target: ActorId, amount: i32) {
    let player = w.state.player;
    let here = w.player_pos();
    let damage = (Damage::new(amount, None), Strike::Hit { critical: false });
    w.spell_strikes(player, target, (here, true), damage, &mut Vec::new());
}

fn told(w: &World) -> bool {
    w.news.iter().any(|n| n == "The hex comes home.")
}

#[test]
fn the_witch_gains_it_at_eighteen() {
    let mut w = world("witch");
    let def = w.content().class("witch").expect("witch");
    let at = (def.abilities.iter()).find(|(_, t, _)| t == "malison");
    assert_eq!(at.map(|(level, _, _)| *level), Some(18));
    let id = w.content().talents().id("malison").expect("the talent");
    assert!(w.content().talents().get(id).granted, "never picked");
    reach(&mut w, 17);
    assert!(!w.state.player().talents.has("malison", 1));
    reach(&mut w, 18);
    assert!(w.state.player().talents.has("malison", 1));
}

#[test]
fn its_bite_names_every_hex_status() {
    let w = witch();
    let mods = w.state.player().talent_mods(w.content());
    let bitten: Vec<StatusId> = (mods.status_damage.iter())
        .filter(|(_, v)| *v == 3)
        .map(|(s, _)| *s)
        .collect();
    let mut hexes = w.content().hex_statuses().to_vec();
    hexes.sort();
    let mut bitten = bitten;
    bitten.sort();
    assert!(!hexes.is_empty());
    assert_eq!(
        bitten, hexes,
        "a row of `malison.toml` for each hex a spell lays"
    );
    assert_eq!(mods.hex_tithe, Some(2));
}

#[test]
fn a_blow_bites_harder_on_the_hexed_and_more_for_each_hex() {
    let mut w = witch();
    let player = w.state.player;
    let target = foe(&mut w, "kobold", 1);
    let blow = Blow::plain(5, None);
    assert_eq!(w.lore_blow(player, target, blow).damage, 5, "not hexed");
    lay(&mut w, target, "ill_wished", None);
    assert_eq!(
        w.lore_blow(player, target, blow).damage,
        8,
        "any hex, anyone's"
    );
    lay(&mut w, target, "weakened", Some(player));
    assert_eq!(w.lore_blow(player, target, blow).damage, 11, "two hexes");
    let missed = Blow {
        strike: Strike::Missed { by: None },
        ..blow
    };
    assert_eq!(w.lore_blow(player, target, missed).damage, 5, "a miss");
    w.player_mut().talents.forget("malison");
    assert_eq!(w.lore_blow(player, target, blow).damage, 5, "without it");
}

#[test]
fn a_spell_bites_harder_on_the_hexed() {
    let mut w = witch();
    let player = w.state.player;
    let plain = foe(&mut w, "kobold", 2);
    let hexed = foe(&mut w, "kobold", 3);
    lay(&mut w, hexed, "sluggish", Some(player));
    spell(&mut w, plain, 10);
    spell(&mut w, hexed, 10);
    assert_eq!(hp(&w, plain), 40);
    assert_eq!(hp(&w, hexed), 37);
}

/// Hit points a kobold five cells east lost to the player's cast of Blight on it, after
/// `prepare` (given the world and the kobold).
fn blighted(talent: &str, prepare: impl Fn(&mut World, ActorId)) -> i32 {
    let mut w = world("witch");
    w.player_mut().talents.take(talent, None);
    w.player_mut().talents.take("blight", None);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    w.player_mut().mana = 200;
    let target = foe(&mut w, "kobold", 5);
    prepare(&mut w, target);
    let at = w.state.actors[&target].pos;
    run(&mut w, &format!("use:blight:{},{}", at.x, at.y));
    50 - hp(&w, target)
}

#[test]
fn blight_bites_harder_on_the_hexed_and_with_poison_lore_on_the_poisoned() {
    let none = |_: &mut World, _: ActorId| {};
    let player = |w: &World| w.state.player;
    let plain = blighted("malison", none);
    assert!(plain > 0, "Blight lands");
    let hexed = blighted("malison", |w, t| lay(w, t, "drowsy", Some(player(w))));
    assert_eq!(hexed - plain, 3);
    let plain = blighted("poison_lore", none);
    let poisoned = blighted("poison_lore", |w, t| lay(w, t, "poisoned", None));
    assert_eq!(poisoned - plain, 2, "Poison lore with a spell");
}

#[test]
fn a_death_under_her_hex_pays_half_its_level() {
    // A kobold (level 2) slain by her spell: 1; a lesser air elemental (10): 5.
    for (creature, paid) in [("kobold", 1), ("lesser_air_elemental", 5), ("bat", 1)] {
        let mut w = witch();
        let player = w.state.player;
        let target = foe(&mut w, creature, 2);
        lay(&mut w, target, "ill_wished", Some(player));
        w.player_mut().mana = 0;
        spell(&mut w, target, 500);
        assert!(!w.state.actors.contains_key(&target), "slain");
        assert_eq!(w.state.player().mana, paid, "{creature}");
        assert!(told(&w));
    }
}

#[test]
fn any_cause_pays_and_no_draw_is_made() {
    // A death by another's hand, by a status, by a trap: every death goes through `dies`.
    let causes: [fn(ActorId) -> (Option<ActorId>, Option<&'static str>); 3] = [
        |other| (Some(other), None),
        |_| (None, Some("poisoned")),
        |_| (None, None),
    ];
    for cause in causes {
        let mut w = witch();
        let player = w.state.player;
        let target = foe(&mut w, "kobold", 4);
        let other = foe(&mut w, "kobold", 6);
        lay(&mut w, target, "webbed", Some(player));
        w.player_mut().mana = 0;
        let (by, of) = cause(other);
        let of = of.map(|s| w.content().status_id(s).expect("a status"));
        let rng = w.state.rng.clone();
        let when = w.state.time;
        w.dies(target, by, of, when, &mut Vec::new());
        assert_eq!(w.state.player().mana, 1);
        assert_eq!(w.state.rng, rng, "no draws");
    }
}

#[test]
fn no_tithe_without_her_hex_or_the_talent() {
    let mut w = witch();
    let player = w.state.player;
    let other = foe(&mut w, "kobold", 6);
    // Another's hex, a status that is no hex, no hex at all.
    let preps: [(&str, Option<ActorId>); 3] = [
        ("ill_wished", Some(other)),
        ("poisoned", Some(player)),
        ("ill_wished", None),
    ];
    for (status, by) in preps {
        let target = foe(&mut w, "kobold", 2);
        lay(&mut w, target, status, by);
        w.player_mut().mana = 0;
        spell(&mut w, target, 500);
        assert_eq!(w.state.player().mana, 0, "{status} by {by:?}");
    }
    w.player_mut().talents.forget("malison");
    let target = foe(&mut w, "kobold", 2);
    lay(&mut w, target, "ill_wished", Some(player));
    spell(&mut w, target, 500);
    assert_eq!(w.state.player().mana, 0, "without the talent");
    assert!(!told(&w));
}

#[test]
fn the_tithe_stops_at_her_maximum_and_says_nothing_when_full() {
    let mut w = witch();
    let player = w.state.player;
    let max = w.state.player().max_mana(w.content());
    let target = foe(&mut w, "lesser_air_elemental", 2);
    lay(&mut w, target, "ill_wished", Some(player));
    w.player_mut().mana = max - 2;
    spell(&mut w, target, 500);
    assert_eq!(w.state.player().mana, max, "5 owed, 2 paid");
    let target = foe(&mut w, "lesser_air_elemental", 2);
    lay(&mut w, target, "ill_wished", Some(player));
    w.news.clear();
    spell(&mut w, target, 500);
    assert_eq!(w.state.player().mana, max);
    assert!(!told(&w), "nothing came home");
}

#[test]
fn never_for_her_own_death() {
    let mut w = witch();
    let player = w.state.player;
    lay(&mut w, player, "ill_wished", Some(player));
    w.player_mut().mana = 0;
    let when = w.state.time;
    w.dies(player, None, None, when, &mut Vec::new());
    assert!(w.state.death.is_some());
    assert_eq!(w.state.player().mana, 0);
}
