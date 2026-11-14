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
//! on their level, a **second viewpoint** ([`BondView`]): the cells the companion sees by its own
//! line of sight and eyes ([`crate::sight::sees_by`], creature sight's rule) are seen, with wall
//! completion around them, and so are the cells of the hostiles its nose finds
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
use crate::chest::{ChestLook, ChestState};
use crate::color::Rgb;
use crate::combat::Health;
use crate::content::Shops;
use crate::content::{Choice, DialogueId, ResourceId, TalentId, TreeId};
use crate::content::{
    CombatStance, Content, CreatureId, DamageTypeId, DiseaseId, FactionId, Factor, Flow,
    ItemClassId, LevelKind, LightId, MaterialId, ObjectId, Place, RaceDef, RoomEffectDef, SlotDef,
    SlotId, Stance, StatusId, TerrainId, TrapId, VisionProfile, WeaponSkillId, ZoneKind,
};
use crate::daylight::{self, DayPhase};
use crate::equipment::Equipment;
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::grid::Window;
use crate::hunger::Hunger;
use crate::light::{
    Area, BOUNCE_STEPS, Bounce, Light, LightCache, Moon, SUN_KEY, ShadowSteps, Sky, Sun, Sunlight,
    falloff, lit_in_sight, moon_key, wall_faces,
};
use crate::noise::{self, PerceivedSound};
use crate::object::{Stack, Weight};
use crate::sight::{self, LineOfSight};
use crate::skills::{self, SkillId};
use crate::stats::PerStat;
use crate::talent::{self, Candidate, TalentMods};
use crate::temperature::Temperature;
use crate::time::GameTime;
use crate::weapon_skills::WeaponSkills;
use crate::weather::Weather;
use crate::world::mirrors::image_id;
use crate::world::surprise::creature_unaware;
use crate::world::{
    self, Actor, ActorId, ActorKind, Death, Due, LevelState, RollCount, Shop, SkillRoll,
    WorldState, count_of, favour, part_of, unpaid_parts,
};
use serde::Serialize;
use std::collections::BTreeMap;

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

/// One of the player's own alternative resources (spec §8 "Resources"), as the status bar and
/// the character screen show it: what it holds and its maximum. Never its pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnResource {
    pub resource: ResourceId,
    pub value: u16,
    pub max: u16,
    /// Its points are in one creature's guard (`marked` in data: Openings), the mark
    /// [`PerceptionView::own_mark`] names.
    pub marked: bool,
}

/// A grimoire the player could learn from (spec §8 "Spells": "Learning"): one teaching a spell
/// they do not know, or one of a spell they know below its last rank (a deepening, "Ranks"),
/// with the chance in words ("easy", "a struggle", "far beyond you", or "certain"). Never the
/// number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnStudy {
    /// The inventory stack, by position.
    pub item: usize,
    pub spell: crate::content::SpellId,
    /// The rank reading it would bring the spell to; `None` for a spell to learn.
    pub rank: Option<u8>,
    pub chance: &'static str,
}

/// A spell the player could write a scroll of (spec §8 "Scrolls": the wizard's Scribe), as the
/// scribe list shows it: what it costs them in mana (they know their own costs) and how long it
/// takes at a normal pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnScribe {
    pub spell: crate::content::SpellId,
    pub mana: i32,
    pub time: crate::time::Duration,
}

/// One of the player's own talents (spec §8 "Talents"), as the Talents page and the character
/// screen list it: its name for its choice and what it does in the
/// game's words. Never a rule's number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnTalent {
    pub id: TalentId,
    /// "Hardy", "Weapon focus (long blades)".
    pub name: String,
    pub tree: TreeId,
    /// The choice it was taken for, by id.
    pub choice: Option<String>,
    pub description: String,
    /// Given, not picked: a birth sign's gift (spec §8 "Birth sign") or a race's.
    pub granted: bool,
    /// Their race gives it (spec §8 "Talents": the humans' Versatile), a granted talent of the
    /// racial tree; `granted` too.
    pub race: bool,
    /// A birth sign's flaw.
    pub flaw: bool,
    /// Their class gives it (spec §8 "Class abilities"): a class ability, or a talent their
    /// class's table hands out at their level or below.
    pub class: bool,
    /// What it gives once in a life has been given (spec §8 "Birth sign": Rise from the ashes,
    /// risen): "(spent)".
    pub spent: bool,
    /// A forgotten altar gave it (spec §7 "Special rooms": Chosen, Forsaken).
    pub boon: bool,
    /// The background story gave it (spec §8 "Background story"): "(story)", "(story, flaw)".
    pub story: bool,
}

/// How fresh food smells to those who can tell (spec §8 "Birth sign": Hound's nose): a word,
/// never its age.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    /// Old: still fine to eat.
    Turning,
    Rotten,
}

impl Freshness {
    /// The word: `fresh`, `turning`, `rotten`.
    pub const fn word(self) -> &'static str {
        match self {
            Freshness::Fresh => "fresh",
            Freshness::Turning => "turning",
            Freshness::Rotten => "rotten",
        }
    }
}

/// A talent as the Talents page and the level-up screen list it (spec §8 "Talents"): whether
/// the player holds it, what it asks and whether each part holds, and the choices it may be
/// taken for now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TalentView {
    pub id: TalentId,
    pub name: String,
    pub tree: TreeId,
    pub choice: Option<Choice>,
    /// Whether the player holds it, for any choice.
    pub held: bool,
    /// Given, never picked.
    pub granted: bool,
    pub description: String,
    /// What it asks, in words, each with whether it holds now.
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
fn allies_of(state: &WorldState, content: &Content) -> Vec<AllyView> {
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

/// The second viewpoint of Hunter's bond and Familiar's sight (the module docs): the kept beast
/// on the player's level, its line of sight within its type's sight (on the overworld no farther
/// than daylight allows), its eyes, and how far its nose reaches for them.
struct BondView<'a> {
    beast: &'a Actor,
    los: LineOfSight,
    eye: VisionProfile,
    nose: Option<i32>,
}

/// The [`BondView`] of the beast the player's talents share, Hunter's bond the companion's and
/// Familiar's sight the familiar's, if it stands on their level (not in the retinue). No class
/// holds both talents; were one to, the lower id's would be the viewpoint.
fn bond_view<'a>(
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
    /// The campaign the run is of: whose chapters tell the player's story.
    pub(crate) campaign: &'a str,
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
    /// The patron's open commission, which the player knows (`world/patron.rs`).
    pub(crate) patron: &'a crate::world::patron::Patron,
    /// Everything the player was told, which they know.
    pub(crate) messages: &'a crate::messages::MessageLog,
    /// What the player knows of scroll types: how they name what they see.
    pub(crate) lore: crate::scroll_label::Lore<'a>,
    pub(crate) snapshot: &'a Perception,
    /// The overworld encounter the player must answer, which they know.
    pub(crate) encounter: Option<&'a crate::world::Pending>,
    /// The run's seed, for the freshness check (`world/freshness.rs`), which only draws a stream
    /// of its own.
    pub(crate) seed: u64,
}

