//! Magic crystals in play (docs/design/enchanting.md §5, §10; docs/dev/enchanting.md; the
//! colours are `data/crystals.toml`, `content/crystal.rs`).
//!
//! - **Consuming** ([`World::consume_crystal`], [`Action::Consume`], the Consume crystal ability
//!   of the abilities menu's Items group, everyone's): one crystal of the stack at a pack
//!   position, a turn of the `drink` action, no check. It gives `[consume] mana` (never past the
//!   maximum, as a potion's mana does), one Willpower mark (`StatUse::Consumed`), and a step
//!   toward evil ([`Deed::ConsumedCrystal`], `[shift] consume_crystal` in
//!   `data/alignment.toml`, diminishing as every deed does). Never refused for a full pool.
//!   Refused, taking no time: no such stack, a stack that is no crystal (the rare materials are
//!   not), unpaid shop goods.
//! - **Salvage's affix checks** ([`World::catch_magic`], called by `world/salvage.rs` before its
//!   material checks): an identified magic or rare item makes one check per affix, in its
//!   order, two gameplay draws each: 2d12 + Willpower's modifier + the talents' Willpower
//!   checks + Enchanting's [`Effect::Craft`] against `data/enchanting.toml`'s
//!   [`EnchantingRules::salvage_against`]; a natural 2 fails, a natural 24 succeeds. Each check
//!   made is one crystal of the affix's colour ([`Crystals::colour_of_affix`]). Then a rare of
//!   item level `[rare] salvage_level` or more draws once for `salvage_chance` %, and a made
//!   draw picks one of the rare materials at even odds (a second draw). An unidentified item
//!   draws nothing and gives nothing ("Its magic goes out unread."): a crystal's colour would
//!   tell its families. Everything is drawn before anything changes.
//!
//! [`Action::Consume`]: crate::action::Action::Consume
//! [`EnchantingRules::salvage_against`]: crate::content::EnchantingRules::salvage_against
//! [`Crystals::colour_of_affix`]: crate::content::Crystals::colour_of_affix

use super::alignment::Deed;
use super::{Spent, World};
use crate::action::Positions;
use crate::combat;
use crate::content::{ActionKind, AffixKind, StatUse};
use crate::object::Stack;
use crate::skills::Effect;
use crate::stats::Stat;

/// What salvage's affix checks caught: the lines to say, in order, the stacks caught, and
/// whether any check was made (Enchanting's mark).
#[derive(Debug, Default)]
pub(super) struct Caught {
    pub lines: Vec<String>,
    pub stacks: Vec<Stack>,
    pub checked: bool,
}

impl World {
    /// `consume:LETTER`: the player consumes one crystal of the stack at `item` (module docs).
    pub(super) fn consume_crystal(&mut self, item: usize) -> Spent {
        let content = self.content.clone();
        let Some(crystals) = content.crystals() else {
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let Some(colour) = crystals.colour_of_object(stack.object) else {
            self.say("That is no magic crystal.");
            return Spent::Nothing;
        };
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let name = colour.name.clone();
        let spent = self.price_for(&[ActionKind::Drink]);
        let inventory = self.state.inventory_mut();
        inventory[item].count -= 1;
        if inventory[item].count == 0 {
            inventory.remove(item);
            (self.player_mut().equipment).after_removal(&Positions::of([item]));
        }
        self.say(format!(
            "You crush the {name} crystal and drink what is in it. Something of it stays."
        ));
        let me = self.player_mut();
        let max = me.max_mana(&content);
        me.mana = me
            .mana
            .saturating_add(crystals.consume_mana)
            .min(max)
            .max(me.mana);
        self.train_stat(StatUse::Consumed);
        self.player_did(Deed::ConsumedCrystal);
        spent
    }

    /// Salvage's affix checks on `stack`, the one item being taken apart (module docs): drawn
    /// now, before the material checks; nothing changes but the stream.
    pub(super) fn catch_magic(&mut self, stack: &Stack) -> Caught {
        let content = self.content.clone();
        let mut caught = Caught::default();
        let (Some(crystals), Some(rules)) = (content.crystals(), content.enchanting()) else {
            return caught;
        };
        let Some(magic) = stack.magic() else {
            return caught;
        };
        if !magic.identified {
            caught.lines.push("Its magic goes out unread.".into());
            return caught;
        }
        let me = self.state.player();
        let enchanting = content.skill_id("enchanting");
        let modifier = me.effective_stats(&content).modifier(Stat::Willpower)
            + me.talent_mods(&content).check(Stat::Willpower)
            + enchanting.map_or(0, |s| me.skill_effect_of(&content, s, Effect::Craft));
        for affix in &magic.affixes {
            let Some(def) = content.affixes().get(&affix.id) else {
                continue;
            };
            let natural = combat::roll_natural(&mut self.state.rng);
            let made = crate::check::makes(natural, modifier, rules.salvage_against(affix));
            caught.checked = true;
            let named = match def.kind {
                AffixKind::Prefix => format!("{} magic", def.name),
                AffixKind::Suffix => format!("magic {}", def.name),
            };
            let colour = crystals.colour_of_affix(&content, &affix.id);
            match colour.filter(|_| made) {
                Some(colour) => {
                    let a = crate::content::article(&colour.name);
                    let name = &colour.name;
                    caught
                        .lines
                        .push(format!("You catch its {named} in {a} {name} crystal."));
                    let crystal = Stack::new(colour.object, 1);
                    match caught.stacks.iter_mut().find(|s| s.object == colour.object) {
                        Some(s) => s.count += 1,
                        None => caught.stacks.push(crystal),
                    }
                }
                None => caught.lines.push(format!("Its {named} goes out unread.")),
            }
        }
        let rare = &crystals.rare;
        if magic.rarity == crate::content::Rarity::Rare
            && magic.level >= rare.salvage_level
            && self.state.rng.chance(u32::from(rare.salvage_chance))
        {
            let pick = self.state.rng.index(rare.materials.len());
            let found = rare.materials[pick];
            let what = content.object(found).describe(1);
            caught
                .lines
                .push(format!("Where its magic was, {what} is left."));
            caught.stacks.push(Stack::new(found, 1));
        }
        caught
    }
}

#[cfg(test)]
#[path = "crystal_tests.rs"]
mod tests;
