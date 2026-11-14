//! How often items are made of each material (#1191, spec §7 "Materials"): the base material
//! dominates, a material that suits the item poorly is rare and an odd one very rare, and a
//! creature's faction favours its own materials within the item's construction list.

use crate::content::{Content, Fit, MaterialId, ObjectDef};
use crate::magic::{LootGrade, loot};
use crate::rng::Rng;
use crate::variant::{self, PER_MILLION, Source};
use std::collections::BTreeMap;

fn builtin() -> Content {
    Content::builtin().expect("built-in content")
}

/// The share of `weights` in parts per million that `which` picks.
fn share(weights: &[(MaterialId, u32)], which: impl Fn(MaterialId) -> bool) -> u32 {
    (weights.iter())
        .filter(|(m, _)| which(*m))
        .map(|(_, w)| w)
        .sum()
}

/// What kind of item `def` is, for the reports.
fn use_of(def: &ObjectDef) -> &'static str {
    if def.is_jewellery() {
        return "jewellery";
    }
    match def.role() {
        crate::content::PartRole::Weapon => "weapon",
        crate::content::PartRole::Armour => "armour",
        crate::content::PartRole::Any => "other",
    }
}

#[test]
fn the_base_material_dominates_and_odd_ones_are_very_rare() {
    // Exact, from the weights of every object that may be made of another material.
    let content = builtin();
    let mut checked = 0;
    for def in content.object_defs() {
        for source in [Source::Loot, Source::gear(1), Source::gear(20)] {
            for level in [1, 3, 5, 10, 20, 40] {
                let weights = variant::weights(&content, def, source, level);
                if weights.is_empty() {
                    continue;
                }
                checked += 1;
                let total = share(&weights, |_| true);
                assert!(
                    (PER_MILLION - 100..PER_MILLION + 100).contains(&total),
                    "{}: {total}",
                    def.id
                );
                let own = share(&weights, |m| m == def.material());
                assert!(own >= 800_000, "{} at {level}: {own} own", def.id);
                if level <= 5 && matches!(source, Source::Loot) {
                    assert!(own >= 880_000, "{} at {level}: {own} own", def.id);
                }
                let fit_of = |m: MaterialId| variant::fit(def, content.material(m));
                let odd = share(&weights, |m| m != def.material() && fit_of(m) == Fit::Odd);
                assert!(odd < 10_000, "{} at {level}: {odd} odd", def.id);
            }
        }
    }
    assert!(checked > 300, "{checked}");
}

#[test]
fn drawn_loot_holds_the_targets_by_item_class() {
    // Sampled through the loot roll: by what an item is for, how many finds are their own
    // material, another that suits them, a poor one, or an odd one.
    let content = builtin();
    for level in [1, 5] {
        let mut by: BTreeMap<&str, [u32; 4]> = BTreeMap::new();
        for seed in 1..=6u64 {
            let mut rng = Rng::stream(seed, &format!("test/materials/{level}"));
            for _ in 0..4000 {
                let Some(stack) =
                    loot::roll_from(&content, &mut rng, level, LootGrade::Any, Source::Loot)
                else {
                    continue;
                };
                let def = content.object(stack.object);
                if def.made_of(0).len() < 2 || stack.gem(&content).is_some() {
                    continue;
                }
                let main = stack.parts(&content)[0].material;
                let slot = if main == def.material() {
                    0
                } else {
                    match variant::fit(def, content.material(main)) {
                        Fit::Good => 1,
                        Fit::Poor => 2,
                        Fit::Odd => 3,
                    }
                };
                by.entry(use_of(def)).or_default()[slot] += 1;
            }
        }
        for (kind, counts) in &by {
            let total: u32 = counts.iter().sum();
            if total < 500 {
                continue;
            }
            let percent = |n: u32| f64::from(n) * 100.0 / f64::from(total);
            let own = percent(counts[0]);
            assert!(
                (85.0..=93.0).contains(&own),
                "{kind} at {level}: {counts:?}"
            );
            assert!(percent(counts[2]) < 3.0, "{kind} at {level}: {counts:?}");
            assert!(percent(counts[3]) < 0.5, "{kind} at {level}: {counts:?}");
        }
        assert!(
            by.contains_key("weapon") && by.contains_key("armour"),
            "{by:?}"
        );
    }
}

