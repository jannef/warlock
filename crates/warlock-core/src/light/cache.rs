//! Light sources per level and their cached shadow masks (spec §5, "Recomputation").
//!
//! A mask depends only on the source's position, its preset, and opacity within its radius, so
//! it is computed once and reused until the source moves or a cell within its radius changes
//! ([`LightCache::terrain_changed`]). The cache is derived data: it is never saved, and a cold
//! cache gives exactly the same light as a warm one.

use super::{Light, falloff};
use crate::content::{Content, LightId, LightPreset};
use crate::fov::compute_fov;
use crate::geom::Pos;
use crate::world::{LevelState, WorldState};
use std::collections::BTreeMap;

/// Key tag for lights carried by actors (creature lights).
const CARRIED: u64 = 1 << 63;
/// Extra key tag for the player's lantern.
const LANTERN: u64 = 1 << 62;

/// A light source active on a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Source {
    /// Stable across turns: level + position for static lights, the actor for carried ones.
    pub key: u64,
    pub pos: Pos,
    pub preset: LightId,
}

/// The transparent cells a source lights, each with the light it receives there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Mask {
    origin: Pos,
    preset: LightId,
    radius: i32,
    /// Row-major; cells that receive no light are left out.
    pub cells: Vec<(Pos, Light)>,
}

impl Mask {
    /// Shadowcasts `preset` from `origin`.
    pub fn compute(
        origin: Pos,
        id: LightId,
        preset: &LightPreset,
        is_opaque: impl Fn(Pos) -> bool,
    ) -> Self {
        let r = preset.radius.max(0);
        let side = 2 * r + 1;
        let offset = |p: Pos| {
            let (dx, dy) = (p.x - origin.x + r, p.y - origin.y + r);
            ((0..side).contains(&dx) && (0..side).contains(&dy))
                .then_some((dy * side + dx) as usize)
        };
        let mut lit = vec![false; (side * side) as usize];
        compute_fov(origin, r, &is_opaque, |p| {
            if let Some(i) = offset(p) {
                lit[i] = true;
            }
        });
        let mut cells = Vec::new();
        for (i, _) in lit.iter().enumerate().filter(|(_, lit)| **lit) {
            let i = i as i32;
            let p = Pos::new(origin.x + i % side - r, origin.y + i / side - r);
            if is_opaque(p) {
                continue;
            }
            let light = falloff(preset.color, r, p.dist2(origin));
            if !light.is_dark() {
                cells.push((p, light));
            }
        }
        Self {
            origin,
            preset: id,
            radius: r,
            cells,
        }
    }

    /// Whether any lit cell can lie within `min..=max` (corners included): the mask's square of
    /// side `2 · radius + 1` overlaps it.
    pub fn overlaps(&self, min: Pos, max: Pos) -> bool {
        let r = self.radius;
        self.origin.x + r >= min.x
            && self.origin.x - r <= max.x
            && self.origin.y + r >= min.y
            && self.origin.y - r <= max.y
    }

    fn is_for(&self, source: &Source) -> bool {
        self.origin == source.pos && self.preset == source.preset
    }
}

/// Cached sources and masks for every level visited this session.
#[derive(Clone, Debug, Default)]
pub(crate) struct LightCache {
    levels: BTreeMap<String, LevelLights>,
}

#[derive(Clone, Debug)]
struct LevelLights {
    /// Static lights: placed by the level legend, or emitted by terrain.
    statics: BTreeMap<Pos, LightId>,
    /// Shadow masks by source key.
    masks: BTreeMap<u64, Mask>,
}

impl LightCache {
    /// Sources on the player's level whose light can reach a cell within `reach` of `center`, in a
    /// stable order (static lights by position, then carried lights by actor id), each with its
    /// shadow mask. Masks are computed only for these sources, and only when missing or stale.
    pub fn near(
        &mut self,
        state: &WorldState,
        content: &Content,
        center: Pos,
        reach: i32,
    ) -> Vec<(Source, &Mask)> {
        let level = state.current();
        let lights = self
            .levels
            .entry(level.id.clone())
            .or_insert_with(|| LevelLights::new(level, content));
        let sources: Vec<Source> = lights
            .sources(state, content)
            .into_iter()
            .filter(|s| {
                let max = reach + content.light(s.preset).radius;
                s.pos.dist2(center) <= max * max
            })
            .collect();
        for source in &sources {
            let fresh = lights
                .masks
                .get(&source.key)
                .is_some_and(|m| m.is_for(source));
            if !fresh {
                let preset = content.light(source.preset);
                let mask = Mask::compute(source.pos, source.preset, preset, |p| {
                    level.is_opaque(content, p)
                });
                lights.masks.insert(source.key, mask);
            }
        }
        let lights = &*lights;
        sources
            .into_iter()
            .map(|s| (s, &lights.masks[&s.key]))
            .collect()
    }

    /// Call after the terrain at `p` on `level` changed: drops masks whose radius covers `p` (its
    /// opacity may have changed) and re-derives the static light at `p`.
    pub fn terrain_changed(&mut self, level: &LevelState, content: &Content, p: Pos) {
        let Some(lights) = self.levels.get_mut(&level.id) else {
            return;
        };
        lights.masks.retain(|_, m| m.origin.chebyshev(p) > m.radius);
        lights.statics.remove(&p);
        if let Some(id) = static_light(level, content, p) {
            lights.statics.insert(p, id);
        }
    }

    /// Number of cached masks on `level` (for tests and diagnostics).
    #[cfg(test)]
    pub fn cached_masks(&self, level: &str) -> usize {
        self.levels.get(level).map_or(0, |l| l.masks.len())
    }
}

