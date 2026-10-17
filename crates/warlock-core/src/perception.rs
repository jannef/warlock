//! The per-turn perception snapshot (spec §8).
//!
//! Everything the player can know — and therefore everything rendering, UI, and dumps may show —
//! goes through [`PerceptionView`]. It exposes currently visible cells and creatures, and
//! remembered terrain; it never exposes unseen terrain or hidden creatures.

use crate::content::{Content, CreatureId, TerrainId};
use crate::fov::compute_fov;
use crate::geom::Pos;
use crate::grid::Grid;
use crate::world::{ActorId, ActorKind, LevelState, WorldState};
use serde::Serialize;

/// Maximum sight distance until lighting decides visibility (M3).
pub const VIEW_RADIUS: i32 = 30;

/// What the player knows about one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "terrain")]
pub enum CellView {
    /// Never seen.
    Unknown,
    /// Seen before; terrain as it was last seen. Never includes creatures or effects.
    Remembered(TerrainId),
    /// In view right now.
    Visible(TerrainId),
}

/// A creature the player currently sees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SeenActor {
    pub id: ActorId,
    pub kind: ActorKind,
    pub creature: CreatureId,
    pub pos: Pos,
}

/// Derived each turn from world state; not saved.
#[derive(Clone, Debug)]
pub(crate) struct Perception {
    viewer: Pos,
    visible: Grid<bool>,
    actors: Vec<SeenActor>,
}

impl Perception {
    /// Computes what the player perceives on their current level.
    pub(crate) fn compute(state: &WorldState, content: &Content) -> Self {
        let player = state.player();
        let level = state.current();
        let mut visible = Grid::new(level.width(), level.height(), false);
        compute_fov(
            player.pos,
            VIEW_RADIUS,
            |p| level.is_opaque(content, p),
            |p| {
                visible.set(p, true);
            },
        );
        let actors = state
            .actors
            .values()
            .filter(|a| a.level == level.id && visible.get(a.pos) == Some(&true))
            .map(|a| SeenActor {
                id: a.id,
                kind: a.kind,
                creature: a.creature,
                pos: a.pos,
            })
            .collect();
        Self {
            viewer: player.pos,
            visible,
            actors,
        }
    }

    pub(crate) fn is_visible(&self, p: Pos) -> bool {
        self.visible.get(p) == Some(&true)
    }

    /// Writes currently visible terrain into the level's memory.
    pub(crate) fn remember(&self, level: &mut LevelState) {
        for (p, seen) in self.visible.iter() {
            if *seen {
                level.memory.set(p, Some(level.terrain[p]));
            }
        }
    }
}

/// Read-only view of what the player perceives. Obtain one with [`crate::World::perception`].
#[derive(Clone, Copy, Debug)]
pub struct PerceptionView<'a> {
    pub(crate) content: &'a Content,
    pub(crate) level: &'a LevelState,
    pub(crate) snapshot: &'a Perception,
}

impl<'a> PerceptionView<'a> {
    pub fn content(&self) -> &'a Content {
        self.content
    }

    pub fn level_id(&self) -> &'a str {
        &self.level.id
    }

    pub fn level_name(&self) -> &'a str {
        &self.level.name
    }

    pub fn width(&self) -> i32 {
        self.level.width()
    }

    pub fn height(&self) -> i32 {
        self.level.height()
    }

    /// Where the player is looking from.
    pub fn viewer(&self) -> Pos {
        self.snapshot.viewer
    }

    pub fn cell(&self, p: Pos) -> CellView {
        if self.snapshot.is_visible(p) {
            CellView::Visible(self.level.terrain[p])
        } else {
            match self.level.memory.get(p) {
                Some(Some(t)) => CellView::Remembered(*t),
                _ => CellView::Unknown,
            }
        }
    }

    pub fn is_visible(&self, p: Pos) -> bool {
        self.snapshot.is_visible(p)
    }

    /// Creatures in view, in ascending id order (the player included).
    pub fn actors(&self) -> &'a [SeenActor] {
        &self.snapshot.actors
    }

    pub fn actor_at(&self, p: Pos) -> Option<&'a SeenActor> {
        self.snapshot.actors.iter().find(|a| a.pos == p)
    }
}
