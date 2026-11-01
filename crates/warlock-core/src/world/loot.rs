//! Magic items in play (spec §8 "Magic items"): identifying them, what creatures drop, and the
//! hit points an actor keeps when what it wears changes.
//!
//! **Identify** ([`World::identify`]): the core API the scroll of identify (#150) calls from its
//! action. It reveals an inventory stack's affixes ("This is a Sharp long sword of the Bear!").
//! Wearing or wielding an item for a while does not identify it; carrying one for a day does,
//! for a player with Lore ([`World::lore`], spec §8 "Talents").
//!
//! **Drops**: a creature type with `loot` (a chance in percent) drops a magic or rare item where
//! it dies, besides what it carried, or now and then a loose gem instead (by the gems' `chance`,
//! spec §7 "Gems"). The roll draws from the stream `drop/<actor id>` (spec §8 "Determinism"),
//! never the gameplay stream, so a kill changes no later roll of a fight; its item level is the
//! creature's level, at least the level's item level ([`drop_level`]: an out-of-depth creature
//! drops a deeper item). What it drops is its gear: its material is drawn by the materials'
//! `gear` at the creature's level (#193: primitive for kobolds, steel only for tougher
//! creatures).

use super::{Actor, ActorId, Spent, World};
use crate::action::Positions;
use crate::content::{Content, MAX_ITEM_LEVEL, ObjectId};
use crate::gem;
use crate::magic::{LootGrade, loot};
use crate::object::Stack;
use crate::rng::Rng;
use crate::variant::Source;

/// What came of [`World::identify`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identified {
    /// It is identified now: what it is called, as the player now knows it.
    Now(String),
    /// It was already known (or is a normal item, with nothing to learn).
    Known,
    /// The player carries no such stack.
    NoSuchItem,
}

impl World {
    /// Identifies the player's inventory stack `item` (spec §8 "Magic items", "Scrolls"): its
    /// affixes are known from now on, wherever it goes, and so is its type if it is an
    /// unidentified scroll's or wand's (every one of that type shows by its name from now on),
    /// and a wand's charges left (spec §8 "Magic items": "Wands"). Says
    /// "This is a Sharp long sword of the Bear!" when there was something to learn. Takes no
    /// time of its own: the action that calls it (reading the scroll of identify, #150) is what
    /// takes time.
    pub(crate) fn identify(&mut self, item: usize) -> Identified {
        let learned = self.learn_item(item);
        if let Identified::Now(name) = &learned {
            self.say(format!("This is {name}!"));
        }
        learned
    }

    /// [`World::identify`] without a word: the caller says it its own way (Appraise, spec §8
    /// "Skills").
    pub(crate) fn learn_item(&mut self, item: usize) -> Identified {
        let Some(object) = self.state.player().inventory.get(item).map(|s| s.object) else {
            return Identified::NoSuchItem;
        };
        let typed = self.learn_scroll(object);
        let content = self.content.clone();
        let stack = &mut self.player_mut().inventory[item];
        let charges = stack.identify_charges(&content);
        if !stack.identify() && !typed && !charges {
            return Identified::Known;
        }
        Identified::Now(self.lore().describe(&self.state.player().inventory[item]))
    }

    /// The player identifies scroll type `object` (spec §8 "Scrolls"): from now on every scroll
    /// of it shows by its name. Whether there was anything to learn (false for a type already
    /// known, or one without a label). Says nothing.
    pub(crate) fn learn_scroll(&mut self, object: ObjectId) -> bool {
        self.lore().label(object).is_some() && self.state.known_scrolls.insert(object)
    }

    /// Dev: identifies the inventory stacks `positions` names (everything with none).
    /// Takes no time; says what each turned out to be, or that there was nothing to learn.
    pub(super) fn identify_dev(&mut self, positions: &Positions) -> Spent {
        let len = self.state.player().inventory.len();
        let Some(chosen) = self.chosen(positions, len, "You carry no such thing.") else {
            return Spent::Nothing;
        };
        let learned = (chosen.iter())
            .filter(|&i| matches!(self.identify(i), Identified::Now(_)))
            .count();
        if learned == 0 {
            self.say("There is nothing to identify (dev).");
            return Spent::Nothing;
        }
        Spent::NoTime
    }

    /// What `actor`, a creature that just died, drops besides what it carried: by its type's
    /// `loot` chance, a loose gem (by the gems' chance) or else a magic or rare item at its
    /// [`drop_level`], of a material by the creature's level (its gear), from its own stream.
    pub(super) fn loot_of(&self, actor: &Actor) -> Option<Stack> {
        let chance = content_loot(self, actor)?;
        let mut rng = Rng::stream(self.state.seed, &format!("drop/{}", actor.id.0));
        if !rng.chance(u32::from(chance)) {
            return None;
        }
        let level = drop_level(&self.content, actor);
        let gear = Source::Gear(self.content.creature(actor.creature).level);
        // Treasure hunter (spec §8 "Talents"), Hoard sense (spec §8 "Birth sign"): the player's
        // luck, whoever made the kill.
        let talents = self.state.player().talent_mods(&self.content);
        let luck = loot::Luck::of(&talents);
        let gem = gem::find_with(&self.content, &mut rng, level, luck.gems);
        gem.or_else(|| {
            loot::roll_with(&self.content, &mut rng, level, LootGrade::Magic, gear, luck)
        })
    }

    /// Keeps actor `id`'s hit points within its maximum, which falls when it takes off something
    /// that raised it, or a status that raised it ends: its lasting maximum
    /// ([`Actor::lasting_max_hp`]), since what a status effect takes off it comes back, or its
    /// maximum now while a status still raises it above that.
    pub(super) fn keep_hp_within_max(&mut self, id: ActorId) {
        let content = self.content.clone();
        if let Some(actor) = self.state.actors.get_mut(&id) {
            let max = (actor.lasting_max_hp(&content)).max(actor.max_hp(&content));
            if actor.hp > max {
                actor.hp = max;
            }
        }
    }
}

/// The item level of what `actor`, a creature, drops (docs/design/affixes-systems.md §3): its
/// creature type's level, at least the item level of the level it dies on, at most
/// [`MAX_ITEM_LEVEL`]. Floor loot keeps the level's own item level.
pub(crate) fn drop_level(content: &Content, actor: &Actor) -> u8 {
    let floor = content.item_level(&actor.level);
    let creature = content.creature(actor.creature).level;
    floor.max(creature).clamp(1, MAX_ITEM_LEVEL)
}

/// The chance in percent that `actor`'s creature type drops loot, if it has one above 0.
fn content_loot(world: &World, actor: &Actor) -> Option<u8> {
    let chance = world.content.creature(actor.creature).loot;
    (chance > 0).then_some(chance)
}
