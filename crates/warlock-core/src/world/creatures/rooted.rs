//! Rooted creatures (docs/dev/creatures.md "Rooted", #1294): a creature whose type says
//! `rooted = true` ([`crate::content::CreatureDef::rooted`]) stands where it was made for as long
//! as it lives, as a miscast's gate does (docs/design/miscasts-review.md §6.2). It still strikes,
//! shoots, breathes, gazes, casts, and summons from there. One check a place that moves an actor:
//! - its own turn ([`World::creature_turn`] through [`World::rooted_plan`]): a step, a patrol, or
//!   a blink its AI or its fear chose is a wait instead; a coarse turn while the player rests
//!   (`stride.rs`) is a stride's waits; one with no natural attack and no weapon in hand (a
//!   gate, #1295) strikes no blow either: it has no hands to strike with, so a blow is a wait;
//! - it never gives way to the player (`talk.rs::refusal`, as `budge = "never"`), so no swap;
//! - it is never driven back (`weapon_active.rs::drive_back`: knockback, a shove, a spell's
//!   push), teleported (`trap.rs::teleport_to`, so every teleport, a rod's or a trap's, and a
//!   miscast's swap, which passes it by), or carried along on stairs, as a hostile follower
//!   (`stair_followers.rs`) or as an ally (`ally.rs::follow_along`: it stays behind).

use super::ai::Plan;
use super::{ActorId, World};
use crate::action::AttackKind;
use crate::content::ActionKind;

impl World {
    /// Whether actor `id` is of a rooted type (module docs). The player never is.
    pub(super) fn is_rooted(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| self.content.creature(a.creature).rooted)
            && id != self.state.player
    }

    /// What rooted creature `id` does of `plan` (module docs): anything that would move it is a
    /// wait; anything else stands.
    pub(super) fn rooted_plan(&self, id: ActorId, plan: Plan) -> Plan {
        match plan {
            Plan::Step(_) | Plan::Patrol | Plan::Blink(_) if self.is_rooted(id) => Plan::Wait,
            Plan::Attack {
                kind: AttackKind::Melee,
                ..
            } if self.is_rooted(id) && self.strikes_nothing(id) => Plan::Wait,
            plan => plan,
        }
    }

    /// Whether actor `id` has nothing to strike a blow with: no natural attack of its type and no
    /// weapon in hand (a gate's case, module docs).
    fn strikes_nothing(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| {
            self.content.creature(a.creature).attack.is_none() && a.weapon(&self.content).is_none()
        })
    }

    /// The price of a stride's waits for rooted creature `id`, which never strides (`stride.rs`).
    pub(super) fn rooted_stride(&self, id: ActorId) -> Option<i64> {
        self.is_rooted(id)
            .then(|| self.price_of(id, ActionKind::Wait) * i64::from(super::stride::STRIDE))
    }
}
