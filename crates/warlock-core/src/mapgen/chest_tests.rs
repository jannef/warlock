//! Chests on generated floors (spec §8 "Chests", `chest.rs`; docs/design/chests-and-treasure.md
//! §3.3, §4, §8): one floor in five, on open floor that stays joined, from the level's own chest
//! stream, so the rest of the level is the same without them; their loot as the bands' tables
//! say.

use super::*;
use crate::chest::{ChestState, roll_contents};
use crate::content::{ContentFiles, GOLD, Rarity};

/// The built-in content with every branch's `chests` set to `odds` (a TOML inline table), or
/// left as the data has it with `None`.
fn content(odds: Option<&str>) -> Content {
    let mut files = ContentFiles::builtin();
    if let Some(odds) = odds {
        files.edit_table("dungeons", |text| {
            text.replace("[[dungeon]]\n", &format!("[[dungeon]]\nchests = {odds}\n"))
        });
    }
    Content::load(&files).expect("fixture content")
}

/// Whether `level`'s chest on `at` stands on open floor: its eight neighbours walkable and no
/// door, and away from the stairs.
fn stands_clear(level: &LevelTemplate, content: &Content, at: Pos) -> bool {
    let open = Dir::ALL.iter().all(|d| {
        let def = content.terrain(level.terrain[at.step(*d)]);
        def.walkable && !def.is_door()
    });
    open && (level.stairs.iter()).all(|s| s.pos.chebyshev(at) >= chest::AWAY)
}

#[test]
fn one_floor_in_five_has_a_chest() {
    let c = content(None);
    let shut = c.chest_terrain(ChestState::Shut).expect("chest terrain");
    let (mut floors, mut with) = (0, 0);
    for seed in 0..60 {
        for id in ["depths:1", "depths:3"] {
            let level = generate(&c, ("showcase", seed), id, None).expect("level");
            floors += 1;
            // A treasure room's chests are its own (`treasury.rs`), not the floor's.
            let treasury = |p: &Pos| {
                (level.rooms.iter())
                    .any(|r| r.contains(*p) && c.room_effect(r.effect).treasure.is_some())
            };
            let floor: Vec<_> = (level.chests.iter())
                .filter(|(p, _)| !treasury(p))
                .collect();
            with += usize::from(!floor.is_empty());
            assert!(floor.len() <= 1, "seed {seed} {id}: count = [1, 1]");
            for (&at, chest) in floor {
                assert_eq!(level.terrain[at], shut, "seed {seed} {id}: shut");
                assert_eq!(chest.level, level.item_level, "the floor's level");
                assert!(stands_clear(&level, &c, at), "seed {seed} {id}: {at}");
            }
        }
    }
    assert!((12..=36).contains(&with), "{with} of {floors} floors");
}

#[test]
fn a_level_is_the_same_with_or_without_its_chests() {
    let (with, without) = (
        content(Some("{ chance = 100, count = [2, 3] }")),
        content(Some("{ chance = 0 }")),
    );
    let mut chests = 0;
    for seed in 0..20 {
        for id in ["depths:2", "depths:4"] {
            let level = generate(&with, ("showcase", seed), id, None).expect("level");
            let plain = generate(&without, ("showcase", seed), id, None).expect("level");
            assert!(plain.chests.is_empty());
            chests += level.chests.len();
            for (p, t) in level.terrain.iter() {
                if level.chests.contains_key(&p) {
                    assert_eq!(with.terrain(plain.terrain[p]).id, "floor", "{p}");
                    assert!(stands_clear(&level, &with, p), "seed {seed} {id}: {p}");
                } else {
                    assert_eq!(*t, plain.terrain[p], "seed {seed} {id}: {p}");
                }
            }
            assert_eq!(level.creatures, plain.creatures, "seed {seed} {id}");
            assert_eq!(level.objects, plain.objects, "seed {seed} {id}");
            let off_chests = |keys: Vec<Pos>| -> Vec<Pos> {
                keys.into_iter()
                    .filter(|p| !level.chests.contains_key(p))
                    .collect()
            };
            assert_eq!(
                off_chests(level.traps.keys().copied().collect()),
                plain.traps.keys().copied().collect::<Vec<_>>()
            );
            assert_eq!(
                off_chests(level.locks.keys().copied().collect()),
                plain.locks.keys().copied().collect::<Vec<_>>()
            );
            let touch = |a: &Pos, b: &Pos| a != b && a.chebyshev(*b) <= 1;
            let keys: Vec<&Pos> = level.chests.keys().collect();
            assert!(
                keys.iter().all(|a| keys.iter().all(|b| !touch(a, b))),
                "seed {seed} {id}: two chests never touch"
            );
            assert!(connect::validate(&level, &with).is_ok(), "seed {seed} {id}");
        }
    }
    assert!(chests >= 60, "{chests} chests on 40 floors");
}

