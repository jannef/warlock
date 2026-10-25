//! Eating and corpses (spec §8 "Hunger").
//!
//! **A meal** ([`Action::Eat`](crate::Action::Eat): one of a stack from the pack or the floor)
//! takes as many turns as the food says, each the `eat` action, and passes them like a rest
//! ([`World::dine`], through [`World::keep_at`]): it stops early after a turn in which a creature
//! came into view or a message was logged that stops a rest ("You stop eating."), and the food is
//! left as it was. Only when
//! the last turn is done is one eaten, and it feeds: "You eat the ration." In a shop, what is
//! eaten is bought ([`World::ate_goods`], spec §7 "Shops"). Then come what a corpse does when
//! eaten and the news of the player's hunger ("You feel satiated.").
//!
//! **What food does**: its nutrition, up to the most there can be. A meal that would go past it
//! is overeating: the player may choke to death on it (a chance, 0 by default: off), else vomit
//! (a chance), else what does not fit is lost. Vomiting leaves at most `vomit_leaves` nutrition
//! and feeds nothing. A remedy (food with `cures`, `data/objects.toml`) then cures what it
//! names, as [`World::cure`] does (a disease remedy ends every disease, spec §8 "Diseases").
//!
//! **Rotten food** (spec §8 "Hunger"; food rots by its age, `world/rot.rs`): eating any food
//! past its keeping is risky (vomiting, else sickness and exposure to the diseases rotten food
//! carries, [`World::expose_to`], and then it feeds). Old food is as good as fresh. A race that
//! `eats_rotten` (ratfolk) eats it without harm: it only feeds them.
//!
//! **Corpses**: a creature that dies leaves one on top of its pile ([`World::leave_corpse`]), its
//! death time in the stack, from which it rots like any food. A fresh one feeds by its
//! creature's size and does what its creature's `on_eat` says, in this order, each draw from the
//! gameplay stream: its message, a chance to vomit, a chance per stat to raise it (up to its
//! potential), healing, harm, and extra nutrition; a rotten one has lost all of that. The player
//! learns what a corpse does only by eating it.
//!
//! **Sickness**: harm from what was eaten is degrees of the status `sickness` names
//! (`data/hunger.toml`: poisoned, spec §8 "Status effects"), given through [`World::afflict`],
//! the one way to give a status; it costs hit points as time passes and may kill ("Died of
//! poison").

use super::{Actor, Cause, Ended, Spent, World};
use crate::action::{DiseaseNews, Event, Meal, Positions};
use crate::content::{ActionKind, ObjectId};
use crate::object::{self, Stack};
use crate::stats::Stat;
use crate::talent::TalentMods;
use crate::time::GameTime;
use std::cell::Cell;

