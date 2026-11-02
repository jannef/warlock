//! The edge rule (spec §7 "Leaving an outdoor area", #803): walking off the edge of an outdoor
//! area with an overworld cell (`travel.rs::player_moves`) is refused while a creature hostile
//! to the player is within `edge_reach` cells of them (`data/encounters.toml`, 3; Chebyshev),
//! seen or not. Like the quick leave's (`quick_leave.rs`), the check reads ground truth as a
//! rule, the refusal names nobody, and it takes no time. The quick leave keeps its own, wider
//! rule; this one only adds to walking.

use super::World;
use crate::content::Stance;

/// What the player is told when walking off the edge is refused.
pub const EDGE_LEAVE_REFUSED: &str = "They are too close to slip away.";

impl World {
    /// Whether a creature hostile to the player stops them walking off the area's edge: one on
    /// their level within the rules' `edge_reach` of them. Says why when it does.
    pub(super) fn held_at_edge(&mut self) -> bool {
        let reach = self.content.encounters().edge_reach;
        let player = self.state.player;
        let here = self.player_pos();
        let level = &self.state.current_level;
        let held = reach > 0
            && (self.state.actors.values())
                .filter(|a| a.id != player && &a.level == level)
                .filter(|a| a.pos.chebyshev(here) <= reach)
                .any(|a| self.stance(player, a.id) == Stance::Hostile);
        if held {
            self.say(EDGE_LEAVE_REFUSED);
        }
        held
    }
}
