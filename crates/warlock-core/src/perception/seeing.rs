//! Sight, the first and third steps of [`Perception::compute`](super::Perception::compute): the
//! player's line of sight and the bond's viewpoint ([`line_of_sight`]), then which cells they see
//! by the light found on them ([`Seeing`]: by eye, Blood scent, the bond, wall completion, and
//! walls by their face light).

use super::{Sight, VIEW_RADIUS, Viewing};
use crate::content::{Content, Stance, VisionProfile};
use crate::daylight::{self, DayPhase};
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::grid::Window;
use crate::light::{Area, Light, LightCache};
use crate::sight::{self, LineOfSight};
use crate::talent::TalentMods;
use crate::world::{self, Actor, ActorId, ActorKind, WorldState};

/// The second viewpoint of Hunter's bond and Familiar's sight (the module docs): the kept beast
/// on the player's level, its line of sight within its type's sight (on the overworld no farther
/// than daylight allows), its eyes, and how far its nose reaches for them.
pub(super) struct BondView<'a> {
    pub(super) beast: &'a Actor,
    pub(super) los: LineOfSight,
    pub(super) eye: VisionProfile,
    pub(super) nose: Option<i32>,
}

/// The [`BondView`] of the beast the player's talents share, Hunter's bond the companion's and
/// Familiar's sight the familiar's, if it stands on their level (not in the retinue). No class
/// holds both talents; were one to, the lower id's would be the viewpoint.
pub(super) fn bond_view<'a>(
    state: &'a WorldState,
    content: &Content,
    talents: &TalentMods,
) -> Option<BondView<'a>> {
    if !talents.beast.shared_sight && !talents.beast.familiar_sight {
        return None;
    }
    let level = state.current();
    let beast = (state.actors.values()).find(|a| {
        a.master() == Some(state.player)
            && ((talents.beast.shared_sight && a.companion().is_some())
                || (talents.beast.familiar_sight && a.familiar().is_some()))
            && a.level == level.id
            && !a.in_retinue()
    })?;
    let def = content.creature(beast.creature);
    let radius = sight::sight_radius(state, content, def.sight);
    let stops = level.sight_stops(content, beast.pos, state.time);
    let los = LineOfSight::compute(beast.pos, radius, stops);
    let eye = (beast.eyes(content, &content.vision_profiles()[def.vision])).into_owned();
    let least = (beast.familiar())
        .and_then(|_| content.familiars())
        .map(|rules| rules.nose);
    Some(BondView {
        beast,
        los,
        eye,
        nose: def.nose.max(least),
    })
}

/// The player's line of sight, and the bond's viewpoint beside it.
pub(super) struct Los<'a> {
    /// How far the player sees, however bright the light.
    pub(super) range: i32,
    /// The cells in the player's line of sight.
    pub(super) grid: Window<bool>,
    /// The same cells, in the order they were found.
    pub(super) cells: Vec<Pos>,
    pub(super) bond: Option<BondView<'a>>,
    /// The player's line of sight and the beast's together, as `grid` and `cells` are: where
    /// light is found. `None` without a bond: the player's alone.
    both: Option<(Window<bool>, Vec<Pos>)>,
    /// What is in view and a cell round it (wall completion, wall faces): the box of the per-turn
    /// grids. A write beyond (a creature smelt out of sight) grows it.
    pub(super) bounds: (Pos, Pos),
}

impl Los<'_> {
    /// The cells light is found on (the player's line of sight, and the beast's).
    pub(super) fn lit_grid(&self) -> &Window<bool> {
        self.both.as_ref().map_or(&self.grid, |(g, _)| g)
    }

    /// [`Los::lit_grid`]'s cells, in the order they were found.
    pub(super) fn lit_cells(&self) -> &[Pos] {
        self.both.as_ref().map_or(&self.cells, |(_, c)| c)
    }

    /// Whether light is found on `p`.
    pub(super) fn lit(&self, p: Pos) -> bool {
        self.lit_grid().get(p) == Some(&true)
    }

    /// How far from the player light is found: their view, or as far as the beast sees beyond
    /// it.
    pub(super) fn light_reach(&self, viewer: Pos) -> i32 {
        self.bond.as_ref().map_or(VIEW_RADIUS, |bond| {
            let apart = bond.beast.pos.dist2(viewer).isqrt() + 1;
            VIEW_RADIUS.max(apart + bond.los.radius())
        })
    }
}

/// How far the player sees, however bright the light: [`VIEW_RADIUS`], but on the overworld as
/// far as daylight (and moonlight, and rain) allows; light does not matter there.
fn range(v: &Viewing) -> i32 {
    let (state, content, talents) = (v.state, v.content, v.talents);
    match &v.sky {
        Some(sky) if v.overworld => {
            let range = daylight::sight_under(state.time, sky, content.climate());
            // Moon-eyed (spec §8 "Talents"): farther at night; Eyes of the heights (spec §8
            // "Birth sign"): farther by day too.
            // A full moon lengthens a gift's (spec §8 "Birth sign", the flavour).
            let moon = if talents.gift_reach.night_sight {
                content.full_moon_reach(state.time)
            } else {
                0
            };
            match daylight::phase(state.time) {
                DayPhase::Night => range + talents.senses.night_sight + moon,
                DayPhase::Day => range + talents.senses.day_sight,
                DayPhase::Twilight => range,
            }
        }
        _ => VIEW_RADIUS,
    }
}

