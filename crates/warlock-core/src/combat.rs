//! Combat (spec §8 "Combat"): hitting, Defence, criticals, damage, hit points, and
//! regeneration. These are the pure rules; `world/fight.rs` applies them to actors.
//!
//! - **Hitting**: an attack hits when 2d12 (its *natural* roll, 2 to 24) plus its to-hit is at
//!   least the target's Defence ([`hits`]). A natural 2 always misses, a natural 24 always hits.
//! - **Defence** ("Dv") is [`BASE_DEFENCE`] (10) plus a bonus in each [`Category`]: within a
//!   category only the highest bonus counts, except Dodge, whose bonuses add up ([`Defence`]).
//! - **Why an attack missed**: a roll plus to-hit below 10 would have missed an unarmoured,
//!   motionless target, so it is a plain miss; otherwise one category with a positive bonus made
//!   the difference, picked by one draw weighted by its bonus ([`Defence::why_missed`]). A
//!   negative bonus (clumsy Dexterity) lowers Defence but is never the reason.
//! - **Criticals** (D&D 3.5): a hit whose natural roll is at least the attack's threat
//!   ([`DEFAULT_THREAT`] 22, 3.5's natural 20) threatens; a second roll against the same Defence
//!   confirms it, and then the damage is rolled `crit` times ([`DEFAULT_CRIT`] 2).
//! - **Damage**: the attack's dice plus its bonus, at least 1 ([`Profile`]).
//! - **Protection** ("Pv") takes damage off every physical hit that lands: one draw from ⌈Pv/2⌉
//!   to Pv ([`soak`]), after the critical multiplied the damage, less the attack's penetration;
//!   the Pv is the target's against the kind of damage ([`Protection`]: a piece may stand
//!   better or worse against one physical kind), and an element is never soaked. A hit may be
//!   soaked to nothing ([`Strike::Soaked`]).
//! - **Damage types** (spec §8 "Damage types"): an attack does one of its [`Kinds`], whichever
//!   is better against the target (decided before the dice, no draw), and may carry
//!   [`Rider`]s of other kinds (a flaming arrow's fire), rolled after its own dice. After the
//!   soak, the target's [`Resistances`] take their percentage off each kind
//!   ([`damage::resist`]); a hit resisted to nothing is [`Strike::Resisted`].
//! - **Hit points**: Toughness plus what race and class (a creature: its type) give
//!   ([`max_hp`]); one comes back every `max(60 s, 360 s − 12 s × Toughness)` of game time
//!   ([`regen_seconds`]) at the normal pace (resting and status effects change the pace,
//!   `world/status.rs`). A **big hit** takes at least a third of them ([`big_hit`]).
//!
//! Every roll comes from the gameplay random stream, so fights replay exactly. Nothing here is
//! shown to the player as a number but their own hit points, Defence, and Protection: never a
//! to-hit, a modifier, a roll, or a penetration.

use crate::content::{Benefit, DamageTypeId};
use crate::damage::{self, Kinds, Protection, Resistances, Rider};
use crate::dice::Dice;
use crate::rng::Rng;
use crate::stats::{Stat, Stats};
use serde::{Deserialize, Serialize};

/// Defence with no bonus at all: an unarmoured, motionless target. Also the plain-miss line: a
/// roll plus to-hit below it is a plain miss.
pub const BASE_DEFENCE: i32 = 10;
/// Sides of each of the two dice of an attack roll.
pub const DIE: i32 = 12;
/// The natural roll that always misses (1 + 1)...
pub const FUMBLE: i32 = 2;
/// ...and the one that always hits (12 + 12).
pub const SURE_HIT: i32 = 2 * DIE;
/// The lowest natural roll that threatens a critical unless the attack says otherwise: 22 to
/// 24, 3.5's natural 20 by probability.
pub const DEFAULT_THREAT: u8 = 22;
/// How many times a confirmed critical rolls its damage unless the attack says otherwise.
pub const DEFAULT_CRIT: u8 = 2;
/// The threats an attack may have: from any hit (2) to a natural 24 only.
pub const THREATS: std::ops::RangeInclusive<u8> = 2..=24;
/// The damage multipliers an attack may have.
pub const CRITS: std::ops::RangeInclusive<u8> = 2..=5;
/// The most hit points beyond Toughness that data may give: a race, a class, a creature type
/// (`base_hp`), a scenario without a character.
pub const MAX_BASE_HP: u16 = 1000;

/// Bare hands' dice: 1d3 (plus the Strength modifier, as every melee attack).
pub fn unarmed_dice() -> Dice {
    // The text is valid dice; the fallback is never used.
    "1d3".parse().unwrap_or(Dice::fixed(2))
}

/// The natural roll of an attack: 2d12, one draw per die.
pub fn roll_natural(rng: &mut Rng) -> i32 {
    rng.range(1, DIE) + rng.range(1, DIE)
}

/// How many of the 144 ways two twelve-sided dice can fall give the natural roll `natural`.
pub const fn ways(natural: i32) -> u32 {
    if natural < 2 || natural > SURE_HIT {
        0
    } else if natural <= DIE + 1 {
        (natural - 1) as u32
    } else {
        (2 * DIE + 1 - natural) as u32
    }
}

/// Whether an attack roll hits: never on a natural 2, always on a natural 24, else when the
/// natural roll plus the to-hit is at least the Defence.
pub const fn hits(natural: i32, to_hit: i32, defence: i32) -> bool {
    match natural {
        FUMBLE => false,
        SURE_HIT => true,
        _ => natural + to_hit >= defence,
    }
}

/// How many of the 144 natural rolls hit `defence` with `to_hit`.
pub fn hit_ways(to_hit: i32, defence: i32) -> u32 {
    (FUMBLE..=SURE_HIT)
        .filter(|&n| hits(n, to_hit, defence))
        .map(ways)
        .sum()
}

