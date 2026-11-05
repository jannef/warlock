//! A vault's monster pools (spec §7 "Level file format"): `monsters = { pool = "level", count = 20
//! }` on a legend entry marks cells of a pool of creatures, drawn at the floor's danger when the
//! level is generated (`mapgen/population.rs`), so who is inside differs from one visit to the
//! next.
//!
//! `pool` is `"level"`, the floor's own population with its strays, or one tag or a list of them
//! (`"goblin"`, `["orc", "troll"]`: a tribe is its tag). A tag's types are those that may come as
//! strays ([`Content::may_stray`]: hostile to the player, no unique, no animated object) but
//! mimics (a vault places none), so a draw stays in the tribe and only reaches deeper in it.
//! `count` monsters come, each on a different marked cell of the entry; without one, every marked
//! cell gets one.
//!
//! The entry may say what a placed creature's may (spec §7 "Level file format"): `status` (each
//! draw is placed with it, `["asleep"]`), `carry` (the first placed carries it, a key to the
//! vault's lock: `check_keys` counts it within reach when every marked cell is), and `route` (each
//! draw walks it from its cell, checked from every marked cell as a placed creature's is).

use super::level::OneOrMany;
use super::{Content, CreatureId, StatusId};
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use serde::Deserialize;

/// The word in a pool for the floor's own population.
const LEVEL: &str = "level";

/// A legend's `monsters`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MonsterPoolSpec {
    pool: OneOrMany,
    count: Option<u32>,
}

/// Who a monster pool may bring.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonsterPool {
    /// The floor's own: its population, now and then a stray ([`super::danger::draw`]).
    Level,
    /// The types of its tags, sorted, each of its campaign or none where it is drawn.
    Tags(Vec<CreatureId>),
}

impl MonsterPool {
    /// The types of a tag pool a branch of `campaign` may draw; none for [`MonsterPool::Level`].
    pub fn types(&self, content: &Content, campaign: Option<&str>) -> Vec<CreatureId> {
        let MonsterPool::Tags(types) = self else {
            return Vec::new();
        };
        (types.iter().copied())
            .filter(|&c| {
                let of = content.creature(c).campaign.as_deref();
                of.is_none() || of == campaign
            })
            .collect()
    }

    /// Whether a tag pool of a vault of `campaign` holds nothing it may meet: a shared vault may
    /// stamp in any campaign, so any type will do there.
    pub(super) fn holds_none(&self, content: &Content, campaign: Option<&str>) -> bool {
        match (self, campaign) {
            (MonsterPool::Level, _) | (MonsterPool::Tags(_), None) => false,
            (MonsterPool::Tags(_), Some(_)) => self.types(content, campaign).is_empty(),
        }
    }
}

/// One legend entry's monster pool: how many it brings, and the cells they may stand on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VaultMonsters {
    pub pool: MonsterPool,
    pub count: u32,
    /// In row order.
    pub cells: Vec<Pos>,
    /// The statuses each draw is placed with, at their degrees (`status = ["asleep"]`).
    pub status: Vec<(StatusId, u16)>,
    /// What the first draw placed carries on top of its gear (`carry = ["iron_key"]`).
    pub carry: Vec<Stack>,
    /// The patrol each draw walks from its cell; empty: a short walk drawn for each.
    pub route: Vec<Dir>,
}

impl MonsterPoolSpec {
    /// The pool it names, with no cells yet; refuses a name that can bring nothing.
    pub(super) fn resolve(&self, content: &Content) -> Result<VaultMonsters, String> {
        if self.count == Some(0) {
            return Err("`count` must be at least 1".into());
        }
        let names = self.pool.ids();
        let pool = match names {
            [only] if only == LEVEL => MonsterPool::Level,
            _ => MonsterPool::Tags(tagged(content, names)?),
        };
        Ok(VaultMonsters {
            pool,
            count: self.count.unwrap_or(0),
            cells: Vec::new(),
            status: Vec::new(),
            carry: Vec::new(),
            route: Vec::new(),
        })
    }
}

impl VaultMonsters {
    /// With its cells known: its count, every cell by default; refuses more than the cells hold.
    pub(super) fn counted(mut self) -> Result<Self, String> {
        let cells = u32::try_from(self.cells.len()).unwrap_or(u32::MAX);
        if self.count > cells {
            return Err(format!(
                "`count` {} is more than the {cells} cells it marks",
                self.count
            ));
        }
        if self.count == 0 {
            self.count = cells;
        }
        Ok(self)
    }
}

/// The types of tags `names`, sorted; refuses a name that brings none.
fn tagged(content: &Content, names: &[String]) -> Result<Vec<CreatureId>, String> {
    let of_tag = |tag: &str| -> Vec<CreatureId> {
        (0..content.creature_defs().len())
            .map(|i| CreatureId(i as u16))
            .filter(|&c| content.may_stray(c) && !content.is_mimic(c))
            .filter(|&c| content.creature(c).tags.iter().any(|t| t == tag))
            .collect()
    };
    let mut types = std::collections::BTreeSet::new();
    for name in names {
        if name == LEVEL {
            return Err(format!("`{LEVEL}` stands alone in a pool, not with tags"));
        }
        let found = of_tag(name);
        if found.is_empty() {
            return Err(format!(
                "pool `{name}` is `{LEVEL}` or a tag, and no creature that may be drawn has that tag"
            ));
        }
        types.extend(found);
    }
    Ok(types.into_iter().collect())
}

// Tests: `mapgen/vault_monsters_tests.rs`.
