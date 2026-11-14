//! Crafting (docs/dev/crafting.md; docs/design/professions.md): which skill works each material
//! group (`data/stock.toml`), how hard each material is to work (`work` in `materials/`), and the
//! odds and numbers of a salvage (`data/salvage.toml` and the 2d12 rules).

use super::util::capital;
use std::fmt::Write as _;
use warlock_core::combat::{FUMBLE, HIGHEST};
use warlock_core::content::Content;

/// The check modifiers the salvage odds table has a row for.
const MODIFIERS: [i32; 5] = [0, 4, 8, 12, 16];
/// The materials' `work` the salvage odds table has a column for.
const WORKS: [i32; 6] = [-3, 0, 2, 4, 6, 8];

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

/// Each material group's stock word and the skill and stat that work it, then the gems, by
/// group name.
pub(super) fn crafting_groups(content: &Content) -> String {
    let stock = content.stock();
    let mut out = String::from(
        "| Group | Stock | Worked by | Stat |\n\
         |---|---|---|---|\n",
    );
    let mut groups: Vec<_> = stock.groups.iter().collect();
    groups.sort_by(|a, b| a.group.cmp(&b.group));
    let worked = |p: Option<&warlock_core::content::Profession>| match p {
        Some(p) => (content.skill(p.skill).name.clone(), capital(p.stat.name())),
        None => ("nothing yet".to_string(), "-".to_string()),
    };
    for g in groups {
        let (skill, stat) = worked(g.profession.as_ref());
        let _ = writeln!(
            out,
            "| {} | {} | {skill} | {stat} |",
            capital(&g.group),
            g.plural
        );
    }
    let (skill, stat) = worked(stock.gems.as_ref());
    let _ = writeln!(out, "| Gems | loose gems | {skill} | {stat} |");
    out
}

/// Every material by how hard it is to work (`work`), easiest first, each row the materials of
/// that difficulty by name.
pub(super) fn crafting_work(content: &Content) -> String {
    let mut by: Vec<(i32, String)> = (content.materials().iter())
        .filter(|m| m.group.is_some() || m.gem.is_some())
        .map(|m| (m.work, m.name.clone()))
        .collect();
    by.sort();
    let mut out = String::from(
        "| Work | Materials |\n\
         |:---:|---|\n",
    );
    let mut works: Vec<i32> = by.iter().map(|(w, _)| *w).collect();
    works.dedup();
    for work in works {
        let names: Vec<&str> = (by.iter())
            .filter(|(w, _)| *w == work)
            .map(|(_, n)| n.as_str())
            .collect();
        let _ = writeln!(out, "| {work:+} | {} |", names.join(", "));
    }
    out
}

/// A salvage check's odds by its modifier and the material's `work`, then the numbers behind
/// them.
pub(super) fn salvage(content: &Content) -> String {
    let Some(rules) = content.salvage() else {
        return "Nobody salvages.\n".to_string();
    };
    let mut out = String::from("| Modifier |");
    for work in WORKS {
        let _ = write!(out, " Work {work:+} |");
    }
    out.push_str("\n|:---:|");
    out.push_str(&":---:|".repeat(WORKS.len()));
    out.push('\n');
    for modifier in MODIFIERS {
        let _ = write!(out, "| {modifier:+} |");
        for work in WORKS {
            let _ = write!(out, " {:.0}% |", chance(modifier, rules.difficulty + work));
        }
        out.push('\n');
    }
    let _ = write!(
        out,
        "\nSalvaging takes {} seconds ({} minutes). Each material's check is 2d12 plus the \
         modifier of the stat its skill works with, what talents add to checks of that stat, and \
         what that one skill gives to its craft, against {} plus the material's work. A natural \
         {FUMBLE} always fails, a natural {HIGHEST} always succeeds. A material salvaged gives back \
         {}% of its weight as stock, at least one piece.\n",
        rules.seconds,
        rules.seconds / 60,
        rules.difficulty,
        rules.percent,
    );
    out
}