/// Defines [`Category`] from one entry per category: its id, its name, and whether its bonuses
/// stack, in order (the order a miss is attributed in).
macro_rules! categories {
    ($($(#[$doc:meta])* $variant:ident = $id:literal, $name:literal, $stacks:literal;)+) => {
        /// A category of Defence, after D&D's third edition (spec §8 "Combat").
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Category {
            $($(#[$doc])* $variant,)+
        }

        impl Category {
            /// Every category, in order.
            pub const ALL: [Category; [$($id),+].len()] = [$(Category::$variant),+];

            /// Its id: `armour`.
            pub const fn id(self) -> &'static str {
                match self {
                    $(Category::$variant => $id,)+
                }
            }

            /// Its name: `Armour`.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Category::$variant => $name,)+
                }
            }

            /// Whether its bonuses add up; every other category counts only its highest.
            pub const fn stacks(self) -> bool {
                match self {
                    $(Category::$variant => $stacks,)+
                }
            }
        }
    };
}

categories! {
    /// The Dexterity modifier; the worn armour with the lowest maximum Dexterity bonus caps it.
    Dexterity = "dexterity", "Dexterity", false;
    /// Worn body armour.
    Armour = "armour", "Armour", false;
    /// A held shield.
    Shield = "shield", "Shield", false;
    /// The wielded weapon's parry bonus.
    Parry = "parry", "Parry", false;
    /// A creature's hide, shell, or scales.
    Natural = "natural", "Natural", false;
    /// Training and effects (none yet). The one category whose bonuses stack.
    Dodge = "dodge", "Dodge", true;
    /// Magic (none yet).
    Deflection = "deflection", "Deflection", false;
}

impl Category {
    fn index(self) -> usize {
        self as usize
    }
}

/// A Defence: [`BASE_DEFENCE`] plus a bonus in each category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Defence {
    bonus: [i32; Category::ALL.len()],
}

impl Defence {
    /// The Defence these bonuses give: in each category the highest of its bonuses (a single
    /// negative one counts), the sum of them for Dodge; no bonus is 0.
    pub fn from_bonuses(bonuses: impl IntoIterator<Item = (Category, i32)>) -> Self {
        let mut best: [Option<i32>; Category::ALL.len()] = [None; Category::ALL.len()];
        for (category, bonus) in bonuses {
            let slot = &mut best[category.index()];
            *slot = Some(match *slot {
                None => bonus,
                Some(b) if category.stacks() => b + bonus,
                Some(b) => b.max(bonus),
            });
        }
        Self {
            bonus: best.map(|b| b.unwrap_or(0)),
        }
    }

    /// The bonus in `category`.
    /// This Defence with `bonus` more in `category` (a bonus against one kind of attack).
    pub fn plus(mut self, category: Category, bonus: i32) -> Self {
        self.bonus[category.index()] += bonus;
        self
    }

    pub fn bonus(&self, category: Category) -> i32 {
        self.bonus[category.index()]
    }

    /// 10 plus every category's bonus: what an attack roll must reach.
    pub fn total(&self) -> i32 {
        BASE_DEFENCE + self.bonus.iter().sum::<i32>()
    }

    /// What a miss is attributed by: the positive bonuses, in category order.
    fn weights(&self) -> impl Iterator<Item = (Category, u32)> + '_ {
        (Category::ALL.into_iter())
            .map(|c| (c, u32::try_from(self.bonus(c)).unwrap_or(0)))
            .filter(|&(_, w)| w > 0)
    }

    /// The sum of the positive bonuses: the range of the draw that picks why an attack missed.
    pub fn reason_weight(&self) -> u32 {
        self.weights().map(|(_, w)| w).sum()
    }

    /// Why an attack missed whose natural roll plus to-hit came to `total`: `None` for a plain
    /// miss (below [`BASE_DEFENCE`]: it would have missed an unarmoured, motionless target),
    /// else the category `pick` (a draw in `0..`[`Defence::reason_weight`]) falls in, each
    /// category taking as many values as its bonus.
    pub fn why_missed(&self, total: i32, pick: u32) -> Option<Category> {
        if total < BASE_DEFENCE {
            return None;
        }
        let mut left = pick;
        for (category, weight) in self.weights() {
            if left < weight {
                return Some(category);
            }
            left -= weight;
        }
        None
    }
}

/// What an attack does: its to-hit, its damage (dice plus a bonus to every roll of them), and
/// its critical (threat and multiplier). Built from a weapon, a creature's natural attack, or
/// bare hands, with the attacker's stats ([`Profile::melee`], [`Profile::ranged`],
/// [`Profile::thrown`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub to_hit: i32,
    pub damage: Dice,
    /// Added to every roll of the damage dice.
    pub bonus: i32,
    /// The lowest natural roll that threatens a critical.
    pub threat: u8,
    /// How many times a confirmed critical rolls the damage.
    pub crit: u8,
    /// Added to the roll that confirms a critical, on top of the to-hit (spec §8 "Class
    /// abilities": Peerless).
    pub confirm: i32,
    /// Dice rolled with every roll of the damage, after its own (a magic weapon's "adds 1d4
    /// damage"; spec §8 "Magic items").
    pub extra: Option<Dice>,
    /// How much of the target's Protection it ignores (a weapon's or a missile's
    /// `penetration`: a war hammer, a crossbow bolt).
    pub penetration: i32,
    /// The kinds of damage its own dice do (spec §8 "Damage types"): a blow does whichever is
    /// better against its target; none for an untyped attack.
    pub kinds: Kinds,
    /// Damage of other kinds added to every roll (a flaming arrow's fire, a magic weapon's
    /// "adds 1d4 fire damage"), rolled after the extra die, in this order; never soaked.
    pub riders: Vec<Rider>,
}

/// What an attack is made with, as data gives it: a weapon (`data/objects.toml`) or a creature's
/// natural attack (`data/creatures.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attack {
    /// Its damage dice, which may carry a fixed bonus (`1d8+2`: a bow's draw weight).
    pub damage: Dice,
    pub to_hit: i32,
    pub threat: u8,
    pub crit: u8,
    /// The Strength a bow needs (`None`: any).
    pub strength: Option<u8>,
    /// The kinds of damage it does (spec §8 "Damage types"): one, or two of which a blow does
    /// the better; none without damage types in content.
    pub kinds: Kinds,
}

impl Attack {
    /// Bare hands: 1d3, default critical, of the kind `unarmed` (the content's unarmed damage
    /// type, bludgeoning; `None`: untyped).
    pub fn unarmed(unarmed: Option<DamageTypeId>) -> Self {
        Self {
            damage: unarmed_dice(),
            to_hit: 0,
            threat: DEFAULT_THREAT,
            crit: DEFAULT_CRIT,
            strength: None,
            kinds: unarmed.map_or(Kinds::NONE, Kinds::one),
        }
    }
}

