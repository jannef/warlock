//! Surprise (spec §8 "Surprise and sneak attacks"; `docs/dev/surprise.md`): who is unaware of
//! whom, the initiative check, and the Surprised status, both ways round.
//!
//! - [`World::unaware`] is the one definition. A creature is unaware of an actor when it hunts
//!   no one it has seen ([`super::Mind::last_seen`] unset) and its eyes, by creature sight
//!   ([`crate::sight`]), do not see the actor's cell now; asleep or disguised, whatever it sees.
//!   The player is unaware of a creature their last perception did not see and they have not
//!   seen within `memory_turns` turns ([`super::Mind::noticed`], set by [`World::mark_noticed`] in
//!   the turn pipeline, and by [`World::startle`] for an attacker that caught them unawares: its
//!   first blow surprises, the next ones do not). It draws nothing.
//! - [`World::startle`] is the one place anyone is surprised: by an attack that finds its target
//!   unaware (`fight.rs::resolve`, `cast.rs::spell_attack`), or by a hidden trap springing under
//!   its victim (`trap.rs::step_onto_trap`). A sleeper or a disguised mimic is Surprised with no
//!   check; anyone else makes the initiative check (two gameplay draws) and suffers Surprised if
//!   it fails.
//! - Surprised (and unaware, for the surprising attack itself) is flat-footed
//!   ([`World::flat_footed`]); it makes no free blow out of its turn
//!   (`free_blow.rs::free_blows_of`, `weapon_ability.rs::approached`); its next action is priced
//!   by [`World::status_cost`].
//!
//! Without `data/surprise.toml` nobody is surprised: no check, no status, no strip, and the
//! player is never unaware (a creature's half still decides the ambush dice, `ambush.rs`).

use super::{ActorId, ActorKind, World};
use crate::action::Event;
use crate::check;
use crate::combat::{Category, Target};
use crate::content::{StatusId, the, the_start};
use crate::fov::radius2;
use crate::sight::{self, LineOfSight};
use crate::skills::{Effect, Use};
use crate::stats::Stat;

/// What counts nothing toward a flat-footed target's Defence: the categories that react.
const FLAT_FOOTED: [Category; 2] = [Category::Dexterity, Category::Dodge];

