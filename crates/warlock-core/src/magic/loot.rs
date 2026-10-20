//! Loot (spec §8 "Magic items"): rolling a random item at an item level.
//!
//! [`roll`] draws, in this order, from the stream it is given (a level's `loot/<level id>` while
//! it is generated, `drop/<actor id>` for what a creature drops; spec §8 "Determinism"):
//!
//! 0. **a loose gem**: a find that may be anything (grade `any`) is a gem by the gems' `chance`
//!    (one draw, then the gem's draws: [`crate::gem::find`]), and that is all;
//! 1. **the base item**: one draw among the objects that are loot (base items, with a `kind`,
//!    and scrolls, which are never magic and so left out when the grade asks for magic or
//!    better) whose `level` is at most the item level, weighted by their `frequency`;
//! 2. **the count**: ammunition comes in stacks of [`AMMO`] (one draw); anything else is one;
//! 3. **the rarity**: one draw over the weights of `[rarity]` in `data/affixes.toml` (those below
//!    the grade asked for left out); none for what is never magic (a scroll), which is normal;
//! 4. **how many affixes**: a magic item has both a prefix and a suffix by the chance
//!    `magic.both` (one draw), else one of either (one draw); a rare one draws its count from
//!    `rare.affixes` (one draw) and how many of them are prefixes (one draw, at most three of
//!    each);
//! 5. **each affix**, prefixes first: one draw among those that may roll on the item (its kind
//!    in the affix's `bases`, something for the effect to change, a tier at the item level, and
//!    an effect the item has no affix of yet), weighted by their `weight`; one draw for the tier
//!    among those found at the item level; one draw for the value in the tier's range (none when
//!    the range is a single value). With nothing left to roll, the item has fewer affixes; with
//!    none at all it is normal;
//! 6. **a rare item's name**: one draw for its first word and one for its second, which goes
//!    with the kind of item ("Bite" for weapons, "Loop" for rings);
//! 7. **an ornament**: an item of a class set with gems is set with one by its class's chance
//!    (one draw, then the gem's draws: [`crate::gem::ornament`]).
//!
//! What is rolled is unidentified (its gems show at once: materials are not hidden).

use super::{Affix, AffixKind, Magic, Rarity, may_roll};
use crate::content::{Content, ItemKind, ObjectDef, ObjectId};
use crate::gem;
use crate::object::Stack;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// How many arrows, bolts, or stones a stack of loot ammunition holds.
pub const AMMO: RangeInclusive<i32> = 8..=20;

/// The least an item of loot may be: anything, magic or better, or rare (a vault's `loot`, what
/// creatures drop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LootGrade {
    /// By the weights of every rarity.
    #[default]
    Any,
    /// Magic or rare.
    Magic,
    Rare,
}

impl LootGrade {
    /// The lowest rarity it allows.
    pub const fn least(self) -> Rarity {
        match self {
            LootGrade::Any => Rarity::Normal,
            LootGrade::Magic => Rarity::Magic,
            LootGrade::Rare => Rarity::Rare,
        }
    }
}

/// Rolls an item of loot at item level `level`, at least of `grade`, from `rng` (the module docs
/// give the draws). `None` when content has no base item found at that level.
pub fn roll(content: &Content, rng: &mut Rng, level: u8, grade: LootGrade) -> Option<Stack> {
    let level = level.max(1);
    if grade == LootGrade::Any
        && let Some(gem) = gem::find(content, rng, level)
    {
        return Some(gem);
    }
    // The candidates by object id (the string), never by load order: a draw picks the same base
    // however the object files are laid out (docs/dev/content.md).
    let magic_only = grade != LootGrade::Any;
    let mut bases: Vec<(ObjectId, u32)> = (content.object_defs().iter().enumerate())
        .filter(|(_, o)| o.frequency > 0 && o.level <= level)
        .filter(|(_, o)| o.kind.is_some() || !magic_only)
        .map(|(i, o)| (ObjectId(i as u16), o.frequency))
        .collect();
    bases.sort_by(|a, b| content.object(a.0).id.cmp(&content.object(b.0).id));
    let object = *pick_weighted(rng, &bases)?;
    let count = if content.object(object).kind == Some(ItemKind::Ammo) {
        u32::try_from(rng.range(*AMMO.start(), *AMMO.end())).unwrap_or(1)
    } else {
        1
    };
    Some(make(content, rng, object, count, level, grade))
}

/// `count` of `object` made as loot makes its base item at item level `level`, at least of
/// `grade`: draws 3 to 7 of [`roll`] (the rarity, the affixes, a rare item's name, an ornament),
/// unidentified. What is never magic (no `kind`) draws no rarity and stays normal. A starting
/// kit's magic items are made so (spec §8 "Character creation").
pub fn make(
    content: &Content,
    rng: &mut Rng,
    object: ObjectId,
    count: u32,
    level: u8,
    grade: LootGrade,
) -> Stack {
    let level = level.max(1);
    let def = content.object(object);
    let rarity = match def.kind {
        Some(_) => roll_rarity(content, rng, grade),
        None => Rarity::Normal,
    };
    let magic = match rarity {
        Rarity::Normal => None,
        rarity => roll_magic(content, rng, def, level, rarity),
    };
    let stack = match magic {
        Some(magic) => Stack::with_magic(object, count, magic),
        None => Stack::new(object, count),
    };
    match gem::ornament(content, rng, def, level) {
        Some(part) => stack.with_parts(vec![part]),
        None => stack,
    }
}

