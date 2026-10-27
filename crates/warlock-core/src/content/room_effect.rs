//! Special rooms (`assets/data/room_effects.toml`, spec §7 "Special rooms"), ADOM-like: a room of
//! a generated level, or an area a hand-made level marks, can carry an effect that changes the
//! rules while one is inside it.
//!
//! An effect ([`RoomEffectDef`]) may keep only part of the light ([`RoomEffectDef::light`]: a
//! shadowed room), add a glow ([`RoomEffectDef::glow`]: a sanctuary's), make the noises made
//! inside louder or quieter ([`RoomEffectDef::noise`]: an echoing hall, a hushed chamber), speed
//! up regeneration ([`RoomEffectDef::regen`]) and mana's ([`RoomEffectDef::mana_regen`]), keep
//! creatures hostile to the player out ([`RoomEffectDef::shunned`]), and have generators lay a
//! floor of its own ([`RoomEffectDef::floor`]: a flooded room's shallow water). Rarely an effect
//! covers a whole level instead of one room ([`RoomEffectDef::level_chance`], a level file's
//! `effect`). The rules ask one question, [`crate::World::effects_at`] (inside the core, the level
//! state's `effects_at`), and fold what it returns, a [`Here`]: the room's effect and the level's.
//!
//! Which effect a generated room gets is drawn by the effects' own `weight`s (a dungeon may
//! favour some with fixed shares and ban others: `content/dungeon.rs`), among those the room is
//! eligible for ([`RoomEffectDef::eligible`]).
//!
//! An effect may scale everyone's carrying limit inside ([`RoomEffectDef::burden`]: a heavy hall
//! halves it; `WorldState::load`, the one burden hook).
//!
//! An effect may give everyone inside a status while they stay ([`RoomEffectDef::status`]: a
//! draining room's Ebbing; `World::dwell`), which eases by its own pace once they leave.
//!
//! Where a special room lies is a [`SpecialRoom`]: a rectangle (a room of the rooms generator, an
//! area of a level file) or a blob of cells (a cave's), kept in the level's state with whether
//! the player has been inside yet.
//!
//! Without the file there are no special rooms.

use super::{ContentError, StatusDef, StatusId, TerrainDef, TerrainId};
use crate::geom::Pos;
use crate::light::Light;
use crate::temperature::Temperature;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Index of an effect in [`RoomEffects::defs`]; assigned in file order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoomEffectId(pub u8);

/// The percent of a noise's loudness an effect may leave (spec §5 "Hearing").
pub const NOISE: RangeInclusive<u16> = 1..=1000;
/// The percent of the usual pace of regeneration an effect may give (spec §8 "Combat").
pub const REGEN: RangeInclusive<u16> = 1..=1000;
/// The most effects there can be.
pub const MAX_EFFECTS: usize = 32;
/// The weight an effect may have in the draw (spec §7 "Special rooms").
pub const WEIGHT: RangeInclusive<u8> = 1..=100;
/// The fewest seconds a room's status may take to ease a degree: the longest single action, so
/// it never lapses while one stays inside.
pub const DWELL_EASES: u32 = 10;
/// The percent of the carrying limit an effect may leave (spec §8 "Burden").
pub const BURDEN: RangeInclusive<u16> = 10..=1000;
/// The degrees Celsius an effect may add to the level's temperature (spec §7 "Special rooms").
pub const TEMPERATURE: RangeInclusive<i8> = -40..=40;
/// How many reflections a hall of mirrors may show of each actor inside (spec §7 "Special
/// rooms").
pub const IMAGES: RangeInclusive<u8> = 1..=8;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomEffectFile {
    /// Percent per room (or cave chamber) of a generated level.
    #[serde(default)]
    chance: u8,
    /// At most this many special rooms a generated level.
    #[serde(default = "one")]
    per_level: u8,
    #[serde(default)]
    effect: Vec<RoomEffectSpec>,
}

fn one() -> u8 {
    1
}

fn hundred() -> u16 {
    100
}

