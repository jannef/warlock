//! Chests (spec §8 "Chests"; `data/chests.toml`): the bands, how hard each is to kick open, and
//! which blasts blow it apart.

use std::fmt::Write as _;
use warlock_core::content::Content;
use warlock_core::door;

/// The Strength scores the kick columns show (what the kicker's athletics and talents add counts
/// as Strength).
const KICK_STRENGTHS: [u8; 6] = [6, 10, 14, 18, 24, 30];

/// Every chest band, least first (the file's order: rising levels): its levels, name, how
/// sturdy it is with the chance a kick breaks it by Strength, and the blast it takes to blow
/// it apart.
pub(super) fn chest_bands(content: &Content) -> String {
    let rules = content.chests();
    let mut out = String::from("| Levels | Chest | Sturdiness |");
    let mut rule = String::from("|:---:|---|:---:|");
    for st in KICK_STRENGTHS {
        let _ = write!(out, " Strength {st} |");
        rule.push_str(":---:|");
    }
    let _ = writeln!(out, " Hardness |\n{rule}:---:|");
    for (i, band) in rules.bands.iter().enumerate() {
        let levels = match rules.bands.get(i + 1) {
            Some(next) if next.from - 1 == band.from => band.from.to_string(),
            Some(next) => format!("{}–{}", band.from, next.from - 1),
            None => format!("{} and up", band.from),
        };
        let _ = write!(out, "| {levels} | {} | {} |", band.name, band.sturdiness);
        for st in KICK_STRENGTHS {
            let _ = write!(out, " {}% |", door::break_chance(st, band.sturdiness));
        }
        let _ = writeln!(out, " {} |", band.hardness);
    }
    out
}

/// What each band's description says, as look mode adds it.
pub(super) fn chest_descriptions(content: &Content) -> String {
    let mut out = String::from("| Chest | What you see |\n|---|---|\n");
    for band in &content.chests().bands {
        let _ = writeln!(out, "| {} | {} |", band.name, band.description);
    }
    out
}

/// The chance each stack breaks when a chest is kicked open or blown apart, and what is spared.
pub(super) fn chest_breaking(content: &Content) -> String {
    let destroyed = content.chests().destroyed;
    let deft = (content.talents().id("deft_breaker"))
        .map(|id| content.talents().get(id))
        .and_then(|t| {
            t.effects.iter().find_map(|e| match e {
                warlock_core::content::TalentEffect::SmashDestroyPercent { value } => Some(*value),
                _ => None,
            })
        })
        .unwrap_or(0);
    let with_deft = i32::from(destroyed) * (100 + deft).max(0) / 100;
    format!(
        "| How it opens | Each stack inside breaks | Spared |\n|---|:---:|---|\n\
         | Opened (a key, Pick lock, Knock then open, or unlocked) | never | everything |\n\
         | Kicked open | {destroyed}% | coins and loose gems |\n\
         | Kicked open, with Deft breaker | {with_deft}% | coins and loose gems |\n\
         | Blown apart by a blast | {destroyed}% | nothing |\n"
    )
}
