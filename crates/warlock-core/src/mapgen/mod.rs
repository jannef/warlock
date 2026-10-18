//! Procedural level generation (spec §2, §7, §8 Determinism).
//!
//! [`generate`] builds the [`LevelTemplate`] of a dungeon-branch level such as `depths:3` (see
//! [`crate::content::DungeonDef`]), or of a local area of the overworld such as `local:12,30`
//! ([`local`]). The level is a pure function of the content, the run seed, the level id, and (for
//! a branch's first level) the level it was entered from. Branch levels:
//!
//! 1. Draw from the level's own stream, [`Rng::map`]`(seed, id, attempt)`.
//! 2. Build the map with the branch's generator for that depth: `rooms` (rooms joined by
//!    corridors, doors at room entrances) or `caves` (cellular automata). Both stamp
//!    [vaults](crate::content::Vault) where they fit, place the stairs, light sources (torches in
//!    rooms; lamps and glowing fungus in caves), and a few patrolling creatures.
//! 3. Turn rock that touches open ground into walls.
//! 4. Validate (`connect::validate`): every stair, door, and walkable cell is reachable from the
//!    up staircase (flood fill), patrols are valid, and something emits light. A failed attempt is
//!    discarded and the next attempt's stream is used, so retries are deterministic too.
//!
//! Generation is integer-only and never iterates a hash map. Seeds are stable within a game
//! version; the snapshot tests in this module flag any change in generator output.

mod caves;
mod connect;
pub mod local;
mod rooms;
mod vault;

use crate::content::{
    Content, CreatureId, CreatureSpawn, Flow, Generator, LevelKind, LevelTemplate, LightId,
    StairsDir, StairsLink, TerrainId,
};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeMap;

/// Attempts before giving up on a level (each with its own derived stream).
const MAX_ATTEMPTS: u32 = 16;

/// The four directions corridors, tunnels, and flood fills use.
const CARDINALS: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

/// Why a level could not be generated.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GenError {
    #[error("`{0}` is neither a level of a dungeon branch nor a local area of the overworld")]
    NotAGeneratedLevel(String),
    #[error("{0}")]
    Content(String),
    #[error("no valid map after {0} attempts")]
    Exhausted(u32),
}

/// The terrain and light ids generators build with.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Palette {
    /// Solid rock; rock that touches open ground becomes `wall`.
    pub void: TerrainId,
    pub wall: TerrainId,
    pub floor: TerrainId,
    pub door_closed: TerrainId,
    pub door_open: TerrainId,
    pub stairs_up: TerrainId,
    pub stairs_down: TerrainId,
    pub water: TerrainId,
    /// Wall torches lighting rooms.
    pub sconce: TerrainId,
    /// Faintly glowing cave fungus.
    pub fungus: TerrainId,
    /// Hanging lamps lighting caves.
    pub lamp: LightId,
}

impl Palette {
    /// Looks up the ids generators need; content with dungeon branches must provide them.
    pub(crate) fn resolve(content: &Content) -> Result<Self, String> {
        let terrain = |id: &str| {
            content
                .terrain_id(id)
                .ok_or_else(|| format!("procedural generation needs terrain `{id}`"))
        };
        let pal = Palette {
            void: terrain("void")?,
            wall: terrain("wall")?,
            floor: terrain("floor")?,
            door_closed: terrain("door_closed")?,
            door_open: terrain("door_open")?,
            stairs_up: terrain("stairs_up")?,
            stairs_down: terrain("stairs_down")?,
            water: terrain("water")?,
            sconce: terrain("sconce")?,
            fungus: terrain("fungus")?,
            lamp: content
                .light_id("lamp")
                .ok_or("procedural generation needs light `lamp`")?,
        };
        let def = |t: TerrainId| content.terrain(t);
        let checks = [
            (
                !def(pal.void).walkable && !def(pal.wall).walkable,
                "`void` and `wall` must not be walkable",
            ),
            (
                def(pal.floor).walkable && def(pal.water).walkable && def(pal.fungus).walkable,
                "`floor`, `water`, and `fungus` must be walkable",
            ),
            (
                !def(pal.water).flows(),
                "`water` must be still water (generated pools have no flow)",
            ),
            (
                def(pal.door_closed).opens_to == Some(pal.door_open),
                "`door_closed` must open to `door_open`",
            ),
            (
                def(pal.stairs_up).stairs == Some(StairsDir::Up)
                    && def(pal.stairs_down).stairs == Some(StairsDir::Down),
                "`stairs_up` and `stairs_down` must be stairs up and down",
            ),
            (
                def(pal.sconce).transparent && def(pal.sconce).light.is_some(),
                "`sconce` must be transparent and emit light",
            ),
        ];
        match checks.iter().find(|(ok, _)| !ok) {
            Some((_, problem)) => Err(format!("procedural generation: {problem}")),
            None => Ok(pal),
        }
    }
}

