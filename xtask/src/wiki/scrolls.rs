//! Scrolls (spec §8 "Scrolls").

use super::text::{Term, anchored, capital};
use super::traps::sound_text;
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Content, Reading};

/// The bonuses the check's table shows a column for: Intelligence's modifier, talents, and
/// Decipher script's points (at most 12 at 200, more with what raises the skill, for one trained in it).
const DECIPHER_BONUSES: [i32; 7] = [-2, 0, 2, 4, 8, 12, 16];

/// The spell levels the check's table samples, by rows of rising difficulty.
const DECIPHER_LEVELS: [u8; 8] = [0, 6, 11, 22, 35, 46, 56, 70];

/// The check of reading a scroll: its difficulty by spell level, and the chance at each bonus.
pub(super) fn decipher_chances(content: &Content) -> String {
    let Some(rules) = content.decipher() else {
        return String::new();
    };
    let mut out = String::from("| Spell level | Difficulty |");
    for b in DECIPHER_BONUSES {
        let _ = write!(out, " Bonus {b:+} |");
    }
    out.push_str("\n|:---:|:---:|");
    out.push_str(&":---:|".repeat(DECIPHER_BONUSES.len()));
    out.push('\n');
    for level in DECIPHER_LEVELS {
        let difficulty = rules.difficulty(level);
        let _ = write!(out, "| {level} | {difficulty} |");
        for b in DECIPHER_BONUSES {
            let percent = f64::from(check::ways(b, difficulty)) * 100.0 / 144.0;
            let _ = write!(out, " {percent:.0} % |");
        }
        out.push('\n');
    }
    out
}

/// The scrolls: what reading each does, the roll it casts at (its rung of the spell's ladder,
/// before the reader's Scroll adept), how far it reaches, what it sounds like, from which item
/// level (depth) loot holds it, and what it is worth. By name.
pub(super) fn scrolls(content: &Content) -> String {
    let mut out = String::from(
        "| Scroll | What reading it does | Casts at | Reach | Sound | Found from | Value |\n\
         |---|---|:---:|---:|---|---|---:|\n",
    );
    let mut scrolls: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| o.read.as_ref().map(|r| (o, r)))
        .collect();
    scrolls.sort_by_key(|(o, _)| o.name.to_lowercase());
    for (o, reading) in scrolls {
        let (what, roll, reach, sound) = match reading {
            Reading::Cast(scroll) => {
                let def = content.spell(scroll.spell);
                let rung = match scroll.rungs {
                    0 => "at its base".to_string(),
                    1 => "a rung up its ladder".to_string(),
                    n => format!("{n} rungs up its ladder"),
                };
                let what = format!(
                    "Casts {} {rung}, free of mana (see the spell list), and teaches it if \
                     studied",
                    def.name
                );
                let roll = def.roll_at(usize::from(scroll.rungs)).map_or_else(
                    || "—".to_string(),
                    |roll| match def.damage_type {
                        Some(_) => super::damage::typed_dice(content, roll, def.damage_type),
                        None => format!("{roll} healed"),
                    },
                );
                let reach = match def.base_range() {
                    None if def.aims_item() => "an item you carry".to_string(),
                    None => "yourself".to_string(),
                    Some(1) => "next to you".to_string(),
                    Some(r) => format!("{r} cells"),
                };
                (what, roll, reach, sound_text(def.look.noise))
            }
            Reading::Cleanse(cleansing) => {
                let does = match (cleansing.mends, cleansing.restores) {
                    (true, true) => {
                        "Mends every point of [stat damage](diseases.md#stat-damage) and gives back every point [drains](stats.md#drained-stats) took"
                    }
                    (true, false) => "Mends every point of [stat damage](diseases.md#stat-damage)",
                    _ => "Gives back every point [drains](stats.md#drained-stats) took",
                };
                let what = format!(
                    "{does}; casts no spell, so there is nothing to study. Refused, and kept, \
                     with nothing to restore"
                );
                (
                    what,
                    "—".to_string(),
                    "yourself".to_string(),
                    sound_text(None),
                )
            }
        };
        let found = match o.frequency {
            0 => "not as loot".to_string(),
            f => format!("item level {}, frequency {f}", o.level),
        };
        let _ = writeln!(
            out,
            "| {} `{}` | {what} | {roll} | {reach} | {sound} | {found} | {} gold |",
            anchored(Term::Scroll, &o.id, &capital(&o.name)),
            o.glyph,
            o.value
        );
    }
    out
}
