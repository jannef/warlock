//! Object definitions (`assets/data/objects/`, one file per object, docs/dev/content.md; spec
//! §7): the things that lie on the floor and
//! are picked up and carried.
//!
//! Every object defines the glyph it is drawn with, its color, what it is made of (`materials`,
//! its parts, the first the main one: [`super::part`]), and its weight in grams. Its name shows
//! its notable parts (gems and precious metals) that the written name does not mention yet, and
//! its value follows its parts' materials ([`ObjectDef::value_with`]). Glyphs follow ADOM
//! loosely: `)` weapons, `/` missiles, `}` missile weapons, `[` armor, `(` tools, `!` potions,
//! `?` scrolls, `%` food, `*` stones and gems, `$` coins, `=` rings, `"` amulets. Every object
//! also names its item class (`class`, an id in `data/item_classes.toml`, [`super::item_class`]):
//! what item lists group it under and shops trade it as; an object with a magic item `kind` is of
//! the class that lists the kind.
//!
//! Things that can be worn or wielded say where they go (`slot`, the id or `fits` of a slot in
//! `data/slots.toml`: `slot = "body"`, `slot = "ring"` for either ring slot); a `two_handed` one
//! takes both hands, and a `light_weapon` (a dagger) goes in any hand: the off hand too, for
//! fighting with two weapons (spec §8 "Fighting styles"). Missiles say what kind of ammunition they are (`ammo = "arrow"`) and missile
//! weapons what they shoot (`launches = "arrow"`): launchers and ammunition pair by that kind.
//!
//! Weapons and armour say what they do in a fight (spec §8 "Combat"): a weapon its `damage` dice
//! (a launcher's carry its draw weight, `1d8+2`), its critical (`threat`, the lowest natural roll
//! that threatens, default 22; `crit`, the damage multiplier, default 2), an optional `to_hit`,
//! a `parry` bonus to Defence while wielded, and for bows the `strength` they need; armour its
//! `armour` bonus and the most Dexterity bonus it allows (`max_dex`), and its `protection` (the
//! damage it soaks off every hit, which adds up across what is worn); a shield its `shield` bonus,
//! and the `bash` dice of a blow with it (spec §8 "Weapon skills": the shield bash). A weapon or
//! a missile may have `penetration`: that much of a target's Protection it ignores. Armour and
//! shields may have `spell_cost_percent`: worn, a spell costs that much more mana (spec §8
//! "Spells"; what is worn adds up).
//!
//! Damage types (spec §8 "Damage types"): every weapon (and a shield that bashes) names the
//! kind of damage it does (`damage_type`, an id in `data/damage_types.toml`, or two: a sword's
//! `["slashing", "piercing"]` does whichever is better against its target), and may name the
//! `verb` its blows are said with ("cleaves") instead of the kind's; a weapon or a missile may
//! add damage of another kind (`added = { type = "fire", damage = "1d4" }`: the flaming arrow).
//! Anything worn or held may `resist` kinds in percent (`resist = { cold = 20 }`), and armour
//! with Protection may stand better or worse against a physical kind (`protects = { slashing =
//! 1, piercing = -1 }`).
//!
//! Every weapon names its weapon skill (`category`, an id in `data/weapon_skills.toml`; spec §8
//! "Weapon skills"): a skill of role `melee` for what is wielded in a hand, `launcher` for a
//! missile weapon, `thrown` for what is only thrown (a rock).
//!
//! Keys (spec §8 "Doors") say which locks they open: `key = "pantry"` opens the locks with that
//! lock id, `key = "*"` (a skeleton key) every lock.
//!
//! Ranged combat (spec §8 "Ranged combat"): a missile weapon shoots as far as its `range` (and
//! one that `reloads`, a crossbow, takes the `reload` action after every shot); anything else
//! with a `range` is a throwing weapon, thrown that far, its `damage` its own (a rock needs no
//! slot). What flies may `break` on a hit (a percent chance), and what is `lit` as it is loosed
//! flies and lands as that other object: a flaming arrow lands as a burning arrow, which gives
//! the `light` of a preset where it lies for `burns` seconds, then is gone. The rest is
//! presentation only: the `missile` glyph it flies as (pointing right, turned along its flight),
//! whether it `spins`, how much it `glow`s where it lies, and its `particles` (`ambient` while
//! it lies in view on top of its pile, `flight` while it flies).
//!
//! Base items (spec §8 "Magic items"): an object that may be found magic says what `kind` of
//! item it is ([`ItemKind`]: a weapon, a missile weapon, ammunition, body armour, a helmet, a
//! cloak, gloves, boots, a shield, a ring, or an amulet), the item `level` from which loot holds
//! it (default 1), and how often (`frequency`, default 100; 0: never). Weapons may cost more or
//! less than a normal attack (`attack_cost`, instead of the `melee` or `shoot` action's).
//!
//! Food (spec §8 "Hunger") says what it gives, how long it takes to eat, and how long it keeps
//! and lasts: `food = { nutrition = 1500, turns = 4, keeps = "30d", lasts = "60d" }` (it rots
//! from when it was made, [`Stack::died`], [`crate::rot`]). Corpses are objects too, one made for
//! each creature type that leaves one ([`ObjectDef::corpse_of`]): "kobold corpse", `%` in the
//! creature's color, weighing what the creature does, feeding and keeping by its size. Level
//! files cannot place them: they lie where creatures died, each with the moment it died, from
//! which it rots.
//!
//! Digging tools (spec §8 "Digging") have a `dig` power: the work a dig with one does against a
//! wall's hardness (a pick-axe 10, at Strength 10). They go in a slot (the Tool slot).
//!
//! Scrolls (spec §8 "Scrolls") say what reading one does (`read`, [`super::scroll`]): cast a
//! spell the reader aims, with the traps' rules, or identify an item. They are loot too (`level`,
//! `frequency`), never magic.

use super::action::check_cost;
use super::affix::{ItemKind, MAX_ITEM_LEVEL};
use super::creature::CreatureDef;
use super::damage_type::TypeNames;
use super::hunger::Food;
use super::item_class::{ItemClassDef, ItemClassId};
use super::light::LightId;
use super::material::MaterialId;
use super::part::{self, Part, PartEntry, PartId};
use super::particles::ParticleEmitter;
use super::potion::{Drink, DrinkSpec};
use super::rod::{Rod, RodSpec};
use super::scroll::{ReadSpec, Reading};
use super::slot::SlotDef;
use super::wand::{Wand, WandSpec};
use super::weapon_skill::{Role, WeaponSkillId, WeaponSkillTable};
use super::{Content, ContentError, CreatureId, DamageTypeId};
use crate::color::Rgb;
use crate::combat::{Attack, CRITS, DEFAULT_CRIT, DEFAULT_THREAT, THREATS};
use crate::damage::{Kinds, MAX_RESISTANCE, MIN_RESISTANCE, Rider};
use crate::dice::Dice;
use crate::door::Key;
use crate::item_parts::ItemPart;
use crate::object::Stack;
use crate::time::{DAY, Duration};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The largest Defence bonus one object may give, and the largest `to_hit` either way.
pub const MAX_BONUS: i32 = 20;
/// The farthest anything shoots or is thrown, in cells: as far as anyone sees.
pub const MAX_RANGE: i32 = crate::perception::VIEW_RADIUS;
/// The longest anything burns where it lies, in seconds: a day.
pub const MAX_BURN: u32 = DAY as u32;
/// The most power a digging tool may have (a pick-axe has 10).
pub const MAX_DIG: u16 = 100;
/// The most one object may be worth, in gold coins.
pub const MAX_VALUE: u32 = 1_000_000;

