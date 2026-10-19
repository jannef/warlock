//! Eating and corpses (spec §8 "Hunger").
//!
//! **A meal** ([`Action::Eat`](crate::Action::Eat): one of a stack from the pack or the floor)
//! takes as many turns as the food says, each the `eat` action, and passes them like a rest
//! ([`World::dine`], through [`World::keep_at`]): it stops early after a turn in which a creature
//! came into view or a message was logged that stops a rest ("You stop eating."), and the food is
//! left as it was. Only when
//! the last turn is done is one eaten, and it feeds: "You eat the ration." Then come what a
//! corpse does when eaten and the news of the player's hunger ("You feel satiated.").
//!
//! **What food does**: its nutrition, up to the most there can be. A meal that would go past it
//! is overeating: the player may choke to death on it (a chance, 0 by default: off), else vomit
//! (a chance), else what does not fit is lost. Vomiting leaves at most `vomit_leaves` nutrition
//! and feeds nothing.
//!
//! **Corpses**: a creature that dies leaves one on top of its pile ([`World::leave_corpse`]), its
//! death time in the stack. A corpse rots: rotten after `rotten_hours`, gone after `gone_hours`
//! ([`World::rot_away`], from the level the player is on and from their pack). Eating a rotten
//! one is risky (vomiting, else sickness, and then it feeds); a fresh one feeds by its
//! creature's size and does what its creature's `on_eat` says, in this order, each draw from the
//! gameplay stream: its message, a chance to vomit, a chance per stat to raise it (up to its
//! potential), healing, harm, and extra nutrition. The player learns what a corpse does only by
//! eating it.
//!
//! **Sickness**: harm from what was eaten is degrees of the status `[corpses] sickness` names
//! (poisoned: spec §8 "Status effects"), given through [`World::afflict`], the one way to give a
//! status; it costs hit points as time passes and may kill ("Died of poison").

use super::{Actor, Cause, Ended, Spent, World};
use crate::action::{Event, Letters, Meal, letter};
use crate::content::{ActionKind, ObjectId};
use crate::object::{self, Stack};
use crate::stats::Stat;
use crate::time::GameTime;
use std::cell::Cell;

/// A meal under way: where the food is, one of it as it lay when the meal began, and the turns
/// it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Dish {
    meal: Meal,
    food: Stack,
    turns: u32,
}

/// How a corpse is, by its age.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rot {
    Fresh,
    Rotten,
    /// Rotted away: it is gone.
    Gone,
}

/// What the player feels when a corpse raises `stat`.
fn rise_news(stat: Stat) -> &'static str {
    match stat {
        Stat::Strength => "You feel stronger!",
        Stat::Dexterity => "You feel more agile!",
        Stat::Toughness => "You feel tougher!",
        Stat::Intelligence => "You feel smarter!",
        Stat::Willpower => "You feel more resolute!",
        Stat::Presence => "You feel more self-assured!",
    }
}

impl World {
    /// How `stack` has rotted by now: `None` for anything that is no corpse.
    pub(crate) fn rot(&self, stack: &Stack) -> Option<Rot> {
        rot_at(&self.content, stack, self.state.time)
    }

    /// The player sets out to eat `meal`. Refused, taking no time, when nobody hungers, or there
    /// is no such stack, or it is not food.
    pub(super) fn eat(&mut self, meal: Meal) -> Spent {
        let content = self.content.clone();
        if content.hunger().is_none() || self.state.player().satiation.is_none() {
            self.say("You feel no need to eat.");
            return Spent::Nothing;
        }
        let (stack, i) = match meal {
            Meal::Pack(i) => (self.state.player().inventory.get(i).copied(), i),
            Meal::Floor(i) => {
                let here = self.player_pos();
                (self.state.current().objects_at(here).get(i).copied(), i)
            }
        };
        let Some(stack) = stack else {
            let c = letter(i).unwrap_or('?');
            self.say(match meal {
                Meal::Pack(_) => format!("You carry no {c}."),
                Meal::Floor(_) => format!("There is no {c} here."),
            });
            return Spent::Nothing;
        };
        let def = content.object(stack.object);
        let Some(food) = def.food else {
            self.say(format!("You cannot eat {}.", def.describe(1)));
            return Spent::Nothing;
        };
        Spent::Meal(Dish {
            meal,
            food: Stack { count: 1, ..stack },
            turns: food.turns,
        })
    }

    /// Eats `dish` turn after turn through the normal pipeline ([`World::keep_at`]: the player
    /// eats, the others act, perception), each turn the `eat` action, stopping early after a turn
    /// in which a creature came into view or a message was logged that stops a rest, or the
    /// player died. The food is eaten only when the last turn is done.
    pub(super) fn dine(&mut self, dish: Dish, events: &mut Vec<Event>) {
        let left = Cell::new(dish.turns);
        let ended = self.keep_at(
            events,
            |_| left.get() > 0,
            |w, _| {
                left.set(left.get() - 1);
                w.price_of(w.state.player, ActionKind::Eat)
            },
        );
        match ended {
            Ended::Died => {}
            _ if left.get() == 0 => self.finish(dish, events),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you stop eating.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop eating."),
        }
    }

