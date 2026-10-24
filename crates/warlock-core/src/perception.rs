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
//!
//! A cell is **visible** iff it is in line of sight within [`VIEW_RADIUS`] **and** its perceived
//! light is at least the vision profile's threshold or it lies within the darkvision radius. The
//! player's own cell is always visible. The profile is the player's **eyes**: their vision
//! profile as their talents change it ([`crate::world::WorldState::eyes`]: Night eyes, Dark
//! adaptation, Owl eyes, Rune sight). Blood scent sees a bleeding creature's cell in line of
//! sight within its radius as darkvision would. A **light-blind** player (their race's
//! [`LightBlind`](crate::content::LightBlind): drow) whose own cell is lit at least to its
//! `glare` is dazzled: they see nothing farther than its `sight`.
//!
//! The overworld is seen at map scale instead: a cell is visible iff it is in line of sight
//! within the daylight sight radius ([`daylight::overworld_sight`]: day 9, twilight 5, night 2,
//! more under a bright moon, less in rain; Moon-eyed adds its cells at night), whatever its
//! light; eyes and darkvision do not change that.
//!
//! **Weather** ([`crate::weather`]) is known only under the open sky: on such a level the sky
//! carries it ([`PerceptionView::weather`]), and its clouds dim the sun, the moons, and the sky.
//! Weather-wise players also know the weather coming within its hours
//! ([`PerceptionView::forecast`]).
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

use crate::ability::{Ability, OwnAbility};
use crate::action::Positions;
use crate::blood::Stain;
use crate::burden::{self, Load};
use crate::character::Character;
use crate::combat::Health;
use crate::content::Shops;
use crate::content::{Choice, TalentId, TreeId};
use crate::content::{
    CombatStance, Content, CreatureId, DamageTypeId, DiseaseId, FactionId, Factor, Flow,
    ItemClassId, LevelKind, LightId, ObjectId, RaceDef, RoomEffectDef, SlotId, Stance, StatusId,
    TerrainId, TrapId, VisionProfile, WeaponSkillId,
};
use crate::daylight::{self, DayPhase};
use crate::equipment::Equipment;
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::hunger::Hunger;
use crate::light::{
    Area, BOUNCE_STEPS, Bounce, Light, LightCache, Moon, SUN_KEY, ShadowSteps, Sky, Sun, Sunlight,
    falloff, lit_in_sight, moon_key, wall_faces,
};
use crate::noise::{self, PerceivedSound};
use crate::object::{Stack, Weight};
use crate::skills::{self, SkillId};
use crate::stats::PerStat;
use crate::talent::{self, Candidate};
use crate::temperature::Temperature;
use crate::time::GameTime;
use crate::weapon_skills::WeaponSkills;
use crate::weather::{self, Weather};
use crate::world::{
    Actor, ActorId, ActorKind, Death, Due, LevelState, Shop, WorldState, count_of, favour, part_of,
    unpaid_parts,
};
use serde::Serialize;

/// Maximum sight distance, however bright the light.
pub const VIEW_RADIUS: i32 = 40;

/// One of the player's own trained skills, as they know it (spec §8 "Skills"): its value and a
/// coarse sense of how near its next point is. Never its marks or what it gives. (Its active use,
/// if it has one, is an ability: [`PerceptionView::abilities`].)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct OwnSkill {
    pub skill: SkillId,
    pub value: u16,
    /// How practised it is, its marks' step on the skill ladder in quarters: 0 (no marks, or
    /// just spent) to 4 (the top dice): how big its next level-up roll is, coarsely.
    pub progress: u8,
}

/// One of the player's own talents (spec §8 "Talents"), as the Talents page and the character
/// screen list it: its name for its choice, its rank of how many, and what it does in the
/// game's words. Never a rule's number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnTalent {
    pub id: TalentId,
    /// "Hardy", "Weapon focus (long blades)".
    pub name: String,
    pub tree: TreeId,
    /// The rank held.
    pub rank: u8,
    /// How many ranks it has.
    pub ranks: u8,
    /// The choice it was taken for, by id.
    pub choice: Option<String>,
    pub description: String,
    /// Their class gives it (spec §8 "Class abilities"): a class ability, or a talent their
    /// class's table hands out at their level or below.
    pub class: bool,
}

