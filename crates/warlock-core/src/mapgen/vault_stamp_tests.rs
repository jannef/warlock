//! Every shared dungeon vault stamps on real floors of a rooms branch and of a cave branch, and
//! its doors lead out (#1072). A vault added to `assets/vaults/` joins the test by being there.
//! A vault stamps only on a floor whose danger its `danger` holds (#1073); one whose `danger`
//! keeps it off some of those floors is also looked for on deeper ones (#1077). What its monster
//! pools give is checked as stamped: sleepers asleep, every carried key opening its lock and
//! reachable without it, and patrols walking their route turned with the vault.

use super::vault_turn_tests::{reached, stamps};
use super::*;
use crate::content::{Stance, Turn};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::RangeInclusive;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

/// A rooms branch and a cave branch of the first campaign (both in a release build too), each of
/// dangers 6 to 9.
const BRANCHES: [&str; 2] = ["banfeld", "wulfeshol"];
const DANGERS: RangeInclusive<u8> = 6..=9;
const SEEDS: u64 = 300;
/// Deeper floors for a vault whose `danger` keeps it off some of [`BRANCHES`]': the endless
/// branch's rooms floors (its danger is its depth) and Fyrdelf's caves, each of dangers 9 to 16.
const DEEP: [(&str, RangeInclusive<u32>); 2] = [("grundleas", 9..=16), ("fyrdelf", 5..=12)];
/// Fewer seeds there: eight floors a seed, and only the deep vaults looked for.
const DEEP_SEEDS: u64 = 100;

#[test]
fn every_shared_dungeon_vault_stamps_on_rooms_and_caves_floors() {
    let shared: Vec<&Vault> = (CONTENT.vaults().iter())
        .filter(|v| !v.is_outdoor() && v.campaign.is_none())
        .collect();
    assert!(shared.len() >= 25, "{}", shared.len());
    let deep: Vec<&Vault> = (shared.iter().copied())
        .filter(|v| !DANGERS.clone().all(|d| v.suits(d)))
        .collect();
    for vault in &deep {
        assert!(
            (9..=16).any(|d| vault.suits(d)),
            "{}: no floor of the test suits it",
            vault.id()
        );
    }
    let mut routes: BTreeMap<&str, BTreeSet<Vec<Dir>>> = BTreeMap::new();
    let floors = (BRANCHES.iter()).map(|b| {
        (
            *b,
            1..=CONTENT.dungeon(b).expect("branch").depth,
            SEEDS,
            &shared,
        )
    });
    let deeper = (DEEP.iter()).map(|(b, d)| (*b, d.clone(), DEEP_SEEDS, &deep));
    for (branch, depths, seeds, vaults) in floors.chain(deeper) {
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        let mut dangers: BTreeSet<u8> = BTreeSet::new();
        for seed in 1..=seeds {
            for depth in depths.clone() {
                let level_id = format!("{branch}:{depth}");
                let Ok(level) = generate(&CONTENT, ("ascmearc", seed), &level_id, None) else {
                    continue;
                };
                // The danger the floor's pools draw at, which a vault's `danger` asks (#1073).
                let danger = (CONTENT.spawning(&level_id)).map_or(0, |(rule, at, _)| {
                    rule.at(at, Visit::default().player_level)
                });
                dangers.insert(danger);
                let reach = reached(&level);
                for vault in vaults.iter() {
                    let found: Vec<(Pos, Turn)> = (stamps(&level, vault).into_iter())
                        .filter(|(at, turn)| piles_hold(&level, vault, *at, *turn))
                        .collect();
                    if found.is_empty() {
                        continue;
                    }
                    let id = vault.id();
                    let what = format!("{id} {level_id} seed {seed}");
                    assert!(vault.suits(danger), "{what} at danger {danger}");
                    *seen.entry(id).or_default() += 1;
                    let at_cells: BTreeSet<Pos> = found.iter().map(|(at, _)| *at).collect();
                    for (at, turn) in &found {
                        check_doors_and_traps(&level, &reach, vault, *at, *turn);
                    }
                    // Of the ways round that match the terrain (a symmetric map matches more than
                    // one), one is the way it was stamped, and what it placed and its pools gave
                    // hold for it.
                    for at in at_cells {
                        let turns = found.iter().filter(|(a, _)| *a == at).map(|(_, t)| *t);
                        let checked: Vec<_> = turns
                            .map(|t| {
                                check_finds(&level, vault, at, t)?;
                                check_pools(&level, vault, at, t)
                            })
                            .collect();
                        let Some(walked) = checked.iter().find_map(|c| c.as_ref().ok()) else {
                            panic!("{what}: {checked:?}");
                        };
                        routes.entry(id).or_default().extend(walked.iter().cloned());
                    }
                }
            }
        }
        // Every vault looked for here that some floor of it suits stamps at least once.
        for vault in vaults
            .iter()
            .filter(|v| dangers.iter().any(|d| v.suits(*d)))
        {
            assert!(
                seen.contains_key(vault.id()),
                "`{}` never stamped on {branch} in {seeds} seeds: {seen:?}",
                vault.id()
            );
        }
    }
    // A patrol turns with its vault: over the stamps it walks more than one way.
    for vault in &shared {
        if vault.template.monsters.iter().any(|m| !m.route.is_empty()) {
            let ways = routes.get(vault.id()).map_or(0, BTreeSet::len);
            assert!(ways > 1, "{}: its patrols walked {ways} way(s)", vault.id());
        }
    }
}

