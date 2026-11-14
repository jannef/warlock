//! Magic crystals in play (docs/design/enchanting.md §5, §10; docs/dev/enchanting.md; the
//! colours are `data/crystals.toml`, `content/crystal.rs`).
//!
//! - **Consuming** ([`World::consume_crystal`]): a crystal is consumed as food is eaten and a
//!   potion drunk, from the Consume list (`e`) through [`Action::Eat`] (`eat:b`, `eat:floor:a`;
//!   `world/eat.rs` hands a crystal here): one crystal of the stack, from the pack or the pile
//!   underfoot, a turn of the `drink` action, no check. It gives `[consume] mana` (never past
//!   the maximum, as a potion's mana does), `[consume] marks` Willpower marks
//!   (`StatUse::Consumed`, [`World::train_stat_times`]), and a step toward evil
//!   ([`Deed::ConsumedCrystal`], `[shift] consume_crystal` in `data/alignment.toml`,
//!   diminishing as every deed does). Never refused for a full pool, nor for want of hunger.
//!   In a shop it is bought as a potion drunk is ([`World::drank_goods`]). The rare materials
//!   are no crystals: the Consume list leaves them out and `eat` refuses them as any non-food.
//! - **Salvage's affix checks** ([`World::catch_magic`], called by `world/salvage.rs` before its
//!   material checks): a magic or rare item, identified or not (it is destroyed at once, so
//!   what its crystals tell matters to no one), makes one check per affix, in its order, two
//!   gameplay draws each: 2d12 + Willpower's modifier + the talents' Willpower checks +
//!   Enchanting's [`Effect::Craft`] against `data/enchanting.toml`'s
//!   [`EnchantingRules::salvage_against`]; a natural 2 fails, a natural 24 succeeds. Each check
//!   made is one crystal of the affix's colour ([`Crystals::colour_of_affix`]). Then a rare of
//!   item level `[rare] salvage_level` or more draws once for `salvage_chance` %, and a made
//!   draw picks one of the rare materials at even odds (a second draw). The lines name an
//!   affix only when the player knows the item's magic; an unidentified item's say "its
//!   magic". Everything is drawn before anything changes.
//! - **Soulglass** ([`World::crystallise`], docs/design/enchanting.md §6): a spell with
//!   `[spell.effect] crystal` marks the creature it strikes (`World::glassing`, set in
//!   `cast.rs::spell_attack` around its blow, whoever casts it, from a scroll or a wand too). If
//!   the blow kills it, `fight.rs::creature_dies` asks here before it lays a corpse: a kind that
//!   leaves a corpse, is no animated thing, and has a colour ([`Crystals::colour_of`]) leaves
//!   no corpse and no crafting stock (`material_drops.rs`), and in their place its colour's
//!   crystals, [`Crystals::yield_at`] its type's level (no draw), and a rare material: a
//!   `dragon`'s always `[rare] dragon`, else from `[rare] level` on one `chance` % draw and,
//!   made, an even pick of the two (a second draw), from the gameplay stream. What it carried
//!   falls as any dead creature's; a summoned or risen one leaves nothing, as ever.
//!
//! [`Action::Eat`]: crate::action::Action::Eat
//! [`Crystals::colour_of`]: crate::content::Crystals::colour_of
//! [`Crystals::yield_at`]: crate::content::Crystals::yield_at
//! [`EnchantingRules::salvage_against`]: crate::content::EnchantingRules::salvage_against
//! [`Crystals::colour_of_affix`]: crate::content::Crystals::colour_of_affix

use super::alignment::Deed;
use super::{Actor, Spent, World};
use crate::action::{Meal, Positions};
use crate::combat;
use crate::content::{ActionKind, AffixKind, StatUse};
use crate::object::Stack;
use crate::skills::Effect;
use crate::stats::Stat;
use crate::text::capitalized;

/// What salvage's affix checks caught: the lines to say, in order, the stacks caught, and
/// whether any check was made (Enchanting's mark).
#[derive(Debug, Default)]
pub(super) struct Caught {
    pub lines: Vec<String>,
    pub stacks: Vec<Stack>,
    pub checked: bool,
}

impl World {
    /// The player consumes one crystal of `stack`, found where `meal` says (module docs);
    /// [`World::eat`] hands it here. Nothing for content without crystals or a stack that is
    /// no crystal (the caller checks).
    pub(super) fn consume_crystal(&mut self, meal: Meal, stack: &Stack) -> Spent {
        let content = self.content.clone();
        let Some(crystals) = content.crystals() else {
            return Spent::Nothing;
        };
        let Some(colour) = crystals.colour_of_object(stack.object) else {
            return Spent::Nothing;
        };
        let name = colour.name.clone();
        let spent = self.price_for(&[ActionKind::Drink]);
        let one = self.take_crystal(meal);
        self.say(format!(
            "You crush the {name} crystal and drink what is in it. Something of it stays."
        ));
        self.drank_goods(meal, &one);
        let me = self.player_mut();
        let max = me.max_mana(&content);
        me.mana = me
            .mana
            .saturating_add(crystals.consume_mana)
            .min(max)
            .max(me.mana);
        self.train_stat_times(StatUse::Consumed, None, crystals.consume_marks);
        self.player_did(Deed::ConsumedCrystal);
        spent
    }

