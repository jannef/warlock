//! Objects in the world (spec §5, §6): stacks lying in piles on cells and carried in
//! inventories, and what things weigh.
//!
//! A **pile** is the ordered list of stacks on a cell, the top first: the top is what the map
//! shows, and dropped objects go on top. An **inventory** is an ordered list of at most
//! [`INVENTORY_SLOTS`] stacks, lettered `a` to `z` in order; new stacks go last. A stackable
//! object (coins, arrows) joins a stack of the same object wherever it goes; any other object is
//! a stack of one.
//!
//! Weights are integer grams. An actor weighs its creature's base weight plus everything it
//! carries ([`Weight`]).

use crate::content::{Content, ObjectId};
use serde::{Deserialize, Serialize};

/// Most stacks an inventory holds: one per letter, `a` to `z`.
pub const INVENTORY_SLOTS: usize = 26;

/// Some number of one object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Stack {
    pub object: ObjectId,
    /// At least 1; always 1 for objects that do not stack.
    pub count: u32,
}

impl Stack {
    pub fn one(object: ObjectId) -> Self {
        Self { object, count: 1 }
    }

    /// Grams.
    pub fn weight(&self, content: &Content) -> u64 {
        u64::from(content.object(self.object).weight) * u64::from(self.count)
    }

    /// "a dagger", "12 arrows".
    pub fn describe(&self, content: &Content) -> String {
        content.object(self.object).describe(self.count)
    }

    /// Whether the object exists in `content` and the count makes sense (a guard for saves).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        usize::from(self.object.0) < content.object_defs().len()
            && self.count >= 1
            && (self.count == 1 || content.object(self.object).stackable)
    }
}

/// Total weight of `stacks`, in grams.
pub fn weight_of(stacks: &[Stack], content: &Content) -> u64 {
    stacks.iter().map(|s| s.weight(content)).sum()
}

/// What an actor weighs, in grams: its creature's base weight plus everything it carries. The
/// rules know every actor's weight; the player is shown only their own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Weight {
    /// The creature itself.
    pub base: u64,
    /// Everything it carries.
    pub carried: u64,
}

impl Weight {
    pub fn total(self) -> u64 {
        self.base + self.carried
    }
}

/// Grams for people: `450 g` below a kilogram, else kilograms to one decimal (`1.3 kg`, `70 kg`).
pub fn format_weight(grams: u64) -> String {
    if grams < 1000 {
        return format!("{grams} g");
    }
    let tenths = (grams + 50) / 100;
    match tenths % 10 {
        0 => format!("{} kg", tenths / 10),
        d => format!("{}.{d} kg", tenths / 10),
    }
}

/// Stacks in a sentence: "a dagger", "a dagger and 12 arrows", "a dagger, 12 arrows, and a
/// ruby"; more than three stacks are counted ("17 objects").
pub fn phrase(stacks: &[Stack], content: &Content) -> String {
    phrase_with(stacks, content, |_| None)
}

