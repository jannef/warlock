//! What a family hands down (spec §8 "Talents": the talents only for character creation, #697):
//! Family wealth's gold and gems and Father's sword's heirloom weapon, handed out once, as the
//! talent is taken ([`World::hand_down`], from `World::learn`). Their `creation` need keeps them
//! to the creation picks, so they come while the run is being made, on top of the starting kit.
//!
//! - **Gold** (`starting_gold`): `percent` of the gold coins the character carries then (the
//!   kit's and the scenario's), at least `at_least` and at most `at_most`, joining their coins.
//! - **Gems** (`starting_gems`): `min` to `max` loose gems at item level `level` ([`gem::roll`]),
//!   one draw for how many and then each gem's, from the stream
//!   `character/family-wealth/<race>,<gender>,<class>`.
//! - **The heirloom** (`heirloom`): the race's weapon (else the effect's `fallback`) with `min` to
//!   `max` affixes at item level `level` ([`loot::heirloom`]), one draw for how many and then the
//!   affixes', from the stream `character/fathers-sword/<race>,<gender>,<class>`; identified (it
//!   is the family's own), readied in its own slot (the main hand, every hand for a two-handed
//!   one, the missile weapon slot for a launcher) when that is free, else carried: the kit's own
//!   weapon stays where it is.
//!
//! Each has a stream of its own, so no other roll of the run moves.

use super::World;
use crate::content::{Content, GOLD, TalentEffect};
use crate::equipment;
use crate::gem;
use crate::item_parts::ItemPart;
use crate::magic::loot;
use crate::object::{self, Stack};
use crate::rng::Rng;

impl World {
    /// Hands out what `effects` (a rank just taken) give once: gold, gems, an heirloom.
    pub(super) fn hand_down(&mut self, effects: impl Iterator<Item = TalentEffect>) {
        let content = self.content.clone();
        let Some(spec) = self.state.character.as_ref().map(|c| c.spec.to_string()) else {
            return;
        };
        let seed = self.state.seed;
        for effect in effects {
            match effect {
                TalentEffect::StartingGold {
                    percent,
                    at_least,
                    at_most,
                } => self.inherit_gold(&content, percent, at_least, at_most),
                TalentEffect::StartingGems { min, max, level } => {
                    let mut rng = Rng::stream(seed, &format!("character/family-wealth/{spec}"));
                    let count = rng.range(i32::from(min), i32::from(max));
                    for _ in 0..count {
                        if let Some(stack) = loose_gem(&content, &mut rng, level) {
                            self.carry_new(&content, stack);
                        }
                    }
                }
                TalentEffect::Heirloom {
                    weapons,
                    fallback,
                    min,
                    max,
                    level,
                } => {
                    let race = self.state.character.as_ref().map(|c| c.spec.race.clone());
                    let weapon = (race.and_then(|r| weapons.get(&r).cloned())).unwrap_or(fallback);
                    let Some(object) = content.object_id(&weapon) else {
                        continue;
                    };
                    let mut rng = Rng::stream(seed, &format!("character/fathers-sword/{spec}"));
                    let count = rng.range(i32::from(min), i32::from(max)) as u8;
                    let mut stack = loot::heirloom(&content, &mut rng, object, count, level);
                    stack.identify();
                    self.wield_heirloom(&content, stack);
                }
                _ => {}
            }
        }
    }

    /// Family wealth's gold: `percent` of the coins carried, between `at_least` and `at_most`.
    fn inherit_gold(&mut self, content: &Content, percent: u32, at_least: u32, at_most: u32) {
        let Some(gold) = content.object_id(GOLD) else {
            return;
        };
        let carried: u64 = (self.state.player().inventory.iter())
            .filter(|s| s.object == gold)
            .map(|s| u64::from(s.count))
            .sum();
        let more =
            (carried * u64::from(percent) / 100).clamp(u64::from(at_least), u64::from(at_most));
        if more > 0 {
            self.carry_new(content, Stack::new(gold, more as u32));
        }
    }

    /// Puts `stack` in the player's pack (joining a stack of the same, else last).
    fn carry_new(&mut self, content: &Content, stack: Stack) {
        object::add_to_inventory(&mut self.player_mut().inventory, stack, content);
    }

    /// Puts the heirloom in the pack and readies it in its own slot (the main hand, the missile
    /// weapon slot for a launcher) if that is free.
    fn wield_heirloom(&mut self, content: &Content, stack: Stack) {
        let object = stack.object;
        let player = self.player_mut();
        player.inventory.push(stack);
        let item = player.inventory.len() - 1;
        let own = content.object(object).slot.as_ref();
        let Some((slot, _)) = content.slot_ids().find(|(_, s)| own == Some(&s.fits)) else {
            return;
        };
        let slots = equipment::slots_taken(content, object, slot);
        if slots.iter().all(|&s| player.equipment.get(s).is_none()) {
            player.equipment.put(&slots, item);
        }
    }
}

/// A loose gem found at item level `level`: a stack of one.
fn loose_gem(content: &Content, rng: &mut Rng, level: u8) -> Option<Stack> {
    let gems = content.gems()?;
    let (gem, cutting) = gem::roll(content, rng, level, false)?;
    Some(Stack::one(gems.object).with_parts(vec![ItemPart::gem(content, None, gem, cutting)]))
}
