//! The Crafts trades (spec §8 "Talents"; the craftsperson design §3): Smith, Wright, Gemcutter,
//! Brewster, Pitman, Wortcunner, and Spellwright, each a `trade` hand-out
//! ([`TalentEffect::Trade`](crate::content::TalentEffect::Trade)), taken as a creation pick by a
//! craftsperson alone (`needs = { creation = true, class = "craftsperson", race_lacks = "..." }`).
//!
//! Handed down once, as the talent is taken ([`World::learn_trade`], from `World::hand_down`):
//!
//! - **The skill**: its dice, plus the holder's `TalentMods::trade_bonus` (Time served), added
//!   to the skill with `Skills::add`, as a race's or a story's grant is (two grants of a skill
//!   sum).
//! - **The tools of the trade**: kit lines (`content/kit.rs`), each line's alternative drawn as
//!   a kit draws it (one draw among `a|b`), made, and put in the pack, worn or wielded in a free
//!   slot when the line says so, its labelled type known as everything a run starts with is
//!   (`World::story_item`).
//!
//! Both draw from the stream `character/trade/<race>,<gender>,<class>/<skill>`, its own, so no
//! other roll of the run moves. The skill stays a trade of the holder's
//! (`TalentMods::trades`): a level-up roll on it throws the die one step up
//! (`world/skill_roll.rs`).

use super::World;
use crate::content::{Content, KitLineSpec};
use crate::dice::Dice;
use crate::magic::loot;
use crate::rng::Rng;

impl World {
    /// A trade just taken: `skill` trained at `dice` plus the trade bonus, and `tools` handed
    /// down, from the trade's own stream for the character `spec` (`race,gender,class`).
    pub(super) fn learn_trade(
        &mut self,
        content: &Content,
        spec: &str,
        skill: &str,
        dice: Dice,
        tools: &[KitLineSpec],
    ) {
        let Some(id) = content.skill_id(skill) else {
            return;
        };
        let seed = self.state.seed;
        let mut rng = Rng::stream(seed, &format!("character/trade/{spec}/{skill}"));
        let bonus = self.state.player().talent_mods(content).trade_bonus;
        let rolled = dice.roll(&mut rng) + bonus;
        self.player_mut().skills.add(id, rolled);
        // The loader checked the lines (`check_trades`).
        let Ok(kit) = content.kit_lines(tools) else {
            return;
        };
        for line in kit.items() {
            let picked = line.pick(content, &mut rng, &[]);
            for stack in &line.one_of[picked] {
                let stack = match line.magic {
                    Some(grade) => {
                        let (object, count) = (stack.object, stack.count);
                        let level = crate::kit::ITEM_LEVEL;
                        loot::make(content, &mut rng, object, count, level, grade)
                    }
                    None => {
                        let mut plain = stack.clone();
                        plain.draw_wand_rung(content, &mut rng);
                        plain
                    }
                };
                self.story_item(content, stack, line.keep);
            }
        }
    }
}
