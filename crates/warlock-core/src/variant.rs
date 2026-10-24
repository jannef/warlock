//! Items made of another material than their base item (spec §7 "Materials", #193, #194): "a
//! bronze long sword", "a duralumin chain mail", "a dragonbone quarterstaff".
//!
//! A variant is an item whose main part is of another material than its base item's: an
//! instance part ([`crate::item_parts`], saved by id with the stack), drawn only from its base
//! item's construction list ([`crate::content::ObjectDef::made_of`]). Against its base item's
//! own material it is:
//!
//! - **heavier or lighter**: each part's share of the weight scales by its material's `weight`
//!   over the base part's ([`weight`]);
//! - **better or worse to strike with**: to hit and damage change by the difference of the main
//!   parts' weapon modifiers ([`weapon`]), on whatever attacks with it or is shot;
//! - **more or less protective**: armour keeps its Protection × the main parts' `protection`
//!   ratio, rounded ([`protection`]);
//! - **dearer or cheaper**: by its parts' shares and values, as every item of other parts
//!   ([`crate::content::ObjectDef::value_with`]);
//! - drawn in its main material's `color` ([`color`]).
//!
//! Jewellery (rings, amulets) changes only in worth and colour. Materials are never hidden:
//! everything here shows at once, though the to-hit stays one of the numbers the game keeps to
//! itself (the wiki states it).
//!
//! **Where variants come from** ([`draw`]): loot draws its item's material by the materials'
//! `rarity` at the item level, and what a creature drops (its gear) by their `gear` at the
//! creature's level, both from the base item's construction list, its own material among them
//! ([`weights`]); a material worse in a fight than the base item's own but dearer weighs at most
//! [`VERY_RARE`]. Hand-made levels place what their files name.

use crate::color::Rgb;
use crate::content::{Content, MaterialDef, MaterialId, ObjectDef, Part, VERY_RARE};
use crate::item_parts::ItemPart;
use crate::magic::loot::pick_weighted;
use crate::rng::Rng;

/// The base item's main material and the one `parts` has instead, if it differs.
fn main<'c>(
    def: &ObjectDef,
    parts: &[Part],
    content: &'c Content,
) -> Option<(&'c MaterialDef, &'c MaterialDef)> {
    let made = parts.first()?.material;
    let base = def.material();
    (made != base).then(|| (content.material(base), content.material(made)))
}

/// What one of `def` made of `parts` weighs, in grams: its weight with each of its base parts'
/// share scaled by the material it is made of over the base part's (rounded, at least 1); parts
/// it has besides its base item's (a gem it was set with) weigh nothing. Jewellery weighs what
/// its base item does.
pub fn weight(def: &ObjectDef, parts: &[Part], content: &Content) -> u32 {
    if def.is_jewellery() || parts == def.parts.as_slice() {
        return def.weight;
    }
    let weight_of = |p: &Part| u64::from(content.material(p.material).weight);
    // Each term in thousandths of a percent of the base weight.
    let scaled: u64 = (def.parts.iter().zip(parts))
        .map(|(base, made)| u64::from(base.share) * 1000 * weight_of(made) / weight_of(base))
        .sum();
    let grams = (u64::from(def.weight) * scaled + 50_000) / 100_000;
    u32::try_from(grams.max(1)).unwrap_or(u32::MAX)
}

/// What the main part's material adds to hit and to damage with one of `def` made of `parts`:
/// the difference of its weapon modifiers from the base item's main material's (0 and 0 when
/// either cannot be a weapon's working part, and for jewellery).
pub fn weapon(def: &ObjectDef, parts: &[Part], content: &Content) -> (i32, i32) {
    let Some((base, made)) = main(def, parts, content) else {
        return (0, 0);
    };
    match (base.weapon(), made.weapon()) {
        (Some((hit, damage)), Some((more_hit, more_damage))) if !def.is_jewellery() => {
            (more_hit - hit, more_damage - damage)
        }
        _ => (0, 0),
    }
}