fn ten() -> u8 {
    10
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomEffectSpec {
    id: String,
    name: String,
    mark: char,
    enter: String,
    #[serde(default = "ten")]
    weight: u8,
    near: Option<String>,
    #[serde(default)]
    level_chance: u8,
    pervade: Option<String>,
    #[serde(default)]
    needs_danger: bool,
    #[serde(default)]
    needs_below: bool,
    #[serde(default)]
    min_cells: u16,
    max_cells: Option<u16>,
    #[serde(default = "hundred")]
    light: u16,
    #[serde(default)]
    glow: Light,
    #[serde(default = "hundred")]
    noise: u16,
    #[serde(default = "hundred")]
    regen: u16,
    #[serde(default = "hundred")]
    mana_regen: u16,
    #[serde(default)]
    shunned: bool,
    floor: Option<String>,
    status: Option<String>,
    swarm: Option<Swarm>,
    #[serde(default)]
    temperature: i8,
    #[serde(default = "hundred")]
    burden: u16,
    mirrors: Option<MirrorsSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MirrorsSpec {
    images: u8,
}

/// A hall of mirrors' reflections (spec §7 "Special rooms"; `world/mirrors.rs`): every actor
/// inside is shown to the player with this many images of it on free floor of the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mirrors {
    /// Reflections of each actor inside.
    pub images: u8,
}

/// A teeming den's numbers (`swarm = { per_cells = 4, ... }`, spec §7 "Special rooms"): the
/// generator fills such a room with sleepers of the floor's danger and their hoard
/// (`mapgen/den.rs`); they wake by noise, a blow, or a cry (`world/den.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Swarm {
    /// One sleeper for this many floor cells of the room...
    pub per_cells: u16,
    /// ...but at least this many...
    pub min: u16,
    /// ...and at most this many.
    pub max: u16,
    /// A noise wakes a sleeper when it is this percent of its hearing threshold where it lies.
    pub wake: u16,
    /// How loud a waking sleeper's cry is where it lies.
    pub cry: u16,
    /// The hoard's items are this many item levels above the floor's.
    pub loot_level: u8,
    /// Percent chances that a den is themed: all of one creature type, or all of one faction.
    #[serde(default)]
    pub themes: Themes,
}

/// A den's themes (`themes = { kind = 10, faction = 20 }`): percents, at most 100 together.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Themes {
    /// A warren: every sleeper of one type.
    #[serde(default)]
    pub kind: u8,
    /// A tribe: every sleeper of one faction.
    #[serde(default)]
    pub faction: u8,
}

/// The most sleepers a den may hold.
pub const MAX_SLEEPERS: u16 = 64;
/// The percents of the hearing threshold a den's `wake` may be.
pub const WAKE: RangeInclusive<u16> = 100..=1000;

impl Swarm {
    /// The fault in these numbers, if any.
    fn check(&self) -> Result<(), String> {
        if self.per_cells == 0 {
            return Err("swarm: per_cells must be at least 1".into());
        }
        if self.min == 0 || self.min > self.max || self.max > MAX_SLEEPERS {
            return Err(format!(
                "swarm: min and max are 1 to {MAX_SLEEPERS}, min at most max"
            ));
        }
        if !WAKE.contains(&self.wake) {
            return Err(format!(
                "swarm: wake {} is out of range ({} to {})",
                self.wake,
                WAKE.start(),
                WAKE.end()
            ));
        }
        if self.cry > crate::noise::MAX_LOUDNESS {
            return Err(format!(
                "swarm: cry is at most {}",
                crate::noise::MAX_LOUDNESS
            ));
        }
        if u16::from(self.themes.kind) + u16::from(self.themes.faction) > 100 {
            return Err("swarm: themes' percents add up to at most 100".into());
        }
        Ok(())
    }

    /// How many sleepers a den of `cells` free floor cells holds: one per `per_cells`, within
    /// `min` and `max`, and never more than the cells.
    pub fn sleepers(&self, cells: usize) -> usize {
        let wanted = cells / usize::from(self.per_cells);
        wanted
            .clamp(usize::from(self.min), usize::from(self.max))
            .min(cells)
    }

    /// How many items the hoard of `sleepers` sleepers holds: one per four, at least one.
    pub fn hoard(sleepers: usize) -> usize {
        (sleepers / 4).max(1)
    }
}

