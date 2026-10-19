//! The per-turn perception snapshot (spec §5, §8).
//!
//! Everything the player can know — and therefore everything rendering, UI, and dumps may show —
//! goes through [`PerceptionView`]. It exposes currently visible cells, creatures (what they
//! wear and wield, and how hurt they look, in words), objects, and light, remembered terrain and
//! objects, and what the player carries, wears, weighs, and scores in each stat, their trained
//! skills' values, their burden and carrying limit, and their hit points and Defence; it never
//! exposes unseen terrain or objects, hidden creatures, what else creatures carry, their burden,
//! their skills, or their hit points, any stat's modifier or potential, a skill's marks or
//! benefits, an untrained skill, a creature's stats, a to-hit, or light the player's eyes cannot
//! perceive.
//!
//! **Memory** (spec §5): terrain and piles of objects as last seen. Objects are seen where
//! creatures are (cells in line of sight, never completed walls); a pile seen gone is forgotten,
//! and one that changed out of sight is remembered as it was.
//!
//! A cell is **visible** iff it is in line of sight within [`VIEW_RADIUS`] **and** its perceived
//! light is at least the vision profile's threshold or it lies within the darkvision radius. The
//! player's own cell is always visible.
//!
//! The overworld is seen at map scale instead: a cell is visible iff it is in line of sight
//! within the daylight sight radius ([`daylight::overworld_sight`]: day 9, twilight 5, night 2,
//! more under a bright moon, less in rain), whatever its light; eyes and darkvision do not change
//! that.
//!
//! **Weather** ([`crate::weather`]) is known only under the open sky: on such a level the sky
//! carries it ([`PerceptionView::weather`]), and its clouds dim the sun, the moons, and the sky.
//!
//! Light on a transparent cell is the level's ambient light, plus the light of every source that
//! reaches it, plus weak **bounce light** reflected by the lit surfaces around it
//! ([`crate::light::BOUNCE`]). On a level under the open sky ([`Sky`]) the **sun** by day and
//! the **moons** by night are more sources (directional, shaded by tall terrain; see [`Sun`] and
//! [`Moon`]), and the **sky** lights every cell that is not roofed. Ambient plus sky plus bounce
//! is the cell's *base light*: steady, and not tied to one source.
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

use crate::burden::{self, Load};
use crate::character::Character;
use crate::combat::Health;
use crate::content::{
    Content, CreatureId, FactionId, Flow, LevelKind, LightId, ObjectId, RoomEffectDef, SlotId,
    Stance, TerrainId, VisionProfile,
};
use crate::daylight;
use crate::equipment::Equipment;
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::light::{
    Area, BOUNCE_STEPS, Bounce, Light, LightCache, Moon, SUN_KEY, ShadowSteps, Sky, Sun, Sunlight,
    falloff, lit_in_sight, moon_key, wall_faces,
};
use crate::noise::{self, PerceivedSound};
use crate::object::{Stack, Weight};
use crate::skills::{self, SkillId};
use crate::stats::PerStat;
use crate::time::GameTime;
use crate::weather::Weather;
use crate::world::{Actor, ActorId, ActorKind, Death, LevelState, WorldState};
use serde::Serialize;

/// Maximum sight distance, however bright the light.
pub const VIEW_RADIUS: i32 = 40;

/// One of the player's own trained skills, as they know it (spec §8 "Skills"): its value, a
/// coarse sense of how near its next point is, and for the abilities menu (#156) whether it has
/// an active use and whether that is available now. Never its marks or what it gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct OwnSkill {
    pub skill: SkillId,
    pub value: u16,
    /// Toward the next point, in quarters: 0 (just gained one) to 3 (nearly there).
    pub progress: u8,
    /// Whether it has an active use: it is used as an action (`kind = "action"` in data), which
    /// the abilities menu lists.
    pub active: bool,
    /// Whether its active use can be used now. No active use exists yet (they come with their
    /// actions), so this is always false for now.
    pub available: bool,
}

/// What the player knows about one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "terrain")]
pub enum CellView {
    /// Never seen.
    Unknown,
    /// Seen before; terrain as it was last seen. Never includes creatures or effects.
    Remembered(TerrainId),
    /// In view right now.
    Visible(TerrainId),
}

/// A creature the player currently sees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SeenActor {
    pub id: ActorId,
    pub kind: ActorKind,
    pub creature: CreatureId,
    /// The faction it fights for (its type's, or the one its level placed it in).
    #[serde(skip)]
    pub faction: Option<FactionId>,
    pub pos: Pos,
    /// How hurt it looks: words, never hit points (spec §8 "Combat").
    pub health: Health,
}

