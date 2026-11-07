//! Item damage (`world/item_damage.rs`, #1168): every material's `resists` reviewed and loaded,
//! the one check by the material's chance over seeds, never a material against a failed
//! attempt, damaged then destroyed.

use super::class_active_tests::world;
use super::item_damage::Harmed;
use super::*;
use crate::content::{ContentFiles, ItemHarm};
use crate::item_parts::ItemPart;
use crate::object::Stack;
use crate::rng::Rng;

/// A world whose player carries only a long sword of `material`, in the pack's first place.
fn sword_of(material: &str) -> World {
    let mut w = world("warrior");
    let content = w.content().clone();
    let m = content.material_id(material).expect(material);
    let stack = Stack::one(content.object_id("long_sword").expect("sword"))
        .with_parts(vec![ItemPart::material(&content, None, m)]);
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    me.inventory.push(stack);
    w
}

/// Over 200 fates: how often `harm` damages the sword of `material` (from undamaged).
fn damaged_of_200(material: &str, harm: ItemHarm) -> usize {
    let w = sword_of(material);
    (0..200)
        .filter(|&n| {
            let mut v = w.clone();
            v.state.rng = Rng::stream(n, "item_damage_tests");
            v.harm_item(0, harm) == Harmed::Damaged
        })
        .count()
}

#[test]
fn the_material_chance_decides_over_seeds() {
    // Adamantium withstands everything, cloth nothing but rust; iron rusts nearly always.
    for harm in ItemHarm::RESISTED {
        assert_eq!(damaged_of_200("adamantium", harm), 0, "{harm:?}");
    }
    assert_eq!(damaged_of_200("cloth", ItemHarm::Acid), 200);
    assert_eq!(damaged_of_200("cloth", ItemHarm::Fire), 200);
    assert_eq!(damaged_of_200("cloth", ItemHarm::Rust), 0);
    assert_eq!(damaged_of_200("iron", ItemHarm::Rust), 200);
    let steel = damaged_of_200("steel", ItemHarm::Acid);
    assert!(
        (130..=190).contains(&steel),
        "steel withstands acid 20%: {steel}"
    );
    let gold = damaged_of_200("gold", ItemHarm::Acid);
    assert!(gold <= 25, "gold withstands acid 95%: {gold}");
}

#[test]
fn no_material_withstands_a_failed_attempt_and_the_extremes_draw_nothing() {
    let mut w = sword_of("adamantium");
    let rng = w.state.rng.clone();
    assert!(w.withstands(0, ItemHarm::Acid));
    assert!(!w.withstands(0, ItemHarm::Work));
    assert_eq!(w.state.rng, rng, "0 and 100 draw nothing");
    assert_eq!(w.harm_item(0, ItemHarm::Work), Harmed::Damaged);
    let worked = w.state.player().inventory[0]
        .worked()
        .cloned()
        .expect("worked");
    assert_eq!((worked.damaged, worked.by), (true, None));
    assert_eq!(w.harm_item(0, ItemHarm::Work), Harmed::Destroyed);
    assert!(w.state.player().inventory.is_empty());
}

#[test]
fn a_damaged_item_damaged_again_is_destroyed_whatever_the_kind() {
    let mut w = sword_of("iron");
    assert_eq!(w.harm_item(0, ItemHarm::Rust), Harmed::Damaged);
    assert_eq!(
        w.lore().describe(&w.state.player().inventory[0]),
        "a rusted iron long sword"
    );
    let mut cloth = sword_of("cloth");
    assert_eq!(cloth.harm_item(0, ItemHarm::Fire), Harmed::Damaged);
    assert_eq!(cloth.harm_item(0, ItemHarm::Acid), Harmed::Destroyed);
    assert_eq!(w.harm_item(0, ItemHarm::Work), Harmed::Destroyed);
}

#[test]
fn every_material_sets_its_resists_as_reviewed() {
    let content = Content::builtin().expect("content");
    for def in content.materials() {
        assert!(
            def.sets_resists(),
            "material `{}` sets no `resists`",
            def.id
        );
    }
    let of = |id: &str| {
        content
            .material(content.material_id(id).expect(id))
            .resists()
    };
    for id in ["gold", "silver", "electrum", "mithril", "adamantium"] {
        let r = of(id);
        assert!(r.acid >= 85 && r.rust == 100, "{id}: a noble metal: {r:?}");
    }
    for id in ["iron", "steel"] {
        let r = of(id);
        assert!(
            r.rust <= 10 && r.acid <= 20,
            "{id} rusts and corrodes: {r:?}"
        );
    }
    for id in ["wood", "cloth", "leather", "bone"] {
        assert_eq!(of(id).rust, 100, "{id} never rusts");
    }
    assert!(of("wood").fire <= 20 && of("cloth").fire == 0, "they burn");
}

/// The built-in content with tin's file edited by `edit`: the load's error.
fn tin_with(edit: impl Fn(&str) -> String) -> String {
    let mut files = ContentFiles::builtin();
    let path = "data/materials/tin.toml";
    let text = (files.table("materials"))
        .find(|(p, _)| *p == path)
        .map(|(_, t)| t.to_string())
        .expect("tin");
    files.insert(path, &edit(&text));
    Content::load(&files).expect_err("refused").to_string()
}

#[test]
fn the_loader_refuses_a_chance_past_100_and_the_old_key() {
    let err = tin_with(|t| t.replace("acid = 30", "acid = 101"));
    assert!(err.contains("`resists`' `acid` is in percent"), "{err}");
    let err = tin_with(|t| t.replace("group = \"metal\"", "group = \"metal\"\ncorrodes = true"));
    assert!(err.contains("corrodes"), "{err}");
    let err = tin_with(|t| t.replace(", fire = 30", ""));
    assert!(err.contains("fire"), "every kind is given: {err}");
}
