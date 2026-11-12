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
//!    then roll its loot from its loot stream ([`Rng::loot`]), then, on a branch with a danger,
//!    lay its initial population down from its population stream ([`Rng::population`],
//!    `population.rs`), then its teeming dens' sleepers and hoards from its swarm stream
//!    ([`Rng::swarm`], `den.rs`), then what its vaults' empty chests hold from its vault chest
//!    stream ([`Rng::vault_chests`]), then its chests from its chest stream ([`Rng::chests`],
//!    `chest.rs`), then its creatures' gear from its gear stream ([`Rng::gear`],
//!    [`crate::gear::arm`]).
//!
//! Generation is integer-only and never iterates a hash map. Seeds are stable within a game
//! version; the snapshot tests in this module flag any change in generator output.

mod caves;
mod chest;
mod connect;
mod deep;
mod den;
mod dress;
mod hoard;
mod leavings;
pub mod local;
mod population;
mod room_loot;
mod rooms;
mod shop;
mod special;
mod treasury;
mod vault;
mod vein;
mod way_up;

use crate::chest::Chest;
use crate::content::{
    Content, CreatureId, CreatureSpawn, DungeonDef, Flow, Generator, LevelKind, LevelTemplate,
    LightId, ObjectId, Place, RoomEffectId, RoomOdds, Secret, ShopOdds, ShopPlan, Spawning,
    SpecialRoom, StairsDir, StairsLink, TerrainId, Trap, Vault, VaultMonsters, secret_look,
    sibling,
};
use crate::door::Lock;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::magic::{LootGrade, LootPool, loot, loot::Luck};
use crate::object::{Stack, add_to_pile};
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
    /// Deep water: a cave pool's middle, where only a swimmer goes (`deep.rs`).
    pub deep_water: TerrainId,
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
            deep_water: terrain("deep_water")?,
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
    /// Vaults (ids) the level stamps for certain (its branch level's `vaults`): a generator
    /// that cannot fit one fails the attempt.
    pub vaults: &'a [String],
}

/// A level under construction.
#[derive(Clone, Debug)]
pub(crate) struct Canvas<'a> {
    pub content: &'a Content,
    pub pal: Palette,
    /// How often the level's rooms are special, and with what (the branch's odds at its depth).
    pub room_odds: &'a RoomOdds,
    /// How often the level has a shop, and what kind (the branch's); `None`: no shops at all.
    pub shop_odds: Option<&'a ShopOdds>,
    /// The vaults that may be stamped here ([`Vault::stamps_on`]), in the content's order.
    pub vaults: Vec<&'a Vault>,
    pub terrain: Grid<TerrainId>,
    pub lights: BTreeMap<Pos, LightId>,
    /// Flow of running water stamped from vaults (generators make none of their own).
    pub flow: BTreeMap<Pos, Flow>,
    /// Objects stamped from vaults, and keys to locked rooms.
    pub objects: BTreeMap<Pos, Vec<Stack>>,
    /// Traps stamped from vaults.
    pub traps: BTreeMap<Pos, Trap>,
    /// Where vaults stamped loot to roll, and what it may be.
    pub loot: Vec<(Pos, LootPool)>,
    /// Where vaults stamped monster pools to draw, and who may come.
    pub monsters: Vec<VaultMonsters>,
    /// Locks on closed doors: stamped from vaults, and on locked rooms (spec §8 "Doors").
    pub locks: BTreeMap<Pos, Lock>,
    /// Secret doors stamped from vaults (spec §8 "Searching").
    pub secrets: BTreeMap<Pos, Secret>,
    /// Chests stamped from vaults (spec §8 "Chests"), their level the floor's once it is known.
    pub chests: BTreeMap<Pos, Chest>,
    /// Named places stamped from vaults (a legend's `name` and `note`: an inscribed statue).
    pub places: BTreeMap<Pos, Place>,
    pub creatures: Vec<CreatureSpawn>,
    /// Special rooms (spec §7 "Special rooms"; `special.rs`).
    pub rooms: Vec<SpecialRoom>,
    /// The effect covering the whole level, if one drawn for a room does (`special.rs`).
    pub pervading: Option<RoomEffectId>,
    /// Whether the level has a danger (some effects need one) and another level below it.
    pub danger: bool,
    pub below: bool,
    /// The level's least danger (for a player of level 1; 0 without one): a den with its own
    /// population needs a type of it that weighs something there (`den::peopled`).
    pub least: u8,
    /// The branch's own population: a den kind that names factions needs one of them in it
    /// (`den::peopled`).
    pub population: &'a [CreatureId],
    /// Shops (spec §7 "Shops"; `shop.rs`), their keepers among `creatures`.
    pub shops: Vec<ShopPlan>,
    /// Cells stamped from vaults; later passes leave them alone.
    pub stamped: Grid<bool>,
    /// The entrances of the vaults the level names (`Needs::vaults`), as stamped: what a way up
    /// from below is placed near (`way_up.rs`).
    pub named_doors: Vec<Pos>,
    pub up: Option<Pos>,
    pub down: Option<Pos>,
    /// A layer boss's lair's hidden way down, stamped as floor: its cell and the stairs' terrain
    /// ([`Vault::down`], `world/layer_boss.rs`).
    pub lair: Option<(Pos, TerrainId)>,
}