/// The player's line of sight, and the bond's viewpoint (Hunter's bond, Familiar's sight).
pub(super) fn line_of_sight<'a>(v: &Viewing<'a>) -> Los<'a> {
    let (viewer, level) = (v.viewer, v.level);
    let (width, height) = (level.width(), level.height());
    let range = range(v);
    // The per-turn grids store only the box of cells in view (`Window`): on the vast level a
    // whole grid is 120,000 cells.
    let (lo, hi) = (
        Pos::new(viewer.x - range, viewer.y - range),
        Pos::new(viewer.x + range, viewer.y + range),
    );
    let mut grid = Window::new(width, height, lo, hi, false);
    let mut cells = Vec::new();
    // Smoke stops sight as a wall does, but is no wall (`world/magic/zone.rs`).
    let stops = level.sight_stops(v.content, viewer, v.state.time);
    compute_fov(viewer, range, stops, |p| {
        if let Some(seen) = grid.get_mut(p)
            && !*seen
        {
            *seen = true;
            cells.push(p);
        }
    });
    // Hunter's bond, Familiar's sight: the beast's line of sight is lit as the player's is
    // (without either the same as the player's).
    let bond = bond_view(v.state, v.content, v.talents);
    let both = bond.as_ref().map(|bond| {
        let (mut lit_grid, mut lit_cells) = (grid.clone(), cells.clone());
        let (r, at) = (bond.los.radius(), bond.beast.pos);
        lit_grid.cover(Pos::new(at.x - r, at.y - r), Pos::new(at.x + r, at.y + r));
        for p in bond.los.cells() {
            if let Some(seen) = lit_grid.get_mut(p)
                && !*seen
            {
                *seen = true;
                lit_cells.push(p);
            }
        }
        (lit_grid, lit_cells)
    });
    let lit_cells = both.as_ref().map_or(&cells, |(_, c)| c);
    let bounds = Area::around(lit_cells.iter().copied(), 1, width, height)
        .map_or((viewer, viewer), |a| (a.min, a.max));
    Los {
        range,
        grid,
        cells,
        bond,
        both,
        bounds,
    }
}

/// Which cells the player sees this turn, and how.
pub(super) struct Seeing {
    pub(super) sight: Window<Sight>,
    /// Visible cells in the order they were found.
    pub(super) visible_cells: Vec<Pos>,
    /// How far the player sees now, squared: their range, or less dazzled.
    reach2: i32,
    /// Whether strong light dazzles the player's light-blind eyes.
    pub(super) dazzled: bool,
    /// The creatures Blood scent finds, in actor id order.
    pub(super) scented: Vec<ActorId>,
}

impl Seeing {
    /// Which cells the player sees by `light` (the light step's): transparent cells with enough
    /// light or within darkvision, those Blood scent and the bond find, and the walls around
    /// them (wall completion). Walls seen by their face light come after the light on wall faces
    /// is found ([`Seeing::see_walls`]).
    pub(super) fn see(
        v: &Viewing,
        los: &Los,
        light: &Window<Light>,
        lights: &mut LightCache,
    ) -> Self {
        let (state, content, level, viewer) = (v.state, v.content, v.level, v.viewer);
        let (width, height) = (level.width(), level.height());
        // Light-blind eyes in glare: strong light on the viewer's own cell dazzles them, and
        // they see nothing farther than their race says.
        let dazzled = (state.race_of(content, state.player()))
            .and_then(|race| race.light_blind)
            .filter(|blind| blind.dazzled_by(light[viewer]));
        let range = los.range;
        let mut seeing = Seeing {
            sight: Window::new(width, height, los.bounds.0, los.bounds.1, Sight::Hidden),
            visible_cells: Vec::new(),
            reach2: radius2(dazzled.map_or(range, |blind| range.min(blind.sight))),
            dazzled: dazzled.is_some(),
            scented: Vec::new(),
        };

        // Transparent cells: visible with enough light or within darkvision; the viewer's own
        // cell always.
        let mut seen = Vec::new();
        let smoke_hides = |p: Pos| level.smoke_hides(viewer, p, state.time);
        for &p in los.cells.iter().filter(|p| !v.is_opaque(**p)) {
            let by_eye = seeing.sees(v, light[p], p.dist2(viewer)) && !smoke_hides(p);
            if by_eye || p == viewer {
                seeing.show(p);
            }
            if by_eye {
                seen.push(p);
            }
        }
        // Blood scent (spec §8 "Talents"): creatures suffering a status, in line of sight within
        // its radius, however dark their cells.
        for (id, p) in scented(v, los) {
            seeing.scented.push(id);
            if seeing.sight[p] == Sight::Hidden {
                seeing.show(p);
            }
        }
        // The beast's viewpoint: what it sees by its eyes, and the hostiles its nose finds.
        let bond_seen = match &los.bond {
            Some(bond) => seeing.see_by_bond(v, bond, lights),
            None => Vec::new(),
        };

        // Wall completion: opaque cells bounding a transparent cell seen by eye (the companion's
        // too, within its own sight).
        for q in seen {
            for w in Dir::ALL.map(|d| q.step(d)) {
                if seeing.sight.get(w) == Some(&Sight::Hidden)
                    && v.is_opaque(w)
                    && w.dist2(viewer) <= seeing.reach2
                {
                    seeing.sight[w] = if los.grid[w] {
                        Sight::Seen
                    } else {
                        Sight::Edge
                    };
                    seeing.visible_cells.push(w);
                }
            }
        }
        for q in bond_seen {
            for w in Dir::ALL.map(|d| q.step(d)) {
                if seeing.sight.get(w) == Some(&Sight::Hidden) && v.is_opaque(w) {
                    seeing.sight[w] = if los.lit_grid()[w] {
                        Sight::Seen
                    } else {
                        Sight::Edge
                    };
                    seeing.visible_cells.push(w);
                }
            }
        }
        seeing
    }

