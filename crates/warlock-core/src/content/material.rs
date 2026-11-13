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

/// The most a [`Found`] weight counts for, at any level ([`Found::at`] caps it there).
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
    /// parts' materials by their shares ([`super::ObjectDef::value_with`]).
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
    /// Dice more of a weapon's own size a weapon of it rolls as its working part, steel 0, the
    /// default (adamantium 1: a long sword's 1d9 is 2d9; [`crate::variant::dice`]).
    #[serde(default)]
    pub dice: u32,
    /// What of an armour's Protection an armour of it keeps, in percent (steel 100). None: it
    /// cannot be armour.
    #[serde(default)]
    pub protection: Option<u32>,
    /// What of a shield's Defence a shield of it gives, in percent (steel 100, the default;
    /// adamantium 200; [`crate::variant::shield`]).
    #[serde(default = "steel")]
    pub defence: u32,
    /// How often loot makes an item of it ([`Found`]); none: never.
    #[serde(default)]
    pub rarity: Option<Found>,
    /// How often a creature's gear is of it, by the creature's level ([`Found`]); none: never.
    #[serde(default)]
    pub gear: Option<Found>,
    /// How well it suits an item of each use, for how often an item of another material is
    /// made of it (#1191, [`Fits`]): `fit = { weapon = "odd", armour = "poor" }`; every use
    /// `good` by default.
    #[serde(default)]
    pub fit: Fits,
    /// How hard it is to work, a signed modifier on the check of every salvage and recipe that
    /// works it (docs/design/professions.md §4; steel 0, the default; tin −3, adamantium +8).
    #[serde(default)]
    pub work: i32,
    /// How well it holds magic (docs/design/enchanting.md §3): a term of the affix capacity of
    /// an item whose main part is of it ([`crate::magic::capacity`]), −2 to +2; steel 0, the
    /// default; silver −2, mithril +2.
    #[serde(default)]
    pub binding: i8,
    /// How it withstands each kind of item damage, in percent (spec §7 "Materials";
    /// `world/item_damage.rs`): `resists = { acid = 20, rust = 10, fire = 95 }`, every kind
    /// given. One without withstands nothing.
    #[serde(default)]
    pub(super) resists: Option<Resists>,
    /// Never damaged by anything (`world/item_damage.rs`): acid, rust, heat, a failed working,
    /// a spell's ruin, and never found damaged in a creature's gear (adamantium; default false).
    #[serde(default)]
    pub proof: bool,
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

/// How well a material suits an item of some use (#1191): a `good` one is found by its own
/// weight, a `poor` one at a tenth of it ([`POOR`]), an `odd` one (a wool ring, a tin long
/// sword) at a hundredth ([`ODD`]), never below 1 where it weighs anything, so nothing found is
/// ruled out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fit {
    #[default]
    Good,
    Poor,
    Odd,
}

/// What a [`Fit::Poor`] material's weight is divided by.
pub const POOR: u32 = 10;
/// What a [`Fit::Odd`] material's weight is divided by.
pub const ODD: u32 = 100;

impl Fit {
    /// Every fit, best first.
    pub const ALL: [Fit; 3] = [Fit::Good, Fit::Poor, Fit::Odd];

    /// `weight` scaled by the fit: divided by [`POOR`] or [`ODD`], rounded up.
    pub const fn scale(self, weight: u32) -> u32 {
        match self {
            Fit::Good => weight,
            Fit::Poor => weight.div_ceil(POOR),
            Fit::Odd => weight.div_ceil(ODD),
        }
    }

    /// Its word, as data and the wiki write it.
    pub const fn id(self) -> &'static str {
        match self {
            Fit::Good => "good",
            Fit::Poor => "poor",
            Fit::Odd => "odd",
        }
    }
}

/// How well a material suits an item by its use (#1191): what strikes or is shot (`weapon`),
/// armour and shields (`armour`), rings and amulets (`jewellery`), and everything else
/// (`other`). Read with [`crate::variant::fit`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Fits {
    pub weapon: Fit,
    pub armour: Fit,
    pub jewellery: Fit,
    pub other: Fit,
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
}

impl MaterialDef {
    /// Its chances to withstand item damage: its own `resists`, else none at all.
    pub fn resists(&self) -> Resists {
        self.resists.unwrap_or_default()
    }

    /// Its weapon modifiers, to hit and to damage, if it can be a weapon's working part (one of
    /// them given: the other is 0).
    pub fn weapon(&self) -> Option<(i32, i32)> {
        match (self.to_hit, self.damage) {
            (None, None) => None,
            (hit, damage) => Some((hit.unwrap_or(0), damage.unwrap_or(0))),
        }
    }

    /// Checks what a material file cannot say on its own: a `weight` above 0 (a variant's
    /// weight divides by its base material's, [`crate::variant::weight`]); a `rarity` or `gear`
    /// only with a `group` (only grouped materials are drawn for another item's); and a gem
    /// with no `group` (a part's material is never a gem: gems are set, [`super::gem`]); a
    /// `binding` of −2 to +2.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.weight == 0 {
            return Err(format!(
                "material `{}`: `weight` is at least 1 (percent of steel's)",
                self.id
            ));
        }
        for (what, found) in [("rarity", &self.rarity), ("gear", &self.gear)] {
            if found.is_some() && self.group.is_none() {
                return Err(format!(
                    "material `{}`: `{what}`: only a material of a `group` is found as \
                     another item's material",
                    self.id
                ));
            }
        }
        if !(-2..=2).contains(&self.binding) {
            return Err(format!(
                "material `{}`: `binding` is -2 to 2, not {}",
                self.id, self.binding
            ));
        }
        if self.gem.is_some() && self.group.is_some() {
            return Err(format!(
                "material `{}`: a gem is set in an item, never what one is made of: no \
                 `group`",
                self.id
            ));
        }
        Ok(())
    }
}
