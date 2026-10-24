//! Magic item content, rolls, and names (spec §8 "Magic items").

use super::*;
use crate::content::{Content, ObjectId};
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeSet;

fn content() -> Content {
    Content::builtin().expect("built-in content")
}

fn object(c: &Content, id: &str) -> ObjectId {
    c.object_id(id).unwrap_or_else(|| panic!("object {id}"))
}

/// A magic item of `object` with `affixes` (id, tier, value), identified or not.
fn item(c: &Content, id: &str, affixes: &[(&str, u8, i32)], name: Option<&str>) -> Stack {
    let magic = Magic {
        rarity: if name.is_some() {
            Rarity::Rare
        } else {
            Rarity::Magic
        },
        level: 1,
        affixes: (affixes.iter())
            .map(|(id, tier, value)| Affix {
                id: (*id).into(),
                tier: *tier,
                value: *value,
            })
            .collect(),
        name: name.map(String::from),
        identified: false,
        carried: 0,
    };
    magic
        .check(c.object(object(c, id)), c)
        .expect("a valid item");
    Stack::with_magic(object(c, id), 1, magic)
}

/// Rolls `n` items at `level` of at least `grade` from one stream.
fn roll_many(c: &Content, n: usize, level: u8, grade: LootGrade) -> Vec<Stack> {
    let mut rng = Rng::stream(7, "magic-test");
    (0..n)
        .filter_map(|_| loot::roll(c, &mut rng, level, grade))
        .collect()
}

#[test]
fn the_builtin_affixes_cover_every_kind_of_item() {
    let c = content();
    let affixes = c.affixes();
    let count = |kind: AffixKind| affixes.all().iter().filter(|a| a.kind == kind).count();
    assert!(
        (20..=30).contains(&count(AffixKind::Prefix)),
        "about 20 to 30 prefixes"
    );
    assert!(
        (20..=30).contains(&count(AffixKind::Suffix)),
        "about 20 to 30 suffixes"
    );
    // Every affix rolls on some base item, and every base item has a prefix and a suffix to roll
    // at the first item level.
    let bases: Vec<_> = (c.object_defs().iter())
        .filter(|o| o.kind.is_some())
        .collect();
    for a in affixes.all() {
        assert!(
            bases.iter().any(|o| may_roll(a, o.kind.unwrap(), o)),
            "{} rolls on nothing",
            a.id
        );
    }
    for o in &bases {
        let kind = o.kind.unwrap();
        for k in [AffixKind::Prefix, AffixKind::Suffix] {
            let any = (affixes.all().iter())
                .any(|a| a.kind == k && a.tiers_at(1) > 0 && may_roll(a, kind, o));
            assert!(any, "{} has no {k:?} at item level 1", o.id);
        }
    }
}

#[test]
fn magic_items_have_one_or_two_affixes_and_mostly_one() {
    let c = content();
    let items = roll_many(&c, 3000, 8, LootGrade::Magic);
    let (mut one, mut two) = (0, 0);
    for stack in &items {
        let Some(magic) = stack.magic() else { continue };
        if magic.rarity != Rarity::Magic {
            continue;
        }
        magic.check(c.object(stack.object), &c).expect("valid");
        let rolled = magic.rolled(&c);
        let per = |k: AffixKind| rolled.iter().filter(|r| r.def.kind == k).count();
        assert!(per(AffixKind::Prefix) <= 1 && per(AffixKind::Suffix) <= 1);
        match rolled.len() {
            1 => one += 1,
            2 => two += 1,
            n => panic!("a magic item with {n} affixes"),
        }
        assert!(magic.name.is_none() && !magic.identified);
    }
    assert!(
        one > 2 * two && two > 0,
        "{one} with one affix, {two} with two"
    );
}

