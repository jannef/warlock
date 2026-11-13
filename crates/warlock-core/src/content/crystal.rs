//! Magic crystals (`assets/data/crystals.toml`) and Enchanting's numbers
//! (`assets/data/enchanting.toml`): docs/design/enchanting.md §5, §9, §10; docs/dev/enchanting.md.
//!
//! Six colours, in file order, each a crystal object, the creature tags whose nature it is, and
//! the affix categories (families) an enchanter may roll with it. The order is the rule: a
//! creature's colour is the first whose tags it carries ([`Crystals::colour_of`]), an affix's
//! the first whose families hold its category ([`Crystals::colour_of_affix`]).
//!
//! The loader refuses: no colour, a colour twice, an unknown or non-stacking object, an object
//! two colours share, a malformed tag, **a category of some affix in no colour** (it could never be
//! enchanted), a `[[family]]` word missing for a category or given twice, an unknown rare
//! material, a chance above 100, and no mana or no marks for `[consume]`.
//!
//! A family that is no affix's category, and a tag no creature carries, are not refused: tests
//! load content with affixes or campaigns left out. `crystal_tests.rs` holds the built-in table
//! to both.
//!
//! Both files are optional: without `crystals.toml` no crystal is caught or consumed; without
//! `enchanting.toml` salvage makes no affix checks.

use super::creature::CreatureId;
use super::{Content, ContentError, ObjectId};
use crate::magic::Affix;
use serde::Deserialize;

/// One colour of crystal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrystalColour {
    pub id: String,
    /// The colour in words ("red").
    pub name: String,
    /// The crystal object of the colour.
    pub object: ObjectId,
    /// What the colour is, in words ("the blood and the fire").
    pub nature: String,
    /// The creature tags whose nature it is, in file order.
    pub tags: Vec<String>,
    /// The affix categories an enchanter may roll with it, in file order.
    pub families: Vec<String>,
}

/// The rare materials (docs/design/enchanting.md §9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RareMaterials {
    /// The two objects, at even odds where one is given.
    pub materials: Vec<ObjectId>,
    /// Soulglass on a creature of this level or more may give one.
    pub level: u8,
    /// The percent chance it does.
    pub chance: u8,
    /// The one a dragon always gives.
    pub dragon: ObjectId,
    /// Salvaging a rare of this item level or more, identified or not, may give one.
    pub salvage_level: u8,
    /// The percent chance it does.
    pub salvage_chance: u8,
}

/// The crystals' table (`data/crystals.toml`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crystals {
    /// The colours, in file order.
    pub colours: Vec<CrystalColour>,
    /// Each affix category with the words a seal names it by, in file order.
    pub families: Vec<(String, String)>,
    /// The levels from each of which a creature leaves Soulglass one more crystal.
    pub yield_levels: Vec<u8>,
    pub rare: RareMaterials,
    /// The mana a crystal consumed gives.
    pub consume_mana: i32,
    /// The Willpower training marks a crystal consumed gives (`StatUse::Consumed`'s count).
    pub consume_marks: u16,
}

/// Enchanting's numbers (`data/enchanting.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnchantingRules {
    /// What a salvaged affix's check must reach, before its tier.
    pub salvage_difficulty: i32,
    /// The tiers that add one to that check's difficulty.
    pub salvage_tier_step: u8,
    /// The item levels of an enchanting roll that add one to its difficulty (the recipes).
    pub level_step: u8,
    /// The time a seal adds to a recipe, in seconds.
    pub seal_seconds: u32,
}

impl EnchantingRules {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if rules.salvage_tier_step == 0 || rules.level_step == 0 {
            return Err(err(
                "`salvage_tier_step` and `level_step` must be at least 1".into(),
            ));
        }
        Ok(rules)
    }

    /// What the salvage check of `affix` must reach: the difficulty plus a point per
    /// `salvage_tier_step` tiers.
    pub fn salvage_against(&self, affix: &Affix) -> i32 {
        self.salvage_difficulty + i32::from(affix.tier / self.salvage_tier_step)
    }
}

impl Crystals {
    /// The colour with id `id`.
    pub fn colour(&self, id: &str) -> Option<&CrystalColour> {
        self.colours.iter().find(|c| c.id == id)
    }

    /// The colour whose crystal object is `object`; `None` for anything that is no crystal (a
    /// rare material neither).
    pub fn colour_of_object(&self, object: ObjectId) -> Option<&CrystalColour> {
        self.colours.iter().find(|c| c.object == object)
    }

    /// A creature type's colour: the first colour one of whose tags it carries; `None` when it
    /// carries none of them.
    pub fn colour_of(&self, content: &Content, creature: CreatureId) -> Option<&CrystalColour> {
        let tags = &content.creature(creature).tags;
        (self.colours.iter()).find(|c| c.tags.iter().any(|t| tags.contains(t)))
    }

    /// The colour of an affix's category: the first colour whose families hold it.
    pub fn colour_of_category(&self, category: &str) -> Option<&CrystalColour> {
        (self.colours.iter()).find(|c| c.families.iter().any(|f| f == category))
    }

