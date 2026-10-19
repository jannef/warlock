//! Object definitions (`assets/data/objects.toml`, spec §7): the things that lie on the floor and
//! are picked up and carried.
//!
//! Every object defines the glyph it is drawn with, its color, its material, and its weight in
//! grams. Glyphs follow ADOM loosely: `)` weapons, `/` missiles, `}` missile weapons, `[` armor,
//! `(` tools, `!` potions, `?` scrolls, `%` food, `*` stones and gems, `$` coins, `=` rings,
//! `"` amulets.
//!
//! Things that can be worn or wielded say where they go (`slot`, the id or `fits` of a slot in
//! `data/slots.toml`: `slot = "body"`, `slot = "ring"` for either ring slot); a `two_handed` one
//! takes both hands. Missiles say what kind of ammunition they are (`ammo = "arrow"`) and missile
//! weapons what they shoot (`launches = "arrow"`): launchers and ammunition pair by that kind.
//!
//! Weapons and armour say what they do in a fight (spec §8 "Combat"): a weapon its `damage` dice
//! (a launcher's carry its draw weight, `1d8+2`), its critical (`threat`, the lowest natural roll
//! that threatens, default 22; `crit`, the damage multiplier, default 2), an optional `to_hit`,
//! a `parry` bonus to Defence while wielded, and for bows the `strength` they need; armour its
//! `armour` bonus and the most Dexterity bonus it allows (`max_dex`); a shield its `shield` bonus,
//! and the `bash` dice of a blow with it (spec §8 "Weapon skills": the shield bash).
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
//! Food (spec §8 "Hunger") says what it gives and how long it takes to eat: `food = { nutrition
//! = 1500, turns = 4 }`. Corpses are objects too, one made for each creature type that leaves
//! one ([`ObjectDef::corpse_of`]): "kobold corpse", `%` in the creature's color, weighing what
//! the creature does, feeding by its size. Level files cannot place them: they lie where
//! creatures died, each with the moment it died ([`Stack::died`]), from which it rots.
//!
//! Digging tools (spec §8 "Digging") have a `dig` power: the work a dig with one does against a
//! wall's hardness (a pick-axe 10, at Strength 10). They go in a slot (the Tool slot).

use super::action::check_cost;
use super::affix::{ItemKind, MAX_ITEM_LEVEL};
use super::creature::CreatureDef;
use super::hunger::Food;
use super::light::LightId;
use super::material::MaterialId;
use super::particles::ParticleEmitter;
use super::slot::SlotDef;
use super::weapon_skill::{Role, WeaponSkillId, WeaponSkillTable};
use super::{Content, ContentError, CreatureId};
use crate::color::Rgb;
use crate::combat::{Attack, CRITS, DEFAULT_CRIT, DEFAULT_THREAT, THREATS};
use crate::dice::Dice;
use crate::door::Key;
use crate::object::Stack;
use crate::time::{DAY, Duration};
use serde::{Deserialize, Serialize};

/// The largest Defence bonus one object may give, and the largest `to_hit` either way.
pub const MAX_BONUS: i32 = 20;
/// The farthest anything shoots or is thrown, in cells: as far as anyone sees.
pub const MAX_RANGE: i32 = crate::perception::VIEW_RADIUS;
/// The longest anything burns where it lies, in seconds: a day.
pub const MAX_BURN: u32 = DAY as u32;
/// The most power a digging tool may have (a pick-axe has 10).
pub const MAX_DIG: u16 = 100;

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
    /// A material id in `materials.toml`.
    pub material: String,
    /// Grams, at least 1.
    pub weight: u32,
    #[serde(default)]
    pub stackable: bool,
    /// Where it is worn or wielded: a slot's `fits` (its id unless it says otherwise).
    pub slot: Option<String>,
    /// Takes both hands (only for things that go in a hand).
    #[serde(default)]
    pub two_handed: bool,
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
    /// Defence while held (the Shield category).
    #[serde(default)]
    pub shield: i32,
    /// A shield's damage dice when bashing with it.
    pub bash: Option<Dice>,
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
}

