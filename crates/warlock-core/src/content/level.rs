//! Hand-authored level files (`assets/levels/*.toml`, spec §7).
//!
//! A level is a multiline ASCII `map` plus a `legend` mapping characters to terrain and to things
//! placed on that terrain (player start, creatures with patrol routes, stair links, lights, the
//! flow of running water). Characters not in the level's legend fall back to the shared default
//! legend (`assets/data/legend.toml`, which has arrows for running water). An optional
//! `ambient = [r, g, b, uv]` lights the whole level. `kind` (`dungeon`, the default, `outdoor`,
//! or `overworld`) and `turn_seconds` decide how time passes on it (spec §8 "Time"); outdoor
//! levels and the overworld are lit by the sun and the sky by day (spec §5). Optional
//! `[[region]]` tables name areas of the level (`name`, `area = [x0, y0, x1, y1]`, inclusive; the
//! first match wins), shown in the status bar.

use super::creature::CreatureId;
use super::flow::{FLOW_SPEEDS, Flow};
use super::light::LightId;
use super::terrain::{StairsDir, TerrainId};
use super::{Content, ContentError};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::light::Light;
use crate::time::{Duration, HOUR};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What kind of place a level is (spec §8 "Time"). It sets the default length of a turn there,
/// what a step costs, and whether the sun shines on it.
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

    /// How long a turn (a wait, opening a door, taking stairs) takes unless the level overrides
    /// it: six seconds on local levels, ten minutes on the overworld.
    pub const fn default_turn(self) -> Duration {
        match self {
            LevelKind::Dungeon | LevelKind::Outdoor => Duration::seconds(6),
            LevelKind::Overworld => Duration::minutes(10),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelFile {
    name: String,
    /// Light everywhere on the level, `[r, g, b, uv]`; default none.
    #[serde(default)]
    ambient: Light,
    #[serde(default)]
    kind: LevelKind,
    /// Length of a turn in seconds; default [`LevelKind::default_turn`].
    turn_seconds: Option<u64>,
    map: String,
    #[serde(default)]
    legend: BTreeMap<String, LegendEntry>,
    #[serde(default)]
    region: Vec<RegionSpec>,
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
    /// Terrain plus something placed on it.
    Detailed(LegendDetail),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LegendDetail {
    terrain: String,
    spawn: Option<Spawn>,
    creature: Option<String>,
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
            LegendEntry::Detailed(d) => Some(d),
        }
    }
}

/// A creature placed by a level file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureSpawn {
    pub creature: CreatureId,
    pub pos: Pos,
    /// Looping patrol route; empty means the creature stays put.
    pub route: Vec<Dir>,
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
    /// Light everywhere on the level.
    pub ambient: Light,
    pub kind: LevelKind,
    /// How long a turn takes here; a step takes this scaled by the terrain's `move_cost`.
    pub turn: Duration,
    pub terrain: Grid<TerrainId>,
    pub player_start: Option<Pos>,
    pub creatures: Vec<CreatureSpawn>,
    pub stairs: Vec<StairsLink>,
    /// Lights placed by the legend, by position.
    pub lights: BTreeMap<Pos, LightId>,
    /// Flow of every running-water cell, by position (presentation-only).
    pub flow: BTreeMap<Pos, Flow>,
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
    let turn = match parsed.turn_seconds {
        None => parsed.kind.default_turn(),
        Some(s) if (1..=HOUR).contains(&s) => Duration::seconds(s),
        Some(s) => {
            return Err(err(format!(
                "turn_seconds {s} is outside 1 to {HOUR} (one hour)"
            )));
        }
    };

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
            if let Some(creature) = &detail.creature {
                let id = content.creature_id(creature).ok_or_else(|| {
                    err(format!("legend `{ch}` uses unknown creature `{creature}`"))
                })?;
                let route = match &detail.route {
                    Some(r) => parse_route(r).map_err(|m| err(format!("legend `{ch}`: {m}")))?,
                    None => Vec::new(),
                };
                creatures.push(CreatureSpawn {
                    creature: id,
                    pos,
                    route,
                });
            } else if detail.route.is_some() {
                return Err(err(format!("legend `{ch}` has a route but no creature")));
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

    let template = LevelTemplate {
        id: id.to_string(),
        name: parsed.name,
        ambient: parsed.ambient,
        kind: parsed.kind,
        turn,
        terrain: Grid::from_vec(width, height, cells),
        player_start,
        creatures,
        stairs,
        lights,
        flow,
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