#[test]
fn rare_items_have_up_to_three_of_each_usually_three_or_four() {
    let c = content();
    let items = roll_many(&c, 2000, 8, LootGrade::Rare);
    let mut counts = [0usize; 7];
    for stack in &items {
        let magic = stack.magic().expect("rare");
        assert_eq!(magic.rarity, Rarity::Rare);
        magic.check(c.object(stack.object), &c).expect("valid");
        let rolled = magic.rolled(&c);
        let per = |k: AffixKind| rolled.iter().filter(|r| r.def.kind == k).count();
        assert!(per(AffixKind::Prefix) <= 3 && per(AffixKind::Suffix) <= 3);
        let effects: BTreeSet<_> = rolled.iter().map(|r| r.def.group()).collect();
        assert_eq!(effects.len(), rolled.len(), "no two affixes of one effect");
        let name = magic.name.as_deref().expect("a name");
        assert_eq!(name.split(' ').count(), 2, "two words: {name}");
        counts[rolled.len()] += 1;
    }
    let usual = counts[3] + counts[4];
    assert!(usual * 3 > items.len() * 2, "mostly 3 or 4: {counts:?}");
    assert!(counts[6] > 0, "now and then six: {counts:?}");
}

#[test]
fn tiers_follow_the_item_level_and_values_their_range() {
    let c = content();
    for level in [1, 3, 8] {
        for stack in roll_many(&c, 800, level, LootGrade::Rare) {
            let magic = stack.magic().expect("rare");
            assert_eq!(magic.level, level);
            for r in magic.rolled(&c) {
                let tier = r.def.tier(r.tier).expect("a tier");
                assert!(
                    tier.level <= level,
                    "{} tier {} at level {level}",
                    r.def.id,
                    r.tier
                );
                assert!((tier.min..=tier.max).contains(&r.value));
                let kind = c.object(stack.object).kind.expect("a base item");
                assert!(r.def.bases.contains(&kind), "{} on {kind:?}", r.def.id);
            }
            // Bases come from their level on.
            assert!(c.object(stack.object).level <= level);
        }
    }
    // Deep items reach the top tiers; shallow ones never do.
    let top = |level| {
        roll_many(&c, 800, level, LootGrade::Rare).iter().any(|s| {
            let m = s.magic().expect("rare");
            m.rolled(&c).iter().any(|r| r.tier >= 4)
        })
    };
    assert!(top(8) && !top(1));
}

#[test]
fn loot_is_mostly_normal_and_rolls_the_same_from_the_same_stream() {
    let c = content();
    let items = roll_many(&c, 2000, 4, LootGrade::Any);
    let of = |r: Rarity| items.iter().filter(|s| s.rarity() == r).count();
    let (normal, magic, rare) = (of(Rarity::Normal), of(Rarity::Magic), of(Rarity::Rare));
    assert!(
        normal > magic && magic > rare && rare > 0,
        "{normal} {magic} {rare}"
    );
    assert_eq!(
        items,
        roll_many(&c, 2000, 4, LootGrade::Any),
        "deterministic"
    );
    // Ammunition comes in stacks.
    let ammo: Vec<_> = (items.iter())
        .filter(|s| c.object(s.object).kind == Some(ItemKind::Ammo))
        .collect();
    assert!(!ammo.is_empty());
    assert!(ammo.iter().all(|s| loot::AMMO.contains(&(s.count as i32))));
}

#[test]
fn names_follow_rarity_and_what_the_player_knows() {
    let c = content();
    let mut sword = item(
        &c,
        "long_sword",
        &[("sharp", 2, 2), ("of_the_bear", 1, 1)],
        None,
    );
    assert_eq!(sword.describe(&c), "a long sword (magic)");
    assert_eq!(sword.title(&c), "long sword");
    assert!(sword.known_magic().is_none() && !sword.is_identified());
    assert_eq!(sword.rarity(), Rarity::Magic);
    assert!(sword.identify());
    assert!(!sword.identify(), "once is enough");
    assert_eq!(sword.describe(&c), "a Sharp long sword of the Bear");
    assert_eq!(sword.title(&c), "Sharp long sword of the Bear");
    assert!(sword.known_magic().is_some());

    let ring = item(&c, "iron_ring", &[("accurate", 1, 1)], None);
    let mut ring = ring;
    ring.identify();
    assert_eq!(ring.describe(&c), "an Accurate iron ring");

    let mut helm = item(
        &c,
        "iron_helm",
        &[("hale", 1, 3), ("of_the_owl", 1, 1), ("radiant", 1, 0)],
        Some("Dread Visage"),
    );
    assert_eq!(helm.describe(&c), "an iron helm (rare)");
    helm.identify();
    assert_eq!(helm.describe(&c), "Dread Visage, an iron helm");
    assert_eq!(helm.title(&c), "Dread Visage");

    let mut arrows = item(
        &c,
        "arrow",
        &[("sharp", 1, 1), ("of_durability", 1, 25)],
        None,
    );
    arrows.count = 12;
    assert_eq!(arrows.describe(&c), "12 arrows (magic)");
    arrows.identify();
    assert_eq!(arrows.describe(&c), "12 Sharp arrows of Durability");
    assert_eq!(arrows.describe_truth(&c), "12 Sharp arrows of Durability");
    let unknown = item(&c, "arrow", &[("sharp", 1, 1)], None);
    assert_eq!(unknown.describe_truth(&c), "a Sharp arrow (unidentified)");
}

