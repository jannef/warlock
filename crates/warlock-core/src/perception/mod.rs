//! The per-turn perception snapshot (spec §5, §8).
//!
//! Everything the player can know — and therefore everything rendering, UI, and dumps may show —
//! goes through [`PerceptionView`]. It exposes currently visible cells, creatures (what they
//! wear and wield, how hurt they look, in words, and the status effects that show on them, such
//! as bleeding), objects, and light, remembered terrain and objects, and what the player carries,
//! wears, weighs, and scores in each stat, their trained skills' values, their burden and
//! carrying limit, their hit points and Defence, their own status effects by name, and how
//! hungry they are; it never exposes unseen terrain or objects, hidden creatures, what else
//! creatures carry, their burden, their skills, or their hit points, any stat's modifier or
//! potential, a skill's marks or benefits, an untrained skill, a creature's stats, a to-hit, how
//! severe a status effect is or how long it lasts, the nutrition counter, a corpse's age, or
//! light the player's eyes cannot perceive.
//!
//! **Memory** (spec §5): terrain, piles of objects, and blood as last seen. Objects are seen where
//! creatures are (cells in line of sight, never completed walls); a pile seen gone is forgotten,
//! and one that changed out of sight is remembered as it was. Known traps (spec §8 "Traps") are
//! seen and remembered the same way; hidden ones never show.
//!
//! **Blood** (spec §8 "Blood", [`crate::blood`]): a stain on an open cell is seen where objects
//! are; one on a wall only when a stained face is seen (a visible open cell next to the wall on
//! that side), so blood on the far side of a wall stays hidden. Blood is remembered like piles.
//! Char ([`crate::scorch`]) is seen and remembered by the same rules.
//!
//! A cell is **visible** iff it is in line of sight within [`VIEW_RADIUS`] **and** its perceived
//! light is at least the vision profile's threshold or it lies within the darkvision radius. The
//! player's own cell is always visible. The profile is the player's **eyes**: their vision
//! profile as their talents change it ([`crate::world::WorldState::eyes`]: Night eyes, Dark
//! adaptation, Rune sight). Blood scent sees a bleeding creature's cell in line of
//! sight within its radius as darkvision would. A **light-blind** player (their race's
//! [`LightBlind`](crate::content::LightBlind): drow) whose own cell is lit at least to its
//! `glare` is dazzled: they see nothing farther than its `sight`.
//!
//! **The unseen** (`world/unseen.rs`): a shadowy creature on a cell dim to the player's eyes
//! (not beside them) and an invisible one anywhere are left out of the actors, and their cells
//! show no one ([`Perception::sees_actors_at`]), unless Seeing, Blood scent, the companion's nose,
//! or a blow that just found it shows them; their cells count as hidden to the player's hearing,
//! so what they make is heard there.
//!
//! **Hunter's bond** (spec §8 "Allies"; `TalentMods::shared_sight`): with the player's companion
//! on their level, a **second viewpoint** ([`seeing::BondView`]): the cells the companion sees by
//! its own line of sight and eyes ([`crate::sight::sees_by`], creature sight's rule) are seen,
//! with wall completion around them, and so are the cells of the hostiles its nose finds
//! ([`crate::world::smelt`]: within its `nose`, through the dark and closed doors). The light grid
//! covers its line of sight too, perceived by the player's eye. Dazzle does not touch it.
//! **Familiar's sight** (`TalentMods::familiar_sight`) opens the same viewpoint on the player's
//! familiar, its nose reaching at least `familiars.toml`'s `nose`.
//!
//! The overworld is seen at map scale instead: a cell is visible iff it is in line of sight
//! within the daylight sight radius ([`daylight::overworld_sight`]: day 9, twilight 5, night 2,
//! more under a bright moon, less in rain; Moon-eyed adds its cells at night), whatever its
//! light; eyes and darkvision do not change that.
//!
//! **Weather** ([`crate::weather`]) is known only under the open sky: on such a level the sky
//! carries it ([`PerceptionView::weather`]), and its clouds dim the sun, the moons, and the sky.
//!
//! Light on a transparent cell is the level's ambient light, plus the light of every source that
//! reaches it, plus weak **bounce light** reflected by the lit surfaces around it
//! ([`crate::light::BOUNCE`]). On a level under the open sky ([`Sky`]) the **sun** by day and
//! the **moons** by night are more sources (directional, shaded by tall terrain; see
//! [`crate::light::Sun`] and [`crate::light::Moon`]), and the **sky** lights every cell that is
//! not roofed. Ambient plus sky plus bounce is the cell's *base light*: steady, and not tied to
//! one source.
//!
//! A **special room** (spec §7 "Special rooms") changes the light inside it: a shadowed room
//! keeps only part of every bit of it (base light and each source's contribution alike, so they
//! still add up), and a sanctuary's glow adds to the base light like ambient light (on walls
//! through the wall-face rule, from the room's side only). Creature sight finds the same light.
//!
//! Opaque cells follow two extra rules:
//! - **Wall faces**: an opaque cell is lit by a source as a transparent cell at its own distance
//!   would be, but only if the viewer sees a face the source lights (an adjacent visible
//!   transparent cell receives the same source), so light never leaks through a wall to a viewer
//!   on its dark side. Its bounce light is the brightest of its visible transparent neighbors'.
//! - **Wall completion**: an opaque cell within view range that bounds a transparent cell seen by
//!   light or darkvision is visible even outside line of sight (symmetric shadowcasting can show
//!   floor through a narrow gap without the walls around it). Such a cell shows only its terrain
//!   and wall-face light: never creatures, and nothing beyond it.
//!
//! **Hearing** ([`crate::noise`]): sounds the player heard during the last turn whose source cell
//! is not visible are perceived with their kind, volume, and a rough place (a cell the player
//! does not see, near the source); never with who made them.
//!
//! **Layout.** This module holds the snapshot ([`Perception`]), its computation, and what the
//! rules ask of it; `seeing.rs` the bond's viewpoint, `actors.rs` how a seen creature looks;
//! `own.rs` and `seen.rs` the types the view hands out (what the player knows of themselves, and
//! of the world); `view.rs` [`PerceptionView`] and its accessors that read only the snapshot,
//! `truth.rs` those that read ground truth (with its review note), and `shop.rs` shops.