/// What a generator must provide besides the map itself.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Needs<'a> {
    /// Whether the level gets a staircase down.
    pub down: bool,
    /// Creatures that may patrol the level.
    pub creatures: &'a [CreatureId],
}

/// A level under construction.
#[derive(Clone, Debug)]
pub(crate) struct Canvas<'a> {
    pub content: &'a Content,
    pub pal: Palette,
    pub terrain: Grid<TerrainId>,
    pub lights: BTreeMap<Pos, LightId>,
    /// Flow of running water stamped from vaults (generators make none of their own).
    pub flow: BTreeMap<Pos, Flow>,
    /// Objects stamped from vaults.
    pub objects: BTreeMap<Pos, Vec<Stack>>,
    pub creatures: Vec<CreatureSpawn>,
    /// Cells stamped from vaults; later passes leave them alone.
    pub locked: Grid<bool>,
    pub up: Option<Pos>,
    pub down: Option<Pos>,
}

impl<'a> Canvas<'a> {
    fn new(content: &'a Content, pal: Palette, width: i32, height: i32) -> Self {
        Self {
            content,
            pal,
            terrain: Grid::new(width, height, pal.void),
            lights: BTreeMap::new(),
            flow: BTreeMap::new(),
            objects: BTreeMap::new(),
            creatures: Vec::new(),
            locked: Grid::new(width, height, false),
            up: None,
            down: None,
        }
    }

    pub fn width(&self) -> i32 {
        self.terrain.width()
    }

    pub fn height(&self) -> i32 {
        self.terrain.height()
    }

    /// Inside the map and not on its outer edge (which always stays solid).
    pub fn is_inner(&self, p: Pos) -> bool {
        p.x > 0 && p.y > 0 && p.x < self.width() - 1 && p.y < self.height() - 1
    }

    /// Terrain at `p`; rock outside the map.
    pub fn get(&self, p: Pos) -> TerrainId {
        self.terrain.get(p).copied().unwrap_or(self.pal.void)
    }

    pub fn is_rock(&self, p: Pos) -> bool {
        let t = self.get(p);
        t == self.pal.void || t == self.pal.wall
    }

    pub fn is_locked(&self, p: Pos) -> bool {
        self.locked.get(p).copied().unwrap_or(false)
    }

    /// Walkable, or a door the player can open.
    pub fn is_passable(&self, p: Pos) -> bool {
        self.terrain.get(p).is_some_and(|t| {
            let def = self.content.terrain(*t);
            def.walkable || def.is_closed_door()
        })
    }

    pub fn is_walkable(&self, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_some_and(|t| self.content.terrain(*t).walkable)
    }

    /// Plain floor nothing has claimed yet: no vault, stairs, light, or creature.
    pub fn is_free_floor(&self, p: Pos) -> bool {
        self.get(p) == self.pal.floor
            && !self.is_locked(p)
            && !self.lights.contains_key(&p)
            && !self.creatures.iter().any(|c| c.pos == p)
    }

    /// The 3×3 block around `p` is free floor (a roomy spot for stairs).
    pub fn is_open_area(&self, p: Pos) -> bool {
        (-1..=1).all(|dy| (-1..=1).all(|dx| self.is_free_floor(Pos::new(p.x + dx, p.y + dy))))
    }

    pub fn set(&mut self, p: Pos, t: TerrainId) {
        self.terrain.set(p, t);
    }

    /// Every cell, row by row.
    pub fn cells(&self) -> impl Iterator<Item = Pos> + use<> {
        let (w, h) = (self.width(), self.height());
        (0..h).flat_map(move |y| (0..w).map(move |x| Pos::new(x, y)))
    }

    /// Places a staircase; the up staircase is also where the player arrives.
    pub fn place_stairs(&mut self, p: Pos, dir: StairsDir) {
        match dir {
            StairsDir::Up => {
                self.set(p, self.pal.stairs_up);
                self.up = Some(p);
            }
            StairsDir::Down => {
                self.set(p, self.pal.stairs_down);
                self.down = Some(p);
            }
        }
    }

    /// Spawns `creature` at `start` patrolling `route` if every step stays on walkable terrain
    /// and the route loops back; returns whether it was placed.
    pub fn add_patrol(&mut self, creature: CreatureId, start: Pos, route: Vec<Dir>) -> bool {
        let mut p = start;
        for dir in &route {
            p = p.step(*dir);
            if !self.is_walkable(p) {
                return false;
            }
        }
        if p != start || !self.is_free_floor(start) {
            return false;
        }
        self.creatures.push(CreatureSpawn {
            creature,
            pos: start,
            route,
            carry: Vec::new(),
        });
        true
    }

