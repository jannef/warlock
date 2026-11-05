//! A vault's monster pools (spec §7 "Level file format"): who comes, where, and what is refused.
//! How they stand with the spawner's cap: `world/spawn_tests.rs`.

use super::*;
use crate::content::{ContentFiles, danger};
use std::collections::BTreeSet;

/// A hall behind a door: twelve marked cells, its pool from `legend`.
fn vault_file(legend: &str) -> String {
    format!(
        "name = \"Crowd\"\nmap = '''\n########\n#mmmmmm#\n#......#\n#mmmmmm#\n####+###\n'''\n\
         [legend]\n\"m\" = {{ terrain = \"floor\", {legend} }}\n"
    )
}

/// The builtin content with a rooms branch of `danger` of one floor that stamps the vault above,
/// no scenario of its own.
fn content_with(legend: &str, danger: u8) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert("vaults/crowd.toml", &vault_file(legend));
    // The guard post has a pool of its own: here it stamps elsewhere.
    files.insert(
        "vaults/guard_post.toml",
        "name = \"Elsewhere\"\ndungeons = [\"elsewhere\"]\nmap = '''\n##+##\n#...#\n#####\n'''\n",
    );
    let branch = format!(
        "\n[[dungeon]]\nid = \"crowded\"\nname = \"Crowded\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [80, 40]\ndanger = {danger}\npopulation = [\"kobold\", \"rat\"]\n\
         population_count = 25\nspawn = {{ every = \"1m\", chance = 100 }}\n\
         [[dungeon.levels]]\nfrom = 1\nvaults = [\"crowd\"]\n\
         [[dungeon]]\nid = \"elsewhere\"\nname = \"Elsewhere\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [80, 21]\n"
    );
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    Content::load(&files).map_err(|e| e.message)
}

fn floor(content: &Content, seed: u64) -> LevelTemplate {
    generate(content, ("showcase", seed), "crowded:1", None).expect("the floor")
}

/// The spawns of the floor's vault pool, by the cells the floor says stand outside the cap.
fn drawn(level: &LevelTemplate) -> Vec<&CreatureSpawn> {
    (level.creatures.iter())
        .filter(|c| level.outside_cap.contains(&c.pos))
        .collect()
}

#[test]
fn a_tag_pool_draws_its_count_of_the_tribe_at_the_floor_s_danger() {
    let content = content_with("monsters = { pool = \"goblin\", count = 10 }", 5).expect("content");
    let mut kinds = BTreeSet::new();
    for seed in 1..=12 {
        let level = floor(&content, seed);
        let spawns = drawn(&level);
        assert_eq!(spawns.len(), 10, "seed {seed}");
        let cells: BTreeSet<Pos> = spawns.iter().map(|s| s.pos).collect();
        assert_eq!(cells.len(), 10, "seed {seed}: a cell each");
        for spawn in &spawns {
            let def = content.creature(spawn.creature);
            assert!(def.tags.iter().any(|t| t == "goblin"), "{}", def.id);
            assert!(!def.unique, "{}", def.id);
            // At the danger, or an exploding step deeper as any draw may be.
            assert!(def.level <= 5 + 6, "{} at danger 5", def.id);
            assert!(!spawn.rolls_gear, "armed with the rest");
            kinds.insert(def.id.clone());
        }
    }
    assert!(kinds.len() > 1, "who is inside differs: {kinds:?}");
}

#[test]
fn the_level_pool_draws_as_the_population_does() {
    let content = content_with("monsters = { pool = \"level\" }", 3).expect("content");
    let weighs = |id: &str| {
        let c = content.creature_id(id).expect("a type");
        danger::level_weight(content.creature(c).level, 3) > 0
    };
    assert!(weighs("kobold"));
    let mut own = 0;
    let mut all = 0;
    for seed in 1..=8 {
        let level = floor(&content, seed);
        // Without a count every marked cell gets one.
        let spawns = drawn(&level);
        assert_eq!(spawns.len(), 12, "seed {seed}");
        for spawn in spawns {
            let id = &content.creature(spawn.creature).id;
            all += 1;
            if id == "kobold" || id == "rat" {
                own += 1;
            }
        }
    }
    // The population, with its few strays.
    assert!(own * 100 >= all * 70, "{own} of {all} of the floor's own");
}

#[test]
fn they_come_on_top_of_the_population_which_keeps_off_their_cells() {
    let content = content_with("monsters = { pool = \"level\", count = 4 }", 3).expect("content");
    for seed in 1..=8 {
        let level = floor(&content, seed);
        let spawns = drawn(&level);
        assert_eq!(spawns.len(), 4, "seed {seed}");
        // The population's count stands (the generator's own patrols come on top of it too).
        assert!(level.creatures.len() >= 25 + 4, "seed {seed}");
        let cells: BTreeSet<Pos> = level.creatures.iter().map(|c| c.pos).collect();
        assert_eq!(
            cells.len(),
            level.creatures.len(),
            "seed {seed}: no cell twice"
        );
        // No hidden trap lies under one, and a walk stays in the vault: no door on it.
        for spawn in spawns {
            assert!(!level.traps.contains_key(&spawn.pos), "seed {seed}");
            let mut p = spawn.pos;
            for dir in &spawn.route {
                p = p.step(*dir);
                assert!(!content.terrain(level.terrain[p]).is_door(), "seed {seed}");
            }
        }
    }
}

