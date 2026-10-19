//! Equipment (spec §6, §7): what an actor wears and wields.
//!
//! Equipped things stay in the actor's inventory (they keep their letters and still count
//! toward its weight); [`Equipment`] records which inventory stack is in which slot
//! (`data/slots.toml`, [`crate::content::SlotDef`]). A two-handed object is in every hand slot at
//! once. Combat reads it (`Actor::weapon`, `launcher`, `defence`).

use crate::action::Letters;
use crate::content::{ActionKind, Content, ObjectId, SlotId};
use crate::object::Stack;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Which inventory stack is in each slot that holds something, by slot.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment(BTreeMap<SlotId, u8>);

impl Equipment {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The inventory position of what is in `slot`.
    pub fn get(&self, slot: SlotId) -> Option<usize> {
        self.0.get(&slot).map(|&i| usize::from(i))
    }

    /// Every slot that holds something, in display order, with the inventory position there.
    pub fn iter(&self) -> impl Iterator<Item = (SlotId, usize)> + '_ {
        self.0.iter().map(|(&s, &i)| (s, usize::from(i)))
    }

    /// Each equipped inventory stack once, with the first slot it is in, in display order.
    pub fn items(&self) -> Vec<(SlotId, usize)> {
        let mut items: Vec<(SlotId, usize)> = Vec::new();
        for (slot, item) in self.iter() {
            if !items.iter().any(|&(_, i)| i == item) {
                items.push((slot, item));
            }
        }
        items
    }

    /// The slots inventory stack `item` is in (two for a two-handed object), in display order.
    pub fn slots_of(&self, item: usize) -> Vec<SlotId> {
        self.iter()
            .filter(|&(_, i)| i == item)
            .map(|(s, _)| s)
            .collect()
    }

    /// Whether inventory stack `item` is worn or wielded.
    pub fn holds(&self, item: usize) -> bool {
        self.iter().any(|(_, i)| i == item)
    }

    /// Puts inventory stack `item` in `slots` (what [`slots_taken`] gives).
    pub(crate) fn put(&mut self, slots: &[SlotId], item: usize) {
        let Ok(item) = u8::try_from(item) else { return };
        for &slot in slots {
            self.0.insert(slot, item);
        }
    }

    /// Takes inventory stack `item` out of every slot it is in.
    pub(crate) fn take_off(&mut self, item: usize) {
        self.0.retain(|_, i| usize::from(*i) != item);
    }

    /// Follows the inventory losing the stacks `removed` names: those come off, and the stacks
    /// after them move up a letter.
    pub(crate) fn after_removal(&mut self, removed: Letters) {
        self.0.retain(|_, i| !removed.contains(usize::from(*i)));
        for i in self.0.values_mut() {
            let below = removed.iter().filter(|&r| r < usize::from(*i)).count();
            *i -= u8::try_from(below).unwrap_or(0);
        }
    }

    /// Whether everything equipped is in `inventory` and in the slots it takes (a guard for
    /// saves and spawn data).
    pub(crate) fn is_valid_for(&self, inventory: &[Stack], content: &Content) -> bool {
        self.items().into_iter().all(|(_, item)| {
            let Some(stack) = inventory.get(item) else {
                return false;
            };
            let slots = self.slots_of(item);
            slots
                .iter()
                .all(|s| usize::from(s.0) < content.slots().len())
                && slots
                    .iter()
                    .find(|&&s| content.fits(stack.object, s))
                    .is_some_and(|&s| slots == slots_taken(content, stack.object, s))
        })
    }
}

/// The slots `object` takes when it is put in `slot`: every hand slot for a two-handed object,
/// else just `slot`.
pub fn slots_taken(content: &Content, object: ObjectId, slot: SlotId) -> Vec<SlotId> {
    if content.object(object).two_handed && content.slot(slot).hand {
        content.hand_slots().collect()
    } else {
        vec![slot]
    }
}

/// The first slot `object` fits that is free (every hand free, for a two-handed object), in
/// display order: where it goes when nobody says which slot (creatures' spawn data).
pub fn free_slot(content: &Content, equipment: &Equipment, object: ObjectId) -> Option<SlotId> {
    // Only where it belongs: not the missile weapon slot that takes any weapon too.
    content
        .slot_ids()
        .map(|(id, _)| id)
        .filter(|&s| content.fits_properly(object, s))
        .find(|&s| {
            let taken = slots_taken(content, object, s);
            taken.iter().all(|&t| equipment.get(t).is_none())
        })
}

/// What the inventory says of stack `item`, without parentheses: the mark of the slot it is in
/// ("being worn", "wielded"), or for a two-handed object "wielded in both hands"; `None` if it
/// is not equipped.
pub fn mark(content: &Content, equipment: &Equipment, item: usize) -> Option<String> {
    let slots = equipment.slots_of(item);
    let first = content.slot(*slots.first()?);
    Some(if slots.len() > 1 {
        format!("{} in both hands", first.mark)
    } else {
        first.mark.clone()
    })
}

