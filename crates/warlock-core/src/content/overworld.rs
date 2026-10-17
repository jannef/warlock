//! The overworld and its local areas (spec §3, §7).
//!
//! The overworld is the one level of kind `overworld`: a map of the land at large in the usual
//! level format, with overworld terrain (each with `travel_minutes`) and locations, which are
//! stairs down to other levels. Every other overworld cell whose terrain names a `local` ground
//! leads into a **local area**, an outdoor level generated on the first visit
//! ([`crate::mapgen`]) whose id is `local:x,y` after its overworld cell. Like a dungeon branch
//! level, it then persists with the run.

use super::{Content, LevelKind, LevelTemplate, TerrainId};
use crate::geom::Pos;

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

/// A local area: the overworld it belongs to, its cell, and the terrain there.
#[derive(Clone, Copy, Debug)]
pub struct LocalArea<'a> {
    pub overworld: &'a LevelTemplate,
    pub pos: Pos,
    pub terrain: TerrainId,
    /// The ground the local area is made of (the overworld terrain's `local`).
    pub ground: TerrainId,
}

impl Content {
    /// The overworld: the level of kind `overworld`, if content has one (it has at most one).
    pub fn overworld(&self) -> Option<&LevelTemplate> {
        self.levels
            .values()
            .find(|l| l.kind == LevelKind::Overworld)
    }

    /// The local area with level id `id` (`local:x,y`), if the overworld has an explorable cell
    /// there: in bounds, walkable, with a `local` ground, and not a location.
    pub fn local_area(&self, id: &str) -> Option<LocalArea<'_>> {
        let pos = parse_local_area(id)?;
        let overworld = self.overworld()?;
        let terrain = *overworld.terrain.get(pos)?;
        let def = self.terrain(terrain);
        let ground = def.local.filter(|_| def.walkable && def.stairs.is_none())?;
        Some(LocalArea {
            overworld,
            pos,
            terrain,
            ground,
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
        let err = load_with(|f| f.insert("levels/overworld.toml", &overworld("", "..#\n")));
        assert!(
            err.as_ref()
                .is_err_and(|e| e.contains("`floor` is walkable but has no travel_minutes")),
            "{err:?}"
        );
        let err = load_with(|f| f.insert("levels/second.toml", &overworld("", "...\n")));
        assert!(
            err.as_ref()
                .is_err_and(|e| e.contains("only one level of kind `overworld`")),
            "{err:?}"
        );
        let region = "[[region]]\nname = \"Far\"\narea = [0, 0, 3, 0]\n";
        let err = load_with(|f| {
            let text = format!("{}{region}", overworld("", "...\n"));
            f.insert("levels/overworld.toml", &text);
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
    }

    #[test]
    fn builtin_overworld_cells_have_local_areas_but_locations_do_not() {
        let content = Content::builtin().expect("builtin content");
        let overworld = content.overworld().expect("an overworld");
        let mut explorable = 0;
        for y in 0..overworld.terrain.height() {
            for x in 0..overworld.terrain.width() {
                let p = Pos::new(x, y);
                let def = content.terrain(overworld.terrain[p]);
                let local = content.local_area(&local_area_id(p));
                if def.stairs.is_some() || !def.walkable {
                    assert!(local.is_none(), "{p} {}", def.id);
                }
                if let Some(area) = local {
                    assert!(content.terrain(area.ground).walkable);
                    assert!(content.has_level(&local_area_id(p)));
                    explorable += 1;
                }
            }
        }
        assert!(explorable > 1000, "{explorable} explorable cells");
        let outside = Pos::new(overworld.terrain.width(), 0);
        assert!(!content.has_level(&local_area_id(outside)));
    }
}
