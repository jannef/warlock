//! Making out a scroll (spec §8 "Scrolls"; #1024, the skill uses design §6.6): whether the
//! reading the player has asked for, with nothing left to refuse it, goes ahead.
//!
//! One trained in Spellcraft whose learning (`Actor::learning`) is at least the scroll's spell
//! level reads it at once: no draw, no mark, no words. A scroll with no spell behind it counts as
//! level 0. Everyone else makes a check, never a refusal: 2d12 + the Intelligence modifier + what
//! talents add to its checks + Decipher script's points ([`Effect::Decipher`]; an untrained
//! reader has none, unless something they wear raises the skill), against
//! [`DecipherRules::difficulty`](crate::content::DecipherRules::difficulty). Two draws of the
//! gameplay stream, only when asked, and [`Use::Decipher`] marks whatever comes of it.
//!
//! Failed: "The writing swims before your eyes." The caller spends the read's time and stops
//! there: the scroll stays in the pack and a label stays unknown, with no draw for Scroll savant
//! and no Intelligence mark for a read (`use_up` has both). Made: the scroll reads as any other.
//!
//! Only reading asks. Studying a scroll has its own check (`learn.rs`), writing one reads
//! nothing (`scribe.rs`), and casting a spell one knows needs no scroll.

use super::World;
use crate::skills::{Effect, Use};
use crate::stats::Stat;

/// What the player is told when they fail to make a scroll out.
const SWIMS: &str = "The writing swims before your eyes.";

impl World {
    /// Whether the player makes out a scroll of spell level `level` (module docs); `false` has
    /// said so.
    pub(super) fn deciphers(&mut self, level: u8) -> bool {
        let content = self.content.clone();
        let Some(rules) = content.decipher() else {
            return true;
        };
        let me = self.state.player();
        if me.learning(&content).is_some_and(|l| l >= i32::from(level)) {
            return true;
        }
        let bonus = me.effective_stats(&content).modifier(Stat::Intelligence)
            + me.talent_mods(&content).check(Stat::Intelligence)
            + me.skill_effect(&content, Effect::Decipher);
        let made = crate::check::check(&mut self.state.rng, bonus, rules.difficulty(level));
        self.train(Use::Decipher);
        if !made {
            self.say(SWIMS);
        }
        made
    }
}