#[test]
fn tin_long_swords_and_wool_rings_are_odd() {
    let content = builtin();
    let object = |id: &str| content.object(content.object_id(id).expect(id));
    let material = |id: &str| content.material(content.material_id(id).expect(id));
    let fit = |o: &str, m: &str| variant::fit(object(o), material(m));
    assert_eq!(fit("long_sword", "tin"), Fit::Odd);
    assert_eq!(fit("long_sword", "bronze"), Fit::Good);
    assert_eq!(fit("long_sword", "copper"), Fit::Poor);
    assert_eq!(fit("gold_ring", "wool"), Fit::Odd);
    assert_eq!(fit("gold_ring", "silver"), Fit::Good);
    assert_eq!(fit("gold_ring", "bone"), Fit::Good);
    assert_eq!(fit("chain_mail", "tin"), Fit::Poor);
    // Odd stays possible: nothing found is ruled out by its fit.
    let ring = object("gold_ring");
    let wool = content.material_id("wool").expect("wool");
    let weights = variant::weights(&content, ring, Source::Loot, 1);
    let ppm = weights.iter().find(|(m, _)| *m == wool).expect("wool").1;
    assert!((1..100).contains(&ppm), "{ppm}");
}

#[test]
fn a_faction_favours_its_materials_within_the_list() {
    let content = builtin();
    let faction = |id: &str| content.factions().id(id).expect(id);
    let material = |id: &str| content.material_id(id).expect(id);
    let object = |id: &str| content.object(content.object_id(id).expect(id));
    let gear = |f: &str| Source::Gear {
        level: 3,
        faction: Some(faction(f)),
    };
    let ppm = |def: &ObjectDef, source, m: &str| {
        let weights = variant::weights(&content, def, source, 3);
        assert_eq!(
            weights.iter().map(|(m, _)| *m).collect::<Vec<_>>(),
            def.made_of(0),
            "{}: the list is the item's own",
            def.id
        );
        (weights.iter())
            .find(|(w, _)| *w == material(m))
            .map_or(0, |(_, w)| *w)
    };
    // An orc's spear is mostly bone; a goblin's short sword often bronze.
    let spear = object("spear");
    let orc_bone = ppm(spear, gear("orcs"), "bone");
    assert!(orc_bone > 500_000, "{orc_bone}");
    assert!(ppm(spear, Source::gear(3), "bone") < 100_000);
    assert!(ppm(object("short_sword"), gear("goblins"), "bronze") > 300_000);
    // The dead of the barrows carry bronze.
    assert!(ppm(object("long_sword"), gear("undead"), "bronze") > 400_000);
    // Never a material the item cannot be made of: no bone or flint long sword for a kobold,
    // whose copper still counts.
    let sword = object("long_sword");
    for m in ["bone", "flint"] {
        assert!(!sword.made_of(0).contains(&material(m)), "{m}");
        assert_eq!(ppm(sword, gear("kobolds"), m), 0);
    }
    assert!(ppm(sword, gear("kobolds"), "copper") >= 100_000);
    // A faction with no favourites draws as gear does.
    let plain = Source::Gear {
        level: 3,
        faction: Some(faction("cult")),
    };
    assert_eq!(
        variant::weights(&content, sword, plain, 3),
        variant::weights(&content, sword, Source::gear(3), 3)
    );
}

#[test]
fn rolled_creature_gear_follows_its_tribe() {
    // Sampled through the gear roll: a skeleton warrior's sword or spear is bronze far more
    // often than a faction's plain draw would make it.
    let content = builtin();
    let skeleton = content.creature_id("skeleton_warrior").expect("skeleton");
    let bronze = content.material_id("bronze").expect("bronze");
    let (mut arms, mut of_bronze) = (0, 0);
    for seed in 1..=400u64 {
        let mut rng = Rng::stream(seed, "test/tribe");
        let pack = crate::gear::roll(&content, &mut rng, skeleton, 3);
        for stack in &pack.items {
            if content.object(stack.object).attack.is_none() {
                continue;
            }
            arms += 1;
            of_bronze += u32::from(stack.parts(&content)[0].material == bronze);
        }
    }
    assert!(arms > 200, "{arms}");
    assert!(of_bronze * 100 / arms > 30, "{of_bronze} of {arms}");
}
