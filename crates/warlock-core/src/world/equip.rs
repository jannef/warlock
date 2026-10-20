//! Wearing and wielding (spec §6 "Equipment"): the player puts inventory stacks on in slots and
//! takes them off ([`crate::equipment`]).
//!
//! Every thing put on or taken off takes the time of its slot's action (`wield`, `wear`,
//! `body_armor`, or `ready` in `data/actions.toml`): a swap pays for both, and a two-handed
//! weapon taking the place of a sword and a shield pays for all three. Taking something off
//! costs what putting it on does. A refused attempt takes no time. What is worn and wielded
//! fights and protects (`fight.rs`); it weighs what it weighs, worn or not.

use super::{Spent, World};
use crate::content::{ActionKind, SlotId};
use crate::equipment::{self, verbs};

impl World {
    /// Puts inventory stack `item` on in slot `slot` (an id), taking off first what is in the
    /// slots it takes, and moving it if it is in another slot.
    pub(super) fn equip(&mut self, slot: &str, item: usize) -> Spent {
        let content = self.content.clone();
        let Some(slot) = content.slot_id(slot) else {
            self.say(format!("There is no slot `{slot}`."));
            return Spent::Nothing;
        };
        let player = self.state.player();
        let Some(stack) = player.inventory.get(item).cloned() else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let what = stack.describe(&content);
        if !content.fits(stack.object, slot) {
            let does = if stack.count > 1 { "do" } else { "does" };
            self.say(format!("{} {does} not go there.", capitalized(&what)));
            return Spent::Nothing;
        }
        let taken = equipment::slots_taken(&content, stack.object, slot);
        let worn = &player.equipment;
        if worn.slots_of(item) == taken {
            self.say(format!("You already have {what} there."));
            return Spent::Nothing;
        }
        // What comes off first: the thing itself if it is in another slot, then whatever is in
        // the slots it takes. Each pays the action of the first slot it is in.
        let mut off: Vec<usize> = Vec::new();
        if worn.holds(item) {
            off.push(item);
        }
        for s in &taken {
            if let Some(other) = worn.get(*s)
                && !off.contains(&other)
            {
                off.push(other);
            }
        }
        let action = |i: usize| content.slot(worn.slots_of(i)[0]).action;
        let mut kinds: Vec<ActionKind> = off.iter().map(|&i| action(i)).collect();
        let mut said: Vec<String> = off
            .iter()
            .filter(|&&i| i != item)
            .map(|&i| {
                let (_, take_off) = verbs(action(i));
                format!("You {take_off} {}.", player.inventory[i].describe(&content))
            })
            .collect();
        let kind = content.slot(slot).action;
        kinds.push(kind);
        let (put_on, _) = verbs(kind);
        let hands = if taken.len() > 1 {
            " in both hands"
        } else {
            ""
        };
        said.push(format!("You {put_on} {what}{hands}."));
        let worn = &mut self.player_mut().equipment;
        for i in off {
            worn.take_off(i);
        }
        worn.put(&taken, item);
        for text in said {
            self.say(text);
        }
        self.price_for(&kinds)
    }

    /// Takes off what is in slot `slot` (an id).
    pub(super) fn remove(&mut self, slot: &str) -> Spent {
        let content = self.content.clone();
        let Some(slot) = content.slot_id(slot) else {
            self.say(format!("There is no slot `{slot}`."));
            return Spent::Nothing;
        };
        let player = self.state.player();
        let Some(item) = player.equipment.get(slot) else {
            self.say(format!(
                "You have nothing there ({}).",
                content.slot(slot).name.to_lowercase()
            ));
            return Spent::Nothing;
        };
        let kind = self.first_action(item);
        let (_, take_off) = verbs(kind);
        let what = player.inventory[item].describe(&content);
        self.player_mut().equipment.take_off(item);
        self.say(format!("You {take_off} {what}."));
        self.price_for(&[kind])
    }

    /// Takes off whichever of the inventory stacks `items` are worn or wielded, before they are
    /// dropped: says so, and returns the action of each.
    pub(super) fn take_off_before_dropping(
        &mut self,
        items: impl Iterator<Item = usize>,
    ) -> Vec<ActionKind> {
        let content = self.content.clone();
        let mut kinds = Vec::new();
        for item in items {
            if !self.state.player().equipment.holds(item) {
                continue;
            }
            let kind = self.first_action(item);
            let (_, take_off) = verbs(kind);
            let what = self.state.player().inventory[item].describe(&content);
            self.player_mut().equipment.take_off(item);
            self.say(format!("You {take_off} {what}."));
            kinds.push(kind);
        }
        kinds
    }

    /// The action of the first slot the player's inventory stack `item` is in.
    fn first_action(&self, item: usize) -> ActionKind {
        let slots: Vec<SlotId> = self.state.player().equipment.slots_of(item);
        slots
            .first()
            .map_or(ActionKind::Wear, |&s| self.content.slot(s).action)
    }

    /// A turn of the player's actions `kinds`, one after the other.
    pub(super) fn price_for(&self, kinds: &[ActionKind]) -> Spent {
        let player = self.state.player;
        Spent::Turn(kinds.iter().map(|&k| self.price_of(player, k)).sum())
    }
}

/// `text` with its first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}
