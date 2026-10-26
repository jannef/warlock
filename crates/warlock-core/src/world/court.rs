//! Courts (spec §7 "Factions"): a creature type may list several factions with weights
//! (`factions = { seelie = 50, unseelie = 50 }`, the fey's), and each creature of it joins one
//! when it is made ([`join`], one call in `travel::make_creature`), drawn from its own stream
//! `court/<actor id>` ([`crate::rng::Rng::court`]), so no other draw moves. The pick is the
//! actor's `faction_override`, saved and hashed as a level's faction is; a level's own `faction`
//! for the creature wins over the draw.
//!
//! A faction with `provoked` (the Seelie court's: the wrathful fey) gives one of its creatures
//! the player attacks to that faction ([`World::provoke_court`], beside a shopkeeper's
//! outrage in `fight.rs` and `spell.rs`).

use super::{ActorId, World};
use crate::content::{Content, CreatureId, FactionId};
use crate::rng::Rng;

/// The faction a creature of type `creature`, made as actor `id` in the run of `seed`, joins:
/// one of its type's factions, drawn by weight; `None` for a type of one faction.
pub(super) fn join(
    content: &Content,
    seed: u64,
    id: ActorId,
    creature: CreatureId,
) -> Option<FactionId> {
    let factions = &content.creature(creature).factions;
    let total: u32 = factions.iter().map(|&(_, w)| w).sum();
    if total == 0 {
        return None;
    }
    let mut roll = Rng::court(seed, &id.0.to_string()).below(total);
    for &(faction, weight) in factions {
        if roll < weight {
            return Some(faction);
        }
        roll -= weight;
    }
    unreachable!("a roll below the total weight falls in a faction")
}

impl World {
    /// The player attacked `target`: a creature of a faction with `provoked` joins that one (a
    /// Seelie fey turns wrathful), and stays there.
    pub(super) fn provoke_court(&mut self, target: ActorId) {
        let Some(actor) = self.state.actors.get_mut(&target) else {
            return;
        };
        let provoked =
            (actor.faction(&self.content)).and_then(|f| self.content.factions().def(f).provoked);
        if let Some(faction) = provoked {
            actor.faction_override = Some(faction);
        }
    }
}