/// Index of an object definition in [`super::Content::object_defs`].
///
/// Ids are assigned in file order when content loads, like terrain ids; saves record the content
/// fingerprint to catch a reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectId(pub u16);

/// Most of one object a level file may place with `"id*N"`.
pub const MAX_PLACED: u32 = 9999;

/// What one weapon of each weapon skill (`category`, an id in `data/weapon_skills.toml`) is
/// called, for item descriptions (spec §8 "Magic items"): a skill not listed is called by its
/// name.
pub const WEAPON_NOUNS: [(&str, &str); 13] = [
    ("daggers", "dagger"),
    ("short_blades", "short blade"),
    ("long_blades", "long blade"),
    ("great_blades", "great blade"),
    ("axes", "axe"),
    ("maces", "mace or club"),
    ("spears", "spear"),
    ("polearms", "polearm"),
    ("staves", "staff"),
    ("bows", "bow"),
    ("crossbows", "crossbow"),
    ("slings", "sling"),
    ("thrown", "thrown weapon"),
];

/// What one weapon of the weapon skill with id `id` is called: "long blade".
pub fn category_name(id: &str) -> Option<&'static str> {
    (WEAPON_NOUNS.iter())
        .find(|(c, _)| *c == id)
        .map(|(_, name)| *name)
}

/// What one weapon like `def` is called by its weapon skill ([`WEAPON_NOUNS`], else the skill's
/// name): "long blade". `None` for what names no weapon skill.
pub fn weapon_noun(def: &ObjectDef, content: &Content) -> Option<String> {
    let skill = content.weapon_skills().skill(def.skill?);
    Some(category_name(&skill.id).map_or_else(|| skill.name.to_lowercase(), str::to_string))
}

/// An object type as written in content files.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObjectSpec {
    pub id: String,
    pub name: String,
    /// Plural name; the name plus `s` if absent.
    pub plural: Option<String>,
    pub glyph: char,
    pub fg: Rgb,
    /// Its item class, an id in `data/item_classes.toml`: what kind of thing it is.
    pub class: String,
    /// What it is made of: its parts, the first the main one, each a material id (`["steel"]`)
    /// or a table (`{ part = "setting", of = "ruby", share = 10 }`; [`super::part`]).
    #[serde(default)]
    pub materials: Vec<PartEntry>,
    /// What its main part may be made of (#194, the shorthand for its table's `made_of`): groups
    /// and materials ([`part::construction`]).
    pub made_of: Option<Vec<String>>,
    /// The single material objects had before parts: refused, with a pointer to `materials`.
    pub material: Option<String>,
    /// The part a gem is set as when loot ornaments it (a part id: `tip`), instead of its
    /// class's in `data/gems.toml` (spec §7 "Gems").
    pub ornament: Option<String>,
    /// Grams, at least 1.
    pub weight: u32,
    /// What one is worth, in gold coins (shops; spec §7 "Shops").
    pub value: u32,
    #[serde(default)]
    pub stackable: bool,
    /// Where it is worn or wielded: a slot's `fits` (its id unless it says otherwise).
    pub slot: Option<String>,
    /// Takes both hands (only for things that go in a hand).
    #[serde(default)]
    pub two_handed: bool,
    /// A light one-handed weapon: it goes in any hand, the off hand too (two-weapon fighting).
    #[serde(default)]
    pub light_weapon: bool,
    /// The kind of ammunition it is ("arrow"), for launchers of that kind.
    pub ammo: Option<String>,
    /// The kind of ammunition it shoots, if it is a missile weapon ("arrow").
    pub launches: Option<String>,
    /// A weapon's damage dice: of a blow with it, or of a shot from a launcher.
    pub damage: Option<Dice>,
    /// The lowest natural roll that threatens a critical (default 22).
    pub threat: Option<u8>,
    /// How many times a confirmed critical rolls the damage (default 2).
    pub crit: Option<u8>,
    #[serde(default)]
    pub to_hit: i32,
    /// The Strength a bow needs.
    pub strength: Option<u8>,
    /// Defence while wielded (the Parry category).
    #[serde(default)]
    pub parry: i32,
    /// Defence while worn (the Armour category).
    #[serde(default)]
    pub armour: i32,
    /// The most Dexterity bonus to Defence it allows while worn.
    pub max_dex: Option<i32>,
    /// Damage it soaks off every hit while worn (Protection; armour, never a shield).
    #[serde(default)]
    pub protection: i32,
    /// Percent more mana a spell costs while it is worn (heavy armour, a big shield).
    #[serde(default)]
    pub spell_cost_percent: u16,
    /// Defence while held (the Shield category).
    #[serde(default)]
    pub shield: i32,
    /// A shield's damage dice when bashing with it.
    pub bash: Option<Dice>,
    /// How much of a target's Protection a blow or a shot with it ignores (a weapon, a
    /// missile weapon, ammunition).
    #[serde(default)]
    pub penetration: i32,
    /// The kind of damage a weapon does (spec §8 "Damage types"), or two of which a blow does
    /// the better: `damage_type = "slashing"`, `["slashing", "piercing"]`. A shield's is its
    /// bash's.
    pub damage_type: Option<TypeNames>,
    /// What a blow with a weapon does to its target, instead of its damage type's verb:
    /// "cleaves".
    pub verb: Option<String>,
    /// Damage of another kind a weapon, a missile weapon, or ammunition adds to every hit:
    /// `added = { type = "fire", damage = "1d4" }`.
    pub added: Option<AddedSpec>,
    /// What it resists while worn or held, in percent per damage type: `resist = { cold = 20 }`.
    #[serde(default)]
    pub resist: BTreeMap<String, i32>,
    /// How its Protection stands against each physical kind, more or less than its
    /// `protection`: `protects = { slashing = 1, piercing = -1 }`.
    #[serde(default)]
    pub protects: BTreeMap<String, i32>,
    /// The weapon skill a weapon trains and draws on (`data/weapon_skills.toml`).
    pub category: Option<String>,
    /// How far a missile weapon shoots, or a throwing weapon (anything else with a range) is
    /// thrown, in cells.
    pub range: Option<i32>,
    /// A missile weapon reloaded after every shot (a crossbow): a shot also takes `reload`.
    #[serde(default)]
    pub reloads: bool,
    /// The chance in percent that it breaks when it hits (ammunition, throwing weapons).
    #[serde(default)]
    pub breaks: u8,
    /// Lit as it is shot or thrown: it flies and lands as this object (an object id).
    pub lit: Option<String>,
    /// A key: the lock id it opens, or `*` for a skeleton key (every lock).
    pub key: Option<String>,
    /// The light it gives while it burns where it lies (a light preset id); with `burns`.
    pub light: Option<String>,
    /// How long it burns where it lands, in seconds of game time; then it is gone.
    pub burns: Option<u32>,
    /// A digging tool's power (spec §8 "Digging"): how much work a dig with it does.
    pub dig: Option<u16>,
    /// Presentation: the glyph it flies as, drawn pointing right and turned along its flight
    /// (default: its own glyph, upright).
    pub missile: Option<char>,
    /// Presentation: it tumbles as it flies (a thrown dagger).
    #[serde(default)]
    pub spins: bool,
    /// Presentation: how much its glyph glows where it lies, 0 to 1.
    #[serde(default)]
    pub glow: f32,
    /// Presentation: particle emitters, `ambient` (while it lies in view on top of its pile) or
    /// `flight` (while it flies).
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
    /// The kind of item it is for magic: what affixes it may roll. None: never magic, never
    /// loot.
    pub kind: Option<ItemKind>,
    /// What an attack with it costs, instead of the `melee` action's (a missile weapon: the
    /// `shoot` action's).
    pub attack_cost: Option<u32>,
    /// The lowest item level loot holds it at (default 1).
    pub level: Option<u8>,
    /// How often loot holds it among what may be there (default 100; 0: never).
    pub frequency: Option<u32>,
    /// What eating it gives (spec §8 "Hunger"): its nutrition and the turns it takes.
    pub food: Option<Food>,
    /// What eating it cures (food only): means named in the statuses' and diseases' `cured_by`
    /// (`remove_disease`).
    #[serde(default)]
    pub cures: Vec<String>,
    /// What reading it does (a scroll, spec §8 "Scrolls").
    pub read: Option<ReadSpec>,
    /// The spell it holds and its charges, if it is a wand (spec §8 "Magic items": "Wands").
    pub wand: Option<WandSpec>,
    /// The effect it holds and its mana cost, if it is a rod (spec §8 "Magic items": "Rods").
    pub rod: Option<RodSpec>,
    /// What drinking it does, if it is a potion (spec §8 "Potions").
    pub drink: Option<DrinkSpec>,
}

