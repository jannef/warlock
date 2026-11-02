//! Merchant's eye (spec §8 "Class abilities"; #816): the merchant's class active,
//! `use:merchants_eye:c` ([`crate::action::Action::UseOn`]), from the abilities menu's Class group
//! through the identify list. It tells one stack the player carries for what it is, as the
//! scroll of identify and the Identify spell do ([`World::identify`]: its magic, a labelled
//! scroll's or wand's type, a wand's charges), for the time of a reading (`read`; a status that
//! blocks reading blocks it too). It costs nothing else: no mana, no cooldown, no check, and no
//! limit on how often. Refused, with no time, without the ability, for a stack the player does
//! not carry, and for one they already know ("You already know what ... is."). A thing told
//! may train the Intelligence the scroll trains (`StatUse::Identified`). It draws nothing at
//! random but that training.

use super::loot::Identified;
use super::{Spent, World};
use crate::content::{ActionKind, ClassActive, StatUse};

impl World {
    /// `use:ID:LETTER`: the player uses class active `active` on inventory stack `item`; only
    /// one that asks for a stack ([`ClassActive::aims_item`]) may.
    pub(super) fn use_class_on(&mut self, active: ClassActive, item: usize) -> Spent {
        if !active.aims_item() {
            self.say(format!("`{}` works on nothing you carry.", active.id()));
            return Spent::Nothing;
        }
        if !self.may_use_class(active) {
            return Spent::Nothing;
        }
        match active {
            ClassActive::MerchantsEye => self.merchants_eye(item),
            _ => Spent::Nothing,
        }
    }

    /// Merchant's eye on inventory stack `item` (see the module docs).
    fn merchants_eye(&mut self, item: usize) -> Spent {
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if self.lore().knows(stack) {
            let what = self.lore().describe(&stack.single());
            self.say(format!("You already know what {what} is."));
            return Spent::Nothing;
        }
        let what = self.lore().describe(&stack.single());
        self.say(format!("You look over {what} with a merchant's eye."));
        if matches!(self.identify(item), Identified::Now(_)) {
            self.train_stat(StatUse::Identified);
        }
        Spent::Turn(self.price_of(self.state.player, ActionKind::Read))
    }
}
