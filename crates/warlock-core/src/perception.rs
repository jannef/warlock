//! The per-turn perception snapshot (spec §5, §8).
//!
//! Everything the player can know — and therefore everything rendering, UI, and dumps may show —
//! goes through [`PerceptionView`]. It exposes currently visible cells, creatures, and light, and
//! remembered terrain; it never exposes unseen terrain, hidden creatures, or light the player's
//! eyes cannot perceive.
//!
//! A cell is **visible** iff it is in line of sight within [`VIEW_RADIUS`] **and** its perceived
//! light is at least the vision profile's threshold or it lies within the darkvision radius. The
//! player's own cell is always visible.
//!
//! Light on a transparent cell is the level's ambient light, plus the light of every source that
//! reaches it, plus weak **bounce light** reflected by the lit surfaces around it
//! ([`crate::light::BOUNCE`]). Ambient plus bounce is the cell's *base light*: steady, and not
//! tied to one source.
//!
//! Opaque cells follow two extra rules:
//! - **Wall faces**: an opaque cell is lit by a source as a transparent cell at its own distance
//!   would be, but only if the viewer sees a face the source lights (an adjacent visible
//!   transparent cell receives the same source), so light never leaks through a wall to a viewer
//!   on its dark side. Its bounce light is the brightest of its visible transparent neighbors'.
//! - **Wall completion**: an opaque cell within view range that bounds a transparent cell seen by
//!   light or darkvision is visible even outside line of sight (symmetric shadowcasting can show
//!   floor through a narrow gap without the walls around it). Such a cell shows only its terrain
//!   and wall-face light: never creatures, and nothing beyond it.

use crate::content::{Content, CreatureId, Flow, LightId, TerrainId, VisionProfile};
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::light::{
    Area, BOUNCE_STEPS, Bounce, Light, LightCache, falloff, lit_in_sight, wall_faces,
};
use crate::world::{ActorId, ActorKind, LevelState, WorldState};
use serde::Serialize;

/// Maximum sight distance, however bright the light.
pub const VIEW_RADIUS: i32 = 40;

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

/// Light as the player perceives it at a visible cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PerceivedLight {
    /// Red, green, blue: 255 = fully lit, more = overbright.
    pub rgb: [u16; 3],
    /// Ultraviolet; always 0 unless the vision profile sees UV.
    pub uv: u16,
    /// Seen only thanks to darkvision: too dark for the eye's threshold.
    pub darkvision: bool,
    /// Squared distance from the viewer.
    pub dist2: i32,
}

impl PerceivedLight {
    /// The light value without the viewer-relative fields.
    pub fn light(&self) -> Light {
        Light {
            rgb: self.rgb,
            uv: self.uv,
        }
    }

    /// Brightness compared against the vision threshold (luma, plus UV if perceived).
    pub fn brightness(&self) -> u16 {
        self.light().brightness()
    }
}

/// A light source whose light reaches cells the player sees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SeenLight {
    /// Stable across turns (derived from level and position for static lights, from the actor for
    /// carried ones), e.g. to seed visual flicker.
    pub key: u64,
    /// Where the source is; it may itself be out of view (light from around a corner).
    pub pos: Pos,
    /// The source's preset (color, radius, flicker).
    pub preset: LightId,
    /// Perceived light this source adds to each visible cell it reaches, wall-face rule applied.
    /// Summed over all sources plus [`PerceptionView::base_light`] this gives
    /// [`PerceptionView::light`].
    pub contributions: Vec<(Pos, Light)>,
}

/// How the player sees a cell this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sight {
    Hidden,
    /// In line of sight and visible.
    Seen,
    /// An opaque cell outside line of sight, visible by wall completion: terrain only.
    Edge,
}

/// Derived each turn from world state; not saved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Perception {
    viewer: Pos,
    /// Index into [`Content::vision_profiles`].
    vision: usize,
    /// Perceived ambient light.
    ambient: Light,
    /// Ambient plus bounce light; meaningful on visible cells only.
    base: Grid<Light>,
    sight: Grid<Sight>,
    /// Visible cells in the order they were found.
    visible_cells: Vec<Pos>,
    /// Perceived light; meaningful on visible cells only.
    light: Grid<Light>,
    sources: Vec<SeenLight>,
    actors: Vec<SeenActor>,
}

