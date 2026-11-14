//! Healing harms the undead (`world/heal_harm.rs`, #1278): Cure wounds and Mend aimed at an
//! undead beside the caster, Benison about them, and the creature healers that never heal one.

use super::caster_tests::{kobold_at, sp};
use super::spell_type_tests::{cell, files, start};
use super::*;
use crate::action::Action;

/// The shipped creature `text` of id `id` as `new`, of the kobolds' side but undead (its `tags`
/// line replaced): a kobold that died and walks.
fn undead_kobold(text: &str, (id, new): (&str, &str), tags: &str) -> String {
    let name = |s: &str| format!("name = \"{}\"", s.replace('_', " "));
    (text.replacen(&format!("id = \"{id}\""), &format!("id = \"{new}\""), 1))
        .replacen(&name(id), &name(new), 1)
        .replacen(tags, "tags = [\"kobold\", \"undead\"]", 1)
}

/// The spell types' lit hall, the player (a wizard) at (2, 3) knowing the healing spells, with
/// mana, and an undead kobold and kobold shaman among the creature types.
fn world(seed: u64) -> World {
    let mut files = files();
    let kobold = include_str!("../../../../assets/data/creatures/kobold.toml");
    let shaman = include_str!("../../../../assets/data/creatures/kobold_shaman.toml");
    let wight = undead_kobold(kobold, ("kobold", "kobold_wight"), "tags = [\"kobold\"]");
    let tags = "tags = [\"kobold\", \"caster\"]";
    let wight_shaman = undead_kobold(shaman, ("kobold_shaman", "wight_shaman"), tags);
    files.insert("data/creatures/kobold_wight.toml", &wight);
    files.insert("data/creatures/wight_shaman.toml", &wight_shaman);
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = start(content, seed);
    for spell in ["cure_wounds", "mend", "benison", "grave_feast"] {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    w
}

/// A creature of type `creature` `(dx, dy)` from the player with `hp` hit points.
fn put(w: &mut World, creature: &str, (dx, dy): (i32, i32), hp: i32) -> ActorId {
    let id = kobold_at(w, (dx, dy), hp);
    let kind = w.content().creature_id(creature).expect("a creature");
    w.state.actors.get_mut(&id).expect("there").creature = kind;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    id
}

/// The player, with mana, casts `spell` at `at` (none: about themselves).
fn cast(w: &mut World, spell: &str, at: Option<Pos>) {
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    w.apply(&Action::Cast(spell.into(), at));
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display().contains(text))
}

#[test]
fn the_healing_spells_are_the_heals_that_feed_on_no_corpse() {
    let w = world(1);
    let heals: Vec<&str> = (w.content().spells().iter())
        .filter(|s| s.heals_wounds())
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(heals, ["benison", "cure_wounds", "mend"]);
}

#[test]
fn a_healing_touch_on_an_undead_is_an_attack_that_sears_it() {
    let mut w = world(3);
    let skeleton = put(&mut w, "skeleton", (1, 0), 1000);
    let mut harmed = 0;
    for _ in 0..20 {
        let before = w.state.actors[&skeleton].hp;
        cast(&mut w, "cure_wounds", Some(cell(3, 3)));
        let after = w.state.actors[&skeleton].hp;
        assert!(after <= before, "a heal never mends the dead");
        harmed += i32::from(after < before);
    }
    assert!(harmed > 0, "some touches land");
    assert!(harmed < 20, "a touch on an undead is an attack: some miss");
    assert!(
        said(&w, "light of mending sears the skeleton"),
        "said as harm"
    );
}

#[test]
fn an_undead_at_full_health_is_still_seared() {
    let mut w = world(5);
    let skeleton = put(&mut w, "skeleton", (1, 0), 1000);
    let mana = |w: &World| w.state.player().mana;
    cast(&mut w, "mend", Some(cell(3, 3)));
    assert!(mana(&w) < 400, "not refused as not hurt: the mana is spent");
    assert!(w.state.actors.contains_key(&skeleton));
}

#[test]
fn a_healing_touch_on_a_living_creature_heals_it() {
    let mut w = world(3);
    let kobold = put(&mut w, "kobold", (1, 0), 1000);
    w.state.actors.get_mut(&kobold).expect("there").hp = 1;
    cast(&mut w, "cure_wounds", Some(cell(3, 3)));
    assert!(
        w.state.actors[&kobold].hp > 1,
        "the living are healed, friend or foe"
    );
    assert!(!said(&w, "sears"));
}

#[test]
fn benison_sears_the_undead_about_the_caster_and_heals_the_caster() {
    let mut w = world(3);
    let skeleton = put(&mut w, "skeleton", (1, 0), 1000);
    let far = put(&mut w, "skeleton", (5, 0), 1000);
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max - 1;
    cast(&mut w, "benison", None);
    assert!(
        w.state.actors[&skeleton].hp < 1000,
        "the undead beside is seared"
    );
    assert_eq!(w.state.actors[&far].hp, 1000, "one out of the burst is not");
    assert_eq!(w.state.player().hp, max, "the caster is healed");
}

#[test]
fn benison_goes_off_with_only_undead_about() {
    let mut w = world(3);
    let skeleton = put(&mut w, "skeleton", (0, 1), 1000);
    cast(&mut w, "benison", None);
    assert!(w.state.actors[&skeleton].hp < 1000);
}

#[test]
fn grave_feast_is_no_healing_spell() {
    let w = world(1);
    let def = w.content().spell(sp("grave_feast"));
    assert!(!def.heals_wounds());
}

#[test]
fn a_creature_healer_never_heals_an_undead() {
    let mut w = world(3);
    let shaman = put(&mut w, "kobold_shaman", (8, 0), 30);
    let friend = put(&mut w, "kobold", (9, 0), 30);
    for id in [shaman, friend] {
        let a = w.state.actors.get_mut(&id).expect("there");
        a.faction_override = None;
        a.mana = 20;
    }
    w.state.actors.get_mut(&friend).expect("there").hp = 1;
    let player = w.state.player;
    let here = w.player_pos();
    let mend = w.content().spell_id("mend").expect("mend");
    assert_eq!(w.creature_spell(shaman, player, here), Some((mend, friend)));
    // The friend an undead: left alone.
    let wight = w.content().creature_id("kobold_wight").expect("a wight");
    w.state.actors.get_mut(&friend).expect("there").creature = wight;
    assert_ne!(w.creature_spell(shaman, player, here), Some((mend, friend)));
}

#[test]
fn an_undead_caster_heals_no_one_not_even_itself() {
    let mut w = world(3);
    let shaman = put(&mut w, "wight_shaman", (8, 0), 30);
    let a = w.state.actors.get_mut(&shaman).expect("there");
    a.mana = 20;
    a.hp = 1;
    let player = w.state.player;
    let here = w.player_pos();
    let mend = w.content().spell_id("mend").expect("mend");
    let chose = w.creature_spell(shaman, player, here).map(|(s, _)| s);
    assert_ne!(chose, Some(mend));
}
