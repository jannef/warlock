//! Drinking potions (spec §8 "Potions"): the Consume command's action, [`Action::Eat`], hands a
//! stack with a `drink` table here instead of starting a meal ([`World::eat`]).
//!
//! - **A drink** takes the `drink` action's time (one turn) and one of the stack, from the pack
//!   or the floor (in a shop, a potion drunk is bought, as food eaten is). With Thrifty drinker
//!   or worn items of the Alchemist (`potion_kept`) the potion is kept by their chance
//!   ([`Actor::potion_kept`]): one draw of the gameplay stream a drink, none while the chance
//!   is 0.
//! - **What it was**: "You drink the fizzy red potion." and, for a type the player did not know,
//!   "It is a potion of healing." Drinking identifies its type, as reading does a scroll's
//!   ([`World::learn_scroll`]).
//! - **What it does**, by its kind ([`Draught`]), each through the one path that does it: hit
//!   points back (never past the maximum) and what it cures; cures ([`World::cure`], and every
//!   disease for `diseases`); mana back; a stat raised for good (an elixir, `world/elixir.rs`);
//!   sickness ([`World::afflict`]). Dice are gameplay draws.
//!   A boost, a vision, a haste, or a resistance gives its status for a while
//!   ([`World::afflict`]: Fortified, Owl-eyed, Hasted, Warmed), which lasts as the status's
//!   `eases` says and the drinker's talents lengthen (Lasting draughts: `status_lasts`). What begins and ends of statuses and diseases is told from the action's
//!   events, as perception reports them ("You feel hale.", "The poison wears off."); a dose
//!   that only strengthens one already felt says the status's own line again, and one that
//!   changes nothing says so.
//! - **Marks**: each stat its `marks` name gets that many sure training marks (the `drank` use,
//!   [`World::train_stat_times`]; an elixir's are the `elixir` use's).
//!
//! Creatures do not drink.
//!
//! [`Action::Eat`]: crate::action::Action::Eat
//! [`Draught`]: crate::content::Draught

use super::{Actor, Spent, World};
use crate::action::{Event, Meal, Positions};
use crate::content::{ActionKind, Content, Draught, Effect, StatUse, StatusId};
use crate::object::Stack;

/// The most the worn items of the Alchemist raise the chance a potion is kept to, in percent,
/// with Thrifty drinker's (a talent's own chance above it stands).
pub(super) const MAX_KEPT: u8 = 50;

impl Actor {
    /// The chance in percent a potion this actor drinks is not used up: Thrifty drinker's
    /// (`TalentMods::potion_kept`) plus the worn items' `potion_kept` (of the Alchemist,
    /// unidentified too), the sum held to [`MAX_KEPT`] unless the talent alone is above it.
    pub fn potion_kept(&self, content: &Content) -> u8 {
        let talent = self.talent_mods(content).potion_kept;
        let worn = self.mods(content).get(Effect::PotionKept).max(0);
        let sum = (i32::from(talent) + worn).min(i32::from(MAX_KEPT.max(talent)));
        u8::try_from(sum).unwrap_or(talent)
    }
}

/// What the player is told when Thrifty drinker keeps a potion they drank.
pub(super) const KEPT: &str = "There is enough left in the flask for another draught.";

