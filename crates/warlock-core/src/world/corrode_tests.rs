//! Corroding gear and regeneration (`world/corrode.rs`, `world/regen.rs`; the dread design §5.8
//! and §5.9, #1131): the materials' review, the roll and what it eats, never twice, the weapon
//! form, the once-only runoff line, acid resistance; a troll's pace and what stops it.

use super::class_active_tests::{beside, last, world};
use super::corrode::{RUNS_OFF, chance};
use super::*;
use crate::action::AttackKind;
use crate::content::Corrosion;
use crate::item_parts::ItemPart;
use crate::object::Stack;

/// What the review (the dread design §5.8's table) says of every material.
const CORRODES: &[&str] = &[
    "iron",
    "steel",
    "copper",
    "bronze",
    "brass",
    "tin",
    "zinc",
    "lead",
    "pewter",
    "cadmium",
    "aluminium",
    "duralumin",
    "bendalloy",
    "leather",
    "hardened_leather",
    "fur",
    "cloth",
    "wool",
    "silk",
];
const HALF: &[&str] = &["bone", "horn", "ivory", "chitin"];

#[test]
fn every_material_is_reviewed_for_acid() {
    let content = Content::builtin().expect("content");
    for def in content.materials() {
        let id = def.id.as_str();
        let want = if CORRODES.contains(&id) {
            Corrosion::Yes
        } else if HALF.contains(&id) {
            Corrosion::Half
        } else {
            Corrosion::No
        };
        assert_eq!(def.corrodes(), want, "{id}");
    }
    for id in [
        "gold",
        "silver",
        "electrum",
        "chromium",
        "nicrosil",
        "mithril",
        "adamantium",
    ] {
        let def = content.material(content.material_id(id).expect(id));
        assert_eq!(def.corrodes(), Corrosion::No, "{id}: a metal acid runs off");
    }
    // The two new metals: metal, and no more precious than bronze.
    for id in ["mithril", "adamantium"] {
        let def = content.material(content.material_id(id).expect(id));
        assert_eq!(
            (def.group.as_deref(), def.precious),
            (Some("metal"), false),
            "{id}"
        );
    }
}

#[test]
fn a_bad_corrodes_word_is_refused_at_load() {
    let mut files = crate::content::ContentFiles::builtin();
    let path = "data/materials/tin.toml";
    let text = (files.table("materials"))
        .find(|(p, _)| *p == path)
        .map(|(_, t)| t.to_string())
        .expect("tin");
    let text = text.replace(
        "group = \"metal\"",
        "group = \"metal\"\ncorrodes = \"some\"",
    );
    files.insert(path, &text);
    let err = Content::load(&files).expect_err("refused");
    assert!(err.to_string().contains("`corrodes`"), "{err}");
}

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

#[test]
fn the_hezrou_eats_at_worn_mail_once_and_never_again() {
    let mut w = world("warrior");
    let player = w.state.player;
    let mail = dressed(&mut w, &[("chain_mail", "body", None)])[0];
    let hezrou = beside(&mut w, "hezrou", 500);
    let mut n = 0;
    while !damaged(&w, mail) {
        hits(&mut w, (hezrou, player), 1);
        n += 1;
        assert!(n < 200, "the acid never took");
    }
    assert_eq!(last(&w), "The acid eats at your chain mail shirt.");
    let after = w.state.player().inventory[mail].worked().cloned();
    hits(&mut w, (hezrou, player), 200);
    assert_eq!(
        w.state.player().inventory[mail].worked().cloned(),
        after,
        "never twice"
    );
    assert_eq!(count(&w, "The acid eats at your chain mail shirt."), 1);
    // Its armour acid eats nothing of the player's when they strike it.
    let sword = dressed(&mut w, &[("long_sword", "main_hand", None)])[0];
    hits(&mut w, (player, hezrou), 200);
    assert!(!damaged(&w, sword));
}

#[test]
fn the_draw_takes_only_what_corrodes() {
    let mut w = world("warrior");
    let player = w.state.player;
    let at = dressed(
        &mut w,
        &[
            ("chain_mail", "body", Some("mithril")),
            ("iron_helm", "head", None),
        ],
    );
    let hezrou = beside(&mut w, "hezrou", 500);
    hits(&mut w, (hezrou, player), 300);
    assert!(!damaged(&w, at[0]), "mithril is never eaten");
    assert!(damaged(&w, at[1]), "the helm is");
    assert_eq!(
        count(&w, "The acid runs off your mithril chain mail shirt."),
        0
    );
}

#[test]
fn acid_runs_off_mithril_and_says_so_once_in_a_life() {
    let mut w = world("warrior");
    let player = w.state.player;
    let mail = dressed(&mut w, &[("chain_mail", "body", Some("mithril"))])[0];
    let hezrou = beside(&mut w, "hezrou", 500);
    hits(&mut w, (hezrou, player), 300);
    assert!(!damaged(&w, mail));
    assert_eq!(
        count(&w, "The acid runs off your mithril chain mail shirt."),
        1
    );
    assert!(w.state.progress.happened(RUNS_OFF));
    // Nothing worn at all: nothing to say.
    let mut w = world("warrior");
    dressed(&mut w, &[]);
    let hezrou = beside(&mut w, "hezrou", 500);
    hits(&mut w, (hezrou, player), 100);
    assert!(!w.state.progress.happened(RUNS_OFF));
}

#[test]
fn the_ember_beetles_blood_pits_the_weapon_that_strikes_it() {
    let mut w = world("warrior");
    let player = w.state.player;
    let sword = dressed(&mut w, &[("long_sword", "main_hand", None)])[0];
    let beetle = beside(&mut w, "ember_beetle", 500);
    let mut n = 0;
    while !damaged(&w, sword) {
        hits(&mut w, (player, beetle), 1);
        n += 1;
        assert!(n < 200, "the blood never took");
    }
    assert_eq!(last(&w), "Your long sword comes away pitted.");
    // Its blows on the player eat nothing (its acid is in its blood).
    let mail = dressed(&mut w, &[("chain_mail", "body", None)])[0];
    hits(&mut w, (beetle, player), 200);
    assert!(!damaged(&w, mail));
    // A club of wood is never pitted.
    let club = dressed(&mut w, &[("club", "main_hand", None)])[0];
    hits(&mut w, (player, beetle), 200);
    assert!(!damaged(&w, club));
    assert_eq!(count(&w, "The acid runs off your club."), 1);
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
