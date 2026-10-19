//! Magic item content (`assets/data/affixes.toml`, spec §7 "Magic items"): the prefixes and
//! suffixes magic and rare items roll, how often loot is normal, magic, or rare, how many affixes
//! each rolls, and the words rare names are made of.
//!
//! Every object that may be found magic says what kind of item it is (`kind` in
//! `data/objects.toml`, [`ItemKind`]): a weapon, a missile weapon, ammunition, body armour, a
//! helmet, a cloak, gloves, boots, a shield, a ring, or an amulet. An affix names the kinds it may
//! roll on (`bases`), the one [`Effect`] it has, and its **tiers**: from which item level each is
//! found, and the range its value is rolled in (a light's tier names a light preset instead). An
//! item never has two affixes with the same effect.
//!
//! Attack effects (to-hit, damage, criticals, attack speed, range, ...) on a weapon, a missile
//! weapon, or ammunition are **local**: they change attacks made with that item only. Elsewhere
//! (gloves, rings, amulets) they are **global**: every attack. Effects on the item itself (its
//! Armour, Shield, Parry, the Dexterity it allows, its weight) are always local, and the rest
//! (stats, hit points, Dodge, Deflection, natural armour, regeneration, moving, carrying,
//! stealth, light) always global. Effects apply while the item is worn or wielded.

use super::{ContentError, LightId};
use crate::skills::SkillId;
use crate::stats::Stat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The highest item level: dungeon depth gives it (spec §8 "Magic items").
pub const MAX_ITEM_LEVEL: u8 = 99;
/// The most prefixes, and the most suffixes, an item may have (a rare: three of each).
pub const MAX_PER_KIND: usize = 3;
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
}

