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

use super::{Actor, ActorId, AttackSkills, World};
use crate::action::Event;
use crate::content::Effect as ItemEffect;
use crate::content::{
    ActionKind, BASE_TURN, Benefit, LevelKind, Stance, TerrainDef, percent_of, price,
};
use crate::geom::Pos;
use crate::magic::Mods;
use crate::magic::mods::faster;
use crate::perception::VIEW_RADIUS;
use crate::skills::Effect;
use crate::time::{Duration, GameTime};
use std::collections::BTreeMap;

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
    /// The energy price of action `kind` for actor `id`: what it costs it ([`World::cost_of`])
    /// over the 6-second base, as dear as its combat stance makes an attack, slowed by its burden
    /// ([`World::burdened`]). Steps are priced by the terrain instead ([`World::step_price`]).
    pub(super) fn price_of(&self, id: ActorId, kind: ActionKind) -> i64 {
        let actor = &self.state.actors[&id];
        let base = price(self.cost_of(id, kind), BASE_TURN);
        self.burdened(id, self.stanced(actor, kind, base))
    }

    /// What action `kind` costs actor `id`: its creature type's cost of it (spec §8 "Time"),
    /// except that an attack with a weapon that has its own `attack_cost` costs that (a blow with
    /// the weapon it wields, a shot with its missile weapon) unless the creature type gives its
    /// attacks a cost of their own. Then magic items make attacks and steps faster (spec §8
    /// "Magic items"): the attack speed of the weapon's affixes and of what the actor wears and
    /// wields, and the moving speed of what it wears.
    pub(super) fn cost_of(&self, id: ActorId, kind: ActionKind) -> u32 {
        let content = &self.content;
        let actor = &self.state.actors[&id];
        let cost = content.action_cost(actor.creature, kind);
        let item = match kind {
            ActionKind::Melee => actor.weapon_item(content),
            ActionKind::Shoot => actor.launcher_item(content),
            ActionKind::Step | ActionKind::Throw => None,
            _ => return cost,
        };
        let stack = item.and_then(|i| actor.inventory.get(i));
        let own = stack.and_then(|s| content.object(s.object).attack_cost);
        let cost = match own {
            Some(own) if content.creature(actor.creature).attack_cost.is_none() => own,
            _ => cost,
        };
        let mods = actor.mods(content);
        let speed = match kind {
            ActionKind::Step => mods.get(ItemEffect::MoveSpeed),
            _ => {
                let local =
                    stack.map_or(0, |s| Mods::local(s, content).get(ItemEffect::AttackSpeed));
                mods.get(ItemEffect::AttackSpeed) + local
            }
        };
        if speed == 0 {
            cost
        } else {
            faster(cost, speed)
        }
    }

    /// `base` as dear as `actor`'s combat stance makes its attacks (spec §8 "Stances"): melee,
    /// shots, and throws; reloading is not an attack.
    fn stanced(&self, actor: &Actor, kind: ActionKind, base: i64) -> i64 {
        match kind {
            ActionKind::Melee | ActionKind::Shoot | ActionKind::Throw => {
                percent_of(base, self.content.stance_def(actor.stance).attack_cost)
            }
            _ => base,
        }
    }

    /// Whether actor `id` stepping to `to` flees: some hostile creature stands next to it now and
    /// the step takes it out of that creature's reach (spec §8 "Stances": a coward's flight is
    /// cheaper and quieter).
    pub(super) fn flees(&self, id: ActorId, to: Pos) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        let stance = self.content.stance_def(me.stance);
        if stance.flee_cost == 100 && stance.flee_noise == 100 {
            return false;
        }
        self.state.actors.values().any(|other| {
            other.id != id
                && other.level == me.level
                && other.pos.chebyshev(me.pos) == 1
                && other.pos.chebyshev(to) > 1
                && self.stance(id, other.id) == Stance::Hostile
        })
    }

    /// The energy price of attack action `kind` (`melee`, `shoot`, `throw`, `reload`) for actor
    /// `id` drawing on weapon skill and fighting style `skills` (spec §8 "Weapon skills",
    /// "Fighting styles"): its cost with its weapon's and its magic items' (`World::cost_of`; a
    /// shield bash costs what a blow does) less what the ranks of both take off (`speed`;
    /// `reload` for reloading), then priced like any action ([`World::price_of`]), its combat
    /// stance included.
    pub(super) fn attack_price(&self, id: ActorId, kind: ActionKind, skills: AttackSkills) -> i64 {
        let actor = &self.state.actors[&id];
        let benefit = actor.benefit_of(&self.content, skills);
        let saving = match kind {
            ActionKind::Reload => benefit.reload,
            _ => benefit.speed,
        };
        let cost = self.cost_of(id, kind);
        let base = price(Benefit::saving(cost, saving), BASE_TURN);
        self.burdened(id, self.stanced(actor, kind, base))
    }

    /// The energy price of actor `id` stepping onto terrain `to` on the current level, slowed by
    /// its burden ([`World::burdened`]); `fleeing` ([`World::flees`]) makes it as dear as its
    /// stance says. On the overworld its skills (survival; spec §8 "Skills") take their percent
    /// off the journey, rounded up (at least 1).
    pub(super) fn step_price(&self, id: ActorId, to: &TerrainDef, fleeing: bool) -> i64 {
        let actor = &self.state.actors[&id];
        let step = self.cost_of(id, ActionKind::Step);
        let level = self.state.current();
        let mut price = level.step_price(to, step);
        if level.kind == LevelKind::Overworld {
            price = shortened(price, actor.skill_effect(&self.content, Effect::Travel));
        }
        if fleeing {
            price = percent_of(price, self.content.stance_def(actor.stance).flee_cost);
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
        let ease = self.state.actors[&id].skill_effect(&self.content, Effect::Burden);
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
    /// actors when the player is ready. As the clock moves, every actor on the level lives
    /// through the time ([`World::pass_time`]: status effects tick and ease, hit points come
    /// back, the player's nutrition burns). A player who faints from hunger as the turn begins
    /// pays for the turns lost too ([`World::faint`]). A creature that dies leaves the schedule;
    /// the player's death ends the turn there and then.
    pub(super) fn run_turn(&mut self, price: i64, events: &mut Vec<Event>) {
        self.state.turn += 1;
        // The player has acted: what they heard before is spent, and they hear afresh (lines
        // muttered too: said only if heard this turn).
        self.player_mut().heard.clear();
        self.mutters.clear();
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
                let (start, fallen) = (self.state.time, self.fallen.len());
                self.state.time += Duration::seconds(seconds);
                // Whoever steps onto the ice next meets the ice of the moment.
                self.update_ice();
                self.pass_time(schedule.iter().map(|s| s.id), start, seconds, events);
                let seconds = i64::try_from(seconds).unwrap_or(i64::MAX);
                for s in &mut schedule {
                    let gain = i64::from(s.speed).saturating_mul(seconds);
                    s.energy = s.energy.saturating_add(gain);
                }
                // Some may have died of their wounds meanwhile: whoever is ready now acts
                // next (the same actor, unless it died).
                if self.fallen.len() > fallen {
                    schedule.retain(|s| self.state.actors.contains_key(&s.id));
                }
                continue;
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

    /// How long from now no creature on the current level that disturbs a rest
    /// ([`World::disturbs_rest`]) could come into view, so a rest need not look after turns that
    /// end by then and in which the player heard nothing (spec §8 "Rest"). A creature is seen
    /// only within [`VIEW_RADIUS`] (and heard only within [`crate::noise::MAX_REACH`], less than
    /// that), and every step costs at least the level's cheapest step: before its first step that
    /// could bring it into view, it needs the energy for all the steps that cannot. (Creature
    /// types give only their attacks costs of their own, so a step costs every creature the same
    /// but for what magic items it wears take off; a burden only makes it dearer, and an
    /// overloaded creature cannot step at all.) The others (the showcase's people) never stop a
    /// rest by coming into view, and what they do that the player could notice is heard (their
    /// steps, their muttering) or done nowhere near.
    pub(super) fn out_of_view_for(&self) -> Duration {
        const _: () = assert!(crate::noise::MAX_REACH < VIEW_RADIUS);
        let level = self.state.current();
        // The cheapest step on the level for each step cost, as creatures need them.
        let mut cheapest_for: BTreeMap<u32, i64> = BTreeMap::new();
        let mut cheapest = |step: u32| {
            *cheapest_for.entry(step).or_insert_with(|| {
                (self.content.terrain_defs().iter())
                    .filter(|t| t.walkable)
                    .map(|t| level.step_price(t, step))
                    .min()
                    .unwrap_or(1)
            })
        };
        let player = self.player_pos();
        self.state
            .actors
            .values()
            .filter(|a| a.level == level.id && self.disturbs_rest(a))
            .map(|a| {
                let steps = i64::from((a.pos.chebyshev(player) - VIEW_RADIUS - 1).max(0));
                let cheapest = cheapest(self.cost_of(a.id, ActionKind::Step));
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
