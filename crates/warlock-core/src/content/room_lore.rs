//! What a scriptorium carries (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): items of
//! one class the generator lays on its floor ([`RoomLoot`], `loot = { class = "scrolls", count =
//! [2, 4] }`; `mapgen/room_loot.rs`), and skills it trains faster while one is inside
//! (`train = { decipher_script = 200 }`, percent of each training mark's chance; `World::train`).
//!
//! Both name content loaded after the effects (item classes come with the objects), so the
//! names are kept until `RoomEffects::resolve_lore` turns them into ids.

use super::ItemClassId;
use crate::skills::SkillId;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// How many items a room's `loot` may lay.
pub const LOOT_COUNT: RangeInclusive<u8> = 1..=16;
/// The percent of a training mark's chance a room may give.
pub const TRAIN: RangeInclusive<u16> = 1..=1000;

/// `loot = { class = "scrolls", count = [2, 4] }` as data writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LootSpec {
    class: String,
    count: [u8; 2],
}

/// The items the generator lays on such a room's floor: between `count[0]` and `count[1]` of
/// them, each a base item of `class` that is loot at the floor's item level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomLoot {
    pub class: ItemClassId,
    pub count: [u8; 2],
}

/// An effect's `loot` and `train` by name, until they are resolved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LoreNames {
    pub(super) loot: Option<(String, [u8; 2])>,
    pub(super) train: Vec<(String, u16)>,
}

impl LoreNames {
    /// Checks the numbers of `loot` and `train` and keeps their names.
    pub(super) fn check(
        loot: Option<LootSpec>,
        train: BTreeMap<String, u16>,
    ) -> Result<Self, String> {
        if let Some(l) = &loot {
            let [least, most] = l.count;
            if least > most || !LOOT_COUNT.contains(&least) || !LOOT_COUNT.contains(&most) {
                return Err(format!(
                    "loot.count is [least, most], each {} to {}, least at most most",
                    LOOT_COUNT.start(),
                    LOOT_COUNT.end()
                ));
            }
        }
        for (skill, percent) in &train {
            if !TRAIN.contains(percent) {
                return Err(format!(
                    "train.{skill} {percent} is out of range ({} to {})",
                    TRAIN.start(),
                    TRAIN.end()
                ));
            }
        }
        Ok(Self {
            loot: loot.map(|l| (l.class, l.count)),
            train: train.into_iter().collect(),
        })
    }

    /// The ids of these names: `class` resolves an item class, `skill` a skill. Content without
    /// skills at all (`skills` false: no `data/skills.toml`) trains none, so `train` is dropped.
    pub(super) fn resolve(
        &self,
        class: impl Fn(&str) -> Option<ItemClassId>,
        (skill, skills): (impl Fn(&str) -> Option<SkillId>, bool),
    ) -> Result<(Option<RoomLoot>, Vec<(SkillId, u16)>), String> {
        let loot = match &self.loot {
            Some((name, count)) => Some(RoomLoot {
                class: class(name).ok_or_else(|| format!("loot: unknown item class `{name}`"))?,
                count: *count,
            }),
            None => None,
        };
        let train = (self.train.iter())
            .filter(|_| skills)
            .map(|(name, percent)| {
                skill(name)
                    .map(|id| (id, *percent))
                    .ok_or_else(|| format!("train: unknown skill `{name}`"))
            })
            .collect::<Result<_, _>>()?;
        Ok((loot, train))
    }
}
