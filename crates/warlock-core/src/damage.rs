//! Typed damage (spec §8 "Damage types"): what kind of harm an amount of damage is, what a
//! target resists of it, and how its Protection stands against each physical kind. The pure
//! model; `combat::strike` and the world's blows, bursts, falls, and plunges apply it.
//!
//! - [`Damage`] is an amount of one kind ([`DamageTypeId`], or none: untyped harm such as
//!   starving, a disease, or a bleeding wound's drip, which nothing resists).
//! - [`Kinds`] are the kinds an attack may do (a sword: slashing or piercing): a blow does
//!   **whichever is better against its target** ([`Kinds::best`]): the kind the target resists
//!   least, then the one its Protection stands least against, ties to the first named. No draw.
//! - A [`Rider`] is damage added to an attack of another kind (a flaming arrow's 1d4 fire, a
//!   magic weapon's "adds 1d4 fire damage"): rolled after the attack's own dice, each kind
//!   resisted on its own, never soaked.
//! - [`Resistances`] are a target's percent per kind: negative is vulnerability. Every source
//!   (its type, its race, gear and affixes, talents, statuses) adds up with no cap, so a total
//!   may stand at 200 or −150; what counts is the **effective** resistance ([`effective`], read
//!   through [`Resistances::get`]): the total clamped to [`MAX_RESISTANCE`] (75) above and
//!   [`MIN_RESISTANCE`] (−100) below (#1620). Immunity is no resistance: a creature type's
//!   `immune` kinds take nothing ([`IMMUNE`], 100), whatever else it has of that kind.
//!   [`resist`] takes the percentage off an amount, rounded to the nearest (half up), so a
//!   1-point hit at 50 % still hurts and at 75 % does not.
//! - [`Protection`] is a target's Pv against each kind: its base for the physical kinds, each
//!   adjusted by what the pieces worn say (chain mail +1 against slashing, −1 against piercing),
//!   nothing against an element, and the base against untyped damage.

use crate::content::{Content, DamageTypeId};
use crate::dice::Dice;
use serde::{Deserialize, Serialize};

/// What an immunity reads as: all of a kind is taken off, nothing hurts.
pub const IMMUNE: i32 = 100;
/// The most resistance that counts, whatever the total (the effective cap).
pub const MAX_RESISTANCE: i32 = 75;
/// The lowest resistance that counts: a vulnerability that doubles the damage.
pub const MIN_RESISTANCE: i32 = -100;
/// The most kinds one attack may name (a sword: slashing or piercing).
pub const MAX_KINDS: usize = 2;

/// An amount of damage of one kind (`None`: untyped harm, which nothing resists).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Damage {
    pub amount: i32,
    pub kind: Option<DamageTypeId>,
}

impl Damage {
    pub const fn new(amount: i32, kind: Option<DamageTypeId>) -> Self {
        Self { amount, kind }
    }

    /// Untyped harm: starving, a disease, a wound's bleeding.
    pub const fn untyped(amount: i32) -> Self {
        Self { amount, kind: None }
    }
}

/// The resistance that counts of a `total` added up from every source: at most
/// [`MAX_RESISTANCE`], at least [`MIN_RESISTANCE`] (#1620). The one place the cap is applied;
/// every reader goes through [`Resistances::get`], which calls it.
pub const fn effective(total: i32) -> i32 {
    if total > MAX_RESISTANCE {
        MAX_RESISTANCE
    } else if total < MIN_RESISTANCE {
        MIN_RESISTANCE
    } else {
        total
    }
}

/// `amount` less `percent` of it, rounded to the nearest (half up): what a target of that
/// resistance takes. A negative percent is a vulnerability and adds; 100 takes everything.
pub fn resist(amount: i32, percent: i32) -> i32 {
    if amount <= 0 {
        return 0;
    }
    let kept = 100 - percent.clamp(MIN_RESISTANCE, IMMUNE);
    (i64::from(amount) * i64::from(kept) + 50).div_euclid(100) as i32
}

/// The kinds of damage an attack may do, in the order data names them: one, two, or none (an
/// untyped attack, without `data/damage_types.toml`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Kinds([Option<DamageTypeId>; MAX_KINDS]);

impl Kinds {
    /// No kind: untyped.
    pub const NONE: Kinds = Kinds([None; MAX_KINDS]);

    /// One kind.
    pub const fn one(kind: DamageTypeId) -> Self {
        Kinds([Some(kind), None])
    }

