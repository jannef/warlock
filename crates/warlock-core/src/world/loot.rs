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
//! level's. What it drops is its gear: its material is drawn by the materials' `gear` at the
//! creature's level (#193: primitive for kobolds, steel only for tougher creatures).

use super::{Actor, ActorId, Spent, World};
use crate::action::Positions;
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
    /// Identifies the player's inventory stack `item` (spec §8 "Magic items"): its affixes are
    /// known from now on, wherever it goes. Says "This is a Sharp long sword of the Bear!" when
    /// there was something to learn. Takes no time of its own: the action that calls it (reading
    /// the scroll of identify, #150) is what takes time.
    pub(crate) fn identify(&mut self, item: usize) -> Identified {
        let content = self.content.clone();
        let Some(stack) = self.player_mut().inventory.get_mut(item) else {
            return Identified::NoSuchItem;
        };
        if !stack.identify() {
            return Identified::Known;
        }
        let name = stack.describe(&content);
        self.say(format!("This is {name}!"));
        Identified::Now(name)
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
    /// level's item level, of a material by the creature's level (its gear), from its own
    /// stream.
    pub(super) fn loot_of(&self, actor: &Actor) -> Option<Stack> {
        let chance = content_loot(self, actor)?;
        let mut rng = Rng::stream(self.state.seed, &format!("drop/{}", actor.id.0));
        if !rng.chance(u32::from(chance)) {
            return None;
        }
        let level = self.content.item_level(&actor.level);
        let gear = Source::Gear(self.content.creature(actor.creature).level);
        // Treasure hunter (spec §8 "Talents"), Hoard sense (spec §8 "Birth sign"): the player's
        // luck, whoever made the kill.
        let talents = self.state.player().talent_mods(&self.content);
        let luck = loot::Luck::of(&talents);
        // Jeweller (spec §8 "Class abilities"): a gem a kill drops is cut finer.
        let gem = gem::find_with(&self.content, &mut rng, level, luck.gems).map(|mut stack| {
            gem::cut_finer(&self.content, &mut stack, talents.gem_cut);
            stack
        });
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

/// The chance in percent that `actor`'s creature type drops loot, if it has one above 0.
fn content_loot(world: &World, actor: &Actor) -> Option<u8> {
    let chance = world.content.creature(actor.creature).loot;
    (chance > 0).then_some(chance)
}
