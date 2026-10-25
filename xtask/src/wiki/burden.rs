//! Burden (spec §8 "Burden"): its levels, the races' sizes, and sample carrying limits.

use super::actions::at_normal_pace;
use std::fmt::Write as _;
use warlock_core::burden::{self, Burden, Load};
use warlock_core::content::{BASE_TURN, Content, price};
use warlock_core::object::format_weight;

/// What each level of burden costs (spec §8 "Burden"), from the rules' own numbers.
pub(super) fn burden_levels(_content: &Content) -> String {
    let mut out = String::from(
        "| Burden | You carry | Every action takes | Dexterity adds to Defence | Your steps |\n\
         |---|---|---|---|---|\n",
    );
    for b in Burden::ALL {
        let carried = match b {
            Burden::Unburdened => "up to a third of your limit",
            Burden::Burdened => "up to two thirds",
            Burden::Strained => "up to your limit",
            Burden::Overloaded => "more than your limit",
        };
        // How long a normal action takes, from its price: 6 s at a normal pace.
        let normal = price(warlock_core::content::NORMAL_COST, BASE_TURN);
        let takes = match (b.can_move(), at_normal_pace(b.slow(normal))) {
            (true, t) if b == Burden::Unburdened => format!("{t} (as usual)"),
            (true, t) => format!("{t} (×{})", factor(b.slow(100))),
            (false, t) => format!("{t} (×{}); you cannot move", factor(b.slow(100))),
        };
        let dex = b
            .max_dex()
            .map_or("all of it".to_string(), |cap| match cap {
                0 => "nothing".to_string(),
                cap => format!("at most +{cap}"),
            });
        let steps = match b.footsteps() {
            100 => "as loud as ever".to_string(),
            _ if !b.can_move() => "-".to_string(),
            percent => format!("×{} as loud", factor(i64::from(percent))),
        };
        let _ = writeln!(
            out,
            "| {} | {carried} | {takes} | {dex} | {steps} |",
            b.name()
        );
    }
    out
}

/// `percent` as a factor: `1`, `0.75`, `1.25`, `1.5`.
pub(super) fn factor(percent: i64) -> String {
    let text = format!("{}.{:02}", percent / 100, percent % 100);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Each race's size and what it carries next to a medium race, and whether it shrugs off
/// Burdened.
pub(super) fn burden_races(content: &Content) -> String {
    let mut out = String::from("| Race | Size | Carries | When burdened |\n|---|---|:---:|---|\n");
    for r in &content.races().list {
        let size = content.size(r.size);
        let burdened = if r.ignores_burdened {
            "no penalty at all"
        } else {
            "slowed, louder, less nimble"
        };
        let _ = writeln!(
            out,
            "| {} | {} | ×{} | {burdened} |",
            r.name,
            size.name,
            factor(i64::from(size.carry))
        );
    }
    out
}

/// Carrying limits of a few characters: the race's size with the given Strength and Toughness.
pub(super) fn burden_examples(content: &Content) -> String {
    let mut out = String::from(
        "| Character | Limit | Unburdened up to | Burdened up to | Strained up to |\n\
         |---|---:|---:|---:|---:|\n",
    );
    for (race, strength, toughness) in [
        ("human", 10, 10),
        ("half_orc", 16, 14),
        ("dwarf", 14, 16),
        ("halfling", 8, 12),
        ("elf", 6, 8),
    ] {
        let Some(r) = content.race(race) else {
            continue;
        };
        let load = Load {
            carried: 0,
            limit: burden::limit(strength, toughness, content.size(r.size).carry),
            ignores_burdened: r.ignores_burdened,
            stout: false,
        };
        let [burdened, strained, overloaded] = load.thresholds().map(|(_, g)| format_weight(g));
        let _ = writeln!(
            out,
            "| {}, St {strength} To {toughness} | {overloaded} | {burdened} | {strained} | \
             {overloaded} |",
            r.name
        );
    }
    out
}
