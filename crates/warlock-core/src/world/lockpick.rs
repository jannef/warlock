//! Picking locks (spec §8 "Doors"; the Pick lock ability, the Open lock skill's use, #233).
//!
//! - **Who**: an actor trained in a skill with the `pick` effect (Open lock, `data/skills.toml`),
//!   [`crate::skills::Skills::trains_effect`]: the abilities menu lists Pick lock for them
//!   ([`crate::ability::Ability::PickLock`]). Anyone else is told "You do not know how to pick
//!   locks." and no time passes.
//! - **A try** at the locked closed door in a direction is a check ([`crate::check`]): 2d12 plus
//!   the picker's Dexterity modifier (of the effective score, spec §8 "Stats") plus what Open lock
//!   gives ([`Effect::Pick`]) against the lock's difficulty ([`World::lock_difficulty`]). It takes
//!   the `pick_lock` action whatever comes of it, may earn Open lock a training mark
//!   ([`Use::Pick`]), and makes no noise of its own.
//! - **Picked**, the lock is gone and the door swings open in the same action, as a key opens
//!   it ([`World::swing_open`]: its noise, and a trap on the door going off at the picker), on top
//!   of the try's time (`open_door`); both are priced before the check, since a trap may kill.
//!   **Failed**, the lock holds: "You fail to pick the lock." Nothing is remembered of a failed
//!   try, so the next may succeed (no limit on trying again, the owner's rule: time is the
//!   price).
//! - A closed door without a lock, or anything else, is refused and takes no time: "That door is
//!   not locked." (as `o` would find out at once), "There is no lock there."

use super::{Spent, World};
use crate::action::Event;
use crate::check::check;
use crate::content::ActionKind;
use crate::geom::{Dir, Pos};
use crate::skills::{Effect, Use};
use crate::stats::Stat;

impl World {
    /// The player tries to pick the lock of the closed door in `dir`. Returns what it cost.
    pub(super) fn pick_lock(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        if !me.skills.trains_effect(&content, Effect::Pick) {
            self.say("You do not know how to pick locks.");
            return Spent::Nothing;
        }
        let modifier = me.effective_stats(&content).modifier(Stat::Dexterity)
            + me.skill_effect(&content, Effect::Pick)
            + me.talent_mods(&content).checks[Stat::Dexterity];
        let at = self.player_pos().step(dir);
        let level = self.state.current();
        let closed = (level.terrain.get(at)).is_some_and(|t| content.terrain(*t).is_closed_door());
        if !closed {
            self.say("There is no lock there.");
            return Spent::Nothing;
        }
        if !level.locks.contains_key(&at) {
            self.say("That door is not locked.");
            return Spent::Nothing;
        }
        let difficulty = self.lock_difficulty(at);
        let player = self.state.player;
        let try_price = self.price_of(player, ActionKind::PickLock);
        let open_price = self.price_of(player, ActionKind::OpenDoor);
        let picked = check(&mut self.state.rng, modifier, difficulty);
        if self.trains(Use::Pick) {
            self.train(Use::Pick);
        }
        if !picked {
            self.say("You fail to pick the lock.");
            return Spent::Turn(try_price);
        }
        self.state.current_mut().locks.remove(&at);
        self.say("You pick the lock and open the door.");
        self.swing_open(player, at, events);
        Spent::Turn(try_price + open_price)
    }

    /// How hard the lock at `at` on the current level is to pick (spec §8 "Doors"): its own
    /// difficulty ([`crate::door::Lock::difficulty`]), plus on a dungeon branch its
    /// `locks_per_depth` for each level below the first. 0 where there is no lock.
    pub(super) fn lock_difficulty(&self, at: Pos) -> i32 {
        let Some(lock) = self.state.current().locks.get(&at) else {
            return 0;
        };
        let deeper = (self.content.branch_level(&self.state.current_level))
            .map_or(0, |(def, depth)| def.locks_at(depth));
        i32::from(lock.difficulty()).saturating_add(deeper)
    }
}
