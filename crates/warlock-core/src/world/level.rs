//! Per-level mutable state: terrain and piles of objects as they are now, and what the player
//! remembers of them.

use crate::content::{
    Content, EdgeExit, Flow, LevelKind, LevelTemplate, LightId, StairsLink, TerrainDef, TerrainId,
};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::light::Light;
use crate::object::Stack;
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
    /// Where the player arrives when no staircase leads back to where they came from.
    pub(crate) start: Option<Pos>,
    /// Where walking off the edge leads (local areas: back to their overworld cell).
    pub(crate) exit: Option<EdgeExit>,
    /// Lights placed on cells (overriding the terrain's own light), by position.
    #[serde(with = "by_pos")]
    pub(crate) lights: BTreeMap<Pos, LightId>,
    /// Flow of running water, by position. Presentation-only: no rule reads it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) flow: BTreeMap<Pos, Flow>,
    /// Piles of objects lying on cells, by position, each top first and never empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) objects: BTreeMap<Pos, Vec<Stack>>,
    /// Piles as the player last saw them, by position (spec §5 "Memory"): a pile seen taken
    /// away is forgotten, one that changed out of sight is remembered as it was.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) object_memory: BTreeMap<Pos, Vec<Stack>>,
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
            start: template.player_start,
            exit: template.exit.clone(),
            lights: template.lights.clone(),
            flow: template.flow.clone(),
            objects: template.objects.clone(),
            object_memory: BTreeMap::new(),
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
    /// on local levels it is the level's turn scaled by the terrain's `move_cost` percent; on the
    /// overworld it is the terrain's `travel_minutes` (content validation guarantees walkable
    /// overworld terrain has them).
    pub fn step_time(&self, terrain: &TerrainDef) -> Duration {
        let local = self.turn.percent(terrain.move_cost);
        match self.kind {
            LevelKind::Dungeon | LevelKind::Outdoor => local,
            LevelKind::Overworld => terrain
                .travel_minutes
                .map_or(local, |m| Duration::minutes(u64::from(m))),
        }
    }

    /// The terrain at `p` as it is now (ground truth, `None` out of bounds). Only reachable
    /// through [`crate::World::ground_truth`], **for developer tools only**.
    pub fn terrain_at(&self, p: Pos) -> Option<TerrainId> {
        self.terrain.get(p).copied()
    }

    /// The pile at `p` as it is now, top first (ground truth, empty where nothing lies). Only
    /// reachable through [`crate::World::ground_truth`], **for developer tools only**.
    pub fn objects_at(&self, p: Pos) -> &[Stack] {
        self.objects.get(&p).map_or(&[], Vec::as_slice)
    }

    /// Replaces the pile at `p`; an empty pile removes it.
    pub(crate) fn set_pile(&mut self, p: Pos, pile: Vec<Stack>) {
        if pile.is_empty() {
            self.objects.remove(&p);
        } else {
            self.objects.insert(p, pile);
        }
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

    /// Whether every terrain, light, and object id is valid for `content` (a guard for loaded
    /// saves).
    pub(crate) fn ids_valid_for(&self, content: &Content) -> bool {
        let terrain = content.terrain_defs().len();
        let lights = content.light_presets().len();
        let known = |t: &TerrainId| usize::from(t.0) < terrain;
        let piles = self.objects.values().chain(self.object_memory.values());
        self.terrain.cells().iter().all(known)
            && self.memory.cells().iter().flatten().all(known)
            && self.lights.values().all(|l| usize::from(l.0) < lights)
            && piles.flatten().all(|s| s.is_valid_for(content))
    }
}

/// Serde adapter: JSON object keys must be strings, so maps by position (placed lights, flow,
/// piles) are written as a list of `[{x, y}, value]` pairs in position order.
mod by_pos {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer, V: Serialize>(
        map: &BTreeMap<Pos, V>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        s.collect_seq(map.iter())
    }

    pub fn deserialize<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
        d: D,
    ) -> Result<BTreeMap<Pos, V>, D::Error> {
        let pairs: Vec<(Pos, V)> = Vec::deserialize(d)?;
        Ok(pairs.into_iter().collect())
    }
}
