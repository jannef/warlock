//! Working an item at an anvil (spec §7 "Special rooms"): the odds of an attempt, computed from
//! `data/temper.toml` and the 2d12 rules, and the grades of a worked item with their worth, from
//! `magic/worked.rs`.

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::combat::{FUMBLE, HIGHEST};
use warlock_core::content::Content;
use warlock_core::magic::worked::{DAMAGED, GRADES, SIDES_PER_DIE};

/// The modifiers the odds table has a row for.
const MODIFIERS: [i32; 4] = [0, 4, 8, 12];
/// The attempts already made the odds table has a column for.
const ATTEMPTS: std::ops::Range<u8> = 0..6;

/// The chance of a check of `modifier` against `against`, in percent: a natural 2 always fails, a
/// natural 24 always succeeds.
fn chance(modifier: i32, against: i32) -> f64 {
    let made: i32 = (1..=HIGHEST / 2)
        .flat_map(|a| (1..=HIGHEST / 2).map(move |b| a + b))
        .filter(|&n| n == HIGHEST || (n != FUMBLE && n + modifier >= against))
        .map(|_| 1)
        .sum();
    f64::from(made) * 100.0 / f64::from((HIGHEST / 2) * (HIGHEST / 2))
}

/// An attempt's odds by the check's modifier and the attempts already made on the item, then the
/// numbers behind them.
pub(super) fn temper_odds(content: &Content) -> String {
    let rules = content.temper();
    let mut out = String::from("| Modifier |");
    for n in ATTEMPTS {
        let _ = write!(out, " Attempt {} |", n + 1);
    }
    out.push_str("\n|:---:|");
    out.push_str(&":---:|".repeat(ATTEMPTS.len()));
    out.push('\n');
    for modifier in MODIFIERS {
        let _ = write!(out, "| {modifier:+} |");
        for n in ATTEMPTS {
            let _ = write!(out, " {:.0}% |", chance(modifier, rules.against(n)));
        }
        out.push('\n');
    }
    let _ = write!(
        out,
        "\nAn attempt takes {} seconds ({} minutes). Its check is 2d12 plus your Strength \
         modifier, what Metalcraft gives, and what talents add to Strength checks, against {} \
         plus {} for every attempt ever made on the item. A natural {FUMBLE} always fails, a \
         natural {HIGHEST} always succeeds.\n",
        rules.seconds,
        rules.seconds / 60,
        rules.difficulty,
        rules.per_attempt,
    );
    out
}

/// What a worked item is called and what it is worth, then what an improvement may better.
pub(super) fn temper_grades(_: &Content) -> String {
    let mut out = String::from(
        "| Improvements | Called | Worth |\n\
         |:---:|---|:---:|\n",
    );
    for (i, (word, times)) in GRADES.iter().enumerate() {
        let _ = writeln!(out, "| {} | {word} | ×{times} |", i + 1);
    }
    let _ = writeln!(out, "| none | {} | ×½, rounded up |", capital(DAMAGED));
    let _ = write!(
        out,
        "\n| Betters | Which items | Each time |\n\
         |---|---|---|\n\
         | Damage dice | Anything with an attack | {SIDES_PER_DIE} more sides on its dice (1d8 \
         becomes 1d{}, 2d6 becomes 2d{}) |\n\
         | To hit | Anything with an attack | +1 |\n\
         | Damage | Anything with an attack | +1 |\n\
         | Defence | Armour, shields, and parrying weapons | +1, of the kind it gives |\n\
         | Protection | Anything that gives Protection | +1 |\n",
        8 + SIDES_PER_DIE,
        6 + SIDES_PER_DIE,
    );
    out
}