impl World {
    /// Whether actor `id` has not seen actor `other` (module docs): a creature by its own eyes
    /// and mind, the player by what they perceived and remember.
    pub(crate) fn unaware(&mut self, id: ActorId, other: ActorId) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        if !self.state.actors.contains_key(&other) {
            return false;
        }
        if id == self.state.player {
            return self.player_unaware(other);
        }
        if me.kind != ActorKind::Creature {
            return false;
        }
        // Asleep, it has seen no one (`den.rs`); nor has a disguised mimic (`mimic.rs`).
        if self.asleep(id) || self.disguised(id) {
            return true;
        }
        me.mind.last_seen.is_none() && !self.sees_now(id, other)
    }

    /// Whether the player has not seen creature `other`: their last perception did not see it,
    /// and they have not seen it within `memory_turns` turns. Never without the surprise rules.
    fn player_unaware(&self, other: ActorId) -> bool {
        let Some(rules) = self.content.surprise() else {
            return false;
        };
        let Some(them) = self.state.actors.get(&other) else {
            return false;
        };
        if them.kind != ActorKind::Creature
            || self.perception().actors().iter().any(|a| a.id == other)
        {
            return false;
        }
        (them.mind.noticed)
            .is_none_or(|turn| self.state.turn.saturating_sub(turn) > rules.memory_turns)
    }

    /// Whether creature `id`'s eyes see actor `other`'s cell now, by creature sight
    /// ([`crate::sight`]): within its sight radius, in its line of sight, light enough for its
    /// eyes (the Transfixing stare asks it). Never for the player.
    pub(super) fn sees_now(&mut self, id: ActorId, other: ActorId) -> bool {
        let World {
            state,
            content,
            lights,
            ..
        } = self;
        let (Some(me), Some(them)) = (state.actors.get(&id), state.actors.get(&other)) else {
            return false;
        };
        if me.kind != ActorKind::Creature {
            return false;
        }
        let at = them.pos;
        let def = content.creature(me.creature);
        let radius = sight::sight_radius(state, content, def.sight);
        let d2 = me.pos.dist2(at);
        if d2 > radius2(radius) {
            return false;
        }
        let level = state.current();
        let reach = LineOfSight::radius_for(d2).min(radius);
        let los = LineOfSight::compute(me.pos, reach, |p| level.is_opaque(content, p));
        let eye = me.eyes(content, &content.vision_profiles()[def.vision]);
        sight::sees(state, content, lights, &los, &eye, at)
    }

    /// The player has just perceived the world: every creature they see now is one they have
    /// noticed this turn. Called from `perceive` in the turn pipeline only, never when the
    /// snapshot is recomputed from a saved state.
    pub(super) fn mark_noticed(&mut self) {
        if self.content.surprise().is_none() {
            return;
        }
        let turn = self.state.turn;
        for id in self.creatures_in_view() {
            if let Some(actor) = self.state.actors.get_mut(&id) {
                actor.mind.noticed = Some(turn);
            }
        }
    }

    /// The Surprised status, if surprise is in the game.
    fn surprise_status(&self) -> Option<StatusId> {
        self.content.surprise().map(|r| r.status)
    }

    /// Whether actor `id` is Surprised now.
    pub(super) fn surprised(&self, id: ActorId) -> bool {
        let Some(status) = self.surprise_status() else {
            return false;
        };
        (self.state.actors.get(&id)).is_some_and(|a| a.effects.has(status))
    }

    /// `struck`, actor `target`'s Defence against an attack, flat-footed (nothing of its
    /// Dexterity or Dodge) when it is `unaware` of the attacker or Surprised.
    pub(super) fn flat_footed(&self, target: ActorId, unaware: bool, struck: Target) -> Target {
        let caught = (unaware && self.content.surprise().is_some()) || self.surprised(target);
        if caught {
            struck.without(FLAT_FOOTED)
        } else {
            struck
        }
    }

    /// Actor `target` is caught unawares, by actor `by`'s attack or (`None`) a hidden trap
    /// springing under it: a sleeper or a disguised mimic is Surprised with no check; anyone
    /// else makes the initiative check (two gameplay draws: its Dexterity modifier, its checks'
    /// talent bonus, and Wary's against the rules' difficulty) and is Surprised if it fails.
    /// The player is told what came of it (after the fact, never the roll).
    pub(super) fn startle(
        &mut self,
        target: ActorId,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) {
        let Some(rules) = self.content.surprise() else {
            return;
        };
        let (status, difficulty) = (rules.status, rules.difficulty);
        let Some(me) = self.state.actors.get(&target) else {
            return;
        };
        if self.state.death.is_some() {
            return;
        }
        let content = &self.content;
        let (pos, name) = (me.pos, content.creature(me.creature).name.clone());
        let helpless =
            me.kind == ActorKind::Creature && (self.asleep(target) || self.disguised(target));
        let talents = me.talent_mods(content);
        let modifier = me.effective_stats(content).modifier(Stat::Dexterity)
            + talents.check(Stat::Dexterity)
            + talents.initiative_bonus
            + me.skill_effect(content, Effect::Initiative);
        let difficulty = difficulty + by.map_or(0, |by| self.craft(by));
        let made = !helpless && check::check(&mut self.state.rng, modifier, difficulty);
        if !made {
            self.afflict(target, status, 1, by, events);
        }
        let player = self.state.player;
        // A check made, whatever came of it, marks the player's Initiative (training stream).
        if target == player && !helpless && self.trains(Use::Surprise) {
            self.train(Use::Surprise);
        }
        // Struck once, the player knows the attacker is there, seen or not: its next blows find
        // them aware until the memory lapses (`player_unaware`).
        if target == player
            && let Some(attacker) = by.and_then(|a| self.state.actors.get_mut(&a))
        {
            attacker.mind.noticed = Some(self.state.turn);
        }
        if target == player && made {
            let text = "You keep your wits about you.".to_string();
            // A trap's own line is said with the turn's reports: this one after it.
            match by {
                Some(_) => self.say(text),
                None => self.news.push(text),
            }
        } else if by == Some(player) && !helpless && self.perception.sees_actors_at(pos) {
            // Failed, but immune to the status (a ward, its type): no word either way.
            if made {
                self.say(format!("{} whirls to face you.", the_start(&name)));
            } else if self.surprised(target) {
                self.say(format!("You catch {} unawares.", the(&name)));
            }
        }
    }

    /// Actor `id`'s craft at catching others unawares: what it adds to the difficulty of their
    /// initiative check, its Sneak attack skill's `surprise` and its talents' (the thief's
    /// Sneak attack).
    fn craft(&self, id: ActorId) -> i32 {
        (self.state.actors.get(&id)).map_or(0, |a| {
            a.skill_effect(&self.content, Effect::Surprise)
                + a.talent_mods(&self.content).surprise_difficulty
        })
    }

    /// The `cost` percent status `status` puts on actor `id`'s actions: its data's, but
    /// Surprised's extra eased by the actor's Initiative skill and talents (Cool head), together
    /// at most all of it.
    pub(super) fn status_cost(&self, id: ActorId, status: StatusId) -> u32 {
        let cost = self.content.status(status).cost;
        if self.surprise_status() != Some(status) || cost <= 100 {
            return cost;
        }
        let ease = (self.state.actors.get(&id))
            .map_or(0, |a| {
                a.talent_mods(&self.content).surprise_ease
                    + a.skill_effect(&self.content, Effect::SurpriseEase)
            })
            .clamp(0, 100);
        let ease = u32::try_from(ease).unwrap_or(0);
        100 + (cost - 100) * (100 - ease) / 100
    }
}
