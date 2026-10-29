//! Stat training (spec §8 "Stats"; `data/stats.toml`): what trains each stat, what a point
//! costs, how likely a check is to pay it, and how fast; and the creature attacks that drain
//! stats for good. The game says
//! none of these numbers; the wiki states them exactly.

use super::combat::percent;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Affliction, Content, ODDS_SCALE};
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

/// Every deed that costs a stat marks, stat by stat in display order, the deeds in file order,
/// with what it takes; then, by score, the marks a drop pays back and the pile at which one
/// comes within reach (and the game warns of it).
pub(super) fn stat_losses(content: &Content) -> String {
    let rules = content.stat_training();
    let mut out = String::from("| Stat | Lost by | Takes |\n|---|---|---|\n");
    for stat in Stat::ALL {
        for rule in rules.deeds.iter().filter(|r| r.stat == stat) {
            let takes = match (rule.marks, rule.chance) {
                (Some(1), _) => "1 mark".to_string(),
                (Some(marks), _) => format!("{marks} marks"),
                (None, Some(chance)) => format!("a mark, {chance} % of the time"),
                (None, None) => continue,
            };
            let _ = writeln!(
                out,
                "| {} | {} | {takes} |",
                stat.name(),
                rule.on.describe()
            );
        }
    }
    out.push_str("\n| Score | A drop pays back | Within reach (warned) at |\n|---:|---:|---:|\n");
    for score in PRICE_SCORES {
        let price = rules.drop_price(score);
        // The quarter rule: in play once quarter × |marks| reaches the price.
        let reach = (1..=price)
            .find(|&m| rules.in_play(-(m as i32), price))
            .unwrap_or(price);
        let _ = writeln!(out, "| {score} | {price} | −{reach} |");
    }
    out
}

/// The scores [`stat_prices`] lists.
const PRICE_SCORES: [u8; 11] = [6, 8, 10, 12, 14, 16, 18, 20, 25, 30, 35];
/// How many points past the potential [`stat_prices`] prices.
const PAST: u8 = 6;

/// The price in marks of a stat's next point by its score: below the potential, then the 1st to
/// 6th point past it, then the marks a drop from that score pays back.
pub(super) fn stat_prices(content: &Content) -> String {
    let rules = content.stat_training();
    let mut out = String::from("| Score | Below the potential |");
    for over in 1..=PAST {
        let _ = write!(out, " {} past |", ordinal(over));
    }
    out.push_str(" A drop |\n|---:|---:|");
    out.push_str(&"---:|".repeat(usize::from(PAST) + 1));
    out.push('\n');
    for score in PRICE_SCORES {
        let _ = write!(out, "| {score} | {} |", rules.base_price(score));
        for over in 1..=PAST {
            // The `over`-th point past the potential: the potential `over` - 1 below the score.
            let potential = (score + 1).saturating_sub(over).max(1);
            let _ = write!(out, " {} |", rules.price(score, potential));
        }
        let _ = writeln!(out, " {} |", rules.drop_price(score));
    }
    out
}

/// `1st`, `2nd`, `3rd`, `4th`, ...
fn ordinal(n: u8) -> String {
    let suffix = match n {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// The marks held at a check, as shares of the price ([`stat_odds`]), in hundredths.
const SHARES: [(u32, &str); 11] = [
    (24, "under a quarter"),
    (25, "a quarter"),
    (50, "half"),
    (75, "three quarters"),
    (100, "the price"),
    (125, "1¼ ×"),
    (150, "1½ ×"),
    (200, "2 ×"),
    (300, "3 ×"),
    (500, "5 ×"),
    (800, "8 × or more"),
];

/// The chance a check moves a stat by the marks it holds against the price (a rise's, or a
/// drop's for a negative pile): the S-curve with the quarter rule.
pub(super) fn stat_odds(content: &Content) -> String {
    let rules = content.stat_training();
    let price = 10_000;
    let mut out = String::from("| Marks held | Chance at a check |\n|---|---:|\n");
    for (share, name) in SHARES {
        let odds = rules.odds(price * share / 100, price);
        let _ = writeln!(out, "| {name} | {} |", odds_percent(odds));
    }
    out
}

/// A chance out of [`ODDS_SCALE`] in words of percent: `0 %`, `under 1 %`, `11 %`, `99.8 %`.
fn odds_percent(odds: u32) -> String {
    let p = 100.0 * f64::from(odds) / f64::from(ODDS_SCALE);
    match p {
        0.0 => "0 %".to_string(),
        p if p < 1.0 => "under 1 %".to_string(),
        p if p > 99.0 => format!("{p:.1} %"),
        p => format!("{p:.0} %"),
    }
}

/// The marks a day [`stat_pace`] lists.
const MARKS_A_DAY: [u32; 10] = [1, 2, 4, 8, 15, 25, 40, 80, 160, 320];

/// The expected game days to a stat's next point by the marks it earns a day, at the daily
/// check alone (a level's checks come on top), for a few prices: worked out from the odds, the
/// pile growing by the same marks every day from nothing.
pub(super) fn stat_pace(content: &Content) -> String {
    let rules = content.stat_training();
    // (score, potential): below it at 10, 14, and 20; at 14 the 2nd point past a potential of 13.
    let cases = [(10u8, 20u8), (14, 20), (20, 25), (14, 13)];
    let mut out = String::from("| Marks a day |");
    for (score, potential) in cases {
        let price = rules.price(score, potential);
        let past = match (score + 1).saturating_sub(potential) {
            0 => String::new(),
            over => format!(", {} point past the potential", ordinal(over)),
        };
        let _ = write!(out, " Score {score}{past} (price {price}) |");
    }
    out.push_str("\n|---:|");
    out.push_str(&"---:|".repeat(cases.len()));
    out.push('\n');
    for a_day in MARKS_A_DAY {
        let _ = write!(out, "| {a_day} |");
        for (score, potential) in cases {
            let days = expected_days(rules, a_day, rules.price(score, potential));
            let _ = write!(out, " {} |", days_text(days));
        }
        out.push('\n');
    }
    out
}

/// The expected days until a pile that grows by `a_day` marks a day from nothing wins a daily
/// check against `price`.
fn expected_days(rules: &warlock_core::content::StatTraining, a_day: u32, price: u32) -> f64 {
    let (mut none_yet, mut expected) = (1.0f64, 0.0f64);
    for day in 1..=100_000u32 {
        let p = f64::from(rules.odds(a_day.saturating_mul(day), price)) / f64::from(ODDS_SCALE);
        expected += f64::from(day) * none_yet * p;
        none_yet *= 1.0 - p;
        if none_yet < 1e-9 {
            break;
        }
    }
    expected
}

/// Days to one decimal, whole days from 10.
fn days_text(days: f64) -> String {
    match days {
        d if d < 9.95 => format!("{d:.1}"),
        d => format!("{d:.0}"),
    }
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
