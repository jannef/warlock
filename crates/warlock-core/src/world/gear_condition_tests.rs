//! The condition creature gear spawns in (`world/gear_condition.rs`, #1193): the shares by tribe
//! over many creatures, the kind by the material, protected items untouched, the loader's checks,
//! and a damaged weapon in a creature's hand; deeper finds' make (#1436).

use super::gear_condition::{apply, conditioned, kind, quest_objects, stream};
use super::*;
use crate::content::{ContentFiles, CreatureId, ItemHarm, PER_MILLE};
use crate::gear;
use crate::item_parts::ItemPart;
use crate::kit::Pack;
use crate::magic::Worked;
use crate::magic::make::make;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeSet;
use std::sync::{Arc, LazyLock};

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("built-in")));

fn creature(id: &str) -> CreatureId {
    CONTENT.creature_id(id).expect(id)
}

/// A long sword (or `object`) of `material`.
fn of(object: &str, material: &str) -> Stack {
    let m = CONTENT.material_id(material).expect(material);
    Stack::one(CONTENT.object_id(object).expect(object))
        .with_parts(vec![ItemPart::material(&CONTENT, None, m)])
}

/// Over many creatures of type `id`: the pieces that could take a condition, and of those the
/// damaged and the improved by count (Fine, Marvellous, Masterwork).
fn shares(id: &str, seeds: u64) -> (u32, u32, [u32; 3]) {
    let quest = quest_objects(&CONTENT);
    let (mut could, mut damaged, mut improved) = (0, 0, [0; 3]);
    for seed in 0..seeds {
        let mut pack = gear::roll(&CONTENT, &mut Rng::gear(seed, "test"), creature(id), 10);
        let may: Vec<bool> = (pack.items.iter())
            .map(|s| conditioned(&CONTENT, &quest, s))
            .collect();
        apply(
            &CONTENT,
            &mut stream(seed, "test"),
            (creature(id), 1),
            &mut pack,
        );
        for (stack, may) in pack.items.iter().zip(may) {
            could += u32::from(may);
            let Some(w) = stack.worked() else {
                continue;
            };
            assert!(
                may,
                "only what may take one: {}",
                CONTENT.object(stack.object).id
            );
            if w.damaged {
                damaged += 1;
            } else {
                improved[w.improved.len() - 1] += 1;
            }
        }
    }
    (could, damaged, improved)
}

/// Whether `seen` of `n` is within four standard deviations of `per_mille`.
fn near(seen: u32, n: u32, per_mille: u16) -> bool {
    let p = f64::from(per_mille) / f64::from(PER_MILLE);
    let expected = p * f64::from(n);
    let sd = (f64::from(n) * p * (1.0 - p)).sqrt().max(1.0);
    (f64::from(seen) - expected).abs() <= 4.0 * sd
}

#[test]
fn the_shares_follow_the_tribe_over_many_creatures() {
    let rules = CONTENT.gear_condition();
    for id in [
        "skeleton_warrior",
        "kobold",
        "duergar_warrior",
        "hobgoblin",
        "footpad",
    ] {
        let odds = rules.odds(&CONTENT.creature(creature(id)).tags);
        let (n, damaged, improved) = shares(id, 1500);
        assert!(n > 500, "{id}: enough pieces ({n})");
        assert!(
            near(damaged, n, odds.damaged),
            "{id}: {damaged} of {n} damaged"
        );
        let bands = [odds.fine, odds.marvellous, odds.masterwork];
        for (seen, band) in improved.into_iter().zip(bands) {
            assert!(near(seen, n, band), "{id}: {improved:?} of {n} improved");
        }
    }
    // The tribes differ as the issue asks: the dead's almost always damaged, scavengers' often,
    // disciplined orders' seldom and more often Fine than the rest.
    let odds = |id: &str| rules.odds(&CONTENT.creature(creature(id)).tags);
    let (dead, kobold, duergar, plain) = (
        odds("skeleton"),
        odds("kobold"),
        odds("duergar_warrior"),
        odds("footpad"),
    );
    assert!(dead.damaged >= 900);
    assert!(kobold.damaged > plain.damaged && plain.damaged > duergar.damaged);
    assert!(duergar.fine > plain.fine && plain.fine > kobold.fine);
    assert!((250..=350).contains(&plain.damaged), "the baseline");
    assert_eq!(
        odds("hobgoblin"),
        rules.tribes[2].odds,
        "hobgoblin before goblin"
    );
}

