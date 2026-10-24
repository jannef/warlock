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
            Some(0) => notes.push("lets sound through".into()),
            Some(m) => notes.push(format!("muffles sound by {m}")),
        }
        let footsteps = if t.walkable {
            footsteps_text(t.step_noise)
        } else {
            "-".into()
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

/// How loud steps onto terrain are, in words.
fn footsteps_word(step_noise: u16) -> &'static str {
    match step_noise {
        0..=90 => "quiet",
        91..=109 => "normal",
        110..=199 => "loud",
        _ => "very loud",
    }
}

/// How loud steps onto terrain are, in words and exactly (`step_noise`: percent of the walker's
/// footsteps): `loud, 150%`.
fn footsteps_text(step_noise: u16) -> String {
    format!("{}, {step_noise}%", footsteps_word(step_noise))
}

/// Walkable dungeon and outdoor terrain on which steps are louder or quieter than normal, by how
/// loud, loudest first (spec §5 "Hearing"); the overworld is silent.
pub(super) fn footsteps(content: &Content) -> String {
    let ids = overworld_ids(content);
    let walked: Vec<&TerrainDef> = (content.terrain_defs().iter())
        .filter(|t| t.walkable && !is_overworld_terrain(t, &ids) && t.step_noise != 100)
        .collect();
    let mut levels: Vec<u16> = walked.iter().map(|t| t.step_noise).collect();
    levels.sort_unstable_by(|a, b| b.cmp(a));
    levels.dedup();
    let mut out = String::from("| Footsteps | Terrain |\n|---|---|\n");
    for level in levels {
        let names: Vec<String> = (walked.iter())
            .filter(|t| t.step_noise == level)
            .map(|t| t.name.clone())
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} |",
            capital(&footsteps_text(level)),
            capital(&names.join(", "))
        );
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
