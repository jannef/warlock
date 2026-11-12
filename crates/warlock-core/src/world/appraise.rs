//! Appraise (spec §8 "Skills"; #666): now and then a player trained in Appraise tells one
//! unidentified thing they carry or wear for what it is.
//!
//! The check comes on every whole game hour the clock passes ([`World::appraise_hours`], from
//! the status loop's `live`, so a rest or a dev skip rolls each hour it passes and the turn costs
//! nothing more). Each hour has its own stream, `appraise/<hour>` (the hour counted from the
//! calendar's start), drawn only when the player is trained in a skill with the `identify`
//! benefit and carries something they do not fully know (`World::unidentified`: what the scroll
//! of identify and an appraisal would tell): one draw for the chance (the benefit, in percent,
//! with what magic items and talents add to the skill), and on success one for which thing. No
//! state is kept and no other stream moves, so a run that never draws hashes as before. A thing
//! told this way may train the skill ([`Use::Appraise`]).
//!
//! Alchemy and Enchanting (#1343, professions §9) look the same way, each after Appraise and in
//! that order, from the same hour's stream: a skill with the `identify_potion` benefit at the
//! potions and flasks whose kind the player does not know, one with `identify_magic` at the
//! magic things whose magic they do not know (arms, armour, jewellery, wands, rods: anything
//! not known that is neither a potion nor a scroll). Each draws only when trained and with a
//! candidate: one draw for the chance, one for which. They mark nothing until their crafts come.

use super::World;
use super::loot::Identified;
use crate::rng::Rng;
use crate::skills::{Effect, Use};
use crate::time::{GameTime, HOUR};

/// The looks an hour may bring, in the order they draw (module docs).
const LOOKS: [Effect; 3] = [
    Effect::Identify,
    Effect::IdentifyPotion,
    Effect::IdentifyMagic,
];

impl World {
    /// `seconds` of game time from `start` pass for the player: each whole hour the clock passes
    /// may tell one of the things they do not know for what it is (see the module docs).
    pub(super) fn appraise_hours(&mut self, start: GameTime, seconds: u64) {
        let (first, last) = (
            start.seconds() / HOUR + 1,
            (start.seconds() + seconds) / HOUR,
        );
        if first > last || self.state.death.is_some() {
            return;
        }
        let content = self.content.clone();
        let skills = &self.state.player().skills;
        let looks: Vec<Effect> = (LOOKS.iter().copied())
            .filter(|&e| skills.trains_effect(&content, e))
            .collect();
        if looks.is_empty() {
            return;
        }
        for hour in first..=last {
            if self.unidentified().is_empty() {
                return;
            }
            let mut rng = Rng::stream(self.state.seed, &format!("appraise/{hour}"));
            for &look in &looks {
                let unknown = self.unknown_to(look);
                if unknown.is_empty() {
                    continue;
                }
                // Each look's own benefit, read where it is used (one hook each).
                let me = self.state.player();
                let chance = match look {
                    Effect::IdentifyPotion => me.skill_effect(&content, Effect::IdentifyPotion),
                    Effect::IdentifyMagic => me.skill_effect(&content, Effect::IdentifyMagic),
                    _ => me.skill_effect(&content, Effect::Identify),
                };
                let chance = chance.clamp(0, 100) as u32;
                if rng.chance(chance) {
                    self.appraise(unknown[rng.index(unknown.len())], look);
                }
            }
        }
    }

    /// The player's inventory stacks `look` looks at (module docs), in position order.
    fn unknown_to(&self, look: Effect) -> Vec<usize> {
        let mut unknown = self.unidentified();
        let inventory = &self.state.player().inventory;
        let potion = |i: &usize| self.content.object(inventory[*i].object).drink.is_some();
        let scroll = |i: &usize| self.content.object(inventory[*i].object).read.is_some();
        match look {
            Effect::IdentifyPotion => unknown.retain(potion),
            Effect::IdentifyMagic => unknown.retain(|i| !potion(i) && !scroll(i)),
            _ => {}
        }
        unknown
    }

    /// The player tells inventory stack `item` for what it is by `look`: "You realise your gold
    /// ring is a gold ring of the Bear." (Appraise, which marks it), "You recognise your fizzy
    /// red potion: a potion of healing." (Alchemy), "You read the magic bound in your gold ring:
    /// a gold ring of the Bear." (Enchanting).
    fn appraise(&mut self, item: usize, look: Effect) {
        let Some(stack) = self.state.player().inventory.get(item) else {
            return;
        };
        let before = self.lore().describe(stack);
        let many = stack.count > 1;
        let Identified::Now(after) = self.learn_item(item) else {
            return;
        };
        let before = bare(&before);
        match look {
            Effect::IdentifyPotion => self.say(format!("You recognise your {before}: {after}.")),
            Effect::IdentifyMagic => {
                self.say(format!(
                    "You read the magic bound in your {before}: {after}."
                ));
            }
            _ => {
                let is = if many { "are" } else { "is" };
                self.say(format!("You realise your {before} {is} {after}."));
                self.train(Use::Appraise);
            }
        }
    }
}

/// `name` without its leading article: "a gold ring" → "gold ring"; "2 potions" stays.
fn bare(name: &str) -> &str {
    ["a ", "an ", "the "]
        .iter()
        .find_map(|article| name.strip_prefix(article))
        .unwrap_or(name)
}