/// A resolved object type.
#[derive(Clone, Debug)]
pub struct ObjectDef {
    pub id: String,
    /// Singular, without an article: "dagger"; with the notable parts its written name does not
    /// mention ("gold and ruby amulet", [`super::part::names`]).
    pub name: String,
    /// "daggers".
    pub plural: String,
    /// Its name and plural as its file writes them, before its parts are added: what an item
    /// made of other parts is named from ([`crate::object::Stack::names`]).
    pub written_name: String,
    pub written_plural: String,
    pub glyph: char,
    pub fg: Rgb,
    /// Its item class (`data/item_classes.toml`): what kind of thing it is, which item lists
    /// group by and shops trade by. The player knows it of anything they see, identified or not.
    pub class: ItemClassId,
    /// What it is made of, part by part, the main part first ([`ObjectDef::material`]); the
    /// shares sum to 100. Never empty.
    pub parts: Vec<Part>,
    /// What each part may be made of (#194, [`ObjectDef::made_of`]): one list per part, in
    /// `parts`' order, each sorted by material id; empty when no part has a list, as then each
    /// is made only of its own material.
    pub construction: Vec<Vec<MaterialId>>,
    /// The part a gem is set as when loot ornaments it, instead of its class's (spec §7 "Gems").
    pub ornament: Option<PartId>,
    /// Weight of one, in grams; at least 1.
    pub weight: u32,
    /// What one is worth, in gold coins, at most [`MAX_VALUE`]; 0 for junk (spec §7 "Shops":
    /// nobody buys or sells it).
    pub value: u32,
    /// Several lie in one stack (coins, arrows) instead of a stack each.
    pub stackable: bool,
    /// Where it is worn or wielded: the `fits` of the slots it goes in (`None`: nowhere).
    pub slot: Option<String>,
    /// Takes both hands: every hand slot.
    pub two_handed: bool,
    /// A light one-handed weapon: it goes in any hand slot (the off hand too), not only its own.
    pub light_weapon: bool,
    /// The kind of ammunition it is, if it is ammunition ("arrow").
    pub ammo: Option<String>,
    /// The kind of ammunition it shoots, if it is a missile weapon ("arrow").
    pub launches: Option<String>,
    /// What a blow with it does (a launcher: a shot from it); `None` for anything that is no
    /// weapon. Rules only, like everything about combat but its words.
    pub attack: Option<Attack>,
    /// Defence while wielded: its Parry bonus.
    pub parry: i32,
    /// Defence while worn: its Armour bonus.
    pub armour: i32,
    /// While worn, the most the Dexterity bonus may add to Defence (`None`: no limit).
    pub max_dex: Option<i32>,
    /// While worn, the damage it soaks off every hit: its Protection, which adds up with the
    /// rest of what is worn (spec §8 "Combat").
    pub protection: i32,
    /// While worn, percent more mana a spell costs (spec §8 "Spells"); what is worn adds up.
    pub spell_cost_percent: u16,
    /// Defence while held: its Shield bonus.
    pub shield: i32,
    /// What a bash with it does, if it is a shield one can bash with.
    pub bash: Option<Attack>,
    /// How much of a target's Protection an attack with it ignores (a weapon, a missile weapon,
    /// ammunition: a shot's is its launcher's and its ammunition's together). Rules only.
    pub penetration: i32,
    /// What a blow with it does to its target, if it says ("cleaves"); else its damage type's
    /// verb, else "hits" (spec §8 "Damage types").
    pub verb: Option<String>,
    /// Damage of another kind every hit with it adds (a weapon's, a missile weapon's, or
    /// ammunition's: a shot adds the launcher's and the ammunition's).
    pub added: Option<Rider>,
    /// What it resists while worn or held, in percent per damage type (spec §8 "Damage types").
    pub resist: Vec<(DamageTypeId, i32)>,
    /// How its Protection stands against each physical kind, more or less than `protection`.
    pub protects: Vec<(DamageTypeId, i32)>,
    /// Its weapon skill: a weapon's `category` (`None` for anything else, and without
    /// `data/weapon_skills.toml`).
    pub skill: Option<WeaponSkillId>,
    /// How far it shoots (a missile weapon) or is thrown (a throwing weapon), in cells.
    pub range: Option<i32>,
    /// A missile weapon reloaded after every shot: a shot also takes the `reload` action.
    pub reloads: bool,
    /// The chance in percent that it breaks when it hits.
    pub breaks: u8,
    /// What it flies and lands as, lit (a flaming arrow: a burning arrow).
    pub lit: Option<ObjectId>,
    /// The light it gives where it lies, while it burns, and for how long it burns.
    pub burning: Option<Burning>,
    /// A key: what locks it opens (spec §8 "Doors").
    pub key: Option<Key>,
    /// A digging tool's power (spec §8 "Digging"): a dig with it at Strength 10 does this much
    /// work against a wall's hardness ([`crate::world::dig_work`]). `None`: it does not dig.
    pub dig: Option<u16>,
    /// Presentation: the glyph it flies as, pointing right; `None`: its glyph, upright.
    pub missile: Option<char>,
    /// Presentation: it tumbles as it flies.
    pub spins: bool,
    /// Presentation: how much its glyph glows where it lies.
    pub glow: f32,
    /// Presentation: its particle emitters.
    pub particles: Vec<ParticleEmitter>,
    /// The kind of item it is for magic; `None`: never magic, never loot.
    pub kind: Option<ItemKind>,
    /// What an attack with it costs instead of the action's (`melee`, or `shoot` for a missile
    /// weapon).
    pub attack_cost: Option<u32>,
    /// The lowest item level loot holds it at.
    pub level: u8,
    /// How often loot holds it, relative to the rest (0 for what is no loot: neither a base item,
    /// with a `kind`, nor a scroll).
    pub frequency: u32,
    /// What eating it gives, if it is food or a corpse (spec §8 "Hunger").
    pub food: Option<Food>,
    /// What eating it cures: means named in the statuses' and diseases' `cured_by` (a remedy
    /// for diseases: `remove_disease`; spec §8 "Diseases").
    pub cures: Vec<String>,
    /// The creature type it is the corpse of, if it is a corpse: it rots from the moment its
    /// creature died ([`Stack::died`]).
    pub corpse: Option<CreatureId>,
    /// What reading it does, if it is a scroll (spec §8 "Scrolls").
    pub read: Option<Reading>,
    /// The spell it holds, its charges, and its caster level, if it is a wand (spec §8 "Magic
    /// items": "Wands").
    pub wand: Option<Wand>,
    /// The effect it holds and what a use costs in mana, if it is a rod (spec §8 "Magic items":
    /// "Rods").
    pub rod: Option<Rod>,
    /// What drinking it does, if it is a potion (spec §8 "Potions").
    pub drink: Option<Drink>,
}