/// Something a creature the player sees wears or wields: what is seen of what it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeenGear {
    pub actor: ActorId,
    /// The first slot it is in (a two-handed weapon is in both hands).
    pub slot: SlotId,
    pub stack: Stack,
}

/// Light as the player perceives it at a visible cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PerceivedLight {
    /// Red, green, blue: 255 = fully lit, more = overbright.
    pub rgb: [u16; 3],
    /// Ultraviolet; always 0 unless the vision profile sees UV.
    pub uv: u16,
    /// Seen only thanks to darkvision: too dark for the eye's threshold.
    pub darkvision: bool,
    /// Squared distance from the viewer.
    pub dist2: i32,
}

impl PerceivedLight {
    /// The light value without the viewer-relative fields.
    pub fn light(&self) -> Light {
        Light {
            rgb: self.rgb,
            uv: self.uv,
        }
    }

    /// Brightness compared against the vision threshold (luma, plus UV if perceived).
    pub fn brightness(&self) -> u16 {
        self.light().brightness()
    }
}

/// Where a perceived light comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Emitter {
    /// A light on a cell: placed by the level or emitted by terrain, carried by a creature, or the
    /// player's lantern.
    Point {
        /// Where the source is; it may itself be out of view (light from around a corner).
        pos: Pos,
        /// The source's preset (color, radius, flicker).
        preset: LightId,
    },
    /// The sun over a level under the open sky: directional (its shadows fall along
    /// [`Sun::shadow_steps`]), on no cell, and steady.
    Sun(Sun),
    /// A moon over a level under the open sky: directional like the sun, and steady.
    Moon(Moon),
}

impl Emitter {
    /// For the sun and the moons, the cells a caster `height` cells tall shades from them, as
    /// offsets from it ([`Sun::shadow_steps`]); `None` for point lights.
    pub fn shadow_steps(&self, height: u8) -> Option<ShadowSteps> {
        match self {
            Emitter::Point { .. } => None,
            Emitter::Sun(sun) => Some(sun.shadow_steps(height)),
            Emitter::Moon(moon) => Some(moon.shadow_steps(height)),
        }
    }
}

/// A light source whose light reaches cells the player sees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SeenLight {
    /// Stable across turns (derived from level and position for static lights, from the actor for
    /// carried ones; one fixed key for the sun and one for each moon), e.g. to seed visual
    /// flicker.
    pub key: u64,
    /// What gives the light.
    pub emitter: Emitter,
    /// Perceived light this source adds to each visible cell it reaches, wall-face rule applied.
    /// Summed over all sources plus [`PerceptionView::base_light`] this gives
    /// [`PerceptionView::light`].
    pub contributions: Vec<(Pos, Light)>,
}

impl SeenLight {
    /// Where a point light is; `None` for the sun and the moons.
    pub fn pos(&self) -> Option<Pos> {
        match self.emitter {
            Emitter::Point { pos, .. } => Some(pos),
            Emitter::Sun(_) | Emitter::Moon(_) => None,
        }
    }

    /// A point light's preset; `None` for the sun and the moons.
    pub fn preset(&self) -> Option<LightId> {
        match self.emitter {
            Emitter::Point { preset, .. } => Some(preset),
            Emitter::Sun(_) | Emitter::Moon(_) => None,
        }
    }
}

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
    /// Index into [`Content::vision_profiles`].
    vision: usize,
    /// Perceived ambient light.
    ambient: Light,
    /// The sun, the moons, and the sky over a level under the open sky.
    sky: Option<Sky>,
    /// Ambient plus sky plus bounce light; meaningful on visible cells only.
    base: Grid<Light>,
    sight: Grid<Sight>,
    /// Visible cells in the order they were found.
    visible_cells: Vec<Pos>,
    /// Perceived light; meaningful on visible cells only.
    light: Grid<Light>,
    sources: Vec<SeenLight>,
    actors: Vec<SeenActor>,
    /// What the seen creatures (not the player) wear and wield, by actor, in slot order.
    gear: Vec<SeenGear>,
    /// Sounds heard during the last turn from sources the player does not see.
    sounds: Vec<PerceivedSound>,
}

