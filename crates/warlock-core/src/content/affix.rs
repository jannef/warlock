//! Magic item content (spec §7 "Magic items"): the prefixes and suffixes magic and rare items
//! roll, how often loot is normal, magic, or rare, how many affixes each rolls, and the words
//! rare names are made of. The table is a directory (docs/dev/content.md): the settings
//! (`[rarity]`, `[magic]`, `[rare]`, `[names]`) in `assets/data/affixes.toml`, one `[[prefix]]`
//! or `[[suffix]]` per file under `assets/data/affixes/`, numbered in path order.
//!
//! Every object that may be found magic says what kind of item it is (`kind` in
//! `data/objects.toml`, [`ItemKind`]): a weapon, a missile weapon, ammunition, body armour, a
//! helmet, a cloak, gloves, boots, a shield, a ring, an amulet, or a tool. An affix names the
//! kinds it may roll on (`bases`), the one [`Effect`] it has, and its **tiers** (at most
//! [`MAX_TIERS`]): from which item level each is found, and the range its value is rolled in (a
//! light's tier names a light preset instead). An item never has two affixes of one [`Group`]
//! (the effect, and the skill, damage type, or resource it names).
//!
//! Attack effects (to-hit, damage, criticals, attack speed, range, ...) on a weapon, a missile
//! weapon, or ammunition are **local**: they change attacks made with that item only. Elsewhere
//! (gloves, rings, amulets) they are **global**: every attack. Effects on the item itself (its
//! Armour, Protection, Shield, Parry, the Dexterity it allows, its weight) are always local, and
//! the rest
//! (stats, hit points, mana, Dodge, Deflection, natural armour, regeneration, moving, carrying,
//! stealth, light) always global. Effects apply while the item is worn or wielded.

use super::{ContentError, DamageTypeId, LightId, ResourceId};
use crate::skills::SkillId;
use crate::stats::Stat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The highest item level: dungeon depth gives it (spec §8 "Magic items").
pub const MAX_ITEM_LEVEL: u8 = 99;
/// The most prefixes, and the most suffixes, an item may have (a rare: three of each).
pub const MAX_PER_KIND: usize = 3;
/// The most tiers an affix may have: a ladder that steps up every item level or two
/// (docs/design/affixes-systems.md §3).
pub const MAX_TIERS: usize = 12;
/// The most an item's attack speed or its wearer's moving speed may be raised, in percent off
/// the cost.
pub const MAX_SPEED: i32 = 50;

/// What kind of item an object is, for magic (`kind` in `data/objects.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A weapon wielded in a hand (it may also be thrown).
    Weapon,
    /// A missile weapon: bows, crossbows, slings.
    Launcher,
    Ammo,
    /// Body armour.
    Body,
    Helmet,
    Cloak,
    Gloves,
    Boots,
    Shield,
    Ring,
    Amulet,
    /// A tool worn in the Tool slot: a pick-axe.
    Tool,
}

impl ItemKind {
    pub const ALL: &'static [ItemKind] = &[
        ItemKind::Weapon,
        ItemKind::Launcher,
        ItemKind::Ammo,
        ItemKind::Body,
        ItemKind::Helmet,
        ItemKind::Cloak,
        ItemKind::Gloves,
        ItemKind::Boots,
        ItemKind::Shield,
        ItemKind::Ring,
        ItemKind::Amulet,
        ItemKind::Tool,
    ];

    /// Its id in data files: `body`.
    pub const fn id(self) -> &'static str {
        match self {
            ItemKind::Weapon => "weapon",
            ItemKind::Launcher => "launcher",
            ItemKind::Ammo => "ammo",
            ItemKind::Body => "body",
            ItemKind::Helmet => "helmet",
            ItemKind::Cloak => "cloak",
            ItemKind::Gloves => "gloves",
            ItemKind::Boots => "boots",
            ItemKind::Shield => "shield",
            ItemKind::Ring => "ring",
            ItemKind::Amulet => "amulet",
            ItemKind::Tool => "tool",
        }
    }

    /// What it is called, in the plural, for people: `body armour`, `missile weapons`.
    pub const fn plural(self) -> &'static str {
        match self {
            ItemKind::Weapon => "weapons",
            ItemKind::Launcher => "missile weapons",
            ItemKind::Ammo => "ammunition",
            ItemKind::Body => "body armour",
            ItemKind::Helmet => "helmets",
            ItemKind::Cloak => "cloaks",
            ItemKind::Gloves => "gloves",
            ItemKind::Boots => "boots",
            ItemKind::Shield => "shields",
            ItemKind::Ring => "rings",
            ItemKind::Amulet => "amulets",
            ItemKind::Tool => "tools",
        }
    }

    pub fn from_id(id: &str) -> Option<ItemKind> {
        ItemKind::ALL.iter().copied().find(|k| k.id() == id)
    }

    /// Whether attacks are made with it: then its attack effects are local, changing only
    /// attacks made with it.
    pub const fn attacks(self) -> bool {
        matches!(self, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Ammo)
    }

    /// Whether it is armour worn for its Protection: body armour, a helmet, a cloak, gloves, or
    /// boots (a shield's is Defence only).
    pub const fn is_armour(self) -> bool {
        matches!(
            self,
            ItemKind::Body
                | ItemKind::Helmet
                | ItemKind::Cloak
                | ItemKind::Gloves
                | ItemKind::Boots
        )
    }
}

/// A prefix or a suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffixKind {
    /// Its name goes before the item's: "Sharp long sword".
    Prefix,
    /// Its name goes after: "long sword of the Bear".
    Suffix,
}

impl AffixKind {
    /// `Prefix`.
    pub const fn name(self) -> &'static str {
        match self {
            AffixKind::Prefix => "Prefix",
            AffixKind::Suffix => "Suffix",
        }
    }
}

