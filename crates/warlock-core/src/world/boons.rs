//! A forgotten altar's great boons and banes (spec §7 "Special rooms"; the design's §7.4, #407):
//! the hooks of the outcome fields `gift`, `wrath`, `levels`, `talent`, and `flaw`
//! (`world/outcome.rs` calls them; the relic is `loot` with a `rarity` floor, the horde a
//! `summon`).
//!
//! - **Gift of the forgotten** ([`World::gift_stat`]): a random stat's score and potential rise
//!   by the roll, for good (a Toughness rise brings its hit points at once).
//! - **Wrath of the forgotten** ([`World::wrath_stat`]): a random stat's potential and score fall
//!   by the roll, for good, never below [`DRAIN_FLOOR`]; stat damage and hit points are kept
//!   within what is left.
//! - **Exalted** ([`World::gain_level`]): experience to the next level's threshold, exactly (no
//!   talent scales it), with all the level brings.
//! - **Unmade** ([`World::lose_level`]): experience to the start of the level below; the hit
//!   points and mana the lost level gave are taken back (never the last hit point), and its
//!   choices still due are dropped. Choices already made stay, so [`Progress::peak`] remembers
//!   the level: gained again, it gives its hit points and mana back but no choices and no stat
//!   checks.
//! - **Chosen** ([`World::bestow_talent`]): one of the talents the player may take now and does
//!   not hold (by id and choice, [`talent::offers`]), drawn, given at rank 1 with what it hands
//!   out, and recorded in [`Actor::boons`](super::Actor): never a level's pick.
//! - **Forsaken** ([`World::bestow_flaw`]): one of the flaw talents (a birth sign's `flaw`) the
//!   player lacks, drawn, given likewise, and recorded in `Actor::boons`, which a restored run
//!   puts back beside the race's and the sign's gifts (`world/stars.rs`).
//!
//! Each returns whether it did anything (an outcome's `otherwise` line is said when none did)
//! and calls `found` first. The draws are from the gameplay stream.
//!
//! [`Progress::peak`]: super::Progress
//! [`DRAIN_FLOOR`]: crate::stats::DRAIN_FLOOR

use super::{World, class};
use crate::dice::Dice;
use crate::stats::{DRAIN_FLOOR, Stat};
use crate::talent::{self, Taken};

impl World {
    /// Gift of the forgotten: a random stat's score and potential rise by `dice`, for good.
    pub(super) fn gift_stat(&mut self, dice: Dice) {
        let content = self.content.clone();
        let stat = Stat::ALL[self.state.rng.index(Stat::ALL.len())];
        let points = dice.roll(&mut self.state.rng).max(1);
        let me = self.player_mut();
        let before = me.max_hp(&content);
        me.stats.gift(stat, points);
        let grown = me.max_hp(&content) - before;
        if grown > 0 {
            me.hp += grown;
        }
    }

    /// Wrath of the forgotten: a random stat's potential and score fall by `dice`, for good,
    /// never below [`DRAIN_FLOOR`]. Whether anything fell.
    pub(super) fn wrath_stat(&mut self, dice: Dice) -> bool {
        let stat = Stat::ALL[self.state.rng.index(Stat::ALL.len())];
        let points = u8::try_from(dice.roll(&mut self.state.rng).max(1)).unwrap_or(u8::MAX);
        let me = self.player_mut();
        let fell = me.stats.lessen(stat, points, DRAIN_FLOOR);
        let score = me.stats.score(stat);
        me.damage.take(stat, 0, score);
        let id = self.state.player;
        self.keep_hp_within_max(id);
        fell
    }

    /// Exalted: the player gains the experience to the next level's threshold, and the level.
    /// Nothing at the highest level.
    pub(super) fn gain_level(&mut self) -> bool {
        let rules = self.content.experience();
        let level = self.player_level();
        if level >= rules.max_level() {
            return false;
        }
        let factor = self.experience_factor();
        let needed = rules
            .threshold_for(level + 1, factor)
            .saturating_sub(self.state.experience);
        self.add_xp(needed.max(1));
        true
    }