impl Perception {
    /// Computes what the player perceives on their current level: line of sight, light, and
    /// visibility. Light masks come from (and are added to) `lights`.
    pub(crate) fn compute(state: &WorldState, content: &Content, lights: &mut LightCache) -> Self {
        let viewer = state.player().pos;
        let level = state.current();
        let (width, height) = (level.width(), level.height());
        let vision = content.vision_index(&state.vision).unwrap_or(0);
        let eye = &content.vision_profiles()[vision];
        let is_opaque = |p: Pos| level.is_opaque(content, p);

        // Line of sight.
        let mut los = Grid::new(width, height, false);
        let mut los_cells = Vec::new();
        compute_fov(viewer, VIEW_RADIUS, is_opaque, |p| {
            if let Some(seen) = los.get_mut(p)
                && !*seen
            {
                *seen = true;
                los_cells.push(p);
            }
        });

        // Sources that can light a cell in view, or a surface whose bounce light reaches one (a
        // bounce step moves at most √2 < 2 cells; wall faces are within a cell of view).
        let near = lights.near(state, content, viewer, VIEW_RADIUS + 1 + 2 * BOUNCE_STEPS);

        // Base light on cells in line of sight: ambient plus bounce (on transparent cells; RGB
        // only, so every eye perceives all of it).
        let ambient = eye.perceive(level.ambient);
        let bounce =
            Area::around(los_cells.iter().copied(), BOUNCE_STEPS, width, height).map(|area| {
                let tint = |p: Pos| {
                    let rgb = level.terrain.get(p).map(|t| content.terrain(*t).bounce);
                    rgb.map_or([0; 3], |c| [c.r, c.g, c.b])
                };
                Bounce::compute(area, near.iter().map(|(_, mask)| *mask), is_opaque, tint)
            });
        let indirect = |p: Pos| bounce.as_ref().map_or(Light::DARK, |b| b.at(p));
        let mut base = Grid::new(width, height, Light::DARK);
        for &p in &los_cells {
            base[p] = ambient.saturating_add(indirect(p));
        }

        // Light on cells in line of sight: base light plus every source reaching them.
        let mut light = base.clone();
        let lit: Vec<_> = near
            .into_iter()
            .map(|(source, mask)| (source, lit_in_sight(mask, &los, |l| eye.perceive(l))))
            .collect();
        for (_, cells) in &lit {
            for &(p, c) in cells {
                light[p] = light[p].saturating_add(c);
            }
        }

        // Transparent cells: visible with enough light or within darkvision; the viewer's own
        // cell always.
        let mut sight = Grid::new(width, height, Sight::Hidden);
        let mut visible_cells = Vec::new();
        let mut seen = Vec::new();
        for &p in los_cells.iter().filter(|p| !is_opaque(**p)) {
            let by_eye = eye.sees(light[p], p.dist2(viewer));
            if by_eye || p == viewer {
                sight[p] = Sight::Seen;
                visible_cells.push(p);
            }
            if by_eye {
                seen.push(p);
            }
        }

        // Wall completion: opaque cells bounding a transparent cell seen by eye.
        let reach2 = radius2(VIEW_RADIUS);
        for q in seen {
            for w in Dir::ALL.map(|d| q.step(d)) {
                if sight.get(w) == Some(&Sight::Hidden) && is_opaque(w) && w.dist2(viewer) <= reach2
                {
                    sight[w] = if los[w] { Sight::Seen } else { Sight::Edge };
                    visible_cells.push(w);
                }
            }
        }

        // Wall faces: opaque cells in line of sight or completed take the brightest bounce light
        // of their visible transparent neighbors, then each source's light at their own distance
        // if they face a visible cell it lights.
        let is_seen = |q: Pos| sight.get(q) == Some(&Sight::Seen);
        let is_face = |w: Pos| {
            is_opaque(w) && (los.get(w) == Some(&true) || sight.get(w) == Some(&Sight::Edge))
        };
        for &w in los_cells.iter().chain(&visible_cells) {
            if is_face(w) {
                let from_neighbors = Dir::ALL
                    .map(|d| w.step(d))
                    .into_iter()
                    .filter(|q| is_seen(*q) && !is_opaque(*q))
                    .fold(Light::DARK, |acc, q| acc.max(indirect(q)));
                base[w] = ambient.saturating_add(from_neighbors);
                light[w] = base[w];
            }
        }
        let lit: Vec<_> = lit
            .into_iter()
            .map(|(source, cells)| {
                let preset = content.light(source.preset);
                let (color, radius) = (preset.color, preset.radius.max(0));
                let light_at = |w: Pos| eye.perceive(falloff(color, radius, w.dist2(source.pos)));
                let faces = wall_faces(&cells, light_at, is_seen, is_face);
                (source, cells, faces)
            })
            .collect();
        for (_, _, faces) in &lit {
            for &(p, c) in faces {
                light[p] = light[p].saturating_add(c);
            }
        }

        // Other opaque cells in line of sight: visible by their face light or darkvision.
        for &p in &los_cells {
            if sight[p] == Sight::Hidden && is_opaque(p) && eye.sees(light[p], p.dist2(viewer)) {
                sight[p] = Sight::Seen;
                visible_cells.push(p);
            }
        }

        let is_visible = |p: Pos| sight.get(p).is_some_and(|s| *s != Sight::Hidden);
        let sources = lit
            .into_iter()
            .filter_map(|(source, cells, faces)| {
                let contributions: Vec<_> = cells
                    .into_iter()
                    .chain(faces)
                    .filter(|(p, _)| is_visible(*p))
                    .collect();
                (!contributions.is_empty()).then_some(SeenLight {
                    key: source.key,
                    pos: source.pos,
                    preset: source.preset,
                    contributions,
                })
            })
            .collect();

        let actors = state
            .actors
            .values()
            .filter(|a| a.level == level.id && sight.get(a.pos) == Some(&Sight::Seen))
            .map(|a| SeenActor {
                id: a.id,
                kind: a.kind,
                creature: a.creature,
                pos: a.pos,
            })
            .collect();
        Self {
            viewer,
            vision,
            ambient,
            base,
            sight,
            visible_cells,
            light,
            sources,
            actors,
        }
    }

