//! Procedural level generation (spec §2, §7, §8 Determinism).
//!
//! [`generate`] builds the [`LevelTemplate`] of a dungeon-branch level such as `depths:3` (see
//! [`crate::content::DungeonDef`]), or of a local area of the overworld such as `local:12,30`
//! ([`local`]). The level is a pure function of the content, the run seed, the level id, and (for
//! a branch's first level) the level it was entered from. Branch levels:
//!
//! 1. Draw from the level's own stream, [`Rng::map`]`(seed, id, attempt)`.
//! 2. Build the map with the branch's generator for that depth: `rooms` (rooms joined by
//!    corridors, doors at room entrances; with the branch's `key`, a few rooms locked and the
//!    key left where the player can reach it) or `caves` (cellular automata). Both stamp
//!    [vaults](crate::content::Vault) where they fit (with their locks and keys), place the
//!    stairs, light sources (torches in rooms; lamps and glowing fungus in caves), and a few
//!    patrolling creatures, and last now and then make a room (a cave chamber) special
//!    (`special.rs`, spec §7 "Special rooms") and, rarer, a room of a rooms level a shop
//!    (`shop.rs`, spec §7 "Shops").
//! 3. Turn rock that touches open ground into walls.
//! 4. Validate (`connect::validate`): every stair, door, and walkable cell is reachable from the
//!    up staircase (flood fill), the stairs and a key to every lock without unlocking a door,
//!    patrols are valid, and something emits light. A failed attempt is discarded and the next
//!    attempt's stream is used, so retries are deterministic too.
//! 5. Hide the branch's traps (spec §8 "Traps") from the level's trap stream ([`Rng::traps`]),
//!    then a few of its doors (secret doors, spec §8 "Searching") from its secret stream
//!    ([`Rng::secrets`]), only where the stairs and every key are still reached without them,
//!    then roll its loot from its loot stream ([`Rng::loot`]).
//!
//! Generation is integer-only and never iterates a hash map. Seeds are stable within a game
//! version; the snapshot tests in this module flag any change in generator output.

mod caves;
mod connect;
pub mod local;
mod rooms;
mod shop;
mod special;
mod vault;

