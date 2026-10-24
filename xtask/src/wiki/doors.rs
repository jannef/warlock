//! Doors (spec §8 "Doors"): kicking them in, the keys, and picking locks.

use super::combat::percent;
use super::order;
use super::text::{capital, glyph};
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::Content;
use warlock_core::door;

/// The Strength scores the door table shows, and the last column's "and up".
const KICK_STRENGTHS: [u8; 8] = [3, 6, 9, 10, 12, 15, 18, 20];

/// How often a kick breaks each door that can break, by the kicker's Strength score
/// (spec §8 "Doors").
pub(super) fn doors(content: &Content) -> String {
    let mut out = String::from("| Door | Sturdiness |");
    let mut rule = String::from("|---|:---:|");
    for st in KICK_STRENGTHS {
        let _ = write!(out, " Strength {st} |");
        rule.push_str(":---:|");
    }
    let _ = writeln!(out, "\n{rule}");
    for t in content.terrain_defs() {
        let Some((_, sturdiness)) = t.kicked_in else {
            continue;
        };
        let _ = write!(out, "| {} | {sturdiness} |", capital(&t.name));
        for st in KICK_STRENGTHS {
            let _ = write!(out, " {}% |", door::break_chance(st, sturdiness));
        }
        out.push('\n');
    }
    out
}

/// What the picker adds to the check the lock-picking table shows: their Dexterity modifier plus
/// what Open lock gives.
const PICK_BONUSES: [i32; 7] = [0, 2, 4, 6, 8, 10, 12];

/// How often a try picks a lock, by its difficulty and what the picker adds (spec §8 "Doors"): a
/// lock of the usual difficulty, then the usual lock deeper in each branch whose locks harden.
pub(super) fn lock_picking(content: &Content) -> String {
    let mut out = String::from("| Lock | Difficulty |");
    let mut rule = String::from("|---|:---:|");
    for plus in PICK_BONUSES {
        let _ = write!(out, " +{plus} |");
        rule.push_str(":---:|");
    }
    let _ = writeln!(out, "\n{rule}");
    let usual = i32::from(door::LOCK_DIFFICULTY);
    let mut rows = vec![("A lock of the usual difficulty".to_string(), usual)];
    let mut branches: Vec<_> = content.dungeons().collect();
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    for def in branches.into_iter().filter(|d| d.locks_per_depth > 0) {
        for depth in 2..=def.depth {
            let place = def.level_name(depth);
            rows.push((format!("Usual, on {place}"), usual + def.locks_at(depth)));
        }
    }
    for (lock, difficulty) in rows {
        let _ = write!(out, "| {lock} | {difficulty} |");
        for plus in PICK_BONUSES {
            let _ = write!(out, " {} |", percent(check::ways(plus, difficulty), 144));
        }
        out.push('\n');
    }
    out
}

/// The keys and the locks they open (spec §8 "Doors").
pub(super) fn keys(content: &Content) -> String {
    let mut out = String::from("| Key | Glyph | Opens |\n|---|:---:|---|\n");
    for (_, o) in order::objects(content) {
        let opens = match &o.key {
            Some(door::Key::Skeleton) => "every lock".to_string(),
            Some(door::Key::Lock(id)) => format!("locks named `{id}`"),
            None => continue,
        };
        let _ = writeln!(
            out,
            "| {} | {} | {opens} |",
            capital(&o.name),
            glyph(o.glyph)
        );
    }
    out
}
