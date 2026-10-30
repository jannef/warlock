//! Scrolls (spec §8 "Scrolls").

use super::text::{Term, anchored, capital};
use super::traps::sound_text;
use std::fmt::Write as _;
use warlock_core::content::{Content, Reading};

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
