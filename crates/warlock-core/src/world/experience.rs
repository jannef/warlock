//! Experience (spec §8 "Experience"): the player's XP total and level. Kills the player makes
//! with their own attacks (melee, shots, throws) are worth the creature type's XP scaled by the
//! difference in levels ([`Experience::kill_xp`](crate::content::Experience::kill_xp)); kills by
//! traps or by other creatures are worth nothing. The level follows from the total (thresholds
//! in `data/experience.toml`), so only the total is saved. Each level gained, up to the file's
//! `hp_level`, adds `max(1, class hp ÷ 2 + the Toughness modifier)` to the player's maximum hit
//! points and to their current ones.

use super::World;
use crate::content::{Content, CreatureId};
use crate::stats::Stat;

impl World {
    /// The player's level (spec §8 "Experience").
    pub fn player_level(&self) -> u8 {
        self.content.experience().level_for(self.state.experience)
    }

    /// The player's experience total.
    pub fn player_xp(&self) -> u32 {
        self.state.experience
    }

    /// The player has killed a creature of type `creature` by their own attack: gains its XP,
    /// and any levels that earns. The messages wait in `news` until the attack's own are said.
    pub(super) fn earn_kill(&mut self, creature: CreatureId) {
        let content = self.content.clone();
        let def = content.creature(creature);
        let rules = content.experience();
        let gain = rules.kill_xp(def.xp, def.level, self.player_level());
        self.grant_xp(gain);
    }

    /// The player gains `gain` experience, and any levels that earns.
    pub(super) fn grant_xp(&mut self, gain: u32) {
        let content = self.content.clone();
        let rules = content.experience();
        let before = self.player_level();
        if gain == 0 {
            return;
        }
        self.state.experience = self.state.experience.saturating_add(gain);
        let after = self.player_level();
        if after == before {
            return;
        }
        let grown = (before + 1..=after)
            .filter(|&l| l <= rules.hp_level())
            .count();
        let hp = i32::try_from(grown).unwrap_or(0) * self.level_hp(&content);
        if let Some(player) = self.state.actors.get_mut(&self.state.player) {
            player.hp_base += hp;
            player.hp += hp;
        }
        self.news.push("You feel more experienced.".into());
        self.news.push(format!("Welcome to level {after}."));
    }

    /// What one level adds to the player's hit points: half the class's hit points (the
    /// scenario's `hp` without a character) plus the Toughness modifier, at least 1.
    fn level_hp(&self, content: &Content) -> i32 {
        let class_hp = match &self.state.character {
            Some(c) => c.spec.class(content).map_or(0, |class| class.hp),
            None => content
                .scenario(&self.state.scenario)
                .map_or(0, |s| i32::from(s.hp)),
        };
        // The base Toughness, not the effective one: a status penalty passes, these stay.
        let toughness = self.state.player().stats.modifier(Stat::Toughness);
        (class_hp / 2 + toughness).max(1)
    }

    /// Says what the player's kills earned them, after the attack's own messages.
    pub(super) fn say_news(&mut self) {
        for text in std::mem::take(&mut self.news) {
            self.say(text);
        }
    }
}
