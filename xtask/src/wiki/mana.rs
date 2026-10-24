//! Mana (spec §8 "Mana"): what races and classes give, how fast it comes back by Willpower, and
//! what changes the pace.

use super::healing::pace;
use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::hunger::Hunger;
use warlock_core::mana;
use warlock_core::time::Duration;

/// What each race and each class adds to the mana, sorted by mana, most first, then by name.
pub(super) fn races_and_classes(content: &Content) -> String {
    let mut races: Vec<(&str, i32)> = (content.races().list.iter())
        .map(|r| (r.name.as_str(), r.mana))
        .collect();
    let mut classes: Vec<(&str, i32)> = (content.classes().iter())
        .map(|c| (c.name.as_str(), c.mana))
        .collect();
    let by_mana = |a: &(&str, i32), b: &(&str, i32)| b.1.cmp(&a.1).then(a.0.cmp(b.0));
    races.sort_by(by_mana);
    classes.sort_by(by_mana);
    let mut out = String::from("| Race | Mana |\n|---|:---:|\n");
    for (name, mana) in races {
        let _ = writeln!(out, "| {name} | {mana} |");
    }
    out.push_str("\n| Class | Mana |\n|---|:---:|\n");
    for (name, mana) in classes {
        let _ = writeln!(out, "| {name} | {mana} |");
    }
    out
}

/// How long a point of mana takes to come back, by Willpower.
pub(super) fn regeneration(_content: &Content) -> String {
    let mut out = String::from("| Willpower | A point of mana back every |\n|:---:|---|\n");
    for willpower in [3, 6, 8, 10, 12, 14, 16, 18, 20, 22, 25] {
        let seconds = mana::regen_seconds(willpower);
        let label = if willpower == 25 {
            "25 or more".to_string()
        } else {
            willpower.to_string()
        };
        let _ = writeln!(out, "| {label} | {} |", minutes_and_seconds(seconds));
    }
    out
}

/// `4 minutes`, `5 minutes and 48 seconds`, `1 minute`.
fn minutes_and_seconds(seconds: u32) -> String {
    let (m, s) = (seconds / 60, seconds % 60);
    let minutes = if m == 1 {
        "1 minute".to_string()
    } else {
        format!("{m} minutes")
    };
    match s {
        0 => minutes,
        s => format!("{minutes} and {s} seconds"),
    }
}

/// What changes how fast mana comes back, and what drains it: resting, each status and each
/// disease that changes it (a disease when it slows mana more than Diseased does), starving,
/// and each special room.
pub(super) fn paces(content: &Content) -> String {
    let mut out = String::from("| While | Mana comes back |\n|---|---|\n");
    let resting = u32::from(content.healing().mana_resting);
    let _ = writeln!(out, "| Resting (`R`) | {} |", pace(resting));
    for def in content.status_defs() {
        let drains = match (def.mana_damage, def.tick) {
            (0, _) | (_, None) => String::new(),
            (points, Some(tick)) => format!(
                "; it drains {points} mana every {}",
                Duration::seconds(u64::from(tick)).spoken()
            ),
        };
        if def.mana_regen != 100 || !drains.is_empty() {
            let pace = pace(u32::from(def.mana_regen));
            let _ = writeln!(out, "| {} | {pace}{drains} |", def.name);
        }
    }
    let diseased = content
        .disease_status()
        .map_or(100, |s| content.status(s).mana_regen);
    for (_, def) in content.disease_ids() {
        if let Some(own) = def.mana_regen.filter(|&r| r < diseased) {
            let _ = writeln!(
                out,
                "| Diseased with {} | {} |",
                def.name,
                pace(u32::from(own))
            );
        }
    }
    if content.hunger().is_some() {
        let _ = writeln!(out, "| {} | {} |", Hunger::Starving.name(), pace(0));
    }
    for room in content.room_effects().defs() {
        if room.mana_regen != 100 {
            let _ = writeln!(
                out,
                "| In {} | {} |",
                room.with_article(),
                pace(u32::from(room.mana_regen))
            );
        }
    }
    out
}
