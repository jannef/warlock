//! What an item gives, in numbers (spec §6 "Item lists", "Item description"; the owner's
//! exception to hidden numbers, 2026-10-27): a weapon's to-hit and damage (and a launcher's or a
//! throwing weapon's range), what ammunition adds to a shot, and the Defence (Dv) and Protection
//! (Pv) a piece of armour, a shield, a parrying weapon, or a ring gives.
//!
//! The numbers are the item's own, never its wearer's: its base, its material, and its affixes
//! as the player knows them. They are figured on the stack as the player knows it
//! ([`Stack::as_known`]): an unidentified item's magic counts for nothing until it is
//! identified, so no hidden affix moves a number. The item lists write them after the name
//! ([`Numbers::text`], through `Lore::listed`), the item description on lines of their own
//! ([`Numbers::lines`]).

use super::{Effect, Mods};
use crate::combat::{Attack, DEFAULT_CRIT, DEFAULT_THREAT, Profile};
use crate::content::{Content, ItemKind};
use crate::damage::{Kinds, Rider};
use crate::dice::{Dice, Roll};
use crate::object::Stack;

/// What an item gives, as the player knows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Numbers {
    /// What it attacks with: a weapon's (or a launcher's) to-hit and damage, or what
    /// ammunition adds to a shot.
    pub attack: Option<AttackNumbers>,
    /// What it adds to its wearer's Defence and Protection.
    pub defence: Option<DefenceNumbers>,
}

/// What attack numbers are of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// A weapon's (or a launcher's) own attack.
    Weapon,
    /// What ammunition adds to the launcher's shot.
    Ammo,
    /// What something worn (a ring, a helm) adds to every attack: its known "+1 to hit with
    /// every attack" and "+2 damage with every attack".
    Worn,
}

/// A weapon's to-hit and damage, its own and its known affixes', or what ammunition or a worn
/// thing adds to attacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackNumbers {
    /// Added to the 2d12 attack roll.
    pub to_hit: i32,
    /// Its damage dice with every flat bonus folded in; for what only adds (ammunition, a worn
    /// thing), which has no dice of its own, only the flat bonus it adds.
    pub damage: Roll,
    /// What it is of.
    pub role: Role,
    /// More dice rolled with its own (an affix's "adds 1d4 damage").
    pub extra: Option<Dice>,
    /// Damage of other kinds added to every hit (a flaming arrow's, an elemental affix's).
    pub riders: Vec<Rider>,
    /// How far it shoots or is thrown, for a launcher or a throwing weapon.
    pub range: Option<i32>,
}

/// The Defence and Protection a worn or held thing gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefenceNumbers {
    /// What it adds to Defence: its armour, shield, or parry bonus, and the Dodge, Deflection,
    /// and natural armour its known affixes give.
    pub dv: i32,
    /// Its Protection: `None` for a shield or anything else that never has any.
    pub pv: Option<i32>,
}

impl Numbers {
    /// What `stack` gives, as the player knows it (module docs).
    pub fn of(stack: &Stack, content: &Content) -> Self {
        // A tool (a pick-axe) neither attacks nor defends: it shows no numbers, whatever its
        // affixes; they stay in its description.
        if content.object(stack.object).kind == Some(ItemKind::Tool) {
            return Self {
                attack: None,
                defence: None,
            };
        }
        let known = stack.as_known();
        Self {
            attack: attack(&known, content),
            defence: defence(&known, content),
        }
    }

    /// Whether there is nothing to say.
    pub fn is_empty(&self) -> bool {
        self.attack.is_none() && self.defence.is_none()
    }

