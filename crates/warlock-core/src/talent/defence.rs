//! The defence talents (#1495): what a talent does to the holder's defences: Defence, Protection,
//! natural armour, combat stances, pain, and resistances. [`DefenceEffect`] is the data (a talent's
//! `[[effect]]` of one of [`DefenceEffect::KINDS`]), [`DefenceMods`] the sum every rule reads
//! (`TalentMods::defence`). A new effect of the area is a variant here, its field and its arm in
//! [`DefenceMods::add`] here, its wiki text in `xtask/src/wiki/talents/defence.rs`, and its one
//! read.

use crate::content::CombatStance;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DefenceEffect {
    /// Defence in the Dodge category (bonuses add up).
    Dodge { value: i32 },
    /// Natural armour: added to the actor's own, whatever the best item gives.
    Natural { value: i32 },
    /// Worn armour's cap on Dexterity in Defence (`max_dex`), this much higher.
    MaxDex { value: i32 },
    /// Defence in the Deflection category, on top of what magic items give.
    Deflection { value: i32 },
    /// Protection, added to what is worn; only while the body armour worn has `body_armour`
    /// Protection at least (if set), only while holding a shield (if `shield`), and once for
    /// every armour piece worn (if `per_piece`).
    Protection {
        value: i32,
        #[serde(default)]
        body_armour: Option<i32>,
        #[serde(default)]
        shield: bool,
        /// For every piece of armour worn (anything worn with Protection of its own).
        #[serde(default)]
        per_piece: bool,
    },
    /// The to-hit penalty of combat stance `stance`, this percent smaller (100: none).
    StanceToHit { stance: CombatStance, percent: i32 },
    /// The Dodge penalty of combat stance `stance`, this percent smaller (100: none; Iron
    /// stance).
    StanceDodge { stance: CombatStance, percent: i32 },
    /// Changing its combat stance costs an action, as a wait does (Set in its ways), not
    /// nothing.
    StanceCosts,
    /// A step that takes it out of reach of every hostile beside it costs this percent more
    /// (negative: cheaper; Slip away), whatever its stance.
    SlipPercent { value: i32 },
    /// Every hit on the actor (a blow, a shot, a throw, a trap) does this much less after its
    /// Protection soaked it, never below 1.
    Pain { value: i32 },
    /// A creature whose melee blow hits it takes `damage` (Barbed hide): no blow, no roll, and
    /// no experience if it dies of it.
    Barbs { damage: i32 },
    /// Resistance in percent to damage type `damage` (an id in `data/damage_types.toml`), else
    /// to the chosen element (the talent takes an `element` choice).
    Resist {
        value: i32,
        #[serde(default)]
        damage: Option<String>,
    },
    /// Dodge added against shots, throws, and trap missiles.
    MissileDodge { value: i32 },
    /// Dodge added against the attacks of creatures larger than it (Small target).
    SmallTarget { value: i32 },
    /// The damage bursts and bolts (traps, scrolls) do it, this percent more (negative: less).
    SpellDamagePercent { value: i32 },
    /// Defence in the Deflection category, only while it wears no body armour (Mage armour).
    DeflectionUnarmoured { value: i32 },
}

