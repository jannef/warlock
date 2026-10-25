//! Burden (spec §8 "Burden"): how much an actor can carry, and what carrying more costs it.
//!
//! **Limit**: 1.5 kg × (2 × Strength + Toughness) × its size's `carry` percent (small races and
//! creatures 75 %; `data/sizes.toml`), from the scores, never the modifiers: a St 10 To 10 human
//! carries up to 45 kg. Everything it carries counts, worn and wielded things included; its own
//! body does not. The limit in grams is always a multiple of 3 (15 × the score sum × the
//! percent), so the thirds below are exact.
//!
//! **Levels**, by thirds of the limit ([`Burden`]): Unburdened up to a third, Burdened up to two
//! thirds, Strained up to the limit, Overloaded beyond it. Each level costs the actor:
//! - **time**: every action takes ×1.25 Burdened and ×1.5 Strained or Overloaded
//!   ([`Burden::slow`], applied to every price in `world/schedule.rs`);
//! - **Defence**: its Dexterity bonus is capped at +3 Burdened, +1 Strained, 0 Overloaded
//!   ([`Burden::max_dex`]; worn armour's `max_dex` caps it too, and the lower cap wins);
//! - **noise**: its footsteps are ×1.25 and ×1.5 as loud ([`Burden::footsteps`]);
//! - **moving**: Overloaded, it cannot move at all ("You carry too much to move."); it can do
//!   anything else (drop things, pick things up, fight, wait), as slowly as when Strained.
//!
//! Magic items (spec §8 "Magic items") count by their scores and weight: a Strength affix raises
//! the limit with the score, a carrying affix adds its kilograms, and a lighter item weighs less.
//!
//! A race may shrug off Burdened (dwarves, `ignores_burdened`): Burdened costs it nothing, and
//! from Strained on it is treated like everyone else ([`Load::penalty`]). Creatures follow the
//! same rules from their stats and their type's size.
//!
//! The player sees their burden in words and their own load in weights (the limit and its thirds
//! follow from scores they know): never a modifier, never a creature's load.

use crate::content::{Content, Effect, RaceDef};
use crate::object::{format_weight, weight_of};
use crate::stats::Stat;
use crate::talent::TalentMods;
use crate::world::Actor;
use serde::Serialize;

/// Grams of carrying limit per point of (2 × Strength + Toughness) for a medium actor.
pub const GRAMS_PER_POINT: u64 = 1500;

/// How burdened an actor is: its load against its limit, by thirds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Burden {
    /// Up to a third of the limit: no penalty.
    Unburdened,
    /// Up to two thirds.
    Burdened,
    /// Up to the limit.
    Strained,
    /// Beyond the limit: it cannot move.
    Overloaded,
}

impl Burden {
    pub const ALL: [Burden; 4] = [
        Burden::Unburdened,
        Burden::Burdened,
        Burden::Strained,
        Burden::Overloaded,
    ];