#[test]
fn bad_items_are_refused() {
    let c = content();
    let sword = c.object(object(&c, "long_sword"));
    let magic = |affixes: &[(&str, u8, i32)], name: Option<&str>| Magic {
        rarity: if name.is_some() {
            Rarity::Rare
        } else {
            Rarity::Magic
        },
        level: 5,
        affixes: (affixes.iter())
            .map(|(id, tier, value)| Affix {
                id: (*id).into(),
                tier: *tier,
                value: *value,
            })
            .collect(),
        name: name.map(String::from),
        identified: false,
        carried: 0,
    };
    let bad = [
        (magic(&[], None), "an affix"),
        (magic(&[("nonsense", 1, 1)], None), "unknown affix"),
        (magic(&[("hale", 1, 3)], None), "does not roll"),
        (magic(&[("sharp", 9, 1)], None), "no tier"),
        (magic(&[("sharp", 1, 5)], None), "rolls 1 to 1"),
        (
            magic(&[("sharp", 1, 1), ("heavy", 1, 2)], None),
            "at most 1 prefixes",
        ),
        (
            magic(
                &[("sharp", 1, 1), ("glinting", 1, 1), ("of_precision", 1, 1)],
                Some("X"),
            ),
            "two affixes do",
        ),
    ];
    for (m, needle) in bad {
        let e = m.check(sword, &c).expect_err(needle);
        assert!(e.contains(needle), "{e}");
    }
    let mut rare = magic(&[("sharp", 1, 1)], None);
    rare.rarity = Rarity::Rare;
    assert!(rare.check(sword, &c).is_err(), "a rare item has a name");
    let rock = c.object(object(&c, "rock"));
    assert!(
        magic(&[("sharp", 1, 1)], None).check(rock, &c).is_err(),
        "no kind"
    );
}

#[test]
fn magic_stacks_join_only_the_same_magic() {
    let c = content();
    let sharp = item(&c, "arrow", &[("sharp", 1, 1)], None);
    let plain = Stack::new(object(&c, "arrow"), 5);
    let mut pile = vec![plain.clone()];
    crate::object::add_to_pile(&mut pile, &[sharp.clone(), sharp.clone()], &c);
    assert_eq!(
        pile.len(),
        2,
        "the sharp arrows join each other, not the plain ones"
    );
    assert_eq!((pile[0].count, pile[1].count), (2, 5));
}

#[test]
fn effects_sum_with_their_rules() {
    let c = content();
    let shimmering = item(&c, "iron_ring", &[("shimmering", 2, 2)], None);
    let warding = item(
        &c,
        "leather_boots",
        &[("of_warding", 1, 1), ("nimble", 1, 1)],
        None,
    );
    let gloves = item(
        &c,
        "leather_gloves",
        &[("nimble", 1, 1), ("of_precision", 1, 1)],
        None,
    );
    let inventory = vec![shimmering, warding, gloves];
    let mut worn = crate::equipment::Equipment::default();
    for (slot, i) in [("left_ring", 0), ("feet", 1), ("hands", 2)] {
        worn.put(&[c.slot_id(slot).unwrap()], i);
    }
    let mods = Mods::worn(&inventory, &worn, &c);
    assert_eq!(
        mods.get(Effect::Deflection),
        2,
        "the best Deflection counts"
    );
    assert_eq!(mods.get(Effect::Dodge), 2, "Dodge adds up");
    assert_eq!(mods.get(Effect::ToHit), 1, "global on gloves");
    // A weapon's attack effects are its own.
    let sword = item(
        &c,
        "long_sword",
        &[("sharp", 1, 1), ("of_the_bear", 1, 1)],
        None,
    );
    let local = Mods::local(&sword, &c);
    assert_eq!(
        (
            local.get(Effect::Damage),
            local.stat(crate::stats::Stat::Strength)
        ),
        (1, 0)
    );
    assert_eq!(mods::faster(1000, 10), 900);
    assert_eq!(mods::faster(1000, 90), 500, "at most 50 % off");
    assert_eq!(
        mods::faster(120, 50),
        100,
        "never below the cheapest action"
    );
}

