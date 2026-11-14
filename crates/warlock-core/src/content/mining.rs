//! Mining (`assets/data/mining.toml`, spec §8 "Digging" and "Skills": Mining; #1389): the chance
//! that a wall a miner digs through gives anything, the check that breaks it out, and the table
//! of what it can be, by rarity and the floor's danger.
//!
//! Veins (#1390): an ore row with a `vein` has veins of its ore, wall sections drawn in its
//! colour that always give it; the loader makes one wall terrain per vein (`vein_<metal>`, a copy
//! of the `[veins]` table's `wall`), and `mapgen/vein.rs` lays them.
//!
//! The rule itself lives in `world/crafting/mine.rs`. The file is optional: without it nothing is mined.

use super::{ContentError, MaterialId, ObjectId, SkillId, TerrainId};
use crate::color::Rgb;
use crate::stats::Stat;
use serde::Deserialize;

/// The rules of mining that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiningRules {
    /// The skill a miner must be trained in, whose `craft` benefit helps the check (Mining).
    pub skill: SkillId,
    /// The stat whose modifier the check adds.
    pub stat: Stat,
    /// The generic ore object, named and made by its metal ("a lump of iron ore").
    pub ore: ObjectId,
    /// The hardness bands, softest first.
    pub bands: Vec<MiningBand>,
    /// One more lump (or block) for every this much of the check's margin.
    pub more_every: i32,
    /// The most lumps one wall gives.
    pub most: u32,
    /// The table, in file order.
    pub finds: Vec<MiningFind>,
    /// How often veins run through a branch's floor, and how many; `None`: never.
    pub veins: Option<Veins>,
}

/// How often a branch's floor has veins (`mapgen/vein.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Veins {
    /// Percent chance a floor has any.
    pub chance: u8,
    /// How many a floor with veins has, fewest and most.
    pub count: [u8; 2],
    /// The wall terrain veins replace and are copies of (its hardness, glyph, and look).
    pub wall: TerrainId,
}

/// A vein of an ore row's metal: the wall terrain of it, and how many cells one has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vein {
    pub terrain: TerrainId,
    /// Fewest and most cells.
    pub size: [u16; 2],
}

/// One hardness band: walls at least this hard (and softer than the next band) give a check
/// with this chance, against this difficulty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiningBand {
    pub hardness: u16,
    /// Percent chance a check is made at all.
    pub chance: u8,
    /// The check's difficulty.
    pub difficulty: i32,
}

/// What a row of the table gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mined {
    /// Lumps of the ore object, of this metal.
    Ore(MaterialId),
    /// Pieces of this material (blocks of stone).
    Piece(MaterialId),
    /// One uncut loose gem, as `gems.toml` makes them at the floor's item level.
    Gem,
}

/// One row of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiningFind {
    pub what: Mined,
    /// Its weight at its first danger.
    pub rarity: u32,
    /// Added to the weight for each danger past its first (it may be negative; the weight
    /// never goes below 0).
    pub deeper: i32,
    /// The first danger it is found at: none on a floor less dangerous.
    pub danger: u32,
    /// Its veins (an ore row's only); `None`: it has none.
    pub vein: Option<Vein>,
}

impl MiningFind {
    /// Its weight on a floor of danger `danger`: 0 below its own `danger`, else `rarity` plus
    /// `deeper` for each danger past it, never below 0.
    pub fn weight_at(&self, danger: u32) -> u32 {
        let Some(past) = danger.checked_sub(self.danger) else {
            return 0;
        };
        let weight = i64::from(self.rarity) + i64::from(self.deeper) * i64::from(past);
        u32::try_from(weight.max(0)).unwrap_or(u32::MAX)
    }
}

impl MiningRules {
    /// The band of a wall of `hardness`: the hardest band not harder than it; `None` for a wall
    /// softer than every band.
    pub fn band(&self, hardness: u16) -> Option<&MiningBand> {
        self.bands.iter().rev().find(|b| b.hardness <= hardness)
    }

    /// The table on a floor of danger `danger` whose ores are limited to `ores` (a branch's
    /// allow-list; `None`: every ore): each row's index with its weight, those above 0, in file
    /// order.
    pub fn at(&self, danger: u32, ores: Option<&[MaterialId]>) -> Vec<(usize, u32)> {
        (self.finds.iter().enumerate())
            .filter(|(_, f)| match (f.what, ores) {
                (Mined::Ore(m), Some(list)) => list.contains(&m),
                _ => true,
            })
            .map(|(i, f)| (i, f.weight_at(danger)))
            .filter(|&(_, w)| w > 0)
            .collect()
    }

