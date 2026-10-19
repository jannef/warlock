//! Hand-authored level files (`assets/levels/*.toml`, spec §7).
//!
//! A level is a multiline ASCII `map` plus a `legend` mapping characters to terrain and to things
//! placed on that terrain (player start, creatures with patrol routes, what they carry, and what
//! of that they wield and wear (`wield = "spear"`, `wear = ["leather_jerkin"]`), the faction they
//! are placed in, and the unique character they are (`npc = "hermit"`, whose dialogue they
//! speak), stair links,
//! lights, the flow of running water, objects). Characters not in the level's legend fall
//! back to the shared default legend (`assets/data/legend.toml`, which has arrows for running
//! water). Objects may also be placed without a legend character: `[[pile]]` tables with `at =
//! [x, y]` and `objects`. Objects are written as ids with an optional count, top of the pile
//! first: `objects = ["dagger", "arrow*12"]`. An optional
//! `ambient = [r, g, b, uv]` lights the whole level. `kind` (`dungeon`, the default, `outdoor`,
//! or `overworld`) decides what a step there takes (spec §8 "Time"); outdoor levels and the
//! overworld are lit by the sun and the sky by day (spec §5). Optional
//! `[[region]]` tables name areas of the level (`name`, `area = [x0, y0, x1, y1]`, inclusive; the
//! first match wins), shown in the status bar.

use super::creature::CreatureId;
use super::faction::FactionId;
use super::flow::{FLOW_SPEEDS, Flow};
use super::light::LightId;
use super::object::parse_stacks;
use super::terrain::{StairsDir, TerrainId};
use super::{ActionKind, Content, ContentError, Speaker};
use crate::equipment::{self, Equipment};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::light::Light;
use crate::object::{INVENTORY_SLOTS, Stack, add_to_pile};
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
    map: String,
    #[serde(default)]
    legend: BTreeMap<String, LegendEntry>,
    #[serde(default)]
    region: Vec<RegionSpec>,
    /// Objects placed by position rather than by a legend character.
    #[serde(default)]
    pile: Vec<PileSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PileSpec {
    at: [i32; 2],
    /// Top first, e.g. `["dagger", "arrow*12"]`.
    objects: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionSpec {
    name: String,
    /// `[x0, y0, x1, y1]`, inclusive.
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
    /// Objects lying on this cell, top first, e.g. `["dagger", "arrow*12"]`.
    objects: Option<Vec<String>>,
    /// Objects the creature on this cell carries.
    carry: Option<Vec<String>>,
    /// Objects of what it carries that it wields (weapons, shields, missile weapons and
    /// missiles), by object id: `wield = "spear"` or a list.
    wield: Option<OneOrMany>,
    /// Objects of what it carries that it wears (armor, rings, amulets), by object id.
    wear: Option<OneOrMany>,
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
    /// Named areas, first match wins.
    pub regions: Vec<Region>,
    /// Where walking off the edge leads (local areas); `None`: the edge cannot be crossed.
    pub exit: Option<EdgeExit>,
}

impl LevelTemplate {
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
    let mut objects: BTreeMap<Pos, Vec<Stack>> = BTreeMap::new();

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
    if let Some(at) = objects
        .keys()
        .find(|p| !content.terrain(terrain[**p]).walkable)
    {
        return Err(err(format!(
            "objects at {at} lie on terrain `{}`, which is not walkable",
            content.terrain(terrain[*at]).id
        )));
    }

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
        regions,
        exit: None,
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
