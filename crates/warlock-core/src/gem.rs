//! Gems in play (spec §7 "Gems", #191; the data is [`crate::content::Gems`]): what a loose gem is
//! called and looks like, and how loot finds gems and sets them in items.
//!
//! **Names**: a loose gem is called by its gem, with its size and cut unless they are the plain
//! ones, and a rough cut before the size: "a ruby", "a large fine emerald", "an uncut tiny
//! garnet", "3 medium rubies". An ornament shows in its item's name as its part does ("an
//! emerald-pommelled long sword", "a gold and sapphire amulet", [`crate::content::Part`]), and
//! with its size and cut in the item's description ([`phrase`]).
//!
//! **Loot** draws from the stream it is given (a level's `loot/<level id>`, a creature's
//! `drop/<actor id>`), so maps never change:
//!
//! - [`find`]: one draw against the gems' `chance`; on a hit, a loose gem ([`roll`]).
//! - [`ornament`]: for an item whose class is set with gems, one draw against its `chance`; on a
//!   hit, a gem (never a rough one) as the class's part, or the object's own `ornament` part.
//! - [`roll`]: one draw for the gem among the gem materials by `rarity` (by material id), one
//!   for the size and one for the cut by their weights at the item level (`find` plus `deeper`
//!   for each level above the first), so deeper finds run larger and finer.

use crate::color::Rgb;
use crate::content::{Content, Cutting, MaterialId, ObjectDef, dull};
use crate::item_parts::ItemPart;
use crate::magic::loot::pick_weighted;
use crate::object::Stack;
use crate::rng::Rng;

/// What a loose `gem` cut as `cutting` is called, singular and plural, without an article or a
/// count: "ruby"/"rubies", "large fine emerald", "uncut tiny garnet".
pub fn names(content: &Content, gem: MaterialId, cutting: Cutting) -> (String, String) {
    let material = content.material(gem);
    let plural = (material.gem.as_ref())
        .and_then(|g| g.plural.clone())
        .unwrap_or_else(|| content.plural(&material.name));
    let words = |noun: &str| {
        let Some(gems) = content.gems() else {
            return noun.to_string();
        };
        let plain = gems.plain();
        let cut = gems.cut(cutting.cut);
        let size = (cutting.size != plain.size).then(|| gems.size(cutting.size).name.as_str());
        let cut_word = (cutting.cut != plain.cut).then_some(cut.name.as_str());
        let order = if cut.rough {
            [cut_word, size]
        } else {
            [size, cut_word]
        };
        (order.into_iter().flatten())
            .chain([noun])
            .collect::<Vec<_>>()
            .join(" ")
    };
    (words(&material.name), words(&plural))
}

/// `gem` cut as `cutting` in words for a description, its size and cut always said: "small
/// emerald, fine cut", "tiny garnet, uncut".
pub fn phrase(content: &Content, gem: MaterialId, cutting: Cutting) -> String {
    let name = &content.material(gem).name;
    let Some(gems) = content.gems() else {
        return name.clone();
    };
    let (size, cut) = (gems.size(cutting.size), gems.cut(cutting.cut));
    if cut.rough {
        format!("{} {name}, {}", size.name, cut.name)
    } else {
        format!("{} {name}, {} cut", size.name, cut.name)
    }
}

/// The colour of `gem` cut as `cutting`: its own, duller if the cut is rough.
pub fn color(content: &Content, gem: MaterialId, cutting: Cutting) -> Option<Rgb> {
    let def = content.material(gem).gem.as_ref()?;
    let rough = content.gems().is_some_and(|g| g.cut(cutting.cut).rough);
    Some(if rough { dull(def.color) } else { def.color })
}

/// A gem at item level `level`, drawn from `rng` (the module docs give the draws): its
/// material and its size and cut; `set`: one to set in an item, never rough. `None` without
/// gems to find.
pub fn roll(
    content: &Content,
    rng: &mut Rng,
    level: u8,
    set: bool,
) -> Option<(MaterialId, Cutting)> {
    let gems = content.gems()?;
    // By material id, never by load order (docs/dev/content.md).
    let mut found: Vec<(MaterialId, u32)> = (content.materials().iter().enumerate())
        .filter_map(|(i, m)| Some((MaterialId(i as u16), m.gem.as_ref()?.rarity)))
        .collect();
    found.sort_by(|a, b| content.material(a.0).id.cmp(&content.material(b.0).id));
    let gem = *pick_weighted(rng, &found)?;
    let size = *pick_weighted(rng, &gems.size_weights(level))?;
    let cut = *pick_weighted(rng, &gems.cut_weights(level, set))?;
    Some((gem, Cutting { size, cut }))
}

/// Whether a find at item level `level` is a loose gem, by the gems' `chance` (one draw), and
/// if so the gem: a stack of one.
pub fn find(content: &Content, rng: &mut Rng, level: u8) -> Option<Stack> {
    find_with(content, rng, level, 0)
}

/// [`find`] for a finder whose gems' chance is `more` percent higher (Hoard sense, spec §8
/// "Birth sign"), never above certain: the same draws.
pub fn find_with(content: &Content, rng: &mut Rng, level: u8, more: i32) -> Option<Stack> {
    let gems = content.gems()?;
    let chance = crate::talent::TalentMods::scaled(u64::from(gems.chance), more).min(100);
    if gems.chance == 0 || !rng.chance(chance as u32) {
        return None;
    }
    let (gem, cutting) = roll(content, rng, level, false)?;
    Some(Stack::one(gems.object).with_parts(vec![ItemPart::gem(content, None, gem, cutting)]))
}

/// Whether loot sets an item of `def` at item level `level` with a gem, by its class's
/// ornament `chance` (one draw, none for a class never set), and if so the part it becomes.
pub fn ornament(content: &Content, rng: &mut Rng, def: &ObjectDef, level: u8) -> Option<ItemPart> {
    let placed = content.gems()?.ornament(def.class)?;
    if placed.chance == 0 || !rng.chance(u32::from(placed.chance)) {
        return None;
    }
    let (gem, cutting) = roll(content, rng, level, true)?;
    let part = def.ornament.unwrap_or(placed.part);
    Some(ItemPart::gem(content, Some(part), gem, cutting))
}
