//! Burial grounds as level data (spec §7 "Graves"; #1682; `docs/dev/graves.md`): a hand-made
//! level's `[burial]` table puts it in a chain (`chain`, its first level's id; `depth`, 0 the
//! churchyard, 1 the first crypt below it, ...), and its legend marks what the rules fill:
//!
//! - its **places**: every cell of a terrain with `burial` (an empty plot, an empty niche, an open
//!   sarcophagus), in row order, which becomes that terrain (a gravestone, a sealed niche, a
//!   lidded sarcophagus) when one of the run's dead is laid there (`world/graves.rs`);
//! - its fixed graves: a `grave` terrain cell with an `epitaph` (one of the town's own dead);
//! - its **way down**: one cell with `burial_stairs = "stairs_down"`, its own terrain until the
//!   level is full, then those stairs to the next level of the chain;
//! - its **statue places**: cells with `statue_place = true`, kept free for statues (none placed
//!   yet).
//!
//! [`link_chains`] orders each chain by depth once every level is read and points each way down
//! at the next level; the deepest's leads nowhere and never opens.

use super::level::{LevelTemplate, StairsLink};
use super::{Content, ContentError, StairsDir, TerrainDef, TerrainId};
use crate::geom::Pos;
use crate::grid::Grid;
use serde::Deserialize;
use std::collections::BTreeMap;

/// What a place says of one of the run's dead when the level gives no `epitaph`.
pub const DEFAULT_EPITAPH: &str = "Here lies {name}, {death}.";

/// A level file's `[burial]` table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BurialSpec {
    /// The chain it belongs to: the id of the chain's first level (the churchyard).
    chain: String,
    /// Its place in the chain: 0 the churchyard, 1 the first crypt, and so on.
    depth: u8,
    /// The words a place is cut with for one of the dead: `{name}` ("Brakka the dwarf
    /// warrior") and `{death}` ("killed by a kobold in Fyrdelf 3, The Ascmearc"); default
    /// [`DEFAULT_EPITAPH`].
    epitaph: Option<String>,
}

/// What a level's legend marks for its burial ground, gathered cell by cell.
#[derive(Debug, Default)]
pub(super) struct Marks {
    /// The fixed graves' words, by cell.
    pub epitaphs: BTreeMap<Pos, String>,
    /// The cells kept for statues.
    pub statue_places: Vec<Pos>,
    /// The way down: its cell and the stairs it becomes.
    pub stairs: Vec<(Pos, TerrainId)>,
}

/// A level's burial ground (module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Burial {
    /// The chain: the id of its first level.
    pub chain: String,
    /// Its place in the chain, 0 the churchyard.
    pub depth: u8,
    /// The words for one of the dead (`{name}`, `{death}`).
    pub epitaph: String,
    /// The places for the dead, in row order: the order they are filled in.
    pub places: Vec<Pos>,
    /// The fixed graves' words, by cell (the town's own dead): no place for the run's.
    pub epitaphs: BTreeMap<Pos, String>,
    /// The cells kept for statues, in row order; nothing is placed there yet.
    pub statue_places: Vec<Pos>,
    /// The way down, which opens once every place is filled.
    pub stairs: Option<BurialStairs>,
}

/// A burial ground's way down: closed (its cell's own terrain) until the level is full.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurialStairs {
    pub pos: Pos,
    /// The stairs it becomes.
    pub terrain: TerrainId,
    /// The next level of the chain; `None` on the deepest, whose way down never opens.
    pub to: Option<String>,
}

impl Burial {
    /// The stairs link of the opened way down, if it leads anywhere.
    pub fn opened(&self) -> Option<StairsLink> {
        let stairs = self.stairs.as_ref()?;
        Some(StairsLink {
            pos: stairs.pos,
            dir: StairsDir::Down,
            to: Some(stairs.to.clone()?),
            name: None,
        })
    }
}