/// What an affix does: one effect, mapped onto a rule that exists (spec §8 "Magic items").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Effect {
    /// To hit.
    ToHit,
    /// Added to every roll of the damage dice.
    Damage,
    /// Adds 1dN to every roll of the damage (the value is N).
    ExtraDamage,
    /// Adds 1dN of the affix's damage type (`damage_type`, an element; spec §8 "Damage
    /// types") to every roll of the damage (the value is N).
    Elemental,
    /// Widens the critical threat range by this many natural rolls.
    Threat,
    /// Added to the critical multiplier.
    Crit,
    /// Percent off the cost of attacks.
    AttackSpeed,
    /// Cells added to how far it shoots or is thrown.
    Range,
    /// Strength a bow needs less.
    Draw,
    /// Percent less chance to break on a hit.
    Durability,
    /// Added to the weapon's Parry.
    Parry,
    /// Added to the armour's Armour.
    Armour,
    /// Added to the armour's Protection (the damage it soaks off every hit).
    Protection,
    /// Added to the shield's Shield.
    Shield,
    /// Added to the most Dexterity may add to Defence while it is worn.
    MaxDex,
    /// Percent off its weight.
    Weight,
    /// Defence: Dodge (stacks).
    Dodge,
    /// Defence: Deflection (the best counts).
    Deflection,
    /// Defence: natural armour (the best counts, with the creature's own).
    Natural,
    /// Maximum hit points.
    MaxHp,
    /// Percent faster hit point regeneration.
    Regeneration,
    /// Maximum mana (spec §8 "Mana").
    MaxMana,
    /// Percent faster mana regeneration.
    ManaRegeneration,
    /// Caster level (spec §8 "Spells").
    CasterLevel,
    /// Percent off every mana cost (spec §8 "Mana").
    ManaCost,
    /// Points off this piece's spell cost surcharge (its `spell_cost_percent`; local).
    SpellEase,
    /// To hit with one's own spells (spec §8 "Spells").
    SpellToHit,
    /// Widens one's own spells' critical threat range by this many natural rolls.
    SpellThreat,
    /// Points more of a spell's damage lift.
    SpellLift,
    /// Cells more of a spell's reach.
    SpellReach,
    /// Degrees more of the statuses one's spells give.
    SpellDuration,
    /// Mana back on a kill close by.
    ManaOnKill,
    /// Added to the caster level wands are zapped at.
    WandLevel,
    /// Added to a resource's maximum (the affix's `resource`; spec §8 "Resources").
    ResourceMax,
    /// Percent faster a resource comes back.
    ResourceRate,
    /// Points off every weapon active's cost in the affix's resource (stamina).
    ActiveStamina,
    /// Points of the affix's resource on a kill close by.
    ResourceOnKill,
    /// Percent resistance to the affix's damage type (`damage_type`; spec §8 "Damage types").
    Resistance,
    /// A stat's score.
    Stat(Stat),
    /// Percent off the cost of steps.
    MoveSpeed,
    /// Kilograms of carrying limit.
    Carry,
    /// Percent quieter footsteps.
    Stealth,
    /// Added to the Strength a door (or a chest) is kicked in with.
    DoorKick,
    /// Percent faster digging.
    DigPercent,
    /// Percent slower hunger.
    Hunger,
    /// Percent less damage from traps.
    TrapDamage,
    /// Added to every disease check.
    DiseaseCheck,
    /// Percent chance a potion drunk is not used up.
    PotionKept,
    /// It glows: its wearer carries a light (a preset per tier, no value).
    Light,
    /// Added to a skill's value (the affix's `skill`; spec §8 "Skills") for what the skill gives.
    Skill,
}

impl Effect {
    /// Every effect, in the order the wiki lists them.
    pub fn all() -> Vec<Effect> {
        let mut all = vec![
            Effect::ToHit,
            Effect::Damage,
            Effect::ExtraDamage,
            Effect::Elemental,
            Effect::Threat,
            Effect::Crit,
            Effect::AttackSpeed,
            Effect::Range,
            Effect::Draw,
            Effect::Durability,
            Effect::Parry,
            Effect::Armour,
            Effect::Protection,
            Effect::Shield,
            Effect::MaxDex,
            Effect::Weight,
            Effect::Dodge,
            Effect::Deflection,
            Effect::Natural,
            Effect::MaxHp,
            Effect::Regeneration,
            Effect::MaxMana,
            Effect::ManaRegeneration,
            Effect::CasterLevel,
            Effect::ManaCost,
            Effect::SpellEase,
            Effect::SpellToHit,
            Effect::SpellThreat,
            Effect::SpellLift,
            Effect::SpellReach,
            Effect::SpellDuration,
            Effect::ManaOnKill,
            Effect::WandLevel,
            Effect::ResourceMax,
            Effect::ResourceRate,
            Effect::ActiveStamina,
            Effect::ResourceOnKill,
            Effect::Resistance,
        ];
        all.extend(Stat::ALL.map(Effect::Stat));
        all.extend([
            Effect::MoveSpeed,
            Effect::Carry,
            Effect::Stealth,
            Effect::DoorKick,
            Effect::DigPercent,
            Effect::Hunger,
            Effect::TrapDamage,
            Effect::DiseaseCheck,
            Effect::PotionKept,
            Effect::Light,
            Effect::Skill,
        ]);
        all
    }

