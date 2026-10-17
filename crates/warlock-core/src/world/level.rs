//! Per-level mutable state: terrain as it is now, and what the player remembers of it.

use crate::content::{Content, LevelTemplate, StairsLink, TerrainId};
use crate::geom::Pos;
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

/// A level that has been visited. Levels persist when the player leaves them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelState {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) terrain: Grid<TerrainId>,
    /// Terrain as last seen by the player; `None` where never seen.
    pub(crate) memory: Grid<Option<TerrainId>>,
    pub(crate) stairs: Vec<StairsLink>,
}

impl LevelState {
    pub(crate) fn from_template(template: &LevelTemplate) -> Self {
        let terrain = template.terrain.clone();
        let memory = Grid::new(terrain.width(), terrain.height(), None);
        Self {
            id: template.id.clone(),
            name: template.name.clone(),
            terrain,
            memory,
            stairs: template.stairs.clone(),
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
}
