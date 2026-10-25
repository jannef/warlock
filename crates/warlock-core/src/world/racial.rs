//! The racial talents in the world (spec §8 "Talents", the Racial tree): what a race grants
//! (Versatile, the humans'), put into the player's talents with the birth sign's gifts
//! (`stars.rs`), and Small target, the one racial rule that needs two actors. The others are
//! effects read where their rules live, as every talent's are.

use super::{ActorId, World};
use crate::character::Character;
use crate::combat::{Category, Target};
use crate::content::Content;

/// The talents `character`'s race grants, in data order (none for a run without a character).
pub(super) fn grants<'c>(content: &'c Content, character: Option<&Character>) -> Vec<&'c str> {
    (character.and_then(|c| content.race(&c.spec.race))).map_or(Vec::new(), |r| {
        r.talents.iter().map(String::as_str).collect()
    })
}

impl World {
    /// `struck`, what actor `target` defends with against `attacker`, with Small target's Dodge
    /// when the attacker is larger than it.
    pub(super) fn small_target(
        &self,
        attacker: ActorId,
        target: ActorId,
        struck: Target,
    ) -> Target {
        let (Some(a), Some(t)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
        ) else {
            return struck;
        };
        if t.talents.is_empty() {
            return struck;
        }
        let content = &self.content;
        let dodge = t.talent_mods(content).small_target;
        if dodge == 0 || self.state.size_of(content, a) <= self.state.size_of(content, t) {
            return struck;
        }
        let mut struck = struck;
        struck.defence = struck.defence.plus(Category::Dodge, dodge);
        struck
    }
}
