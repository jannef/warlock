//! A roll of dice of more than one size (spec §8 "Dice"; the spell scaling design §6.2): a sum
//! of [`Dice`] groups and one fixed bonus, such as `3d6+1d10`, `5d6+2`, `2d8`, or `+3`.
//!
//! [`Dice`] stays one group for stats, weapons, and everything else; a `Roll` is what an attack
//! rolls for its damage ([`crate::combat::Profile::damage`]), so a spell's mixed dice roll as a
//! blow's do. The text form is groups `NdS` joined by `+`, at most [`Roll::MAX_GROUPS`], each of
//! 1 to [`MAX_COUNT`] dice of 2 to [`Roll::MAX_SIDES`] sides (a d7 or a d9 is a die like any
//! other), then a fixed number last and at most once (`+2`, `-1`); or a fixed number alone. No
//! spaces, lower-case `d`.
//!
//! A `Roll` made from a `Dice` ([`From<Dice>`]) keeps it exactly, a subtracted group or more
//! sides than the text allows included, and rolls the very draws the `Dice` would: one draw per
//! die, group by group, the bonus drawing nothing.

use super::{Dice, MAX_BONUS, MAX_COUNT, in_bonus_range, number};
use crate::rng::Rng;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Groups of dice plus one fixed bonus. Unused group slots are empty ([`Dice::default`]), so
/// equal rolls compare equal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Roll {
    /// The groups, each with no bonus of its own; the first `len` are used.
    groups: [Dice; Roll::MAX_GROUPS],
    len: u8,
    bonus: i32,
}

impl Roll {
    /// The most groups of dice one roll may have.
    pub const MAX_GROUPS: usize = 4;
    /// The most sides a die may have in the text form.
    pub const MAX_SIDES: u32 = 20;

    /// A fixed number: no dice.
    pub const fn fixed(value: i32) -> Self {
        Self {
            groups: [Dice::fixed(0); Self::MAX_GROUPS],
            len: 0,
            bonus: value,
        }
    }

    /// Its groups of dice, in order, each without a bonus.
    pub fn groups(&self) -> &[Dice] {
        &self.groups[..usize::from(self.len)]
    }

    /// Its fixed bonus (`+2` of `5d6+2`).
    pub const fn bonus(&self) -> i32 {
        self.bonus
    }

    /// Whether it rolls no dice.
    pub const fn is_fixed(&self) -> bool {
        self.len == 0
    }

    /// Whether it is always 0.
    pub const fn is_zero(&self) -> bool {
        self.is_fixed() && self.bonus == 0
    }

    /// The lowest result.
    pub fn min(&self) -> i32 {
        self.bonus + self.groups().iter().map(Dice::min).sum::<i32>()
    }

    /// The highest result.
    pub fn max(&self) -> i32 {
        self.bonus + self.groups().iter().map(Dice::max).sum::<i32>()
    }

    /// Twice its mean, an integer (`3d6+1d10`: 2 × 16, 32): a die of `S` sides averages
    /// `(S + 1) / 2`.
    pub fn mean_x2(&self) -> i32 {
        let dice: i32 = (self.groups().iter())
            .map(|g| {
                let twice = (g.count * (g.sides + 1)) as i32;
                if g.negative { -twice } else { twice }
            })
            .sum();
        2 * self.bonus + dice
    }

    /// Rolls it: each group in order, one draw from `rng` per die (uniform 1 to its sides); a
    /// die showing `reroll` or less is drawn once more, and that draw stands (once, never
    /// twice). With `reroll` 0 it draws exactly what the [`Dice`] it may have come from draws.
    pub fn roll(&self, rng: &mut Rng, reroll: u32) -> i32 {
        let dice: i32 = (self.groups().iter())
            .map(|g| {
                let sides = g.sides as i32;
                let sum: i32 = (0..g.count)
                    .map(|_| match rng.range(1, sides) {
                        low if low as u32 <= reroll => rng.range(1, sides),
                        die => die,
                    })
                    .sum();
                if g.negative { -sum } else { sum }
            })
            .sum();
        self.bonus + dice
    }