/// To hit with a weapon thrown that was not made for it (a sword, a club), as in D&D: -4.
pub const IMPROVISED_TO_HIT: i32 = -4;

/// How far such a weapon is thrown, in cells.
pub const IMPROVISED_RANGE: i32 = 3;

impl Profile {
    /// A melee attack with `attack` (a weapon, a natural attack, bare hands): to-hit and damage
    /// both add the Strength modifier.
    pub fn melee(attack: &Attack, stats: &Stats) -> Self {
        let strength = stats.modifier(Stat::Strength);
        Self::of(attack, attack.to_hit + strength, strength)
    }

    /// A thrown weapon (ranged combat, #117): to-hit adds the Dexterity modifier, damage the
    /// Strength modifier.
    pub fn thrown(attack: &Attack, stats: &Stats) -> Self {
        let (dexterity, strength) = (
            stats.modifier(Stat::Dexterity),
            stats.modifier(Stat::Strength),
        );
        Self::of(attack, attack.to_hit + dexterity, strength)
    }

    /// A shot from launcher `attack`: to-hit adds the Dexterity modifier; damage is the
    /// launcher's own (its dice and their fixed bonus, its draw weight), with no modifier. Each
    /// point of Strength below what a bow needs costs 1 to hit and 1 damage.
    pub fn ranged(attack: &Attack, stats: &Stats) -> Self {
        let short = attack.strength.map_or(0, |needed| {
            (i32::from(needed) - i32::from(stats.score(Stat::Strength))).max(0)
        });
        let dexterity = stats.modifier(Stat::Dexterity);
        Self::of(attack, attack.to_hit + dexterity - short, -short)
    }

    /// This attack with what a weapon skill's rank gives (spec §8 "Weapon skills"; a fighting
    /// style's adds to it): more to hit, more to every roll of the damage, a wider threat range
    /// (never past a natural 2), and a bigger critical multiplier (never past ×5).
    pub fn with(self, benefit: &Benefit) -> Self {
        Self {
            to_hit: self.to_hit + benefit.to_hit,
            bonus: self.bonus + benefit.damage,
            threat: self
                .threat
                .saturating_sub(benefit.threat)
                .max(*THREATS.start()),
            crit: self.crit.saturating_add(benefit.crit).min(*CRITS.end()),
            ..self
        }
    }

    /// This throw, of a weapon thrown whole though not made for throwing: [`IMPROVISED_TO_HIT`]
    /// to hit.
    pub fn improvised(self) -> Self {
        Self {
            to_hit: self.to_hit + IMPROVISED_TO_HIT,
            ..self
        }
    }

    /// An attack with no one's stats behind it (a trap's, spec §8 "Traps"): its own to-hit and
    /// damage, nothing added.
    pub fn bare(attack: &Attack) -> Self {
        Self::of(attack, attack.to_hit, 0)
    }

    fn of(attack: &Attack, to_hit: i32, bonus: i32) -> Self {
        Self {
            to_hit,
            damage: attack.damage,
            bonus,
            threat: attack.threat,
            crit: attack.crit,
            confirm: 0,
            extra: None,
            penetration: 0,
            kinds: attack.kinds,
            riders: Vec::new(),
        }
    }

    /// Adds `rider` to every roll of the damage (after those added before it).
    pub fn add_rider(&mut self, rider: Rider) {
        self.riders.push(rider);
    }

    /// One roll of the damage: the dice, the extra dice if any, plus the bonus (it may be below
    /// 1; a blow's total is at least 1); then one roll of each rider, in order, added to
    /// `riders` by kind.
    fn roll_damage(&self, rng: &mut Rng, riders: &mut Vec<(DamageTypeId, i32)>) -> i32 {
        let dice = self.damage.roll(rng);
        let extra = self.extra.map_or(0, |d| d.roll(rng));
        for rider in &self.riders {
            let rolled = rider.dice.roll(rng).max(0);
            match riders.iter_mut().find(|(k, _)| *k == rider.kind) {
                Some((_, sum)) => *sum += rolled,
                None => riders.push((rider.kind, rolled)),
            }
        }
        dice + extra + self.bonus
    }
}

/// The side of an attack that is struck: its Defence, its Protection against each kind, and
/// its resistances.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub defence: Defence,
    pub protection: Protection,
    pub resistances: Resistances,
}

impl Target {
    /// A target of `defence` with no Protection and no resistance (tests, intents).
    pub fn bare(defence: Defence) -> Self {
        Self::protected(defence, 0)
    }

    /// A target of `defence` with plain Protection `pv` against everything.
    pub fn protected(defence: Defence, pv: i32) -> Self {
        Self {
            defence,
            protection: Protection::plain(pv),
            resistances: Resistances::default(),
        }
    }
}

/// What came of an attack, as the event and the message say it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strike {
    /// It missed: plainly (`by` is `None`), or because of the Defence category `by`.
    Missed {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        by: Option<Category>,
    },
    /// It hit; `critical`: a confirmed critical.
    Hit { critical: bool },
    /// It hit, but the target's Protection took all of its damage: it glanced off.
    Soaked,
    /// It hit, but the target resisted all of what was left: it did no harm (spec §8 "Damage
    /// types").
    Resisted,
}

impl Strike {
    /// Whether it landed: a hit, whether or not it was soaked or resisted to nothing.
    pub fn is_hit(self) -> bool {
        matches!(self, Strike::Hit { .. } | Strike::Soaked | Strike::Resisted)
    }
}

/// An attack resolved: what came of it, the damage it does (0 for a miss or a hit soaked or
/// resisted to nothing, else at least 1), and the kind its own dice did (what it is said with;
/// `None` for an untyped attack or a miss).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blow {
    pub strike: Strike,
    pub damage: i32,
    pub kind: Option<DamageTypeId>,
}

impl Blow {
    /// A miss.
    pub const fn missed(by: Option<Category>) -> Self {
        Self {
            strike: Strike::Missed { by },
            damage: 0,
            kind: None,
        }
    }

    /// A plain hit of `damage` (a blast, a bolt, a fall: nothing rolled to hit) of `kind`.
    pub const fn plain(damage: i32, kind: Option<DamageTypeId>) -> Self {
        Self {
            strike: Strike::Hit { critical: false },
            damage,
            kind,
        }
    }
}

