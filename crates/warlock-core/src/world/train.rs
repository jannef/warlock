//! Training (spec §8 "Skills"): meaningful uses of the player's skills earn training marks,
//! which gather until a skill roll spends them (`world/skill_roll.rs`): a level-up's, or the
//! automatic one when they reach `[rolls] auto_at`. The more marks, the bigger the roll's die.
//!
//! A use ([`Use`]) is reported where it happens: a step within earshot of a creature
//! (`make_noise`), a sound that is news (`hear`), a dodged attack (`fight.rs`), an action under a
//! load that slows the player (`apply`), a journey on the overworld (`player_moves`). Each of the
//! player's trained skills with a mark rule for that use then gets a mark with the rule's chance,
//! drawn from the training stream ([`crate::rng::Rng::training`]); untrained skills get none.
//! Nothing is said: the player senses the marks only as the Skills screen's meter. A special
//! room may make a skill's marks come more often while the player is inside, or more than one
//! at a time (a scriptorium's two for Decipher script, `train` in `data/room_effects.toml`).
//!
//! Only the player trains for now; creatures keep the skills their type gives them.

use super::{ActorId, World};
use crate::combat::{Blow, Category, Strike};
use crate::content::StatUse;
use crate::skills::{SkillId, Use};

impl World {
    /// `blow`, a plain miss at the player (`Missed { by: None }`) made a dodge one time in
    /// four, on the training stream (`docs/design/skill-uses.md` Tu1): everything after it sees
    /// one dodge. Blows between creatures, and at creatures, draw nothing.
    pub(super) fn plain_miss_dodged(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        blow: Option<Blow>,
    ) -> Option<Blow> {
        let plain = blow.is_some_and(|b| matches!(b.strike, Strike::Missed { by: None }));
        if plain
            && target == self.state.player
            && attacker != target
            && self.state.training.chance(25)
        {
            return Some(Blow::missed(Some(Category::Dodge)));
        }
        blow
    }

    /// Whether any of the player's trained skills earns marks on `used`: when none does, a use
    /// need not even be looked for.
    pub(super) fn trains(&self, used: Use) -> bool {
        let content = &self.content;
        (self.state.player().skills.iter())
            .any(|(id, _)| content.skill(id).marks.iter().any(|m| m.on == used))
            || StatUse::of_skill(used).is_some_and(|s| self.trains_stat(s))
    }

    /// The player made a meaningful use `used` of their skills: each trained skill with a mark
    /// rule for it gets a mark with the rule's chance (one draw of the training stream per rule),
    /// scaled by the special rooms where they stand (a scriptorium's 200 for Decipher script
    /// gives two marks; `Here::train`).
    pub(super) fn train(&mut self, used: Use) {
        self.train_some(used, None);
    }

    /// [`World::train`] for skill `only` alone: a use that practises one skill of several that
    /// mark it (a salvage's check practises the profession that worked the material,
    /// `world/salvage.rs`).
    pub(super) fn train_skill(&mut self, used: Use, only: SkillId) {
        self.train_some(used, Some(only));
    }

    /// [`World::train`], for every trained skill or for `only`.
    fn train_some(&mut self, used: Use, only: Option<SkillId>) {
        let content = self.content.clone();
        let (level, at) = (self.state.player().level.clone(), self.state.player().pos);
        let trained: Vec<_> = (self.state.player().skills.iter())
            .map(|(id, _)| id)
            .filter(|&id| only.is_none_or(|o| o == id))
            .collect();
        for id in trained {
            let def = content.skill(id);
            let room = self.effects_at(&level, at).train(id);
            for rule in def.marks.iter().filter(|m| m.on == used) {
                // Each whole hundred is a mark for certain and the rest a chance of one more
                // (a scriptorium's 200 for Decipher script: two marks per check).
                let mut chance = u32::from(rule.chance) * room / 100;
                while chance > 0 {
                    let step = chance.min(100);
                    chance -= step;
                    if !self.state.training.chance(step) {
                        continue;
                    }
                    self.mark_skill(id, 1);
                }
            }
        }
        if let Some(stat) = StatUse::of_skill(used) {
            self.train_stat(stat);
        }
    }
}
