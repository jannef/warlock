//! Ore veins (#1390; `docs/dev/mapgen.md`, `docs/dev/digging.md`): now and then a branch's floor
//! has a few wall sections of an ore, drawn in its colour, which always give it to a miner
//! (`world/crafting/mine.rs`). `data/mining.toml`'s `[veins]` table says how often (`chance`, percent of
//! floors) and how many (`count`); an ore row's `vein` its colour and size. Which ore is drawn by
//! the mining table's weights at the floor's danger, the branch's `ores` allow-list kept
//! ([`crate::content::MiningRules::vein_table`]), so the precious ores lie deeper and mithril and
//! adamantium only very deep.
//!
//! Laid once the floor is otherwise finished, from its own stream `veins/<level id>`
//! ([`Rng::veins`]), so a floor without veins is exactly its old self and one with them differs
//! only in those walls. Draws, in order: the chance; the count; then per vein its ore, its first
//! cell (a wall the open ground touches, so a vein shows), its size, and its growth (a random cell
//! of it, a random side) through rock, walls and the solid rock behind them alike: a vein runs on
//! into the rock, bared as it is dug. A vein never takes a vault's cell, a cell on the level's
//! edge, a secret door (which looks like the wall it was hidden in, and was a door before: so the
//! veins are the same with the secret doors or without them), or the ring round a special room
//! (a sealed room's breach is one of its walls).

use super::{CARDINALS, Palette};
use crate::content::{Content, DungeonDef, LevelTemplate, MaterialId};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::magic::loot::pick_weighted;
use crate::rng::Rng;

/// Tries at growing a vein, per cell it should have, before it stays smaller.
const TRIES_PER_CELL: usize = 8;

/// Lays the floor's veins, if its stream says it has any (the module docs).
pub(super) fn lay(
    template: &mut LevelTemplate,
    content: &Content,
    (seed, pal): (u64, Palette),
    (def, danger): (&DungeonDef, u32),
    stamped: &Grid<bool>,
) {
    let Some(rules) = content.mining() else {
        return;
    };
    let Some(veins) = rules.veins else {
        return;
    };
    let mut rng = Rng::veins(seed, &template.id);
    if !rng.chance(u32::from(veins.chance)) {
        return;
    }
    let ores: Option<Vec<MaterialId>> = (def.ores.as_ref())
        .map(|list| list.iter().filter_map(|m| content.material_id(m)).collect());
    let table = rules.vein_table(danger, ores.as_deref());
    if table.is_empty() {
        return;
    }
    let (w, h) = (template.terrain.width(), template.terrain.height());
    let near_room = |p: Pos| {
        (template.rooms.iter()).any(|r| {
            p.x >= r.min.x - 1 && p.x <= r.max.x + 1 && p.y >= r.min.y - 1 && p.y <= r.max.y + 1
        })
    };
    let rock = |terrain: &Grid<_>, p: Pos| {
        p.x > 0
            && p.y > 0
            && p.x < w - 1
            && p.y < h - 1
            && (terrain[p] == pal.wall || terrain[p] == pal.void)
            && !stamped.get(p).copied().unwrap_or(false)
            && !template.secrets.contains_key(&p)
            && !near_room(p)
    };
    let count = rng.range(i32::from(veins.count[0]), i32::from(veins.count[1]));
    for _ in 0..count {
        let Some(&row) = pick_weighted(&mut rng, &table) else {
            return;
        };
        let Some(vein) = rules.finds[row].vein else {
            continue;
        };
        // A vein starts in a wall, where it shows.
        let starts: Vec<Pos> = (0..h)
            .flat_map(|y| (0..w).map(move |x| Pos::new(x, y)))
            .filter(|&p| template.terrain[p] == pal.wall && rock(&template.terrain, p))
            .collect();
        if starts.is_empty() {
            return;
        }
        let start = starts[rng.index(starts.len())];
        let size = rng.range(i32::from(vein.size[0]), i32::from(vein.size[1])) as usize;
        let mut cells = vec![start];
        template.terrain[start] = vein.terrain;
        for _ in 0..size * TRIES_PER_CELL {
            if cells.len() >= size {
                break;
            }
            let from = cells[rng.index(cells.len())];
            let p = from.step(CARDINALS[rng.index(CARDINALS.len())]);
            if rock(&template.terrain, p) {
                template.terrain[p] = vein.terrain;
                cells.push(p);
            }
        }
    }
}