mod actors;
mod own;
mod seeing;
mod seen;
mod shop;
mod truth;
mod view;

pub use own::{
    Freshness, NeedView, OwnResource, OwnScribe, OwnSkill, OwnStudy, OwnTalent, Progress,
    TalentView,
};
pub use seen::{
    AllyView, CellView, Emitter, PerceivedLight, SeenActor, SeenGear, SeenLight, Shape,
};
pub use shop::{Sale, ShopView};
pub use view::PerceptionView;

use actors::shape;
use seeing::bond_view;
use seen::allies_of;

use crate::blood::Stain;
use crate::content::{Content, LevelKind, Stance, StatusId, TerrainId, VisionProfile, ZoneKind};
use crate::daylight::{self, DayPhase};
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::grid::Window;
use crate::light::{
    Area, BOUNCE_STEPS, Bounce, Light, LightCache, SUN_KEY, Sky, Sunlight, falloff, lit_in_sight,
    moon_key, wall_faces,
};
use crate::noise::{self, PerceivedSound};
use crate::sight;
use crate::temperature::Temperature;
use crate::weather::Weather;
use crate::world::mirrors::image_id;
use crate::world::surprise::creature_unaware;
use crate::world::{self, Actor, ActorId, ActorKind, LevelState, WorldState};
use std::collections::BTreeMap;

/// Maximum sight distance, however bright the light.
pub const VIEW_RADIUS: i32 = 40;

/// How the player sees a cell this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sight {
    Hidden,
    /// In line of sight and visible.
    Seen,
    /// An opaque cell outside line of sight, visible by wall completion: terrain only.
    Edge,
}

/// Derived each turn from world state; not saved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Perception {
    viewer: Pos,
    /// The player's eyes: their vision profile as their talents change it
    /// ([`crate::world::WorldState::eyes`]).
    eye: VisionProfile,
    /// Perceived ambient light.
    ambient: Light,
    /// The sun, the moons, and the sky over a level under the open sky.
    sky: Option<Sky>,
    /// The temperature of the air where the player is (spec §5 "Temperature").
    temperature: Temperature,
    /// Ambient plus sky plus bounce light; meaningful on visible cells only.
    base: Window<Light>,
    sight: Window<Sight>,
    /// Visible cells in the order they were found.
    visible_cells: Vec<Pos>,
    /// Perceived light; meaningful on visible cells only.
    light: Window<Light>,
    sources: Vec<SeenLight>,
    actors: Vec<SeenActor>,
    /// What the seen creatures (not the player) wear and wield, by actor, in slot order.
    gear: Vec<SeenGear>,
    /// The status effects that show on the seen creatures (not the player), by actor, the most
    /// important first.
    effects: Vec<(ActorId, StatusId)>,
    /// Sounds heard during the last turn from sources the player does not see.
    sounds: Vec<PerceivedSound>,
    /// Cells whose blood the player sees, in position order.
    blood: Vec<Pos>,
    /// Cells whose char the player sees, in position order (as blood).
    scorch: Vec<Pos>,
    /// Cells where the player sees an image of a hall of mirrors (shown among `actors`).
    images: Vec<Pos>,
    /// Whether strong light dazzles the player's light-blind eyes (their race's `light_blind`).
    dazzled: bool,
    /// Whether the player averts their eyes from a gaze marked `averted` they see
    /// (`world/avert.rs`): derived here, never saved.
    averted: bool,
    /// The disguised mimics of the level (`world/mimic.rs`), by position: each is a shut chest to
    /// the player, never an actor, but those Chest-wise sees through.
    disguises: Vec<Disguise>,
    /// Those on the level the player does not see for being unseen (`world/unseen.rs`), in
    /// actor id order.
    concealed: Vec<crate::world::Concealed>,
    /// An unseen shadowy creature is within [`crate::world::DRAUGHT`] of the player: they feel
    /// its cold.
    draught: bool,
    /// The zones on the cells the player sees (`world/zone.rs`), by position.
    zones: BTreeMap<Pos, ZoneKind>,
    /// The player's allies, wherever they are (`world/ally.rs`), in actor id order: the player
    /// knows their own.
    allies: Vec<AllyView>,
    /// What the player may treat on their allies beside them (`world/heal.rs`): they know their
    /// own allies' hurts.
    ally_ailments: Vec<crate::heal::OwnAilment>,
    /// The ways to the player's kept beasts and escorts beside them (`world/heal.rs`,
    /// `world/escort.rs`), in direction order: those who take what the player gives.
    kept_beside: Vec<crate::geom::Dir>,
    /// An escort of the player's stands beside them (`world/escort.rs`): the Give list holds
    /// anything one could put on.
    escort_beside: bool,
}