    /// Its name, as the status bar shows it: `Burdened`.
    pub const fn name(self) -> &'static str {
        match self {
            Burden::Unburdened => "Unburdened",
            Burden::Burdened => "Burdened",
            Burden::Strained => "Strained",
            Burden::Overloaded => "Overloaded",
        }
    }

    /// Its id in dumps: `burdened`.
    pub const fn id(self) -> &'static str {
        match self {
            Burden::Unburdened => "unburdened",
            Burden::Burdened => "burdened",
            Burden::Strained => "strained",
            Burden::Overloaded => "overloaded",
        }
    }

    /// How much longer every action takes at this burden, as a fraction (numerator,
    /// denominator).
    const fn slowdown(self) -> (i64, i64) {
        match self {
            Burden::Unburdened => (1, 1),
            Burden::Burdened => (5, 4),
            Burden::Strained | Burden::Overloaded => (3, 2),
        }
    }

    /// The energy price of an action of `price` at this burden: ×1.25 Burdened, ×1.5 Strained
    /// or Overloaded, rounded up to a whole unit of energy (a thousandth of a second at normal
    /// speed; built-in prices scale exactly). Like any price it is paid from energy that carries
    /// over, so a run of slowed actions takes exactly its total time, without drift.
    pub fn slow(self, price: i64) -> i64 {
        self.slow_eased(price, 0)
    }

    /// [`Burden::slow`] for an actor whose skills (balance; spec §8 "Skills") take `ease`
    /// percent (0 to 100) off the slowdown: Burdened ×1.25 becomes ×1.10 with 60 off, Strained
    /// ×1.5 becomes ×1.2. Rounded up like the rest.
    pub fn slow_eased(self, price: i64, ease: i32) -> i64 {
        let (num, den) = self.slowdown();
        let keep = i64::from(100 - ease.clamp(0, 100));
        let whole = 100 * den;
        let factor = whole + (num - den) * keep;
        price.saturating_mul(factor).saturating_add(whole - 1) / whole
    }

    /// The most its Dexterity modifier may add to Defence at this burden: +3 Burdened, +1
    /// Strained, 0 Overloaded; `None` (no cap) Unburdened. A penalty counts in full.
    pub const fn max_dex(self) -> Option<i32> {
        match self {
            Burden::Unburdened => None,
            Burden::Burdened => Some(3),
            Burden::Strained => Some(1),
            Burden::Overloaded => Some(0),
        }
    }

    /// How loud its footsteps are at this burden, in percent of its unburdened ones.
    pub const fn footsteps(self) -> u32 {
        match self {
            Burden::Unburdened => 100,
            Burden::Burdened => 125,
            Burden::Strained | Burden::Overloaded => 150,
        }
    }

    /// Whether it can move at this burden: everything but Overloaded.
    pub const fn can_move(self) -> bool {
        !matches!(self, Burden::Overloaded)
    }

    /// What the player is told when their burden becomes this.
    pub const fn news(self) -> &'static str {
        match self {
            Burden::Unburdened => "You are no longer burdened.",
            Burden::Burdened => "You are burdened.",
            Burden::Strained => "You strain under your load.",
            Burden::Overloaded => MOVE_REFUSED,
        }
    }
}

/// What an overloaded player is told when they try to move.
pub const MOVE_REFUSED: &str = "You carry too much to move.";

/// What an actor carries against what it can (spec §8 "Burden").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Load {
    /// Everything it carries, worn and wielded things included, in grams.
    pub carried: u64,
    /// The most it can carry without being overloaded, in grams ([`limit`]).
    pub limit: u64,
    /// Whether being Burdened costs it nothing (dwarves).
    pub ignores_burdened: bool,
    /// Whether being Strained costs it what Burdened costs others (Stout, a dwarf's talent).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub stout: bool,
}

impl Load {
    /// How burdened it is: Unburdened up to a third of its limit, Burdened up to two thirds,
    /// Strained up to the limit, Overloaded beyond.
    pub fn burden(&self) -> Burden {
        let thirds = self.carried.saturating_mul(3);
        if thirds <= self.limit {
            Burden::Unburdened
        } else if thirds <= self.limit.saturating_mul(2) {
            Burden::Burdened
        } else if self.carried <= self.limit {
            Burden::Strained
        } else {
            Burden::Overloaded
        }
    }

    /// The burden it suffers: its burden, but Unburdened for one that ignores Burdened while it
    /// is Burdened, and Burdened for a stout one while it is Strained. Every penalty (time,
    /// Defence, noise, moving) goes by this.
    pub fn penalty(&self) -> Burden {
        match self.burden() {
            Burden::Burdened if self.ignores_burdened => Burden::Unburdened,
            Burden::Strained if self.stout => Burden::Burdened,
            burden => burden,
        }
    }

    /// The weights over which it is Burdened, Strained, and Overloaded, in grams: a third of its
    /// limit, two thirds, and the limit.
    pub fn thresholds(&self) -> [(Burden, u64); 3] {
        [
            (Burden::Burdened, self.limit / 3),
            (Burden::Strained, self.limit / 3 * 2),
            (Burden::Overloaded, self.limit),
        ]
    }

    /// The thresholds in words: `burdened over 15 kg`, `strained over 30 kg`, `overloaded over
    /// 45 kg`.
    pub fn over(&self) -> [String; 3] {
        (self.thresholds()).map(|(b, g)| format!("{} over {}", b.id(), format_weight(g)))
    }
}

/// The carrying limit in grams of an actor with Strength `strength` and Toughness `toughness`
/// whose size carries `carry` percent: 1.5 kg × (2 × Strength + Toughness) × `carry` / 100.
pub fn limit(strength: u8, toughness: u8, carry: u16) -> u64 {
    let points = 2 * u64::from(strength) + u64::from(toughness);
    GRAMS_PER_POINT * points * u64::from(carry) / 100
}

