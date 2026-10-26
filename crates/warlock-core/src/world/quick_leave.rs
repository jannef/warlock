//! The quick leave (spec §7 "Leaving an outdoor area"): the ascend command on an outdoor level
//! with an edge exit, off any stairs up, leaves it back to the overworld cell as walking off its
//! edge would, unless a hostile creature is about: one the player sees, or one they do not see
//! within [`QUICK_LEAVE_REACH`] cells. The check reads ground truth (it is a rule, like a
//! creature noticing the player); the refusal says neither which kind of creature blocked it nor
//! where, and takes no time.

use super::{Spent, World};
use crate::action::Event;
use crate::content::{LevelKind, Stance};

/// How near (Chebyshev distance, in cells) an unseen hostile creature stops the quick leave.
pub const QUICK_LEAVE_REACH: i32 = 10;

/// What the player is told when the quick leave is refused.
pub const QUICK_LEAVE_REFUSED: &str = "You cannot slip away with enemies about.";

impl World {
    /// `<` off any stairs up: on an outdoor level with an edge exit, the quick leave (whether it
    /// was done: refused, no time passes); `None` on any other level (the usual "no way up").
    /// Done, it is priced as taking stairs, like walking off the edge.
    pub(super) fn quick_leave(&mut self, events: &mut Vec<Event>) -> Option<bool> {
        let level = self.state.current();
        if level.kind != LevelKind::Outdoor {
            return None;
        }
        let exit = level.exit.clone()?;
        if self.enemies_about() {
            self.say(QUICK_LEAVE_REFUSED);
            return Some(false);
        }
        if !self.can_move() {
            return Some(false);
        }
        Some(!matches!(self.leave_by_edge(exit, events), Spent::Nothing))
    }

    /// Whether a creature hostile to the player is on their level where they see it, or unseen
    /// within [`QUICK_LEAVE_REACH`] of them.
    fn enemies_about(&self) -> bool {
        let player = self.state.player;
        let here = self.player_pos();
        let level = &self.state.current_level;
        (self.state.actors.values())
            .filter(|a| a.id != player && &a.level == level)
            .filter(|a| self.stance(player, a.id) == Stance::Hostile)
            .any(|a| {
                self.perception.sees_actors_at(a.pos) || a.pos.chebyshev(here) <= QUICK_LEAVE_REACH
            })
    }
}