    /// The numbers as the item lists write them, after the name in parentheses: `+2 hit,
    /// 1d8+1`, `+0 hit, 1d6, range 15`, `+1 hit, +1d4 fire` (ammunition), `Dv +1, Pv 2`, `Dv +2`
    /// (a shield), `+0 hit, 1d6, Dv +1` (a weapon that parries). `None` when there is nothing.
    pub fn text(&self, content: &Content) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(attack) = &self.attack {
            parts.push(attack.text(content));
        }
        if let Some(defence) = self.defence {
            parts.push(defence.text());
        }
        (!parts.is_empty()).then(|| parts.join(", "))
    }

    /// The numbers as the item description writes them, a line each: `To hit +2, damage
    /// 1d8+1, range 15`, `Adds to a shot: +1 to hit, +1d4 fire`, `Dv +1, Pv 2`.
    pub fn lines(&self, content: &Content) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(attack) = &self.attack {
            out.push(attack.line(content));
        }
        if let Some(defence) = self.defence {
            out.push(defence.text());
        }
        out
    }
}

impl AttackNumbers {
    /// Whether it only adds to attacks made with something else (ammunition, a worn thing).
    fn adds(&self) -> bool {
        self.role != Role::Weapon
    }

    /// Its damage dice and riders as written: `1d8+1`, `1d8+1d4+1`, `1d6 +1d6 fire`; for what
    /// only adds, what it adds, `+2 damage`, `+1d4+2 damage, +1d4 fire`, or nothing.
    pub fn damage_text(&self, content: &Content) -> String {
        let riders = (self.riders.iter())
            .map(|r| format!("+{} {}", r.dice, content.damage_type(r.kind).name));
        if !self.adds() {
            let mut text = dice_text(self.damage, self.extra);
            for rider in riders {
                text.push(' ');
                text.push_str(&rider);
            }
            return text;
        }
        let bonus = self.damage.bonus();
        let added = match self.extra {
            Some(extra) => {
                let mut t = format!("+{}", extra.plus(-extra.bonus()));
                if bonus != 0 {
                    t.push_str(&format!("{bonus:+}"));
                }
                Some(format!("{t} damage"))
            }
            None if bonus != 0 => Some(format!("{bonus:+} damage")),
            None => None,
        };
        let parts: Vec<String> = added.into_iter().chain(riders).collect();
        parts.join(", ")
    }

    /// As the item lists write it: `+2 hit, 1d8+1`, `+0 hit, 1d6, range 15`, `+1 hit, +2
    /// damage`.
    fn text(&self, content: &Content) -> String {
        let mut parts = Vec::new();
        if !self.adds() || self.to_hit != 0 {
            parts.push(format!("{:+} hit", self.to_hit));
        }
        let damage = self.damage_text(content);
        if !damage.is_empty() {
            parts.push(damage);
        }
        if let Some(range) = self.range {
            parts.push(format!("range {range}"));
        }
        parts.join(", ")
    }

    /// As the item description writes it: `To hit +2, damage 1d8+1, range 15`; for ammunition
    /// `Adds to a shot: +1 to hit, +2 damage, +1d4 fire`, for a worn thing `Adds to every
    /// attack: +1 to hit`.
    fn line(&self, content: &Content) -> String {
        let damage = self.damage_text(content);
        if self.adds() {
            let mut parts = Vec::new();
            if self.to_hit != 0 {
                parts.push(format!("{:+} to hit", self.to_hit));
            }
            if !damage.is_empty() {
                parts.push(damage);
            }
            let to = match self.role {
                Role::Ammo => "a shot",
                _ => "every attack",
            };
            return format!("Adds to {to}: {}", parts.join(", "));
        }
        let mut text = format!("To hit {:+}, damage {damage}", self.to_hit);
        if let Some(range) = self.range {
            text.push_str(&format!(", range {range}"));
        }
        text
    }

    /// Whether it adds nothing (plain ammunition, a plain ring).
    fn is_nothing(&self) -> bool {
        self.to_hit == 0 && self.damage.is_zero() && self.extra.is_none() && self.riders.is_empty()
    }
}

impl DefenceNumbers {
    /// `Dv +1, Pv 2`, or `Dv +2` without Protection.
    fn text(self) -> String {
        match self.pv {
            Some(pv) => format!("Dv {:+}, Pv {pv}", self.dv),
            None => format!("Dv {:+}", self.dv),
        }
    }
}

