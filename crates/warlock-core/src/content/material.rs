//! Materials (`assets/data/materials/`, one file each; docs/dev/content.md): what objects are
//! made of, part by part ([`super::part`]). A material has a name, whether it is `precious` (a gem
//! or a precious metal: it shows in the names of what it is part of), and its `value` (percent,
//! steel 100), which weighs it against the others in what an object of other parts is worth;
//! a gem has a `gem` table besides ([`GemDef`], spec §7 "Gems").
//!
//! Its properties (#193, #194) are on one scale, steel's: `weight` (percent of steel's for an
//! item of it), weapon modifiers `to_hit` and `damage` (steel 0 and 0; neither: it cannot be a
//! weapon's working part), `protection` (percent of an armour's Protection kept, steel 100;
//! none: it cannot be armour), and `color`, what an item made of it instead of its base item's
//! material is drawn in. Its `group` (metal, wood, bone, ...) is what construction lists name it
//! by (`made_of`, [`super::part::construction`]); `rarity` and `gear` are how often loot and
//! creatures' gear are made of it ([`Found`]). What an item of another material than its base
//! item's is like follows from these ([`crate::variant`]).

use super::gem::GemDef;
use crate::color::Rgb;
use serde::{Deserialize, Serialize};

/// Index of a material in [`super::Content::materials`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MaterialId(pub u16);

/// The most a material's `value` may be, in percent of steel's.
pub const MAX_MATERIAL_VALUE: u32 = 10_000;
/// The most a material's `weight` may be, in percent of steel's.
pub const MAX_MATERIAL_WEIGHT: u32 = 1000;
/// The most a material's weapon modifiers (`to_hit`, `damage`) may be, either way.
pub const MAX_MODIFIER: i32 = 10;
/// The most a material's `protection` may be, in percent of an armour's Protection.
pub const MAX_PROTECTION: u32 = 500;
/// The most a [`Found`] weight may be, at any level.
pub const MAX_FOUND: u32 = 100_000;
/// The heaviest weight a very rare material is found with (spec §7 "Materials"): a variant
/// worse in a fight than its base item's own material but dearer is found at most this often
/// ([`crate::variant`]).
pub const VERY_RARE: u32 = 10;
/// A group no material may have: `made_of = ["any"]` is every material with a group.
pub const ANY: &str = "any";

/// A material, as written in content files and as loaded.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialDef {
    pub id: String,
    pub name: String,
    /// A gem or a precious metal: names show it (spec §7 "Objects and materials").
    #[serde(default)]
    pub precious: bool,
    /// What it is worth, in percent of steel (100, the default): an object's value follows its
    /// parts' materials by their shares ([`super::ObjectDef::value_with`]). 1 to
    /// [`MAX_MATERIAL_VALUE`].
    #[serde(default = "steel")]
    pub value: u32,
    /// A gem (spec §7 "Gems"): its colour, what a plain one is worth, and how often loot finds
    /// it. A gem is precious.
    #[serde(default)]
    pub gem: Option<GemDef>,
    /// What construction lists name it by with the others of its kind (`metal`, `wood`, `bone`,
    /// `stone`, `hide`, `cloth`, ...; `made_of`). None: nothing is made of it but what its base
    /// item is (paper, wax, food, gems).
    #[serde(default)]
    pub group: Option<String>,
    /// The colour an item made of it is drawn in where its base item is made of another
    /// material (a bronze long sword); none: its base item's.
    #[serde(default)]
    pub color: Option<Rgb>,
    /// What an item of it weighs, in percent of the same item of steel (100, the default): an
    /// item's weight scales by it against its base item's material, part by part.
    #[serde(default = "steel")]
    pub weight: u32,
    /// To hit as a weapon's working part (a blade, a head), steel 0. None with no `damage`
    /// either: it cannot be one.
    #[serde(default)]
    pub to_hit: Option<i32>,
    /// To every roll of a weapon's damage as its working part, steel 0.
    #[serde(default)]
    pub damage: Option<i32>,
    /// What of an armour's Protection an armour of it keeps, in percent (steel 100). None: it
    /// cannot be armour.
    #[serde(default)]
    pub protection: Option<u32>,
    /// How often loot makes an item of it ([`Found`]); none: never.
    #[serde(default)]
    pub rarity: Option<Found>,
    /// How often a creature's gear is of it, by the creature's level ([`Found`]); none: never.
    #[serde(default)]
    pub gear: Option<Found>,
    /// How it withstands each kind of item damage, in percent (spec §7 "Materials";
    /// `world/item_damage.rs`): `resists = { acid = 20, rust = 10, fire = 95 }`, every kind
    /// given. Every material of the game's content sets it (`material_resists_tests.rs`); one
    /// without (a test's) withstands nothing.
    #[serde(default)]
    pub(super) resists: Option<Resists>,
}

