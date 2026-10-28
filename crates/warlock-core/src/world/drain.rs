//! Drains (spec §8 "Stats": "Drain"; #382): the touch of some undead takes points off a stat
//! for good.
//!
//! A natural attack's `on_hit` may name a drain ([`crate::content::Drain`]); a hit that lands,
//! leaves its target alive, and wins its chance comes here ([`World::drain`], from
//! `after_blow`):
//! - **The check**: the target rolls a check of the drain's `check` stat (its effective
//!   modifier, what its talents add to every check of that stat (Iron will), and its talents'
//!   `drain_checks` (Unbroken soul, Divine grace)), and for the player against a creature's
//!   drain what Knowledge gives (`knowledge.rs`), against the drain's `difficulty`. Made, nothing
//!   is lost, and it trains the stat as a disease resisted does.
//! - **The loss**: failed, the drain's `points` come off the stat's **score**
//!   ([`crate::stats::Stats::drain`]), one of the six drawn for a `random` drain. The potential
//!   stays, so lost points may be won back the slow way (training, corpses), and
//!   [`Actor::drained`](super::Actor::drained) counts them for a temple's mending
//!   (`World::mend`), which gives them all back.
//! - **Never a kill-lock**: no score falls below [`DRAIN_FLOOR`], nor so low that it could not
//!   carry the actor's stat damage. Hit points and mana are cut down to the new maxima (a
//!   Toughness drain lowers the most hit points), never below 1: a drain never kills.
//!
//! The player hears of their own drains in words ([`drained_message`]), never the points.
//! Draws of the gameplay stream: the check's two, and one before it for a `random` stat.

use super::{ActorId, World};
use crate::action::Event;
use crate::check::check;
use crate::content::Drain;
use crate::stats::{DRAIN_FLOOR, Stat};

impl World {
    /// Actor `target` was hit by a drain of `by`'s (a creature's touch; `None`: no creature's,
    /// a scroll's backlash) (module docs).
    pub(super) fn drain(
        &mut self,
        target: ActorId,
        drain: Drain,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get(&target) else {
            return;
        };
        let stat = match drain.stat {
            Some(stat) => stat,
            None => {
                let i = self.state.rng.below(Stat::COUNT as u32) as usize;
                Stat::ALL[i]
            }
        };
        let talents = actor.talent_mods(&content);
        let modifier = (actor.effective_stats(&content).modifier(drain.check))
            .saturating_add(talents.check(drain.check))
            .saturating_add(talents.drain_checks);
        // Know your enemy (Knowledge, `knowledge.rs`).
        let modifier = modifier.saturating_add(self.resisting(target, by));
        if check(&mut self.state.rng, modifier, drain.difficulty) {
            self.train_resisting(target, drain.check);
            events.push(Event::Drained {
                actor: target,
                stat: None,
            });
            return;
        }
        let Some(actor) = self.state.actors.get_mut(&target) else {
            return;
        };
        // A score always carries the stat damage on it (saves refuse one that does not).
        let floor = DRAIN_FLOOR.max(actor.damage.get(stat).saturating_add(1));
        let lost = actor.stats.drain(stat, drain.points, floor);
        actor.drained[stat] = actor.drained[stat].saturating_add(lost);
        // The most hit points and mana may have fallen with it: what is left is cut down to
        // them (at least 1 each way: a drain never kills).
        actor.hp = actor.hp.min(actor.max_hp(&content)).max(1);
        actor.mana = actor.mana.min(actor.max_mana(&content));
        events.push(Event::Drained {
            actor: target,
            stat: (lost > 0).then_some(stat),
        });
    }

    /// Every drained point of actor `id`'s comes back (a temple's mending, with its stat
    /// damage): the scores rise by what drains took, and nothing is counted any more.
    pub(super) fn restore_drained(&mut self, id: ActorId) {
        if let Some(actor) = self.state.actors.get_mut(&id) {
            for stat in Stat::ALL {
                let points = std::mem::take(&mut actor.drained[stat]);
                actor.stats.restore(stat, points);
            }
        }
    }
}

/// What the player is told of a drain of theirs: the stat it took from, or a chill that passes.
pub(super) fn drained_message(stat: Option<Stat>) -> String {
    match stat {
        Some(stat) => format!("You feel your {} drain away!", stat.name().to_lowercase()),
        None => "A deathly chill passes through you.".to_string(),
    }
}