/// Damage of another kind an attack with the object adds, as data writes it: `added = { type =
/// "fire", damage = "1d4" }`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddedSpec {
    #[serde(rename = "type")]
    pub kind: String,
    pub damage: Dice,
}

impl AddedSpec {
    /// The rider it names: a damage type that is no physical kind (Protection never soaks
    /// added damage), with dice that can do harm.
    fn resolve(&self, content: &Content) -> Result<Rider, String> {
        let kind = (content.damage_type_id(&self.kind))
            .ok_or_else(|| format!("`added`: unknown damage type `{}`", self.kind))?;
        if content.damage_type(kind).physical {
            return Err(format!(
                "`added`: `{}` is physical; added damage is of another kind (fire, cold, ...)",
                self.kind
            ));
        }
        if self.damage.min() < 0 || self.damage.max() < 1 {
            return Err("`added`: `damage` must be dice that can do harm, e.g. \"1d4\"".into());
        }
        Ok(Rider {
            kind,
            dice: self.damage,
        })
    }
}

/// How a burning object burns where it lies: the light it gives, and for how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Burning {
    pub light: LightId,
    pub lasts: Duration,
}

impl ObjectDef {
    /// The corpse of creature type `creature` (id `id`): "kobold corpse", `%` in its color, made
    /// of `material` and weighing what it does, feeding as `food` (its size's).
    pub(super) fn corpse_of(
        id: CreatureId,
        creature: &CreatureDef,
        class: ItemClassId,
        material: MaterialId,
        food: Food,
    ) -> Self {
        Self {
            id: format!("{}_corpse", creature.id),
            name: format!("{} corpse", creature.name),
            plural: format!("{} corpses", creature.name),
            written_name: format!("{} corpse", creature.name),
            written_plural: format!("{} corpses", creature.name),
            glyph: '%',
            fg: creature.fg,
            class,
            parts: vec![Part::whole(material)],
            construction: Vec::new(),
            ornament: None,
            weight: creature.weight,
            // Nobody buys or sells a corpse (spec §7 "Shops").
            value: 0,
            stackable: false,
            slot: None,
            two_handed: false,
            light_weapon: false,
            ammo: None,
            launches: None,
            attack: None,
            parry: 0,
            armour: 0,
            max_dex: None,
            protection: 0,
            spell_cost_percent: 0,
            shield: 0,
            bash: None,
            penetration: 0,
            verb: None,
            added: None,
            resist: Vec::new(),
            protects: Vec::new(),
            skill: None,
            range: None,
            reloads: false,
            breaks: 0,
            lit: None,
            burning: None,
            dig: None,
            missile: None,
            spins: false,
            glow: 0.0,
            particles: Vec::new(),
            food: Some(food),
            cures: Vec::new(),
            corpse: Some(id),
            read: None,
            wand: None,
            rod: None,
            drink: None,
            key: None,
            kind: None,
            attack_cost: None,
            level: 1,
            frequency: 0,
        }
    }

    /// What it is made of: its main part's material (spec §7 "Objects and materials"), which
    /// decides what a single material decides. [`ObjectDef::parts`] has every part.
    pub fn material(&self) -> MaterialId {
        self.parts[0].material
    }

    /// What its part `part` (an index in `parts`) may be made of (#194): its construction list,
    /// else only its own material.
    pub fn made_of(&self, part: usize) -> &[MaterialId] {
        match self.construction.get(part) {
            Some(list) => list,
            None => std::slice::from_ref(&self.parts[part].material),
        }
    }

    /// What its main part must be able to be, for its construction list: a weapon's working
    /// part for what strikes or is shot (a weapon, a launcher, ammunition), armour for what is
    /// worn as armour or held as a shield, else anything.
    pub fn role(&self) -> part::PartRole {
        if self.attack.is_some() || self.ammo.is_some() {
            part::PartRole::Weapon
        } else if (self.kind).is_some_and(|k| k.is_armour() || k == ItemKind::Shield) {
            part::PartRole::Armour
        } else {
            part::PartRole::Any
        }
    }

    /// Whether it is jewellery (a ring, an amulet): another material changes only what it is
    /// worth and how it looks (#193).
    pub fn is_jewellery(&self) -> bool {
        matches!(self.kind, Some(ItemKind::Ring | ItemKind::Amulet))
    }

    /// What one of it made of `parts` is worth, in gold coins: its `value`, scaled by what
    /// `parts` without a gem are worth against its own parts (each part's share times its
    /// material's value, [`part::worth`]), rounded to the nearest coin, plus what each gem part
    /// is worth by its size and cut ([`super::Gems::value`]: a loose gem, an ornament), at most
    /// [`MAX_VALUE`]. Made of its own parts, its `value`.
    pub fn value_with(&self, parts: &[Part], content: &Content) -> u32 {
        if parts == self.parts.as_slice() {
            return self.value;
        }
        let plain: Vec<Part> = parts.iter().filter(|p| p.gem.is_none()).copied().collect();
        let (worth, own) = (
            part::worth(&plain, content.materials()),
            part::worth(&self.parts, content.materials()),
        );
        let made = (u64::from(self.value) * worth + own / 2) / own.max(1);
        let gems: u64 = (parts.iter())
            .filter_map(|p| {
                let (gems, gem) = (content.gems()?, content.material(p.material).gem.as_ref()?);
                Some(u64::from(gems.value(gem, p.gem?)))
            })
            .sum();
        u32::try_from((made + gems).min(u64::from(MAX_VALUE))).unwrap_or(MAX_VALUE)
    }

    /// Whether it is a throwing weapon: it has a range and is no missile weapon.
    pub fn is_thrown(&self) -> bool {
        self.range.is_some() && self.launches.is_none()
    }

    /// Whether it can be thrown at all: a throwing weapon, or any other weapon that is no
    /// missile weapon or ammunition, thrown whole and badly ([`ObjectDef::is_improvised`]).
    pub fn can_throw(&self) -> bool {
        self.is_thrown() || self.is_improvised()
    }

