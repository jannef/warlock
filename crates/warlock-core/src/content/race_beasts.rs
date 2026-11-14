//! The starting beasts by race (#1567): the `[start]` table the companion's and the familiar's
//! data share (`data/companions.toml`, `data/familiars.toml`), the lists by race and the rares,
//! with the checks both files make, and the names check. The rules that draw from them are
//! `world/allies/companion.rs` and `world/allies/familiar.rs`.

use super::{Content, ContentError, CreatureId};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Every race's starting beasts and the rares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceBeasts {
    /// The jackpot, in a thousand: a beast of `rare` instead of the race's list.
    pub rare_per_mille: u32,
    /// The rares, shared by every race, in file order.
    pub rare: Vec<CreatureId>,
    /// A race's list holds beasts of at most this level.
    pub most_level: u8,
    /// Every race's starting beasts, by race id, in file order.
    pub by_race: Vec<(String, Vec<CreatureId>)>,
}

/// The `[start]` table as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StartSpec {
    rare_per_mille: u32,
    rare: Vec<String>,
    most_level: u8,
    by_race: BTreeMap<String, Vec<String>>,
}

impl RaceBeasts {
    /// The beasts of race `race`'s list, if it has one.
    pub fn list_of(&self, race: &str) -> Option<&[CreatureId]> {
        (self.by_race.iter())
            .find(|(r, _)| r == race)
            .map(|(_, list)| list.as_slice())
    }

    /// Every beast a start may draw, the races' lists and the rares, each once, in file order.
    pub fn every_start(&self) -> Vec<CreatureId> {
        let mut out: Vec<CreatureId> = Vec::new();
        for id in (self.by_race.iter().flat_map(|(_, l)| l.iter())).chain(&self.rare) {
            if !out.contains(id) {
                out.push(*id);
            }
        }
        out
    }

    /// Reads `[start]` of `path` once the creatures and the races are known: every list is a
    /// known race's and holds known creatures (a race without a list draws no starting beast:
    /// test fixtures add races), and the lists are not empty (the draws index them).
    pub(super) fn parse(
        path: &str,
        start: StartSpec,
        content: &Content,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        // The draws index the lists.
        if start.rare.is_empty() {
            return Err(err("`start.rare` lists at least one beast".into()));
        }
        let rare = (start.rare.iter())
            .map(|n| beast(path, content, "`start.rare`", n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut by_race = Vec::new();
        for race in &content.races().list {
            let Some(names) = start.by_race.get(&race.id) else {
                continue;
            };
            if names.is_empty() {
                return Err(err(format!(
                    "`start.by_race.{}` lists at least one beast",
                    race.id
                )));
            }
            let where_ = format!("`start.by_race.{}`", race.id);
            let list = (names.iter())
                .map(|name| beast(path, content, &where_, name))
                .collect::<Result<Vec<_>, _>>()?;
            by_race.push((race.id.clone(), list));
        }
        // A list for a race the content lacks (where it has races at all: a fixture without
        // `races.toml` keeps the shipped file).
        if !content.races().list.is_empty()
            && let Some(race) = (start.by_race.keys()).find(|r| content.race(r).is_none())
        {
            return Err(err(format!("`start.by_race`: unknown race `{race}`")));
        }
        Ok(RaceBeasts {
            rare_per_mille: start.rare_per_mille,
            rare,
            most_level: start.most_level,
            by_race,
        })
    }
}

/// The creature `name` names at `where_` in `path`, a beast (never the player).
pub(super) fn beast(
    path: &str,
    content: &Content,
    where_: &str,
    name: &str,
) -> Result<CreatureId, ContentError> {
    (content.creature_id(name))
        .filter(|_| name != "player")
        .ok_or_else(|| ContentError::new(path, format!("{where_}: unknown creature `{name}`")))
}

/// The names a kept beast may be called, refused empty (the draw indexes them).
pub(super) fn names(path: &str, names: Vec<String>) -> Result<Vec<String>, ContentError> {
    if names.is_empty() {
        return Err(ContentError::new(path, "`names`: at least one name"));
    }
    Ok(names)
}