    /// The last turn of a meal is done: one of the food is eaten (from where it lay, or the first
    /// like it there if the pile changed meanwhile), and it feeds and does what it does.
    fn finish(&mut self, dish: Dish, events: &mut Vec<Event>) {
        let burden = self.load_of(self.state.player).burden();
        if !self.take_one(dish.meal, dish.food) {
            self.say("There is nothing left to eat.");
            return;
        }
        let content = self.content.clone();
        let Some(before) = self.player_hunger() else {
            return;
        };
        let def = content.object(dish.food.object);
        self.say(format!("You eat the {}.", def.name));
        let nutrition = def.food.map_or(0, |f| f.nutrition) as i32;
        match (def.corpse, self.rot(&dish.food)) {
            (Some(_), Some(Rot::Rotten | Rot::Gone)) => {
                self.eat_rotten(nutrition, dish.food.object, events);
            }
            (Some(creature), _) => {
                let on_eat = content.creature(creature).on_eat.clone();
                let on_eat = on_eat.unwrap_or_default();
                self.eat_fresh(nutrition, dish.food.object, on_eat, events);
            }
            (None, _) => self.feed(nutrition, dish.food.object),
        }
        if self.state.death.is_none() {
            self.hunger_news(before);
            self.burden_news(burden);
        }
    }

    /// Takes one of `food` from where `meal` says: the stack there if it is still that food, else
    /// the first stack like it there. Returns whether there was one.
    fn take_one(&mut self, meal: Meal, food: Stack) -> bool {
        let same = |s: &Stack| s.object == food.object && s.died == food.died;
        let find = |stacks: &[Stack], i: usize| {
            (stacks.get(i).filter(|s| same(s)).map(|_| i)).or_else(|| stacks.iter().position(same))
        };
        match meal {
            Meal::Pack(i) => {
                let player = self.player_mut();
                let Some(at) = find(&player.inventory, i) else {
                    return false;
                };
                if player.inventory[at].count > 1 {
                    player.inventory[at].count -= 1;
                } else {
                    player.inventory.remove(at);
                    player.equipment.after_removal(Letters::of([at]));
                }
            }
            Meal::Floor(i) => {
                let here = self.player_pos();
                let level = self.state.current_mut();
                let mut pile = level.objects_at(here).to_vec();
                let Some(at) = find(&pile, i) else {
                    return false;
                };
                if pile[at].count > 1 {
                    pile[at].count -= 1;
                } else {
                    pile.remove(at);
                }
                level.set_pile(here, pile);
            }
        }
        true
    }

    /// A fresh corpse feeds by its size and does what its creature's `on_eat` says (module docs).
    fn eat_fresh(
        &mut self,
        nutrition: i32,
        what: ObjectId,
        on_eat: crate::content::OnEat,
        events: &mut Vec<Event>,
    ) {
        if let Some(text) = on_eat.message {
            self.say(text);
        }
        if on_eat.vomit > 0 && self.state.rng.chance(u32::from(on_eat.vomit)) {
            self.vomit();
            return;
        }
        for (&stat, &chance) in &on_eat.stat {
            if chance > 0
                && self.state.rng.chance(u32::from(chance))
                && self.player_mut().stats.raise(stat)
            {
                self.say(rise_news(stat));
            }
        }
        if let Some(heal) = on_eat.heal {
            let points = heal.roll(&mut self.state.rng);
            let max = self.state.player().max_hp(&self.content);
            let player = self.player_mut();
            if points > 0 && player.hp < max {
                player.hp = player.hp.saturating_add(points).min(max);
                self.say("You feel better.");
            }
        }
        if let Some(harm) = on_eat.harm {
            let degrees = harm.roll(&mut self.state.rng);
            self.sicken(degrees, events);
        }
        self.feed(nutrition.saturating_add(on_eat.satiation).max(0), what);
    }

    /// A rotten corpse: the player vomits (a chance), else it makes them sick and then feeds
    /// them.
    fn eat_rotten(&mut self, nutrition: i32, what: ObjectId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(corpses) = content.hunger().and_then(|h| h.corpses.as_ref()) else {
            return;
        };
        self.say(format!(
            "Ugh! The {} was rotten.",
            content.object(what).name
        ));
        if corpses.rotten_vomit > 0 && self.state.rng.chance(u32::from(corpses.rotten_vomit)) {
            self.vomit();
            return;
        }
        let degrees = corpses.rotten_harm.roll(&mut self.state.rng);
        self.sicken(degrees, events);
        // Diseases (#139) will expose the eater to rotten food here too
        // (`World::expose_to(player, "rotten_food")`), once they land.
        self.feed(nutrition, what);
    }

