//! Sic 'em, the ranger's level 40 (spec §8 "Allies": "Sic 'em"; docs/design/ranger-companion.md
//! §11.3; `docs/dev/companion.md`): a class active aimed at a creature the player sees
//! ([`crate::action::Action::UseAt`], `use:sic:12,5`), for Focus and the time of a wait.
//!
//! - **The hunt**: the companion takes `Hunt` at the creature's cell, with the creature as its
//!   [`CompanionData::quarry`](super::companion::CompanionData). While the beast senses it
//!   (sight, then the nose), it goes for that creature before any other, and the hunt's cell
//!   follows it where the player sees that cell (the cell shows in the dumps, so it never tells
//!   of what the player cannot see; Hunter's bond, which every ranger of this level has, shows
//!   them what the beast senses). Lost, it is the ordinary hunt to the last cell. Any new order
//!   ends it (`World::set_order`).
//! - **Savage** (`data/status/`): the talent's `blows` degrees of a status with
//!   `auto_critical`. While the attacker suffers one, its blows are
//!   [`Profile::sure_critical`](crate::combat::Profile), the read `combat::strike_with` already
//!   has for an exploit's sure critical, so a critical comes with no confirmation roll; a target with no
//!   vitals still takes a plain hit. Each blow that hits spends a degree. The status's `cost`
//!   makes the beast half again as fast. It is no buff spell's and no potion's status, so
//!   Hunter's bond never carries it to the ranger.
//!
//! Refused, taking no time and spending nothing, with no companion on the level, or at a cell
//! with no creature the player sees, out of reach, or not hostile to them. Nothing is drawn.

use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::combat::Profile;
use crate::content::{ActionKind, ClassActive, Content, Stance};
use crate::geom::Pos;
use crate::noise::Act;
use crate::skills::Use;
use crate::world::{Actor, Order, WorldState, stance_between};

impl World {
    /// Sic 'em at `at` (the module docs); `use_class_at` has checked the ability and its focus.
    pub(super) fn sic(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(sic) = self.state.player().talent_mods(&content).sic else {
            return Spent::Nothing;
        };
        let companion = self.companion().filter(|id| {
            let pet = &self.state.actors[id];
            pet.level == self.state.current_level && !pet.in_retinue()
        });
        let Some(companion) = companion else {
            self.say("You have no companion here.");
            return Spent::Nothing;
        };
        if !self.within_reach(at, sic.range, "set your companion") {
            return Spent::Nothing;
        }
        let quarry = (self.state.actor_at(&self.state.current_level, at))
            .filter(|a| a.id != player && a.id != companion)
            // A disguised mimic is a chest to every eye (`mimic.rs`).
            .filter(|a| self.perception.sees_actors_at(at) && !a.effects.hides(&content))
            .map(|a| (a.id, a.creature));
        let who = self.ally_subject(companion);
        let Some((quarry, creature)) = quarry else {
            self.say(format!("There is nothing there to set {who} on."));
            return Spent::Nothing;
        };
        let state = &self.state;
        if stance_between(state, &content, state.player(), &state.actors[&quarry])
            != Stance::Hostile
        {
            self.say(format!("You have no quarrel there to set {who} on."));
            return Spent::Nothing;
        }
        if let Some(cost) = (content.class_active(ClassActive::Sic)).and_then(|a| a.resource) {
            self.spend_resource(player, cost.resource, cost.amount, events);
        }
        self.set_order(companion, Order::Hunt { at });
        if let Some(data) = (self.state.actors.get_mut(&companion)).and_then(Actor::companion_mut) {
            data.quarry = Some(quarry);
        }
        self.afflict(companion, sic.status, sic.blows, None, events);
        let name = crate::content::the_start(&content.creature(creature).name).to_lowercase();
        self.say(format!("{who} goes for {name}'s throat."));
        self.make_noise(player, Act::Order);
        events.push(Event::Ordered {
            at: self.player_pos(),
        });
        if self.trains(Use::Handle) {
            self.train(Use::Handle);
        }
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// What companion `id` hunts for Sic 'em, if it senses it now, as `ally_plan`'s target: the
    /// hunt's cell moves with it where the player sees that (the module docs).
    pub(super) fn quarry_sensed(
        &mut self,
        id: ActorId,
        hostile: &dyn Fn(&WorldState, &Content, &Actor, &Actor) -> bool,
    ) -> Option<(ActorId, Pos, bool)> {
        let quarry = self.state.actors.get(&id)?.companion()?.quarry?;
        let found = self.senses(id, &|s, c, me, a| a.id == quarry && hostile(s, c, me, a))?;
        if self.perception.is_visible(found.1)
            && let Some(ally) = (self.state.actors.get_mut(&id)).and_then(|a| a.ally.as_mut())
        {
            ally.order = Order::Hunt { at: found.1 };
        }
        Some(found)
    }

    /// `profile` of a blow by `attacker`, a sure critical while it suffers a status with
    /// `auto_critical` (Savage). One of `fight.rs::resolve`'s hooks.
    pub(super) fn savage_blow(&self, attacker: ActorId, mut profile: Profile) -> Profile {
        if self.savage_status(attacker).is_some() {
            profile.sure_critical = true;
        }
        profile
    }

    /// `attacker`'s blow hit: its Savage eases a degree, gone at none.
    pub(super) fn savage_spent(&mut self, attacker: ActorId, events: &mut Vec<Event>) {
        let Some(status) = self.savage_status(attacker) else {
            return;
        };
        let Some(actor) = self.state.actors.get_mut(&attacker) else {
            return;
        };
        let left = (actor.effects.get_mut(status)).map_or(0, |e| {
            e.severity = e.severity.saturating_sub(1);
            e.severity
        });
        if left == 0 {
            actor.effects.remove(status);
            events.push(Event::Status {
                actor: attacker,
                status,
                began: false,
            });
        }
    }

    /// The status with `auto_critical` that `id` suffers, if any.
    fn savage_status(&self, id: ActorId) -> Option<crate::content::StatusId> {
        let actor = self.state.actors.get(&id)?;
        (actor.effects.iter())
            .map(|e| e.status)
            .find(|&s| self.content.status(s).auto_critical)
    }
}
