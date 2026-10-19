//! Doors (spec §8 "Doors"): kicking them in, and the keys.

use super::text::{capital, glyph};
use std::fmt::Write as _;
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

/// The keys and the locks they open (spec §8 "Doors").
pub(super) fn keys(content: &Content) -> String {
    let mut out = String::from("| Key | Glyph | Opens |\n|---|:---:|---|\n");
    for o in content.object_defs() {
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