/// [`phrase`] with what `tag` says of the stack at each position after its name, in
/// parentheses: "a dagger (for sale, 45 gold) and 12 arrows (for sale, 18 gold)". More than
/// three stacks are counted, untagged.
pub fn phrase_with(
    stacks: &[Stack],
    content: &Content,
    tag: impl Fn(usize) -> Option<String>,
) -> String {
    let names: Vec<String> = (stacks.iter().enumerate())
        .map(|(i, s)| match tag(i) {
            Some(tag) => format!("{} ({tag})", s.describe(content)),
            None => s.describe(content),
        })
        .collect();
    match names.as_slice() {
        [] => "nothing".into(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [a, b, c] => format!("{a}, {b}, and {c}"),
        _ => {
            let count: u64 = stacks.iter().map(|s| u64::from(s.count)).sum();
            format!("{count} objects")
        }
    }
}

/// Joins `stack` to a stack of the same object in `stacks` if it stacks; returns whether it did.
fn join(stacks: &mut [Stack], stack: Stack, content: &Content) -> bool {
    if !content.object(stack.object).stackable {
        return false;
    }
    match stacks.iter_mut().find(|s| s.object == stack.object) {
        Some(same) => {
            same.count += stack.count;
            true
        }
        None => false,
    }
}

/// Puts `stacks` on top of `pile`, keeping their order (the first ends on top). Stackable
/// objects join a stack of the same object already there.
pub fn add_to_pile(pile: &mut Vec<Stack>, stacks: &[Stack], content: &Content) {
    for stack in stacks.iter().rev() {
        if !join(pile, *stack, content) {
            pile.insert(0, *stack);
        }
    }
}

/// Adds `stack` to `inventory`: joins a stack of the same object if it stacks, else takes the
/// next letter. Returns false (and changes nothing) when every letter is taken.
pub fn add_to_inventory(inventory: &mut Vec<Stack>, stack: Stack, content: &Content) -> bool {
    if join(inventory, stack, content) {
        return true;
    }
    if inventory.len() >= INVENTORY_SLOTS {
        return false;
    }
    inventory.push(stack);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        Content::builtin().expect("built-in content")
    }

    fn stack(content: &Content, id: &str, count: u32) -> Stack {
        let object = content.object_id(id).unwrap_or_else(|| panic!("{id}"));
        Stack { object, count }
    }

    #[test]
    fn weights_read_like_a_person_would_say_them() {
        assert_eq!(format_weight(0), "0 g");
        assert_eq!(format_weight(450), "450 g");
        assert_eq!(format_weight(999), "999 g");
        assert_eq!(format_weight(1000), "1 kg");
        assert_eq!(format_weight(1250), "1.3 kg");
        assert_eq!(format_weight(1949), "1.9 kg");
        assert_eq!(format_weight(70_000), "70 kg");
        assert_eq!(format_weight(71_960), "72 kg");
    }

    #[test]
    fn stackable_objects_join_and_others_stay_apart() {
        let c = content();
        let mut pile = vec![stack(&c, "dagger", 1)];
        add_to_pile(
            &mut pile,
            &[stack(&c, "arrow", 5), stack(&c, "dagger", 1)],
            &c,
        );
        assert_eq!(
            pile,
            [
                stack(&c, "arrow", 5),
                stack(&c, "dagger", 1),
                stack(&c, "dagger", 1)
            ]
        );
        // More arrows join the stack where it lies; the top stays the top.
        add_to_pile(&mut pile, &[stack(&c, "arrow", 7)], &c);
        assert_eq!(pile.len(), 3);
        assert_eq!(pile[0], stack(&c, "arrow", 12));

        let mut inventory = Vec::new();
        assert!(add_to_inventory(
            &mut inventory,
            stack(&c, "gold_coin", 3),
            &c
        ));
        assert!(add_to_inventory(
            &mut inventory,
            stack(&c, "gold_coin", 4),
            &c
        ));
        assert!(add_to_inventory(&mut inventory, stack(&c, "dagger", 1), &c));
        assert!(add_to_inventory(&mut inventory, stack(&c, "dagger", 1), &c));
        assert_eq!(inventory.len(), 3);
        assert_eq!(inventory[0].count, 7);
    }

    #[test]
    fn an_inventory_has_a_letter_per_stack() {
        let c = content();
        let mut inventory = Vec::new();
        for _ in 0..INVENTORY_SLOTS {
            assert!(add_to_inventory(&mut inventory, stack(&c, "dagger", 1), &c));
        }
        assert!(!add_to_inventory(
            &mut inventory,
            stack(&c, "dagger", 1),
            &c
        ));
        assert_eq!(inventory.len(), INVENTORY_SLOTS);
        // A stackable object still fits when it joins a stack.
        inventory[0] = stack(&c, "arrow", 1);
        assert!(add_to_inventory(&mut inventory, stack(&c, "arrow", 3), &c));
        assert_eq!(inventory[0].count, 4);
    }

    #[test]
    fn weights_and_phrases() {
        let c = content();
        let dagger = stack(&c, "dagger", 1);
        let arrows = stack(&c, "arrow", 12);
        let each = u64::from(c.object(arrows.object).weight);
        assert_eq!(arrows.weight(&c), 12 * each);
        assert_eq!(
            weight_of(&[dagger, arrows], &c),
            dagger.weight(&c) + 12 * each
        );
        assert_eq!(phrase(&[dagger], &c), "a dagger");
        assert_eq!(phrase(&[dagger, arrows], &c), "a dagger and 12 arrows");
        let ruby = stack(&c, "ruby", 1);
        assert_eq!(
            phrase(&[dagger, arrows, ruby], &c),
            "a dagger, 12 arrows, and a ruby"
        );
        assert_eq!(phrase(&[dagger, arrows, ruby, dagger], &c), "15 objects");
        let w = Weight {
            base: 70_000,
            carried: 1_500,
        };
        assert_eq!(w.total(), 71_500);
    }
}