    /// Unmade (module docs): the player loses a level, back to its start's experience below
    /// it. At level 1 the experience toward level 2 is lost. Whether anything was.
    pub fn lose_level(&mut self) -> bool {
        let content = self.content.clone();
        let rules = content.experience();
        let level = self.player_level();
        let factor = self.experience_factor();
        if level <= 1 {
            let had = self.state.experience > 0;
            self.state.experience = 0;
            return had;
        }
        let below = level - 1;
        let progress = &mut self.state.progress;
        progress.peak = progress.peak.max(level);
        progress.due.retain(|d| d.level < level);
        // What the level gave: its own hit points and mana, the talents' growth, Hardened's.
        let mods = self.state.player().talent_mods(&content).into_owned();
        let mut hp = mods.hp_at(level) - mods.hp_at(below);
        if level <= rules.hp_level() {
            hp += self.level_hp(&content);
        }
        if let Some(bonus) = mods.hardened.filter(|_| level > rules.hp_level()) {
            hp += self.level_hp(&content) + bonus;
        }
        let mut mana = mods.mana_at(level) - mods.mana_at(below);
        if level <= rules.mana_level() {
            mana += self.level_mana(&content);
        }
        self.state.experience = rules.threshold_for(below, factor);
        let me = self.player_mut();
        me.hp_base = (me.hp_base - hp).max(1);
        me.mana_base = (me.mana_base - mana).max(0);
        let (max_hp, max_mana) = (me.max_hp(&content), me.max_mana(&content));
        me.hp = me.hp.min(max_hp).max(1);
        me.mana = me.mana.min(max_mana).max(0);
        self.news.push(format!("You are level {below} again."));
        true
    }

    /// Chosen: a talent the player may take now and does not hold, drawn, given for life.
    /// Whether one was; `found` first.
    pub(super) fn bestow_talent(&mut self, found: impl FnOnce(&mut Self)) -> bool {
        let content = self.content.clone();
        let me = self.state.player();
        let ways: Vec<(String, Option<String>)> = (talent::offers(&content, &self.candidate()))
            .into_iter()
            .flat_map(|(id, choices)| {
                let id = content.talents().get(id).id.clone();
                choices.into_iter().map(move |c| (id.clone(), c))
            })
            .filter(|(id, c)| me.talents.rank_of(id, c.as_deref()) == 0)
            .collect();
        if ways.is_empty() {
            return false;
        }
        found(self);
        let (id, choice) = ways[self.state.rng.index(ways.len())].clone();
        self.bestow(&id, choice.as_deref());
        let def = content
            .talents()
            .get(content.talents().id(&id).expect("offered"));
        let name = def.name_for(&content, choice.as_deref());
        self.say(format!("You gain {name}."));
        true
    }

    /// Forsaken: a flaw talent the player lacks, drawn, given for life. Whether one was;
    /// `found` first.
    pub(super) fn bestow_flaw(&mut self, found: impl FnOnce(&mut Self)) -> bool {
        let content = self.content.clone();
        let me = self.state.player();
        let mut flaws: Vec<String> = (content.talents().by_tree())
            .filter(|(_, def)| def.flaw && !me.talents.has(&def.id, 1))
            .map(|(_, def)| def.id.clone())
            .collect();
        flaws.sort();
        if flaws.is_empty() {
            return false;
        }
        found(self);
        let id = flaws[self.state.rng.index(flaws.len())].clone();
        self.bestow(&id, None);
        let def = content
            .talents()
            .get(content.talents().id(&id).expect("a flaw"));
        self.say(format!("You bear the mark: {}.", def.name));
        true
    }

    /// Talent `id` for `choice` is given to the player at rank 1, with what it hands out, and
    /// kept among their boons (a story's talent and flaw too, `world/background.rs`).
    pub(super) fn bestow(&mut self, id: &str, choice: Option<&str>) {
        let content = self.content.clone();
        let Some(talent) = content.talents().id(id) else {
            return;
        };
        let def = content.talents().get(talent);
        let level = self.player_level();
        let me = self.player_mut();
        me.talents.take(id, choice);
        class::hand_out(&content, me, def, 1, level);
        me.boons.push(Taken {
            id: id.to_string(),
            rank: 1,
            choice: choice.map(str::to_string),
        });
        // Put in the order a restored run gives them (a flaw among the granted talents).
        let character = self.state.character.clone();
        let me = self.player_mut();
        let boons = std::mem::take(&mut me.boons);
        super::stars::grant(&content, character.as_ref(), &boons, &mut me.talents);
        me.boons = boons;
        self.train_chosen(def.effects_at(1), choice);
    }
}

#[cfg(test)]
#[path = "boons_tests.rs"]
mod tests;