/// Damage dice as written: the dice (and any extra dice) with the flat bonus last, `1d8+2`,
/// `1d6+1d4+1`; fixed damage as its number (at least 1). The character sheet's attacks
/// (`OwnAttack::damage_text`) write theirs the same way.
pub fn dice_text(damage: Roll, extra: Option<Dice>) -> String {
    match (damage.is_fixed(), extra) {
        (true, None) => damage.bonus().max(1).to_string(),
        (_, None) => damage.to_string(),
        (_, Some(extra)) => {
            let bonus = damage.bonus() + extra.bonus();
            let mut text = format!(
                "{}+{}",
                damage.plus(-damage.bonus()),
                extra.plus(-extra.bonus())
            );
            if bonus != 0 {
                text.push_str(&format!("{bonus:+}"));
            }
            text
        }
    }
}

/// What `stack` (as known) attacks with: a weapon's or launcher's own attack with what its
/// material and affixes do to it (the very [`Mods::sharpen`] a strike uses), what ammunition
/// adds to a shot, or what anything else adds to every attack by its affixes' to-hit and
/// damage (`None` if it adds nothing).
fn attack(stack: &Stack, content: &Content) -> Option<AttackNumbers> {
    let def = content.object(stack.object);
    let (base, role, mods) = match def.attack {
        Some(attack) => (attack, Role::Weapon, Mods::local(stack, content)),
        None if def.ammo.is_some() => (nothing(), Role::Ammo, Mods::local(stack, content)),
        None => (nothing(), Role::Worn, Mods::global(stack, content)),
    };
    let mut profile = Profile::bare(&base);
    if role != Role::Worn {
        profile.riders.extend(def.added);
    }
    let range = (def.range)
        .filter(|_| role == Role::Weapon)
        .map(|range| range + mods.get(Effect::Range));
    if role == Role::Worn {
        // Only its to-hit and damage with every attack: the rest stays in its description.
        profile.to_hit += mods.get(Effect::ToHit);
        profile.bonus += mods.get(Effect::Damage);
    } else {
        mods.sharpen(&mut profile);
    }
    let numbers = AttackNumbers {
        to_hit: profile.to_hit,
        damage: profile.damage.plus(profile.bonus),
        role,
        extra: profile.extra,
        riders: profile.riders,
        range,
    };
    (role == Role::Weapon || !numbers.is_nothing()).then_some(numbers)
}

/// An attack of no dice: what ammunition or a worn thing adds to.
fn nothing() -> Attack {
    Attack {
        damage: Roll::fixed(0),
        to_hit: 0,
        threat: DEFAULT_THREAT,
        crit: DEFAULT_CRIT,
        strength: None,
        kinds: Kinds::NONE,
    }
}

/// What `stack` (as known) adds to Defence and Protection: armour shows both, a shield its Dv,
/// anything else (a parrying weapon, a ring of Dodge) only what it gives; `None` if nothing.
fn defence(stack: &Stack, content: &Content) -> Option<DefenceNumbers> {
    let def = content.object(stack.object);
    let local = Mods::local(stack, content);
    let global = Mods::global(stack, content);
    let dv = def.armour
        + def.shield
        + def.parry
        + local.get(Effect::Armour)
        + local.get(Effect::Shield)
        + local.get(Effect::Parry)
        + global.get(Effect::Dodge)
        + global.get(Effect::Deflection)
        + global.get(Effect::Natural);
    let pv = stack.protection(content);
    let armour = def.kind.is_some_and(ItemKind::is_armour);
    match (armour, dv, pv) {
        (true, _, _) => Some(DefenceNumbers { dv, pv: Some(pv) }),
        (false, 0, 0) => None,
        (false, _, 0) => Some(DefenceNumbers { dv, pv: None }),
        (false, _, _) => Some(DefenceNumbers { dv, pv: Some(pv) }),
    }
}
