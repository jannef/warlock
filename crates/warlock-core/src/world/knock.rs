//! Knock (spec §8 "Class abilities", "Chests"; docs/design/chests-and-treasure.md §9): the
//! wizard's class active, `use:knock:DIR`, paid in mana and priced as a spell (`cast`). The lock
//! of the locked door or shut chest that way springs open: the lock is gone ("The lock springs
//! open with a click."), nothing opens, and no trap springs; `open:DIR` then opens it as an
//! unlocked one. Where nothing locked stands (an unlocked door or chest, anything else) it is
//! refused with no time and nothing spent, the same line for each. It draws nothing at random.

use super::{Spent, World};
use crate::content::ActionKind;
use crate::geom::Dir;

impl World {
    /// `use:knock:DIR`: the lock in `dir` springs open, its mana spent; refused, with no time,
    /// where nothing locked stands.
    pub(super) fn knock(&mut self, dir: Dir) -> Spent {
        let Some(knock) = self.state.player().talent_mods(&self.content).knock else {
            return Spent::Nothing;
        };
        let at = self.player_pos().step(dir);
        let level = self.state.current();
        let lockable = (level.terrain.get(at)).is_some_and(|t| {
            let def = self.content.terrain(*t);
            def.is_closed_door() || def.is_shut_chest()
        });
        if !lockable || !level.locks.contains_key(&at) {
            self.say("There is no lock there to spring.");
            return Spent::Nothing;
        }
        let player = self.state.player;
        if !self.spend_mana(player, knock.mana) {
            self.say("You have too little mana.");
            return Spent::Nothing;
        }
        self.state.current_mut().locks.remove(&at);
        self.say("The lock springs open with a click.");
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }
}
