//! The starters (#1200, `docs/balance.md` "Starters"): the trained player of level 1 and of
//! level 5 in plain gear against the monsters of each level from 1 to 5 alone, not a window: the
//! median rounds to kill and to die, and how many of those monsters win (the player dies first
//! by the means), against one and against two at once.

use super::{Gear, Probe, Row, median, per_monster, r};
use crate::rng::Rng;
use std::fmt::Write as _;

/// The classes measured.
const CLASSES: [&str; 4] = ["warrior", "thief", "wizard", "priest"];

/// The player levels measured (levels the probe snaps).
const PLAYER_LEVELS: [u8; 2] = [1, 5];

/// `kill / die · lose % / lose to two %` over `rows`: one foe wins when the rounds to die are
/// fewer than the rounds to kill; two at once (killed one after the other, both striking until
/// the first falls) when they are fewer than three times the rounds to kill.
fn cell(rows: &[Row]) -> String {
    let kill = median(rows.iter().map(|r| r.4).collect());
    let die = median(rows.iter().map(|r| r.5).collect());
    let share = |lost: &dyn Fn(&Row) -> bool| {
        100.0 * rows.iter().filter(|r| lost(r)).count() as f64 / rows.len().max(1) as f64
    };
    let one = share(&|r| r.5 < r.4);
    let two = share(&|r| r.5 < 3.0 * r.4);
    format!("{} / {} · {one:.0} % / {two:.0} %", r(kill), r(die))
}

pub(super) fn starters(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Starters\n").unwrap();
    writeln!(
        out,
        "The trained player in plain gear against the monsters of one level (not a window). Each cell: rounds to kill / rounds to die (the medians) · the share of those monsters that win alone / two at once (by the means).\n"
    )
    .unwrap();
    writeln!(
        out,
        "| Player L | Monster L | n | Warrior | Thief | Wizard (melee) | Priest |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|").unwrap();
    for level in PLAYER_LEVELS {
        for monster in 1..=5u8 {
            let w: Vec<_> = (p.monsters.iter()).filter(|m| m.level == monster).collect();
            let mut cells = Vec::new();
            for class in CLASSES {
                let snaps = p.of(class, level, Gear::Plain, true);
                let sides: Vec<_> = snaps.iter().map(|s| &s.side).collect();
                cells.push(cell(&per_monster(&sides, &w, None, rng)));
            }
            let n = w.len();
            writeln!(out, "| {level} | {monster} | {n} | {} |", cells.join(" | ")).unwrap();
        }
    }
}
