//! Pieces of a material, and breaking an item into the parts it is made of (spec §7 "Objects and
//! materials"; the design: docs/design/animated-objects.md §9.2).
//!
//! A **piece** is the one generic object [`ID`] (`data/objects/piece.toml`), made of whatever
//! material its main part carries, as the loose gem is made of its gem. It is the material stock
//! salvage gives and recipes take (docs/design/professions.md §3): 500 g of steel, named by its
//! group's form word in `data/stock.toml` ("a steel ingot", "3 hardwood billets", "a bolt of
//! wool"; [`names`]), "a piece of glass" for a group without one. Its material colours it, weighs
//! it (its `weight` % of the object's steel), and prices it (its `value` %), all through the
//! existing variant machinery ([`crate::variant`], [`crate::content::ObjectDef::value_with`]);
//! pieces of one material stack. A piece of steel, the object's own material, carries no part of
//! its own. How many pieces some grams of a material make is [`count_for`].
//!
//! [`break_up`] is what an item falls to when it breaks (an animated thing's broken component,
//! `world/creatures/animated.rs`; later sundering and fire): for each of its resolved parts, in part order,
//! a gem gives its loose gem, a material with a group pieces of it by weight, and a material
//! without one (paper, wax, flesh) nothing. Its magic is lost. No draws.

use crate::content::{Content, MaterialId, ObjectDef, Part};
use crate::item_parts::ItemPart;
use crate::object::Stack;

/// The piece object's id.
pub const ID: &str = "piece";

/// What a piece of `material` is called, singular and plural, without an article or a count: by
/// its form word in `stock.toml`, its own or its group's ("steel ingot"/"steel ingots", "fur
/// pelt", "lump of amber"/"lumps of amber"), else the object's own ("piece of glass"/"pieces of
/// glass").
pub fn names(content: &Content, def: &ObjectDef, material: MaterialId) -> (String, String) {
    let m = content.material(material);
    match content.stock().form_of(content, material) {
        Some((name, plural, false)) => {
            (format!("{} {name}", m.name), format!("{} {plural}", m.name))
        }
        Some((name, plural, true)) => (
            format!("{name} of {}", m.name),
            format!("{plural} of {}", m.name),
        ),
        None => (
            format!("{} of {}", def.name, m.name),
            format!("{} of {}", def.plural, m.name),
        ),
    }
}

/// How many pieces of `material` `grams` of it make: `max(1, round(grams / one piece's
/// grams))`, the rule salvage and recipes share, one piece's grams being 500 × its `weight` % ×
/// its group's `unit` ([`unit`]); `None` for a material no piece is made of.
pub fn count_for(content: &Content, grams: u64, material: MaterialId) -> Option<u32> {
    count_of_share(content, grams, 100, material)
}

/// [`count_for`] of `percent` % of `grams`, rounded once.
fn count_of_share(
    content: &Content,
    grams: u64,
    percent: u64,
    material: MaterialId,
) -> Option<u32> {
    let one = make(content, material, 1)?.weight(content).max(1);
    let n = (grams * percent + one * 50) / (one * 100);
    Some(u32::try_from(n.max(1)).unwrap_or(u32::MAX))
}

