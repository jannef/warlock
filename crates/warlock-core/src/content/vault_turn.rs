//! Which way a vault lies (#959): a vault drawn at random is stamped rotated or mirrored, so the
//! same one looks less the same. A [`Turn`] maps everything in a vault that holds a position or a
//! direction; one that opts out (`turns = false`) or is named by a branch level never turns
//! ([`Vault::may_turn`]).

use super::Flow;
use super::level::{LevelTemplate, Region};
use super::vault::Vault;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use std::collections::BTreeMap;

/// One of the eight ways a rectangle can lie: mirrored left to right or not, then turned clockwise
/// by quarters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    mirror: bool,
    quarters: u8,
}

impl Turn {
    /// The way a file draws it.
    pub const NONE: Turn = Turn::new(false, 0);

    /// All eight, [`Turn::NONE`] first.
    pub const ALL: [Turn; 8] = [
        Turn::NONE,
        Turn::new(false, 1),
        Turn::new(false, 2),
        Turn::new(false, 3),
        Turn::new(true, 0),
        Turn::new(true, 1),
        Turn::new(true, 2),
        Turn::new(true, 3),
    ];

    const fn new(mirror: bool, quarters: u8) -> Self {
        Self { mirror, quarters }
    }

    /// Where cell `p` of a `w`×`h` rectangle goes (`y` grows downward, so a clockwise quarter
    /// takes the top row to the right-hand column).
    pub fn pos(self, p: Pos, w: i32, h: i32) -> Pos {
        let mut q = if self.mirror {
            Pos::new(w - 1 - p.x, p.y)
        } else {
            p
        };
        let (mut width, mut height) = (w, h);
        for _ in 0..self.quarters {
            q = Pos::new(height - 1 - q.y, q.x);
            (width, height) = (height, width);
        }
        q
    }

    /// Which way direction `dir` points afterwards.
    pub fn dir(self, dir: Dir) -> Dir {
        let (mut dx, mut dy) = dir.delta();
        if self.mirror {
            dx = -dx;
        }
        for _ in 0..self.quarters {
            (dx, dy) = (-dy, dx);
        }
        Dir::from_delta(dx, dy).expect("a turned direction is a direction")
    }

    /// The size of a `w`×`h` rectangle afterwards.
    fn size(self, w: i32, h: i32) -> (i32, i32) {
        if self.quarters % 2 == 1 {
            (h, w)
        } else {
            (w, h)
        }
    }
}

/// Row order, the order a vault file's lists are read in.
fn row_order(p: Pos) -> (i32, i32) {
    (p.y, p.x)
}

/// `map` with its keys moved by `at`.
fn moved<T: Clone>(map: &BTreeMap<Pos, T>, at: impl Fn(Pos) -> Pos) -> BTreeMap<Pos, T> {
    map.iter().map(|(p, v)| (at(*p), v.clone())).collect()
}

impl Vault {
    /// Whether it may be stamped any way but the way its file draws it: not if it opts out
    /// (`turns = false`) or a branch level names it.
    pub fn may_turn(&self) -> bool {
        self.turns && !self.pinned
    }

    /// The vault lying as `turn` says, everything in it that holds a position or a direction
    /// turned with the map: what a file drawn that way would load as.
    pub fn turned(&self, turn: Turn) -> Vault {
        let (w, h) = (self.width(), self.height());
        let mut entrances: Vec<(Pos, Dir)> = (self.entrances.iter())
            .map(|(p, out)| (turn.pos(*p, w, h), turn.dir(*out)))
            .collect();
        entrances.sort_by_key(|(p, _)| row_order(*p));
        Vault {
            template: turned_template(&self.template, turn),
            entrances,
            campaign: self.campaign.clone(),
            dungeons: self.dungeons.clone(),
            pinned: self.pinned,
            turns: self.turns,
            danger: self.danger,
            down: (self.down).map(|(p, t)| (turn.pos(p, w, h), t)),
        }
    }
}

