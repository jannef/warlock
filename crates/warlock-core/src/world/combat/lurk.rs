//! Lurking (spec §8 "Creature behavior"; `docs/dev/surprise.md`): a creature type with
//! `lurks = true` that sees a hostile does not close in. It waits where it stands, aware of the
//! hostile (`ai.rs::decide` sets its `last_seen` as for anyone), until the hostile is beside it
//! (then it attacks as usual) or it has been harmed (hit points below its maximum: then it
//! hunts as any creature). A ranged lurker shoots from where it stands when it can, and a
//! summoner summons. Deterministic: no draws. A lurker that sees no one searches, listens, and
//! goes home as usual; its waits while the player rests away from it are coarse as anyone's
//! (`stride.rs`).

use super::ai::Plan;
use super::{ActorId, World};

impl World {
    /// What creature `id` does instead of `plan` (`ai.rs::fight`'s, against a hostile it sees):
    /// a step toward it becomes a wait while it lurks unharmed; anything else stands.
    pub(super) fn lurker_waits(&self, id: ActorId, plan: Plan) -> Plan {
        let me = &self.state.actors[&id];
        let lurks = self.content.creature(me.creature).lurks;
        match plan {
            Plan::Step(_) if lurks && me.hp >= me.max_hp(&self.content) => Plan::Wait,
            plan => plan,
        }
    }
}