impl DefenceEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`DefenceEffect`]).
    pub const KINDS: [&'static str; 16] = [
        "barbs",
        "deflection",
        "deflection_unarmoured",
        "dodge",
        "max_dex",
        "missile_dodge",
        "natural",
        "pain",
        "protection",
        "resist",
        "slip_percent",
        "small_target",
        "spell_damage_percent",
        "stance_costs",
        "stance_dodge",
        "stance_to_hit",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DefenceMods {
    /// Defence in the Dodge category.
    pub dodge: i32,
    /// Natural armour, on top of the actor's own.
    pub natural: i32,
    /// Added to worn armour's cap on Dexterity in Defence.
    pub max_dex: i32,
    /// Defence in the Deflection category.
    pub deflection: i32,
    /// Protection, each with what it asks to be worn ([`DefenceMods::protection_with`]).
    pub protection: Vec<Guard>,
    /// Percent less of a stance's to-hit penalty, by stance.
    pub stance_to_hit: Vec<(CombatStance, i32)>,
    /// Percent more a step out of reach of every hostile beside it costs (negative: cheaper).
    pub slip_percent: i32,
    /// Taken off every hit after the soak, never below 1.
    pub pain: i32,
    /// Damage a creature whose melee blow hits it takes (0: none).
    pub barbs: i32,
    /// Resistances in percent, by element id.
    pub resist: Vec<(String, i32)>,
    /// Dodge added against shots, throws, and trap missiles.
    pub missile_dodge: i32,
    /// Dodge added against creatures larger than it.
    pub small_target: i32,
    /// Percent more damage bursts and bolts do it (negative: less).
    pub spell_damage_percent: i32,
    /// Percent less of a stance's Dodge penalty, by stance.
    pub stance_dodge: Vec<(CombatStance, i32)>,
    /// Changing its combat stance costs an action.
    pub stance_costs: bool,
    /// Deflection while it wears no body armour.
    pub deflection_unarmoured: i32,
}

impl DefenceMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, effect: DefenceEffect, choice: Option<&str>) {
        match effect {
            DefenceEffect::Dodge { value } => self.dodge += value,
            DefenceEffect::Natural { value } => self.natural += value,
            DefenceEffect::MaxDex { value } => self.max_dex += value,
            DefenceEffect::Deflection { value } => self.deflection += value,
            DefenceEffect::Protection {
                value,
                body_armour,
                shield,
                per_piece,
            } => self.protection.push(Guard {
                value,
                body_armour,
                shield,
                per_piece,
            }),
            DefenceEffect::StanceToHit { stance, percent } => {
                match self.stance_to_hit.iter_mut().find(|(s, _)| *s == stance) {
                    Some((_, p)) => *p += percent,
                    None => self.stance_to_hit.push((stance, percent)),
                }
            }
            DefenceEffect::SlipPercent { value } => self.slip_percent += value,
            DefenceEffect::Pain { value } => self.pain += value,
            DefenceEffect::Barbs { damage } => self.barbs += damage,
            DefenceEffect::Resist { value, damage } => {
                let element = damage.as_deref().or(choice).unwrap_or_default();
                match self.resist.iter_mut().find(|(e, _)| e == element) {
                    Some((_, r)) => *r += value,
                    None => self.resist.push((element.to_string(), value)),
                }
            }
            DefenceEffect::MissileDodge { value } => self.missile_dodge += value,
            DefenceEffect::SmallTarget { value } => self.small_target += value,
            DefenceEffect::SpellDamagePercent { value } => self.spell_damage_percent += value,
            DefenceEffect::DeflectionUnarmoured { value } => self.deflection_unarmoured += value,
            DefenceEffect::StanceCosts => self.stance_costs = true,
            DefenceEffect::StanceDodge { stance, percent } => {
                match self.stance_dodge.iter_mut().find(|(s, _)| *s == stance) {
                    Some((_, p)) => *p += percent,
                    None => self.stance_dodge.push((stance, percent)),
                }
            }
        }
    }

    /// `dodge` (a stance's) with its penalty softened as its talents say for `stance`.
    pub fn stance_dodge(&self, stance: CombatStance, dodge: i32) -> i32 {
        let percent = (self.stance_dodge.iter())
            .filter(|(s, _)| *s == stance)
            .map(|(_, p)| *p)
            .sum::<i32>()
            .clamp(0, 100);
        if dodge >= 0 || percent == 0 {
            dodge
        } else {
            -((-dodge) * (100 - percent) / 100)
        }
    }

    /// The Protection its talents add while it wears body armour of `body` Protection (none:
    /// no body armour), holds a shield or not, and wears `pieces` pieces of armour.
    pub fn protection_with(&self, body: Option<i32>, shield: bool, pieces: i32) -> i32 {
        (self.protection.iter())
            .filter(|g| g.body_armour.is_none_or(|at| body.is_some_and(|b| b >= at)))
            .filter(|g| !g.shield || shield)
            .map(|g| {
                if g.per_piece {
                    g.value * pieces
                } else {
                    g.value
                }
            })
            .sum()
    }

    /// `to_hit` (a stance's) with its penalty softened as its talents say for `stance`.
    pub fn stance_to_hit(&self, stance: CombatStance, to_hit: i32) -> i32 {
        let percent = (self.stance_to_hit.iter())
            .filter(|(s, _)| *s == stance)
            .map(|(_, p)| *p)
            .sum::<i32>()
            .clamp(0, 100);
        if to_hit >= 0 || percent == 0 {
            to_hit
        } else {
            // Rounded toward 0: half of −3 is −1.
            -((-to_hit) * (100 - percent) / 100)
        }
    }
}

/// Protection a talent adds, while what it asks is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    pub value: i32,
    /// Only while the body armour worn has this much Protection at least.
    pub body_armour: Option<i32>,
    /// Only while holding a shield.
    pub shield: bool,
    /// Once for every armour piece worn.
    pub per_piece: bool,
}
