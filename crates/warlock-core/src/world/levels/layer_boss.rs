//! Layer bosses (spec §7 "The endless branch", #1118): an endless branch's `[[dungeon.endless.boss]]`
//! rows give some depths a unique boss of its own and its lair, a vault (`content/layer_boss.rs`
//! checks them).
//!
//! **Made**: a floor of such a depth is made (`mapgen::generate_for`) with the lair stamped for
//! certain and no staircase down of the generator's: the floor's only way down is the lair's `>`,
//! stamped as plain floor and kept as a hidden location named for the boss (`LevelState::hidden`,
//! ground truth: the `truth` dump shows it, the player sees floor). The `<` is as on any floor.
//! While the boss lives, every floor made at that depth makes it and its lair again: Grundlēas
//! forgets the floor left (`endless.rs`), the unique is forgotten as met with it, and the boss
//! never follows the player on the stairs (`stair_followers.rs`), so it is never on two floors.
//!
//! **Slain**: its death, at any hand, is in the slain list kept for named monsters
//! (`quest.rs::Quests::slain`), and [`World::open_lair`] (one line in `fight.rs`) opens the way
//! down where the lair hides it on the level it died on: the cell becomes the stairs, linked to
//! the next floor. From then on a floor made at that depth reads the slain list
//! (`mapgen::Visit::slain`) and is an ordinary floor.

use super::World;
use crate::content::{StairsDir, StairsLink, the};

impl World {
    /// Unique creature type `id` died on level `level`: if that level hides its lair's way down,
    /// the stairs open, and the player is told if they see them, or hear it on the level.
    pub(super) fn open_lair(&mut self, level: &str, id: &str) {
        let Some(state) = self.state.levels.get_mut(level) else {
            return;
        };
        let Some(at) = (state.hidden.iter()).find_map(|(p, h)| (h.id == id).then_some(*p)) else {
            return;
        };
        let Some(hidden) = state.hidden.remove(&at) else {
            return;
        };
        state.terrain.set(at, hidden.terrain);
        state.stairs.push(StairsLink {
            pos: at,
            dir: StairsDir::Down,
            to: hidden.to,
            name: None,
        });
        // On any level: the cache keeps every level visited (#1518).
        self.lights.terrain_changed(state, &self.content, at);
        if self.state.current_level != level {
            return;
        }
        let line = if self.perception.is_visible(at) {
            format!("With {} dead, stairs open in the floor.", the(&hidden.name))
        } else {
            "Somewhere stone grinds on stone.".to_string()
        };
        self.say(line);
    }
}