/// What a special room does while one is inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomEffectDef {
    pub id: String,
    /// What it is called, without an article: "echoing hall" (look mode: "in an echoing hall").
    pub name: String,
    /// Marks its cells in the `rooms` dev dump layer.
    pub mark: char,
    /// Said the first time the player enters such a room.
    pub enter: String,
    /// How often generated levels draw it against the others, where no dungeon says otherwise
    /// (1 to 100).
    pub weight: u8,
    /// Said the first time a cell of such a room comes into the player's view from outside it.
    pub near: Option<String>,
    /// Percent of the levels that draw it for a room that it covers whole instead (0: never).
    pub level_chance: u8,
    /// Said each time the player arrives on a level it covers whole.
    pub pervade: Option<String>,
    /// Drawn only on a level with a danger (spec §7 "Danger levels").
    pub needs_danger: bool,
    /// Drawn only on a branch level with another below it.
    pub needs_below: bool,
    /// Drawn only for a room of at least this many floor cells.
    pub min_cells: u16,
    /// Drawn only for a room of at most this many floor cells.
    pub max_cells: Option<u16>,
    /// Percent of all light kept inside (spec §5 "Special rooms"): 100 leaves it as it is.
    pub light: u16,
    /// Light added over the room, like ambient light (a sanctuary's warm glow).
    pub glow: Light,
    /// Percent of the loudness of every noise made inside (spec §5 "Hearing").
    pub noise: u16,
    /// Percent of the usual pace at which hit points come back inside (spec §8 "Combat").
    pub regen: u16,
    /// Percent of the usual pace at which mana comes back inside (spec §8 "Mana").
    pub mana_regen: u16,
    /// Creatures hostile to the player do not step into it (spec §8 "Creature behavior").
    pub shunned: bool,
    /// The floor generators lay over the room's floor (a flooded room's shallow water).
    pub floor: Option<TerrainId>,
    /// The status everyone inside holds, at least at one degree, while they stay (a draining
    /// room's Ebbing; `World::dwell`); it eases by its own `eases` once they leave.
    pub status: Option<StatusId>,
    /// A teeming den's numbers: generators fill such a room with sleepers and their hoard.
    pub swarm: Option<Swarm>,
    /// Degrees Celsius added to the level's temperature inside (a hot room's +25, a cold room's
    /// -25); `WorldState::temperature_at` is the one place it is read.
    pub temperature: i8,
    /// Percent of everyone's carrying limit inside (a heavy hall's 50 halves it; spec §8
    /// "Burden"); `WorldState::load` is the one place it is read.
    pub burden: u16,
    /// The reflections a hall of mirrors shows of everyone inside (`World::images`).
    pub mirrors: Option<Mirrors>,
}

impl RoomEffectDef {
    /// Its name with an article: "an echoing hall", "a sanctuary".
    pub fn with_article(&self) -> String {
        format!("{} {}", super::article(&self.name), self.name)
    }

    /// `light` as this room leaves it: all of it but in a room that keeps only part of it.
    pub fn shade(&self, light: Light) -> Light {
        light.percent(self.light)
    }

    /// Whether a room of `cells` floor cells may get it, on a level with a danger (`danger`) and
    /// with another level below it (`below`).
    pub fn eligible(&self, cells: usize, danger: bool, below: bool) -> bool {
        (danger || !self.needs_danger)
            && (below || !self.needs_below)
            && cells >= usize::from(self.min_cells)
            && self.max_cells.is_none_or(|most| cells <= usize::from(most))
    }

    /// Whether it may cover a whole level: neither creatures shun it (a level hostiles shun is
    /// broken) nor does it lay a floor of its own, and it has its `pervade` line.
    pub fn can_pervade(&self) -> bool {
        !self.shunned
            && self.floor.is_none()
            && self.swarm.is_none()
            && self.mirrors.is_none()
            && self.pervade.is_some()
    }
}

/// What the special rooms at a cell do (spec §7 "Special rooms"): the effect of the room there,
/// then the one that covers the level, folded the same way by every reader: percents multiply,
/// light added adds up, flags hold if either holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Here<'c> {
    /// The effect of the special room at the cell.
    pub room: Option<&'c RoomEffectDef>,
    /// The effect that covers the whole level.
    pub level: Option<&'c RoomEffectDef>,
}

