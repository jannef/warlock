//! The energy scheduler (spec §8 "Speed"), ADOM-like: who acts when.
//!
//! Every actor has a speed (its creature type's [`crate::content::CreatureDef::speed`];
//! [`crate::content::NORMAL_SPEED`], 1000, for the player and normal creatures) and energy
//! ([`Actor::energy`]). Actors on the current level gain their speed in energy per second of game
//! time. An action costs its energy price ([`crate::content::price`]): its cost (1000 for a
//! normal action, `data/actions.toml`) times its base time in seconds (6 s, or a step's travel
//! time on the overworld), so it takes an actor `cost / 1000 × 1000 / speed × base`. An actor may
//! act while its energy is at least 0, and whatever is left over carries on, so over any span the
//! actions an actor takes stay proportional to its speed and their costs while the clock keeps
//! whole seconds: a run of actions takes their exact total time, up to the next whole second,
//! with no drift. An actor's burden (spec §8 "Burden", [`crate::burden`]) raises every price it
//! pays, ×1.25 Burdened and ×1.5 Strained or Overloaded, as a lower speed would: the prices
//! here ([`World::price_of`], [`World::step_price`]) are the one place that happens, so it
//! covers every action, overworld journeys included; being Bloated (spec §8 "Hunger") raises
//! them the same way, after the burden. Creatures on other levels are frozen (they
//! gain nothing until the player is back), so coming back to a level never sets off a burst of
//! actions; the dev time skip gives nobody energy either.
//!
//! After the player acts, the other actors on the level act in time order until the player may
//! act again ([`World::run_turn`]): next is whoever reaches 0 soonest, the clock moves to that
//! moment, and ties go to the player first, then to ascending actor ids. With equal speeds and
//! costs every creature acts once per player turn, in id order, right after the player.
//!
//! Speed is hidden from the player: it is in neither the perception snapshot nor any message.

use super::{Actor, ActorId, ActorKind, World};
use crate::action::Event;
use crate::content::{ActionKind, BASE_TURN, LevelKind, TerrainDef, price};
use crate::perception::VIEW_RADIUS;
use crate::skills::Effect;
use crate::time::{Duration, GameTime};

/// Whole seconds until an actor with `energy` and `speed` may act: 0 if it may act now.
fn wait(energy: i64, speed: u32) -> u64 {
    if energy >= 0 {
        0
    } else {
        energy.unsigned_abs().div_ceil(u64::from(speed.max(1)))
    }
}

/// `price` with `percent` (0 to 100) taken off, rounded up; at least 1.
fn shortened(price: i64, percent: i32) -> i64 {
    let keep = i64::from(100 - percent.clamp(0, 100));
    (price.saturating_mul(keep).saturating_add(99) / 100).max(1)
}

/// An actor in the running turn's schedule.
struct Scheduled {
    id: ActorId,
    speed: u32,
    energy: i64,
}

impl World {
    /// The energy price of action `kind` for actor `id`: its creature type's cost of it over the
    /// 6-second base, slowed by its burden ([`World::burdened`]). Steps are priced by the terrain
    /// instead ([`World::step_price`]).
    pub(super) fn price_of(&self, id: ActorId, kind: ActionKind) -> i64 {
        let creature = self.state.actors[&id].creature;
        self.burdened(
            id,
            price(self.content.action_cost(creature, kind), BASE_TURN),
        )
    }

    /// The energy price of actor `id` stepping onto terrain `to` on the current level, slowed by
    /// its burden ([`World::burdened`]). On the overworld its skills (survival; spec §8
    /// "Skills") take their percent off the journey, rounded up (at least 1).
    pub(super) fn step_price(&self, id: ActorId, to: &TerrainDef) -> i64 {
        let actor = &self.state.actors[&id];
        let step = self.content.action_cost(actor.creature, ActionKind::Step);
        let level = self.state.current();
        let mut price = level.step_price(to, step);
        if level.kind == LevelKind::Overworld {
            price = shortened(price, actor.skills.effect(&self.content, Effect::Travel));
        }
        self.burdened(id, price)
    }

    /// `price` as actor `id` pays it now: ×1.25 Burdened, ×1.5 Strained or Overloaded (spec §8
    /// "Burden"; [`crate::burden::Burden::slow`]), nothing more for a race that ignores Burdened
    /// while it is Burdened, and less of the slowdown for its skills (balance; spec §8
    /// "Skills"); and then, while it is Bloated, by `[bloated] slow` (spec §8 "Hunger",
    /// [`World::bloated`]). An action is priced at the burden and hunger its actor has as it
    /// begins.
    fn burdened(&self, id: ActorId, price: i64) -> i64 {
        let ease = (self.state.actors[&id].skills).effect(&self.content, Effect::Burden);
        self.bloated(id, self.load_of(id).penalty().slow_eased(price, ease))
    }

