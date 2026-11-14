//! The scroll of restoration (spec §8 "Scrolls"; the priest design §11, #887): a read with no
//! spell behind it ([`Reading::Cleanse`](crate::content::Reading::Cleanse)). Read on oneself, it
//! mends every point of stat damage (`mends`) and gives back every point drains took
//! (`restores`), as a temple's mending does, free, wherever the reader is; a disease showing
//! does not stop it (the temple's "Be cured first." is advice, not a rule here: the disease may
//! damage the stat again). It also ends every status `cured_by` [`RESTORATION`] (Stiffening, a
//! basilisk's gaze turning the reader to stone; the end line is said). With nothing of these to
//! mend or end it is refused, takes no time, and
//! keeps the scroll, as a cure with nothing to cure is. Otherwise, once made out (`decipher.rs`,
//! as a spell of level 0), the scroll is used up (Scroll savant's chance keeps it, and the read trains its stat, through `use_up`), said read (which
//! identifies its type), and each stat that comes back is named: "You feel your strength
//! return." No stream is drawn but the check's and Scroll savant's, and nothing new is saved.
//!
//! The cleanse itself ([`World::restore`]) is shared with the potion of restoration (kind
//! `restore`, `world/health/potion.rs`), which mends and restores both.

use crate::action::Event;
use crate::content::Cleansing;
use crate::stats::Stat;
use crate::world::World;

/// The means the scroll of restoration cures by (`cured_by`): Stiffening's.
pub const RESTORATION: &str = "restoration";

impl World {
    /// The player reads the cleansing scroll of inventory stack `item` (module docs); whether
    /// it was read (`false`: refused, no time, the scroll kept).
    pub(super) fn read_cleanse(
        &mut self,
        item: usize,
        cleansing: Cleansing,
        events: &mut Vec<Event>,
    ) -> bool {
        if !self.needs_cleansing(cleansing) {
            self.say("There is nothing to restore.");
            return false;
        }
        // A read with no spell behind it: level 0 (`decipher.rs`). Failed, its time and no more.
        if !self.deciphers(None, 0) {
            return true;
        }
        let Some((scroll, kept)) = self.use_up(item) else {
            return false;
        };
        self.say_read(&scroll);
        self.restore(cleansing, events);
        if kept {
            self.say(super::read::KEPT);
        }
        true
    }

    /// Whether `cleansing` would do anything for the player: stat damage to mend, drained points
    /// to give back, or a status `restoration` cures.
    fn needs_cleansing(&self, cleansing: Cleansing) -> bool {
        let me = self.state.player();
        let cures = |by: &[String]| by.iter().any(|c| c == RESTORATION);
        (me.effects.iter()).any(|e| cures(&self.content.status(e.status).cured_by))
            || !self.cleansed_stats(cleansing).is_empty()
    }

    /// The stats `cleansing` would bring back for the player, in stat order.
    fn cleansed_stats(&self, cleansing: Cleansing) -> Vec<Stat> {
        let me = self.state.player();
        (Stat::ALL.into_iter())
            .filter(|&stat| {
                (cleansing.mends && me.damage.get(stat) > 0)
                    || (cleansing.restores && me.drained[stat] > 0)
            })
            .collect()
    }

    /// The one cleanse (the scroll of restoration's read and the potion of restoration's
    /// draught): every point of stat damage mended (`mends`), every drained point back
    /// (`restores`), and every status `restoration` cures ended (Stiffening, its end line
    /// said); each stat that comes back is named. Draws nothing. Returns whether anything was
    /// done.
    pub(in crate::world) fn restore(
        &mut self,
        cleansing: Cleansing,
        events: &mut Vec<Event>,
    ) -> bool {
        let back = self.cleansed_stats(cleansing);
        let player = self.state.player;
        if cleansing.mends
            && let Some(actor) = self.state.actors.get_mut(&player)
        {
            actor.damage.mend_all();
        }
        if cleansing.restores {
            self.restore_drained(player);
        }
        let cured = self.cure(player, RESTORATION, events);
        for stat in &back {
            self.say(format!(
                "You feel your {} return.",
                stat.name().to_lowercase()
            ));
        }
        cured || !back.is_empty()
    }
}