/// A rarity by the weights in content, leaving out those below `grade`.
fn roll_rarity(content: &Content, rng: &mut Rng, grade: LootGrade) -> Rarity {
    let affixes = content.affixes();
    if affixes.is_empty() {
        return Rarity::Normal;
    }
    let weights: Vec<(Rarity, u32)> = (Rarity::ALL.into_iter())
        .filter(|r| *r >= grade.least())
        .map(|r| (r, affixes.rarity_weight(r)))
        .collect();
    match pick_weighted(rng, &weights) {
        Some(r) => *r,
        // Every allowed rarity weighs nothing: the least allowed.
        None => grade.least(),
    }
}

/// A magic or rare item of `def` at `level`: its affixes and, if rare, its name; `None` if it
/// rolled no affix.
fn roll_magic(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    rarity: Rarity,
) -> Option<Magic> {
    let affixes = content.affixes();
    let (prefixes, suffixes) = match rarity {
        Rarity::Rare => {
            let count = i32::from(*pick_weighted(rng, &affixes.rare_counts).unwrap_or(&3));
            let most = crate::content::MAX_PER_KIND as i32;
            let prefixes = rng.range((count - most).max(0), count.min(most));
            (prefixes, count - prefixes)
        }
        _ => {
            if rng.chance(affixes.magic_both) {
                (1, 1)
            } else if rng.chance(50) {
                (1, 0)
            } else {
                (0, 1)
            }
        }
    };
    let mut rolled: Vec<Affix> = Vec::new();
    for (kind, count) in [(AffixKind::Prefix, prefixes), (AffixKind::Suffix, suffixes)] {
        for _ in 0..count {
            match roll_affix(content, rng, def, level, kind, &rolled) {
                Some(affix) => rolled.push(affix),
                None => break,
            }
        }
    }
    if rolled.is_empty() {
        return None;
    }
    let name = (rarity == Rarity::Rare).then(|| rare_name(content, rng, def));
    Some(Magic {
        rarity,
        level,
        affixes: rolled,
        name,
        identified: false,
    })
}

/// One affix of `kind` for `def` at `level` that does nothing `rolled` already does.
fn roll_affix(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    level: u8,
    kind: AffixKind,
    rolled: &[Affix],
) -> Option<Affix> {
    let item = def.kind?;
    let affixes = content.affixes();
    let taken: Vec<_> = (rolled.iter())
        .filter_map(|a| affixes.get(&a.id).map(|d| d.group()))
        .collect();
    // By affix id, like the bases: not by the order the affix files load in.
    let mut candidates: Vec<(usize, u32)> = (affixes.all().iter().enumerate())
        .filter(|(_, a)| {
            a.kind == kind
                && a.weight > 0
                && a.tiers_at(level) > 0
                && !taken.contains(&a.group())
                && may_roll(a, item, def)
        })
        .map(|(i, a)| (i, a.weight))
        .collect();
    candidates.sort_by(|a, b| affixes.all()[a.0].id.cmp(&affixes.all()[b.0].id));
    let a = &affixes.all()[*pick_weighted(rng, &candidates)?];
    let tiers = a.tiers_at(level);
    let tier = rng.index(tiers);
    let t = &a.tiers[tier];
    Some(Affix {
        id: a.id.clone(),
        tier: u8::try_from(tier + 1).unwrap_or(1),
        value: rng.range(t.min, t.max),
    })
}

/// A rare item's two-word name: a first word, and a second for its kind of item.
fn rare_name(content: &Content, rng: &mut Rng, def: &ObjectDef) -> String {
    let affixes = content.affixes();
    let first = rng.pick(&affixes.first_names).cloned().unwrap_or_default();
    let seconds = def.kind.and_then(|k| affixes.second_names.get(&k));
    let second = seconds
        .and_then(|words| rng.pick(words))
        .cloned()
        .unwrap_or_default();
    format!("{first} {second}").trim().to_string()
}

/// One of `items` drawn by weight (one draw); `None` if they weigh nothing.
pub(crate) fn pick_weighted<'a, T>(rng: &mut Rng, items: &'a [(T, u32)]) -> Option<&'a T> {
    let total: u32 = items.iter().map(|(_, w)| *w).sum();
    if total == 0 {
        return None;
    }
    let mut left = rng.below(total);
    for (item, weight) in items {
        if left < *weight {
            return Some(item);
        }
        left -= weight;
    }
    None
}