/// The level's burial ground from its `[burial]` table and its legend's `marks`, checked.
pub(super) fn resolve(
    spec: Option<BurialSpec>,
    marks: Marks,
    terrain: &Grid<TerrainId>,
    content: &Content,
) -> Result<Option<Burial>, String> {
    let def = |p: Pos| content.terrain(terrain[p]);
    let mut places = Vec::new();
    for y in 0..terrain.height() {
        for x in 0..terrain.width() {
            let p = Pos::new(x, y);
            if def(p).burial.is_some() {
                places.push(p);
            } else if def(p).grave && !marks.epitaphs.contains_key(&p) {
                return Err(format!(
                    "the grave at {p} has no `epitaph`: a stone for the run's dead is a place \
                     (a terrain with `burial`)"
                ));
            }
        }
    }
    let Some(spec) = spec else {
        let marked = !marks.epitaphs.is_empty()
            || !marks.statue_places.is_empty()
            || !marks.stairs.is_empty()
            || !places.is_empty();
        if marked {
            return Err(
                "places, `epitaph`, `statue_place`, and `burial_stairs` need a \
                        `[burial]` table"
                    .into(),
            );
        }
        return Ok(None);
    };
    if places.is_empty() {
        return Err("`[burial]`: no place for the dead (a terrain with `burial`)".into());
    }
    let epitaph = spec.epitaph.unwrap_or_else(|| DEFAULT_EPITAPH.to_string());
    if !epitaph.contains("{name}") {
        return Err("`[burial]`: the `epitaph` names no one (`{name}`)".into());
    }
    if let Some(p) = (marks.epitaphs.keys()).find(|p| !def(**p).grave) {
        return Err(format!(
            "`epitaph` at {p} on terrain `{}`, which is no grave",
            def(*p).id
        ));
    }
    if let Some(p) = (marks.statue_places.iter()).find(|p| !plain(def(**p))) {
        return Err(format!(
            "`statue_place` at {p} on terrain `{}`: only on ground one walks on",
            def(*p).id
        ));
    }
    let stairs = match marks.stairs[..] {
        [] => None,
        [(pos, to)] => {
            let (closed, open) = (def(pos), content.terrain(to));
            if open.stairs != Some(StairsDir::Down) {
                return Err(format!(
                    "`burial_stairs` at {pos}: `{}` is no way down",
                    open.id
                ));
            }
            if !plain(closed) || closed.transparent != open.transparent {
                return Err(format!(
                    "`burial_stairs` at {pos} on terrain `{}`: only on ground one walks on and \
                     sees past",
                    closed.id
                ));
            }
            Some(BurialStairs {
                pos,
                terrain: to,
                to: None,
            })
        }
        [_, (pos, _), ..] => return Err(format!("a second `burial_stairs` at {pos}")),
    };
    Ok(Some(Burial {
        chain: spec.chain,
        depth: spec.depth,
        epitaph,
        places,
        epitaphs: marks.epitaphs,
        statue_places: marks.statue_places,
        stairs,
    }))
}

/// Ground one walks on that is nothing else: no stairs, door, chest, or light.
fn plain(def: &TerrainDef) -> bool {
    def.is_plain_ground() && def.burial.is_none()
}

/// The terrain table's burial places (`burial`): each becomes a grave (`grave`) that is in
/// every rule the same as the place, so the run's dead change nothing but what is drawn and
/// read (the state hash leaves them out, `world/graves.rs`).
pub(super) fn check_terrain(defs: &[TerrainDef]) -> Result<(), String> {
    for place in defs {
        let Some(to) = place.burial else { continue };
        let grave = &defs[usize::from(to.0)];
        let err = |why: &str| {
            Err(format!(
                "terrain `{}`: `burial` `{}` {why}",
                place.id, grave.id
            ))
        };
        if !grave.grave || place.grave || grave.burial.is_some() {
            return err("must be a grave (`grave`), and the place none");
        }
        let same = place.walkable == grave.walkable
            && place.transparent == grave.transparent
            && place.height == grave.height
            && place.roofed == grave.roofed
            && place.stone == grave.stone
            && place.bounce == grave.bounce
            && place.muffle == grave.muffle
            && place.step_noise == grave.step_noise
            && place.move_cost == grave.move_cost
            && place.plants == grave.plants
            && place.swim == grave.swim;
        let inert = |d: &TerrainDef| {
            d.light.is_none()
                && d.stairs.is_none()
                && d.breaks.is_none()
                && d.kicked_in.is_none()
                && d.chest.is_none()
                && d.statue.is_none()
                && d.opens_to.is_none()
                && d.closes_to.is_none()
                && d.freeze.is_none()
                && d.works_as.is_none()
        };
        if !same || !inert(place) || !inert(grave) {
            return err(
                "must be the same in every rule (walkable, transparent, height, roofed, stone, \
                 bounce, muffle, noise, cost), and neither a light, stairs, door, chest, \
                 statue, station, or breakable",
            );
        }
    }
    Ok(())
}