/// A resolved object type.
#[derive(Clone, Debug)]
pub struct ObjectDef {
    pub id: String,
    /// Singular, without an article: "dagger".
    pub name: String,
    /// "daggers".
    pub plural: String,
    pub glyph: char,
    pub fg: Rgb,
    pub material: MaterialId,
    /// Weight of one, in grams; at least 1.
    pub weight: u32,
    /// Several lie in one stack (coins, arrows) instead of a stack each.
    pub stackable: bool,
    /// Where it is worn or wielded: the `fits` of the slots it goes in (`None`: nowhere).
    pub slot: Option<String>,
    /// Takes both hands: every hand slot.
    pub two_handed: bool,
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
    /// Defence while held: its Shield bonus.
    pub shield: i32,
    /// What a bash with it does, if it is a shield one can bash with.
    pub bash: Option<Attack>,
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
    /// How often loot holds it, relative to the rest (0 for what has no `kind`).
    pub frequency: u32,
    /// What eating it gives, if it is food or a corpse (spec §8 "Hunger").
    pub food: Option<Food>,
    /// The creature type it is the corpse of, if it is a corpse: it rots from the moment its
    /// creature died ([`Stack::died`]).
    pub corpse: Option<CreatureId>,
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
        material: MaterialId,
        food: Food,
    ) -> Self {
        Self {
            id: format!("{}_corpse", creature.id),
            name: format!("{} corpse", creature.name),
            plural: format!("{} corpses", creature.name),
            glyph: '%',
            fg: creature.fg,
            material,
            weight: creature.weight,
            stackable: false,
            slot: None,
            two_handed: false,
            ammo: None,
            launches: None,
            attack: None,
            parry: 0,
            armour: 0,
            max_dex: None,
            shield: 0,
            bash: None,
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
            corpse: Some(id),
            key: None,
            kind: None,
            attack_cost: None,
            level: 1,
            frequency: 0,
        }
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
        if count == 1 {
            format!("{} {}", article(&self.name), self.name)
        } else {
            format!("{count} {}", self.plural)
        }
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
    /// Checks the definition and resolves its material, light, and weapon skill; `slots` are
    /// where things may be worn or wielded, `skills` the weapon skills. Returns the object and
    /// the id of what it is `lit` as, which names another object and is resolved once every
    /// object is known.
    pub(super) fn resolve(
        self,
        file: &str,
        material: impl Fn(&str) -> Option<MaterialId>,
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
        let material = material(&self.material)
            .ok_or_else(|| err(format!("unknown material `{}`", self.material)))?;
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
        for (field, kind) in [("ammo", &self.ammo), ("launches", &self.launches)] {
            if kind.as_ref().is_some_and(|k| k.trim().is_empty()) {
                return Err(err(format!("`{field}` must not be empty")));
            }
        }
        let attack = self.attack().map_err(err)?;
        self.check_defence().map_err(err)?;
        self.check_ranged().map_err(err)?;
        let skill = self.skill(slots, skills).map_err(err)?;
        let bash = match (self.bash, self.shield) {
            (None, _) => None,
            (Some(damage), shield) if shield > 0 => Some(Attack {
                damage,
                ..Attack::unarmed()
            }),
            (Some(_), _) => return Err(err("`bash` is for shields (`shield`)".into())),
        };
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
        let plural = self.plural.unwrap_or_else(|| format!("{}s", self.name));
        let def = ObjectDef {
            id: self.id,
            name: self.name,
            plural,
            glyph: self.glyph,
            fg: self.fg,
            material,
            weight: self.weight,
            stackable: self.stackable,
            slot: self.slot,
            two_handed: self.two_handed,
            ammo: self.ammo,
            launches: self.launches,
            attack,
            parry: self.parry,
            armour: self.armour,
            max_dex: self.max_dex,
            shield: self.shield,
            bash,
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
            frequency: match self.kind {
                Some(_) => self.frequency.unwrap_or(100),
                None => 0,
            },
            food: self.food,
            corpse: None,
        };
        Ok((def, self.lit))
    }

    /// The fields of a base item: its `kind` fits what it is (a weapon has an attack and goes in
    /// a hand, ...), and `level` and `frequency` are for
    /// things with a kind.
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
        } else if self.level.is_some() || self.frequency.is_some() {
            return Err("`level` and `frequency` are for base items, which have a `kind`".into());
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
    /// worn or wielded is a weapon, and a missile weapon must be one.
    fn attack(&self) -> Result<Option<Attack>, String> {
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
}

/// Parses objects as level files place them: ids, each optionally with a count (`"arrow*12"`),
/// first on top. A stackable object becomes one stack (the same id twice joins it); any other
/// object becomes a stack of one per count.
pub(super) fn parse_stacks(specs: &[String], content: &Content) -> Result<Vec<Stack>, String> {
    let mut stacks: Vec<Stack> = Vec::new();
    for spec in specs {
        let (id, count) = match spec.split_once('*') {
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
        let object = content
            .object_id(id)
            .ok_or_else(|| format!("unknown object `{id}`"))?;
        if content.object(object).corpse.is_some() {
            return Err(format!(
                "`{id}` is a corpse: corpses lie only where creatures died"
            ));
        }
        if content.object(object).stackable {
            match stacks.iter_mut().find(|s| s.object == object) {
                Some(stack) => stack.count += count,
                None => stacks.push(Stack::new(object, count)),
            }
        } else {
            stacks.extend((0..count).map(|_| Stack::one(object)));
        }
    }
    Ok(stacks)
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
            glyph: ')',
            fg: Rgb::WHITE,
            material: MaterialId(0),
            weight: 1,
            stackable: false,
            slot: None,
            two_handed: false,
            ammo: None,
            launches: None,
            attack: None,
            parry: 0,
            armour: 0,
            max_dex: None,
            shield: 0,
            bash: None,
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
            corpse: None,
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