impl<'a> Canvas<'a> {
    fn new(
        content: &'a Content,
        pal: Palette,
        (room_odds, shop_odds): (&'a RoomOdds, Option<&'a ShopOdds>),
        vaults: Vec<&'a Vault>,
        (width, height): (i32, i32),
    ) -> Self {
        Self {
            content,
            pal,
            room_odds,
            shop_odds,
            vaults,
            terrain: Grid::new(width, height, pal.void),
            lights: BTreeMap::new(),
            flow: BTreeMap::new(),
            objects: BTreeMap::new(),
            traps: BTreeMap::new(),
            loot: Vec::new(),
            monsters: Vec::new(),
            locks: BTreeMap::new(),
            secrets: BTreeMap::new(),
            chests: BTreeMap::new(),
            places: BTreeMap::new(),
            creatures: Vec::new(),
            rooms: Vec::new(),
            pervading: None,
            danger: false,
            below: false,
            least: 0,
            population: &[],
            shops: Vec::new(),
            stamped: Grid::new(width, height, false),
            named_doors: Vec::new(),
            up: None,
            down: None,
            lair: None,
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
            rolls_gear: true,
            asleep: false,
            status: Vec::new(),
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

/// Generates the level `id` for a run of campaign `campaign` with `seed`: a branch level (e.g.
/// `depths:3`; on a branch's first level the up staircase leads to `entered_from`, or nowhere if
/// `None`) or a local area of the campaign's overworld (`local:12,30`).
pub fn generate(
    content: &Content,
    (campaign, seed): (&str, u64),
    id: &str,
    entered_from: Option<&str>,
) -> Result<LevelTemplate, GenError> {
    generate_with(content, (campaign, seed), id, entered_from, Luck::default())
}

/// [`generate`] for a player whose talents give their finds `luck` ([`Luck`]: Treasure hunter,
/// Hoard sense): the same draws from the same streams, so the map is the same; only what its
/// loot rolls may differ.
pub fn generate_with(
    content: &Content,
    (campaign, seed): (&str, u64),
    id: &str,
    entered_from: Option<&str>,
    luck: Luck,
) -> Result<LevelTemplate, GenError> {
    let visit = Visit {
        luck,
        ..Visit::default()
    };
    generate_for(content, (campaign, seed), id, entered_from, &visit)
}

/// A branch's floors repeat a random vault only every this many floors at the soonest.
const VAULT_PHASES: usize = 4;

/// The vaults that may be stamped on `def`'s level at `depth` ([`Vault::stamps_on`]): pinned ones
/// (named by a level, so stamped by [`Needs::vaults`] anyway) as they are, and of the random ones
/// only the phase of this depth that suit the floor's `danger` ([`Vault::suits`]). The run's seed
/// shuffles the branch's random vaults and deals them round the phases, so a vault can come at
/// most every [`VAULT_PHASES`] floors of the branch, and which ones differs from run to run and
/// branch to branch; a run keeps no count of what it met. The deal is made before the danger is
/// asked, so a vault's `danger` changes no other vault's phase.
fn spaced_vaults<'a>(
    content: &'a Content,
    seed: u64,
    (def, depth): (&DungeonDef, u32),
    danger: u8,
) -> Vec<&'a Vault> {
    let mut all: Vec<&Vault> = (content.vaults().iter())
        .filter(|v| v.stamps_on(def, depth))
        .collect();
    let mut random: Vec<&Vault> = all.iter().copied().filter(|v| !v.pinned).collect();
    // Sorted by id first: the content's order must not decide the deal.
    random.sort_by(|a, b| a.id().cmp(b.id()));
    Rng::stream(seed, &format!("vault_phases/{}", def.id)).shuffle(&mut random);
    let phase = depth as usize % VAULT_PHASES;
    let mine: Vec<&Vault> = (random.iter().copied().enumerate())
        .filter(|(i, _)| i % VAULT_PHASES == phase)
        .map(|(_, v)| v)
        .collect();
    all.retain(|v| v.pinned || (v.suits(danger) && mine.iter().any(|m| m.id() == v.id())));
    all
}

/// Whom a level is generated for, as they first come: what their talents give their finds
/// (`luck`), their level (a player-scaled danger reads it), the unique creature types the
/// run has met (none comes twice), and those slain (a slain layer boss's floor has no lair; a
/// Pit's floor is given its own warden here once the Pit is sealed: its hall is made without it). Only
/// the loot, the initial population, and a layer boss's lair read it, so the map is otherwise
/// the same for anyone.
#[derive(Clone, Copy, Debug, Default)]
pub struct Visit<'a> {
    pub luck: Luck,
    pub player_level: u8,
    pub uniques: Option<&'a std::collections::BTreeSet<String>>,
    pub slain: Option<&'a std::collections::BTreeSet<String>>,
}