/// Orders every burial chain by depth and points each level's way down at the next (module
/// docs): the chains, each its levels' ids from the churchyard down.
pub(super) fn link_chains(
    levels: &mut BTreeMap<String, LevelTemplate>,
    level_path: impl Fn(&str) -> String,
) -> Result<BTreeMap<String, Vec<String>>, ContentError> {
    let mut chains: BTreeMap<String, BTreeMap<u8, String>> = BTreeMap::new();
    for level in levels.values() {
        let Some(burial) = &level.burial else {
            continue;
        };
        let err = |m: String| ContentError::new(&level_path(&level.id), m);
        let chain = chains.entry(burial.chain.clone()).or_default();
        if let Some(other) = chain.insert(burial.depth, level.id.clone()) {
            return Err(err(format!(
                "`[burial]`: `{}` is at depth {} of `{}` too",
                other, burial.depth, burial.chain
            )));
        }
    }
    let mut ordered = BTreeMap::new();
    for (chain, by_depth) in chains {
        let ids: Vec<String> = by_depth.values().cloned().collect();
        for (i, (&depth, id)) in by_depth.iter().enumerate() {
            let err = |m: String| ContentError::new(&level_path(id), m);
            if usize::from(depth) != i {
                return Err(err(format!(
                    "`[burial]`: depth {depth} of `{chain}`, but depth {i} has no level"
                )));
            }
            if depth == 0 && *id != chain {
                return Err(err(format!(
                    "`[burial]`: the chain is named after its depth-0 level, `{id}`, not \
                     `{chain}`"
                )));
            }
            let level = levels.get_mut(id).expect("listed");
            let up = (i > 0).then(|| ids[i - 1].as_str());
            if let Some(up) = up
                && !(level.stairs.iter())
                    .any(|s| s.dir == StairsDir::Up && s.to.as_deref() == Some(up))
            {
                return Err(err(format!(
                    "`[burial]`: no stairs up (`to = \"{up}\"`) to the level above"
                )));
            }
            let next = ids.get(i + 1).cloned();
            let burial = level.burial.as_mut().expect("in a chain");
            match (&mut burial.stairs, next) {
                (Some(stairs), next) => stairs.to = next,
                (None, Some(_)) => {
                    return Err(err(
                        "`[burial]`: no way down (`burial_stairs`); only the chain's deepest \
                         level may lack one"
                            .into(),
                    ));
                }
                (None, None) => {}
            }
        }
        ordered.insert(chain, ids);
    }
    Ok(ordered)
}

impl Content {
    /// Burial chain `chain`'s levels, the churchyard first (empty for none).
    pub fn burial_chain(&self, chain: &str) -> &[String] {
        self.burial_chains.get(chain).map_or(&[], Vec::as_slice)
    }

    /// How many of the run's dead lie on the levels of level `id`'s chain above it (the places
    /// filled before its own), and its own burial ground; `None` for a level with none.
    pub fn burial_of(&self, id: &str) -> Option<(usize, &Burial)> {
        let burial = self.level(id)?.burial.as_ref()?;
        let above = (self.burial_chain(&burial.chain).iter())
            .take(usize::from(burial.depth))
            .filter_map(|l| self.level(l)?.burial.as_ref())
            .map(|b| b.places.len())
            .sum();
        Some((above, burial))
    }
}