impl Perception {
    /// Computes what the player perceives on their current level: line of sight, light, and
    /// visibility. Light masks come from (and are added to) `lights`.
    pub(crate) fn compute(state: &WorldState, content: &Content, lights: &mut LightCache) -> Self {
        let viewer = state.player().pos;
        let level = state.current();
        let (width, height) = (level.width(), level.height());
        let vision = content.vision_index(&state.vision).unwrap_or(0);
        let eye = &content.vision_profiles()[vision];
        let is_opaque = |p: Pos| level.is_opaque(content, p);
        let sky = state.sky(content);
        // On the overworld, daylight (and moonlight, and rain) sets how far one sees; light does
        // not matter.
        let overworld = level.kind == LevelKind::Overworld;
        let range = match &sky {
            Some(sky) if overworld => daylight::sight_under(state.time, sky, content.climate()),
            _ => VIEW_RADIUS,
        };
        let sees = |light: Light, dist2: i32| overworld || eye.sees(light, dist2);

        // Line of sight.
        let mut los = Grid::new(width, height, false);
        let mut los_cells = Vec::new();
        compute_fov(viewer, range, is_opaque, |p| {
            if let Some(seen) = los.get_mut(p)
                && !*seen
            {
                *seen = true;
                los_cells.push(p);
            }
        });

        // Sources that can light a cell in view, or a surface whose bounce light reaches one (a
        // bounce step moves at most √2 < 2 cells; wall faces are within a cell of view).
        let near = lights.near(state, content, viewer, VIEW_RADIUS + 1 + 2 * BOUNCE_STEPS);
        // Direct light is needed on everything in sight and within bounce reach of it.
        let area = Area::around(los_cells.iter().copied(), BOUNCE_STEPS, width, height);

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
        let shade = |p: Pos, light: Light| level.room_shade(content, p, light);
        let mut base = Grid::new(width, height, Light::DARK);
        for &p in &los_cells {
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
                let cells = lit_in_sight(&mask.cells, &los, |q, l| shade(q, eye.perceive(l)));
                (source.key, emitter, cells)
            })
            .collect();
        for (key, emitter, beam) in &beams {
            let cells = lit_in_sight(&beam.cells, &los, |q, l| shade(q, eye.perceive(l)));
            lit.push((*key, *emitter, cells));
        }
        for (_, _, cells) in &lit {
            for &(p, c) in cells {
                light[p] = light[p].saturating_add(c);
            }
        }

        // Transparent cells: visible with enough light or within darkvision; the viewer's own
        // cell always.
        let mut sight = Grid::new(width, height, Sight::Hidden);
        let mut visible_cells = Vec::new();
        let mut seen = Vec::new();
        for &p in los_cells.iter().filter(|p| !is_opaque(**p)) {
            let by_eye = sees(light[p], p.dist2(viewer));
            if by_eye || p == viewer {
                sight[p] = Sight::Seen;
                visible_cells.push(p);
            }
            if by_eye {
                seen.push(p);
            }
        }