/// The template turned. It takes every field by name, so a new one that holds a position or a
/// direction cannot be forgotten here.
fn turned_template(template: &LevelTemplate, turn: Turn) -> LevelTemplate {
    let LevelTemplate {
        id,
        name,
        arrival,
        ambient,
        kind,
        terrain,
        player_start,
        creatures,
        stairs,
        lights,
        flow,
        objects,
        traps,
        locks,
        chests,
        secrets,
        hidden,
        places,
        regions,
        rooms,
        pervading,
        shops,
        exit,
        item_level,
        loot,
        monsters,
        outside_cap,
        temperature,
        ice,
        spawning,
    } = template;
    let (w, h) = (terrain.width(), terrain.height());
    let at = |p: Pos| turn.pos(p, w, h);
    let mut creatures = creatures.clone();
    for spawn in &mut creatures {
        spawn.pos = at(spawn.pos);
        for step in &mut spawn.route {
            *step = turn.dir(*step);
        }
    }
    creatures.sort_by_key(|c| row_order(c.pos));
    let mut stairs = stairs.clone();
    for link in &mut stairs {
        link.pos = at(link.pos);
    }
    let mut traps = moved(traps, at);
    for trap in traps.values_mut() {
        trap.dir = trap.dir.map(|d| turn.dir(d));
    }
    let mut loot = loot.clone();
    for (p, _) in &mut loot {
        *p = at(*p);
    }
    loot.sort_by_key(|(p, _)| row_order(*p));
    let mut monsters = monsters.clone();
    for group in &mut monsters {
        for p in &mut group.cells {
            *p = at(*p);
        }
        group.cells.sort_by_key(|p| row_order(*p));
        for step in &mut group.route {
            *step = turn.dir(*step);
        }
    }
    LevelTemplate {
        id: id.clone(),
        name: name.clone(),
        arrival: arrival.clone(),
        ambient: *ambient,
        kind: *kind,
        terrain: turned_grid(terrain, turn),
        player_start: player_start.map(at),
        creatures,
        stairs,
        lights: moved(lights, at),
        flow: (flow.iter())
            .map(|(p, f)| (at(*p), Flow::new(turn.dir(f.dir), f.speed)))
            .collect(),
        objects: moved(objects, at),
        traps,
        locks: moved(locks, at),
        chests: moved(chests, at),
        secrets: moved(secrets, at),
        hidden: moved(hidden, at),
        places: moved(places, at),
        regions: (regions.iter()).map(|r| turned_region(r, at)).collect(),
        // A vault marks no special rooms and has no shops (`Vault::new`).
        rooms: rooms.clone(),
        pervading: *pervading,
        shops: shops.clone(),
        exit: exit.clone(),
        item_level: *item_level,
        loot,
        monsters,
        outside_cap: outside_cap.iter().map(|p| at(*p)).collect(),
        temperature: *temperature,
        ice: moved(ice, at),
        spawning: spawning.clone(),
    }
}

fn turned_region(region: &Region, at: impl Fn(Pos) -> Pos) -> Region {
    let (a, b) = (at(region.min), at(region.max));
    Region {
        name: region.name.clone(),
        min: Pos::new(a.x.min(b.x), a.y.min(b.y)),
        max: Pos::new(a.x.max(b.x), a.y.max(b.y)),
    }
}

fn turned_grid<T: Clone>(grid: &Grid<T>, turn: Turn) -> Grid<T> {
    let (w, h) = (grid.width(), grid.height());
    let (tw, th) = turn.size(w, h);
    let mut cells: Vec<(Pos, T)> = (grid.iter())
        .map(|(p, t)| (turn.pos(p, w, h), t.clone()))
        .collect();
    cells.sort_by_key(|(p, _)| row_order(*p));
    Grid::from_vec(tw, th, cells.into_iter().map(|(_, t)| t).collect())
}

#[cfg(test)]
#[path = "vault_turn_tests.rs"]
mod tests;
