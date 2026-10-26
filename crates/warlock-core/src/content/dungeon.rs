//! Dungeon branches (`assets/data/dungeons.toml`): stacks of procedurally generated levels.
//!
//! A branch `depths` with `depth = 8` provides the levels `depths:1` … `depths:8`. Stairs in any
//! level may lead to `depths:N` (`to = "depths:1"`); such a level is generated on its first visit
//! from the run seed (see [`crate::mapgen`]) and then persists like a hand-made one. Its `<` leads
//! to `depths:N-1` (on depth 1: back to the level it was first entered from) and its `>` to
//! `depths:N+1` (the deepest level has none).
//!
//! ```toml
//! [[dungeon]]
//! id = "depths"
//! name = "The Depths"          # level names: "The Depths 3"; default: the id
//! generator = "rooms"          # "rooms" (rooms and corridors) or "caves" (cellular automata)
//! depth = 8
//! size = [80, 21]
//! creatures = ["rat", "kobold"] # patrolling creatures generators may place; default none
//! traps = ["arrow", "pit"]      # traps generators may hide (spec §8 "Traps"); default none
//! traps_per_level = [2, 4]      # how many a level gets, at least and at most; default [2, 4]
//! secret_doors = [0, 2]         # doors hidden on each level (spec §8 "Searching"); default none
//! hidden_per_depth = 1          # hidden features this much harder to find per level down
//! locks_per_depth = 1           # locks this much harder to pick per level down
//! loot = [2, 4]                 # items of loot on each level's floor; default none
//! ambient = [0, 0, 0, 0]        # optional
//! temperature = 9               # °C; default the dungeon default (10)
//! key = "rusty_key"             # optional: rooms levels lock a few rooms with its lock
//! danger = 4                    # optional (spec §7 "Danger levels"; `super::danger`): its
//! population = ["rat"]          # floors' danger, who lives there, how many, and how they
//! population_count = 40         # come back; none: danger 0, nothing comes
//! stray = 5                    # the draws' odds (`super::danger::Odds`); default
//! out_of_depth = 5             # data/dungeons.toml's `[defaults]`
//! deeper = 5
//! deeper_cap = 10
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
    CountSpec, DangerRule, DangerSpec, Odds, OddsSpec, SpawnSpec, Spawning, SpawningSpec,
};
use super::dialogue::{MAX_REPLY, check_lines};
use super::object::ObjectId;
use super::room_effect::{RoomEffectId, RoomEffects};
use super::shop::{ShopId, Shops};
use super::trap::TrapId;
use crate::light::Light;
use crate::temperature::{LEVEL_TEMPERATURES, Temperature};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The most items of loot a branch level's floor may hold.
pub const MAX_LOOT: u8 = 50;

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
    depth: u32,
    size: [i32; 2],
    #[serde(default)]
    ambient: Light,
    #[serde(default)]
    creatures: Vec<String>,
    /// How many items of loot lie on the floor of each level, `[fewest, most]`.
    #[serde(default)]
    loot: [u8; 2],
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
    out_of_depth: Option<u8>,
    deeper: Option<u8>,
    deeper_cap: Option<u8>,
    spawn: Option<SpawnSpec>,
    room_effects: Option<RoomOddsSpec>,
    shops: Option<ShopOddsSpec>,
    #[serde(default)]
    levels: Vec<LevelsSpec>,
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
    /// Retired: refused with a pointer to `favoured` and `banned`.
    effects: Option<toml::Value>,
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

/// The most special rooms a branch may give a level.
const MAX_ROOMS_PER_LEVEL: u8 = 8;

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