impl<'c> Here<'c> {
    /// Each effect, the room's first.
    pub fn each(&self) -> impl Iterator<Item = &'c RoomEffectDef> {
        self.room.into_iter().chain(self.level)
    }

    /// Whether no effect holds here.
    pub fn is_none(&self) -> bool {
        self.room.is_none() && self.level.is_none()
    }

    /// The product of the effects' percents `of` (100: none).
    fn percent(&self, of: impl Fn(&RoomEffectDef) -> u16) -> u32 {
        self.each().fold(100, |p, e| p * u32::from(of(e)) / 100)
    }

    /// Percent of all light kept here (100 leaves it as it is).
    pub fn light(&self) -> u16 {
        u16::try_from(self.percent(|e| e.light)).unwrap_or(100)
    }

    /// `light` as the effects here leave it.
    pub fn shade(&self, light: Light) -> Light {
        match self.light() {
            100 => light,
            kept => light.percent(kept),
        }
    }

    /// The light the effects add here, like ambient light.
    pub fn glow(&self) -> Light {
        self.each()
            .fold(Light::DARK, |sum, e| sum.saturating_add(e.glow))
    }

    /// Percent of the loudness of a noise made here.
    pub fn noise(&self) -> u32 {
        self.percent(|e| e.noise)
    }

    /// Percent of the usual pace at which hit points come back here.
    pub fn regen(&self) -> u32 {
        self.percent(|e| e.regen)
    }

    /// Percent of the usual pace at which mana comes back here.
    pub fn mana_regen(&self) -> u32 {
        self.percent(|e| e.mana_regen)
    }

    /// Whether creatures hostile to the player keep out.
    pub fn shunned(&self) -> bool {
        self.each().any(|e| e.shunned)
    }

    /// The den's numbers here, if a teeming den is here (the room's, never a whole level's).
    pub fn swarm(&self) -> Option<&'c Swarm> {
        self.room.and_then(|e| e.swarm.as_ref())
    }

    /// What the effects here add to the level's temperature: their degrees, added up.
    pub fn warmth(&self) -> Temperature {
        Temperature::degrees(self.each().map(|e| i32::from(e.temperature)).sum())
    }

    /// Percent of the carrying limit left here (100: as it is).
    pub fn burden(&self) -> u32 {
        self.percent(|e| e.burden)
    }

    /// The statuses held here, the room's first: both apply.
    pub fn statuses(self) -> impl Iterator<Item = StatusId> + 'c {
        (self.room.into_iter().chain(self.level)).filter_map(|e| e.status)
    }
}

/// The effects of `data/room_effects.toml` and how often generated levels get them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomEffects {
    /// Percent per room of a generated level (each of a cave's chambers) to be special.
    pub chance: u8,
    /// At most this many special rooms a generated level.
    pub per_level: u8,
    defs: Vec<RoomEffectDef>,
}

impl RoomEffects {
    /// Every effect, in file order (by [`RoomEffectId`]).
    pub fn defs(&self) -> &[RoomEffectDef] {
        &self.defs
    }

    pub fn get(&self, id: RoomEffectId) -> &RoomEffectDef {
        &self.defs[usize::from(id.0)]
    }

    pub fn id(&self, name: &str) -> Option<RoomEffectId> {
        let i = self.defs.iter().position(|d| d.id == name)?;
        Some(RoomEffectId(u8::try_from(i).ok()?))
    }