/// Resolves an attack with `profile` against `target`, drawing from `rng` in this order: the
/// two dice of the attack roll; on an attributable miss one draw for why; on a threat the two
/// dice of the confirmation roll; then, for each time the damage is rolled, one draw per damage
/// die, per extra die, and per rider die (in rider order); then, on a hit against Protection
/// (against the kind done) above the attack's penetration, one draw for the soak ([`soak`]).
/// The kind the attack does is picked before the dice ([`Kinds::best`]), and the resistances
/// come last ([`damage::resist`]), each kind on its own: no draws.
pub fn strike(profile: &Profile, target: &Target, rng: &mut Rng) -> Blow {
    let defence = &target.defence;
    let dv = defence.total();
    let natural = roll_natural(rng);
    if !hits(natural, profile.to_hit, dv) {
        let total = natural + profile.to_hit;
        let weight = defence.reason_weight();
        // A natural 2 is a fumble: a plain miss, whatever the to-hit.
        let by = if natural == FUMBLE || total < BASE_DEFENCE || weight == 0 {
            None
        } else {
            defence.why_missed(total, rng.below(weight))
        };
        return Blow::missed(by);
    }
    let kind = profile.kinds.best(
        |k| target.resistances.get(Some(k)),
        |k| target.protection.against(Some(k)),
    );
    let threat = natural >= i32::from(profile.threat);
    let critical = threat && hits(roll_natural(rng), profile.to_hit + profile.confirm, dv);
    let times = if critical { profile.crit.max(1) } else { 1 };
    let mut riders = Vec::new();
    let damage: i32 = (0..times)
        .map(|_| profile.roll_damage(rng, &mut riders))
        .sum();
    // The critical multiplied the damage first; the soak comes off once, so criticals punch
    // through armour. An element meets no armour: the Protection against it is 0.
    let pv = target.protection.against(kind);
    let main = damage.max(1) - soak(pv - profile.penetration, rng);
    let added: i32 = riders.iter().map(|(_, d)| d).sum();
    if main <= 0 && added <= 0 {
        return Blow {
            strike: Strike::Soaked,
            damage: 0,
            kind,
        };
    }
    // Then what the target resists of each kind, on its own.
    let resisted = |kind: Option<DamageTypeId>, amount: i32| {
        damage::resist(amount, target.resistances.get(kind))
    };
    let total = resisted(kind, main)
        + (riders.iter())
            .map(|&(k, d)| resisted(Some(k), d))
            .sum::<i32>();
    if total <= 0 {
        return Blow {
            strike: Strike::Resisted,
            damage: 0,
            kind,
        };
    }
    Blow {
        strike: Strike::Hit { critical },
        damage: total,
        kind,
    }
}

/// The damage `protection` takes off a hit (spec §8 "Combat": Protection, "Pv"): one draw in
/// [`soaks`], uniform; nothing, and no draw, when it is 0 or less (an unprotected target, a
/// penetration that ignores all of it, or an element, which no armour stops).
pub fn soak(protection: i32, rng: &mut Rng) -> i32 {
    if protection <= 0 {
        return 0;
    }
    let range = soaks(protection);
    let span = u32::try_from(range.end() - range.start() + 1).unwrap_or(1);
    range.start() + i32::try_from(rng.below(span)).unwrap_or(0)
}

/// What `protection` may take off a hit: ⌈Pv/2⌉ to Pv (0 to 0 when it is 0 or less).
pub fn soaks(protection: i32) -> std::ops::RangeInclusive<i32> {
    let pv = protection.max(0);
    (pv + 1) / 2..=pv
}

/// Whether a blow of `damage` against a target of `max` hit points is a **big hit**: at least a
/// third of them (spec §8 "Status effects": it may make the target bleed).
pub fn big_hit(damage: i32, max: i32) -> bool {
    damage > 0 && 3 * i64::from(damage) >= i64::from(max.max(1))
}

/// Maximum hit points: Toughness plus `base` (race and class for the player, the creature
/// type's `base_hp` for a creature), at least 1. Linear in Toughness: no runaway pools.
pub fn max_hp(toughness: u8, base: i32) -> i32 {
    (i32::from(toughness) + base).max(1)
}

/// The longest a hit point takes to come back, and the shortest...
pub const REGEN_SLOWEST: u32 = 360;
pub const REGEN_FASTEST: u32 = 60;
/// ...and how much each point of Toughness takes off.
pub const REGEN_PER_TOUGHNESS: u32 = 12;

/// Seconds of game time per regenerated hit point at the normal pace: `max(60, 360 − 12 ×
/// Toughness)`.
pub fn regen_seconds(toughness: u8) -> u32 {
    REGEN_SLOWEST
        .saturating_sub(REGEN_PER_TOUGHNESS * u32::from(toughness))
        .max(REGEN_FASTEST)
}

/// How hurt a creature looks, in words (look mode): never its hit points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Unhurt,
    LightlyWounded,
    Wounded,
    BadlyWounded,
    NearlyDead,
}

impl Health {
    /// How `hp` of `max` looks: unhurt at full health, lightly wounded above two thirds, wounded
    /// above a third, badly wounded above a sixth, else nearly dead.
    pub fn of(hp: i32, max: i32) -> Self {
        let (hp, max) = (i64::from(hp), i64::from(max.max(1)));
        if hp >= max {
            Health::Unhurt
        } else if 3 * hp > 2 * max {
            Health::LightlyWounded
        } else if 3 * hp > max {
            Health::Wounded
        } else if 6 * hp > max {
            Health::BadlyWounded
        } else {
            Health::NearlyDead
        }
    }

