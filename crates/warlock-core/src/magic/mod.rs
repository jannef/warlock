//! Magic items (spec §8 "Magic items"), after Path of Exile: base items roll prefixes and
//! suffixes (`data/affixes.toml`, [`crate::content::Affixes`]).
//!
//! - **Rarity**: a *normal* item has no affixes; a *magic* item a prefix, a suffix, or one of
//!   each (most have one); a *rare* item up to three prefixes and three suffixes (usually three
//!   or four affixes in all) and a generated two-word name ("Grim Bite").
//! - **Instances**: a magic or rare item records what it rolled ([`Magic`]): its rarity, its item
//!   level, each affix by id with its tier and rolled value, its name, and whether the player has
//!   identified it. Enchanting may reroll, add, or remove affixes later. A stack carries its
//!   [`Magic`] wherever it goes (piles, inventories, missiles in flight, saves); magic stacks
//!   join only stacks with the very same magic.
//! - **Unidentified**: magic and rare items are found unidentified. Their rarity shows ("a long
//!   sword (magic)"), their affixes do not, but their effects apply all the same (as in ADOM and
//!   Path of Exile). [`crate::World`]'s `identify` reveals them. Presentation reaches an item's
//!   affixes only through [`crate::object::Stack::known_magic`], which gives nothing for an
//!   unidentified item, so they cannot leak.
//! - **Effects** ([`mods`]) apply while the item is worn or wielded: attack effects on a weapon,
//!   a missile weapon, or ammunition change the attacks made with it (local), elsewhere every
//!   attack (global); effects on the item itself (Armour, Shield, Parry, the Dexterity it
//!   allows, its weight) are local; the rest (stats, hit points, Defence, regeneration, speed,
//!   carrying, stealth, light) global.
//! - **Loot** ([`loot`]): the item level comes from the dungeon depth; rarity is a weighted draw.

pub mod describe;
pub mod loot;
pub mod mods;
pub(crate) mod name;
pub mod numbers;
pub mod temper;

pub use crate::content::{AffixKind, Effect, ItemKind, Rarity};
pub use loot::LootGrade;
pub use mods::Mods;
pub use numbers::Numbers;

use crate::content::{AffixDef, Content, ObjectDef};
use serde::{Deserialize, Serialize};

/// One affix an item rolled: which, its tier (1 for the first), and the value rolled within the
/// tier's range (0 for a light).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Affix {
    /// The affix's id in `data/affixes.toml`.
    pub id: String,
    pub tier: u8,
    pub value: i32,
}

/// What a magic or rare item rolled (spec §8 "Magic items").
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Magic {
    /// Magic or rare (a normal item has no `Magic`).
    pub rarity: Rarity,
    /// Its item level: what tiers it could roll.
    pub level: u8,
    /// Its prefixes, then its suffixes, each in the order rolled.
    pub affixes: Vec<Affix>,
    /// A rare item's name ("Grim Bite").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the player knows its affixes.
    #[serde(default, skip_serializing_if = "is_false")]
    pub identified: bool,
    /// The seconds the player has carried it unidentified while carrying taught them (spec §8
    /// "Class abilities": Connoisseur), up to what that asks.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub carried: u32,
    /// The unique item it is (an id in `data/uniques/`; spec §8 "Magic items"), by which its
    /// power and lore are found: a rare item with its set name and affixes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique: Option<String>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// An affix an item has, resolved: its definition and what it rolled.
#[derive(Clone, Copy, Debug)]
pub struct Rolled<'a> {
    pub def: &'a AffixDef,
    pub tier: u8,
    pub value: i32,
}