impl ItemKind {
    pub const ALL: [ItemKind; 11] = [
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
        }
    }

    pub fn from_id(id: &str) -> Option<ItemKind> {
        ItemKind::ALL.into_iter().find(|k| k.id() == id)
    }

    /// Whether attacks are made with it: then its attack effects are local, changing only
    /// attacks made with it.
    pub const fn attacks(self) -> bool {
        matches!(self, ItemKind::Weapon | ItemKind::Launcher | ItemKind::Ammo)
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
    /// A stat's score.
    Stat(Stat),
    /// Percent off the cost of steps.
    MoveSpeed,
    /// Kilograms of carrying limit.
    Carry,
    /// Percent quieter footsteps.
    Stealth,
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
            Effect::Threat,
            Effect::Crit,
            Effect::AttackSpeed,
            Effect::Range,
            Effect::Draw,
            Effect::Durability,
            Effect::Parry,
            Effect::Armour,
            Effect::Shield,
            Effect::MaxDex,
            Effect::Weight,
            Effect::Dodge,
            Effect::Deflection,
            Effect::Natural,
            Effect::MaxHp,
            Effect::Regeneration,
        ];
        all.extend(Stat::ALL.map(Effect::Stat));
        all.extend([
            Effect::MoveSpeed,
            Effect::Carry,
            Effect::Stealth,
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
            Effect::Threat => "threat",
            Effect::Crit => "crit",
            Effect::AttackSpeed => "attack_speed",
            Effect::Range => "range",
            Effect::Draw => "draw",
            Effect::Durability => "durability",
            Effect::Parry => "parry",
            Effect::Armour => "armour",
            Effect::Shield => "shield",
            Effect::MaxDex => "max_dex",
            Effect::Weight => "weight",
            Effect::Dodge => "dodge",
            Effect::Deflection => "deflection",
            Effect::Natural => "natural",
            Effect::MaxHp => "max_hp",
            Effect::Regeneration => "regeneration",
            Effect::Stat(stat) => stat.id(),
            Effect::MoveSpeed => "move_speed",
            Effect::Carry => "carry",
            Effect::Stealth => "stealth",
            Effect::Light => "light",
            Effect::Skill => "skill",
        }
    }

    pub fn from_id(id: &str) -> Option<Effect> {
        Effect::all().into_iter().find(|e| e.id() == id)
    }

    /// How many effects there are.
    pub const COUNT: usize = 24 + Stat::COUNT;

    /// Its place in [`Effect::all`].
    pub const fn index(self) -> usize {
        match self {
            Effect::ToHit => 0,
            Effect::Damage => 1,
            Effect::ExtraDamage => 2,
            Effect::Threat => 3,
            Effect::Crit => 4,
            Effect::AttackSpeed => 5,
            Effect::Range => 6,
            Effect::Draw => 7,
            Effect::Durability => 8,
            Effect::Parry => 9,
            Effect::Armour => 10,
            Effect::Shield => 11,
            Effect::MaxDex => 12,
            Effect::Weight => 13,
            Effect::Dodge => 14,
            Effect::Deflection => 15,
            Effect::Natural => 16,
            Effect::MaxHp => 17,
            Effect::Regeneration => 18,
            Effect::Stat(stat) => 19 + stat as usize,
            Effect::MoveSpeed => 19 + Stat::COUNT,
            Effect::Carry => 20 + Stat::COUNT,
            Effect::Stealth => 21 + Stat::COUNT,
            Effect::Light => 22 + Stat::COUNT,
            Effect::Skill => 23 + Stat::COUNT,
        }
    }

    /// The values a tier may roll (a light's tier has none).
    pub const fn values(self) -> RangeInclusive<i32> {
        match self {
            Effect::ExtraDamage => 2..=20,
            Effect::Threat | Effect::Crit => 1..=3,
            Effect::AttackSpeed | Effect::MoveSpeed => 1..=MAX_SPEED,
            Effect::Durability | Effect::Weight | Effect::Stealth => 1..=90,
            Effect::MaxHp | Effect::Carry | Effect::Skill => 1..=100,
            Effect::Regeneration => 1..=300,
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
                | Effect::Threat
                | Effect::Crit
                | Effect::AttackSpeed
                | Effect::Range
                | Effect::Draw
                | Effect::Durability
        )
    }

    /// Whether it changes the item itself wherever it is (its Armour, Shield, Parry, the
    /// Dexterity it allows, its weight).
    pub const fn is_item(self) -> bool {
        matches!(
            self,
            Effect::Parry | Effect::Armour | Effect::Shield | Effect::MaxDex | Effect::Weight
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
            Effect::ExtraDamage | Effect::Range | Effect::Draw | Effect::Durability => {
                kind.attacks()
            }
            Effect::Parry => kind == ItemKind::Weapon,
            Effect::Armour => kind == ItemKind::Body,
            Effect::Shield => kind == ItemKind::Shield,
            Effect::MaxDex => matches!(kind, ItemKind::Body | ItemKind::Shield),
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
    /// The skill a [`Effect::Skill`] affix adds to.
    pub skill: Option<SkillId>,
    /// What kinds of item may roll it.
    pub bases: Vec<ItemKind>,
    /// How often it rolls among those that may (default 100).
    pub weight: u32,
    /// Tier 1 first, by item level.
    pub tiers: Vec<Tier>,
}

impl AffixDef {
    /// What it changes, for the rule that an item has at most one affix of each: its effect, and
    /// for a skill affix which skill.
    pub fn group(&self) -> (Effect, Option<SkillId>) {
        (self.effect, self.skill)
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
    /// The chance in percent that a magic item has both a prefix and a suffix (else one of
    /// either).
    pub magic_both: u32,
    /// How many affixes a rare item rolls, each with its weight.
    pub rare_counts: Vec<(u8, u32)>,
    /// The first words of rare names ("Grim").
    pub first_names: Vec<String>,
    /// The second words of rare names, by kind of item ("Bite" for weapons).
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixFile {
    rarity: RaritySpec,
    magic: MagicSpec,
    rare: RareSpec,
    names: NamesSpec,
    #[serde(default)]
    prefix: Vec<AffixSpec>,
    #[serde(default)]
    suffix: Vec<AffixSpec>,
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
    bases: Vec<String>,
    weight: Option<u32>,
    /// The skill an affix of effect `skill` adds to (an id in `data/skills.toml`).
    skill: Option<String>,
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

/// Parses `data/affixes.toml`; `light` resolves light preset ids and `skill` skill ids
/// (`has_skills`: whether content has any; without, skill affixes are left out).
pub(super) fn parse(
    file: &str,
    text: &str,
    light: impl Fn(&str) -> Option<LightId>,
    (skill, has_skills): (impl Fn(&str) -> Option<SkillId>, bool),
) -> Result<Affixes, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: AffixFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let rarity = [
        parsed.rarity.normal,
        parsed.rarity.magic,
        parsed.rarity.rare,
    ];
    if rarity.iter().sum::<u32>() == 0 {
        return Err(err("`rarity`: the weights may not all be 0".into()));
    }
    if parsed.magic.both > 100 {
        return Err(err("`magic.both` is a chance in percent, 0 to 100".into()));
    }
    let most = 2 * MAX_PER_KIND as u32;
    let mut rare_counts = Vec::new();
    for [count, weight] in parsed.rare.affixes {
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
    let names = |list: &[String]| list.iter().all(|n| !n.trim().is_empty()) && !list.is_empty();
    if !names(&parsed.names.first) {
        return Err(err("`names.first` needs words, none of them empty".into()));
    }
    let mut second_names = BTreeMap::new();
    for (kind, words) in parsed.names.second {
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
    if let Some(kind) = ItemKind::ALL
        .into_iter()
        .find(|k| !second_names.contains_key(k))
    {
        return Err(err(format!(
            "`names.second` has no words for `{}`",
            kind.id()
        )));
    }

    let mut affixes = Affixes {
        rarity,
        magic_both: parsed.magic.both,
        rare_counts,
        first_names: parsed.names.first,
        second_names,
        ..Affixes::default()
    };
    let specs = (parsed.prefix.into_iter().map(|s| (AffixKind::Prefix, s)))
        .chain(parsed.suffix.into_iter().map(|s| (AffixKind::Suffix, s)));
    for (kind, spec) in specs {
        // Without skills (no `data/skills.toml`) skill affixes have nothing to raise: left out.
        if spec.effect == Effect::Skill.id() && !has_skills {
            continue;
        }
        let id = spec.id.clone();
        let def = spec
            .resolve(kind, &light, &skill)
            .map_err(|m| err(format!("{} `{id}`: {m}", kind.name().to_lowercase())))?;
        if affixes.ids.insert(id.clone(), affixes.defs.len()).is_some() {
            return Err(err(format!("duplicate affix id `{id}`")));
        }
        affixes.defs.push(def);
    }
    Ok(affixes)
}

impl AffixSpec {
    fn resolve(
        self,
        kind: AffixKind,
        light: &impl Fn(&str) -> Option<LightId>,
        skill: &impl Fn(&str) -> Option<SkillId>,
    ) -> Result<AffixDef, String> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err("`id` and `name` must not be empty".into());
        }
        let effect = Effect::from_id(&self.effect)
            .ok_or_else(|| format!("unknown effect `{}`", self.effect))?;
        let skill = match (effect, &self.skill) {
            (Effect::Skill, Some(id)) => {
                Some(skill(id).ok_or_else(|| format!("unknown skill `{id}`"))?)
            }
            (Effect::Skill, None) => return Err("effect `skill` names its `skill`".into()),
            (_, Some(_)) => return Err("`skill` is for affixes of effect `skill`".into()),
            (_, None) => None,
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
            skill,
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
both = 25
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
"#;

    fn parse_with(affixes: &str) -> Result<Affixes, ContentError> {
        let text = format!("{HEAD}\n{affixes}");
        parse(
            "data/affixes.toml",
            &text,
            |id| (id == "glow").then_some(LightId(0)),
            (|id: &str| (id == "stealth").then_some(SkillId(0)), true),
        )
    }

    #[test]
    fn affixes_parse_with_tiers_and_bases() {
        let a = parse_with(
            r#"
[[prefix]]
id = "sharp"
name = "Sharp"
effect = "damage"
bases = ["weapon", "ammo"]
tiers = [{ level = 1, min = 1, max = 1 }, { level = 4, min = 2, max = 3 }]

[[suffix]]
id = "of_light"
name = "of Light"
effect = "light"
bases = ["ring"]
tiers = [{ level = 1, light = "glow" }]

[[suffix]]
id = "of_the_bear"
name = "of the Bear"
effect = "strength"
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
            ("effect = \"luck\"", "unknown effect"),
            (
                "effect = \"armour\"\nbases = [\"ring\"]",
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
                "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\nbases = \
                 [\"weapon\"]\n{tiers}\n"
            );
            assert!(parse_with(&text).is_err(), "{tiers}");
        }
        let twice = "[[prefix]]\nid = \"x\"\nname = \"X\"\neffect = \"damage\"\nbases = \
                     [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 1 }]\n";
        let e = parse_with(&format!("{twice}{twice}")).expect_err("duplicate");
        assert!(e.to_string().contains("duplicate"), "{e}");
    }

    #[test]
    fn local_and_global_effects() {
        assert!(Effect::ToHit.is_local_on(ItemKind::Weapon));
        assert!(Effect::ToHit.is_local_on(ItemKind::Ammo));
        assert!(!Effect::ToHit.is_local_on(ItemKind::Gloves));
        assert!(Effect::Armour.is_local_on(ItemKind::Body));
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
