//! Local areas of the overworld (`local:x,y`, spec §3): the outdoor level behind one overworld
//! cell, entered with `>` there and left by walking off any edge, which leads back to the cell.
//!
//! An area is generated from its own random stream, [`Rng::map`]`(seed, id, attempt)`, so it
//! depends only on the run seed and its overworld position, never on when it is first visited.
//! What it looks like follows the overworld around its cell ([`Setting`]):
//!
//! - the cell's **landscape** picks how the land is grown (`cover.rs`): plains, grassland,
//!   forest, dense forest, hills, or swamp; a road cell takes the land of most of its neighbors;
//! - a **river** or **lake** next to the cell runs or lies along that side (or corner) of the
//!   area (`water.rs`), the river flowing the way the overworld's does;
//! - on a road cell the **road** runs from near the middle out towards every neighboring road,
//!   bridge, or location (`road.rs`);
//! - the share of woods blends towards the neighbors' near each side, so areas side by side meet.
//!
//! Each attempt lays water and the road, keeps a glade clear around the middle (the arrival when
//! the player did not come from a side), adds ponds, a brook, and sometimes ruins (outdoor
//! vaults), grows the land, then joins every walkable pocket to the middle and opens every side
//! (`reach.rs`); deer may graze. An attempt that fails validation is discarded and the next
//! attempt's stream used. Integer-only; no hash-map iteration.

mod cover;
mod kit;
mod land;
mod noise;
mod reach;
mod road;
mod water;

pub(crate) use kit::Kit;
pub use reach::arrival;

use super::{GenError, MAX_ATTEMPTS, out_and_back};
use crate::content::{
    Content, CreatureSpawn, EdgeExit, Landscape, LevelKind, LevelTemplate, LocalArea, TerrainId,
};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;
use land::{Claim, Land};
use std::collections::BTreeMap;

/// Width and height of a local area.
pub const SIZE: (i32, i32) = (80, 40);

/// What the overworld says about a local area: its land, and what lies around its cell.
#[derive(Clone, Debug)]
pub(super) struct Setting {
    /// The land (never a road or water).
    pub landscape: Landscape,
    pub ground: TerrainId,
    /// Directions of the neighboring roads, bridges, and locations, if the cell is a road.
    pub roads: Vec<Dir>,
    /// Sides with a river beside them, and which way it flows on the overworld.
    pub rivers: Vec<(Dir, Dir)>,
    /// Sides with a lake beside them.
    pub lakes: Vec<Dir>,
    /// The land of the neighboring cells.
    pub neighbors: Vec<(Dir, Landscape)>,
}

impl Setting {
    fn read(content: &Content, area: &LocalArea<'_>) -> Self {
        let mut setting = Setting {
            landscape: area.landscape,
            ground: area.ground,
            roads: Vec::new(),
            rivers: Vec::new(),
            lakes: Vec::new(),
            neighbors: Vec::new(),
        };
        // Weight of each neighboring land, and its ground (first seen), for a road cell.
        let mut lands: BTreeMap<Landscape, (i32, Option<TerrainId>)> = BTreeMap::new();
        for dir in Dir::ALL {
            let Some(t) = area.neighbor(dir) else {
                continue;
            };
            let def = content.terrain(t);
            if def.stairs.is_some() {
                setting.roads.push(dir);
            }
            match def.landscape {
                Some(Landscape::Road) => setting.roads.push(dir),
                Some(Landscape::River) => {
                    let p = area.pos.step(dir);
                    let flow = area.overworld.flow.get(&p).map_or(Dir::S, |f| f.dir);
                    setting.rivers.push((dir, flow));
                }
                Some(Landscape::Lake) => setting.lakes.push(dir),
                Some(land) => {
                    setting.neighbors.push((dir, land));
                    let weight = if dir.delta().0 == 0 || dir.delta().1 == 0 {
                        2
                    } else {
                        1
                    };
                    let entry = lands.entry(land).or_insert((0, def.local));
                    entry.0 += weight;
                }
                None => {}
            }
        }
        // Water at a corner only counts where neither side next to it has water already (the
        // band along the side covers that corner).
        let wet: Vec<Dir> = (setting.rivers.iter().map(|(d, _)| *d))
            .chain(setting.lakes.iter().copied())
            .collect();
        let counts = |d: Dir| {
            let (dx, dy) = d.delta();
            let side = |x, y| Dir::from_delta(x, y).is_some_and(|s| wet.contains(&s));
            dx == 0 || dy == 0 || !(side(dx, 0) || side(0, dy))
        };
        setting.rivers.retain(|(d, _)| counts(*d));
        setting.lakes.retain(|d| counts(*d));
        if area.landscape == Landscape::Road {
            let most = lands
                .into_iter()
                .max_by_key(|(land, (weight, _))| (*weight, std::cmp::Reverse(*land)));
            let (land, ground) = most.map_or((Landscape::Plains, None), |(l, (_, g))| (l, g));
            setting.landscape = land;
            setting.ground = ground.unwrap_or(area.ground);
        } else {
            setting.roads.clear();
        }
        setting
    }
}

