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
//! here ([`World::price_of`], [`World::step_price`], [`World::attack_price`],
//! [`World::reload_price`]) are the one place that happens, so it covers every action, overworld
//! journeys included; being Bloated (spec §8 "Hunger") raises
//! them the same way, after the burden, and so do slowing status effects (spec §8 "Status
//! effects": `cost`), after both. Creatures on other levels are frozen (they
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
use crate::content::{ActionKind, BASE_TURN, Benefit, LevelKind, TerrainDef, percent_of, price};
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
    /// over the 6-second base, slowed by its burden ([`World::burdened`]). Steps are priced by
    /// the terrain instead ([`World::step_price`]), attacks by [`World::attack_price`].
    pub(super) fn price_of(&self, id: ActorId, kind: ActionKind) -> i64 {
        let base = price(self.cost_of(id, kind), BASE_TURN);
        self.burdened(id, base)
    }

    /// What action `kind` costs actor `id`: its creature type's cost of it (spec §8 "Time"); a
    /// step is faster by the moving speed of what it wears (spec §8 "Magic items"). Attacks
    /// cost what [`World::attack_cost`] says.
    pub(super) fn cost_of(&self, id: ActorId, kind: ActionKind) -> u32 {
        let content = &self.content;
        let actor = &self.state.actors[&id];
        let cost = content.action_cost(actor.creature, kind);
        let speed = match kind {
            ActionKind::Step => actor.mods(content).get(ItemEffect::MoveSpeed),
            _ => 0,
        };
        if speed == 0 {
            cost
        } else {
            faster(cost, speed)
        }
    }

    /// What attack `kind` costs actor `id` made `with` the object at that position in its
    /// inventory (the weapon it wields for a blow, the shield it bashes with, the missile weapon
    /// it shoots), or with nothing (bare hands, a kick, a throw): its creature type's cost of
    /// the attack (spec §8 "Time"), or the object's own `attack_cost` unless the creature type
    /// gives its attacks a cost of their own, and then faster by the attack speed of what it
    /// wears and wields and of the object's own affixes (spec §8 "Magic items"). So a shield
    /// bash costs what a blow with the shield does, never what one with the wielded weapon
    /// would.
    pub(super) fn attack_cost(&self, id: ActorId, kind: ActionKind, with: Option<usize>) -> u32 {
        let content = &self.content;
        let actor = &self.state.actors[&id];
        let cost = content.action_cost(actor.creature, kind);
        let stack = with.and_then(|i| actor.inventory.get(i));
        let own = stack.and_then(|s| content.object(s.object).attack_cost);
        let cost = match own {
            Some(own) if content.creature(actor.creature).attack_cost.is_none() => own,
            _ => cost,
        };
        let local = stack.map_or(0, |s| Mods::local(s, content).get(ItemEffect::AttackSpeed));
        let speed = actor.mods(content).get(ItemEffect::AttackSpeed) + local;
        if speed == 0 {
            cost
        } else {
            faster(cost, speed)
        }
    }

    /// Whether actor `id` stepping to `to` flees: some hostile creature stands next to it now and
    /// the step takes it out of that creature's reach ([`World::leaves`]; spec §8 "Stances": a
    /// coward's flight is cheaper and quieter).
    pub(super) fn flees(&self, id: ActorId, to: Pos) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        let stance = self.content.stance_def(me.stance);
        if stance.flee_cost == 100 && stance.flee_noise == 100 {
            return false;
        }
        (self.state.actors.values()).any(|other| self.leaves(me, other, to))
    }

    /// The energy price of attack `kind` (a blow, a bash, a kick, a shot, a throw, a spell) for
    /// actor `id`, made `with` the object at that position in its inventory, if any, drawing on
    /// weapon skill and fighting style `skills` (spec §8 "Weapon skills", "Fighting styles"):
    /// its cost ([`World::attack_cost`]) less what the ranks of both take off (`speed`), as dear
    /// as the actor's combat stance makes every attack (spec §8 "Stances"), slowed by its burden
    /// ([`World::burdened`]). Every attack is priced as it begins, before it is made: a rank its
    /// marks raise, a load it lightens (a throw), or a kill changes the price of the next one,
    /// never its own.
    pub(super) fn attack_price(
        &self,
        id: ActorId,
        kind: ActionKind,
        with: Option<usize>,
        skills: AttackSkills,
    ) -> i64 {
        let actor = &self.state.actors[&id];
        let saving = actor.benefit_of(&self.content, skills).speed;
        let base = price(
            Benefit::saving(self.attack_cost(id, kind, with), saving),
            BASE_TURN,
        );
        let stance = self.content.stance_def(actor.stance).attack_cost;
        let base = percent_of(base, stance);
        // Iron grip (spec §8 "Talents"): blows with a two-handed weapon cost less.
        let two_handed = (with.and_then(|i| actor.inventory.get(i)))
            .is_some_and(|s| self.content.object(s.object).two_handed);
        let base = match actor.talent_mods(&self.content).two_handed_price {
            0 => base,
            _ if kind != ActionKind::Melee || !two_handed => base,
            percent => percent_of(base, u32::try_from((100 + percent).max(0)).unwrap_or(0)),
        };
        let base = self.shields_slowed(id, base);
        // A status that makes attacks dearer (Walled) on top of what makes every action so.
        let base = actor.effects.slow_attack(&self.content, base);
        self.burdened(id, base)
    }

    /// The energy price of reloading a missile weapon after a shot for actor `id` drawing on
    /// `skills`: the `reload` action less what their ranks take off (`reload`), slowed by its
    /// burden. Reloading is not an attack: neither attack speed nor the stance changes it.
    pub(super) fn reload_price(&self, id: ActorId, skills: AttackSkills) -> i64 {
        let actor = &self.state.actors[&id];
        let saving = actor.benefit_of(&self.content, skills).reload;
        let cost = self.cost_of(id, ActionKind::Reload);
        self.burdened(id, price(Benefit::saving(cost, saving), BASE_TURN))
    }

    /// The energy price of actor `id` stepping onto terrain `to` on the current level, slowed by
    /// its burden ([`World::burdened`]); `fleeing` ([`World::flees`]) makes it as dear as its
    /// stance says. Terrain its talents price as another's (`terrain_as`: Pathfinder's forest,
    /// Swimmer's water) costs that terrain's step. On the overworld its skills (survival; spec §8
    /// "Skills") take their percent off the journey, rounded up (at least 1).
    pub(super) fn step_price(&self, id: ActorId, to: &TerrainDef, fleeing: bool) -> i64 {
        let actor = &self.state.actors[&id];
        let talents = actor.talent_mods(&self.content);
        let step = self.cost_of(id, ActionKind::Step);
        let level = self.state.current();
        // Its talents may price the terrain as another (Pathfinder, Swimmer, Marsh walker).
        let to = match self.content.terrain_id(&to.id) {
            Some(t) => (talents.terrain_as.iter())
                .find(|(from, _)| *from == t)
                .map_or(to, |&(_, like)| self.content.terrain(like)),
            None => to,
        };
        let mut price = level.step_price(to, step);
        // A strong swimmer (spec §8 "Birth sign") wades as if on dry land, and so does one
        // Undertow (a Water-born's pick) bears.
        if level.kind != LevelKind::Overworld
            && to.move_cost > 100
            && !actor.talents.is_empty()
            && (talents.swimmer || talents.wades_free)
            && super::stars::wading(&self.content, to)
        {
            price = price.min(crate::content::price(step, BASE_TURN).max(1));
        }
        if level.kind == LevelKind::Overworld {
            price = shortened(price, actor.skill_effect(&self.content, Effect::Travel));
            // Then its talents (Long strider): a signed percent of the journey's time.
            let percent = talents.travel_percent;
            if percent != 0 {
                price = percent_of(price, u32::try_from((100 + percent).max(1)).unwrap_or(1));
            }
        }
        // Fast movement and Fleet (spec §8 "Class abilities"): lighter steps, unarmoured.
        let talents = actor.talent_mods(&self.content);
        if !talents.step_percent.is_empty() {
            let armoured = actor.wears_body_armour(&self.content);
            let burdened = self.load_of(id).penalty() != crate::burden::Burden::Unburdened;
            let shield = actor.shield(&self.content).is_some();
            let percent = talents.step_percent(armoured, burdened, shield);
            if percent != 0 {
                price = percent_of(price, u32::try_from((100 + percent).max(1)).unwrap_or(1));
            }
        }
        if fleeing {
            price = percent_of(price, self.content.stance_def(actor.stance).flee_cost);
            // Then its talents (Unflinching): a signed percent of the fleeing step.
            let percent = talents.flee_percent;
            if percent != 0 {
                price = percent_of(price, u32::try_from((100 + percent).max(1)).unwrap_or(1));
            }
        }
        // Sure-footed (spec §8 "Talents"): less of the burden's and Bloated's slowdown on steps.
        let slowdown = actor.talent_mods(&self.content).step_slowdown_percent;
        self.slowed(id, price, slowdown)
    }

    /// `price` as actor `id` pays it now: ×1.25 Burdened, ×1.5 Strained or Overloaded (spec §8
    /// "Burden"; [`crate::burden::Burden::slow`]), nothing more for a race that ignores Burdened
    /// while it is Burdened, and less of the slowdown for its skills (balance; spec §8
    /// "Skills"); and then, while it is Bloated, by `[bloated] slow` (spec §8 "Hunger",
    /// [`World::bloated`]); and then by the `cost` of each of its status effects (spec §8
    /// "Status effects": Slowed). An action is priced at the burden, hunger, and statuses its
    /// actor has as it begins.
    fn burdened(&self, id: ActorId, price: i64) -> i64 {
        self.slowed(id, price, 0)
    }

    /// [`World::burdened`], the slowdown its burden and being Bloated put on `price` that
    /// `slowdown` percent more (negative: less; a step's, Sure-footed).
    fn slowed(&self, id: ActorId, price: i64, slowdown: i32) -> i64 {
        let actor = &self.state.actors[&id];
        let talents = actor.talent_mods(&self.content);
        // Hard work (spec §8 "Class abilities") eases the burden's slowdown on top of Balance.
        let ease = actor.skill_effect(&self.content, Effect::Burden) + talents.burden_ease;
        let keep = (100 - ease.clamp(0, 100)) * (100 + slowdown).max(0) / 100;
        let ease = 100 - keep.clamp(0, 100);
        let price = self.bloated(
            id,
            self.load_of(id).penalty().slow_eased(price, ease),
            slowdown,
        );
        // A talent's `speed_percent` (Quick, Fleet-footed) is not read here: `Actor::speed` counts
        // it, once.
        actor.effects.slow_with(price, |s| self.status_cost(id, s))
    }

    /// The price that leaves the player ready to act exactly at `until` (the last turn of a
    /// rest): all the energy they have and will gain until then.
    pub(super) fn price_until(&self, until: GameTime) -> i64 {
        let player = self.state.player();
        let speed = i64::from(player.speed(&self.content));
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
        let began = self.state.time;
        self.state.turn += 1;
        // The player has acted: what they heard before is spent, and they hear afresh (lines
        // muttered too: said only if heard this turn).
        self.player_mut().heard.clear();
        self.mutters.clear();
        // What the player notices in passing where their turn left them (spec §8 "Searching").
        self.notice_hidden();
        let price = price.saturating_add(self.faint());
        let player = self.state.player;
        let level = &self.state.current_level;
        let scheduled = |a: &Actor| Scheduled {
            id: a.id,
            speed: a.speed(&self.content),
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
        // The player's action is taken: what eases by actions eases (`weapon_ability.rs`).
        self.spend_actions(player, events);
        self.acting = None;
        // Holds whose holder no longer holds end (`hold.rs`): after the player's action, after
        // every creature's turn, and as the turn ends.
        self.loosen_holds(events);
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
            let id = schedule[next].id;
            self.acting = Some(id);
            let price = self.creature_turn(id, events);
            schedule[next].energy = schedule[next].energy.saturating_sub(price.max(1));
            self.spend_actions(id, events);
            self.acting = None;
            self.loosen_holds(events);
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
        // The player's next action is under way from here.
        self.acting = Some(player);
        self.fresh.clear();
        // What comes to the level meanwhile (spec §7 "Danger levels").
        self.spawn_tick();
        // The dead that rise meanwhile (spec §7 "Special rooms": a cursed room).
        self.rise_tick();
        // What haunts a rest (spec §7 "Special rooms": a haunted room).
        self.haunt_tick(began);
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
        // While the player rests, an idle creature may take a whole stride at once.
        let ahead = if self.resting {
            super::stride::STRIDE
        } else {
            0
        };
        self.state
            .actors
            .values()
            .filter(|a| a.level == level.id && self.disturbs_rest(a))
            .map(|a| {
                let steps = i64::from((a.pos.chebyshev(player) - VIEW_RADIUS - 1 - ahead).max(0));
                let cheapest = cheapest(self.cost_of(a.id, ActionKind::Step));
                let short = steps.saturating_mul(cheapest).saturating_sub(a.energy);
                let speed = a.speed(&self.content);
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
