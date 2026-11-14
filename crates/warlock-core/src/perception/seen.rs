//! What the player sees of the world: cells, creatures and their gear, their allies, and light.

use crate::color::Rgb;
use crate::combat::Health;
use crate::content::DialogueId;
use crate::content::{Content, CreatureId, FactionId, LightId, MaterialId, SlotId, TerrainId};
use crate::geom::Pos;
use crate::light::{Light, Moon, ShadowSteps, Sun};
use crate::object::Stack;
use crate::world::{ActorId, ActorKind, WorldState};
use serde::Serialize;

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
    /// What it says when talked to, if anything (`Content::dialogue_of`): a unique character's
    /// proper name and the services it sells are known by sight.
    #[serde(skip)]
    pub dialogue: Option<DialogueId>,
    /// The faction it fights for (its type's, or the one its level placed it in).
    #[serde(skip)]
    pub faction: Option<FactionId>,
    pub pos: Pos,
    /// How hurt it looks: words, never hit points (spec §8 "Combat").
    pub health: Health,
    /// How it looks when not as its type does: an animated thing by what it is made of (spec §8
    /// "Animated objects"), which anyone sees.
    #[serde(skip)]
    pub shape: Option<Shape>,
    /// It has not noticed the player (`World::unaware`: asleep, disguised, slipped past, or
    /// hunting no one and not seeing them), told only to a player with Stalker's eye; false
    /// for everyone otherwise (spec §8 "Surprise and sneak attacks": the unaware mark).
    #[serde(skip)]
    pub unaware: bool,
    /// It is the player's mark, the creature their openings are in (`world/mark.rs`): their
    /// own choice, so they know it.
    #[serde(skip)]
    pub marked: bool,
    /// It is an ally of the player's (spec §8 "Allies"), which they know by sight.
    #[serde(skip)]
    pub ally: bool,
    /// It is the player's kept beast, a companion or a familiar (`world/companion.rs`,
    /// `familiar.rs`): its name is in [`PerceptionView::allies`] under its id.
    #[serde(skip)]
    pub companion: bool,
    /// Its grade (spec §8 "Grades"), which anyone sees: −1 lesser, +1 greater, 0
    /// neither ([`crate::grade::word`] names it).
    #[serde(skip)]
    pub grade: i8,
}

/// One of the player's allies as they know it (spec §8 "Allies"): never a number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllyView {
    pub id: ActorId,
    pub creature: CreatureId,
    /// Its type's name.
    pub name: String,
    /// How hurt it looks.
    pub health: Health,
    pub order: crate::world::Order,
    pub source: crate::world::AllySource,
    /// How long its stay has left, if it ends; whole minutes, rounded up.
    pub lasts: Option<u32>,
    /// Travelling with the player off any level.
    pub retinue: bool,
    /// On the level the player is on, acting.
    pub here: bool,
    /// What it wears (a companion's collar, `world/collar.rs`), which the player knows wherever
    /// it is: they fastened it.
    pub wears: Vec<Stack>,
}

/// The player's allies ([`AllyView`]), in actor id order.
pub(super) fn allies_of(state: &WorldState, content: &Content) -> Vec<AllyView> {
    let player = state.player;
    (state.actors.values())
        .filter(|a| a.master() == Some(player))
        .filter_map(|a| {
            let ally = a.ally.as_ref()?;
            let lasts = ally.until.map(|until| {
                let seconds = until.since(state.time).as_seconds();
                u32::try_from(seconds.div_ceil(60)).unwrap_or(u32::MAX)
            });
            Some(AllyView {
                id: a.id,
                creature: a.creature,
                name: content.creature(a.creature).name.clone(),
                health: a.health(content),
                order: ally.order,
                source: ally.source.clone(),
                lasts,
                retinue: ally.retinue,
                here: a.level == state.current_level && !ally.retinue,
                wears: (a.equipment.items().into_iter())
                    .filter_map(|(_, item)| a.inventory.get(item).cloned())
                    .collect(),
            })
        })
        .collect()
}

/// How a creature looks by what it is made of (spec §8 "Animated objects"): an animated
/// candlestick of silver is drawn in silver's colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub glyph: char,
    pub fg: Rgb,
    /// The material its main component is made of (look mode: "of silver").
    pub material: MaterialId,
    /// A composite's colours (spec §8 "Animated objects"): the distinct material colours of
    /// what it wears and wields, in slot order, the first [`Shape::BANDS`]; the first `count`
    /// are used, and fewer than two mean it shows `fg` alone (a one-item thing).
    pub bands: [Rgb; Shape::BANDS],
    pub count: u8,
}

impl Shape {
    /// The most colours a composite shows on its one cell.
    pub const BANDS: usize = 4;

    /// The colours it shows in horizontal bands, top first: two or more for a composite of
    /// several materials, none for a thing of one colour.
    pub fn bands(&self) -> &[Rgb] {
        match usize::from(self.count) {
            n @ 2.. => &self.bands[..n],
            _ => &[],
        }
    }
}

impl SeenActor {
    /// The glyph it is drawn with: its shape's, else its type's.
    pub fn glyph(&self, content: &Content) -> char {
        self.shape
            .map_or(content.creature(self.creature).glyph, |s| s.glyph)
    }

    /// The colour it is drawn in: its shape's, else its type's.
    pub fn fg(&self, content: &Content) -> Rgb {
        self.shape
            .map_or(content.creature(self.creature).fg, |s| s.fg)
    }
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
    /// light the player has ready (a lantern).
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