    /// The vein table on a floor of danger `danger` whose ores are limited to `ores`: [`Self::at`]
    /// with only the rows that have veins.
    pub fn vein_table(&self, danger: u32, ores: Option<&[MaterialId]>) -> Vec<(usize, u32)> {
        let mut table = self.at(danger, ores);
        table.retain(|&(i, _)| self.finds[i].vein.is_some());
        table
    }

    /// The metal whose vein `terrain` is, if it is one.
    pub fn vein_of(&self, terrain: TerrainId) -> Option<MaterialId> {
        self.finds.iter().find_map(|f| match (f.what, f.vein) {
            (Mined::Ore(m), Some(v)) if v.terrain == terrain => Some(m),
            _ => None,
        })
    }

    /// The metals the table has an ore row for, in file order.
    pub fn ores(&self) -> impl Iterator<Item = MaterialId> + '_ {
        self.finds.iter().filter_map(|f| match f.what {
            Mined::Ore(m) => Some(m),
            _ => None,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MiningFile {
    skill: String,
    stat: String,
    ore: String,
    more_every: i32,
    most: u32,
    band: Vec<BandSpec>,
    find: Vec<FindSpec>,
    veins: Option<VeinsSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VeinsSpec {
    chance: u8,
    count: [u8; 2],
    wall: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VeinSpec {
    fg: Rgb,
    size: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BandSpec {
    hardness: u16,
    chance: u8,
    difficulty: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FindSpec {
    ore: Option<String>,
    piece: Option<String>,
    #[serde(default)]
    gem: bool,
    rarity: u32,
    #[serde(default)]
    deeper: i32,
    #[serde(default)]
    danger: u32,
    vein: Option<VeinSpec>,
}

impl MiningRules {
    /// Parses `data/mining.toml` (checked: a known skill and stat; an ore object that stacks;
    /// bands in rising hardness with a chance of at most 100; `more_every` above 0 and `most` at
    /// least 1; rows naming exactly one of `ore`, `piece`, `gem`, an ore a material the ore
    /// object can be made of, a piece a material pieces are made of, no row twice, a rarity above
    /// 0), then checks every branch's `ores` against it: each a metal with an ore row. Veins:
    /// a `[veins]` table with a chance of at most 100, a count and sizes rising from at least 1,
    /// and a `wall` that breaks; a `vein` only on an ore row, and only with the table. Each vein
    /// becomes a wall terrain of its own, added to `content`.
    pub(super) fn parse(
        file: &str,
        text: &str,
        content: &mut super::Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: MiningFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let skill = (content.skill_id(&parsed.skill))
            .ok_or_else(|| err(format!("unknown skill `{}`", parsed.skill)))?;
        let stat = Stat::from_id(&parsed.stat)
            .ok_or_else(|| err(format!("unknown stat `{}`", parsed.stat)))?;
        let ore = (content.object_id(&parsed.ore))
            .ok_or_else(|| err(format!("unknown object `{}`", parsed.ore)))?;
        let ore_def = content.object(ore);
        if !ore_def.stackable {
            return Err(err(format!("ore `{}` must be stackable", parsed.ore)));
        }
        if parsed.more_every <= 0 {
            return Err(err("`more_every` must be above 0".into()));
        }
        if parsed.most == 0 {
            return Err(err("`most` must be 1 or more".into()));
        }
        let mut bands: Vec<MiningBand> = Vec::new();
        for b in parsed.band {
            if b.chance > 100 {
                return Err(err(format!("band {}: `chance` above 100", b.hardness)));
            }
            if bands.last().is_some_and(|l| l.hardness >= b.hardness) {
                return Err(err("bands must rise in `hardness`".into()));
            }
            bands.push(MiningBand {
                hardness: b.hardness,
                chance: b.chance,
                difficulty: b.difficulty,
            });
        }
        if bands.is_empty() {
            return Err(err("there must be at least one `[[band]]`".into()));
        }
        let material = |name: &str| {
            (content.material_id(name)).ok_or_else(|| err(format!("unknown material `{name}`")))
        };
        let veins = match &parsed.veins {
            None => None,
            Some(v) => {
                let wall = (content.terrain_id(&v.wall))
                    .ok_or_else(|| err(format!("unknown terrain `{}`", v.wall)))?;
                let def = content.terrain(wall);
                if def.breaks.is_none() || def.walkable {
                    return Err(err(format!("veins: `{}` is no wall that breaks", v.wall)));
                }
                if v.chance > 100 {
                    return Err(err("veins: `chance` above 100".into()));
                }
                if v.count[0] == 0 || v.count[0] > v.count[1] {
                    return Err(err("veins: `count` must rise from 1 or more".into()));
                }
                Some(Veins {
                    chance: v.chance,
                    count: v.count,
                    wall,
                })
            }
        };
        let mut finds: Vec<MiningFind> = Vec::new();
        let mut vein_looks: Vec<(usize, MaterialId, Rgb)> = Vec::new();
        for spec in parsed.find {
            let what = match (&spec.ore, &spec.piece, spec.gem) {
                (Some(name), None, false) => {
                    let m = material(name)?;
                    if !ore_def.made_of(0).contains(&m) && ore_def.material() != m {
                        return Err(err(format!("there is no ore of `{name}`")));
                    }
                    Mined::Ore(m)
                }
                (None, Some(name), false) => {
                    let m = material(name)?;
                    if crate::piece::make(content, m, 1).is_none() {
                        return Err(err(format!("there are no pieces of `{name}`")));
                    }
                    Mined::Piece(m)
                }
                (None, None, true) => Mined::Gem,
                _ => {
                    return Err(err(
                        "a `[[find]]` names exactly one of `ore`, `piece`, `gem`".into(),
                    ));
                }
            };
            if finds.iter().any(|f| f.what == what) {
                return Err(err(format!("a `[[find]]` is listed twice: {what:?}")));
            }
            if spec.rarity == 0 {
                return Err(err(format!("{what:?}: `rarity` must be above 0")));
            }
            let vein = match (&spec.vein, what) {
                (None, _) => None,
                (Some(_), _) if veins.is_none() => {
                    return Err(err(format!("{what:?}: a `vein` needs a `[veins]` table")));
                }
                (Some(v), Mined::Ore(metal)) => {
                    if v.size[0] == 0 || v.size[0] > v.size[1] {
                        return Err(err(format!(
                            "{what:?}: vein `size` must rise from 1 or more"
                        )));
                    }
                    vein_looks.push((finds.len(), metal, v.fg));
                    Some(Vein {
                        // Made below, once the table is read.
                        terrain: TerrainId(0),
                        size: v.size,
                    })
                }
                (Some(_), _) => return Err(err(format!("{what:?}: only an ore has veins"))),
            };
            finds.push(MiningFind {
                what,
                rarity: spec.rarity,
                deeper: spec.deeper,
                danger: spec.danger,
                vein,
            });
        }
        if finds.is_empty() {
            return Err(err("there must be at least one `[[find]]`".into()));
        }
        if let Some(v) = veins {
            for (row, metal, fg) in vein_looks {
                let terrain = add_vein_terrain(content, v.wall, metal, fg)
                    .map_err(|m| err(format!("veins: {m}")))?;
                if let Some(vein) = finds[row].vein.as_mut() {
                    vein.terrain = terrain;
                }
            }
        }
        let rules = MiningRules {
            skill,
            stat,
            ore,
            bands,
            more_every: parsed.more_every,
            most: parsed.most,
            finds,
            veins,
        };
        for def in content.dungeons() {
            for name in def.ores.iter().flatten() {
                let known =
                    (content.material_id(name)).is_some_and(|m| rules.ores().any(|o| o == m));
                if !known {
                    return Err(err(format!(
                        "dungeon `{}`: `ores` names `{name}`, which has no ore row",
                        def.id
                    )));
                }
            }
        }
        Ok(rules)
    }
}

/// Adds the wall terrain of a vein of `metal`: a copy of `wall` with the id `vein_<metal>`, named
/// "vein of <metal> ore", drawn in `fg` on the wall's own ground.
fn add_vein_terrain(
    content: &mut super::Content,
    wall: TerrainId,
    metal: MaterialId,
    fg: Rgb,
) -> Result<TerrainId, String> {
    let metal_def = content.material(metal);
    let (id, name) = (format!("vein_{}", metal_def.id), metal_def.name.clone());
    let terrain = (u16::try_from(content.terrain.len()))
        .map(TerrainId)
        .map_err(|_| "too many terrain types".to_string())?;
    if content.terrain_ids.insert(id.clone(), terrain).is_some() {
        return Err(format!("terrain `{id}` already exists"));
    }
    let def = super::TerrainDef {
        id,
        name: format!("vein of {name} ore"),
        description: Some(format!("A seam of {name} ore runs through the rock.")),
        fg,
        ..content.terrain(wall).clone()
    };
    content.terrain.push(def);
    Ok(terrain)
}
