//! Dungeon branches (`assets/data/dungeons.toml`): stacks of procedurally generated levels.
//!
//! A branch `depths` with `depth = 8` provides the levels `depths:1` … `depths:8`. Stairs in any
//! level may lead to `depths:N` (`to = "depths:1"`); such a level is generated on its first visit
//! from the run seed (see [`crate::mapgen`]) and then persists like a hand-made one. Its `<` leads
//! to `depths:N-1` (on depth 1: back to the level it was first entered from) and its `>` to
//! `depths:N+1` (the deepest level has none).
//!
//! A branch with `below` (a level of another branch, `below = "fyrdelf:12"`) is a way between
//! that level and the overworld (spec §7 "Ways up"): its deepest level's `>` leads there, that
//! level gets a `<` back up (`mapgen/way_up.rs`), and its first level's `<` climbs out to its way
//! in, however the branch was entered.
//!
//! A branch with `endless` instead of `depth` has no last floor (spec §7 "The endless branch",
//! `world/endless.rs`): every staircase makes a new floor, only the current one exists, and each
//! floor is drawn rooms or caves as it is made. It has one row of per-depth tables, read at every
//! depth, and so no `[[dungeon.levels]]`, and with them no pinned vault.
//!
//! ```toml
//! [[dungeon]]
//! id = "depths"
//! name = "The Depths"          # level names: "The Depths 3"; default: the id
//! generator = "rooms"          # "rooms" (rooms and corridors) or "caves" (cellular automata)
//! depth = 8                     # or, instead: [dungeon.endless] (below)
//! size = [80, 21]
//! creatures = ["rat", "kobold"] # patrolling creatures generators may place; default none
//! traps = ["arrow", "pit"]      # traps generators may hide (spec §8 "Traps"); default none
//! traps_per_level = [2, 4]      # how many a level gets, at least and at most; default [2, 4]
//! secret_doors = [0, 2]         # doors hidden on each level (spec §8 "Searching"); default none
//! hidden_per_depth = 1          # hidden features this much harder to find per level down
//! locks_per_depth = 1           # locks this much harder to pick per level down
//! loot = [2, 4]                 # items of loot on each level's floor; default none
//! chests = { chance = 20, count = [1, 1] } # chests a floor gets (spec §8 "Chests"); the default
//! ambient = [0, 0, 0, 0]        # optional
//! temperature = 9               # °C; default the dungeon default (10)
//! key = "rusty_key"             # optional: rooms levels lock a few rooms with its lock
//! danger = 4                    # optional (spec §7 "Danger levels"; `super::danger`): its
//! population = ["rat"]          # floors' danger, who lives there, how many, and how they
//! population_count = 40         # come back; none: danger 0, nothing comes
//! stray = 5                    # the draws' odds (`super::danger::Odds`); default
//! stray_pool = "populations"    # strays only from populations; default "all"
//! out_of_depth = 5             # data/dungeons.toml's `[defaults]`
//! deeper = 5
//! deeper_cap = 10
//! dread_one_in = 100            # one draw in this many among the dread types
//! spawn = { every = "15m", chance = 60 }
//!
//! [[dungeon.levels]]            # optional per-depth overrides, first match wins
//! from = 3
//! to = 5                        # inclusive; default: just `from`
//! generator = "caves"           # default: the branch's generator
//! size = [120, 60]              # default: the branch's size
//! temperature = 7               # default: the branch's temperature
//! danger = { base = 9, per_depth = 2 }   # default: the branch's (which it needs)
//! room_effects = { chance = 40 } # default: the branch's (below), field by field
//! dread_one_in = 50             # default: the branch's (which needs a `danger`)
//!
//! [dungeon.endless]             # instead of `depth`: floors without end (`generator` is "rooms")
//! caves = 15                    # percent of floors that are caves, the rest rooms
//! caves_size = [100, 40]        # the caves' size; the rooms' is the branch's `size`
//! dread = { steps = [[70, 40]], halves = 10, least = 5 }   # `dread_one_in` by depth
//!                               # (`super::dread`), as `[[dungeon.levels]]` give it elsewhere
//!
//! [[dungeon.endless.boss]]      # optional, one per layer: a layer boss and its lair
//! depth = 10                    # the floor (each depth once)
//! creature = "gnaegan"          # a unique type, placed by the lair alone
//! vault = "gnaegans_midden"     # the lair: a vault placing the boss, with a `>` (hidden)
//!
//! [dungeon.room_effects]        # optional: the branch's own special-room odds
//! chance = 25                   # percent a room is special; default the shared `chance`
//! per_level = 2                 # at most this many a level (1 to 8); default the shared
//! favoured = { flooded = 40 }   # effects with a fixed share of the draws, in percent
//! banned = ["sanctuary"]        # effects never drawn here; the rest by their own `weight`
//!
//! [dungeon.shops]               # optional: the branch's own shop odds
//! chance = 100                  # percent of rooms levels with a shop; default the shared
//! kinds = ["general_store"]     # ids in shops.toml, picked uniformly; default all
//! names = ["Durin"]             # the keepers' names; default the shared `names`
//! keeper = "shopkeeper"         # the keepers' creature type; default the shared `[keeper]`'s
//! ```
//!
//! A branch without the tables gets the shared odds of `room_effects.toml` and `shops.toml`
//! ([`RoomOdds::shared`]: no favoured or banned effects; [`ShopOdds::shared`]), resolved here
//! once per depth ([`DungeonDef::room_odds`], [`DungeonDef::shop_odds`]); the generators draw
//! the same way whichever odds they read. The old `effects` list is refused.

use super::ContentError;
use super::creature::CreatureId;
use super::danger::{
    CountSpec, DangerRule, DangerSpec, Odds, OddsSpec, SpawnSpec, Spawning, SpawningSpec, StrayPool,
};
use super::dread::{DreadCurve, DreadCurveSpec};
use super::object::ObjectId;
use super::room_effect::{RoomEffectId, RoomEffects};
use super::shop::{ShopId, Shops};
use super::trap::TrapId;
use crate::light::Light;
use crate::temperature::Temperature;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Which generator builds a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Generator {
    /// Rooms joined by corridors, with doors at room entrances (ADOM-like).
    Rooms,
    /// Cellular-automata caverns.
    Caves,
}

