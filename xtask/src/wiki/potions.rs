//! Potions (spec §8 "Potions").

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, Draught};

/// The potions: what drinking each does in numbers, its colour, the training marks it gives,
/// from which item level (depth) loot holds it and how often, and what it is worth. By name.
pub(super) fn potions(content: &Content) -> String {
    let mut out = String::from(
        "| Potion | Kind | What drinking it does | Colour | Marks | Found from | Value |\n\
         |---|---|---|---|---|---|---:|\n",
    );
    let mut potions: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| o.drink.as_ref().map(|d| (o, d)))
        .collect();
    potions.sort_by_key(|(o, _)| o.name.to_lowercase());
    for (o, drink) in potions {
        // What its cures end, by the statuses' names.
        let cured: Vec<String> = (content.status_ids())
            .filter(|(_, s)| drink.does.cures().iter().any(|c| s.cured_by.contains(c)))
            .map(|(_, s)| s.name.clone())
            .collect();
        let also = match cured.is_empty() {
            true => String::new(),
            false => format!("; ends {}", cured.join(" and ")),
        };
        let what = match &drink.does {
            Draught::Heal { dice, .. } => {
                format!("{dice} hit points back at once, never past your maximum{also}")
            }
            Draught::Cure { diseases, .. } => {
                let diseases = match (diseases, cured.is_empty()) {
                    (true, true) => "ends every disease".to_string(),
                    (true, false) => format!("ends every disease and {}", cured.join(" and ")),
                    (false, _) => format!("ends {}", cured.join(" and ")),
                };
                format!("{} at once", capital(&diseases))
            }
            Draught::Mana { dice } => {
                format!("{dice} mana back at once, never past your maximum")
            }
            Draught::Elixir {
                stat,
                points,
                potential,
            } => format!(
                "{} +{points} for good (a trained rise: the potential lifted to the score if \
                 below), then its potential +{} (a roll)",
                stat.name(),
                potential.to_string().trim_start_matches('+')
            ),
            Draught::Harm { dice, status } => format!(
                "{} degrees of {} (as rotten food)",
                dice.to_string().trim_start_matches('+'),
                content.status(*status).name
            ),
        };
        let marks = match drink.marks.as_slice() {
            [] => "none".to_string(),
            marks => (marks.iter())
                .map(|(stat, n)| format!("{n} {}", stat.name()))
                .collect::<Vec<_>>()
                .join(", "),
        };
        let found = match o.frequency {
            0 => "not as loot".to_string(),
            f => format!("item level {}, frequency {f}", o.level),
        };
        let _ = writeln!(
            out,
            "| {} `{}` | {} | {what} | {} (`{}`) | {marks} | {found} | {} gold |",
            capital(&o.name),
            o.glyph,
            drink.does.kind(),
            drink.colour,
            o.fg,
            o.value
        );
    }
    out
}
