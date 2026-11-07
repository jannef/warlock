//! Harm to gear and regeneration (`world/corrode.rs`, `world/regen.rs`; the dread design §5.8
//! and §5.9, #1131, #1168): the roll and what it reaches, the material's chance, damaged then
//! destroyed, the weapon form, the once-only line of a piece withstanding it, acid resistance,
//! rust; a troll's pace and what stops it.

use super::class_active_tests::{beside, last, world};
use super::corrode::chance;
use super::*;
use crate::action::AttackKind;
use crate::item_parts::ItemPart;
use crate::object::Stack;

#[test]
fn acid_resistance_halves_the_chance() {
    assert_eq!(
        (chance(25, 0), chance(25, 25), chance(25, -50)),
        (25, 12, 25)
    );
}

/// The player in nothing but `pieces` (object id, slot id, material: none for its own), every
/// one worn or wielded, and their positions.
fn dressed(w: &mut World, pieces: &[(&str, &str, Option<&str>)]) -> Vec<usize> {
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    let mut at = Vec::new();
    for &(object, slot, material) in pieces {
        let mut stack = Stack::one(content.object_id(object).expect(object));
        if let Some(m) = material {
            let m = content.material_id(m).expect(m);
            stack = stack.with_parts(vec![ItemPart::material(&content, None, m)]);
        }
        let item = w.state.player().inventory.len();
        w.player_mut().inventory.push(stack);
        let slot = content.slot_id(slot).expect(slot);
        w.player_mut().equipment.put(&[slot], item);
        at.push(item);
    }
    at
}

fn damaged(w: &World, item: usize) -> bool {
    (w.state.player().inventory[item].worked()).is_some_and(|worked| worked.damaged)
}

/// `n` melee hits of `attacker` on `target`, as far as acid goes.
fn hits(w: &mut World, (attacker, target): (ActorId, ActorId), n: usize) {
    for _ in 0..n {
        w.corrode((attacker, target), (AttackKind::Melee, None));
    }
}

fn count(w: &World, line: &str) -> usize {
    (w.messages().iter())
        .filter(|m| m.display() == line)
        .count()
}

/// Hits of `attacker` on `target` until `done` holds (at most 400).
fn hit_until(w: &mut World, who: (ActorId, ActorId), done: impl Fn(&World) -> bool) {
    for _ in 0..400 {
        if done(w) {
            return;
        }
        hits(w, who, 1);
    }
    assert!(done(w), "it never came");
}

#[test]
fn the_hezrou_damages_worn_mail_then_eats_it_away() {
    let mut w = world("warrior");
    let player = w.state.player;
    let mail = dressed(&mut w, &[("chain_mail", "body", None)])[0];
    let hezrou = beside(&mut w, "hezrou", 500);
    hit_until(&mut w, (hezrou, player), |w| damaged(w, mail));
    assert_eq!(last(&w), "The acid eats at your chain mail shirt.");
    let worked = w.state.player().inventory[mail].worked().cloned();
    assert_eq!(
        worked.and_then(|w| w.by),
        Some(crate::content::ItemHarm::Acid)
    );
    hit_until(&mut w, (hezrou, player), |w| {
        w.state.player().inventory.is_empty()
    });
    assert_eq!(last(&w), "The acid eats your chain mail shirt away.");
    assert!(
        w.state.player().equipment.items().is_empty(),
        "nothing worn"
    );
    // Its armour acid eats nothing of the player's when they strike it.
    let sword = dressed(&mut w, &[("long_sword", "main_hand", None)])[0];
    hits(&mut w, (player, hezrou), 200);
    assert!(!damaged(&w, sword));
}

#[test]
fn the_draw_takes_any_piece_and_its_material_decides() {
    let mut w = world("warrior");
    let player = w.state.player;
    let at = dressed(
        &mut w,
        &[
            ("chain_mail", "body", Some("adamantium")),
            ("iron_helm", "head", None),
        ],
    );
    let hezrou = beside(&mut w, "hezrou", 500);
    hit_until(&mut w, (hezrou, player), |w| damaged(w, at[1]));
    assert!(!damaged(&w, at[0]), "adamantium withstands acid");
    assert_eq!(
        count(&w, "The acid runs off your adamantium chain mail shirt."),
        1,
        "the draw came to the mail too, and said so once"
    );
}

#[test]
fn acid_runs_off_adamantium_and_says_so_once_in_a_life() {
    let mut w = world("warrior");
    let player = w.state.player;
    let mail = dressed(&mut w, &[("chain_mail", "body", Some("adamantium"))])[0];
    let hezrou = beside(&mut w, "hezrou", 500);
    hits(&mut w, (hezrou, player), 300);
    assert!(!damaged(&w, mail));
    assert_eq!(
        count(&w, "The acid runs off your adamantium chain mail shirt."),
        1
    );
    assert!(w.state.progress.happened("acid_runs_off"));
    // Nothing worn at all: nothing to say.
    let mut w = world("warrior");
    dressed(&mut w, &[]);
    let hezrou = beside(&mut w, "hezrou", 500);
    hits(&mut w, (hezrou, player), 100);
    assert!(!w.state.progress.happened("acid_runs_off"));
}