#[test]
fn descriptions_say_base_stats_and_known_affixes_in_words() {
    use super::describe::{LineKind, lines};
    let c = content();
    let text = |s: &Stack| -> Vec<String> { lines(s, &c).into_iter().map(|l| l.text).collect() };
    let plain = Stack::one(object(&c, "long_sword"));
    assert_eq!(
        text(&plain),
        [
            "Long sword · steel blade, leather grip · 1.8 kg",
            "Long blade, for the main hand",
            "Damage 1d8; criticals on 20–24, ×2 damage",
            "Slashing or piercing damage",
        ]
    );
    let mut sword = item(
        &c,
        "long_sword",
        &[("sharp", 2, 2), ("of_celerity", 1, 6)],
        None,
    );
    let unknown = text(&sword);
    assert_eq!(
        unknown[0],
        "Magic long sword · steel blade, leather grip · 1.8 kg"
    );
    assert!(
        unknown
            .last()
            .is_some_and(|l| l.starts_with("Unidentified"))
    );
    assert!(
        !unknown
            .iter()
            .any(|l| l.contains("Sharp") || l.contains("Celerity"))
    );
    sword.identify();
    let known = lines(&sword, &c);
    let affixes: Vec<(LineKind, &str)> = (known.iter())
        .filter(|l| matches!(l.kind, LineKind::Affix(_)))
        .map(|l| (l.kind, l.text.as_str()))
        .collect();
    assert_eq!(
        affixes,
        [
            (LineKind::Affix(AffixKind::Prefix), "Sharp: +2 damage"),
            (
                LineKind::Affix(AffixKind::Suffix),
                "of Celerity: attacks a little faster"
            ),
        ]
    );
    assert!(known.iter().any(|l| l.text == "Item level 1"));
    // Global effects say so; speed stays in words.
    let mut gloves = item(
        &c,
        "leather_gloves",
        &[("vicious", 1, 1), ("of_celerity", 3, 14)],
        None,
    );
    gloves.identify();
    let words = text(&gloves);
    assert!(
        words.contains(&"Vicious: +1 damage with every attack".to_string()),
        "{words:?}"
    );
    assert!(words.contains(&"of Celerity: all your attacks are much faster".to_string()));
    assert!(
        !words
            .iter()
            .any(|l| l.contains('%') && l.contains("faster"))
    );
    // Missile weapons and armour.
    let bow = text(&Stack::one(object(&c, "long_bow")));
    assert_eq!(
        bow[1..],
        [
            "Bow, readied as a missile weapon; shoots arrows",
            "Damage 1d8+2 a shot; criticals on 22–24, ×3 damage",
            "Piercing damage",
            "Needs Strength 14 to draw",
            "Shoots up to 15 cells",
        ]
    );
    let mail = text(&Stack::one(object(&c, "chain_mail")));
    assert_eq!(
        mail[1..],
        [
            "Worn: body",
            "Armour +2",
            "Protection 3 (+1 against slashing, −1 against piercing)",
            "Dexterity adds at most +4 to Defence while worn",
        ]
    );
    // Penetration is a fact, never a number.
    let hammer = text(&Stack::one(object(&c, "war_hammer")));
    assert!(hammer.iter().any(|l| l == "Pierces armour"), "{hammer:?}");
    assert!(!hammer.iter().any(|l| l.contains("enetrat")), "{hammer:?}");
    let great = text(&Stack::one(object(&c, "great_sword")));
    assert!(great.contains(&"Great blade, for both hands".to_string()));
    assert!(great.contains(&"Slow to strike".to_string()));
    let mut arrows = Stack::one(object(&c, "arrow"));
    arrows.count = 12;
    let arrows = text(&arrows);
    assert_eq!(arrows[0], "12 arrows · iron head, wood shaft · 360 g");
    assert!(arrows.contains(&"Ammunition, quivered; shot from bows".to_string()));
}