    /// Whether it is a weapon not made for throwing (a sword, a club): thrown whole, with
    /// [`crate::combat::IMPROVISED_TO_HIT`] to hit, [`crate::combat::IMPROVISED_RANGE`] cells.
    pub fn is_improvised(&self) -> bool {
        self.attack.is_some()
            && self.range.is_none()
            && self.launches.is_none()
            && self.ammo.is_none()
    }

    /// How far it is thrown, if it can be.
    pub fn throw_range(&self) -> Option<i32> {
        if self.is_thrown() {
            self.range
        } else {
            self.is_improvised()
                .then_some(crate::combat::IMPROVISED_RANGE)
        }
    }

    /// Whether it flies: ammunition, or a throwing weapon.
    pub fn flies(&self) -> bool {
        self.ammo.is_some() || self.is_thrown()
    }

    /// `count` of it in words: "a dagger", "an iron key", "12 arrows".
    pub fn describe(&self, count: u32) -> String {
        counted(&self.name, &self.plural, count)
    }
}

/// `count` of something called `name` (`plural` for several) in words: "a dagger", "an iron
/// key", "12 arrows".
pub fn counted(name: &str, plural: &str, count: u32) -> String {
    if count == 1 {
        format!("{} {name}", article(name))
    } else {
        format!("{count} {plural}")
    }
}

/// `a` or `an` for a noun, by its first letter.
pub fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

impl ObjectSpec {
    /// Checks the definition and resolves its class, parts, light, and weapon skill;
    /// `classes` are the item classes, `parts` resolves its `materials` ([`part::resolve`]),
    /// `slots` are where things may be worn or wielded, `skills` the weapon skills. Returns the
    /// object, its name as written (its notable parts are added once content knows them all:
    /// [`part::names`]), and the id of what it is `lit` as, which names another object and is
    /// resolved once every object is known.
    pub(super) fn resolve(
        self,
        file: &str,
        content: &Content,
        classes: &[ItemClassDef],
        parts: impl Fn(&[PartEntry]) -> Result<Vec<Part>, String>,
        light: impl Fn(&str) -> Option<LightId>,
        slots: &[SlotDef],
        skills: &WeaponSkillTable,
    ) -> Result<(ObjectDef, Option<String>), ContentError> {
        let err = |m: String| ContentError::new(file, format!("object `{}`: {m}", self.id));
        if self.name.trim().is_empty() || self.plural.as_ref().is_some_and(|p| p.trim().is_empty())
        {
            return Err(err("`name` and `plural` must not be empty".into()));
        }
        if self.glyph.is_whitespace() || self.glyph.is_control() {
            return Err(err("`glyph` must be a visible character".into()));
        }
        if self.weight == 0 {
            return Err(err(
                "`weight` is in grams and must be at least 1; every object weighs something".into(),
            ));
        }
        if self.value > MAX_VALUE {
            return Err(err(format!("`value` is in gold coins, 0 to {MAX_VALUE}")));
        }
        let class = self.item_class(classes).map_err(err)?;
        if let Some(material) = &self.material {
            return Err(err(format!(
                "`material` is now `materials`, a list of parts, the first the main one: \
                 `materials = [\"{material}\"]` for one"
            )));
        }
        let parts = parts(&self.materials).map_err(err)?;
        if let Some(slot) = &self.slot
            && !slots.iter().any(|s| s.fits == *slot)
        {
            return Err(err(format!(
                "`slot = \"{slot}\"` is no slot of data/slots.toml, nor what one fits"
            )));
        }
        if self.two_handed {
            let hands = slots.iter().filter(|s| s.hand).count();
            let in_hand = (slots.iter()).any(|s| s.hand && Some(&s.fits) == self.slot.as_ref());
            if !in_hand || hands < 2 {
                return Err(err(
                    "`two_handed` needs a `slot` that is a hand, and two hand slots".into(),
                ));
            }
        }
        if self.light_weapon {
            let in_hand = (slots.iter()).any(|s| s.hand && Some(&s.fits) == self.slot.as_ref());
            if !in_hand || self.two_handed || self.damage.is_none() || self.launches.is_some() {
                return Err(err(
                    "`light_weapon` is for one-handed weapons wielded in a hand (`damage`, a hand \
                     `slot`, not `two_handed`)"
                        .into(),
                ));
            }
        }
        for (field, kind) in [("ammo", &self.ammo), ("launches", &self.launches)] {
            if kind.as_ref().is_some_and(|k| k.trim().is_empty()) {
                return Err(err(format!("`{field}` must not be empty")));
            }
        }
        // A weapon names its kind of damage; a shield's names its bash's, else bare hands'.
        let armed = self.damage.is_some() || self.bash.is_some();
        let kinds = content
            .resolve_kinds(self.damage_type.as_ref())
            .map_err(err)?;
        if !kinds.is_empty() && !armed {
            return Err(err(
                "`damage_type` is for weapons (`damage`) and shields that `bash`".into(),
            ));
        }
        let attack = self.attack(kinds).map_err(err)?;
        self.check_defence().map_err(err)?;
        self.check_protection(slots).map_err(err)?;
        self.check_ranged().map_err(err)?;
        let skill = self.skill(slots, skills).map_err(err)?;
        let bash = match (self.bash, self.shield) {
            (None, _) => None,
            (Some(damage), shield) if shield > 0 => Some(Attack {
                damage,
                kinds: if kinds.is_empty() {
                    Attack::unarmed(content.unarmed_damage()).kinds
                } else {
                    kinds
                },
                ..Attack::unarmed(None)
            }),
            (Some(_), _) => return Err(err("`bash` is for shields (`shield`)".into())),
        };
        if self.verb.as_ref().is_some_and(|v| v.trim().is_empty()) {
            return Err(err("`verb` must not be empty".into()));
        }
        if self.verb.is_some() && !armed {
            return Err(err(
                "`verb` is for weapons (`damage`) and shields that `bash`".into(),
            ));
        }
        let added = match &self.added {
            None => None,
            Some(_) if self.damage.is_none() && self.ammo.is_none() => {
                return Err(err(
                    "`added` is for weapons (`damage`) and ammunition (`ammo`)".into(),
                ));
            }
            Some(spec) => Some(spec.resolve(content).map_err(err)?),
        };
        let percent = MIN_RESISTANCE..=MAX_RESISTANCE;
        let resist =
            (content.resolve_per_kind("resist", &self.resist, percent, false)).map_err(err)?;
        if !resist.is_empty() && self.slot.is_none() {
            return Err(err("`resist` is for things worn or held (`slot`)".into()));
        }
        let protects =
            (content.resolve_per_kind("protects", &self.protects, -MAX_BONUS..=MAX_BONUS, true))
                .map_err(err)?;
        if !protects.is_empty() && self.protection == 0 {
            return Err(err("`protects` is for armour with `protection`".into()));
        }
        let burning = match (&self.light, self.burns) {
            (None, None) => None,
            (Some(id), Some(seconds)) => {
                let light = light(id).ok_or_else(|| err(format!("unknown light preset `{id}`")))?;
                Some(Burning {
                    light,
                    lasts: Duration::seconds(u64::from(seconds)),
                })
            }
            _ => return Err(err("`light` and `burns` go together".into())),
        };
        if !(0.0..=1.0).contains(&self.glow) {
            return Err(err("`glow` must be between 0 and 1".into()));
        }
        self.check_base(attack.is_some()).map_err(err)?;
        if let Some(food) = &self.food {
            food.check().map_err(|m| err(format!("food: {m}")))?;
            if self.slot.is_some() || self.flies() || self.burns.is_some() {
                return Err(err(
                    "`food` is for things eaten, not worn, wielded, shot, thrown, or burned".into(),
                ));
            }
        }
        if !self.cures.is_empty() && self.food.is_none() {
            return Err(err("`cures` is for food: it cures as it is eaten".into()));
        }
        let key = self
            .key
            .as_deref()
            .map(Key::parse)
            .transpose()
            .map_err(err)?;
        if let Some(power) = self.dig
            && (!(1..=MAX_DIG).contains(&power) || self.slot.is_none())
        {
            return Err(err(format!(
                "`dig` is a digging tool's power, 1 to {MAX_DIG}, and a tool goes in a `slot`"
            )));
        }
        let read = self
            .read
            .clone()
            .map(|r| r.resolve(content))
            .transpose()
            .map_err(err)?;
        if read.is_some() && (self.slot.is_some() || self.food.is_some() || self.flies()) {
            return Err(err(
                "`read` is for scrolls: not worn, wielded, eaten, shot, or thrown".into(),
            ));
        }
        let wand = self.wand_of(content).map_err(err)?;
        let rod = self.rod_of(content).map_err(err)?;
        let drink = self.drink_of(content).map_err(err)?;
        let plural = self.plural.unwrap_or_else(|| content.plural(&self.name));
        let loot = self.kind.is_some()
            || read.is_some()
            || wand.is_some()
            || rod.is_some()
            || drink.is_some();
        let def = ObjectDef {
            id: self.id,
            written_name: self.name.clone(),
            written_plural: plural.clone(),
            name: self.name,
            plural,
            glyph: self.glyph,
            fg: self.fg,
            class,
            parts,
            // Resolved as content loads, once the object's role is known.
            construction: Vec::new(),
            // Resolved as content loads, once the part names are known.
            ornament: None,
            weight: self.weight,
            value: self.value,
            stackable: self.stackable,
            slot: self.slot,
            two_handed: self.two_handed,
            light_weapon: self.light_weapon,
            ammo: self.ammo,
            launches: self.launches,
            attack,
            parry: self.parry,
            armour: self.armour,
            max_dex: self.max_dex,
            protection: self.protection,
            spell_cost_percent: self.spell_cost_percent,
            shield: self.shield,
            bash,
            penetration: self.penetration,
            verb: self.verb,
            added,
            resist,
            protects,
            skill,
            range: self.range,
            reloads: self.reloads,
            breaks: self.breaks,
            lit: None,
            burning,
            key,
            dig: self.dig,
            missile: self.missile,
            spins: self.spins,
            glow: self.glow,
            particles: self.particles,
            kind: self.kind,
            attack_cost: self.attack_cost,
            level: self.level.unwrap_or(1),
            frequency: if loot {
                self.frequency.unwrap_or(100)
            } else {
                0
            },
            food: self.food,
            cures: self.cures,
            corpse: None,
            read,
            wand,
            rod,
            drink,
        };
        Ok((def, self.lit))
    }

