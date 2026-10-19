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
//! Digging tools (spec §8 "Digging") have a `dig` power: the work a dig with one does against a
//! wall's hardness (a pick-axe 10, at Strength 10). They go in a slot (the Tool slot).

use super::light::LightId;
use super::material::MaterialId;
use super::particles::ParticleEmitter;
use super::slot::SlotDef;
use super::weapon_skill::{Role, WeaponSkillId, WeaponSkillTable};
use super::{Content, ContentError};
use crate::color::Rgb;
use crate::combat::{Attack, CRITS, DEFAULT_CRIT, DEFAULT_THREAT, THREATS};
use crate::dice::Dice;
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
/// The most one object may be worth, in gold coins.
pub const MAX_VALUE: u32 = 1_000_000;

/// An object's class, ADOM-like: its glyph says what kind of thing it is (spec §7 "Objects and
/// materials"). Shops sell and buy by class (spec §7 "Shops").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectClass {
    /// `)`
    Weapons,
    /// `}`
    MissileWeapons,
    /// `/`
    Missiles,
    /// `[`
    Armour,
    /// `(`
    Tools,
    /// `!`
    Potions,
    /// `?`
    Scrolls,
    /// `%`
    Food,
    /// `*`: stones and gems.
    Stones,
    /// `$`
    Coins,
    /// `=`
    Rings,
    /// `"`
    Amulets,
}

impl ObjectClass {
    /// Every class, in glyph order of the ADOM list.
    pub const ALL: [ObjectClass; 12] = [
        ObjectClass::Weapons,
        ObjectClass::MissileWeapons,
        ObjectClass::Missiles,
        ObjectClass::Armour,
        ObjectClass::Tools,
        ObjectClass::Potions,
        ObjectClass::Scrolls,
        ObjectClass::Food,
        ObjectClass::Stones,
        ObjectClass::Coins,
        ObjectClass::Rings,
        ObjectClass::Amulets,
    ];

    /// The class of objects drawn with `glyph`; `None` for a glyph no class uses.
    pub fn of_glyph(glyph: char) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.glyph() == glyph)
    }

    /// The glyph its objects are drawn with.
    pub const fn glyph(self) -> char {
        match self {
            ObjectClass::Weapons => ')',
            ObjectClass::MissileWeapons => '}',
            ObjectClass::Missiles => '/',
            ObjectClass::Armour => '[',
            ObjectClass::Tools => '(',
            ObjectClass::Potions => '!',
            ObjectClass::Scrolls => '?',
            ObjectClass::Food => '%',
            ObjectClass::Stones => '*',
            ObjectClass::Coins => '$',
            ObjectClass::Rings => '=',
            ObjectClass::Amulets => '"',
        }
    }

    /// What the class is called, in the plural: "missile weapons".
    pub const fn name(self) -> &'static str {
        match self {
            ObjectClass::Weapons => "weapons",
            ObjectClass::MissileWeapons => "missile weapons",
            ObjectClass::Missiles => "missiles",
            ObjectClass::Armour => "armour",
            ObjectClass::Tools => "tools",
            ObjectClass::Potions => "potions",
            ObjectClass::Scrolls => "scrolls",
            ObjectClass::Food => "food",
            ObjectClass::Stones => "stones and gems",
            ObjectClass::Coins => "coins",
            ObjectClass::Rings => "rings",
            ObjectClass::Amulets => "amulets",
        }
    }
}

/// Index of an object definition in [`super::Content::object_defs`].
///
/// Ids are assigned in file order when content loads, like terrain ids; saves record the content
/// fingerprint to catch a reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectId(pub u16);

/// Most of one object a level file may place with `"id*N"`.
pub const MAX_PLACED: u32 = 9999;

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
    /// What one is worth, in gold coins (shops; spec §7 "Shops").
    pub value: u32,
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
    /// What one is worth, in gold coins, at most [`MAX_VALUE`]; 0 for junk (spec §7 "Shops":
    /// nobody buys or sells it).
    pub value: u32,
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
}

/// How a burning object burns where it lies: the light it gives, and for how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Burning {
    pub light: LightId,
    pub lasts: Duration,
}

impl ObjectDef {
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

    /// Its class, from its glyph (ADOM-like); `None` for a glyph no class uses.
    pub fn class(&self) -> Option<ObjectClass> {
        ObjectClass::of_glyph(self.glyph)
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
        if self.value > MAX_VALUE {
            return Err(err(format!("`value` is in gold coins, 0 to {MAX_VALUE}")));
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
            value: self.value,
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
            dig: self.dig,
            missile: self.missile,
            spins: self.spins,
            glow: self.glow,
            particles: self.particles,
        };
        Ok((def, self.lit))
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
        if content.object(object).stackable {
            match stacks.iter_mut().find(|s| s.object == object) {
                Some(stack) => stack.count += count,
                None => stacks.push(Stack { object, count }),
            }
        } else {
            stacks.extend((0..count).map(|_| Stack { object, count: 1 }));
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
            value: 0,
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
