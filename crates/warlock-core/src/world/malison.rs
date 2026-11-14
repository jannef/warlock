//! Malison's tithe (spec §8 "Class abilities"; docs/design/witch-and-wizard.md §5.2; #905): a
//! creature that dies under a hex the player laid (an effect whose status is one of
//! [`crate::content::Content::hex_statuses`] and whose giver is the player), of any cause and by
//! anyone's hand, pays the player, while they hold the `hex_tithe` effect, the dead creature's
//! level divided by its `per_levels` in mana, at least 1, never past their maximum. No range and
//! no line of sight: the hex is the tether. The one caller is `World::dies` (fight.rs), for a
//! creature's death, before the creature leaves the world. Malison's other half, the bite, is
//! `status_damage` (`caster.rs::lore_blow`). No draw, no saved state.

use super::{ActorId, World};

impl World {
    /// Creature `slain` is dying: if it is under a hex of the player's and the player holds the
    /// tithe, their mana rises by its level / `per_levels` (at least 1, to their maximum), and
    /// "The hex comes home." waits in the news when it rose at all.
    pub(super) fn hex_tithe(&mut self, slain: ActorId) {
        let player = self.state.player;
        if slain == player {
            return;
        }
        let (Some(me), Some(dead)) = (
            self.state.actors.get(&player),
            self.state.actors.get(&slain),
        ) else {
            return;
        };
        if me.talents.is_empty() {
            return;
        }
        let Some(per_levels) = me.talent_mods(&self.content).magic.hex_tithe else {
            return;
        };
        let hexes = self.content.hex_statuses();
        let hexed =
            (dead.effects.iter()).any(|e| e.by == Some(player) && hexes.contains(&e.status));
        if !hexed {
            return;
        }
        let level = u32::from(self.content.creature(dead.creature).level);
        let paid = (level / u32::from(per_levels.max(1))).max(1);
        let max = me.max_mana(&self.content);
        let before = me.mana;
        let after = before.max(
            before
                .saturating_add(i32::try_from(paid).unwrap_or(i32::MAX))
                .min(max),
        );
        if after > before {
            self.player_mut().mana = after;
            self.news.push("The hex comes home.".to_string());
        }
    }
}
