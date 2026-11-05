//! The initial population of a generated floor (spec §7 "Danger levels"): a floor whose branch
//! gives a danger is laid down full, its creatures drawn by the danger from the branch's
//! population, now and then a stray from the generic creature list or one from deeper
//! ([`danger::draw`]).
//!
//! How many: the branch's `population_count` for the floor's walkable cells
//! ([`crate::content::danger::Count::for_cells`]: by default 35 per 1000, 25 to 80), on top of
//! the generator's own patrols. Everything comes from the level's own stream
//! [`Rng::population`] (`population/<level id>`), drawn after its loot and before its gear, so
//! the map, traps, secrets, and loot are the same with any population. Per creature: its type,
//! its cell, and its route. The cell is plain floor (the palette's), free of creatures, lights,
//! traps, and shops, outside a sanctuary or a teeming den (its sleepers come on top,
//! `den.rs`), and at least [`AWAY`] cells (Chebyshev) from every staircase (the player arrives
//! on one); the route is a short out-and-back walk over cells as free (no door, no trap), else
//! the creature keeps a post. A unique type comes once a run: not
//! if the run has met it, and once on the floor. The creatures roll their gear afterwards with
//! the rest ([`crate::gear::arm`]).
//!
//! A vault's monster pools ([`VaultMonsters`]) are drawn the same way, just before it, each from
//! its pool on the cells its legend marks, from their own stream [`Rng::vault_monsters`]. They
//! come on top of the count, and the spawner's cap does not count them
//! ([`LevelTemplate::outside_cap`]). A marked cell is left to them whether it is drawn or not,
//! but a hidden trap may lie on one (the traps come first, the same with any creatures): it stays
//! empty.

use super::{Palette, Visit, out_and_back};
use crate::content::danger::{self, Spawning};
use crate::content::{
    Content, CreatureId, CreatureSpawn, LevelTemplate, MonsterPool, VaultMonsters,
};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;
use std::collections::BTreeSet;

/// The fewest cells (Chebyshev) between a staircase and a creature of the initial population.
pub const AWAY: i32 = 8;

/// Lays the initial population of generated level `template` down: `danger` its danger,
/// `spawning` its branch's rules, `visit` whom it is made for (the uniques they have met), and
/// `monsters` the monster pools its vaults stamped.
pub(super) fn populate(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    (pal, danger, spawning): (Palette, u8, &Spawning),
    (visit, monsters): (&Visit, &[VaultMonsters]),
) {
    if danger == 0 {
        return;
    }
    let mut rng = Rng::population(seed, &template.id);
    let walkable = (template.terrain.iter())
        .filter(|(_, t)| content.terrain(**t).walkable)
        .count();
    let count = spawning
        .count
        .for_cells(u32::try_from(walkable).unwrap_or(u32::MAX));
    // Kept clear: a sanctuary (shunned), a teeming den (its sleepers come on top, `den.rs`), a
    // tempting hoard (unguarded until it seals, `world/hoard.rs`), a treasure room (sealed).
    let shunned = |p: Pos| {
        (template.rooms.iter()).any(|r| {
            let effect = content.room_effect(r.effect);
            r.contains(p)
                && (effect.shunned
                    || effect.swarm.is_some()
                    || effect.hoard.is_some()
                    || effect.treasure.is_some())
        })
    };
    let free_of = |p: Pos| {
        !template.traps.contains_key(&p)
            && !template.lights.contains_key(&p)
            && !template.shops.iter().any(|s| s.contains(p))
            && !shunned(p)
    };
    let near_stairs = |p: Pos| template.stairs.iter().any(|s| s.pos.chebyshev(p) < AWAY);
    let vaults = monsters.iter().flat_map(|m| m.cells.iter().copied());
    let taken: BTreeSet<Pos> = (template.creatures.iter().map(|c| c.pos))
        .chain(vaults)
        .collect();
    let mut cells: Vec<Pos> = (template.terrain.iter())
        .filter(|&(p, t)| *t == pal.floor && free_of(p) && !near_stairs(p) && !taken.contains(&p))
        .map(|(p, _)| p)
        .collect();
    let walks = |p: Pos| {
        template.terrain.get(p).is_some_and(|t| {
            let def = content.terrain(*t);
            def.walkable && !def.is_door()
        }) && free_of(p)
    };
    let mut met = visit.uniques.cloned().unwrap_or_default();
    let mut placed = Vec::new();
    let campaign = (content.branch_level(&template.id)).and_then(|(d, _)| d.campaign.as_deref());
    let mut vault_rng = Rng::vault_monsters(seed, &template.id);
    for group in monsters {
        let tagged = group.pool.types(content, campaign);
        let (own, generic) = match group.pool {
            MonsterPool::Level => (spawning.population.as_slice(), spawning.generic.as_slice()),
            MonsterPool::Tags(_) => (tagged.as_slice(), tagged.as_slice()),
        };
        let mut cells = group.cells.clone();
        cells.retain(|p| !template.traps.contains_key(p));
        for _ in 0..group.count {
            if cells.is_empty() {
                break;
            }
            let own = content.candidates(own, &met);
            let generic = content.candidates(generic, &met);
            let odds = &spawning.odds;
            let Some(creature) = danger::draw(&mut vault_rng, &own, &generic, danger, odds) else {
                break;
            };
            let pos = cells.swap_remove(vault_rng.index(cells.len()));
            let route = route_from(&mut vault_rng, pos, walks);
            met_one(content, creature, &mut met);
            placed.push(spawn_of(creature, pos, route));
        }
    }
    let outside: BTreeSet<Pos> = placed.iter().map(|s| s.pos).collect();
    // The generic list holds no uniques: its candidates stay the same all the floor long.
    let generic = content.candidates(&spawning.generic, &met);
    for _ in 0..count {
        let own = content.candidates(&spawning.population, &met);
        let odds = &spawning.odds;
        let Some(creature) = danger::draw(&mut rng, &own, &generic, danger, odds) else {
            break;
        };
        if cells.is_empty() {
            break;
        }
        let pos = cells.swap_remove(rng.index(cells.len()));
        let route = route_from(&mut rng, pos, walks);
        met_one(content, creature, &mut met);
        placed.push(spawn_of(creature, pos, route));
    }
    template.creatures.extend(placed);
    template.outside_cap = outside;
}

/// A short out-and-back walk from `pos` over cells that `walks`, else a post (none).
fn route_from(rng: &mut Rng, pos: Pos, walks: impl Fn(Pos) -> bool) -> Vec<Dir> {
    let len = rng.range(4, 10);
    out_and_back(rng, pos, len, walks).unwrap_or_default()
}

/// A unique type just drawn leaves the draws after it.
fn met_one(content: &Content, creature: CreatureId, met: &mut BTreeSet<String>) {
    let def = content.creature(creature);
    if def.unique {
        met.insert(def.id.clone());
    }
}

/// A creature drawn for a generated floor: its gear is rolled with the rest.
fn spawn_of(creature: CreatureId, pos: Pos, route: Vec<Dir>) -> CreatureSpawn {
    CreatureSpawn {
        creature,
        faction: None,
        alignment: None,
        npc: None,
        pos,
        route,
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: true,
        asleep: false,
        status: Vec::new(),
    }
}
