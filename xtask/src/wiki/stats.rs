//! Stat training (spec §8 "Stats"; `data/stats.toml`): what trains each stat and how likely a
//! level-up is to raise it; and the creature attacks that drain stats for good. The game says
//! none of these numbers; the wiki states them exactly.

use super::combat::percent;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Affliction, Content};
use warlock_core::stats::{self, DRAIN_FLOOR, Stat};

/// Every use that earns a stat a training mark, stat by stat in display order, the uses in
/// file order, with the chance of a mark.
pub(super) fn stat_training(content: &Content) -> String {
    let rules = content.stat_training();
    let mut out = String::from("| Stat | Trained by | Chance of a mark |\n|---|---|---:|\n");
    // Stat by stat, then the uses that name their own stat (a potion's).
    let stats = Stat::ALL.into_iter().map(Some).chain([None]);
    for stat in stats {
        for rule in rules.uses.iter().filter(|r| r.stat == stat) {
            let _ = writeln!(
                out,
                "| {} | {} | {} % |",
                stat.map_or("Any", Stat::name),
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

/// The scores [`drain_attacks`] gives the odds of resisting for.
const RESIST_SCORES: [u8; 6] = [6, 8, 10, 12, 14, 16];

/// A drain's stat in words: `Strength`, or `a random stat`.
pub(super) fn drained_stat(stat: Option<Stat>) -> String {
    stat.map_or_else(|| "a random stat".to_string(), |s| s.name().to_string())
}

/// Every creature attack that drains (spec §8 "Stats": "Drain"): the stat and the points it
/// takes, the chance a hit tries, the check that resists it and how often it is held by score,
/// then the floor no drain goes below.
pub(super) fn drain_attacks(content: &Content) -> String {
    let scores: String = (RESIST_SCORES.iter())
        .map(|s| format!(" Held at {s} |"))
        .collect();
    let mut out = format!(
        "| Creature | Attack | Drains | Points | Chance a hit tries | Resisted with | Difficulty \
         |{scores}\n|---|---|---|:---:|:---:|---|:---:|{}\n",
        ":---:|".repeat(RESIST_SCORES.len())
    );
    for (_, c) in super::order::creatures(content) {
        let Some(attack) = &c.attack else {
            continue;
        };
        for hit in &attack.on_hit {
            let Affliction::Drain(drain) = hit.what else {
                continue;
            };
            let odds: String = (RESIST_SCORES.iter())
                .map(|&s| {
                    let modifier = stats::modifier(i32::from(s));
                    let ways = check::ways(modifier, drain.difficulty);
                    format!(" {} |", percent(ways, 144))
                })
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} % | {} | {} |{odds}",
                capital(&c.name),
                attack.name,
                drained_stat(drain.stat),
                drain.points,
                hit.chance,
                drain.check.name(),
                drain.difficulty
            );
        }
    }
    let _ = writeln!(
        out,
        "\nNo drain takes a score below {DRAIN_FLOOR}, nor below one more than the stat damage \
         on it."
    );
    out
}