impl Magic {
    /// Its affixes with their definitions, prefixes first (unknown ids, which a checked item does
    /// not have, are left out).
    pub fn rolled<'c>(&self, content: &'c Content) -> Vec<Rolled<'c>> {
        let affixes = content.affixes();
        let mut rolled: Vec<Rolled<'c>> = (self.affixes.iter())
            .filter_map(|a| {
                Some(Rolled {
                    def: affixes.get(&a.id)?,
                    tier: a.tier,
                    value: a.value,
                })
            })
            .collect();
        rolled.sort_by_key(|r| r.def.kind);
        rolled
    }

    /// Whether it is a valid item of `def` for `content` (a guard for saves and level files):
    /// magic or rare, with a name exactly when rare, known affixes that may roll on it (each
    /// tier one the affix has, each value within the tier's range, no two with the same effect),
    /// and no more prefixes or suffixes than its rarity allows (at least one affix).
    pub fn check(&self, def: &ObjectDef, content: &Content) -> Result<(), String> {
        let Some(kind) = def.kind else {
            return Err(format!("`{}` cannot be magic: it has no `kind`", def.id));
        };
        match (self.rarity, &self.name) {
            (Rarity::Normal, _) => return Err("a normal item has no magic".into()),
            (Rarity::Rare, None) => return Err("a rare item has a name".into()),
            (Rarity::Magic, Some(_)) => return Err("only rare items have names".into()),
            _ => {}
        }
        if let Some(id) = &self.unique {
            let unique =
                (content.unique(id)).ok_or_else(|| format!("unknown unique item `{id}`"))?;
            let base = &content.object(unique.object).id;
            if self.rarity != Rarity::Rare || *base != def.id {
                return Err(format!("unique item `{id}` is a rare `{base}`"));
            }
        }
        if self.affixes.is_empty() {
            return Err("a magic or rare item has an affix at least".into());
        }
        if self.level == 0 || self.level > crate::content::MAX_ITEM_LEVEL {
            return Err(format!("item level {} is out of range", self.level));
        }
        let mut effects = Vec::new();
        let mut per_kind = [0usize; 2];
        for affix in &self.affixes {
            let a = content
                .affixes()
                .get(&affix.id)
                .ok_or_else(|| format!("unknown affix `{}`", affix.id))?;
            if !may_roll(a, kind, def) {
                return Err(format!("`{}` does not roll on {}", a.id, def.plural));
            }
            let tier = a
                .tier(affix.tier)
                .ok_or_else(|| format!("`{}` has no tier {}", a.id, affix.tier))?;
            if !(tier.min..=tier.max).contains(&affix.value) {
                return Err(format!(
                    "`{}` tier {} rolls {} to {}, not {}",
                    a.id, affix.tier, tier.min, tier.max, affix.value
                ));
            }
            if effects.contains(&a.group()) {
                return Err(format!("two affixes do `{}`", a.effect.id()));
            }
            effects.push(a.group());
            per_kind[a.kind as usize] += 1;
        }
        let most = self.rarity.most_per_kind();
        if per_kind.iter().any(|&n| n > most) {
            return Err(format!(
                "a {} item has at most {most} prefixes and {most} suffixes",
                self.rarity.word()
            ));
        }
        Ok(())
    }
}

/// The powers of the unique items among the things in `inventory` that `equipment` holds (spec
/// §8 "Magic items": a power works while its item is worn or wielded), in equipment order.
pub fn worn_powers<'c>(
    inventory: &'c [crate::object::Stack],
    equipment: &'c crate::equipment::Equipment,
    content: &'c Content,
) -> impl Iterator<Item = &'c crate::content::Power> + 'c {
    (equipment.items().into_iter()).filter_map(move |(_, item)| {
        let id = inventory.get(item)?.magic()?.unique.as_deref()?;
        content.unique(id)?.power.as_ref()
    })
}

/// Whether affix `a` may roll on `def`, an item of `kind`: its bases name the kind, and the item
/// has what the effect changes (a range to lengthen, a bow's Strength to lower, a chance to break,
/// a Dexterity limit, Armour, or Shield).
pub fn may_roll(a: &AffixDef, kind: ItemKind, def: &ObjectDef) -> bool {
    let has = match a.effect {
        Effect::Range => def.range.is_some(),
        Effect::Draw => def.attack.is_some_and(|at| at.strength.is_some()),
        Effect::Durability => def.breaks > 0,
        Effect::MaxDex => def.max_dex.is_some(),
        Effect::Armour => def.armour > 0,
        Effect::Shield => def.shield > 0,
        _ => true,
    };
    has && a.bases.contains(&kind)
}

#[cfg(test)]
mod tests;