#[test]
fn chests_come_locked_and_trapped_by_their_band() {
    let c = content(Some("{ chance = 100, count = [3, 3] }"));
    let coffer = c.chests().band("coffer").expect("coffer").clone();
    let (mut chests, mut locked, mut trapped) = (0, 0, 0);
    for seed in 0..60 {
        let level = generate(&c, ("showcase", seed), "depths:1", None).expect("level");
        for at in level.chests.keys() {
            chests += 1;
            locked += usize::from(level.locks.contains_key(at));
            if let Some(trap) = level.traps.get(at) {
                trapped += 1;
                assert!(!trap.known, "hidden");
                assert!(c.trap(trap.kind).effect.fits_fixture(), "never a pit");
            }
        }
    }
    let near = |n: usize, p: u8| (n * 100 / chests).abs_diff(usize::from(p)) <= 12;
    assert!(near(locked, coffer.locked), "{locked} of {chests} locked");
    assert!(
        near(trapped, coffer.trapped),
        "{trapped} of {chests} trapped"
    );
}

#[test]
fn a_branch_without_traps_traps_nothing_but_draws_the_same() {
    let c = content(None);
    let shut = c.chest_terrain(ChestState::Shut).expect("chest terrain");
    let arrow = c.trap_id("arrow").expect("arrow");
    let level = generate(&c, ("showcase", 3), "depths:1", None).expect("level");
    let at = Pos::new(1, 1);
    let mut trapped = 0;
    for seed in 0..40 {
        let (mut a, mut b) = (level.clone(), level.clone());
        let (mut ra, mut rb) = (Rng::chests(seed, "x"), Rng::chests(seed, "x"));
        chest::lay(&mut a, &c, &mut ra, (at, shut), (&[arrow], Luck::default()));
        chest::lay(&mut b, &c, &mut rb, (at, shut), (&[], Luck::default()));
        assert_eq!(ra, rb, "seed {seed}: the same draws");
        assert_eq!(a.chests, b.chests, "seed {seed}: the same contents");
        assert_eq!(a.locks, b.locks, "seed {seed}: the same lock");
        assert_eq!(
            b.traps.get(&at),
            level.traps.get(&at),
            "seed {seed}: no trap"
        );
        trapped += usize::from(a.traps.get(&at).is_some_and(|t| t.kind == arrow));
    }
    assert!(trapped > 0, "an arrow sometimes");
}

#[test]
fn a_floor_at_depth_eight_lays_banded_chests() {
    let c = content(Some("{ chance = 100 }"));
    let mut seen = 0;
    for seed in 0..10 {
        let level = generate(&c, ("showcase", seed), "depths:8", None).expect("level");
        for chest in level.chests.values() {
            seen += 1;
            assert_eq!(chest.level, 8);
            let band = c.chests().band_of(chest.level).expect("band");
            assert_eq!(band.id, "banded_chest");
            assert!(!chest.holds.is_empty(), "2d3 + 1 items and a purse");
        }
    }
    assert!(seen >= 8, "{seen}");
}

/// The means of what [`roll_contents`] gives a chest of `level` over `n` rolls: (stacks of
/// items, gold coins, loose gems).
fn means(c: &Content, level: u8, n: u32) -> (f64, f64, f64) {
    let coins = c.object_id(GOLD).expect("gold");
    let gems = c.gems().expect("gems").object;
    let mut rng = Rng::chests(7, "means");
    let (mut items, mut gold, mut stones) = (0u32, 0u32, 0u32);
    for _ in 0..n {
        for stack in roll_contents(c, &mut rng, level, Luck::default()) {
            match stack.object {
                o if o == coins => gold += stack.count,
                o if o == gems => stones += stack.count,
                _ => items += 1,
            }
        }
    }
    let mean = |x: u32| f64::from(x) / f64::from(n);
    (mean(items), mean(gold), mean(stones))
}

#[test]
fn chest_loot_rolls_as_the_bands_table_says() {
    let c = content(None);
    // A coffer: 1d3 items (a few found as gems), 2d6 gold, 1d4 - 3 gems.
    let (items, gold, gems) = means(&c, 1, 2000);
    assert!((1.6..=2.05).contains(&items), "coffer items {items}");
    assert!((6.6..=7.4).contains(&gold), "coffer gold {gold}");
    assert!((0.2..=0.5).contains(&gems), "coffer gems {gems}");
    // A banded chest: 2d3 + 1 items, 3d6 × 3 gold, 1d2 gems.
    let (items, gold, gems) = means(&c, 9, 1000);
    assert!((4.3..=5.1).contains(&items), "banded items {items}");
    assert!((30.0..=33.0).contains(&gold), "banded gold {gold}");
    assert!((1.4..=1.9).contains(&gems), "banded gems {gems}");
}

#[test]
fn chest_loot_is_magic_as_often_as_the_table_says() {
    let c = content(None);
    let mut rng = Rng::chests(11, "magic");
    let (mut items, mut magic) = (0u32, 0u32);
    for _ in 0..1500 {
        for stack in roll_contents(&c, &mut rng, 5, Luck::default()) {
            magic += u32::from(stack.rarity() != Rarity::Normal);
            items += 1;
        }
    }
    // A hutch: 20 % of items at least magic, plus what an ordinary roll makes magic.
    let share = magic * 100 / items;
    assert!((15..=50).contains(&share), "{share} % magic");
}