    /// What the player ate makes them sick: `degrees` of the status `[corpses] sickness` names
    /// (none for 0 or less), given through [`World::afflict`]. Its beginning is said at once ("You
    /// feel sick."), in order with the meal's other messages.
    fn sicken(&mut self, degrees: i32, events: &mut Vec<Event>) {
        let sickness = (self.content.hunger())
            .and_then(|h| h.corpses.as_ref())
            .and_then(|c| self.content.status_id(c.sickness.as_deref()?));
        let (Some(status), Ok(degrees @ 1..)) = (sickness, u16::try_from(degrees)) else {
            return;
        };
        let from = events.len();
        self.afflict(self.state.player, status, degrees, None, events);
        let began: Vec<String> = (events[from..].iter())
            .filter_map(|e| match *e {
                Event::Status {
                    status,
                    began: true,
                    ..
                } => self.content.status(status).start.clone(),
                _ => None,
            })
            .collect();
        for text in began {
            self.say(text);
        }
    }

    /// The player gains `points` of nutrition from `what`. A meal that would go past the most
    /// there can be is overeating: they may choke to death (if choking is on), else vomit, else
    /// what does not fit is lost.
    fn feed(&mut self, points: i32, what: ObjectId) {
        let content = self.content.clone();
        let Some(def) = content.hunger() else {
            return;
        };
        let Some(nutrition) = self.state.player().satiation.map(|s| s.nutrition) else {
            return;
        };
        if nutrition.saturating_add(points) > def.max {
            let o = def.overeating;
            if o.choke > 0 && self.state.rng.chance(u32::from(o.choke)) {
                self.die_of(Cause::Choked(what), self.state.time);
                return;
            }
            if o.vomit > 0 && self.state.rng.chance(u32::from(o.vomit)) {
                self.vomit();
                return;
            }
            self.say("You have a hard time getting it all down.");
        }
        if let Some(fed) = self.player_mut().satiation.as_mut() {
            fed.feed(points, def.max);
        }
    }

    /// The player vomits: what they ate does not feed them, and they are left with at most
    /// `vomit_leaves` nutrition.
    fn vomit(&mut self) {
        let Some(leaves) = self.content.hunger().map(|h| h.vomit_leaves) else {
            return;
        };
        self.say("You vomit!");
        if let Some(fed) = self.player_mut().satiation.as_mut() {
            fed.nutrition = fed.nutrition.min(leaves);
        }
    }

    /// `actor`, a creature that just died, leaves its corpse (if its type leaves one) on top of
    /// the pile where it fell, dying now.
    pub(super) fn leave_corpse(&mut self, actor: &Actor) {
        let Some(corpse) = self.content.corpse_of(actor.creature) else {
            return;
        };
        let died = self.state.time;
        let content = self.content.clone();
        let Some(level) = self.state.levels.get_mut(&actor.level) else {
            return;
        };
        let mut pile = level.objects_at(actor.pos).to_vec();
        object::add_to_pile(&mut pile, &[Stack::corpse(corpse, died)], &content);
        level.set_pile(actor.pos, pile);
    }

    /// Corpses that have rotted away are gone: from the piles of the level the player is on, and
    /// from their pack ("Your kobold corpse rots away.", logged without stopping a rest). Checked
    /// as perception runs; corpses elsewhere go when the player comes back.
    pub(super) fn rot_away(&mut self) {
        let content = self.content.clone();
        let now = self.state.time;
        if content.hunger().and_then(|h| h.corpses.as_ref()).is_none() {
            return;
        }
        let gone = |s: &Stack| rot_at(&content, s, now) == Some(Rot::Gone);
        let level = self.state.current_mut();
        let cells: Vec<_> = (level.objects.iter())
            .filter(|(_, pile)| pile.iter().any(gone))
            .map(|(p, _)| *p)
            .collect();
        for p in cells {
            let pile: Vec<Stack> = level
                .objects_at(p)
                .iter()
                .filter(|s| !gone(s))
                .copied()
                .collect();
            level.set_pile(p, pile);
        }
        let inventory = &self.state.player().inventory;
        let rotted =
            Letters::of((inventory.iter().enumerate()).filter_map(|(i, s)| gone(s).then_some(i)));
        if rotted.is_empty() {
            return;
        }
        let names: Vec<String> = (rotted.iter())
            .map(|i| content.object(inventory[i].object).name.clone())
            .collect();
        let player = self.player_mut();
        let mut i = 0;
        player.inventory.retain(|_| {
            let keep = !rotted.contains(i);
            i += 1;
            keep
        });
        player.equipment.after_removal(rotted);
        for name in names {
            self.note(format!("Your {name} rots away."));
        }
    }
}

/// How `stack` has rotted at `now`, by the corpse rules of `content`: `None` for anything that is
/// no corpse (or without corpse rules).
pub(crate) fn rot_at(
    content: &crate::content::Content,
    stack: &Stack,
    now: GameTime,
) -> Option<Rot> {
    let died = stack.died?;
    let corpses = content.hunger()?.corpses.as_ref()?;
    let age = now.since(died);
    Some(if age >= corpses.gone {
        Rot::Gone
    } else if age >= corpses.rotten {
        Rot::Rotten
    } else {
        Rot::Fresh
    })
}