        // Wall completion: opaque cells bounding a transparent cell seen by eye.
        let reach2 = radius2(range);
        for q in seen {
            for w in Dir::ALL.map(|d| q.step(d)) {
                if sight.get(w) == Some(&Sight::Hidden) && is_opaque(w) && w.dist2(viewer) <= reach2
                {
                    sight[w] = if los[w] { Sight::Seen } else { Sight::Edge };
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
            is_opaque(w) && (los.get(w) == Some(&true) || sight.get(w) == Some(&Sight::Edge))
        };
        for &w in los_cells.iter().chain(&visible_cells) {
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

        let seen: Vec<&Actor> = state
            .actors
            .values()
            .filter(|a| a.level == level.id && sight.get(a.pos) == Some(&Sight::Seen))
            .collect();
        let actors = (seen.iter())
            .map(|a| SeenActor {
                id: a.id,
                kind: a.kind,
                creature: a.creature,
                faction: a.faction(content),
                pos: a.pos,
                health: a.health(content),
            })
            .collect();
        // What creatures wear and wield is in plain sight; the rest of what they carry is not.
        let gear = (seen.iter())
            .filter(|a| a.kind == ActorKind::Creature)
            .flat_map(|a| {
                (a.equipment.items().into_iter()).filter_map(|(slot, item)| {
                    let stack = a.inventory.get(item)?.clone();
                    Some(SeenGear {
                        actor: a.id,
                        slot,
                        stack,
                    })
                })
            })
            .collect();

        // Sounds the player heard whose source they do not see, placed roughly.
        let player = state.player();
        let sounds = match player.hearing(content) {
            Some(threshold) => {
                let hidden = |p: Pos| sight.get(p) == Some(&Sight::Hidden) && p != viewer;
                noise::perceive(level, content, &player.heard, threshold, state.turn, hidden)
            }
            None => Vec::new(),
        };
        Self {
            viewer,
            vision,
            ambient,
            sky,
            base,
            sight,
            visible_cells,
            light,
            sources,
            actors,
            gear,
            sounds,
        }
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

    /// Whether creatures at `p` can be seen: only in line of sight, never on completed walls.
    pub(crate) fn sees_actors_at(&self, p: Pos) -> bool {
        self.sight.get(p) == Some(&Sight::Seen)
    }

    /// Writes currently visible terrain, and the piles on cells in sight, into the level's
    /// memory.
    pub(crate) fn remember(&self, level: &mut LevelState) {
        for &p in &self.visible_cells {
            level.memory.set(p, Some(level.terrain[p]));
            if !self.sees_actors_at(p) {
                continue;
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

/// The player's experience (spec §8 "Experience"), all of which they are shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub level: u8,
    /// The experience total.
    pub xp: u32,
    /// The total at which the next level comes; none at the highest level.
    pub next: Option<u32>,
}

/// Read-only view of what the player perceives. Obtain one with [`crate::World::perception`].
#[derive(Clone, Copy, Debug)]
pub struct PerceptionView<'a> {
    pub(crate) content: &'a Content,
    pub(crate) level: &'a LevelState,
    /// The player, for what they carry.
    pub(crate) player: &'a Actor,
    /// The player's character, which they know.
    pub(crate) character: Option<&'a Character>,
    /// The time on the clock, which the player knows.
    pub(crate) time: GameTime,
    /// How the run ended, once the player has died.
    pub(crate) death: Option<&'a Death>,
    /// The player's experience total.
    pub(crate) experience: u32,
    pub(crate) snapshot: &'a Perception,
}

impl<'a> PerceptionView<'a> {
    pub fn content(&self) -> &'a Content {
        self.content
    }

    pub fn level_id(&self) -> &'a str {
        &self.level.id
    }

    pub fn level_name(&self) -> &'a str {
        &self.level.name
    }

    pub fn level_kind(&self) -> LevelKind {
        self.level.kind
    }

    /// The named region the player is in: on a level with regions (the overworld), the one
    /// around the viewer; in a local area, the one around its overworld cell.
    pub fn region(&self) -> Option<&'a str> {
        let (level, at) = match &self.level.exit {
            Some(exit) => (exit.to.as_str(), exit.pos),
            None => (self.level.id.as_str(), self.snapshot.viewer),
        };
        self.content.level(level)?.region_at(at)
    }

    /// The special room at `p` (spec §7 "Special rooms"), if the player knows it: they have been
    /// inside it, and `p` is a cell they see or remember.
    pub fn room(&self, p: Pos) -> Option<&'a RoomEffectDef> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let room = self.level.room_at(p).filter(|r| r.entered)?;
        Some(self.content.room_effect(room.effect))
    }