    /// Whether the cell's terrain is visible (in sight, or a completed wall).
    pub(crate) fn is_visible(&self, p: Pos) -> bool {
        self.sight.get(p).is_some_and(|s| *s != Sight::Hidden)
    }

    /// Whether creatures at `p` can be seen: only in line of sight, never on completed walls.
    pub(crate) fn sees_actors_at(&self, p: Pos) -> bool {
        self.sight.get(p) == Some(&Sight::Seen)
    }

    /// Writes currently visible terrain into the level's memory.
    pub(crate) fn remember(&self, level: &mut LevelState) {
        for &p in &self.visible_cells {
            level.memory.set(p, Some(level.terrain[p]));
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

    /// The flow of running water at a visible cell (presentation-only); `None` for cells that are
    /// not in view or not running water.
    pub fn flow(&self, p: Pos) -> Option<Flow> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        self.level.flow.get(&p).copied()
    }

    /// Creatures in view, in ascending id order (the player included).
    pub fn actors(&self) -> &'a [SeenActor] {
        &self.snapshot.actors
    }

    pub fn actor_at(&self, p: Pos) -> Option<&'a SeenActor> {
        self.snapshot.actors.iter().find(|a| a.pos == p)
    }

    /// Perceived light at a visible cell (wall-face rule applied); `None` for cells not in view.
    pub fn light(&self, p: Pos) -> Option<PerceivedLight> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        let light = self.snapshot.light[p];
        let vision = self.vision();
        Some(PerceivedLight {
            rgb: light.rgb,
            uv: light.uv,
            darkvision: vision.darkvision > 0 && light.brightness() < vision.threshold,
            dist2: p.dist2(self.snapshot.viewer),
        })
    }

    /// Light sources whose light reaches visible cells: static lights by position, then carried
    /// lights by actor id.
    pub fn light_sources(&self) -> &'a [SeenLight] {
        &self.snapshot.sources
    }

    /// The player's active vision profile.
    pub fn vision(&self) -> &'a VisionProfile {
        &self.content.vision_profiles()[self.snapshot.vision]
    }

    /// The level's ambient light, as the player perceives it.
    pub fn ambient(&self) -> Light {
        self.snapshot.ambient
    }

    /// Steady light at a visible cell that no single source gives: ambient light plus bounce
    /// light; `None` for cells not in view. With every [`SeenLight`] contribution to the cell
    /// added, it is [`PerceptionView::light`].
    pub fn base_light(&self, p: Pos) -> Option<Light> {
        self.snapshot.is_visible(p).then(|| self.snapshot.base[p])
    }
}