    /// The kinds listed, at most [`MAX_KINDS`] (`None` for more).
    pub fn of(kinds: &[DamageTypeId]) -> Option<Self> {
        if kinds.len() > MAX_KINDS {
            return None;
        }
        let mut out = [None; MAX_KINDS];
        for (slot, kind) in out.iter_mut().zip(kinds) {
            *slot = Some(*kind);
        }
        Some(Kinds(out))
    }

    /// The kinds, in order.
    pub fn iter(&self) -> impl Iterator<Item = DamageTypeId> + '_ {
        self.0.iter().flatten().copied()
    }

    /// The first kind named: what an attack is called by.
    pub fn main(&self) -> Option<DamageTypeId> {
        self.0[0]
    }

    pub fn is_empty(&self) -> bool {
        self.0[0].is_none()
    }

    /// The kind that is best against a target: the one it resists least (`resistance`), then
    /// the one its Protection stands least against (`protection`), ties to the first named.
    /// `None` for an untyped attack. Decided before anything is rolled: no draw.
    pub fn best(
        &self,
        resistance: impl Fn(DamageTypeId) -> i32,
        protection: impl Fn(DamageTypeId) -> i32,
    ) -> Option<DamageTypeId> {
        let mut best: Option<(i32, i32, DamageTypeId)> = None;
        for kind in self.iter() {
            let key = (resistance(kind), protection(kind));
            if best.is_none_or(|(r, p, _)| key < (r, p)) {
                best = Some((key.0, key.1, kind));
            }
        }
        best.map(|(_, _, kind)| kind)
    }
}

/// Damage of another kind added to an attack: its dice, rolled with every roll of the attack's
/// own, after them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rider {
    pub kind: DamageTypeId,
    pub dice: Dice,
}

/// A target's resistance to each kind, in percent: one total per damage type in content order,
/// added up from every source with no cap, and the kinds it is immune to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resistances {
    total: Vec<i32>,
    immune: Vec<bool>,
}

impl Resistances {
    /// No resistance to anything (`count` kinds).
    pub fn none(count: usize) -> Self {
        Self {
            total: vec![0; count],
            immune: vec![false; count],
        }
    }

    /// The kinds in `immune` (a creature type's immunities) and every (kind, percent) of
    /// `sources` (its type's and race's, what it wears, its affixes, talents, and statuses)
    /// added up: the totals are kept as they are, uncapped; [`Resistances::get`] caps them.
    pub fn sum(
        count: usize,
        immune: impl IntoIterator<Item = DamageTypeId>,
        sources: impl IntoIterator<Item = (DamageTypeId, i32)>,
    ) -> Self {
        let mut out = Self::none(count);
        for kind in immune {
            if let Some(slot) = out.immune.get_mut(usize::from(kind.0)) {
                *slot = true;
            }
        }
        for (kind, percent) in sources {
            if let Some(slot) = out.total.get_mut(usize::from(kind.0)) {
                *slot = slot.saturating_add(percent);
            }
        }
        out
    }

    /// The resistance to `kind` that counts, in percent: [`IMMUNE`] for a kind it is immune to,
    /// else its total made [`effective`] (−100 to 75); 0 for untyped damage and unknown kinds.
    /// What every rule reads.
    pub fn get(&self, kind: Option<DamageTypeId>) -> i32 {
        let Some(i) = kind.map(|k| usize::from(k.0)) else {
            return 0;
        };
        if self.immune.get(i).copied().unwrap_or(false) {
            return IMMUNE;
        }
        effective(self.total.get(i).copied().unwrap_or(0))
    }

    /// The total resistance to `kind` added up from every source, uncapped (an immunity aside):
    /// what is left after a later penalty is [`effective`] of it less the penalty.
    pub fn total(&self, kind: Option<DamageTypeId>) -> i32 {
        kind.and_then(|k| self.total.get(usize::from(k.0)))
            .copied()
            .unwrap_or(0)
    }

    /// Every kind resisted, suffered, or immune, with the percent that counts
    /// ([`Resistances::get`]), in content order (0 left out).
    pub fn iter(&self) -> impl Iterator<Item = (DamageTypeId, i32)> + '_ {
        (0..self.total.len())
            .map(|i| DamageTypeId(i as u8))
            .map(|k| (k, self.get(Some(k))))
            .filter(|(_, p)| *p != 0)
    }

    /// Whether there is nothing to tell.
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
}