    /// Its id in data files: `to_hit`, `strength`.
    pub const fn id(self) -> &'static str {
        match self {
            Effect::ToHit => "to_hit",
            Effect::Damage => "damage",
            Effect::ExtraDamage => "extra_damage",
            Effect::Elemental => "elemental",
            Effect::Threat => "threat",
            Effect::Crit => "crit",
            Effect::AttackSpeed => "attack_speed",
            Effect::Range => "range",
            Effect::Draw => "draw",
            Effect::Durability => "durability",
            Effect::Parry => "parry",
            Effect::Armour => "armour",
            Effect::Protection => "protection",
            Effect::Shield => "shield",
            Effect::MaxDex => "max_dex",
            Effect::Weight => "weight",
            Effect::Dodge => "dodge",
            Effect::Deflection => "deflection",
            Effect::Natural => "natural",
            Effect::MaxHp => "max_hp",
            Effect::Regeneration => "regeneration",
            Effect::MaxMana => "max_mana",
            Effect::ManaRegeneration => "mana_regeneration",
            Effect::CasterLevel => "caster_level",
            Effect::ManaCost => "mana_cost",
            Effect::SpellEase => "spell_ease",
            Effect::SpellToHit => "spell_to_hit",
            Effect::SpellThreat => "spell_threat",
            Effect::SpellLift => "spell_lift",
            Effect::SpellReach => "spell_reach",
            Effect::SpellDuration => "spell_duration",
            Effect::ManaOnKill => "mana_on_kill",
            Effect::WandLevel => "wand_level",
            Effect::ResourceMax => "resource_max",
            Effect::ResourceRate => "resource_rate",
            Effect::ActiveStamina => "active_stamina",
            Effect::ResourceOnKill => "resource_on_kill",
            Effect::Resistance => "resistance",
            Effect::Stat(stat) => stat.id(),
            Effect::MoveSpeed => "move_speed",
            Effect::Carry => "carry",
            Effect::Stealth => "stealth",
            Effect::DoorKick => "door_kick",
            Effect::DigPercent => "dig_percent",
            Effect::Hunger => "hunger",
            Effect::TrapDamage => "trap_damage",
            Effect::DiseaseCheck => "disease_check",
            Effect::PotionKept => "potion_kept",
            Effect::Light => "light",
            Effect::Skill => "skill",
        }
    }

    pub fn from_id(id: &str) -> Option<Effect> {
        Effect::all().into_iter().find(|e| e.id() == id)
    }

    /// How many effects there are.
    pub const COUNT: usize = 49 + Stat::COUNT;

    /// Its place in [`Effect::all`].
    pub const fn index(self) -> usize {
        match self {
            Effect::ToHit => 0,
            Effect::Damage => 1,
            Effect::ExtraDamage => 2,
            Effect::Elemental => 3,
            Effect::Threat => 4,
            Effect::Crit => 5,
            Effect::AttackSpeed => 6,
            Effect::Range => 7,
            Effect::Draw => 8,
            Effect::Durability => 9,
            Effect::Parry => 10,
            Effect::Armour => 11,
            Effect::Protection => 12,
            Effect::Shield => 13,
            Effect::MaxDex => 14,
            Effect::Weight => 15,
            Effect::Dodge => 16,
            Effect::Deflection => 17,
            Effect::Natural => 18,
            Effect::MaxHp => 19,
            Effect::Regeneration => 20,
            Effect::MaxMana => 21,
            Effect::ManaRegeneration => 22,
            Effect::CasterLevel => 23,
            Effect::ManaCost => 24,
            Effect::SpellEase => 25,
            Effect::SpellToHit => 26,
            Effect::SpellThreat => 27,
            Effect::SpellLift => 28,
            Effect::SpellReach => 29,
            Effect::SpellDuration => 30,
            Effect::ManaOnKill => 31,
            Effect::WandLevel => 32,
            Effect::ResourceMax => 33,
            Effect::ResourceRate => 34,
            Effect::ActiveStamina => 35,
            Effect::ResourceOnKill => 36,
            Effect::Resistance => 37,
            Effect::Stat(stat) => 38 + stat as usize,
            Effect::MoveSpeed => 38 + Stat::COUNT,
            Effect::Carry => 39 + Stat::COUNT,
            Effect::Stealth => 40 + Stat::COUNT,
            Effect::DoorKick => 41 + Stat::COUNT,
            Effect::DigPercent => 42 + Stat::COUNT,
            Effect::Hunger => 43 + Stat::COUNT,
            Effect::TrapDamage => 44 + Stat::COUNT,
            Effect::DiseaseCheck => 45 + Stat::COUNT,
            Effect::PotionKept => 46 + Stat::COUNT,
            Effect::Light => 47 + Stat::COUNT,
            Effect::Skill => 48 + Stat::COUNT,
        }
    }

    /// Whether it names a damage type (`damage_type` in its affix): what it adds or resists.
    pub const fn is_typed(self) -> bool {
        matches!(self, Effect::Elemental | Effect::Resistance)
    }

    /// Whether it names a resource (`resource` in its affix): the pool whose maximum, pace,
    /// costs, or gains it changes (spec §8 "Resources").
    pub const fn is_pooled(self) -> bool {
        matches!(
            self,
            Effect::ResourceMax
                | Effect::ResourceRate
                | Effect::ActiveStamina
                | Effect::ResourceOnKill
        )
    }

    /// The values a tier may roll (a light's tier has none).
    pub const fn values(self) -> RangeInclusive<i32> {
        match self {
            Effect::ExtraDamage | Effect::Elemental => 2..=20,
            Effect::Threat | Effect::Crit | Effect::SpellThreat => 1..=3,
            Effect::AttackSpeed | Effect::MoveSpeed | Effect::DigPercent => 1..=MAX_SPEED,
            Effect::ManaCost | Effect::SpellLift | Effect::PotionKept => 1..=50,
            Effect::ActiveStamina => 1..=5,
            Effect::ManaOnKill | Effect::ResourceOnKill => 1..=20,
            Effect::Hunger | Effect::TrapDamage => 1..=75,
            Effect::Durability | Effect::Weight | Effect::Stealth => 1..=90,
            Effect::MaxHp
            | Effect::MaxMana
            | Effect::Carry
            | Effect::Skill
            | Effect::SpellEase
            | Effect::ResourceMax => 1..=100,
            Effect::Regeneration | Effect::ManaRegeneration | Effect::ResourceRate => 1..=300,
            Effect::Resistance => 1..=crate::damage::MAX_RESISTANCE,
            Effect::Light => 0..=0,
            _ => 1..=10,
        }
    }

    /// Whether it is an attack effect: local on what attacks are made with, global elsewhere.
    pub const fn is_attack(self) -> bool {
        matches!(
            self,
            Effect::ToHit
                | Effect::Damage
                | Effect::ExtraDamage
                | Effect::Elemental
                | Effect::Threat
                | Effect::Crit
                | Effect::AttackSpeed
                | Effect::Range
                | Effect::Draw
                | Effect::Durability
        )
    }

    /// Whether it changes the item itself wherever it is (its Armour, Protection, Shield, Parry,
    /// the Dexterity it allows, its weight, how much it hinders casting).
    pub const fn is_item(self) -> bool {
        matches!(
            self,
            Effect::Parry
                | Effect::Armour
                | Effect::Protection
                | Effect::Shield
                | Effect::MaxDex
                | Effect::Weight
                | Effect::SpellEase
        )
    }

    /// Whether it is local on an item of `kind`: it changes that item (or attacks made with it)
    /// rather than its wearer.
    pub const fn is_local_on(self, kind: ItemKind) -> bool {
        self.is_item() || (self.is_attack() && kind.attacks())
    }

    /// The kinds of item it may be on at all: what only attacks made with an item can use needs
    /// such an item, and what changes an item needs one that has it.
    fn allowed_on(self, kind: ItemKind) -> bool {
        match self {
            Effect::ExtraDamage
            | Effect::Elemental
            | Effect::Range
            | Effect::Draw
            | Effect::Durability => kind.attacks(),
            Effect::Parry => kind == ItemKind::Weapon,
            Effect::Armour => kind == ItemKind::Body,
            Effect::Protection => kind.is_armour(),
            Effect::Shield => kind == ItemKind::Shield,
            Effect::MaxDex => matches!(kind, ItemKind::Body | ItemKind::Shield),
            // What may hinder casting (`spell_cost_percent`): body armour, a helmet, a shield.
            Effect::SpellEase => {
                matches!(kind, ItemKind::Body | ItemKind::Helmet | ItemKind::Shield)
            }
            _ => true,
        }
    }
}

/// One tier of an affix: from which item level it is found, and what it rolls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tier {
    /// The lowest item level it is found at.
    pub level: u8,
    /// The value is rolled uniformly in `min..=max`.
    pub min: i32,
    pub max: i32,
    /// A light's tier: the light the item gives.
    pub light: Option<LightId>,
}

/// A prefix or a suffix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffixDef {
    /// Stable id: items record their affixes by it.
    pub id: String,
    /// "Sharp", "of the Bear".
    pub name: String,
    pub kind: AffixKind,
    pub effect: Effect,
    /// Its category, as Path of Exile's mod groups (`physical_damage`, `life`): an item never
    /// has two affixes of one category. Every affix of one [`Group`] shares it, so no item has
    /// two of one group either.
    pub category: String,
    /// The skill a [`Effect::Skill`] affix adds to.
    pub skill: Option<SkillId>,
    /// The damage type an [`Effect::Elemental`] affix adds or an [`Effect::Resistance`] one
    /// resists.
    pub damage_type: Option<DamageTypeId>,
    /// The resource a pooled affix ([`Effect::is_pooled`]) changes: `stamina`, `rage`.
    pub resource: Option<ResourceId>,
    /// What kinds of item may roll it.
    pub bases: Vec<ItemKind>,
    /// How often it rolls among those that may (default 100).
    pub weight: u32,
    /// Tier 1 first, by item level.
    pub tiers: Vec<Tier>,
}

