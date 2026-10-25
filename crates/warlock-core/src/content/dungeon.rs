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
//!
//! [[dungeon.levels]]            # optional per-depth overrides, first match wins
//! from = 3
//! to = 5                        # inclusive; default: just `from`
//! generator = "caves"           # default: the branch's generator
//! size = [120, 60]              # default: the branch's size
//! temperature = 7               # default: the branch's temperature
//! ```

use super::ContentError;
use super::creature::CreatureId;
use super::object::ObjectId;
use super::trap::TrapId;
use crate::light::Light;
use crate::temperature::{LEVEL_TEMPERATURES, Temperature};
use serde::Deserialize;

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
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DungeonSpec {
    id: String,
    name: Option<String>,
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
    #[serde(default)]
    levels: Vec<LevelsSpec>,
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
}

/// How one level of a branch is generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelPlan {
    pub generator: Generator,
    pub width: i32,
    pub height: i32,
    /// Its temperature (spec §5 "Temperature"); `None`: the dungeon default.
    pub temperature: Option<Temperature>,
}

/// A validated dungeon branch.
#[derive(Clone, Debug)]
pub struct DungeonDef {
    pub id: String,
    pub name: String,
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
    /// One plan per depth, index `depth - 1`.
    plans: Vec<LevelPlan>,
    /// The campaign whose `data/dungeons/` it is in; `None` outside every campaign.
    pub campaign: Option<String>,
}

impl DungeonDef {
    /// The plan for level `depth` (1-based), if the branch has that level.
    pub fn plan(&self, depth: u32) -> Option<LevelPlan> {
        let i = usize::try_from(depth.checked_sub(1)?).ok()?;
        self.plans.get(i).copied()
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
    /// Validates every branch; `creature_id` resolves creature names, `key_id` the names of
    /// keys (objects that open a lock), `trap_id` trap names.
    pub(super) fn resolve(
        self,
        file: &str,
        creature_id: impl Fn(&str) -> Option<CreatureId>,
        key_id: impl Fn(&str) -> Option<ObjectId>,
        trap_id: impl Fn(&str) -> Option<TrapId>,
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
            let mut plans = Vec::new();
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
                });
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
            let creatures = spec
                .creatures
                .iter()
                .map(|c| {
                    creature_id(c)
                        .ok_or_else(|| err(format!("dungeon `{id}` uses unknown creature `{c}`")))
                })
                .collect::<Result<Vec<_>, _>>()?;
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
            defs.push(DungeonDef {
                name: spec.name.unwrap_or_else(|| id.clone()),
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
                plans,
                campaign: None,
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
            |c| (c == "rat").then_some(CreatureId(1)),
            |k| (k == "key").then_some(ObjectId(2)),
            |t| (t == "pit").then_some(TrapId(0)),
        )
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