/// How a resistance reads to the player: never a number (spec §6 "Character screen").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResistanceWord {
    /// Below 0: it takes more.
    Vulnerable,
    /// Above 0, short of immunity.
    Resistant,
    /// An immunity ([`IMMUNE`]): nothing of that kind hurts it.
    Immune,
}

impl ResistanceWord {
    /// The word for `percent`; `None` at 0.
    pub fn of(percent: i32) -> Option<Self> {
        match percent {
            0 => None,
            IMMUNE.. => Some(ResistanceWord::Immune),
            1.. => Some(ResistanceWord::Resistant),
            _ => Some(ResistanceWord::Vulnerable),
        }
    }

    /// `resistant`.
    pub const fn word(self) -> &'static str {
        match self {
            ResistanceWord::Vulnerable => "vulnerable",
            ResistanceWord::Resistant => "resistant",
            ResistanceWord::Immune => "immune",
        }
    }

    /// `Resistant to`.
    pub const fn label(self) -> &'static str {
        match self {
            ResistanceWord::Vulnerable => "Vulnerable to",
            ResistanceWord::Resistant => "Resistant to",
            ResistanceWord::Immune => "Immune to",
        }
    }
}

/// A target's Protection (spec §8 "Combat", "Pv") as each kind of damage meets it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Protection {
    /// The total, as the player is shown it: what every piece worn and the hide give.
    pub base: i32,
    /// Against each kind, in content order: the base plus the pieces' adjustments for a
    /// physical kind, 0 for an element.
    against: Vec<i32>,
}

impl Protection {
    /// `base` Protection against every physical kind of `content`, adjusted by `adjustments`
    /// (kind, more or less) summed per kind; nothing against the elements.
    pub fn new(
        content: &Content,
        base: i32,
        adjustments: impl IntoIterator<Item = (DamageTypeId, i32)>,
    ) -> Self {
        let mut against: Vec<i32> = (content.damage_types().iter())
            .map(|t| if t.physical { base } else { 0 })
            .collect();
        for (kind, more) in adjustments {
            if let Some(slot) = against.get_mut(usize::from(kind.0))
                && content.damage_type(kind).physical
            {
                *slot = slot.saturating_add(more);
            }
        }
        Self { base, against }
    }

    /// `base` with `against` given outright (unit tests without content).
    #[cfg(test)]
    pub(crate) fn for_test(base: i32, against: Vec<i32>) -> Self {
        Self { base, against }
    }

    /// Plain Protection: `base` against every kind, for content without damage types.
    pub fn plain(base: i32) -> Self {
        Self {
            base,
            against: Vec::new(),
        }
    }

    /// What stands against damage of `kind`: the adjusted base for a physical kind, nothing for
    /// an element, the base for untyped damage (and for a kind the content does not know);
    /// never below 0.
    pub fn against(&self, kind: Option<DamageTypeId>) -> i32 {
        let pv = match kind {
            None => self.base,
            Some(k) => (self.against.get(usize::from(k.0)).copied()).unwrap_or(self.base),
        };
        pv.max(0)
    }