/// What an affix changes: its effect, the skill a skill affix raises, the damage type an
/// elemental or resistance affix names, and the resource a pooled one changes. An item has at
/// most one affix of each (one per category, `check_category`), and [`crate::magic::Mods`] sums
/// by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Group {
    pub effect: Effect,
    pub skill: Option<SkillId>,
    pub damage_type: Option<DamageTypeId>,
    pub resource: Option<ResourceId>,
}

impl Group {
    /// The group of an effect that names nothing else.
    pub const fn of(effect: Effect) -> Group {
        Group {
            effect,
            skill: None,
            damage_type: None,
            resource: None,
        }
    }
}

impl AffixDef {
    /// What it changes, for the rule that an item has at most one affix of each: its effect, and
    /// for a skill affix which skill, for a typed one which damage type, for a pooled one which
    /// resource.
    pub fn group(&self) -> Group {
        Group {
            effect: self.effect,
            skill: self.skill,
            damage_type: self.damage_type,
            resource: self.resource,
        }
    }

    /// Its tier `tier` (1 for the first), if it has one.
    pub fn tier(&self, tier: u8) -> Option<&Tier> {
        self.tiers.get(usize::from(tier).checked_sub(1)?)
    }

    /// How many of its tiers are found at item level `level`.
    pub fn tiers_at(&self, level: u8) -> usize {
        self.tiers.iter().filter(|t| t.level <= level).count()
    }
}

/// What loot may be: normal (no affixes), magic (a prefix, a suffix, or one of each), or rare
/// (up to three of each, and a name).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
}

impl Rarity {
    pub const ALL: [Rarity; 3] = [Rarity::Normal, Rarity::Magic, Rarity::Rare];

    /// `magic`.
    pub const fn word(self) -> &'static str {
        match self {
            Rarity::Normal => "normal",
            Rarity::Magic => "magic",
            Rarity::Rare => "rare",
        }
    }

    /// The most prefixes and suffixes (each) it may have.
    pub const fn most_per_kind(self) -> usize {
        match self {
            Rarity::Normal => 0,
            Rarity::Magic => 1,
            Rarity::Rare => MAX_PER_KIND,
        }
    }
}

/// Everything in `data/affixes.toml`. Without the file there are no magic items: loot is always
/// normal.
#[derive(Clone, Debug, Default)]
pub struct Affixes {
    /// Prefixes, then suffixes, each in file order.
    defs: Vec<AffixDef>,
    ids: BTreeMap<String, usize>,
    /// Weights of normal, magic, and rare loot.
    pub rarity: [u32; 3],
    /// The weights of what a magic item has: a prefix only, a suffix only, or one of each.
    pub magic_affixes: [u32; 3],
    /// How many affixes a rare item rolls, each with its weight.
    pub rare_counts: Vec<(u8, u32)>,
    /// The first words of rare names ("Grim").
    pub first_names: Vec<String>,
    /// The second words of rare names, by kind of item ("Bite" for weapons, "Tooth" for tools).
    pub second_names: BTreeMap<ItemKind, Vec<String>>,
}

impl Affixes {
    /// Every affix: the prefixes, then the suffixes.
    pub fn all(&self) -> &[AffixDef] {
        &self.defs
    }

    /// The affix with id `id`.
    pub fn get(&self, id: &str) -> Option<&AffixDef> {
        self.ids.get(id).map(|&i| &self.defs[i])
    }

    /// Whether there are any affixes (else loot is always normal).
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// The weight of loot of `rarity`.
    pub fn rarity_weight(&self, rarity: Rarity) -> u32 {
        self.rarity[rarity as usize]
    }
}

/// One file of the affix table: the settings, each set in exactly one file of the table (the
/// settings file, `data/affixes.toml`), and any number of entries (none in a file of comments).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixFile {
    rarity: Option<RaritySpec>,
    magic: Option<MagicSpec>,
    rare: Option<RareSpec>,
    names: Option<NamesSpec>,
    #[serde(default)]
    prefix: Vec<AffixSpec>,
    #[serde(default)]
    suffix: Vec<AffixSpec>,
}

/// A settings table of a content table's files (the affixes', the talents'), set once: in
/// `file`.
pub(super) struct Once<'a, T> {
    name: &'static str,
    set: Option<(&'a str, T)>,
}

impl<'a, T> Once<'a, T> {
    pub(super) fn new(name: &'static str) -> Self {
        Self { name, set: None }
    }

    /// Takes the table from `file`, if it has one; a second file setting it is an error naming
    /// both.
    pub(super) fn take(&mut self, file: &'a str, value: Option<T>) -> Result<(), ContentError> {
        let Some(value) = value else { return Ok(()) };
        if let Some((first, _)) = &self.set {
            return Err(ContentError::new(
                file,
                format!("`[{}]` is set here and in {first}", self.name),
            ));
        }
        self.set = Some((file, value));
        Ok(())
    }

    /// The table and the file it is in; missing from every file is an error naming `first`.
    pub(super) fn get(self, first: &str) -> Result<(&'a str, T), ContentError> {
        self.set
            .ok_or_else(|| ContentError::new(first, format!("`[{}]` is missing", self.name)))
    }

