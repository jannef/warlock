//! Gems (spec §7 "Gems", #191): ten gems from smokestone to diamond, each with a size and a cut.
//!
//! A gem is a material of its own (`assets/data/materials/emerald.toml`) with a `gem` table: its
//! colour, what a plain one is worth in gold, and how often loot finds it ([`GemDef`]). What
//! every gem shares lives in `assets/data/gems.toml` ([`Gems`]): the object a loose gem is, the
//! chance that a find is one, the sizes and cuts (ordered, each a value multiplier; sizes weigh
//! something, and both shift up with depth in loot), and where items of each class are set with a
//! gem (an ornament: a part of the item, [`Ornament`]).
//!
//! A gem's worth is its `value` × its size's × its cut's multiplier ([`Gems::value`]); the plain
//! size and cut (`plain = true`, ×1 both) are what names leave unsaid ("a ruby", "a large fine
//! emerald") and what a placed gem is unless it says otherwise. A rough cut (`rough = true`:
//! uncut) is named before the size ("an uncut tiny garnet"), drawn duller, and never set in an
//! item.
//!
//! Without `data/gems.toml` there are no gems to find or set, whatever the materials say.

use super::ContentError;
use super::item_class::{ItemClassDef, ItemClassId};
use super::object::{MAX_VALUE, ObjectDef, ObjectId};
use super::part::{PartDef, PartId};
use crate::color::Rgb;
use serde::Deserialize;
use std::collections::BTreeSet;

/// The most a plain gem may be worth, in gold coins.
pub const MAX_GEM_VALUE: u32 = 100_000;
/// The most a size or cut may multiply a gem's value by, in percent (×100).
pub const MAX_MULTIPLIER: u32 = 10_000;
/// The most a gem of one size may weigh, in grams.
pub const MAX_GEM_WEIGHT: u32 = 1000;
/// The most sizes, and the most cuts, there may be.
pub const MAX_GRADES: usize = 16;
/// How much of its colour's saturation a rough (uncut) gem shows, in percent.
pub const ROUGH_SATURATION: u32 = 50;

/// What makes a material a gem: `gem = { color = "#2ea05a", value = 300, rarity = 5 }` in its
/// file (spec §7 "Gems").
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GemDef {
    /// Its colour: a loose gem's glyph, and the glyph of an item that takes its gem's colour (a
    /// ring set with it).
    pub color: Rgb,
    /// What a plain one (of the plain size and cut) is worth, in gold coins: 1 to
    /// [`MAX_GEM_VALUE`].
    pub value: u32,
    /// How often loot finds it among the gems: a weight (0: never found, but it may be placed).
    pub rarity: u32,
    /// "rubies"; the material's name plus `s` if absent.
    pub plural: Option<String>,
}

impl GemDef {
    /// Checks what a material's `gem` table cannot say on its own.
    pub(super) fn check(&self, material: &str) -> Result<(), String> {
        if !(1..=MAX_GEM_VALUE).contains(&self.value) {
            return Err(format!(
                "material `{material}`: a gem's `value` is in gold, 1 to {MAX_GEM_VALUE}"
            ));
        }
        if self.plural.as_deref().is_some_and(|p| p.trim().is_empty()) {
            return Err(format!(
                "material `{material}`: a gem's `plural` must not be empty"
            ));
        }
        Ok(())
    }
}

/// Index of a gem size in [`Gems::sizes`], smallest first. Stacks save sizes by id, so sizes
/// are not in the save fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GemSizeId(pub u8);

/// Index of a cut in [`Gems::cuts`], roughest first. Stacks save cuts by id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CutId(pub u8);

/// A gem's size and cut: what a loose gem or an ornament is besides its gem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cutting {
    pub size: GemSizeId,
    pub cut: CutId,
}

/// A gem size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GemSize {
    pub id: String,
    /// As names write it: "large".
    pub name: String,
    /// What it multiplies a gem's value by, in percent.
    pub value: u32,
    /// What a loose gem of this size weighs, in grams.
    pub weight: u32,
    /// Its weight in loot at item level 1, and what each item level above adds.
    pub find: u32,
    pub deeper: u32,
}

