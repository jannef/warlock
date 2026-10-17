//! Creature shadows (spec §4.4): visible creatures shade the visual light of the sources behind
//! them. Visual only: gameplay light and visibility never see them.
//!
//! A creature casts a shadow from every perceived light source that lights its cell, except one
//! on its own cell (a creature never shades the light it carries). The source is shadowcast
//! twice ([`compute_fov`]) over the terrain the player perceives: as it is, and with the
//! creature's cell as an extra occluder. The source's contribution cells lit the first time but
//! not the second are behind the creature. A creature does not fill its cell, so they keep part
//! of the light ([`keep`]): [`KEEP`] right behind the creature, all of it again [`REACH`] cells
//! away. Where several creatures stand in the way their shadows multiply. Colored shadows follow
//! on their own: a creature blocking the red crystal leaves the cells behind it lit by the others.
//!
//! The sun is directional: a creature, [`CREATURE_HEIGHT`] cell tall, shades the cells along the
//! sun's shadow line from its cell ([`warlock_core::light::Sun::shadow_steps`], the line terrain
//! shadows use), up to the first opaque cell: [`KEEP`] next to it, easing back to all of the light
//! one step past the shadow's end, so shadows are long at dawn and dusk and short at noon.
//!
//! [`Shadows`] is derived from the perception snapshot and recomputed when it changes (once per
//! turn), not per frame; [`super::lighting::VisualLight::composite`] scales each source's
//! contributions by [`SourceShadows::keep`]. Creatures sliding between cells cast their shadow
//! from the cell they move to.

use crate::game::Game;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use warlock_core::Pos;
use warlock_core::fov::compute_fov;
use warlock_core::light::Sun;
use warlock_core::perception::{CellView, Emitter, PerceptionView};

/// Fraction of a source's light that reaches the cell right behind a creature.
pub const KEEP: f32 = 0.4;
/// How tall a creature stands for sun shadows, in cells (terrain heights are in the same unit).
pub const CREATURE_HEIGHT: u8 = 1;
/// Distance from the creature (cells, Euclidean) at which its shadow has faded out.
pub const REACH: f32 = 3.5;

/// How much of a source's light gets past a creature to a cell `behind` cells away from it (in
/// its shadow): [`KEEP`] next to it, easing back to all of it at [`REACH`].
pub fn keep(behind: f32) -> f32 {
    let t = ((behind - 1.0) / (REACH - 1.0)).clamp(0.0, 1.0);
    KEEP + (1.0 - KEEP) * t * t * (3.0 - 2.0 * t)
}

/// Creature shadows for the current perception snapshot.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct Shadows {
    /// By source key: the cells its light is shaded at, sorted by position, with the fraction of
    /// its light they keep.
    sources: BTreeMap<u64, Vec<(Pos, f32)>>,
}

impl Shadows {
    /// The shadows the creatures in `view` cast from its light sources.
    pub fn compute(view: &PerceptionView<'_>) -> Self {
        let content = view.content();
        let (width, height) = (view.width(), view.height());
        // Terrain as the player knows it; unknown cells pass light (it reached the cells in view
        // somehow), and the shadowcast needs everything off the level opaque.
        let is_opaque = |p: Pos| {
            if p.x < 0 || p.y < 0 || p.x >= width || p.y >= height {
                return true;
            }
            match view.cell(p) {
                CellView::Visible(t) | CellView::Remembered(t) => !content.terrain(t).transparent,
                CellView::Unknown => false,
            }
        };
        let mut sources = BTreeMap::new();
        for source in view.light_sources() {
            let lit: BTreeSet<Pos> = source.contributions.iter().map(|(p, _)| *p).collect();
            let blockers: Vec<Pos> = view
                .actors()
                .iter()
                .map(|a| a.pos)
                .filter(|&p| Some(p) != source.pos() && lit.contains(&p))
                .collect();
            if blockers.is_empty() {
                continue;
            }
            let mut kept: BTreeMap<Pos, f32> = BTreeMap::new();
            match source.emitter {
                Emitter::Point { pos, preset } => {
                    let radius = content.light(preset).radius;
                    let open = Shadowcast::new(pos, radius, is_opaque);
                    for blocker in blockers {
                        let cells = source.contributions.iter().map(|(p, _)| *p);
                        for (p, k) in shade(&open, blocker, is_opaque, cells) {
                            *kept.entry(p).or_insert(1.0) *= k;
                        }
                    }
                }
                Emitter::Sun(sun) => {
                    for blocker in blockers {
                        for (p, k) in sun_shade(&sun, blocker, is_opaque, |p| lit.contains(&p)) {
                            *kept.entry(p).or_insert(1.0) *= k;
                        }
                    }
                }
            }
            if !kept.is_empty() {
                sources.insert(source.key, kept.into_iter().collect());
            }
        }
        Self { sources }
    }