/// [`generate_with`] for the player `visit` describes.
pub fn generate_for(
    content: &Content,
    (campaign, seed): (&str, u64),
    id: &str,
    entered_from: Option<&str>,
    visit: &Visit,
) -> Result<LevelTemplate, GenError> {
    let luck = visit.luck;
    if content.local_area(campaign, id).is_some() {
        return local::generate(content, (campaign, seed), id, luck);
    }
    let (def, depth) = content
        .branch_level(id)
        .ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    let plan = def
        .plan(depth)
        .ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    let plan = def
        .endless
        .map_or(plan, |e| endless_plan(plan, e, seed, id));
    let pal = Palette::resolve(content).map_err(GenError::Content)?;
    let room_odds =
        (def.room_odds(depth)).ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    let odds = (room_odds, def.shop_odds(depth));
    // A layer boss not yet slain keeps its lair here, with the floor's way down hidden in it
    // (spec §7 "The endless branch", `world/layer_boss.rs`).
    // A Pit's warden's hall is stamped whether the warden lives or not; slain (its Pit sealed),
    // the hall is made without it (spec §7 "The Pits", `world/pit.rs`).
    let slain = |b: &&crate::content::LayerBoss| {
        let id = &content.creature(b.creature).id;
        visit.slain.is_some_and(|s| s.contains(id))
    };
    let boss = (def.boss_at(depth)).filter(|b| def.portal.is_some() || !slain(b));
    let warden_gone = boss.filter(|b| slain(b)).map(|b| b.creature);
    // Its lair hides the floor's way down, where the floor leads anywhere.
    let lair = boss.is_some() && def.leads_down(depth);
    let needs = Needs {
        down: def.leads_down(depth) && boss.is_none(),
        creatures: &def.creatures,
        key: def.key,
        vaults: boss.map_or(def.vaults(depth), |b| std::slice::from_ref(&b.vault)),
    };
    // A Pit's floors lead only to its own (spec §7 "The Pits"): the neighbours are the floor's
    // siblings, and its first floor's `<` is where it was entered from.
    let up_to = match (depth > 1, def.endless.is_some() || def.below.is_some()) {
        (true, _) => Some(sibling(id, depth - 1)),
        // However the first floor was reached (from the second, it may be), it climbs out: an
        // endless branch's, and a way up's (spec §7 "Ways up").
        (false, true) => content.way_into(id).map(str::to_string),
        (false, false) => entered_from.map(str::to_string),
    };
    // A lair's hidden way down leads where the `>` would (the stairs need not be placed).
    let down_to = def
        .leads_down(depth)
        .then(|| def.down_to(id, depth))
        .flatten();
    let ways_up = content.ways_up_from(id);

    // The danger the floor's pools draw at, which a vault's `danger` asks (`Vault::suits`).
    let danger = (content.spawning(id)).map_or(0, |(rule, at, _)| rule.at(at, visit.player_level));
    for attempt in 0..MAX_ATTEMPTS {
        let mut rng = Rng::map(seed, id, attempt);
        let vaults = spaced_vaults(content, seed, (def, depth), danger);
        let size = (plan.width, plan.height);
        let mut canvas = Canvas::new(content, pal, odds, vaults, size);
        canvas.danger = content.spawning(id).is_some();
        canvas.least = content
            .spawning(id)
            .map_or(0, |(rule, depth, _)| rule.at(depth, 1));
        canvas.population = content.spawning(id).map_or(&[], |(_, _, s)| &s.population);
        canvas.below = needs.down;
        let built = match plan.generator {
            Generator::Rooms => rooms::build(&mut canvas, &mut rng, needs),
            Generator::Caves => caves::build(&mut canvas, &mut rng, needs),
        };
        if built.is_none() {
            continue;
        }
        let Some(extra) = way_up::place(&mut canvas, &ways_up) else {
            continue;
        };
        if plan.generator == Generator::Caves {
            deep::lay(&mut canvas, seed, id);
        }
        canvas.build_walls();
        let (Some(up), down) = (canvas.up, canvas.down) else {
            continue;
        };
        if down.is_some() != needs.down || canvas.lair.is_some() != lair {
            continue;
        }
        if let Some(gone) = warden_gone {
            canvas.creatures.retain(|c| c.creature != gone);
        }
        let mut stairs = vec![StairsLink {
            pos: up,
            dir: StairsDir::Up,
            to: up_to.clone(),
            name: None,
        }];
        if let Some(pos) = down {
            stairs.push(StairsLink {
                pos,
                dir: StairsDir::Down,
                to: down_to.clone(),
                name: None,
            });
        }
        stairs.extend(extra);
        let item_level = content.item_level(id);
        // A vault's chests are of the floor's level (spec §8 "Chests").
        let chests = (canvas.chests.into_iter())
            .map(|(p, chest)| {
                (
                    p,
                    Chest {
                        level: item_level,
                        ..chest
                    },
                )
            })
            .collect();
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
            chests,
            statues: BTreeMap::new(),
            secrets: canvas.secrets,
            hidden: (boss.zip(canvas.lair))
                .map(|(b, lair)| layer_boss_stairs(content, b, lair, down_to.clone()))
                .into_iter()
                .collect(),
            places: canvas.places,
            regions: Vec::new(),
            rooms: canvas.rooms,
            pervading: canvas.pervading,
            shops: canvas.shops,
            exit: None,
            item_level,
            loot: Vec::new(),
            // Its vaults' pools, until they are drawn: a key one carries counts (`connect.rs`).
            monsters: canvas.monsters,
            outside_cap: Default::default(),
            temperature: plan.temperature,
            ice: Default::default(),
            // Its branch's, asked of [`Content::spawning`].
            spawning: None,
            graves: None,
        };
        if connect::validate(&template, content).is_ok() {
            treasury::seal(&mut template, content, pal);
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
            let monsters = std::mem::take(&mut template.monsters);
            if let Some((rule, depth, spawning)) = content.spawning(id) {
                let danger = rule.at(depth, visit.player_level);
                // The odds of this floor's own draws: its depth's `dread_one_in`.
                let odds = (content.draw_odds_at(id)).unwrap_or(spawning.odds);
                let spawning = &Spawning {
                    odds,
                    ..spawning.clone()
                };
                population::populate(
                    &mut template,
                    content,
                    seed,
                    (pal, danger, spawning),
                    (visit, &monsters, &canvas.stamped),
                );
                den::fill(&mut template, content, seed, (pal, danger, spawning), visit);
                hoard::fill(&mut template, content, seed, (pal, danger), visit);
            }
            population::leave_unborne(&mut template, content, &monsters);
            room_loot::fill(&mut template, content, seed, pal);
            chest::fill_vaults(&mut template, content, (seed, luck));
            chest::fill(
                &mut template,
                content,
                (seed, luck),
                (pal, def),
                &canvas.stamped,
            );
            treasury::fill(&mut template, content, (seed, luck), (pal, def));
            crate::gear::arm(&mut template, content, seed);
            crate::animated::arm(&mut template, content, seed);
            // Last, from its own stream: only walls change (`vein.rs`).
            let danger = content.floor_danger(id, visit.player_level);
            vein::lay(
                &mut template,
                content,
                (seed, pal),
                (def, danger),
                &canvas.stamped,
            );
            return Ok(template);
        }
    }
    Err(GenError::Exhausted(MAX_ATTEMPTS))
}

