//! The lucky days' gifts that are more than one read at a hook (spec §8 "Birth sign"; the
//! `stars` tree's talents a `[[lucky]]` grants): the Firebird's Reborn whole, read as the Phoenix
//! rises (`stars.rs`), and the Deep One's Drowning grip, the hold a melee hit puts on a creature
//! ([`crate::talent::TalentMods::grip`]). The Iron Scourge's and the Gate Warden's gifts are
//! free blows and sums read where those rules live. Nothing here draws at random.

use super::talk::capitalized;
use super::{ActorId, World};
use crate::action::{AttackKind, Event};
use crate::combat::Blow;
use crate::content::{ActionKind, Stance};
use crate::status::Effect;

impl World {
    /// The player rising from the ashes is reborn whole, if their talents say so: every status
    /// effect and disease burned away (nothing `follows` them) and every hit point back. Returns
    /// whether they were; else the rising leaves them Ashen as it does.
    pub(super) fn reborn_whole(&mut self, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let player = self.state.player;
        if !self.state.player().talent_mods(&content).reborn_whole {
            return false;
        }
        let diseases: Vec<_> = (self.state.player().diseases.iter())
            .map(|i| i.disease)
            .collect();
        for disease in diseases {
            self.end_disease(player, disease, events);
        }
        let me = self.player_mut();
        let gone: Vec<Effect> = me.effects.iter().copied().collect();
        for e in &gone {
            me.effects.remove(e.status);
        }
        events.extend(gone.iter().map(|e| Event::Status {
            actor: player,
            status: e.status,
            began: false,
        }));
        let me = self.player_mut();
        me.hp = me.max_hp(&content);
        true
    }

    /// `attacker`'s attack of `kind` at `target` has been rolled (`blow`, if it was): with
    /// Drowning grip, a melee blow lets go of every creature it held but `target`, a hit that
    /// leaves `target` alive holds it, and a miss lets go of it too.
    pub(super) fn grip(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        blow: Option<&Blow>,
        events: &mut Vec<Event>,
    ) {
        if kind != AttackKind::Melee || attacker == target {
            return;
        }
        let Some(status) = self.grip_of(attacker) else {
            return;
        };
        let hit = blow.is_some_and(|b| b.strike.is_hit());
        let keep = (hit && self.state.actors.contains_key(&target)).then_some(target);
        self.let_go(attacker, keep);
        let Some(target) = keep else {
            return;
        };
        let held = self.state.actors[&target].effects.has(status);
        self.afflict(target, status, 1, Some(attacker), events);
        let at = self.state.actors[&target].pos;
        if !held && attacker == self.state.player && self.perception.sees_actors_at(at) {
            let who = capitalized(&self.called(&self.state.actors[&target]));
            // Said after the blow's own report ("You hit the kobold.").
            self.news.push(format!("{who} is caught in your grip."));
        }
    }

    /// `holder` lets go of every creature it holds but `keep` (it struck another, missed, or
    /// moved): their grip status ends, silently. Nothing for an actor without Drowning grip.
    pub(super) fn let_go(&mut self, holder: ActorId, keep: Option<ActorId>) {
        let Some(status) = self.grip_of(holder) else {
            return;
        };
        for actor in self.state.actors.values_mut() {
            let held = actor
                .effects
                .get(status)
                .is_some_and(|e| e.by == Some(holder));
            if held && Some(actor.id) != keep {
                actor.effects.remove(status);
            }
        }
    }

    /// The status actor `id`'s melee hits hold with, if it has Drowning grip.
    fn grip_of(&self, id: ActorId) -> Option<crate::content::StatusId> {
        (self.state.actors.get(&id))
            .filter(|a| !a.talents.is_empty())
            .and_then(|a| a.talent_mods(&self.content).grip)
    }

    /// What creature `id`, held fast (a status of its `blocks` its steps), does instead of
    /// `plan`'s step: it attacks whoever holds it if that one is hostile and beside it, else
    /// waits. Any other plan stands.
    pub(super) fn held_back(&self, id: ActorId, plan: super::ai::Plan) -> super::ai::Plan {
        use super::ai::Plan;
        if !matches!(plan, Plan::Step(_) | Plan::Patrol) {
            return plan;
        }
        let Some(me) = self.state.actors.get(&id) else {
            return plan;
        };
        let Some(status) = me.effects.blocking(&self.content, ActionKind::Step) else {
            return plan;
        };
        let holder = (me.effects.get(status).and_then(|e| e.by))
            .and_then(|by| self.state.actors.get(&by))
            .filter(|h| h.level == me.level && h.pos.chebyshev(me.pos) == 1)
            .filter(|h| self.stance(id, h.id) == Stance::Hostile);
        match holder {
            Some(h) => Plan::Attack {
                target: h.id,
                kind: AttackKind::Melee,
            },
            None => Plan::Wait,
        }
    }
}