/// A cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cut {
    pub id: String,
    /// As names write it: "fine", "uncut".
    pub name: String,
    /// What it multiplies a gem's value by, in percent.
    pub value: u32,
    /// Its weight in loot at item level 1, and what each item level above adds.
    pub find: u32,
    pub deeper: u32,
    /// Not cut at all: named before the size ("an uncut tiny garnet"), drawn duller, never set
    /// in an item.
    pub rough: bool,
}

/// Where the items of one class are set with a gem when loot ornaments them: the part it
/// becomes, and how often.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ornament {
    pub class: ItemClassId,
    /// The part the gem is set as (`pommel`, `setting`), unless the object names its own
    /// (`ornament` in its file).
    pub part: PartId,
    /// Percent of the class's loot set with a gem.
    pub chance: u8,
    /// The item takes its gem's colour (a ring).
    pub colour: bool,
}

/// What every gem shares (`data/gems.toml`).
#[derive(Clone, Debug)]
pub struct Gems {
    /// The object a loose gem is: what it is made of is its gem, with a size and cut.
    pub object: ObjectId,
    /// Percent of finds (floor loot, `any` vault loot, creature drops) that are a loose gem.
    pub chance: u8,
    sizes: Vec<GemSize>,
    cuts: Vec<Cut>,
    plain: Cutting,
    ornaments: Vec<Ornament>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GemsFile {
    object: String,
    chance: u8,
    size: Vec<SizeSpec>,
    cut: Vec<CutSpec>,
    #[serde(default)]
    ornament: Vec<OrnamentSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SizeSpec {
    id: String,
    name: String,
    value: u32,
    weight: u32,
    find: u32,
    #[serde(default)]
    deeper: u32,
    #[serde(default)]
    plain: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CutSpec {
    id: String,
    name: String,
    value: u32,
    find: u32,
    #[serde(default)]
    deeper: u32,
    #[serde(default)]
    plain: bool,
    #[serde(default)]
    rough: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrnamentSpec {
    class: String,
    part: String,
    chance: u8,
    #[serde(default)]
    colour: bool,
}

/// What `data/gems.toml` is checked against: the objects, item classes, and part names loaded
/// before it.
pub(super) struct Known<'a> {
    pub object_id: &'a dyn Fn(&str) -> Option<ObjectId>,
    pub objects: &'a [ObjectDef],
    pub classes: &'a [ItemClassDef],
    pub parts: &'a [PartDef],
}

/// Checks a size's or cut's common fields; `what` names it in errors.
fn check_grade(what: &str, id: &str, name: &str, value: u32) -> Result<(), String> {
    if id.trim().is_empty() || name.trim().is_empty() {
        return Err(format!("{what} `{id}`: `id` and `name` must not be empty"));
    }
    if !(1..=MAX_MULTIPLIER).contains(&value) {
        return Err(format!(
            "{what} `{id}`: `value` multiplies a gem's value, in percent: 1 to {MAX_MULTIPLIER}"
        ));
    }
    Ok(())
}

/// The one entry of `plain` that is true, as an index: names leave it unsaid.
fn the_plain(what: &str, plain: impl Iterator<Item = bool>) -> Result<u8, String> {
    let marked: Vec<usize> = (plain.enumerate())
        .filter_map(|(i, p)| p.then_some(i))
        .collect();
    match marked.as_slice() {
        [one] => Ok(*one as u8),
        _ => Err(format!(
            "exactly one {what} is `plain = true`: the one names leave unsaid"
        )),
    }
}

impl Gems {
    /// Parses `data/gems.toml`.
    pub(super) fn parse(file: &str, text: &str, known: &Known<'_>) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: GemsFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let object = (known.object_id)(&parsed.object)
            .ok_or_else(|| err(format!("`object = \"{}\"`: no such object", parsed.object)))?;
        let def = &known.objects[usize::from(object.0)];
        if def.kind.is_some() || def.food.is_some() || def.slot.is_some() {
            return Err(err(format!(
                "`object = \"{}\"`: a loose gem is no magic item, food, or equipment",
                parsed.object
            )));
        }
        if parsed.chance > 100 {
            return Err(err("`chance` is a percent, 0 to 100".into()));
        }
        let counts = (parsed.size.len(), parsed.cut.len());
        if !(1..=MAX_GRADES).contains(&counts.0) || !(1..=MAX_GRADES).contains(&counts.1) {
            return Err(err(format!(
                "1 to {MAX_GRADES} sizes and 1 to {MAX_GRADES} cuts"
            )));
        }
        let mut ids = BTreeSet::new();
        let mut sizes = Vec::with_capacity(parsed.size.len());
        for s in &parsed.size {
            check_grade("size", &s.id, &s.name, s.value).map_err(err)?;
            if !(1..=MAX_GEM_WEIGHT).contains(&s.weight) {
                return Err(err(format!(
                    "size `{}`: `weight` is in grams, 1 to {MAX_GEM_WEIGHT}",
                    s.id
                )));
            }
            if !ids.insert(s.id.clone()) {
                return Err(err(format!("duplicate size id `{}`", s.id)));
            }
            sizes.push(GemSize {
                id: s.id.clone(),
                name: s.name.clone(),
                value: s.value,
                weight: s.weight,
                find: s.find,
                deeper: s.deeper,
            });
        }
        let mut ids = BTreeSet::new();
        let mut cuts = Vec::with_capacity(parsed.cut.len());
        for c in &parsed.cut {
            check_grade("cut", &c.id, &c.name, c.value).map_err(err)?;
            if c.plain && c.rough {
                return Err(err(format!("cut `{}`: the plain cut is not rough", c.id)));
            }
            if !ids.insert(c.id.clone()) {
                return Err(err(format!("duplicate cut id `{}`", c.id)));
            }
            cuts.push(Cut {
                id: c.id.clone(),
                name: c.name.clone(),
                value: c.value,
                find: c.find,
                deeper: c.deeper,
                rough: c.rough,
            });
        }
        let plain = Cutting {
            size: GemSizeId(the_plain("size", parsed.size.iter().map(|s| s.plain)).map_err(err)?),
            cut: CutId(the_plain("cut", parsed.cut.iter().map(|c| c.plain)).map_err(err)?),
        };
        let mut ornaments: Vec<Ornament> = Vec::with_capacity(parsed.ornament.len());
        for o in &parsed.ornament {
            let bad = |m: String| err(format!("ornament of `{}`: {m}", o.class));
            let class = (known.classes.iter().position(|c| c.id == o.class))
                .ok_or_else(|| bad("no such item class (data/item_classes.toml)".into()))?;
            let class = ItemClassId(class as u8);
            let part = (known.parts.iter().position(|p| p.id == o.part))
                .ok_or_else(|| bad(format!("no part `{}` (data/parts.toml)", o.part)))?;
            if o.chance > 100 {
                return Err(bad("`chance` is a percent, 0 to 100".into()));
            }
            if ornaments.iter().any(|x| x.class == class) {
                return Err(bad("one ornament per class".into()));
            }
            ornaments.push(Ornament {
                class,
                part: PartId(part as u8),
                chance: o.chance,
                colour: o.colour,
            });
        }
        for def in known.objects {
            let placed = ornaments.iter().find(|o| o.class == def.class);
            match (placed, def.ornament) {
                (None, Some(_)) => {
                    return Err(err(format!(
                        "object `{}` names its `ornament`, but its class has no `[[ornament]]` \
                         here",
                        def.id
                    )));
                }
                (Some(o), part) if def.parts[0].part == Some(part.unwrap_or(o.part)) => {
                    return Err(err(format!(
                        "object `{}`: a gem is set as one of its other parts, never as its main \
                         part (give it another `ornament`)",
                        def.id
                    )));
                }
                _ => {}
            }
        }
        Ok(Self {
            object,
            chance: parsed.chance,
            sizes,
            cuts,
            plain,
            ornaments,
        })
    }

    /// The sizes, smallest first.
    pub fn sizes(&self) -> &[GemSize] {
        &self.sizes
    }

    /// The cuts, roughest first.
    pub fn cuts(&self) -> &[Cut] {
        &self.cuts
    }

    pub fn size(&self, id: GemSizeId) -> &GemSize {
        &self.sizes[usize::from(id.0)]
    }

    pub fn cut(&self, id: CutId) -> &Cut {
        &self.cuts[usize::from(id.0)]
    }

    pub fn size_id(&self, name: &str) -> Option<GemSizeId> {
        let i = self.sizes.iter().position(|s| s.id == name)?;
        Some(GemSizeId(i as u8))
    }

    pub fn cut_id(&self, name: &str) -> Option<CutId> {
        let i = self.cuts.iter().position(|c| c.id == name)?;
        Some(CutId(i as u8))
    }

    /// The plain size and cut: what names leave unsaid, and what a placed gem is unless it says
    /// otherwise.
    pub fn plain(&self) -> Cutting {
        self.plain
    }

    /// Where the items of `class` are set with a gem, if loot ever sets them.
    pub fn ornament(&self, class: ItemClassId) -> Option<&Ornament> {
        self.ornaments.iter().find(|o| o.class == class)
    }

    /// Every class's ornament, in file order.
    pub fn ornaments(&self) -> &[Ornament] {
        &self.ornaments
    }

    /// What `gem` cut as `cutting` is worth, in gold coins: its value × its size's × its cut's
    /// multiplier, rounded to the nearest coin; at least 1, at most [`MAX_VALUE`].
    pub fn value(&self, gem: &GemDef, cutting: Cutting) -> u32 {
        let percent =
            u64::from(self.size(cutting.size).value) * u64::from(self.cut(cutting.cut).value);
        let value = (u64::from(gem.value) * percent + 5_000) / 10_000;
        u32::try_from(value.clamp(1, u64::from(MAX_VALUE))).unwrap_or(MAX_VALUE)
    }

    /// The sizes by their weight in loot at item level `level`, smallest first.
    pub fn size_weights(&self, level: u8) -> Vec<(GemSizeId, u32)> {
        let deeper = u32::from(level.max(1) - 1);
        (self.sizes.iter().enumerate())
            .map(|(i, s)| (GemSizeId(i as u8), s.find + s.deeper * deeper))
            .collect()
    }

    /// The cuts by their weight in loot at item level `level`, roughest first; `set`: only those
    /// an item may be set with (not rough).
    pub fn cut_weights(&self, level: u8, set: bool) -> Vec<(CutId, u32)> {
        let deeper = u32::from(level.max(1) - 1);
        (self.cuts.iter().enumerate())
            .filter(|(_, c)| !(set && c.rough))
            .map(|(i, c)| (CutId(i as u8), c.find + c.deeper * deeper))
            .collect()
    }
}

/// `color` as a rough gem shows it: [`ROUGH_SATURATION`] of its saturation, toward its own
/// grey (integer luma).
pub fn dull(color: Rgb) -> Rgb {
    let grey =
        (u32::from(color.r) * 299 + u32::from(color.g) * 587 + u32::from(color.b) * 114) / 1000;
    let mix = |c: u8| {
        let c = u32::from(c);
        let v = (c * ROUGH_SATURATION + grey * (100 - ROUGH_SATURATION) + 50) / 100;
        u8::try_from(v.min(255)).unwrap_or(255)
    };
    Rgb::new(mix(color.r), mix(color.g), mix(color.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rough_gems_are_duller_but_keep_their_brightness() {
        let ruby = Rgb::new(0xd3, 0x21, 0x3b);
        let dulled = dull(ruby);
        assert!(dulled.r < ruby.r && dulled.g > ruby.g && dulled.b > ruby.b);
        // A grey stays as it is.
        let grey = Rgb::new(120, 120, 120);
        assert_eq!(dull(grey), grey);
    }
}
