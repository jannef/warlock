//! The spread (#1134, the dread design §7.3, `docs/balance.md` "Spread"): per level, the rounds
//! the trained player in plain gear takes to kill each monster within two levels, and to die to
//! it, as a distribution over the monsters: the median, the fastest tenth, and the slowest tenth.
//! The variance pass's target: the fastest tenth at half the median, the slowest at twice.

use super::{Gear, LEVELS, Probe, Row, median, per_monster, r};
use crate::rng::Rng;
use std::fmt::Write as _;

/// The classes measured: rounds to kill for the first, rounds to die for every one.
const CLASSES: [&str; 4] = ["warrior", "thief", "wizard", "priest"];

/// The `q` quantile of `v` (0 to 1), linear between the nearest ranks.
fn quantile(v: &[f64], q: f64) -> f64 {
    let mut v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let at = q * (v.len() - 1) as f64;
    let (lo, hi) = (at.floor() as usize, at.ceil() as usize);
    v[lo] + (v[hi] - v[lo]) * (at - lo as f64)
}

/// `median (fastest tenth–slowest tenth) ×low/×high`.
fn cell(v: &[f64]) -> String {
    let mid = median(v.to_vec());
    let (lo, hi) = (quantile(v, 0.1), quantile(v, 0.9));
    format!(
        "{} ({}–{}) ×{:.2}/×{:.1}",
        r(mid),
        r(lo),
        r(hi),
        lo / mid,
        hi / mid
    )
}

pub(super) fn spread(p: &Probe, rng: &mut Rng, out: &mut String) {
    writeln!(out, "\n## Spread\n").unwrap();
    writeln!(
        out,
        "Rounds to kill and to die over the monsters within two levels, trained stats in plain gear: the median (fastest tenth–slowest tenth) ×the tenths over the median. The target: ×0.5 and ×2.\n"
    )
    .unwrap();
    writeln!(
        out,
        "| L | n | Warrior kills | Warrior dies | Thief dies | Wizard dies | Priest dies |"
    )
    .unwrap();
    writeln!(out, "|---|---|---|---|---|---|---|").unwrap();
    for &level in &LEVELS {
        let w = p.window(level, -2, 2);
        let mut cells = Vec::new();
        for (i, class) in CLASSES.into_iter().enumerate() {
            let snaps = p.of(class, level, Gear::Plain, true);
            let sides: Vec<_> = snaps.iter().map(|s| &s.side).collect();
            let rows: Vec<Row> = per_monster(&sides, &w, None, rng);
            if i == 0 {
                cells.push(cell(&rows.iter().map(|r| r.4).collect::<Vec<_>>()));
            }
            cells.push(cell(&rows.iter().map(|r| r.5).collect::<Vec<_>>()));
        }
        writeln!(out, "| {level} | {} | {} |", w.len(), cells.join(" | ")).unwrap();
    }
}
