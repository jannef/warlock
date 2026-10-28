//! Per-level mutable state: terrain, locks, secret doors, piles of objects, traps, and blood as
//! they are now, and what the player remembers of them; fires, and the walls dug into; the
//! level's special rooms (spec §7 "Special rooms") and whether the player has been inside each;
//! its shops (spec §7 "Shops").

use super::curse::Rising;
use super::shop::Shop;
use super::spawn::Spawner;
use crate::blood::{Spatter, Stain};
use crate::chest::Chest;
use crate::color::Rgb;
use crate::content::{
    Content, EdgeExit, Flow, Here, HiddenLocation, LevelKind, LevelTemplate, LightId, RoomEffectId,
    Secret, SpecialRoom, StairsLink, TerrainDef, TerrainId, Trap, TrapId,
};
use crate::door::Lock;
use crate::geom::Pos;
use crate::grid::Grid;
use crate::ice::Ice;
use crate::light::Light;
use crate::object::Stack;
use crate::temperature::{self, Temperature};
use crate::time::GameTime;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Something burning on a cell (spec §8 "Ranged combat"): a burning object landed there (a
/// flaming arrow), giving light until it burns out and is gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fire {
    /// The light it gives, a preset.
    pub light: LightId,
    /// When it has burned out: it gives light while the clock is before this.
    pub until: GameTime,
}

/// A level that has been visited. Levels persist when the player leaves them.
///
/// Everything the rules need is kept here rather than looked up in the level's template, because
/// generated levels have no template in [`Content`] once they are instantiated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelState {
    pub(crate) id: String,
    pub(crate) name: String,
    /// Light everywhere on the level.
    pub(crate) ambient: Light,
    pub(crate) kind: LevelKind,
    pub(crate) terrain: Grid<TerrainId>,
    /// Terrain as last seen by the player; `None` where never seen.
    pub(crate) memory: Grid<Option<TerrainId>>,
    pub(crate) stairs: Vec<StairsLink>,
    /// Where the player arrives when no staircase leads back to where they came from.
    pub(crate) start: Option<Pos>,
    /// Where walking off the edge leads (local areas: back to their overworld cell).
    pub(crate) exit: Option<EdgeExit>,
    /// Lights placed on cells (overriding the terrain's own light), by position.
    #[serde(with = "by_pos")]
    pub(crate) lights: BTreeMap<Pos, LightId>,
    /// Flow of running water, by position. Presentation-only: no rule reads it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) flow: BTreeMap<Pos, Flow>,
    /// Piles of objects lying on cells, by position, each top first and never empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) objects: BTreeMap<Pos, Vec<Stack>>,
    /// Piles as the player last saw them, by position (spec §5 "Memory"): a pile seen taken
    /// away is forgotten, one that changed out of sight is remembered as it was.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) object_memory: BTreeMap<Pos, Vec<Stack>>,
    /// Burning objects lying on cells, by position: light sources until they burn out. The
    /// burning objects themselves lie in the pile there.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) fires: BTreeMap<Pos, Fire>,
    /// Locks on closed doors, by position (spec §8 "Doors"): gone once the door is unlocked or
    /// broken. Ground truth: a locked door looks like any closed door.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) locks: BTreeMap<Pos, Lock>,
    /// Chests, by position (spec §8 "Chests"): on every chest terrain cell, shut, open, or
    /// smashed, its level (its band follows) and, while shut, what it holds. Ground truth.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) chests: BTreeMap<Pos, Chest>,
    /// Secret doors not yet found, by position (spec §8 "Searching"): the cell's terrain is the
    /// wall each looks like; found, it becomes its door and is gone from here. Ground truth.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) secrets: BTreeMap<Pos, Secret>,
    /// Hidden locations not yet revealed, by position (spec §7 "Hidden locations"; the
    /// overworld only): the cell's terrain is the plain terrain each shows; revealed, it becomes
    /// its entrance and is gone from here. Ground truth.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) hidden: BTreeMap<Pos, HiddenLocation>,
    /// The breaches of sealed rooms the player has found (spec §7 "Special rooms": a greater
    /// treasure room's), by position: walls that sound hollow, which look mode names. Gone once
    /// the wall is broken.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) hollows: BTreeSet<Pos>,
    /// Blood on cells, by position (spec §8 "Blood"). Presentation-only: no rule reads it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) blood: BTreeMap<Pos, Stain>,
    /// Blood as the player last saw it, by position (like remembered piles).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) blood_memory: BTreeMap<Pos, Stain>,
    /// Walls dug into and still standing, by position (spec §8 "Digging"): the work done on each,
    /// in [`super::dig::WORK_SCALE`]ths of the work its hardness counts.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) dug: BTreeMap<Pos, u32>,
    /// Special rooms (spec §7 "Special rooms"), none overlapping another, each with whether the
    /// player has been inside it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) rooms: Vec<SpecialRoom>,
    /// The effect that covers the whole level, if one does (spec §7 "Special rooms": a level
    /// file's `effect`, or a generated level's rare draw).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pervading: Option<RoomEffectId>,
    /// The dead due to rise from their corpses (spec §7 "Special rooms": a cursed room;
    /// `world/curse.rs`), in the order they died.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) risings: Vec<Rising>,
    /// Traps, by position (spec §8 "Traps"), hidden or known; a spent one is gone.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) traps: BTreeMap<Pos, Trap>,
    /// Known traps as the player last saw them, by position: a trap seen gone is forgotten.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) trap_memory: BTreeMap<Pos, TrapId>,
    /// The temperature here, underground: fixed. `None` under the open sky, where it follows
    /// the month, the time of day, and the weather ([`crate::temperature`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) temperature: Option<Temperature>,
    /// Ice on water, by position (spec §5 "Ice"; [`crate::ice`]): its thickness and the water
    /// beneath. A cell shows the ice terrain its thickness reaches, else the water.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", with = "by_pos")]
    pub(crate) ice: BTreeMap<Pos, Ice>,
    /// When the ice was last brought up to date: a tick of the clock ([`crate::ice::TICK`]).
    pub(crate) ice_until: GameTime,
    /// Shops (spec §7 "Shops"), none overlapping another: who keeps each, what the player owes
    /// there, and how its keeper stands toward them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) shops: Vec<Shop>,
    /// What comes to the level over time (spec §7 "Danger levels"; `world/spawn.rs`): only on a
    /// level with a danger rule, so every other level's state is as it was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) spawner: Option<Spawner>,
}

