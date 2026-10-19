//! A local area under construction: terrain, the flow of running water, what is placed on it, and
//! which cells later passes may still change.

use super::kit::Kit;
use crate::content::{Content, CreatureSpawn, Flow, LightId, TerrainId};
use crate::door::Lock;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::magic::LootGrade;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeMap;

/// How much of a cell later passes may change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Claim {
    /// Anything may grow here.
    Free,
    /// Ground cover only (grass, flowers), never an obstacle: around the arrival, around ruins,
    /// along banks.
    Clear,
    /// Laid for good: water, roads, ruins.
    Fixed,
}

#[derive(Clone, Debug)]
pub(super) struct Land<'a> {
    pub content: &'a Content,
    pub kit: Kit,
    /// The area's own ground.
    pub ground: TerrainId,
    pub terrain: Grid<TerrainId>,
    pub claim: Grid<Claim>,
    /// Flow of every running water cell.
    pub flow: BTreeMap<Pos, Flow>,
    pub lights: BTreeMap<Pos, LightId>,
    /// Objects stamped from ruins.
    pub objects: BTreeMap<Pos, Vec<Stack>>,
    /// Where ruins stamped loot to roll, and the least it may be.
    pub loot: Vec<(Pos, LootGrade)>,
    /// Locks on doors stamped from ruins.
    pub locks: BTreeMap<Pos, Lock>,
    pub creatures: Vec<CreatureSpawn>,
}

impl<'a> Land<'a> {
    pub fn new(content: &'a Content, kit: Kit, ground: TerrainId, width: i32, height: i32) -> Self {
        Self {
            content,
            kit,
            ground,
            terrain: Grid::new(width, height, ground),
            claim: Grid::new(width, height, Claim::Free),
            flow: BTreeMap::new(),
            lights: BTreeMap::new(),
            objects: BTreeMap::new(),
            loot: Vec::new(),
            locks: BTreeMap::new(),
            creatures: Vec::new(),
        }
    }

    pub fn width(&self) -> i32 {
        self.terrain.width()
    }

    pub fn height(&self) -> i32 {
        self.terrain.height()
    }

    pub fn center(&self) -> Pos {
        Pos::new(self.width() / 2, self.height() / 2)
    }

    pub fn in_bounds(&self, p: Pos) -> bool {
        self.terrain.in_bounds(p)
    }

    /// On the outermost ring of cells, where walking on leaves the area.
    pub fn on_edge(&self, p: Pos) -> bool {
        p.x == 0 || p.y == 0 || p.x == self.width() - 1 || p.y == self.height() - 1
    }

    pub fn get(&self, p: Pos) -> Option<TerrainId> {
        self.terrain.get(p).copied()
    }

    pub fn claim(&self, p: Pos) -> Claim {
        self.claim.get(p).copied().unwrap_or(Claim::Fixed)
    }

    /// Sets the terrain of `p` (dropping a flow it no longer has).
    pub fn set(&mut self, p: Pos, t: TerrainId) {
        if self.terrain.set(p, t) && t != self.kit.running_water {
            self.flow.remove(&p);
        }
    }

    /// Lays running water flowing `flow` on `p`.
    pub fn set_running(&mut self, p: Pos, flow: Flow) {
        if self.terrain.set(p, self.kit.running_water) {
            self.flow.insert(p, flow);
        }
    }

    pub fn set_claim(&mut self, p: Pos, claim: Claim) {
        self.claim.set(p, claim);
    }

    /// Raises the claim on `p` to at least `claim`.
    pub fn claim_at_least(&mut self, p: Pos, claim: Claim) {
        let raised = match (self.claim(p), claim) {
            (Claim::Fixed, _) | (_, Claim::Free) => return,
            (Claim::Clear, Claim::Clear) => return,
            (_, c) => c,
        };
        self.claim.set(p, raised);
    }

    pub fn is_walkable(&self, p: Pos) -> bool {
        self.get(p)
            .is_some_and(|t| self.content.terrain(t).walkable)
    }

    pub fn is_water(&self, p: Pos) -> bool {
        self.get(p)
            .is_some_and(|t| t == self.kit.water || t == self.kit.running_water)
    }

    /// Walkable and quick to cross: no water, thicket, or reeds.
    pub fn is_dry_footing(&self, p: Pos) -> bool {
        self.get(p).is_some_and(|t| {
            let def = self.content.terrain(t);
            def.walkable && def.move_cost <= 100
        })
    }

    /// Whether an obstacle (tree, rock) may go on `p`.
    pub fn may_block(&self, p: Pos) -> bool {
        self.claim(p) == Claim::Free
    }

    /// Whether ground cover (grass, flowers) may go on `p`.
    pub fn may_cover(&self, p: Pos) -> bool {
        self.claim(p) != Claim::Fixed
    }

    /// Every cell, row by row.
    pub fn cells(&self) -> impl Iterator<Item = Pos> + use<> {
        let (w, h) = (self.width(), self.height());
        (0..h).flat_map(move |y| (0..w).map(move |x| Pos::new(x, y)))
    }

    /// Cells within an ellipse of half-widths `rx` (columns) and `ry` (rows) around `c` whose
    /// rim is pushed in and out by up to `wobble` permille of the radius (from `rng`).
    pub fn blob(&self, rng: &mut Rng, c: Pos, rx: i32, ry: i32, wobble: i32) -> Vec<Pos> {
        let (rx, ry) = (rx.max(1), ry.max(1));
        let bumps = super::noise::field(rng, 2 * rx + 3, 2 * ry + 3, &[(1, 1)]);
        let mut cells = Vec::new();
        for dy in -ry - 1..=ry + 1 {
            for dx in -rx - 1..=rx + 1 {
                let p = Pos::new(c.x + dx, c.y + dy);
                if !self.in_bounds(p) {
                    continue;
                }
                let (u, v) = (
                    i64::from(dx) * 1000 / i64::from(rx),
                    i64::from(dy) * 1000 / i64::from(ry),
                );
                let bump = bumps[Pos::new(dx + rx + 1, dy + ry + 1)] - super::noise::MAX / 2;
                let rim = i64::from(1000 + bump * wobble / 500);
                if u * u + v * v <= rim * rim {
                    cells.push(p);
                }
            }
        }
        cells
    }

    /// Number of the 8 neighbors of `p` with terrain `t`.
    pub fn count_around(&self, p: Pos, t: TerrainId) -> usize {
        Dir::ALL
            .iter()
            .filter(|d| self.get(p.step(**d)) == Some(t))
            .count()
    }
}
