//! The ascetic's ki tricks (spec §8 "Class abilities"; the ki tricks design §6.2): class actives
//! paid in ki ([`crate::content::ClassActive`]), each one function here with one line in
//! `use_class` (`world/class_ability.rs`).
//!
//! - **Step of the wind** (`use:step_of_the_wind`, level 6, for a point of ki): a degree of its
//!   status (Light-footed) on the player, **taking no time**: the next action they take, whatever
//!   it is, costs half its price and ends it. Refused, no time and no ki, while they are
//!   Light-footed already (one degree is the most it holds), and by `may_use_class` with no ki.
//!   The talent's passive half (Fleet's quicker steps, unarmoured and shieldless) is a
//!   `step_percent` read in `schedule.rs`.
//! - **Diamond soul's catch** (level 40, a point of ki spent by itself; no action): a missile
//!   that would hit the bare-handed ascetic misses and falls at their feet
//!   ([`World::caught_missile`], from `resolve` and `trap_strikes`).
//!
//! Nothing here draws.

use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::combat::Blow;
use crate::content::{ObjectId, TrapId};

// Step of the wind (#847).
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

// Diamond soul's catch (#851).
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
        let Some(cost) = me.talent_mods(&self.content).catch_missiles else {
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
