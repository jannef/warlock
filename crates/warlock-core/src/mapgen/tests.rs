use super::*;
use crate::World;
use crate::content::{ContentFiles, Vault};
use crate::rng::parse_seed;
use proptest::prelude::*;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, LazyLock};

/// The built-in content plus scenarios starting on a rooms level and a caves level.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("rooms", "depths:1"), ("caves", "depths:3")] {
        files.insert(
            &format!("scenarios/test_{name}.toml"),
            &format!("name = \"T\"\nstart_level = \"{level}\"\n"),
        );
    }
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn level(seed: u64, depth: u32) -> LevelTemplate {
    generate(&CONTENT, seed, &format!("depths:{depth}"), Some("vast")).expect("generates")
}

/// Header (name, stairs, creatures) plus the map, for snapshots.
fn describe(level: &LevelTemplate) -> String {
    let (w, h) = (level.terrain.width(), level.terrain.height());
    let mut out = format!("{} ({}) {w}x{h}\n", level.name, level.id);
    for s in &level.stairs {
        let to = s.to.as_deref().unwrap_or("nowhere");
        let _ = writeln!(out, "stairs {:?} at {} -> {to}", s.dir, s.pos);
    }
    for c in &level.creatures {
        let route: Vec<&str> = c.route.iter().map(|d| d.name()).collect();
        let name = &CONTENT.creature(c.creature).id;
        let _ = writeln!(out, "{name} at {} route {}", c.pos, route.join(" "));
    }
    let _ = writeln!(out, "{} placed lights", level.lights.len());
    out + &ascii(level, &CONTENT)
}

#[test]
fn rooms_snapshot() {
    insta::assert_snapshot!("rooms_seed_1_depth_1", describe(&level(1, 1)));
}

#[test]
fn caves_snapshot() {
    insta::assert_snapshot!("caves_seed_1_depth_3", describe(&level(1, 3)));
}

#[test]
fn deepest_level_snapshot() {
    let seed = parse_seed("warlock").expect("word seed");
    insta::assert_snapshot!("rooms_seed_warlock_depth_8", describe(&level(seed, 8)));
}

/// Cells the player can walk to from `from` (8-way moves; closed doors open on a bump).
fn player_reach(level: &LevelTemplate, from: Pos) -> Grid<bool> {
    let terrain = &level.terrain;
    let passable = |p: Pos| {
        terrain.get(p).is_some_and(|t| {
            let def = CONTENT.terrain(*t);
            def.walkable || def.is_closed_door()
        })
    };
    let mut seen = Grid::new(terrain.width(), terrain.height(), false);
    seen[from] = true;
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if passable(q) && !seen[q] {
                seen[q] = true;
                queue.push_back(q);
            }
        }
    }
    seen
}

/// Everything a generated level promises.
fn check(level: &LevelTemplate, depth: u32) -> Result<(), TestCaseError> {
    let content = &**CONTENT;
    let def = content.dungeon("depths").expect("depths branch");
    let plan = def.plan(depth).expect("plan");
    let terrain = &level.terrain;
    prop_assert_eq!(
        (terrain.width(), terrain.height()),
        (plan.width, plan.height)
    );
    prop_assert_eq!(&level.name, &def.level_name(depth));

    // Stairs: up first (the arrival point), down unless deepest, linked to the neighbors.
    let up_to = if depth == 1 {
        "vast".to_string()
    } else {
        def.level_id(depth - 1)
    };
    let mut expected = vec![(StairsDir::Up, Some(up_to))];
    if depth < def.depth {
        expected.push((StairsDir::Down, Some(def.level_id(depth + 1))));
    }
    let links: Vec<_> = level.stairs.iter().map(|s| (s.dir, s.to.clone())).collect();
    prop_assert_eq!(links, expected);
    prop_assert_eq!(level.player_start, Some(level.stairs[0].pos));
    for s in &level.stairs {
        prop_assert_eq!(content.terrain(terrain[s.pos]).stairs, Some(s.dir));
    }

    // Connectivity: the player reaches every stair and every walkable cell from the arrival.
    let reach = player_reach(level, level.stairs[0].pos);
    for (p, t) in terrain.iter() {
        let def = content.terrain(*t);
        if def.walkable || def.is_closed_door() {
            prop_assert!(reach[p], "{} unreachable in {}", p, level.id);
        }
        let edge =
            p.x == 0 || p.y == 0 || p.x == terrain.width() - 1 || p.y == terrain.height() - 1;
        prop_assert!(!edge || !def.walkable, "walkable edge at {}", p);
    }

    // Light: something shines.
    let emitters = terrain
        .cells()
        .iter()
        .filter(|t| content.terrain(**t).light.is_some())
        .count();
    prop_assert!(emitters + level.lights.len() > 0, "{} is dark", level.id);

    // Patrols loop over walkable terrain.
    for spawn in &level.creatures {
        crate::content::validate_route(level, content, spawn).map_err(TestCaseError::fail)?;
    }

    // Objects stamped from vaults lie on walkable terrain, in piles that are never empty.
    for (p, pile) in &level.objects {
        prop_assert!(
            content.terrain(terrain[*p]).walkable,
            "objects on a wall at {}",
            p
        );
        prop_assert!(!pile.is_empty());
    }

    // Locks (spec §8 "Doors"): only on closed doors, and never in the way of the stairs or of a
    // key to every lock, which the player reaches without unlocking anything (the level is
    // solvable without kicking a door in).
    let unlocked = player_reach_unlocked(level, level.stairs[0].pos);
    for (p, lock) in &level.locks {
        prop_assert!(
            content.terrain(terrain[*p]).is_closed_door(),
            "lock at {}",
            p
        );
        prop_assert!(
            level.has_key_for(content, lock, |q| unlocked[q]),
            "no key to the lock at {} in {}",
            p,
            level.id
        );
    }
    for s in &level.stairs {
        prop_assert!(unlocked[s.pos], "stairs at {} locked away", s.pos);
    }
    Ok(())
}

