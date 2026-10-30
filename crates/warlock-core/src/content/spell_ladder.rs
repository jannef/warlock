//! A spell's ladders (`[spell.grows]`; spec §8 "Spells", the spell scaling design
//! `docs/design/spell-scaling.md` §4.3, §6.1): what a spell does as its caster's stats rise,
//! written in its own file as one string a dimension and printed as written in its wiki row.
//!
//! ```toml
//! [spell.grows]
//! damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
//! range  = "willpower 14 11, 18 12"
//! ```
//!
//! A ladder is the stat's id, then rungs separated by commas, each `AT VALUE` with one space:
//! `AT` a score, 1 to [`MAX_SCORE`], strictly ascending; at most [`MAX_RUNGS`] rungs. `VALUE` is a
//! [`Roll`] for `damage` and `heal`, whole cells for `range`, `length`, and `radius`, and whole
//! degrees for `degrees`. The spell's own value is rung 0, the **base**; a caster whose
//! effective score in the stat reaches a rung's `AT` casts that rung ([`Ladder::rung_at`]).
//! Integers only: a comparison and a count. Nothing here, nor anywhere, bounds how much a rung
//! does (the spell scaling design, decision 16); the caller checks a rung's value against its
//! base and the engine's own limits of reach and degrees.

use crate::dice::Roll;
use crate::stats::Stat;
use std::fmt;

/// The most rungs one ladder may have.
pub const MAX_RUNGS: usize = 8;
/// The highest score a rung may be at (the stats' range).
pub const MAX_SCORE: u8 = 40;

/// What a ladder's values are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// A roll (`damage`, `heal`).
    Roll,
    /// Whole cells (`range`, `length`, `radius`).
    Cells,
    /// Whole degrees of a status (`degrees`).
    Degrees,
}

/// One rung's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RungValue {
    Roll(Roll),
    Cells(u8),
    Degrees(u16),
}

impl RungValue {
    /// Its roll, for a `damage` or `heal` ladder.
    pub fn roll(&self) -> Option<Roll> {
        match *self {
            RungValue::Roll(roll) => Some(roll),
            _ => None,
        }
    }

    /// Its whole number, for a ladder of cells or degrees.
    pub fn number(&self) -> Option<i32> {
        match *self {
            RungValue::Cells(n) => Some(i32::from(n)),
            RungValue::Degrees(n) => Some(i32::from(n)),
            RungValue::Roll(_) => None,
        }
    }
}

impl fmt::Display for RungValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RungValue::Roll(roll) => roll.fmt(f),
            RungValue::Cells(n) => n.fmt(f),
            RungValue::Degrees(n) => n.fmt(f),
        }
    }
}

/// A stat and its rungs, `(at, value)` with `at` strictly ascending.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ladder {
    pub stat: Stat,
    pub rungs: Vec<(u8, RungValue)>,
}

impl Ladder {
    /// Reads `text` (module docs), its values in `unit`: the stat known, 1 to [`MAX_RUNGS`]
    /// rungs, scores 1 to [`MAX_SCORE`] strictly ascending, each value of its unit. What a value
    /// may be against the spell's base is the caller's check.
    pub fn parse(text: &str, unit: Unit) -> Result<Self, String> {
        let bad = |why: String| format!("ladder `{text}`: {why}");
        let (stat, rest) = (text.trim())
            .split_once(' ')
            .ok_or_else(|| bad("a stat, then rungs: `intelligence 13 3d6, 16 4d6`".into()))?;
        let stat = Stat::from_id(stat).ok_or_else(|| bad(format!("unknown stat `{stat}`")))?;
        let mut rungs: Vec<(u8, RungValue)> = Vec::new();
        for rung in rest.split(',') {
            let rung = rung.trim();
            let (at, value) = rung
                .split_once(' ')
                .ok_or_else(|| bad(format!("rung `{rung}` is not `AT VALUE`")))?;
            let at: u8 = at
                .parse()
                .ok()
                .filter(|a| (1..=MAX_SCORE).contains(a))
                .ok_or_else(|| bad(format!("a rung's score is 1 to {MAX_SCORE}, not `{at}`")))?;
            if rungs.last().is_some_and(|(before, _)| *before >= at) {
                return Err(bad("the scores rise strictly, rung by rung".into()));
            }
            let value = match unit {
                Unit::Roll => RungValue::Roll(value.parse().map_err(bad)?),
                Unit::Cells => RungValue::Cells(
                    (value.parse())
                        .map_err(|_| bad(format!("`{value}` is not a whole number of cells")))?,
                ),
                Unit::Degrees => RungValue::Degrees(
                    (value.parse())
                        .map_err(|_| bad(format!("`{value}` is not a whole number of degrees")))?,
                ),
            };
            rungs.push((at, value));
        }
        if rungs.len() > MAX_RUNGS {
            return Err(bad(format!("at most {MAX_RUNGS} rungs")));
        }
        Ok(Ladder { stat, rungs })
    }

