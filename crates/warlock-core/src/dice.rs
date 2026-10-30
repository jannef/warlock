//! Dice (spec §8 "Dice"): a roll such as `2d3+1`, or a fixed number such as `+2`.
//!
//! One group of dice of one size: the stat adjustments of character creation (races, classes,
//! the horoscope), weapon and natural attack damage, and every other roll in data. The text form
//! is
//!
//! - a fixed number: `+2`, `-1`, `3`, `0`;
//! - dice: `NdS`, `N` dice with `S` sides each (`2d3`), optionally negated (`-1d2`, subtracted)
//!   and with a fixed number after them (`2d3+1`, `1d4-1`).
//!
//! `N` is 1 to [`MAX_COUNT`], `S` 2 to [`MAX_SIDES`], and fixed numbers lie within
//! ±[`MAX_BONUS`]. No spaces, lower-case `d`. In TOML a fixed number may also be written as an
//! integer (`strength = 2`). Rolling draws one number per die from a deterministic stream
//! ([`Rng`]); a fixed number draws nothing.
//!
//! A [`Roll`] (`dice/roll.rs`) is a sum of such groups, of more than one size (`3d6+1d10`): what
//! an attack rolls for its damage.

mod roll;

pub use roll::Roll;

use crate::rng::Rng;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// The most dice one expression may roll.
pub const MAX_COUNT: u32 = 100;
/// The most sides a die may have.
pub const MAX_SIDES: u32 = 1000;
/// The largest fixed number, either way.
pub const MAX_BONUS: i32 = 1000;

/// A dice expression: `count` dice of `sides` sides, added or (`negative`) subtracted, plus a
/// fixed `bonus`. With no dice (`count` 0) it is just the bonus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Dice {
    count: u32,
    sides: u32,
    negative: bool,
    bonus: i32,
}

impl Dice {
    /// A fixed number: no dice.
    pub const fn fixed(value: i32) -> Self {
        Self {
            count: 0,
            sides: 0,
            negative: false,
            bonus: value,
        }
    }

    /// `count` dice of `sides` sides each (`1d4`): at least 1 die of at least 2 sides.
    pub const fn of(count: u32, sides: u32) -> Self {
        Self {
            count: if count == 0 { 1 } else { count },
            sides: if sides < 2 { 2 } else { sides },
            negative: false,
            bonus: 0,
        }
    }

    /// It with `more` added to its fixed bonus (what a weapon's material adds, #193).
    pub const fn plus(self, more: i32) -> Self {
        Self {
            bonus: self.bonus + more,
            ..self
        }
    }

    /// It and `other` rolled together, as one: dice of the same sides (and sign) add up
    /// (`1d6` and `2d6`: `3d6`), as do fixed bonuses; otherwise the one with the higher best
    /// result counts (Sneak attack and Improved sneak attack).
    pub fn and(self, other: Self) -> Self {
        if self.negative == other.negative && (self.sides == other.sides || other.is_fixed()) {
            Self {
                count: self.count + other.count,
                bonus: self.bonus + other.bonus,
                ..self
            }
        } else if self.is_fixed() && !self.negative && !other.negative {
            other.plus(self.bonus)
        } else if other.max() > self.max() {
            other
        } else {
            self
        }
    }

    /// How many dice it rolls (0 for a fixed number).
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// How many sides its dice have (0 for a fixed number).
    pub const fn sides(&self) -> u32 {
        self.sides
    }

    /// Its fixed bonus (`+1` of `2d3+1`).
    pub const fn bonus(&self) -> i32 {
        self.bonus
    }

    /// Whether its dice are subtracted (`-1d2`).
    pub const fn is_negative(&self) -> bool {
        self.negative
    }

    /// Whether it rolls no dice.
    pub const fn is_fixed(&self) -> bool {
        self.count == 0
    }

    /// Whether it is always 0.
    pub const fn is_zero(&self) -> bool {
        self.is_fixed() && self.bonus == 0
    }

    /// The lowest result.
    pub const fn min(&self) -> i32 {
        if self.negative {
            self.bonus - (self.count * self.sides) as i32
        } else {
            self.bonus + self.count as i32
        }
    }

    /// The highest result.
    pub const fn max(&self) -> i32 {
        if self.negative {
            self.bonus - self.count as i32
        } else {
            self.bonus + (self.count * self.sides) as i32
        }
    }

    /// Rolls it: one draw from `rng` per die (uniform 1 to `sides`), none for a fixed number.
    pub fn roll(&self, rng: &mut Rng) -> i32 {
        let sides = self.sides as i32;
        let dice: i32 = (0..self.count).map(|_| rng.range(1, sides)).sum();
        if self.negative {
            self.bonus - dice
        } else {
            self.bonus + dice
        }
    }
}

