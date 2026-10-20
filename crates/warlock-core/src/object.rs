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
//!
//! A magic or rare item (spec §8 "Magic items", [`crate::magic`]) is a stack that carries what it
//! rolled ([`Magic`]); it joins only a stack with the very same magic (arrows shot from a magic
//! stack and picked up again). What the player is shown of it goes through
//! [`Stack::describe`] and [`Stack::known_magic`], which keep an unidentified item's affixes
//! hidden.
//!
//! Food (spec §8 "Hunger") remembers when it was made ([`Stack::died`]; a corpse, a stack of one,
//! when its creature died): it rots from then on, and is gone in the end ([`crate::rot`]). Food
//! joins a stack of the same food whatever its age, and the stack keeps the oldest.

use crate::content::{Content, ObjectId, Rarity};
use crate::magic::{Effect, Magic, Mods, name};
use crate::time::GameTime;
use serde::{Deserialize, Serialize};

/// Most stacks an inventory holds: one per letter, `a` to `z`.
pub const INVENTORY_SLOTS: usize = 26;

/// Some number of one object, and what it rolled if it is magic.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Stack {
    pub object: ObjectId,
    /// At least 1; always 1 for objects that do not stack.
    pub count: u32,
    /// What a magic or rare item rolled (boxed: most stacks have none). Ground truth: the rules
    /// read it, presentation only through [`Stack::known_magic`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    magic: Option<Box<Magic>>,
    /// Food: when it was made (a corpse: when its creature died), the moment it rots from (rules
    /// and dev tools only: food does not show its age, so presentation cannot read it). `None`
    /// for everything else, and for food in content that has not come into the world yet
    /// ([`crate::rot::made_at`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) died: Option<GameTime>,
}

impl Stack {
    /// `count` of a normal `object`; food made by it does not know its age yet.
    pub fn new(object: ObjectId, count: u32) -> Self {
        Self {
            object,
            count,
            magic: None,
            died: None,
        }
    }

    pub fn one(object: ObjectId) -> Self {
        Self::new(object, 1)
    }

    /// A corpse, `object`, of a creature that died at `died`.
    pub fn corpse(object: ObjectId, died: GameTime) -> Self {
        Self {
            died: Some(died),
            ..Self::new(object, 1)
        }
    }

    /// `count` of `object` that rolled `magic`.
    pub(crate) fn with_magic(object: ObjectId, count: u32, magic: Magic) -> Self {
        Self {
            magic: Some(Box::new(magic)),
            ..Self::new(object, count)
        }
    }

    /// One of this stack: the same object and magic.
    pub(crate) fn single(&self) -> Self {
        Self {
            count: 1,
            ..self.clone()
        }
    }

    /// What it rolled, identified or not: ground truth, for the rules and dev tools.
    pub(crate) fn magic(&self) -> Option<&Magic> {
        self.magic.as_deref()
    }

    /// Normal, magic, or rare: this the player sees at once.
    pub fn rarity(&self) -> Rarity {
        self.magic.as_ref().map_or(Rarity::Normal, |m| m.rarity)
    }

    /// Whether the player knows what it does: a normal item always, a magic or rare one once
    /// identified.
    pub fn is_identified(&self) -> bool {
        self.magic.as_ref().is_none_or(|m| m.identified)
    }

    /// What it rolled, if the player has identified it: all presentation may know of its magic.
    pub fn known_magic(&self) -> Option<&Magic> {
        self.magic().filter(|m| m.identified)
    }

    /// Identifies it; returns whether there was anything to learn.
    pub(crate) fn identify(&mut self) -> bool {
        match self.magic.as_mut() {
            Some(m) if !m.identified => {
                m.identified = true;
                true
            }
            _ => false,
        }
    }

    /// Grams: its object's weight, less what a weight affix takes off, times the count.
    pub fn weight(&self, content: &Content) -> u64 {
        let each = u64::from(content.object(self.object).weight);
        let lighter = match self.magic {
            Some(_) => Mods::local(self, content).get(Effect::Weight),
            None => 0,
        };
        crate::magic::mods::less(each, lighter).max(1) * u64::from(self.count)
    }

    /// As the player knows it: "a dagger", "12 arrows", "a Sharp long sword of the Bear", "Grim
    /// Bite, a long sword", unidentified "a long sword (magic)".
    pub fn describe(&self, content: &Content) -> String {
        let def = content.object(self.object);
        let known = self.is_identified();
        name::describe(def, self.count, self.magic(), known, content)
    }