/// A meal under way: where the food is, the food as it lay when the meal began (its object and
/// when it was made; food is never magic), and the turns it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Dish {
    meal: Meal,
    object: ObjectId,
    died: Option<GameTime>,
    turns: u32,
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
    /// The player sets out to eat `meal`. Refused, taking no time, when nobody hungers, or there
    /// is no such stack, or it is not food.
    pub(super) fn eat(&mut self, meal: Meal) -> Spent {
        let content = self.content.clone();
        if content.hunger().is_none() || self.state.player().satiation.is_none() {
            self.say("You feel no need to eat.");
            return Spent::Nothing;
        }
        let stack = match meal {
            Meal::Pack(i) => self.state.player().inventory.get(i).cloned(),
            Meal::Floor(i) => {
                let here = self.player_pos();
                self.state.current().objects_at(here).get(i).cloned()
            }
        };
        let Some(stack) = stack else {
            self.say(match meal {
                Meal::Pack(_) => "You carry no such thing.",
                Meal::Floor(_) => "There is no such thing here.",
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
            object: stack.object,
            died: stack.died,
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
    /// like it there if the pile changed meanwhile), and it feeds and does what it does: rotten
    /// food, by its age now, what rotten food does.
    fn finish(&mut self, dish: Dish, events: &mut Vec<Event>) {
        let burden = self.load_of(self.state.player).burden();
        let Some(food) = self.take_one(&dish) else {
            self.say("There is nothing left to eat.");
            return;
        };
        let content = self.content.clone();
        let Some(before) = self.player_hunger() else {
            return;
        };
        let def = content.object(food.object);
        self.say(format!("You eat the {}.", def.name));
        self.ate_goods(dish.meal, &food);
        if let Some(creature) = def.corpse {
            self.ate_corpse(creature);
        }
        let nutrition = def.food.map_or(0, |f| f.nutrition) as i32;
        let rotten = self.stage(&food).is_some_and(|s| s.is_rotten());
        match def.corpse {
            _ if rotten => self.eat_rotten(nutrition, food.object, events),
            Some(creature) => {
                let on_eat = content.creature(creature).on_eat.clone();
                let on_eat = on_eat.unwrap_or_default();
                self.eat_fresh(nutrition, food.object, on_eat, events);
            }
            None => self.feed(nutrition, food.object),
        }
        self.remedy(&def.cures, events);
        self.meal_heals(&food, rotten);
        if self.state.death.is_none() {
            self.hunger_news(before);
            self.burden_news(burden);
        }
    }

    /// Takes one of `dish`'s food from where its meal says: the stack there if it is still that
    /// food, else the first stack like it there (of the same age if there is one: corpses do not
    /// join). Returns the one taken, with its age, if there was one.
    fn take_one(&mut self, dish: &Dish) -> Option<Stack> {
        let same = |s: &Stack| s.object == dish.object && s.died == dish.died;
        let like = |s: &Stack| s.object == dish.object;
        let find = |stacks: &[Stack], i: usize| {
            (stacks.get(i).filter(|s| same(s)).map(|_| i))
                .or_else(|| stacks.iter().position(same))
                .or_else(|| stacks.get(i).filter(|s| like(s)).map(|_| i))
                .or_else(|| stacks.iter().position(like))
        };
        match dish.meal {
            Meal::Pack(i) => {
                let player = self.player_mut();
                let at = find(&player.inventory, i)?;
                let one = player.inventory[at].single();
                if player.inventory[at].count > 1 {
                    player.inventory[at].count -= 1;
                } else {
                    player.inventory.remove(at);
                    player.equipment.after_removal(&Positions::of([at]));
                }
                Some(one)
            }
            Meal::Floor(i) => {
                let here = self.player_pos();
                let level = self.state.current_mut();
                let mut pile = level.objects_at(here).to_vec();
                let at = find(&pile, i)?;
                let one = pile[at].single();
                if pile[at].count > 1 {
                    pile[at].count -= 1;
                } else {
                    pile.remove(at);
                }
                level.set_pile(here, pile);
                Some(one)
            }
        }
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
        // Herb lore, Cauldron (spec §8 "Class abilities"): a corpse's own harm never touches
        // them (no draw for it).
        let harmless = self
            .state
            .player()
            .talent_mods(&self.content)
            .corpse_harmless;
        if !harmless && on_eat.vomit > 0 && self.state.rng.chance(u32::from(on_eat.vomit)) {
            self.vomit();
            return;
        }
        // Gourmand (spec §8 "Talents"): better chances, never above certain.
        let more = self
            .state
            .player()
            .talent_mods(&self.content)
            .corpse_stats_percent;
        for (&stat, &chance) in &on_eat.stat {
            let chance = TalentMods::scaled(u64::from(chance), more).min(100) as u32;
            if chance > 0 && self.state.rng.chance(chance) && self.player_mut().stats.raise(stat) {
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
        if let Some(harm) = on_eat.harm.filter(|_| !harmless) {
            let degrees = harm.roll(&mut self.state.rng);
            self.sicken(degrees, events);
        }
        self.feed(nutrition.saturating_add(on_eat.satiation).max(0), what);
    }

    /// A rotten corpse: the player vomits (a chance), else it makes them sick, exposes them to
    /// the diseases rotten food carries (grave rot), and then feeds them; a player whose race
    /// `eats_rotten` it only feeds.
    pub(super) fn eat_rotten(&mut self, nutrition: i32, what: ObjectId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rotten) = content.hunger().map(|h| &h.rotten) else {
            return;
        };
        let race = self.state.race_of(&content, self.state.player());
        // Deep roots (a Wood-born's pick) eats it as such a race does.
        let rooted = self.state.player().talent_mods(&content).eats_rotten;
        if rooted || race.is_some_and(|r| r.eats_rotten) {
            let name = &content.object(what).name;
            self.say(format!(
                "The {name} was rotten, but your stomach does not mind."
            ));
            self.feed(nutrition, what);
            return;
        }
        self.say(format!(
            "Ugh! The {} was rotten.",
            content.object(what).name
        ));
        // Iron stomach (spec §8 "Talents"): never vomits, and half the sickness (no draw for
        // the vomiting it is spared).
        let iron = self.state.player().talent_mods(&content).iron_stomach;
        if !iron && rotten.vomit > 0 && self.state.rng.chance(u32::from(rotten.vomit)) {
            self.vomit();
            return;
        }
        // Purify (spec §8 "Class abilities"): rotten food neither sickens them nor exposes them
        // to a disease, and draws nothing for it.
        if self.state.player().talent_mods(&content).purify {
            self.feed(nutrition, what);
            return;
        }
        let degrees = rotten.harm.roll(&mut self.state.rng);
        // Iron stomach halves it, and a talent that eats rot better (a birth sign's Evergreen)
        // lessens it, rounded down.
        let degrees = if iron { degrees / 2 } else { degrees };
        let percent = self.state.player().talent_mods(&content).rot_harm_percent;
        let degrees = if percent == 0 {
            degrees
        } else {
            let kept =
                crate::talent::TalentMods::scaled(u64::try_from(degrees).unwrap_or(0), percent);
            i32::try_from(kept).unwrap_or(0)
        };
        if degrees > 0 {
            self.sicken(degrees, events);
        }
        // Rotten food may carry a disease (grave rot): a check to resist it (spec §8 "Diseases").
        self.expose_to(self.state.player, "rotten_food");
        self.feed(nutrition, what);
    }

    /// What the player's talents heal of a meal of `food` besides (spec §8 "Class abilities":
    /// Herb lore's angelica root, Cauldron's fresh corpses): each match rolls its dice, never
    /// past the maximum. A rotten meal heals nothing; no draw without a match.
    fn meal_heals(&mut self, food: &Stack, rotten: bool) {
        let mods = self.state.player().talent_mods(&self.content);
        let corpse = self.content.object(food.object).corpse.is_some();
        let matched: Vec<_> = (mods.meal_heals.iter())
            .filter(|(object, _)| match object {
                Some(o) => *o == food.object,
                None => corpse && !rotten,
            })
            .map(|(_, dice)| *dice)
            .collect();
        for dice in matched {
            let points = dice.roll(&mut self.state.rng).max(0);
            let max = self.state.player().max_hp(&self.content);
            let me = self.player_mut();
            if points > 0 && me.hp < max {
                me.hp = me.hp.saturating_add(points).min(max);
                self.say("You feel better.");
            }
        }
    }

    /// What the player ate cures what `cures` names ([`World::cure`]): each disease it ends
    /// and each status it ends is said at once, in order with the meal's other messages
    /// ("The rot is gone from your flesh.").
    fn remedy(&mut self, cures: &[String], events: &mut Vec<Event>) {
        let from = events.len();
        for cure in cures {
            self.cure(self.state.player, cure, events);
        }
        let content = &self.content;
        let ended: Vec<String> = (events[from..].iter())
            .filter_map(|e| match *e {
                Event::Disease {
                    disease,
                    news: DiseaseNews::Recovered,
                    ..
                } => Some(content.disease(disease).recovered.clone()),
                Event::Status {
                    status,
                    began: false,
                    ..
                } => content.status(status).end.clone(),
                _ => None,
            })
            .collect();
        for text in ended {
            self.say(text);
        }
    }

    /// What the player ate makes them sick: `degrees` of the status `sickness` names
    /// (`data/hunger.toml`; none for 0 or less), given through [`World::afflict`]. Its beginning
    /// is said at once ("You feel sick."), in order with the meal's other messages.
    fn sicken(&mut self, degrees: i32, events: &mut Vec<Event>) {
        let sickness =
            (self.content.hunger()).and_then(|h| self.content.status_id(h.sickness.as_deref()?));
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
    /// what does not fit is lost. With Gourmand (spec §8 "Talents") a meal never leaves them
    /// Bloated: what does not fit below it is lost, and they never overeat.
    fn feed(&mut self, points: i32, what: ObjectId) {
        let content = self.content.clone();
        let Some(def) = content.hunger() else {
            return;
        };
        let Some(nutrition) = self.state.player().satiation.map(|s| s.nutrition) else {
            return;
        };
        if self.state.player().talent_mods(&content).no_bloating {
            let most = (def.states.bloated - 1).max(nutrition);
            if nutrition.saturating_add(points) > most {
                self.say("You eat your fill; the rest does you no good.");
            }
            if let Some(fed) = self.player_mut().satiation.as_mut() {
                fed.feed(points, most);
            }
            return;
        }
        if nutrition.saturating_add(points) > def.max {
            let o = def.overeating;
            if o.choke > 0 && self.state.rng.chance(u32::from(o.choke)) {
                self.die_of(Cause::Choked(what), self.state.time);
                return;
            }
            let iron = self.state.player().talent_mods(&content).iron_stomach;
            if !iron && o.vomit > 0 && self.state.rng.chance(u32::from(o.vomit)) {
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
}
