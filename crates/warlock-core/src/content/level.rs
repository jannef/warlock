//! Hand-authored level files (`assets/levels/*.toml`, spec §7).
//!
//! A level is a multiline ASCII `map` plus a `legend` mapping characters to terrain and to things
//! placed on that terrain (player start, creatures with patrol routes, stair links). Characters not
//! in the level's legend fall back to the shared default legend (`assets/data/legend.toml`).

use super::creature::CreatureId;
use super::terrain::{StairsDir, TerrainId};
use super::{Content, ContentError};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelFile {
    name: String,
    map: String,
    #[serde(default)]
    legend: BTreeMap<String, LegendEntry>,
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
#[derive(Clone, Debug)]
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

/// A parsed, validated level ready to instantiate.
#[derive(Clone, Debug)]
pub struct LevelTemplate {
    pub id: String,
    pub name: String,
    pub terrain: Grid<TerrainId>,
    pub player_start: Option<Pos>,
    pub creatures: Vec<CreatureSpawn>,
    pub stairs: Vec<StairsLink>,
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

    let template = LevelTemplate {
        id: id.to_string(),
        name: parsed.name,
        terrain: Grid::from_vec(width, height, cells),
        player_start,
        creatures,
        stairs,
    };
    for spawn in &template.creatures {
        validate_route(&template, content, spawn).map_err(err)?;
    }
    Ok(template)
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
fn validate_route(
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
