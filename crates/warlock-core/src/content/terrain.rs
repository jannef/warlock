//! Terrain definitions (`assets/data/terrain.toml`).

use super::action::{BASE_TURN, price};
use super::anim::GlyphAnim;
use super::level::LevelKind;
use super::light::LightId;
use super::overworld::Landscape;
use super::particles::ParticleEmitter;
use crate::color::Rgb;
use crate::dice::Dice;
use crate::geom::Pos;
use crate::time::Duration;
use serde::{Deserialize, Serialize};

/// Tallest terrain for sun shadows, in cells: it bounds how far a shadow can reach.
pub const MAX_HEIGHT: u8 = 4;
/// The hardest a wall may be (spec §8 "Digging"): a thousand digs with a pick-axe at Strength
/// 10.
pub const MAX_HARDNESS: u16 = 10_000;

/// Index of a terrain definition in [`super::Content::terrain`].
///
/// Ids are assigned in file order when content loads; saves must not assume they are stable
/// across content changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TerrainId(pub u16);

/// Which way a staircase leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StairsDir {
    Up,
    Down,
}

/// A terrain type as written in content files.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TerrainSpec {
    pub id: String,
    pub name: String,
    /// What look mode adds after its name (a forgotten altar's), if anything.
    #[serde(default)]
    pub description: Option<String>,
    /// A special feature of the ground (an altar): stepping onto it says so.
    #[serde(default)]
    pub announce: bool,
    pub glyph: char,
    /// Presentation-only: glyphs the map draws instead of `glyph`, one per cell picked by its
    /// position (a space draws none), e.g. hairline cracks in thin ice.
    #[serde(default)]
    pub variants: String,
    pub fg: Rgb,
    #[serde(default)]
    pub bg: Rgb,
    #[serde(default = "yes")]
    pub walkable: bool,
    #[serde(default = "yes")]
    pub transparent: bool,
    /// What a step onto this terrain costs, in percent of a normal step (spec §8 "Time").
    #[serde(default = "full_cost")]
    pub move_cost: u16,
    /// Overworld terrain: minutes one overworld step onto it takes (spec §8 "Time").
    pub travel_minutes: Option<u16>,
    /// Overworld terrain: the ground of the local area entered from it (`>` on the cell); none
    /// means the cell has no local area.
    pub local: Option<String>,
    /// Overworld terrain: what its land is like in local areas (the generator of its own local
    /// areas, or what it adds to its neighbors').
    pub landscape: Option<Landscape>,
    /// For a closed door: the terrain it becomes when opened.
    pub opens_to: Option<String>,
    /// For an open door: the terrain it becomes when closed.
    pub closes_to: Option<String>,
    /// For a closed door that can be kicked in: the terrain it becomes when broken (a broken
    /// door, open for good); with `sturdiness`.
    pub breaks_to: Option<String>,
    /// For a closed door that can be kicked in: a kick breaks it with a chance of the kicker's
    /// Strength in this (spec §8 "Doors"); with `breaks_to`.
    pub sturdiness: Option<u8>,
    pub stairs: Option<StairsDir>,
    /// Light preset the terrain emits (torch sconce, glowing fungus, rune). Must be transparent.
    pub light: Option<String>,
    /// Tint of the light the surface bounces (spec §5); white (neutral) by default.
    #[serde(default = "neutral")]
    pub bounce: Rgb,
    /// How tall it stands for sun shadows, in cells (spec §5 "Sunlight"); default 1 for opaque
    /// terrain, 0 for transparent terrain.
    pub height: Option<u8>,
    /// Under a roof: never lit by the sun or the sky (indoor floors on outdoor levels).
    #[serde(default)]
    pub roofed: bool,
    /// Whether blood stains it (spec §8 "Blood"); water, lava, and fire do not hold it.
    #[serde(default = "yes")]
    pub stains: bool,
    /// Whether it is stone (spec §8 "Searching": dwarves notice what is hidden on stone).
    #[serde(default)]
    pub stone: bool,
    /// How loud steps onto it are, in percent of the walker's footsteps (spec §5 "Hearing").
    #[serde(default = "full_noise")]
    pub step_noise: u16,
    /// For a door: how loud opening or closing it is; none: silent.
    #[serde(default)]
    pub toggle_noise: u16,
    /// Loudness a sound loses entering the cell; none: 0, or for solid terrain (neither walkable
    /// nor transparent) blocking sound.
    pub muffle: Option<u16>,
    /// How hard it is to break (spec §8 "Digging"): a wall that can be dug through or blasted
    /// open, taking this much work; none: it never breaks. Goes with `becomes`.
    pub hardness: Option<u16>,
    /// What a broken wall becomes (rubble): a walkable, transparent terrain.
    pub becomes: Option<String>,
    /// Solid rock deep in the ground: the terrain it shows as once a cell next to it is broken
    /// open (its wall face).
    pub face: Option<String>,
    /// Water: how ice grows on it (spec §5 "Ice").
    pub freeze: Option<FreezeSpec>,
    /// Ice: the thinnest ice that looks like this, in mm (spec §5 "Ice"); it lies on water.
    pub ice: Option<u16>,
    pub anim: Option<GlyphAnim>,
    #[serde(default)]
    pub glow: f32,
    #[serde(default)]
    pub haze: f32,
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
}

