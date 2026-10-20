//! Disarming traps (spec §8 "Disarming traps"; #232): the use of the Disable device skill, the
//! Disarm ability ([`crate::ability::Ability::Disarm`], [`crate::Action::Disarm`]).
//!
//! The player may try to take apart a trap they know of on a cell next to them (a door's trap
//! too, spec §8 "Trapped doors"), if they are trained in Disable device (the skill with the
//! disarm effect). The try is a check (spec §8 "Checks", [`crate::check::roll`]): 2d12 plus the
//! Dexterity modifier (of the effective score, spec §8 "Stats") plus what Disable device gives,
//! against the trap's difficulty (the one finding it takes, deeper on a branch too:
//! [`World::difficulty_of`]). Made, the trap is gone; failed, it stays, and failed by
//! [`SPRINGS_SHORT_BY`] or more it goes off at the player ([`World::spring_trap`]: an arrow from
//! its slit at them, spears at them, its blast, its bolt through them, a pit's fall and climb, an
//! alarm, a teleport of them). Every try takes the `disarm` action's time and may earn Disable
//! device a training mark (training stream), whatever comes of it; nothing remembers a failed
//! one, so the next try may make it (no limit on trying again: the owner's rule; time and the
//! risk are the price).
//!
//! Without the skill, or without a trap the player knows of there (a hidden one is as good as
//! none: the refusal is the same, so it gives nothing away), the action is refused and takes no
//! time and no draw.

use super::search::Hidden;
use super::{Spent, World};
use crate::ability::Ability;
use crate::action::Event;
use crate::check::{self, Checked};
use crate::content::ActionKind;
use crate::geom::Dir;
use crate::skills::{Effect, Use};
use crate::stats::Stat;

/// How far short of a trap's difficulty a failed try must fall to set the trap off.
pub const SPRINGS_SHORT_BY: i32 = 5;

impl World {
    /// The player tries to disarm the trap they know of in `dir` (module docs). Returns what it
    /// cost: the `disarm` action, priced before the trap may go off (it may kill them), and a
    /// pit's climb on top.
    pub(super) fn disarm(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        if !Ability::Disarm.had_by(self.state.player(), &self.content) {
            self.say("You do not know how to disarm traps.");
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        let known = self.state.current().traps.get(&at).filter(|t| t.known);
        let Some(kind) = known.map(|t| t.kind) else {
            self.say("You know of no trap there.");
            return Spent::Nothing;
        };
        let price = self.price_of(player, ActionKind::Disarm);
        let modifier = self.disarm_modifier();
        let difficulty = self.difficulty_of(Hidden::Trap(kind));
        let checked = check::roll(&mut self.state.rng, modifier, difficulty);
        if self.trains(Use::Disarm) {
            self.train(Use::Disarm);
        }
        let name = self.content.trap(kind).name.clone();
        match checked {
            Checked::Made => {
                let level = self.state.current_mut();
                level.traps.remove(&at);
                level.trap_memory.remove(&at);
                self.say(format!("You disarm the {name}."));
                Spent::Turn(price)
            }
            Checked::Failed { by } if by >= SPRINGS_SHORT_BY => {
                self.say(format!("You set off the {name}!"));
                let climb = self.spring_trap(at, player, events);
                Spent::Turn(price + climb)
            }
            Checked::Failed { .. } => {
                self.say(format!("You fail to disarm the {name}."));
                Spent::Turn(price)
            }
        }
    }

    /// What the player adds to a try at disarming a trap: the Dexterity modifier (of the
    /// effective score) and what Disable device gives.
    pub(super) fn disarm_modifier(&self) -> i32 {
        let content = &self.content;
        let player = self.state.player();
        let dexterity = player.effective_stats(content).modifier(Stat::Dexterity);
        dexterity.saturating_add(player.skill_effect(content, Effect::Disarm))
    }
}