/// The load of `actor`. `race` is the player's race when the run has a character: it gives their
/// size and whether Burdened costs them anything. Anyone else goes by its creature type's size.
pub fn load(actor: &Actor, race: Option<&RaceDef>, content: &Content) -> Load {
    let (size, ignores_burdened) = match race {
        Some(race) => (race.size, race.ignores_burdened),
        None => (content.creature(actor.creature).size, false),
    };
    // Magic items that raise Strength or Toughness raise the limit; a status effect that lowers
    // either, or hunger sapping Strength (Weak, spec §8 "Hunger"), lowers it.
    let stats = actor.effective_stats(content);
    let base = limit(
        stats.score(Stat::Strength),
        stats.score(Stat::Toughness),
        content.size(size).carry,
    );
    // Talents (Pack mule) scale the limit; magic items and talents (Deep pockets) add to it.
    // Porter shrugs off Burdened as a dwarf does.
    let talents = actor.talent_mods(content);
    let base = TalentMods::scaled(base, talents.carry_percent);
    // Deep pockets (spec §8 "Class abilities"): coins weigh nothing.
    let coins = content.item_class_id("coins");
    let carried = match talents.weightless_coins {
        false => weight_of(&actor.inventory, content),
        true => (actor.inventory.iter())
            .filter(|s| coins.is_none_or(|c| content.object(s.object).class != c))
            .map(|s| s.weight(content))
            .sum(),
    };
    Load {
        carried,
        limit: with_carry(
            base,
            actor.mods(content).get(Effect::Carry) + talents.carry_kg,
        ),
        // Porter and Steadfast (spec §8) shrug Burdened off as a dwarf does.
        ignores_burdened: ignores_burdened || talents.ignore_burdened || talents.steadfast,
        // Stout (a dwarf's talent): Strained costs what Burdened costs others.
        stout: talents.stout,
    }
}