impl World {
    /// The player drinks one of `stack`, lying where `meal` says (module docs). Returns what it
    /// cost: always a turn.
    pub(super) fn drink_potion(
        &mut self,
        meal: Meal,
        stack: &Stack,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let Some(drink) = content.object(stack.object).drink.clone() else {
            return Spent::Nothing;
        };
        let price = self.price_of(self.state.player, ActionKind::Drink);
        let title = self.lore().title(stack);
        let (one, kept) = self.take_draught(meal);
        self.say(format!("You drink the {title}."));
        if self.learn_scroll(stack.object) {
            let what = content.object(stack.object).describe(1);
            self.say(format!("It is {what}."));
        }
        if !kept {
            self.drank_goods(meal, &one);
        }
        match &drink.does {
            Draught::Heal { dice, cures } => {
                let points = dice.roll(&mut self.state.rng).max(0);
                let max = self.state.player().max_hp(&content);
                let me = self.player_mut();
                let before = me.hp;
                me.hp = me.hp.saturating_add(points).min(max).max(me.hp);
                let healed = me.hp > before;
                self.say(if healed {
                    "You feel your wounds close."
                } else {
                    "You feel healthy."
                });
                let player = self.state.player;
                for cure in cures {
                    self.cure(player, cure, events);
                }
            }
            Draught::Cure { cures, diseases } => {
                let from = events.len();
                let player = self.state.player;
                for cure in cures {
                    self.cure(player, cure, events);
                }
                if *diseases {
                    let all: Vec<_> = (self.state.player().diseases.iter())
                        .map(|i| i.disease)
                        .collect();
                    for disease in all {
                        self.end_disease(player, disease, events);
                    }
                }
                if events.len() == from {
                    self.say("You feel cleansed.");
                }
            }
            Draught::Mana { dice } => {
                let points = dice.roll(&mut self.state.rng).max(0);
                let max = self.state.player().max_mana(&content);
                let me = self.player_mut();
                let before = me.mana;
                me.mana = me.mana.saturating_add(points).min(max).max(me.mana);
                let gained = me.mana > before;
                self.say(if gained {
                    "Your mind clears, and power wells up in you."
                } else {
                    "Your mind feels clear."
                });
            }
            &Draught::Elixir {
                stat,
                points,
                potential,
            } => self.drink_elixir(stat, points, potential),
            Draught::Harm { dice, status } => {
                let degrees = dice.roll(&mut self.state.rng);
                self.say("It tastes foul!");
                if let Ok(degrees @ 1..) = u16::try_from(degrees) {
                    let player = self.state.player;
                    self.afflict(player, *status, degrees, None, events);
                }
            }
            Draught::Boost { .. }
            | Draught::Vision { .. }
            | Draught::Haste { .. }
            | Draught::Resistance { .. } => {
                if let Some((status, severity)) = drink.does.lends() {
                    self.lend(status, severity, events);
                }
            }
        }
        let used = match drink.does {
            Draught::Elixir { .. } => StatUse::Elixir,
            _ => StatUse::Drank,
        };
        for &(stat, marks) in &drink.marks {
            self.train_stat_times(used, Some(stat), marks);
        }
        if kept {
            self.say(KEPT);
        }
        Spent::Turn(price)
    }

    /// A potion's status for a while (a boost, a vision, a haste, a resistance): `severity`
    /// degrees of `status` on the player. Its start line is the action's event when it begins;
    /// when it only grows stronger the line is said here, and when nothing changes, that.
    fn lend(&mut self, status: StatusId, severity: u16, events: &mut Vec<Event>) {
        let player = self.state.player;
        let before = self.state.player().effects.get(status).map(|e| e.severity);
        self.afflict(player, status, severity, None, events);
        let after = self.state.player().effects.get(status).map(|e| e.severity);
        match (before, after) {
            (Some(was), Some(now)) if now > was => {
                if let Some(start) = self.content.status(status).start.clone() {
                    self.say(start);
                }
            }
            (Some(_), _) => self.say("Nothing more comes of it."),
            _ => {}
        }
    }

    /// Takes one potion from where `meal` says: from the pack (the last empties its stack, and
    /// what was worn follows its letters) or the pile underfoot. Returns it, and whether it was
    /// kept instead ([`Actor::potion_kept`]; a draw of the gameplay stream only above 0).
    fn take_draught(&mut self, meal: Meal) -> (Stack, bool) {
        let keep = self.state.player().potion_kept(&self.content);
        let kept = keep > 0 && self.state.rng.chance(u32::from(keep));
        match meal {
            Meal::Pack(i) => {
                let player = self.player_mut();
                let one = player.inventory[i].single();
                if !kept {
                    player.inventory[i].count -= 1;
                    if player.inventory[i].count == 0 {
                        player.inventory.remove(i);
                        player.equipment.after_removal(&Positions::of([i]));
                    }
                }
                (one, kept)
            }
            Meal::Floor(i) => {
                let here = self.player_pos();
                let level = self.state.current_mut();
                let mut pile = level.objects_at(here).to_vec();
                let one = pile[i].single();
                if !kept {
                    pile[i].count -= 1;
                    if pile[i].count == 0 {
                        pile.remove(i);
                    }
                    level.set_pile(here, pile);
                }
                (one, kept)
            }
        }
    }
}