    /// The percent of all light a special room keeps at a visible cell (a shadowed room's
    /// darkness, which the player sees; 100 elsewhere), for light that presentation adds of its
    /// own (a burning missile in flight); `None` for cells not in view.
    pub fn light_kept(&self, p: Pos) -> Option<u16> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        let room = self.level.room_effect_at(self.content, p);
        Some(room.map_or(100, |effect| effect.light))
    }

    /// Where the stairs or location at `p` lead, by name — only for cells the player sees or
    /// remembers (the staircase itself is known; its destination is public knowledge).
    pub fn leads_to(&self, p: Pos) -> Option<String> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let to = self.level.stairs_at(p)?.to.as_deref()?;
        self.content.level_display_name(to)
    }

    pub fn width(&self) -> i32 {
        self.level.width()
    }

    pub fn height(&self) -> i32 {
        self.level.height()
    }

    /// Where the player is looking from.
    pub fn viewer(&self) -> Pos {
        self.snapshot.viewer
    }

    pub fn cell(&self, p: Pos) -> CellView {
        if self.snapshot.is_visible(p) {
            CellView::Visible(self.level.terrain[p])
        } else {
            match self.level.memory.get(p) {
                Some(Some(t)) => CellView::Remembered(*t),
                _ => CellView::Unknown,
            }
        }
    }

    pub fn is_visible(&self, p: Pos) -> bool {
        self.snapshot.is_visible(p)
    }

    /// The flow of running water at a visible cell (presentation-only); `None` for cells that are
    /// not in view or not running water.
    pub fn flow(&self, p: Pos) -> Option<Flow> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        self.level.flow.get(&p).copied()
    }

    /// Creatures in view, in ascending id order (the player included).
    pub fn actors(&self) -> &'a [SeenActor] {
        &self.snapshot.actors
    }

    pub fn actor_at(&self, p: Pos) -> Option<&'a SeenActor> {
        self.snapshot.actors.iter().find(|a| a.pos == p)
    }

    /// The pile at `p` as the player knows it, top first: what lies there now on a cell in sight
    /// (where creatures would be seen), as last seen on any other cell; empty where nothing is
    /// known to lie.
    pub fn objects(&self, p: Pos) -> &'a [Stack] {
        let pile = if self.snapshot.sees_actors_at(p) {
            self.level.objects.get(&p)
        } else {
            self.level.object_memory.get(&p)
        };
        pile.map_or(&[], Vec::as_slice)
    }

    /// Whether the pile at `p` is in sight now (rather than remembered or unknown).
    pub fn sees_objects_at(&self, p: Pos) -> bool {
        self.snapshot.sees_actors_at(p)
    }

    /// What the player carries, letter `a` first.
    pub fn inventory(&self) -> &'a [Stack] {
        &self.player.inventory
    }

    /// The player's own weight: their base weight plus what they carry, worn and wielded things
    /// included.
    pub fn own_weight(&self) -> Weight {
        self.player.weight(self.content)
    }

    /// Who the player is (spec §8 "Character creation"): their race, gender, and class, the
    /// stat their gender raised, their birth date, and their birth sign; `None` when the
    /// scenario gave no character.
    pub fn character(&self) -> Option<&'a Character> {
        self.character
    }

    /// The time on the clock.
    pub fn time(&self) -> GameTime {
        self.time
    }

    /// What of their inventory the player wears and wields, by slot.
    pub fn equipment(&self) -> &'a Equipment {
        &self.player.equipment
    }

    /// What the player has ready to shoot (spec §8 "Ranged combat"): their missile weapon, where
    /// in their inventory the ammunition for it is, and how far it shoots; `None` unless they
    /// have both.
    pub fn ready_to_fire(&self) -> Option<(ObjectId, usize, i32)> {
        let (launcher, _) = self.player.launcher(self.content)?;
        let item = self.player.ammunition_item(self.content)?;
        Some((launcher, item, self.player.launcher_range(self.content)?))
    }

    /// Where in their inventory the throwing weapon the player has ready is (a quiver of rocks).
    pub fn ready_to_throw(&self) -> Option<usize> {
        self.player.ready_to_throw(self.content)
    }

    /// How far the player throws inventory stack `item`, if it is a throwing weapon.
    pub fn throw_range(&self, item: usize) -> Option<i32> {
        self.player.throw_range(item, self.content)
    }

    /// What seen creature `actor` wears and wields, each thing once, in slot order: all the
    /// player sees of what a creature carries. Empty for creatures not in view and the player.
    pub fn gear(&self, actor: ActorId) -> impl Iterator<Item = &'a SeenGear> + 'a {
        self.snapshot.gear.iter().filter(move |g| g.actor == actor)
    }

    /// The player's own scores, by stat (spec §8 "Stats"). That is all the player knows of
    /// stats: never their potentials or modifiers, and nothing of a creature's.
    pub fn own_scores(&self) -> PerStat<u8> {
        self.player.scores(self.content).scores()
    }

    /// The player's own trained skills, in skill order (spec §8 "Skills"): each one's value and
    /// how near its next point is, coarsely. Untrained skills are not here; nor is anything of a
    /// creature's skills, a mark, or a benefit.
    pub fn own_skills(&self) -> Vec<OwnSkill> {
        let increase = self.content.increase();
        (self.player.skills.iter())
            .map(|(skill, trained)| OwnSkill {
                skill,
                value: trained.value,
                progress: skills::progress(trained, increase),
                active: self.content.skill(skill).is_active(),
                available: false,
            })
            .collect()
    }

    /// The player's own trained skills that have an active use, in skill order, each with
    /// whether that use is available now: what the abilities menu (#156) lists of skills.
    pub fn active_skills(&self) -> Vec<OwnSkill> {
        self.own_skills().into_iter().filter(|s| s.active).collect()
    }

    /// The player's hit points: what they have left (at most their maximum, never below 0) and
    /// their maximum (spec §8 "Combat").
    pub fn own_hp(&self) -> (i32, i32) {
        let max = self.player.max_hp(self.content);
        (self.player.hp.clamp(0, max), max)
    }

    /// The player's level, experience total, and the total at which the next level comes (none
    /// at the highest level) (spec §8 "Experience"). All of it is shown to the player.
    pub fn own_experience(&self) -> Progress {
        let rules = self.content.experience();
        let level = rules.level_for(self.experience);
        Progress {
            level,
            xp: self.experience,
            next: (level < rules.max_level()).then(|| rules.threshold(level + 1)),
        }
    }

    /// What the player carries against what they can (spec §8 "Burden"): their burden, and the
    /// weights at which it changes, which follow from scores they know.
    pub fn own_load(&self) -> Load {
        let race = self.character.and_then(|c| c.spec.race(self.content).ok());
        burden::load(self.player, race, self.content)
    }

    /// The player's Defence ("Dv"): the total an attack roll must reach (their burden capping
    /// their Dexterity). Only the total: its Dexterity part would give away a modifier.
    pub fn own_defence(&self) -> i32 {
        let burden = self.own_load().penalty();
        self.player.defence(self.content, burden).total()
    }

    /// How the run ended, once the player has died (what killed them only if they saw it).
    pub fn death(&self) -> Option<&'a Death> {
        self.death
    }

    /// How a seen creature stands toward the player: its faction's stance (spec §7 "Factions"),
    /// which the player knows from its type. Nothing else about factions is perceived.
    pub fn stance(&self, actor: &SeenActor) -> Stance {
        match self.content.creature_id("player") {
            Some(player) => self
                .content
                .stance_of(self.content.creature(player).faction, actor.faction),
            None => Stance::Neutral,
        }
    }

    /// Perceived light at a visible cell (wall-face rule applied); `None` for cells not in view.
    pub fn light(&self, p: Pos) -> Option<PerceivedLight> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        let light = self.snapshot.light[p];
        let vision = self.vision();
        let overworld = self.level.kind == LevelKind::Overworld;
        Some(PerceivedLight {
            rgb: light.rgb,
            uv: light.uv,
            darkvision: !overworld
                && vision.darkvision > 0
                && light.brightness() < vision.threshold,
            dist2: p.dist2(self.snapshot.viewer),
        })
    }

    /// Light sources whose light reaches visible cells: static lights by position, then carried
    /// lights by actor id, then the sun, then the moons in content order.
    pub fn light_sources(&self) -> &'a [SeenLight] {
        &self.snapshot.sources
    }

    /// The player's active vision profile.
    pub fn vision(&self) -> &'a VisionProfile {
        &self.content.vision_profiles()[self.snapshot.vision]
    }

    /// The level's ambient light, as the player perceives it.
    pub fn ambient(&self) -> Light {
        self.snapshot.ambient
    }

    /// The sun and the sky over the player's level right now (they may be down); `None` on a
    /// level not under the open sky.
    pub fn sun(&self) -> Option<Sun> {
        self.snapshot.sky.as_ref().map(|s| s.sun)
    }

    /// The sun, the moons, and the sky over the player's level right now (they may be down),
    /// under the weather; `None` on a level not under the open sky.
    pub fn sky(&self) -> Option<&'a Sky> {
        self.snapshot.sky.as_ref()
    }

    /// The weather, which the player knows only under the open sky; `None` elsewhere.
    pub fn weather(&self) -> Option<Weather> {
        self.snapshot.weather()
    }

    /// Steady light at a visible cell that no single source gives: ambient light plus sky light
    /// (under the open sky: the sun's and the moons') plus bounce light; `None` for cells not in
    /// view. With every [`SeenLight`] contribution to the cell added, it is
    /// [`PerceptionView::light`].
    pub fn base_light(&self, p: Pos) -> Option<Light> {
        self.snapshot.is_visible(p).then(|| self.snapshot.base[p])
    }

    /// Sounds the player heard during the last turn from sources they do not see (spec §5
    /// "Hearing"), in the order heard: the kind, how loud, and roughly where (a cell the player
    /// does not see) — never who or what made them. Sounds whose source the player sees are not
    /// here: the player sees what happens.
    pub fn sounds(&self) -> &'a [PerceivedSound] {
        &self.snapshot.sounds
    }

    /// The loudest sound perceived at `p`, if any.
    pub fn sound_at(&self, p: Pos) -> Option<&'a PerceivedSound> {
        self.snapshot
            .sounds
            .iter()
            .filter(|s| s.pos == p)
            .max_by_key(|s| s.volume)
    }
}
