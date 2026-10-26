//! Services: who sells a cure, a mending, or an identification, and for how much.

use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, DialogueId, ServiceKind};

/// The services (`[[dialogue.service]]` in the dialogue data): who sells each, what it does, and
/// its price before Pilgrim, sorted by who sells it and then by the service's name.
pub(super) fn services(content: &Content) -> String {
    let mut rows: Vec<(String, String, &'static str, String)> = Vec::new();
    for (i, d) in content.dialogues().iter().enumerate() {
        let who = capital(&content.speaker_name(DialogueId(i as u16)));
        for s in &d.services {
            let does = match s.does {
                ServiceKind::Cure => "ends every disease, even one not yet broken out",
                ServiceKind::Identify => {
                    "identifies everything carried that is not yet known: magic items and \
                     scroll types"
                }
                ServiceKind::Mend => "mends every point of stat damage (not while a disease shows)",
            };
            let price = match s.does {
                ServiceKind::Identify => format!("{} gold per unidentified stack", s.price),
                _ => format!("{} gold", s.price),
            };
            rows.push((who.clone(), s.name.clone(), does, price));
        }
    }
    if rows.is_empty() {
        return "No one offers a service yet.\n".to_string();
    }
    rows.sort();
    let mut out = String::from("| Who | Service | What it does | Price |\n|---|---|---|---:|\n");
    for (who, name, does, price) in rows {
        let _ = writeln!(out, "| {who} | {name} | {does} | {price} |");
    }
    out
}