impl LevelState {
    /// The level as it starts at `now`: its template's terrain, things, and ice.
    pub(crate) fn from_template(
        template: &LevelTemplate,
        content: &Content,
        now: GameTime,
    ) -> Self {
        let terrain = template.terrain.clone();
        let memory = Grid::new(terrain.width(), terrain.height(), None);
        let ice = (template.ice.iter())
            .map(|(&p, &um)| {
                let water = terrain[p];
                (p, Ice { water, um })
            })
            .collect();
        let mut level = Self {
            id: template.id.clone(),
            name: template.name.clone(),
            ambient: template.ambient,
            kind: template.kind,
            terrain,
            memory,
            stairs: template.stairs.clone(),
            start: template.player_start,
            exit: template.exit.clone(),
            lights: template.lights.clone(),
            flow: template.flow.clone(),
            objects: template.objects.clone(),
            object_memory: BTreeMap::new(),
            fires: BTreeMap::new(),
            locks: template.locks.clone(),
            chests: template.chests.clone(),
            secrets: template.secrets.clone(),
            hidden: template.hidden.clone(),
            hollows: BTreeSet::new(),
            blood: BTreeMap::new(),
            blood_memory: BTreeMap::new(),
            dug: BTreeMap::new(),
            rooms: template.rooms.clone(),
            pervading: template.pervading,
            risings: Vec::new(),
            shops: Vec::new(),
            traps: template.traps.clone(),
            trap_memory: BTreeMap::new(),
            temperature: (!template.kind.is_sky_exposed()).then(|| {
                (template.temperature)
                    .unwrap_or(Temperature::degrees(temperature::UNDERGROUND.into()))
            }),
            ice,
            ice_until: crate::ice::tick_floor(now),
            spawner: None,
        };
        let iced: Vec<Pos> = level.ice.keys().copied().collect();
        for p in iced {
            level.show_ice(content, p);
        }
        level
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn width(&self) -> i32 {
        self.terrain.width()
    }

    pub fn height(&self) -> i32 {
        self.terrain.height()
    }

    pub fn kind(&self) -> LevelKind {
        self.kind
    }

    /// The energy price of stepping onto `terrain` here for an actor whose step costs `step`
    /// (its cost of [`crate::content::ActionKind::Step`]). This is how the rules decide movement
    /// time (spec §8 "Time"; [`TerrainDef::step_price`]): on local levels a step is the step's
    /// cost scaled by the terrain's `move_cost` percent over the 6-second base; on the overworld
    /// it is the step's cost over the terrain's `travel_minutes` (content validation guarantees
    /// walkable overworld terrain has them). The actor's speed then decides how long it takes.
    pub fn step_price(&self, terrain: &TerrainDef, step: u32) -> i64 {
        terrain.step_price(step, self.kind)
    }

    /// The terrain at `p` as it is now (ground truth, `None` out of bounds). Only reachable
    /// through [`crate::World::ground_truth`], **for developer tools only**.
    pub fn terrain_at(&self, p: Pos) -> Option<TerrainId> {
        self.terrain.get(p).copied()
    }

    /// The pile at `p` as it is now, top first (ground truth, empty where nothing lies). Only
    /// reachable through [`crate::World::ground_truth`], **for developer tools only**.
    pub fn objects_at(&self, p: Pos) -> &[Stack] {
        self.objects.get(&p).map_or(&[], Vec::as_slice)
    }

    /// The trap at `p` as it is now, hidden or known (ground truth). Only reachable through
    /// [`crate::World::ground_truth`], **for developer tools only**.
    pub fn trap_at(&self, p: Pos) -> Option<&Trap> {
        self.traps.get(&p)
    }

    /// The lock on the door at `p`, if it is locked (ground truth, `None` for any other cell).
    /// Only reachable through [`crate::World::ground_truth`] outside the rules, **for developer
    /// tools only**: the player learns a door is locked by trying it.
    pub fn lock_at(&self, p: Pos) -> Option<&Lock> {
        self.locks.get(&p)
    }

    /// The chests on the level, by position (ground truth: what each holds). Only reachable
    /// through [`crate::World::ground_truth`] outside the rules, **for developer tools only**.
    pub fn chests(&self) -> &BTreeMap<Pos, Chest> {
        &self.chests
    }

    /// The secret door at `p`, if one is there and not yet found (ground truth: it looks like
    /// the wall around it). Only reachable through [`crate::World::ground_truth`] outside the
    /// rules, **for developer tools only**.
    pub fn secret_at(&self, p: Pos) -> Option<&Secret> {
        self.secrets.get(&p)
    }

    /// Whether `p` is a sealed room's breach still standing (spec §7 "Special rooms"): the wall a
    /// greater treasure room is guaranteed to open by, found or not. Ground truth.
    pub fn breach_at(&self, content: &Content, p: Pos) -> bool {
        self.rooms.iter().any(|r| r.breach == Some(p)) && !self.is_walkable(content, p)
    }

    /// The sealed rooms' breaches the player has found on this level, by position.
    pub fn hollows(&self) -> &BTreeSet<Pos> {
        &self.hollows
    }

    /// The hidden location at `p`, if one is there and not yet revealed (ground truth: the cell
    /// is the plain terrain it shows). Only reachable through [`crate::World::ground_truth`]
    /// outside the rules, **for developer tools only**.
    pub fn hidden_at(&self, p: Pos) -> Option<&HiddenLocation> {
        self.hidden.get(&p)
    }

    /// Blood of `color` lands here at `now`: on a cell whose terrain holds it, it adds to the
    /// stain there (spec §8 "Blood"); water, lava, and the like take none.
    pub(crate) fn stain(&mut self, content: &Content, spatter: Spatter, color: Rgb, now: GameTime) {
        let holds = (self.terrain.get(spatter.at)).is_some_and(|t| content.terrain(*t).stains);
        if !holds || spatter.amount == 0 {
            return;
        }
        let Spatter { at, amount, face } = spatter;
        self.blood
            .entry(at)
            .and_modify(|s| s.add(amount, color, now, face))
            .or_insert_with(|| Stain::new(amount, color, now, face));
    }

    /// Replaces the pile at `p`; an empty pile removes it.
    pub(crate) fn set_pile(&mut self, p: Pos, pile: Vec<Stack>) {
        if pile.is_empty() {
            self.objects.remove(&p);
        } else {
            self.objects.insert(p, pile);
        }
    }

    /// The special rooms of the level as they are now (ground truth: whether the player has been
    /// inside each). Only reachable through [`crate::World::ground_truth`], **for developer tools
    /// only**; the player knows the rooms they have entered
    /// ([`crate::perception::PerceptionView::room`]).
    pub fn rooms(&self) -> &[SpecialRoom] {
        &self.rooms
    }

    /// The shops of the level as they are now (ground truth: what the player owes, how each
    /// keeper stands). Only reachable through [`crate::World::ground_truth`], **for developer
    /// tools only**; the player knows a shop by what they see of it
    /// ([`crate::perception::PerceptionView::shop_at`]).
    pub fn shops(&self) -> &[Shop] {
        &self.shops
    }

    /// The special room at `p`, if any.
    pub(crate) fn room_at(&self, p: Pos) -> Option<&SpecialRoom> {
        self.rooms.iter().find(|r| r.contains(p))
    }

    /// The effect that covers the whole level, if one does (ground truth; the player is told of
    /// it on arrival).
    pub fn pervading(&self) -> Option<RoomEffectId> {
        self.pervading
    }

    /// What the special rooms do at `p` (spec §7 "Special rooms"): the effect of the room there,
    /// then the level's. The one question the rules ask of special rooms (light, creature sight,
    /// noise, regeneration, creatures that shun a sanctuary). [`crate::World::effects_at`] asks
    /// it of any level.
    pub(crate) fn effects_at<'c>(&self, content: &'c Content, p: Pos) -> Here<'c> {
        let level = self.pervading.map(|id| content.room_effect(id));
        if self.rooms.is_empty() {
            return Here { room: None, level };
        }
        let room = self.room_at(p).map(|r| content.room_effect(r.effect));
        Here { room, level }
    }

    /// `light` at `p` as the special rooms there leave it: a shadowed room keeps only part of it
    /// (spec §5 "Special rooms"). Every part of a cell's light goes through this: its base light
    /// and each source's contribution, so they still add up to the whole.
    pub(crate) fn room_shade(&self, content: &Content, p: Pos, light: Light) -> Light {
        self.effects_at(content, p).shade(light)
    }

    /// The glow the special rooms add at `p` (a sanctuary's), lighting it like ambient light.
    pub(crate) fn room_glow(&self, content: &Content, p: Pos) -> Light {
        self.effects_at(content, p).glow()
    }

    /// Opaque cells block sight. Out-of-bounds counts as opaque.
    pub(crate) fn is_opaque(&self, content: &Content, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_none_or(|t| !content.terrain(*t).transparent)
    }

    pub(crate) fn is_walkable(&self, content: &Content, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_some_and(|t| content.terrain(*t).walkable)
    }

    /// Whether a door, closed or open, is at `p` ([`crate::content::TerrainDef::is_door`]).
    pub(crate) fn is_door(&self, content: &Content, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_some_and(|t| content.terrain(*t).is_door())
    }

    /// Whether a shut chest is at `p` ([`crate::content::TerrainDef::is_shut_chest`]).
    pub(crate) fn is_shut_chest(&self, content: &Content, p: Pos) -> bool {
        self.terrain
            .get(p)
            .is_some_and(|t| content.terrain(*t).is_shut_chest())
    }

    pub(crate) fn stairs_at(&self, p: Pos) -> Option<&StairsLink> {
        self.stairs.iter().find(|s| s.pos == p)
    }

    /// Whether every terrain, light, object, room effect, and trap id is valid for `content`,
    /// and the special rooms are well formed and apart (a guard for loaded saves).
    pub(crate) fn ids_valid_for(&self, content: &Content) -> bool {
        let effects = content.room_effects().defs().len();
        let rooms_ok = self.rooms.iter().enumerate().all(|(i, r)| {
            usize::from(r.effect.0) < effects
                && r.is_valid_within(self.width(), self.height())
                && self.rooms[..i].iter().all(|o| !o.overlaps(r))
        }) && self.pervading.is_none_or(|e| usize::from(e.0) < effects);
        let terrain = content.terrain_defs().len();
        let lights = content.light_presets().len();
        let traps = content.trap_defs().len();
        let known = |t: &TerrainId| usize::from(t.0) < terrain;
        let held = self.chests.values().map(|c| &c.holds);
        let piles = (self.objects.values().chain(self.object_memory.values())).chain(held);
        let mut trap_kinds =
            (self.traps.values().map(|t| t.kind)).chain(self.trap_memory.values().copied());
        let water = |t: &TerrainId| known(t) && content.terrain(*t).freeze.is_some();
        self.terrain.cells().iter().all(known)
            && self.memory.cells().iter().flatten().all(known)
            && self.lights.values().all(|l| usize::from(l.0) < lights)
            && self.fires.values().all(|f| usize::from(f.light.0) < lights)
            && piles.flatten().all(|s| s.is_valid_for(content))
            && rooms_ok
            && trap_kinds.all(|t| usize::from(t.0) < traps)
            && (self.ice.iter()).all(|(p, i)| water(&i.water) && self.terrain.in_bounds(*p))
            && (self.shops.iter()).all(|s| s.is_valid_for(content, self.width(), self.height()))
            && (self.hidden.iter()).all(|(p, h)| known(&h.terrain) && self.terrain.in_bounds(*p))
            && (self.risings.iter()).all(|r| r.is_valid_for(content))
            && (self.chests.keys()).all(|p| self.terrain.in_bounds(*p))
    }
}

/// Serde adapter: JSON object keys must be strings, so maps by position (placed lights, flow,
/// piles) are written as a list of `[{x, y}, value]` pairs in position order.
mod by_pos {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer, V: Serialize>(
        map: &BTreeMap<Pos, V>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        s.collect_seq(map.iter())
    }

    pub fn deserialize<'de, D: Deserializer<'de>, V: Deserialize<'de>>(
        d: D,
    ) -> Result<BTreeMap<Pos, V>, D::Error> {
        let pairs: Vec<(Pos, V)> = Vec::deserialize(d)?;
        Ok(pairs.into_iter().collect())
    }
}
