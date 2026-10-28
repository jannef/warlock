//! Knowledge (spec §8 "Skills": Knowledge; #667): know your enemy.
//!
//! - **The bonus**: the player's check to resist a creature's ability (a drain, exposure to a
//!   disease a creature's attack carries, struggling free of a creature's hold) adds what the
//!   skills with the [`Effect::Resist`] benefit give, read at how far each skill's value stands
//!   **above the creature's level** ([`Actor::skill_effect_above`]): nothing at or below it,
//!   growing with the margin by the benefit's curve in `data/skills.toml`, at most
//!   [`crate::skills::MAX_RESIST_BONUS`]. A rank of 0 stands above no creature, so in practice
//!   only a trained value past the creature's level helps (and an untrained skill gives nothing,
//!   as every skill). Creatures resisting each other's abilities get nothing: the player's
//!   Knowledge only. Nothing is said when it helps.
//! - **Marks**: each such check, whatever comes of it, is a [`Use::Resist`] (one mark draw of
//!   the training stream per rule, as every use); discovery experience (a first kill of a kind,
//!   an arrival that pays, `world/experience.rs`) is a [`Use::Discover`].
//!
//! [`World::resisting`] is the one hook: the three checks call it for their modifier. It draws
//! only from the training stream, never the gameplay stream, so a fight's rolls stay as they
//! were.

use super::{Actor, ActorId, World};
use crate::skills::{Effect, Use};

impl World {
    /// Actor `target` rolls a check resisting an ability of `by` (the creature whose ability it
    /// is, if any): for the player and a creature, marks a [`Use::Resist`] and returns what
    /// their Knowledge adds to the check ([`resist_bonus`]); 0 otherwise.
    pub(super) fn resisting(&mut self, target: ActorId, by: Option<ActorId>) -> i32 {
        if target != self.state.player {
            return 0;
        }
        let player = self.state.player;
        let Some(by) = by.filter(|&by| by != player && self.actor_or_fallen(by).is_some()) else {
            return 0;
        };
        // The creature's type's level, as weapon marks weigh it (`fight.rs`).
        let level = u16::try_from(self.level_of(by)).unwrap_or(u16::MAX);
        if self.trains(Use::Resist) {
            self.train(Use::Resist);
        }
        resist_bonus(&self.content, self.state.player(), level)
    }

    /// The player gained discovery experience: a [`Use::Discover`].
    pub(super) fn discovered(&mut self) {
        if self.trains(Use::Discover) {
            self.train(Use::Discover);
        }
    }
}

/// What `actor`'s skills add to a check resisting the ability of a creature of level `level`
/// ([`Effect::Resist`], read above the level).
pub(crate) fn resist_bonus(content: &crate::content::Content, actor: &Actor, level: u16) -> i32 {
    actor.skill_effect_above(content, Effect::Resist, level)
}