/// A disguised mimic as the snapshot hides it: a shut chest of its level's band at its cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Disguise {
    pos: Pos,
    /// The shut chest terrain it shows.
    terrain: TerrainId,
    /// Its level, which names the chest's band (the floor's, as a chest's).
    level: u8,
}

impl Perception {
    /// Computes what the player perceives on their current level: line of sight, light, and
    /// visibility. Light masks come from (and are added to) `lights`.
    pub(crate) fn compute(state: &WorldState, content: &Content, lights: &mut LightCache) -> Self {
        let viewer = state.player().pos;
        let level = state.current();
        let (width, height) = (level.width(), level.height());
        let talents = state.player().talent_mods(content);
        let eyes = state.eyes(content);
        let eye = &eyes;
        let is_opaque = |p: Pos| level.is_opaque(content, p);
        let sky = state.sky(content);
        // On the overworld, daylight (and moonlight, and rain) sets how far one sees; light does
        // not matter.
        let overworld = level.kind == LevelKind::Overworld;
        let range = match &sky {
            Some(sky) if overworld => {
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
                    DayPhase::Night => range + talents.night_sight + moon,
                    DayPhase::Day => range + talents.day_sight,
                    DayPhase::Twilight => range,
                }
            }
            _ => VIEW_RADIUS,
        };

        // Line of sight. The per-turn grids store only the box of cells in view (`Window`): on
        // the vast level a whole grid is 120,000 cells.
        let (lo, hi) = (
            Pos::new(viewer.x - range, viewer.y - range),
            Pos::new(viewer.x + range, viewer.y + range),
        );
        let mut los = Window::new(width, height, lo, hi, false);
        let mut los_cells = Vec::new();
        // Smoke stops sight as a wall does, but is no wall (`world/zone.rs`).
        let stops = level.sight_stops(content, viewer, state.time);
        compute_fov(viewer, range, stops, |p| {
            if let Some(seen) = los.get_mut(p)
                && !*seen
            {
                *seen = true;
                los_cells.push(p);
            }
        });
        // Hunter's bond, Familiar's sight: the beast's line of sight is lit as the player's is
        // (`lit_los`; without either the same as the player's).
        let bond = bond_view(state, content, &talents);
        let both = bond.as_ref().map(|bond| {
            let (mut lit_los, mut lit_cells) = (los.clone(), los_cells.clone());
            let (r, at) = (bond.los.radius(), bond.beast.pos);
            lit_los.cover(Pos::new(at.x - r, at.y - r), Pos::new(at.x + r, at.y + r));
            for p in bond.los.cells() {
                if let Some(seen) = lit_los.get_mut(p)
                    && !*seen
                {
                    *seen = true;
                    lit_cells.push(p);
                }
            }
            (lit_los, lit_cells)
        });
        let (lit_los, lit_cells) = both.as_ref().map_or((&los, &los_cells), |(g, c)| (g, c));
        let light_reach = bond.as_ref().map_or(VIEW_RADIUS, |bond| {
            let apart = bond.beast.pos.dist2(viewer).isqrt() + 1;
            VIEW_RADIUS.max(apart + bond.los.radius())
        });

        // Sources that can light a cell in view, or a surface whose bounce light reaches one (a
        // bounce step moves at most √2 < 2 cells; wall faces are within a cell of view).
        let near = lights.near(state, content, viewer, light_reach + 1 + 2 * BOUNCE_STEPS);
        // Direct light is needed on everything in sight and within bounce reach of it.
        let area = Area::around(lit_cells.iter().copied(), BOUNCE_STEPS, width, height);

