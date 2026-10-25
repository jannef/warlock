//! The overworld's places: each described campaign's locations and landmarks, hidden ones too
//! (the wiki tells the truth, campaign design §11.4).

use super::order;
use super::text::capital;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use warlock_core::Pos;
use warlock_core::content::Content;

/// Per campaign the wiki describes, its overworld's named places by name: the cell, the region,
/// what the cell is, and where it leads (a hidden location: what reveals it is the quests'
/// business, so the row says only that it is hidden).
pub(super) fn locations(content: &Content) -> String {
    let mut out = String::new();
    for campaign in order::campaigns(content) {
        let Some(map) = content.overworld_of(&campaign.id) else {
            continue;
        };
        // Name -> (cell, terrain, leads to, hidden); a landmark of several cells by its first.
        let mut rows: BTreeMap<String, (Pos, String, String, bool)> = BTreeMap::new();
        let leads = |to: Option<&str>| match to {
            Some(to) => content
                .level_display_name(to)
                .unwrap_or_else(|| to.to_string()),
            None => "nowhere yet".to_string(),
        };
        for (pos, place) in &map.places {
            let link = map.stairs.iter().find(|s| s.pos == *pos);
            let to = match link {
                Some(link) => leads(link.to.as_deref()),
                None => "—".to_string(),
            };
            let terrain = content.terrain(map.terrain[*pos]).name.clone();
            (rows.entry(place.name.clone())).or_insert((*pos, terrain, to, false));
        }
        for (pos, hidden) in &map.hidden {
            let terrain = content.terrain(hidden.terrain).name.clone();
            let to = leads(hidden.to.as_deref());
            rows.insert(hidden.name.clone(), (*pos, terrain, to, true));
        }
        if rows.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "**{}**: the overworld, {} by {} cells.\n\n| Place | Cell | Region | What it is | Leads to | Hidden |\n\
             |---|---|---|---|---|:---:|",
            campaign.name,
            map.terrain.width(),
            map.terrain.height()
        );
        for (name, (cell, terrain, to, hidden)) in rows {
            let region = map.region_at(cell).unwrap_or("—");
            let hidden = if hidden {
                "until a quest reveals it"
            } else {
                "—"
            };
            let _ = writeln!(
                out,
                "| {name} | {cell} | {region} | {} | {to} | {hidden} |",
                capital(&terrain)
            );
        }
    }
    if out.is_empty() {
        out.push_str("No world has an overworld yet.\n");
    }
    out
}
