//! The balance pass of the regraded ladder (docs/design/affixes-systems.md §13, X1): 10 000
//! magic items rolled at item levels 1, 4, 8, and 12. No affix takes more than its weight's
//! share of the picks it could have been, none is starved below it, and a deep find rolls the
//! tiers near its level. `cargo test -p warlock-core shares -- --nocapture` prints the shares.

use super::*;
use crate::rng::Rng;
use std::collections::BTreeMap;

const ITEMS: usize = 10_000;
const LEVELS: [u8; 4] = [1, 4, 8, 12];

/// The trim's levels: the shares' and two deeper (#1436: deeper finds are better made).
const TRIM_LEVELS: [u8; 6] = [1, 4, 8, 12, 16, 20];

/// Each affix's chance to be a pick of its kind on `def` at `level`: its weight over the weights
/// of every affix of that kind that may roll there (the first pick's, before any category is
/// taken).
fn chances<'c>(c: &'c Content, def: &ObjectDef, level: u8, kind: AffixKind) -> Vec<(&'c str, f64)> {
    let Some(item) = def.kind else {
        return Vec::new();
    };
    let open: Vec<&AffixDef> = (c.affixes().all().iter())
        .filter(|a| a.kind == kind && a.weight > 0 && a.tiers_at(level) > 0)
        .filter(|a| may_roll(a, item, def))
        .collect();
    let total: u32 = open.iter().map(|a| a.weight).sum();
    (open.iter())
        .map(|a| (a.id.as_str(), f64::from(a.weight) / f64::from(total.max(1))))
        .collect()
}

#[test]
fn shares_follow_the_weights_and_deep_finds_roll_deep_tiers() {
    let c = Content::builtin().expect("built-in content");
    for level in LEVELS {
        let mut rng = Rng::stream(11, "shares");
        // By affix id: (picks seen, picks its weight's share expects).
        let mut seen: BTreeMap<&str, (u32, f64)> = BTreeMap::new();
        let mut items = 0;
        while items < ITEMS {
            let Some(stack) = loot::roll(&c, &mut rng, level, LootGrade::Magic) else {
                continue;
            };
            let Some(magic) = stack.magic().filter(|m| m.rarity == Rarity::Magic) else {
                continue;
            };
            items += 1;
            let def = c.object(stack.object);
            for rolled in magic.rolled(&c) {
                for (id, p) in chances(&c, def, level, rolled.def.kind) {
                    seen.entry(id).or_default().1 += p;
                }
                seen.entry(rolled.def.id.as_str()).or_default().0 += 1;
                // The tier window: a level-12 find rolls the tiers of levels 9 to 12, or the
                // highest found there where the ladder stops short or has a gap.
                let window = loot::tier_window(rolled.def, level);
                let tier = usize::from(rolled.tier - 1);
                assert!(
                    window.contains(&tier),
                    "{} tier {}",
                    rolled.def.id,
                    rolled.tier
                );
                let top = usize::from(rolled.tier) == rolled.def.tiers_at(level);
                let found = rolled.def.tier(rolled.tier).expect("tier").level;
                assert!(
                    top || level - found < loot::TIER_WINDOW,
                    "level {level}: {} tier {} of level {found}",
                    rolled.def.id,
                    rolled.tier
                );
            }
        }
        println!("item level {level}: affix, picks, expected by weight");
        let mut bad = Vec::new();
        for (id, (got, expected)) in &seen {
            println!("  {id:<20} {got:>5} {expected:>8.1}");
            // At most its weight's share, and never starved below it: within four standard
            // deviations of chance and a tenth more (a taken category leaves the rest a little
            // more of a second pick).
            let got = f64::from(*got);
            let slack = 4.0 * expected.sqrt() + expected / 10.0;
            if (got - expected).abs() > slack {
                bad.push(format!("{id} took {got}, its weight's share {expected:.1}"));
            }
        }
        assert!(bad.is_empty(), "level {level}: {bad:#?}");
    }
}

/// The trim's effect on loot (docs/design/enchanting.md §12, E1): 4 000 rares and 4 000 magic
/// items rolled at item levels 1, 4, 8, 12, 16, and 20 as loot makes them (its make and
/// materials better deeper, #1436), before and after the trim. Prints, per level, the capacity
/// distribution, the affix counts before and after, and the share left untrimmed; fails if
/// shallow rares (levels 1 and 4) keep three affixes or more a fifth of the time or more, or deep
/// ones (16 and 20) less than a quarter of the time (half before #1444 scaled the quality's term
/// down), or if any level's share is below the one before it. `cargo test -p warlock-core trim_shares -- --nocapture` prints it.
#[test]
fn trim_shares_hold_every_find_to_its_capacity() {
    let c = Content::builtin().expect("built-in content");
    let mut classes: Vec<crate::content::ItemClassId> = (c.object_defs().iter())
        .filter(|o| o.frequency > 0 && o.kind.is_some())
        .map(|o| o.class)
        .collect();
    classes.sort();
    classes.dedup();
    const N: usize = 4_000;
    let mut rising = 0.0;
    for level in TRIM_LEVELS {
        let mut rng = Rng::stream(13, "trim shares");
        for grade in [LootGrade::Rare, LootGrade::Magic] {
            // Magic grade: magic or rare, by the weights.
            let (mut rolled, mut kept, mut room) = ([0u32; 7], [0u32; 7], [0u32; 7]);
            let mut untrimmed = 0;
            let mut items = 0;
            while items < N {
                let Some(base) = loot::base_in(&c, &mut rng, &classes, level) else {
                    continue;
                };
                let source = crate::variant::Source::Loot;
                let made = loot::make_as_unfinished(&c, &mut rng, base, 1, (level, grade), source);
                let Some(before) = made.magic().map(|m| m.affixes.len()) else {
                    continue;
                };
                items += 1;
                let (cap, was) = (capacity(&made, &c), made.rarity());
                let done = loot::finished(&c, &mut rng, made);
                let after = done.magic().map_or(0, |m| m.affixes.len());
                assert!(after <= usize::from(cap) && after >= 1);
                assert_eq!(done.rarity(), was, "the trim keeps the rarity");
                rolled[before] += 1;
                kept[after] += 1;
                room[usize::from(cap)] += 1;
                untrimmed += u32::from(before == after);
            }
            let pct = |n: u32| 100.0 * f64::from(n) / N as f64;
            let row = |a: &[u32; 7]| {
                (1..7)
                    .map(|i| format!("{i}:{:5.1}%", pct(a[i])))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            println!(
                "item level {level}, {grade:?}: untrimmed {:.1}%",
                pct(untrimmed)
            );
            println!("  capacity       {}", row(&room));
            println!("  affixes rolled {}", row(&rolled));
            println!("  affixes kept   {}", row(&kept));
            // Deeper finds are better made (#1436): shallow rares stay modest, deep ones keep
            // three affixes or more far more often, and the share never falls with depth.
            let three_or_more = pct(kept[3..].iter().sum());
            if grade == LootGrade::Rare {
                match level {
                    1 | 4 => assert!(three_or_more < 20.0, "level {level}: {three_or_more:.1}%"),
                    16 | 20 => assert!(three_or_more > 25.0, "level {level}: {three_or_more:.1}%"),
                    _ => {}
                }
                assert!(
                    three_or_more + 1.0 >= rising,
                    "level {level}: {three_or_more:.1}% after {rising:.1}%"
                );
                rising = three_or_more;
            }
        }
    }
}
