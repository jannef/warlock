//! Per-level mutable state: terrain as it is now, and what the player remembers of it.

use crate::content::{
    Content, LevelKind, LevelTemplate, LightId, StairsLink, TerrainDef, TerrainId,
};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::light::Light;
use crate::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A level that has been visited. Levels persist when the player leaves them.
///
/// Everything the rules need is kept here rather than looked up in the level's template, because
/// generated levels have no template in [`Content`] once they are instantiated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelState {
    pub(crate) id: String,
    pub(crate) name: String,
    /// Light everywhere on the level.
    pub(crate) ambient: Light,
    pub(crate) kind: LevelKind,
    /// How long a turn takes here (see [`LevelState::step_time`]).
    pub(crate) turn: Duration,
    pub(crate) terrain: Grid<TerrainId>,
    /// Terrain as last seen by the player; `None` where never seen.
    pub(crate) memory: Grid<Option<TerrainId>>,
    pub(crate) stairs: Vec<StairsLink>,
    /// Lights placed on cells (overriding the terrain's own light), by position.
    #[serde(with = "placed_lights")]
    pub(crate) lights: BTreeMap<Pos, LightId>,
}

impl LevelState {
    pub(crate) fn from_template(template: &LevelTemplate) -> Self {
        let terrain = template.terrain.clone();
        let memory = Grid::new(terrain.width(), terrain.height(), None);
        Self {
            id: template.id.clone(),
            name: template.name.clone(),
            ambient: template.ambient,
            kind: template.kind,
            turn: template.turn,
            terrain,
            memory,
            stairs: template.stairs.clone(),
            lights: template.lights.clone(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn width(&self) -> i32 {
        self.terrain.width()
    }

    pub fn height(&self) -> i32 {
        self.terrain.height()
    }

    pub fn kind(&self) -> LevelKind {
        self.kind
    }

    /// How long a turn takes on this level: a wait, opening or closing a door, taking stairs.
    pub fn turn(&self) -> Duration {
        self.turn
    }

    /// How long stepping onto `terrain` takes. This is the one place movement time is decided:
    /// on local levels it is the level's turn scaled by the terrain's `move_cost` percent. (The
    /// overworld adds its own kind here, whose steps cost the terrain's travel time.)
    pub fn step_time(&self, terrain: &TerrainDef) -> Duration {
        match self.kind {
            LevelKind::Dungeon | LevelKind::Outdoor => self.turn.percent(terrain.move_cost),
        }
    }

    /// The terrain at `p` as it is now (ground truth, `None` out of bounds). Only reachable
    /// through [`crate::World::ground_truth`], **for developer tools only**.
    pub fn terrain_at(&self, p: Pos) -> Option<TerrainId> {
        self.terrain.get(p).copied()
    }

    /// Opaque cells block sight. Out-of-bounds counts as opaque.
    pub(crate) fn is_opaque(&self, content: &Content, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_none_or(|t| !content.terrain(*t).transparent)
    }

    pub(crate) fn is_walkable(&self, content: &Content, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_some_and(|t| content.terrain(*t).walkable)
    }

    pub(crate) fn stairs_at(&self, p: Pos) -> Option<&StairsLink> {
        self.stairs.iter().find(|s| s.pos == p)
    }

    /// Whether every terrain and light id is valid for `content` (a guard for loaded saves).
    pub(crate) fn ids_valid_for(&self, content: &Content) -> bool {
        let terrain = content.terrain_defs().len();
        let lights = content.light_presets().len();
        let known = |t: &TerrainId| usize::from(t.0) < terrain;
        self.terrain.cells().iter().all(known)
            && self.memory.cells().iter().flatten().all(known)
            && self.lights.values().all(|l| usize::from(l.0) < lights)
    }
}

/// Serde adapter: JSON object keys must be strings, so placed lights are written as a list of
/// `[{x, y}, light]` pairs in position order.
mod placed_lights {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        lights: &BTreeMap<Pos, LightId>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        s.collect_seq(lights.iter())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<BTreeMap<Pos, LightId>, D::Error> {
        let pairs: Vec<(Pos, LightId)> = Vec::deserialize(d)?;
        Ok(pairs.into_iter().collect())
    }
}