        // Under the open sky: the sun and each moon that shines where no taller terrain shades a
        // cell, the sky everywhere (none of them under a roof).
        let terrain = |p: Pos| level.terrain.get(p).map(|t| content.terrain(*t));
        let exposed = |p: Pos| terrain(p).is_some_and(|t| !t.roofed);
        let mut bodies: Vec<(u64, Emitter, [i16; 2], Light)> = Vec::new();
        if let Some(sky) = &sky {
            if sky.sun.is_up() {
                let sun = sky.sun;
                bodies.push((SUN_KEY, Emitter::Sun(sun), sun.shadow, sun.light));
            }
            for moon in sky.moons.iter().filter(|m| m.shines()) {
                let emitter = Emitter::Moon(*moon);
                bodies.push((moon_key(moon.index), emitter, moon.shadow, moon.light));
            }
        }
        let beams: Vec<(u64, Emitter, Sunlight)> = match area {
            Some(area) => (bodies.into_iter())
                .map(|(key, emitter, shadow, light)| {
                    let height = |p: Pos| terrain(p).map_or(0, |t| t.height);
                    let tallest = content.tallest_terrain();
                    let cells =
                        Sunlight::compute(shadow, light, area, tallest, height, exposed, is_opaque);
                    (key, emitter, cells)
                })
                .collect(),
            None => Vec::new(),
        };
        let glow = sky.as_ref().map_or(Light::DARK, |s| eye.perceive(s.glow()));

        // Base light on cells in line of sight: ambient, sky, and bounce (on transparent cells;
        // RGB only, so every eye perceives all of it). The overworld never gets darker than its
        // faint night light (moonlight comes on top).
        let night = match &sky {
            Some(sky) if overworld => daylight::overworld_night(sky.sun.sky),
            _ => Light::DARK,
        };
        let ambient = eye.perceive(level.ambient.saturating_add(night));
        let bounce = area.map(|area| {
            let tint = |p: Pos| {
                let rgb = terrain(p).map(|t| t.bounce);
                rgb.map_or([0; 3], |c| [c.r, c.g, c.b])
            };
            let masks = (near.iter())
                .filter(|(_, mask)| mask.overlaps(area.min, area.max))
                .map(|(_, mask)| mask.cells.as_slice());
            let beams = beams.iter().map(|(_, _, s)| s.cells.as_slice());
            Bounce::compute(area, masks.chain(beams), is_opaque, tint)
        });
        // Steady light on a transparent cell beyond the ambient light: sky, bounce, and a special
        // room's glow (a sanctuary's).
        let indirect = |p: Pos| {
            let bounce = bounce.as_ref().map_or(Light::DARK, |b| b.at(p));
            let bounce = bounce.saturating_add(eye.perceive(level.room_glow(content, p)));
            if !glow.is_dark() && exposed(p) {
                bounce.saturating_add(glow)
            } else {
                bounce
            }
        };
        // A shadowed room keeps only part of every bit of a cell's light: its base light and each
        // source's contribution alike, so they still add up to the whole.
        let shade = |p: Pos, light: Light| level.shade(content, p, light, state.time);
        // What is in view and a cell round it (wall completion, wall faces); a write beyond
        // (a creature smelt out of sight) grows the box.
        let (seen_min, seen_max) = Area::around(lit_cells.iter().copied(), 1, width, height)
            .map_or((viewer, viewer), |a| (a.min, a.max));
        let mut base = Window::new(width, height, seen_min, seen_max, Light::DARK);
        for &p in lit_cells {
            base[p] = shade(p, ambient.saturating_add(indirect(p)));
        }

        // Light on cells in line of sight: base light plus every source reaching them, the sun
        // and the moons last.
        let mut light = base.clone();
        let mut lit: Vec<_> = near
            .into_iter()
            .map(|(source, mask)| {
                let emitter = Emitter::Point {
                    pos: source.pos,
                    preset: source.preset,
                };
                let cells = lit_in_sight(
                    &mask.cells,
                    |q| lit_los.get(q) == Some(&true),
                    |q, l| shade(q, eye.perceive(l)),
                );
                (source.key, emitter, cells)
            })
            .collect();
        for (key, emitter, beam) in &beams {
            let cells = lit_in_sight(
                &beam.cells,
                |q| lit_los.get(q) == Some(&true),
                |q, l| shade(q, eye.perceive(l)),
            );
            lit.push((*key, *emitter, cells));
        }
        for (_, _, cells) in &lit {
            for &(p, c) in cells {
                light[p] = light[p].saturating_add(c);
            }
        }

        // Light-blind eyes in glare: strong light on the viewer's own cell dazzles them, and
        // they see nothing farther than their race says.
        let dazzled = (state.race_of(content, state.player()))
            .and_then(|race| race.light_blind)
            .filter(|blind| blind.dazzled_by(light[viewer]));
        let reach2 = radius2(dazzled.map_or(range, |blind| range.min(blind.sight)));
        let sees =
            |light: Light, dist2: i32| dist2 <= reach2 && (overworld || eye.sees(light, dist2));

