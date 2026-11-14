//! Searching (`assets/data/rules/search.toml`, spec §8 "Searching"): the numbers of the one
//! perception check every hidden feature is found with, and secret doors as levels place them.
//!
//! Hidden features are secret doors and hidden traps. A **secret door** is a closed door that
//! looks and acts like the wall around it until it is found: the level's terrain there is that
//! wall ([`secret_look`]), and the door it really is waits beside it ([`Secret`], ground truth
//! like a lock). Level legends make one with `secret = true` (the default difficulty,
//! [`SearchRules::secret_door`]) or `secret = N` (difficulty N) on a closed door.
//!
//! Without the file the rules are [`SearchRules::default`].

use super::{Content, ContentError, TerrainId};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// How many turns searching for a while may last: what a `search:N` action may say, so the
/// action it sends reads back from its text (`crate::action::MAX_SEARCH`).
pub const SEARCH_TURNS: RangeInclusive<u8> = 1..=crate::action::MAX_SEARCH;

/// The numbers of searching.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRules {
    /// What a passive check loses against an active search (spec §8 "Searching": you notice
    /// less in passing than where you look): 4.
    pub passive_penalty: i32,
    /// How many turns searching for a while lasts at most: 10.
    pub turns: u8,
    /// A secret door's difficulty where its level gives none (`secret = true`, and the doors a
    /// generator hides): 15.
    pub secret_door: u8,
}

impl Default for SearchRules {
    fn default() -> Self {
        Self {
            passive_penalty: 4,
            turns: 10,
            secret_door: 15,
        }
    }
}

impl SearchRules {
    /// Parses `data/rules/search.toml`: `turns` within [`SEARCH_TURNS`].
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let rules: Self = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !SEARCH_TURNS.contains(&rules.turns) {
            return Err(err(format!(
                "turns {} is out of range ({} to {})",
                rules.turns,
                SEARCH_TURNS.start(),
                SEARCH_TURNS.end()
            )));
        }
        Ok(rules)
    }
}

impl super::RulesFile for SearchRules {
    const PATH: &'static str = "data/rules/search.toml";

    fn parse(path: &str, text: &str, _content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

/// Which hidden features a race notices in passing, untrained (spec §8 "Searching"; races'
/// `passive_search`): elves everything, dwarves what is on stone (terrain with `stone`),
/// kobolds hidden traps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PassiveSearch {
    All,
    Stone,
    Traps,
}

/// A secret door on a cell (ground truth, spec §8 "Searching"): the closed door it is, and how
/// hard it is to find. The cell's terrain is the wall it looks like until it is found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Secret {
    /// The closed door it becomes once found.
    pub door: TerrainId,
    /// Its difficulty (spec §8 "Checks"): what a search must reach.
    pub difficulty: u8,
}

/// A legend's `secret`: `true` (the default difficulty) or a difficulty.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum SecretSpec {
    Default(bool),
    Difficulty(u8),
}

impl SecretSpec {
    /// The difficulty it asks for (`None`: `secret = false`, no secret).
    pub(super) fn difficulty(self, rules: &SearchRules) -> Option<u8> {
        match self {
            SecretSpec::Default(false) => None,
            SecretSpec::Default(true) => Some(rules.secret_door),
            SecretSpec::Difficulty(d) => Some(d),
        }
    }
}

/// The wall a secret door at `p` looks like: the terrain of the wall around it, its solid
/// neighbors north, east, south, and west that are no door (rock deep in the ground counts as
/// the face it shows), the most common of them, ties to the first in that order. `None` without
/// any such neighbor: a secret door must stand in a wall.
pub fn secret_look(terrain: &Grid<TerrainId>, content: &Content, p: Pos) -> Option<TerrainId> {
    let walls: Vec<TerrainId> = [Dir::N, Dir::E, Dir::S, Dir::W]
        .iter()
        .filter_map(|d| terrain.get(p.step(*d)).copied())
        .filter_map(|t| {
            let def = content.terrain(t);
            let solid = !def.walkable && !def.transparent && !def.is_closed_door();
            solid.then(|| def.face.unwrap_or(t))
        })
        .collect();
    let count = |t: TerrainId| walls.iter().filter(|w| **w == t).count();
    walls.iter().copied().fold(None, |best, t| match best {
        Some(b) if count(b) >= count(t) => Some(b),
        _ => Some(t),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_parse_and_turns_stay_an_action_s() {
        let rules = SearchRules::parse("s", "passive_penalty = 4\nturns = 10\nsecret_door = 15\n")
            .expect("valid");
        assert_eq!(rules, SearchRules::default());
        for (text, message) in [
            (
                "passive_penalty = 4\nturns = 0\nsecret_door = 15\n",
                "turns 0",
            ),
            (
                "passive_penalty = 4\nturns = 101\nsecret_door = 15\n",
                "turns 101",
            ),
            ("passive_penalty = 4\nturns = 10\n", "missing field"),
        ] {
            let e = SearchRules::parse("s", text).unwrap_err();
            assert!(e.message.contains(message), "{text}: {}", e.message);
        }
    }

    #[test]
    fn a_secret_spec_is_the_default_or_a_difficulty() {
        let rules = SearchRules::default();
        assert_eq!(SecretSpec::Default(true).difficulty(&rules), Some(15));
        assert_eq!(SecretSpec::Default(false).difficulty(&rules), None);
        assert_eq!(SecretSpec::Difficulty(22).difficulty(&rules), Some(22));
    }

    #[test]
    fn a_secret_door_looks_like_the_wall_around_it() {
        let content = crate::test_support::builtin();
        let id = |name: &str| content.terrain_id(name).expect(name);
        let (wall, granite, floor, door, void) = (
            id("wall"),
            id("granite"),
            id("floor"),
            id("door_closed"),
            id("void"),
        );
        let mut grid = Grid::new(3, 3, floor);
        let middle = Pos::new(1, 1);
        assert_eq!(secret_look(&grid, &content, middle), None, "no wall");
        // In a wall running north to south, between granite (north) and stone (south): a tie,
        // to the first (north).
        grid.set(Pos::new(1, 0), granite);
        grid.set(Pos::new(1, 2), wall);
        assert_eq!(secret_look(&grid, &content, middle), Some(granite));
        // Two stone walls beat one granite; a door beside it is no wall.
        grid.set(Pos::new(0, 1), wall);
        grid.set(Pos::new(2, 1), door);
        assert_eq!(secret_look(&grid, &content, middle), Some(wall));
        // Solid rock counts as the face it shows.
        grid.set(Pos::new(1, 0), void);
        grid.set(Pos::new(0, 1), floor);
        assert_eq!(secret_look(&grid, &content, middle), Some(wall));
    }
}
