//! The energy scheduler (spec §8 "Speed"), ADOM-like: who acts when.
//!
//! Every actor has a speed (its creature type's [`crate::content::CreatureDef::speed`];
//! [`NORMAL_SPEED`], 1000, for the player and normal creatures) and energy ([`Actor::energy`]).
//! Actors on the current level gain their speed in energy per second of game time; an action
//! that takes `d` seconds costs [`cost`]`(d)`, 1000 per second. An actor may act while its energy
//! is at least 0, and whatever is left over carries on, so over any span the actions an actor
//! takes stay proportional to its speed while the clock keeps whole seconds. Creatures on other
//! levels are frozen (they gain nothing until the player is back), so coming back to a level
//! never sets off a burst of actions; the dev time skip gives nobody energy either.
//!
//! After the player acts, the other actors on the level act in time order until the player may
//! act again ([`World::run_turn`]): next is whoever reaches 0 soonest, the clock moves to that
//! moment, and ties go to the player first, then to ascending actor ids. With equal speeds every
//! creature acts once per player turn, in id order, right after the player.
//!
//! Speed is hidden from the player: it is in neither the perception snapshot nor any message.

use super::{Actor, ActorId, ActorKind, World};
use crate::action::Event;
use crate::content::NORMAL_SPEED;
use crate::perception::VIEW_RADIUS;
use crate::time::Duration;

/// Energy an action costs per second it takes.
const ENERGY_PER_SECOND: i64 = NORMAL_SPEED as i64;

/// The energy an action that takes `d` costs: 1000 per second. Every action takes at least a
/// second, so the scheduler always moves on.
pub(super) fn cost(d: Duration) -> i64 {
    i64::try_from(d.as_seconds().max(1))
        .unwrap_or(i64::MAX)
        .saturating_mul(ENERGY_PER_SECOND)
}

/// Whole seconds until an actor with `energy` and `speed` may act: 0 if it may act now.
fn wait(energy: i64, speed: u32) -> u64 {
    if energy >= 0 {
        0
    } else {
        energy.unsigned_abs().div_ceil(u64::from(speed.max(1)))
    }
}

/// An actor in the running turn's schedule.
struct Scheduled {
    id: ActorId,
    speed: u32,
    energy: i64,
}

impl World {
    /// Phase 2, the rest of a turn whose player action took `took`: the player pays for it, then
    /// the other actors on the current level act in time order ([`World::creature_turn`]) until
    /// the player may act again. The clock advances to the moment of each action, and ends at the
    /// moment the player is ready.
    ///
    /// While the turn runs, energies live in the turn's schedule; they are written back to the
    /// actors when the player is ready.
    pub(super) fn run_turn(&mut self, took: Duration, events: &mut Vec<Event>) {
        self.state.turn += 1;
        // The player has acted: what they heard before is spent, and they hear afresh.
        self.player_mut().heard.clear();
        let player = self.state.player;
        let level = &self.state.current_level;
        let scheduled = |a: &Actor| Scheduled {
            id: a.id,
            speed: self.content.creature(a.creature).speed,
            energy: a.energy,
        };
        // In the order ties go: the player, then the creatures by ascending id.
        let creatures = self.state.actors.values();
        let creatures = creatures.filter(|a| a.level == *level && a.id != player);
        let mut schedule: Vec<Scheduled> = std::iter::once(self.state.player())
            .chain(creatures)
            .map(scheduled)
            .collect();
        schedule[0].energy = schedule[0].energy.saturating_sub(cost(took));
        loop {
            let (seconds, next) = schedule
                .iter()
                .enumerate()
                .map(|(i, s)| (wait(s.energy, s.speed), i))
                .min()
                .expect("the player is scheduled");
            if seconds > 0 {
                self.state.time += Duration::seconds(seconds);
                let seconds = i64::try_from(seconds).unwrap_or(i64::MAX);
                for s in &mut schedule {
                    let gain = i64::from(s.speed).saturating_mul(seconds);
                    s.energy = s.energy.saturating_add(gain);
                }
            }
            if next == 0 {
                break;
            }
            let took = self.creature_turn(schedule[next].id, events);
            schedule[next].energy = schedule[next].energy.saturating_sub(cost(took));
        }
        for s in schedule {
            if let Some(actor) = self.state.actors.get_mut(&s.id) {
                actor.energy = s.energy;
            }
        }
    }

    /// How long from now no creature on the current level could come into view, so a rest need
    /// not look after turns that end by then (spec §8 "Rest"). A creature is seen only within
    /// [`VIEW_RADIUS`] (and heard only within [`crate::noise::MAX_REACH`], less than that), and
    /// every step takes at least the level's quickest step: before its first step that could
    /// bring it into view, it needs the energy for all the steps that cannot.
    pub(super) fn out_of_view_for(&self) -> Duration {
        const _: () = assert!(crate::noise::MAX_REACH < VIEW_RADIUS);
        let level = self.state.current();
        let quickest = self
            .content
            .terrain_defs()
            .iter()
            .filter(|t| t.walkable)
            .map(|t| level.step_time(t))
            .min()
            .unwrap_or(level.turn);
        let step = cost(quickest);
        let player = self.player_pos();
        self.state
            .actors
            .values()
            .filter(|a| a.level == level.id && a.kind == ActorKind::Creature)
            .map(|a| {
                let steps = i64::from((a.pos.chebyshev(player) - VIEW_RADIUS - 1).max(0));
                let short = steps.saturating_mul(step).saturating_sub(a.energy);
                let speed = self.content.creature(a.creature).speed;
                Duration::seconds(wait(short.saturating_neg(), speed))
            })
            .min()
            .unwrap_or(Duration::seconds(u64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_cost_a_thousand_a_second_and_at_least_a_second() {
        assert_eq!(cost(Duration::seconds(6)), 6000);
        assert_eq!(cost(Duration::minutes(10)), 600_000);
        assert_eq!(cost(Duration::ZERO), 1000);
    }

    #[test]
    fn waits_round_up_to_whole_seconds() {
        assert_eq!(wait(0, 1000), 0);
        assert_eq!(wait(250, 1000), 0);
        assert_eq!(wait(-6000, 1000), 6);
        assert_eq!(wait(-6000, 2000), 3);
        assert_eq!(wait(-6000, 1500), 4);
        assert_eq!(wait(-6001, 1000), 7);
        assert_eq!(wait(-1, 10_000), 1);
    }
}
