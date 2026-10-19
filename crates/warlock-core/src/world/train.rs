//! Training (spec §8 "Skills"): meaningful uses of the player's skills earn training marks, and
//! enough marks raise a skill by a point (**[provisional]**: [`crate::skills::Increase`]).
//!
//! A use ([`Use`]) is reported where it happens: a step within earshot of a creature
//! (`make_noise`), a sound that is news (`hear`), a dodged attack (`fight.rs`), an action under a
//! load that slows the player (`apply`), a journey on the overworld (`player_moves`). Each of the
//! player's trained skills with a mark rule for that use then gets a mark with the rule's chance,
//! drawn from the training stream ([`crate::rng::Rng::training`]); untrained skills get none. A
//! point gained is said: "Your stealth improves." (never a number).
//!
//! Only the player trains for now; creatures keep the skills their type gives them.

use super::World;
use crate::skills::Use;

impl World {
    /// Whether any of the player's trained skills earns marks on `used`: when none does, a use
    /// need not even be looked for.
    pub(super) fn trains(&self, used: Use) -> bool {
        let content = &self.content;
        (self.state.player().skills.iter())
            .any(|(id, _)| content.skill(id).marks.iter().any(|m| m.on == used))
    }

    /// The player made a meaningful use `used` of their skills: each trained skill with a mark
    /// rule for it gets a mark with the rule's chance (one draw of the training stream per rule),
    /// and says so when that raises it a point.
    pub(super) fn train(&mut self, used: Use) {
        let content = self.content.clone();
        let player = self.state.player;
        let trained: Vec<_> = (self.state.player().skills.iter())
            .map(|(id, _)| id)
            .collect();
        let mut improved = Vec::new();
        for id in trained {
            let def = content.skill(id);
            for rule in def.marks.iter().filter(|m| m.on == used) {
                if !self.state.training.chance(u32::from(rule.chance)) {
                    continue;
                }
                let actor = self.state.actors.get_mut(&player).expect("the player");
                if actor.skills.mark(id, content.increase()) {
                    improved.push(def.name.to_lowercase());
                }
            }
        }
        for name in improved {
            self.say(format!("Your {name} improves."));
        }
    }
}