    /// It with every group's dice `sides` sides bigger (`grow(1)`: `3d6` → `3d7`).
    pub fn grow(mut self, sides: u32) -> Self {
        let len = usize::from(self.len);
        for group in &mut self.groups[..len] {
            group.sides += sides;
        }
        self
    }

    /// It with `dice` dice more of its first group's kind (`more(1)`: `3d6+1d10` → `4d6+1d10`);
    /// a fixed number stays as it is.
    pub fn more(mut self, dice: u32) -> Self {
        if self.len > 0 {
            self.groups[0].count += dice;
        }
        self
    }

    /// It with `bonus` added to its fixed bonus.
    pub const fn plus(self, bonus: i32) -> Self {
        Self {
            bonus: self.bonus + bonus,
            ..self
        }
    }
}

impl From<Dice> for Roll {
    /// The one group of `dice` (none for a fixed number) and its bonus.
    fn from(dice: Dice) -> Self {
        let mut roll = Self::fixed(dice.bonus);
        if !dice.is_fixed() {
            roll.groups[0] = Dice { bonus: 0, ..dice };
            roll.len = 1;
        }
        roll
    }
}

impl fmt::Display for Roll {
    /// As written: `3d6+1d10`, `5d6+2`, `2d8`, `+3`, `-1`, `0` (a one-group roll prints as the
    /// [`Dice`] it came from: `-1d2`, `1d4-1`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_fixed() {
            return Dice::fixed(self.bonus).fmt(f);
        }
        for (i, g) in self.groups().iter().enumerate() {
            let sign = match (g.negative, i) {
                (true, _) => "-",
                (false, 0) => "",
                (false, _) => "+",
            };
            write!(f, "{sign}{}d{}", g.count, g.sides)?;
        }
        match self.bonus {
            0 => Ok(()),
            b => write!(f, "{b:+}"),
        }
    }
}

impl FromStr for Roll {
    type Err = String;

    /// Parses the text form (see the module docs); surrounding whitespace is ignored.
    fn from_str(text: &str) -> Result<Self, String> {
        let bad =
            |why: &str| format!("bad roll `{text}`: {why} (expected e.g. 3d6+1d10, 5d6+2, +3)");
        let s = text.trim();
        // A fixed number alone, as a `Dice` reads it.
        if !s.contains('d') {
            let dice: Dice = s.parse().map_err(|_| bad("not a number"))?;
            return Ok(Self::from(dice));
        }
        let mut roll = Self::fixed(0);
        let mut rest = s;
        let mut fixed = false;
        while !rest.is_empty() {
            if fixed {
                return Err(bad("the fixed number comes last and once"));
            }
            let (sign, body) = match rest.as_bytes()[0] {
                b'+' => ('+', &rest[1..]),
                b'-' => ('-', &rest[1..]),
                _ => ('+', rest),
            };
            let end = body.find(['+', '-']).unwrap_or(body.len());
            let (term, after) = body.split_at(end);
            rest = after;
            let Some((count, sides)) = term.split_once('d') else {
                let n = number(term).ok_or_else(|| bad("a number is missing"))?;
                let n = if sign == '-' { -n } else { n };
                if !in_bonus_range(n) {
                    return Err(bad(&format!("beyond ±{MAX_BONUS}")));
                }
                roll.bonus = n;
                fixed = true;
                continue;
            };
            if sign == '-' {
                return Err(bad("dice are added, never subtracted"));
            }
            let count = number(count).ok_or_else(|| bad("the number of dice is missing"))?;
            let sides = number(sides).ok_or_else(|| bad("the number of sides is missing"))?;
            if !(1..=MAX_COUNT as i32).contains(&count) {
                return Err(bad(&format!("1 to {MAX_COUNT} dice")));
            }
            if !(2..=Self::MAX_SIDES as i32).contains(&sides) {
                return Err(bad(&format!("dice have 2 to {} sides", Self::MAX_SIDES)));
            }
            if usize::from(roll.len) == Self::MAX_GROUPS {
                return Err(bad(&format!("at most {} groups of dice", Self::MAX_GROUPS)));
            }
            roll.groups[usize::from(roll.len)] = Dice::of(count as u32, sides as u32);
            roll.len += 1;
        }
        if roll.len == 0 {
            return Err(bad("no dice"));
        }
        Ok(roll)
    }
}

