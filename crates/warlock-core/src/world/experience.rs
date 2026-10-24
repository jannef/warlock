//! Experience (spec §8 "Experience"): the player's XP total and level. Kills the player makes
//! with their own attacks (melee, shots, throws) are worth the creature type's XP scaled by the
//! difference in levels ([`Experience::kill_xp`](crate::content::Experience::kill_xp)); kills by
//! traps or by other creatures are worth nothing. The level follows from the total and the
//! character's experience factor (thresholds in `data/experience.toml`, times their race's and
//! their class's `experience`), so only the total is saved. Each level gained, up to the file's
//! `hp_level`, adds `max(1, class hp ÷ 2 + the Toughness modifier)` to the player's maximum hit
//! points and to their current ones, and each up to `mana_level` adds mana (`world/mana.rs`).
//!
//! **Discovery** (`[discovery]`): the player's first kill of each creature type is worth the
//! type's XP once more, unscaled ("You learn something of kobolds."), and their first arrival on
//! a level `arrival` × its item level ("You gain 20 experience for finding your way here."); the
//! level a run starts on, the overworld, and local areas give none. Neither draws anything at
//! random. The creature types are remembered in [`Progress`]; a level is new when this arrival
//! made its state.

use super::{Due, Spent, World};
use crate::content::StatUse;
use crate::content::{Content, CreatureId, Factor, LevelKind, parse_local_area};
use crate::stats::Stat;
use crate::talent::TalentMods;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The player's progression beyond the XP total (spec §8 "Experience", "Talents"): what they
/// have discovered, and what their levels still ask them to choose. Left out of the saved state
/// while empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    #[serde(default, skip_serializing_if = "Discovered::is_empty")]
    pub(crate) discovered: Discovered,
    /// The choices levels still ask for, the lowest level's first (`world/talent.rs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) due: Vec<Due>,
}

impl Progress {
    pub fn is_empty(&self) -> bool {
        self.discovered.is_empty() && self.due.is_empty()
    }
}

/// What the player has been rewarded for discovering.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discovered {
    /// The creature types (by id) whose first kill has paid.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) kills: BTreeSet<String>,
}

impl Discovered {
    pub fn is_empty(&self) -> bool {
        self.kills.is_empty()
    }
}

impl World {
    /// The player's level (spec §8 "Experience").
    pub fn player_level(&self) -> u8 {
        let factor = self.experience_factor();
        (self.content.experience()).level_for(self.state.experience, factor)
    }

    /// The player's experience total.
    pub fn player_xp(&self) -> u32 {
        self.state.experience
    }

    /// The player's experience factor: their race's and their class's (100 % without a
    /// character).
    fn experience_factor(&self) -> Factor {
        let spec = self.state.character.as_ref().map(|c| &c.spec);
        Factor::of(&self.content, spec)
    }

    /// The player has killed a creature of type `creature` by their own attack: gains its XP,
    /// and its XP once more the first time they kill one of its type, and any levels that
    /// earns. The messages wait in `news` until the attack's own are said.
    pub(super) fn earn_kill(&mut self, creature: CreatureId) {
        let content = self.content.clone();
        let def = content.creature(creature);
        let rules = content.experience();
        let mut gain = rules.kill_xp(def.xp, def.level, self.player_level());
        let first = rules.first_kill_xp(def.xp);
        let discovered = &mut self.state.progress.discovered.kills;
        if first > 0 && !discovered.contains(&def.id) {
            discovered.insert(def.id.clone());
            self.news
                .push(format!("You learn something of {}.", def.plural));
            gain += first;
            self.train_stat(StatUse::Discovered);
        }
        self.grant_xp(gain);
    }

    /// The player has arrived on level `id` for the first time: a level of a dungeon branch or
    /// a hand-made one is worth `arrival` × its item level; the overworld and its local areas
    /// nothing. Said after the arrival's own messages, as the turn is perceived.
    pub(super) fn discover_level(&mut self, id: &str) {
        let overworld = self.state.levels.get(id).map(|l| l.kind) == Some(LevelKind::Overworld);
        if overworld || parse_local_area(id).is_some() {
            return;
        }
        let gain = (self.content.experience()).arrival_xp(self.content.item_level(id));
        if gain == 0 {
            return;
        }
        self.news.push(format!(
            "You gain {gain} experience for finding your way here."
        ));
        self.grant_xp(gain);
    }

    /// The dev action `xp:N` ([`crate::action::Action::GrantXp`]): the player gains `xp`
    /// experience, and any levels that earns. Takes no time.
    pub(super) fn grant_xp_dev(&mut self, xp: u32) -> Spent {
        self.say(format!("You gain {xp} experience (dev)."));
        self.grant_xp(xp);
        Spent::NoTime
    }

    /// The player gains `gain` experience (more by the talents that teach quicker, spec §8
    /// "Talents": Good learner), and any levels that earns, each handing out what it brings
    /// ([`World::level_up`]).
    pub(super) fn grant_xp(&mut self, gain: u32) {
        let content = self.content.clone();
        let rules = content.experience();
        let before = self.player_level();
        if gain == 0 {
            return;
        }
        let percent = self.state.player().talent_mods(&content).xp_percent;
        let gain = TalentMods::scaled(u64::from(gain), percent);
        let gain = u32::try_from(gain).unwrap_or(u32::MAX).max(1);
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
        self.level_up_mana(before, after);
        self.news.push("You feel more experienced.".into());
        self.news.push(format!("Welcome to level {after}."));
        self.level_up(before, after);
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
        // Toughness as what the player wears raises it, but not as a status effect lowers it:
        // the penalty passes, these hit points stay.
        let toughness = (self.state.player().raised_stats(content)).modifier(Stat::Toughness);
        (class_hp / 2 + toughness).max(1)
    }

    /// Says what the player's kills earned them, after the attack's own messages, then the
    /// notes the action earned (an alignment band crossed), which do not stop a rest.
    pub(super) fn say_news(&mut self) {
        for text in std::mem::take(&mut self.news) {
            self.say(text);
        }
        // In a free blow the news is held for after that turn's reports, and so are the notes.
        if self.held.is_none() {
            for text in std::mem::take(&mut self.notes) {
                self.note(text);
            }
        }
    }
}