#[test]
fn the_same_seed_gives_the_same_vault() {
    let content = content_with("monsters = { pool = \"goblin\", count = 6 }", 5).expect("content");
    let who = |level: &LevelTemplate| {
        (drawn(level).iter())
            .map(|s| (s.creature, s.pos, s.route.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(who(&floor(&content, 3)), who(&floor(&content, 3)));
    assert_ne!(who(&floor(&content, 3)), who(&floor(&content, 4)));
}

#[test]
fn a_tribe_with_nothing_at_the_danger_leaves_the_cells_bare_and_nothing_else_moves() {
    // Trolls are far deeper than danger 1; the floor stands as it would.
    let trolls = content_with("monsters = { pool = \"troll\", count = 5 }", 1).expect("content");
    let goblins = content_with("monsters = { pool = \"goblin\", count = 5 }", 1).expect("content");
    let bare = |level: &LevelTemplate| {
        (
            level.terrain.clone(),
            level.traps.len(),
            level.objects.len(),
        )
    };
    for seed in 1..=4 {
        let (t, g) = (floor(&trolls, seed), floor(&goblins, seed));
        assert_eq!(drawn(&t).len(), 0, "seed {seed}");
        assert_eq!(bare(&t), bare(&g), "seed {seed}");
    }
}

#[test]
fn a_pool_that_can_hold_nothing_is_refused() {
    for (legend, says) in [
        ("monsters = { pool = \"nobody\" }", "`nobody`"),
        // A name that only people or uniques carry brings no one; so does a list with one.
        ("monsters = { pool = [\"goblin\", \"nobody\"] }", "`nobody`"),
        (
            "monsters = { pool = [\"level\", \"goblin\"] }",
            "stands alone",
        ),
        ("monsters = { pool = \"level\", count = 0 }", "at least 1"),
        (
            "monsters = { pool = \"level\", count = 13 }",
            "more than the 12",
        ),
        ("monsters = { pool = \"level\", extra = 1 }", "extra"),
        (
            "monsters = { pool = \"level\" }, creature = \"kobold\"",
            "holds no",
        ),
        (
            "monsters = { pool = \"level\" }, trap = \"spear\"",
            "holds no",
        ),
    ] {
        let err = content_with(legend, 3).expect_err(legend);
        assert!(err.contains(says), "{legend}: {err}");
    }
}

#[test]
fn only_a_dungeon_vault_draws_monsters() {
    // Not on a door.
    let mut files = ContentFiles::builtin();
    files.insert(
        "vaults/door.toml",
        "name = \"Door\"\nmap = '''\n##D##\n#...#\n#####\n'''\n[legend]\n\
         \"D\" = { terrain = \"door_closed\", monsters = { pool = \"level\" } }\n",
    );
    let err = Content::load(&files).expect_err("a door").message;
    assert!(err.contains("not a door"), "{err}");
    // Not in the local areas, which have no danger.
    let mut files = ContentFiles::builtin();
    files.insert(
        "vaults/open.toml",
        "name = \"Open\"\nkind = \"outdoor\"\nmap = '''\n...\n.m.\n...\n'''\n[legend]\n\
         \".\" = \"flagstones\"\n\
         \"m\" = { terrain = \"flagstones\", monsters = { pool = \"level\" } }\n",
    );
    let err = Content::load(&files).expect_err("outdoors").message;
    assert!(err.contains("no danger"), "{err}");
    // Not in a level file.
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/pooled.toml",
        "name = \"Pooled\"\nmap = '''\n###\n#m#\n###\n'''\n[legend]\n\
         \"m\" = { terrain = \"floor\", monsters = { pool = \"level\" } }\n",
    );
    let err = Content::load(&files).expect_err("a level").message;
    assert!(err.contains("only in vaults"), "{err}");
}

#[test]
fn the_guard_post_draws_two_of_the_floor_s_own_beside_its_guard() {
    let content = Content::builtin().expect("content");
    let guard = (content.vaults().iter())
        .find(|v| v.id() == "guard_post")
        .expect("the guard post");
    assert!(!guard.pinned);
    let [group] = guard.template.monsters.as_slice() else {
        panic!("one pool");
    };
    assert_eq!(group.pool, crate::content::MonsterPool::Level);
    assert_eq!(group.count, 2);
    // The fixed guard keeps the key.
    assert_eq!(guard.template.creatures.len(), 1);
    assert!(!guard.template.creatures[0].carry.is_empty());
}
