//! The ascetic's ki tricks (spec §8 "Class abilities"; `docs/design/ki-tricks.md` §6): class
//! actives of the ascetic's table, used from the abilities menu as [`Action::Use`] and paid in
//! ki by `class_ability.rs::use_class`, which also refuses them (no time, nothing spent) before
//! the spend. Each trick is its own section below.
//!
//! - **Patient defence** (`use:patient_defence`, level 18, for a point of ki and a wait's
//!   price): a degree of its status (Poised) on the player until their next action. The status's
//!   own `combat` gives Dodge +4, and while it lasts [`World::free_blows_of`] switches the
//!   riposte on, which answers a melee blow missed by the player's Parry **or dodged** (missed by
//!   their Dexterity or Dodge, [`World::poised`]). Given during the player's own priced action,
//!   the status sits on `World::fresh`, so that action does not ease it: the creatures' turns in
//!   between meet it, and the player's next action, whatever it is, ends it. Refused, no time and
//!   no ki, only by `may_use_class` with no ki; never for "nobody near", and a second use while
//!   Poised keeps it for one more action.
//! - **Step of the wind** (`use:step_of_the_wind`, level 6, for a point of ki): a degree of its
//!   status (Light-footed) on the player, **taking no time**: the next action they take, whatever
//!   it is, costs half its price and ends it. Refused, no time and no ki, while they are
//!   Light-footed already (one degree is the most it holds), and by `may_use_class` with no ki.
//!   The talent's passive half (Fleet's quicker steps, unarmoured and shieldless) is a
//!   `step_percent` read in `schedule.rs`.
//! - **Stunning strike** and the palm blow it shares with Quivering palm (below).
//! - **Diamond soul's catch** (level 40, a point of ki spent by itself; no action): a missile
//!   that would hit the bare-handed ascetic misses and falls at their feet
//!   ([`World::caught_missile`], from `resolve` and `trap_strikes`).
//!
//! [`Action::Use`]: crate::action::Action::Use

use super::{Actor, ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::combat::Blow;
use crate::content::{ActionKind, ObjectId, TrapId, the};
use crate::geom::Dir;
use crate::talent::PalmBlow;

// Patient defence (#849).
impl World {
    /// Patient defence: the player waits in their guard (a degree of its status, Poised), for
    /// the time of a wait; its ki was spent by `use_class`.
    pub(super) fn patient_defence(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(status) = self
            .state
            .player()
            .talent_mods(&self.content)
            .martial
            .patient_defence
        else {
            return Spent::Nothing;
        };
        self.afflict(player, status, 1, Some(player), events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Whether `actor` is Poised: it suffers the status of its Patient defence. Its riposte is
    /// on ([`World::free_blows_of`]) and answers a blow it dodged as well as one it parried.
    pub(super) fn poised(&self, actor: &Actor) -> bool {
        !actor.talents.is_empty()
            && (actor.talent_mods(&self.content).martial.patient_defence)
                .is_some_and(|s| actor.effects.has(s))
    }
}

// --- Step of the wind (#847) --------------------------------------------------------------------

impl World {
    /// Step of the wind's refusal before its ki is spent (no time; said): the player is
    /// Light-footed already.
    pub(super) fn may_wind_step(&mut self) -> bool {
        let content = self.content.clone();
        let me = self.state.player();
        let Some(status) = me.talent_mods(&content).martial.wind_step else {
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
        let Some(status) = self
            .state
            .player()
            .talent_mods(&self.content)
            .martial
            .wind_step
        else {
            return Spent::Nothing;
        };
        self.afflict(player, status, 1, Some(player), events);
        self.fresh.retain(|&(a, s)| (a, s) != (player, status));
        Spent::NoTime
    }
}

// --- Diamond soul's catch (#851) ----------------------------------------------------------------

impl World {
    /// Diamond soul's catch: a missile (a shot or a throw, `resolve`; a trap's, `trap_strikes`)
    /// that hit actor `target` is caught, if it has the catch (`catch_missiles`), holds no
    /// weapon and no shield, and holds its cost (a point of Ki), which is spent by itself. The
    /// hit is then a miss, the missile's flight ends on the target's cell, where it falls
    /// (`caught`), and the player is told after the attack's own words: "You pluck the arrow
    /// from the air." (`noun`, what flew). Returns whether it was caught. No draw.
    pub(super) fn caught_missile(
        &mut self,
        target: ActorId,
        noun: &str,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(me) = self.state.actors.get(&target) else {
            return false;
        };
        if me.talents.is_empty() {
            return false;
        }
        let Some(cost) = me.talent_mods(&self.content).martial.catch_missiles else {
            return false;
        };
        if me.weapon_item(&self.content).is_some() || me.shield_item(&self.content).is_some() {
            return false;
        }
        if !self.spend_resource(target, cost.resource, cost.amount, events) {
            return false;
        }
        self.caught = true;
        if target == self.state.player {
            self.news
                .push(format!("You pluck the {noun} from the air."));
        }
        true
    }

    /// `blow`, a shot or throw (`missile`) made `with` an object at actor `target`, as it
    /// stands once Diamond soul's catch has had its say (`resolve`'s one line): a hit caught is
    /// a plain miss.
    pub(super) fn caught_shot(
        &mut self,
        target: ActorId,
        (missile, with): (bool, Option<ObjectId>),
        blow: Option<Blow>,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        if !missile || !blow.is_some_and(|b| b.strike.is_hit()) {
            return blow;
        }
        let noun = with.map_or_else(|| "missile".into(), |o| self.content.object(o).name.clone());
        if self.caught_missile(target, &noun, events) {
            return Some(Blow::missed(None));
        }
        blow
    }

    /// `blow`, trap `trap`'s at actor `target`, a missile if it `flew`, as it stands once
    /// Diamond soul's catch has had its say (`trap_strikes`'s one line): a hit caught is a
    /// plain miss.
    pub(super) fn caught_trap_shot(
        &mut self,
        trap: TrapId,
        target: ActorId,
        flew: bool,
        blow: Blow,
        events: &mut Vec<Event>,
    ) -> Blow {
        if !flew || !blow.strike.is_hit() {
            return blow;
        }
        let noun = self.content.trap(trap).noun.clone();
        if self.caught_missile(target, &noun, events) {
            return Blow::missed(None);
        }
        blow
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
            .martial
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
        let Some(ends) = self
            .state
            .player()
            .talent_mods(&content)
            .martial
            .stillness
            .clone()
        else {
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
