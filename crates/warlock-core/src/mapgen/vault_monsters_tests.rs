//! A vault's monster pools (spec §7 "Level file format"): who comes, where, and what is refused.
//! How they stand with the spawner's cap: `world/spawn_tests.rs`.

use super::*;
use crate::content::{ContentFiles, Turn, danger};
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
    content_of(&vault_file(legend), danger)
}

/// [`content_with`] for a whole vault file.
fn content_of(vault: &str, danger: u8) -> Result<Content, String> {
    content_in(
        vault,
        danger,
        "[[dungeon.levels]]\nfrom = 1\nvaults = [\"crowd\"]\n",
    )
}

/// [`content_of`] with `levels` the branch's `[[dungeon.levels]]`: none, and the vault is drawn
/// at random, turned.
fn content_in(vault: &str, danger: u8, levels: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    // The shared vaults have pools of their own (the guard post, the Kennels ...): here only the
    // crowd's are drawn. Two without keep it company, so that the phase a floor offers (one vault
    // in four, `spaced_vaults`) may hold the crowd.
    files.remove("vaults/");
    let quiet = [
        (
            "flooded_cell",
            include_str!("../../../../assets/vaults/flooded_cell.toml"),
        ),
        (
            "shrine",
            include_str!("../../../../assets/vaults/shrine.toml"),
        ),
    ];
    for (id, text) in quiet {
        files.insert(&format!("vaults/{id}.toml"), text);
    }
    files.insert("vaults/crowd.toml", vault);
    let branch = format!(
        "\n[[dungeon]]\nid = \"crowded\"\nname = \"Crowded\"\ngenerator = \"rooms\"\ndepth = 1\n\
         size = [80, 40]\ndanger = {danger}\npopulation = [\"kobold\", \"rat\"]\n\
         population_count = 25\nspawn = {{ every = \"1m\", chance = 100 }}\n{levels}"
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
fn a_bad_pool_is_refused_and_an_empty_one_loads() {
    // A tag no type that may be drawn carries brings no one, and a count beyond the cells brings
    // as many as they hold: the data author's choice.
    content_with("monsters = { pool = [\"goblin\", \"nobody\"] }", 3).expect("loads");
    content_with("monsters = { pool = \"level\", count = 13 }", 3).expect("loads");
    for (legend, says) in [
        (
            "monsters = { pool = [\"level\", \"goblin\"] }",
            "stands alone",
        ),
        ("monsters = { pool = \"level\", count = 0 }", "at least 1"),
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

/// A crowd in a hall before a locked store room, `legend` on its ten pool cells (`m`), and, when
/// `behind`, on one more behind the lock; the brass key opens it.
fn treasury(legend: &str, behind: bool) -> String {
    let store = if behind { "m" } else { "." };
    format!(
        "name = \"Treasury\"\nmap = '''\n#########\n#mmmmm#{store}#\n#.....L.#\n#mmmmm#.#\n\
         ####+####\n'''\n[legend]\n\"m\" = {{ terrain = \"floor\", {legend} }}\n\
         \"L\" = {{ terrain = \"door_closed\", lock = \"guard_post\" }}\n"
    )
}

fn holds_key(content: &Content, stacks: &[Stack]) -> bool {
    (stacks.iter()).any(|s| content.object(s.object).id == "brass_key")
}

#[test]
fn a_sleeping_pool_places_every_draw_asleep() {
    let legend =
        "monsters = { pool = \"goblin\", count = 10 }, status = [\"asleep\", \"weakened*2\"]";
    let content = content_with(legend, 5).expect("content");
    let asleep = content.status_id("asleep").expect("asleep");
    let weakened = content.status_id("weakened").expect("weakened");
    for seed in 1..=6 {
        let level = floor(&content, seed);
        let spawns = drawn(&level);
        assert_eq!(spawns.len(), 10, "seed {seed}");
        for spawn in spawns {
            assert_eq!(
                spawn.status,
                vec![(asleep, 1), (weakened, 2)],
                "seed {seed}"
            );
        }
        // The floor's own population sleeps no more than it did.
        let others = (level.creatures.iter()).filter(|c| !level.outside_cap.contains(&c.pos));
        assert!(
            others.into_iter().all(|c| c.status.is_empty()),
            "seed {seed}"
        );
    }
}

#[test]
fn a_pool_s_key_is_carried_by_exactly_one_draw_on_top_of_its_gear() {
    let legend = "monsters = { pool = \"goblin\", count = 4 }, carry = [\"brass_key\"]";
    let content = content_of(&treasury(legend, false), 5).expect("content");
    let mut armed = 0;
    for seed in 1..=8 {
        let level = floor(&content, seed);
        let spawns = drawn(&level);
        assert_eq!(spawns.len(), 4, "seed {seed}");
        let bearers: Vec<_> = (spawns.iter())
            .filter(|s| holds_key(&content, &s.carry))
            .collect();
        assert_eq!(bearers.len(), 1, "seed {seed}: one key-bearer");
        // Its rolled gear stands, the key after it.
        let carry = &bearers[0].carry;
        assert!(
            holds_key(&content, &carry[carry.len() - 1..]),
            "seed {seed}"
        );
        if carry.len() > 1 {
            armed += 1;
        }
        assert!(
            level
                .objects
                .values()
                .all(|pile| !holds_key(&content, pile))
        );
    }
    assert!(armed > 0, "a key-bearer rolls its gear too");
}

#[test]
fn a_key_no_draw_carries_lies_on_the_pool_s_first_cell() {
    // Trolls are far deeper than danger 1: nobody is drawn.
    let legend = "monsters = { pool = \"troll\", count = 4 }, carry = [\"brass_key\"]";
    let content = content_of(&treasury(legend, false), 1).expect("content");
    for seed in 1..=4 {
        let level = floor(&content, seed);
        assert!(drawn(&level).is_empty(), "seed {seed}");
        let piles = (level.objects.values()).filter(|pile| holds_key(&content, pile));
        assert_eq!(piles.count(), 1, "seed {seed}: the key lies");
    }
}

#[test]
fn a_pool_s_route_is_walked_by_every_draw_and_turns_with_the_vault() {
    // Drawn at random (only here: `dungeons`), so it comes turned.
    let vault = "name = \"Hall\"\ndungeons = [\"crowded\"]\n\
         map = '''\n########\n#mmmmmm#\n#......#\n###+####\n'''\n\
         [legend]\n\"m\" = { terrain = \"floor\", monsters = { pool = \"goblin\", count = 4 }, \
         route = \"S1 N1\" }\n";
    let content = content_in(vault, 5, "").expect("content");
    let way = |turn: Turn| vec![turn.dir(Dir::S), turn.dir(Dir::N)];
    let ways: Vec<Vec<Dir>> = Turn::ALL.into_iter().map(way).collect();
    let mut seen = BTreeSet::new();
    for seed in 1..=40 {
        let level = floor(&content, seed);
        let spawns = drawn(&level);
        if spawns.is_empty() {
            continue;
        }
        assert_eq!(spawns.len(), 4, "seed {seed}");
        let routes: BTreeSet<Vec<Dir>> = spawns.iter().map(|s| s.route.clone()).collect();
        assert_eq!(routes.len(), 1, "seed {seed}: one route for all");
        for spawn in spawns {
            assert!(
                ways.contains(&spawn.route),
                "seed {seed}: {:?}",
                spawn.route
            );
            let step = spawn.pos.step(spawn.route[0]);
            assert!(content.terrain(level.terrain[step]).walkable, "seed {seed}");
        }
        seen.extend(routes);
    }
    assert!(
        seen.len() > 1,
        "the vault turns, its route with it: {seen:?}"
    );
    // The vault itself, every way it may lie.
    let hall = (content.vaults().iter())
        .find(|v| v.id() == "crowd")
        .expect("the hall");
    for turn in Turn::ALL {
        assert_eq!(hall.turned(turn).template.monsters[0].route, way(turn));
    }
}

#[test]
fn a_pool_s_status_carry_and_route_are_checked_as_a_placed_creature_s() {
    for (legend, says) in [
        // From the cells of the far row the walk leaves the floor; from any it must come back.
        (
            "monsters = { pool = \"goblin\" }, route = \"S1 N1\"",
            "leaves walkable",
        ),
        (
            "monsters = { pool = \"goblin\" }, route = \"S1\"",
            "loop back",
        ),
        (
            "monsters = { pool = \"goblin\" }, route = \"Q2\"",
            "legend `m`",
        ),
        (
            "monsters = { pool = \"goblin\" }, status = [\"nothing\"]",
            "unknown status",
        ),
        (
            "monsters = { pool = \"goblin\" }, carry = [\"nothing\"]",
            "carry",
        ),
        // A faction is not yet a pool's (F6), and its draws roll their own arms.
        (
            "monsters = { pool = \"goblin\" }, faction = \"peaceful\"",
            "not a faction",
        ),
        (
            "monsters = { pool = \"goblin\" }, wield = [\"dagger\"]",
            "not a faction",
        ),
    ] {
        let err = content_with(legend, 3).expect_err(legend);
        assert!(err.contains(says), "{legend}: {err}");
    }
}
