//! The frenzy stance (spec §8 "Stances"; #680): a berserker's rage puts them in it and holds
//! them there, and nothing else ever does. [`settle`] is the one rule: an actor raging with its
//! own Rage (the status its Rage ability brings) is in the frenzy stance, and one in the
//! frenzy stance that is not raging so is put in the aggressive stance. The world settles the
//! stance wherever a status begins (`World::afflict`) or ends (`World::ended`, and the rising
//! that burns every status away). `World::set_stance` refuses the frenzy stance, and any change
//! while it holds.

use super::{Actor, ActorId, Spent, World};
use crate::content::{CombatStance, Content};

/// Whether `actor` rages with its own Rage: the status its Rage ability brings is on it. A
/// creature, or anyone without the ability, never is, whatever statuses they suffer.
pub(super) fn raging(actor: &Actor, content: &Content) -> bool {
    (actor.talent_mods(content).martial.rage_status(content)).is_some_and(|s| actor.effects.has(s))
}

/// Puts `actor`'s stance right by the rule (the module's doc): returns the stance it was in, if
/// it changed.
pub(super) fn settle(actor: &mut Actor, content: &Content) -> Option<CombatStance> {
    let was = actor.stance;
    actor.stance = match (raging(actor, content), was) {
        (true, _) => CombatStance::Frenzy,
        (false, CombatStance::Frenzy) => CombatStance::Aggressive,
        (false, other) => other,
    };
    (actor.stance != was).then_some(was)
}

impl World {
    /// Settles actor `id`'s stance ([`settle`]), telling the player of theirs (a note: said after
    /// the action's own messages, and it does not stop a rest).
    pub(super) fn settle_stance(&mut self, id: ActorId) {
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if settle(actor, &self.content).is_none() || id != self.state.player {
            return;
        }
        // A note, said after the rage's own words.
        let text = match self.state.player().stance {
            CombatStance::Frenzy => "You fight in a frenzy.",
            _ => "You fall back into an aggressive stance.",
        };
        self.notes.push(text.into());
    }

    /// Why the player may not take `stance` now, said, if they may not: the frenzy comes
    /// only of rage, and rage holds them in it.
    pub(super) fn stance_refused(&mut self, stance: CombatStance) -> Option<Spent> {
        if raging(self.state.player(), &self.content) {
            self.say("Your rage holds you in the frenzy.");
            return Some(Spent::Nothing);
        }
        if stance == CombatStance::Frenzy {
            self.say("Only rage brings the frenzy.");
            return Some(Spent::Nothing);
        }
        None
    }
}
