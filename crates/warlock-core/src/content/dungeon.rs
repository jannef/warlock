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
//! ambient = [0, 0, 0, 0]        # optional
//! temperature = 9               # °C; default the dungeon default (10)
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
use crate::light::Light;
use crate::temperature::{LEVEL_TEMPERATURES, Temperature};
use serde::Deserialize;

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
    /// The temperature on its levels, in °C; default the dungeon default.
    temperature: Option<i8>,
    #[serde(default)]
    levels: Vec<LevelsSpec>,
}

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
    /// One plan per depth, index `depth - 1`.
    plans: Vec<LevelPlan>,
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

    /// The display name of level `depth`, e.g. `The Depths 3`.
    pub fn level_name(&self, depth: u32) -> String {
        format!("{} {depth}", self.name)
    }
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
    /// Validates every branch; `creature_id` resolves creature names.
    pub(super) fn resolve(
        self,
        file: &str,
        creature_id: impl Fn(&str) -> Option<CreatureId>,
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
            defs.push(DungeonDef {
                name: spec.name.unwrap_or_else(|| id.clone()),
                id,
                depth: spec.depth,
                ambient: spec.ambient,
                creatures,
                plans,
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
        file.resolve("d", |c| (c == "rat").then_some(CreatureId(1)))
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