#[test]
fn the_kind_is_one_the_material_can_suffer() {
    let kinds = [
        ("long_sword", "iron", ItemHarm::Rust),
        ("long_sword", "steel", ItemHarm::Rust),
        ("long_sword", "bronze", ItemHarm::Acid),
        ("long_sword", "silver", ItemHarm::Acid),
        ("long_sword", "mithril", ItemHarm::Acid),
        ("long_sword", "adamantium", ItemHarm::Work),
        ("quarterstaff", "wood", ItemHarm::Work),
        ("leather_jerkin", "leather", ItemHarm::Work),
    ];
    for (object, material, harm) in kinds {
        assert_eq!(
            kind(&CONTENT, &of(object, material)),
            harm,
            "{object} of {material}"
        );
    }
    // Every material: never a kind it always withstands.
    for m in CONTENT.materials() {
        let stack = of("long_sword", &m.id);
        let k = kind(&CONTENT, &stack);
        assert!(k == ItemHarm::Work || m.resists().of(k) < 100, "{}", m.id);
    }
}

/// A unique sword, one of Proof, and a quest's object never take a condition, however often
/// they are drawn for the dead (who almost always damage theirs).
#[test]
fn protected_items_are_untouched() {
    use crate::magic::{Affix, Magic, Rarity};
    let sword = CONTENT.object_id("long_sword").expect("sword");
    let magic = |unique: Option<&str>, affixes: &[&str]| Magic {
        rarity: Rarity::Rare,
        level: 6,
        affixes: (affixes.iter())
            .map(|id| Affix {
                id: (*id).into(),
                tier: 1,
                value: 1,
            })
            .collect(),
        name: unique.map(|_| "Widowmaker".into()),
        identified: false,
        carried: 0,
        unique: unique.map(String::from),
    };
    let unique = Stack::with_magic(sword, 1, magic(Some("widowmaker"), &[]));
    let proof = Stack::with_magic(sword, 1, magic(None, &["of_proof"]));
    let sharp = Stack::with_magic(sword, 1, magic(None, &["sharp"]));
    let none = BTreeSet::new();
    assert!(!conditioned(&CONTENT, &none, &unique));
    assert!(!conditioned(&CONTENT, &none, &proof));
    assert!(
        conditioned(&CONTENT, &none, &sharp),
        "another affix protects nothing"
    );
    let plain = of("long_sword", "iron");
    assert!(conditioned(&CONTENT, &none, &plain));
    assert!(
        !conditioned(&CONTENT, &BTreeSet::from([sword]), &plain),
        "a quest's"
    );
    let skeleton = creature("skeleton_warrior");
    let mut touched = 0;
    for seed in 0..100 {
        let mut pack = Pack::default();
        for stack in [&unique, &proof, &sharp] {
            pack.items.push(stack.clone());
        }
        apply(
            &CONTENT,
            &mut stream(seed, "test"),
            (skeleton, 1),
            &mut pack,
        );
        assert!(pack.items[..2].iter().all(|s| s.worked().is_none()));
        touched += usize::from(pack.items[2].worked().is_some());
    }
    assert!(
        touched > 80,
        "the unprotected one nearly always ({touched})"
    );
    // Every fixed quest's object is among the protected.
    for q in CONTENT.quests().iter().filter(|q| !q.is_rolled()) {
        if let Some((o, _)) = q.hand_in() {
            assert!(quest_objects(&CONTENT).contains(&o));
        }
    }
}

#[test]
fn adamantium_gear_is_never_found_damaged() {
    // #1444: a skeleton's gear is nearly always damaged; adamantium of it never is, and the
    // draws are the same with or without it (an iron sword after it fares the same).
    let skeleton = creature("skeleton_warrior");
    let (mut improved, mut iron_same) = (0, 0);
    for seed in 0..200 {
        let mut pack = Pack {
            items: vec![of("long_sword", "adamantium"), of("long_sword", "iron")],
            ..Pack::default()
        };
        let mut alone = Pack {
            items: vec![of("long_sword", "iron"), of("long_sword", "iron")],
            ..Pack::default()
        };
        apply(
            &CONTENT,
            &mut stream(seed, "test"),
            (skeleton, 1),
            &mut pack,
        );
        apply(
            &CONTENT,
            &mut stream(seed, "test"),
            (skeleton, 1),
            &mut alone,
        );
        let adamantium = pack.items[0].worked();
        assert!(adamantium.is_none_or(|w| !w.damaged), "seed {seed}");
        improved += usize::from(adamantium.is_some());
        iron_same += usize::from(pack.items[1].worked() == alone.items[1].worked());
    }
    assert_eq!(iron_same, 200, "the same draws");
    assert!(improved < 200);
}

