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
        unique: None,
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
    // A prefix only 40 %, a suffix only 40 %, one of each 20 % (affixes.toml).
    let share = two * 100 / (one + two);
    assert!(
        (15..=25).contains(&share),
        "{one} with one affix, {two} with two"
    );
}

#[test]
fn rare_items_have_three_to_six_affixes_at_most_three_of_each() {
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
        let categories: BTreeSet<_> = rolled.iter().map(|r| &r.def.category).collect();
        assert_eq!(categories.len(), rolled.len(), "no two of one category");
        let name = magic.name.as_deref().expect("a name");
        assert_eq!(name.split(' ').count(), 2, "two words: {name}");
        counts[rolled.len()] += 1;
    }
    // 3: 50 %, 4: 30 %, 5: 15 %, 6: 5 % (affixes.toml), a few short where nothing is left.
    let share = |n: usize| counts[n] * 100 / items.len();
    assert!((44..=56).contains(&share(3)), "half three: {counts:?}");
    assert!((25..=35).contains(&share(4)), "{counts:?}");
    assert!((11..=19).contains(&share(5)), "{counts:?}");
    assert!((2..=8).contains(&share(6)), "now and then six: {counts:?}");
}

/// No item loot makes has two affixes of one category (nor, so, of one effect), at any item
/// level, over many draws.
#[test]
fn no_rolled_item_repeats_an_affix_category() {
    let c = content();
    let mut rolled_any = 0;
    for level in [1, 4, 8, 12] {
        for stack in roll_many(&c, 1500, level, LootGrade::Rare) {
            let magic = stack.magic().expect("rare");
            magic.check(c.object(stack.object), &c).expect("valid");
            let rolled = magic.rolled(&c);
            let categories: BTreeSet<_> = rolled.iter().map(|r| &r.def.category).collect();
            let groups: BTreeSet<_> = rolled.iter().map(|r| r.def.group()).collect();
            assert_eq!(categories.len(), rolled.len(), "{:?}", magic.affixes);
            assert_eq!(groups.len(), rolled.len(), "{:?}", magic.affixes);
            rolled_any += rolled.len();
        }
    }
    assert!(rolled_any > 6000, "{rolled_any} affixes");
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
        unique: None,
    };
    let bad = [
        (magic(&[], None), "an affix"),
        (magic(&[("nonsense", 1, 1)], None), "unknown affix"),
        (magic(&[("hale", 1, 3)], None), "does not roll"),
        (magic(&[("sharp", 9, 1)], None), "no tier"),
        (magic(&[("sharp", 1, 5)], None), "rolls 1 to 1"),
        (
            magic(&[("sharp", 1, 1), ("glinting", 1, 1)], None),
            "at most 1 prefixes",
        ),
        (
            magic(
                &[("sharp", 1, 1), ("glinting", 1, 1), ("of_precision", 1, 1)],
                Some("X"),
            ),
            "two affixes of category `accuracy`",
        ),
        (
            magic(&[("sharp", 1, 1), ("heavy", 1, 2)], Some("X")),
            "two affixes of category `physical_damage`",
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
            "Damage 1d9; criticals on 20–24, ×2 damage",
            "Slashing or piercing damage",
            "To hit +0, damage 1d9",
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
    // Its numbers are its base ones until it is identified, then its affixes count.
    assert!(unknown.iter().any(|l| l == "To hit +0, damage 1d9"));
    sword.identify();
    let known = lines(&sword, &c);
    assert!(known.iter().any(|l| l.text == "To hit +0, damage 1d9+2"));
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
            "To hit +0, damage 1d8+2, range 15",
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
            "Dv +2, Pv 3",
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

#[test]
fn spell_scrolls_drop_by_tier() {
    let c = content();
    // The tier of the spell a stack's scroll teaches, if it is one.
    let tier = |s: &Stack| {
        let reading = c.object(s.object).read.as_ref()?;
        Some(c.spell(reading.teaches()?).tier)
    };
    // Shallow: no tier-3 scroll (item level 6) yet, but cantrips.
    let shallow: Vec<u8> = (roll_many(&c, 4000, 3, LootGrade::Any).iter())
        .filter_map(tier)
        .collect();
    assert!(shallow.contains(&0) && !shallow.contains(&3), "{shallow:?}");
    // Deep: every tier, the low ones commoner.
    let deep: Vec<u8> = (roll_many(&c, 8000, 8, LootGrade::Any).iter())
        .filter_map(tier)
        .collect();
    let of = |t: u8| deep.iter().filter(|&&d| d == t).count();
    assert!(of(3) > 0, "{deep:?}");
    assert!(of(0) > of(3) && of(1) > of(3), "{deep:?}");
}

/// Fixture affixes for the system effects (docs/design/affixes-systems.md): no shipped file has
/// them yet, so the tests bring their own.
const SYSTEM_AFFIXES: &str = r#"
[[prefix]]
id = "fixture_tireless"
name = "Tireless"
effect = "resource_max"
resource = "stamina"
category = "stamina"
bases = ["ring", "body"]
tiers = [{ level = 1, min = 1, max = 5 }]

[[prefix]]
id = "fixture_mage_wrought"
name = "Mage-wrought"
effect = "spell_ease"
category = "spell_armour"
bases = ["body", "helmet", "shield"]
tiers = [{ level = 1, min = 4, max = 20 }]

[[suffix]]
id = "fixture_of_fury"
name = "of Fury"
effect = "resource_max"
resource = "rage"
category = "rage"
bases = ["ring"]
tiers = [{ level = 1, min = 2, max = 8 }]

[[suffix]]
id = "fixture_of_thrift"
name = "of Thrift"
effect = "mana_cost"
category = "mana_cost"
bases = ["ring"]
tiers = [{ level = 1, min = 4, max = 16 }]
"#;

fn system_content() -> Content {
    let mut files = crate::content::ContentFiles::builtin();
    files.insert("data/affixes/zz_fixture.toml", SYSTEM_AFFIXES);
    Content::load(&files).expect("fixture affixes are valid")
}

/// The system effects sum like the rest; a pooled one by its resource, never into another
/// pool's; Mage-wrought is the piece's own.
#[test]
fn system_effects_sum_and_pooled_ones_by_their_resource() {
    let c = system_content();
    let ring = |affixes: &[(&str, u8, i32)]| item(&c, "iron_ring", affixes, None);
    let inventory = vec![
        ring(&[("fixture_tireless", 1, 2), ("fixture_of_thrift", 1, 5)]),
        ring(&[("fixture_tireless", 1, 3), ("fixture_of_fury", 1, 4)]),
        item(&c, "chain_mail", &[("fixture_mage_wrought", 1, 7)], None),
    ];
    let mut worn = crate::equipment::Equipment::default();
    for (slot, i) in [("left_ring", 0), ("right_ring", 1), ("body", 2)] {
        worn.put(&[c.slot_id(slot).unwrap()], i);
    }
    let mods = Mods::worn(&inventory, &worn, &c);
    let pool = |id: &str| c.resource_id(id).expect(id);
    let max = |id: &str| mods.resource(Effect::ResourceMax, pool(id));
    assert_eq!(max("stamina"), 5, "two Tireless rings add");
    assert_eq!(max("rage"), 4);
    assert_eq!(max("ki"), 0);
    assert_eq!(mods.resource(Effect::ResourceRate, pool("stamina")), 0);
    assert_eq!(mods.get(Effect::ResourceMax), 0, "only by a pool");
    assert_eq!(mods.get(Effect::ManaCost), 5);
    assert_eq!(mods.get(Effect::SpellEase), 0, "local: not the wearer's");
    assert_eq!(Mods::local(&inventory[2], &c).get(Effect::SpellEase), 7);
    // In words, once identified (docs/design/affixes-systems.md §11).
    let mut known = inventory[1].clone();
    known.identify();
    let mut armour = inventory[2].clone();
    armour.identify();
    let words: Vec<String> = [known, armour]
        .iter()
        .flat_map(|s| describe::lines(s, &c))
        .map(|l| l.text)
        .collect();
    for phrase in [
        "+3 to maximum stamina",
        "+4 to maximum rage",
        "hinders casting 7 points less",
    ] {
        assert!(
            words.iter().any(|w| w.contains(phrase)),
            "{phrase} in {words:?}"
        );
    }
    // Mage-wrought rolls only on a piece that hinders casting.
    let a = c.affixes().get("fixture_mage_wrought").expect("fixture");
    let body = |id: &str| c.object(object(&c, id));
    assert!(may_roll(a, ItemKind::Body, body("chain_mail")));
    assert!(!may_roll(a, ItemKind::Body, body("leather_jerkin")));
}

/// The tier window: evenly among the tiers within four item levels below the item's, the top
/// one alone past the last tier (docs/design/affixes-systems.md §8).
#[test]
fn tiers_roll_from_a_window_below_the_item_level() {
    use crate::content::Tier;
    let ladder = |levels: &[u8]| AffixDef {
        id: "x".into(),
        name: "X".into(),
        kind: AffixKind::Prefix,
        effect: Effect::Damage,
        category: "x".into(),
        skill: None,
        damage_type: None,
        resource: None,
        bases: vec![ItemKind::Weapon],
        weight: 100,
        tiers: (levels.iter())
            .map(|&level| Tier {
                level,
                min: 1,
                max: 1,
                light: None,
            })
            .collect(),
    };
    let twelve = ladder(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    let window = |a: &AffixDef, level| loot::tier_window(a, level);
    assert_eq!(window(&twelve, 1), 0..1, "level 1: tier 1");
    assert_eq!(window(&twelve, 3), 0..3, "level 3: tiers 1 to 3");
    assert_eq!(window(&twelve, 8), 4..8, "level 8: tiers 5 to 8");
    assert_eq!(window(&twelve, 12), 8..12, "level 12: tiers 9 to 12");
    assert_eq!(window(&twelve, 13), 11..12, "past the last: the top alone");
    let old = ladder(&[1, 4, 7]);
    assert_eq!(window(&old, 1), 0..1);
    assert_eq!(window(&old, 4), 0..2);
    assert_eq!(window(&old, 7), 1..3);
    assert_eq!(window(&old, 12), 2..3);
    let gap = ladder(&[3, 12]);
    assert_eq!(window(&gap, 2), 0..0, "none found");
    assert_eq!(window(&gap, 9), 0..1, "a gap: the top found");
    // Loot rolls within it.
    let c = content();
    for stack in roll_many(&c, 600, 12, LootGrade::Rare) {
        for r in stack.magic().expect("rare").rolled(&c) {
            let w = loot::tier_window(r.def, 12);
            let tier = usize::from(r.tier - 1);
            assert!(w.contains(&tier), "{} tier {}", r.def.id, r.tier);
        }
    }
}