/// How many pieces' worth one of `stack` is: its material's group's `unit` in `stock.toml` for
/// a piece (a leather hide is eight, [`crate::content::Stock::unit_of`]), 1 for anything else.
/// [`Stack::weight`] and [`Stack::value`] multiply by it.
pub fn unit(content: &Content, def: &ObjectDef, stack: &Stack) -> u32 {
    if def.id != ID {
        return 1;
    }
    let material = stack
        .parts(content)
        .first()
        .map_or(def.material(), |p| p.material);
    content.stock().unit_of(content, material)
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
                let n = count_of_share(content, grams, u64::from(percent), material)?;
                make(content, material, n)
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
        assert_eq!(tin.describe(&content), "a tin ingot");
        assert_eq!(
            pieces(&content, "silk", 3).describe(&content),
            "3 bolts of silk"
        );
        assert_eq!(
            pieces(&content, "hardwood", 3).describe(&content),
            "3 hardwood billets"
        );
        assert_eq!(
            pieces(&content, "amber", 1).describe(&content),
            "a lump of amber"
        );
        assert_eq!(
            pieces(&content, "leather", 1).describe(&content),
            "a leather hide"
        );
        assert_eq!(
            pieces(&content, "iron", 2).describe(&content),
            "2 iron ingots"
        );
        // Steel is the object's own material: no part of its own, named all the same.
        let steel = pieces(&content, "steel", 1);
        assert!(steel.own_parts().is_none());
        assert_eq!(steel.describe(&content), "a steel ingot");
        assert_eq!((steel.weight(&content), steel.value(&content)), (500, 1));
        // Tin weighs 93 % of steel and is worth 40 %: nothing, a coin rounded.
        assert_eq!((tin.weight(&content), tin.value(&content)), (465, 0));
        // Gold: 246 % and 2000 %, real coin.
        let gold = pieces(&content, "gold", 1);
        assert_eq!((gold.weight(&content), gold.value(&content)), (1230, 20));
        let gold_color = content.material(material(&content, "gold")).color;
        assert_eq!(Some(gold.fg(&content)), gold_color);
        assert!(tin.is_valid_for(&content) && steel.is_valid_for(&content));
        // Paper has no group: nothing is a piece of it.
        assert!(make(&content, material(&content, "paper"), 1).is_none());
        // Stock is a class of its own.
        let class = content.object(steel.object).class;
        assert_eq!(content.item_class(class).id, "materials");
    }

    #[test]
    fn a_group_without_a_row_keeps_the_pieces_name() {
        let text = include_str!("../../../assets/data/stock.toml");
        let row = "[[group]]\ngroup = \"glass\"\nname = \"lump\"\nplural = \"lumps\"\nof = true\n";
        let without = text.replace(row, "");
        assert_ne!(without, text);
        let mut files = crate::content::ContentFiles::builtin();
        files.insert("data/stock.toml", &without);
        let content = Content::load(&files).expect("content");
        assert_eq!(
            pieces(&content, "glass", 1).describe(&content),
            "a piece of glass"
        );
        assert_eq!(
            pieces(&content, "glass", 2).describe(&content),
            "2 pieces of glass"
        );
        // Without the file at all, every piece is a piece (and no recipe or rig takes stock).
        files.remove("data/stock.toml");
        files.remove("data/rigs.toml");
        let made: Vec<String> = (files.table("recipes"))
            .filter(|(_, text)| text.contains("\nresult ="))
            .map(|(path, _)| path.to_string())
            .collect();
        for path in made {
            files.remove(&path);
        }
        let content = Content::load(&files).expect("content");
        assert_eq!(
            pieces(&content, "steel", 1).describe(&content),
            "a piece of steel"
        );
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
        assert!(!pack[0].same_goods(&pack[1]));
    }

    #[test]
    fn an_items_stock_is_counted_by_weight() {
        let content = content();
        // The design's table (docs/design/professions.md §3): a part's grams over one piece's.
        let main = |id: &str| {
            let def = content.object(content.object_id(id).expect(id));
            let part = &def.parts[0];
            let grams = u64::from(def.weight) * u64::from(part.share) / 100;
            count_for(&content, grams, part.material).expect("a piece")
        };
        for (item, count) in [
            ("knife", 1),
            ("dagger", 1),
            ("short_sword", 2),
            ("scimitar", 3),
            ("long_sword", 3),
            ("great_sword", 7),
            ("quarterstaff", 10),
            ("short_bow", 4),
            ("long_bow", 7),
        ] {
            assert_eq!(main(item), count, "{item}");
        }
        // However little, one; nothing of a material no piece is made of.
        assert_eq!(count_for(&content, 1, material(&content, "steel")), Some(1));
        assert_eq!(count_for(&content, 999, material(&content, "paper")), None);
    }

    #[test]
    fn a_groups_unit_scales_its_pieces() {
        let content = content();
        // A hide is 8 pieces' worth: 1.4 kg of leather, worth 8 times a piece's coin.
        let hide = pieces(&content, "leather", 1);
        assert_eq!((hide.weight(&content), hide.value(&content)), (1400, 8));
        assert_eq!(pieces(&content, "leather", 3).weight(&content), 4200);
        // A jerkin (3.5 kg of leather) takes 3 hides, not 20.
        let jerkin = content.object(content.object_id("leather_jerkin").expect("jerkin"));
        let grams = u64::from(jerkin.weight);
        assert_eq!(
            count_for(&content, grams, material(&content, "leather")),
            Some(3)
        );
        // A bolt of cloth is 4: 4 × 500 g × wool's weight %.
        let wool = material(&content, "wool");
        let bolt = pieces(&content, "wool", 1);
        let per = 500 * u64::from(content.material(wool).weight) / 100;
        assert_eq!(bolt.weight(&content), 4 * per);
        assert_eq!(count_for(&content, 8 * per, wool), Some(2));
        // A unit of 1 is unchanged: three steel ingots in a long sword's 1.5 kg.
        assert_eq!(
            count_for(&content, 1500, material(&content, "steel")),
            Some(3)
        );
        assert_eq!(unit(&content, content.object(hide.object), &hide), 8);
        // Anything not a piece is 1.
        let jerkin_stack = object(&content, "leather_jerkin");
        let def = content.object(jerkin_stack.object);
        assert_eq!(unit(&content, def, &jerkin_stack), 1);
    }

    #[test]
    fn sinew_comes_in_lengths_and_fur_in_pelts() {
        let content = content();
        assert_eq!(
            pieces(&content, "sinew", 1).describe(&content),
            "a length of sinew"
        );
        assert_eq!(
            pieces(&content, "sinew", 2).describe(&content),
            "2 lengths of sinew"
        );
        let sinew = pieces(&content, "sinew", 1);
        assert_eq!((sinew.weight(&content), sinew.value(&content)), (150, 0));
        let mut pack = Vec::new();
        add_to_inventory(&mut pack, pieces(&content, "sinew", 2), &content);
        add_to_inventory(&mut pack, pieces(&content, "sinew", 1), &content);
        assert_eq!(pack.len(), 1);
        assert_eq!(pack[0].count, 3);
        // Fur's own form word, the hide group's unit.
        assert_eq!(pieces(&content, "fur", 1).describe(&content), "a fur pelt");
        assert_eq!(pieces(&content, "fur", 3).describe(&content), "3 fur pelts");
        assert_eq!(pieces(&content, "fur", 1).weight(&content), 1600);
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
        // 450 g of steel is 418 g of tin; half of it in ingots of 465 g: one at least.
        assert_eq!(
            broken(&content, &dagger),
            ["a tin ingot", "a large fine ruby"]
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
        // 1.8 kg: a 90 % blade of steel (half, 810 g, in ingots of 500 g: 2), a 10 % grip of
        // leather (90 g in hides of 1.4 kg: at least 1).
        assert_eq!(
            broken(&content, &sword),
            ["2 steel ingots", "a leather hide"]
        );
        // A stack breaks whole: three swords' worth.
        let mut three = sword.clone();
        three.count = 3;
        assert_eq!(
            broken(&content, &three),
            ["5 steel ingots", "a leather hide"]
        );
    }

    #[test]
    fn paper_gives_nothing_and_a_little_gives_one_piece() {
        let content = content();
        // A book is paper with a leather trim: the leather only.
        assert_eq!(
            broken(&content, &object(&content, "book")),
            ["a leather hide"]
        );
        // A scroll is paper: nothing at all.
        assert!(broken(&content, &object(&content, "scroll_of_blessing")).is_empty());
        // However little there is, a grouped material gives one piece at least.
        let one = break_up(&object(&content, "long_sword"), &content, 0);
        assert!(one.iter().all(|s| s.count == 1) && one.len() == 2);
    }
}