impl fmt::Display for Dice {
    /// The canonical text form: `+2`, `-1`, `0`, `2d3+1`, `-1d2`, `1d4-1` (ASCII hyphens).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_fixed() {
            return match self.bonus {
                0 => write!(f, "0"),
                b => write!(f, "{b:+}"),
            };
        }
        let sign = if self.negative { "-" } else { "" };
        write!(f, "{sign}{}d{}", self.count, self.sides)?;
        match self.bonus {
            0 => Ok(()),
            b => write!(f, "{b:+}"),
        }
    }
}

impl FromStr for Dice {
    type Err = String;

    /// Parses the text form (see the module docs); surrounding whitespace is ignored.
    fn from_str(text: &str) -> Result<Self, String> {
        let bad = |why: &str| format!("bad dice `{text}`: {why} (expected e.g. 2d3+1, 1d4-1, +2)");
        let s = text.trim();
        let (negative, rest) = match s.as_bytes().first() {
            Some(b'-') => (true, &s[1..]),
            Some(b'+') => (false, &s[1..]),
            _ => (false, s),
        };
        let Some((count, after)) = rest.split_once('d') else {
            let value = number(rest).ok_or_else(|| bad("not a number"))?;
            let value = if negative { -value } else { value };
            return in_bonus_range(value)
                .then(|| Dice::fixed(value))
                .ok_or_else(|| bad(&format!("beyond ±{MAX_BONUS}")));
        };
        let count = number(count).ok_or_else(|| bad("the number of dice is missing"))?;
        let split = after.find(['+', '-']).unwrap_or(after.len());
        let (sides, bonus) = after.split_at(split);
        let sides = number(sides).ok_or_else(|| bad("the number of sides is missing"))?;
        let bonus = match bonus.split_at_checked(1) {
            None => 0,
            Some((sign, digits)) => {
                let n =
                    number(digits).ok_or_else(|| bad("the number after the dice is missing"))?;
                if sign == "-" { -n } else { n }
            }
        };
        if !(1..=MAX_COUNT as i32).contains(&count) {
            return Err(bad(&format!("1 to {MAX_COUNT} dice")));
        }
        if !(2..=MAX_SIDES as i32).contains(&sides) {
            return Err(bad(&format!("dice have 2 to {MAX_SIDES} sides")));
        }
        if !in_bonus_range(bonus) {
            return Err(bad(&format!("beyond ±{MAX_BONUS}")));
        }
        Ok(Dice {
            count: count as u32,
            sides: sides as u32,
            negative,
            bonus,
        })
    }
}