    /// An affix's colour (by its id in `data/affixes/`): its category's.
    pub fn colour_of_affix(&self, content: &Content, affix: &str) -> Option<&CrystalColour> {
        let def = content.affixes().get(affix)?;
        self.colour_of_category(&def.category)
    }

    /// The words a seal names `category` by ("fire").
    pub fn family_word(&self, category: &str) -> Option<&str> {
        (self.families.iter())
            .find(|(c, _)| c == category)
            .map(|(_, w)| w.as_str())
    }

    /// How many crystals a creature of `level` leaves to Soulglass: one, and one more from each
    /// of the yield levels it reaches.
    pub fn yield_at(&self, level: u8) -> u32 {
        1 + self.yield_levels.iter().filter(|&&l| level >= l).count() as u32
    }

    /// Parses `data/crystals.toml` against the loaded objects, creatures, and affixes (module
    /// docs: the refusals).
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let spec: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let object = |id: &str| -> Result<ObjectId, ContentError> {
            content
                .object_id(id)
                .ok_or_else(|| err(format!("unknown object `{id}`")))
        };
        if spec.colour.is_empty() {
            return Err(err("at least one [[colour]]".into()));
        }
        let categories: Vec<&str> = (content.affixes().all().iter())
            .map(|a| a.category.as_str())
            .collect();
        let mut colours: Vec<CrystalColour> = Vec::with_capacity(spec.colour.len());
        for c in spec.colour {
            if colours.iter().any(|o| o.id == c.id) {
                return Err(err(format!("colour `{}` twice", c.id)));
            }
            let id = object(&c.object)?;
            if !content.object(id).stackable {
                return Err(err(format!("colour `{}`: `{}` must stack", c.id, c.object)));
            }
            if colours.iter().any(|o| o.object == id) {
                return Err(err(format!(
                    "colour `{}`: `{}` is another's",
                    c.id, c.object
                )));
            }
            if let Some(m) = c.tags.iter().find_map(|t| super::tag::check(t).err()) {
                return Err(err(format!("colour `{}`: {m}", c.id)));
            }
            colours.push(CrystalColour {
                id: c.id,
                name: c.name,
                object: id,
                nature: c.nature,
                tags: c.tags,
                families: c.families,
            });
        }
        if let Some(lost) = (categories.iter())
            .find(|cat| !colours.iter().any(|c| c.families.iter().any(|f| f == *cat)))
        {
            return Err(err(format!(
                "affix category `{lost}` is in no colour: it could never be enchanted"
            )));
        }
        let mut families: Vec<(String, String)> = Vec::with_capacity(spec.family.len());
        for f in spec.family {
            if families.iter().any(|(c, _)| *c == f.category) {
                return Err(err(format!("[[family]] `{}` twice", f.category)));
            }
            families.push((f.category, f.word));
        }
        if let Some(lost) = (categories.iter()).find(|cat| !families.iter().any(|(c, _)| c == *cat))
        {
            return Err(err(format!(
                "affix category `{lost}` has no [[family]] word"
            )));
        }
        let r = spec.rare;
        if r.chance > 100 || r.salvage_chance > 100 {
            return Err(err("[rare] chances are percents, 0 to 100".into()));
        }
        if r.materials.is_empty() {
            return Err(err("[rare] needs at least one material".into()));
        }
        let materials = (r.materials.iter())
            .map(|m| object(m))
            .collect::<Result<Vec<_>, _>>()?;
        if materials
            .iter()
            .any(|&m| colours.iter().any(|c| c.object == m))
        {
            return Err(err("[rare] a rare material is no crystal".into()));
        }
        let dragon = object(&r.dragon)?;
        if spec.consume.mana < 1 {
            return Err(err("[consume] `mana` must be at least 1".into()));
        }
        if spec.consume.marks < 1 {
            return Err(err("[consume] `marks` must be at least 1".into()));
        }
        Ok(Crystals {
            colours,
            families,
            yield_levels: spec.r#yield.levels,
            rare: RareMaterials {
                materials,
                level: r.level,
                chance: r.chance,
                dragon,
                salvage_level: r.salvage_level,
                salvage_chance: r.salvage_chance,
            },
            consume_mana: spec.consume.mana,
            consume_marks: spec.consume.marks,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    colour: Vec<ColourSpec>,
    #[serde(default)]
    family: Vec<FamilySpec>,
    r#yield: YieldSpec,
    rare: RareSpec,
    consume: ConsumeSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColourSpec {
    id: String,
    name: String,
    object: String,
    nature: String,
    tags: Vec<String>,
    families: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilySpec {
    category: String,
    word: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct YieldSpec {
    levels: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RareSpec {
    materials: Vec<String>,
    level: u8,
    chance: u8,
    dragon: String,
    salvage_level: u8,
    salvage_chance: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsumeSpec {
    mana: i32,
    marks: u16,
}

#[cfg(test)]
#[path = "crystal_tests.rs"]
mod tests;
