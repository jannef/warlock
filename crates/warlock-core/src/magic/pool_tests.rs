//! A vault's loot pools (spec §7 "Level file format"): what they roll, and what is refused.

use super::loot::{self, Luck};
use super::{LootGrade, LootPool, Rarity};
use crate::chest;
use crate::content::{Content, ContentFiles, MAX_ITEM_LEVEL};
use crate::geom::Pos;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeSet;

fn content() -> Content {
    Content::builtin().expect("built-in content")
}

fn pool(c: &Content, names: &[&str], grade: LootGrade) -> LootPool {
    let names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
    LootPool::named(c, &names, grade).expect("a pool")
}

fn roll_many(c: &Content, pool: &LootPool, level: u8, n: u64) -> Vec<Stack> {
    (0..n)
        .filter_map(|seed| {
            let mut rng = Rng::stream(seed, "pool");
            loot::roll_pool(c, &mut rng, level, pool, Luck::default())
        })
        .collect()
}

fn classes(c: &Content, stacks: &[Stack]) -> BTreeSet<String> {
    (stacks.iter())
        .map(|s| c.item_class(c.object(s.object).class).id.clone())
        .collect()
}

#[test]
fn a_group_rolls_all_of_its_classes_and_nothing_else() {
    let c = content();
    let cases: [(&str, &[&str]); 4] = [
        ("weapons", &["melee_weapons", "missile_weapons", "missiles"]),
        (
            "armour",
            &[
                "body_armour",
                "boots",
                "cloaks",
                "gloves",
                "helmets",
                "shields",
            ],
        ),
        ("jewellery", &["amulets", "rings"]),
        ("money", &["coins", "gems"]),
    ];
    for (group, want) in cases {
        let rolled = roll_many(&c, &pool(&c, &[group], LootGrade::Any), 12, 300);
        let want: BTreeSet<String> = want.iter().map(|s| s.to_string()).collect();
        assert_eq!(classes(&c, &rolled), want, "{group}");
    }
}

#[test]
fn a_pool_names_classes_and_groups_together_each_class_once() {
    let c = content();
    let both = pool(&c, &["rings", "jewellery", "potions"], LootGrade::Any);
    let id = |name| c.item_class_id(name).expect(name);
    assert_eq!(both.classes, [id("rings"), id("amulets"), id("potions")]);
}

#[test]
fn a_pool_s_finds_fit_the_item_level() {
    let c = content();
    let armour = pool(&c, &["armour", "weapons"], LootGrade::Magic);
    for level in [1, 3, 8] {
        for stack in roll_many(&c, &armour, level, 200) {
            let def = c.object(stack.object);
            assert!(def.level <= level, "{} at level {level}", def.id);
            let magic = stack.magic();
            assert!(magic.is_none_or(|m| m.level == level), "{}", def.id);
            for affix in magic.into_iter().flat_map(|m| &m.affixes) {
                let affix_def = c.affixes().get(&affix.id).expect("an affix");
                let window = loot::tier_window(affix_def, level);
                let tier = usize::from(affix.tier) - 1;
                assert!(window.contains(&tier), "{} at level {level}", affix.id);
            }
        }
    }
    // The purse is a chest's of the same level: deep floors pay more.
    let money = pool(&c, &["coins"], LootGrade::Any);
    let total = |level| -> u32 {
        (roll_many(&c, &money, level, 200).iter())
            .map(|s| s.count)
            .sum()
    };
    assert!(total(40) > total(1) * 4, "{} {}", total(40), total(1));
    let currency = chest::currency(&c).expect("a currency");
    assert!(
        roll_many(&c, &money, 1, 50)
            .iter()
            .all(|s| s.object == currency)
    );
}

#[test]
fn a_pool_s_grade_is_the_least_it_rolls() {
    let c = content();
    let rare = pool(&c, &["jewellery", "armour"], LootGrade::Rare);
    let rolled = roll_many(&c, &rare, 12, 200);
    assert!(!rolled.is_empty());
    for stack in rolled {
        let rarity = stack.magic().map(|m| m.rarity);
        assert_eq!(rarity, Some(Rarity::Rare), "{}", c.object(stack.object).id);
    }
}