fn yes() -> bool {
    true
}

/// The `freeze` table of a water terrain as data writes it (spec §5 "Ice"): [`Freeze`] with
/// the kind of damage the plunge does by name (`damage_type = "cold"`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreezeSpec {
    pub below: i8,
    #[serde(default = "full_rate")]
    pub rate: u8,
    pub plunge: Dice,
    /// The kind of damage the plunge does (spec §8 "Damage types": cold).
    pub damage_type: Option<String>,
}

impl FreezeSpec {
    /// The freeze it describes, its plunge of `kind`.
    pub(super) fn resolve(&self, kind: Option<super::DamageTypeId>) -> Freeze {
        Freeze {
            below: self.below,
            rate: self.rate,
            plunge: self.plunge,
            kind,
        }
    }
}

/// How water freezes (spec §5 "Ice"): the `freeze` table of a water terrain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Freeze {
    /// Ice grows on it while the air is colder than this, in °C: still water at 0, running water
    /// only in a harder frost. It melts above 0.
    pub below: i8,
    /// How fast it grows, in percent of still water's pace (1 to 100).
    pub rate: u8,
    /// The hit points a fall through the ice into it takes (dice; it never takes the last one).
    pub plunge: Dice,
    /// The kind of damage the plunge does (cold; spec §8 "Damage types"), which the one who
    /// falls in resists; `None`: untyped.
    pub kind: Option<super::DamageTypeId>,
}

fn full_rate() -> u8 {
    100
}

fn full_cost() -> u16 {
    100
}

fn full_noise() -> u16 {
    100
}

fn neutral() -> Rgb {
    Rgb::WHITE
}

/// A resolved terrain definition.
#[derive(Clone, Debug)]
pub struct TerrainDef {
    pub id: String,
    pub name: String,
    /// What look mode says of it after its name, if anything (a forgotten altar's, spec §7
    /// "Special rooms").
    pub description: Option<String>,
    /// A special feature of the ground (an altar, a faint rune; spec §6): arriving
    /// on it says so (`world/feature.rs`). Only walkable terrain has it.
    pub announce: bool,
    pub glyph: char,
    /// Presentation-only: glyphs the map draws in place of `glyph`, one per cell
    /// ([`TerrainDef::glyph_at`]); empty: always `glyph`.
    pub variants: Vec<char>,
    pub fg: Rgb,
    pub bg: Rgb,
    pub walkable: bool,
    pub transparent: bool,
    /// What a step onto this terrain costs, in percent of a normal step (the `step` action's
    /// cost): 100 is a normal step, 200 (shallow water) takes twice as long. Always at least 1.
    /// Local levels only: on the overworld a step takes the terrain's `travel_minutes`.
    pub move_cost: u16,
    /// Overworld terrain: minutes one overworld step onto it takes (1 to a day). Every walkable
    /// terrain on the overworld has it.
    pub travel_minutes: Option<u16>,
    /// Overworld terrain: the ground of the local area entered from a cell of it; `None` if such
    /// cells have no local area (roads have one; bridges and locations do not).
    pub local: Option<TerrainId>,
    /// Overworld terrain: what its land is like in local areas. Every terrain with a `local`
    /// ground has a land or road landscape.
    pub landscape: Option<Landscape>,
    pub opens_to: Option<TerrainId>,
    pub closes_to: Option<TerrainId>,
    /// Gameplay: a closed door a kick can break, the terrain it becomes then (a broken door) and
    /// its sturdiness: a kick breaks it with a chance of the kicker's Strength score in this
    /// ([`crate::door::kick_breaks`]). `None`: kicks never break it. (Digging and blasts break
    /// walls, `breaks`, never doors.)
    pub kicked_in: Option<(TerrainId, u8)>,
    pub stairs: Option<StairsDir>,
    /// Light this terrain emits, if any.
    pub light: Option<LightId>,
    /// Gameplay: the tint of the light this surface bounces, per channel (white = neutral, the
    /// full [`crate::light::BOUNCE`] fraction).
    pub bounce: Rgb,
    /// Gameplay: how tall it stands, in cells, for sun shadows ([`crate::light::Sun`]); 0 casts
    /// none. At most [`MAX_HEIGHT`].
    pub height: u8,
    /// Gameplay: under a roof, so never lit by the sun or the sky.
    pub roofed: bool,
    /// Presentation-only: whether blood stains it (spec §8 "Blood"); water, lava, and fire do
    /// not hold it.
    pub stains: bool,
    /// Gameplay: stone, worked or not (spec §8 "Searching"): dwarves notice the hidden features
    /// on it in passing (a secret door in a stone wall, a trap in a stone floor).
    pub stone: bool,
    /// Gameplay: how loud steps onto it are, in percent of the walker's footsteps
    /// ([`crate::noise::noise`]): water splashes (200), grass is soft (70).
    pub step_noise: u16,
    /// Gameplay: for a door, how loud opening it (a closed door) or closing it (an open one) is;
    /// 0 is silent.
    pub toggle_noise: u16,
    /// Gameplay: the loudness a sound loses entering the cell ([`crate::noise::propagate`]);
    /// `None` blocks sound (solid terrain without a `muffle`: walls, rock, trees). A closed door
    /// muffles.
    pub muffle: Option<u16>,
    /// Gameplay: whether it breaks (dug through with a tool, blasted open by an explosion), how
    /// hard it is, and what it becomes; `None`: it never breaks (doors, trees, mountains). Cells
    /// on a level's edge never break, whatever their terrain.
    pub breaks: Option<Breakable>,
    /// Gameplay: solid rock that shows as this terrain once a cell next to it is broken open
    /// (rock bared by a dig shows its face, a wall), as generated levels turn rock that touches
    /// open ground into walls.
    pub face: Option<TerrainId>,
    /// Gameplay: water that freezes, and how ([`crate::ice`]).
    pub freeze: Option<Freeze>,
    /// Gameplay: ice of at least this many mm looks like this terrain (thin ice, ice, thick ice),
    /// up to the next such terrain's; it covers water ([`crate::ice`]).
    pub ice: Option<u16>,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: strength of the heat haze rising above the cell, 0 to 1.
    pub haze: f32,
    /// Presentation-only: glyph particle emitters.
    pub particles: Vec<ParticleEmitter>,
}

