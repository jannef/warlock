//! Beast-friend (spec §8 "Talents"): creatures of animal factions (`animal = true` in
//! `data/factions.toml`: the vermin, the wildlife) are neutral to a player with the talent,
//! until the player attacks one: from then on that creature type stands toward them as its
//! faction does (`Progress::empathy_broken`, by creature type id, saved). Every stance between
//! the player and a creature goes through [`befriended`]: [`World::stance`], the creatures'
//! choice of whom to fight ([`super::ai`]), and what look mode says.

use super::{Actor, ActorId, ActorKind, World, WorldState};
use crate::content::Content;

/// Whether `a` and `b` are a Beast-friend player and a creature of an animal faction whose type
/// they have not attacked (either way round).
pub(crate) fn befriended(state: &WorldState, content: &Content, a: &Actor, b: &Actor) -> bool {
    let (player, other) = match (a.kind, b.kind) {
        (ActorKind::Player, _) => (a, b),
        (_, ActorKind::Player) => (b, a),
        _ => return false,
    };
    other.kind == ActorKind::Creature
        && other
            .faction(content)
            .is_some_and(|f| content.factions().def(f).animal)
        && player.talent_mods(content).beast_friend
        && !(state.progress.empathy_broken).contains(&content.creature(other.creature).id)
}

impl World {
    /// The player attacked `target`: a creature of an animal faction they befriended is theirs
    /// to fight from now on, and every creature of its type.
    pub(super) fn break_empathy(&mut self, target: ActorId) {
        let (Some(player), Some(other)) = (
            self.state.actors.get(&self.state.player),
            self.state.actors.get(&target),
        ) else {
            return;
        };
        if befriended(&self.state, &self.content, player, other) {
            let id = self.content.creature(other.creature).id.clone();
            self.state.progress.empathy_broken.insert(id);
        }
    }
}