    /// Whether generated levels get special rooms at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && self.per_level > 0 && !self.defs.is_empty()
    }

    /// Parses `data/room_effects.toml` with no statuses known; `terrain` resolves a floor's
    /// terrain id and definition.
    #[cfg(test)]
    pub(super) fn parse<'t>(
        file: &str,
        text: &str,
        terrain: impl Fn(&str) -> Option<(TerrainId, &'t TerrainDef)>,
    ) -> Result<Self, ContentError> {
        Self::parse_with(file, text, terrain, |_| None)
    }

    /// Parses `data/room_effects.toml`; `terrain` resolves a floor's terrain id and definition,
    /// `status` a status's id and definition.
    pub(super) fn parse_with<'t, 's>(
        file: &str,
        text: &str,
        terrain: impl Fn(&str) -> Option<(TerrainId, &'t TerrainDef)>,
        status: impl Fn(&str) -> Option<(StatusId, &'s StatusDef)>,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RoomEffectFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.chance > 100 {
            return Err(err(format!(
                "chance {} is a percent: 0 to 100",
                parsed.chance
            )));
        }
        if parsed.effect.len() > MAX_EFFECTS {
            return Err(err(format!("at most {MAX_EFFECTS} effects")));
        }
        let mut defs: Vec<RoomEffectDef> = Vec::new();
        for spec in parsed.effect {
            let bad = |m: String| err(format!("effect `{}`: {m}", spec.id));
            if defs.iter().any(|d| d.id == spec.id) {
                return Err(err(format!("duplicate effect id `{}`", spec.id)));
            }
            if spec.name.trim().is_empty() || spec.enter.trim().is_empty() {
                return Err(bad("`name` and `enter` must not be empty".into()));
            }
            if spec.mark.is_whitespace() || defs.iter().any(|d| d.mark == spec.mark) {
                return Err(bad(format!(
                    "mark `{}` must be visible and not another effect's",
                    spec.mark
                )));
            }
            if spec.light > 100 {
                return Err(bad(format!(
                    "light {} is the percent kept: 0 to 100",
                    spec.light
                )));
            }
            if !WEIGHT.contains(&spec.weight) {
                return Err(bad(format!(
                    "weight {} is out of range ({} to {})",
                    spec.weight,
                    WEIGHT.start(),
                    WEIGHT.end()
                )));
            }
            if spec.level_chance > 100 {
                return Err(bad(format!(
                    "level_chance {} is a percent: 0 to 100",
                    spec.level_chance
                )));
            }
            if spec.max_cells.is_some_and(|most| most < spec.min_cells) {
                return Err(bad("max_cells is below min_cells".into()));
            }
            let blank = |line: &Option<String>| line.as_ref().is_some_and(|l| l.trim().is_empty());
            if blank(&spec.near) || blank(&spec.pervade) {
                return Err(bad("`near` and `pervade` must not be empty".into()));
            }
            for (what, value, range) in [
                ("noise", spec.noise, NOISE),
                ("regen", spec.regen, REGEN),
                ("mana_regen", spec.mana_regen, REGEN),
                ("burden", spec.burden, BURDEN),
            ] {
                if !range.contains(&value) {
                    return Err(bad(format!(
                        "{what} {value} is out of range ({} to {})",
                        range.start(),
                        range.end()
                    )));
                }
            }
            if !TEMPERATURE.contains(&spec.temperature) {
                return Err(bad(format!(
                    "temperature {} is out of range ({} to {} °C)",
                    spec.temperature,
                    TEMPERATURE.start(),
                    TEMPERATURE.end()
                )));
            }
            let floor = match &spec.floor {
                Some(name) => {
                    let (id, def) =
                        terrain(name).ok_or_else(|| bad(format!("unknown terrain `{name}`")))?;
                    let plain = def.walkable
                        && def.transparent
                        && def.stairs.is_none()
                        && !def.is_closed_door()
                        && !def.is_open_door()
                        && !def.flows();
                    if !plain {
                        return Err(bad(format!(
                            "floor `{name}` must be walkable and transparent, and not stairs, a \
                             door, or running water"
                        )));
                    }
                    Some(id)
                }
                None => None,
            };
            let status = match &spec.status {
                Some(name) => {
                    let (id, def) =
                        status(name).ok_or_else(|| bad(format!("unknown status `{name}`")))?;
                    if def.eases.is_none_or(|every| every < DWELL_EASES) {
                        return Err(bad(format!(
                            "status `{name}` must ease, and take at least {DWELL_EASES} seconds a \
                             degree, so it never lapses while one stays inside"
                        )));
                    }
                    Some(id)
                }
                None => None,
            };
            if let Some(m) = &spec.mirrors
                && !IMAGES.contains(&m.images)
            {
                return Err(bad(format!(
                    "mirrors.images {} is out of range ({} to {})",
                    m.images,
                    IMAGES.start(),
                    IMAGES.end()
                )));
            }
            let mirrors = spec.mirrors.as_ref().map(|m| Mirrors { images: m.images });
            if let Some(swarm) = &spec.swarm {
                swarm.check().map_err(bad)?;
                if !spec.needs_danger {
                    return Err(bad("a `swarm` needs `needs_danger`".into()));
                }
            }
            let def = RoomEffectDef {
                id: spec.id,
                name: spec.name,
                mark: spec.mark,
                enter: spec.enter,
                weight: spec.weight,
                near: spec.near,
                level_chance: spec.level_chance,
                pervade: spec.pervade,
                needs_danger: spec.needs_danger,
                needs_below: spec.needs_below,
                min_cells: spec.min_cells,
                max_cells: spec.max_cells,
                light: spec.light,
                glow: spec.glow,
                noise: spec.noise,
                regen: spec.regen,
                mana_regen: spec.mana_regen,
                shunned: spec.shunned,
                floor,
                status,
                swarm: spec.swarm,
                temperature: spec.temperature,
                burden: spec.burden,
                mirrors,
            };
            if def.level_chance > 0 && !def.can_pervade() {
                return Err(err(format!(
                    "effect `{}`: a `level_chance` needs a `pervade` line, and no effect that \
                     creatures shun, that lays a floor, or that holds mirrors covers a whole level",
                    def.id
                )));
            }
            defs.push(def);
        }
        Ok(Self {
            chance: parsed.chance,
            per_level: parsed.per_level,
            defs,
        })
    }
}