/// A percent from data: 0 to 100.
fn check_percent(chance: Option<u8>) -> Result<(), String> {
    match chance {
        Some(c) if c > 100 => Err(format!("chance {c} is a percent: 0 to 100")),
        _ => Ok(()),
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
            effects: over.effects.or(branch.effects),
        }
    }

    /// The odds, the shared ones filling in what the table leaves out.
    fn resolve(self, rooms: &RoomEffects) -> Result<RoomOdds, String> {
        if self.effects.is_some() {
            return Err(
                "`effects` is retired: name `favoured` effects (each with its share of \
                        the draws) and `banned` ones instead"
                    .into(),
            );
        }
        let shared = RoomOdds::shared(rooms);
        check_percent(self.chance)?;
        if self
            .per_level
            .is_some_and(|n| !(1..=MAX_ROOMS_PER_LEVEL).contains(&n))
        {
            return Err(format!("per_level must be 1 to {MAX_ROOMS_PER_LEVEL}"));
        }
        let id = |e: &str| {
            (rooms.id(e)).ok_or_else(|| format!("unknown effect `{e}` (room_effects.toml)"))
        };
        let mut favoured = Vec::new();
        for (name, &share) in self.favoured.iter().flatten() {
            if !(1..=100).contains(&share) {
                return Err(format!("favoured `{name}`: a share is 1 to 100 (percent)"));
            }
            favoured.push((id(name)?, share));
        }
        favoured.sort();
        let mut banned = (self.banned.iter().flatten())
            .map(|e| id(e))
            .collect::<Result<Vec<_>, _>>()?;
        banned.sort();
        banned.dedup();
        if let Some(&(both, _)) = favoured.iter().find(|(e, _)| banned.contains(e)) {
            let name = &rooms.get(both).id;
            return Err(format!("`{name}` is both favoured and banned"));
        }
        let shares: u32 = favoured.iter().map(|&(_, s)| u32::from(s)).sum();
        if shares > 100 {
            return Err(format!(
                "the favoured shares add up to {shares}: at most 100"
            ));
        }
        let odds = RoomOdds {
            chance: self.chance.unwrap_or(shared.chance),
            per_level: self.per_level.unwrap_or(shared.per_level),
            favoured,
            banned,
        };
        if odds.chance > 0 && !rooms.defs().is_empty() && odds.banned.len() >= rooms.defs().len() {
            return Err("every effect is banned while `chance` is above 0".into());
        }
        Ok(odds)
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
        let given = self.chance.is_some()
            || self.kinds.is_some()
            || self.names.is_some()
            || self.keeper.is_some();
        let shared = ShopOdds::shared(shops);
        check_percent(self.chance)?;
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
        if self.names.iter().flatten().any(|n| n.trim().is_empty()) {
            return Err("`names` must not be empty".into());
        }
        let keeper = match self.keeper {
            Some(k) => (creature_id(&k))
                .filter(|_| k != "player")
                .ok_or_else(|| format!("unknown keeper creature `{k}`"))?,
            None => shared.keeper,
        };
        let odds = ShopOdds {
            chance: self.chance.unwrap_or(shared.chance),
            kinds,
            names: self.names.unwrap_or(shared.names),
            keeper,
        };
        if given && odds.chance > 0 && (odds.kinds.is_empty() || odds.names.is_empty()) {
            return Err("`kinds` and `names` must not be empty while `chance` is above 0".into());
        }
        Ok(odds)
    }
}

/// How many traps a level gets unless its branch says otherwise: 2 to 4.
const TRAPS_PER_LEVEL: [u32; 2] = [2, 4];
/// The most traps a generated level may get.
const MAX_TRAPS_PER_LEVEL: u32 = 50;
/// The most secret doors a generated level may get.
const MAX_SECRET_DOORS: u32 = 20;
/// The most a level down may add to how hard hidden features are to find.
const MAX_HIDDEN_PER_DEPTH: u8 = 5;
/// The most a level down may add to how hard locks are to pick.
const MAX_LOCKS_PER_DEPTH: u8 = 5;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelsSpec {
    from: u32,
    to: Option<u32>,
    generator: Option<Generator>,
    size: Option<[i32; 2]>,
    temperature: Option<i8>,
    danger: Option<DangerSpec>,
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
}

/// A validated dungeon branch.
#[derive(Clone, Debug)]
pub struct DungeonDef {
    pub id: String,
    pub name: String,
    /// A line said each time the player arrives on its first level (`id:1`).
    pub arrival: Option<String>,
    /// Number of levels: `id:1` to `id:depth`.
    pub depth: u32,
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
    /// The plan for level `depth` (1-based), if the branch has that level.
    pub fn plan(&self, depth: u32) -> Option<LevelPlan> {
        let i = usize::try_from(depth.checked_sub(1)?).ok()?;
        self.plans.get(i).copied()
    }

    /// The special-room odds of level `depth` (1-based), if the branch has that level.
    pub fn room_odds(&self, depth: u32) -> Option<&RoomOdds> {
        self.room_odds
            .get(usize::try_from(depth.checked_sub(1)?).ok()?)
    }

    /// The shop odds of level `depth` (1-based); `None` without `shops.toml`.
    pub fn shop_odds(&self, depth: u32) -> Option<&ShopOdds> {
        self.shop_odds
            .get(usize::try_from(depth.checked_sub(1)?).ok()?)
    }

