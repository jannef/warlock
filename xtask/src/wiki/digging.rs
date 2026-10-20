//! Digging (spec §8 "Digging"): the walls that break and how long each tool takes through them.

use super::text::{article, capital, glyph, yes};
use std::fmt::Write as _;
use warlock_core::content::{
    ActionKind, BASE_TURN, Content, NORMAL_SPEED, TerrainDef, TrapEffect, price,
};
use warlock_core::stats::AVERAGE;
use warlock_core::time::Duration;
use warlock_core::world::digs_to_break;

/// How long digging through a wall of `hardness` takes with a tool of `power` at Strength
/// `strength`, at a normal pace: to the second under ten minutes, else to the nearest minute.
fn dig_time(content: &Content, hardness: u16, power: u16, strength: u8) -> String {
    let turn = price(content.action_costs().cost(ActionKind::Dig), BASE_TURN);
    let turns = i64::from(digs_to_break(hardness, power, strength));
    let seconds = u64::try_from(turn * turns / i64::from(NORMAL_SPEED)).unwrap_or(0);
    let (minutes, rest) = (seconds / 60, seconds % 60);
    match (minutes, rest) {
        (0, _) | (_, 0) => Duration::seconds(seconds).spoken(),
        (1..10, _) => format!(
            "{} {}",
            Duration::minutes(minutes).spoken(),
            Duration::seconds(rest).spoken()
        ),
        _ => Duration::minutes((seconds + 30) / 60).spoken(),
    }
}

/// The walls that break (spec §8 "Digging"): how hard each is, how long each digging tool takes
/// through it at average Strength, and what it becomes.
pub(super) fn digging(content: &Content) -> String {
    // The tools from the weakest, so the pick-axe comes before the mattock.
    let mut tools: Vec<(&str, u16)> = (content.object_defs().iter())
        .filter_map(|o| o.dig.map(|power| (o.name.as_str(), power)))
        .collect();
    tools.sort_by_key(|&(name, power)| (power, name));
    let blasts: Vec<(&str, u16)> = (content.trap_defs().iter())
        .filter_map(|t| match &t.effect {
            TrapEffect::Burst(b) if b.blast > 0 => Some((t.noun.as_str(), b.blast)),
            _ => None,
        })
        .collect();
    let mut out = String::from("| Wall | Glyph | Hardness |");
    for (name, _) in &tools {
        let _ = write!(out, " With {} {name} |", article(name));
    }
    for (name, _) in &blasts {
        let _ = write!(out, " {} {name} breaks it |", capital(article(name)));
    }
    out.push_str(" Becomes |\n|---|:---:|---:|");
    out.push_str(&"---:|".repeat(tools.len()));
    out.push_str(&":---:|".repeat(blasts.len()));
    out.push_str("---|\n");
    for t in content.terrain_defs() {
        let Some(breaks) = t.breaks else { continue };
        let _ = write!(
            out,
            "| {} | {} | {} |",
            capital(&t.name),
            glyph(t.glyph),
            breaks.hardness
        );
        for (_, power) in &tools {
            let _ = write!(
                out,
                " {} |",
                dig_time(content, breaks.hardness, *power, AVERAGE)
            );
        }
        for (_, blast) in &blasts {
            let _ = write!(out, " {} |", yes(breaks.hardness < *blast));
        }
        let _ = writeln!(out, " {} |", content.terrain(breaks.becomes).name);
    }
    out
}

/// How Strength changes digging: the first digging tool through each wall that breaks, at a
/// few Strength scores.
pub(super) fn digging_strength(content: &Content) -> String {
    // The weakest tool: the pick-axe.
    let Some((tool, power)) = (content.object_defs().iter())
        .filter_map(|o| o.dig.map(|power| (o.name.as_str(), power)))
        .min_by_key(|&(name, power)| (power, name))
    else {
        return String::new();
    };
    let walls: Vec<&TerrainDef> = (content.terrain_defs().iter())
        .filter(|t| t.breaks.is_some())
        .collect();
    let mut out = String::from("| Strength |");
    for t in &walls {
        let _ = write!(out, " {} |", capital(&t.name));
    }
    out.push_str("\n|---:|");
    out.push_str(&"---:|".repeat(walls.len()));
    out.push('\n');
    for strength in [3, 6, 10, 14, 18, 25] {
        let _ = write!(out, "| {strength} |");
        for t in &walls {
            let hardness = t.breaks.map_or(0, |b| b.hardness);
            let _ = write!(out, " {} |", dig_time(content, hardness, power, strength));
        }
        out.push('\n');
    }
    let _ = writeln!(out, "\nWith {} {tool}, at a normal pace.", article(tool));
    out
}