    /// What it is called as the player knows it, without an article or count: "long sword",
    /// "Sharp long sword of the Bear", "Grim Bite".
    pub fn title(&self, content: &Content) -> String {
        let def = content.object(self.object);
        name::title(def, self.magic(), self.is_identified(), content)
    }

    /// As it truly is, identified or not: for dev tools only.
    pub(crate) fn describe_truth(&self, content: &Content) -> String {
        let def = content.object(self.object);
        let text = name::describe(def, self.count, self.magic(), true, content);
        match self.magic() {
            Some(m) if !m.identified => format!("{text} (unidentified)"),
            _ => text,
        }
    }

    /// Whether the object exists in `content`, the count makes sense, its magic is valid for it,
    /// and it knows when it was made if and only if it is food (a guard for saves).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        usize::from(self.object.0) < content.object_defs().len()
            && self.count >= 1
            && (self.count == 1 || content.object(self.object).stackable)
            && (self.magic()).is_none_or(|m| m.check(content.object(self.object), content).is_ok())
            && self.died.is_some() == content.object(self.object).food.is_some()
    }
}

/// The age a stack of food keeps when two join: the oldest (a stack that does not know its age
/// takes the other's).
fn oldest(a: Option<GameTime>, b: Option<GameTime>) -> Option<GameTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
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

/// Joins `stack` to a stack of the same object (and the same magic, if any) in `stacks` if it
/// stacks; returns whether it did. Food joins whatever its age, and keeps the oldest.
fn join(stacks: &mut [Stack], stack: &Stack, content: &Content) -> bool {
    if !content.object(stack.object).stackable {
        return false;
    }
    match (stacks.iter_mut()).find(|s| s.object == stack.object && s.magic == stack.magic) {
        Some(same) => {
            same.count += stack.count;
            same.died = oldest(same.died, stack.died);
            true
        }
        None => false,
    }
}

/// Puts `stacks` on top of `pile`, keeping their order (the first ends on top). Stackable
/// objects join a stack of the same object already there.
pub fn add_to_pile(pile: &mut Vec<Stack>, stacks: &[Stack], content: &Content) {
    for stack in stacks.iter().rev() {
        if !join(pile, stack, content) {
            pile.insert(0, stack.clone());
        }
    }
}

/// Adds `stack` to `inventory`: joins a stack of the same object if it stacks, else takes the
/// next letter. Returns false (and changes nothing) when every letter is taken.
pub fn add_to_inventory(inventory: &mut Vec<Stack>, stack: Stack, content: &Content) -> bool {
    if join(inventory, &stack, content) {
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
        Stack::new(object, count)
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
    fn food_joins_whatever_its_age_and_keeps_the_oldest() {
        let c = content();
        let made = |count, hour: u64| Stack {
            died: Some(GameTime::from_seconds(hour * 3600)),
            ..stack(&c, "bread", count)
        };
        let mut inventory = vec![made(2, 10)];
        assert!(add_to_inventory(&mut inventory, made(1, 4), &c));
        assert!(add_to_inventory(&mut inventory, made(3, 20), &c));
        assert_eq!(inventory, [made(6, 4)], "one stack, as old as the oldest");
        let mut pile = vec![made(1, 30)];
        add_to_pile(&mut pile, &[made(2, 12)], &c);
        assert_eq!(pile, [made(3, 12)]);
        // A loaf that does not know its age yet takes the stack's.
        add_to_pile(&mut pile, &[stack(&c, "bread", 1)], &c);
        assert_eq!(pile, [made(4, 12)]);
        // Corpses never join: each keeps the moment its creature died.
        let rat = c.object_id("rat_corpse").expect("a rat corpse");
        let hour = |h: u64| GameTime::from_seconds(h * 3600);
        let mut pile = vec![Stack::corpse(rat, hour(1))];
        add_to_pile(&mut pile, &[Stack::corpse(rat, hour(2))], &c);
        assert_eq!(pile.len(), 2);
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
        let (d, a) = (|| dagger.clone(), || arrows.clone());
        assert_eq!(weight_of(&[d(), a()], &c), dagger.weight(&c) + 12 * each);
        assert_eq!(phrase(&[d()], &c), "a dagger");
        assert_eq!(phrase(&[d(), a()], &c), "a dagger and 12 arrows");
        let ruby = stack(&c, "ruby", 1);
        assert_eq!(
            phrase(&[d(), a(), ruby.clone()], &c),
            "a dagger, 12 arrows, and a ruby"
        );
        assert_eq!(phrase(&[d(), a(), ruby, d()], &c), "15 objects");
        let w = Weight {
            base: 70_000,
            carried: 1_500,
        };
        assert_eq!(w.total(), 71_500);
    }
}