    /// How many rungs it has: the highest rung that may be cast.
    pub fn len(&self) -> usize {
        self.rungs.len()
    }

    /// Whether it has no rungs (never, once read).
    pub fn is_empty(&self) -> bool {
        self.rungs.is_empty()
    }

    /// The rung a caster of `score` in its stat reaches: how many rungs are at that score or
    /// below (0, the base, below the first).
    pub fn rung_at(&self, score: i32) -> usize {
        (self.rungs.iter())
            .filter(|(at, _)| i32::from(*at) <= score)
            .count()
    }

    /// Rung `rung`'s value; `None` for rung 0, the base (and past its last rung).
    pub fn value(&self, rung: usize) -> Option<&RungValue> {
        rung.checked_sub(1)
            .and_then(|i| self.rungs.get(i))
            .map(|(_, v)| v)
    }

    /// Its values in order, the base first's successors (for the loader's checks).
    pub fn values(&self) -> impl Iterator<Item = &RungValue> {
        self.rungs.iter().map(|(_, v)| v)
    }

    /// In words, as the wiki prints it: `Intelligence 13: 3d6, 16: 4d6`.
    pub fn words(&self) -> String {
        let rungs: Vec<String> = (self.rungs.iter())
            .map(|(at, v)| format!("{at}: {v}"))
            .collect();
        format!("{} {}", self.stat.name(), rungs.join(", "))
    }
}

impl fmt::Display for Ladder {
    /// As data writes it: `intelligence 13 3d6, 16 4d6`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rungs: Vec<String> = (self.rungs.iter())
            .map(|(at, v)| format!("{at} {v}"))
            .collect();
        write!(f, "{} {}", self.stat.id(), rungs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roll(text: &str) -> RungValue {
        RungValue::Roll(text.parse().expect("roll"))
    }

    #[test]
    fn a_ladder_reads_as_written_and_prints_back() {
        let text = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2";
        let ladder = Ladder::parse(text, Unit::Roll).expect("reads");
        assert_eq!(ladder.stat, Stat::Intelligence);
        assert_eq!(ladder.len(), 4);
        assert_eq!(ladder.rungs[3], (25, roll("5d6+2")));
        assert_eq!(ladder.to_string(), text);
        assert_eq!(
            ladder.words(),
            "Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2"
        );
        let cells = Ladder::parse("willpower 14 11, 18 12", Unit::Cells).expect("cells");
        assert_eq!(
            cells.rungs,
            vec![(14, RungValue::Cells(11)), (18, RungValue::Cells(12))]
        );
    }

    #[test]
    fn the_rung_is_the_count_of_rungs_at_or_below_the_score() {
        let ladder = Ladder::parse("intelligence 13 3d6, 16 4d6", Unit::Roll).expect("reads");
        assert_eq!(ladder.rung_at(3), 0);
        assert_eq!(ladder.rung_at(12), 0);
        assert_eq!(ladder.rung_at(13), 1);
        assert_eq!(ladder.rung_at(15), 1);
        assert_eq!(ladder.rung_at(16), 2);
        assert_eq!(ladder.rung_at(40), 2);
        assert_eq!(ladder.value(0), None);
        assert_eq!(ladder.value(2), Some(&roll("4d6")));
        assert_eq!(ladder.value(3), None);
    }

    #[test]
    fn malformed_ladders_are_refused_with_what_is_wrong() {
        let refused = |text: &str, unit: Unit, words: &str| {
            let e = Ladder::parse(text, unit).expect_err(text);
            assert!(e.contains(words), "{text}: {e}");
        };
        refused("cunning 13 3d6", Unit::Roll, "unknown stat `cunning`");
        refused("intelligence 16 4d6, 13 3d6", Unit::Roll, "rise strictly");
        refused("intelligence 13 3d6, 13 4d6", Unit::Roll, "rise strictly");
        refused("intelligence 0 3d6", Unit::Roll, "1 to 40");
        refused("intelligence 41 3d6", Unit::Roll, "1 to 40");
        refused("intelligence 13:3d6", Unit::Roll, "not `AT VALUE`");
        refused("intelligence", Unit::Roll, "a stat, then rungs");
        refused("willpower 14 two", Unit::Cells, "whole number of cells");
        refused("presence 14 2d6", Unit::Degrees, "whole number of degrees");
        refused("intelligence 13 3x6", Unit::Roll, "bad roll");
        let nine: Vec<String> = (1..=9).map(|i| format!("{} {}d6", i * 2, i)).collect();
        refused(
            &format!("intelligence {}", nine.join(", ")),
            Unit::Roll,
            "at most 8 rungs",
        );
    }
}