impl Generator {
    /// Smallest and largest supported `[width, height]`.
    fn size_limits(self) -> ([i32; 2], [i32; 2]) {
        match self {
            Generator::Rooms => ([40, 15], [200, 100]),
            Generator::Caves => ([40, 30], [400, 300]),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DungeonFile {
    #[serde(default)]
    dungeon: Vec<DungeonSpec>,
    /// The default odds of a draw (spec §7 "Danger levels"): only `data/dungeons.toml` gives
    /// them, read before the levels (`danger::DefaultsFile`).
    pub(super) defaults: Option<Odds>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DungeonSpec {
    id: String,
    name: Option<String>,
    /// A line said each time the player arrives on its first level; default none.
    arrival: Option<String>,
    generator: Generator,
    /// How many levels it has; left out of a branch with `endless`.
    depth: Option<u32>,
    /// The level (another branch's) its deepest level's stairs down lead to; default none.
    below: Option<String>,
    /// Floors without end, instead of `depth`.
    endless: Option<EndlessSpec>,
    size: [i32; 2],
    #[serde(default)]
    ambient: Light,
    #[serde(default)]
    creatures: Vec<String>,
    /// How many items of loot lie on the floor of each level, `[fewest, most]`.
    #[serde(default)]
    loot: [u8; 2],
    /// How often its floors get chests; default one floor in five, one chest.
    #[serde(default)]
    chests: ChestOdds,
    /// The temperature on its levels, in °C; default the dungeon default.
    temperature: Option<i8>,
    /// The key (an object) to the rooms the generators lock; none: no locked rooms.
    key: Option<String>,
    #[serde(default)]
    traps: Vec<String>,
    traps_per_level: Option<[u32; 2]>,
    #[serde(default)]
    secret_doors: [u32; 2],
    #[serde(default)]
    hidden_per_depth: u8,
    #[serde(default)]
    locks_per_depth: u8,
    /// How dangerous its floors are (spec §7 "Danger levels"); none: 0, nothing comes.
    danger: Option<DangerSpec>,
    /// The creature types that live there; default: `creatures`.
    population: Option<Vec<String>>,
    population_count: Option<CountSpec>,
    stray: Option<u8>,
    /// Where its strays come from; default: every type that may stray.
    stray_pool: Option<StrayPool>,
    out_of_depth: Option<u8>,
    deeper: Option<u8>,
    deeper_cap: Option<u8>,
    dread_one_in: Option<u16>,
    /// Neutral animals' weight in percent; default `[defaults]`'s.
    neutral: Option<u8>,
    spawn: Option<SpawnSpec>,
    room_effects: Option<RoomOddsSpec>,
    shops: Option<ShopOddsSpec>,
    #[serde(default)]
    levels: Vec<LevelsSpec>,
}

/// A branch's `[dungeon.endless]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndlessSpec {
    caves: u8,
    caves_size: [i32; 2],
    #[serde(default)]
    boss: Vec<BossSpec>,
    /// `dread_one_in` by depth (`super::dread`).
    dread: Option<DreadCurveSpec>,
}

/// One `[[dungeon.endless.boss]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BossSpec {
    depth: u32,
    creature: String,
    vault: String,
}

/// A layer boss of an endless branch (spec §7 "The endless branch", `world/layer_boss.rs`): the
/// unique creature whose lair, a vault, floor `depth` stamps until it is slain, with the floor's
/// only way down hidden in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerBoss {
    pub depth: u32,
    pub creature: CreatureId,
    /// The lair's vault id (checked by `Content::load_dungeons`: it places the boss and marks
    /// the stairs, [`super::Vault::down`]).
    pub vault: String,
}

/// What makes a branch endless (spec §7 "The endless branch"): the odds of a floor's generator,
/// and how deep its danger still finds anything to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endless {
    /// Percent of floors that are caves; the rest are rooms, of the branch's size.
    pub caves: u8,
    /// The width and height of a caves floor.
    pub caves_size: (i32, i32),
    /// The deepest floor whose danger the branch's roster still reaches: floors below it draw as
    /// it does ([`crate::content::Content::spawning`]). Filled once everything is loaded
    /// (`Content::fill_generic`).
    pub roster_depth: u32,
}

/// A branch's (or a range of its depths') `room_effects` table: each field left out falls back
/// to the branch's table, then to the shared `room_effects.toml`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomOddsSpec {
    chance: Option<u8>,
    per_level: Option<u8>,
    favoured: Option<BTreeMap<String, u8>>,
    banned: Option<Vec<String>>,
}

/// A branch's (or a range of its depths') `shops` table, falling back as [`RoomOddsSpec`] does.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopOddsSpec {
    chance: Option<u8>,
    kinds: Option<Vec<String>>,
    names: Option<Vec<String>>,
    keeper: Option<String>,
}

/// How often a generated level's rooms are special, and which effects they get (spec §7
/// "Special rooms"): the shared odds of `room_effects.toml` or a branch's own. The effects not
/// banned are drawn by their own `weight`, but the favoured ones, which keep a fixed share of the
/// draws (`mapgen/special.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomOdds {
    /// Percent per room (each of a cave's chambers) to be special.
    pub chance: u8,
    /// At most this many special rooms a level.
    pub per_level: u8,
    /// Effects with a fixed share of the draws, in percent (together at most 100), in effect
    /// order.
    pub favoured: Vec<(RoomEffectId, u8)>,
    /// Effects never drawn, in effect order.
    pub banned: Vec<RoomEffectId>,
}

impl RoomOdds {
    /// The shared odds: `room_effects.toml`'s `chance` and `per_level`, nothing favoured or
    /// banned.
    pub fn shared(rooms: &RoomEffects) -> Self {
        Self {
            chance: rooms.chance,
            per_level: rooms.per_level,
            favoured: Vec::new(),
            banned: Vec::new(),
        }
    }

    /// Whether levels with these odds get special rooms at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && self.per_level > 0
    }

    /// The share of the draws `effect` keeps here, if it is favoured.
    pub fn share(&self, effect: RoomEffectId) -> Option<u8> {
        (self.favoured.iter()).find_map(|&(e, share)| (e == effect).then_some(share))
    }

    /// Whether `effect` is never drawn here.
    pub fn bans(&self, effect: RoomEffectId) -> bool {
        self.banned.contains(&effect)
    }
}