/// A kind of item damage (spec §7 "Materials", `world/item_damage.rs`): what leaves an item
/// damaged, the smith's one state ([`crate::magic::Worked`]), and destroys one damaged already.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemHarm {
    /// Acid eating at it (the hezrou's filth, the ember beetle's blood, a pudding).
    Acid,
    /// Rust (the rust monster's feelers).
    Rust,
    /// Heat scorching it.
    Fire,
    /// A failed attempt at the anvil or with the knife (`world/temper.rs`): no material
    /// withstands it, being the craft's own blunder.
    Work,
}

impl ItemHarm {
    /// The kinds a material's `resists` gives, in its order.
    pub const RESISTED: [ItemHarm; 3] = [ItemHarm::Acid, ItemHarm::Rust, ItemHarm::Fire];

    /// Its id, as data and the wiki write it.
    pub const fn id(self) -> &'static str {
        match self {
            ItemHarm::Acid => "acid",
            ItemHarm::Rust => "rust",
            ItemHarm::Fire => "fire",
            ItemHarm::Work => "work",
        }
    }

    /// What an item it damaged is called, ahead of its name: "a rusted long sword".
    pub const fn word(self) -> &'static str {
        match self {
            ItemHarm::Acid => "corroded",
            ItemHarm::Rust => "rusted",
            ItemHarm::Fire => "scorched",
            ItemHarm::Work => crate::magic::worked::DAMAGED,
        }
    }
}

/// A material's chance in percent (0 to 100) to withstand each kind of item damage that
/// materials resist ([`ItemHarm::RESISTED`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resists {
    pub acid: u8,
    pub rust: u8,
    pub fire: u8,
}

impl Resists {
    /// The chance to withstand `harm`: 0 for [`ItemHarm::Work`].
    pub const fn of(&self, harm: ItemHarm) -> u8 {
        match harm {
            ItemHarm::Acid => self.acid,
            ItemHarm::Rust => self.rust,
            ItemHarm::Fire => self.fire,
            ItemHarm::Work => 0,
        }
    }
}

/// How often something is made of a material, among the materials its item may be made of: a
/// `weight` at level 1, `deeper` more (or less) for each level past the first, never below 0,
/// and nothing below `level`. Loot's level is the item level; gear's, the creature's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Found {
    pub weight: u32,
    #[serde(default)]
    pub deeper: i32,
    #[serde(default = "first")]
    pub level: u8,
}

fn steel() -> u32 {
    100
}

fn first() -> u8 {
    1
}

impl Found {
    /// Its weight at `level`: 0 below its own level, else `weight` plus `deeper` for each level
    /// past the first, at least 0 and at most [`MAX_FOUND`].
    pub fn at(&self, level: u8) -> u32 {
        let level = level.max(1);
        if level < self.level {
            return 0;
        }
        let more = i64::from(self.deeper) * i64::from(level - 1);
        let weight = (i64::from(self.weight) + more).clamp(0, i64::from(MAX_FOUND));
        u32::try_from(weight).unwrap_or(0)
    }

