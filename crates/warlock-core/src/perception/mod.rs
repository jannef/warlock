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
mod lighting;
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

use seeing::{Los, Seeing};
use seen::allies_of;

use crate::blood::Stain;
use crate::content::{
    Content, LevelKind, StatusId, TerrainDef, TerrainId, VisionProfile, ZoneKind,
};
use crate::geom::{Dir, Pos};
use crate::grid::Window;
use crate::light::{Light, LightCache, Sky};
use crate::noise::{self, PerceivedSound};
use crate::talent::TalentMods;
use crate::temperature::Temperature;
use crate::weather::Weather;
use crate::world::{ActorId, LevelState, WorldState};
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
    /// An unseen shadowy creature is within the `[unseen]` draught of `data/rules/tuning.toml`
    /// of the player: they feel
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

/// What every step of [`Perception::compute`] reads: the world, the player's level and cell,
/// their talents and eyes, and the sky over them.
struct Viewing<'a> {
    state: &'a WorldState,
    content: &'a Content,
    level: &'a LevelState,
    viewer: Pos,
    talents: &'a TalentMods,
    /// The player's eyes: their vision profile as their talents change it.
    eye: VisionProfile,
    /// The sun, the moons, and the sky over a level under the open sky.
    sky: Option<Sky>,
    /// Whether the level is the overworld, seen at map scale (the module docs).
    overworld: bool,
}

impl Viewing<'_> {
    fn is_opaque(&self, p: Pos) -> bool {
        self.level.is_opaque(self.content, p)
    }

    /// The terrain at `p`, if on the level.
    fn terrain(&self, p: Pos) -> Option<&TerrainDef> {
        self.level.terrain.get(p).map(|t| self.content.terrain(*t))
    }

    /// Whether `p` is under the open sky (no roof).
    fn exposed(&self, p: Pos) -> bool {
        self.terrain(p).is_some_and(|t| !t.roofed)
    }

    /// A shadowed room keeps only part of every bit of a cell's light: its base light and each
    /// source's contribution alike, so they still add up to the whole.
    fn shade(&self, p: Pos, light: Light) -> Light {
        self.level.shade(self.content, p, light, self.state.time)
    }
}

/// The zones on the cells the player sees, by position: smoke is seen where sight stops at it,
/// though what is in it is not.
fn zones_seen(v: &Viewing, los: &Los, seeing: &Seeing) -> BTreeMap<Pos, ZoneKind> {
    let (level, now) = (v.level, v.state.time);
    if level.zones.is_empty() {
        return BTreeMap::new();
    }
    let smoke = (los.cells.iter()).filter(|&&p| level.smoky(p, now));
    (seeing.visible_cells.iter().chain(smoke))
        .filter_map(|&p| Some((p, level.zone_at(p, now)?.kind())))
        .collect()
}

/// The cells of `stains` (blood or char) the player sees: on an open cell in sight; on a wall,
/// where a stained face is seen.
fn stained(v: &Viewing, seeing: &Seeing, stains: &BTreeMap<Pos, Stain>) -> Vec<Pos> {
    let sight = &seeing.sight;
    (stains.iter())
        .filter(|(p, stain)| {
            let p = **p;
            if !v.is_opaque(p) {
                return sight.get(p) == Some(&Sight::Seen);
            }
            let faces = |d: Dir| stain.faces == 0 || stain.on_face(d);
            sight.get(p).is_some_and(|s| *s != Sight::Hidden)
                && Dir::ALL.into_iter().filter(|d| faces(*d)).any(|d| {
                    let q = p.step(d);
                    sight.get(q) == Some(&Sight::Seen) && !v.is_opaque(q)
                })
        })
        .map(|(p, _)| *p)
        .collect()
}

/// Sounds the player heard whose source they do not see, placed roughly.
fn heard(
    v: &Viewing,
    seeing: &Seeing,
    concealed: &[crate::world::Concealed],
) -> Vec<PerceivedSound> {
    let (content, talents, viewer) = (v.content, v.talents, v.viewer);
    let player = v.state.player();
    let Some(threshold) = player.hearing(content) else {
        return Vec::new();
    };
    let hidden = |p: Pos| {
        p != viewer
            && (seeing.sight.get(p) == Some(&Sight::Hidden) || concealed.iter().any(|c| c.pos == p))
    };
    // Watchful (spec §8 "Birth sign"): footsteps placed exactly; the ranger's Tracker: every
    // sound.
    let exact = match (talents.senses.exact_sounds, talents.senses.exact_footsteps) {
        (true, _) => noise::Exact::All,
        (false, true) => noise::Exact::Footsteps,
        (false, false) => noise::Exact::No,
    };
    let heard = (&player.heard[..], threshold, exact);
    noise::perceive(v.level, content, heard, v.state.turn, hidden)
}

impl Perception {
    /// Computes what the player perceives on their current level: line of sight, light, and
    /// visibility. Light masks come from (and are added to) `lights`.
    ///
    /// The steps, in order: line of sight (`seeing.rs`), light on the cells in it
    /// (`lighting.rs`), the cells seen by that light, light on the wall faces seen, the walls
    /// seen by it, then the creatures seen (`actors.rs`), and the zones, blood, char, and sounds.
    pub(crate) fn compute(state: &WorldState, content: &Content, lights: &mut LightCache) -> Self {
        let level = state.current();
        let talents = state.player().talent_mods(content);
        let v = Viewing {
            state,
            content,
            level,
            viewer: state.player().pos,
            talents: &talents,
            eye: state.eyes(content),
            sky: state.sky(content),
            // On the overworld, daylight (and moonlight, and rain) sets how far one sees; light
            // does not matter.
            overworld: level.kind == LevelKind::Overworld,
        };
        let los = seeing::line_of_sight(&v);
        let mut lighting = lighting::light(&v, &los, lights);
        let mut seeing = Seeing::see(&v, &los, &lighting.light, lights);
        lighting.light_faces(&v, &los, &seeing);
        seeing.see_walls(&v, &los, &lighting.light);
        let sources = lighting.sources(&seeing);
        let seen = actors::seen(&v, &seeing, &lighting.light, lights);
        let zones = zones_seen(&v, &los, &seeing);
        // Blood and char: on an open cell in sight; on a wall, where a stained face is seen.
        let blood = stained(&v, &seeing, &level.blood);
        let scorch = stained(&v, &seeing, &level.scorch);
        let sounds = heard(&v, &seeing, &seen.concealed);
        // The player feels the air where they stand: fixed underground, the weather's under the
        // open sky, and what a hot or a cold room adds.
        let temperature = state.temperature_at(level, content, state.time, v.viewer);
        Self {
            viewer: v.viewer,
            eye: v.eye,
            ambient: lighting.ambient,
            sky: v.sky,
            temperature,
            base: lighting.base,
            sight: seeing.sight,
            visible_cells: seeing.visible_cells,
            light: lighting.light,
            sources,
            actors: seen.actors,
            gear: seen.gear,
            effects: seen.effects,
            sounds,
            blood,
            scorch,
            images: seen.images,
            dazzled: seeing.dazzled,
            averted: seen.averted,
            disguises: seen.disguises,
            concealed: seen.concealed,
            draught: seen.draught,
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