    /// `badly wounded`.
    pub const fn word(self) -> &'static str {
        match self {
            Health::Unhurt => "unhurt",
            Health::LightlyWounded => "lightly wounded",
            Health::Wounded => "wounded",
            Health::BadlyWounded => "badly wounded",
            Health::NearlyDead => "nearly dead",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::IMMUNE;
    use std::collections::BTreeMap;

    fn dice(text: &str) -> Dice {
        text.parse().expect("dice")
    }

    fn profile(to_hit: i32, damage: &str, threat: u8, crit: u8) -> Profile {
        Profile {
            to_hit,
            damage: dice(damage),
            bonus: 0,
            threat,
            crit,
            confirm: 0,
            extra: None,
            penetration: 0,
            kinds: Kinds::NONE,
            riders: Vec::new(),
        }
    }

    #[test]
    fn extra_dice_roll_after_the_weapons_own() {
        let dv = Defence::from_bonuses([(Category::Dexterity, -10)]);
        let p = Profile {
            extra: Some(Dice::of(1, 4)),
            ..profile(0, "1d6", 24, 2)
        };
        let mut rng = Rng::stream(5, "combat-test");
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..1000 {
            let blow = strike(&p, &Target::bare(dv), &mut rng);
            if blow.strike == (Strike::Hit { critical: false }) {
                assert!((2..=10).contains(&blow.damage), "{}", blow.damage);
                seen.insert(blow.damage);
            }
        }
        assert_eq!(seen.len(), 9, "1d6 + 1d4 falls 2 to 10");
    }

    #[test]
    fn two_twelve_sided_dice_fall_144_ways() {
        assert_eq!((2..=24).map(ways).sum::<u32>(), 144);
        assert_eq!(
            (ways(2), ways(13), ways(24), ways(1), ways(25)),
            (1, 12, 1, 0, 0)
        );
        // The roll draws two dice, each 1 to 12: every sum comes up about as often as it should.
        let mut rng = Rng::stream(1, "combat-test");
        let mut seen = BTreeMap::new();
        for _ in 0..144_000 {
            *seen.entry(roll_natural(&mut rng)).or_insert(0u32) += 1;
        }
        for (n, count) in seen {
            let expected = ways(n) * 1000;
            assert!(
                count.abs_diff(expected) < expected / 5 + 150,
                "{n}: {count}"
            );
        }
    }

    /// The table in #113: the hit chance by what the roll needs (Defence − to-hit).
    #[test]
    fn hit_chances_follow_the_2d12_bell() {
        let percent = |needed: i32| f64::from(hit_ways(0, needed)) * 100.0 / 144.0;
        for (needed, expected) in [
            (6, 93.0),
            (8, 85.0),
            (10, 75.0),
            (11, 69.0),
            (12, 62.0),
            (13, 54.0),
            (14, 46.0),
            (15, 38.0),
            (16, 31.0),
            (18, 19.0),
            (20, 10.0),
            (22, 4.0),
            (24, 0.7),
        ] {
            let p = percent(needed);
            assert!((p - expected).abs() <= 0.5, "{needed}: {p}");
        }
        // +0 against the default Defence of 10 hits 75 % of the time; to-hit and Defence trade
        // one for one.
        assert_eq!(hit_ways(0, BASE_DEFENCE), 108);
        assert_eq!(hit_ways(3, 15), hit_ways(0, 12));
    }

    #[test]
    fn a_natural_2_always_misses_and_a_natural_24_always_hits() {
        assert!(!hits(2, 100, 10));
        assert!(hits(24, -100, 60));
        assert!(hits(3, 7, 10) && !hits(3, 6, 10));
        // However good or bad the odds, one roll in 144 hits and one misses.
        assert_eq!(hit_ways(1000, 10), 143);
        assert_eq!(hit_ways(-1000, 10), 1);
    }

    #[test]
    fn defence_takes_the_highest_bonus_per_category_but_dodge_stacks() {
        use Category::*;
        let dv = Defence::from_bonuses([
            (Armour, 2),
            (Armour, 5),
            (Shield, 1),
            (Dodge, 1),
            (Dodge, 2),
            (Dexterity, 3),
            (Natural, 1),
            (Natural, 0),
        ]);
        assert_eq!(
            Category::ALL.map(|c| dv.bonus(c)),
            [3, 5, 1, 0, 1, 3, 0],
            "Dexterity, Armour, Shield, Parry, Natural, Dodge, Deflection"
        );
        assert_eq!(dv.total(), 10 + 3 + 5 + 1 + 1 + 3);
        assert_eq!(Defence::default().total(), BASE_DEFENCE);
        // A Dexterity penalty lowers Defence...
        let clumsy = Defence::from_bonuses([(Dexterity, -2), (Armour, 3)]);
        assert_eq!(clumsy.total(), 11);
        assert_eq!(clumsy.reason_weight(), 3);
        // ...but is never why an attack missed.
        for pick in 0..3 {
            assert_eq!(clumsy.why_missed(10, pick), Some(Armour));
        }
        assert!(Dodge.stacks() && Category::ALL.iter().filter(|c| c.stacks()).count() == 1);
    }

    /// The example in #113: against Defence 16 (Dexterity 2, Armour 3, Shield 1) an attacker at
    /// +0 misses 69 % of the time: 36 % of the misses are plain, 32 % the armour, 21 % a dodge,
    /// and 11 % the shield. Exhaustive over every natural roll and every draw.
    #[test]
    fn misses_are_attributed_by_weight() {
        use Category::*;
        let dv = Defence::from_bonuses([(Dexterity, 2), (Armour, 3), (Shield, 1)]);
        assert_eq!((dv.total(), dv.reason_weight()), (16, 6));
        let mut by: BTreeMap<Option<Category>, u32> = BTreeMap::new();
        let mut misses = 0;
        for natural in 2..=24 {
            if hits(natural, 0, dv.total()) {
                continue;
            }
            for pick in 0..dv.reason_weight() {
                misses += ways(natural);
                *by.entry(dv.why_missed(natural, pick)).or_insert(0) += ways(natural);
            }
        }
        // Six draws per miss.
        assert_eq!(misses, 99 * 6);
        let share = |c: Option<Category>| f64::from(by[&c]) / f64::from(misses) * 100.0;
        assert!((share(None) - 36.4).abs() < 0.1);
        assert!((share(Some(Armour)) - 31.8).abs() < 0.1);
        assert!((share(Some(Dexterity)) - 21.2).abs() < 0.1);
        assert!((share(Some(Shield)) - 10.6).abs() < 0.1);
        // The picks fall in category order, each taking as many values as its bonus.
        let picks: Vec<_> = (0..6).map(|p| dv.why_missed(12, p)).collect();
        assert_eq!(
            picks,
            [
                Some(Dexterity),
                Some(Dexterity),
                Some(Armour),
                Some(Armour),
                Some(Armour),
                Some(Shield)
            ]
        );
        assert_eq!(dv.why_missed(9, 0), None, "below 10: a plain miss");
    }

    /// #113: against Defence 10 at +0 a default weapon confirms a critical on 3.1 % of attacks,
    /// a 19–24 weapon on 10.9 %. Exhaustive over the attack roll and the confirmation roll.
    #[test]
    fn criticals_threaten_and_confirm() {
        let confirmed = |threat: i32| -> f64 {
            let threats: u32 = (threat..=24).filter(|&n| hits(n, 0, 10)).map(ways).sum();
            f64::from(threats * hit_ways(0, 10)) / (144.0 * 144.0) * 100.0
        };
        assert!((confirmed(22) - 3.125).abs() < 0.01);
        assert!((confirmed(19) - 10.94).abs() < 0.01);
        assert!((confirmed(20) - 7.81).abs() < 0.01);
        // A roll in the threat range that misses (a high Defence) threatens nothing; against
        // Defence 40 only a natural 24 hits, and it confirms only on another 24.
        let mut rng = Rng::stream(2, "combat-test");
        let hard = Defence::from_bonuses([(Category::Armour, 30)]);
        let (mut hit, mut missed) = (0, 0);
        for _ in 0..2000 {
            match strike(&profile(0, "1d4", 2, 2), &Target::bare(hard), &mut rng).strike {
                Strike::Hit { .. } | Strike::Soaked | Strike::Resisted => hit += 1,
                Strike::Missed { .. } => missed += 1,
            }
        }
        assert!(hit < 40 && missed > 1960, "{hit} hits");
    }

    #[test]
    fn a_confirmed_critical_rolls_the_damage_crit_times() {
        // Against Defence 0 everything but a natural 2 hits and confirms; a threat of 2 makes
        // every hit threaten. 1d1 is not dice, so a fixed bonus shows the multiplier.
        let dv = Defence::from_bonuses([(Category::Dexterity, -10)]);
        let fixed = |crit| Profile {
            bonus: 3,
            ..profile(0, "+0", 2, crit)
        };
        let mut rng = Rng::stream(3, "combat-test");
        let mut seen = BTreeMap::new();
        for _ in 0..2000 {
            let blow = strike(&fixed(3), &Target::bare(dv), &mut rng);
            *seen.entry((blow.strike, blow.damage)).or_insert(0) += 1;
        }
        let keys: Vec<_> = seen.keys().copied().collect();
        assert_eq!(
            keys,
            [
                (Strike::Missed { by: None }, 0),
                (Strike::Hit { critical: false }, 3),
                (Strike::Hit { critical: true }, 9),
            ],
            "a normal hit once, a critical three times; misses (fumbles) do nothing"
        );
        // Each roll is the dice plus the bonus: 2 × (1d4 + 1) lies in 4 ..= 10.
        let p = Profile {
            bonus: 1,
            ..profile(0, "1d4", 2, 2)
        };
        for _ in 0..500 {
            let blow = strike(&p, &Target::bare(dv), &mut rng);
            match blow.strike {
                Strike::Hit { critical: true } => assert!((4..=10).contains(&blow.damage)),
                Strike::Hit { critical: false } => assert!((2..=5).contains(&blow.damage)),
                Strike::Missed { .. } | Strike::Soaked | Strike::Resisted => {
                    assert_eq!(blow.damage, 0)
                }
            }
        }
    }

    #[test]
    fn a_hit_always_does_at_least_1() {
        let dv = Defence::from_bonuses([(Category::Dexterity, -10)]);
        let weak = Profile {
            bonus: -5,
            ..profile(0, "1d3", 22, 2)
        };
        let mut rng = Rng::stream(4, "combat-test");
        for _ in 0..500 {
            let blow = strike(&weak, &Target::bare(dv), &mut rng);
            if blow.strike.is_hit() {
                assert_eq!(blow.damage, 1);
            }
        }
    }

    #[test]
    fn strikes_are_deterministic_and_draw_in_order() {
        let dv = Defence::from_bonuses([(Category::Armour, 4), (Category::Dexterity, 2)]);
        let p = profile(1, "2d6", 20, 2);
        let run = |seed| {
            let mut rng = Rng::stream(seed, "combat-test");
            (0..50)
                .map(|_| strike(&p, &Target::bare(dv), &mut rng))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(5), run(5));
        assert_ne!(run(5), run(6));
        let mut rng = Rng::stream(7, "combat-test");
        let hard = Defence::from_bonuses([(Category::Armour, 20)]);
        for _ in 0..200 {
            // Far out of reach, everything but a natural 24 is a plain miss: two draws.
            let mut expected = rng.clone();
            let natural = roll_natural(&mut expected);
            let blow = strike(&profile(-20, "1d4", 22, 2), &Target::bare(dv), &mut rng);
            if natural < SURE_HIT {
                assert_eq!(blow.strike, Strike::Missed { by: None });
                assert_eq!(rng, expected, "two draws");
            }
            // Against a high Defence a roll of 10 or more misses for a reason: one draw more.
            let mut expected = rng.clone();
            let natural = roll_natural(&mut expected);
            let blow = strike(&profile(0, "1d4", 22, 2), &Target::bare(hard), &mut rng);
            if (BASE_DEFENCE..SURE_HIT).contains(&natural) {
                expected.below(20);
                let armour = Some(Category::Armour);
                assert_eq!(blow.strike, Strike::Missed { by: armour });
                assert_eq!(rng, expected, "three draws");
            }
        }
    }

    #[test]
    fn protection_soaks_half_to_all_of_it_evenly() {
        // Exhaustive: every Pv's range, and each value in it as often as the others.
        assert_eq!(soaks(0), 0..=0);
        assert_eq!(soaks(-3), 0..=0);
        assert_eq!(soaks(1), 1..=1);
        assert_eq!(soaks(2), 1..=2);
        assert_eq!(soaks(3), 2..=3);
        assert_eq!(soaks(6), 3..=6);
        assert_eq!(soaks(7), 4..=7);
        for pv in 1..=20 {
            let range = soaks(pv);
            assert_eq!(*range.start(), (pv + 1) / 2, "⌈{pv}/2⌉");
            let mut rng = Rng::stream(pv as u64, "soak-test");
            let mut seen = BTreeMap::new();
            let n = 4000;
            for _ in 0..n {
                let mut one = rng.clone();
                let soaked = soak(pv, &mut rng);
                one.below(1);
                assert_eq!(rng, one, "one draw at Pv {pv}");
                *seen.entry(soaked).or_insert(0u32) += 1;
            }
            let values: Vec<i32> = seen.keys().copied().collect();
            assert_eq!(values, range.clone().collect::<Vec<_>>(), "Pv {pv}");
            let even = n / range.count() as u32;
            for (value, count) in seen {
                assert!(
                    count.abs_diff(even) < even / 4,
                    "Pv {pv}: {value} {count} times"
                );
            }
        }
        // No Protection: no draw at all.
        let mut rng = Rng::stream(1, "soak-test");
        let before = rng.clone();
        assert_eq!((soak(0, &mut rng), soak(-2, &mut rng)), (0, 0));
        assert_eq!(rng, before);
    }

    #[test]
    fn the_soak_comes_after_the_damage_once_and_penetration_ignores_some() {
        // Against Defence 0 everything but a natural 2 hits; threat 2 makes every hit a threat,
        // and a fixed 3 a roll shows what came off.
        let dv = Defence::from_bonuses([(Category::Dexterity, -10)]);
        let fixed = |crit, penetration| Profile {
            bonus: 3,
            penetration,
            ..profile(0, "+0", 2, crit)
        };
        let mut rng = Rng::stream(8, "combat-test");
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..2000 {
            // Pv 4 soaks 2 to 4: a normal hit of 3 does 1 or nothing; a ×3 critical's 9 does
            // 5 to 7, the soak taken once.
            let mut expected = rng.clone();
            let natural = roll_natural(&mut expected);
            let blow = strike(&fixed(3, 0), &Target::protected(dv, 4), &mut rng);
            seen.insert((blow.strike, blow.damage));
            if natural != FUMBLE {
                // The confirmation, the damage (fixed: no draws), then one for the soak.
                roll_natural(&mut expected);
                let soaked = soak(4, &mut expected);
                assert_eq!(rng, expected, "the soak is the last draw");
                let raw = if blow.strike == (Strike::Hit { critical: true }) {
                    9
                } else {
                    3
                };
                assert_eq!(blow.damage, (raw - soaked).max(0));
            }
        }
        let outcomes: Vec<_> = seen.into_iter().collect();
        assert_eq!(
            outcomes,
            [
                (Strike::Missed { by: None }, 0),
                (Strike::Hit { critical: false }, 1),
                (Strike::Hit { critical: true }, 5),
                (Strike::Hit { critical: true }, 6),
                (Strike::Hit { critical: true }, 7),
                (Strike::Soaked, 0),
            ]
        );
        // Penetration 2 against Pv 4 leaves Pv 2 (soaks 1 or 2); penetration 4 leaves none, and
        // no draw: the blow draws as against an unprotected target.
        for _ in 0..500 {
            let blow = strike(&fixed(2, 2), &Target::protected(dv, 4), &mut rng);
            if blow.strike == (Strike::Hit { critical: false }) {
                assert!(matches!(blow.damage, 1 | 2), "{}", blow.damage);
            }
            let mut bare = rng.clone();
            let through = strike(&fixed(2, 4), &Target::protected(dv, 4), &mut rng);
            assert_eq!(through, strike(&fixed(2, 0), &Target::bare(dv), &mut bare));
            assert_eq!(rng, bare);
            let past = strike(&fixed(2, 9), &Target::protected(dv, 4), &mut rng);
            assert_ne!(past.strike, Strike::Soaked);
        }
    }

    /// A target that resists `resist` (kind, percent) with `pv` against the physical kinds 0
    /// and 1 (2 is an element), `pv` adjusted by `vs` (kind, more).
    fn typed_target(pv: i32, vs: &[(usize, i32)], resist: &[(usize, i32)]) -> Target {
        let mut against = vec![pv, pv, 0];
        for &(k, more) in vs {
            against[k] += more;
        }
        let resistances = Resistances::sum(
            3,
            resist.iter().map(|&(k, p)| (DamageTypeId(k as u8), p)),
            [],
        );
        Target {
            defence: Defence::from_bonuses([(Category::Dexterity, -10)]),
            protection: Protection::for_test(pv, against),
            resistances,
        }
    }

    #[test]
    fn a_blow_does_the_better_of_its_kinds_and_elements_meet_no_armour() {
        let (slash, pierce, fire) = (DamageTypeId(0), DamageTypeId(1), DamageTypeId(2));
        let sword = Profile {
            bonus: 3,
            kinds: Kinds::of(&[slash, pierce]).expect("two kinds"),
            ..profile(0, "+0", 24, 2)
        };
        let mut rng = Rng::stream(11, "combat-test");
        // Mail that stands +1 against slashing: the sword stabs, through Pv 2 (soaks 1 or 2).
        let mail = typed_target(2, &[(0, 1)], &[]);
        for _ in 0..200 {
            let blow = strike(&sword, &mail, &mut rng);
            if blow.strike.is_hit() {
                assert_eq!(blow.kind, Some(pierce));
            }
            if blow.strike == (Strike::Hit { critical: false }) {
                assert!(blow.damage <= 2, "{}", blow.damage);
            }
        }
        // A target that resists piercing: the sword slashes, whatever the armour says.
        let tough = typed_target(2, &[(0, 1)], &[(1, 50)]);
        for _ in 0..50 {
            let blow = strike(&sword, &tough, &mut rng);
            if blow.strike.is_hit() {
                assert_eq!(blow.kind, Some(slash));
            }
        }
        // A fiery blow meets no Protection at all, and the resistance comes off after: 3 at
        // 50 % is 2 (half up); at 100 % it is resisted, not soaked, and does nothing.
        let torch = Profile {
            kinds: Kinds::one(fire),
            ..sword.clone()
        };
        let plated = typed_target(20, &[], &[(2, 50)]);
        let immune = typed_target(20, &[], &[(2, IMMUNE)]);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..100 {
            let mut bare = rng.clone();
            let blow = strike(&torch, &plated, &mut rng);
            // No soak draw: the same draws as against nobody's armour.
            let unarmoured = strike(&torch, &typed_target(0, &[], &[(2, 50)]), &mut bare);
            assert_eq!(blow, unarmoured);
            assert_eq!(rng, bare);
            seen.insert((blow.strike, blow.damage));
            let shrugged = strike(&torch, &immune, &mut rng);
            if shrugged.strike.is_hit() {
                assert_eq!((shrugged.strike, shrugged.damage), (Strike::Resisted, 0));
            }
        }
        assert!(
            seen.contains(&(Strike::Hit { critical: false }, 2)),
            "{seen:?}"
        );
        assert!(!seen.iter().any(|(s, _)| *s == Strike::Soaked));
    }

    #[test]
    fn riders_roll_after_the_weapons_dice_and_are_resisted_on_their_own() {
        let (pierce, fire) = (DamageTypeId(1), DamageTypeId(2));
        let arrow = Profile {
            bonus: 2,
            kinds: Kinds::one(pierce),
            riders: vec![Rider {
                kind: fire,
                dice: Dice::of(1, 4),
            }],
            ..profile(0, "+0", 24, 2)
        };
        let mut rng = Rng::stream(12, "combat-test");
        let plain = typed_target(0, &[], &[]);
        for _ in 0..200 {
            // The attack roll, (on a natural 24 the confirmation,) then for each roll of the
            // damage (a fixed 2, no draw) the fire die, nothing else.
            let mut expected = rng.clone();
            let natural = roll_natural(&mut expected);
            let blow = strike(&arrow, &plain, &mut rng);
            if natural != FUMBLE {
                let critical = natural == SURE_HIT && roll_natural(&mut expected) != FUMBLE;
                let times = if critical { 2 } else { 1 };
                let fire: i32 = (0..times).map(|_| Dice::of(1, 4).roll(&mut expected)).sum();
                assert_eq!(rng, expected, "the rider is the last draw");
                assert_eq!(blow.damage, 2 * times + fire);
                assert_eq!(blow.kind, Some(pierce));
            }
        }
        // Mail of Pv 4 soaks the point (2 to 4 off the 2: nothing left) but never the flame:
        // the blow still burns for the fire die alone, and is a hit, not soaked.
        let mail = typed_target(4, &[], &[]);
        let mut burnt = 0;
        for _ in 0..200 {
            let blow = strike(&arrow, &mail, &mut rng);
            if blow.strike == (Strike::Hit { critical: false }) {
                assert!((1..=4).contains(&blow.damage), "{}", blow.damage);
                burnt += 1;
            }
            assert_ne!(blow.strike, Strike::Soaked, "the flame is never soaked");
        }
        assert!(burnt > 100);
        // Against a target immune to fire the point alone counts; one immune to both is
        // resisted, not soaked (there was damage to resist).
        let fireproof = typed_target(0, &[], &[(2, IMMUNE)]);
        let unhurt = typed_target(0, &[], &[(1, IMMUNE), (2, IMMUNE)]);
        for _ in 0..50 {
            let blow = strike(&arrow, &fireproof, &mut rng);
            if blow.strike == (Strike::Hit { critical: false }) {
                assert_eq!(blow.damage, 2);
            }
            let blow = strike(&arrow, &unhurt, &mut rng);
            if blow.strike.is_hit() {
                assert_eq!((blow.strike, blow.damage), (Strike::Resisted, 0));
            }
        }
        // A blow soaked to nothing with no rider is still soaked, not resisted.
        let point = Profile {
            riders: Vec::new(),
            ..arrow
        };
        let mut soaked = false;
        for _ in 0..100 {
            let blow = strike(&point, &mail, &mut rng);
            soaked |= blow.strike == Strike::Soaked;
            assert_ne!(blow.strike, Strike::Resisted);
        }
        assert!(soaked);
    }

    #[test]
    fn profiles_add_the_right_modifiers() {
        let stats = |st: u8, dx: u8| {
            let scores = BTreeMap::from([(Stat::Strength, st), (Stat::Dexterity, dx)]);
            Stats::from_data(&scores, &BTreeMap::new()).expect("stats")
        };
        let sword = Attack {
            damage: dice("1d8"),
            to_hit: 1,
            threat: 20,
            crit: 2,
            strength: None,
            kinds: Kinds::NONE,
        };
        // Melee: Strength to hit and to damage.
        let melee = Profile::melee(&sword, &stats(16, 18));
        assert_eq!((melee.to_hit, melee.bonus, melee.threat), (4, 3, 20));
        // Thrown: Dexterity to hit, Strength to damage.
        let thrown = Profile::thrown(&sword, &stats(16, 18));
        assert_eq!((thrown.to_hit, thrown.bonus), (5, 3));
        // Ranged: Dexterity to hit, no modifier to damage; the bow's draw weight is its dice's.
        let bow = Attack {
            damage: dice("1d8+2"),
            to_hit: 0,
            threat: 22,
            crit: 3,
            strength: Some(14),
            kinds: Kinds::NONE,
        };
        let strong = Profile::ranged(&bow, &stats(16, 14));
        assert_eq!((strong.to_hit, strong.bonus, strong.crit), (2, 0, 3));
        // Each point of Strength short of what the bow needs: −1 to hit and −1 damage.
        let weak = Profile::ranged(&bow, &stats(11, 14));
        assert_eq!((weak.to_hit, weak.bonus), (-1, -3));
        // Bare hands: 1d3 plus Strength.
        let fists = Profile::melee(&Attack::unarmed(None), &stats(8, 10));
        assert_eq!(
            (fists.damage, fists.to_hit, fists.bonus),
            (dice("1d3"), -1, -1)
        );
    }

    #[test]
    fn hit_points_and_regeneration_follow_toughness() {
        // #113: a dwarf barbarian (6 + 8) with To 16 has 30; a human warrior (4 + 7) with To 12
        // has 23; an elf wizard (2 + 0) with To 8 has 10; a giant rat (0) with To 8 has 8.
        assert_eq!(max_hp(16, 14), 30);
        assert_eq!(max_hp(12, 11), 23);
        assert_eq!(max_hp(8, 2), 10);
        assert_eq!(max_hp(8, 0), 8);
        assert_eq!(max_hp(1, -5), 1, "never below 1");
        assert_eq!(regen_seconds(10), 240);
        assert_eq!(regen_seconds(1), 348);
        assert_eq!(regen_seconds(25), 60);
        assert_eq!(regen_seconds(40), 60, "a minute at the quickest");
        assert_eq!(regen_seconds(0), 360);
    }

    #[test]
    fn health_reads_in_words() {
        let words: Vec<&str> = (0..=12).map(|hp| Health::of(hp, 12).word()).collect();
        assert_eq!(
            words,
            [
                "nearly dead",
                "nearly dead",
                "nearly dead",
                "badly wounded",
                "badly wounded",
                "wounded",
                "wounded",
                "wounded",
                "wounded",
                "lightly wounded",
                "lightly wounded",
                "lightly wounded",
                "unhurt"
            ]
        );
        assert_eq!(Health::of(-3, 12), Health::NearlyDead);
        assert_eq!(Health::of(1, 1), Health::Unhurt);
    }
}