/// A talent as the Talents page and the level-up screen list it (spec §8 "Talents"): the rank
/// the player holds, what the next rank asks and whether each part holds, and the choices it
/// may be taken for now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TalentView {
    pub id: TalentId,
    pub name: String,
    pub tree: TreeId,
    pub choice: Option<Choice>,
    /// How many ranks it has.
    pub ranks: u8,
    /// The highest rank the player holds of it, for any choice (0: none).
    pub held: u8,
    /// Given, never picked.
    pub granted: bool,
    pub description: String,
    /// What the next rank asks, in words, each with whether it holds now.
    pub needs: Vec<NeedView>,
    /// The ways it may be taken now: `[None]` for a talent without a choice, the choice ids
    /// for one with a choice; empty when it may not be taken now.
    pub openings: Vec<Option<String>>,
}

/// One prerequisite as shown: "Toughness 14", met or not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeedView {
    pub words: String,
    pub met: bool,
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
    /// The player's eyes: their vision profile as their talents change it
    /// ([`crate::world::WorldState::eyes`]).
    eye: VisionProfile,
    /// Perceived ambient light.
    ambient: Light,
    /// The sun, the moons, and the sky over a level under the open sky.
    sky: Option<Sky>,
    /// The weather coming, which a Weather-wise player knows under the open sky.
    forecast: Option<Weather>,
    /// The temperature of the air where the player is (spec §5 "Temperature").
    temperature: Temperature,
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
    /// The status effects that show on the seen creatures (not the player), by actor, the most
    /// important first.
    effects: Vec<(ActorId, StatusId)>,
    /// Sounds heard during the last turn from sources the player does not see.
    sounds: Vec<PerceivedSound>,
    /// Cells whose blood the player sees, in position order.
    blood: Vec<Pos>,
    /// Whether strong light dazzles the player's light-blind eyes (their race's `light_blind`).
    dazzled: bool,
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
                // Moon-eyed (spec §8 "Talents"): farther at night.
                match daylight::phase(state.time) {
                    DayPhase::Night => range + talents.night_sight,
                    _ => range,
                }
            }
            _ => VIEW_RADIUS,
        };

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
        // Blood scent (spec §8 "Talents"): creatures suffering a status, in line of sight within
        // its radius, however dark their cells.
        if !overworld && !talents.scent.is_empty() {
            let smelt = (state.actors.values())
                .filter(|a| a.level == level.id && a.kind == ActorKind::Creature)
                .filter(|a| los.get(a.pos) == Some(&true) && !is_opaque(a.pos))
                .filter(|a| {
                    (talents.scent.iter()).any(|&(status, radius)| {
                        a.effects.has(status) && a.pos.dist2(viewer) <= radius2(radius)
                    })
                });
            for a in smelt {
                if sight[a.pos] == Sight::Hidden {
                    sight[a.pos] = Sight::Seen;
                    visible_cells.push(a.pos);
                }
            }
        }

        // Wall completion: opaque cells bounding a transparent cell seen by eye.
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
        // So do the statuses that show, such as bleeding; others (poison) do not.
        let effects = (seen.iter())
            .filter(|a| a.kind == ActorKind::Creature)
            .flat_map(|a| {
                (a.effects.iter())
                    .filter(|e| content.status(e.status).seen.is_some())
                    .map(|e| (a.id, e.status))
            })
            .collect();

        // Blood: on an open cell in sight; on a wall, where a stained face is seen.
        let blood = (level.blood.iter())
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
        // The player feels the air: fixed underground, the weather's under the open sky.
        let temperature = state.temperature_on(level, content, state.time);
        // Weather-wise (spec §8 "Talents"): the weather coming, under the open sky.
        let forecast = (sky.is_some() && talents.forecast > 0)
            .then(|| {
                let over = state.weather_override.as_ref();
                weather::forecast(
                    state.seed,
                    state.time,
                    content.climate(),
                    over,
                    talents.forecast,
                )
            })
            .flatten();
        Self {
            viewer,
            eye: eyes,
            ambient,
            sky,
            forecast,
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
            dazzled: dazzled.is_some(),
        }
    }

    /// Whether strong light dazzles the player's light-blind eyes.
    pub(crate) fn dazzled(&self) -> bool {
        self.dazzled
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
    /// they may place a sound they hear ([`noise::rough_place`]).
    pub(crate) fn hides(&self, p: Pos) -> bool {
        self.sight.get(p) == Some(&Sight::Hidden) && p != self.viewer
    }

    /// Whether creatures at `p` can be seen: only in line of sight, never on completed walls.
    pub(crate) fn sees_actors_at(&self, p: Pos) -> bool {
        self.sight.get(p) == Some(&Sight::Seen)
    }

    /// Whether the player sees the blood at `p` (module docs).
    pub(crate) fn sees_blood_at(&self, p: Pos) -> bool {
        self.blood.binary_search(&p).is_ok()
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
        for &p in &self.visible_cells {
            level.memory.set(p, Some(level.terrain[p]));
            if !level.blood.contains_key(&p) {
                level.blood_memory.remove(&p);
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
    /// The player's progression: what their levels still ask them to choose.
    pub(crate) progress: &'a crate::world::Progress,
    /// The player's quests, which they know.
    pub(crate) quests: &'a crate::world::Quests,
    /// Everything the player was told, which they know.
    pub(crate) messages: &'a crate::messages::MessageLog,
    pub(crate) snapshot: &'a Perception,
}

impl<'a> PerceptionView<'a> {
    pub fn content(&self) -> &'a Content {
        self.content
    }

    /// The message log: what the player was told, oldest first, at most
    /// [`crate::messages::CAPACITY`] entries (the message line and the message history).
    pub fn messages(&self) -> &'a crate::messages::MessageLog {
        self.messages
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

    /// The blood at `p` as the player knows it (spec §8 "Blood"): what is there now where they
    /// see it (an open cell in sight, a wall's stained face), as last seen on a cell out of
    /// sight; `None` where they know of none (a visible wall seen only from its clean side
    /// shows none).
    pub fn blood(&self, p: Pos) -> Option<&'a Stain> {
        if self.snapshot.sees_blood_at(p) {
            self.level.blood.get(&p)
        } else if self.snapshot.is_visible(p) {
            None
        } else {
            self.level.blood_memory.get(&p)
        }
    }

    /// Whether the pile at `p` is in sight now (rather than remembered or unknown).
    pub fn sees_objects_at(&self, p: Pos) -> bool {
        self.snapshot.sees_actors_at(p)
    }

    /// The trap the player knows at `p` (spec §8 "Traps"): on a cell in sight, a known trap lying
    /// there now; anywhere else, the trap remembered there. Hidden traps are never here.
    pub fn trap(&self, p: Pos) -> Option<TrapId> {
        if self.snapshot.sees_actors_at(p) {
            (self.level.traps.get(&p))
                .filter(|t| t.known)
                .map(|t| t.kind)
        } else {
            self.level.trap_memory.get(&p).copied()
        }
    }

    /// Every trap the player knows on this level ([`PerceptionView::trap`]), by position.
    pub fn known_traps(&self) -> Vec<(Pos, TrapId)> {
        let cells = (self.level.traps.keys()).chain(self.level.trap_memory.keys());
        let mut cells: Vec<Pos> = cells.copied().collect();
        cells.sort_unstable();
        cells.dedup();
        (cells.into_iter())
            .filter_map(|p| Some((p, self.trap(p)?)))
            .collect()
    }

    /// What the player carries, by position (0 first: the order they picked it up in).
    pub fn inventory(&self) -> &'a [Stack] {
        &self.player.inventory
    }

    /// What item descriptions show the player of worth (spec §8 "Class abilities": Keen
    /// appraisal, Connoisseur, Appraiser), by their own abilities.
    pub fn appraisal(&self) -> crate::magic::describe::Appraisal {
        let talents = self.player.talent_mods(self.content);
        crate::magic::describe::Appraisal {
            worth: talents.item_worth,
            gems: talents.gem_worth,
        }
    }

    /// The item class `stack` shows as (spec §7 "Objects and materials"): its base item's, which
    /// the player knows of anything they see, identified or not.
    pub fn item_class(&self, stack: &Stack) -> ItemClassId {
        self.content.object(stack.object).class
    }

    /// The positions of `stacks` as item lists show them (spec §6 "Item lists"): grouped by item
    /// class in class order, each class in the stacks' own order.
    pub fn by_class(&self, stacks: &[Stack]) -> Vec<usize> {
        let mut order: Vec<usize> = (0..stacks.len()).collect();
        order.sort_by_key(|&i| self.item_class(&stacks[i]));
        order
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

    /// Where in their inventory what the player has ready to throw is (a stack of rocks, or a
    /// sword in the missile weapon slot).
    pub fn ready_to_throw(&self) -> Option<usize> {
        self.player.ready_to_throw(self.content)
    }

    /// The digging tool the player has ready (spec §8 "Digging"), if any.
    pub fn ready_to_dig(&self) -> Option<ObjectId> {
        self.player.dig_tool(self.content).map(|(tool, _)| tool)
    }

    /// Whether the wall at `p`, which the player sees, has been dug into and still stands.
    pub fn is_dug(&self, p: Pos) -> bool {
        self.is_visible(p) && self.level.dug.contains_key(&p)
    }

    /// How far the player throws inventory stack `item`, if it can be thrown (a throwing
    /// weapon, or another weapon, badly).
    pub fn throw_range(&self, item: usize) -> Option<i32> {
        self.player.throw_range(item, self.content)
    }

    /// Whether inventory stack `item` is a throwing weapon, made for throwing (what the throw
    /// choice lists).
    pub fn is_throwing_weapon(&self, item: usize) -> bool {
        (self.player.inventory.get(item)).is_some_and(|s| self.content.object(s.object).is_thrown())
    }

    /// What seen creature `actor` wears and wields, each thing once, in slot order: all the
    /// player sees of what a creature carries. Empty for creatures not in view and the player.
    pub fn gear(&self, actor: ActorId) -> impl Iterator<Item = &'a SeenGear> + 'a {
        self.snapshot.gear.iter().filter(move |g| g.actor == actor)
    }

    /// The player's own scores, by stat (spec §8 "Stats"), as the rules read them: raised by
    /// what they wear and wield (spec §8 "Magic items"), lowered while a status effect or stat
    /// damage lowers them (spec §8 "Status effects", "Diseases") and while hunger leaves them
    /// Weak or worse (spec §8 "Hunger"). That is all the player
    /// knows of stats: never their potentials or modifiers, and nothing of a creature's.
    pub fn own_scores(&self) -> PerStat<u8> {
        self.player.effective_stats(self.content).scores()
    }

    /// How hungry the player is (spec §8 "Hunger"): the state, never the nutrition counter;
    /// `None` when nobody hungers.
    pub fn own_hunger(&self) -> Option<Hunger> {
        self.player.hunger(self.content)
    }

    /// The player's scores as they are without their status effects, hunger, or stat damage
    /// (raised by what they wear and wield all the same): what [`PerceptionView::own_scores`]
    /// would be with none (the character screen marks the lowered ones).
    pub fn own_base_scores(&self) -> PerStat<u8> {
        self.player.raised_stats(self.content).scores()
    }

    /// The player's own trained skills, in skill order (spec §8 "Skills"): each one's value and
    /// how practised it is, coarsely. Untrained skills are not here; nor is anything of a
    /// creature's skills, a mark, or a benefit.
    pub fn own_skills(&self) -> Vec<OwnSkill> {
        let ladder = self.content.ladder();
        (self.player.skills.iter())
            .map(|(skill, trained)| OwnSkill {
                skill,
                value: trained.value,
                progress: skills::progress(trained, ladder),
            })
            .collect()
    }

    /// The player's own status effects, the most important first (spec §8 "Status effects"):
    /// what they suffer, never how badly or for how long.
    pub fn own_effects(&self) -> impl Iterator<Item = StatusId> + 'a {
        self.player.effects.iter().map(|e| e.status)
    }

    /// The diseases the player knows they have, in disease order (spec §8 "Diseases"): those
    /// that have progressed at least once. One that incubates, or has only just shown, is not
    /// here (a showing one makes the player Diseased, [`PerceptionView::own_effects`]).
    pub fn own_diseases(&self) -> impl Iterator<Item = DiseaseId> + 'a {
        self.player.diseases.known()
    }

    /// The status effects that show on seen creature `actor` (`seen` in `data/status.toml`:
    /// bleeding), the most important first; none for creatures not in view and the player.
    pub fn effects_of(&self, actor: ActorId) -> impl Iterator<Item = StatusId> + 'a {
        (self.snapshot.effects.iter())
            .filter(move |(a, _)| *a == actor)
            .map(|(_, s)| *s)
    }

    /// The player's abilities, in menu order (grouped by source, [`crate::ability`]), each with
    /// why it cannot be used now, if it cannot: what the abilities menu (#156) lists. They know
    /// what they can do and what they hold.
    pub fn abilities(&self) -> Vec<OwnAbility> {
        (Ability::of(self.player, self.content).into_iter())
            .map(|ability| OwnAbility {
                ability,
                unavailable: ability.unavailable(self.player, self.content),
            })
            .collect()
    }

    /// The player's ranks and marks in weapon skills (spec §8 "Weapon skills"): they know their
    /// ranks, and see their marks as a bar toward the next. Never what a rank gives.
    pub fn own_weapon_skills(&self) -> &'a WeaponSkills {
        &self.player.weapon_skills
    }

    /// The fighting styles the player fights in now (spec §8 "Fighting styles"), in skill
    /// order: their hands', their readied missile weapon's, and throwing's while something is
    /// ready to throw. They know how they hold their weapons.
    pub fn own_styles_in_use(&self) -> Vec<WeaponSkillId> {
        self.player.styles_in_use(self.content)
    }

    /// The player's hit points: what they have left (at most their maximum, never below 0) and
    /// their maximum (spec §8 "Combat").
    pub fn own_hp(&self) -> (i32, i32) {
        let max = self.player.max_hp(self.content);
        (self.player.hp.clamp(0, max), max)
    }

    /// The player's mana: what they have left (at most their maximum, never below 0) and their
    /// maximum (spec §8 "Mana"). Never how fast it comes back.
    pub fn own_mana(&self) -> (i32, i32) {
        let max = self.player.max_mana(self.content);
        (self.player.mana.clamp(0, max), max)
    }

    /// The player's level, experience total, and the total at which the next level comes (none
    /// at the highest level) (spec §8 "Experience"). All of it is shown to the player.
    pub fn own_experience(&self) -> Progress {
        let rules = self.content.experience();
        let factor = Factor::of(self.content, self.character.map(|c| &c.spec));
        let level = rules.level_for(self.experience, factor);
        Progress {
            level,
            xp: self.experience,
            next: (level < rules.max_level()).then(|| rules.threshold_for(level + 1, factor)),
        }
    }

    /// The choices a level still asks of the player (spec §8 "Talents"), if any: the level and
    /// how many talent picks are left. Nothing else may be done until they are made.
    pub fn due(&self) -> Option<Due> {
        self.progress.due.first().copied()
    }

    /// Whether the player's own bursts spare them (spec §8 "Talents": Careful reader), so a
    /// scroll read close needs no warning.
    pub fn bursts_spare_me(&self) -> bool {
        self.player.talent_mods(self.content).spares_self
    }

    /// The player as prerequisites see them.
    fn candidate(&self) -> Candidate<'a> {
        let level = self.own_experience().level;
        Candidate::player(self.content, self.player, self.character, level)
    }

    /// The player's own talents, in the order taken (spec §8 "Talents"): each by its name for
    /// its choice, its tree, the rank held of how many, and what it does in the game's words;
    /// never a rule's number.
    pub fn own_talents(&self) -> Vec<OwnTalent> {
        let book = self.content.talents();
        let level = self.own_experience().level;
        let class = (self.character)
            .and_then(|c| self.content.class(&c.spec.class))
            .map(|c| c.abilities_to(book, level))
            .unwrap_or_default();
        (self.player.talents.iter())
            .filter_map(|taken| {
                let id = book.id(&taken.id)?;
                let def = book.get(id);
                Some(OwnTalent {
                    id,
                    name: def.name_for(self.content, taken.choice.as_deref()),
                    tree: def.tree,
                    rank: taken.rank,
                    ranks: def.rank_count(),
                    choice: taken.choice.clone(),
                    description: def.description.clone(),
                    class: class.contains(&id),
                })
            })
            .collect()
    }

    /// Every talent of the game, tree by tree in display order (the Talents page and the pick
    /// list): the rank the player holds, the prerequisites of the next rank with whether each
    /// holds, and the ways it may be taken now (none: not now).
    pub fn talents(&self) -> Vec<TalentView> {
        let content = self.content;
        let who = self.candidate();
        (content.talents().by_tree())
            .map(|(id, def)| {
                let held = (self.player.talents.iter())
                    .filter(|t| t.id == def.id)
                    .map(|t| t.rank)
                    .max()
                    .unwrap_or(0);
                let next = (held + 1).min(def.rank_count());
                let needs = (def.needs_at(next))
                    .map(|need| NeedView {
                        met: who.meets(content, &need, def.choice, None),
                        words: need.words(content),
                    })
                    .collect();
                TalentView {
                    id,
                    name: def.name.clone(),
                    tree: def.tree,
                    choice: def.choice,
                    ranks: def.rank_count(),
                    held,
                    granted: def.granted,
                    description: def.description.clone(),
                    needs,
                    openings: talent::openings(content, &who, def),
                }
            })
            .collect()
    }

    /// The player's alignment band (spec §8 "Alignment"), which they know; `None` without
    /// alignment rules. Never the score.
    pub fn own_alignment(&self) -> Option<crate::alignment::Band> {
        (self.content.alignment()).map(|_| self.player.band(self.content))
    }

    /// What the player carries against what they can (spec §8 "Burden"): their burden, and the
    /// weights at which it changes, which follow from scores they know.
    pub fn own_load(&self) -> Load {
        burden::load(self.player, self.own_race(), self.content)
    }

    /// The player's race, when the run has a character.
    fn own_race(&self) -> Option<&'a RaceDef> {
        self.character.and_then(|c| c.spec.race(self.content).ok())
    }

    /// The player's combat stance (spec §8 "Stances"): only its name is ever shown.
    pub fn own_stance(&self) -> CombatStance {
        self.player.stance
    }

    /// The player's Defence ("Dv"): the total an attack roll must reach (their burden capping
    /// their Dexterity). Only the total: its Dexterity part would give away a modifier.
    pub fn own_defence(&self) -> i32 {
        let burden = self.own_load().penalty();
        self.player
            .defence(self.content, burden, self.own_race())
            .total()
    }

    /// The player's Protection ("Pv"): the damage what they wear soaks off every hit, the total
    /// only (never what each piece does against a kind, nor a creature's).
    pub fn own_protection(&self) -> i32 {
        self.player.protection(self.content, self.own_race()).base
    }

    /// What the player resists (spec §8 "Damage types"), in words only: each kind they resist,
    /// are vulnerable to, or are immune to, in damage type order; never a percentage, and never
    /// a creature's.
    pub fn own_resistances(&self) -> Vec<(DamageTypeId, crate::damage::ResistanceWord)> {
        (self
            .player
            .resistances(self.content, self.own_race())
            .iter())
        .filter_map(|(kind, percent)| Some((kind, crate::damage::ResistanceWord::of(percent)?)))
        .collect()
    }

    /// How the run ended, once the player has died (what killed them only if they saw it).
    pub fn death(&self) -> Option<&'a Death> {
        self.death
    }

    /// How a seen creature stands toward the player: its faction's stance (spec §7 "Factions"),
    /// which the player knows from its type; neutral for an animal a Beast-friend has not
    /// attacked one of (spec §8 "Talents"). Nothing else about factions is perceived.
    pub fn stance(&self, actor: &SeenActor) -> Stance {
        let animal = actor
            .faction
            .is_some_and(|f| self.content.factions().def(f).animal);
        if animal
            && actor.kind == ActorKind::Creature
            && self.player.talent_mods(self.content).beast_friend
            && !(self.progress.empathy_broken).contains(&self.content.creature(actor.creature).id)
        {
            return Stance::Neutral;
        }
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

    /// The player's eyes: their active vision profile as their talents change it (its id and
    /// name the base profile's).
    pub fn vision(&self) -> &'a VisionProfile {
        &self.snapshot.eye
    }

    /// Whether strong light dazzles the player (light-blind eyes in glare): they see less far,
    /// and fight worse. Only that it does, never how much.
    pub fn dazzled(&self) -> bool {
        self.snapshot.dazzled
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

    /// The weather coming within the hours a Weather-wise player foresees (spec §8 "Talents"),
    /// if it changes; `None` when it stays, without the talent, or away from the open sky.
    pub fn forecast(&self) -> Option<Weather> {
        self.snapshot.forecast
    }

    /// The temperature of the air where the player is (spec §5 "Temperature"), which they feel,
    /// in dungeons and under the open sky alike (the status bar shows it).
    pub fn temperature(&self) -> Temperature {
        self.snapshot.temperature
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

/// A shop as the player knows it (spec §7 "Shops"): its sign, its keeper's name, and its prices
/// for them, in gold. Never the value, the markup, or how their Presence or skills move the
/// prices.
#[derive(Clone, Copy, Debug)]
pub struct ShopView<'a> {
    shop: &'a Shop,
    shops: &'a Shops,
    content: &'a Content,
    /// The player's favour (`world::favour`), hidden: only prices come of it.
    favour: i32,
    /// Percent more it pays the player for what it buys (spec §8 "Class abilities": Insured).
    more: i32,
    /// The moment it is: rotten food is worth nothing (spec §8 "Hunger"), which only the
    /// prices tell.
    now: GameTime,
}

impl<'a> ShopView<'a> {
    /// Whether `stack` is rotten food, which the shop neither sells nor buys.
    fn spoilt(&self, stack: &Stack) -> bool {
        crate::rot::stage(self.content, stack, self.now).is_some_and(|s| s.is_rotten())
    }

    /// What `stack` costs on the bill (what the player took is priced as it was).
    fn bill(&self, stack: &Stack) -> u32 {
        (self.shops).price(self.content, self.shop.kind, stack, self.favour)
    }

    /// Its sign: "Grimbold's General Store".
    pub fn sign(&self) -> String {
        self.shop.sign(self.content)
    }

    /// Its keeper's name.
    pub fn keeper(&self) -> &'a str {
        &self.shop.name
    }

    /// Whether `p` is on its floor.
    pub fn contains(&self, p: Pos) -> bool {
        self.shop.contains(p)
    }

    /// Whether `p` is one of its ways out (its door, in plain sight): a step out there owing
    /// anything is turned back, any other way out is theft.
    pub fn is_exit(&self, p: Pos) -> bool {
        self.shop.is_exit(p)
    }

    /// What it asks for `stack` lying on its floor, if it is for sale (worth something, not the
    /// currency, not rotten food).
    pub fn price(&self, stack: &Stack) -> Option<u32> {
        let price = if self.spoilt(stack) {
            0
        } else {
            self.bill(stack)
        };
        (price > 0).then_some(price)
    }

    /// What it offers for `stack`, if it buys it (never rotten food).
    pub fn offer(&self, stack: &Stack) -> Option<u32> {
        if self.spoilt(stack) {
            return None;
        }
        let offer =
            (self.shops).offer_more(self.content, self.shop.kind, stack, self.favour, self.more);
        (offer > 0).then_some(offer)
    }

    /// What the player owes it.
    pub fn owed(&self) -> u32 {
        self.shop.owed(self.shops, self.content, self.favour)
    }
}