/// Chances (percent) and counts of the features of a landscape.
struct Features {
    pond: u32,
    brook: u32,
    ruins: u32,
    deer: (i32, i32),
    /// Half-width of the glade kept clear around the middle, in columns.
    glade: (i32, i32),
}

fn features(landscape: Landscape) -> Features {
    let f = |pond, brook, ruins, deer, glade| Features {
        pond,
        brook,
        ruins,
        deer,
        glade,
    };
    match landscape {
        Landscape::Grassland => f(45, 25, 15, (1, 2), (4, 6)),
        Landscape::Forest => f(25, 35, 15, (0, 2), (7, 10)),
        Landscape::DenseForest => f(15, 25, 10, (0, 1), (6, 8)),
        Landscape::Hills => f(15, 25, 30, (0, 0), (4, 6)),
        Landscape::Swamp => f(0, 0, 10, (0, 0), (4, 6)),
        Landscape::Plains | Landscape::Road | Landscape::River | Landscape::Lake => {
            f(25, 10, 15, (0, 1), (4, 6))
        }
    }
}

/// Generates the local area `id` (`local:x,y`) for the run with `seed`, its loot rolled with the
/// finder's `luck` (Treasure hunter, Hoard sense).
pub(super) fn generate(
    content: &Content,
    seed: u64,
    id: &str,
    luck: crate::magic::loot::Luck,
) -> Result<LevelTemplate, GenError> {
    let area = content
        .local_area(id)
        .ok_or_else(|| GenError::NotAGeneratedLevel(id.to_string()))?;
    let kit = Kit::resolve(content).map_err(GenError::Content)?;
    let setting = Setting::read(content, &area);
    for attempt in 0..MAX_ATTEMPTS {
        let mut rng = Rng::map(seed, id, attempt);
        let Some((land, start)) = build(content, kit, &setting, &mut rng) else {
            continue;
        };
        let mut template = LevelTemplate {
            id: id.to_string(),
            name: capitalize(&content.terrain(area.terrain).name),
            arrival: None,
            ambient: crate::light::Light::DARK,
            kind: LevelKind::Outdoor,
            player_start: Some(start),
            terrain: land.terrain,
            creatures: land.creatures,
            stairs: Vec::new(),
            lights: land.lights,
            flow: land.flow,
            objects: land.objects,
            traps: Default::default(),
            locks: land.locks,
            secrets: Default::default(),
            hidden: Default::default(),
            regions: Vec::new(),
            rooms: Vec::new(),
            shops: Vec::new(),
            exit: Some(EdgeExit {
                to: area.overworld.id.clone(),
                pos: area.pos,
            }),
            item_level: content.item_level(id),
            loot: Vec::new(),
            // Under the open sky: the weather's temperature, and ice as it brings.
            temperature: None,
            ice: Default::default(),
        };
        if reach::validate(&template, content).is_ok() {
            // Ruins may hold loot (spec §8 "Magic items"); the land holds none of its own.
            super::place_loot(&mut template, content, seed, &land.loot, (None, luck));
            return Ok(template);
        }
    }
    Err(GenError::Exhausted(MAX_ATTEMPTS))
}

/// One attempt: the land and its start (the middle, for arrivals not from a side).
fn build<'c>(
    content: &'c Content,
    kit: Kit,
    setting: &Setting,
    rng: &mut Rng,
) -> Option<(Land<'c>, Pos)> {
    let (width, height) = SIZE;
    let features = features(setting.landscape);
    let mut land = Land::new(content, kit, setting.ground, width, height);
    water::sides(&mut land, setting, rng);
    if setting.rivers.is_empty() && rng.chance(features.brook) {
        water::brook(&mut land, rng, &setting.roads);
    }
    road::lay(&mut land, rng, &setting.roads);
    glade(&mut land, rng, setting.landscape, features.glade);
    if rng.chance(features.pond) {
        water::pond(&mut land, rng, (4, 8));
    }
    if rng.chance(features.ruins) {
        ruins(&mut land, rng);
    }
    cover::grow(&mut land, setting, rng);
    let start = reach::start(&land)?;
    reach::join(&mut land, start)?;
    let deer = rng.range(features.deer.0, features.deer.1);
    graze(&mut land, rng, deer, start);
    Some((land, start))
}