    /// The vaults (ids) level `depth` (1-based) stamps for certain: a `[[dungeon.levels]]`
    /// entry's `vaults`. A vault named by any branch level is stamped only where it is named.
    pub fn vaults(&self, depth: u32) -> &[String] {
        let i = depth.checked_sub(1).and_then(|i| usize::try_from(i).ok());
        i.and_then(|i| self.vaults.get(i))
            .map_or(&[], Vec::as_slice)
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
        for spec in self.dungeon {
            let id = spec.id;
            if id.is_empty() || id.contains(':') || id.chars().any(char::is_whitespace) {
                return Err(err(format!(
                    "dungeon id `{id}` must be non-empty, without `:` or spaces"
                )));
            }
            if defs.iter().any(|d| d.id == id) {
                return Err(err(format!("duplicate dungeon id `{id}`")));
            }
            if !(1..=99).contains(&spec.depth) {
                return Err(err(format!("dungeon `{id}`: depth must be 1 to 99")));
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
                },
                spawn: spec.spawn,
            }
            .resolve(true, &creatures, &creature_id, defaults)
            .map_err(|m| err(format!("dungeon `{id}`: {m}")))?;
            let branch_danger = spawning.as_ref().map(|(rule, _)| *rule);
            let mut plans = Vec::new();
            let (mut room_odds, mut shop_odds) = (Vec::new(), Vec::new());
            let mut vaults = Vec::new();
            for depth in 1..=spec.depth {
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
                check_size(generator, [width, height])?;
                if let Some(c) = temperature.filter(|c| !LEVEL_TEMPERATURES.contains(c)) {
                    return Err(err(format!(
                        "dungeon `{id}`: temperature {c} is outside {} to {} (°C)",
                        LEVEL_TEMPERATURES.start(),
                        LEVEL_TEMPERATURES.end()
                    )));
                }
                plans.push(LevelPlan {
                    generator,
                    width,
                    height,
                    temperature: temperature.map(|c| Temperature::degrees(c.into())),
                    danger,
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
                if l.from < 1 || to < l.from || to > spec.depth {
                    return Err(err(format!(
                        "dungeon `{id}`: levels {}..={to} are outside 1..={}",
                        l.from, spec.depth
                    )));
                }
            }
            let traps = (spec.traps.iter())
                .map(|t| {
                    trap_id(t).ok_or_else(|| err(format!("dungeon `{id}` uses unknown trap `{t}`")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let traps_per_level = spec.traps_per_level.unwrap_or(TRAPS_PER_LEVEL);
            let [least, most] = traps_per_level;
            if least > most || most > MAX_TRAPS_PER_LEVEL {
                return Err(err(format!(
                    "dungeon `{id}`: traps_per_level must be [least, most], at most \
                     {MAX_TRAPS_PER_LEVEL}"
                )));
            }
            if spec.traps_per_level.is_some() && traps.is_empty() {
                return Err(err(format!(
                    "dungeon `{id}` has traps_per_level but no `traps`"
                )));
            }
            let [least, most] = spec.secret_doors;
            if least > most || most > MAX_SECRET_DOORS {
                return Err(err(format!(
                    "dungeon `{id}`: secret_doors must be [least, most], at most \
                     {MAX_SECRET_DOORS}"
                )));
            }
            if spec.hidden_per_depth > MAX_HIDDEN_PER_DEPTH {
                return Err(err(format!(
                    "dungeon `{id}`: hidden_per_depth must be 0 to {MAX_HIDDEN_PER_DEPTH}"
                )));
            }
            if spec.locks_per_depth > MAX_LOCKS_PER_DEPTH {
                return Err(err(format!(
                    "dungeon `{id}`: locks_per_depth must be 0 to {MAX_LOCKS_PER_DEPTH}"
                )));
            }
            if spec.loot[0] > spec.loot[1] || spec.loot[1] > MAX_LOOT {
                return Err(err(format!(
                    "dungeon `{id}`: `loot` is [fewest, most], at most {MAX_LOOT}"
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
            if let Some(line) = &spec.arrival {
                check_lines([line], MAX_REPLY)
                    .map_err(|m| err(format!("dungeon `{id}`: `arrival`: {m}")))?;
            }
            defs.push(DungeonDef {
                name: spec.name.unwrap_or_else(|| id.clone()),
                arrival: spec.arrival,
                id,
                depth: spec.depth,
                ambient: spec.ambient,
                creatures,
                traps,
                traps_per_level,
                secret_doors: spec.secret_doors,
                hidden_per_depth: spec.hidden_per_depth,
                locks_per_depth: spec.locks_per_depth,
                loot: spec.loot,
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
            ("room_effects = { chance = 101 }\n", "percent"),
            ("room_effects = { per_level = 0 }\n", "per_level"),
            ("room_effects = { per_level = 9 }\n", "per_level"),
            (
                "room_effects = { favoured = { lava = 10 } }\n",
                "unknown effect `lava`",
            ),
            ("room_effects = { banned = [\"lava\"] }\n", "unknown effect"),
            (
                "room_effects = { effects = [\"hushed\"] }\n",
                "`effects` is retired",
            ),
            (
                "room_effects = { favoured = { hushed = 10 }, banned = [\"hushed\"] }\n",
                "both favoured and banned",
            ),
            (
                "room_effects = { favoured = { hushed = 60, echoing = 50 } }\n",
                "add up to 110",
            ),
            (
                "room_effects = { favoured = { hushed = 0 } }\n",
                "a share is 1 to 100",
            ),
            (
                "room_effects = { banned = [\"hushed\", \"shadowed\", \"echoing\"] }\n",
                "every effect is banned",
            ),
            ("room_effects = { odds = 3 }\n", "unknown field"),
            ("shops = { chance = 0 }\n", "shops.toml"),
        ] {
            let e = resolve(&format!("{base}{bad}")).unwrap_err().message;
            assert!(e.contains(says), "{bad}: {e}");
        }
        // No chance may ban them all.
        assert!(
            resolve(&format!(
                "{base}room_effects = {{ chance = 0, banned = [\"hushed\", \"shadowed\", \
                 \"echoing\"] }}\n"
            ))
            .is_ok()
        );
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
            "traps = [\"pit\"]\ntraps_per_level = [1, 100]\n",
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
        let hot = format!("{plain}temperature = 41\n");
        assert!(resolve(&hot).unwrap_err().message.contains("temperature"));
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
