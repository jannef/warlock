//! Hand-authored level files (`assets/levels/*.toml`, spec §7).
//!
//! A level is a multiline ASCII `map` plus a `legend` mapping characters to terrain and to things
//! placed on that terrain (player start, creatures with patrol routes, what they carry, and what
//! of that they wield and wear (`wield = "spear"`, `wear = ["leather_jerkin"]`), the faction they
//! are placed in, and the unique character they are (`npc = "hermit"`, whose dialogue they
//! speak), stair links, lights, the flow of running water, objects, and locks on closed doors:
//! `lock = "pantry"`, a lock its keys open, or `locked = true`, one only a skeleton key opens;
//! spec §8 "Doors"). Characters not in the level's legend fall back to the shared default legend
//! (`assets/data/legend.toml`, which has arrows for running water). Objects may also be placed
//! without a legend character: `[[pile]]` tables with `at = [x, y]` and `objects`. Objects are
//! written as ids with an optional count, top of the pile first: `objects = ["dagger",
//! "arrow*12"]`. An optional
//! `ambient = [r, g, b, uv]` lights the whole level. `kind` (`dungeon`, the default, `outdoor`,
//! or `overworld`) decides what a step there takes (spec §8 "Time"); outdoor levels and the
//! overworld are lit by the sun and the sky by day (spec §5). Optional
//! `[[region]]` tables name areas of the level (`name`, `area = [x0, y0, x1, y1]`, inclusive; the
//! first match wins), shown in the status bar. Optional `[[room]]` tables mark special rooms
//! (spec §7 "Special rooms"): `effect` (an id in `data/room_effects.toml`) over `area = [x0, y0,
//! x1, y1]` (inclusive, walls and doors included); rooms do not overlap.

use super::creature::CreatureId;
use super::faction::FactionId;
use super::flow::{FLOW_SPEEDS, Flow};
use super::light::LightId;
use super::object::parse_stacks;
use super::room_effect::SpecialRoom;
use super::terrain::{StairsDir, TerrainDef, TerrainId};
use super::{ActionKind, Content, ContentError, MAX_ITEM_LEVEL, Rarity, Speaker};
use crate::door::{Lock, check_lock_id};
use crate::equipment::{self, Equipment};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::light::Light;
use crate::magic::{Affix, LootGrade, Magic};
use crate::object::{INVENTORY_SLOTS, Stack, add_to_pile};
use crate::temperature::{LEVEL_TEMPERATURES, Temperature};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What kind of place a level is (spec §8 "Time"). It decides what a step there takes and
/// whether the sun shines on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LevelKind {
    /// Underground or indoors.
    #[default]
    Dungeon,
    /// Under the open sky.
    Outdoor,
    /// The map of the land at large: a step is a journey of the terrain's `travel_minutes`, sight
    /// reaches as far as daylight allows (spec §5), and cells that are not locations lead into
    /// outdoor local areas.
    Overworld,
}

