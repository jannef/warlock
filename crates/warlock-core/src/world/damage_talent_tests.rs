//! The damage-type talents in the world (spec §8 "Talents", "Damage types"; #248): Elemental
//! ward takes its share off the chosen element's damage, spells' and statuses' doses alike (the
//! path a poison-typed burst such as the witch's blight takes), and says so on the character
//! screen in words; Crusher drives bludgeoning blows through Protection; and the damage-types
//! system is held to the coverage bar.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::damage::{Damage, ResistanceWord};
use crate::equipment::{free_slot, slots_taken};
use crate::object::Stack;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in the showcase, every score 14, holding nothing.
fn world() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let me = w.player_mut();
    me.stats = Stats::new(PerStat::splat(14), PerStat::splat(14));
    me.equipment = Default::default();
    w
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// Gives the player talent `id` (one rank more), for `choice`.
fn give(w: &mut World, id: &str, choice: Option<&str>) {
    w.player_mut().talents.take(id, choice);
}

/// The player wields `object` in the first free slot it fits.
fn wield(w: &mut World, object: &str) {
    let content = w.content().clone();
    let me = w.player_mut();
    let object = content.object_id(object).expect(object);
    me.inventory.push(Stack::new(object, 1));
    let item = me.inventory.len() - 1;
    let slot = free_slot(&content, &me.equipment, object).expect("a free slot");
    me.equipment.put(&slots_taken(&content, object, slot), item);
}

#[test]
fn elemental_ward_takes_its_share_off_the_chosen_element_only() {
    let mut w = world();
    let me = w.state.player;
    let content = w.content().clone();
    let kind = |id: &str| content.damage_type_id(id);
    let fire = Damage::new(10, kind("fire"));
    let cold = Damage::new(10, kind("cold"));
    let slash = Damage::new(10, kind("slashing"));
    assert_eq!(w.resisted(me, fire), 10);
    give(&mut w, "elemental_ward", Some("fire"));
    assert_eq!(w.resisted(me, fire), 8, "25 %, rounded half up");
    give(&mut w, "elemental_ward", Some("fire"));
    assert_eq!(w.resisted(me, fire), 5, "50 % at the second rank");
    assert_eq!(w.resisted(me, cold), 10, "another element is untouched");
    assert_eq!(w.resisted(me, slash), 10, "a physical kind is untouched");
    // Every element gets its own ward, and the gear's share adds to it under the cap.
    give(&mut w, "elemental_ward", Some("cold"));
    assert_eq!(w.resisted(me, cold), 8);
}

#[test]
fn a_ward_against_poison_shrinks_poisons_damage_and_its_doses() {
    // A poison-typed burst (the witch's blight, #245) does poison damage and poisons: the ward
    // meets both, the damage through the resistance, the dose through `afflict`.
    let mut w = world();
    let me = w.state.player;
    let content = w.content().clone();
    let poison = Damage::new(6, content.damage_type_id("poison"));
    let poisoned = content.status_id("poisoned").expect("poisoned");
    give(&mut w, "elemental_ward", Some("poison"));
    give(&mut w, "elemental_ward", Some("poison"));
    assert_eq!(w.resisted(me, poison), 3);
    let mut events = Vec::new();
    w.afflict(me, poisoned, 4, None, &mut events);
    let severity = w.state.player().effects.get(poisoned).map(|e| e.severity);
    assert_eq!(severity, Some(2), "half the dose");
}

#[test]
fn the_character_screen_says_the_ward_in_words() {
    let mut w = world();
    let fire = w.content().damage_type_id("fire").expect("fire");
    assert!(
        !(w.perception().own_resistances().iter()).any(|(kind, _)| *kind == fire),
        "nothing resisted before"
    );
    give(&mut w, "elemental_ward", Some("fire"));
    let resists = w.perception().own_resistances();
    assert!(
        resists.contains(&(fire, ResistanceWord::Resistant)),
        "{resists:?}"
    );
}

#[test]
fn crusher_drives_bludgeoning_blows_through_protection() {
    let mut w = world();
    let content = w.content().clone();
    let melee = |w: &World| w.state.player().melee(&content).penetration;
    wield(&mut w, "mace");
    let mace = melee(&w);
    give(&mut w, "crusher", None);
    assert_eq!(melee(&w), mace + 2, "a mace crushes");
    // A sword cuts and stabs: Crusher does nothing for it.
    w.player_mut().equipment = Default::default();
    wield(&mut w, "long_sword");
    let sword = w.state.player().melee(&content);
    let bare = sword.penetration;
    assert!(
        !sword
            .kinds
            .iter()
            .any(|k| Some(k) == content.damage_type_id("bludgeoning"))
    );
    assert_eq!(
        bare,
        content
            .object(content.object_id("long_sword").expect("sword"))
            .penetration
    );
    // Bare hands strike.
    w.player_mut().equipment = Default::default();
    assert_eq!(melee(&w), 2, "fists crush too");
}

#[test]
fn the_ward_is_chosen_for_an_element_only() {
    let mut w = world();
    let spec = w.state.character.as_ref().map(|c| &c.spec);
    let factor = crate::content::Factor::of(w.content(), spec);
    let need = w.content().experience().threshold_for(3, factor);
    run(&mut w, &format!("xp:{need}"));
    run(&mut w, "talent:elemental_ward:slashing");
    assert_eq!(last(&w), "`slashing` is not an element.");
    assert!(
        (w.state.player().talents.iter()).all(|t| t.id != "elemental_ward"),
        "a physical kind is no element"
    );
    run(&mut w, "talent:elemental_ward:fire");
    assert_eq!(last(&w), "You learn Elemental ward (fire) (1).");
}

#[test]
fn damage_types_are_held_to_the_coverage_bar() {
    let book = BUILTIN.talents();
    let system = book.system_id("damage_types").expect("the system");
    assert!(!book.system(system).pending, "its talents have landed");
    let held: Vec<&str> = (book.of_system(system))
        .map(|(_, t)| t.id.as_str())
        .collect();
    for id in ["crusher", "elemental_ward"] {
        assert!(held.contains(&id), "{id} in {held:?}");
    }
    assert_eq!(book.coverage_gap(&|_| 0), None);
}
