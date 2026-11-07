//! Gazes (spec §8 "Creature behavior": "Gazes", #1125, design `docs/design/monster-dread.md`
//! §5.1): a creature with `[creature.gaze]` in its type (`content/gaze.rs`) gives its riders at
//! sight, not at touch. Enemy-only: no player ability.
//!
//! **When** ([`World::gazes_at`], asked in `ai.rs::fight` after a summoning and before a
//! throw): the hostile it sees (the look's target, so in its line of sight: never through a
//! wall) stands within the gaze's `range`, has eyes to meet (vitals, `Content::has_vitals`), and
//! its cooldown status is off. Deciding draws nothing.
//!
//! **What** ([`World::creature_gazes`]): it takes the cooldown status and spends a `cast`. Then
//! the eyes must meet: the player's when the gazer stands where they see actors (the snapshot's
//! `sight`, not disguised: `Perception::sees_actors_at`), a creature's by its own eyes
//! (`World::sees_now`). Unmet, the stare is wasted; the player is told they feel it. Met, one
//! draw at the gaze's `chance` (below 100 only), a glance's share of it against the player's
//! averted eyes (`avert.rs`, a gaze marked `averted`); caught, each rider falls by its own
//! chance as a blow's does (`World::rider`: a status's degrees, a drain's check). A gaze makes
//! no noise.
//!
//! **Words**: "The vampire's eyes find yours." then what the riders begin ("You are
//! transfixed."); "You tear your eyes from the vampire's." when they do not catch, "You keep your
//! eyes from the basilisk's." when averted eyes escape it; "You feel a cold stare on you." when
//! the player does not see the gazer. At a creature the player sees the gazer stare at: "The
//! vampire stares at the wolf."

use super::fight::End;
use super::{ActorId, World};
use crate::action::Event;
use crate::content::{ActionKind, Gaze, the, the_start};
use crate::geom::Pos;

/// Said when a gazer the player does not see looks at them.
pub(super) const UNSEEN: &str = "You feel a cold stare on you.";

impl World {
    /// Whether creature `id` gazes now at `target`, a hostile it sees at `at` (module docs).
    pub(super) fn gazes_at(&self, id: ActorId, target: ActorId, at: Pos) -> bool {
        let me = &self.state.actors[&id];
        let Some(gaze) = self.content.creature(me.creature).gaze.as_ref() else {
            return false;
        };
        me.pos.chebyshev(at) <= gaze.range && !me.effects.has(gaze.status) && self.has_eyes(target)
    }

    /// Whether `target` has eyes to meet a gaze: vitals, by its type's `vitals` and the
    /// surprise rules' reading of them (`Content::has_vitals`).
    fn has_eyes(&self, target: ActorId) -> bool {
        (self.state.actors.get(&target))
            .is_some_and(|t| self.content.creature(t.creature).vitals && self.has_vitals(target))
    }

    /// Creature `id` gazes at `target` (its AI saw to it that it can). Returns the price: a
    /// `cast`.
    pub(super) fn creature_gazes(
        &mut self,
        id: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let price = self.price_of(id, ActionKind::Cast);
        let me = &self.state.actors[&id];
        let def = content.creature(me.creature);
        let Some(gaze) = def.gaze.as_ref() else {
            return price;
        };
        let here = me.pos;
        self.afflict(id, gaze.status, gaze.degrees, None, events);
        let player = target == self.state.player;
        let met = if player {
            self.perception.sees_actors_at(here)
        } else {
            self.sees_now(target, id)
        };
        let gazer = the_start(&def.name);
        if !met {
            if player {
                self.say(UNSEEN);
            }
            return price;
        }
        let averted = player && self.averted(target);
        let caught = self.caught(gaze, averted);
        if player {
            let line = match (caught, averted && gaze.averted) {
                (true, _) => format!("{gazer}'s eyes find yours."),
                (false, true) => format!("You keep your eyes from {}'s.", the(&def.name)),
                (false, false) => format!("You tear your eyes from {}'s.", the(&def.name)),
            };
            self.say(line);
        } else if self.perception.sees_actors_at(here) {
            // As the fight's lines name them: "Your vampire", "the hobgoblin", "Snel".
            let seen = (self.state.actors.get(&target))
                .is_some_and(|t| self.perception.sees_actors_at(t.pos));
            let line = format!(
                "{} stares at {}.",
                self.end_of(id, true).subject(),
                self.end_of(target, seen).object()
            );
            self.say(line);
        }
        if caught {
            for rider in gaze.riders.clone() {
                if !self.state.actors.contains_key(&target) {
                    break;
                }
                self.rider(id, target, rider, events);
            }
        }
        price
    }

    /// Actor `id` as the fight's lines name it, `seen` or not (`fight.rs::End`).
    pub(super) fn end_of(&self, id: ActorId, seen: bool) -> End<'_> {
        let player = self.state.player;
        match self.state.actors.get(&id) {
            _ if id == player => End::You,
            Some(a) if seen && a.master() == Some(player) => match a.kept() {
                Some(data) => End::Named(data.name.as_str()),
                None => End::Yours(self.content.creature(a.creature).name.as_str()),
            },
            Some(a) if seen => End::Seen(self.content.creature(a.creature).name.as_str()),
            _ => End::Unseen,
        }
    }

    /// Whether `gaze` catches the eyes it met: its chance, a glance's share of it against
    /// `averted` eyes, one draw below 100.
    fn caught(&mut self, gaze: &Gaze, averted: bool) -> bool {
        let chance = gaze.chance_against(averted);
        chance >= 100 || self.state.rng.chance(u32::from(chance))
    }
}