    /// Shadows on given cells, by source key: the fraction of the light each cell keeps.
    #[cfg(test)]
    pub fn from_cells(sources: impl IntoIterator<Item = (u64, Vec<(Pos, f32)>)>) -> Self {
        let sources = sources
            .into_iter()
            .map(|(key, mut cells)| {
                cells.sort_by_key(|(p, _)| *p);
                (key, cells)
            })
            .collect();
        Self { sources }
    }

    /// The shadows on the light of the source with key `key`.
    pub fn of(&self, key: u64) -> SourceShadows<'_> {
        SourceShadows(self.sources.get(&key).map_or(&[], Vec::as_slice))
    }
}

/// One light source's shadows, from [`Shadows::of`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SourceShadows<'a>(&'a [(Pos, f32)]);

impl SourceShadows<'_> {
    /// The fraction of the source's light that reaches `p`: 1 unless a creature shades it.
    pub fn keep(&self, p: Pos) -> f32 {
        self.0
            .binary_search_by_key(&p, |&(q, _)| q)
            .map_or(1.0, |i| self.0[i].1)
    }
}

/// The cells a light source reaches: its shadowcast within its radius.
struct Shadowcast {
    origin: Pos,
    radius: i32,
    /// Row-major over the square of side `2 · radius + 1` around `origin`.
    lit: Vec<bool>,
}

impl Shadowcast {
    fn new(origin: Pos, radius: i32, is_opaque: impl Fn(Pos) -> bool) -> Self {
        let radius = radius.max(0);
        let side = 2 * radius + 1;
        let mut cast = Self {
            origin,
            radius,
            lit: vec![false; (side * side) as usize],
        };
        compute_fov(origin, radius, is_opaque, |p| {
            if let Some(i) = cast.index(p) {
                cast.lit[i] = true;
            }
        });
        cast
    }

    fn index(&self, p: Pos) -> Option<usize> {
        let side = 2 * self.radius + 1;
        let (dx, dy) = (
            p.x - self.origin.x + self.radius,
            p.y - self.origin.y + self.radius,
        );
        ((0..side).contains(&dx) && (0..side).contains(&dy)).then_some((dy * side + dx) as usize)
    }

    fn reaches(&self, p: Pos) -> bool {
        self.index(p).is_some_and(|i| self.lit[i])
    }
}

/// The cells among `cells` that a creature at `blocker` shades from the light whose shadowcast
/// over the terrain (`is_opaque`) is `open`, each with the fraction of the light it keeps. Cells
/// the light does not reach anyway are left alone, as are the blocker's own cell and cells
/// [`REACH`] or more away from it.
fn shade(
    open: &Shadowcast,
    blocker: Pos,
    is_opaque: impl Fn(Pos) -> bool,
    cells: impl IntoIterator<Item = Pos>,
) -> Vec<(Pos, f32)> {
    let near = |p: Pos| p != blocker && (p.dist2(blocker) as f32) < REACH * REACH;
    let candidates: Vec<Pos> = cells
        .into_iter()
        .filter(|&p| near(p) && open.reaches(p))
        .collect();
    if candidates.is_empty() {
        return Vec::new();
    }
    let blocked = Shadowcast::new(open.origin, open.radius, |p| p == blocker || is_opaque(p));
    candidates
        .into_iter()
        .filter(|&p| !blocked.reaches(p))
        .map(|p| (p, keep((p.dist2(blocker) as f32).sqrt())))
        .collect()
}

/// The cells a creature at `blocker` shades from the sun, each with the fraction of the light it
/// keeps: the sun's shadow line for a creature's height, up to the first opaque cell, among the
/// cells the sun lights (`lit`). [`KEEP`] right behind the creature, easing back to all of the
/// light one step past the shadow's end.
fn sun_shade(
    sun: &Sun,
    blocker: Pos,
    is_opaque: impl Fn(Pos) -> bool,
    lit: impl Fn(Pos) -> bool,
) -> Vec<(Pos, f32)> {
    let steps = sun.shadow_steps(CREATURE_HEIGHT);
    let len = steps.len() as f32;
    steps
        .map(|(dx, dy)| Pos::new(blocker.x + dx, blocker.y + dy))
        .take_while(|&p| !is_opaque(p))
        .enumerate()
        .filter(|&(_, p)| lit(p))
        .map(|(k, p)| (p, keep(1.0 + k as f32 * (REACH - 1.0) / len)))
        .collect()
}

