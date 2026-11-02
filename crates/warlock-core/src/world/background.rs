//! A chosen character's background story, told as the run begins (spec §8 "Background story";
//! docs/dev/background.md): [`World::tell_background`], called by `World::start_run` after the
//! creation picks, so the story reads every choice (the Random picks too) and no pick's
//! prerequisite can be met by a gift of fate. It builds the [`Witness`] from the finished
//! character, tells the story ([`background::tell`]), keeps the ids in `Character::story`, and
//! applies each rolled effect once, one function per kind:
//!
//! - **stat**: `Stats::gift`, the score and its potential together, a score never below 1; with
//!   Hard-won, each point taken is stat training marks on the stat;
//! - **gold**: coins joining the pack, or taken from it (never below none);
//! - **item**: the kit's way (`Pack::put`, `Pack::equip`: a free slot, else carried); a magic one
//!   identified; a scroll or potion known, as the kit's are; food made now;
//! - **skill**: `Skills::add` (started at the roll, or raised);
//! - **weapon marks**: `WeaponSkills::add_marks`, never a rank (`best`: the weapon skill held
//!   highest);
//! - **alignment**: the starting score moved, the start's fourth term (never `shift_alignment`:
//!   nothing has happened yet);
//! - **talent** and **flaw**: `World::bestow` (`world/boons.rs`, the forgotten altar's Chosen
//!   and Forsaken with the id fixed): rank 1 with what it hands out, kept in `Actor::boons` so
//!   a restored run puts it back (the roll never draws a line giving what is held).
//!
//! Then hit points and mana to their maxima. A run that nobody chose (no kit) has no story.

use super::World;
use crate::action::Positions;
use crate::alignment::Score;
use crate::background::{self, Rolled, Witness};
use crate::content::{Content, GOLD, Keep, Marks, WeaponSkillId};
use crate::kit::Pack;
use crate::object::Stack;

impl World {
    /// Tells the player's background story and applies it (see the module docs).
    pub(super) fn tell_background(&mut self) {
        let content = self.content.clone();
        let Some(character) = self.state.character.clone() else {
            return;
        };
        if content.background().is_none() {
            return;
        }
        let player = self.state.player();
        let band = (content.alignment())
            .zip(player.alignment)
            .map(|(rules, score)| rules.band(score));
        let trained: Vec<WeaponSkillId> = (player.weapon_skills.trained())
            .filter(|(_, t)| t.rank > 0)
            .map(|(id, _)| id)
            .collect();
        let witness = Witness {
            character: &character,
            age: character.age_on(self.state.time),
            lucky: character.lucky(&content).is_some(),
            band,
            skills: &player.skills,
            talents: &player.talents,
            trained: &trained,
            campaign: &self.state.campaign,
        };
        let story = background::tell(&content, self.state.seed, &witness);
        for effect in story.effects {
            self.apply_story(&content, effect);
        }
        if let Some(c) = self.state.character.as_mut() {
            c.story = story.told;
        }
        let player = self.player_mut();
        player.hp = player.max_hp(&content);
        player.mana = player.max_mana(&content);
    }

    /// Applies one rolled effect of the story to the player.
    fn apply_story(&mut self, content: &Content, effect: Rolled) {
        match effect {
            Rolled::Stat(stat, by) => self.player_mut().stats.gift(stat, by),
            Rolled::Gold(by) => self.story_gold(content, by),
            Rolled::Item(stack, keep) => self.story_item(content, stack, keep),
            Rolled::Skill(id, by) => self.player_mut().skills.add(id, by),
            Rolled::WeaponMarks(to, count) => {
                let table = content.weapon_skills();
                let skills = &mut self.player_mut().weapon_skills;
                let id = match to {
                    Marks::Skill(id) => Some(id),
                    Marks::Best => best_weapon_skill(content, skills),
                };
                if let Some(id) = id {
                    skills.add_marks(table, id, count);
                }
            }
            Rolled::Alignment(by) => {
                let player = self.player_mut();
                if let Some(score) = player.alignment {
                    player.alignment = Some(Score::new(score.value() + by));
                }
            }
            Rolled::Talent(id, choice) => self.bestow(&id, choice.as_deref()),
            Rolled::Flaw(id) => self.bestow(&id, None),
        }
    }

    /// `by` gold coins into the pack, or (negative) out of it, as many as there are.
    fn story_gold(&mut self, content: &Content, by: i32) {
        let Some(gold) = content.object_id(GOLD) else {
            return;
        };
        let player = self.player_mut();
        if by > 0 {
            crate::object::add_to_inventory(
                &mut player.inventory,
                Stack::new(gold, by as u32),
                content,
            );
            return;
        }
        let mut left = by.unsigned_abs();
        for stack in player.inventory.iter_mut().rev() {
            if stack.object == gold && left > 0 {
                let taken = stack.count.min(left);
                stack.count -= taken;
                left -= taken;
            }
        }
        let gone =
            Positions::of((0..player.inventory.len()).filter(|&i| player.inventory[i].count == 0));
        player.inventory.retain(|s| s.count > 0);
        player.equipment.after_removal(&gone);
    }

    /// An item into the pack as a kit line puts it: worn or wielded in a free slot, else carried.
    fn story_item(&mut self, content: &Content, mut stack: Stack, keep: Keep) {
        let date = self.state.time;
        crate::rot::made_at(std::iter::once(&mut stack), content, date);
        let def = content.object(stack.object);
        if def.read.is_some() || def.drink.is_some() {
            self.state.known_scrolls.insert(stack.object);
        }
        let player = self.player_mut();
        let mut pack = Pack {
            items: std::mem::take(&mut player.inventory),
            equipment: std::mem::take(&mut player.equipment),
        };
        let at = pack.put(stack, content);
        if keep != Keep::Carry {
            pack.equip(at, content);
        }
        player.inventory = pack.items;
        player.equipment = pack.equipment;
    }
}

/// The weapon skill (not a style) held at the highest rank, ties by data order; `None` when no
/// weapon skill has a rank.
fn best_weapon_skill(
    content: &Content,
    skills: &crate::weapon_skills::WeaponSkills,
) -> Option<WeaponSkillId> {
    let table = content.weapon_skills();
    let mut best: Option<(WeaponSkillId, u8)> = None;
    for (id, t) in skills.trained() {
        if table.skill(id).is_style() || t.rank == 0 {
            continue;
        }
        if best.is_none_or(|(_, rank)| t.rank > rank) {
            best = Some((id, t.rank));
        }
    }
    best.map(|(id, _)| id)
}

#[cfg(test)]
#[path = "background_tests.rs"]
mod tests;
