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

use crate::content::{Content, CreatureId, FactionId};
use crate::rng::Rng;
use crate::world::{ActorId, World};

/// The faction a creature of type `creature`, made as the actor keyed `key` in the run of
/// `seed`, joins: one of its type's factions, drawn by weight; `None` for a type of one faction.
pub(in crate::world) fn join(
    content: &Content,
    seed: u64,
    key: &str,
    creature: CreatureId,
) -> Option<FactionId> {
    let factions = &content.creature(creature).factions;
    if factions.is_empty() {
        return None;
    }
    let mut rng = Rng::court(seed, key);
    rng.weighted(factions, |&(_, w)| w)
        .map(|&(faction, _)| faction)
}

impl World {
    /// The player attacked `target`: a creature of a faction with `provoked` joins that one (a
    /// Seelie fey turns wrathful), and stays there.
    pub(in crate::world) fn provoke_court(&mut self, target: ActorId) {
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