        // Transparent cells: visible with enough light or within darkvision; the viewer's own
        // cell always.
        let mut sight = Window::new(width, height, seen_min, seen_max, Sight::Hidden);
        let mut visible_cells = Vec::new();
        let mut seen = Vec::new();
        let smoke_hides = |p: Pos| level.smoke_hides(viewer, p, state.time);
        for &p in los_cells.iter().filter(|p| !is_opaque(**p)) {
            let by_eye = sees(light[p], p.dist2(viewer)) && !smoke_hides(p);
            if by_eye || p == viewer {
                sight[p] = Sight::Seen;
                visible_cells.push(p);
            }
            if by_eye {
                seen.push(p);
            }
        }
        // Blood scent (spec §8 "Talents"): creatures suffering a status, in line of sight within
        // its radius, however dark their cells.
        let mut scented = Vec::new();
        if !overworld && !talents.scent.is_empty() {
            // A full moon lengthens a gift's (Hound's nose; spec §8 "Birth sign").
            let moon = if talents.gift_reach.scent.is_empty() {
                0
            } else {
                content.full_moon_reach(state.time)
            };
            let reach = |status: StatusId, radius: i32| {
                radius
                    + if talents.gift_reach.scent.contains(&status) {
                        moon
                    } else {
                        0
                    }
            };
            let smelt = (state.actors.values())
                .filter(|a| a.level == level.id && a.kind == ActorKind::Creature)
                .filter(|a| los.get(a.pos) == Some(&true) && !is_opaque(a.pos))
                .filter(|a| {
                    (talents.scent.iter()).any(|&(status, radius)| {
                        a.effects.has(status)
                            && a.pos.dist2(viewer) <= radius2(reach(status, radius))
                    })
                });
            for a in smelt {
                scented.push(a.id);
                if sight[a.pos] == Sight::Hidden {
                    sight[a.pos] = Sight::Seen;
                    visible_cells.push(a.pos);
                }
            }
        }

