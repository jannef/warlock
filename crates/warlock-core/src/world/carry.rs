//! Picking up and dropping (spec §6 "Objects", §8 "Burden"): [`crate::action::Action::PickUp`]
//! and [`crate::action::Action::Drop`], the positions they name, and the burden news a changed
//! load is said with. `world/mod.rs` has only the dispatch line.

use super::{Spent, World};
use crate::action::Positions;
use crate::burden::Burden;
use crate::content::ActionKind;
use crate::object::{self, Stack};

impl World {
    /// [`crate::action::Action::PickUp`]: picks up what `letters` names ([`World::pick_up`])
    /// and returns what it cost.
    pub(super) fn pick_up_objects(&mut self, letters: &Positions) -> Spent {
        // Priced at the load the player has as they begin.
        let price = self.price_of(self.state.player, ActionKind::PickUp);
        if self.pick_up(letters) {
            Spent::Turn(price)
        } else {
            Spent::Nothing
        }
    }

    /// A message when the player's burden is no longer `before` (spec §8 "Burden"): picking
    /// something up or dropping it changed it. Said right after the action that did it.
    pub(super) fn burden_news(&mut self, before: Burden) {
        let now = self.load_of(self.state.player).burden();
        if now != before {
            self.say(now.news());
        }
    }

    /// Picks up the stacks `letters` names in the pile on the player's cell (everything there
    /// with none). Returns whether anything was taken.
    pub(super) fn pick_up(&mut self, letters: &Positions) -> bool {
        let here = self.player_pos();
        let pile = self.state.current().objects_at(here).to_vec();
        if pile.is_empty() {
            self.say("There is nothing here to pick up.");
            return false;
        }
        let Some(mut chosen) = self.chosen(letters, pile.len(), "There is no such thing here.")
        else {
            return false;
        };
        let content = self.content.clone();
        // What burns stays where it lies.
        let burning: Vec<usize> = (chosen.iter())
            .filter(|&i| content.object(pile[i].object).burning.is_some())
            .collect();
        for &i in &burning {
            chosen.remove(i);
        }
        if chosen.is_empty() {
            let what = burning.first().map(|&i| pile[i].describe(&content));
            self.say(format!(
                "You cannot pick up {} while it burns.",
                what.unwrap_or_default()
            ));
            return false;
        }
        let inventory = self.state.inventory_mut();
        for i in chosen.iter() {
            object::add_to_inventory(inventory, pile[i].clone(), &content);
        }
        let (got, left) = split(&pile, &chosen);
        self.state.current_mut().set_pile(here, left);
        // In a shop, goods for sale are now owed for (spec §7 "Shops").
        let prices = self.picked_up_goods(&got);
        let tag = |i: usize| prices[i].map(|p| format!("unpaid, {p} gold"));
        let mut text = format!("You pick up {}.", object::phrase_with(&got, &content, tag));
        let shown: u32 = prices.iter().flatten().sum();
        let priced = prices.iter().flatten().count();
        if let Some((name, owed)) = self.owing()
            && priced > 0
            && (priced > 1 || owed != shown)
        {
            text.push_str(&format!(" You owe {name} {owed} gold."));
        }
        self.say(text);
        true
    }

    /// Drops the inventory stacks `letters` names (everything with none) on top of the pile on
    /// the player's cell, taking off first what of them is worn or wielded (which takes its time
    /// too).
    pub(super) fn drop_objects(&mut self, letters: &Positions) -> Spent {
        // In an open shop, unpaid goods go back and the player's own are offered for.
        if let Some(spent) = self.trade(letters, false) {
            return spent;
        }
        let inventory = self.state.player().inventory.clone();
        if inventory.is_empty() {
            self.say("You are not carrying anything.");
            return Spent::Nothing;
        }
        let Some(chosen) = self.chosen(letters, inventory.len(), "You carry no such thing.") else {
            return Spent::Nothing;
        };
        let mut kinds = self.take_off_before_dropping(chosen.iter());
        kinds.push(ActionKind::Drop);
        // Priced at the load the player has as they begin.
        let spent = self.price_for(&kinds);
        let (dropped, kept) = split(&inventory, &chosen);
        *self.state.inventory_mut() = kept;
        self.player_mut().equipment.after_removal(&chosen);
        let here = self.player_pos();
        let content = self.content.clone();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        object::add_to_pile(&mut pile, &dropped, &content);
        level.set_pile(here, pile);
        self.say(format!("You drop {}.", object::phrase(&dropped, &content)));
        spent
    }

    /// The positions `letters` names in a list of `len` stacks, every position for none. A
    /// position past the end is refused with the message `missing` (no letter: positions are
    /// not what the player sees, spec §6).
    pub(super) fn chosen(
        &mut self,
        letters: &Positions,
        len: usize,
        missing: &str,
    ) -> Option<Positions> {
        if letters.is_empty() {
            return Some(Positions::of(0..len));
        }
        if letters.iter().any(|i| i >= len) {
            self.say(missing);
            return None;
        }
        Some(letters.clone())
    }
}

/// The stacks of `stacks` that `chosen` names, and the others, each in order.
pub(super) fn split(stacks: &[Stack], chosen: &Positions) -> (Vec<Stack>, Vec<Stack>) {
    let (picked, rest): (Vec<_>, Vec<_>) = stacks
        .iter()
        .enumerate()
        .partition(|(i, _)| chosen.contains(*i));
    let unzip = |v: Vec<(usize, &Stack)>| v.into_iter().map(|(_, s)| s.clone()).collect();
    (unzip(picked), unzip(rest))
}