    /// Rock that touches open ground (8-neighborhood) becomes wall; deeper rock stays void.
    fn build_walls(&mut self) {
        let mut walls = Vec::new();
        for p in self.cells() {
            if self.get(p) != self.pal.void || self.is_locked(p) {
                continue;
            }
            if Dir::ALL.iter().any(|d| !self.is_rock(p.step(*d))) {
                walls.push(p);
            }
        }
        for p in walls {
            self.set(p, self.pal.wall);
        }
    }
}

/// Generates the level `id` for the run with `seed`: a branch level (e.g. `depths:3`; on a
/// branch's first level the up staircase leads to `entered_from`, or nowhere if `None`) or a
/// local area (`local:12,30`).
pub fn generate(
    content: &Content,
    seed: u64,
    id: &str,
    entered_from: Option<&str>,
) -> Result<LevelTemplate, GenError> {
    if content.local_area(id).is_some() {
        return local::generate(content, seed, id);
    }
    let (def, depth) = content
        .branch_level(id)
        .ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    let plan = def
        .plan(depth)
        .ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    let pal = Palette::resolve(content).map_err(GenError::Content)?;
    let needs = Needs {
        down: depth < def.depth,
        creatures: &def.creatures,
    };
    let up_to = if depth > 1 {
        Some(def.level_id(depth - 1))
    } else {
        entered_from.map(str::to_string)
    };
    let down_to = needs.down.then(|| def.level_id(depth + 1));

    for attempt in 0..MAX_ATTEMPTS {
        let mut rng = Rng::map(seed, id, attempt);
        let mut canvas = Canvas::new(content, pal, plan.width, plan.height);
        let built = match plan.generator {
            Generator::Rooms => rooms::build(&mut canvas, &mut rng, needs),
            Generator::Caves => caves::build(&mut canvas, &mut rng, needs),
        };
        if built.is_none() {
            continue;
        }
        canvas.build_walls();
        let (Some(up), down) = (canvas.up, canvas.down) else {
            continue;
        };
        if down.is_some() != needs.down {
            continue;
        }
        let mut stairs = vec![StairsLink {
            pos: up,
            dir: StairsDir::Up,
            to: up_to.clone(),
        }];
        if let Some(pos) = down {
            stairs.push(StairsLink {
                pos,
                dir: StairsDir::Down,
                to: down_to.clone(),
            });
        }
        let template = LevelTemplate {
            id: id.to_string(),
            name: def.level_name(depth),
            ambient: def.ambient,
            kind: LevelKind::Dungeon,
            terrain: canvas.terrain,
            player_start: Some(up),
            creatures: canvas.creatures,
            stairs,
            lights: canvas.lights,
            flow: canvas.flow,
            objects: canvas.objects,
            regions: Vec::new(),
            exit: None,
        };
        if connect::validate(&template, content).is_ok() {
            return Ok(template);
        }
    }
    Err(GenError::Exhausted(MAX_ATTEMPTS))
}

/// The template as text: terrain glyphs, the top object of each pile over them, creatures on top,
/// and `*` for a light placed on floor. Trailing spaces are trimmed. Used by snapshot tests and
/// `warlock-cli gen`.
pub fn ascii(template: &LevelTemplate, content: &Content) -> String {
    let floor = content.terrain_id("floor");
    let mut out = String::new();
    for y in 0..template.terrain.height() {
        let row: String = (0..template.terrain.width())
            .map(|x| {
                let p = Pos::new(x, y);
                let t = template.terrain[p];
                if let Some(c) = template.creatures.iter().find(|c| c.pos == p) {
                    content.creature(c.creature).glyph
                } else if let Some(top) = template.objects.get(&p).and_then(|o| o.first()) {
                    content.object(top.object).glyph
                } else if template.lights.contains_key(&p) && Some(t) == floor {
                    '*'
                } else {
                    content.terrain(t).glyph
                }
            })
            .collect();
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}

/// A random-walk patrol from `start` that goes out up to `len` steps over cells `ok` accepts and
/// returns along the same cells, so it always loops. `None` if it cannot get 3 steps out.
pub(crate) fn out_and_back(
    rng: &mut Rng,
    start: Pos,
    len: i32,
    ok: impl Fn(Pos) -> bool,
) -> Option<Vec<Dir>> {
    let mut steps: Vec<Dir> = Vec::new();
    let mut p = start;
    let mut dir = CARDINALS[rng.index(4)];
    for _ in 0..len {
        if rng.chance(25) {
            dir = CARDINALS[rng.index(4)];
        }
        let turn = CARDINALS.iter().position(|d| *d == dir).unwrap_or(0);
        let next = (0..4)
            .map(|k| CARDINALS[(turn + k) % 4])
            .find(|d| ok(p.step(*d)));
        let Some(d) = next else { break };
        p = p.step(d);
        steps.push(d);
        dir = d;
    }
    if steps.len() < 3 {
        return None;
    }
    let back: Vec<Dir> = steps.iter().rev().map(|d| d.opposite()).collect();
    steps.extend(back);
    Some(steps)
}

#[cfg(test)]
mod tests;