/// How an equipment action is said: the verb for putting a thing on and for taking it off.
pub fn verbs(kind: ActionKind) -> (&'static str, &'static str) {
    match kind {
        ActionKind::Wield => ("wield", "put away"),
        ActionKind::Ready => ("ready", "put away"),
        _ => ("put on", "take off"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        Content::builtin().expect("built-in content")
    }

    fn slot(c: &Content, id: &str) -> SlotId {
        c.slot_id(id).unwrap_or_else(|| panic!("slot {id}"))
    }

    fn object(c: &Content, id: &str) -> ObjectId {
        c.object_id(id).unwrap_or_else(|| panic!("object {id}"))
    }

    #[test]
    fn objects_fit_their_slots_and_rings_fit_either_hand() {
        let c = content();
        assert!(c.fits(object(&c, "leather_jerkin"), slot(&c, "body")));
        assert!(!c.fits(object(&c, "leather_jerkin"), slot(&c, "head")));
        let ring = object(&c, "silver_ring");
        assert!(c.fits(ring, slot(&c, "left_ring")) && c.fits(ring, slot(&c, "right_ring")));
        assert!(!c.fits(object(&c, "apple"), slot(&c, "main_hand")));
        // The slots come in ADOM's order, lettered a to m.
        let names: Vec<&str> = c.slots().iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Head",
                "Neck",
                "Body",
                "Cloak",
                "Main hand",
                "Off hand",
                "Hands",
                "Left ring",
                "Right ring",
                "Feet",
                "Missile weapon",
                "Missiles",
                "Tool"
            ]
        );
        // A second ring goes on the other hand.
        let mut worn = Equipment::default();
        worn.put(&[slot(&c, "left_ring")], 0);
        assert_eq!(free_slot(&c, &worn, ring), Some(slot(&c, "right_ring")));
        worn.put(&[slot(&c, "right_ring")], 1);
        assert_eq!(free_slot(&c, &worn, ring), None);
    }

    #[test]
    fn two_handed_objects_take_both_hands() {
        let c = content();
        let staff = object(&c, "quarterstaff");
        let (main, off) = (slot(&c, "main_hand"), slot(&c, "off_hand"));
        assert_eq!(slots_taken(&c, staff, main), [main, off]);
        assert_eq!(slots_taken(&c, object(&c, "spear"), main), [main]);
        let mut held = Equipment::default();
        held.put(&[off], 0);
        assert_eq!(free_slot(&c, &held, staff), None, "the off hand is taken");
        let mut held = Equipment::default();
        held.put(&slots_taken(&c, staff, main), 2);
        assert_eq!(held.slots_of(2), [main, off]);
        assert_eq!(held.items(), [(main, 2)]);
        assert_eq!(mark(&c, &held, 2).as_deref(), Some("wielded in both hands"));
        assert_eq!(mark(&c, &held, 0), None);
        held.take_off(2);
        assert!(held.is_empty());
    }

    #[test]
    fn launchers_shoot_their_kind_of_ammunition() {
        let c = content();
        let bow = object(&c, "short_bow");
        assert!(c.fires(bow, object(&c, "arrow")));
        assert!(!c.fires(bow, object(&c, "rock")));
        assert!(!c.fires(object(&c, "arrow"), object(&c, "arrow")));
        assert!(c.fits(bow, slot(&c, "missile_weapon")));
        assert!(c.fits(object(&c, "arrow"), slot(&c, "missiles")));
    }

    #[test]
    fn removing_stacks_moves_the_letters_up() {
        let c = content();
        let (head, body, feet) = (slot(&c, "head"), slot(&c, "body"), slot(&c, "feet"));
        let mut worn = Equipment::default();
        worn.put(&[head], 1);
        worn.put(&[body], 3);
        worn.put(&[feet], 4);
        worn.after_removal(Letters::of([0, 3]));
        assert_eq!(worn.iter().collect::<Vec<_>>(), [(head, 0), (feet, 2)]);
    }

    #[test]
    fn equipment_serializes_by_slot_and_validates() {
        let c = content();
        let jerkin = Stack::one(object(&c, "leather_jerkin"));
        let mut worn = Equipment::default();
        worn.put(&[slot(&c, "body")], 0);
        let text = serde_json::to_string(&worn).expect("serializes");
        let back: Equipment = serde_json::from_str(&text).expect("deserializes");
        assert_eq!(back, worn);
        assert!(worn.is_valid_for(&[jerkin], &c));
        assert!(!worn.is_valid_for(&[], &c), "nothing to wear");
        let mut wrong = Equipment::default();
        wrong.put(&[slot(&c, "head")], 0);
        assert!(!wrong.is_valid_for(&[jerkin], &c), "a jerkin on the head");
        let staff = Stack::one(object(&c, "quarterstaff"));
        let mut one_hand = Equipment::default();
        one_hand.put(&[slot(&c, "main_hand")], 0);
        assert!(!one_hand.is_valid_for(&[staff], &c), "a staff needs both");
    }
}
