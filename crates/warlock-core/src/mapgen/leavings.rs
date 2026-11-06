//! Leavings (docs/design/monster-dread.md §9 channel 3; `content/leaving.rs`): about each
//! creature a population draw or a vault's monster pool placed whose type has `leavings`, the
//! generator lays each kind's count ([`Leaving::count`]) on cells within [`NEAR`] (Chebyshev) of
//! it, so a room with three statues is a room with a basilisk.
//!
//! Drawn from the level's population stream, after its draws ([`super::population::populate`]),
//! so nothing drawn before moves. A leaving goes on plain floor (the palette's) that nothing else
//! claims: no creature or the walk it keeps, no object, trap, light, lock, chest, shop, or room
//! kept clear of the population. A statue (solid) stands only where all eight cells about it are
//! open ground, so it never cuts a way; it is a nobody of the floor's own population (not unique,
//! not dread, with a body), holding nothing. Torn cloth lies on its cell.

use super::Palette;
use crate::content::{Content, CreatureId, CreatureSpawn, Laid, Leaving, LevelTemplate, Spawning};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::object::{Stack, add_to_pile};
use crate::rng::Rng;
use crate::world::Statue;
use std::collections::BTreeSet;

/// How far (Chebyshev) from its creature a leaving is laid.
pub const NEAR: i32 = 4;

/// Lays the leavings about `placed` (module docs).
pub(super) fn lay(
    template: &mut LevelTemplate,
    content: &Content,
    rng: &mut Rng,
    (pal, spawning, stamped): (Palette, &Spawning, &Grid<bool>),
    placed: &[CreatureSpawn],
) {
    if placed
        .iter()
        .all(|s| content.creature(s.creature).leavings.is_empty())
    {
        return;
    }
    // The cells creatures stand on and walk.
    let mut claimed: BTreeSet<Pos> = BTreeSet::new();
    for spawn in template.creatures.iter().chain(placed) {
        let mut p = spawn.pos;
        claimed.insert(p);
        for &d in &spawn.route {
            p = p.step(d);
            claimed.insert(p);
        }
    }
    let nobodies = nobodies(content, spawning);
    for spawn in placed {
        for &leaving in &content.creature(spawn.creature).leavings {
            let Some(laid) = content.leaving(leaving) else {
                continue;
            };
            let (lo, hi) = leaving.count();
            let count = rng.range(lo, hi);
            let mut cells = near(template, spawn.pos);
            // A vault's own lay theirs inside it; others keep off what vaults stamped.
            let inside = stamped.get(spawn.pos).copied().unwrap_or(false);
            cells.retain(|p| inside || !stamped.get(*p).copied().unwrap_or(false));
            for _ in 0..count {
                cells.retain(|&p| fits(template, content, pal, &claimed, leaving, p));
                if cells.is_empty() {
                    break;
                }
                let p = cells.swap_remove(rng.index(cells.len()));
                match laid {
                    Laid::Terrain(t) => {
                        if leaving == Leaving::Statue {
                            let Some(&of) = (!nobodies.is_empty())
                                .then(|| &nobodies[rng.index(nobodies.len())])
                            else {
                                break;
                            };
                            let holds = Vec::new();
                            template.statues.insert(p, Statue { of, holds });
                        }
                        template.terrain[p] = t;
                    }
                    Laid::Object(o) => {
                        let pile = template.objects.entry(p).or_default();
                        add_to_pile(pile, &[Stack::one(o)], content);
                    }
                }
                claimed.insert(p);
            }
        }
    }
}

/// The types a statue may be of: the floor's own population, not unique or dread, with a body.
fn nobodies(content: &Content, spawning: &Spawning) -> Vec<CreatureId> {
    let mut all: Vec<CreatureId> = (spawning.population.iter())
        .copied()
        .filter(|&c| {
            let def = content.creature(c);
            !def.unique && !def.dread && def.corpse && def.leavings.is_empty()
        })
        .collect();
    all.sort();
    all.dedup();
    all
}

/// The cells within [`NEAR`] of `at` on the map, in row order.
fn near(template: &LevelTemplate, at: Pos) -> Vec<Pos> {
    let mut cells = Vec::new();
    for y in at.y - NEAR..=at.y + NEAR {
        for x in at.x - NEAR..=at.x + NEAR {
            let p = Pos::new(x, y);
            if template.terrain.get(p).is_some() && p != at {
                cells.push(p);
            }
        }
    }
    cells
}

/// Whether `leaving` may be laid at `p` (module docs).
fn fits(
    template: &LevelTemplate,
    content: &Content,
    pal: Palette,
    claimed: &BTreeSet<Pos>,
    leaving: Leaving,
    p: Pos,
) -> bool {
    let kept_clear = (template.rooms.iter()).any(|r| {
        let effect = content.room_effect(r.effect);
        r.contains(p)
            && (effect.shunned
                || effect.swarm.is_some()
                || effect.hoard.is_some()
                || effect.treasure.is_some())
    });
    let plain = template.terrain.get(p) == Some(&pal.floor)
        && !claimed.contains(&p)
        && !template.objects.contains_key(&p)
        && !template.traps.contains_key(&p)
        && !template.lights.contains_key(&p)
        && !template.locks.contains_key(&p)
        && !template.chests.contains_key(&p)
        && !template.shops.iter().any(|s| s.contains(p))
        && !kept_clear;
    plain
        && (leaving != Leaving::Statue
            || Dir::ALL.iter().all(|&d| {
                let q = p.step(d);
                let open = template.terrain.get(q).is_some_and(|t| {
                    let def = content.terrain(*t);
                    def.walkable && !def.is_door()
                });
                open && !template.statues.contains_key(&q)
            }))
}
