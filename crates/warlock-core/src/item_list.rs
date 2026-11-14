//! Which stacks each item list offers: the eligibility of the lists an action or ability chooses
//! a stack from, on the perception side, so the screens and the rules share one answer (#1547).
//! An ability that asks for a stack names its list ([`crate::ability::Aim::Item`]); the app's
//! item-list screens read their rows from here.
//!
//! The rules still refuse what they refuse (a worn stack, a protected one, an unpaid one); a list
//! leaves out only what its rules always refuse (Salvage and Make do and mend never work on what
//! is worn or wielded).

use crate::object::Stack;
use crate::perception::PerceptionView;
use crate::world;

/// A list of stacks to choose one from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemList {
    /// The throwing weapons and flasks carried (the throw list).
    Throw,
    /// The throwing weapons carried, but flasks (the thief's Thrown exploit).
    ThrownExploit,
    /// Food, potions, and magic crystals: lying here first, then carried.
    Eat,
    /// The scrolls and grimoires carried.
    Read,
    /// The stacks carried whose kind is not known (the scroll of identify, the Identify spell,
    /// the merchant's Merchant's eye).
    Identify,
    /// The metal arms and armour carried that can be made finer (Temper, at an anvil).
    Temper,
    /// The wooden weapons carried (Whittle).
    Whittle,
    /// What is carried, not worn or wielded, with a material a profession works (Salvage).
    Salvage,
    /// What is carried damaged, not worn or wielded (the craftsperson's Make do and mend).
    MakeDo,
    /// The loose gems carried that may be cut (Jewelcraft's Cut).
    Cut,
    /// The scrolls carried of a type not yet identified (Decipher).
    Decipher,
    /// What fits a kept beast's neck, or, beside an escort, anything it could put on (Give).
    Give,
}

impl PerceptionView<'_> {
    /// The positions of the stacks the player carries that `list` offers, in inventory order.
    pub fn item_choices(&self, list: ItemList) -> Vec<usize> {
        let pack = self.inventory();
        (0..pack.len())
            .filter(|&i| self.offers(list, i, &pack[i]))
            .collect()
    }

    /// The positions of the stacks lying on the player's cell that `list` offers: food, potions,
    /// and crystals for the eat list (it asks about the floor first); none for any other.
    pub fn floor_choices(&self, list: ItemList) -> Vec<usize> {
        let pile = self.objects(self.viewer());
        match list {
            ItemList::Eat => (0..pile.len()).filter(|&i| self.edible(&pile[i])).collect(),
            _ => Vec::new(),
        }
    }

    /// Whether `list` offers `stack`, carried at inventory position `i`.
    fn offers(&self, list: ItemList, i: usize, stack: &Stack) -> bool {
        let content = self.content();
        let def = content.object(stack.object);
        match list {
            ItemList::Throw => self.is_throwing_weapon(i),
            ItemList::ThrownExploit => {
                self.is_throwing_weapon(i) && self.exploit_throw_range(i).is_some()
            }
            ItemList::Eat => self.edible(stack),
            ItemList::Read => def.read.is_some() || def.grimoire.is_some(),
            ItemList::Identify => !self.lore().knows(stack),
            ItemList::Temper => world::workable(content, stack),
            ItemList::Whittle => world::workable_wood(content, stack),
            ItemList::Salvage => world::salvageable(content, stack) && !self.equipment().holds(i),
            ItemList::MakeDo => world::mendable(stack) && !self.equipment().holds(i),
            ItemList::Cut => world::cuttable(content, stack),
            ItemList::Decipher => world::undeciphered(content, &self.lore(), stack),
            ItemList::Give => {
                world::fits_neck(content, stack)
                    || (self.escort_beside() && world::fits_escort(content, stack))
            }
        }
    }

    /// What may be consumed: food, potions (drunk from the eat list), and magic crystals
    /// (`world/crafting/crystal.rs`).
    fn edible(&self, stack: &Stack) -> bool {
        let def = self.content().object(stack.object);
        def.food.is_some() || def.drink.is_some() || self.is_crystal(stack)
    }
}
