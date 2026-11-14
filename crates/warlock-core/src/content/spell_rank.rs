//! A spell's ranks (spec §8 "Spells": "Ranks"; #1264): what a known spell gains as its caster
//! deepens it, rank 1 (the spell as learned) to [`MAX_SPELL_RANK`], written in the spell's own
//! file as `[spell.ranks]`, one entry a rank with what reaching that rank **adds** (the
//! `spells/README.toml`). The entries are cumulative: at rank N a cast has everything the
//! entries for ranks 2 to N add, as one [`SpellMods`] ([`Ranks::at`]). A spell with no table
//! gains nothing from its ranks. Only an own cast reads them (`world/magic/spell_rank.rs`): a scroll
//! or a wand casts at rank 1.
//!
//! An entry is checked as it loads, like a ladder: only what the spell could read (a roll's
//! fields on a spell that rolls, reach on a spell with one, a radius on a burst, degrees on a
//! status or a summon, to-hit and threat on a spell that rolls to hit), and never empty. Nothing
//! bounds how much a rank adds (the spell scaling design, decision 16).

use super::SpellMods;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The highest rank a spell is known at.
pub const MAX_SPELL_RANK: u8 = 10;

/// One rank's entry as a spell file writes it: each field what reaching the rank adds (the
/// README), every one optional.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct RankSpec {
    /// Faces rerolled: a die showing this or less is drawn once more.
    pub reroll: u32,
    /// Every die this many sides bigger.
    pub sides: u32,
    /// This many dice more.
    pub dice: u32,
    /// Added to the roll.
    pub bonus: i32,
    /// Cells farther on a range or a length.
    pub cells: i32,
    /// Cells wider on a burst's radius.
    pub radius: i32,
    /// Degrees longer a buff or a hex, minutes longer a summon.
    pub degrees: i32,
    /// Percent off the mana.
    pub mana_off: i32,
    /// Percent off the casting time.
    pub time_off: i32,
    /// Added to the attack roll.
    pub to_hit: i32,
    /// The threat range this much wider: a natural roll this far below 22 threatens a critical.
    pub threat: i32,
}

impl RankSpec {
    /// Whether it adds anything.
    fn is_empty(&self) -> bool {
        *self == RankSpec::default()
    }

    /// What it adds, as the hooks read it.
    fn mods(&self) -> SpellMods {
        SpellMods {
            reroll: self.reroll,
            sides: self.sides,
            dice: self.dice,
            bonus: self.bonus,
            cells: self.cells,
            radius_cells: self.radius,
            degrees: self.degrees,
            cost: -self.mana_off,
            time: -self.time_off,
            to_hit: self.to_hit,
            threat: self.threat,
            ..SpellMods::default()
        }
    }
}

/// Which dimensions a spell has, what its rank entries are checked against (its ladders' bases
/// and whether it rolls to hit).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RankBases {
    /// It rolls something: harm, healing, or a summon's count.
    pub(super) roll: bool,
    /// It has a range or a length.
    pub(super) reach: bool,
    /// It is a burst.
    pub(super) radius: bool,
    /// It lays a status or calls for minutes.
    pub(super) degrees: bool,
    /// It rolls to hit.
    pub(super) to_hit: bool,
}

/// A spell's rank table: what each rank from 2 adds, in rank order. Empty for a spell whose
/// ranks give nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ranks {
    steps: Vec<(u8, SpellMods)>,
}

impl Ranks {
    /// Whether no rank adds anything.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Each rank that adds something, with what reaching it adds, rank 2 first.
    pub fn steps(&self) -> &[(u8, SpellMods)] {
        &self.steps
    }

    /// Everything a cast at `rank` has gained: the entries for ranks 2 to `rank`, summed.
    pub fn at(&self, rank: u8) -> SpellMods {
        let mut sum = SpellMods::default();
        for (_, mods) in self.steps.iter().filter(|(r, _)| *r <= rank) {
            sum.add(mods);
        }
        sum
    }