/// Plain decimal digits (no sign) as a number; `None` for anything else, or a number too large
/// to matter.
fn number(digits: &str) -> Option<i32> {
    if digits.is_empty() || digits.len() > 6 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn in_bonus_range(value: i32) -> bool {
    (-MAX_BONUS..=MAX_BONUS).contains(&value)
}

impl Serialize for Dice {
    /// As its text form.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Dice {
    /// From the text form, or from an integer (a fixed number).
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DiceVisitor;

        impl Visitor<'_> for DiceVisitor {
            type Value = Dice;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a number or dice such as \"2d3+1\"")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Dice, E> {
                i32::try_from(v)
                    .ok()
                    .filter(|&v| in_bonus_range(v))
                    .map(Dice::fixed)
                    .ok_or_else(|| E::custom(format!("{v} is beyond ±{MAX_BONUS}")))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Dice, E> {
                self.visit_i64(i64::try_from(v).unwrap_or(i64::MAX))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Dice, E> {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(DiceVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn dice(text: &str) -> Dice {
        text.parse().unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn parses_dice_and_fixed_numbers() {
        for (text, count, sides, negative, bonus) in [
            ("2d3+1", 2, 3, false, 1),
            ("1d4-1", 1, 4, false, -1),
            ("1d2", 1, 2, false, 0),
            ("-1d2", 1, 2, true, 0),
            ("+1d6", 1, 6, false, 0),
            ("-2d4+3", 2, 4, true, 3),
            ("3d6+0", 3, 6, false, 0),
            (" 1d20 ", 1, 20, false, 0),
            ("100d1000-1000", 100, 1000, false, -1000),
        ] {
            let expected = Dice {
                count,
                sides,
                negative,
                bonus,
            };
            assert_eq!(dice(text), expected, "{text}");
        }
        for (text, value) in [
            ("+2", 2),
            ("-1", -1),
            ("3", 3),
            ("0", 0),
            ("-0", 0),
            ("+1000", 1000),
        ] {
            assert_eq!(dice(text), Dice::fixed(value), "{text}");
            assert!(dice(text).is_fixed());
        }
    }

    #[test]
    fn refuses_malformed_and_out_of_range_text() {
        for (text, why) in [
            ("", "not a number"),
            ("+", "not a number"),
            ("abc", "not a number"),
            ("1.5", "not a number"),
            ("--1", "not a number"),
            ("d6", "number of dice is missing"),
            ("2d", "number of sides is missing"),
            ("2d+1", "number of sides is missing"),
            ("2d3+", "number after the dice is missing"),
            ("2d3+-1", "number after the dice is missing"),
            ("2d3+1+1", "number after the dice is missing"),
            ("2d3x", "number of sides is missing"),
            ("2 d3", "number of dice is missing"),
            ("2D3", "not a number"),
            ("1d3d4", "number of sides is missing"),
            ("0d6", "1 to 100 dice"),
            ("101d6", "1 to 100 dice"),
            ("1d1", "2 to 1000 sides"),
            ("1d0", "2 to 1000 sides"),
            ("1d1001", "2 to 1000 sides"),
            ("1d6+1001", "beyond ±1000"),
            ("-1001", "beyond ±1000"),
            ("99999999999", "not a number"),
        ] {
            let err = text.parse::<Dice>().expect_err(text);
            assert!(err.contains(why), "{text}: {err}");
            assert!(err.contains(&format!("`{text}`")), "{err}");
        }
    }

    #[test]
    fn min_and_max_bound_every_roll_and_every_value_comes_up() {
        for text in [
            "2d3+1", "1d4-1", "1d2", "-1d2", "-2d4+3", "3d6", "+2", "-1", "0",
        ] {
            let d = dice(text);
            let mut rng = Rng::stream(7, "dice-test");
            let mut seen = BTreeMap::new();
            for _ in 0..2000 {
                let v = d.roll(&mut rng);
                assert!((d.min()..=d.max()).contains(&v), "{text}: {v}");
                *seen.entry(v).or_insert(0) += 1;
            }
            let all: Vec<i32> = (d.min()..=d.max()).collect();
            assert_eq!(seen.keys().copied().collect::<Vec<_>>(), all, "{text}");
        }
        assert_eq!((dice("2d3+1").min(), dice("2d3+1").max()), (3, 7));
        assert_eq!((dice("1d4-1").min(), dice("1d4-1").max()), (0, 3));
        assert_eq!((dice("-1d2").min(), dice("-1d2").max()), (-2, -1));
        assert_eq!((dice("-2d4+3").min(), dice("-2d4+3").max()), (-5, 1));
        assert_eq!((dice("+2").min(), dice("+2").max()), (2, 2));
    }

    #[test]
    fn rolls_are_deterministic_and_fixed_numbers_draw_nothing() {
        let roll = |seed| {
            let mut rng = Rng::stream(seed, "dice-test");
            (0..20)
                .map(|_| dice("3d6+1").roll(&mut rng))
                .collect::<Vec<_>>()
        };
        assert_eq!(roll(1), roll(1));
        assert_ne!(roll(1), roll(2));
        // A fixed number leaves the stream where it was.
        let mut a = Rng::stream(3, "dice-test");
        let mut b = a.clone();
        assert_eq!(dice("+2").roll(&mut a), 2);
        assert_eq!(a, b);
        // Each die is one draw: 2d6 rolls what two 1d6 do.
        let two = dice("2d6").roll(&mut a);
        let (x, y) = (dice("1d6").roll(&mut b), dice("1d6").roll(&mut b));
        assert_eq!(two, x + y);
        assert_eq!(a, b);
    }

    #[test]
    fn prints_its_canonical_text() {
        for (text, printed) in [
            ("2d3+1", "2d3+1"),
            ("1d4-1", "1d4-1"),
            ("-1d2", "-1d2"),
            ("+1d6", "1d6"),
            ("3d6+0", "3d6"),
            ("-2d4+3", "-2d4+3"),
            ("2", "+2"),
            ("-1", "-1"),
            ("-0", "0"),
        ] {
            assert_eq!(dice(text).to_string(), printed, "{text}");
            assert_eq!(dice(printed), dice(text), "{printed} round trip");
        }
        assert!(Dice::fixed(0).is_zero());
        assert!(!dice("1d2-1").is_zero());
    }

    #[test]
    fn reads_numbers_and_text_from_toml_and_json() {
        #[derive(Deserialize)]
        struct Spec {
            stats: BTreeMap<String, Dice>,
        }
        let spec: Spec =
            toml::from_str("stats = { strength = \"1d3+1\", toughness = -2, presence = \"+1\" }")
                .expect("toml");
        assert_eq!(spec.stats["strength"], dice("1d3+1"));
        assert_eq!(spec.stats["toughness"], Dice::fixed(-2));
        assert_eq!(spec.stats["presence"], Dice::fixed(1));
        for (bad, why) in [
            ("stats = { strength = \"1d1\" }", "2 to 1000 sides"),
            ("stats = { strength = 5000 }", "beyond ±1000"),
            ("stats = { strength = 1.5 }", "a number or dice"),
        ] {
            let err = toml::from_str::<Spec>(bad).err().expect(bad);
            assert!(err.to_string().contains(why), "{bad}: {err}");
        }
        let json = serde_json::to_string(&dice("-1d2+1")).expect("json");
        assert_eq!(json, "\"-1d2+1\"");
        assert_eq!(
            serde_json::from_str::<Dice>(&json).ok(),
            Some(dice("-1d2+1"))
        );
        assert_eq!(serde_json::from_str::<Dice>("3").ok(), Some(Dice::fixed(3)));
    }
}