    /// The price that leaves the player ready to act exactly at `until` (the last turn of a
    /// rest): all the energy they have and will gain until then.
    pub(super) fn price_until(&self, until: GameTime) -> i64 {
        let player = self.state.player();
        let speed = i64::from(self.content.creature(player.creature).speed);
        let seconds = i64::try_from(until.since(self.state.time).as_seconds()).unwrap_or(i64::MAX);
        player.energy.saturating_add(speed.saturating_mul(seconds))
    }

    /// Phase 2, the rest of a turn whose player action costs `price` energy: the player pays for
    /// it, then the other actors on the current level act in time order
    /// ([`World::creature_turn`]) until the player may act again. The clock advances to the
    /// moment of each action, and ends at the moment the player is ready.
    ///
    /// While the turn runs, energies live in the turn's schedule; they are written back to the
    /// actors when the player is ready. As the clock moves, every actor on the level regains
    /// hit points ([`World::regenerate`]) and the player's nutrition burns ([`World::digest`]).
    /// A player who faints from hunger as the turn begins pays for the turns lost too
    /// ([`World::faint`]). A creature that dies leaves the schedule; the player's death ends the
    /// turn there and then.
    pub(super) fn run_turn(&mut self, price: i64, events: &mut Vec<Event>) {
        self.state.turn += 1;
        // The player has acted: what they heard before is spent, and they hear afresh.
        self.player_mut().heard.clear();
        let price = price.saturating_add(self.faint());
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
        schedule[0].energy = schedule[0].energy.saturating_sub(price.max(1));
        // The player's death ends the turn, and the run, at once.
        while self.state.death.is_none() {
            let (seconds, next) = schedule
                .iter()
                .enumerate()
                .map(|(i, s)| (wait(s.energy, s.speed), i))
                .min()
                .expect("the player is scheduled");
            if seconds > 0 {
                self.state.time += Duration::seconds(seconds);
                self.regenerate(schedule.iter().map(|s| s.id), seconds);
                self.digest(seconds);
                let seconds = i64::try_from(seconds).unwrap_or(i64::MAX);
                for s in &mut schedule {
                    let gain = i64::from(s.speed).saturating_mul(seconds);
                    s.energy = s.energy.saturating_add(gain);
                }
            }
            if next == 0 || self.state.death.is_some() {
                break;
            }
            let fallen = self.fallen.len();
            let price = self.creature_turn(schedule[next].id, events);
            schedule[next].energy = schedule[next].energy.saturating_sub(price.max(1));
            // Those who died are gone from the schedule (the player stays, first).
            if self.fallen.len() > fallen {
                schedule.retain(|s| self.state.actors.contains_key(&s.id));
            }
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
    /// every step costs at least the level's cheapest step: before its first step that could
    /// bring it into view, it needs the energy for all the steps that cannot. (Creature types
    /// give only their attacks costs of their own, so a step costs every creature the same; a
    /// burden only makes it dearer, and an overloaded creature cannot step at all.)
    pub(super) fn out_of_view_for(&self) -> Duration {
        const _: () = assert!(crate::noise::MAX_REACH < VIEW_RADIUS);
        let level = self.state.current();
        let step = self.content.action_costs().cost(ActionKind::Step);
        let cheapest = self
            .content
            .terrain_defs()
            .iter()
            .filter(|t| t.walkable)
            .map(|t| level.step_price(t, step))
            .min()
            .unwrap_or(1);
        let player = self.player_pos();
        self.state
            .actors
            .values()
            .filter(|a| a.level == level.id && a.kind == ActorKind::Creature)
            .map(|a| {
                let steps = i64::from((a.pos.chebyshev(player) - VIEW_RADIUS - 1).max(0));
                let short = steps.saturating_mul(cheapest).saturating_sub(a.energy);
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
    fn waits_round_up_to_whole_seconds() {
        assert_eq!(wait(0, 1000), 0);
        assert_eq!(wait(250, 1000), 0);
        assert_eq!(wait(-6000, 1000), 6);
        assert_eq!(wait(-6000, 2000), 3);
        assert_eq!(wait(-6000, 1500), 4);
        assert_eq!(wait(-6001, 1000), 7);
        assert_eq!(wait(-1, 10_000), 1);
    }

    #[test]
    fn journeys_are_shortened_by_whole_percents_rounded_up() {
        assert_eq!(shortened(1_200_000, 0), 1_200_000);
        assert_eq!(shortened(1_200_000, 20), 960_000);
        assert_eq!(shortened(1_200_000, 150), 1, "never free");
        assert_eq!(shortened(1_200_000, -5), 1_200_000);
        assert_eq!(shortened(101, 10), 91);
    }
}
