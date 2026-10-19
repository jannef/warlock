//! Special rooms (spec §7 "Special rooms").

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::Content;

/// How many times as much `percent` is, in words: "twice", "two fifths", "30% of".
fn times(percent: u16) -> String {
    match percent {
        200 => "twice".into(),
        300 => "three times".into(),
        50 => "half".into(),
        40 => "two fifths".into(),
        25 => "a quarter".into(),
        p => format!("{p}% of"),
    }
}

/// The special rooms (spec §7 "Special rooms"): what each is called, what it says the first time
/// you enter it, and what it does inside, in words.
pub(super) fn special_rooms(content: &Content) -> String {
    let mut out = String::from("| Room | The first time you enter | Inside |\n|---|---|---|\n");
    for e in content.room_effects().defs() {
        let mut does: Vec<String> = Vec::new();
        if e.light < 100 {
            does.push(format!(
                "All light is drained to {}% of what it would be.",
                e.light
            ));
        }
        if !e.glow.is_dark() {
            let faint = if e.glow.luma() < 80 { "faint " } else { "" };
            let warm = if e.glow.rgb[0] > e.glow.rgb[2] {
                "warm "
            } else {
                ""
            };
            does.push(format!("A {faint}{warm}glow lights it."));
        }
        if e.noise != 100 {
            let farther = if e.noise > 100 {
                ", and so carries farther"
            } else {
                ""
            };
            does.push(format!(
                "Every noise made in it is {} as loud{farther}.",
                times(e.noise)
            ));
        }
        if e.regen != 100 {
            does.push(format!("Wounds heal {} as fast.", times(e.regen)));
        }
        if e.shunned {
            does.push("Creatures hostile to you do not step in.".into());
        }
        if let Some(floor) = e.floor {
            does.push(format!(
                "Its floor is {}.",
                content.terrain(floor).name.to_lowercase()
            ));
        }
        let _ = writeln!(
            out,
            "| {} | \"{}\" | {} |",
            capital(&e.name),
            e.enter,
            does.join(" ")
        );
    }
    out
}
