//! The second wave (#1138, the dread design §15 and §12 D14): the black pudding, cockatrice,
//! hydra, rust monster, and gorgon, content only on the mechanisms the dread slices built, plus
//! the hydra's heads, each carrying the gimmick it was written for.

use super::creature::CorrodesWhat;
use super::status::Affliction;
use super::{Combat, Content, CreatureDef};
use crate::stats::Stat;

fn def<'c>(content: &'c Content, id: &str) -> &'c CreatureDef {
    content.creature(content.creature_id(id).unwrap_or_else(|| panic!("{id}")))
}

/// Whether `riders` give Stiffening behind a Toughness check.
fn stiffens(content: &Content, riders: &[super::status::OnHit]) -> bool {
    let stiffening = content.status_id("stiffening").expect("stiffening");
    riders.iter().any(|r| {
        matches!(r.what, Affliction::Status { status, check: Some((Stat::Toughness, _)), .. }
            if status == stiffening)
    })
}

#[test]
fn each_carries_its_gimmick() {
    let content = Content::builtin().expect("content");
    // The black pudding: a blade splits it, and its acid eats both ways.
    let pudding = def(&content, "black_pudding");
    let slashing = content.damage_type_id("slashing").expect("slashing");
    let bludgeoning = content.damage_type_id("bludgeoning").expect("bludgeoning");
    let splits = pudding.splits.as_ref().expect("it splits");
    assert!(splits.on.contains(&slashing) && !splits.on.contains(&bludgeoning));
    assert_eq!(pudding.corrodes.map(|c| c.what), Some(CorrodesWhat::Both));
    // The cockatrice: its peck stiffens.
    let cockatrice = def(&content, "cockatrice");
    let peck = cockatrice.attack.as_ref().expect("a peck");
    assert!(stiffens(&content, &peck.on_hit), "the peck stiffens");
    // The hydra: five heads, regeneration, and heads out of the carcass.
    let hydra = def(&content, "hydra");
    assert_eq!(hydra.natural_attacks().count(), 5, "five heads");
    assert!(hydra.regen > 100, "its wounds close");
    let head = content.creature_id("hydra_head").expect("the heads");
    assert_eq!(
        hydra.dies_into.as_ref().map(|d| d.pool.clone()),
        Some(vec![head])
    );
    assert!(
        !content.creature(head).stray,
        "the heads come only of a hydra"
    );
    // The rust monster: its acid both ways.
    let rust = def(&content, "rust_monster");
    assert_eq!(rust.corrodes.map(|c| c.what), Some(CorrodesWhat::Both));
    // The gorgon: a dragon's breath that stiffens.
    let gorgon = def(&content, "gorgon");
    assert_eq!(gorgon.combat, Combat::Dragon);
    let breath = gorgon.breath.as_ref().expect("a breath");
    assert!(stiffens(&content, &breath.riders), "the breath stiffens");
    let poison = content.damage_type_id("poison").expect("poison");
    assert_eq!(breath.kind, poison);
}
