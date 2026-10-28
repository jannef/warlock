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

use super::World;
use super::loot::Identified;
use crate::rng::Rng;
use crate::skills::{Effect, Use};
use crate::time::{GameTime, HOUR};

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
        if !(self.state.player().skills).trains_effect(&content, Effect::Identify) {
            return;
        }
        for hour in first..=last {
            let unknown = self.unidentified();
            if unknown.is_empty() {
                return;
            }
            let chance = (self.state.player())
                .skill_effect(&content, Effect::Identify)
                .clamp(0, 100) as u32;
            let mut rng = Rng::stream(self.state.seed, &format!("appraise/{hour}"));
            if rng.chance(chance) {
                self.appraise(unknown[rng.index(unknown.len())]);
            }
        }
    }

    /// The player tells inventory stack `item` for what it is by looking it over: "You realise
    /// your gold ring is a gold ring of the Bear."
    fn appraise(&mut self, item: usize) {
        let Some(stack) = self.state.player().inventory.get(item) else {
            return;
        };
        let before = self.lore().describe(stack);
        let many = stack.count > 1;
        if let Identified::Now(after) = self.learn_item(item) {
            let is = if many { "are" } else { "is" };
            self.say(format!("You realise your {} {is} {after}.", bare(&before)));
            self.train(Use::Appraise);
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
