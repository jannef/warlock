//! Stat training (spec §8 "Stats"; `data/stats.toml`): what trains each stat and how likely a
//! level-up is to raise it. The game says none of these numbers; the wiki states them exactly.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::stats::Stat;

/// Every use that earns a stat a training mark, stat by stat in display order, the uses in
/// file order, with the chance of a mark.
pub(super) fn stat_training(content: &Content) -> String {
    let rules = content.stat_training();
    let mut out = String::from("| Stat | Trained by | Chance of a mark |\n|---|---|---:|\n");
    for stat in Stat::ALL {
        for rule in rules.uses.iter().filter(|r| r.stat == stat) {
            let _ = writeln!(
                out,
                "| {} | {} | {} % |",
                stat.name(),
                rule.on.describe(),
                rule.chance
            );
        }
    }
    out
}

/// The chance a level-up raises a stat, by its score and the marks it holds, below its
/// potential (at or above it the chance is divided by `over_potential_divisor`).
pub(super) fn stat_checks(content: &Content) -> String {
    let rules = content.stat_training();
    let marks = [5u16, 10, 20, 40, 60, 100, 200];
    let mut out = String::from("| Score | Need |");
    for m in marks {
        let _ = write!(out, " {m} marks |");
    }
    out.push_str("\n|---:|---:|");
    out.push_str(&"---:|".repeat(marks.len()));
    out.push('\n');
    for score in [8u8, 10, 12, 14, 16, 18, 20, 25, 30] {
        let _ = write!(out, "| {score} | {} |", rules.need(score));
        for m in marks {
            let (n, d) = rules.odds(m, score, u8::MAX);
            let _ = write!(out, " {:.0} % |", 100.0 * f64::from(n) / f64::from(d));
        }
        out.push('\n');
    }
    out
}
