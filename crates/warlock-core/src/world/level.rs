//! Per-level mutable state: terrain, piles of objects, and traps as they are now, and what the
//! player remembers of them; fires, and the walls dug into.

use crate::content::{
    Content, EdgeExit, Flow, LevelKind, LevelTemplate, LightId, StairsLink, TerrainDef, TerrainId,
    Trap, TrapId,
};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::light::Light;
use crate::object::Stack;
use crate::time::GameTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Something burning on a cell (spec §8 "Ranged combat"): a burning object landed there (a
/// flaming arrow), giving light until it burns out and is gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fire {
    /// The light it gives, a preset.
    pub light: LightId,
    /// When it has burned out: it gives light while the clock is before this.
    pub until: GameTime,
}

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
    /// Burning objects lying on cells, by position: light sources until they burn out. The
    /// burning objects themselves lie in the pile there.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) fires: BTreeMap<Pos, Fire>,
    /// Traps, by position (spec §8 "Traps"), hidden or known; a spent one is gone.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) traps: BTreeMap<Pos, Trap>,
    /// Known traps as the player last saw them, by position: a trap seen gone is forgotten.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) trap_memory: BTreeMap<Pos, TrapId>,
    /// Walls dug into and still standing, by position (spec §8 "Digging"): the work done on each,
    /// in [`super::dig::WORK_SCALE`]ths of the work its hardness counts.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) dug: BTreeMap<Pos, u32>,
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
            terrain,
            memory,
            stairs: template.stairs.clone(),
            start: template.player_start,
            exit: template.exit.clone(),
            lights: template.lights.clone(),
            flow: template.flow.clone(),
            objects: template.objects.clone(),
            object_memory: BTreeMap::new(),
            fires: BTreeMap::new(),
            traps: template.traps.clone(),
            trap_memory: BTreeMap::new(),
            dug: BTreeMap::new(),
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

    /// The energy price of stepping onto `terrain` here for an actor whose step costs `step`
    /// (its cost of [`crate::content::ActionKind::Step`]). This is how the rules decide movement
    /// time (spec §8 "Time"; [`TerrainDef::step_price`]): on local levels a step is the step's
    /// cost scaled by the terrain's `move_cost` percent over the 6-second base; on the overworld
    /// it is the step's cost over the terrain's `travel_minutes` (content validation guarantees
    /// walkable overworld terrain has them). The actor's speed then decides how long it takes.
    pub fn step_price(&self, terrain: &TerrainDef, step: u32) -> i64 {
        terrain.step_price(step, self.kind)
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

    /// The trap at `p` as it is now, hidden or known (ground truth). Only reachable through
    /// [`crate::World::ground_truth`], **for developer tools only**.
    pub fn trap_at(&self, p: Pos) -> Option<&Trap> {
        self.traps.get(&p)
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

    /// Whether every terrain, light, object, and trap id is valid for `content` (a guard for
    /// loaded saves).
    pub(crate) fn ids_valid_for(&self, content: &Content) -> bool {
        let terrain = content.terrain_defs().len();
        let lights = content.light_presets().len();
        let traps = content.trap_defs().len();
        let known = |t: &TerrainId| usize::from(t.0) < terrain;
        let piles = self.objects.values().chain(self.object_memory.values());
        let mut trap_kinds =
            (self.traps.values().map(|t| t.kind)).chain(self.trap_memory.values().copied());
        self.terrain.cells().iter().all(known)
            && self.memory.cells().iter().flatten().all(known)
            && self.lights.values().all(|l| usize::from(l.0) < lights)
            && self.fires.values().all(|f| usize::from(f.light.0) < lights)
            && piles.flatten().all(|s| s.is_valid_for(content))
            && trap_kinds.all(|t| usize::from(t.0) < traps)
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
