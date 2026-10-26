//! Incorporeal creatures (spec §8 "Incorporeal creatures"): ghosts and their kind pass through
//! solid terrain and may stand inside it.
//!
//! - **Where**: any cell its type may walk, and any *solid* cell ([`LevelState::phaseable`]:
//!   terrain neither walkable nor transparent, so walls, rock, trees, and closed doors, locked or
//!   not) off the level's edge. It slips through a closed door without opening it. Transparent
//!   obstacles (water it cannot walk, lava, a fountain) stop it as they stop anyone.
//! - **Defence**: inside a solid cell it adds its type's `wall_defence` in the Wall category
//!   ([`World::defence_of`] reads [`World::wall_defence`]); a miss may be put down to the stone.
//! - **Attacks**: it strikes out of the wall like any creature; the player strikes at one they
//!   see inside a wall by moving into it (`travel.rs`), and a missile that reaches its cell
//!   attacks it there (`shoot.rs`).
//! - **Footing**: nothing of it presses down: it sets off no trap and breaks no ice
//!   (`ai.rs::creature_steps`).
//! - **Death**: what it carried falls on the nearest walkable cell ([`World::landing`]).
//!
//! Sight needs no rule of its own: the player sees it when they see its cell, as every wall cell
//! is seen (its face lit, or within darkvision; `perception.rs`), and creatures see one in a wall
//! by their rule for opaque cells (`sight::sees`: within darkvision).
//!
//! Hook, not a rule yet: immunity to weapons that are not magical would go where a blow's profile
//! meets its target (`fight.rs::resolve`, which knows the weapon); spells and bolts stop at a
//! wall (`spell.rs`, `read.rs`) and so do not reach one inside it. Noise needs nothing: a sound
//! made inside a wall spreads from its cell into the open (`noise::propagate`).

use super::{ActorId, LevelState, World};
use crate::combat::{Category, Defence};
use crate::content::{Content, CreatureId, Incorporeal};
use crate::geom::Pos;

/// What a path through a solid cell costs an incorporeal creature on top of a step: it keeps to
/// open ground when the way round is no longer, and cuts through stone when it is.
pub(super) const WALL: u32 = 1;

impl LevelState {
    /// Whether an incorporeal creature may pass through cell `p`: solid terrain (neither walkable
    /// nor transparent) on the level and off its edge.
    pub(crate) fn phaseable(&self, content: &Content, p: Pos) -> bool {
        !self.on_edge(p)
            && (self.terrain.get(p)).is_some_and(|t| {
                let def = content.terrain(*t);
                !def.walkable && !def.transparent
            })
    }
}

impl Content {
    /// The incorporeal nature of creature type `id`, if it has one.
    pub fn incorporeal(&self, id: CreatureId) -> Option<Incorporeal> {
        self.creature(id).incorporeal
    }
}

impl World {
    /// Whether actor `id` stands inside solid terrain (an incorporeal creature in a wall).
    pub(crate) fn in_wall(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| {
            (self.state.levels.get(&a.level)).is_some_and(|level| {
                (level.terrain.get(a.pos)).is_some_and(|t| !self.content.terrain(*t).walkable)
            })
        })
    }

    /// `defence` with what the wall around actor `id` adds: its type's `wall_defence` in the
    /// Wall category while it stands inside solid terrain; else `defence` as it is.
    pub(super) fn wall_defence(&self, id: ActorId, defence: Defence) -> Defence {
        let Some(actor) = self.state.actors.get(&id) else {
            return defence;
        };
        match self.content.incorporeal(actor.creature) {
            Some(i) if i.wall_defence != 0 && self.in_wall(id) => {
                defence.plus(Category::Wall, i.wall_defence)
            }
            _ => defence,
        }
    }

    /// Where what falls from cell `p` of level `level` comes to lie: `p` itself if it is
    /// walkable, else the nearest walkable cell (by Chebyshev rings, each in row order), else `p`.
    pub(super) fn landing(&self, level: &str, p: Pos) -> Pos {
        let Some(level) = self.state.levels.get(level) else {
            return p;
        };
        let walkable = |q: Pos| level.is_walkable(&self.content, q);
        if walkable(p) {
            return p;
        }
        let reach = level.width().max(level.height());
        for r in 1..=reach {
            for y in p.y - r..=p.y + r {
                for x in p.x - r..=p.x + r {
                    let q = Pos::new(x, y);
                    if q.chebyshev(p) == r && walkable(q) {
                        return q;
                    }
                }
            }
        }
        p
    }
}
