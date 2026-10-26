//! The overworld and its local areas (spec §3, §7).
//!
//! A campaign's overworld is its one level of kind `overworld` (spec §3): a map of the land at large in the usual
//! level format, with overworld terrain (each with `travel_minutes`) and locations, which are
//! stairs down to other levels. Every other overworld cell whose terrain names a `local` ground
//! leads into a **local area**, an outdoor level generated on the first visit
//! ([`crate::mapgen`]) whose id is `local:x,y` after its overworld cell. Like a dungeon branch
//! level, it then persists with the run. What the area looks like follows the [`Landscape`] of
//! its cell and of the cells around it (a river or a lake beside it, a road through it).

use super::{Content, EdgeExit, HiddenLocation, LevelKind, LevelTemplate, StairsDir, TerrainId};
use crate::geom::{Dir, Pos};
use serde::Deserialize;
use std::collections::BTreeMap;

const LOCAL_PREFIX: &str = "local:";

/// The level id of the local area of overworld cell `p`, e.g. `local:12,30`.
pub fn local_area_id(p: Pos) -> String {
    format!("{LOCAL_PREFIX}{},{}", p.x, p.y)
}

/// The overworld cell of a local area id such as `local:12,30`. Only the canonical spelling is
/// accepted (no signs, spaces, or leading zeros), so one cell has exactly one id.
pub fn parse_local_area(id: &str) -> Option<Pos> {
    let (x, y) = id.strip_prefix(LOCAL_PREFIX)?.split_once(',')?;
    let coord = |s: &str| -> Option<i32> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        s.parse().ok()
    };
    let p = Pos::new(coord(x)?, coord(y)?);
    (local_area_id(p) == id).then_some(p)
}

/// What the land of an overworld terrain is like at the scale of a local area (`landscape` in
/// `data/terrain.toml`): which generator builds the local areas of its cells, and what it adds
/// to the local areas beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Landscape {
    /// Open dry grass, a lone tree here and there.
    Plains,
    /// Green meadows with flowers and copses.
    Grassland,
    /// Woods with clearings.
    Forest,
    /// Thick woods crossed by game trails.
    DenseForest,
    /// Rocky slopes, crags, and boulders.
    Hills,
    /// Marsh, pools, and reeds.
    Swamp,
    /// A road: it runs through the local area towards the neighboring roads and locations; the
    /// land around it is like the neighbors'.
    Road,
    /// Running water: a river along that side of the neighboring local areas.
    River,
    /// Open water: a lake shore along that side of the neighboring local areas.
    Lake,
}

impl Landscape {
    /// Land a local area can be made of (not a road, not water).
    pub const fn is_land(self) -> bool {
        !matches!(self, Landscape::Road | Landscape::River | Landscape::Lake)
    }
}

/// A local area: the overworld it belongs to, its cell, and the terrain there.
#[derive(Clone, Copy, Debug)]
pub struct LocalArea<'a> {
    pub overworld: &'a LevelTemplate,
    pub pos: Pos,
    pub terrain: TerrainId,
    /// The ground the local area is made of (the overworld terrain's `local`).
    pub ground: TerrainId,
    /// The landscape of the overworld terrain: land or a road (content validation guarantees
    /// one).
    pub landscape: Landscape,
}

impl LocalArea<'_> {
    /// The overworld terrain next to the area's cell in direction `dir`, if on the map.
    pub fn neighbor(&self, dir: Dir) -> Option<TerrainId> {
        self.overworld.terrain.get(self.pos.step(dir)).copied()
    }
}

/// Gives every hand-made outdoor level that an overworld location leads into (its stairs down,
/// or a hidden location's `to`) that location as its edge exit (spec §7 "Leaving an outdoor
/// area"): walking off its edge, or the quick leave, returns to the location's cell, as from a
/// local area. The first location in the overworld's order wins; a level with stairs of its own
/// keeps them as well.
pub(super) fn link_edge_exits(levels: &mut BTreeMap<String, LevelTemplate>) {
    let mut exits: Vec<(String, EdgeExit)> = Vec::new();
    for overworld in (levels.values()).filter(|l| l.kind == LevelKind::Overworld) {
        let stairs = (overworld.stairs.iter())
            .filter(|s| s.dir == StairsDir::Down)
            .filter_map(|s| Some((s.to.clone()?, s.pos)));
        let hidden = (overworld.hidden.iter()).filter_map(|(p, h)| Some((h.to.clone()?, *p)));
        for (to, pos) in stairs.chain(hidden) {
            exits.push((
                to,
                EdgeExit {
                    to: overworld.id.clone(),
                    pos,
                },
            ));
        }
    }
    for (to, exit) in exits {
        if let Some(level) = levels.get_mut(&to)
            && level.kind == LevelKind::Outdoor
            && level.exit.is_none()
        {
            level.exit = Some(exit);
        }
    }
}

impl Content {
    /// The campaign `campaign`'s hidden location `id` and its cell (spec §7 "Hidden locations"),
    /// if its overworld ([`Content::overworld_of`]) has one by that id.
    pub fn hidden_location(&self, campaign: &str, id: &str) -> Option<(Pos, &HiddenLocation)> {
        (self.overworld_of(campaign)?.hidden.iter())
            .find(|(_, h)| h.id == id)
            .map(|(p, h)| (*p, h))
    }

