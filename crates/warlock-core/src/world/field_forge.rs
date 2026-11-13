//! Field forge (spec §8 "Class abilities", §7 "Special rooms"; the craftsperson design §7.7):
//! the craftsperson's level-32 active, [`ClassActive::FieldForge`], aimed in a direction
//! (`use:field_forge:E`).
//!
//! - **Takes**: one ingot of any metal carried (a [`crate::piece`] of a material of the `metal`
//!   group, the first such stack in the pack that is not unpaid goods), spent at the end.
//! - **The cell**: the one the way named, beside the player: open ground ([`open_ground`]:
//!   walkable floor that is no door, chest, stairs, grave, water, ice, glowing fungus, or shop
//!   floor), with nobody on it, no trap the player knows of, and nothing lying there; a cell of
//!   a level, never the overworld. Refused with the reason, taking no time.
//! - **The work**: the talent's `seconds` (an hour) in turns of the `craft` action
//!   ([`World::keep_at`], the rest's machinery; the last turn cut short to end on time); a
//!   creature coming into view, a message, or a blow stops it, and nothing is made (the time is
//!   spent, the ingot kept).
//! - **Done**: no check (an hour's labour, not a craft). The cell and the ingot are asked again;
//!   the ingot is spent and the cell becomes the `makeshift_anvil` terrain, for good (a terrain of
//!   the level, saved with it): "You set the block on a heap of stones: a makeshift anvil." It
//!   works as the anvil ([`crate::content::Content::works_as`]), so every recipe at the anvil,
//!   Temper, and the walk-in are made there. A pick clears it to floor, and nothing is mined
//!   out of it (`world/mine.rs`).
//!
//! No draws at all.

use super::rest::Ended;
use super::shop::unpaid_parts;
use super::{LevelState, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Content, LevelKind, TerrainId};
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use crate::time::Duration;

/// The terrain a field forge makes.
pub const ANVIL: &str = "makeshift_anvil";

/// A field forge under way: the cell it is set on and its time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Forging {
    at: Pos,
    seconds: u32,
}

/// Whether `p` on `level`, of terrain `terrain`, is open ground an anvil may be set on (module
/// docs): walkable, and no door, chest, stairs, grave, water or ice, glowing fungus, or shop
/// floor. Who or what stands there is asked apart.
pub(crate) fn open_ground(
    content: &Content,
    level: &LevelState,
    terrain: TerrainId,
    p: Pos,
) -> bool {
    let def = content.terrain(terrain);
    let fungus = content.herbs().is_some_and(|h| h.terrain == terrain);
    def.walkable
        && !def.is_door()
        && def.chest.is_none()
        && def.stairs.is_none()
        && !def.grave
        && def.freeze.is_none()
        && def.ice.is_none()
        && !def.swim
        && !fungus
        && level.stairs_at(p).is_none()
        && !level.shops().iter().any(|s| s.contains(p))
}

/// Whether `stack` is an ingot: a piece of a material of the `metal` group.
fn is_ingot(content: &Content, stack: &Stack) -> bool {
    content.object(stack.object).id == crate::piece::ID
        && (stack.parts(content).first())
            .is_some_and(|p| content.material(p.material).group.as_deref() == Some("metal"))
}

impl World {
    /// The first stack of ingots the player carries and may use (not all of it unpaid goods).
    fn ingot(&self) -> Option<usize> {
        let inventory = &self.state.player().inventory;
        let unpaid = match self.shop_here() {
            Some(i) => unpaid_parts(inventory, &self.state.current().shops[i].unpaid),
            None => Vec::new(),
        };
        (inventory.iter().enumerate()).position(|(i, s)| {
            is_ingot(&self.content, s) && unpaid.get(i).copied().unwrap_or(0) < s.count
        })
    }

    /// Why the cell at `at` will not take an anvil, as the player is told; `None` when it will.
    fn forge_ground(&self, at: Pos) -> Option<&'static str> {
        let level = self.state.current();
        let open = (level.terrain_at(at)).is_some_and(|t| open_ground(&self.content, level, t, at));
        if !open {
            return Some("There is no open ground there to set an anvil on.");
        }
        if self.state.actor_at(&self.state.current_level, at).is_some() {
            return Some("Something is in the way.");
        }
        if level.trap_at(at).is_some_and(|t| t.known) {
            return Some("There is a trap there.");
        }
        if !level.objects_at(at).is_empty() {
            return Some("Clear the ground there first.");
        }
        None
    }

    /// The player begins to set an ingot on a heap of stones the way `dir`; refused, taking no
    /// time, without the talent, on the overworld, without an ingot, or where the ground will not
    /// take it (module docs).
    pub(super) fn start_field_forge(&mut self, dir: Dir) -> Spent {
        let content = self.content.clone();
        let Some(seconds) = self.state.player().talent_mods(&content).field_forge else {
            return Spent::Nothing;
        };
        if self.state.current().kind() == LevelKind::Overworld {
            self.say("You cannot set up a forge out here.");
            return Spent::Nothing;
        }
        if self.ingot().is_none() {
            self.say("You have no ingot to set.");
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        if let Some(why) = self.forge_ground(at) {
            self.say(why);
            return Spent::Nothing;
        }
        self.say("You heap up stones to set an ingot on.");
        let seconds = self.craft_seconds(seconds);
        Spent::FieldForge(Forging { at, seconds })
    }

    /// Works turn after turn until the time is up or something stops it, then sets the anvil
    /// (module docs).
    pub(super) fn field_forge(&mut self, work: Forging, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Craft);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.forged(work.at),
            Ended::Came(came) => {
                let text = format!("{}; you leave the anvil unfinished.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You leave the anvil unfinished."),
        }
    }

    /// The time is up: the ingot spent and the anvil set, if both still hold.
    fn forged(&mut self, at: Pos) {
        let content = self.content.clone();
        let Some(anvil) = content.terrain_id(ANVIL) else {
            return;
        };
        let Some(item) = self.ingot() else {
            self.say("You have no ingot to set.");
            return;
        };
        if let Some(why) = self.forge_ground(at) {
            self.say(why);
            return;
        }
        let pack = &mut self.player_mut().inventory;
        pack[item].count -= 1;
        if pack[item].count == 0 {
            pack.remove(item);
        }
        self.set_terrain(at, anvil);
        self.say("You set the block on a heap of stones: a makeshift anvil.");
    }
}