        // The beast's viewpoint: what it sees by its eyes, and the hostiles its nose finds.
        let mut bond_seen = Vec::new();
        if let Some(bond) = &bond {
            for p in bond.los.cells().filter(|p| !is_opaque(*p)) {
                if sight.get(p) != Some(&Sight::Hidden) {
                    continue;
                }
                if sight::sees_by(state, content, lights, &bond.los, &bond.eye, p).is_some() {
                    sight[p] = Sight::Seen;
                    visible_cells.push(p);
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
                if sight.get(p) == Some(&Sight::Hidden) {
                    sight[p] = Sight::Seen;
                    visible_cells.push(p);
                }
            }
        }

        // Wall completion: opaque cells bounding a transparent cell seen by eye (the companion's
        // too, within its own sight).
        for q in seen {
            for w in Dir::ALL.map(|d| q.step(d)) {
                if sight.get(w) == Some(&Sight::Hidden) && is_opaque(w) && w.dist2(viewer) <= reach2
                {
                    sight[w] = if los[w] { Sight::Seen } else { Sight::Edge };
                    visible_cells.push(w);
                }
            }
        }
        for q in bond_seen {
            for w in Dir::ALL.map(|d| q.step(d)) {
                if sight.get(w) == Some(&Sight::Hidden) && is_opaque(w) {
                    sight[w] = if lit_los[w] { Sight::Seen } else { Sight::Edge };
                    visible_cells.push(w);
                }
            }
        }

        // Wall faces: opaque cells in line of sight or completed take the brightest sky and bounce
        // light of their visible transparent neighbors, then each source's light at their own
        // distance (the sun's or a moon's where it reaches them) if they face a visible cell it
        // lights.
        let is_seen = |q: Pos| sight.get(q) == Some(&Sight::Seen);
        let is_face = |w: Pos| {
            is_opaque(w) && (lit_los.get(w) == Some(&true) || sight.get(w) == Some(&Sight::Edge))
        };
        for &w in lit_cells.iter().chain(&visible_cells) {
            if is_face(w) {
                let from_neighbors = Dir::ALL
                    .map(|d| w.step(d))
                    .into_iter()
                    .filter(|q| is_seen(*q) && !is_opaque(*q))
                    .fold(Light::DARK, |acc, q| acc.max(indirect(q)));
                base[w] = shade(w, ambient.saturating_add(from_neighbors));
                light[w] = base[w];
            }
        }
        let light_at = |key: u64, emitter: Emitter, w: Pos| {
            let light = match emitter {
                Emitter::Point { pos, preset } => {
                    let preset = content.light(preset);
                    falloff(preset.color, preset.radius.max(0), w.dist2(pos))
                }
                Emitter::Sun(_) | Emitter::Moon(_) => (beams.iter())
                    .find(|(k, _, _)| *k == key)
                    .map_or(Light::DARK, |(_, _, beam)| beam.at(w)),
            };
            shade(w, eye.perceive(light))
        };
        let lit: Vec<_> = lit
            .into_iter()
            .map(|(key, emitter, cells)| {
                let faces = wall_faces(&cells, |w| light_at(key, emitter, w), is_seen, is_face);
                (key, emitter, cells, faces)
            })
            .collect();
        for (_, _, _, faces) in &lit {
            for &(p, c) in faces {
                light[p] = light[p].saturating_add(c);
            }
        }

        // Other opaque cells in line of sight: visible by their face light or darkvision.
        for &p in &los_cells {
            if sight[p] == Sight::Hidden && is_opaque(p) && sees(light[p], p.dist2(viewer)) {
                sight[p] = Sight::Seen;
                visible_cells.push(p);
            }
        }

        let is_visible = |p: Pos| sight.get(p).is_some_and(|s| *s != Sight::Hidden);
        let sources = lit
            .into_iter()
            .filter_map(|(key, emitter, cells, faces)| {
                let contributions: Vec<_> = cells
                    .into_iter()
                    .chain(faces)
                    .filter(|(p, _)| is_visible(*p))
                    .collect();
                (!contributions.is_empty()).then_some(SeenLight {
                    key,
                    emitter,
                    contributions,
                })
            })
            .collect();

        // The unseen (`world/unseen.rs`): the shadowy on cells dim to the player's eyes, and the
        // invisible.
        let dim = |p: Pos| !overworld && light.get(p).is_some_and(|l| eye.dim(*l));
        let concealed = world::unseen_by_player(state, content, &dim, &scented);
        let draught =
            (concealed.iter()).any(|c| c.shadowy && c.pos.chebyshev(viewer) <= world::DRAUGHT);
        let unseen = |a: &Actor| concealed.iter().any(|c| c.id == a.id);

        // A disguised mimic is a shut chest (`world/mimic.rs`), but to Chest-wise within its
        // reach where the player sees it.
        let reach = talents.sees_mimics;
        let shut = content.chest_terrain(crate::chest::ChestState::Shut);
        let zones: BTreeMap<Pos, ZoneKind> = if level.zones.is_empty() {
            BTreeMap::new()
        } else {
            // Smoke is seen where sight stops at it, though what is in it is not.
            let smoke = (los_cells.iter()).filter(|&&p| level.smoky(p, state.time));
            (visible_cells.iter().chain(smoke))
                .filter_map(|&p| Some((p, level.zone_at(p, state.time)?.kind())))
                .collect()
        };
        let disguises: Vec<Disguise> = (state.actors.values())
            .filter(|a| a.level == level.id && a.effects.hides(content))
            .filter(|a| {
                !(sight.get(a.pos) == Some(&Sight::Seen) && a.pos.chebyshev(viewer) <= reach)
            })
            .filter_map(|a| {
                Some(Disguise {
                    pos: a.pos,
                    terrain: shut?,
                    level: content.creature(a.creature).level,
                })
            })
            .collect();
        let hidden = |a: &Actor| disguises.iter().any(|d| d.pos == a.pos);
        let seen: Vec<&Actor> = state
            .actors
            .values()
            .filter(|a| a.level == level.id && sight.get(a.pos) == Some(&Sight::Seen))
            // The retinue is with the player, not on the map (`world/ally.rs`).
            .filter(|a| !a.in_retinue())
            .filter(|a| !hidden(a) && !unseen(a))
            .collect();
        // Averted eyes (`world/avert.rs`): a hostile whose gaze is marked `averted`, seen.
        let looker = state.player().faction(content);
        let averted = (seen.iter()).any(|a| {
            a.id != state.player
                && (content.creature(a.creature).gaze.as_ref()).is_some_and(|g| g.averted)
                && content.stance_of(a.faction(content), looker) == Stance::Hostile
        });
        let mut actors: Vec<SeenActor> = (seen.iter())
            .map(|a| SeenActor {
                id: a.id,
                kind: a.kind,
                creature: a.creature,
                dialogue: content.dialogue_of(a.creature, a.npc.as_deref()),
                faction: a.faction(content),
                pos: a.pos,
                health: a.health(content),
                shape: shape(a, content),
                unaware: false,
                marked: a.id != state.player && state.player().mark == Some(a.id),
                ally: a.master() == Some(state.player),
                companion: a.master() == Some(state.player) && a.kept().is_some(),
                grade: a.graded(),
            })
            .collect();
        // Stalker's eye: which of them have not noticed the player.
        if talents.sees_unaware {
            for seen in &mut actors {
                seen.unaware = (seen.kind == ActorKind::Creature)
                    && creature_unaware(state, content, lights, seen.id, state.player);
            }
        }
        // A hall of mirrors' images of those seen (`world/mirrors.rs`), where the player sees
        // their cells, each shown as what it reflects under an id of its own.
        let images: Vec<(ActorId, ActorId, Pos)> = (state.images(content, level).into_iter())
            .filter(|i| sight.get(i.pos) == Some(&Sight::Seen))
            .filter(|i| seen.iter().any(|a| a.id == i.of))
            .map(|i| (i.of, image_id(i.of, i.index, state.turn), i.pos))
            .collect();
        for &(of, id, pos) in &images {
            if let Some(real) = actors.iter().find(|a| a.id == of).copied() {
                actors.push(SeenActor { id, pos, ..real });
            }
        }
        let shown = |a: &Actor| {
            std::iter::once(a.id)
                .chain(images.iter().filter(|i| i.0 == a.id).map(|i| i.1))
                .collect::<Vec<_>>()
        };
        // What creatures wear and wield is in plain sight; the rest of what they carry is not.
        let gear = (seen.iter())
            .filter(|a| a.kind == ActorKind::Creature)
            .flat_map(|a| {
                shown(a).into_iter().flat_map(move |id| {
                    (a.equipment.items().into_iter()).filter_map(move |(slot, item)| {
                        let stack = a.inventory.get(item)?.clone();
                        Some(SeenGear {
                            actor: id,
                            slot,
                            stack,
                        })
                    })
                })
            })
            .collect();
        // So do the statuses that show, such as bleeding; others (poison) do not, nor those its
        // kind always has (`world/aura.rs`: a salamander is always cloaked in flame).
        let effects = (seen.iter())
            .filter(|a| a.kind == ActorKind::Creature)
            .flat_map(|a| {
                let innate = &content.creature(a.creature).innate;
                shown(a).into_iter().flat_map(move |id| {
                    (a.effects.iter())
                        .filter(|e| content.status(e.status).seen.is_some())
                        .filter(|e| !innate.contains(&e.status))
                        .map(move |e| (id, e.status))
                })
            })
            .collect();
        actors.sort_by_key(|a| a.id);
        let images = images.into_iter().map(|(_, _, p)| p).collect();

        // Blood and char: on an open cell in sight; on a wall, where a stained face is seen.
        let stained = |stains: &BTreeMap<Pos, Stain>| -> Vec<Pos> {
            (stains.iter())
                .filter(|(p, stain)| {
                    let p = **p;
                    if !is_opaque(p) {
                        return sight.get(p) == Some(&Sight::Seen);
                    }
                    let faces = |d: Dir| stain.faces == 0 || stain.on_face(d);
                    sight.get(p).is_some_and(|s| *s != Sight::Hidden)
                        && Dir::ALL.into_iter().filter(|d| faces(*d)).any(|d| {
                            let q = p.step(d);
                            sight.get(q) == Some(&Sight::Seen) && !is_opaque(q)
                        })
                })
                .map(|(p, _)| *p)
                .collect()
        };
        let blood = stained(&level.blood);
        let scorch = stained(&level.scorch);

        // Sounds the player heard whose source they do not see, placed roughly.
        let player = state.player();
        let sounds = match player.hearing(content) {
            Some(threshold) => {
                let hidden = |p: Pos| {
                    p != viewer
                        && (sight.get(p) == Some(&Sight::Hidden)
                            || concealed.iter().any(|c| c.pos == p))
                };
                // Watchful (spec §8 "Birth sign"): footsteps placed exactly; the ranger's
                // Tracker: every sound.
                let exact = match (talents.exact_sounds, talents.exact_footsteps) {
                    (true, _) => noise::Exact::All,
                    (false, true) => noise::Exact::Footsteps,
                    (false, false) => noise::Exact::No,
                };
                let heard = (&player.heard[..], threshold, exact);
                noise::perceive(level, content, heard, state.turn, hidden)
            }
            None => Vec::new(),
        };
        // The player feels the air where they stand: fixed underground, the weather's under the
        // open sky, and what a hot or a cold room adds.
        let temperature = state.temperature_at(level, content, state.time, player.pos);
        Self {
            viewer,
            eye: eyes,
            ambient,
            sky,
            temperature,
            base,
            sight,
            visible_cells,
            light,
            sources,
            actors,
            gear,
            effects,
            sounds,
            blood,
            scorch,
            images,
            dazzled: dazzled.is_some(),
            averted,
            disguises,
            concealed,
            draught,
            zones,
            allies: allies_of(state, content),
            ally_ailments: crate::heal::of_allies(&crate::world::allies_beside(state), content),
            kept_beside: (crate::world::allies_beside(state).into_iter())
                .filter(|(_, a)| a.kept().is_some() || a.escorting().is_some())
                .map(|(d, _)| d)
                .collect(),
            escort_beside: (crate::world::allies_beside(state).into_iter())
                .any(|(_, a)| a.escorting().is_some()),
        }
    }

