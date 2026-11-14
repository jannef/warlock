//! A town's goodwill (spec §8 "Dialogue": "Goodwill", #1247): every quest the player completes
//! for one of a town's people (`content/town.rs`, a campaign's `data/towns/`) makes that town's
//! services cheaper by the town's `goodwill` percent, after every other discount, up to free.
//!
//! The count is derived from the quest records (each completion of a repeatable quest counts),
//! so nothing new is saved. [`World::complete_saying`] tells the player, in the town's `told`
//! words, as it rises; [`World::service_price`] takes the discount off.

use super::World;
use crate::content::{DialogueId, TownDef};

/// The most a town's goodwill takes off, in percent: everything.
const MOST: u32 = 100;

impl World {
    /// How many quests the player has completed for `town`'s people.
    pub fn goodwill(&self, town: &TownDef) -> u32 {
        (self.state.quests.records.iter())
            .filter_map(|(id, record)| {
                let q = self.content.quest_id(id)?;
                let giver = self.content.quest(q).giver;
                (self.content.town_of(giver)? == town).then_some(record.completed)
            })
            .fold(0u32, u32::saturating_add)
    }

    /// The percent the goodwill of the town whoever speaks `dialogue` belongs to takes off their
    /// services: its `goodwill` per quest completed for its people, at most 100.
    pub fn goodwill_off(&self, dialogue: DialogueId) -> u32 {
        self.content.town_of(dialogue).map_or(0, |town| {
            (self.goodwill(town).saturating_mul(town.goodwill)).min(MOST)
        })
    }

    /// Says the town's `told` words when a quest from `giver`, one of a town's people, has just
    /// been completed.
    pub(super) fn tell_goodwill(&mut self, giver: DialogueId) {
        if let Some(told) = self.content.town_of(giver).map(|t| t.told.clone()) {
            self.say(told);
        }
    }
}

/// `price` less `off` percent, rounded up: free only at 100% off.
pub(super) fn discounted(price: u32, off: u32) -> u32 {
    let left = u64::from(MOST.saturating_sub(off));
    u32::try_from((u64::from(price) * left).div_ceil(u64::from(MOST))).unwrap_or(u32::MAX)
}