/// Where a special room lies on a level, and whether the player has been inside it: a rectangle
/// (a room of the rooms generator with its walls and doors, an area of a level file), or a blob of
/// cells (a cave chamber with the rock around it). Kept in the level's state (saved).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialRoom {
    pub effect: RoomEffectId,
    /// Corners of the box around it, inclusive.
    pub min: Pos,
    pub max: Pos,
    /// Its cells, sorted; empty when it is the whole box.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    cells: Vec<Pos>,
    /// Whether the player has been inside it: they were told what it is, and look mode names it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub entered: bool,
    /// Whether the player has seen a cell of it (its `near` line was said, if it has one).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub glimpsed: bool,
}

impl SpecialRoom {
    /// The rectangle from `min` to `max`, inclusive.
    pub fn rect(effect: RoomEffectId, min: Pos, max: Pos) -> Self {
        Self {
            effect,
            min,
            max,
            cells: Vec::new(),
            entered: false,
            glimpsed: false,
        }
    }

    /// The cells `cells` (at least one).
    pub fn blob(effect: RoomEffectId, mut cells: Vec<Pos>) -> Self {
        cells.sort();
        cells.dedup();
        let (min, max) = cells.iter().fold(
            (Pos::new(i32::MAX, i32::MAX), Pos::new(i32::MIN, i32::MIN)),
            |(lo, hi), p| {
                (
                    Pos::new(lo.x.min(p.x), lo.y.min(p.y)),
                    Pos::new(hi.x.max(p.x), hi.y.max(p.y)),
                )
            },
        );
        Self {
            effect,
            min,
            max,
            cells,
            entered: false,
            glimpsed: false,
        }
    }

    pub fn contains(&self, p: Pos) -> bool {
        let boxed =
            (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y);
        boxed && (self.cells.is_empty() || self.cells.binary_search(&p).is_ok())
    }

    /// Every cell of it, in position order.
    pub fn cells(&self) -> Vec<Pos> {
        if !self.cells.is_empty() {
            return self.cells.clone();
        }
        (self.min.x..=self.max.x)
            .flat_map(|x| (self.min.y..=self.max.y).map(move |y| Pos::new(x, y)))
            .collect()
    }