    /// The disguised mimic at `p` as the snapshot hides it, if one stands there.
    fn disguise_at(&self, p: Pos) -> Option<&Disguise> {
        self.disguises.iter().find(|d| d.pos == p)
    }

    /// Whether the player sees the cell `p` itself (in line of sight, not a completed wall),
    /// whatever stands there.
    pub(crate) fn in_sight(&self, p: Pos) -> bool {
        self.sight.get(p) == Some(&Sight::Seen)
    }

    /// Whether the player sees an image of a hall of mirrors at `p` (`world/mirrors.rs`): a
    /// step or a blow there meets only glass.
    pub(crate) fn shows_image_at(&self, p: Pos) -> bool {
        self.images.contains(&p)
    }

    /// Whether strong light dazzles the player's light-blind eyes.
    pub(crate) fn dazzled(&self) -> bool {
        self.dazzled
    }

    /// Whether the player averts their eyes from a gaze marked `averted` (`world/avert.rs`).
    pub(crate) fn averted(&self) -> bool {
        self.averted
    }

    /// Sounds heard during the last turn from sources the player does not see.
    pub(crate) fn sounds(&self) -> &[PerceivedSound] {
        &self.sounds
    }

    /// The weather, under the open sky.
    pub(crate) fn weather(&self) -> Option<Weather> {
        self.sky.as_ref().map(|s| s.conditions.weather)
    }

