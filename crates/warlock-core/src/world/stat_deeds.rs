//! Deeds that cost the player's stats marks (spec §8 "Stats"; the rules are
//! [`crate::content::StatDeed`] and `[[training.deed]]`; the design's §7): few, and deliberate.
//! Each is reported at the one hook that already knows the event, through
//! [`World::stat_deed`]:
//!
//! - **hungry dawn**: the daily check at 06:00 made while Weak or worse, before its draws
//!   (`World::daily_stat_check`);
//! - **starved**: each hit point starvation takes (`status.rs`, after `World::digest`);
//! - **vomited**: `eat.rs::vomit`; **fainted**: `World::faint`;
//! - **overloaded**: an action that took time, not a wait or a drop, while Overloaded
//!   (`World::apply`, beside the burdened use);
//! - **knew better**: a trap the player knew of sprung under them (`World::step_onto_trap`);
//! - **caught**: caught stealing (`World::caught_stealing`).
//!
//! A rule's `marks` are taken at once (no draw); a `chance` takes a mark with one draw of the
//! **training** stream per mark (never gameplay). Only the player loses marks, and only while
//! alive. The pile saturates at `i32::MIN`. When a deed takes a pile from out of the drop's play
//! to within it (the quarter rule against the drop's price,
//! [`StatTraining::in_play`](crate::content::StatTraining::in_play)), the player is warned in
//! words ([`warn_news`]); a drop can come only at a later check (`World::settle`), so the warning
//! always comes first.

use super::World;
use crate::content::StatDeed;
use crate::stats::Stat;

/// What the player is told when a stat's pile first comes within reach of a drop.
fn warn_news(stat: Stat) -> &'static str {
    match stat {
        Stat::Strength => "You feel your strength wasting.",
        Stat::Dexterity => "Your hands feel less sure.",
        Stat::Toughness => "You feel your health failing.",
        Stat::Intelligence => "Your mind feels dull.",
        Stat::Willpower => "Your will feels brittle.",
        Stat::Presence => "You feel small and unregarded.",
    }
}

/// What a drop in `stat` says.
pub(super) fn drop_news(stat: Stat) -> &'static str {
    match stat {
        Stat::Strength => "You feel weaker.",
        Stat::Dexterity => "You feel clumsier.",
        Stat::Toughness => "You feel frailer.",
        Stat::Intelligence => "You feel duller.",
        Stat::Willpower => "Your resolve wavers.",
        Stat::Presence => "You feel less sure of yourself.",
    }
}

impl World {
    /// The player did `deed` (module docs).
    pub(super) fn stat_deed(&mut self, deed: StatDeed) {
        self.stat_deed_times(deed, 1);
    }

    /// [`World::stat_deed`] `count` times over (each hit point starvation took): each rule for
    /// `deed` takes its `marks` `count` times, or a mark with its `chance` `count` times (a draw
    /// of the training stream each). A pile that comes within reach of a drop says so.
    pub(super) fn stat_deed_times(&mut self, deed: StatDeed, count: u32) {
        if self.state.death.is_some() || count == 0 {
            return;
        }
        let content = self.content.clone();
        // Hard living, Discipline, Ascetic: a deed their talents spare takes nothing.
        if self.state.player().talent_mods(&content).spares_deed(deed) {
            return;
        }
        let rules = content.stat_training();
        for rule in rules.deeds(deed) {
            let taken = match (rule.marks, rule.chance) {
                (Some(marks), _) => u32::from(marks).saturating_mul(count),
                (None, Some(chance)) => (0..count)
                    .filter(|_| self.state.training.chance(u32::from(chance)))
                    .count() as u32,
                (None, None) => 0,
            };
            let player = self.state.player;
            let Some(actor) = self.state.actors.get_mut(&player) else {
                return;
            };
            let stat = rule.stat;
            let before = actor.stat_marks[stat];
            let after = before.saturating_sub(i32::try_from(taken).unwrap_or(i32::MAX));
            actor.stat_marks[stat] = after;
            let price = rules.drop_price(actor.stats.score(stat));
            let in_play = |marks: i32| marks < 0 && rules.in_play(marks, price);
            if in_play(after) && !in_play(before) {
                self.say(warn_news(stat));
            }
        }
    }
}
