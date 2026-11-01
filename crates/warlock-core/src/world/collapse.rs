//! Collapsed floors (spec §7 "Special rooms", design §9.13): every step of the **player** onto a
//! cell of a collapsed floor (a room whose effect has a `collapse`, `Here::collapse`; its cracked
//! floor terrain, where the room lays one) has the effect's chance, doubled while they are
//! Burdened or worse, to give way: one draw of the gameplay stream, made only on such a cell, and
//! only while the level has a way down that leads somewhere. Old delver
//! (`TalentMods::collapse_proof`) skips the draw. The player falls to the level the way down leads
//! to (generated as usual on a first visit, [`World::travel`]), landing on a random cell of it
//! (another gameplay draw, [`landing`]), takes the effect's damage (`Harm::Fall`: it can kill,
//! and a death by it is `Cause::Fall`; the generator leaves a sound rim round the cracks, so
//! there is always a way round), and is Staggered unless it killed. Creatures never fall. Nothing
//! new is saved: the fall is a level change like the stairs'.

use super::fight::Harm;
use super::{LevelState, World};
use crate::action::Event;
use crate::burden::Burden;
use crate::content::{Content, StairsDir};
use crate::geom::{Dir, Pos};
use std::collections::BTreeSet;

/// The status a fall leaves the player in.
const STAGGER: &str = "staggered";

impl World {
    /// The player has just stepped onto `at`: on a collapsed floor it may give way under them
    /// (module docs). Returns whether they fell.
    pub(super) fn collapse_step(&mut self, at: Pos, events: &mut Vec<Event>) -> bool {
        let content = self.content.clone();
        let level = self.state.current();
        let Some(collapse) = level.effects_at(&content, at).collapse() else {
            return false;
        };
        let player = self.state.player();
        if player.pos != at || player.level != level.id {
            return false;
        }
        // Only a cracked cell gives way, where the room lays a floor of its own.
        let floor = level
            .room_at(at)
            .and_then(|r| content.room_effect(r.effect).floor);
        if floor.is_some_and(|f| level.terrain[at] != f) {
            return false;
        }
        let Some(below) = level_below(level) else {
            return false;
        };
        if player.talent_mods(&content).collapse_proof {
            return false;
        }
        let burdened = self.load_of(self.state.player).penalty() >= Burden::Burdened;
        let chance = u32::from(collapse.chance) * if burdened { 2 } else { 1 };
        if !self.state.rng.chance(chance.min(100)) {
            return false;
        }
        let from = self.state.current_level.clone();
        if self.visit(&below, Some(&from)).is_err() {
            return false;
        }
        let cells = landing(&self.state.levels[&below], &self.state, &content);
        if cells.is_empty() {
            return false;
        }
        let pick = self
            .state
            .rng
            .below(u32::try_from(cells.len()).unwrap_or(u32::MAX));
        let to = cells[pick as usize];
        self.say("The floor gives way beneath you!");
        if self.travel(&below, Some(to)).is_err() {
            return false;
        }
        events.push(Event::LevelChanged {
            from,
            to: below.clone(),
        });
        let name = self.state.current().name.clone();
        self.say(format!("You fall to {name}."));
        let id = self.state.player;
        let damage = collapse.damage.roll(&mut self.state.rng).max(0);
        if self.hurt(id, damage, Harm::Fall, events) {
            return true;
        }
        if let Some(stagger) = content.status_id(STAGGER) {
            self.afflict(id, stagger, 1, None, events);
        }
        true
    }
}

/// The level the first way down of `level` leads to, if it leads anywhere.
fn level_below(level: &LevelState) -> Option<String> {
    (level.stairs.iter())
        .filter(|s| s.dir == StairsDir::Down)
        .find_map(|s| s.to.clone())
}

/// Where a fall onto `level` may land, in cell order: a walkable cell that is no door, stairs,
/// trap, or ice, where no one stands, outside every shop, that one can walk to from where the
/// level is entered without a locked or secret door (so never inside a vault behind one).
pub(super) fn landing(
    level: &LevelState,
    state: &super::WorldState,
    content: &Content,
) -> Vec<Pos> {
    let Some(start) = level.start.or_else(|| level.stairs.first().map(|s| s.pos)) else {
        return Vec::new();
    };
    // Every cell reached from the start through walkable cells and unlocked doors.
    let passable = |p: Pos| {
        level.terrain.in_bounds(p)
            && (level.is_walkable(content, p)
                || (level.is_door(content, p) && !level.locks.contains_key(&p)))
    };
    let mut seen = BTreeSet::from([start]);
    let mut next = vec![start];
    while let Some(p) = next.pop() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if !seen.contains(&q) && passable(q) {
                seen.insert(q);
                next.push(q);
            }
        }
    }
    let taken: BTreeSet<Pos> = (state.actors.values())
        .filter(|a| a.level == level.id)
        .map(|a| a.pos)
        .collect();
    seen.into_iter()
        .filter(|&p| {
            level.is_walkable(content, p)
                && !level.is_door(content, p)
                && level.stairs_at(p).is_none()
                && !level.traps.contains_key(&p)
                && level.ice_at(p).is_none()
                && !taken.contains(&p)
                && !level.shops.iter().any(|s| s.contains(p))
        })
        .collect()
}
