//! Statues of the stoned (spec §8 "Statues"; #1126, design `docs/design/monster-dread.md` §5.2,
//! §17 D9): a creature killed by a status with `statue` (Stiffening, a basilisk's gaze) leaves no
//! corpse but a statue, a **fixture of the cell** as a chest is: the statue terrain (the one with
//! `statue` in `data/terrain.toml`: solid, seen past, stone) plus a [`Statue`] on the cell in
//! `LevelState::statues`, what it was and what it carried.
//!
//! - **Laid** ([`World::lay_statue`], from `fight.rs::creature_dies`): where the creature fell,
//!   on the current level, on plain open ground (walkable, no door, stairs, chest, light, ice or
//!   freezing water, and no trap, lock, or fire there); what it carried (worn and wielded too,
//!   an animated thing's body) and then the pile already there go into it, in that order. Where
//!   no statue can stand, what it carried falls as usual, and still no corpse.
//! - **Dug** through with a digging tool as a wall is (`dig.rs`: the terrain's `hardness`,
//!   between a stone wall's and granite's), broken by a blast that beats it, and **kicked**
//!   ([`World::kick_statue`]): a door's kick, [`kick_breaks`] with the kicker's force against the
//!   terrain's sturdiness ([`crate::door::STATUE_STURDINESS`], well above a door's), one draw of
//!   the gameplay stream; every kick earns the marks a door's does (`Use::Kick`: Athletics and a
//!   Strength mark). Broken, however, it falls to rubble ([`World::break_wall`]), and what it held
//!   spills onto the rubble ([`World::statue_gone`], from `set_terrain`, whatever changes the
//!   terrain).
//! - **Seen**: look mode names it by what it was ("a statue of a dwarf, mouth open",
//!   `PerceptionView::statue_at`), never what it holds; `truth` lists them.
//!
//! The player turned to stone dies (the death screen's line, Stiffening's `death`); no statue is
//! laid for them, as the run ends there.

use super::{Actor, Spent, World};
use crate::action::{Breach, Event};
use crate::content::{ActionKind, Content, CreatureId};
use crate::door::kick_breaks;
use crate::geom::Pos;
use crate::noise::Act;
use crate::object::{self, Stack};
use crate::skills::Use;
use serde::{Deserialize, Serialize};

/// A statue on a cell (ground truth but for what it was, which the player sees).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statue {
    /// The creature type it was.
    pub of: CreatureId,
    /// What it held, top first: what the creature carried, then the pile it stood on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holds: Vec<Stack>,
}

impl Statue {
    /// Whether its creature type is in `content` (a guard for loaded saves; what it holds is
    /// checked with the piles).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        usize::from(self.of.0) < content.creature_defs().len()
    }
}

impl World {
    /// `actor`, a creature just stoned to death, becomes a statue where it fell, holding
    /// `dropped` (taken out of it) and the pile beneath (module docs). Returns whether it was
    /// laid; `false` leaves `dropped` to fall as usual.
    pub(super) fn lay_statue(&mut self, actor: &Actor, dropped: &mut Vec<Stack>) -> bool {
        let content = self.content.clone();
        let Some(terrain) = content.statue_terrain() else {
            return false;
        };
        let p = actor.pos;
        if actor.level != self.state.current_level || !self.holds_statue(p) {
            return false;
        }
        let level = self.state.current_mut();
        let mut holds = std::mem::take(dropped);
        object::add_to_pile(&mut holds, level.objects_at(p), &content);
        level.set_pile(p, Vec::new());
        self.set_terrain(p, terrain);
        let statue = Statue {
            of: actor.creature,
            holds,
        };
        self.state.current_mut().statues.insert(p, statue);
        true
    }

    /// Whether a statue can stand at `p` on the current level: plain open ground, with nothing
    /// of the cell's own (a trap, a lock, a fire) to bury (module docs).
    fn holds_statue(&self, p: Pos) -> bool {
        let level = self.state.current();
        let Some(def) = level.terrain.get(p).map(|t| self.content.terrain(*t)) else {
            return false;
        };
        def.walkable
            && !def.is_door()
            && def.stairs.is_none()
            && def.chest.is_none()
            && def.light.is_none()
            && def.ice.is_none()
            && def.freeze.is_none()
            && level.stairs_at(p).is_none()
            && !level.traps.contains_key(&p)
            && !level.locks.contains_key(&p)
            && !level.fires.contains_key(&p)
            && !level.statues.contains_key(&p)
    }

    /// The terrain at `p` on the current level changed (`set_terrain`): a statue there that it
    /// no longer is is gone, and what it held spills onto the cell, under what lies there.
    pub(super) fn statue_gone(&mut self, p: Pos) {
        let content = self.content.clone();
        let level = self.state.current_mut();
        let still = (level.terrain.get(p)).is_some_and(|t| content.terrain(*t).statue.is_some());
        if still {
            return;
        }
        let Some(statue) = level.statues.remove(&p) else {
            return;
        };
        let mut pile = statue.holds;
        object::add_to_pile(&mut pile, level.objects_at(p), &content);
        level.set_pile(p, pile);
    }

    /// The player kicks the statue at `p` (module docs): a banging noise and the marks a door's
    /// kick earns, and with a chance of their force in its sturdiness it falls to rubble.
    pub(super) fn kick_statue(&mut self, p: Pos, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let price = self.price_of(player, ActionKind::Kick);
        let def = self.content.terrain(self.state.current().terrain[p]);
        let (name, sturdiness) = (def.name.clone(), def.statue.unwrap_or(u8::MAX));
        let force = self.kick_force();
        let broken = kick_breaks(force, sturdiness, &mut self.state.rng);
        if self.trains(Use::Kick) {
            self.train(Use::Kick);
        }
        self.make_noise(player, Act::Kick(p));
        if broken {
            self.break_wall(p, Breach::Kicked, events);
            self.say(format!("The {name} cracks and falls to pieces!"));
        } else {
            events.push(Event::DoorKicked { pos: p });
            self.say(format!(
                "THUD! Your foot meets the stone; the {name} stands."
            ));
        }
        Spent::Turn(price)
    }
}