/// `limit` raised by `kg` kilograms that magic items add (spec §8 "Magic items"), kept a multiple
/// of 3 grams so the thirds stay exact.
pub fn with_carry(limit: u64, kg: i32) -> u64 {
    let more = u64::try_from(kg.max(0)).unwrap_or(0) * 1000;
    (limit + more) / 3 * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(carried: u64, limit: u64) -> Load {
        Load {
            carried,
            limit,
            ignores_burdened: false,
            stout: false,
        }
    }

    #[test]
    fn limits_follow_strength_toughness_and_size() {
        // The issue's sample characters.
        assert_eq!(limit(10, 10, 100), 45_000, "human, St 10 To 10");
        assert_eq!(limit(16, 14, 100), 69_000, "half-orc barbarian");
        assert_eq!(limit(14, 16, 100), 66_000, "dwarf warrior");
        assert_eq!(limit(8, 12, 75), 31_500, "halfling thief");
        assert_eq!(limit(6, 8, 100), 30_000, "elf wizard");
        assert_eq!(limit(1, 1, 75), 3_375);
        assert_eq!(limit(40, 40, 1000), 1_800_000);
        // Always a multiple of 3: the thirds are exact.
        for (st, to, carry) in [(1, 1, 75), (11, 10, 100), (7, 13, 75), (9, 9, 10)] {
            assert_eq!(limit(st, to, carry) % 3, 0, "{st} {to} {carry}");
        }
    }

    #[test]
    fn levels_are_thirds_of_the_limit() {
        let limit = 45_000;
        for (carried, burden) in [
            (0, Burden::Unburdened),
            (15_000, Burden::Unburdened),
            (15_001, Burden::Burdened),
            (30_000, Burden::Burdened),
            (30_001, Burden::Strained),
            (45_000, Burden::Strained),
            (45_001, Burden::Overloaded),
            (u64::MAX, Burden::Overloaded),
        ] {
            assert_eq!(load(carried, limit).burden(), burden, "{carried} g");
        }
        assert_eq!(
            load(0, limit).thresholds(),
            [
                (Burden::Burdened, 15_000),
                (Burden::Strained, 30_000),
                (Burden::Overloaded, 45_000)
            ]
        );
        // A halfling thief: 10.5, 21, and 31.5 kg.
        let halfling = load(21_000, 31_500);
        assert_eq!(halfling.burden(), Burden::Burdened);
        assert_eq!(
            halfling.thresholds().map(|(_, g)| g),
            [10_500, 21_000, 31_500]
        );
        assert_eq!(
            halfling.over(),
            [
                "burdened over 10.5 kg",
                "strained over 21 kg",
                "overloaded over 31.5 kg"
            ]
        );
    }

    #[test]
    fn those_who_ignore_burdened_suffer_nothing_until_strained() {
        let dwarf = |carried| Load {
            carried,
            limit: 66_000,
            ignores_burdened: true,
            stout: false,
        };
        assert_eq!(dwarf(30_000).burden(), Burden::Burdened, "still burdened");
        assert_eq!(
            dwarf(30_000).penalty(),
            Burden::Unburdened,
            "but no penalty"
        );
        assert_eq!(dwarf(50_000).penalty(), Burden::Strained);
        // Stout: Strained costs what Burdened costs others (not nothing, as a dwarf's Burdened).
        let stout = |carried| Load {
            stout: true,
            ..dwarf(carried)
        };
        assert_eq!(stout(50_000).burden(), Burden::Strained);
        assert_eq!(stout(50_000).penalty(), Burden::Burdened);
        assert_eq!(stout(30_000).penalty(), Burden::Unburdened);
        assert_eq!(stout(70_000).penalty(), Burden::Overloaded);
        assert_eq!(dwarf(70_000).penalty(), Burden::Overloaded);
        assert_eq!(load(30_000, 66_000).penalty(), Burden::Burdened);
    }

    #[test]
    fn balance_eases_the_slowdown_and_nothing_else() {
        let eased = |b: Burden, ease| b.slow_eased(6000, ease);
        // Burdened ×1.25 → ×1.10 with 60 off; Strained ×1.5 → ×1.2; all of it off: no slowdown.
        assert_eq!(
            [0, 30, 60, 100].map(|e| eased(Burden::Burdened, e)),
            [7500, 7050, 6600, 6000]
        );
        assert_eq!(eased(Burden::Strained, 60), 7200);
        assert_eq!(eased(Burden::Overloaded, 60), 7200);
        assert_eq!(eased(Burden::Unburdened, 60), 6000);
        // Out of range eases are kept within 0 to 100; no ease is the plain slowdown.
        assert_eq!(eased(Burden::Burdened, -5), 7500);
        assert_eq!(eased(Burden::Burdened, 500), 6000);
        for price in [1, 999, 6000, 5_400_001] {
            assert_eq!(Burden::Unburdened.slow_eased(price, 0), price);
            assert_eq!(Burden::Burdened.slow_eased(price, 0), (price * 5 + 3) / 4);
            assert_eq!(Burden::Strained.slow_eased(price, 0), (price * 3 + 1) / 2);
        }
        // Rounded up.
        assert_eq!(Burden::Burdened.slow_eased(1, 60), 2);
    }

    #[test]
    fn burden_slows_every_price_and_caps_dexterity_and_louder_steps() {
        let prices: Vec<i64> = (Burden::ALL.iter()).map(|b| b.slow(6000)).collect();
        assert_eq!(prices, [6000, 7500, 9000, 9000]);
        // An overworld step of 20 minutes: 25 and 30 minutes.
        assert_eq!(Burden::Burdened.slow(1_200_000), 1_500_000);
        assert_eq!(Burden::Strained.slow(1_200_000), 1_800_000);
        // Rounded up to a whole unit of energy, never below 1.
        assert_eq!(Burden::Burdened.slow(1), 2);
        assert_eq!(Burden::Burdened.slow(5401), 6752);
        assert_eq!(Burden::Strained.slow(1), 2);
        assert_eq!(Burden::Unburdened.slow(5401), 5401);
        let caps: Vec<_> = (Burden::ALL.iter()).map(|b| b.max_dex()).collect();
        assert_eq!(caps, [None, Some(3), Some(1), Some(0)]);
        let steps: Vec<_> = (Burden::ALL.iter()).map(|b| b.footsteps()).collect();
        assert_eq!(steps, [100, 125, 150, 150]);
        let moves: Vec<_> = (Burden::ALL.iter()).map(|b| b.can_move()).collect();
        assert_eq!(moves, [true, true, true, false]);
    }
}
