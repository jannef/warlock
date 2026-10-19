//! Terrain: the local and overworld terrain tables and how loud footsteps are on each.

use super::actions::at_normal_pace;
use super::text::{capital, glyph, yes};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use warlock_core::content::{ActionKind, BASE_TURN, Content, LevelKind, TerrainDef, price};

/// The ids of the terrain the overworld map is drawn with (it has its own table).
fn overworld_ids(content: &Content) -> BTreeSet<String> {
    content
        .levels()
        .filter(|l| l.kind == LevelKind::Overworld)
        .flat_map(|l| l.terrain.cells().iter())
        .map(|t| content.terrain(*t).id.clone())
        .collect()
}

fn is_overworld_terrain(t: &TerrainDef, ids: &BTreeSet<String>) -> bool {
    t.travel_minutes.is_some() || t.landscape.is_some() || ids.contains(&t.id)
}

pub(super) fn local_terrain(content: &Content) -> String {
    let ids = overworld_ids(content);
    let step = content.action_costs().cost(ActionKind::Step);
    let mut out = String::from(
        "| Terrain | Glyph | Walk on it | Blocks sight | Step takes | Footsteps | Height (cells) \
         | Notes |\n|---|:---:|:---:|:---:|---:|:---:|:---:|---|\n",
    );
    for t in content
        .terrain_defs()
        .iter()
        .filter(|t| !is_overworld_terrain(t, &ids))
    {
        let step_time = if t.walkable {
            at_normal_pace(t.step_price(step, LevelKind::Dungeon))
        } else if t.is_closed_door() {
            let open = content.action_costs().cost(ActionKind::OpenDoor);
            format!("{} (opens it)", at_normal_pace(price(open, BASE_TURN)))
        } else {
            "-".into()
        };
        let mut notes = Vec::new();
        if t.is_closed_door() {
            notes.push("door: walking into it opens it".to_string());
        }
        if let Some((to, sturdiness)) = t.kicked_in {
            let broken = content.terrain(to).name.to_lowercase();
            notes.push(format!(
                "kicked in, it breaks into a {broken} (sturdiness {sturdiness})"
            ));
        }
        if t.is_open_door() {
            notes.push("door: can be closed".to_string());
        }
        match t.stairs {
            Some(warlock_core::content::StairsDir::Down) => notes.push("stairs down".into()),
            Some(warlock_core::content::StairsDir::Up) => notes.push("stairs up".into()),
            None => {}
        }
        if t.light.is_some() {
            notes.push("gives light".into());
        }
        if t.breaks.is_some() {
            notes.push("can be dug through or blasted open".into());
        }
        if t.roofed {
            notes.push("roofed: no sun or sky light".into());
        }
        if let Some(f) = t.freeze {
            notes.push(format!("freezes below {} °C", f.below));
        }
        if t.ice.is_some() {
            notes.push("ice on water: holds by its thickness".into());
        }
        match t.muffle {
            None => {}
            Some(0) if t.walkable || t.transparent => {}
            Some(_) => notes.push("muffles sound".into()),
        }
        let footsteps = if t.walkable {
            footsteps_word(t.step_noise)
        } else {
            "-"
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            capital(&t.name),
            glyph(t.glyph),
            yes(t.walkable || t.is_closed_door()),
            yes(!t.transparent),
            step_time,
            footsteps,
            t.height,
            notes.join("; ")
        );
    }
    out
}

/// How loud steps onto terrain are, in words: the exact numbers are rules detail.
fn footsteps_word(step_noise: u16) -> &'static str {
    match step_noise {
        0..=90 => "quiet",
        91..=109 => "normal",
        110..=199 => "loud",
        _ => "very loud",
    }
}

/// Walkable dungeon and outdoor terrain on which steps are louder or quieter than normal,
/// grouped (spec §5 "Hearing"); the overworld is silent.
pub(super) fn footsteps(content: &Content) -> String {
    let ids = overworld_ids(content);
    let mut out = String::from("| Footsteps | Terrain |\n|---|---|\n");
    for word in ["very loud", "loud", "quiet"] {
        let names: Vec<String> = content
            .terrain_defs()
            .iter()
            .filter(|t| t.walkable && !is_overworld_terrain(t, &ids))
            .filter(|t| footsteps_word(t.step_noise) == word)
            .map(|t| t.name.clone())
            .collect();
        if !names.is_empty() {
            let _ = writeln!(
                out,
                "| {} | {} |",
                capital(word),
                capital(&names.join(", "))
            );
        }
    }
    out
}

pub(super) fn overworld_terrain(content: &Content) -> String {
    let ids = overworld_ids(content);
    let step = content.action_costs().cost(ActionKind::Step);
    let mut out = String::from(
        "| Terrain | Glyph | Travel per step | Blocks sight | Local area (`>`) |\n\
         |---|:---:|---:|:---:|:---:|\n",
    );
    for t in content
        .terrain_defs()
        .iter()
        .filter(|t| is_overworld_terrain(t, &ids))
    {
        let travel = match t.travel_minutes {
            Some(_) => at_normal_pace(t.step_price(step, LevelKind::Overworld)),
            None => "impassable".to_string(),
        };
        let local = if t.stairs.is_some() {
            "location: `>` enters it"
        } else {
            yes(t.local.is_some())
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            capital(&t.name),
            glyph(t.glyph),
            travel,
            yes(!t.transparent),
            local
        );
    }
    out
}
