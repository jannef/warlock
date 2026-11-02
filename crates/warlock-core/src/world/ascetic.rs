//! The ascetic's ki tricks (spec §8 "Class abilities"; `docs/design/ki-tricks.md` §6): class
//! actives of the ascetic's table, used from the abilities menu as [`Action::Use`] and paid in
//! ki by `class_ability.rs::use_class`, which also refuses them (no time, nothing spent) before
//! the spend. Each trick is its own section below.
//!
//! - **Step of the wind** (`use:step_of_the_wind`, level 6, for a point of ki): a degree of its
//!   status (Light-footed) on the player, **taking no time**: the next action they take, whatever
//!   it is, costs half its price and ends it. Refused, no time and no ki, while they are
//!   Light-footed already (one degree is the most it holds), and by `may_use_class` with no ki.
//!   The talent's passive half (Fleet's quicker steps, unarmoured and shieldless) is a
//!   `step_percent` read in `schedule.rs`.
//! - **Stunning strike** and the palm blow it shares with Quivering palm (below).
//!
//! [`Action::Use`]: crate::action::Action::Use

use super::{Spent, World};
use crate::action::{AttackKind, Event};
use crate::content::{ActionKind, the};
use crate::geom::Dir;
use crate::talent::PalmBlow;

// --- Step of the wind (#847) --------------------------------------------------------------------

impl World {
    /// Step of the wind's refusal before its ki is spent (no time; said): the player is
    /// Light-footed already.
    pub(super) fn may_wind_step(&mut self) -> bool {
        let content = self.content.clone();
        let me = self.state.player();
        let Some(status) = me.talent_mods(&content).wind_step else {
            return false;
        };
        if me.effects.has(status) {
            self.say("You are already light on your feet.");
            return false;
        }
        true
    }

    /// Step of the wind: a degree of its status on the player, taking no time; its ki was spent
    /// by `use_class`. The status is no fresh one of an action under way (as one given during a
    /// priced action is, `status.rs`): this took no time, so the player's next action eases it.
    pub(super) fn wind_step(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(status) = self.state.player().talent_mods(&self.content).wind_step else {
            return Spent::Nothing;
        };
        self.afflict(player, status, 1, Some(player), events);
        self.fresh.retain(|&(a, s)| (a, s) != (player, status));
        Spent::NoTime
    }
}

// --- Stunning strike and the palm blow (#848) ---------------------------------------------------

impl World {
    /// Stunning strike (`use:stunning_strike:E`): a blow of the player's bare hand at the
    /// hostile in `dir`; a hit that does damage leaves it Stunned (flat-footed, its next action
    /// slowed; `world/surprise.rs`), "You stun the kobold." Its ki was spent by `use_class`,
    /// which refused (no time) with a weapon in hand or no hostile there.
    pub(super) fn stunning_strike(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let Some(strike) = self
            .state
            .player()
            .talent_mods(&self.content)
            .stunning_strike
        else {
            return Spent::Nothing;
        };
        self.palm_blow(dir, strike, Some("stun"), events)
    }

    /// The palm blow, Stunning strike's and Quivering palm's one body: a blow of the player's
    /// bare hand at the hostile in `dir`, at the price of a melee attack (the unarmed skill's
    /// and the style's speed counted); a hit that does damage leaves the target `palm`'s
    /// degrees of its status. With a `verb`, the player is told when the status took hold on
    /// one they see ("You stun the kobold."), before whatever answers the blow.
    pub(super) fn palm_blow(
        &mut self,
        dir: Dir,
        palm: PalmBlow,
        verb: Option<&str>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let price = self.attack_price(player, ActionKind::Melee, None, melee);
        let profile = me.melee(&content);
        let blow = self.player_blow_with(target, seen, profile, events);
        let wounded = blow.is_some_and(|b| b.damage > 0);
        if wounded && self.state.actors.contains_key(&target) {
            self.afflict(target, palm.status, palm.severity, Some(player), events);
            let took = (self.state.actors.get(&target))
                .filter(|a| a.effects.has(palm.status))
                .filter(|a| self.perception.sees_actors_at(a.pos))
                .map(|a| content.creature(a.creature).name.clone());
            if let (Some(verb), Some(name)) = (verb, took) {
                self.say(format!("You {verb} {}.", the(&name)));
            }
        }
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }
}

// --- Stillness (#850) ---------------------------------------------------------------------------

impl World {
    /// Stillness (`use:stillness`, Still mind's active at level 25): every status of its list
    /// the player suffers ends at once, as if cured ([`World::ended`]: each says its `end`
    /// words), **taking no time**, so the player acts this turn, held fast or stunned no longer.
    /// Its two ki were spent by `use_class`; nothing to end refused it before that
    /// ([`crate::ability::Unavailable::Still`]: "Your mind is already still."). Draws nothing.
    pub(super) fn stillness(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let content = self.content.clone();
        let Some(ends) = self.state.player().talent_mods(&content).stillness.clone() else {
            return Spent::Nothing;
        };
        self.say("You still your mind.");
        let me = self.player_mut();
        let ended: Vec<crate::status::Effect> = (ends.iter())
            .filter_map(|&status| me.effects.get(status).copied())
            .collect();
        for e in &ended {
            me.effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        Spent::NoTime
    }
}