    /// Its wand, resolved (spec §8 "Magic items": "Wands"): one that goes in a slot (the missile
    /// weapon slot), one at a time, and is nothing else (no weapon, missile, food, scroll, or
    /// magic item kind).
    fn wand_of(&self, content: &Content) -> Result<Option<Wand>, String> {
        let Some(spec) = self.wand.clone() else {
            return Ok(None);
        };
        let other = self.stackable
            || self.slot.is_none()
            || self.damage.is_some()
            || self.launches.is_some()
            || self.flies()
            || self.food.is_some()
            || self.read.is_some()
            || self.kind.is_some();
        if other {
            return Err(
                "`wand`: a wand goes in a `slot`, one at a time (not `stackable`), and is \
                 no weapon, missile, food, scroll, or magic item `kind`"
                    .into(),
            );
        }
        spec.resolve(content).map(Some)
    }

    /// Its rod, resolved (spec §8 "Magic items": "Rods"): as a wand, one that goes in a slot,
    /// one at a time, and is nothing else (no weapon, missile, food, scroll, wand, or magic item
    /// kind).
    fn rod_of(&self, content: &Content) -> Result<Option<Rod>, String> {
        let Some(spec) = self.rod.clone() else {
            return Ok(None);
        };
        let other = self.stackable
            || self.slot.is_none()
            || self.damage.is_some()
            || self.launches.is_some()
            || self.flies()
            || self.food.is_some()
            || self.read.is_some()
            || self.wand.is_some()
            || self.kind.is_some();
        if other {
            return Err(
                "`rod`: a rod goes in a `slot`, one at a time (not `stackable`), and is no \
                 weapon, missile, food, scroll, wand, or magic item `kind`"
                    .into(),
            );
        }
        spec.resolve(content).map(Some)
    }

    /// Its potion, resolved (spec §8 "Potions"): stackable, and nothing else (no slot, weapon,
    /// missile, food, scroll, wand, rod, or magic item kind).
    fn drink_of(&self, content: &Content) -> Result<Option<Drink>, String> {
        let Some(spec) = self.drink.clone() else {
            return Ok(None);
        };
        let other = !self.stackable
            || self.slot.is_some()
            || self.damage.is_some()
            || self.launches.is_some()
            || self.flies()
            || self.food.is_some()
            || self.read.is_some()
            || self.wand.is_some()
            || self.rod.is_some()
            || self.kind.is_some();
        if other {
            return Err(
                "`drink`: a potion is `stackable` and nothing else: no slot, weapon, missile, \
                 food, scroll, wand, rod, or magic item `kind`"
                    .into(),
            );
        }
        spec.resolve(content).map(Some)
    }

    /// Its class: one of `classes`, and the class of its magic item `kind`, if it has one (so
    /// the two cannot say different things).
    fn item_class(&self, classes: &[ItemClassDef]) -> Result<ItemClassId, String> {
        let at = (classes.iter().position(|c| c.id == self.class)).ok_or_else(|| {
            format!(
                "unknown item class `{}` (data/item_classes.toml)",
                self.class
            )
        })?;
        if let Some(kind) = self.kind
            && !classes[at].kinds.contains(&kind)
        {
            let of = classes.iter().find(|c| c.kinds.contains(&kind));
            return Err(match of {
                Some(c) => format!(
                    "`kind = \"{}\"` is of class `{}`, not `{}`",
                    kind.id(),
                    c.id,
                    self.class
                ),
                None => format!(
                    "no class in data/item_classes.toml lists kind `{}`",
                    kind.id()
                ),
            });
        }
        Ok(ItemClassId(at as u8))
    }

