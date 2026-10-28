//! A generated floor's chests (spec §8 "Chests"; docs/design/chests-and-treasure.md §3.3,
//! `docs/dev/chests.md`): with the branch's `chests.chance` percent a floor gets between
//! `count`'s least and most, each on a cell of open floor, of the floor's item level, locked and
//! trapped by its band's chances, holding what its band's loot table rolls
//! ([`crate::chest::roll_contents`]).
//!
//! Everything comes from the level's chest stream [`Rng::chests`] (`chests/<level id>`), drawn
//! after the rooms' loot and before the gear, so the rest of the level is the same with or
//! without chests. Draws: the chance; if it comes, the count; then per chest its cell, the lock
//! (`chance(locked)`), the trap (`chance(trapped)`, then one draw for which of the branch's
//! traps that fit a fixture, made even when none fits, so a branch without traps draws the
//! same), its contents, and last the mimic draw (`chance(mimic)`): a mimic of the floor's item
//! level stands there instead (spec §8 "Mimics"; `docs/dev/creatures/mimics.md`), disguised,
//! carrying exactly what the chest would have held; the lock and the trap drawn for it are
//! dropped (a mimic is neither), and nothing more is drawn, so the stream is the same either
//! way.
//!
//! A chest's cell is plain floor whose eight neighbours are all walkable and none a door, so a
//! chest never stands in a doorway or a corridor and never cuts a room apart; at least
//! [`AWAY`] from every staircase (the arrival room keeps clear), never in a vault, a shop, or a
//! special room with a feature, sleepers, or a hoard, and nowhere anything lies, stands,
//! shines, patrols, or is trapped. Two chests never touch.

use super::{Palette, patrolled};
use crate::chest::{Chest, ChestState, roll_contents};
use crate::content::{Content, CreatureSpawn, DungeonDef, LevelTemplate, TerrainId, Trap, TrapId};
use crate::door::Lock;
use crate::equipment::Equipment;
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::magic::loot::Luck;
use crate::rng::Rng;

/// The fewest cells (Chebyshev) between a chest and a staircase.
pub const AWAY: i32 = 6;

/// Lays the chests of generated level `template` of branch `def` (see the module docs), the
/// finder's `luck` reaching their contents; `stamped` marks the vaults' cells.
pub(super) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    (seed, luck): (u64, Luck),
    (pal, def): (Palette, &DungeonDef),
    stamped: &Grid<bool>,
) {
    let Some(shut) = content.chest_terrain(ChestState::Shut) else {
        return;
    };
    if content.chests().bands.is_empty() || def.chests.chance == 0 {
        return;
    }
    let mut rng = Rng::chests(seed, &template.id);
    if !rng.chance(u32::from(def.chests.chance)) {
        return;
    }
    let [least, most] = def.chests.count.map(i32::from);
    let count = rng.range(least, most);
    let mut free = cells(template, content, pal, stamped);
    let traps: Vec<_> = (def.traps.iter().copied())
        .filter(|&t| content.trap(t).effect.fits_fixture())
        .collect();
    for _ in 0..count {
        if free.is_empty() {
            break;
        }
        let at = free.swap_remove(rng.index(free.len()));
        free.retain(|p| p.chebyshev(at) > 1);
        lay(template, content, &mut rng, (at, shut), (&traps, luck));
    }
}

/// Lays one chest of the floor's item level on `at` (its `shut` terrain), drawing from `rng`
/// its lock, its trap (one of `traps`, those of the branch that fit a fixture), its contents
/// with the finder's `luck`, and the mimic draw (see the module docs).
pub(super) fn lay(
    template: &mut LevelTemplate,
    content: &Content,
    rng: &mut Rng,
    (at, shut): (Pos, TerrainId),
    (traps, luck): (&[TrapId], Luck),
) {
    let level = template.item_level;
    let Some(band) = content.chests().band_of(level) else {
        return;
    };
    if rng.chance(u32::from(band.locked)) {
        template.locks.insert(at, Lock::new(None));
    }
    if rng.chance(u32::from(band.trapped)) {
        let i = rng.index(traps.len().max(1));
        if let Some(&kind) = traps.get(i) {
            let trap = Trap::new(kind, content.trap(kind), false, None);
            template.traps.insert(at, trap);
        }
    }
    let holds = roll_contents(content, rng, level, luck);
    // The mimic draw (docs/design/chests-and-treasure.md §5.3): a mimic of the floor's level
    // stands in for the chest, carrying what it would have held.
    let mimic = rng.chance(u32::from(band.mimic));
    if let Some(creature) = content.mimic_at(level).filter(|_| mimic) {
        template.locks.remove(&at);
        template.traps.remove(&at);
        template.creatures.push(CreatureSpawn {
            creature,
            faction: None,
            alignment: None,
            npc: None,
            pos: at,
            route: Vec::new(),
            carry: holds,
            equipment: Equipment::default(),
            rolls_gear: false,
            asleep: false,
        });
        return;
    }
    template.terrain.set(at, shut);
    template.chests.insert(at, Chest { level, holds });
}

/// The cells a chest may stand on, in row order (see the module docs).
fn cells(
    template: &LevelTemplate,
    content: &Content,
    pal: Palette,
    stamped: &Grid<bool>,
) -> Vec<Pos> {
    let patrolled = patrolled(template);
    let open = |q: Pos| {
        (template.terrain.get(q)).is_some_and(|&t| {
            let def = content.terrain(t);
            def.walkable && !def.is_door()
        })
    };
    let kept = |p: Pos| {
        template.rooms.iter().any(|r| {
            let effect = content.room_effect(r.effect);
            r.contains(p)
                && (effect.feature.is_some() || effect.swarm.is_some() || effect.hoard.is_some())
        })
    };
    (template.terrain.iter())
        .filter(|&(p, t)| {
            *t == pal.floor
                && !stamped[p]
                && Dir::ALL.iter().all(|d| open(p.step(*d)))
                && template.stairs.iter().all(|s| s.pos.chebyshev(p) >= AWAY)
                && template.player_start.is_none_or(|s| s.chebyshev(p) >= AWAY)
                && !patrolled.contains(&p)
                && !template.objects.contains_key(&p)
                && !template.lights.contains_key(&p)
                && !template.traps.contains_key(&p)
                && !template.locks.contains_key(&p)
                && !template.shops.iter().any(|s| s.contains(p))
                && !kept(p)
        })
        .map(|(p, _)| p)
        .collect()
}
