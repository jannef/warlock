//! Every shared dungeon vault stamps on real floors of a rooms branch and of a cave branch, and
//! its doors lead out (#1072). A vault added to `assets/vaults/` joins the test by being there.

use super::vault_turn_tests::{reached, stamps};
use super::*;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

/// A rooms branch and a cave branch of the first campaign (both in a release build too).
const BRANCHES: [&str; 2] = ["banfeld", "wulfeshol"];
/// Enough that each of the vaults, some kept to a few dangers (the Tithe-barn's floors go no
/// deeper than 6, the first of Bānfeld's four), comes up on each branch.
const SEEDS: u64 = 300;

#[test]
fn every_shared_dungeon_vault_stamps_on_rooms_and_caves_floors() {
    let shared: Vec<&Vault> = (CONTENT.vaults().iter())
        .filter(|v| !v.is_outdoor() && v.campaign.is_none())
        .collect();
    assert!(shared.len() >= 11, "{}", shared.len());
    let fireball = CONTENT.trap_id("fireball").expect("the trap");
    let lead_key = CONTENT.object_id("lead_key").expect("the key");
    let alarm = CONTENT.trap_id("alarm").expect("the trap");
    let water = CONTENT.terrain_id("water").expect("the terrain");
    for branch in BRANCHES {
        let def = CONTENT.dungeon(branch).expect("the branch");
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for seed in 1..=SEEDS {
            for depth in 1..=def.depth {
                let level_id = format!("{branch}:{depth}");
                let Ok(level) = generate(&CONTENT, ("ascmearc", seed), &level_id, None) else {
                    continue;
                };
                for vault in &shared {
                    let found = stamps(&level, vault);
                    if found.is_empty() {
                        continue;
                    }
                    *seen.entry(vault.id()).or_default() += found.len();
                    let reach = reached(&level);
                    for &(at, turn) in &found {
                        let turned = vault.turned(turn);
                        let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
                        for (door, _) in &turned.entrances {
                            let door = shift(*door);
                            let id = vault.id();
                            assert!(reach.contains(&door), "{id} {level_id} seed {seed}: {door}");
                        }
                        // The Spiteful Coffer's trap is known, as on a floor (the catalogue's
                        // unchecked point, docs/design/generic-vaults.md A3).
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
                        // A vault whose terrain is the same another way round (the
                        // Counting-house mirrored) is found that way too, its finds and traps
                        // elsewhere: these hold for some way round at this corner.
                        let id = vault.id();
                        // Named places come with the vault (the Four Fonts' doors say their
                        // elements).
                        let places = |v: &Vault| {
                            (v.template.places.iter())
                                .all(|(p, place)| level.places.get(&shift(*p)) == Some(place))
                        };
                        // The Counting-house's tripwire lies under a pile of its coin.
                        let tripwire = |v: &Vault| {
                            id != "counting_house"
                                || v.template.traps.keys().all(|p| {
                                    level.traps.get(&shift(*p)).map(|t| t.kind) == Some(alarm)
                                        && level.objects.contains_key(&shift(*p))
                                })
                        };
                        // The Well-house's coin lies in its water, wadeable, not deep.
                        let wet = |v: &Vault| {
                            id != "well_house"
                                || v.template.loot.iter().all(|(p, _)| {
                                    level.terrain.get(shift(*p)) == Some(&water)
                                        && level.objects.contains_key(&shift(*p))
                                })
                        };
                        let holds = (found.iter())
                            .filter(|(a, _)| *a == at)
                            .map(|(_, t)| vault.turned(*t))
                            .any(|v| places(&v) && tripwire(&v) && wet(&v));
                        assert!(holds, "{id} {level_id} seed {seed}");
                        // The Sealed Barrow's key lies before its door.
                        if vault.id() == "sealed_barrow" {
                            let (_, lock) = (turned.template.locks.iter())
                                .next()
                                .expect("the barrow's lock");
                            let inside = |p: Pos| {
                                let q = Pos::new(p.x - at.x, p.y - at.y);
                                turned.template.terrain.get(q).is_some()
                            };
                            let key_inside = (level.objects.iter()).any(|(p, pile)| {
                                inside(*p) && pile.iter().any(|s| s.object == lead_key)
                            });
                            assert!(key_inside, "{level_id} seed {seed}: the lead key");
                            assert!(level.has_key_for(&CONTENT, lock, inside));
                        }
                    }
                }
            }
        }
        for vault in &shared {
            assert!(
                seen.contains_key(vault.id()),
                "`{}` never stamped on {branch} in {SEEDS} seeds: {seen:?}",
                vault.id()
            );
        }
    }
}
