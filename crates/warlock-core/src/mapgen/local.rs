//! Local areas of the overworld (`local:x,y`, spec §3): the outdoor level behind one overworld
//! cell, entered with `>` there and left by walking off any edge, which leads back to the cell.
//!
//! For now every local area is a **placeholder**: an open field of the ground its overworld
//! terrain names (`local` in `data/terrain.toml`), with the player starting in the middle. The
//! real outdoor generators replace [`build`]; everything around it (ids, linking, the edge exit,
//! persistence) stays. A generator that draws randomness must use the level's own stream,
//! [`crate::rng::Rng::map`]`(seed, id, attempt)`, so an area depends only on the run seed and its
//! overworld position.

use super::GenError;
use crate::content::{Content, EdgeExit, LevelKind, LevelTemplate, LocalArea};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::light::Light;
use std::collections::BTreeMap;

/// Width and height of a local area.
pub const SIZE: (i32, i32) = (80, 40);

/// Generates the local area `id` (`local:x,y`) for the run with `seed`.
pub(super) fn generate(content: &Content, _seed: u64, id: &str) -> Result<LevelTemplate, GenError> {
    let area = content
        .local_area(id)
        .ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    Ok(build(content, id, &area))
}

/// The placeholder: open ground everywhere, the start in the middle.
fn build(content: &Content, id: &str, area: &LocalArea<'_>) -> LevelTemplate {
    let (width, height) = SIZE;
    LevelTemplate {
        id: id.to_string(),
        name: capitalize(&content.terrain(area.terrain).name),
        ambient: Light::DARK,
        kind: LevelKind::Outdoor,
        turn: LevelKind::Outdoor.default_turn(),
        terrain: Grid::new(width, height, area.ground),
        player_start: Some(Pos::new(width / 2, height / 2)),
        creatures: Vec::new(),
        stairs: Vec::new(),
        lights: BTreeMap::new(),
        flow: BTreeMap::new(),
        regions: Vec::new(),
        exit: Some(EdgeExit {
            to: area.overworld.id.clone(),
            pos: area.pos,
        }),
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
