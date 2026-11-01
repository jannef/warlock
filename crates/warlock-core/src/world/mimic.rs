//! Mimics in play (spec §8 "Mimics"; docs/dev/creatures/mimics.md): a mimic is an ordinary
//! creature with the status that `hides` (Disguised), placed so by `travel::make_creature`.
//!
//! - **While disguised** it takes no turn (`creature_turn`, beside the sleepers), it is unaware of
//!   everyone (`ambush.rs`: a sneak attack lands), no creature sets on it (`ai.rs`), and the
//!   perception snapshot shows a shut chest of its band in its place and never it
//!   (`perception.rs`; Chest-wise sees through it within the talent's reach). Nothing it hears
//!   or sees, and no time, ends the disguise.
//! - **Revealing** ([`World::reveal_mimic`]) ends it, said as a line of its own when the player
//!   sees the cell (nothing is said of the status). It comes of a **touch** (opening, picking,
//!   disarming, kicking, or walking into what the player takes for a chest: the action is
//!   refused with no time, and the mimic strikes the toucher at once, a free blow with its
//!   hold), **Knock** at it (no blow), **a blow** or any harm (an attack at it, a missile, a
//!   burst, a trap), and **being found** (`search.rs`: `Hidden::Mimic`). A revealed mimic
//!   never hides again.
//! - **Chest-wise** (a talent): a mimic's grip never holds the player (`status.rs::afflict`).
//!
//! Nothing here draws but the free blow's own rolls (the gameplay stream), so a run that never
//! touches a mimic is unchanged.

use super::{ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::geom::Pos;

/// How a mimic is found out (module docs): what is said, and whether it strikes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reveal {
    /// The player's hand on it: it lurches, and strikes the toucher.
    Touch,
    /// Knock at it: it lurches, but the caster's touch was a spell, not a hand.
    Knocked,
    /// A blow or any harm: it shudders.
    Struck,
    /// Found out by searching: the player notices it breathing.
    Found,
}

impl World {
    /// Whether actor `id` hides as something else (a disguised mimic).
    pub(super) fn disguised(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| a.effects.hides(&self.content))
    }

    /// Whether actor `id` slips a hold by `by` (Chest-wise: `by` is a mimic).
    pub(super) fn slips(&self, id: ActorId, by: Option<ActorId>) -> bool {
        let mimic = (by.and_then(|b| self.state.actors.get(&b)))
            .is_some_and(|b| self.content.has_tag(b.creature, "mimic"));
        mimic
            && (self.state.actors.get(&id))
                .is_some_and(|a| !a.talents.is_empty() && a.talent_mods(&self.content).slick)
    }

    /// The disguised mimic at `p` on the current level, if one stands there.
    pub(super) fn mimic_at(&self, p: Pos) -> Option<ActorId> {
        let level = &self.state.current_level;
        let id = self.state.actor_at(level, p)?.id;
        self.disguised(id).then_some(id)
    }

    /// The player's hand on what they take for a chest at `p` (opening, picking, disarming,
    /// kicking, a step into it): a disguised mimic there is revealed and strikes them, and the
    /// action is refused with no time (`Some(Spent::NoTime)`); `None` where no mimic hides.
    /// A mimic the player sees for what it is (Chest-wise) is no chest to them: `None`.
    pub(super) fn touch(&mut self, p: Pos, events: &mut Vec<Event>) -> Option<Spent> {
        let mimic = self.mimic_at(p).filter(|&m| !self.perception.shows(m))?;
        self.reveal_mimic(mimic, Reveal::Touch, events);
        Some(Spent::NoTime)
    }

    /// Ends mimic `id`'s disguise (module docs), if it is disguised: the line, when the player
    /// sees its cell, and on a touch its blow at the player.
    pub(super) fn reveal_mimic(&mut self, id: ActorId, how: Reveal, events: &mut Vec<Event>) {
        let Some(status) = self.content.disguise_status() else {
            return;
        };
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if !actor.effects.remove(status) {
            return;
        }
        let (pos, creature) = (actor.pos, actor.creature);
        // Found out by searching, the player knows it in the dark too.
        if how == Reveal::Found || self.perception.in_sight(pos) {
            let def = self.content.creature(creature);
            let chest = (self.content.chests().band_of(def.level))
                .map_or_else(|| "chest".to_string(), |b| b.name.clone());
            let what = format!("{} {}", crate::content::article(&def.name), def.name);
            self.say(match how {
                Reveal::Touch | Reveal::Knocked => format!("The {chest} lurches: it is {what}!"),
                Reveal::Struck => format!("The {chest} shudders: it is {what}!"),
                Reveal::Found => format!("You notice the {chest} breathing: {what}!"),
            });
        }
        if how == Reveal::Touch {
            let player = self.state.player;
            self.attack(id, player, AttackKind::Melee, events);
        }
    }
}
