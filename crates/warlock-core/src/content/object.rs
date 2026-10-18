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
//! `armour` bonus and the most Dexterity bonus it allows (`max_dex`); a shield its `shield` bonus.

use super::material::MaterialId;
use super::slot::SlotDef;
use super::{Content, ContentError};
use crate::color::Rgb;
use crate::combat::{Attack, CRITS, DEFAULT_CRIT, DEFAULT_THREAT, THREATS};
use crate::dice::Dice;
use crate::object::Stack;
use serde::{Deserialize, Serialize};

/// The largest Defence bonus one object may give, and the largest `to_hit` either way.
pub const MAX_BONUS: i32 = 20;

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
}

impl ObjectDef {
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
    /// Checks the definition and resolves its material; `slots` are where things may be worn or
    /// wielded.
    pub(super) fn resolve(
        self,
        file: &str,
        material: impl Fn(&str) -> Option<MaterialId>,
        slots: &[SlotDef],
    ) -> Result<ObjectDef, ContentError> {
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
        let plural = self.plural.unwrap_or_else(|| format!("{}s", self.name));
        Ok(ObjectDef {
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
        })
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
        if self.slot.is_none() {
            return Err("a weapon (`damage`) must go in a slot (`slot`)".into());
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