    /// The local area with level id `id` (`local:x,y`) of campaign `campaign`'s overworld
    /// ([`Content::overworld_of`]), if it has an explorable cell there: in bounds, walkable,
    /// with a `local` ground, and not a location.
    pub fn local_area(&self, campaign: &str, id: &str) -> Option<LocalArea<'_>> {
        let pos = parse_local_area(id)?;
        let overworld = self.overworld_of(campaign)?;
        let terrain = *overworld.terrain.get(pos)?;
        let def = self.terrain(terrain);
        let ground = def.local.filter(|_| def.walkable && def.stairs.is_none())?;
        Some(LocalArea {
            overworld,
            pos,
            terrain,
            ground,
            landscape: def.landscape?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_area_ids_round_trip_and_are_canonical() {
        let p = Pos::new(12, 30);
        assert_eq!(local_area_id(p), "local:12,30");
        assert_eq!(parse_local_area("local:12,30"), Some(p));
        assert_eq!(parse_local_area("local:0,0"), Some(Pos::new(0, 0)));
        for bad in [
            "local:012,30",
            "local:-1,3",
            "local:+1,3",
            "local:1, 3",
            "local:1,",
            "local:1",
            "local:1,2,3",
            "depths:1",
            "local:99999999999,1",
        ] {
            assert_eq!(parse_local_area(bad), None, "{bad}");
        }
    }

    fn load_with(edit: impl FnOnce(&mut super::super::ContentFiles)) -> Result<Content, String> {
        let mut files = super::super::ContentFiles::builtin();
        edit(&mut files);
        Content::load(&files).map_err(|e| e.to_string())
    }

    #[test]
    fn overworlds_are_validated() {
        let overworld = |extra: &str, map: &str| {
            format!(
                "name = \"W\"\nkind = \"overworld\"\n{extra}map = '''\n{map}'''\n[legend]\n\".\" = \"plains\"\n\"#\" = \"floor\"\n"
            )
        };
        let err = load_with(|f| {
            f.insert(
                "campaigns/showcase/levels/overworld.toml",
                &overworld("", "..#\n"),
            )
        });
        assert!(
            err.as_ref()
                .is_err_and(|e| e.contains("`floor` is walkable but has no travel_minutes")),
            "{err:?}"
        );
        let err = load_with(|f| {
            f.insert(
                "campaigns/showcase/levels/second.toml",
                &overworld("", "...\n"),
            )
        });
        assert!(
            err.as_ref()
                .is_err_and(|e| e.contains("can have only one level of kind `overworld`")),
            "{err:?}"
        );
        let region = "[[region]]\nname = \"Far\"\narea = [0, 0, 3, 0]\n";
        let err = load_with(|f| {
            let text = format!("{}{region}", overworld("", "...\n"));
            f.insert("campaigns/showcase/levels/overworld.toml", &text);
        });
        assert!(
            err.as_ref()
                .is_err_and(|e| e.contains("region `Far` area [0, 0, 3, 0]")),
            "{err:?}"
        );
        let terrain = |from: &str, to: &str| {
            load_with(|f| {
                let text = f
                    .get("data/terrain.toml")
                    .expect("terrain")
                    .replacen(from, to, 1);
                f.insert("data/terrain.toml", &text);
            })
        };
        let err = terrain("travel_minutes = 45", "travel_minutes = 0");
        assert!(err.is_err_and(|e| e.contains("travel_minutes must be 1 to 1440")));
        let err = terrain("local = \"grass\"", "local = \"wall\"");
        assert!(err.is_err_and(|e| e.contains("local area ground must be walkable")));
        let err = terrain("local = \"grass\"", "local = \"lawn\"");
        assert!(err.is_err_and(|e| e.contains("unknown terrain `lawn`")));
        for landscape in ["", "landscape = \"lake\""] {
            let err = terrain("landscape = \"plains\"", landscape);
            assert!(
                err.as_ref().is_err_and(
                    |e| e.contains("`plains` has a local area but no land or road `landscape`")
                ),
                "{err:?}"
            );
        }
        let err = terrain("landscape = \"plains\"", "landscape = \"meadow\"");
        assert!(err.is_err_and(|e| e.contains("unknown variant `meadow`")));
    }

    #[test]
    fn builtin_overworld_cells_have_local_areas_but_locations_do_not() {
        let content = Content::builtin().expect("builtin content");
        let overworld = content.overworld_of("showcase").expect("an overworld");
        let mut explorable = 0;
        for y in 0..overworld.terrain.height() {
            for x in 0..overworld.terrain.width() {
                let p = Pos::new(x, y);
                let def = content.terrain(overworld.terrain[p]);
                let local = content.local_area("showcase", &local_area_id(p));
                if def.stairs.is_some() || !def.walkable {
                    assert!(local.is_none(), "{p} {}", def.id);
                }
                if let Some(area) = local {
                    assert!(content.terrain(area.ground).walkable);
                    assert!(content.has_level("showcase", &local_area_id(p)));
                    explorable += 1;
                }
            }
        }
        assert!(explorable > 1000, "{explorable} explorable cells");
        let outside = Pos::new(overworld.terrain.width(), 0);
        assert!(!content.has_level("showcase", &local_area_id(outside)));
    }
}