    /// The adjustments per kind that differ from the base, in content order: for descriptions
    /// ("+1 against slashing").
    pub fn adjustments(&self) -> impl Iterator<Item = (DamageTypeId, i32)> + '_ {
        (self.against.iter().enumerate())
            .filter(|(_, pv)| **pv != self.base && **pv != 0)
            .map(|(i, pv)| (DamageTypeId(i as u8), pv - self.base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resistance_rounds_to_the_nearest_half_up() {
        assert_eq!(resist(1, 0), 1);
        assert_eq!(resist(1, 25), 1);
        assert_eq!(resist(1, 50), 1);
        assert_eq!(resist(1, 75), 0);
        assert_eq!(resist(2, 50), 1);
        assert_eq!(resist(3, 50), 2);
        assert_eq!(resist(4, 75), 1);
        assert_eq!(resist(10, 30), 7);
        assert_eq!(resist(7, IMMUNE), 0);
        assert_eq!(resist(7, 150), 0, "clamped");
        assert_eq!(resist(1, -50), 2);
        assert_eq!(resist(3, -100), 6);
        assert_eq!(resist(3, -300), 6, "clamped");
        assert_eq!(resist(0, -50), 0);
        assert_eq!(resist(-4, 0), 0);
    }

    #[test]
    fn kinds_pick_the_least_resisted_then_the_least_protected_then_the_first() {
        let (a, b) = (DamageTypeId(0), DamageTypeId(1));
        let both = Kinds::of(&[a, b]).expect("two kinds");
        assert_eq!(both.best(|k| if k == a { 50 } else { 0 }, |_| 0), Some(b));
        assert_eq!(both.best(|_| 0, |k| if k == a { 3 } else { 1 }), Some(b));
        assert_eq!(both.best(|_| 0, |_| 2), Some(a), "ties to the first");
        // Resistance comes before Protection.
        assert_eq!(
            both.best(
                |k| if k == b { 10 } else { 0 },
                |k| if k == a { 5 } else { 0 }
            ),
            Some(a)
        );
        assert_eq!(Kinds::NONE.best(|_| 0, |_| 0), None);
        assert_eq!(Kinds::of(&[a, b, a]), None, "at most two");
        assert_eq!(Kinds::one(b).main(), Some(b));
    }

    #[test]
    fn resistances_add_up_uncapped_and_count_capped_and_immunity_is_apart() {
        let (fire, cold) = (DamageTypeId(0), DamageTypeId(1));
        let r = Resistances::sum(3, [], [(fire, 20), (fire, 30), (cold, -40), (fire, 40)]);
        assert_eq!(r.total(Some(fire)), 90, "kept as added up");
        assert_eq!(r.get(Some(fire)), MAX_RESISTANCE, "75 counts");
        assert_eq!(r.get(Some(cold)), -40);
        assert_eq!(r.get(Some(DamageTypeId(2))), 0);
        assert_eq!(r.get(None), 0);
        // The owner's case (#1620): 200 fire counts as 75, and a −100 on top still leaves 75.
        let stacked = Resistances::sum(1, [], [(fire, 120), (fire, 80)]);
        assert_eq!(
            (stacked.total(Some(fire)), stacked.get(Some(fire))),
            (200, 75)
        );
        let cursed = Resistances::sum(1, [], [(fire, 200), (fire, -100)]);
        assert_eq!(cursed.get(Some(fire)), MAX_RESISTANCE);
        let cursed = Resistances::sum(1, [], [(fire, 200), (fire, -150)]);
        assert_eq!(cursed.get(Some(fire)), 50);
        // A 100 from any source is a resistance like another: it counts as 75.
        assert_eq!(
            Resistances::sum(1, [], [(fire, IMMUNE)]).get(Some(fire)),
            75
        );
        // Immunity stands apart from every amount.
        let immune = Resistances::sum(2, [fire], [(fire, -200)]);
        assert_eq!(immune.get(Some(fire)), IMMUNE);
        assert_eq!(immune.iter().collect::<Vec<_>>(), [(fire, IMMUNE)]);
        // The vulnerability floor.
        let weak = Resistances::sum(2, [], [(cold, -80), (cold, -80)]);
        assert_eq!(
            (weak.total(Some(cold)), weak.get(Some(cold))),
            (-160, MIN_RESISTANCE)
        );
        assert_eq!(r.iter().collect::<Vec<_>>(), [(fire, 75), (cold, -40)]);
        assert!(Resistances::none(4).is_empty() && !r.is_empty());
        assert!(Resistances::sum(1, [], [(fire, 50), (fire, -50)]).is_empty());
        assert_eq!(
            Resistances::sum(1, [DamageTypeId(9)], [(DamageTypeId(9), 50)]).get(Some(fire)),
            0
        );
        assert_eq!(
            (effective(140), effective(-101), effective(30)),
            (75, -100, 30)
        );
    }

    #[test]
    fn resistance_words() {
        assert_eq!(ResistanceWord::of(0), None);
        assert_eq!(ResistanceWord::of(1), Some(ResistanceWord::Resistant));
        assert_eq!(ResistanceWord::of(75), Some(ResistanceWord::Resistant));
        assert_eq!(ResistanceWord::of(100), Some(ResistanceWord::Immune));
        assert_eq!(ResistanceWord::of(-1), Some(ResistanceWord::Vulnerable));
        assert_eq!(ResistanceWord::Immune.label(), "Immune to");
    }

    #[test]
    fn plain_protection_stands_against_everything() {
        let pv = Protection::plain(4);
        assert_eq!(pv.against(None), 4);
        assert_eq!(
            pv.against(Some(DamageTypeId(0))),
            4,
            "unknown kinds: the base"
        );
        assert_eq!(pv.adjustments().count(), 0);
    }
}