    /// Whether the cell's terrain is visible (in sight, or a completed wall).
    pub(crate) fn is_visible(&self, p: Pos) -> bool {
        self.sight.get(p).is_some_and(|s| *s != Sight::Hidden)
    }

    /// Whether the player does not see `p`, a cell of the level that is not their own: where
    /// they may place a sound they hear ([`noise::rough_place`]). The cell of one unseen
    /// (`world/unseen.rs`) is such a cell.
    pub(crate) fn hides(&self, p: Pos) -> bool {
        p != self.viewer
            && (self.sight.get(p) == Some(&Sight::Hidden) || self.concealed_at(p).is_some())
    }

    /// The one the player does not see for being unseen at `p`, if any.
    fn concealed_at(&self, p: Pos) -> Option<&crate::world::Concealed> {
        self.concealed.iter().find(|c| c.pos == p)
    }

    /// Whether actor `id` is one the player does not see for being unseen (`world/unseen.rs`).
    pub(crate) fn conceals(&self, id: ActorId) -> bool {
        self.concealed.iter().any(|c| c.id == id)
    }

    /// Whether an unseen shadowy creature is near enough for the player to feel its draught.
    pub(crate) fn draught(&self) -> bool {
        self.draught
    }

    /// Whether actor `id` is among those the player sees (a disguised mimic is not, unless
    /// Chest-wise sees through it).
    pub(crate) fn shows(&self, id: ActorId) -> bool {
        self.actors.iter().any(|a| a.id == id)
    }

    /// Whether creatures at `p` can be seen: only in line of sight, never on completed walls.
    /// A disguised mimic's cell shows a chest, and no one in it.
    pub(crate) fn sees_actors_at(&self, p: Pos) -> bool {
        self.sight.get(p) == Some(&Sight::Seen)
            && self.disguise_at(p).is_none()
            && self.concealed_at(p).is_none()
    }

    /// Whether the player sees the blood at `p` (module docs).
    pub(crate) fn sees_blood_at(&self, p: Pos) -> bool {
        self.blood.binary_search(&p).is_ok()
    }

    /// Whether the player sees the char at `p` (as blood).
    pub(crate) fn sees_scorch_at(&self, p: Pos) -> bool {
        self.scorch.binary_search(&p).is_ok()
    }

    /// Writes currently visible terrain, the piles and known traps on cells in sight, and the
    /// blood seen into the level's memory. Blood on a wall seen only from its clean side is
    /// remembered as it was.
    pub(crate) fn remember(&self, level: &mut LevelState) {
        for &p in &self.blood {
            if let Some(stain) = level.blood.get(&p)
                && level.blood_memory.get(&p) != Some(stain)
            {
                level.blood_memory.insert(p, *stain);
            }
        }
        for &p in &self.scorch {
            if let Some(stain) = level.scorch.get(&p)
                && level.scorch_memory.get(&p) != Some(stain)
            {
                level.scorch_memory.insert(p, *stain);
            }
        }
        // The zones seen; with none seen and none remembered, nothing to look at.
        if !self.zones.is_empty() || !level.zone_memory.is_empty() {
            for &p in &self.visible_cells {
                match self.zones.get(&p) {
                    Some(&kind) => {
                        level.zone_memory.insert(p, kind);
                    }
                    None => {
                        level.zone_memory.remove(&p);
                    }
                }
            }
            // Smoke seen from outside it, on cells not seen.
            for (&p, &kind) in &self.zones {
                level.zone_memory.insert(p, kind);
            }
        }
        for &p in &self.visible_cells {
            let terrain = self.disguise_at(p).map_or(level.terrain[p], |d| d.terrain);
            level.memory.set(p, Some(terrain));
            if !level.blood.contains_key(&p) {
                level.blood_memory.remove(&p);
            }
            if !level.scorch.contains_key(&p) {
                level.scorch_memory.remove(&p);
            }
            if !self.sees_actors_at(p) {
                continue;
            }
            match level.traps.get(&p).filter(|t| t.known) {
                Some(trap) => {
                    level.trap_memory.insert(p, trap.kind);
                }
                None => {
                    level.trap_memory.remove(&p);
                }
            }
            match level.objects.get(&p) {
                Some(pile) if level.object_memory.get(&p) != Some(pile) => {
                    level.object_memory.insert(p, pile.clone());
                }
                Some(_) => {}
                None => {
                    level.object_memory.remove(&p);
                }
            }
        }
    }
}
