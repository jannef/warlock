//! Vaults (`assets/vaults/*.toml`): hand-made map pieces in the level-file format that generators
//! stamp into generated levels where they fit (spec §7).
//!
//! A vault is a rectangle copied verbatim: terrain, placed lights, and creatures with their
//! patrols. Its outer edge must be solid (non-walkable terrain such as walls) except for its
//! **entrances**: doors on the edge, never on a corner. Generators join every entrance to the rest
//! of the level. A vault has no stairs and no player start, and every walkable cell must be
//! reachable from an entrance (doors count as passable), so stamping one never strands anything.

use super::Content;
use super::level::LevelTemplate;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;

/// A validated vault.
#[derive(Clone, Debug)]
pub struct Vault {
    pub template: LevelTemplate,
    /// Entrance doors on the edge, each with the direction leading out of the vault.
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
        let is_door = |p: Pos| {
            let def = content.terrain(terrain[p]);
            def.is_closed_door() || def.is_open_door()
        };
        let mut entrances = Vec::new();
        for (p, t) in terrain.iter() {
            let out = if p.y == 0 {
                Some(Dir::N)
            } else if p.y == h - 1 {
                Some(Dir::S)
            } else if p.x == 0 {
                Some(Dir::W)
            } else if p.x == w - 1 {
                Some(Dir::E)
            } else {
                None
            };
            let Some(out) = out else { continue };
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

        // Everything walkable is reachable from the first entrance (4-connected, doors passable).
        let passable = |p: Pos| {
            terrain
                .get(p)
                .is_some_and(|t| content.terrain(*t).walkable || is_door(p))
        };
        let mut seen = Grid::new(w, h, false);
        let mut stack = vec![entrances[0].0];
        seen[entrances[0].0] = true;
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
}