/// Whether the vault's placed piles lie where it puts them, stamped `turn` round at `at`: with
/// its terrain, what tells a vault that is a plain room in a plain shape (the Wyrm's Bed) from an
/// ordinary room of the floor.
fn piles_hold(level: &LevelTemplate, vault: &Vault, at: Pos, turn: Turn) -> bool {
    let turned = vault.turned(turn);
    (turned.template.objects.iter()).all(|(p, pile)| {
        let here = level.objects.get(&Pos::new(at.x + p.x, at.y + p.y));
        here.is_some_and(|h| pile.iter().all(|s| h.iter().any(|t| t.object == s.object)))
    })
}

/// The vault's entrances lead out to the stairs, and its known traps are known.
fn check_doors_and_traps(
    level: &LevelTemplate,
    reach: &BTreeSet<Pos>,
    vault: &Vault,
    at: Pos,
    turn: Turn,
) {
    let fireball = CONTENT.trap_id("fireball").expect("the trap");
    let turned = vault.turned(turn);
    let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
    for (door, _) in &turned.entrances {
        let door = shift(*door);
        assert!(reach.contains(&door), "{}: {door}", vault.id());
    }
    // The Spiteful Coffer's trap is known, as on a floor (the catalogue's unchecked point,
    // docs/design/generic-vaults.md A3).
    for (p, trap) in &turned.template.traps {
        let stamped = level.traps.get(&shift(*p));
        if trap.known {
            assert!(stamped.is_some_and(|t| t.known), "{}", vault.id());
        }
        if vault.id() == "spiteful_coffer" {
            assert_eq!(stamped.map(|t| t.kind), Some(fireball));
            assert!(level.chests.contains_key(&shift(*p)), "on the chest");
        }
    }
}

/// The finds a vault names lie where it puts them, stamped `turn` round at `at`: the Reliquary's
/// rune a lightning bolt with its find on it, the Delving's pick-axe at its way in, and the
/// Sealed Barrow's lead key inside it (that it opens the door: [`check_pools`]).
fn check_finds(level: &LevelTemplate, vault: &Vault, at: Pos, turn: Turn) -> Result<(), String> {
    let turned = vault.turned(turn);
    let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
    let inside = |p: &Pos| {
        let q = Pos::new(p.x - at.x, p.y - at.y);
        q.x >= 0 && q.y >= 0 && turned.template.terrain.get(q).is_some()
    };
    let lies_inside = |object| {
        (level.objects.iter()).any(|(p, pile)| inside(p) && pile.iter().any(|s| s.object == object))
    };
    match vault.id() {
        "reliquary" => {
            let lightning = CONTENT.trap_id("lightning_bolt").expect("the trap");
            for p in turned.template.traps.keys() {
                if level.traps.get(&shift(*p)).map(|t| t.kind) != Some(lightning) {
                    return Err(format!("{}: no lightning rune", shift(*p)));
                }
                if !level.objects.contains_key(&shift(*p)) {
                    return Err(format!("{}: no find on the rune", shift(*p)));
                }
            }
        }
        "delving" if !lies_inside(CONTENT.object_id("pick_axe").expect("the pick")) => {
            return Err("no pick-axe".into());
        }
        "sealed_barrow" if !lies_inside(CONTENT.object_id("lead_key").expect("the key")) => {
            return Err("no lead key".into());
        }
        _ => {}
    }
    Ok(())
}

