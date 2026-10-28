//! The balance pass of the regraded ladder (docs/design/affixes-systems.md §13, X1): 10 000
//! magic items rolled at item levels 1, 4, 8, and 12. No affix takes more than its weight's
//! share of the picks it could have been, none is starved below it, and a deep find rolls the
//! tiers near its level. `cargo test -p warlock-core shares -- --nocapture` prints the shares.

use super::*;
use crate::rng::Rng;
use std::collections::BTreeMap;

const ITEMS: usize = 10_000;
const LEVELS: [u8; 4] = [1, 4, 8, 12];

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