/// The Protection one of `def` made of `parts` gives while worn: its own, times the main part's
/// material's `protection` over the base item's main material's, rounded to the nearest (its
/// own when either cannot be armour).
pub fn protection(def: &ObjectDef, parts: &[Part], content: &Content) -> i32 {
    let Some((base, made)) = main(def, parts, content) else {
        return def.protection;
    };
    match (base.protection, made.protection) {
        (Some(base), Some(made)) if base > 0 => {
            let (base, made) = (i64::from(base), i64::from(made));
            let scaled = (i64::from(def.protection) * made * 2 + base) / (2 * base);
            i32::try_from(scaled).unwrap_or(def.protection)
        }
        _ => def.protection,
    }
}

/// The colour one of `def` made of `parts` is drawn in for its material: its main part's
/// material's `color` where that is not its base item's (`None`: its base item's own).
pub fn color(def: &ObjectDef, parts: &[Part], content: &Content) -> Option<Rgb> {
    main(def, parts, content).and_then(|(_, made)| made.color)
}

/// Where an item comes from, for how often it is made of each material: loot (by the materials'
/// `rarity` at the item level), or a creature's gear (by their `gear` at the creature's level,
/// which it carries).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Loot,
    Gear(u8),
}

/// How often one of `def` from `source` at item level `level` is made of each material its main
/// part may be made of, in its construction list's order (by material id): each material's
/// `rarity` (loot) or `gear` (a creature's, at its own level) weight there
/// ([`crate::content::Found::at`]), at
/// most [`VERY_RARE`] for one worse in a fight than the base item's own material but dearer
/// ([`worse_but_dearer`]).
/// Empty when it may be made only of its own material.
pub fn weights(
    content: &Content,
    def: &ObjectDef,
    source: Source,
    level: u8,
) -> Vec<(MaterialId, u32)> {
    let list = def.made_of(0);
    if list.len() < 2 {
        return Vec::new();
    }
    let (gear, level) = match source {
        Source::Loot => (false, level),
        Source::Gear(creature) => (true, creature),
    };
    let base = content.material(def.material());
    (list.iter())
        .map(|&id| {
            let made = content.material(id);
            let found = if gear { made.gear } else { made.rarity };
            let weight = found.map_or(0, |f| f.at(level));
            let capped = if id != def.material() && worse_but_dearer(def, base, made) {
                weight.min(VERY_RARE)
            } else {
                weight
            };
            (id, capped)
        })
        .collect()
}

/// The material one of `def` from `source` at item level `level` is made of, drawn from `rng`:
/// one draw among [`weights`] (none when it may be made only of its own material); another than
/// its own is its main part ([`ItemPart`]), its own (or nothing that weighs anything) `None`.
pub fn draw(
    content: &Content,
    rng: &mut Rng,
    def: &ObjectDef,
    source: Source,
    level: u8,
) -> Option<ItemPart> {
    let weights = weights(content, def, source, level);
    if weights.is_empty() {
        return None;
    }
    let made = *pick_weighted(rng, &weights)?;
    (made != def.material()).then(|| ItemPart::material(content, None, made))
}

/// Whether `made` is worse in a fight than `base` on one of `def` and dearer (#194's rule): to
/// hit or damage lower on what strikes or is shot, Protection kept lower on armour that has
/// some. Jewellery, and what neither strikes nor protects (a shield, a tool), is never worse.
pub fn worse_but_dearer(def: &ObjectDef, base: &MaterialDef, made: &MaterialDef) -> bool {
    if def.is_jewellery() || made.value <= base.value {
        return false;
    }
    let strikes = def.attack.is_some() || def.ammo.is_some();
    let weaker = match (base.weapon(), made.weapon()) {
        (Some((hit, damage)), Some((more_hit, more_damage))) => {
            more_hit < hit || more_damage < damage
        }
        _ => false,
    };
    let thinner = match (base.protection, made.protection) {
        (Some(base), Some(made)) => made < base,
        _ => false,
    };
    (strikes && weaker) || (def.protection > 0 && thinner)
}
