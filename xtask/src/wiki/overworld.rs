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

/// Per campaign the wiki describes, its overworld drawn in the game's glyphs, one character a
/// cell, with the hidden locations drawn as what they are once revealed (the wiki tells the
/// truth, campaign design §11.4), and under it what each glyph is.
pub(super) fn map(content: &Content) -> String {
    let mut out = String::new();
    for campaign in order::campaigns(content) {
        let Some(level) = content.overworld_of(&campaign.id) else {
            continue;
        };
        let (w, h) = (level.terrain.width(), level.terrain.height());
        // Glyph -> the names of the terrain drawn with it, in the order first met.
        let mut key: BTreeMap<char, Vec<String>> = BTreeMap::new();
        let mut rows = String::new();
        for y in 0..h {
            for x in 0..w {
                let pos = Pos::new(x, y);
                let id = (level.hidden.get(&pos)).map_or(level.terrain[pos], |l| l.terrain);
                let terrain = content.terrain(id);
                rows.push(terrain.glyph);
                let names = key.entry(terrain.glyph).or_default();
                if !names.contains(&terrain.name) {
                    names.push(terrain.name.clone());
                }
            }
            rows.push('\n');
        }
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "**{}**, {w} by {h} cells, north up, the hidden locations drawn:\n\n```text\n{rows}```\n\n\
             | Glyph | Terrain |\n|:---:|---|",
            campaign.name
        );
        for (glyph, names) in key {
            let names: Vec<String> = names.iter().map(|n| capital(n)).collect();
            let glyph = if glyph == '|' {
                "`\\|`".to_string()
            } else {
                format!("`{glyph}`")
            };
            let _ = writeln!(out, "| {glyph} | {} |", names.join(", "));
        }
    }
    if out.is_empty() {
        out.push_str("No world has an overworld yet.\n");
    }
    out
}

/// Per campaign the wiki describes, its overworld's regions (the names the status bar shows) in
/// the order they are matched: a cell is in the first region whose rectangle holds it.
pub(super) fn regions(content: &Content) -> String {
    let mut out = String::new();
    for campaign in order::campaigns(content) {
        let Some(level) = content.overworld_of(&campaign.id) else {
            continue;
        };
        if level.regions.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "**{}**: a cell is in the first region whose corners hold it.\n\n\
             | Region | From | To |\n|---|---|---|",
            campaign.name
        );
        for r in &level.regions {
            let _ = writeln!(out, "| {} | {} | {} |", r.name, r.min, r.max);
        }
    }
    if out.is_empty() {
        out.push_str("No world has regions yet.\n");
    }
    out
}