    /// Whether it is well formed within a `width` × `height` level: a box inside it, blob cells
    /// sorted, distinct, and inside the box (a guard for loaded saves).
    pub(crate) fn is_valid_within(&self, width: i32, height: i32) -> bool {
        let inside = |p: Pos| (0..width).contains(&p.x) && (0..height).contains(&p.y);
        let boxed = |p: &Pos| {
            (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
        };
        inside(self.min)
            && inside(self.max)
            && self.min.x <= self.max.x
            && self.min.y <= self.max.y
            && self.cells.windows(2).all(|w| w[0] < w[1])
            && self.cells.iter().all(boxed)
    }

    /// Whether it shares a cell with `other`.
    pub fn overlaps(&self, other: &SpecialRoom) -> bool {
        let boxes = self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y;
        boxes && self.cells().into_iter().any(|p| other.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> TerrainDef {
        crate::content::Content::builtin()
            .expect("content")
            .terrain_defs()[1]
            .clone()
    }

    const SHADOWED: &str = "[[effect]]\nid = \"shadowed\"\nname = \"shadowed room\"\nmark = \"s\"\n\
                            enter = \"Dark.\"\nlight = 30\n";

    #[test]
    fn effects_parse_with_their_defaults() {
        let def = floor();
        let text = format!("chance = 7\n{SHADOWED}");
        let effects =
            RoomEffects::parse("e.toml", &text, |_| Some((TerrainId(1), &def))).expect("valid");
        assert_eq!((effects.chance, effects.per_level), (7, 1));
        let e = effects.get(RoomEffectId(0));
        assert_eq!((e.light, e.noise, e.regen), (30, 100, 100));
        assert_eq!((e.glow, e.shunned, e.floor), (Light::DARK, false, None));
        assert_eq!(effects.id("shadowed"), Some(RoomEffectId(0)));
        assert_eq!(e.with_article(), "a shadowed room");
        assert_eq!(
            e.shade(Light::new(100, 200, 10, 50)),
            Light::new(30, 60, 3, 15)
        );
        assert!(effects.generated());
        let none = RoomEffects::parse("e.toml", SHADOWED, |_| None).expect("valid");
        assert!(!none.generated(), "no chance, no generated rooms");
    }

    #[test]
    fn bad_effects_are_refused() {
        let def = floor();
        let wall = {
            let mut w = floor();
            w.walkable = false;
            w
        };
        for (text, why) in [
            (format!("chance = 101\n{SHADOWED}"), "chance 101"),
            (format!("{SHADOWED}{SHADOWED}"), "duplicate effect id"),
            (SHADOWED.replace("30", "101"), "light 101"),
            (format!("{SHADOWED}noise = 0\n"), "noise 0 is out of range"),
            (
                format!("{SHADOWED}regen = 1001\n"),
                "regen 1001 is out of range",
            ),
            (SHADOWED.replace("Dark.", " "), "must not be empty"),
            (SHADOWED.replace("\"s\"", "\" \""), "mark ` `"),
            (
                format!("{SHADOWED}floor = \"lava\"\n"),
                "unknown terrain `lava`",
            ),
            (
                format!("{SHADOWED}floor = \"wall\"\n"),
                "floor `wall` must be walkable",
            ),
            (format!("{SHADOWED}color = 3\n"), "unknown field `color`"),
            (
                format!("{SHADOWED}weight = 0\n"),
                "weight 0 is out of range",
            ),
            (format!("{SHADOWED}weight = 101\n"), "weight 101"),
            (
                format!("{SHADOWED}level_chance = 101\n"),
                "level_chance 101",
            ),
            (
                format!("{SHADOWED}level_chance = 5\n"),
                "needs a `pervade` line",
            ),
            (
                format!("{SHADOWED}level_chance = 5\npervade = \"P.\"\nshunned = true\n"),
                "no effect that creatures shun",
            ),
            (
                format!("{SHADOWED}min_cells = 20\nmax_cells = 10\n"),
                "max_cells is below min_cells",
            ),
            (format!("{SHADOWED}near = \" \"\n"), "must not be empty"),
        ] {
            let terrain = |name: &str| match name {
                "floor" => Some((TerrainId(1), &def)),
                "wall" => Some((TerrainId(2), &wall)),
                _ => None,
            };
            let e = RoomEffects::parse("e.toml", &text, terrain).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        let twin = format!("{SHADOWED}{}", SHADOWED.replace("\"shadowed\"", "\"dark\""));
        let e = RoomEffects::parse("e.toml", &twin, |_| None).unwrap_err();
        assert!(e.message.contains("not another effect's"), "{}", e.message);
    }

    #[test]
    fn the_draw_and_whole_level_fields_parse() {
        let plain = RoomEffects::parse("e.toml", SHADOWED, |_| None).expect("valid");
        let e = plain.get(RoomEffectId(0));
        assert_eq!((e.weight, e.level_chance, e.min_cells), (10, 0, 0));
        assert_eq!((&e.near, &e.pervade, e.max_cells), (&None, &None, None));
        assert!(!e.needs_danger && !e.needs_below && !e.can_pervade());
        assert!(e.eligible(1, false, false));
        let text = format!(
            "{SHADOWED}weight = 6\nnear = \"Near.\"\nlevel_chance = 5\npervade = \"All.\"\n\
             needs_danger = true\nneeds_below = true\nmin_cells = 16\nmax_cells = 40\n"
        );
        let full = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
        let e = full.get(RoomEffectId(0));
        assert_eq!((e.weight, e.level_chance), (6, 5));
        assert_eq!(e.near.as_deref(), Some("Near."));
        assert_eq!(e.pervade.as_deref(), Some("All."));
        assert!(e.can_pervade());
        assert!(e.eligible(16, true, true) && e.eligible(40, true, true));
        assert!(!e.eligible(15, true, true) && !e.eligible(41, true, true));
        assert!(!e.eligible(20, false, true) && !e.eligible(20, true, false));
    }

    #[test]
    fn here_folds_both_effects() {
        let text = format!(
            "{SHADOWED}noise = 200\nregen = 300\nglow = [10, 0, 0, 0]\n\
             [[effect]]\nid = \"b\"\nname = \"b\"\nmark = \"b\"\nenter = \"B.\"\nlight = 50\n\
             noise = 50\nmana_regen = 200\nglow = [5, 5, 0, 0]\nshunned = true\n"
        );
        let effects = RoomEffects::parse("e.toml", &text, |_| None).expect("valid");
        let (a, b) = (effects.get(RoomEffectId(0)), effects.get(RoomEffectId(1)));
        let none = Here::default();
        assert!(none.is_none());
        assert_eq!((none.light(), none.noise(), none.regen()), (100, 100, 100));
        assert_eq!(none.shade(Light::new(9, 9, 9, 9)), Light::new(9, 9, 9, 9));
        let both = Here {
            room: Some(a),
            level: Some(b),
        };
        assert_eq!(both.each().count(), 2);
        assert_eq!((both.light(), both.noise()), (15, 100));
        assert_eq!((both.regen(), both.mana_regen()), (300, 200));
        assert_eq!(both.glow(), Light::new(15, 5, 0, 0));
        assert!(both.shunned());
        let level = Here {
            room: None,
            level: Some(a),
        };
        assert_eq!((level.light(), level.shunned()), (30, false));
    }

    #[test]
    fn a_status_parses_and_must_ease_slowly_enough() {
        let content = crate::content::Content::builtin().expect("content");
        let lookup = |name: &str| {
            let id = content.status_id(name)?;
            Some((id, content.status(id)))
        };
        let parse = |text: &str| RoomEffects::parse_with("e.toml", text, |_| None, lookup);
        let plain = parse(SHADOWED).expect("valid");
        assert_eq!(plain.get(RoomEffectId(0)).status, None, "none by default");
        let ebbing = parse(&format!("{SHADOWED}status = \"ebbing\"\n")).expect("valid");
        let e = ebbing.get(RoomEffectId(0));
        assert_eq!(e.status, content.status_id("ebbing"));
        let here = Here {
            room: Some(e),
            level: Some(e),
        };
        assert_eq!(here.statuses().count(), 2, "both apply");
        for (status, why) in [
            ("nothing", "unknown status `nothing`"),
            // Held eases a degree a turn (6 s): it would lapse between two actions.
            ("held", "at least 10 seconds"),
            // Diseased never eases by itself.
            ("diseased", "must ease"),
        ] {
            let e = parse(&format!("{SHADOWED}status = \"{status}\"\n")).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn rooms_are_boxes_or_blobs() {
        let id = RoomEffectId(0);
        let rect = SpecialRoom::rect(id, Pos::new(2, 1), Pos::new(4, 3));
        assert!(rect.contains(Pos::new(2, 1)) && rect.contains(Pos::new(4, 3)));
        assert!(!rect.contains(Pos::new(5, 3)) && !rect.contains(Pos::new(1, 2)));
        assert_eq!(rect.cells().len(), 9);
        let blob = SpecialRoom::blob(id, vec![Pos::new(6, 2), Pos::new(5, 3), Pos::new(6, 2)]);
        assert_eq!((blob.min, blob.max), (Pos::new(5, 2), Pos::new(6, 3)));
        assert!(blob.contains(Pos::new(6, 2)) && !blob.contains(Pos::new(6, 3)));
        assert_eq!(blob.cells(), vec![Pos::new(5, 3), Pos::new(6, 2)]);
        assert!(rect.is_valid_within(10, 10) && blob.is_valid_within(10, 10));
        assert!(!rect.is_valid_within(4, 10));
        assert!(!rect.overlaps(&blob));
        let touching = SpecialRoom::rect(id, Pos::new(4, 3), Pos::new(5, 3));
        assert!(rect.overlaps(&touching) && blob.overlaps(&touching));
        // Saved compactly: a box has no cell list, a room not entered no flag.
        let json = serde_json::to_string(&rect).expect("serializes");
        assert_eq!(
            json,
            r#"{"effect":0,"min":{"x":2,"y":1},"max":{"x":4,"y":3}}"#
        );
        let back: SpecialRoom = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, rect);
    }

    #[test]
    fn mana_paces_parse_and_are_checked() {
        let well = format!("{SHADOWED}mana_regen = 300\n");
        let effects = RoomEffects::parse("e.toml", &well, |_| None).expect("valid");
        assert_eq!(effects.get(RoomEffectId(0)).mana_regen, 300);
        let plain = RoomEffects::parse("e.toml", SHADOWED, |_| None).expect("valid");
        assert_eq!(plain.get(RoomEffectId(0)).mana_regen, 100, "the default");
        let e = RoomEffects::parse("e.toml", &format!("{SHADOWED}mana_regen = 0\n"), |_| None)
            .unwrap_err();
        assert!(
            e.message.contains("mana_regen 0 is out of range"),
            "{}",
            e.message
        );
    }
}
