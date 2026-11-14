//! Beast-friend (spec §8 "Talents"): creatures of animal factions (`animal = true` in
//! `data/factions/`: the vermin, the wildlife) are neutral to a player with the talent, and
//! creatures whose type carries a tag a `kin_to` talent names (Rat-king's court's vermin) to its
//! holder, until the player attacks one: from then on that creature type stands toward them as its
//! faction does (`Progress::empathy_broken`, by creature type id, saved). Every stance between
//! the player and a creature goes through [`befriended`]: [`World::stance`], the creatures'
//! choice of whom to fight ([`super::ai`]), and what look mode says. A rolled quest never asks
//! a Beast-friend to kill a creature type of an animal faction ([`calms`], `world/quest_roll.rs`).

use super::{Actor, ActorId, ActorKind, World, WorldState};
use crate::content::{Content, CreatureId};

/// Whether `a` and `b` are a Beast-friend player and a creature of an animal faction, or a
/// player akin to a tag the creature's type carries (`kin_to`), whose type they have not attacked
/// (either way round).
pub(crate) fn befriended(state: &WorldState, content: &Content, a: &Actor, b: &Actor) -> bool {
    let (player, other) = match (a.kind, b.kind) {
        (ActorKind::Player, _) => (a, b),
        (_, ActorKind::Player) => (b, a),
        _ => return false,
    };
    other.kind == ActorKind::Creature
        && calms(content, player, other.creature)
        && !(state.progress.empathy_broken).contains(&content.creature(other.creature).id)
}

/// Whether `player`'s talents calm creature type `creature`, by its own faction and tags
/// (whether or not they have attacked one since): a Beast-friend and a type of an animal faction,
/// or a `kin_to` tag the type carries. A rolled quest leaves such a type out of its pool
/// (`world/quest_roll.rs`), and perception shows it neutral until its type is struck.
pub(crate) fn calms(content: &Content, player: &Actor, creature: CreatureId) -> bool {
    if player.talents.is_empty() {
        return false;
    }
    let beasts = &player.talent_mods(content).beast;
    (beasts.beast_friend
        && (content.creature(creature).faction).is_some_and(|f| content.factions().def(f).animal))
        || (beasts.kin_tags.iter()).any(|tag| content.has_tag(creature, tag))
}

impl World {
    /// The player attacked `target`: a creature they befriended (Beast-friend, `kin_to`) is
    /// theirs to fight from now on, and every creature of its type.
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
