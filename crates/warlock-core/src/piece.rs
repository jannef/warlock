//! Pieces of a material, and breaking an item into the parts it is made of (spec §7 "Objects and
//! materials"; the design: docs/design/animated-objects.md §9.2).
//!
//! A **piece** is the one generic object [`ID`] (`data/objects/piece.toml`), made of whatever
//! material its main part carries, as the loose gem is made of its gem: "a piece of tin", "3
//! pieces of silk". Its material names it ([`names`]), colours it, weighs it (its `weight` % of
//! the object's steel), and prices it (its `value` %), all through the existing variant
//! machinery ([`crate::variant`], [`crate::content::ObjectDef::value_with`]); pieces of one
//! material stack. A piece of steel, the object's own material, carries no part of its own.
//!
//! [`break_up`] is what an item falls to when it breaks (an animated thing's broken component,
//! `world/animated.rs`; later sundering and fire): for each of its resolved parts, in part order,
//! a gem gives its loose gem, a material with a group pieces of it by weight, and a material
//! without one (paper, wax, flesh) nothing. Its magic is lost. No draws.

use crate::content::{Content, MaterialId, ObjectDef, Part};
use crate::item_parts::ItemPart;
use crate::object::Stack;

/// The piece object's id.
pub const ID: &str = "piece";

/// What a piece of `material` is called, singular and plural, without an article or a count:
/// "piece of tin"/"pieces of tin".
pub fn names(content: &Content, def: &ObjectDef, material: MaterialId) -> (String, String) {
    let of = &content.material(material).name;
    (
        format!("{} of {of}", def.name),
        format!("{} of {of}", def.plural),
    )
}

/// `count` pieces of `material`; `None` without the piece object or for a material it cannot be
/// made of (one without a group).
pub fn make(content: &Content, material: MaterialId, count: u32) -> Option<Stack> {
    let piece = content.object_id(ID)?;
    let def = content.object(piece);
    if !def.made_of(0).contains(&material) && def.material() != material {
        return None;
    }
    let stack = Stack::new(piece, count);
    Some(if def.material() == material {
        stack
    } else {
        stack.with_parts(vec![ItemPart::material(content, None, material)])
    })
}

/// What a part `made` of an item of `def` (in place of its base part `base`) weighs, in grams:
/// its share of the item's weight, scaled by its material over the base part's as
/// [`crate::variant::weight`] scales it (jewellery is not).
fn grams(def: &ObjectDef, base: &Part, made: &Part, content: &Content) -> u64 {
    let share = u64::from(def.weight) * u64::from(base.share);
    if def.is_jewellery() || base.material == made.material {
        return share / 100;
    }
    let weight = |m: MaterialId| u64::from(content.material(m).weight).max(1);
    share * weight(made.material) / weight(base.material) / 100
}

/// What `stack` breaks into (the module docs): a loose gem for each gem part (its size and cut,
/// a base item's gem the plain cut), one stack of pieces for each grouped material among its
/// parts, `max(1, round(grams × percent / 100 / one piece's grams))` of them (its parts of that
/// material summed, the whole stack's), nothing for the rest. In part order; no magic.
pub fn break_up(stack: &Stack, content: &Content, percent: u8) -> Vec<Stack> {
    let def = content.object(stack.object);
    let parts = stack.parts(content);
    let count = u64::from(stack.count);
    // In part order: loose gems, and each grouped material with the grams of it.
    let mut falls: Vec<Fall> = Vec::new();
    for (i, made) in parts.iter().enumerate() {
        let material = content.material(made.material);
        if material.gem.is_some() {
            let Some(gems) = content.gems() else {
                continue;
            };
            let cutting = made.gem.unwrap_or(gems.plain());
            let part = ItemPart::gem(content, None, made.material, cutting);
            let gem = Stack::new(gems.object, stack.count).with_parts(vec![part]);
            match falls.iter_mut().find_map(|f| match f {
                Fall::Gem(g) if g.same_goods(&gem) => Some(g),
                _ => None,
            }) {
                Some(same) => same.count += gem.count,
                None => falls.push(Fall::Gem(gem)),
            }
            continue;
        }
        // A part the item has besides its base item's weighs nothing (only gems are added).
        let (Some(_), Some(base)) = (&material.group, def.parts.get(i)) else {
            continue;
        };
        let grams = grams(def, base, made, content) * count;
        match falls.iter_mut().find_map(|f| match f {
            Fall::Pieces(m, sum) if *m == made.material => Some(sum),
            _ => None,
        }) {
            Some(sum) => *sum += grams,
            None => falls.push(Fall::Pieces(made.material, grams)),
        }
    }
    (falls.into_iter())
        .filter_map(|fall| match fall {
            Fall::Gem(gem) => Some(gem),
            Fall::Pieces(material, grams) => {
                let one = make(content, material, 1)?.weight(content).max(1);
                let n = (grams * u64::from(percent) + one * 50) / (one * 100);
                make(
                    content,
                    material,
                    u32::try_from(n.max(1)).unwrap_or(u32::MAX),
                )
            }
        })
        .collect()
}