#[test]
fn a_damaged_weapon_is_weaker_in_a_creatures_hand() {
    let w = World::new(CONTENT.clone(), "traps", None).expect("traps");
    let kobold = CONTENT.creature_id("kobold").expect("kobold");
    let actor = (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .expect("a kobold");
    let armed = |worked: Option<Worked>| {
        let mut a = actor.clone();
        let mut sword = of("long_sword", "iron");
        if let Some(worked) = worked {
            sword.set_worked(worked);
        }
        let mut pack = Pack::default();
        let at = pack.put(sword, &CONTENT);
        pack.equip(at, &CONTENT);
        a.inventory = pack.items;
        a.equipment = pack.equipment;
        a.melee(&CONTENT)
    };
    let mut rusted = Worked::default();
    rusted.damage(ItemHarm::Rust);
    let (plain, worse) = (armed(None), armed(Some(rusted)));
    assert_eq!(worse.bonus, plain.bonus - 1, "a point less damage");
    assert_eq!(worse.to_hit, plain.to_hit);
}

#[test]
fn the_loader_refuses_odds_past_1000() {
    let load = |text: &str| {
        let mut files = ContentFiles::builtin();
        files.insert("data/gear_condition.toml", text);
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    let default = "[default]\ndamaged = 300\nfine = 40\nmarvellous = 8\nmasterwork = 2\n";
    assert_eq!(load(default), Ok(()));
    let over = "[default]\ndamaged = 990\nfine = 40\nmarvellous = 8\nmasterwork = 2\n";
    assert!(load(over).expect_err("over").contains("1000"));
}

/// Without the file every piece spawns as made, and nothing is drawn.
#[test]
fn without_the_file_gear_is_as_made() {
    let mut files = ContentFiles::builtin();
    files.remove("data/gear_condition.toml");
    let content = Content::load(&files).expect("valid");
    let skeleton = content.creature_id("skeleton_warrior").expect("skeleton");
    let mut pack = gear::roll(&content, &mut Rng::gear(1, "test"), skeleton, 10);
    let made = pack.clone();
    let mut rng = stream(1, "test");
    let before = rng.clone();
    apply(&content, &mut rng, (skeleton, 10), &mut pack);
    assert_eq!((pack.items, rng), (made.items, before));
}

/// Loot's make (#1436): nothing drawn where the depth row gives nothing (item level 1) or for
/// what takes no make (a ring); deep, a long sword is often improved and never damaged.
#[test]
fn deeper_loot_is_better_made_and_shallow_loot_draws_nothing() {
    let sword = Stack::one(CONTENT.object_id("long_sword").expect("long sword"));
    let ring = (CONTENT.object_defs().iter())
        .position(|o| o.is_jewellery())
        .map(|i| Stack::one(crate::content::ObjectId(i as u16)))
        .expect("a ring");
    for (stack, level) in [(&sword, 1), (&sword, 2), (&ring, 30)] {
        let mut rng = Rng::stream(1, "make");
        let before = rng.clone();
        let mut made = stack.clone();
        make(&CONTENT, &mut rng, level, &mut made);
        assert_eq!((&made, rng), (stack, before));
    }
    let row = CONTENT.gear_condition().depth_at(20);
    assert!(row.odds().improved() > 0);
    let mut improved = [0u32; 3];
    const N: u32 = 4000;
    let mut rng = Rng::stream(1, "make");
    for _ in 0..N {
        let mut made = sword.clone();
        make(&CONTENT, &mut rng, 20, &mut made);
        if let Some(w) = made.worked() {
            assert!(!w.damaged);
            improved[w.improvements() - 1] += 1;
        }
    }
    let expected = [row.fine, row.marvellous, row.masterwork].map(|p| u32::from(p) * N / PER_MILLE);
    for (got, want) in improved.into_iter().zip(expected) {
        let slack = 4 * want.isqrt() + want / 10 + 1;
        assert!(
            got.abs_diff(want) <= slack,
            "{improved:?} against {expected:?}"
        );
    }
}

/// Creature gear adds the depth row of its level to its tribe's odds: deeper, more improved.
#[test]
fn deeper_creature_gear_is_better_made() {
    let improved = |level: u8| {
        let mut n = 0;
        for seed in 0..300 {
            let id = creature("skeleton_warrior");
            let mut pack = gear::roll(&CONTENT, &mut Rng::gear(seed, "test"), id, 10);
            apply(&CONTENT, &mut stream(seed, "test"), (id, level), &mut pack);
            n += (pack.items.iter())
                .filter(|s| s.worked().is_some_and(|w| w.improvements() > 0))
                .count();
        }
        n
    };
    let (shallow, deep) = (improved(1), improved(20));
    assert!(deep > shallow * 5 + 20, "{shallow} shallow, {deep} deep");
}