/// How a wall breaks (spec §8 "Digging"): dug through with a tool, or blasted open by an
/// explosion whose blast is greater than its hardness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakable {
    /// The work it takes to dig through ([`crate::world::dig_work`]: a pick-axe at Strength 10
    /// does 10 a dig), so a wall of hardness 600 takes a pick-axe 60 digs at Strength 10. An
    /// explosion breaks it when its blast is greater.
    pub hardness: u16,
    /// What it becomes: walkable, transparent rubble.
    pub becomes: TerrainId,
}

impl TerrainDef {
    /// The energy price of stepping onto this terrain on a level of `kind` for an actor whose
    /// step costs `step` (spec §8 "Time"; the rules ask through
    /// [`crate::world::LevelState::step_price`]): on the overworld the step's cost over the
    /// terrain's `travel_minutes`; elsewhere the step's cost scaled by its `move_cost` percent
    /// over the 6-second base ([`BASE_TURN`]). The actor's speed decides how long it takes.
    pub fn step_price(&self, step: u32, kind: LevelKind) -> i64 {
        match (kind, self.travel_minutes) {
            (LevelKind::Overworld, Some(minutes)) => {
                price(step, Duration::minutes(u64::from(minutes)))
            }
            _ => (price(step, BASE_TURN) * i64::from(self.move_cost) / 100).max(1),
        }
    }

    /// Presentation-only: the glyph the map draws for this terrain at level cell `p`: one of its
    /// `variants` picked by a hash of the position (so neighbors differ, and a cell keeps its
    /// glyph), else its `glyph`. Look mode and dumps name it by `glyph`.
    pub fn glyph_at(&self, p: Pos) -> char {
        if self.variants.is_empty() {
            return self.glyph;
        }
        let h = (p.x as u32).wrapping_mul(0x9e37_79b1) ^ (p.y as u32).wrapping_mul(0x85eb_ca77);
        let h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
        self.variants[(h ^ (h >> 13)) as usize % self.variants.len()]
    }

    pub fn is_closed_door(&self) -> bool {
        self.opens_to.is_some()
    }

    pub fn is_open_door(&self) -> bool {
        self.closes_to.is_some()
    }

    /// A door, closed or open: what a trap on it belongs to (spec §8 "Trapped doors").
    pub fn is_door(&self) -> bool {
        self.is_closed_door() || self.is_open_door()
    }

    /// Running water: every cell of it has a [`super::Flow`] (presentation-only).
    pub fn flows(&self) -> bool {
        self.anim.as_ref().is_some_and(GlyphAnim::is_flow)
    }

    /// Ground one simply walks on: walkable, and neither stairs, a door, nor giving light (dev
    /// views mark such cells of a special room).
    pub fn is_plain_ground(&self) -> bool {
        self.walkable
            && self.stairs.is_none()
            && self.light.is_none()
            && !self.is_closed_door()
            && !self.is_open_door()
    }
}