    /// `[spell.ranks]` as written (`"2" = { ... }`), checked against `bases` (module docs).
    pub(super) fn parse(
        table: BTreeMap<String, RankSpec>,
        bases: RankBases,
    ) -> Result<Self, String> {
        let mut steps = Vec::with_capacity(table.len());
        for (key, spec) in table {
            let rank: u8 = (key.parse().ok())
                .filter(|r| (2..=MAX_SPELL_RANK).contains(r))
                .ok_or_else(|| {
                    format!("ranks: `{key}` is no rank: ranks are 2 to {MAX_SPELL_RANK}")
                })?;
            if spec.is_empty() {
                return Err(format!("ranks: rank {rank} adds nothing"));
            }
            let needs = |given: bool, has: bool, what: &str, why: &str| {
                if given && !has {
                    Err(format!("ranks: rank {rank}: `{what}` is for {why}"))
                } else {
                    Ok(())
                }
            };
            let rolls = spec.reroll > 0 || spec.sides > 0 || spec.dice > 0 || spec.bonus != 0;
            needs(
                rolls,
                bases.roll,
                "reroll, sides, dice, bonus",
                "a spell that rolls (a `damage`, a `heal`, or a `summon`)",
            )?;
            needs(
                spec.cells != 0,
                bases.reach,
                "cells",
                "a spell with a `range` or a `length`",
            )?;
            needs(spec.radius != 0, bases.radius, "radius", "a `burst`")?;
            needs(
                spec.degrees != 0,
                bases.degrees,
                "degrees",
                "a spell with a status or a summon's minutes",
            )?;
            needs(
                spec.to_hit != 0 || spec.threat != 0,
                bases.to_hit,
                "to_hit, threat",
                "a spell that rolls to hit (an `attack` or `touch` damage spell)",
            )?;
            steps.push((rank, spec.mods()));
        }
        steps.sort_by_key(|(rank, _)| *rank);
        Ok(Ranks { steps })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(text: &str) -> BTreeMap<String, RankSpec> {
        toml::from_str(text).expect("a table")
    }

    const ALL: RankBases = RankBases {
        roll: true,
        reach: true,
        radius: true,
        degrees: true,
        to_hit: true,
    };

    #[test]
    fn ranks_are_cumulative_and_in_order() {
        let ranks = Ranks::parse(
            table("5 = { cells = 1 }\n2 = { bonus = 1 }\n9 = { dice = 1, mana_off = 10 }\n"),
            ALL,
        )
        .expect("parses");
        let steps: Vec<u8> = ranks.steps().iter().map(|(r, _)| *r).collect();
        assert_eq!(steps, vec![2, 5, 9]);
        assert_eq!(ranks.at(1), SpellMods::default());
        assert_eq!(ranks.at(4).bonus, 1);
        assert_eq!(ranks.at(4).cells, 0);
        let nine = ranks.at(9);
        assert_eq!(
            (nine.bonus, nine.cells, nine.dice, nine.cost),
            (1, 1, 1, -10)
        );
        assert_eq!(ranks.at(10), nine);
    }

    #[test]
    fn a_rank_outside_two_to_ten_or_empty_is_refused() {
        for (text, why) in [
            ("1 = { bonus = 1 }", "no rank"),
            ("11 = { bonus = 1 }", "no rank"),
            ("two = { bonus = 1 }", "no rank"),
            ("3 = { }", "adds nothing"),
        ] {
            let err = Ranks::parse(table(text), ALL).expect_err(text);
            assert!(err.contains(why), "{text}: {err}");
        }
    }

    #[test]
    fn a_field_the_spell_could_never_read_is_refused() {
        let none = RankBases::default();
        for (text, what) in [
            ("2 = { bonus = 1 }", "bonus"),
            ("2 = { cells = 1 }", "cells"),
            ("2 = { radius = 1 }", "radius"),
            ("2 = { degrees = 1 }", "degrees"),
            ("2 = { to_hit = 1 }", "to_hit"),
            ("2 = { threat = 1 }", "threat"),
        ] {
            let err = Ranks::parse(table(text), none).expect_err(text);
            assert!(err.contains(what), "{text}: {err}");
        }
        // Mana and time are every spell's.
        let ranks = Ranks::parse(table("2 = { mana_off = 10, time_off = 25 }"), none);
        let mods = ranks.expect("parses").at(2);
        assert_eq!((mods.cost, mods.time), (-10, -25));
    }
}