/// A lair's hidden way down: the level keeps it as a hidden location named for the boss, which
/// its death reveals (`world/layer_boss.rs`).
fn layer_boss_stairs(
    content: &Content,
    boss: &crate::content::LayerBoss,
    (at, terrain): (Pos, TerrainId),
    to: Option<String>,
) -> (Pos, crate::content::HiddenLocation) {
    let creature = content.creature(boss.creature);
    let hidden = crate::content::HiddenLocation {
        id: creature.id.clone(),
        terrain,
        to,
        name: creature.name.clone(),
    };
    (at, hidden)
}

/// The generator and size of a floor of an endless branch, drawn as it is made from its own
/// stream `floor_kind/<level id>` (spec §7 "The endless branch"), so the draw moves nothing else.
fn endless_plan(
    plan: crate::content::LevelPlan,
    endless: crate::content::Endless,
    seed: u64,
    id: &str,
) -> crate::content::LevelPlan {
    if !Rng::stream(seed, &format!("floor_kind/{id}")).chance(endless.caves.into()) {
        return plan;
    }
    let (width, height) = endless.caves_size;
    crate::content::LevelPlan {
        generator: Generator::Caves,
        width,
        height,
        ..plan
    }
}

/// Hides the branch's traps on a generated level (spec §8 "Traps"): `traps_per_level` of them,
/// each of a kind from the branch's list, all hidden. They are drawn from the level's own trap
/// stream ([`Rng::traps`]), so the rest of the level is the same with or without them. A trap
/// lies on plain floor that nothing else claims: never on or next to a staircase (the player
/// arrives there), never on a creature's patrol (creatures do not know where traps are, and
/// would spring one every round), nor where objects lie or a light stands, nor in a shop, nor in
/// a sealed room (a greater treasure room).
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
    // A lair's hidden way down counts: it will be stairs.
    let near_stairs = |p: Pos| {
        (template.stairs.iter().map(|s| s.pos))
            .chain(template.hidden.keys().copied())
            .any(|s| s.chebyshev(p) <= 1)
    };
    // A sealed room (a greater treasure room) is walled up: nothing springs in it.
    let sealed = |p: Pos| {
        (template.rooms.iter())
            .any(|r| content.room_effect(r.effect).sealed.is_some() && r.contains(p))
    };
    let mut free: Vec<Pos> = (template.terrain.iter())
        .filter(|&(p, t)| {
            *t == pal.floor
                && !sealed(p)
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
/// each of `markers` (vaults' loot cells, from their pools), then the floor loot, one draw
/// for how many (between the fewest and the most) and, for each, one for a cell of `floor`
/// terrain where nothing lies, stands, shines, or leads away (in row order), then the item,
/// with the finder's `luck` ([`loot::roll_pool`]).
pub(crate) fn place_loot(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    markers: &[(Pos, LootPool)],
    (floor, luck): (Option<Floor>, Luck),
) {
    let mut rng = Rng::loot(seed, &template.id);
    let level = template.item_level;
    let place = |template: &mut LevelTemplate, rng: &mut Rng, p: Pos, pool: &LootPool| {
        if let Some(stack) = loot::roll_pool(content, rng, level, pool, luck) {
            add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
        }
    };
    for (p, pool) in markers {
        place(template, &mut rng, *p, pool);
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
        place(template, &mut rng, p, &LootGrade::Any.into());
    }
}

/// The template as text: terrain glyphs, the top object of each pile over them, traps (hidden
/// ones too) over those, creatures on top, `*` for a light placed on floor, `&` for a locked
/// door, `S` for a secret door, and `V` for a vein of ore (`vein.rs`). Trailing spaces are
/// trimmed. Used by snapshot tests.
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
    let veins = content.mining();
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
                } else if template.chests.contains_key(&p) {
                    // A chest by its own glyph, whatever its lock or trap.
                    content.terrain(t).glyph
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
                } else if veins.is_some_and(|m| m.vein_of(t).is_some()) {
                    'V'
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
mod altar_tests;
#[cfg(test)]
mod cealdhlaw_tests;
#[cfg(test)]
mod chest_tests;
#[cfg(test)]
mod deep_tests;
#[cfg(test)]
mod fennmynster_tests;
#[cfg(test)]
mod fyrdelf_tests;
#[cfg(test)]
mod hellegeat_tests;
#[cfg(test)]
mod homes_tests;
#[cfg(test)]
mod lairs_tests;
#[cfg(test)]
mod leavings_tests;
#[cfg(test)]
mod lytelscraef_tests;
#[cfg(test)]
mod neutral_share_tests;
#[cfg(test)]
mod scriptorium_tests;
#[cfg(test)]
mod seeds_tests;
#[cfg(test)]
mod shrine_tests;
#[cfg(test)]
mod special_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod treasury_tests;
#[cfg(test)]
mod vault_danger_tests;
#[cfg(test)]
mod vault_monsters_tests;
#[cfg(test)]
mod vault_stamp_tests;
#[cfg(test)]
mod vault_turn_tests;
#[cfg(test)]
mod vein_tests;
#[cfg(test)]
mod way_up_tests;