/// What the vault's pools gave, stamped `turn` round at `at`: each draw with the pool's statuses
/// and route (the route walkable and back where it began), and every lock of the vault with a key
/// that opens it, lying or carried, reached from an entrance without passing a lock. The routes
/// walked, or why not.
fn check_pools(
    level: &LevelTemplate,
    vault: &Vault,
    at: Pos,
    turn: Turn,
) -> Result<Vec<Vec<Dir>>, String> {
    let turned = vault.turned(turn);
    let template = &turned.template;
    let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
    let mut walked = Vec::new();
    for pool in &template.monsters {
        let cells: BTreeSet<Pos> = pool.cells.iter().map(|p| shift(*p)).collect();
        for spawn in (level.creatures.iter()).filter(|c| cells.contains(&c.pos)) {
            if spawn.status != pool.status {
                return Err(format!("{}: status {:?}", spawn.pos, spawn.status));
            }
            if pool.route.is_empty() {
                continue;
            }
            if spawn.route != pool.route {
                return Err(format!("{}: route {:?}", spawn.pos, spawn.route));
            }
            let mut p = spawn.pos;
            for dir in &spawn.route {
                p = p.step(*dir);
                let walkable = level
                    .terrain
                    .get(p)
                    .is_some_and(|t| CONTENT.terrain(*t).walkable);
                if !walkable {
                    return Err(format!("{}: the patrol leaves the floor at {p}", spawn.pos));
                }
            }
            if p != spawn.pos {
                return Err(format!("{}: the patrol ends at {p}", spawn.pos));
            }
            walked.push(spawn.route.clone());
        }
    }
    // A door's lock; a chest's wants no key (README rule 12).
    let door = |p: &Pos| {
        level
            .terrain
            .get(*p)
            .is_some_and(|t| CONTENT.terrain(*t).is_closed_door())
    };
    let locks: BTreeMap<Pos, &Lock> = (template.locks.iter())
        .map(|(p, l)| (shift(*p), l))
        .filter(|(p, _)| door(p))
        .collect();
    let inside = |p: Pos| {
        let q = Pos::new(p.x - at.x, p.y - at.y);
        q.x >= 0 && q.y >= 0 && template.terrain.get(q).is_some()
    };
    // Within the vault, from its entrances, through anything walkable or a door but a locked one.
    let mut open: BTreeSet<Pos> = (turned.entrances.iter()).map(|(p, _)| shift(*p)).collect();
    let mut queue: VecDeque<Pos> = open.iter().copied().collect();
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            let passable = level.terrain.get(q).is_some_and(|t| {
                let def = CONTENT.terrain(*t);
                def.walkable || def.is_closed_door() || def.is_open_door()
            });
            if inside(q) && passable && !locks.contains_key(&q) && open.insert(q) {
                queue.push_back(q);
            }
        }
    }
    for lock in locks.values() {
        let opens =
            |s: &Stack| (CONTENT.object(s.object).key.as_ref()).is_some_and(|k| k.opens(lock));
        let lying =
            (level.objects.iter()).any(|(p, pile)| open.contains(p) && pile.iter().any(opens));
        let carried =
            (level.creatures.iter()).any(|c| open.contains(&c.pos) && c.carry.iter().any(opens));
        if !(lying || carried) {
            return Err(format!("no key reached for the lock {:?}", lock.id));
        }
    }
    // A pool's carried key opens a lock of its vault.
    for pool in &template.monsters {
        for stack in &pool.carry {
            let key = CONTENT.object(stack.object).key.as_ref();
            if !key.is_some_and(|k| locks.values().any(|l| k.opens(l))) {
                return Err(format!(
                    "a pool carries {stack:?}, which opens no lock of the vault"
                ));
            }
        }
    }
    Ok(walked)
}

/// Old Enemies' two halls fight each other with no help from the player: a faction's `hostile`
/// list binds both sides (`content/faction.rs`), and the AI's target is the nearest hostile in
/// sight of either side (`World::nearest_hostile_in_sight`), so a devil needs no `demon` line of
/// its own to strike back (the catalogue's unchecked point, docs/design/generic-vaults.md E1).
#[test]
fn old_enemies_halls_are_hostile_both_ways() {
    let drawn = |tag| (CONTENT.tagged(tag)).filter(|&c| CONTENT.may_stray(c));
    let demons: Vec<_> = drawn("demon").collect();
    let devils: Vec<_> = drawn("devil").collect();
    assert!(!demons.is_empty() && !devils.is_empty());
    for &demon in &demons {
        for &devil in &devils {
            assert_eq!(CONTENT.stance(devil, demon), Stance::Hostile);
            assert_eq!(CONTENT.stance(demon, devil), Stance::Hostile);
        }
    }
}
