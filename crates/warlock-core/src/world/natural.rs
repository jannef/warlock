//! Several natural attacks (spec §8 "Combat": "Natural attacks", #1099): a creature type with
//! a list of them (`[[creature.attack]]`, a dragon's two claws and a bite) strikes each in turn
//! in one melee attack, while it wields no weapon. The first is its melee blow as ever
//! ([`super::Actor::melee`]); the rest follow it here, each rolled on its own against the
//! target's Defence, as every blow is ([`World::attack_with`]), until the target or the
//! attacker is gone. Each blow's event names the natural attack it was made with, so it is said
//! by its own words ("The fire drake claws you."). They take no time of their own: the attack's
//! price is the one melee attack's.

use super::free_blow;
use super::{ActorId, World};
use crate::action::{AttackKind, Event};

impl World {
    /// The natural attacks after the first of `attacker`'s melee attack on `target` (module
    /// docs): whether any of them was parried, dodged, or hit.
    pub(super) fn more_natural_blows(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> (bool, bool, bool) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&attacker) else {
            return (false, false, false);
        };
        let mut made = (false, false, false);
        for (index, profile) in me.more_natural_blows(&content) {
            let standing = |w: &World, id| w.state.actors.contains_key(&id);
            if self.state.death.is_some() || !standing(self, attacker) || !standing(self, target) {
                break;
            }
            let start = events.len();
            let blow = self.attack_with(attacker, target, AttackKind::Melee, Some(profile), events);
            // The blow's own event: said with this attack's words (`perceive.rs`).
            if let Some(Event::Attacked { natural, .. }) = events[start..].iter_mut().find(|e| {
                matches!(e, Event::Attacked { attacker: a, kind: AttackKind::Melee, .. }
                    if *a == attacker)
            }) {
                *natural = Some(index);
            }
            made.0 |= free_blow::parried(blow);
            made.1 |= free_blow::dodged(blow);
            made.2 |= blow.is_some_and(|b| b.strike.is_hit());
        }
        made
    }
}