    /// The table, or `default` when no file set it.
    pub(super) fn get_or(self, default: T) -> T {
        self.set.map_or(default, |(_, value)| value)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RaritySpec {
    normal: u32,
    magic: u32,
    rare: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MagicSpec {
    prefix: u32,
    suffix: u32,
    both: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RareSpec {
    /// `[[affixes, weight], ...]`.
    affixes: Vec<[u32; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamesSpec {
    first: Vec<String>,
    second: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixSpec {
    id: String,
    name: String,
    effect: String,
    /// Its category (lower case, `_` between words): an item never has two affixes of one.
    category: String,
    bases: Vec<String>,
    weight: Option<u32>,
    /// The skill an affix of effect `skill` adds to (an id in `data/skills.toml`).
    skill: Option<String>,
    /// The damage type an affix of effect `elemental` adds or one of effect `resistance`
    /// resists (an id in `data/damage_types.toml`).
    damage_type: Option<String>,
    /// The resource a pooled affix changes (an id in `data/resources.toml`).
    resource: Option<String>,
    tiers: Vec<TierSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TierSpec {
    level: u8,
    min: Option<i32>,
    max: Option<i32>,
    light: Option<String>,
}

/// How the affix table resolves the ids it names. A table content has none of is `None`, and the
/// affixes that would name one of its entries are left out: skill affixes without skills, typed
/// ones without damage types, pooled ones without resources.
pub(super) struct Lookups<'a> {
    /// Light preset ids (a light's tiers).
    pub light: &'a dyn Fn(&str) -> Option<LightId>,
    /// Skill ids (`skill`).
    pub skill: Option<&'a dyn Fn(&str) -> Option<SkillId>>,
    /// Damage type ids (`damage_type`).
    pub damage_type: Option<&'a dyn Fn(&str) -> Option<DamageTypeId>>,
    /// Resource ids (`resource`).
    pub resource: Option<&'a dyn Fn(&str) -> Option<ResourceId>>,
}

/// Parses the affix table: `files` is `ContentFiles::table("affixes")` (`data/affixes.toml`,
/// then `data/affixes/*.toml` in path order; at least one file), each of the settings tables set
/// in exactly one of them, the entries numbered in the order read, the ids they name resolved by
/// `lookups`.
pub(super) fn parse<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    lookups: &Lookups<'_>,
) -> Result<Affixes, ContentError> {
    let mut rarity_spec = Once::new("rarity");
    let mut magic_spec = Once::new("magic");
    let mut rare_spec = Once::new("rare");
    let mut names_spec = Once::new("names");
    let mut first_file = None;
    let mut specs = Vec::new();
    for (file, text) in files {
        first_file.get_or_insert(file);
        let parsed: AffixFile =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        rarity_spec.take(file, parsed.rarity)?;
        magic_spec.take(file, parsed.magic)?;
        rare_spec.take(file, parsed.rare)?;
        names_spec.take(file, parsed.names)?;
        specs.extend((parsed.prefix.into_iter()).map(|s| (file, AffixKind::Prefix, s)));
        specs.extend((parsed.suffix.into_iter()).map(|s| (file, AffixKind::Suffix, s)));
    }
    let first_file = first_file.unwrap_or("data/affixes.toml");
    let (file, rarity_spec) = rarity_spec.get(first_file)?;
    let err = |m: String| ContentError::new(file, m);
    let rarity = [rarity_spec.normal, rarity_spec.magic, rarity_spec.rare];
    if rarity.iter().sum::<u32>() == 0 {
        return Err(err("`rarity`: the weights may not all be 0".into()));
    }
    let (file, magic_spec) = magic_spec.get(first_file)?;
    let err = |m: String| ContentError::new(file, m);
    let magic_affixes = [magic_spec.prefix, magic_spec.suffix, magic_spec.both];
    if rarity[Rarity::Magic as usize] > 0 && magic_affixes.iter().all(|w| *w == 0) {
        return Err(err(
            "`magic`: the weights may not all be 0 while magic items are found".into(),
        ));
    }
    let (file, rare_spec) = rare_spec.get(first_file)?;
    let err = |m: String| ContentError::new(file, m);
    let most = 2 * MAX_PER_KIND as u32;
    let mut rare_counts = Vec::new();
    for [count, weight] in rare_spec.affixes {
        if !(1..=most).contains(&count) {
            return Err(err(format!(
                "`rare.affixes`: a rare item rolls 1 to {most} affixes, not {count}"
            )));
        }
        rare_counts.push((count as u8, weight));
    }
    if rarity[Rarity::Rare as usize] > 0 && rare_counts.iter().all(|(_, w)| *w == 0) {
        return Err(err(
            "`rare.affixes` needs a count with a weight above 0 while rares are found".into(),
        ));
    }
    let (file, names_spec) = names_spec.get(first_file)?;
    let err = |m: String| ContentError::new(file, m);
    let names = |list: &[String]| list.iter().all(|n| !n.trim().is_empty()) && !list.is_empty();
    if !names(&names_spec.first) {
        return Err(err("`names.first` needs words, none of them empty".into()));
    }
    let mut second_names = BTreeMap::new();
    for (kind, words) in names_spec.second {
        let kind = ItemKind::from_id(&kind)
            .ok_or_else(|| err(format!("`names.second`: unknown kind of item `{kind}`")))?;
        if !names(&words) {
            return Err(err(format!(
                "`names.second.{}` needs words, none of them empty",
                kind.id()
            )));
        }
        second_names.insert(kind, words);
    }
    if let Some(kind) = (ItemKind::ALL.iter().copied()).find(|k| !second_names.contains_key(k)) {
        return Err(err(format!(
            "`names.second` has no words for `{}`",
            kind.id()
        )));
    }

    let mut affixes = Affixes {
        rarity,
        magic_affixes,
        rare_counts,
        first_names: names_spec.first,
        second_names,
        ..Affixes::default()
    };
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (file, kind, spec) in specs {
        // Without skills (no `data/skills.toml`) skill affixes have nothing to raise: left out;
        // so are typed affixes without damage types (nothing to add or resist), and pooled ones
        // without resources (no pool to change).
        let effect = Effect::from_id(&spec.effect);
        if (effect == Some(Effect::Skill) && lookups.skill.is_none())
            || (effect.is_some_and(Effect::is_typed) && lookups.damage_type.is_none())
            || (effect.is_some_and(Effect::is_pooled) && lookups.resource.is_none())
        {
            continue;
        }
        let id = spec.id.clone();
        let def = spec.resolve(kind, lookups).map_err(|m| {
            ContentError::new(file, format!("{} `{id}`: {m}", kind.name().to_lowercase()))
        })?;
        if let Some(first) = seen.insert(id.clone(), file) {
            return Err(super::duplicate_id("affix", &id, file, first));
        }
        check_category(&affixes, &def).map_err(|m| ContentError::new(file, m))?;
        affixes.ids.insert(id, affixes.defs.len());
        affixes.defs.push(def);
    }
    Ok(affixes)
}

/// Refuses `def` when its category and the affixes read before it disagree: every affix of one
/// [`Group`] names one category (so one affix per category is one per group too), and a category
/// on both prefixes and suffixes is one group, the same effect either way.
fn check_category(affixes: &Affixes, def: &AffixDef) -> Result<(), String> {
    for other in affixes.all() {
        let same_category = other.category == def.category;
        if other.group() == def.group() && !same_category {
            return Err(format!(
                "{} `{}`: category `{}`, but `{}` does the same in `{}`",
                def.kind.name().to_lowercase(),
                def.id,
                def.category,
                other.id,
                other.category
            ));
        }
        if same_category && other.kind != def.kind && other.group() != def.group() {
            return Err(format!(
                "{} `{}`: category `{}` is on prefixes and suffixes, so both do one thing, \
                 but `{}` does another",
                def.kind.name().to_lowercase(),
                def.id,
                def.category,
                other.id
            ));
        }
    }
    Ok(())
}

impl AffixSpec {
    fn resolve(self, kind: AffixKind, lookups: &Lookups<'_>) -> Result<AffixDef, String> {
        let skill = |id: &str| lookups.skill.and_then(|f| f(id));
        let damage = |id: &str| lookups.damage_type.and_then(|f| f(id));
        let resource = |id: &str| lookups.resource.and_then(|f| f(id));
        let light = lookups.light;
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err("`id` and `name` must not be empty".into());
        }
        let effect = Effect::from_id(&self.effect)
            .ok_or_else(|| format!("unknown effect `{}`", self.effect))?;
        let words = self.category.split('_');
        if !(words.into_iter()).all(|w| !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase()))
        {
            return Err(format!(
                "`category` is lower-case words joined by `_`, not `{}`",
                self.category
            ));
        }
        let skill = match (effect, &self.skill) {
            (Effect::Skill, Some(id)) => {
                Some(skill(id).ok_or_else(|| format!("unknown skill `{id}`"))?)
            }
            (Effect::Skill, None) => return Err("effect `skill` names its `skill`".into()),
            (_, Some(_)) => return Err("`skill` is for affixes of effect `skill`".into()),
            (_, None) => None,
        };
        let damage_type = match (effect.is_typed(), &self.damage_type) {
            (true, Some(id)) => {
                Some(damage(id).ok_or_else(|| format!("unknown damage type `{id}`"))?)
            }
            (true, None) => {
                return Err(format!("effect `{}` names its `damage_type`", effect.id()));
            }
            (false, Some(_)) => {
                return Err(
                    "`damage_type` is for affixes of effect `elemental` or `resistance`".into(),
                );
            }
            (false, None) => None,
        };
        let resource = match (effect.is_pooled(), &self.resource) {
            (true, Some(id)) => {
                Some(resource(id).ok_or_else(|| format!("unknown resource `{id}`"))?)
            }
            (true, None) => return Err(format!("effect `{}` names its `resource`", effect.id())),
            (false, Some(_)) => {
                return Err("`resource` is for affixes of a resource's effects \
                            (`resource_max`, `resource_rate`, `active_stamina`, `resource_on_kill`)"
                    .into());
            }
            (false, None) => None,
        };
        if self.bases.is_empty() {
            return Err("`bases` names no kind of item".into());
        }
        let mut bases = Vec::new();
        for base in &self.bases {
            let k = ItemKind::from_id(base).ok_or_else(|| format!("unknown kind `{base}`"))?;
            if !effect.allowed_on(k) {
                return Err(format!(
                    "effect `{}` cannot be on {}",
                    effect.id(),
                    k.plural()
                ));
            }
            if bases.contains(&k) {
                return Err(format!("`{base}` is in `bases` twice"));
            }
            bases.push(k);
        }
        if self.tiers.is_empty() {
            return Err("an affix needs at least one tier".into());
        }
        if self.tiers.len() > MAX_TIERS {
            return Err(format!(
                "an affix has at most {MAX_TIERS} tiers, not {}",
                self.tiers.len()
            ));
        }
        let mut tiers = Vec::new();
        for (i, t) in self.tiers.into_iter().enumerate() {
            let n = i + 1;
            if t.level == 0 || t.level > MAX_ITEM_LEVEL {
                return Err(format!("tier {n}: `level` must be 1 to {MAX_ITEM_LEVEL}"));
            }
            if tiers.last().is_some_and(|p: &Tier| p.level >= t.level) {
                return Err(format!("tier {n}: levels must rise from tier to tier"));
            }
            let tier = match (effect, t.min, t.max, t.light) {
                (Effect::Light, None, None, Some(id)) => Tier {
                    level: t.level,
                    min: 0,
                    max: 0,
                    light: Some(light(&id).ok_or_else(|| format!("unknown light `{id}`"))?),
                },
                (Effect::Light, ..) => {
                    return Err(format!(
                        "tier {n}: a light's tier names a `light`, no values"
                    ));
                }
                (_, Some(min), Some(max), None) => {
                    let values = effect.values();
                    if min > max || !values.contains(&min) || !values.contains(&max) {
                        return Err(format!(
                            "tier {n}: `min` and `max` must be {} to {}, `min` at most `max`",
                            values.start(),
                            values.end()
                        ));
                    }
                    Tier {
                        level: t.level,
                        min,
                        max,
                        light: None,
                    }
                }
                _ => return Err(format!("tier {n} needs `min` and `max` (and no `light`)")),
            };
            tiers.push(tier);
        }
        Ok(AffixDef {
            id: self.id,
            name: self.name,
            kind,
            effect,
            category: self.category,
            skill,
            damage_type,
            resource,
            bases,
            weight: self.weight.unwrap_or(100),
            tiers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = r#"
[rarity]
normal = 70
magic = 25
rare = 5
[magic]
prefix = 40
suffix = 40
both = 20
[rare]
affixes = [[3, 40], [4, 35]]
[names]
first = ["Grim"]
[names.second]
weapon = ["Bite"]
launcher = ["Song"]
ammo = ["Sting"]
body = ["Shell"]
helmet = ["Crown"]
cloak = ["Shroud"]
gloves = ["Grip"]
boots = ["Stride"]
shield = ["Wall"]
ring = ["Loop"]
amulet = ["Charm"]
tool = ["Tooth"]
"#;

    fn parse_with(affixes: &str) -> Result<Affixes, ContentError> {
        let text = format!("{HEAD}\n{affixes}");
        parse_files([("data/affixes.toml", text.as_str())])
    }

    /// Parses `files` as the affix table, with a light `glow`, a skill `stealth`, a damage type
    /// `fire`, and resources `stamina` and `rage`.
    fn parse_files<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Affixes, ContentError> {
        let light = |id: &str| (id == "glow").then_some(LightId(0));
        let skill = |id: &str| (id == "stealth").then_some(SkillId(0));
        let damage = |id: &str| (id == "fire").then_some(DamageTypeId(0));
        let resource = |id: &str| match id {
            "stamina" => Some(ResourceId(0)),
            "rage" => Some(ResourceId(1)),
            _ => None,
        };
        let lookups = Lookups {
            light: &light,
            skill: Some(&skill),
            damage_type: Some(&damage),
            resource: Some(&resource),
        };
        parse(files, &lookups)
    }

    /// The table split over files: the settings in one, an entry a file in path order; a
    /// settings table in two files, no settings at all, and an id in two files are refused.
    #[test]
    fn affix_files_hold_the_settings_once_and_the_entries_in_path_order() {
        let sharp = "[[prefix]]\nid = \"sharp\"\nname = \"Sharp\"\neffect = \"damage\"\ncategory = \"damage\"\n\
                     bases = [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 1 }]\n";
        let keen = sharp.replace("sharp", "keen").replace("Sharp", "Keen");
        let a = parse_files([
            ("data/affixes.toml", HEAD),
            ("data/affixes/README.toml", "# Notes only.\n"),
            ("data/affixes/keen.toml", keen.as_str()),
            ("data/affixes/sharp.toml", sharp),
        ])
        .expect("valid");
        let ids: Vec<&str> = a.all().iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids, ["keen", "sharp"]);
        let err = parse_files([
            ("data/affixes.toml", HEAD),
            (
                "data/affixes/more.toml",
                "[rarity]\nnormal = 1\nmagic = 1\nrare = 1\n",
            ),
        ])
        .unwrap_err();
        assert_eq!(err.file, "data/affixes/more.toml");
        assert_eq!(
            err.message,
            "`[rarity]` is set here and in data/affixes.toml"
        );
        let err = parse_files([("data/affixes/sharp.toml", sharp)]).unwrap_err();
        assert_eq!(err.file, "data/affixes/sharp.toml");
        assert_eq!(err.message, "`[rarity]` is missing");
        let err = parse_files([
            ("data/affixes.toml", HEAD),
            ("data/affixes/a.toml", sharp),
            ("data/affixes/b.toml", sharp),
        ])
        .unwrap_err();
        assert_eq!(err.file, "data/affixes/b.toml");
        assert_eq!(
            err.message,
            "duplicate affix id `sharp` (also in data/affixes/a.toml)"
        );
    }

    #[test]
    fn affixes_parse_with_tiers_and_bases() {
        let a = parse_with(
            r#"
[[prefix]]
id = "sharp"
name = "Sharp"
effect = "damage"
category = "damage"
bases = ["weapon", "ammo"]
tiers = [{ level = 1, min = 1, max = 1 }, { level = 4, min = 2, max = 3 }]

[[suffix]]
id = "of_light"
name = "of Light"
effect = "light"
category = "light"
bases = ["ring"]
tiers = [{ level = 1, light = "glow" }]

[[suffix]]
id = "of_the_bear"
name = "of the Bear"
effect = "strength"
category = "strength"
bases = ["ring"]
weight = 50
tiers = [{ level = 1, min = 1, max = 2 }]
"#,
        )
        .expect("parses");
        let sharp = a.get("sharp").expect("sharp");
        assert_eq!(sharp.kind, AffixKind::Prefix);
        assert_eq!(sharp.effect, Effect::Damage);
        assert_eq!(sharp.tiers_at(1), 1);
        assert_eq!(sharp.tiers_at(9), 2);
        assert_eq!(sharp.tier(2).map(|t| (t.min, t.max)), Some((2, 3)));
        assert_eq!(
            a.get("of_light").and_then(|d| d.tiers[0].light),
            Some(LightId(0))
        );
        let bear = a.get("of_the_bear").expect("bear");
        assert_eq!(
            (bear.effect, bear.weight),
            (Effect::Stat(Stat::Strength), 50)
        );
        assert_eq!(a.all().len(), 3);
        assert_eq!(a.rarity_weight(Rarity::Magic), 25);
    }

    #[test]
    fn bad_affixes_are_reported() {
        let bad = [
            ("effect = \"luck\"\ncategory = \"luck\"", "unknown effect"),
            (
                "effect = \"armour\"\ncategory = \"armour\"\nbases = [\"ring\"]",
                "cannot be on rings",
            ),
        ];
        for (field, needle) in bad {
            let text = format!(
                "[[prefix]]\nid = \"x\"\nname = \"X\"\n{field}\n{}tiers = [{{ level = 1, min = 1, \
                 max = 1 }}]\n",
                if field.contains("bases") {
                    ""
                } else {
                    "bases = [\"weapon\"]\n"
                }
            );
            let e = parse_with(&text).expect_err(needle);
            assert!(e.to_string().contains(needle), "{e}");
        }
        let cases = [
            "tiers = []",
            "tiers = [{ level = 3, min = 1, max = 1 }, { level = 2, min = 1, max = 1 }]",
            "tiers = [{ level = 1, min = 3, max = 1 }]",
            "tiers = [{ level = 1, min = 1, max = 99 }]",
            "tiers = [{ level = 1, light = \"glow\" }]",
            "tiers = [{ level = 0, min = 1, max = 1 }]",
        ];
        for tiers in cases {
            let text = format!(
                "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\ncategory = \"damage\"\nbases = \
                 [\"weapon\"]\n{tiers}\n"
            );
            assert!(parse_with(&text).is_err(), "{tiers}");
        }
        let twice = "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\ncategory = \"damage\"\nbases = \
                     [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 1 }]\n";
        let e = parse_with(&format!("{twice}{twice}")).expect_err("duplicate");
        assert!(e.to_string().contains("duplicate"), "{e}");
    }

    /// Every affix names a category; one group is one category, and a category on both kinds is
    /// one group.
    #[test]
    fn affix_categories_are_checked() {
        let affix = |kind: &str, id: &str, effect: &str, category: &str| {
            format!(
                "[[{kind}]]\nid = \"{id}\"\nname = \"X\"\neffect = \"{effect}\"\n\
                 category = \"{category}\"\nbases = [\"weapon\"]\n\
                 tiers = [{{ level = 1, min = 1, max = 1 }}]\n"
            )
        };
        let good = [
            affix("prefix", "sharp", "damage", "physical_damage"),
            affix("prefix", "heavy", "extra_damage", "physical_damage")
                .replace("min = 1, max = 1", "min = 2, max = 3"),
            affix("prefix", "glinting", "to_hit", "accuracy"),
            affix("suffix", "of_precision", "to_hit", "accuracy"),
        ]
        .concat();
        let a = parse_with(&good).expect("valid");
        assert_eq!(
            a.get("heavy").map(|d| d.category.as_str()),
            Some("physical_damage")
        );
        let missing = "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\nbases = \
                       [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 1 }]\n";
        let e = parse_with(missing).expect_err("no category");
        assert!(e.to_string().contains("category"), "{e}");
        let bad = [
            (
                affix("prefix", "x", "damage", "Physical damage"),
                "lower-case words",
            ),
            (affix("prefix", "x", "damage", ""), "lower-case words"),
            (
                [
                    affix("prefix", "sharp", "damage", "physical_damage"),
                    affix("prefix", "vicious", "damage", "brutality"),
                ]
                .concat(),
                "`sharp` does the same in `physical_damage`",
            ),
            (
                [
                    affix("prefix", "sharp", "damage", "physical_damage"),
                    affix("suffix", "of_heft", "extra_damage", "physical_damage")
                        .replace("min = 1, max = 1", "min = 2, max = 3"),
                ]
                .concat(),
                "on prefixes and suffixes",
            ),
        ];
        for (text, needle) in bad {
            let e = parse_with(&text).expect_err(needle);
            assert!(e.to_string().contains(needle), "{e}");
        }
    }

    /// One affix file's text: `effect` on `bases` with `extra` lines and one tier of `tier`.
    fn one(kind: &str, id: &str, effect: &str, bases: &str, extra: &str, tier: &str) -> String {
        format!(
            "[[{kind}]]\nid = \"{id}\"\nname = \"X\"\neffect = \"{effect}\"\ncategory = \"{id}\"\n\
             bases = [{bases}]\n{extra}tiers = [{tier}]\n"
        )
    }

    /// The effects of docs/design/affixes-systems.md (§4 to §6) parse, each on a base it may be
    /// on, the pooled ones naming their resource.
    #[test]
    fn the_system_effects_parse() {
        let plain = [
            ("mana_cost", "\"ring\""),
            ("spell_ease", "\"body\", \"helmet\", \"shield\""),
            ("spell_to_hit", "\"weapon\""),
            ("spell_threat", "\"gloves\""),
            ("spell_lift", "\"amulet\""),
            ("spell_reach", "\"cloak\""),
            ("spell_duration", "\"helmet\""),
            ("mana_on_kill", "\"launcher\""),
            ("wand_level", "\"gloves\""),
            ("door_kick", "\"boots\""),
            ("dig_percent", "\"tool\", \"gloves\""),
            ("hunger", "\"cloak\""),
            ("trap_damage", "\"boots\""),
            ("disease_check", "\"body\""),
            ("potion_kept", "\"ring\""),
        ];
        let pooled = [
            "resource_max",
            "resource_rate",
            "active_stamina",
            "resource_on_kill",
        ];
        let tier = "{ level = 1, min = 1, max = 2 }";
        let mut text = String::new();
        for (effect, bases) in plain {
            text += &one("suffix", effect, effect, bases, "", tier);
        }
        for effect in pooled {
            let extra = "resource = \"stamina\"\n";
            text += &one("suffix", effect, effect, "\"ring\"", extra, tier);
        }
        let a = parse_with(&text).expect("the system effects parse");
        for (effect, _) in plain {
            let def = a.get(effect).expect(effect);
            assert_eq!(def.effect.id(), effect);
            assert_eq!(
                def.group(),
                Group::of(def.effect),
                "{effect} names nothing else"
            );
        }
        for effect in pooled {
            let def = a.get(effect).expect(effect);
            assert!(def.effect.is_pooled());
            assert_eq!(def.resource, Some(ResourceId(0)), "{effect}: stamina");
        }
        let ease = a.get("spell_ease").expect("spell_ease").effect;
        assert!(ease.is_local_on(ItemKind::Body), "a piece's own surcharge");
        assert!(
            !Effect::ManaCost.is_local_on(ItemKind::Weapon),
            "no attack effect"
        );
        assert_eq!(
            a.get("dig_percent").map(|d| d.bases.clone()),
            Some(vec![ItemKind::Tool, ItemKind::Gloves])
        );
    }

    /// A pooled effect names a known resource; nothing else names one; Mage-wrought's effect is
    /// only on what may hinder casting.
    #[test]
    fn resources_and_the_spell_ease_bases_are_checked() {
        let tier = "{ level = 1, min = 1, max = 1 }";
        let bad = [
            (
                one("prefix", "x", "resource_max", "\"ring\"", "", tier),
                "effect `resource_max` names its `resource`",
            ),
            (
                one(
                    "prefix",
                    "x",
                    "resource_rate",
                    "\"ring\"",
                    "resource = \"mana\"\n",
                    tier,
                ),
                "unknown resource `mana`",
            ),
            (
                one(
                    "prefix",
                    "x",
                    "mana_cost",
                    "\"ring\"",
                    "resource = \"stamina\"\n",
                    tier,
                ),
                "`resource` is for affixes of a resource's effects",
            ),
            (
                one("prefix", "x", "spell_ease", "\"cloak\"", "", tier),
                "effect `spell_ease` cannot be on cloaks",
            ),
        ];
        for (text, needle) in bad {
            let e = parse_with(&text).expect_err(needle);
            assert!(e.to_string().contains(needle), "{e}");
        }
        // One group per (effect, resource): a stamina and a rage maximum are two categories, two
        // stamina maxima one.
        let stamina = "resource = \"stamina\"\n";
        let rage = "resource = \"rage\"\n";
        let two = [
            one(
                "prefix",
                "tireless",
                "resource_max",
                "\"ring\"",
                stamina,
                tier,
            ),
            one("suffix", "of_fury", "resource_max", "\"ring\"", rage, tier),
        ]
        .concat();
        let a = parse_with(&two).expect("two pools, two groups");
        let (tireless, fury) = (a.get("tireless").unwrap(), a.get("of_fury").unwrap());
        assert_ne!(tireless.group(), fury.group());
        assert_eq!(tireless.group().resource, Some(ResourceId(0)));
        let same = [
            one(
                "prefix",
                "tireless",
                "resource_max",
                "\"ring\"",
                stamina,
                tier,
            ),
            one("prefix", "hardy", "resource_max", "\"ring\"", stamina, tier),
        ]
        .concat();
        let e = parse_with(&same).expect_err("one group, two categories");
        assert!(e.to_string().contains("does the same"), "{e}");
        // Without resources in content, pooled affixes are left out, as skill affixes are
        // without skills.
        let light = |_: &str| None;
        let lookups = Lookups {
            light: &light,
            skill: None,
            damage_type: None,
            resource: None,
        };
        let text = format!("{HEAD}\n{two}");
        let a = parse([("data/affixes.toml", text.as_str())], &lookups).expect("valid");
        assert!(a.is_empty());
    }

    /// An affix has at most twelve tiers.
    #[test]
    fn an_affix_has_at_most_twelve_tiers() {
        let ladder = |n: u8| -> String {
            let tiers: Vec<String> = (1..=n)
                .map(|l| format!("{{ level = {l}, min = 1, max = 2 }}"))
                .collect();
            one("prefix", "x", "damage", "\"weapon\"", "", &tiers.join(", "))
        };
        let a = parse_with(&ladder(12)).expect("twelve tiers");
        assert_eq!(a.get("x").map(|d| d.tiers.len()), Some(MAX_TIERS));
        let e = parse_with(&ladder(13)).expect_err("thirteen");
        assert!(e.to_string().contains("at most 12 tiers, not 13"), "{e}");
    }

    /// Tools are a kind of item: rare names have words for them, and affixes may roll on them.
    #[test]
    fn tools_are_a_kind_of_item() {
        assert_eq!(ItemKind::from_id("tool"), Some(ItemKind::Tool));
        assert_eq!(ItemKind::ALL.last(), Some(&ItemKind::Tool));
        assert!(!ItemKind::Tool.attacks() && !ItemKind::Tool.is_armour());
        let tier = "{ level = 1, min = 10, max = 15 }";
        let a = parse_with(&one("prefix", "x", "weight", "\"tool\"", "", tier)).expect("valid");
        assert_eq!(a.second_names.get(&ItemKind::Tool).map(Vec::len), Some(1));
        assert!(
            a.get("x").unwrap().effect.is_local_on(ItemKind::Tool),
            "a lighter pick"
        );
        for effect in ["protection", "armour", "spell_ease", "range"] {
            let text = one("prefix", "x", effect, "\"tool\"", "", tier);
            assert!(parse_with(&text).is_err(), "{effect} on a tool");
        }
        let no_words = HEAD.replace("tool = [\"Tooth\"]\n", "");
        let e = parse_files([("data/affixes.toml", no_words.as_str())]).unwrap_err();
        assert_eq!(e.message, "`names.second` has no words for `tool`");
    }

    #[test]
    fn local_and_global_effects() {
        assert!(Effect::ToHit.is_local_on(ItemKind::Weapon));
        assert!(Effect::ToHit.is_local_on(ItemKind::Ammo));
        assert!(!Effect::ToHit.is_local_on(ItemKind::Gloves));
        assert!(Effect::Armour.is_local_on(ItemKind::Body));
        assert!(Effect::Protection.is_local_on(ItemKind::Boots));
        let protected: Vec<_> = (ItemKind::ALL.iter().copied())
            .filter(|&k| Effect::Protection.allowed_on(k))
            .collect();
        assert_eq!(
            protected,
            [
                ItemKind::Body,
                ItemKind::Helmet,
                ItemKind::Cloak,
                ItemKind::Gloves,
                ItemKind::Boots
            ],
            "armour, never a shield"
        );
        assert!(Effect::Weight.is_local_on(ItemKind::Ring));
        assert!(!Effect::Stat(Stat::Strength).is_local_on(ItemKind::Weapon));
        assert!(!Effect::Light.is_local_on(ItemKind::Weapon));
        assert_eq!(Effect::all().len(), Effect::COUNT);
        for (i, effect) in Effect::all().into_iter().enumerate() {
            assert_eq!(Effect::from_id(effect.id()), Some(effect));
            assert_eq!(effect.index(), i);
        }
    }
}