    /// The fields of a base item: its `kind` fits what it is (a weapon has an attack and goes in
    /// a hand, ...), and `level` and `frequency` are for loot: things with a kind, scrolls, and
    /// wands.
    fn check_base(&self, weapon: bool) -> Result<(), String> {
        if let Some(kind) = self.kind {
            let fits = match kind {
                ItemKind::Weapon => {
                    weapon && self.slot.is_some() && self.launches.is_none() && self.ammo.is_none()
                }
                ItemKind::Launcher => self.launches.is_some(),
                ItemKind::Ammo => self.ammo.is_some() && self.slot.is_some(),
                ItemKind::Shield => self.shield > 0,
                _ => self.slot.is_some() && !weapon,
            };
            if !fits {
                return Err(format!(
                    "`kind = \"{}\"` does not fit it: a weapon has `damage` and a `slot`, a \
                     missile weapon `launches`, ammunition `ammo`, a shield `shield`, and the \
                     rest are worn (`slot`)",
                    kind.id()
                ));
            }
        } else if (self.level.is_some() || self.frequency.is_some())
            && self.read.is_none()
            && self.wand.is_none()
            && self.rod.is_none()
            && self.drink.is_none()
        {
            return Err(
                "`level` and `frequency` are for loot: base items, which have a `kind`, scrolls \
                 (`read`), wands (`wand`), rods (`rod`), and potions (`drink`)"
                    .into(),
            );
        }
        if self.level.is_some_and(|l| l == 0 || l > MAX_ITEM_LEVEL) {
            return Err(format!("`level` must be 1 to {MAX_ITEM_LEVEL}"));
        }
        if let Some(cost) = self.attack_cost {
            if !weapon || self.slot.is_none() {
                return Err(
                    "`attack_cost` is for weapons worn or wielded (`damage`, `slot`)".into(),
                );
            }
            check_cost(cost).map_err(|m| format!("`attack_cost`: {m}"))?;
        }
        Ok(())
    }

    /// Whether it flies: ammunition, or a throwing weapon (a range without `launches`).
    fn flies(&self) -> bool {
        self.ammo.is_some() || (self.range.is_some() && self.launches.is_none())
    }

    /// The fields of ranged combat: a missile weapon needs a range and may reload; a throwing
    /// weapon (a range without `launches`) needs its own damage; ammunition takes its range from
    /// the launcher; only what flies may break or be lit.
    fn check_ranged(&self) -> Result<(), String> {
        if let Some(range) = self.range
            && !(1..=MAX_RANGE).contains(&range)
        {
            return Err(format!("`range` must be 1 to {MAX_RANGE} cells"));
        }
        if self.launches.is_some() && self.range.is_none() {
            return Err("a missile weapon (`launches`) needs a `range`".into());
        }
        if self.ammo.is_some() && self.range.is_some() {
            return Err(
                "ammunition (`ammo`) flies as far as its launcher shoots: no `range`".into(),
            );
        }
        let thrown = self.range.is_some() && self.launches.is_none();
        if thrown && self.damage.is_none() {
            return Err("a throwing weapon (`range`) needs its `damage` dice".into());
        }
        if self.reloads && self.launches.is_none() {
            return Err("only missile weapons (`launches`) `reload`".into());
        }
        let flies = self.ammo.is_some() || thrown;
        if self.breaks > 100 {
            return Err("`breaks` is a chance in percent, 0 to 100".into());
        }
        if !flies && (self.breaks > 0 || self.lit.is_some()) {
            return Err(
                "`breaks` and `lit` are for things that fly: ammunition and throwing weapons"
                    .into(),
            );
        }
        if self.burns.is_some_and(|s| !(1..=MAX_BURN).contains(&s)) {
            return Err(format!("`burns` must be 1 to {MAX_BURN} seconds"));
        }
        Ok(())
    }

    /// The weapon it is, from `damage` and the fields that go with it: only something that is
    /// worn or wielded is a weapon, and a missile weapon must be one. Its blows do `kinds`.
    fn attack(&self, kinds: Kinds) -> Result<Option<Attack>, String> {
        let Some(damage) = self.damage else {
            let weapon_only = [
                ("threat", self.threat.is_some()),
                ("crit", self.crit.is_some()),
                ("to_hit", self.to_hit != 0),
                ("strength", self.strength.is_some()),
            ];
            if let Some((field, _)) = weapon_only.iter().find(|(_, given)| *given) {
                return Err(format!("`{field}` is for weapons, which have a `damage`"));
            }
            if self.launches.is_some() {
                return Err("a missile weapon (`launches`) needs its `damage` dice".into());
            }
            return Ok(None);
        };
        if self.slot.is_none() && self.range.is_none() {
            return Err(
                "a weapon (`damage`) must go in a slot (`slot`) or be thrown (`range`)".into(),
            );
        }
        let threat = self.threat.unwrap_or(DEFAULT_THREAT);
        let crit = self.crit.unwrap_or(DEFAULT_CRIT);
        if !THREATS.contains(&threat) || !CRITS.contains(&crit) {
            return Err(format!(
                "`threat` must be {} to {} (the lowest natural roll that threatens) and `crit` \
                 {} to {}",
                THREATS.start(),
                THREATS.end(),
                CRITS.start(),
                CRITS.end()
            ));
        }
        if self.to_hit.abs() > MAX_BONUS {
            return Err(format!("`to_hit` must be within ±{MAX_BONUS}"));
        }
        if self
            .strength
            .is_some_and(|s| !crate::stats::SCORES.contains(&s))
        {
            return Err("`strength` is a Strength score, 1 to 40".into());
        }
        Ok(Some(Attack {
            damage,
            to_hit: self.to_hit,
            threat,
            crit,
            strength: self.strength,
            kinds,
        }))
    }

    /// The weapon skill a weapon names (`category`): every weapon names one when there are
    /// weapon skills, nothing else does, and its role fits what the weapon is: a missile weapon
    /// a `launcher`, what is wielded in a hand `melee`, anything else (only thrown) `thrown`.
    fn skill(
        &self,
        slots: &[SlotDef],
        skills: &WeaponSkillTable,
    ) -> Result<Option<WeaponSkillId>, String> {
        let Some(name) = &self.category else {
            if self.damage.is_some() && !skills.is_empty() {
                return Err(
                    "a weapon (`damage`) names its weapon skill (`category`, an id in \
                     data/weapon_skills.toml)"
                        .into(),
                );
            }
            return Ok(None);
        };
        if self.damage.is_none() {
            return Err("`category` is for weapons, which have a `damage`".into());
        }
        let id = skills
            .id(name)
            .ok_or_else(|| format!("unknown weapon skill `{name}` (data/weapon_skills.toml)"))?;
        let in_hand = (slots.iter()).any(|s| s.hand && Some(&s.fits) == self.slot.as_ref());
        let role = if self.launches.is_some() {
            Role::Launcher
        } else if in_hand {
            Role::Melee
        } else {
            Role::Thrown
        };
        let has = skills.skill(id).role;
        if has != role {
            return Err(format!(
                "`category = \"{name}\"` is a skill of role `{}`, but this weapon needs one of \
                 role `{}`",
                has.id(),
                role.id()
            ));
        }
        Ok(Some(id))
    }

    /// Defence bonuses are for things worn or wielded, 0 to [`MAX_BONUS`].
    fn check_defence(&self) -> Result<(), String> {
        let bonuses = [
            ("parry", self.parry),
            ("armour", self.armour),
            ("shield", self.shield),
            ("max_dex", self.max_dex.unwrap_or(0)),
        ];
        if let Some((field, _)) = bonuses.iter().find(|(_, b)| !(0..=MAX_BONUS).contains(b)) {
            return Err(format!("`{field}` must be 0 to {MAX_BONUS}"));
        }
        let given = bonuses.iter().any(|(_, b)| *b != 0) || self.max_dex.is_some();
        if given && self.slot.is_none() {
            return Err(
                "`parry`, `armour`, `max_dex`, and `shield` are for things worn or wielded \
                 (`slot`)"
                    .into(),
            );
        }
        Ok(())
    }