/// Cells the player can walk to from `from` without unlocking a door (8-way moves).
fn player_reach_unlocked(level: &LevelTemplate, from: Pos) -> Grid<bool> {
    let mut open = level.clone();
    for p in level.locks.keys() {
        open.terrain
            .set(*p, CONTENT.terrain_id("wall").expect("wall"));
    }
    player_reach(&open, from)
}

#[test]
fn rooms_levels_lock_a_few_rooms_and_leave_their_keys_in_reach() {
    let key = CONTENT.object_id("rusty_key").expect("rusty key");
    let (mut levels, mut locks, mut keys) = (0, 0, 0);
    for seed in 0..40 {
        for depth in [1, 2, 6, 7, 8] {
            let level = level(seed, depth);
            let generated = (level.locks.iter())
                .filter(|(_, l)| l.id.as_deref() == Some("depths"))
                .count();
            let lying = (level.objects.values().flatten())
                .filter(|s| s.object == key)
                .count();
            assert!(generated <= 2, "{} locks {generated} rooms", level.id);
            assert_eq!(
                lying,
                usize::from(generated > 0),
                "one key per level that locks"
            );
            levels += usize::from(generated > 0);
            locks += generated;
            keys += lying;
        }
    }
    assert!(
        levels > 50 && locks > levels,
        "{levels} levels, {locks} locks"
    );
    assert_eq!(keys, levels);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn every_depth_is_valid_and_deterministic(seed: u64) {
        for depth in 1..=8 {
            let a = level(seed, depth);
            check(&a, depth)?;
            prop_assert_eq!(&a, &level(seed, depth), "same seed, same level");
        }
    }

    /// Arriving on a fresh level, normal eyes without a lantern see more than their own cell:
    /// the arrival room has torches, the cave stairs a lamp.
    #[test]
    fn arrival_is_not_pitch_dark(seed: u64) {
        for scenario in ["test_rooms", "test_caves"] {
            let world = World::new(CONTENT.clone(), scenario, Some(seed)).expect("world");
            let view = world.perception();
            let visible = (0..view.height())
                .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
                .filter(|p| view.is_visible(*p))
                .count();
            prop_assert!(visible > 9, "{scenario} seed {seed}: only {visible} cells visible");
        }
    }
}

#[test]
fn seeds_and_levels_give_different_maps() {
    assert_ne!(ascii(&level(1, 1), &CONTENT), ascii(&level(2, 1), &CONTENT));
    assert_ne!(ascii(&level(1, 1), &CONTENT), ascii(&level(1, 2), &CONTENT));
    let from_nowhere = generate(&CONTENT, 1, "depths:1", None).expect("generates");
    assert_eq!(from_nowhere.stairs[0].to, None);
    assert_eq!(
        from_nowhere.terrain,
        level(1, 1).terrain,
        "the map ignores the entry"
    );
}

#[test]
fn only_branch_levels_generate() {
    let err = |id: &str| generate(&CONTENT, 1, id, None).unwrap_err();
    assert_eq!(err("hub"), GenError::NotAGeneratedLevel("hub".into()));
    assert_eq!(
        err("depths:9"),
        GenError::NotAGeneratedLevel("depths:9".into())
    );
    assert_eq!(
        err("caves:1"),
        GenError::NotAGeneratedLevel("caves:1".into())
    );
}

/// Positions where `vault`'s terrain appears verbatim in `level`.
fn stamped_at(level: &LevelTemplate, vault: &Vault) -> Vec<Pos> {
    let v = &vault.template.terrain;
    let fits = |at: Pos| {
        v.iter()
            .all(|(p, t)| level.terrain.get(Pos::new(at.x + p.x, at.y + p.y)) == Some(t))
    };
    (0..level.terrain.height())
        .flat_map(|y| (0..level.terrain.width()).map(move |x| Pos::new(x, y)))
        .filter(|at| fits(*at))
        .collect()
}

#[test]
fn vaults_are_stamped_verbatim_with_their_lights_and_creatures() {
    let vaults: Vec<&Vault> = CONTENT.vaults().iter().collect();
    let mut seen = vec![0; vaults.len()];
    for seed in 0..10 {
        for depth in 1..=8 {
            let level = level(seed, depth);
            for (i, vault) in vaults.iter().enumerate() {
                for at in stamped_at(&level, vault) {
                    seen[i] += 1;
                    let shift = |p: Pos| Pos::new(at.x + p.x, at.y + p.y);
                    for (p, light) in &vault.template.lights {
                        assert_eq!(level.lights.get(&shift(*p)), Some(light), "{}", vault.id());
                    }
                    for spawn in &vault.template.creatures {
                        assert!(
                            level
                                .creatures
                                .iter()
                                .any(|c| c.pos == shift(spawn.pos) && c.route == spawn.route),
                            "{} creature",
                            vault.id()
                        );
                    }
                }
            }
        }
    }
    // Dungeons get every dungeon vault and never the ruins meant for the overworld's local areas.
    for (vault, count) in vaults.iter().zip(&seen) {
        if vault.is_outdoor() {
            assert_eq!(*count, 0, "outdoor vault `{}` in a dungeon", vault.id());
        } else {
            assert!(*count > 0, "vault `{}` never placed", vault.id());
        }
    }
}