    fn check(&self, material: &str, what: &str) -> Result<(), String> {
        let limit = i32::try_from(MAX_FOUND).unwrap_or(i32::MAX);
        if self.weight > MAX_FOUND || !(-limit..=limit).contains(&self.deeper) {
            return Err(format!(
                "material `{material}`: `{what}` weighs 0 to {MAX_FOUND}, `deeper` ±{MAX_FOUND}"
            ));
        }
        if self.level == 0 {
            return Err(format!(
                "material `{material}`: `{what}`'s `level` is 1 or more"
            ));
        }
        Ok(())
    }
}

impl MaterialDef {
    /// Its chances to withstand item damage: its own `resists`, else none at all.
    pub fn resists(&self) -> Resists {
        self.resists.unwrap_or_default()
    }

    /// Whether its file sets `resists`.
    pub fn sets_resists(&self) -> bool {
        self.resists.is_some()
    }

    /// Its weapon modifiers, to hit and to damage, if it can be a weapon's working part (one of
    /// them given: the other is 0).
    pub fn weapon(&self) -> Option<(i32, i32)> {
        match (self.to_hit, self.damage) {
            (None, None) => None,
            (hit, damage) => Some((hit.unwrap_or(0), damage.unwrap_or(0))),
        }
    }

    /// Checks what a material file cannot say on its own.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(format!(
                "material `{}`: `id` and `name` must not be empty",
                self.id
            ));
        }
        if !(1..=MAX_MATERIAL_VALUE).contains(&self.value) {
            return Err(format!(
                "material `{}`: `value` is in percent of steel's, 1 to {MAX_MATERIAL_VALUE}",
                self.id
            ));
        }
        if !(1..=MAX_MATERIAL_WEIGHT).contains(&self.weight) {
            return Err(format!(
                "material `{}`: `weight` is in percent of steel's, 1 to {MAX_MATERIAL_WEIGHT}",
                self.id
            ));
        }
        let mut modifiers = [self.to_hit, self.damage].into_iter().flatten();
        if modifiers.any(|m| !(-MAX_MODIFIER..=MAX_MODIFIER).contains(&m)) {
            return Err(format!(
                "material `{}`: `to_hit` and `damage` are -{MAX_MODIFIER} to {MAX_MODIFIER}",
                self.id
            ));
        }
        if self.protection.is_some_and(|p| p > MAX_PROTECTION) {
            return Err(format!(
                "material `{}`: `protection` is in percent, 0 to {MAX_PROTECTION}",
                self.id
            ));
        }
        if let Some(group) = &self.group
            && (group.trim().is_empty() || group.contains(char::is_whitespace) || group == ANY)
        {
            return Err(format!(
                "material `{}`: `group` is one word, not `{ANY}`",
                self.id
            ));
        }
        for (what, found) in [("rarity", &self.rarity), ("gear", &self.gear)] {
            if let Some(found) = found {
                found.check(&self.id, what)?;
                if self.group.is_none() {
                    return Err(format!(
                        "material `{}`: `{what}`: only a material of a `group` is found as \
                         another item's material",
                        self.id
                    ));
                }
            }
        }
        let mut resisted = ItemHarm::RESISTED.into_iter();
        if let Some(harm) = resisted.find(|&h| self.resists().of(h) > 100) {
            return Err(format!(
                "material `{}`: `resists`' `{}` is in percent, 0 to 100",
                self.id,
                harm.id()
            ));
        }
        if let Some(gem) = &self.gem {
            if !self.precious {
                return Err(format!(
                    "material `{}`: a gem is precious (`precious = true`)",
                    self.id
                ));
            }
            if self.group.is_some() {
                return Err(format!(
                    "material `{}`: a gem is set in an item, never what one is made of: no \
                     `group`",
                    self.id
                ));
            }
            gem.check(&self.id)?;
        }
        Ok(())
    }
}