impl LevelLights {
    fn new(level: &LevelState, content: &Content) -> Self {
        let mut statics: BTreeMap<Pos, LightId> = level
            .terrain
            .iter()
            .filter_map(|(p, t)| content.terrain(*t).light.map(|id| (p, id)))
            .collect();
        // Placed lights override terrain lights on the same cell.
        statics.extend(level.lights.iter().map(|(p, id)| (*p, *id)));
        Self {
            statics,
            masks: BTreeMap::new(),
        }
    }

    /// Every source currently shining on the player's level. A source in an opaque cell (e.g. a
    /// light placed on a door that was then closed) gives no light.
    fn sources(&self, state: &WorldState, content: &Content) -> Vec<Source> {
        let level = state.current();
        let mut sources: Vec<Source> = self
            .statics
            .iter()
            .map(|(&pos, &preset)| Source {
                key: static_key(&level.id, pos),
                pos,
                preset,
            })
            .collect();
        let lantern = content
            .scenario(&state.scenario)
            .and_then(|s| s.lantern.as_deref())
            .and_then(|id| content.light_id(id))
            .filter(|_| state.lantern_lit);
        for actor in state.actors.values().filter(|a| a.level == level.id) {
            let id = u64::from(actor.id.0);
            if let Some(preset) = content.creature(actor.creature).light {
                sources.push(Source {
                    key: CARRIED | id,
                    pos: actor.pos,
                    preset,
                });
            }
            if actor.id == state.player
                && let Some(preset) = lantern
            {
                sources.push(Source {
                    key: CARRIED | LANTERN | id,
                    pos: actor.pos,
                    preset,
                });
            }
        }
        sources.retain(|s| !level.is_opaque(content, s.pos));
        sources
    }
}

/// The static light at `p`: a light placed there by the level legend, else the terrain's own.
fn static_light(level: &LevelState, content: &Content, p: Pos) -> Option<LightId> {
    let placed = level.lights.get(&p).copied();
    placed.or_else(|| level.terrain.get(p).and_then(|t| content.terrain(*t).light))
}

/// FNV-1a over the level id and position, with the carried-light tag bits cleared.
fn static_key(level: &str, p: Pos) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let bytes = level
        .bytes()
        .chain([0xff])
        .chain(p.x.to_le_bytes())
        .chain(p.y.to_le_bytes());
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash & !(CARRIED | LANTERN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    const SIZE: i32 = 16;

    fn preset(radius: i32) -> LightPreset {
        LightPreset {
            id: "test".into(),
            color: Light::new(255, 255, 255, 255),
            radius,
            stability: Default::default(),
        }
    }

    fn arb_map() -> impl Strategy<Value = Vec<Vec<bool>>> {
        let row = prop::collection::vec(prop::bool::weighted(0.3), SIZE as usize);
        prop::collection::vec(row, SIZE as usize)
    }

    fn opaque_fn(map: &[Vec<bool>]) -> impl Fn(Pos) -> bool + '_ {
        move |p: Pos| {
            !(0..SIZE).contains(&p.x)
                || !(0..SIZE).contains(&p.y)
                || map[p.y as usize][p.x as usize]
        }
    }

    proptest! {
        #[test]
        fn light_never_crosses_walls(
            map in arb_map(), ox in 2i32..SIZE - 2, oy in 2i32..SIZE - 2, radius in 0i32..12,
        ) {
            // A ring of wall around the source's 3×3 neighborhood: no light gets past it.
            let mut map = map;
            let origin = Pos::new(ox, oy);
            for (y, row) in map.iter_mut().enumerate() {
                for (x, cell) in row.iter_mut().enumerate() {
                    match Pos::new(x as i32, y as i32).chebyshev(origin) {
                        0 | 1 => *cell = false,
                        2 => *cell = true,
                        _ => {}
                    }
                }
            }
            let mask = Mask::compute(origin, LightId(0), &preset(radius), opaque_fn(&map));
            for (p, _) in &mask.cells {
                prop_assert!(p.chebyshev(origin) <= 1, "light reached {p} through the ring");
            }
        }

        #[test]
        fn masks_light_the_transparent_cells_in_sight_with_falloff(
            map in arb_map(), ox in 0i32..SIZE, oy in 0i32..SIZE, radius in 0i32..12,
        ) {
            let origin = Pos::new(ox, oy);
            let opaque = opaque_fn(&map);
            let mut in_sight = BTreeSet::new();
            compute_fov(origin, radius, &opaque, |p| {
                in_sight.insert(p);
            });
            let expected: BTreeSet<Pos> = in_sight
                .into_iter()
                .filter(|p| !opaque(*p) && !falloff(preset(radius).color, radius, p.dist2(origin)).is_dark())
                .collect();
            let mask = Mask::compute(origin, LightId(0), &preset(radius), &opaque);
            let lit: BTreeSet<Pos> = mask.cells.iter().map(|(p, _)| *p).collect();
            prop_assert_eq!(lit, expected);
            for (p, light) in &mask.cells {
                prop_assert_eq!(*light, falloff(preset(radius).color, radius, p.dist2(origin)));
            }
        }
    }

    #[test]
    fn static_keys_are_stable_and_distinct_from_carried_keys() {
        let a = static_key("hub", Pos::new(3, 4));
        assert_eq!(a, static_key("hub", Pos::new(3, 4)));
        assert_ne!(a, static_key("hub", Pos::new(4, 3)));
        assert_ne!(a, static_key("vast", Pos::new(3, 4)));
        assert_eq!(a & CARRIED, 0);
    }
}