impl<'a> PerceptionView<'a> {
    pub fn content(&self) -> &'a Content {
        self.content
    }

    /// Whether `stack` is a magic crystal, one the Consume list (`e`) offers beside food and
    /// potions (`data/crystals.toml`; the rare materials are not; `world/crystal.rs`).
    pub fn is_crystal(&self, stack: &crate::object::Stack) -> bool {
        (self.content.crystals()).is_some_and(|c| c.colour_of_object(stack.object).is_some())
    }

    /// What the player knows of scroll types (spec §8 "Scrolls"): every name presentation shows
    /// of a stack goes through it ([`crate::scroll_label::Lore::describe`]), so an unidentified
    /// scroll shows by its label.
    pub fn lore(&self) -> crate::scroll_label::Lore<'a> {
        self.lore
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
    /// The overworld encounter the player must answer now (spec §7 "Overworld encounters"),
    /// as the one dialog asks it: its title, text, and rows; `None` while none is pending. The
    /// app opens the dialog from it after any action and on load, and it cannot be closed
    /// without an answer (the owner, #796 Q9).
    pub fn encounter(&self) -> Option<crate::world::EncounterView> {
        let pending = self.encounter?;
        let (_, entry) = self
            .content
            .encounter_entry(self.campaign, &pending.entry)?;
        let id = crate::content::local_area_id(pending.cell);
        let area = self.content.local_area(self.campaign, &id);
        let choices = crate::world::encounter_choices(pending, entry, self.player, self.content);
        Some(crate::world::encounter_view(
            pending,
            &entry.name,
            area.map(|a| a.landscape),
            &choices,
        ))
    }

    /// The pending encounter's entry id and outcome (dumps); `None` while none is pending.
    pub fn encounter_pending(&self) -> Option<(&'a str, crate::world::EncounterOutcome)> {
        self.encounter.map(|p| (p.entry.as_str(), p.outcome))
    }

    pub fn region(&self) -> Option<&'a str> {
        let (level, at) = match &self.level.exit {
            Some(exit) => (exit.to.as_str(), exit.pos),
            None => (self.level.id.as_str(), self.snapshot.viewer),
        };
        self.content.level(level)?.region_at(at)
    }

    /// The effect covering the whole level (spec §7 "Special rooms"), which the player was told
    /// of on arrival.
    pub fn pervading(&self) -> Option<&'a RoomEffectDef> {
        (self.level.pervading).map(|id| self.content.room_effect(id))
    }

    /// The special room at `p` (spec §7 "Special rooms"), if the player knows it: they have been
    /// inside it, and `p` is a cell they see or remember.
    pub fn room(&self, p: Pos) -> Option<&'a RoomEffectDef> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let room = self.level.room_at(p)?;
        room.entered.then(|| self.content.room_effect(room.effect))
    }

    /// Whether `p` is hallowed ground (Sanctuary, `world/sanctuary.rs`), and whether the dark
    /// powers laid it (`Some(true)`: dreaded ground): the player laid it, so they know it where
    /// they see or remember the cell.
    pub fn hallow(&self, p: Pos) -> Option<bool> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        self.level.hallows_at(p, self.time).next().map(|h| h.dark)
    }

    /// The zone on `p` as the player knows it (`world/zone.rs`): what lies there where they see
    /// the cell, what they last saw there where they remember it, and nothing on a hidden cell.
    pub fn zone_at(&self, p: Pos) -> Option<ZoneKind> {
        match self.cell(p) {
            CellView::Unknown => None,
            CellView::Visible(_) => self.snapshot.zones.get(&p).copied(),
            CellView::Remembered(_) => self.level.zone_memory.get(&p).copied(),
        }
    }

    /// The percent of all light a special room keeps at a visible cell (a shadowed room's
    /// darkness, which the player sees; 100 elsewhere), for light that presentation adds of its
    /// own (a burning missile in flight); `None` for cells not in view.
    pub fn light_kept(&self, p: Pos) -> Option<u16> {
        if !self.snapshot.is_visible(p) {
            return None;
        }
        Some(self.level.effects_at(self.content, p).light())
    }

    /// Where the stairs or location at `p` lead, by name — only for cells the player sees or
    /// remembers (the staircase itself is known; its destination is public knowledge).
    pub fn leads_to(&self, p: Pos) -> Option<String> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        let link = self.level.stairs_at(p)?;
        // A named location (a legend's `name`, a revealed hidden location's) goes by its own
        // name, even while it leads nowhere yet (spec §7 "Hidden locations").
        (link.name.clone()).or_else(|| self.content.level_display_name(link.to.as_deref()?))
    }

    /// The named place at `p` (a legend's `name`: a location, a landmark), if the player sees
    /// or remembers the cell.
    pub fn place(&self, p: Pos) -> Option<&'a Place> {
        if matches!(self.cell(p), CellView::Unknown) {
            return None;
        }
        self.level.place(self.content, p)
    }

    /// The locations the player knows on the overworld, by position, each with its name: the
    /// ways into other places on cells they see or remember (a hidden location only once
    /// revealed, when it is one of them). Empty on any other level.
    pub fn known_locations(&self) -> Vec<(Pos, String)> {
        if self.level.kind() != LevelKind::Overworld {
            return Vec::new();
        }
        let mut known: Vec<(Pos, String)> = (self.level.stairs.iter())
            .filter_map(|s| Some((s.pos, self.leads_to(s.pos)?)))
            .collect();
        known.sort_by_key(|(p, _)| (p.y, p.x));
        known
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
            // A disguised mimic shows a shut chest (`world/mimic.rs`).
            let shown = self.snapshot.disguise_at(p).map(|d| d.terrain);
            CellView::Visible(shown.unwrap_or(self.level.terrain[p]))
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

    /// Whether a corpse lies at `p` where the player sees it now (what a spell aimed at a corpse
    /// asks, `world/corpse_spell.rs`).
    pub fn corpse_at(&self, p: Pos) -> bool {
        self.snapshot.sees_actors_at(p)
            && (self.objects(p).iter()).any(|s| self.content.object(s.object).corpse.is_some())
    }

    /// The cells where the player sees a corpse now, nearest first (ties: row order).
    pub fn corpses_in_sight(&self) -> Vec<Pos> {
        let me = self.viewer();
        let mut cells: Vec<Pos> = (self.level.objects.keys().copied())
            .filter(|&p| self.corpse_at(p))
            .collect();
        cells.sort_by_key(|p| (p.dist2(me), p.y, p.x));
        cells
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

    /// The char at `p` as the player knows it (spec §8 "Blood", [`crate::scorch`]): by
    /// [`Self::blood`]'s rules.
    pub fn scorch(&self, p: Pos) -> Option<&'a Stain> {
        if self.snapshot.sees_scorch_at(p) {
            self.level.scorch.get(&p)
        } else if self.snapshot.is_visible(p) {
            None
        } else {
            self.level.scorch_memory.get(&p)
        }
    }

    /// The chest at `p` as the player sees or remembers it (spec §8 "Chests"): its band and
    /// whether it is shut, open, or smashed (as the cell looks to them), and, with Chest-wise,
    /// on a cell in sight, whether a shut one is locked. Never its level or what it holds.
    /// `None` where they know no chest (or without chest bands).
    pub fn chest_at(&self, p: Pos) -> Option<ChestLook> {
        let terrain = match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => t,
            CellView::Unknown => return None,
        };
        let state = self.content.terrain(terrain).chest?;
        // A disguised mimic's chest is of its level, as the chest it stands in for would be; a
        // chest remembered where none stands now (a mimic found out out of sight) is of the
        // floor's.
        let level = match (self.level.chests.get(&p), self.snapshot.disguise_at(p)) {
            (Some(chest), _) => chest.level,
            (None, Some(d)) => d.level,
            (None, None) => self.content.item_level(&self.level.id),
        };
        let band = self.content.chests().band_index(level)?;
        let wise =
            || !self.player.talents.is_empty() && self.player.talent_mods(self.content).chest_wise;
        let locked = (state == ChestState::Shut && self.snapshot.is_visible(p) && wise())
            .then(|| self.level.locks.contains_key(&p));
        Some(ChestLook {
            band,
            state,
            locked,
        })
    }

    /// What the statue at `p` was, as the player sees or remembers it (spec §8 "Statues"): the
    /// creature type a stoned statue shows (its shape is plain to see), never what it holds.
    /// `None` where they know no statue of the stoned.
    pub fn statue_at(&self, p: Pos) -> Option<crate::content::CreatureId> {
        let terrain = match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => t,
            CellView::Unknown => return None,
        };
        self.content.terrain(terrain).statue?;
        self.level.statues.get(&p).map(|s| s.of)
    }

    /// What is cut in the gravestone at `p` (spec §7 "Graves"; `world/graves.rs`), where the
    /// player sees or remembers one: "Here lies Ælfric Wudu, 1381–1427." `None` elsewhere, or
    /// on a blank stone.
    pub fn epitaph(&self, p: Pos) -> Option<&'a str> {
        let terrain = match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => t,
            CellView::Unknown => return None,
        };
        self.content
            .terrain(terrain)
            .grave
            .then(|| self.level.graves.get(&p).map(String::as_str))
            .flatten()
    }

    /// Whom the statue at `p` is of, where it bears a past character's name from the graveyard
    /// (`world/graveyard.rs`): "Brakka the dwarf". `None` where they know no statue there, or
    /// it bears no name.
    pub fn statue_name(&self, p: Pos) -> Option<String> {
        self.statue_at(p)?;
        let statue = self.level.statues.get(&p)?;
        statue.name.as_ref().map(|f| f.words(self.content))
    }

    /// The leavings the player sees or remembers on this level (`content/leaving.rs`), in row
    /// order: statues, webs, and acid-pitted floor by the terrain they know, torn cloth by the
    /// pile they know. What each means is never told.
    pub fn known_leavings(&self) -> Vec<(Pos, crate::content::Leaving)> {
        let mut known = Vec::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let p = Pos::new(x, y);
                if let CellView::Visible(t) | CellView::Remembered(t) = self.cell(p)
                    && let Some(leaving) = self.content.leaving_terrain(t)
                {
                    known.push((p, leaving));
                }
                if (self.objects(p).iter()).any(|s| self.content.is_leaving_object(s.object)) {
                    known.push((p, crate::content::Leaving::TornCloth));
                }
            }
        }
        known
    }

    /// The chests the player sees or remembers on this level ([`PerceptionView::chest_at`]), by
    /// position.
    pub fn known_chests(&self) -> Vec<(Pos, ChestLook)> {
        let mut cells: Vec<Pos> = (self.level.chests.keys().copied())
            .chain(self.snapshot.disguises.iter().map(|d| d.pos))
            .collect();
        cells.sort_unstable();
        cells.dedup();
        (cells.into_iter())
            .filter_map(|p| Some((p, self.chest_at(p)?)))
            .collect()
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

    /// Whether the wall at `p` is one the player found sounds hollow (spec §7 "Special rooms": a
    /// greater treasure room's breach, found by searching), until it is broken.
    pub fn hollow(&self, p: Pos) -> bool {
        self.level.hollows.contains(&p)
    }

    /// The hollow walls the player has found on this level ([`PerceptionView::hollow`]), by
    /// position.
    pub fn known_hollows(&self) -> Vec<Pos> {
        self.level.hollows.iter().copied().collect()
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

    /// What item descriptions show the player of worth (spec §8 "Talents": Gem lore) and of
    /// make (an item's capacity, to a holder of Eye for the make or one trained in Appraise:
    /// docs/dev/enchanting.md), by their own abilities.
    pub fn appraisal(&self) -> crate::magic::describe::Appraisal {
        let talents = self.player.talent_mods(self.content);
        let appraise = self.content.skill_id("appraise");
        let trained = appraise.and_then(|s| self.player.skills.value(s));
        crate::magic::describe::Appraisal {
            gems: talents.gem_worth,
            capacity: talents.eye_for_make || trained.is_some_and(|v| v > 0),
        }
    }

    /// `stack`'s description as the player knows it (spec §6 "Item description"):
    /// [`crate::magic::describe::appraised`] with their eye for worth, a wand whose type they
    /// know saying what it does in their hands ([`PerceptionView::wand_does`]).
    /// With Eye for the make (the craftsperson's), what it reads of the item's make follows
    /// ([`crate::magic::describe::make_lines`]), after the capacity line, before a unique's
    /// power and lore.
    pub fn describe_item(&self, stack: &Stack) -> Vec<crate::magic::describe::Line> {
        use crate::magic::describe::{self, LineKind};
        let maker = self.character.map(|c| c.name.as_str());
        let mut lines =
            describe::appraised_for(stack, self.lore, self.appraisal(), Some(self.player), maker);
        if self.player.talent_mods(self.content).eye_for_make {
            let percent = crate::world::salvage_percent(self.content, self.player);
            let protected = crate::world::protected_in(self.content, self.quests, stack);
            let mends = self.player.talent_mods(self.content).make_do.is_some();
            let make = describe::make_lines(stack, self.lore, percent, protected, mends);
            let at = (lines.iter())
                .rposition(|l| !matches!(l.kind, LineKind::Power | LineKind::Lore))
                .map_or(lines.len(), |i| i + 1);
            lines.splice(at..at, make);
        }
        lines
    }

    /// What the wand `stack` does in the player's hands once they know its type (its rung comes
    /// with the type, the spell scaling design §7.4): `4d6 fire, radius 2`; `None` before, for
    /// anything else, and for a wand whose spell rolls nothing.
    pub fn wand_does(&self, stack: &Stack) -> Option<String> {
        crate::magic::describe::wand_does(stack, self.lore, self.player)
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

    /// The player's background story (spec §8 "Background story"), which they know: its
    /// paragraphs, told again from the content with the slots filled; empty for a character
    /// without one (a scenario's own, an old save) or a run without a character.
    pub fn story(&self) -> Vec<String> {
        (self.character)
            .map(|c| crate::background::retell(self.content, self.campaign, c))
            .unwrap_or_default()
    }

    /// The time on the clock.
    pub fn time(&self) -> GameTime {
        self.time
    }

    /// What of their inventory the player wears and wields, by slot.
    pub fn equipment(&self) -> &'a Equipment {
        &self.player.equipment
    }

    /// The slots the player has, in display order: every slot but another class's own (the
    /// aristocrat's extra rings, #1404; `Content::slots_for`).
    pub fn own_slots(&self) -> Vec<(SlotId, &'a SlotDef)> {
        let class = self.character.map(|c| c.spec.class.as_str());
        self.content.slots_for(class).collect()
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

    /// Where in their inventory the wand the player has ready is (spec §8 "Magic items":
    /// "Wands"), if any.
    pub fn ready_wand(&self) -> Option<usize> {
        self.player.wand_item(self.content)
    }

    /// Where in their inventory the rod the player has ready is (spec §8 "Magic items":
    /// "Rods"), if any.
    pub fn ready_rod(&self) -> Option<usize> {
        self.player.rod_item(self.content)
    }

    /// What a use of the rod in inventory stack `item` would cost the player in mana and how
    /// far it would reach for them, if it is a rod whose type they know (spec §8 "Magic items":
    /// "Rods").
    pub fn own_rod(&self, item: usize) -> Option<(i32, i32)> {
        let stack = self.player.inventory.get(item)?;
        if !self.lore.is_identified(stack.object) {
            return None;
        }
        let rod = self.content.object(stack.object).rod?;
        let effect = self.content.rod_effect(rod.effect);
        let cost = self.player.rod_cost(self.content, &rod);
        Some((cost, self.player.rod_reach(self.content, effect)))
    }

    /// The digging tool the player has ready (spec §8 "Digging"), if any.
    #[cfg(test)]
    pub fn ready_to_dig(&self) -> Option<ObjectId> {
        self.player.dig_tool(self.content).map(|(tool, _)| tool)
    }

    /// Why the player cannot dig now, in the rules' words (no digging tool ready, or untrained
    /// in Mining: `world::dig_refusal`), or `None` when `u` may ask for a direction.
    pub fn dig_refusal(&self) -> Option<&'static str> {
        crate::world::dig_refusal(self.player, self.content)
    }

    /// The carried light the player has ready (spec §5 "Light sources": a lantern in the Tool
    /// slot), by its inventory position, lit or not: `u` uses it at once, without a direction.
    pub fn ready_light(&self) -> Option<usize> {
        self.player.ready_light(self.content)
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

    /// How far the player throws inventory stack `item` in a thrown exploit (its throw range,
    /// no farther than their sneak dice reach); `None` for what cannot be thrown so (a flask).
    pub fn exploit_throw_range(&self, item: usize) -> Option<i32> {
        self.player.exploit_throw_range(item, self.content)
    }

    /// Whether inventory stack `item` is made for throwing (what the throw choice lists): a
    /// throwing weapon, or a potion, which shatters where it lands (spec §8 "Potions": "Flasks").
    pub fn is_throwing_weapon(&self, item: usize) -> bool {
        (self.player.inventory.get(item)).is_some_and(|s| {
            let def = self.content.object(s.object);
            def.is_thrown() || def.shatters()
        })
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

    /// What the player may treat now with the Heal skill (spec §8 "Skills"), each by how they
    /// know it ("Sickness" for a disease they cannot name yet) and the action that treats it:
    /// the Heal ability's choice. Their own first, then their allies' beside them ("Your
    /// wolf's bleeding").
    pub fn own_ailments(&self) -> Vec<crate::heal::OwnAilment> {
        let mut list = crate::heal::own(self.player, self.content);
        list.extend(self.snapshot.ally_ailments.iter().cloned());
        list
    }

    /// The status effects that show on seen creature `actor` (`seen` in `data/status.toml`:
    /// bleeding), the most important first; none for creatures not in view and the player.
    pub fn effects_of(&self, actor: ActorId) -> impl Iterator<Item = StatusId> + 'a {
        (self.snapshot.effects.iter())
            .filter(move |(a, _)| *a == actor)
            .map(|(_, s)| *s)
    }

    /// The shapes a spirit weapon may be called in (the Hunter's Spirit weapon,
    /// `use:spirit_weapon:ID`): every melee weapon there is, by weapon skill then id. They are
    /// the game's objects, nothing of the run.
    pub fn weapon_shapes(&self) -> Vec<ObjectId> {
        crate::world::weapon_shapes(self.content)
    }

    /// The player's abilities, in menu order (grouped by source, [`crate::ability`]), each with
    /// why it cannot be used now, if it cannot: what the abilities menu (#156) lists. They know
    /// what they can do and what they hold.
    pub fn abilities(&self) -> Vec<OwnAbility> {
        let here = Ability::here(self.level, self.content, self.player.pos);
        let escorts = (self.snapshot.allies.iter()).any(|a| a.source.is_escort());
        let orders = Ability::orders(
            !self.snapshot.allies.is_empty(),
            self.kept().next().is_some() || escorts,
        );
        let allies_here = self.snapshot.allies.iter().any(|a| a.here);
        let companion_here = (self.snapshot.allies.iter())
            .any(|a| a.here && matches!(a.source, crate::world::AllySource::Companion(_)));
        let beside = |ability| self.ways(crate::beside::Beside::Use(ability));
        (Ability::of(self.player, self.content).into_iter())
            .chain(here)
            .chain(orders)
            // Listen and Wedge only beside a closed door (the owner, #1327).
            .filter(|&ability| {
                !matches!(ability, Ability::Listen | Ability::Wedge)
                    || beside(ability).is_some_and(|ways| !ways.is_empty())
            })
            .map(|ability| {
                let unavailable = self.unavailable(ability, allies_here, companion_here);
                // One aimed in a direction with nothing beside it may act on (#1327).
                let no_target = (unavailable.is_none())
                    .then(|| beside(ability))
                    .flatten()
                    .is_some_and(|ways| ways.is_empty());
                let target = crate::beside::Beside::Use(ability).target(self.content);
                OwnAbility {
                    ability,
                    unavailable: unavailable.or(target
                        .filter(|_| no_target)
                        .map(crate::ability::Unavailable::NoTarget)),
                }
            })
            .collect()
    }

    /// Why the player cannot use `ability` now, before what is beside them is asked
    /// ([`PerceptionView::abilities`]).
    fn unavailable(
        &self,
        ability: Ability,
        allies_here: bool,
        companion_here: bool,
    ) -> Option<crate::ability::Unavailable> {
        match ability {
            // An order needs someone on the level to hear it (spec §8 "Allies").
            Ability::Heel | Ability::Hold | Ability::SetOn if !allies_here => {
                Some(crate::ability::Unavailable::NoAllies)
            }
            // Give needs the beast at hand (`world/collar.rs`).
            Ability::Give if self.snapshot.kept_beside.is_empty() => {
                Some(crate::ability::Unavailable::NotBeside)
            }
            // Decipher needs a scroll of an unidentified type (`world/decipher.rs`).
            Ability::Decipher
                if !(self.player.inventory.iter())
                    .any(|stack| crate::world::undeciphered(self.content, &self.lore(), stack)) =>
            {
                Some(crate::ability::Unavailable::NoUnknownScroll)
            }
            // Make do and mend needs a damaged thing carried, not worn (`world/make_do.rs`).
            Ability::Class(crate::content::ClassActive::MakeDo)
                if !crate::world::mends_any(self.player) =>
            {
                Some(crate::ability::Unavailable::NothingToMend)
            }
            // Sic 'em needs its beast on the level (`world/sic.rs`).
            Ability::Class(crate::content::ClassActive::Sic) if !companion_here => {
                Some(crate::ability::Unavailable::NoCompanion)
            }
            // Trapwright rigs on a level, never the overworld (`world/rig.rs`).
            Ability::Class(crate::content::ClassActive::Rig) => {
                (ability.unavailable(self.player, self.content)).or_else(|| {
                    (self.level.kind() == crate::content::LevelKind::Overworld)
                        .then_some(crate::ability::Unavailable::NotHere)
                })
            }
            // A companion is one at a time (spec §8 "Allies": Tame).
            Ability::Tame if self.companion().is_some() => {
                Some(crate::ability::Unavailable::HaveCompanion)
            }
            Ability::Spell(spell) => {
                crate::ability::spell_unavailable(self.player, self.content, spell)
            }
            // An ally beside the player may need tending (`world/heal.rs`).
            Ability::Heal if !self.snapshot.ally_ailments.is_empty() => (ability
                .unavailable(self.player, self.content))
            .filter(|&why| why != crate::ability::Unavailable::NothingToTreat),
            // Calling a familiar asks for none alive and no hostile in sight.
            Ability::Class(crate::content::ClassActive::CallFamiliar) => self
                .calling_barred()
                .or_else(|| ability.unavailable(self.player, self.content)),
            // Field forge sets an anvil on a level, never the overworld (`world/field_forge.rs`).
            Ability::Class(crate::content::ClassActive::FieldForge)
                if self.level.kind() == crate::content::LevelKind::Overworld =>
            {
                Some(crate::ability::Unavailable::NotHere)
            }
            // Foraging asks for the wild underfoot (spec §8 "Foraging").
            Ability::Forage
                if crate::world::forage_ground(self.content, self.level, self.player.pos)
                    .is_none() =>
            {
                Some(crate::ability::Unavailable::NotHere)
            }
            _ => ability.unavailable(self.player, self.content),
        }
    }

    /// The ways to the glowing fungus beside the player, in compass order (the Gather ability,
    /// `world/gather.rs`): the menu gathers at once from the one, and asks the way when there
    /// are more. The player sees what stands beside them.
    pub fn gather_ways(&self) -> Vec<crate::geom::Dir> {
        crate::world::fungus_ways(self.content, self.level, self.player.pos)
    }

    /// What keeps the player from calling a familiar now (`world/familiar.rs`): one alive,
    /// wherever it is (they know their own), or a hostile they see.
    pub(crate) fn calling_barred(&self) -> Option<crate::ability::Unavailable> {
        use crate::ability::Unavailable;
        let lives = (self.snapshot.allies.iter())
            .any(|a| matches!(a.source, crate::world::AllySource::Familiar(_)));
        let watched = (self.actors().iter())
            .any(|a| a.kind == ActorKind::Creature && self.stance(a) == Stance::Hostile);
        (lives.then_some(Unavailable::FamiliarLives)).or(watched.then_some(Unavailable::Watched))
    }

    /// The name the abilities menu gives `ability`: its own, but the priest's prayer by its
    /// highest tier as the face that answers them gives it ("Orison", "Whisper"; they know their
    /// band and their own talents).
    pub fn ability_name(&self, ability: Ability) -> std::borrow::Cow<'a, str> {
        match ability {
            Ability::Class(crate::content::ClassActive::Orison) => {
                match self.player.prayer_title(self.content) {
                    Some((name, _)) => name.into(),
                    None => ability.name(self.content),
                }
            }
            _ => ability.name(self.content),
        }
    }

    /// What the abilities menu says `ability` does: its own line, but the priest's prayer's
    /// that of its highest tier.
    pub fn ability_description(&self, ability: Ability) -> &'a str {
        match ability {
            Ability::Class(crate::content::ClassActive::Orison) => {
                (self.player.prayer_title(self.content))
                    .map_or_else(|| ability.description(self.content), |(_, d)| d)
            }
            _ => ability.description(self.content),
        }
    }

    /// The spells the player knows (spec §8 "Spells": learned, or their class gives them), in
    /// menu order.
    pub fn own_spells(&self) -> Vec<crate::content::SpellId> {
        self.player.known_spells(self.content)
    }

    /// The grimoires the player could learn from now (spec §8 "Spells": "Learning"), in pack
    /// order: those that teach a spell they do not know, and those of a spell they know below
    /// its last rank ("Ranks": a reading would deepen it), each with its chance in words; none
    /// for a character not trained in Decipher script. What the read list tells of a grimoire.
    pub fn studies(&self) -> Vec<OwnStudy> {
        let Some(rules) = self.content.study_rules() else {
            return Vec::new();
        };
        (self.player.inventory.iter().enumerate())
            .filter_map(|(item, stack)| {
                let spell = self.content.object(stack.object).grimoire?;
                let odds =
                    crate::world::study_odds(self.player, self.content, self.character, spell)?;
                Some(OwnStudy {
                    item,
                    spell,
                    rank: odds.rank,
                    chance: rules.words(odds.ways()),
                })
            })
            .collect()
    }

    /// The rank the player knows `spell` at (spec §8 "Spells": "Ranks"; they know their own
    /// spells), 1 to `MAX_SPELL_RANK`; `None` for a spell they do not know.
    pub fn own_spell_rank(&self, spell: crate::content::SpellId) -> Option<u8> {
        self.player.spell_rank(self.content, spell)
    }

    /// The spells the player could write a scroll of (spec §8 "Scrolls": Scribe), in menu
    /// order: those they know that a scroll holds, each with its price and time; empty without
    /// Scribe. What the scribe list offers.
    pub fn scribes(&self) -> Vec<OwnScribe> {
        let costs = self.content.action_costs();
        let turn = crate::content::price(
            costs.cost(crate::content::ActionKind::Scribe),
            crate::content::BASE_TURN,
        );
        (self.own_spells().into_iter())
            .filter(|&spell| self.content.scroll_of(spell).is_some())
            .filter_map(|spell| {
                let def = self.content.spell_of(spell)?;
                let mana = self.player.scribe_cost(self.content, def)?;
                let turns = crate::world::scribe_turns(self.content, def)?;
                let seconds = u64::try_from(turn).unwrap_or(0) * u64::from(turns) / 1000;
                Some(OwnScribe {
                    spell,
                    mana,
                    time: crate::time::Duration::seconds(seconds),
                })
            })
            .collect()
    }

    /// How many blank scrolls the player carries (spec §8 "Scrolls"): what Scribe writes on.
    pub fn blank_scrolls(&self) -> u32 {
        let blank = self.content.blank_scroll();
        (self.player.inventory.iter())
            .filter(|s| Some(s.object) == blank)
            .map(|s| s.count)
            .sum()
    }

    /// The recipes of profession `skill` the player sees, in path order, each with what it takes
    /// and why it cannot be made now (`world/craft.rs`): what the profession's screen lists.
    pub fn recipes(&self, skill: crate::skills::SkillId) -> Vec<crate::world::RecipeRow> {
        crate::world::recipe_rows(self.content, self.lore, self.level, self.player, skill)
    }

    /// The choices of generic recipe `recipe`, in order, each with the stacks carried that it may
    /// take (`world/craft_make.rs`): the forging wizard's steps. Empty for a recipe with none
    /// (or one not made yet).
    pub fn craft_steps(&self, recipe: crate::content::RecipeId) -> Vec<crate::world::CraftStep> {
        crate::world::craft_steps(self.content, self.lore, self.level, self.player, recipe)
    }

    /// [`Self::craft_steps`] as the picks made so far (in step order) narrow them: an
    /// Enchanting recipe's seals follow its crystals and item, its affix kept its item.
    pub fn craft_steps_given(
        &self,
        recipe: crate::content::RecipeId,
        picks: &[Option<usize>],
    ) -> Vec<crate::world::CraftStep> {
        crate::world::craft_steps_given(
            self.content,
            self.lore,
            self.level,
            self.player,
            recipe,
            picks,
        )
    }

    /// What `recipe` made with `picks` (inventory positions in step order, `None` a skipped
    /// optional) would make and use, in words; or why it cannot, as the rules would say it.
    pub fn craft_plan(
        &self,
        recipe: crate::content::RecipeId,
        picks: &[Option<usize>],
    ) -> Result<crate::world::CraftPlan, String> {
        crate::world::craft_plan(
            self.content,
            self.lore,
            self.level,
            self.player,
            recipe,
            picks,
        )
    }

    /// What the player's spell `spell` costs them now, in mana (spec §8 "Spells": they know
    /// what their own spells cost); `None` for a spell they cannot cast.
    pub fn own_spell_cost(&self, spell: crate::content::SpellId) -> Option<i32> {
        let def = self.content.spell_of(spell)?;
        Some((self.player).spell_cost(self.content, def))
    }

    /// What the player's weapon active `id` costs them in stamina now, their talents' discounts
    /// counted (they know their own costs, as a spell's mana).
    pub fn own_weapon_stamina(&self, id: crate::content::WeaponAbilityId) -> u16 {
        self.player.weapon_stamina(self.content, id)
    }

    /// The roll the player's spell `spell` makes now, harm or healing, as they would cast it:
    /// the rung their scores reach on its ladder, their talents' and items' hooks applied
    /// ([`crate::world::Actor::spell_roll`]), with its damage type or "healed" and the text the
    /// abilities menu shows (`4d6 fire`). They know their own spells (the owner's exception,
    /// the spell scaling design decision 11); `None` for one that rolls nothing.
    pub fn own_spell_roll(
        &self,
        spell: crate::content::SpellId,
    ) -> Option<crate::world::SpellRoll> {
        let def = self.content.spell_of(spell)?;
        (self.player).spell_roll_words(self.content, def, crate::world::CastBy::Own)
    }

    /// The faces of the player's spell `spell`'s dice that are rolled again (Sure hand: 1, so
    /// "Your 1s are rolled again."); 0 for none, and for a spell that rolls nothing.
    pub fn own_spell_reroll(&self, spell: crate::content::SpellId) -> u32 {
        let Some(def) = self.content.spell_of(spell) else {
            return 0;
        };
        if def.roll().is_none() {
            return 0;
        }
        self.player.spell_mods(self.content, def).reroll
    }

    /// How far the player's spell `spell` reaches now, its reach ladders climbed: the farthest
    /// cell it may be aimed at, and a burst's radius. They know their own spells.
    pub fn own_spell_reach(&self, spell: crate::content::SpellId) -> Option<crate::world::Reach> {
        let def = self.content.spell_of(spell)?;
        Some(self.player.spell_reach(self.content, def))
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

    /// The attacks the player has ready (spec §6 "Character screen"): the main hand's blow, the
    /// off hand's with two weapons, and the missile weapon's shot or throw, each with its to-hit,
    /// damage, and range, from the profile a strike would roll now in the fighting style in use.
    /// The one place the game shows a to-hit (the owner's exception to hidden numbers: the
    /// player's own attacks only); unidentified magic counts for nothing here.
    pub fn own_attacks(&self) -> Vec<crate::world::OwnAttack> {
        self.player.known_attacks(self.content)
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

    /// The player's alternative resources (spec §8 "Resources"), in resource order: each one
    /// they hold, what it holds now (at most its maximum), and its maximum. Never how fast it
    /// changes.
    pub fn own_resources(&self) -> Vec<OwnResource> {
        (self.player.resources.iter())
            .map(|pool| {
                let level = self.own_experience().level;
                let max = self.player.resource_max(self.content, pool.resource, level);
                OwnResource {
                    resource: pool.resource,
                    value: pool.value.min(max),
                    max,
                    marked: self.content.resource(pool.resource).marked,
                }
            })
            .collect()
    }

    /// The player's mark, the creature their openings are in (`world/mark.rs`), as they know it:
    /// "the kobold" while they see it, "an unseen foe" while they do not; `None` without one.
    /// Their own choice, so nothing hidden.
    pub fn own_mark(&self) -> Option<String> {
        let mark = self.player.mark?;
        let seen = (self.snapshot.actors.iter()).find(|a| a.id == mark);
        Some(seen.map_or_else(
            || "an unseen foe".to_string(),
            |a| crate::content::the(&self.content.creature(a.creature).name),
        ))
    }

    /// The player's level, experience total, and the total at which the next level comes (none
    /// at the highest level) (spec §8 "Experience"). All of it is shown to the player.
    pub fn own_experience(&self) -> Progress {
        let rules = self.content.experience();
        let factor = Factor::of(self.content, self.character.map(|c| &c.spec));
        // The level stops at the classes' cap (spec §8 "Prestige classes"): no next level
        // there, as at the curve's top.
        let cap = (self.character).map_or(rules.class_max(), |c| c.level_cap(self.content));
        let level = rules.level_for(self.experience, factor, cap);
        Progress {
            level,
            xp: self.experience,
            next: (level < cap).then(|| rules.threshold_for(level + 1, factor)),
        }
    }

    /// The level whose choices the level-up screen spends now: the first due's, else the
    /// player's own (the class of that level sets the rolls and the bonus trees, spec §8
    /// "Prestige classes").
    fn due_level(&self) -> u8 {
        (self.due()).map_or_else(|| self.own_experience().level, |d| d.level)
    }

    /// The choices a level still asks of the player (spec §8 "Talents"), if any: the level and
    /// how many talent picks are left. Nothing else may be done until they are made.
    pub fn due(&self) -> Option<Due> {
        self.progress.due.first().copied()
    }

    /// Where a level's skill rolls come from now (spec §8 "Skills"): the level-up screen names
    /// it (the owner's call: numbers on that screen).
    pub fn roll_count(&self) -> RollCount {
        world::roll_count(self.content, self.player, self.character, self.due_level())
    }

    /// What a level-up roll on trained skill `skill` would be: the dice and what set them (the
    /// level-up screen shows them); `None` for a skill the player is not trained in.
    pub fn skill_roll(&self, skill: SkillId) -> Option<SkillRoll> {
        let trained = self.player.skills.get(skill)?;
        let trades = self.player.talent_mods(self.content).trades.clone();
        Some(world::skill_roll(
            self.content,
            self.character,
            &trades,
            skill,
            trained,
        ))
    }

    /// Every level's choices still due, the lowest level's first (several levels gained in one
    /// turn are one sequence on the level-up screen).
    pub fn dues(&self) -> &[Due] {
        &self.progress.due
    }

    /// The trees the bonus pick due now is of (the `bonus_talents` of the class of the level
    /// that brought it); `None`: any.
    pub fn bonus_trees(&self) -> Option<Vec<TreeId>> {
        let (table, _) = self.character?.class_of(self.content, self.due_level())?;
        table.bonus_trees(self.content.talents())
    }

    /// Whether the player's own bursts spare them (spec §8 "Talents": Scroll savant), so a
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
        // The class abilities held: every table's (the prestige class's too), to its level.
        let class: Vec<crate::content::TalentId> = (self.character)
            .map(|c| c.class_levels(self.content, level))
            .unwrap_or_default()
            .into_iter()
            .flat_map(|(table, own)| table.abilities_to(book, own))
            .collect();
        // The rising is spent once it is used up: the first with no Second rising, else both.
        let risen = self.progress.happened(crate::world::RISEN);
        let twice = self.progress.happened(crate::world::RISEN_TWICE);
        let second = (self.player.talent_mods(self.content)).second_rising;
        let spent = |e: &crate::content::TalentEffect| match e {
            crate::content::TalentEffect::RiseFromAshes { .. } => risen && (!second || twice),
            crate::content::TalentEffect::SecondRising => twice,
            _ => false,
        };
        (self.player.talents.iter())
            .filter_map(|taken| {
                let id = book.id(&taken.id)?;
                let def = book.get(id);
                let boon = (self.player.boons.iter())
                    .any(|b| b.id == taken.id && b.choice == taken.choice);
                let story = boon
                    && self.character.is_some_and(|c| {
                        let choice = taken.choice.as_deref();
                        crate::background::gave(self.content, self.campaign, c, &taken.id, choice)
                    });
                Some(OwnTalent {
                    id,
                    name: def.name_for(self.content, taken.choice.as_deref()),
                    tree: def.tree,
                    choice: taken.choice.clone(),
                    description: def.description.clone(),
                    granted: def.granted && !book.tree(def.tree).class,
                    race: def.granted && Some(def.tree) == book.tree_id("racial"),
                    flaw: def.flaw,
                    class: class.contains(&id),
                    spent: def.effects.iter().any(&spent),
                    boon: boon && !story,
                    story,
                })
            })
            .collect()
    }

    /// Every talent of the game, tree by tree in display order (the Talents page and the pick
    /// list): whether the player holds it, its prerequisites with whether each holds, and the ways it may be taken now (none: not now). A birth sign's pick for another
    /// sign is left out: the stars a character was born under never change (spec §8 "Birth
    /// sign"); so is one that asks for a hidden weapon skill or style (Two Shields) the player
    /// has no marks in yet.
    pub fn talents(&self) -> Vec<TalentView> {
        use crate::content::Need;
        let content = self.content;
        let who = self.candidate();
        let born_for = |def: &crate::content::TalentDef| {
            (def.needs.iter())
                .filter(|n| matches!(n, Need::Animal(_) | Need::Element(_) | Need::Lucky))
                .all(|n| who.meets(content, &n, def.choice, None))
        };
        let table = content.weapon_skills();
        let found = |def: &crate::content::TalentDef| {
            (def.needs.iter()).all(|n| match n {
                Need::WeaponSkill(id, _) => who.weapon_skills.shows(table, id),
                _ => true,
            })
        };
        (content.talents().by_tree())
            .filter(|(_, def)| born_for(def) && found(def))
            .map(|(id, def)| {
                let needs = (def.needs.iter())
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
                    held: self.player.talents.has(&def.id),
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
        let room = (self.level.effects_at(self.content, self.player.pos)).burden();
        burden::load(self.player, self.own_race(), self.content, room)
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

    /// What the player's Knowledge tells of a creature they see (spec §8 "Skills": Knowledge),
    /// in words only: what its kind resists, is immune to, or is vulnerable to, in damage type
    /// order; nothing while their Knowledge does not stand above its level, and never what it
    /// wears or what has been done to it.
    pub fn known_resistances(
        &self,
        actor: &SeenActor,
    ) -> Vec<(DamageTypeId, crate::damage::ResistanceWord)> {
        match actor.kind {
            ActorKind::Player => Vec::new(),
            _ => (self.player).known_resistances(self.content, actor.creature),
        }
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
            && self.player.talent_mods(self.content).beast.beast_friend
            && !(self.progress.empathy_broken).contains(&self.content.creature(actor.creature).id)
        {
            return Stance::Neutral;
        }
        match self.content.ids().player {
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

    /// Whether the player averts their eyes from a gaze they see (`world/avert.rs`): they fight
    /// half-blind. Only that they do, never how much.
    pub fn averted(&self) -> bool {
        self.snapshot.averted
    }

    /// The player's allies (spec §8 "Allies"), wherever they are, in actor id order.
    pub fn allies(&self) -> &[AllyView] {
        &self.snapshot.allies
    }

    /// The player's companion among their allies (`world/companion.rs`), with its name and
    /// grade; `None` without one.
    pub fn companion(&self) -> Option<(&AllyView, &crate::world::CompanionData)> {
        (self.snapshot.allies.iter()).find_map(|a| match &a.source {
            crate::world::AllySource::Companion(data) => Some((a, data)),
            _ => None,
        })
    }

    /// The player's familiar among their allies (`world/familiar.rs`), with its name and
    /// grade; `None` without one.
    pub fn familiar(&self) -> Option<(&AllyView, &crate::world::CompanionData)> {
        (self.snapshot.allies.iter()).find_map(|a| match &a.source {
            crate::world::AllySource::Familiar(data) => Some((a, data)),
            _ => None,
        })
    }

    /// The player's kept beasts (a companion, a familiar; `AllySource::kept`), with their names
    /// and grades, in actor id order: what the status bar shows a line of each.
    pub fn kept(&self) -> impl Iterator<Item = (&AllyView, &crate::world::CompanionData)> {
        (self.snapshot.allies.iter()).filter_map(|a| a.source.data().map(|d| (a, d)))
    }

    /// The ways to the player's kept beasts and escorts beside them, in direction order: with
    /// two or more, tending one or giving one something asks which way (spec §8 "Allies"; the
    /// owner, #907 Q4).
    pub fn kept_beside(&self) -> &[crate::geom::Dir] {
        &self.snapshot.kept_beside
    }

    /// Whether an escort of the player's stands beside them (`world/escort.rs`): the Give list
    /// holds anything it could put on.
    pub fn escort_beside(&self) -> bool {
        self.snapshot.escort_beside
    }

    /// Whether the escort of quest `quest` is with the player, on their level or in their
    /// retinue (`world/escort.rs`): the quest log's status.
    pub fn escorting(&self, quest: &str) -> bool {
        (self.snapshot.allies.iter()).any(|a| {
            (a.here || a.retinue)
                && matches!(&a.source, crate::world::AllySource::Escort { quest: q, .. } if q == quest)
        })
    }

    /// How many allies are with the player: on their level or in their retinue.
    pub fn allies_with(&self) -> usize {
        (self.snapshot.allies.iter())
            .filter(|a| a.here || a.retinue)
            .count()
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

    /// The temperature of the air where the player is (spec §5 "Temperature"), which they feel,
    /// in dungeons and under the open sky alike, a hot or a cold room's included (the status bar
    /// shows it).
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

    /// How fresh `stack` smells to the player, if it is food and they can tell by smell (spec §8
    /// "Birth sign": Hound's nose); `None` otherwise.
    pub fn smell(&self, stack: &Stack) -> Option<Freshness> {
        if self.player.talents.is_empty() || !self.player.talent_mods(self.content).smell_rot {
            return None;
        }
        Some(match crate::rot::stage(self.content, stack, self.time)? {
            crate::rot::Stage::Fresh => Freshness::Fresh,
            crate::rot::Stage::Old => Freshness::Turning,
            crate::rot::Stage::Rotten | crate::rot::Stage::Gone => Freshness::Rotten,
        })
    }

    /// How fresh `stack`, a corpse or food the player has in front of them, looks to an eye
    /// trained in Appraise or Survival (spec §8 "Hunger"; `world/freshness.rs`): `fresh`, `old`,
    /// or `rotten` when their check passes, `None` otherwise.
    pub fn judge(&self, stack: &Stack) -> Option<&'static str> {
        let stage = crate::world::freshness::judged(
            self.content,
            self.player,
            self.seed,
            stack,
            self.time,
        )?;
        Some(stage.id())
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
        let offer = (self.shops).offer(self.content, self.shop.kind, stack, self.favour);
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

    /// Whether a step onto `to` would dive into deep water (spec §8 "Skills": Swim): from anything
    /// else, into water the player sees or remembers where no one they see is, and open to them:
    /// an untrained or a Strained player is refused by the rules, with no question to ask. Water
    /// they do not know is never stepped into: the rules stop them at it first (`world/swim.rs`).
    pub fn dives_at(&self, to: Pos) -> bool {
        let swim = |p: Pos| match self.cell(p) {
            CellView::Visible(t) | CellView::Remembered(t) => self.content.terrain(t).swim,
            CellView::Unknown => false,
        };
        swim(to)
            && !swim(self.viewer())
            && self.actor_at(to).is_none()
            && crate::world::shore_refusal(self.content, self.player, self.own_load().penalty())
                .is_none()
    }

    /// What the player owes, and to whom, if a step onto `to` leaves the open shop they stand in
    /// (onto ground they know is walkable): by a way out ([`ShopView::is_exit`]) the keeper turns
    /// them back, any other way is theft, so presentation asks first.
    pub fn owed_on_leaving(&self, to: Pos) -> Option<(ShopView<'a>, u32)> {
        let view = self.shop_here()?;
        let walkable = match self.cell(to) {
            CellView::Visible(t) | CellView::Remembered(t) => self.content.terrain(t).walkable,
            CellView::Unknown => false,
        };
        let owed = view.owed();
        (!view.contains(to) && walkable && owed > 0).then_some((view, owed))
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

/// How actor `a` looks by what it is made of (spec §8 "Animated objects"): a household thing by
/// its body, its object's glyph in its material's colour; one of a family by the piece in its
/// tier's `shown_by` slot, in that piece's colour (and by its glyph with `item_glyph`: a blade
/// by its weapon's); `None` for any other, or one whose piece is gone.
fn shape(a: &Actor, content: &Content) -> Option<Shape> {
    let def = content.creature(a.creature);
    if let Some(tier) = content.animated().family_tier(def.animated.as_ref()) {
        let slot = content.slot_id(tier.shown_by.as_deref()?)?;
        let stack = a.inventory.get(a.equipment.get(slot)?)?;
        let (bands, count) = bands(a, content);
        return Some(Shape {
            glyph: if tier.item_glyph {
                content.object(stack.object).glyph
            } else {
                def.glyph
            },
            fg: stack.fg(content),
            material: stack.parts(content).first()?.material,
            bands,
            count,
        });
    }
    let body = content.animated_body(a.creature)?;
    let stack = (a.inventory.iter()).find(|s| s.object == body.object)?;
    let material = stack.parts(content).first()?.material;
    Some(Shape {
        glyph: content.object(stack.object).glyph,
        fg: stack.fg(content),
        material,
        bands: [Rgb::BLACK; Shape::BANDS],
        count: 0,
    })
}

/// The distinct colours of what actor `a` wears and wields, in slot order (head to feet, more or
/// less: a helm over a hauberk over boots), the first [`Shape::BANDS`], and how many there are.
fn bands(a: &Actor, content: &Content) -> ([Rgb; Shape::BANDS], u8) {
    let mut bands = [Rgb::BLACK; Shape::BANDS];
    let mut count = 0;
    let worn = (a.equipment.iter()).filter_map(|(_, i)| a.inventory.get(i));
    for fg in worn.map(|s| s.fg(content)) {
        if count < Shape::BANDS && !bands[..count].contains(&fg) {
            bands[count] = fg;
            count += 1;
        }
    }
    (bands, count as u8)
}