/// What a shop would pay for the player's own things among the stacks they chose to drop there
/// (spec §7 "Shops"): the offer and what it is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sale {
    pub offer: u32,
    pub stacks: Vec<Stack>,
}

impl<'a> PerceptionView<'a> {
    /// The open shop whose floor `p` is, if the player sees or remembers `p` (its sign and goods
    /// are in plain sight) and its keeper still trades with them.
    pub fn shop_at(&self, p: Pos) -> Option<ShopView<'a>> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let shops = self.content.shops()?;
        let shop = (self.level.shops.iter()).find(|s| s.is_open() && s.contains(p))?;
        Some(ShopView {
            shop,
            shops,
            content: self.content,
            favour: favour(self.content, shops, self.player),
            more: self.player.talent_mods(self.content).sell_percent,
            now: self.time,
        })
    }

    /// The open shop the player stands in.
    pub fn shop_here(&self) -> Option<ShopView<'a>> {
        self.shop_at(self.snapshot.viewer)
    }

    /// The price of `stack` lying at `p`, a cell in sight, if it is for sale there (look mode:
    /// "a dagger (for sale, 45 gold)").
    pub fn price_tag(&self, p: Pos, stack: &Stack) -> Option<u32> {
        if !self.snapshot.sees_actors_at(p) {
            return None;
        }
        self.shop_at(p)?.price(stack)
    }

    /// What the player has not paid for of each inventory stack, priced, in the shop they stand
    /// in (`None` for what is theirs; spec §7 "Shops").
    pub fn unpaid(&self) -> Vec<Option<u32>> {
        let inventory = &self.player.inventory;
        let Some(view) = self.shop_here() else {
            return vec![None; inventory.len()];
        };
        let parts = unpaid_parts(inventory, &view.shop.unpaid);
        (inventory.iter().zip(parts))
            .map(|(s, n)| (n > 0).then(|| view.bill(&part_of(s, n))))
            .collect()
    }

    /// What the player owes, and to whom, if a step onto `to` leaves the open shop they stand in
    /// (onto ground they know is walkable): by a way out ([`ShopView::is_exit`]) the keeper turns
    /// them back, any other way is theft, so presentation asks first. Nothing when the keeper
    /// lets them leave owing it (spec §8 "Class abilities": Credit).
    pub fn owed_on_leaving(&self, to: Pos) -> Option<(ShopView<'a>, u32)> {
        let view = self.shop_here()?;
        let walkable = match self.cell(to) {
            CellView::Visible(t) | CellView::Remembered(t) => self.content.terrain(t).walkable,
            CellView::Unknown => false,
        };
        let owed = view.owed();
        let credit = self.player.talent_mods(self.content).credit;
        let on_credit = credit > 0 && owed <= credit;
        (!view.contains(to) && walkable && owed > 0 && !on_credit).then_some((view, owed))
    }

    /// What the shop the player stands in offers for their own things among the inventory stacks
    /// `positions` names (all of them for none), if it buys any: what dropping them there would
    /// sell ([`crate::Action::Sell`]).
    pub fn sale(&self, positions: &Positions) -> Option<Sale> {
        let view = self.shop_here()?;
        let inventory = &self.player.inventory;
        let parts = unpaid_parts(inventory, &view.shop.unpaid);
        let all = positions.is_empty();
        let mut sale = Sale {
            offer: 0,
            stacks: Vec::new(),
        };
        for (i, stack) in inventory.iter().enumerate() {
            let own = part_of(stack, stack.count - parts[i]);
            if !(all || positions.contains(i)) || own.count == 0 {
                continue;
            }
            if let Some(offer) = view.offer(&own) {
                sale.offer += offer;
                sale.stacks.push(own);
            }
        }
        (!sale.stacks.is_empty()).then_some(sale)
    }

    /// How much gold the player carries (the shops' currency; none without shops).
    pub fn own_gold(&self) -> u32 {
        (self.content.shops()).map_or(0, |s| count_of(&self.player.inventory, s.currency))
    }
}
