//! Vaults (`assets/vaults/*.toml`): hand-made map pieces in the level-file format that generators
//! stamp into generated levels where they fit (spec §7).
//!
//! A vault is a rectangle copied verbatim: terrain, placed lights, objects, traps, and creatures
//! with their patrols. Its outer edge must be solid (non-walkable terrain such as walls) except for its
//! **entrances**: doors on the edge, never on a corner. Generators join every entrance to the rest
//! of the level. A vault has no stairs and no player start, and every walkable cell must be
//! reachable from an entrance (doors count as passable), so stamping one never strands anything.
//!
//! A vault of kind `outdoor` (ruins for the local areas of the overworld) is stamped onto open
//! ground instead of into rock: its `void` cells (the default legend's space) are the local
//! area's own ground, every open cell on its edge (walkable, a door, or void) is an entrance, and
//! every walkable cell must be reachable from its edge. Dungeon generators use only dungeon
//! vaults, local areas only outdoor ones.

use super::level::{LevelKind, LevelTemplate};
use super::{Content, TerrainId};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;

/// A validated vault.
#[derive(Clone, Debug)]
pub struct Vault {
    pub template: LevelTemplate,
    /// Entrance doors on the edge (outdoor vaults: every open edge cell), each with the
    /// direction leading out of the vault.
    pub entrances: Vec<(Pos, Dir)>,
}

impl Vault {
    /// Checks the vault rules for a parsed vault file.
    pub(super) fn new(template: LevelTemplate, content: &Content) -> Result<Self, String> {
        let terrain = &template.terrain;
        let (w, h) = (terrain.width(), terrain.height());
        if w < 3 || h < 3 {
            return Err(format!("a vault must be at least 3x3, not {w}x{h}"));
        }
        if template.player_start.is_some() {
            return Err("a vault cannot have a player start".into());
        }
        if let Some(stairs) = template.stairs.first() {
            return Err(format!("a vault cannot have stairs (at {})", stairs.pos));
        }
        if let (LevelKind::Outdoor, Some(at)) = (template.kind, template.traps.keys().next()) {
            return Err(format!("an outdoor vault cannot have traps (at {at})"));
        }
        let is_door = |p: Pos| {
            let def = content.terrain(terrain[p]);
            def.is_closed_door() || def.is_open_door()
        };
        let entrances = match template.kind {
            LevelKind::Dungeon => door_entrances(&template, content)?,
            LevelKind::Outdoor => open_edges(&template, content)?,
            LevelKind::Overworld => return Err("a vault cannot be of kind `overworld`".into()),
        };

        // Everything walkable is reachable from the first entrance (4-connected, doors passable);
        // in outdoor vaults from any open edge cell, over the open ground too.
        let ground = outdoor_ground(&template, content);
        let passable = |p: Pos| {
            terrain
                .get(p)
                .is_some_and(|t| content.terrain(*t).walkable || is_door(p) || ground(*t))
        };
        let seeds = match template.kind {
            LevelKind::Outdoor => entrances.len(),
            _ => 1,
        };
        let mut seen = Grid::new(w, h, false);
        let mut stack: Vec<Pos> = entrances[..seeds].iter().map(|(p, _)| *p).collect();
        for p in &stack {
            seen[*p] = true;
        }
        while let Some(p) = stack.pop() {
            for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
                let q = p.step(dir);
                if passable(q) && !seen[q] {
                    seen[q] = true;
                    stack.push(q);
                }
            }
        }
        if let Some((p, _)) = terrain.iter().find(|(p, _)| passable(*p) && !seen[*p]) {
            return Err(format!("{p} cannot be reached from the entrances"));
        }
        Ok(Self {
            template,
            entrances,
        })
    }

    pub fn id(&self) -> &str {
        &self.template.id
    }

    pub fn width(&self) -> i32 {
        self.template.terrain.width()
    }

    pub fn height(&self) -> i32 {
        self.template.terrain.height()
    }

    /// Whether the vault is for the local areas of the overworld (kind `outdoor`).
    pub fn is_outdoor(&self) -> bool {
        self.template.kind == LevelKind::Outdoor
    }
}

/// Whether a terrain is an outdoor vault's open ground: `void`, which takes the local area's own
/// ground when stamped. Never in a dungeon vault.
fn outdoor_ground(
    template: &LevelTemplate,
    content: &Content,
) -> impl Fn(TerrainId) -> bool + use<> {
    let void = content
        .terrain_id("void")
        .filter(|_| template.kind == LevelKind::Outdoor);
    move |t| Some(t) == void
}

/// The outward direction of an edge cell of a `w`×`h` vault (`None` inside).
fn outward(p: Pos, w: i32, h: i32) -> Option<Dir> {
    if p.y == 0 {
        Some(Dir::N)
    } else if p.y == h - 1 {
        Some(Dir::S)
    } else if p.x == 0 {
        Some(Dir::W)
    } else if p.x == w - 1 {
        Some(Dir::E)
    } else {
        None
    }
}

/// Outdoor vaults: every open edge cell (walkable, a door, or open ground) is an entrance.
fn open_edges(template: &LevelTemplate, content: &Content) -> Result<Vec<(Pos, Dir)>, String> {
    let terrain = &template.terrain;
    let ground = outdoor_ground(template, content);
    let entrances: Vec<(Pos, Dir)> = terrain
        .iter()
        .filter(|(_, t)| {
            let def = content.terrain(**t);
            def.walkable || def.is_closed_door() || ground(**t)
        })
        .filter_map(|(p, _)| outward(p, terrain.width(), terrain.height()).map(|d| (p, d)))
        .collect();
    if entrances.is_empty() {
        return Err("an outdoor vault needs an open cell on its edge".into());
    }
    Ok(entrances)
}

/// Dungeon vaults: the edge is solid except for entrance doors, never on a corner.
fn door_entrances(template: &LevelTemplate, content: &Content) -> Result<Vec<(Pos, Dir)>, String> {
    let terrain = &template.terrain;
    let (w, h) = (terrain.width(), terrain.height());
    let is_door = |p: Pos| {
        let def = content.terrain(terrain[p]);
        def.is_closed_door() || def.is_open_door()
    };
    let mut entrances = Vec::new();
    for (p, t) in terrain.iter() {
        let Some(out) = outward(p, w, h) else {
            continue;
        };
        let corner = (p.x == 0 || p.x == w - 1) && (p.y == 0 || p.y == h - 1);
        if is_door(p) && !corner {
            entrances.push((p, out));
        } else if content.terrain(*t).walkable || is_door(p) {
            return Err(format!(
                "edge cell {p} must be solid or an entrance door (not on a corner)"
            ));
        }
    }
    if entrances.is_empty() {
        return Err("a vault needs at least one entrance: a door on its edge".into());
    }
    Ok(entrances)
}
