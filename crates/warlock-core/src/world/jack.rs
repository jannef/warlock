//! Jack of all trades (spec §8 "Class abilities", the bard's; the bard design §7): the
//! `take_up_skills` hand-out. Every skill of the content the player is not trained in, but
//! those the effect excepts (Sneak attack, the thief's), is taken up at the effect's dice,
//! rolled from the `levels` stream in skill file order: whatever skills the content holds, never
//! a list of its own, so a skill added later is taken up too. Trained from then on, the skills
//! are shown, marked by use, rollable at a level, and trained at all for every rule that asks.

use super::World;
use crate::content::TalentEffect;
use crate::rng::Rng;

impl World {
    /// Every `take_up_skills` of `effects` (a rank just taken or given) trains each skill the
    /// player lacks at its dice; returns how many were taken up.
    pub(super) fn take_up_skills(&mut self, effects: impl Iterator<Item = TalentEffect>) -> usize {
        let content = self.content.clone();
        let mut taken = 0;
        for effect in effects {
            let TalentEffect::TakeUpSkills { dice, except, .. } = effect else {
                continue;
            };
            for (id, skill) in content.skill_ids() {
                let me = self.state.player();
                if me.skills.get(id).is_some() || except.contains(&skill.id) {
                    continue;
                }
                let seed = self.state.seed;
                let rng = (self.state.levels_rng).get_or_insert_with(|| Rng::levels(seed));
                let rolled = dice.roll(rng);
                self.player_mut().skills.add(id, rolled);
                taken += 1;
            }
        }
        taken
    }
}
