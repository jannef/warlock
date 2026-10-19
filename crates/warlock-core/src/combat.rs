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
//! - **Hit points**: Toughness plus what race and class (a creature: its type) give
//!   ([`max_hp`]); one comes back every `max(60 s, 360 s − 12 s × Toughness)` of game time
//!   ([`regen_seconds`]).
//!
//! Every roll comes from the gameplay random stream, so fights replay exactly. Nothing here is
//! shown to the player as a number but their own hit points and Defence: never a to-hit, a
//! modifier, or a roll.

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub to_hit: i32,
    pub damage: Dice,
    /// Added to every roll of the damage dice.
    pub bonus: i32,
    /// The lowest natural roll that threatens a critical.
    pub threat: u8,
    /// How many times a confirmed critical rolls the damage.
    pub crit: u8,
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
}

impl Attack {
    /// Bare hands: 1d3, default critical.
    pub fn unarmed() -> Self {
        Self {
            damage: unarmed_dice(),
            to_hit: 0,
            threat: DEFAULT_THREAT,
            crit: DEFAULT_CRIT,
            strength: None,
        }
    }
}

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
        }
    }

    /// One roll of the damage: the dice plus the bonus (it may be below 1; a blow's total is
    /// at least 1).
    fn roll_damage(&self, rng: &mut Rng) -> i32 {
        self.damage.roll(rng) + self.bonus
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
}

impl Strike {
    pub fn is_hit(self) -> bool {
        matches!(self, Strike::Hit { .. })
    }
}

/// An attack resolved: what came of it and the damage it does (0 for a miss, else at least 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blow {
    pub strike: Strike,
    pub damage: i32,
}

/// Resolves an attack with `profile` against `defence`, drawing from `rng` in this order: the
/// two dice of the attack roll; on an attributable miss one draw for why; on a threat the two
/// dice of the confirmation roll; then one draw per damage die for each time the damage is
/// rolled.
pub fn strike(profile: &Profile, defence: &Defence, rng: &mut Rng) -> Blow {
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
        return Blow {
            strike: Strike::Missed { by },
            damage: 0,
        };
    }
    let threat = natural >= i32::from(profile.threat);
    let critical = threat && hits(roll_natural(rng), profile.to_hit, dv);
    let times = if critical { profile.crit.max(1) } else { 1 };
    let damage: i32 = (0..times).map(|_| profile.roll_damage(rng)).sum();
    Blow {
        strike: Strike::Hit { critical },
        damage: damage.max(1),
    }
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

/// Seconds of game time per regenerated hit point: `max(60, 360 − 12 × Toughness)`.
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
        }
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
            match strike(&profile(0, "1d4", 2, 2), &hard, &mut rng).strike {
                Strike::Hit { .. } => hit += 1,
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
            let blow = strike(&fixed(3), &dv, &mut rng);
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
            let blow = strike(&p, &dv, &mut rng);
            match blow.strike {
                Strike::Hit { critical: true } => assert!((4..=10).contains(&blow.damage)),
                Strike::Hit { critical: false } => assert!((2..=5).contains(&blow.damage)),
                Strike::Missed { .. } => assert_eq!(blow.damage, 0),
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
            let blow = strike(&weak, &dv, &mut rng);
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
                .map(|_| strike(&p, &dv, &mut rng))
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
            let blow = strike(&profile(-20, "1d4", 22, 2), &dv, &mut rng);
            if natural < SURE_HIT {
                assert_eq!(blow.strike, Strike::Missed { by: None });
                assert_eq!(rng, expected, "two draws");
            }
            // Against a high Defence a roll of 10 or more misses for a reason: one draw more.
            let mut expected = rng.clone();
            let natural = roll_natural(&mut expected);
            let blow = strike(&profile(0, "1d4", 22, 2), &hard, &mut rng);
            if (BASE_DEFENCE..SURE_HIT).contains(&natural) {
                expected.below(20);
                let armour = Some(Category::Armour);
                assert_eq!(blow.strike, Strike::Missed { by: armour });
                assert_eq!(rng, expected, "three draws");
            }
        }
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
        };
        let strong = Profile::ranged(&bow, &stats(16, 14));
        assert_eq!((strong.to_hit, strong.bonus, strong.crit), (2, 0, 3));
        // Each point of Strength short of what the bow needs: −1 to hit and −1 damage.
        let weak = Profile::ranged(&bow, &stats(11, 14));
        assert_eq!((weak.to_hit, weak.bonus), (-1, -3));
        // Bare hands: 1d3 plus Strength.
        let fists = Profile::melee(&Attack::unarmed(), &stats(8, 10));
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