    /// Other opaque cells in line of sight: visible by their face light or darkvision.
    pub(super) fn see_walls(&mut self, v: &Viewing, los: &Los, light: &Window<Light>) {
        for &p in &los.cells {
            if self.sight[p] == Sight::Hidden
                && v.is_opaque(p)
                && self.sees(v, light[p], p.dist2(v.viewer))
            {
                self.show(p);
            }
        }
    }

    /// Whether the player's eyes see a cell `dist2` away lit by `light` (on the overworld, any
    /// within their range).
    fn sees(&self, v: &Viewing, light: Light, dist2: i32) -> bool {
        dist2 <= self.reach2 && (v.overworld || v.eye.sees(light, dist2))
    }

    /// Shows `p` as seen.
    fn show(&mut self, p: Pos) {
        self.sight[p] = Sight::Seen;
        self.visible_cells.push(p);
    }

    /// Whether `p` is visible (seen, or a completed wall).
    pub(super) fn is_visible(&self, p: Pos) -> bool {
        self.sight.get(p).is_some_and(|s| *s != Sight::Hidden)
    }

    /// Shows what the bond's beast sees by its eyes and the hostiles its nose finds. Returns the
    /// open cells it sees, for wall completion.
    fn see_by_bond(&mut self, v: &Viewing, bond: &BondView, lights: &mut LightCache) -> Vec<Pos> {
        let (state, content) = (v.state, v.content);
        let mut bond_seen = Vec::new();
        for p in bond.los.cells().filter(|p| !v.is_opaque(*p)) {
            if self.sight.get(p) != Some(&Sight::Hidden) {
                continue;
            }
            if sight::sees_by(state, content, lights, &bond.los, &bond.eye, p).is_some() {
                self.show(p);
                bond_seen.push(p);
            }
        }
        let hostile = |s: &WorldState, c: &Content, _: &Actor, a: &Actor| {
            world::stance_between(s, c, s.player(), a) == Stance::Hostile
        };
        let smelt = bond.nose.map_or_else(Vec::new, |nose| {
            world::smelt_within(state, content, bond.beast, nose, &hostile)
        });
        for (_, p) in smelt {
            if self.sight.get(p) == Some(&Sight::Hidden) {
                self.show(p);
            }
        }
        bond_seen
    }
}

/// The creatures Blood scent finds (spec §8 "Talents"): those suffering a status of the talent's,
/// in line of sight within its radius, in actor id order.
fn scented(v: &Viewing, los: &Los) -> Vec<(ActorId, Pos)> {
    let (state, content, talents, viewer) = (v.state, v.content, v.talents, v.viewer);
    if v.overworld || talents.senses.scent.is_empty() {
        return Vec::new();
    }
    // A full moon lengthens a gift's (Hound's nose; spec §8 "Birth sign").
    let moon = if talents.gift_reach.scent.is_empty() {
        0
    } else {
        content.full_moon_reach(state.time)
    };
    let reach = |status, radius: i32| {
        radius
            + if talents.gift_reach.scent.contains(&status) {
                moon
            } else {
                0
            }
    };
    (state.actors.values())
        .filter(|a| a.level == v.level.id && a.kind == ActorKind::Creature)
        .filter(|a| los.grid.get(a.pos) == Some(&true) && !v.is_opaque(a.pos))
        .filter(|a| {
            (talents.senses.scent.iter()).any(|&(status, radius)| {
                a.effects.has(status) && a.pos.dist2(viewer) <= radius2(reach(status, radius))
            })
        })
        .map(|a| (a.id, a.pos))
        .collect()
}
