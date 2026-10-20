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
//! - [`Resistances`] are a target's percent per kind: negative is vulnerability, [`IMMUNE`]
//!   (100) takes nothing. Gear and innate resistance add up, capped at [`MAX_RESISTANCE`] (75)
//!   above and [`MIN_RESISTANCE`] (−100) below; only a creature's own 100 makes it immune.
//!   [`resist`] takes the percentage off an amount, rounded to the nearest (half up), so a
//!   1-point hit at 50 % still hurts and at 75 % does not.
//! - [`Protection`] is a target's Pv against each kind: its base for the physical kinds, each
//!   adjusted by what the pieces worn say (chain mail +1 against slashing, −1 against piercing),
//!   nothing against an element, and the base against untyped damage.

use crate::content::{Content, DamageTypeId};
use crate::dice::Dice;
use serde::{Deserialize, Serialize};

/// The resistance that takes all of a kind: nothing hurts.
pub const IMMUNE: i32 = 100;
/// The most resistance gear and a creature's own add up to, short of immunity.
pub const MAX_RESISTANCE: i32 = 75;
/// The lowest resistance: a vulnerability that doubles the damage.
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

/// A target's resistance to each kind, in percent (0 when it has none): one value per damage
/// type in content order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resistances(Vec<i32>);

impl Resistances {
    /// No resistance to anything (`count` kinds).
    pub fn none(count: usize) -> Self {
        Self(vec![0; count])
    }

    /// Adds up `innate` (what a creature is) and `gear` (what it wears, affixes included), each
    /// a list of (kind, percent): the sum, capped at [`MAX_RESISTANCE`] and [`MIN_RESISTANCE`],
    /// except that an innate [`IMMUNE`] stays immune whatever is worn.
    pub fn sum(
        count: usize,
        innate: impl IntoIterator<Item = (DamageTypeId, i32)>,
        gear: impl IntoIterator<Item = (DamageTypeId, i32)>,
    ) -> Self {
        let mut own = vec![0i32; count];
        for (kind, percent) in innate {
            if let Some(slot) = own.get_mut(usize::from(kind.0)) {
                *slot = slot.saturating_add(percent);
            }
        }
        let mut total = own.clone();
        for (kind, percent) in gear {
            if let Some(slot) = total.get_mut(usize::from(kind.0)) {
                *slot = slot.saturating_add(percent);
            }
        }
        for (total, own) in total.iter_mut().zip(&own) {
            *total = if *own >= IMMUNE {
                IMMUNE
            } else {
                (*total).clamp(MIN_RESISTANCE, MAX_RESISTANCE)
            };
        }
        Self(total)
    }

    /// The resistance to `kind`, in percent: 0 for untyped damage and unknown kinds.
    pub fn get(&self, kind: Option<DamageTypeId>) -> i32 {
        kind.and_then(|k| self.0.get(usize::from(k.0)))
            .copied()
            .unwrap_or(0)
    }

    /// Every kind resisted or suffered, with its percent, in content order (0 left out).
    pub fn iter(&self) -> impl Iterator<Item = (DamageTypeId, i32)> + '_ {
        (self.0.iter().enumerate())
            .filter(|(_, p)| **p != 0)
            .map(|(i, p)| (DamageTypeId(i as u8), *p))
    }

    /// Whether there is nothing to tell.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|p| *p == 0)
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
    /// 100: nothing of that kind hurts it.
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
    fn resistances_add_up_and_cap_but_innate_immunity_stays() {
        let (fire, cold) = (DamageTypeId(0), DamageTypeId(1));
        let r = Resistances::sum(3, [(fire, 20)], [(fire, 30), (cold, -40), (fire, 40)]);
        assert_eq!(r.get(Some(fire)), MAX_RESISTANCE, "capped");
        assert_eq!(r.get(Some(cold)), -40);
        assert_eq!(r.get(Some(DamageTypeId(2))), 0);
        assert_eq!(r.get(None), 0);
        let immune = Resistances::sum(2, [(fire, IMMUNE)], [(fire, -200)]);
        assert_eq!(immune.get(Some(fire)), IMMUNE);
        let worn = Resistances::sum(2, [(fire, 10)], [(fire, 200)]);
        assert_eq!(
            worn.get(Some(fire)),
            MAX_RESISTANCE,
            "gear never makes immune"
        );
        let weak = Resistances::sum(2, [(cold, -80)], [(cold, -80)]);
        assert_eq!(weak.get(Some(cold)), MIN_RESISTANCE);
        assert_eq!(r.iter().collect::<Vec<_>>(), [(fire, 75), (cold, -40)]);
        assert!(Resistances::none(4).is_empty() && !r.is_empty());
        assert_eq!(
            Resistances::sum(1, [], [(DamageTypeId(9), 50)]).get(Some(fire)),
            0
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
