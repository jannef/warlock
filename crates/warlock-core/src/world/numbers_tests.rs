//! What items give in numbers (spec §6 "Item lists", "Item description"; `magic::numbers`): a
//! weapon's to-hit and damage, a launcher's range, what ammunition adds, armour's and a shield's
//! Dv and Pv; an unidentified item's affixes count for nothing in them until it is identified.

use super::*;
use crate::action::Positions;
use crate::magic::{Affix, Magic, Numbers, Rarity, describe};

fn world() -> World {
    let content = Arc::new(Content::builtin().expect("content"));
    World::new(content, "showcase", None).expect("world")
}

/// One `object` with the affixes `(id, value)` (each tier 1), magic.
fn magic(w: &World, object: &str, affixes: &[(&str, i32)], identified: bool) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: (affixes.iter())
            .map(|(id, value)| Affix {
                id: (*id).into(),
                tier: 1,
                value: *value,
            })
            .collect(),
        name: None,
        identified,
        carried: 0,
        unique: None,
    };
    Stack::with_magic(w.content().object_id(object).expect("object"), 1, magic)
}

fn plain(w: &World, object: &str) -> Stack {
    Stack::one(w.content().object_id(object).expect("object"))
}

fn text(w: &World, stack: &Stack) -> Option<String> {
    Numbers::of(stack, w.content()).text(w.content())
}

/// The player wears `stack` in `slot`, whatever was there taken off.
fn wear(w: &mut World, slot: &str, stack: Stack) {
    let content = w.content().clone();
    let player = w.state.player;
    let actor = w.state.actors.get_mut(&player).expect("player");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, actor.inventory[item].object, slot);
    for &s in &taken {
        if let Some(other) = actor.equipment.get(s) {
            actor.equipment.take_off(other);
        }
    }
    actor.equipment.put(&taken, item);
}

#[test]
fn a_weapon_shows_its_to_hit_and_damage_with_its_known_affixes() {
    let w = world();
    assert_eq!(
        text(&w, &plain(&w, "long_sword")).as_deref(),
        Some("+0 hit, 1d8")
    );
    let sword = magic(&w, "long_sword", &[("accurate", 1), ("sharp", 1)], true);
    assert_eq!(text(&w, &sword).as_deref(), Some("+1 hit, 1d8+1"));
    let lines = describe::lines(&sword, w.content());
    assert!(
        lines.iter().any(|l| l.text == "To hit +1, damage 1d8+1"),
        "{lines:?}"
    );
    // An elemental affix rides on the damage.
    let flaming = magic(&w, "long_sword", &[("flaming", 4)], true);
    assert_eq!(text(&w, &flaming).as_deref(), Some("+0 hit, 1d8 +1d4 fire"));
    // The listed name carries them after it.
    let listed = w.perception().lore().listed(&sword);
    assert!(listed.ends_with("long sword (+1 hit, 1d8+1)"), "{listed}");
}

#[test]
fn an_unidentified_item_shows_its_base_numbers_until_identified() {
    let mut w = world();
    let mail = magic(
        &w,
        "chain_mail",
        &[("hardened", 2), ("reinforced", 1)],
        false,
    );
    assert_eq!(text(&w, &mail).as_deref(), Some("Dv +2, Pv 3"));
    let pv = w.perception().own_protection();
    wear(&mut w, "body", mail);
    // Worn, it still shows its base numbers only; its affixes work all the same (spec §8
    // "Magic items": "Unidentified"), so the player's total, the rules' own, has them.
    let view = w.perception();
    let worn = view.inventory().last().expect("mail").clone();
    assert_eq!(text(&w, &worn).as_deref(), Some("Dv +2, Pv 3"));
    let lines = describe::lines(&worn, w.content());
    assert!(lines.iter().any(|l| l.text == "Dv +2, Pv 3"), "{lines:?}");
    let (dv, worn_pv) = (view.own_defence(), view.own_protection());
    assert_eq!(worn_pv, pv + 5);
    w.apply(&Action::Identify(Positions::default()));
    // Identified, its affixes count in its numbers; the totals stay as they were.
    let view = w.perception();
    let known = view.inventory().last().expect("mail").clone();
    assert_eq!(text(&w, &known).as_deref(), Some("Dv +3, Pv 5"));
    assert_eq!((view.own_defence(), view.own_protection()), (dv, worn_pv));
}

#[test]
fn a_shield_shows_its_defence_only() {
    let w = world();
    assert_eq!(
        text(&w, &plain(&w, "small_shield")).as_deref(),
        Some("Dv +1")
    );
    let shield = magic(&w, "small_shield", &[("sturdy", 2)], true);
    assert_eq!(text(&w, &shield).as_deref(), Some("Dv +3"));
    let lines = describe::lines(&shield, w.content());
    assert!(lines.iter().any(|l| l.text == "Dv +3"), "{lines:?}");
}

#[test]
fn a_launcher_shows_its_range_and_ammunition_what_it_adds() {
    let w = world();
    assert_eq!(
        text(&w, &plain(&w, "short_bow")).as_deref(),
        Some("+0 hit, 1d6, range 10")
    );
    let bow = magic(&w, "short_bow", &[("of_reach", 2)], true);
    assert_eq!(text(&w, &bow).as_deref(), Some("+0 hit, 1d6, range 12"));
    let lines = describe::lines(&bow, w.content());
    assert!(
        lines
            .iter()
            .any(|l| l.text == "To hit +0, damage 1d6, range 12"),
        "{lines:?}"
    );
    // Plain arrows add nothing; a flaming arrow its fire.
    assert_eq!(text(&w, &plain(&w, "arrow")), None);
    let flaming = plain(&w, "flaming_arrow");
    assert_eq!(text(&w, &flaming).as_deref(), Some("+1d4 fire"));
    let lines = describe::lines(&flaming, w.content());
    assert!(
        lines.iter().any(|l| l.text == "Adds to a shot: +1d4 fire"),
        "{lines:?}"
    );
}

#[test]
fn things_that_give_nothing_say_nothing() {
    let w = world();
    assert_eq!(text(&w, &plain(&w, "iron_ring")), None);
    // A ring of Dodge, once known, adds to Defence.
    let ring = magic(&w, "iron_ring", &[("of_evasion", 2)], true);
    assert_eq!(text(&w, &ring).as_deref(), Some("Dv +2"));
    // A helm of Accurate adds to every attack.
    let helm = magic(&w, "iron_helm", &[("accurate", 1)], true);
    assert_eq!(text(&w, &helm).as_deref(), Some("+1 hit, Dv +0, Pv 1"));
    let lines = describe::lines(&helm, w.content());
    assert!(
        lines
            .iter()
            .any(|l| l.text == "Adds to every attack: +1 to hit"),
        "{lines:?}"
    );
    let hidden = magic(&w, "iron_ring", &[("of_evasion", 2)], false);
    assert_eq!(text(&w, &hidden), None);
}
