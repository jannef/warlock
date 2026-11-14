//! Concentration's hold on mana (spec §8 "Mana" and "Skills"; #668): a skill with
//! [`Effect::ManaRegen`] makes mana come back faster (`World::mana_rate`), one with
//! [`Effect::MaxMana`] deepens the pool a little ([`Actor::max_mana`], at most a fifth,
//! [`crate::mana::deeper`]). Both only while the actor is trained in such a skill: an untrained
//! one gives nothing, even with an item or a talent raising it. Marks:
//! [`crate::skills::Use::Cast`], reported where mana is spent (`World::spend_mana`).

use super::Actor;
use crate::content::Content;
use crate::skills::Effect;

impl Actor {
    /// What its trained skills give of the mana effect `effect` (Concentration), with what
    /// items and talents add to them ([`Actor::skill_effect`]); 0 when it trains no skill
    /// giving it.
    pub(crate) fn mind(&self, content: &Content, effect: Effect) -> i32 {
        if !self.skills.trains_effect(content, effect) {
            return 0;
        }
        self.skill_effect(content, effect)
    }

    /// Its base mana grows by `by` (a level, Deep well), and its mana by what that adds to its
    /// maximum: `by`, and what Concentration deepens the larger pool by.
    pub(crate) fn grow_mana_base(&mut self, content: &Content, by: i32) {
        let before = self.max_mana(content);
        self.mana_base += by;
        self.mana += self.max_mana(content) - before;
    }

    /// Mana pool `pool` deepened by its skills ([`Effect::MaxMana`]), at most by a fifth.
    pub(crate) fn deeper_pool(&self, content: &Content, pool: i32) -> i32 {
        crate::mana::deeper(pool, self.mind(content, Effect::MaxMana))
    }
}