use crate::content::{
    Content, CreatureId, CreatureSpawn, Flow, Generator, LevelKind, LevelTemplate, LightId,
    ObjectId, Secret, ShopPlan, SpecialRoom, StairsDir, StairsLink, TerrainId, Trap, secret_look,
};
use crate::door::Lock;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::magic::{LootGrade, loot};
use crate::object::{Stack, add_to_pile};
use crate::rng::Rng;
use crate::variant::Source;
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
    /// The key to the rooms the level locks (the branch's); `None`: it locks none.
    pub key: Option<ObjectId>,
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
    /// Objects stamped from vaults, and keys to locked rooms.
    pub objects: BTreeMap<Pos, Vec<Stack>>,
    /// Traps stamped from vaults.
    pub traps: BTreeMap<Pos, Trap>,
    /// Where vaults stamped loot to roll, and the least it may be.
    pub loot: Vec<(Pos, LootGrade)>,
    /// Locks on closed doors: stamped from vaults, and on locked rooms (spec §8 "Doors").
    pub locks: BTreeMap<Pos, Lock>,
    /// Secret doors stamped from vaults (spec §8 "Searching").
    pub secrets: BTreeMap<Pos, Secret>,
    pub creatures: Vec<CreatureSpawn>,
    /// Special rooms (spec §7 "Special rooms"; `special.rs`).
    pub rooms: Vec<SpecialRoom>,
    /// Shops (spec §7 "Shops"; `shop.rs`), their keepers among `creatures`.
    pub shops: Vec<ShopPlan>,
    /// Cells stamped from vaults; later passes leave them alone.
    pub stamped: Grid<bool>,
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
            traps: BTreeMap::new(),
            loot: Vec::new(),
            locks: BTreeMap::new(),
            secrets: BTreeMap::new(),
            creatures: Vec::new(),
            rooms: Vec::new(),
            shops: Vec::new(),
            stamped: Grid::new(width, height, false),
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

    /// Whether a vault was stamped on `p`.
    pub fn is_stamped(&self, p: Pos) -> bool {
        self.stamped.get(p).copied().unwrap_or(false)
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
            && !self.is_stamped(p)
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
            faction: None,
            alignment: None,
            npc: None,
            pos: start,
            route,
            carry: Vec::new(),
            equipment: Default::default(),
        });
        true
    }

    /// Rock that touches open ground (8-neighborhood) becomes wall; deeper rock stays void.
    fn build_walls(&mut self) {
        let mut walls = Vec::new();
        for p in self.cells() {
            if self.get(p) != self.pal.void || self.is_stamped(p) {
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
    generate_with(content, seed, id, entered_from, 0)
}

/// [`generate`] for a player whose talents make loot magic and rare `luck` percent more often
/// (Treasure hunter, spec §8 "Talents"): the same draws from the same streams, so the map is
/// the same; only the rarities its loot rolls may differ.
pub fn generate_with(
    content: &Content,
    seed: u64,
    id: &str,
    entered_from: Option<&str>,
    luck: i32,
) -> Result<LevelTemplate, GenError> {
    if content.local_area(id).is_some() {
        return local::generate(content, seed, id, luck);
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
        key: def.key,
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
        let item_level = content.item_level(id);
        let mut template = LevelTemplate {
            id: id.to_string(),
            name: def.level_name(depth),
            arrival: None,
            ambient: def.ambient,
            kind: LevelKind::Dungeon,
            terrain: canvas.terrain,
            player_start: Some(up),
            creatures: canvas.creatures,
            stairs,
            lights: canvas.lights,
            flow: canvas.flow,
            objects: canvas.objects,
            traps: canvas.traps,
            locks: canvas.locks,
            secrets: canvas.secrets,
            regions: Vec::new(),
            rooms: canvas.rooms,
            shops: canvas.shops,
            exit: None,
            item_level,
            loot: Vec::new(),
            temperature: plan.temperature,
            ice: Default::default(),
        };
        if connect::validate(&template, content).is_ok() {
            hide_traps(&mut template, content, pal, seed, def);
            hide_secrets(&mut template, content, seed, def, &canvas.stamped);
            let floor = Floor {
                terrain: pal.floor,
                count: def.loot,
            };
            place_loot(
                &mut template,
                content,
                seed,
                &canvas.loot,
                (Some(floor), luck),
            );
            return Ok(template);
        }
    }
    Err(GenError::Exhausted(MAX_ATTEMPTS))
}

/// Hides the branch's traps on a generated level (spec §8 "Traps"): `traps_per_level` of them,
/// each of a kind from the branch's list, all hidden. They are drawn from the level's own trap
/// stream ([`Rng::traps`]), so the rest of the level is the same with or without them. A trap
/// lies on plain floor that nothing else claims: never on or next to a staircase (the player
/// arrives there), never on a creature's patrol (creatures do not know where traps are, and
/// would spring one every round), nor where objects lie or a light stands, nor in a shop.
fn hide_traps(
    template: &mut LevelTemplate,
    content: &Content,
    pal: Palette,
    seed: u64,
    def: &crate::content::DungeonDef,
) {
    if def.traps.is_empty() {
        return;
    }
    let mut rng = Rng::traps(seed, &template.id);
    let [least, most] = def.traps_per_level;
    let count = least + rng.below(most - least + 1);
    let patrolled = patrolled(template);
    let near_stairs = |p: Pos| template.stairs.iter().any(|s| s.pos.chebyshev(p) <= 1);
    let mut free: Vec<Pos> = (template.terrain.iter())
        .filter(|&(p, t)| {
            *t == pal.floor
                && !near_stairs(p)
                && !patrolled.contains(&p)
                && !template.objects.contains_key(&p)
                && !template.lights.contains_key(&p)
                && !template.traps.contains_key(&p)
                && !template.shops.iter().any(|s| s.contains(p))
        })
        .map(|(p, _)| p)
        .collect();
    for _ in 0..count {
        if free.is_empty() {
            break;
        }
        let at = free.swap_remove(rng.index(free.len()));
        let kind = def.traps[rng.index(def.traps.len())];
        (template.traps).insert(at, Trap::new(kind, content.trap(kind), false, None));
    }
}

/// The cells creatures walk on their patrols, where they start included.
fn patrolled(template: &LevelTemplate) -> std::collections::BTreeSet<Pos> {
    let mut patrolled = std::collections::BTreeSet::new();
    for spawn in &template.creatures {
        let mut p = spawn.pos;
        patrolled.insert(p);
        for dir in &spawn.route {
            p = p.step(*dir);
            patrolled.insert(p);
        }
    }
    patrolled
}

/// Hides a few of a generated level's doors (spec §8 "Searching"): `secret_doors` of them, drawn
/// from the level's own secret stream ([`Rng::secrets`]), so the rest of the level is the same
/// with or without them. A door is hidden only if it is closed and nothing else claims it (no
/// lock, no trap, no patrol through it, no shop it leads into, no vault it was `stamped` from: vaults
/// bring their own secrets), and only where the level stays valid without
/// it: the stairs and a key to every lock are still reached from the arrival
/// (`connect::validate`), so a secret door only ever hides a way the player may do without. It
/// becomes the wall around it ([`secret_look`]), with the branch's difficulty for secret doors
/// ([`crate::content::SearchRules::secret_door`]).
fn hide_secrets(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    def: &crate::content::DungeonDef,
    stamped: &Grid<bool>,
) {
    let [least, most] = def.secret_doors;
    if most == 0 {
        return;
    }
    let mut rng = Rng::secrets(seed, &template.id);
    let count = least + rng.below(most - least + 1);
    let patrolled = patrolled(template);
    let shop_door =
        |p: Pos| (template.shops.iter()).any(|s| Dir::ALL.iter().any(|d| s.contains(p.step(*d))));
    let mut doors: Vec<Pos> = (template.terrain.iter())
        .filter(|&(p, t)| {
            content.terrain(*t).is_closed_door()
                && !template.locks.contains_key(&p)
                && !template.traps.contains_key(&p)
                && !patrolled.contains(&p)
                && !shop_door(p)
                && !stamped[p]
        })
        .map(|(p, _)| p)
        .collect();
    let difficulty = content.search().secret_door;
    let mut hidden = 0;
    while hidden < count && !doors.is_empty() {
        let at = doors.swap_remove(rng.index(doors.len()));
        let door = template.terrain[at];
        let Some(look) = secret_look(&template.terrain, content, at) else {
            continue;
        };
        template.terrain.set(at, look);
        template.secrets.insert(at, Secret { door, difficulty });
        if connect::validate(template, content).is_ok() {
            hidden += 1;
        } else {
            template.terrain.set(at, door);
            template.secrets.remove(&at);
        }
    }
}

/// The floor loot of a generated level: how many items, and the terrain they lie on.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Floor {
    pub terrain: TerrainId,
    /// Fewest and most.
    pub count: [u8; 2],
}

/// Rolls a generated level's loot (spec §8 "Magic items") at its item level, from its own
/// stream `loot/<level id>` so that the map is the same with or without it: first an item on
/// each of `markers` (vaults' loot cells, at least of their grade), then the floor loot, one draw
/// for how many (between the fewest and the most) and, for each, one for a cell of `floor`
/// terrain where nothing lies, stands, shines, or leads away (in row order), then the item,
/// magic and rare `luck` percent more often ([`loot::roll_with`]).
pub(crate) fn place_loot(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    markers: &[(Pos, LootGrade)],
    (floor, luck): (Option<Floor>, i32),
) {
    let mut rng = Rng::loot(seed, &template.id);
    let level = template.item_level;
    let place = |template: &mut LevelTemplate, rng: &mut Rng, p: Pos, grade: LootGrade| {
        if let Some(stack) = loot::roll_with(content, rng, level, grade, Source::Loot, luck) {
            add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
        }
    };
    for &(p, grade) in markers {
        place(template, &mut rng, p, grade);
    }
    let Some(floor) = floor else { return };
    let [fewest, most] = floor.count.map(i32::from);
    let count = rng.range(fewest, most);
    let mut free: Vec<Pos> = (template.terrain.iter())
        .map(|(p, _)| p)
        .filter(|&p| {
            template.terrain[p] == floor.terrain
                && !template.objects.contains_key(&p)
                && !template.lights.contains_key(&p)
                && !template.stairs.iter().any(|s| s.pos == p)
                && !template.creatures.iter().any(|c| c.pos == p)
                && template.player_start != Some(p)
                && !template.shops.iter().any(|s| s.contains(p))
        })
        .collect();
    for _ in 0..count {
        if free.is_empty() {
            break;
        }
        let p = free.remove(rng.index(free.len()));
        place(template, &mut rng, p, LootGrade::Any);
    }
}

/// The template as text: terrain glyphs, the top object of each pile over them, traps (hidden
/// ones too) over those, creatures on top, `*` for a light placed on floor, `&` for a locked
/// door, and `S` for a secret door. Trailing spaces are trimmed. Used by snapshot tests.
pub fn ascii(template: &LevelTemplate, content: &Content) -> String {
    draw(template, content, false)
}

/// [`ascii`] with the plain ground of each special room (spec §7 "Special rooms") drawn as its
/// effect's mark, under creatures, traps, objects, and lights. Used by `warlock-cli gen`.
pub fn ascii_with_rooms(template: &LevelTemplate, content: &Content) -> String {
    draw(template, content, true)
}

fn draw(template: &LevelTemplate, content: &Content, rooms: bool) -> String {
    let floor = content.terrain_id("floor");
    let mut out = String::new();
    for y in 0..template.terrain.height() {
        let row: String = (0..template.terrain.width())
            .map(|x| {
                let p = Pos::new(x, y);
                let t = template.terrain[p];
                let room = (template.rooms.iter())
                    .find(|r| rooms && r.contains(p) && content.terrain(t).is_plain_ground());
                if let Some(c) = template.creatures.iter().find(|c| c.pos == p) {
                    content.creature(c.creature).glyph
                } else if let Some(trap) = template.traps.get(&p) {
                    content.trap(trap.kind).glyph
                } else if let Some(top) = template.objects.get(&p).and_then(|o| o.first()) {
                    content.object(top.object).glyph
                } else if template.lights.contains_key(&p) && Some(t) == floor {
                    '*'
                } else if template.locks.contains_key(&p) {
                    '&'
                } else if template.secrets.contains_key(&p) {
                    'S'
                } else if let Some(room) = room {
                    content.room_effect(room.effect).mark
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