    /// Protection is for armour worn, never in a hand (a shield's is Defence only), and
    /// penetration for weapons and ammunition; each 0 to [`MAX_BONUS`].
    fn check_protection(&self, slots: &[SlotDef]) -> Result<(), String> {
        for (field, value) in [
            ("protection", self.protection),
            ("penetration", self.penetration),
        ] {
            if !(0..=MAX_BONUS).contains(&value) {
                return Err(format!("`{field}` must be 0 to {MAX_BONUS}"));
            }
        }
        let in_hand = (slots.iter()).any(|s| s.hand && Some(&s.fits) == self.slot.as_ref());
        if self.protection > 0 && (self.slot.is_none() || in_hand || self.shield > 0) {
            return Err(
                "`protection` is for armour worn (a `slot` that is no hand; a shield's is \
                 `shield`)"
                    .into(),
            );
        }
        if self.spell_cost_percent > MAX_SPELL_COST
            || (self.spell_cost_percent > 0 && self.slot.is_none())
        {
            return Err(format!(
                "`spell_cost_percent` is 0 to {MAX_SPELL_COST}, for things worn (`slot`)"
            ));
        }
        if self.penetration > 0 && self.damage.is_none() && self.ammo.is_none() {
            return Err("`penetration` is for weapons (`damage`) and ammunition (`ammo`)".into());
        }
        Ok(())
    }
}

/// The most `spell_cost_percent` a worn thing may add.
pub const MAX_SPELL_COST: u16 = 100;

/// Parses objects as level files place them: ids, each optionally with a count (`"arrow*12"`),
/// first on top; a loose gem names its gem (`"gem:ruby*3"`: the gems' object made of that gem,
/// of the plain size and cut; spec §7 "Gems"), and anything else may name another material its
/// base item may be made of (`"dagger:bronze"`, a variant, #193), and a unique item is named by
/// its id (`"unique:widowmaker"`, spec §8 "Magic items": once in all content). A stackable
/// object becomes one stack (the same placement twice joins it); any other object becomes a
/// stack of one per count.
pub(super) fn parse_stacks(specs: &[String], content: &Content) -> Result<Vec<Stack>, String> {
    let mut stacks: Vec<Stack> = Vec::new();
    for spec in specs {
        let (placed, count) = match spec.split_once('*') {
            Some((id, n)) => {
                let count = n
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|n| (1..=MAX_PLACED).contains(n))
                    .ok_or_else(|| {
                        format!("`{spec}`: the count must be a number from 1 to {MAX_PLACED}")
                    })?;
                (id.trim(), count)
            }
            None => (spec.trim(), 1),
        };
        // A unique item (`unique:widowmaker`), one and once (`super::unique`).
        if let Some(unique) = placed.strip_prefix(super::UNIQUE) {
            if count != 1 {
                return Err(format!("`{spec}`: a unique item is placed one at a time"));
            }
            stacks.push(content.place_unique(unique.trim())?);
            continue;
        }
        let (id, made) = match placed.split_once(':') {
            Some((id, made)) => (id.trim(), Some(made.trim())),
            None => (placed, None),
        };
        let object = content
            .object_id(id)
            .ok_or_else(|| format!("unknown object `{id}`"))?;
        if content.object(object).corpse.is_some() {
            return Err(format!(
                "`{id}` is a corpse: corpses lie only where creatures died"
            ));
        }
        let stack = placement(content, object, made)?;
        if content.object(object).stackable {
            match stacks.iter_mut().find(|s| s.same_goods(&stack)) {
                Some(same) => same.count += count,
                None => stacks.push(stack.goods(count)),
            }
        } else {
            stacks.extend((0..count).map(|_| stack.clone()));
        }
    }
    Ok(stacks)
}

/// One of `object` as a placement names it: a loose gem (the gems' object) of the gem it names,
/// of the plain size and cut; anything else as it is, or made of the material it names (its
/// main part, which must be on its construction list).
fn placement(content: &Content, object: ObjectId, made: Option<&str>) -> Result<Stack, String> {
    let def = content.object(object);
    let id = &def.id;
    let loose = content.gems().filter(|g| g.object == object);
    match (loose, made) {
        (None, None) => Ok(Stack::one(object)),
        (None, Some(made)) => {
            let material = (content.material_id(made))
                .ok_or_else(|| format!("`{id}:{made}`: `{made}` is no material"))?;
            if content.material(material).gem.is_some() {
                return Err(format!(
                    "`{id}:{made}`: only a loose gem names its gem (`gem:{made}`); nothing is \
                     made of one"
                ));
            }
            let part = ItemPart::material(content, None, material);
            crate::item_parts::check(def, std::slice::from_ref(&part), content)
                .map_err(|m| format!("`{id}:{made}`: {m}"))?;
            Ok(Stack::one(object).with_parts(vec![part]))
        }
        (Some(gems), Some(gem)) => placed_gem(content, object, gems, gem),
        (Some(_), None) => Err(format!("`{id}` is a loose gem: name its gem, `{id}:ruby`")),
    }
}

/// One of the gems' `object` made of `gem`, of the plain size and cut.
fn placed_gem(
    content: &Content,
    object: ObjectId,
    gems: &super::Gems,
    gem: &str,
) -> Result<Stack, String> {
    let id = &content.object(object).id;
    let material = (content.material_id(gem))
        .filter(|m| content.material(*m).gem.is_some())
        .ok_or_else(|| format!("`{id}:{gem}`: `{gem}` is no gem (data/materials/)"))?;
    let part = ItemPart::gem(content, None, material, gems.plain());
    Ok(Stack::one(object).with_parts(vec![part]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn articles_and_plurals() {
        let def = |name: &str, plural: &str| ObjectDef {
            id: name.into(),
            name: name.into(),
            plural: plural.into(),
            written_name: name.into(),
            written_plural: plural.into(),
            glyph: ')',
            fg: Rgb::WHITE,
            class: ItemClassId(0),
            parts: vec![Part::whole(MaterialId(0))],
            construction: Vec::new(),
            ornament: None,
            weight: 1,
            value: 0,
            stackable: false,
            slot: None,
            two_handed: false,
            light_weapon: false,
            ammo: None,
            launches: None,
            attack: None,
            parry: 0,
            armour: 0,
            max_dex: None,
            protection: 0,
            spell_cost_percent: 0,
            shield: 0,
            bash: None,
            penetration: 0,
            verb: None,
            added: None,
            resist: Vec::new(),
            protects: Vec::new(),
            skill: None,
            range: None,
            reloads: false,
            breaks: 0,
            lit: None,
            burning: None,
            key: None,
            dig: None,
            missile: None,
            spins: false,
            glow: 0.0,
            particles: Vec::new(),
            kind: None,
            attack_cost: None,
            level: 1,
            frequency: 0,
            food: None,
            cures: Vec::new(),
            corpse: None,
            read: None,
            wand: None,
            rod: None,
            drink: None,
        };
        assert_eq!(def("dagger", "daggers").describe(1), "a dagger");
        assert_eq!(def("iron key", "iron keys").describe(1), "an iron key");
        assert_eq!(def("arrow", "arrows").describe(12), "12 arrows");
        assert_eq!(
            def("scroll of light", "scrolls of light").describe(2),
            "2 scrolls of light"
        );
    }
}
