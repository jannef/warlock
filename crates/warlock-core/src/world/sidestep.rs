//! Sidestepping a known trap (spec §8 "Traps", #476): the player who steps onto a trap they know
//! of, of their own will (a move onto it, not a step they were turned into), may avoid it. The
//! chance is `[avoid]` in `data/traps.toml` ([`crate::content::TrapAvoid`]) at their dodge: the
//! Dexterity and Dodge parts of their Defence ([`World::trap_dodge`]), so worn armour and burden
//! cap its Dexterity as they do in combat, and whatever adds Dodge (the Dodge skill, talents,
//! magic, the stance, statuses) adds here. One draw of the gameplay stream, only when a known
//! trap lies there. An avoided trap stays as it was (its charges too), the step costs what it
//! costs, the player is told in words, and Dodge training gets a use ([`Use::Dodge`], as for a
//! dodged attack). A hidden trap, a door's trap (opening the door sets it off, not the step),
//! and creatures keep the rules of `trap.rs`.

use super::World;
use crate::action::Event;
use crate::combat::Category;
use crate::content::TrapEffect;
use crate::geom::Pos;
use crate::skills::Use;

impl World {
    /// The player's dodge against a known trap: the Dexterity and Dodge categories of their
    /// Defence (spec §8 "Combat").
    pub(crate) fn trap_dodge(&self) -> i32 {
        let defence = self.defence_of(self.state.player);
        defence.bonus(Category::Dexterity) + defence.bonus(Category::Dodge)
    }

    /// The player has just stepped onto `at` of their own will: a trap they know of there may be
    /// avoided, else it goes off as any trap does ([`World::step_onto_trap`]). Returns the energy
    /// a pit's climb costs on top of the step.
    pub(super) fn step_willingly(&mut self, at: Pos, events: &mut Vec<Event>) -> i64 {
        if self.sidesteps(at) {
            self.warp_step(at, self.state.player, events);
            return 0;
        }
        self.step_onto_trap(at, self.state.player, events)
    }

    /// Whether the player avoids a known trap at `at` (module docs); says so if they do.
    fn sidesteps(&mut self, at: Pos) -> bool {
        let level = self.state.current();
        if level.is_door(&self.content, at) {
            return false;
        }
        let Some(kind) = (level.traps.get(&at)).filter(|t| t.known).map(|t| t.kind) else {
            return false;
        };
        let chance = self.content.trap_avoid().chance(self.trap_dodge());
        if !self.state.rng.chance(chance) {
            return false;
        }
        let def = self.content.trap(kind);
        let text = match def.effect {
            TrapEffect::Pit { .. } => format!("You step lightly over the {}.", def.name),
            _ => format!("You deftly avoid the {}.", def.name),
        };
        self.say(text);
        if self.trains(Use::Dodge) {
            self.train(Use::Dodge);
        }
        true
    }
}