/// Recomputes the shadows when the perception snapshot may have changed: after a turn, a level
/// change, or a new or loaded game.
pub fn update(game: Res<Game>, mut shadows: ResMut<Shadows>) {
    if game.is_changed() {
        shadows.set_if_neq(Shadows::compute(&game.world.perception()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::content::Content;

    #[test]
    fn shadows_are_soft_and_short() {
        assert_eq!(keep(1.0), KEEP);
        assert_eq!(keep(0.5), KEEP);
        assert_eq!(keep(REACH), 1.0);
        assert_eq!(keep(10.0), 1.0);
        let mut last = KEEP;
        for i in 1..=25 {
            let k = keep(1.0 + i as f32 * 0.1);
            assert!(k > last && k <= 1.0, "{k} after {last}");
            last = k;
        }
        assert!((0.3..=0.5).contains(&KEEP), "a creature does not black out");
    }

    /// A walled 21×21 room with a pillar at (13, 10).
    fn room(p: Pos) -> bool {
        p.x <= 0 || p.y <= 0 || p.x >= 20 || p.y >= 20 || p == Pos::new(13, 10)
    }

    fn floor() -> Vec<Pos> {
        (1..20)
            .flat_map(|y| (1..20).map(move |x| Pos::new(x, y)))
            .filter(|&p| !room(p))
            .collect()
    }

    #[test]
    fn cells_behind_a_creature_are_shaded() {
        let light = Pos::new(5, 5);
        let open = Shadowcast::new(light, 7, room);
        let shaded = shade(&open, Pos::new(7, 5), room, floor());
        let at = |x, y| {
            shaded
                .iter()
                .find(|(p, _)| *p == Pos::new(x, y))
                .map(|(_, k)| *k)
        };
        // Straight behind, darkest next to the creature and lighter farther away.
        assert_eq!(at(8, 5), Some(KEEP));
        assert_eq!(at(9, 5), Some(keep(2.0)));
        assert_eq!(at(10, 5), Some(keep(3.0)));
        assert_eq!(at(11, 5), None, "short: faded out at REACH");
        // Not the creature's own cell, nor cells in front of or beside it.
        for (x, y) in [(7, 5), (6, 5), (7, 4), (7, 6), (5, 5)] {
            assert_eq!(at(x, y), None, "({x}, {y})");
        }
        assert!(
            shaded.iter().all(|(p, _)| p.x > 7),
            "all behind: {shaded:?}"
        );
    }

    #[test]
    fn a_creature_next_to_a_light_casts_a_wide_shadow() {
        let light = Pos::new(5, 5);
        let open = Shadowcast::new(light, 7, room);
        let shaded = shade(&open, Pos::new(5, 6), room, floor());
        let cells: Vec<Pos> = shaded.iter().map(|(p, _)| *p).collect();
        assert!(cells.contains(&Pos::new(5, 7)));
        assert!(cells.contains(&Pos::new(4, 8)) && cells.contains(&Pos::new(6, 8)));
        assert!(cells.iter().all(|p| p.y > 6));
    }

    #[test]
    fn terrain_shadows_are_not_the_creatures() {
        // The pillar at (13, 10) already hides (14, 10) from a light at (10, 10). A creature right
        // in front of it shades only cells the pillar leaves lit: its wider shadow's edges.
        let light = Pos::new(10, 10);
        let open = Shadowcast::new(light, 7, room);
        assert!(!open.reaches(Pos::new(14, 10)));
        let shaded = shade(&open, Pos::new(12, 10), room, floor());
        assert!(!shaded.is_empty());
        assert!(shaded.iter().all(|(p, _)| open.reaches(*p)), "{shaded:?}");
        assert!(!shaded.iter().any(|(p, _)| *p == Pos::new(14, 10)));
    }

    #[test]
    fn source_shadows_look_up_cells() {
        let shadows =
            Shadows::from_cells([(7, vec![(Pos::new(3, 1), 0.8), (Pos::new(1, 2), 0.5)])]);
        assert_eq!(shadows.of(7).keep(Pos::new(1, 2)), 0.5);
        assert_eq!(shadows.of(7).keep(Pos::new(3, 1)), 0.8);
        assert_eq!(shadows.of(7).keep(Pos::new(2, 2)), 1.0);
        assert_eq!(shadows.of(8).keep(Pos::new(1, 2)), 1.0);
    }

    fn run(scenario: &str, actions: &str) -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        let mut world = World::new(content, scenario, None).expect("world");
        for action in parse_actions(actions).expect("actions parse") {
            world.apply(&action);
        }
        world
    }

    fn showcase(actions: &str) -> World {
        run("showcase", actions)
    }

    #[test]
    fn the_player_shades_the_torches_but_not_the_lantern() {
        let world = showcase("lantern");
        let view = world.perception();
        let shadows = Shadows::compute(&view);
        let player = view.viewer();
        let lantern = view
            .light_sources()
            .iter()
            .find(|s| s.pos() == Some(player))
            .expect("the lantern is lit");
        assert!(shadows.of(lantern.key).0.is_empty(), "never its own light");
        // The start room's torches light the player, who shades them.
        let mut shaded = 0;
        for source in view
            .light_sources()
            .iter()
            .filter(|s| s.pos() != Some(player))
        {
            let pos = source.pos().expect("no sun underground");
            for &(p, k) in shadows.of(source.key).0 {
                assert!((KEEP..1.0).contains(&k), "{p}: {k}");
                assert!(view.is_visible(p) && p != player);
                assert!(p.dist2(pos) > player.dist2(pos), "behind");
                shaded += 1;
            }
        }
        assert!(shaded > 0);
    }

    #[test]
    fn shadows_follow_the_perception() {
        let world = showcase("wait");
        let a = Shadows::compute(&world.perception());
        assert_eq!(a, Shadows::compute(&world.perception()), "deterministic");
        let moved = showcase("move:N");
        assert_ne!(a, Shadows::compute(&moved.perception()));
    }

    #[test]
    fn creatures_cast_sun_shadows_away_from_the_sun() {
        // A sun from due east, 1.5 cells of shadow per cell of height: a creature shades the two
        // cells west of it, darkest next to it; the line stops at a wall.
        let sun = Sun {
            elevation: 300,
            shadow: [-384, 0],
            light: warlock_core::light::Light::new(300, 300, 300, 0),
            sky: warlock_core::light::Light::DARK,
        };
        let open = |_: Pos| false;
        let shaded = sun_shade(&sun, Pos::new(5, 5), open, |_| true);
        assert_eq!(
            shaded,
            [
                (Pos::new(4, 5), KEEP),
                (Pos::new(3, 5), keep(1.0 + (REACH - 1.0) / 2.0))
            ]
        );
        let wall = |p: Pos| p == Pos::new(3, 5);
        assert_eq!(sun_shade(&sun, Pos::new(5, 5), wall, |_| true).len(), 1);
        // Only cells the sun lights are shaded.
        let lit = |p: Pos| p != Pos::new(4, 5);
        assert_eq!(sun_shade(&sun, Pos::new(5, 5), open, lit).len(), 1);
    }

    #[test]
    fn the_player_casts_a_sun_shadow_that_turns_with_the_day() {
        // In the courtyard, out in the open on the path: at dawn the shadow falls west, at noon
        // north, and late in the afternoon east.
        let shadow_of = |actions: &str| {
            let world = run("courtyard", actions);
            let view = world.perception();
            let player = view.viewer();
            let sun = view
                .light_sources()
                .iter()
                .find(|s| matches!(s.emitter, Emitter::Sun(_)))
                .expect("the sun is up");
            assert!(
                sun.contributions.iter().any(|(p, _)| *p == player),
                "in the sun"
            );
            let shadows = Shadows::compute(&view);
            let cells: Vec<Pos> = shadows.of(sun.key).0.iter().map(|(p, _)| *p).collect();
            assert!(!cells.is_empty(), "{actions}");
            cells
                .into_iter()
                .map(|p| (p.x - player.x, p.y - player.y))
                .collect::<Vec<_>>()
        };
        let walk = "move:E move:E move:E move:E move:E move:E move:E move:E";
        let dawn = shadow_of(&format!("{walk} skip:15m"));
        assert!(dawn.iter().all(|&(dx, _)| dx < 0), "{dawn:?}");
        let noon = shadow_of(&format!("{walk} skip:329m"));
        assert_eq!(noon, [(0, -1)]);
        let evening = shadow_of(&format!("{walk} skip:585m"));
        assert!(evening.iter().all(|&(dx, _)| dx > 0), "{evening:?}");
        assert!(evening.len() > noon.len());
    }
}
