//! The slots a body bars and opens (docs/design/profane-gifts.md §6.4, §10; `docs/dev/equipment.md`
//! "Barred and opened slots"): the slot talents' one read (`talent/slots.rs`).
//!
//! - **Owned**: a slot is the player's when it is everyone's, their class's own, or one a talent
//!   opens ([`has_slot`]: Many-fingered's extra ring). Wearing, the equipment screen
//!   (`PerceptionView::own_slots`), and what falls when an opening goes read it.
//! - **Barred**: a talent's `bars_slot` keeps the slot shown but empty (`— (horns)` on the
//!   equipment screen, `PerceptionView::barred`); putting anything there is refused with the
//!   talent's own words and no time ([`World::equip`]).
//! - **Shed** ([`World::shed_unworn`]): as a gift takes or goes, what is worn in a slot now barred
//!   or no longer the player's comes off and falls to their cell, said once each.

use crate::action::Positions;
use crate::content::{Content, SlotId};
use crate::object;
use crate::talent::TalentMods;
use crate::world::World;

/// Whether one of class `class` (`None`: no character) whose talents are `mods` has slot `slot`:
/// everyone's slots, their class's own, and the ones a talent opens.
pub(crate) fn has_slot(
    content: &Content,
    class: Option<&str>,
    mods: &TalentMods,
    slot: SlotId,
) -> bool {
    content.slot(slot).belongs_to(class) || mods.slots.opens.contains(&slot)
}

impl World {
    /// The player's slot `id`: none for an id no slot has, or a slot that is not theirs
    /// ([`has_slot`]: another class's own, unless a talent opens it).
    pub(super) fn own_slot(&self, id: &str) -> Option<SlotId> {
        let slot = self.content.slot_id(id)?;
        let class = self.state.character.as_ref().map(|c| c.spec.class.as_str());
        let mods = self.state.player().talent_mods(&self.content);
        has_slot(&self.content, class, &mods, slot).then_some(slot)
    }

    /// The words that refuse putting anything in slot `slot`, if a talent of the player's bars
    /// it ("Your horns are in the way.").
    pub(super) fn barred_words(&self, slot: SlotId) -> Option<String> {
        let me = self.state.player();
        if me.talents.is_empty() {
            return None;
        }
        let mods = me.talent_mods(&self.content);
        mods.slots.barred(slot).map(|b| b.refused.clone())
    }

    /// What the player wears in a slot that is barred to them now, or no longer theirs, comes
    /// off and falls to their cell: "Your iron helm no longer fits, and falls to the floor.", or
    /// for a slot gone, "Your gold ring slips from a finger that is no longer there.". Takes no
    /// time (a gift taking or going, `profane.rs`).
    pub(in crate::world) fn shed_unworn(&mut self) {
        let content = self.content.clone();
        let class = self.state.character.as_ref().map(|c| c.spec.class.clone());
        let me = self.state.player();
        let mods = me.talent_mods(&content);
        let mut shed: Vec<(usize, bool)> = Vec::new();
        for (slot, item) in me.equipment.iter() {
            let gone = !has_slot(&content, class.as_deref(), &mods, slot);
            if (gone || mods.slots.barred(slot).is_some()) && !shed.iter().any(|&(i, _)| i == item)
            {
                shed.push((item, gone));
            }
        }
        if shed.is_empty() {
            return;
        }
        let inventory = self.state.player().inventory.clone();
        for &(item, gone) in &shed {
            let title = self.lore().title(&inventory[item]);
            self.say(if gone {
                format!("Your {title} slips from a finger that is no longer there.")
            } else {
                format!("Your {title} no longer fits, and falls to the floor.")
            });
        }
        let chosen = Positions::of(shed.iter().map(|&(i, _)| i));
        let (dropped, kept) = super::carry::split(&inventory, &chosen);
        *self.state.inventory_mut() = kept;
        self.player_mut().equipment.after_removal(&chosen);
        let here = self.player_pos();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        object::add_to_pile(&mut pile, &dropped, &content);
        level.set_pile(here, pile);
        // Hit points and mana within what is left without it.
        let id = self.state.player;
        self.keep_hp_within_max(id);
        let me = self.player_mut();
        let max_mana = me.max_mana(&content);
        me.mana = me.mana.min(max_mana);
    }
}