    /// Takes one crystal from where `meal` says: from the pack (the last empties its stack, and
    /// what was worn follows its letters) or the pile underfoot.
    fn take_crystal(&mut self, meal: Meal) -> Stack {
        match meal {
            Meal::Pack(i) => {
                let player = self.player_mut();
                let one = player.inventory[i].single();
                player.inventory[i].count -= 1;
                if player.inventory[i].count == 0 {
                    player.inventory.remove(i);
                    player.equipment.after_removal(&Positions::of([i]));
                }
                one
            }
            Meal::Floor(i) => {
                let here = self.player_pos();
                let level = self.state.current_mut();
                let mut pile = level.objects_at(here).to_vec();
                let one = pile[i].single();
                pile[i].count -= 1;
                if pile[i].count == 0 {
                    pile.remove(i);
                }
                level.set_pile(here, pile);
                one
            }
        }
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
        let me = self.state.player();
        let enchanting = content.skill_id("enchanting");
        let modifier = me.check_modifier(&content, Stat::Willpower)
            + enchanting.map_or(0, |s| me.skill_effect_of(&content, s, Effect::Craft));
        for affix in &magic.affixes {
            let Some(def) = content.affixes().get(&affix.id) else {
                continue;
            };
            let natural = combat::roll_natural(&mut self.state.rng);
            let made = crate::check::makes(natural, modifier, rules.salvage_against(affix));
            caught.checked = true;
            // The affix by name only where the player knows the item's magic.
            let named = match def.kind {
                _ if !magic.identified => "some of its magic".to_string(),
                AffixKind::Prefix => format!("its {} magic", def.name),
                AffixKind::Suffix => format!("its magic {}", def.name),
            };
            let colour = crystals.colour_of_affix(&content, &affix.id);
            match colour.filter(|_| made) {
                Some(colour) => {
                    let a = crate::text::article(&colour.name);
                    let name = &colour.name;
                    caught
                        .lines
                        .push(format!("You catch {named} in {a} {name} crystal."));
                    let crystal = Stack::new(colour.object, 1);
                    match caught.stacks.iter_mut().find(|s| s.object == colour.object) {
                        Some(s) => s.count += 1,
                        None => caught.stacks.push(crystal),
                    }
                }
                None => (caught.lines).push(format!("{} goes out unread.", capitalized(&named))),
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

    /// What `actor`, a creature just dead, leaves in place of its body if a Soulglass blow
    /// killed it (module docs): its colour's crystals, perhaps a rare material, and the line
    /// saying so where the player sees its cell. `None` when no `crystal` spell struck it, or
    /// its kind leaves no corpse, an everlasting one (it always leaves its corpse), is an
    /// animated thing, or has no colour: it dies as it would.
    pub(super) fn crystallise(&mut self, actor: &Actor) -> Option<(Vec<Stack>, Option<String>)> {
        if self.glassing != Some(actor.id) {
            return None;
        }
        let content = self.content.clone();
        let crystals = content.crystals()?;
        let def = content.creature(actor.creature);
        if content.corpse_of(actor.creature).is_none() || def.everlasting || def.animated.is_some()
        {
            return None;
        }
        let colour = crystals.colour_of(&content, actor.creature)?;
        let mut left = vec![Stack::new(colour.object, crystals.yield_at(def.level))];
        // The rare material: a dragon's always, else one chance draw from `level` on and an
        // even pick between the two (a second draw).
        let rare = &crystals.rare;
        if def.tags.iter().any(|t| t == "dragon") {
            left.push(Stack::new(rare.dragon, 1));
        } else if def.level >= rare.level && self.state.rng.chance(u32::from(rare.chance)) {
            let pick = self.state.rng.index(rare.materials.len());
            left.push(Stack::new(rare.materials[pick], 1));
        }
        let seen =
            actor.level == self.state.current_level && self.perception.sees_actors_at(actor.pos);
        let line = seen.then(|| {
            let verb = if left.len() == 1 && left[0].count == 1 {
                "lies"
            } else {
                "lie"
            };
            let what = crate::object::phrase(&left, self.lore());
            format!("Its body turns to glass: where it fell {verb} {what}.")
        });
        Some((left, line))
    }
}

#[cfg(test)]
#[path = "crystal_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "soulglass_tests.rs"]
mod soulglass_tests;