/// How often a generated rooms level has a shop, of which kinds, and who keeps it (spec §7
/// "Shops"): the shared odds of `shops.toml` or a branch's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopOdds {
    /// Percent of rooms levels (with a fitting room) with a shop.
    pub chance: u8,
    /// The kinds a shop may be, picked uniformly.
    pub kinds: Vec<ShopId>,
    /// The keepers' names, picked uniformly.
    pub names: Vec<String>,
    /// The keepers' creature type (each carries, wields, and wears the shared `[keeper]`'s kit).
    pub keeper: CreatureId,
}

impl ShopOdds {
    /// The shared odds: `shops.toml`'s `chance`, every kind in file order, its `names`, and its
    /// `[keeper]`'s creature.
    pub fn shared(shops: &Shops) -> Self {
        Self {
            chance: shops.chance,
            kinds: (0..shops.defs().len())
                .filter_map(|i| u8::try_from(i).ok().map(ShopId))
                .collect(),
            names: shops.names.clone(),
            keeper: shops.keeper.creature,
        }
    }

    /// Whether levels with these odds get shops at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && !self.kinds.is_empty() && !self.names.is_empty()
    }
}

impl RoomOddsSpec {
    /// A depth's table: `over`'s fields (its `[[dungeon.levels]]` entry), else `branch`'s.
    fn layer(over: Option<&Self>, branch: Option<&Self>) -> Self {
        let over = over.cloned().unwrap_or_default();
        let branch = branch.cloned().unwrap_or_default();
        Self {
            chance: over.chance.or(branch.chance),
            per_level: over.per_level.or(branch.per_level),
            favoured: over.favoured.or(branch.favoured),
            banned: over.banned.or(branch.banned),
        }
    }

    /// The odds, the shared ones filling in what the table leaves out.
    fn resolve(self, rooms: &RoomEffects) -> Result<RoomOdds, String> {
        let shared = RoomOdds::shared(rooms);
        let id = |e: &str| {
            (rooms.id(e)).ok_or_else(|| format!("unknown effect `{e}` (room_effects.toml)"))
        };
        let mut favoured = Vec::new();
        for (name, &share) in self.favoured.iter().flatten() {
            favoured.push((id(name)?, share));
        }
        favoured.sort();
        let mut banned = (self.banned.iter().flatten())
            .map(|e| id(e))
            .collect::<Result<Vec<_>, _>>()?;
        banned.sort();
        banned.dedup();
        Ok(RoomOdds {
            chance: self.chance.unwrap_or(shared.chance),
            per_level: self.per_level.unwrap_or(shared.per_level),
            favoured,
            banned,
        })
    }
}

impl ShopOddsSpec {
    /// A depth's table: `over`'s fields (its `[[dungeon.levels]]` entry), else `branch`'s.
    fn layer(over: Option<&Self>, branch: Option<&Self>) -> Self {
        let over = over.cloned().unwrap_or_default();
        let branch = branch.cloned().unwrap_or_default();
        Self {
            chance: over.chance.or(branch.chance),
            kinds: over.kinds.or(branch.kinds),
            names: over.names.or(branch.names),
            keeper: over.keeper.or(branch.keeper),
        }
    }

