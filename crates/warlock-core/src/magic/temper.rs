//! What a forge's anvil does to an item's magic (spec §7 "Special rooms": Temper; the rules are
//! `world/temper.rs`): pure changes to a [`Magic`], drawn from the stream given.
//!
//! - **Improve**: an affix the item may roll is added, of a kind (prefix or suffix) its rarity
//!   still has room for (a plain item becomes magic, with room for one of each; a magic item one
//!   of each; a rare three of each), at the item's level, as loot rolls one
//!   ([`super::loot::roll_affix`]). When it has no room, or nothing more may roll on it, its
//!   strongest affix (the highest tier, the first of equal ones) that has a tier above goes a
//!   tier up, its value rolled anew in that tier. Nothing to add or raise: unchanged.
//! - **Flaw**: one of its affixes, drawn, is lost; losing its last, it becomes a plain item. A
//!   unique item never loses its last (it stays itself). A plain item has nothing to lose.
//!
//! Neither changes a rare item's name nor whether the item is identified.

use super::loot::roll_affix;
use super::{AffixKind, Magic, Rarity};
use crate::content::{Content, ObjectDef};
use crate::rng::Rng;

/// `magic` (`None`: a plain item) of `def` improved, at item level `level` for a plain item;
/// `None` when nothing could be added or raised.
pub fn improve(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    magic: Option<&Magic>,
    level: u8,
) -> Option<Magic> {
    def.kind?;
    let mut new = magic.cloned().unwrap_or(Magic {
        rarity: Rarity::Magic,
        level: level.max(1),
        affixes: Vec::new(),
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    });
    let most = new.rarity.most_per_kind();
    let count = |kind: AffixKind, m: &Magic| {
        (m.rolled(content).iter())
            .filter(|r| r.def.kind == kind)
            .count()
    };
    let mut kinds: Vec<AffixKind> = [AffixKind::Prefix, AffixKind::Suffix]
        .into_iter()
        .filter(|&k| count(k, &new) < most)
        .collect();
    // Which kind first, when both have room: one draw.
    if kinds.len() == 2 && rng.chance(50) {
        kinds.reverse();
    }
    for kind in kinds {
        if let Some(affix) = roll_affix(content, rng, def, new.level, kind, &new.affixes) {
            new.affixes.push(affix);
            return Some(new);
        }
    }
    raise(content, rng, magic?.clone())
}

/// `magic` with its strongest affix that has a tier above a tier up; `None` with none.
fn raise(content: &Content, rng: &mut Rng, mut magic: Magic) -> Option<Magic> {
    let affixes = content.affixes();
    let mut order: Vec<usize> = (0..magic.affixes.len()).collect();
    // The highest tier first; the first of equal ones.
    order.sort_by_key(|&i| std::cmp::Reverse(magic.affixes[i].tier));
    let (i, tier) = order.into_iter().find_map(|i| {
        let affix = &magic.affixes[i];
        let def = affixes.get(&affix.id)?;
        let next = affix.tier.checked_add(1)?;
        def.tier(next).map(|t| (i, (next, t.clone())))
    })?;
    let (next, t) = tier;
    let affix = &mut magic.affixes[i];
    affix.tier = next;
    affix.value = rng.range(t.min, t.max);
    magic.level = magic.level.max(t.level);
    Some(magic)
}

/// `magic` with one affix lost (drawn): `Some(None)` when that was its last, so the item is
/// plain; `None` when nothing is lost (no magic, or a unique's last affix).
pub fn flaw(rng: &mut Rng, magic: Option<&Magic>) -> Option<Option<Magic>> {
    let magic = magic?;
    if magic.affixes.is_empty() || magic.unique.is_some() && magic.affixes.len() == 1 {
        return None;
    }
    let mut new = magic.clone();
    let i = rng.index(new.affixes.len());
    new.affixes.remove(i);
    Some((!new.affixes.is_empty()).then_some(new))
}
