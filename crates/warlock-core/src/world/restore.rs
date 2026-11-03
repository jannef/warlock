//! The scroll of restoration (spec §8 "Scrolls"; the priest design §11, #887): a read with no
//! spell behind it ([`Reading::Cleanse`](crate::content::Reading::Cleanse)). Read on oneself, it
//! mends every point of stat damage (`mends`) and gives back every point drains took
//! (`restores`), as a temple's mending does, free, wherever the reader is; a disease showing
//! does not stop it (the temple's "Be cured first." is advice, not a rule here: the disease may
//! damage the stat again). With nothing of either to mend it is refused, takes no time, and
//! keeps the scroll, as a cure with nothing to cure is. Otherwise the scroll is used up (Scroll
//! savant's chance keeps it, and the read trains its stat, through `use_up`), said read (which
//! identifies its type), and each stat that comes back is named: "You feel your strength
//! return." No stream is drawn but Scroll savant's, and nothing new is saved.

use super::World;
use crate::content::Cleansing;
use crate::stats::Stat;

impl World {
    /// The player reads the cleansing scroll of inventory stack `item` (module docs); whether
    /// it was read (`false`: refused, no time, the scroll kept).
    pub(super) fn read_cleanse(&mut self, item: usize, cleansing: Cleansing) -> bool {
        let me = self.state.player();
        let back: Vec<Stat> = (Stat::ALL.into_iter())
            .filter(|&stat| {
                (cleansing.mends && me.damage.get(stat) > 0)
                    || (cleansing.restores && me.drained[stat] > 0)
            })
            .collect();
        if back.is_empty() {
            self.say("There is nothing to restore.");
            return false;
        }
        let Some((scroll, kept)) = self.use_up(item) else {
            return false;
        };
        self.say_read(&scroll);
        let player = self.state.player;
        if cleansing.mends
            && let Some(actor) = self.state.actors.get_mut(&player)
        {
            actor.damage.mend_all();
        }
        if cleansing.restores {
            self.restore_drained(player);
        }
        for stat in back {
            self.say(format!(
                "You feel your {} return.",
                stat.name().to_lowercase()
            ));
        }
        if kept {
            self.say(super::read::KEPT);
        }
        true
    }
}