    /// The odds, the shared ones filling in what the table leaves out; `creature_id` resolves
    /// the keeper's creature type.
    fn resolve(
        self,
        shops: &Shops,
        creature_id: impl Fn(&str) -> Option<CreatureId>,
    ) -> Result<ShopOdds, String> {
        let shared = ShopOdds::shared(shops);
        let kinds = match self.kinds {
            Some(names) => (names.iter())
                .map(|k| {
                    shops
                        .id(k)
                        .ok_or_else(|| format!("unknown shop `{k}` (shops.toml)"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => shared.kinds,
        };
        let keeper = match self.keeper {
            Some(k) => (creature_id(&k))
                .filter(|_| k != "player")
                .ok_or_else(|| format!("unknown keeper creature `{k}`"))?,
            None => shared.keeper,
        };
        Ok(ShopOdds {
            chance: self.chance.unwrap_or(shared.chance),
            kinds,
            names: self.names.unwrap_or(shared.names),
            keeper,
        })
    }
}

/// How many traps a level gets unless its branch says otherwise: 2 to 4.
const TRAPS_PER_LEVEL: [u32; 2] = [2, 4];

/// How often a branch's floors get chests (spec §8 "Chests"; `mapgen/chest.rs`): with `chance`
/// percent a floor has some, between `count`'s least and most. One floor in five has one unless
/// the branch says otherwise (docs/design/chests-and-treasure.md decision 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChestOdds {
    #[serde(default = "ChestOdds::default_chance")]
    pub chance: u8,
    #[serde(default = "ChestOdds::default_count")]
    pub count: [u8; 2],
}

impl ChestOdds {
    const fn default_chance() -> u8 {
        20
    }

    const fn default_count() -> [u8; 2] {
        [1, 1]
    }
}

impl Default for ChestOdds {
    fn default() -> Self {
        Self {
            chance: Self::default_chance(),
            count: Self::default_count(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelsSpec {
    from: u32,
    to: Option<u32>,
    generator: Option<Generator>,
    size: Option<[i32; 2]>,
    temperature: Option<i8>,
    danger: Option<DangerSpec>,
    /// One draw in this many among the dread types on these levels; default the branch's.
    dread_one_in: Option<u16>,
    room_effects: Option<RoomOddsSpec>,
    shops: Option<ShopOddsSpec>,
    /// Vaults (by id) stamped on each of these levels, and nowhere else.
    #[serde(default)]
    vaults: Vec<String>,
}

/// How one level of a branch is generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelPlan {
    pub generator: Generator,
    pub width: i32,
    pub height: i32,
    /// Its temperature (spec §5 "Temperature"); `None`: the dungeon default.
    pub temperature: Option<Temperature>,
    /// How dangerous it is (spec §7 "Danger levels"); `None`: danger 0, nothing comes.
    pub danger: Option<DangerRule>,
    /// Its own `dread_one_in` (a `[[dungeon.levels]]` entry's); `None`: the branch's.
    pub dread_one_in: Option<u16>,
}

/// A validated dungeon branch.
#[derive(Clone, Debug)]
pub struct DungeonDef {
    pub id: String,
    pub name: String,
    /// A line said each time the player arrives on its first level (`id:1`).
    pub arrival: Option<String>,
    /// Number of levels: `id:1` to `id:depth`. An endless branch has no last floor and 1 here:
    /// the one row of its tables, which is every floor's.
    pub depth: u32,
    /// The level of another branch its deepest level's `>` leads to, which gets a `<` back up
    /// (checked by `Content::validate`); `None`: the deepest level has no way down.
    pub below: Option<String>,
    /// Floors without end (spec §7 "The endless branch"); `None`: `depth` levels.
    pub endless: Option<Endless>,
    /// An endless branch's layer bosses, by depth (none for any other branch).
    pub bosses: Vec<LayerBoss>,
    /// An endless branch's `dread_one_in` by depth (`[dungeon.endless.dread]`), if it gives one.
    pub dread: Option<DreadCurve>,
    pub ambient: Light,
    /// Creatures generators may place on patrol.
    pub creatures: Vec<CreatureId>,
    /// Traps generators may hide on a level (spec §8 "Traps"); none: the branch has none.
    pub traps: Vec<TrapId>,
    /// How many traps a level gets, at least and at most.
    pub traps_per_level: [u32; 2],
    /// How many of a level's doors are hidden (secret doors, spec §8 "Searching"), at least and
    /// at most: only doors the level can do without, so a level may get fewer.
    pub secret_doors: [u32; 2],
    /// How much harder the hidden features on a level (secret doors, hidden traps) are to find
    /// for each level below the first ([`DungeonDef::hidden_at`]).
    pub hidden_per_depth: u8,
    /// How much harder the locks on a level are to pick for each level below the first (spec §8
    /// "Doors"; [`DungeonDef::locks_at`]).
    pub locks_per_depth: u8,
    /// How many items of loot lie on the floor of each level, fewest and most (spec §8 "Magic
    /// items"; item level: the depth).
    pub loot: [u8; 2],
    /// How often its floors get chests (spec §8 "Chests").
    pub chests: ChestOdds,
    /// The key to the rooms the rooms generator locks (spec §8 "Doors"): it locks them with the
    /// lock this key opens and leaves one where the player can reach it. `None`: no locked rooms.
    pub key: Option<ObjectId>,
    /// What lives on its floors and how it comes back (spec §7 "Danger levels"); `None`: its
    /// floors have danger 0 and only the generator's patrols.
    pub spawning: Option<Spawning>,
    /// One plan per depth, index `depth - 1`.
    plans: Vec<LevelPlan>,
    /// The campaign whose `data/dungeons/` it is in; `None` outside every campaign.
    pub campaign: Option<String>,
    /// The special-room odds per depth, index `depth - 1`.
    room_odds: Vec<RoomOdds>,
    /// The shop odds per depth, index `depth - 1`; empty without `shops.toml`.
    shop_odds: Vec<ShopOdds>,
    /// The vaults (ids) each depth stamps for certain, index `depth - 1` (checked against the
    /// vaults by `Content::load_dungeons`).
    vaults: Vec<Vec<String>>,
}

impl DungeonDef {
    /// The row of the per-depth tables level `depth` (1-based) reads: an endless branch's one row
    /// at every depth.
    fn row(&self, depth: u32) -> Option<usize> {
        let i = usize::try_from(depth.checked_sub(1)?).ok()?;
        Some(if self.endless.is_some() { 0 } else { i })
    }

    /// The plan for level `depth` (1-based), if the branch has that level. On an endless branch
    /// its generator and size are the rooms': a floor's own are drawn as it is made
    /// (`mapgen::generate_for`).
    pub fn plan(&self, depth: u32) -> Option<LevelPlan> {
        self.plans.get(self.row(depth)?).copied()
    }

    /// Whether level `depth` (1-based) has a way down.
    pub fn leads_down(&self, depth: u32) -> bool {
        self.endless.is_some() || depth < self.depth || self.below.is_some()
    }

    /// Where level `depth`'s `>` leads: the next level, or below the deepest, [`Self::below`].
    pub fn down_to(&self, depth: u32) -> Option<String> {
        if self.endless.is_none() && depth >= self.depth {
            return self.below.clone();
        }
        Some(self.level_id(depth + 1))
    }

    /// Level `depth`'s own `dread_one_in` (1-based): an endless branch's curve there, else its
    /// `[[dungeon.levels]]` entry's; `None`: the branch's (docs/design/monster-dread.md §8).
    pub fn dread_one_in(&self, depth: u32) -> Option<u16> {
        match &self.dread {
            Some(curve) => curve.at(depth),
            None => self.plan(depth)?.dread_one_in,
        }
    }

    /// The special-room odds of level `depth` (1-based), if the branch has that level.
    pub fn room_odds(&self, depth: u32) -> Option<&RoomOdds> {
        self.room_odds.get(self.row(depth)?)
    }

    /// The shop odds of level `depth` (1-based); `None` without `shops.toml`.
    pub fn shop_odds(&self, depth: u32) -> Option<&ShopOdds> {
        self.shop_odds.get(self.row(depth)?)
    }

    /// The vaults (ids) level `depth` (1-based) stamps for certain: a `[[dungeon.levels]]`
    /// entry's `vaults`. A vault named by any branch level is stamped only where it is named.
    pub fn vaults(&self, depth: u32) -> &[String] {
        (self.row(depth))
            .and_then(|i| self.vaults.get(i))
            .map_or(&[], Vec::as_slice)
    }

    /// The layer boss of floor `depth`, if it has one (slain or not).
    pub fn boss_at(&self, depth: u32) -> Option<&LayerBoss> {
        self.bosses.iter().find(|b| b.depth == depth)
    }

    /// Every vault id some level of the branch names.
    pub fn named_vaults(&self) -> impl Iterator<Item = &str> {
        self.vaults.iter().flatten().map(String::as_str)
    }

    /// The level id of `depth`, e.g. `depths:3`.
    pub fn level_id(&self, depth: u32) -> String {
        format!("{}:{depth}", self.id)
    }

    /// What finding a hidden feature on level `depth` takes beyond its own difficulty (spec §8
    /// "Searching"): [`DungeonDef::hidden_per_depth`] for each level below the first.
    pub fn hidden_at(&self, depth: u32) -> i32 {
        per_level_below(self.hidden_per_depth, depth)
    }

    /// What picking a lock on level `depth` takes beyond the lock's own difficulty (spec §8
    /// "Doors"): [`DungeonDef::locks_per_depth`] for each level below the first.
    pub fn locks_at(&self, depth: u32) -> i32 {
        per_level_below(self.locks_per_depth, depth)
    }

    /// The display name of level `depth`, e.g. `The Depths 3`.
    pub fn level_name(&self, depth: u32) -> String {
        format!("{} {depth}", self.name)
    }
}

/// An endless branch's `[[dungeon.endless.boss]]` rows, by depth: each depth once, each creature
/// known. What the content alone can tell (unique, the lair's marks) `Content` checks.
fn resolve_bosses(
    id: &str,
    endless: Option<&EndlessSpec>,
    creature_id: impl Fn(&str) -> Option<(CreatureId, u8)>,
) -> Result<Vec<LayerBoss>, String> {
    let mut bosses: Vec<LayerBoss> = Vec::new();
    for row in endless.map_or(&[][..], |e| &e.boss) {
        if bosses.iter().any(|b| b.depth == row.depth) {
            return Err(format!(
                "dungeon `{id}`: two layer bosses at depth {}",
                row.depth
            ));
        }
        let (creature, _) = creature_id(&row.creature)
            .ok_or_else(|| format!("dungeon `{id}`: unknown layer boss `{}`", row.creature))?;
        bosses.push(LayerBoss {
            depth: row.depth,
            creature,
            vault: row.vault.clone(),
        });
    }
    bosses.sort_by_key(|b| b.depth);
    Ok(bosses)
}

/// `per_level` for each level below the first, on level `depth`.
fn per_level_below(per_level: u8, depth: u32) -> i32 {
    let below = i32::try_from(depth.saturating_sub(1)).unwrap_or(i32::MAX);
    i32::from(per_level).saturating_mul(below)
}

/// Splits a branch level id `branch:N` into the branch id and the depth `N` (≥ 1).
pub fn parse_branch_level(id: &str) -> Option<(&str, u32)> {
    let (branch, depth) = id.rsplit_once(':')?;
    if branch.is_empty() || depth.is_empty() || !depth.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let depth: u32 = depth.parse().ok()?;
    (depth >= 1).then_some((branch, depth))
}

impl DungeonFile {
    /// Validates every branch; `creature_id` resolves creature names (with their levels),
    /// `key_id` the names of keys (objects that open a lock), `trap_id` trap names; `rooms` and
    /// `shops` are the shared special-room and shop odds a branch's own tables fall back to.
    pub(super) fn resolve(
        self,
        file: &str,
        creature_id: impl Fn(&str) -> Option<(CreatureId, u8)>,
        key_id: impl Fn(&str) -> Option<ObjectId>,
        trap_id: impl Fn(&str) -> Option<TrapId>,
        rooms: &RoomEffects,
        shops: Option<&Shops>,
        defaults: Option<Odds>,
    ) -> Result<Vec<DungeonDef>, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let mut defs: Vec<DungeonDef> = Vec::new();
        for mut spec in self.dungeon {
            let id = spec.id;
            if id.is_empty() || id.contains(':') || id.chars().any(char::is_whitespace) {
                return Err(err(format!(
                    "dungeon id `{id}` must be non-empty, without `:` or spaces"
                )));
            }
            if defs.iter().any(|d| d.id == id) {
                return Err(err(format!("duplicate dungeon id `{id}`")));
            }
            let depth_of = match (spec.depth, &spec.endless) {
                (Some(depth), None) => depth,
                // One row of tables: every floor's.
                (None, Some(_)) => 1,
                _ => {
                    return Err(err(format!(
                        "dungeon `{id}`: give `depth` or `endless`, one of them"
                    )));
                }
            };
            if spec.below.is_some() && spec.endless.is_some() {
                return Err(err(format!(
                    "dungeon `{id}`: an endless branch has no deepest level for `below`"
                )));
            }
            let check_size = |generator: Generator, [w, h]: [i32; 2]| {
                let (min, max) = generator.size_limits();
                if (min[0]..=max[0]).contains(&w) && (min[1]..=max[1]).contains(&h) {
                    Ok(())
                } else {
                    Err(err(format!(
                        "dungeon `{id}`: size {w}x{h} is outside {}x{} to {}x{} for {generator:?}",
                        min[0], min[1], max[0], max[1]
                    )))
                }
            };
            check_size(spec.generator, spec.size)?;
            if let Some(e) = &spec.endless {
                check_size(Generator::Caves, e.caves_size)?;
                if spec.generator != Generator::Rooms {
                    return Err(err(format!(
                        "dungeon `{id}`: an endless branch's `generator` is \"rooms\""
                    )));
                }
                if !spec.levels.is_empty() {
                    return Err(err(format!(
                        "dungeon `{id}`: an endless branch has no `[[dungeon.levels]]`"
                    )));
                }
            }
            let creatures = spec
                .creatures
                .iter()
                .map(|c| {
                    creature_id(c)
                        .map(|(c, _)| c)
                        .ok_or_else(|| err(format!("dungeon `{id}` uses unknown creature `{c}`")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let spawning = SpawningSpec {
                danger: spec.danger,
                population: spec.population,
                population_count: spec.population_count,
                odds: OddsSpec {
                    stray: spec.stray,
                    out_of_depth: spec.out_of_depth,
                    deeper: spec.deeper,
                    deeper_cap: spec.deeper_cap,
                    dread_one_in: spec.dread_one_in,
                    neutral: spec.neutral,
                },
                stray_pool: spec.stray_pool,
                spawn: spec.spawn,
            }
            .resolve(true, &creatures, &creature_id, defaults)
            .map_err(|m| err(format!("dungeon `{id}`: {m}")))?;
            let branch_danger = spawning.as_ref().map(|(rule, _)| *rule);
            if spec.endless.is_some() && branch_danger.is_none_or(|r| r.scales()) {
                return Err(err(format!(
                    "dungeon `{id}`: an endless branch needs a `danger`, without `player`"
                )));
            }
            let mut plans = Vec::new();
            let (mut room_odds, mut shop_odds) = (Vec::new(), Vec::new());
            let mut vaults = Vec::new();
            for depth in 1..=depth_of {
                let over = spec
                    .levels
                    .iter()
                    .find(|l| (l.from..=l.to.unwrap_or(l.from)).contains(&depth));
                let (generator, [width, height], temperature) = match over {
                    Some(l) => (
                        l.generator.unwrap_or(spec.generator),
                        l.size.unwrap_or(spec.size),
                        l.temperature.or(spec.temperature),
                    ),
                    None => (spec.generator, spec.size, spec.temperature),
                };
                let danger = match over.and_then(|l| l.danger) {
                    Some(_) if branch_danger.is_none() => {
                        return Err(err(format!(
                            "dungeon `{id}`: a level's `danger` needs the branch's"
                        )));
                    }
                    Some(d) => Some(
                        d.resolve(1, true)
                            .map_err(|m| err(format!("dungeon `{id}`: {m}")))?,
                    ),
                    None => branch_danger,
                };
                let dread_one_in = over.and_then(|l| l.dread_one_in);
                if dread_one_in.is_some() && branch_danger.is_none() {
                    return Err(err(format!(
                        "dungeon `{id}`: a level's `dread_one_in` needs the branch's `danger`"
                    )));
                }
                check_size(generator, [width, height])?;
                plans.push(LevelPlan {
                    generator,
                    width,
                    height,
                    temperature: temperature.map(|c| Temperature::degrees(c.into())),
                    danger,
                    dread_one_in,
                });
                vaults.push(over.map(|l| l.vaults.clone()).unwrap_or_default());
                let room_spec = RoomOddsSpec::layer(
                    over.and_then(|l| l.room_effects.as_ref()),
                    spec.room_effects.as_ref(),
                );
                room_odds.push(
                    room_spec
                        .resolve(rooms)
                        .map_err(|m| err(format!("dungeon `{id}`: room_effects: {m}")))?,
                );
                let shop_spec =
                    ShopOddsSpec::layer(over.and_then(|l| l.shops.as_ref()), spec.shops.as_ref());
                match shops {
                    Some(shops) => shop_odds.push(
                        shop_spec
                            .resolve(shops, |name| creature_id(name).map(|(id, _)| id))
                            .map_err(|m| err(format!("dungeon `{id}`: shops: {m}")))?,
                    ),
                    None if spec.shops.is_some() || over.is_some_and(|l| l.shops.is_some()) => {
                        return Err(err(format!(
                            "dungeon `{id}`: `shops` needs data/shops.toml"
                        )));
                    }
                    None => {}
                }
            }
            for l in &spec.levels {
                let to = l.to.unwrap_or(l.from);
                if l.from < 1 || to < l.from || to > depth_of {
                    return Err(err(format!(
                        "dungeon `{id}`: levels {}..={to} are outside 1..={}",
                        l.from, depth_of
                    )));
                }
            }
            let traps = (spec.traps.iter())
                .map(|t| {
                    trap_id(t).ok_or_else(|| err(format!("dungeon `{id}` uses unknown trap `{t}`")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let traps_per_level = spec.traps_per_level.unwrap_or(TRAPS_PER_LEVEL);
            // Each `[least, most]` is a range a draw is made in: never empty.
            let [least, most] = traps_per_level;
            if least > most {
                return Err(err(format!(
                    "dungeon `{id}`: traps_per_level is [least, most]"
                )));
            }
            if spec.traps_per_level.is_some() && traps.is_empty() {
                return Err(err(format!(
                    "dungeon `{id}` has traps_per_level but no `traps`"
                )));
            }
            if spec.secret_doors[0] > spec.secret_doors[1] {
                return Err(err(format!(
                    "dungeon `{id}`: secret_doors is [least, most]"
                )));
            }
            if spec.loot[0] > spec.loot[1] {
                return Err(err(format!("dungeon `{id}`: `loot` is [fewest, most]")));
            }
            if spec.chests.count[0] > spec.chests.count[1] {
                return Err(err(format!(
                    "dungeon `{id}`: `chests.count` is [least, most]"
                )));
            }
            let key = match &spec.key {
                Some(k) => Some(key_id(k).ok_or_else(|| {
                    err(format!(
                        "dungeon `{id}`: `key = \"{k}\"` is no key (objects.toml)"
                    ))
                })?),
                None => None,
            };
            let bosses = resolve_bosses(&id, spec.endless.as_ref(), &creature_id).map_err(err)?;
            let dread = (spec.endless.as_mut())
                .and_then(|e| e.dread.take())
                .map(DreadCurveSpec::resolve)
                .transpose()
                .map_err(|m| err(format!("dungeon `{id}`: {m}")))?;
            defs.push(DungeonDef {
                name: spec.name.unwrap_or_else(|| id.clone()),
                arrival: spec.arrival,
                id,
                depth: depth_of,
                bosses,
                dread,
                below: spec.below,
                endless: spec.endless.map(|e| Endless {
                    caves: e.caves,
                    caves_size: (e.caves_size[0], e.caves_size[1]),
                    roster_depth: u32::MAX,
                }),
                ambient: spec.ambient,
                creatures,
                traps,
                traps_per_level,
                secret_doors: spec.secret_doors,
                hidden_per_depth: spec.hidden_per_depth,
                locks_per_depth: spec.locks_per_depth,
                loot: spec.loot,
                chests: spec.chests,
                key,
                spawning: spawning.map(|(_, s)| s),
                plans,
                campaign: None,
                room_odds,
                shop_odds,
                vaults,
            });
        }
        Ok(defs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(text: &str) -> Result<Vec<DungeonDef>, ContentError> {
        let file: DungeonFile =
            toml::from_str(text).map_err(|e| ContentError::new("d", e.to_string()))?;
        file.resolve(
            "d",
            |c| (c == "rat").then_some((CreatureId(1), 1)),
            |k| (k == "key").then_some(ObjectId(2)),
            |t| (t == "pit").then_some(TrapId(0)),
            &rooms(),
            None,
            Some(Odds {
                stray: 5,
                out_of_depth: 5,
                deeper: 5,
                deeper_cap: 10,
                dread_one_in: 100,
                neutral: 100,
            }),
        )
    }

    /// Three effects, a 7 % chance, one a level.
    fn rooms() -> RoomEffects {
        let effect = |id: &str, mark: char| {
            format!(
                "[[effect]]\nid = \"{id}\"\nname = \"{id}\"\nmark = \"{mark}\"\nenter = \"In.\"\n"
            )
        };
        let text = format!(
            "chance = 7\nper_level = 1\n{}{}{}",
            effect("hushed", 'h'),
            effect("shadowed", 's'),
            effect("echoing", 'e')
        );
        RoomEffects::parse("r", &text, |_| None).expect("room effects")
    }

    #[test]
    fn branches_and_depths_set_their_special_room_odds() {
        let base = "[[dungeon]]\nid = \"a\"\ngenerator = \"rooms\"\ndepth = 4\nsize = [80, 21]\n";
        let shared = resolve(base).unwrap();
        for depth in 1..=4 {
            assert_eq!(
                shared[0].room_odds(depth),
                Some(&RoomOdds::shared(&rooms())),
                "no table: the shared odds"
            );
        }
        assert_eq!(shared[0].room_odds(5), None);
        assert_eq!(shared[0].shop_odds(1), None, "no shops.toml");
        let own = resolve(&format!(
            "{base}room_effects = {{ chance = 25, per_level = 2, favoured = {{ echoing = 40 }}, \
             banned = [\"hushed\"] }}\n\
             [[dungeon.levels]]\nfrom = 3\nto = 4\nroom_effects = {{ favoured = {{ shadowed = 30, \
             hushed = 20 }}, banned = [] }}\n"
        ))
        .unwrap();
        let odds = |depth| own[0].room_odds(depth).cloned().unwrap();
        assert_eq!(
            odds(1),
            RoomOdds {
                chance: 25,
                per_level: 2,
                favoured: vec![(RoomEffectId(2), 40)],
                banned: vec![RoomEffectId(0)],
            }
        );
        // A depth's table overrides the branch's field by field; favoured in effect order.
        assert_eq!(
            odds(3),
            RoomOdds {
                chance: 25,
                per_level: 2,
                favoured: vec![(RoomEffectId(0), 20), (RoomEffectId(1), 30)],
                banned: vec![],
            }
        );
        assert_eq!(odds(3).share(RoomEffectId(1)), Some(30));
        assert!(odds(1).bans(RoomEffectId(0)) && !odds(1).bans(RoomEffectId(2)));
        let only_chance = resolve(&format!("{base}room_effects = {{ chance = 40 }}\n")).unwrap();
        let odds = only_chance[0].room_odds(2).unwrap();
        assert_eq!((odds.chance, odds.per_level), (40, 1));
        assert!(odds.favoured.is_empty() && odds.banned.is_empty());
        for (bad, says) in [
            (
                "room_effects = { favoured = { lava = 10 } }\n",
                "unknown effect `lava`",
            ),
            ("room_effects = { banned = [\"lava\"] }\n", "unknown effect"),
            (
                "room_effects = { effects = [\"hushed\"] }\n",
                "unknown field",
            ),
            ("room_effects = { odds = 3 }\n", "unknown field"),
            ("shops = { chance = 0 }\n", "shops.toml"),
        ] {
            let e = resolve(&format!("{base}{bad}")).unwrap_err().message;
            assert!(e.contains(says), "{bad}: {e}");
        }
    }

    #[test]
    fn branches_say_how_often_their_floors_get_chests() {
        let base = "[[dungeon]]\nid = \"a\"\ngenerator = \"rooms\"\ndepth = 2\nsize = [80, 21]\n";
        let odds = |text: &str| resolve(&format!("{base}{text}")).map(|d| d[0].chests);
        let default = ChestOdds {
            chance: 20,
            count: [1, 1],
        };
        assert_eq!(odds("").unwrap(), default, "one floor in five");
        assert_eq!(
            odds("chests = { chance = 50, count = [1, 3] }\n").unwrap(),
            ChestOdds {
                chance: 50,
                count: [1, 3]
            }
        );
        assert_eq!(odds("chests = { chance = 0 }\n").unwrap().count, [1, 1]);
        for bad in ["chests = { count = [2, 1] }\n", "chests = { often = 3 }\n"] {
            assert!(odds(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn branches_name_the_traps_their_levels_hide() {
        let base = "[[dungeon]]\nid = \"a\"\ngenerator = \"rooms\"\ndepth = 2\nsize = [80, 21]\n";
        let none = resolve(base).unwrap();
        assert!(none[0].traps.is_empty());
        let pits = resolve(&format!("{base}traps = [\"pit\"]\n")).unwrap();
        assert_eq!(
            (pits[0].traps.as_slice(), pits[0].traps_per_level),
            ([TrapId(0)].as_slice(), [2, 4])
        );
        let more = resolve(&format!(
            "{base}traps = [\"pit\"]\ntraps_per_level = [5, 5]\n"
        ));
        assert_eq!(more.unwrap()[0].traps_per_level, [5, 5]);
        for bad in [
            "traps = [\"dragon_trap\"]\n",
            "traps = [\"pit\"]\ntraps_per_level = [3, 1]\n",
            "traps_per_level = [1, 2]\n",
        ] {
            assert!(resolve(&format!("{base}{bad}")).is_err(), "{bad}");
        }
    }

    #[test]
    fn parses_branch_level_ids() {
        assert_eq!(parse_branch_level("depths:3"), Some(("depths", 3)));
        assert_eq!(parse_branch_level("depths:0"), None);
        assert_eq!(parse_branch_level("depths:"), None);
        assert_eq!(parse_branch_level(":3"), None);
        assert_eq!(parse_branch_level("depths:+3"), None);
        assert_eq!(parse_branch_level("hub"), None);
    }

    #[test]
    fn per_depth_overrides_pick_generator_and_size() {
        let defs = resolve(
            "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 4\nsize = [80, 21]\n\
             creatures = [\"rat\"]\n\
             [[dungeon.levels]]\nfrom = 2\nto = 3\ngenerator = \"caves\"\nsize = [120, 60]\n",
        )
        .unwrap();
        let d = &defs[0];
        assert_eq!(d.name, "d");
        assert_eq!(d.plan(1).map(|p| p.generator), Some(Generator::Rooms));
        assert_eq!(
            d.plan(3),
            Some(LevelPlan {
                generator: Generator::Caves,
                width: 120,
                height: 60,
                temperature: None,
                danger: None,
                dread_one_in: None,
            })
        );
        assert_eq!(d.plan(4).map(|p| p.width), Some(80));
        assert_eq!(d.plan(5), None);
        assert_eq!(d.plan(0), None);
        assert_eq!(d.level_id(2), "d:2");
    }

    #[test]
    fn branches_and_depths_set_the_temperature() {
        let defs = resolve(
            "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 4\nsize = [80, 21]\n\
             temperature = 9\n\
             [[dungeon.levels]]\nfrom = 3\nto = 4\ntemperature = -2\n",
        )
        .unwrap();
        let t = |depth| defs[0].plan(depth).and_then(|p| p.temperature);
        assert_eq!(t(1), Some(Temperature::degrees(9)));
        assert_eq!(t(3), Some(Temperature::degrees(-2)));
        // The override keeps the branch's generator and size.
        assert_eq!(defs[0].plan(4).map(|p| p.generator), Some(Generator::Rooms));
        let plain = "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 1\nsize = [80, 21]\n";
        assert_eq!(
            resolve(plain).unwrap()[0].plan(1).unwrap().temperature,
            None
        );
    }

    #[test]
    fn a_depths_dread_one_in_wins_over_the_branchs_over_the_defaults() {
        let base = "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 3\nsize = [80, 21]\n\
                    creatures = [\"rat\"]\ndanger = 2\n";
        let odds = |text: &str| {
            let defs = resolve(&format!("{base}{text}")).unwrap();
            let branch = defs[0].spawning.as_ref().unwrap().odds.dread_one_in;
            [1, 2, 3].map(|depth| defs[0].dread_one_in(depth).unwrap_or(branch))
        };
        assert_eq!(odds(""), [100, 100, 100], "the defaults'");
        assert_eq!(odds("dread_one_in = 60\n"), [60, 60, 60], "the branch's");
        assert_eq!(
            odds("dread_one_in = 60\n[[dungeon.levels]]\nfrom = 2\nto = 3\ndread_one_in = 20\n"),
            [60, 20, 20],
            "the depths' own"
        );
        assert_eq!(
            odds("[[dungeon.levels]]\nfrom = 3\ndread_one_in = 20\n"),
            [100, 100, 20]
        );
        let no_danger = "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 3\n\
                         size = [80, 21]\n[[dungeon.levels]]\nfrom = 3\ndread_one_in = 20\n";
        assert!(
            resolve(no_danger).is_err(),
            "a depth's needs the branch's danger"
        );
    }

    #[test]
    fn an_endless_branch_reads_its_dread_curve_at_the_true_depth() {
        let text = "[[dungeon]]\nid = \"g\"\ngenerator = \"rooms\"\nsize = [80, 21]\n\
                    creatures = [\"rat\"]\ndanger = 1\n\
                    [dungeon.endless]\ncaves = 0\ncaves_size = [100, 40]\n\
                    dread = { steps = [[30, 80], [70, 40]], halves = 10, least = 5 }\n";
        let defs = resolve(text).unwrap();
        let at = |depth| defs[0].dread_one_in(depth);
        assert_eq!(at(1), None, "above the first step: the branch's");
        assert_eq!((at(30), at(69)), (Some(80), Some(80)));
        assert_eq!(
            (at(70), at(80), at(90), at(100), at(500)),
            (Some(40), Some(20), Some(10), Some(5), Some(5))
        );
        let bad = text.replace("halves = 10", "halves = 0");
        assert!(resolve(&bad).is_err());
    }

    #[test]
    fn floors_take_the_branch_danger_or_their_own() {
        let base = "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 3\nsize = [80, 21]\n\
                    creatures = [\"rat\"]\n";
        let none = resolve(base).unwrap();
        assert!(none[0].spawning.is_none() && none[0].plan(1).unwrap().danger.is_none());
        let defs = resolve(&format!(
            "{base}danger = 2\n[[dungeon.levels]]\nfrom = 3\ndanger = {{ base = 9, per_depth = 0 }}\n"
        ))
        .unwrap();
        let at = |depth| defs[0].plan(depth).unwrap().danger.unwrap().at(depth, 1);
        assert_eq!((at(1), at(2), at(3)), (2, 3, 9));
        // The population defaults to the patrols.
        assert_eq!(
            defs[0].spawning.as_ref().unwrap().population,
            [CreatureId(1)]
        );
        let lone = format!("{base}[[dungeon.levels]]\nfrom = 3\ndanger = 4\n");
        assert!(
            resolve(&lone)
                .unwrap_err()
                .message
                .contains("needs the branch")
        );
    }

    #[test]
    fn rejects_bad_branches() {
        let base = "[[dungeon]]\ngenerator = \"rooms\"\ndepth = 2\nsize = [80, 21]\n";
        assert!(resolve(&format!("{base}id = \"a:b\"\n")).is_err());
        assert!(resolve(&format!("{base}id = \"a\"\ncreatures = [\"dragon\"]\n")).is_err());
        assert!(resolve(&format!("{base}id = \"a\"\nkey = \"dagger\"\n")).is_err());
        assert_eq!(
            resolve(&format!("{base}id = \"a\"\nkey = \"key\"\n")).unwrap()[0].key,
            Some(ObjectId(2))
        );
        let tiny = "[[dungeon]]\nid = \"a\"\ngenerator = \"caves\"\ndepth = 2\nsize = [80, 21]\n";
        assert!(resolve(tiny).unwrap_err().message.contains("outside"));
        let range = format!(
            "{base}id = \"a\"\n[[dungeon.levels]]\nfrom = 2\nto = 3\ngenerator = \"rooms\"\n"
        );
        assert!(resolve(&range).is_err());
        let dup = format!("{base}id = \"a\"\n{base}id = \"a\"\n");
        assert!(resolve(&dup).unwrap_err().message.contains("duplicate"));
    }
}