/// What one part of a broken item falls as.
enum Fall {
    /// A loose gem.
    Gem(Stack),
    /// Pieces of a material, by the grams of it.
    Pieces(MaterialId, u64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Cutting;
    use crate::object::add_to_inventory;

    fn content() -> Content {
        Content::builtin().expect("built-in content")
    }

    fn material(content: &Content, id: &str) -> MaterialId {
        content.material_id(id).unwrap_or_else(|| panic!("{id}"))
    }

    fn pieces(content: &Content, of: &str, count: u32) -> Stack {
        make(content, material(content, of), count).expect("a piece")
    }

    /// What `stack` falls to, described: "2 pieces of tin", "a ruby".
    fn broken(content: &Content, stack: &Stack) -> Vec<String> {
        (break_up(stack, content, 50).iter())
            .map(|s| s.describe(content))
            .collect()
    }

    fn object(content: &Content, id: &str) -> Stack {
        Stack::one(content.object_id(id).unwrap_or_else(|| panic!("{id}")))
    }

    #[test]
    fn a_piece_is_named_weighed_and_priced_by_its_material() {
        let content = content();
        let tin = pieces(&content, "tin", 1);
        assert_eq!(tin.describe(&content), "a piece of tin");
        assert_eq!(
            pieces(&content, "silk", 3).describe(&content),
            "3 pieces of silk"
        );
        // Steel is the object's own material: no part of its own, named all the same.
        let steel = pieces(&content, "steel", 1);
        assert!(steel.own_parts().is_none());
        assert_eq!(steel.describe(&content), "a piece of steel");
        assert_eq!((steel.weight(&content), steel.value(&content)), (100, 1));
        // Tin weighs 93 % of steel and is worth 40 %: nothing, a coin rounded.
        assert_eq!((tin.weight(&content), tin.value(&content)), (93, 0));
        // Gold: 246 % and 2000 %, real coin.
        let gold = pieces(&content, "gold", 1);
        assert_eq!((gold.weight(&content), gold.value(&content)), (246, 20));
        let gold_color = content.material(material(&content, "gold")).color;
        assert_eq!(Some(gold.fg(&content)), gold_color);
        assert!(tin.is_valid_for(&content) && steel.is_valid_for(&content));
        // Paper has no group: nothing is a piece of it.
        assert!(make(&content, material(&content, "paper"), 1).is_none());
    }

    #[test]
    fn pieces_of_one_material_stack() {
        let content = content();
        let mut pack = Vec::new();
        add_to_inventory(&mut pack, pieces(&content, "tin", 2), &content);
        add_to_inventory(&mut pack, pieces(&content, "steel", 1), &content);
        add_to_inventory(&mut pack, pieces(&content, "tin", 3), &content);
        add_to_inventory(&mut pack, pieces(&content, "steel", 1), &content);
        let counts: Vec<u32> = pack.iter().map(|s| s.count).collect();
        assert_eq!(counts, [5, 2], "tin together, steel together");
        assert!(pack[0].same_goods(&pieces(&content, "tin", 1)));
    }

    #[test]
    fn a_tin_dagger_with_a_ruby_pommel_breaks_into_tin_and_its_ruby() {
        let content = content();
        let gems = content.gems().expect("gems");
        let cutting = Cutting {
            size: gems.size_id("large").expect("large"),
            cut: gems.cut_id("fine").expect("fine"),
        };
        let pommel = content.part_id("pommel").expect("pommel");
        let ruby = material(&content, "ruby");
        let dagger = object(&content, "dagger").with_parts(vec![
            ItemPart::material(&content, None, material(&content, "tin")),
            ItemPart::gem(&content, Some(pommel), ruby, cutting),
        ]);
        // 450 g of steel is 418 g of tin; half of it in pieces of 93 g: 2.
        assert_eq!(
            broken(&content, &dagger),
            ["2 pieces of tin", "a large fine ruby"]
        );
        let fell = break_up(&dagger, &content, 50);
        assert_eq!(
            fell[1].gem(&content),
            Some((ruby, cutting)),
            "size and cut kept"
        );
        assert!(fell.iter().all(|s| s.is_valid_for(&content)));
    }

    #[test]
    fn a_long_sword_breaks_into_steel_and_leather_by_weight() {
        let content = content();
        let sword = object(&content, "long_sword");
        // 1.8 kg: a 90 % blade of steel (810 g of pieces: 8), a 10 % grip of leather (90 g
        // in pieces of 35 g: 3).
        assert_eq!(
            broken(&content, &sword),
            ["8 pieces of steel", "3 pieces of leather"]
        );
        // A stack breaks whole: three swords' worth of pieces.
        let mut three = sword.clone();
        three.count = 3;
        assert_eq!(
            broken(&content, &three),
            ["24 pieces of steel", "8 pieces of leather"]
        );
    }

    #[test]
    fn paper_gives_nothing_and_a_little_gives_one_piece() {
        let content = content();
        // A book is paper with a leather trim: the leather only.
        assert_eq!(
            broken(&content, &object(&content, "book")),
            ["3 pieces of leather"]
        );
        // A scroll is paper: nothing at all.
        assert!(broken(&content, &object(&content, "scroll_of_blessing")).is_empty());
        // However little there is, a grouped material gives one piece at least.
        let one = break_up(&object(&content, "long_sword"), &content, 0);
        assert!(one.iter().all(|s| s.count == 1) && one.len() == 2);
    }
}