impl Serialize for Roll {
    /// As its text form.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Roll {
    /// From the text form, or from an integer (a fixed number).
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RollVisitor;

        impl Visitor<'_> for RollVisitor {
            type Value = Roll;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a number or dice such as \"3d6+1d10\"")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Roll, E> {
                i32::try_from(v)
                    .ok()
                    .filter(|&v| in_bonus_range(v))
                    .map(Roll::fixed)
                    .ok_or_else(|| E::custom(format!("{v} is beyond ±{MAX_BONUS}")))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Roll, E> {
                self.visit_i64(i64::try_from(v).unwrap_or(i64::MAX))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Roll, E> {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(RollVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn roll(text: &str) -> Roll {
        text.parse().unwrap_or_else(|e| panic!("{e}"))
    }

    fn dice(text: &str) -> Dice {
        text.parse().unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn parses_and_prints_as_written() {
        for text in [
            "3d6+1d10",
            "5d6+2",
            "2d8",
            "+3",
            "-1",
            "0",
            "1d7+1d9",
            "1d2+1d3+1d4+1d20-5",
        ] {
            assert_eq!(roll(text).to_string(), text, "{text}");
            assert_eq!(
                roll(&roll(text).to_string()),
                roll(text),
                "{text} round trip"
            );
        }
        let r = roll("3d6+1d10+2");
        assert_eq!(r.groups(), [Dice::of(3, 6), Dice::of(1, 10)]);
        assert_eq!(r.bonus(), 2);
        assert_eq!(roll(" 2d8 "), roll("2d8"));
        assert_eq!(roll("3"), Roll::fixed(3));
        assert!(roll("+3").is_fixed());
    }

    #[test]
    fn refuses_malformed_and_out_of_range_text() {
        for (text, why) in [
            ("", "not a number"),
            ("1d6+1d6+1d6+1d6+1d6", "at most 4 groups"),
            ("1d6+1+2", "comes last and once"),
            ("1d6+1+1d4", "comes last and once"),
            ("+1+1d6", "comes last and once"),
            ("1d1", "2 to 20 sides"),
            ("1d21", "2 to 20 sides"),
            ("0d6", "1 to 100 dice"),
            ("1d6-1d4", "never subtracted"),
            ("-1d6", "never subtracted"),
            ("1d6+", "a number is missing"),
            ("1d6++1", "a number is missing"),
            ("d6", "number of dice is missing"),
            ("2d", "number of sides is missing"),
            ("2D3", "not a number"),
            ("1d6+1001", "beyond ±1000"),
        ] {
            let err = text.parse::<Roll>().expect_err(text);
            assert!(err.contains(why), "{text}: {err}");
            assert!(err.contains(&format!("`{text}`")), "{err}");
        }
    }

    #[test]
    fn min_max_and_twice_the_mean() {
        for (text, min, max, mean_x2) in [
            ("3d6+1d10", 4, 28, 32),
            ("5d6+2", 7, 32, 39),
            ("2d8", 2, 16, 18),
            ("1d7", 1, 7, 8),
            ("+3", 3, 3, 6),
        ] {
            let r = roll(text);
            assert_eq!(
                (r.min(), r.max(), r.mean_x2()),
                (min, max, mean_x2),
                "{text}"
            );
        }
        let minus = Roll::from(dice("-2d4+3"));
        assert_eq!((minus.min(), minus.max(), minus.mean_x2()), (-5, 1, -4));
    }

    #[test]
    fn every_value_comes_up_within_min_and_max() {
        for text in ["3d6+1d10", "1d7+1d9-2", "2d8", "+3"] {
            let r = roll(text);
            let mut rng = Rng::stream(7, "roll-test");
            let mut seen = BTreeMap::new();
            for _ in 0..20_000 {
                *seen.entry(r.roll(&mut rng, 0)).or_insert(0) += 1;
            }
            let all: Vec<i32> = (r.min()..=r.max()).collect();
            assert_eq!(seen.keys().copied().collect::<Vec<_>>(), all, "{text}");
        }
    }

    #[test]
    fn grows_adds_dice_and_bonus() {
        assert_eq!(roll("3d6+1d10").grow(1), roll("3d7+1d11"));
        assert_eq!(roll("3d6+1d10").more(2), roll("5d6+1d10"));
        assert_eq!(roll("5d6").plus(2), roll("5d6+2"));
        assert_eq!(roll("5d6+2").plus(-2), roll("5d6"));
        assert_eq!(roll("+3").grow(1).more(1), roll("+3"));
        assert_eq!(roll("1d20").grow(2).to_string(), "1d22");
    }

    #[test]
    fn a_dice_converts_exactly_and_rolls_its_draws() {
        for text in [
            "1d8",
            "2d3+1",
            "1d4-1",
            "-1d2",
            "-2d4+3",
            "3d1000-1000",
            "+2",
            "0",
        ] {
            let d = dice(text);
            let r = Roll::from(d);
            assert_eq!(r.to_string(), d.to_string(), "{text}");
            assert_eq!((r.min(), r.max()), (d.min(), d.max()), "{text}");
            let mut a = Rng::stream(11, "roll-test");
            let mut b = a.clone();
            for _ in 0..200 {
                assert_eq!(r.roll(&mut a, 0), d.roll(&mut b), "{text}");
            }
            assert_eq!(a, b, "{text}: the same number of draws");
        }
    }

    #[test]
    fn a_reroll_draws_once_more_per_low_die_and_never_twice() {
        // Each die by hand: a die at or below the reroll is drawn once more and that one stands.
        let r = roll("4d6+1d10");
        for reroll in [1, 2, 5] {
            let mut a = Rng::stream(5, "roll-test");
            let mut b = a.clone();
            for _ in 0..500 {
                let got = r.roll(&mut a, reroll);
                let mut want = 0;
                for sides in [6, 6, 6, 6, 10] {
                    let first = b.range(1, sides);
                    want += if first as u32 <= reroll {
                        b.range(1, sides)
                    } else {
                        first
                    };
                }
                assert_eq!(got, want, "reroll {reroll}");
                assert_eq!(a, b, "reroll {reroll}: the same draws");
            }
        }
        // Rerolling every face still draws twice a die, never more.
        let mut a = Rng::stream(5, "roll-test");
        let mut b = a.clone();
        roll("3d6").roll(&mut a, 6);
        for _ in 0..6 {
            b.range(1, 6);
        }
        assert_eq!(a, b);
    }

    #[test]
    fn reads_and_writes_its_text() {
        let json = serde_json::to_string(&roll("3d6+1d10+2")).expect("json");
        assert_eq!(json, "\"3d6+1d10+2\"");
        assert_eq!(
            serde_json::from_str::<Roll>(&json).ok(),
            Some(roll("3d6+1d10+2"))
        );
        assert_eq!(serde_json::from_str::<Roll>("3").ok(), Some(Roll::fixed(3)));
        #[derive(Deserialize)]
        struct Spell {
            damage: Roll,
        }
        let spell: Spell = toml::from_str("damage = \"2d6+1d8\"").expect("toml");
        assert_eq!(spell.damage, roll("2d6+1d8"));
        let err = toml::from_str::<Spell>("damage = \"1d21\"")
            .err()
            .expect("refused");
        assert!(err.to_string().contains("2 to 20 sides"), "{err}");
    }
}
