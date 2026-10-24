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

use crate::color::Rgb;
use crate::content::{Content, MaterialDef, ObjectDef, Part};

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