#[test]
fn an_acid_spell_that_harms_the_player_eats_the_body_armour_first() {
    // Acid's `corrodes` (`[types]`, the potions wave 2 design §9.3): no chance roll, the body
    // armour first, the material's draw.
    let mut w = world("warrior");
    let player = w.state.player;
    let at = dressed(
        &mut w,
        &[
            ("iron_helm", "head", None),
            ("chain_mail", "body", Some("silk")),
        ],
    );
    let acid = w.content().damage_type_id("acid");
    let mut events = Vec::new();
    w.spell_type_lands(player, acid, &[player], &mut events);
    assert!(damaged(&w, at[1]), "silk never withstands acid");
    assert!(!damaged(&w, at[0]), "one piece");
    assert_eq!(last(&w), "The acid eats at your silk chain mail shirt.");
    w.spell_type_lands(player, acid, &[player], &mut events);
    assert_eq!(w.state.player().inventory.len(), 1, "eaten away");
    // Fire eats no gear.
    let fire = w.content().damage_type_id("fire");
    w.spell_type_lands(player, fire, &[player], &mut events);
    assert!(!damaged(&w, 0));
}

#[test]
fn the_ember_beetles_blood_pits_the_weapon_that_strikes_it() {
    let mut w = world("warrior");
    let player = w.state.player;
    let sword = dressed(&mut w, &[("long_sword", "main_hand", None)])[0];
    let beetle = beside(&mut w, "ember_beetle", 500);
    hit_until(&mut w, (player, beetle), |w| damaged(w, sword));
    assert_eq!(last(&w), "Your long sword comes away pitted.");
    // Its blows on the player eat nothing (its acid is in its blood).
    let mail = dressed(&mut w, &[("chain_mail", "body", None)])[0];
    hits(&mut w, (beetle, player), 200);
    assert!(!damaged(&w, mail));
}

#[test]
fn the_rust_monster_rusts_both_ways_and_never_cloth() {
    // #1138: `what = "both"`, the worn mail by its hits and the sword that strikes it.
    let mut w = world("warrior");
    let player = w.state.player;
    let at = dressed(
        &mut w,
        &[
            ("chain_mail", "body", None),
            ("long_sword", "main_hand", None),
        ],
    );
    let rust = beside(&mut w, "rust_monster", 500);
    hit_until(&mut w, (rust, player), |w| damaged(w, at[0]));
    assert_eq!(last(&w), "Rust spreads over your chain mail shirt.");
    assert!(!damaged(&w, at[1]), "its hits eat no wielded sword");
    hit_until(&mut w, (player, rust), |w| damaged(w, at[1]));
    assert_eq!(last(&w), "Your long sword comes away rusted.");
    // Leather takes no rust, and acid resistance does not help against it.
    let mut w = world("warrior");
    let leather = dressed(&mut w, &[("chain_mail", "body", Some("leather"))])[0];
    let rust = beside(&mut w, "rust_monster", 500);
    hits(&mut w, (rust, player), 200);
    assert!(!damaged(&w, leather));
    assert_eq!(count(&w, "Your leather chain mail shirt takes no rust."), 1);
}
#[test]
fn a_hit_that_lands_corrodes_through_the_blow() {
    let mut w = world("warrior");
    let player = w.state.player;
    let mail = dressed(&mut w, &[("chain_mail", "body", None)])[0];
    let hezrou = beside(&mut w, "hezrou", 500);
    w.player_mut().hp = 100_000;
    let kind = (AttackKind::Melee, None, AttackSkills::default());
    let profile = w.state.actors[&hezrou].melee(w.content());
    for _ in 0..200 {
        let mut events = Vec::new();
        w.resolve(
            hezrou,
            player,
            kind,
            Some(profile.clone()),
            false,
            &mut events,
        );
        if damaged(&w, mail) {
            return;
        }
    }
    panic!("no blow of the hezrou's ate at the mail");
}

#[test]
fn a_troll_heals_four_times_as_fast_until_fire_or_acid_stops_it() {
    let mut w = world("warrior");
    let troll = beside(&mut w, "troll", 10);
    let kobold = beside(&mut w, "kobold", 1);
    let pace = w.healing_rate(kobold);
    assert_eq!(w.healing_rate(troll), pace * 4);
    let content = w.content().clone();
    for status in ["burning", "corroded"] {
        let id = content.status_id(status).expect(status);
        let mut events = Vec::new();
        let mut w = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
        w.afflict(troll, id, 1, None, &mut events);
        assert_eq!(w.healing_rate(troll), 0, "{status}");
        // The player's own too.
        let player = w.state.player;
        w.afflict(player, id, 1, None, &mut events);
        assert_eq!(w.healing_rate(player), 0, "{status}");
    }
    // The rotting troll heals as anything does.
    let rotting = beside(&mut w, "rotting_troll", 10);
    assert_eq!(w.healing_rate(rotting), w.healing_rate(kobold));
}

#[test]
fn the_player_sees_the_trolls_wounds_close() {
    let mut w = world("warrior");
    let troll = beside(&mut w, "troll", 10);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let before = w.regenerating_hp(troll);
    assert_eq!(before, Some(10));
    w.say_wounds_close(troll, before);
    assert_ne!(last(&w), "The troll's wounds close.", "nothing healed yet");
    let max = w.state.actors[&troll].max_hp(w.content());
    w.state.actors.get_mut(&troll).expect("troll").hp = 10 + max / 10;
    w.say_wounds_close(troll, before);
    assert_eq!(last(&w), "The troll's wounds close.");
    // Neither the player nor a kobold is watched.
    let kobold = beside(&mut w, "kobold", 1);
    assert_eq!(w.regenerating_hp(kobold), None);
    assert_eq!(w.regenerating_hp(w.state.player), None);
}
