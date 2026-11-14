//! Averted eyes, forced (spec §8 "Creature behavior": "Gazes", #1125; the owner's decision D4
//! of `docs/design/monster-dread.md` §17): while the player sees a hostile whose gaze is marked
//! `averted` (`content/gaze.rs`), they avert their eyes by reflex. No toggle, no action, no actor
//! flag, nothing saved: the condition is derived each turn from the perception snapshot, as
//! being dazzled is (`Perception::averted`), so it is out of the state hash. Whatever else keeps
//! the player from seeing the gazer (an unlit cell, a wall or a door, a `dark` zone, a disguise)
//! keeps them from being Averted, and its gaze from taking (`gaze.rs`).
//!
//! Averted, they fight half-blind: the `[averted]` to-hit of `data/rules/tuning.toml` on every
//! attack they make ([`World::averted_blow`], read in `fight.rs::resolve` beside the glare's),
//! their Dodge halved ([`World::averted_defence`], in `fight.rs::defence_of`), and no free blows
//! (`free_blow.rs::free_blows_of`). The gaze itself takes at its `glance` share of its chance.
//! [`World::avert_news`] says when it begins and ends.

use crate::combat::{Category, Defence, Profile};
use crate::world::{ActorId, World};

/// Said as the player averts their eyes.
const AVERTS: &str = "You avert your eyes.";
/// Said as they may look again.
const LOOKS: &str = "You raise your eyes again.";

impl World {
    /// Whether actor `id` is Averted now: the player, while their last perception sees a
    /// hostile whose gaze is marked `averted`.
    pub(in crate::world) fn averted(&self, id: ActorId) -> bool {
        id == self.state.player && self.perception.averted()
    }

    /// `profile`, an attack actor `attacker` makes, as averted eyes leave it.
    pub(in crate::world) fn averted_blow(
        &self,
        attacker: ActorId,
        mut profile: Profile,
    ) -> Profile {
        if self.averted(attacker) {
            profile.to_hit += self.content.tuning().averted.to_hit;
        }
        profile
    }

    /// `defence`, actor `id`'s, with its Dodge halved (a positive Dodge, rounded toward none)
    /// while it is Averted.
    pub(in crate::world) fn averted_defence(&self, id: ActorId, defence: Defence) -> Defence {
        let dodge = defence.bonus(Category::Dodge);
        if dodge > 0 && self.averted(id) {
            defence.plus(Category::Dodge, -(dodge - dodge / 2))
        } else {
            defence
        }
    }

    /// Tells the player they averted their eyes, or raised them, if they did since the
    /// perception before (`before`); a note that does not stop a rest.
    pub(in crate::world) fn avert_news(&mut self, before: bool) {
        match (before, self.perception.averted()) {
            (false, true) => self.note(AVERTS),
            (true, false) => self.note(LOOKS),
            _ => {}
        }
    }
}