impl LevelKind {
    /// Whether the level lies under the open sky, lit by the sun and the sky by day (spec §5
    /// "Sunlight"): outdoor levels and the overworld; never dungeons. Terrain with `roofed` is
    /// covered even here.
    pub const fn is_sky_exposed(self) -> bool {
        match self {
            LevelKind::Dungeon => false,
            LevelKind::Outdoor | LevelKind::Overworld => true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelFile {
    name: String,
    /// A line said each time the player arrives on the level by stairs; default none.
    arrival: Option<String>,
    /// Light everywhere on the level, `[r, g, b, uv]`; default none.
    #[serde(default)]
    ambient: Light,
    #[serde(default)]
    kind: LevelKind,
    /// The temperature underground, in °C (spec §5 "Temperature"); default the dungeon default.
    /// Under the open sky it follows the weather, so outdoor levels have none.
    temperature: Option<i8>,
    map: String,
    #[serde(default)]
    legend: BTreeMap<String, LegendEntry>,
    #[serde(default)]
    region: Vec<RegionSpec>,
    /// Special rooms (spec §7 "Special rooms").
    #[serde(default)]
    room: Vec<RoomSpec>,
    /// Objects placed by position rather than by a legend character.
    #[serde(default)]
    pile: Vec<PileSpec>,
    /// Magic and rare items placed by position, each with its affixes (spec §8 "Magic items").
    #[serde(default)]
    item: Vec<ItemSpec>,
    /// The item level of the creatures' drops here (default 1; spec §8 "Magic items").
    item_level: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PileSpec {
    at: [i32; 2],
    /// Top first, e.g. `["dagger", "arrow*12"]`.
    objects: Vec<String>,
}

/// A magic or rare item placed by hand: `[[item]] at = [x, y] object = "long_sword" affixes =
/// ["sharp:2", "of_the_bear"]`, each affix an id with an optional tier (default 1) whose value is
/// the top of the tier's range; a rare one has a `name`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemSpec {
    at: [i32; 2],
    object: String,
    affixes: Vec<String>,
    /// A rare item's name; with a name it is rare, without magic.
    name: Option<String>,
    /// Whether the player knows its affixes from the start (default false).
    #[serde(default)]
    identified: bool,
    /// Its item level (default: that of its highest tier).
    level: Option<u8>,
    /// How many (ammunition), default 1.
    count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionSpec {
    name: String,
    /// `[x0, y0, x1, y1]`, inclusive.
    area: [i32; 4],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomSpec {
    /// An effect id in `data/room_effects.toml`.
    effect: String,
    /// `[x0, y0, x1, y1]`, inclusive: the room with its walls.
    area: [i32; 4],
}

/// A named area of a level, e.g. a forest or a range of hills on the overworld.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub name: String,
    /// Corners, inclusive.
    pub min: Pos,
    pub max: Pos,
}

impl Region {
    pub fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }
}

/// Where walking off the edge of a level leads: from a local area, back to its overworld cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeExit {
    /// The level id (the overworld).
    pub to: String,
    /// Where the player arrives there.
    pub pos: Pos,
}

/// What a map character stands for.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum LegendEntry {
    /// Just a terrain id.
    Terrain(String),
    /// Terrain plus something placed on it (boxed: it is much larger).
    Detailed(Box<LegendDetail>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegendDetail {
    terrain: String,
    spawn: Option<Spawn>,
    creature: Option<String>,
    /// Faction the creature on this cell belongs to instead of its type's (`faction =
    /// "peaceful"`); an id in `data/factions.toml`.
    faction: Option<String>,
    /// The unique character the creature on this cell is (`npc = "hermit"`): it says what the
    /// npc's dialogue in `data/dialogue.toml` says, not its type's.
    npc: Option<String>,
    /// Patrol route, e.g. `"E4 S2 W4 N2"`; must return to its start.
    route: Option<String>,
    /// Level id the stairs on this cell lead to.
    to: Option<String>,
    /// Light preset placed on this cell (overrides the terrain's own light).
    light: Option<String>,
    /// Which way running water on this cell flows (presentation-only); required on terrain with
    /// a `flow` animation and allowed nowhere else.
    flow: Option<Dir>,
    /// Flow speed in percent of the terrain's (default 100).
    flow_speed: Option<u16>,
    /// Ice on the water of this cell, in mm (spec §5 "Ice"); only on water that freezes.
    ice: Option<u16>,
    /// Objects lying on this cell, top first, e.g. `["dagger", "arrow*12"]`.
    objects: Option<Vec<String>>,
    /// Vaults only: an item of loot rolled here when a level is generated, at least of this
    /// grade (`any`, `magic`, `rare`; spec §8 "Magic items").
    loot: Option<LootGrade>,
    /// Objects the creature on this cell carries.
    carry: Option<Vec<String>>,
    /// Objects of what it carries that it wields (weapons, shields, missile weapons and
    /// missiles), by object id: `wield = "spear"` or a list.
    wield: Option<OneOrMany>,
    /// Objects of what it carries that it wears (armor, rings, amulets), by object id.
    wear: Option<OneOrMany>,
    /// The closed door on this cell is locked (only a skeleton key opens it unless it has a
    /// `lock` id).
    locked: Option<bool>,
    /// The closed door on this cell is locked, with this lock id: keys of that id open it.
    lock: Option<String>,
}

/// One object id or a list of them.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn ids(&self) -> &[String] {
        match self {
            OneOrMany::One(id) => std::slice::from_ref(id),
            OneOrMany::Many(ids) => ids,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Spawn {
    Player,
}

impl LegendEntry {
    fn terrain(&self) -> &str {
        match self {
            LegendEntry::Terrain(t) => t,
            LegendEntry::Detailed(d) => &d.terrain,
        }
    }

    fn detail(&self) -> Option<&LegendDetail> {
        match self {
            LegendEntry::Terrain(_) => None,
            LegendEntry::Detailed(d) => Some(&**d),
        }
    }
}

/// A creature placed by a level file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureSpawn {
    pub creature: CreatureId,
    /// The faction it is placed in when not its type's.
    pub faction: Option<FactionId>,
    /// The unique character it is (an npc id of `data/dialogue.toml`), if any.
    pub npc: Option<String>,
    pub pos: Pos,
    /// Looping patrol route; empty means the creature stays put.
    pub route: Vec<Dir>,
    /// What it carries.
    pub carry: Vec<Stack>,
    /// What of that it wears and wields.
    pub equipment: Equipment,
}

/// A staircase cell and where it leads.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, Deserialize)]
pub struct StairsLink {
    pub pos: Pos,
    pub dir: StairsDir,
    /// Target level id; `None` for stairs that lead nowhere yet.
    pub to: Option<String>,
}

/// A parsed, validated level ready to instantiate: from a level file, or generated (see
/// [`crate::mapgen`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelTemplate {
    pub id: String,
    pub name: String,
    /// A message said when the player arrives on the level (hand-made levels only).
    pub arrival: Option<String>,
    /// Light everywhere on the level.
    pub ambient: Light,
    pub kind: LevelKind,
    pub terrain: Grid<TerrainId>,
    pub player_start: Option<Pos>,
    pub creatures: Vec<CreatureSpawn>,
    pub stairs: Vec<StairsLink>,
    /// Lights placed by the legend, by position.
    pub lights: BTreeMap<Pos, LightId>,
    /// Flow of every running-water cell, by position (presentation-only).
    pub flow: BTreeMap<Pos, Flow>,
    /// Piles of objects, by position, each top first; only on walkable terrain.
    pub objects: BTreeMap<Pos, Vec<Stack>>,
    /// Locks, by position: only on closed doors (spec §8 "Doors").
    pub locks: BTreeMap<Pos, Lock>,
    /// Named areas, first match wins.
    pub regions: Vec<Region>,
    /// Special rooms (spec §7 "Special rooms"), none overlapping another.
    pub rooms: Vec<SpecialRoom>,
    /// Where walking off the edge leads (local areas); `None`: the edge cannot be crossed.
    pub exit: Option<EdgeExit>,
    /// The item level of loot here (spec §8 "Magic items"): a hand-made level's `item_level`
    /// (default 1), a branch level's depth.
    pub item_level: u8,
    /// Vaults: cells where loot is rolled when they are stamped into a generated level, each
    /// with the least it may be. Empty on levels.
    pub loot: Vec<(Pos, LootGrade)>,
    /// The temperature underground (spec §5 "Temperature"); `None`: the dungeon default (or,
    /// under the open sky, the weather's).
    pub temperature: Option<Temperature>,
    /// Ice on water, by position, in µm (spec §5 "Ice"); only on water that freezes.
    pub ice: BTreeMap<Pos, u32>,
}

impl LevelTemplate {
    /// Whether a key that opens `lock` lies in a pile, or is carried by a creature placed, on a
    /// cell `within` accepts (spec §8 "Doors").
    pub fn has_key_for(
        &self,
        content: &Content,
        lock: &Lock,
        within: impl Fn(Pos) -> bool,
    ) -> bool {
        let fits =
            |s: &Stack| (content.object(s.object).key.as_ref()).is_some_and(|k| k.opens(lock));
        let lying = (self.objects.iter()).any(|(p, pile)| within(*p) && pile.iter().any(fits));
        lying || (self.creatures.iter()).any(|c| within(c.pos) && c.carry.iter().any(fits))
    }

    /// The name of the first region containing `p`.
    pub fn region_at(&self, p: Pos) -> Option<&str> {
        self.regions
            .iter()
            .find(|r| r.contains(p))
            .map(|r| r.name.as_str())
    }
}

/// Parses a single-character legend key.
pub(super) fn legend_char(file: &str, key: &str) -> Result<char, ContentError> {
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(ContentError::new(
            file,
            format!("legend key `{key}` must be exactly one character"),
        )),
    }
}

pub(super) fn parse_level(
    file: &str,
    id: &str,
    text: &str,
    content: &Content,
    default_legend: &BTreeMap<char, LegendEntry>,
) -> Result<LevelTemplate, ContentError> {
    let err = |msg: String| ContentError::new(file, msg);
    let parsed: LevelFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;

    let mut legend = default_legend.clone();
    for (key, entry) in parsed.legend {
        legend.insert(legend_char(file, &key)?, entry);
    }

    let mut rows: Vec<Vec<char>> = parsed.map.lines().map(|l| l.chars().collect()).collect();
    while rows
        .last()
        .is_some_and(|r| r.iter().all(|c| c.is_whitespace()))
    {
        rows.pop();
    }
    let height = rows.len() as i32;
    let width = rows.iter().map(Vec::len).max().unwrap_or(0) as i32;
    if width == 0 || height == 0 {
        return Err(err("map is empty".into()));
    }

    let mut cells = Vec::with_capacity((width * height) as usize);
    let mut player_start = None;
    let mut creatures = Vec::new();
    let mut stairs = Vec::new();
    let mut lights = BTreeMap::new();
    let mut flow = BTreeMap::new();
    let mut ice = BTreeMap::new();
    let mut objects: BTreeMap<Pos, Vec<Stack>> = BTreeMap::new();
    let mut loot = Vec::new();
    let mut locks = BTreeMap::new();

    for (y, row) in rows.iter().enumerate() {
        for x in 0..width as usize {
            let pos = Pos::new(x as i32, y as i32);
            let ch = row.get(x).copied().unwrap_or(' ');
            let entry = legend
                .get(&ch)
                .ok_or_else(|| err(format!("character `{ch}` at {pos} is not in the legend")))?;
            let terrain = content.terrain_id(entry.terrain()).ok_or_else(|| {
                err(format!(
                    "legend `{ch}` uses unknown terrain `{}`",
                    entry.terrain()
                ))
            })?;
            cells.push(terrain);

            let detail = entry.detail();
            let running = content.terrain(terrain).flows();
            if let Some(f) = parse_flow(ch, entry, running, pos).map_err(err)? {
                flow.insert(pos, f);
            }
            if let Some(um) = parse_ice(ch, entry, content.terrain(terrain)).map_err(err)? {
                ice.insert(pos, um);
            }
            if let Some(dir) = content.terrain(terrain).stairs {
                stairs.push(StairsLink {
                    pos,
                    dir,
                    to: detail.and_then(|d| d.to.clone()),
                });
            } else if detail.is_some_and(|d| d.to.is_some()) {
                return Err(err(format!(
                    "legend `{ch}` has `to` but terrain `{}` is not stairs",
                    entry.terrain()
                )));
            }
            let Some(detail) = detail else { continue };
            if let Some(lock) =
                parse_lock(detail).map_err(|m| err(format!("legend `{ch}`: {m}")))?
            {
                if !content.terrain(terrain).is_closed_door() {
                    return Err(err(format!(
                        "legend `{ch}` locks terrain `{}`, which is no closed door",
                        entry.terrain()
                    )));
                }
                locks.insert(pos, lock);
            }
            if let Some(light) = &detail.light {
                let id = content
                    .light_id(light)
                    .ok_or_else(|| err(format!("legend `{ch}` uses unknown light `{light}`")))?;
                if !content.terrain(terrain).transparent {
                    return Err(err(format!(
                        "legend `{ch}` places a light on opaque terrain `{}`",
                        entry.terrain()
                    )));
                }
                lights.insert(pos, id);
            }
            if let Some(Spawn::Player) = detail.spawn
                && player_start.replace(pos).is_some()
            {
                return Err(err("more than one player start".into()));
            }
            if let Some(specs) = &detail.objects {
                let pile = parse_stacks(specs, content)
                    .map_err(|m| err(format!("legend `{ch}`: objects: {m}")))?;
                if !pile.is_empty() {
                    objects.insert(pos, pile);
                }
            }
            if let Some(grade) = detail.loot {
                if !content.terrain(terrain).walkable {
                    return Err(err(format!(
                        "legend `{ch}` places loot on terrain `{}`, which is not walkable",
                        entry.terrain()
                    )));
                }
                loot.push((pos, grade));
            }
            if let Some(creature) = &detail.creature {
                let id = content.creature_id(creature).ok_or_else(|| {
                    err(format!("legend `{ch}` uses unknown creature `{creature}`"))
                })?;
                let faction = match &detail.faction {
                    Some(name) => Some(content.factions().id(name).ok_or_else(|| {
                        err(format!("legend `{ch}` uses unknown faction `{name}`"))
                    })?),
                    None => None,
                };
                let route = match &detail.route {
                    Some(r) => parse_route(r).map_err(|m| err(format!("legend `{ch}`: {m}")))?,
                    None => Vec::new(),
                };
                if let Some(npc) = &detail.npc {
                    let known = (content.dialogues().iter())
                        .any(|d| d.speaker == Speaker::Npc(npc.clone()));
                    if !known {
                        return Err(err(format!(
                            "legend `{ch}`: npc `{npc}` has no dialogue in data/dialogue.toml"
                        )));
                    }
                }
                let carry = match &detail.carry {
                    Some(specs) => parse_stacks(specs, content)
                        .map_err(|m| err(format!("legend `{ch}`: carry: {m}")))?,
                    None => Vec::new(),
                };
                if carry.len() > INVENTORY_SLOTS {
                    return Err(err(format!(
                        "legend `{ch}`: a creature carries at most {INVENTORY_SLOTS} stacks"
                    )));
                }
                let equipment = equip(detail, &carry, content)
                    .map_err(|m| err(format!("legend `{ch}`: {m}")))?;
                creatures.push(CreatureSpawn {
                    creature: id,
                    faction,
                    npc: detail.npc.clone(),
                    pos,
                    route,
                    carry,
                    equipment,
                });
            } else if detail.route.is_some()
                || detail.faction.is_some()
                || detail.npc.is_some()
                || detail.carry.is_some()
                || detail.wield.is_some()
                || detail.wear.is_some()
            {
                return Err(err(format!(
                    "legend `{ch}` has a faction, npc, route, carry, wield, or wear but no creature"
                )));
            }
        }
    }

    let mut regions = Vec::with_capacity(parsed.region.len());
    for spec in parsed.region {
        let [x0, y0, x1, y1] = spec.area;
        if !(0 <= x0 && x0 <= x1 && x1 < width && 0 <= y0 && y0 <= y1 && y1 < height) {
            return Err(err(format!(
                "region `{}` area {:?} is not a rectangle inside the {width}x{height} map",
                spec.name, spec.area
            )));
        }
        regions.push(Region {
            name: spec.name,
            min: Pos::new(x0, y0),
            max: Pos::new(x1, y1),
        });
    }

    let mut rooms: Vec<SpecialRoom> = Vec::with_capacity(parsed.room.len());
    for spec in parsed.room {
        let effect = content.room_effects().id(&spec.effect).ok_or_else(|| {
            err(format!(
                "room at {:?} has unknown effect `{}`",
                spec.area, spec.effect
            ))
        })?;
        let [x0, y0, x1, y1] = spec.area;
        let room = SpecialRoom::rect(effect, Pos::new(x0, y0), Pos::new(x1, y1));
        if !room.is_valid_within(width, height) {
            return Err(err(format!(
                "room `{}` area {:?} is not a rectangle inside the {width}x{height} map",
                spec.effect, spec.area
            )));
        }
        if rooms.iter().any(|r| r.overlaps(&room)) {
            return Err(err(format!(
                "room `{}` area {:?} overlaps another room",
                spec.effect, spec.area
            )));
        }
        rooms.push(room);
    }

    let terrain = Grid::from_vec(width, height, cells);
    for spec in parsed.pile {
        let at = Pos::new(spec.at[0], spec.at[1]);
        if !terrain.in_bounds(at) {
            return Err(err(format!(
                "pile at {at} is outside the {width}x{height} map"
            )));
        }
        let stacks =
            parse_stacks(&spec.objects, content).map_err(|m| err(format!("pile at {at}: {m}")))?;
        if stacks.is_empty() {
            return Err(err(format!("pile at {at} has no objects")));
        }
        add_to_pile(objects.entry(at).or_default(), &stacks, content);
    }
    for spec in &parsed.item {
        let at = Pos::new(spec.at[0], spec.at[1]);
        if !terrain.in_bounds(at) {
            return Err(err(format!(
                "item at {at} is outside the {width}x{height} map"
            )));
        }
        let stack = parse_item(spec, content).map_err(|m| err(format!("item at {at}: {m}")))?;
        add_to_pile(objects.entry(at).or_default(), &[stack], content);
    }
    let item_level = parsed.item_level.unwrap_or(1);
    if item_level == 0 || item_level > MAX_ITEM_LEVEL {
        return Err(err(format!("`item_level` must be 1 to {MAX_ITEM_LEVEL}")));
    }
    if let Some(at) = objects
        .keys()
        .find(|p| !content.terrain(terrain[**p]).walkable)
    {
        return Err(err(format!(
            "objects at {at} lie on terrain `{}`, which is not walkable",
            content.terrain(terrain[*at]).id
        )));
    }

    let temperature = match parsed.temperature {
        Some(_) if parsed.kind.is_sky_exposed() => {
            return Err(err(
                "only levels under no sky have a `temperature`: under the open sky it follows \
                 the weather"
                    .into(),
            ));
        }
        Some(c) if !LEVEL_TEMPERATURES.contains(&c) => {
            return Err(err(format!(
                "`temperature` must be {} to {} (°C)",
                LEVEL_TEMPERATURES.start(),
                LEVEL_TEMPERATURES.end()
            )));
        }
        c => c.map(|c| Temperature::degrees(c.into())),
    };
    let template = LevelTemplate {
        id: id.to_string(),
        name: parsed.name,
        arrival: parsed.arrival,
        ambient: parsed.ambient,
        kind: parsed.kind,
        terrain,
        player_start,
        creatures,
        stairs,
        lights,
        flow,
        objects,
        locks,
        regions,
        rooms,
        exit: None,
        item_level,
        loot,
        temperature,
        ice,
    };
    for spawn in &template.creatures {
        validate_route(&template, content, spawn).map_err(err)?;
    }
    if template.kind == LevelKind::Overworld {
        let untimed = template.terrain.cells().iter().find(|t| {
            let def = content.terrain(**t);
            def.walkable && def.travel_minutes.is_none()
        });
        if let Some(t) = untimed {
            return Err(err(format!(
                "overworld terrain `{}` is walkable but has no travel_minutes",
                content.terrain(*t).id
            )));
        }
    }
    Ok(template)
}

/// The flow a legend entry gives the cell at `pos`: required on running water, an error anywhere
/// else.
fn parse_flow(
    ch: char,
    entry: &LegendEntry,
    running: bool,
    pos: Pos,
) -> Result<Option<Flow>, String> {
    let detail = entry.detail();
    let dir = detail.and_then(|d| d.flow);
    let speed = detail.and_then(|d| d.flow_speed);
    match (dir, running) {
        (None, true) => Err(format!(
            "running water `{}` at {pos} needs a flow direction (`flow` in legend `{ch}`)",
            entry.terrain()
        )),
        (Some(_), false) => Err(format!(
            "legend `{ch}` has `flow` but terrain `{}` is not running water",
            entry.terrain()
        )),
        (None, false) if speed.is_some() => {
            Err(format!("legend `{ch}` has `flow_speed` but no `flow`"))
        }
        (None, false) => Ok(None),
        (Some(dir), true) => {
            let speed = speed.unwrap_or(100);
            if !FLOW_SPEEDS.contains(&speed) {
                return Err(format!(
                    "legend `{ch}` has `flow_speed` {speed}; expected {} to {}",
                    FLOW_SPEEDS.start(),
                    FLOW_SPEEDS.end()
                ));
            }
            Ok(Some(Flow::new(dir, speed)))
        }
    }
}

/// The magic or rare item an `[[item]]` table places: rare with a `name`, else magic; each
/// affix's value the top of its tier's range; its item level that of its highest tier unless
/// given. It must be an item its affixes may roll on (spec §8 "Magic items").
fn parse_item(spec: &ItemSpec, content: &Content) -> Result<Stack, String> {
    let object = content
        .object_id(&spec.object)
        .ok_or_else(|| format!("unknown object `{}`", spec.object))?;
    let def = content.object(object);
    let mut affixes = Vec::new();
    let mut highest = 1;
    for text in &spec.affixes {
        let (id, tier) = match text.split_once(':') {
            Some((id, tier)) => {
                let tier = tier
                    .parse::<u8>()
                    .map_err(|_| format!("`{text}`: the tier must be a number, e.g. `sharp:2`"))?;
                (id, tier)
            }
            None => (text.as_str(), 1),
        };
        let a = (content.affixes().get(id)).ok_or_else(|| format!("unknown affix `{id}`"))?;
        let t = a
            .tier(tier)
            .ok_or_else(|| format!("`{id}` has no tier {tier}"))?;
        highest = highest.max(t.level);
        affixes.push(Affix {
            id: id.to_string(),
            tier,
            value: t.max,
        });
    }
    let rarity = if spec.name.is_some() {
        Rarity::Rare
    } else {
        Rarity::Magic
    };
    let magic = Magic {
        rarity,
        level: spec.level.unwrap_or(highest),
        affixes,
        name: spec.name.clone(),
        identified: spec.identified,
    };
    magic.check(def, content)?;
    let count = spec.count.unwrap_or(1);
    if count == 0 || (count > 1 && !def.stackable) {
        return Err(format!(
            "`count` {count} of `{}`, which does not stack",
            def.id
        ));
    }
    Ok(Stack::with_magic(object, count, magic))
}

/// The ice a legend entry lays on the water of its cell, in µm (`ice = mm`): only on water that
/// freezes. Ice terrain is never placed by itself: it lies on water, which it needs to thaw back
/// into.
fn parse_ice(ch: char, entry: &LegendEntry, terrain: &TerrainDef) -> Result<Option<u32>, String> {
    if terrain.ice.is_some() {
        return Err(format!(
            "legend `{ch}` places ice (`{}`) itself; lay it on water instead: \
             `{{ terrain = \"water\", ice = MM }}`",
            terrain.id
        ));
    }
    let Some(mm) = entry.detail().and_then(|d| d.ice) else {
        return Ok(None);
    };
    if terrain.freeze.is_none() {
        return Err(format!(
            "legend `{ch}` has `ice` but terrain `{}` is not water that freezes",
            terrain.id
        ));
    }
    if !(1..=crate::ice::MAX_MM).contains(&mm) {
        return Err(format!(
            "legend `{ch}` has `ice` {mm}; expected 1 to {} (mm)",
            crate::ice::MAX_MM
        ));
    }
    Ok(Some(u32::from(mm) * 1000))
}

/// The lock a legend entry puts on its cell: `lock = "id"` (a lock of that id) or `locked =
/// true` (a lock only a skeleton key opens); none for neither or `locked = false` alone.
fn parse_lock(detail: &LegendDetail) -> Result<Option<Lock>, String> {
    match (detail.locked, &detail.lock) {
        (Some(false), Some(_)) => Err("has a `lock` but `locked = false`".into()),
        (_, Some(id)) => {
            check_lock_id(id)?;
            Ok(Some(Lock::new(Some(id))))
        }
        (Some(true), None) => Ok(Some(Lock::new(None))),
        (Some(false) | None, None) => Ok(None),
    }
}

/// Parses a route like `"E4 S2 W4 N2"` (count defaults to 1).
pub fn parse_route(route: &str) -> Result<Vec<Dir>, String> {
    let mut steps = Vec::new();
    for token in route.split_whitespace() {
        let split = token
            .find(|c: char| c.is_ascii_digit())
            .unwrap_or(token.len());
        let (dir, count) = token.split_at(split);
        let dir: Dir = dir.parse()?;
        let count: usize = if count.is_empty() {
            1
        } else {
            count
                .parse()
                .map_err(|_| format!("bad step count in `{token}`"))?
        };
        steps.extend(std::iter::repeat_n(dir, count));
    }
    if steps.is_empty() {
        return Err("route is empty".into());
    }
    Ok(steps)
}

/// What a creature spawned carrying `carry` wears and wields: its `wield` objects, then its
/// `wear` ones, each the first carried stack of it not yet equipped, in the first free slot it
/// fits (a second ring on the other hand).
fn equip(detail: &LegendDetail, carry: &[Stack], content: &Content) -> Result<Equipment, String> {
    let mut equipment = Equipment::default();
    for (wields, list) in [(true, &detail.wield), (false, &detail.wear)] {
        let verb = if wields { "wield" } else { "wear" };
        for id in list.iter().flat_map(OneOrMany::ids) {
            let object = content
                .object_id(id)
                .ok_or_else(|| format!("{verb}: unknown object `{id}`"))?;
            let item = (0..carry.len())
                .find(|&i| carry[i].object == object && !equipment.holds(i))
                .ok_or_else(|| format!("{verb}s `{id}`, which it does not carry"))?;
            let slot = equipment::free_slot(content, &equipment, object)
                .ok_or_else(|| format!("cannot {verb} `{id}`: no free slot fits it"))?;
            let held = matches!(
                content.slot(slot).action,
                ActionKind::Wield | ActionKind::Ready
            );
            if held != wields {
                let (is, other) = if held {
                    ("wielded, not worn", "wield")
                } else {
                    ("worn, not wielded", "wear")
                };
                return Err(format!("`{id}` is {is}: use `{other}`"));
            }
            equipment.put(&equipment::slots_taken(content, object, slot), item);
        }
    }
    Ok(equipment)
}

/// A patrol must stay on walkable terrain and loop back to its start.
pub(crate) fn validate_route(
    level: &LevelTemplate,
    content: &Content,
    spawn: &CreatureSpawn,
) -> Result<(), String> {
    if spawn.route.is_empty() {
        return Ok(());
    }
    let mut pos = spawn.pos;
    for (i, dir) in spawn.route.iter().enumerate() {
        pos = pos.step(*dir);
        let walkable = level
            .terrain
            .get(pos)
            .is_some_and(|t| content.terrain(*t).walkable);
        if !walkable {
            return Err(format!(
                "patrol from {} leaves walkable terrain at step {} ({pos})",
                spawn.pos,
                i + 1
            ));
        }
    }
    if pos != spawn.pos {
        return Err(format!(
            "patrol from {} ends at {pos}; routes must loop back to their start",
            spawn.pos
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_routes() {
        assert_eq!(
            parse_route("E2 n SW3").unwrap(),
            vec![Dir::E, Dir::E, Dir::N, Dir::SW, Dir::SW, Dir::SW]
        );
        assert!(parse_route("").is_err());
        assert!(parse_route("X2").is_err());
    }
}