#[test]
fn a_shallow_floor_rolls_only_the_classes_found_there() {
    // Rods are first found at item level 3: alone a pool of them is empty above that, and beside
    // what is found everywhere only that is drawn.
    let c = content();
    let rods = pool(&c, &["rods"], LootGrade::Any);
    assert!(roll_many(&c, &rods, 2, 50).is_empty());
    assert_eq!(roll_many(&c, &rods, 3, 50).len(), 50);
    let beside = pool(&c, &["rods", "rings"], LootGrade::Any);
    let shallow = roll_many(&c, &beside, 2, 50);
    assert_eq!(shallow.len(), 50);
    assert_eq!(classes(&c, &shallow), BTreeSet::from(["rings".to_string()]));
}

#[test]
fn a_grade_alone_rolls_as_it_always_did() {
    let c = content();
    for grade in [LootGrade::Any, LootGrade::Magic, LootGrade::Rare] {
        for seed in 0..100 {
            let mut plain = Rng::stream(seed, "pool");
            let mut pooled = Rng::stream(seed, "pool");
            let source = crate::variant::Source::Loot;
            let luck = Luck::default();
            assert_eq!(
                loot::roll_with(&c, &mut plain, 7, grade, source, luck),
                loot::roll_pool(&c, &mut pooled, 7, &grade.into(), luck),
            );
            assert_eq!(plain.below(1 << 30), pooled.below(1 << 30));
        }
    }
}

/// The flooded cell's one loot cell rolled on a floor of `level` from the run seed.
fn flooded_cell(c: &Content, seed: u64, level: u8) -> Vec<Stack> {
    let vault = (c.vaults().iter())
        .find(|v| v.id() == "flooded_cell")
        .expect("flooded cell");
    let mut template = vault.template.clone();
    template.item_level = level;
    let markers = template.loot.clone();
    crate::mapgen::place_loot(&mut template, c, seed, &markers, (None, Luck::default()));
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].0, Pos::new(4, 2));
    assert_eq!(template.objects.len(), 1);
    template.objects[&markers[0].0].clone()
}

#[test]
fn the_flooded_cell_holds_money_or_jewellery_differing_by_run() {
    let c = content();
    let mut found = BTreeSet::new();
    for seed in 0..100 {
        let pile = flooded_cell(&c, seed, 4);
        assert_eq!(
            pile,
            flooded_cell(&c, seed, 4),
            "the same seed, the same find"
        );
        found.extend(classes(&c, &pile));
    }
    let want = ["amulets", "coins", "gems", "rings"].map(String::from);
    assert_eq!(found, BTreeSet::from(want));
    // Its pool leaves it bare on no floor, however shallow or deep.
    assert!(!flooded_cell(&c, 7, 1).is_empty());
    assert!(!flooded_cell(&c, 7, MAX_ITEM_LEVEL).is_empty());
}

fn vault_with(legend: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "vaults/pooled.toml",
        &format!("name = \"Pooled\"\nmap = '''\n##+##\n#.m.#\n#####\n'''\n[legend]\n{legend}\n"),
    );
    Content::load(&files).map_err(|e| e.message)
}

#[test]
fn a_vault_s_legend_takes_a_grade_or_a_pool() {
    for loot in [
        "\"magic\"",
        "{ pool = \"jewellery\" }",
        "{ pool = \"rings\", grade = \"rare\" }",
        "{ pool = [\"armour\", \"coins\"], grade = \"any\" }",
    ] {
        let legend = format!("\"m\" = {{ terrain = \"floor\", loot = {loot} }}");
        assert!(vault_with(&legend).is_ok(), "{loot}");
    }
}

#[test]
fn an_unknown_pool_is_refused_and_an_empty_one_loads() {
    let legend = "\"m\" = { terrain = \"floor\", loot = { pool = \"swords\" } }";
    let err = vault_with(legend).expect_err("swords");
    assert!(err.contains("`swords` is no item class"), "{err}");
    // A pool that can hold nothing of its grade leaves its cell bare.
    for loot in ["{ pool = [] }", "{ pool = \"coins\", grade = \"rare\" }"] {
        let legend = format!("\"m\" = {{ terrain = \"floor\", loot = {loot} }}");
        assert!(vault_with(&legend).is_ok(), "{loot}");
    }
    let c = content();
    let coins = pool(&c, &["coins"], LootGrade::Rare);
    assert!(roll_many(&c, &coins, MAX_ITEM_LEVEL, 50).is_empty());
}

#[test]
fn a_level_file_may_not_roll_a_pool() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/pooled.toml",
        "name = \"Pooled\"\nmap = '''\n...m\n'''\n[legend]\n\
         \"m\" = { terrain = \"floor\", loot = { pool = \"rings\" } }\n",
    );
    let err = Content::load(&files)
        .expect_err("a level with a pool")
        .message;
    assert!(err.contains("rolled only in vaults"), "{err}");
}