/// Keeps a glade clear around the middle; in the woods it is grassy, in a swamp a dry hummock.
fn glade(land: &mut Land, rng: &mut Rng, landscape: Landscape, size: (i32, i32)) {
    let rx = rng.range(size.0, size.1);
    let grassy = matches!(
        landscape,
        Landscape::Forest | Landscape::DenseForest | Landscape::Swamp
    );
    for p in land.blob(rng, land.center(), rx, rx / 2, 350) {
        land.claim_at_least(p, Claim::Clear);
        if grassy && land.may_cover(p) && land.get(p) == Some(land.ground) {
            land.set(p, land.kit.grass);
        }
    }
}

/// Stamps one outdoor vault (ruins) on open land away from the middle, with a ring of open
/// ground around it, if one fits.
fn ruins(land: &mut Land, rng: &mut Rng) {
    let content = land.content;
    let mut vaults: Vec<_> = content.vaults().iter().filter(|v| v.is_outdoor()).collect();
    rng.shuffle(&mut vaults);
    let (w, h) = (land.width(), land.height());
    let c = land.center();
    for vault in vaults {
        let (vw, vh) = (vault.width(), vault.height());
        for _ in 0..30 {
            let at = Pos::new(rng.range(3, w - vw - 3), rng.range(2, h - vh - 2));
            let (dx, dy) = (at.x + vw / 2 - c.x, 2 * (at.y + vh / 2 - c.y));
            if dx * dx + dy * dy < 24 * 24 {
                continue;
            }
            let fits = (at.y - 1..=at.y + vh).all(|y| {
                (at.x - 1..=at.x + vw).all(|x| {
                    let p = Pos::new(x, y);
                    land.claim(p) == Claim::Free && !land.is_water(p)
                })
            });
            if !fits {
                continue;
            }
            let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
            for y in at.y - 1..=at.y + vh {
                for x in at.x - 1..=at.x + vw {
                    land.set_claim(Pos::new(x, y), Claim::Clear);
                }
            }
            let template = &vault.template;
            for (p, t) in template.terrain.iter() {
                if *t != land.kit.void {
                    land.set(shift(p), *t);
                    land.set_claim(shift(p), Claim::Fixed);
                }
            }
            for (p, light) in &template.lights {
                land.lights.insert(shift(*p), *light);
            }
            for (p, flow) in &template.flow {
                land.flow.insert(shift(*p), *flow);
            }
            for (p, pile) in &template.objects {
                land.objects.insert(shift(*p), pile.clone());
            }
            for &(p, grade) in &template.loot {
                land.loot.push((shift(p), grade));
            }
            for (p, lock) in &template.locks {
                land.locks.insert(shift(*p), lock.clone());
            }
            for spawn in &template.creatures {
                land.creatures.push(CreatureSpawn {
                    pos: shift(spawn.pos),
                    ..spawn.clone()
                });
            }
            return;
        }
    }
}

/// Deer grazing to and fro on open ground, away from the middle.
fn graze(land: &mut Land, rng: &mut Rng, count: i32, start: Pos) {
    let (w, h) = (land.width(), land.height());
    for _ in 0..count {
        for _ in 0..30 {
            let p = Pos::new(rng.range(2, w - 3), rng.range(2, h - 3));
            let (dx, dy) = (p.x - start.x, 2 * (p.y - start.y));
            let taken = |q: Pos| land.creatures.iter().any(|c| c.pos == q);
            let grazing = |q: Pos| land.is_dry_footing(q) && !land.on_edge(q);
            if dx * dx + dy * dy < 20 * 20 || !grazing(p) || taken(p) {
                continue;
            }
            let len = rng.range(4, 8);
            let Some(route) = out_and_back(rng, p, len, grazing) else {
                continue;
            };
            land.creatures.push(CreatureSpawn {
                creature: land.kit.deer,
                faction: None,
                alignment: None,
                npc: None,
                pos: p,
                route,
                carry: Vec::new(),
                equipment: Default::default(),
            });
            break;
        }
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests;
