//! Fyrdelf, the Ascmearc's second hidden dungeon (#339; docs/design/campaign.md §7): the branch's
//! floors, heat, and danger, its population and duergar traders, its first level (a snapshot),
//! the hidden way in on Ascdūn, and the vaults its levels name: the foreman's cabin on every
//! gallery floor, the drake's hoard at the bottom, each stamped there for certain and nowhere
//! else.

use super::tests::describe;
use super::*;
use crate::World;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock};

/// Built-in content with nobody hungering (the walk to Ascdūn takes days).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// Fyrdelf's hidden way in on the Ascmearc.
const MINE: Pos = Pos::new(97, 21);

fn level(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("fyrdelf:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

fn creature_ids(level: &LevelTemplate) -> Vec<&str> {
    (level.creatures.iter())
        .map(|c| CONTENT.creature(c.creature).id.as_str())
        .collect()
}

/// Whether `level` holds the foreman's cabin: its plank floor, which no generator lays.
fn has_cabin(level: &LevelTemplate) -> bool {
    let planks = CONTENT.terrain_id("plank_floor").expect("plank floor");
    level.terrain.iter().any(|(_, t)| *t == planks)
}

/// Whether `level` holds the drake's hoard: the drake, which comes from nowhere else.
fn has_hoard(level: &LevelTemplate) -> bool {
    creature_ids(level).contains(&"fire_drake")
}

#[test]
fn first_level_snapshot() {
    insta::assert_snapshot!("fyrdelf_seed_1_depth_1", describe(&level(1, 1)));
}

#[test]
fn twelve_floors_of_danger_eight_to_nineteen_hotter_downwards() {
    let def = CONTENT.dungeon("fyrdelf").expect("Fyrdelf");
    assert_eq!((def.name.as_str(), def.depth), ("Fyrdelf", 12));
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    for depth in 1..=12 {
        let plan = def.plan(depth).expect("a floor");
        let (generator, size, celsius) = match depth {
            1..=4 => (Generator::Rooms, (80, 21), 10),
            5..=8 => (Generator::Caves, (120, 60), 16),
            9..=11 => (Generator::Caves, (120, 60), 20),
            _ => (Generator::Caves, (120, 60), 24),
        };
        assert_eq!(plan.generator, generator, "depth {depth}");
        assert_eq!((plan.width, plan.height), size, "depth {depth}");
        assert_eq!(
            plan.temperature.map(|t| t.celsius()),
            Some(celsius),
            "depth {depth}"
        );
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        // Nothing here grows with the player (the design's decision 13).
        for player_level in [1, 10, 30] {
            assert_eq!(rule.at(at, player_level), depth as u8 + 7, "depth {depth}");
        }
    }
    let key = CONTENT.object_id("brass_key");
    assert_eq!(def.key, key);
}

#[test]
fn the_floors_start_full_of_the_delf_wave() {
    let (_, _, spawning) = CONTENT.spawning("fyrdelf:1").expect("a danger");
    let lives: Vec<&str> = (spawning.population.iter())
        .map(|c| CONTENT.creature(*c).id.as_str())
        .collect();
    for id in [
        "ember_beetle",
        "duergar_miner",
        "duergar_smith",
        "troll",
        "cinder_elemental",
        "lesser_fire_elemental",
        "earth_elemental",
    ] {
        assert!(lives.contains(&id), "{id} lives in Fyrdelf");
    }
    // The drake is the hoard's alone.
    assert!(!lives.contains(&"fire_drake"));
    for (depth, least) in [(1, 25), (5, 80)] {
        let floor = level(1, depth);
        let drawn = (floor.creatures.iter())
            .filter(|c| {
                let drawn = [&spawning.population, &spawning.generic];
                drawn.iter().any(|kinds| kinds.contains(&c.creature))
            })
            .count();
        assert!(drawn >= least, "depth {depth}: {drawn}");
        assert!(!has_hoard(&floor), "depth {depth}");
    }
}

#[test]
fn the_cabin_stands_on_every_gallery_floor_and_the_hoard_at_the_bottom() {
    for vault in ["foremans_cabin", "drakes_hoard"] {
        let v = (CONTENT.vaults().iter())
            .find(|v| v.id() == vault)
            .expect(vault);
        assert_eq!(v.campaign.as_deref(), Some("ascmearc"));
        assert!(v.pinned, "{vault} is stamped only where named");
    }
    let key = CONTENT.object_id("brass_key").expect("the brass key");
    let pick = CONTENT.object_id("pick_axe").expect("a pick-axe");
    for seed in 1..=4 {
        for depth in 1..=4 {
            let floor = level(seed, depth);
            assert!(has_cabin(&floor), "seed {seed} depth {depth}");
            let lying = |o| (floor.objects.values()).any(|pile| pile.iter().any(|s| s.object == o));
            assert!(lying(key) && lying(pick), "seed {seed} depth {depth}");
        }
        for depth in [5, 11] {
            let floor = level(seed, depth);
            assert!(
                !has_cabin(&floor) && !has_hoard(&floor),
                "seed {seed} {depth}"
            );
        }
        let bottom = level(seed, 12);
        let drakes = (creature_ids(&bottom).iter())
            .filter(|c| **c == "fire_drake")
            .count();
        assert_eq!(drakes, 1, "seed {seed}");
        assert!(!has_cabin(&bottom));
        let lava = CONTENT.terrain_id("lava").expect("lava");
        assert!(bottom.terrain.iter().any(|(_, t)| *t == lava));
    }
    // Never drawn at random elsewhere: the Depths have neither.
    for seed in 1..=8 {
        for depth in [1, 3] {
            let floor = generate(
                &CONTENT,
                ("showcase", seed),
                &format!("depths:{depth}"),
                None,
            )
            .expect("generates");
            assert!(!has_cabin(&floor) && !has_hoard(&floor));
        }
    }
}

#[test]
fn duergar_keep_the_galleries_stores() {
    let def = CONTENT.dungeon("fyrdelf").expect("Fyrdelf");
    let odds = def.shop_odds(1).expect("shop odds");
    assert_eq!(odds.chance, 100);
    assert!(def.shop_odds(5).is_some_and(|o| o.chance == 100));
    let trader = CONTENT.creature_id("duergar_trader").expect("the trader");
    let mut shops = 0;
    for seed in 1..=12 {
        for depth in 1..=4 {
            let floor = level(seed, depth);
            for shop in &floor.shops {
                shops += 1;
                assert!(odds.names.contains(&shop.keeper), "{}", shop.keeper);
                let keeper = (floor.creatures.iter())
                    .find(|c| c.pos == shop.post)
                    .expect("a keeper at the post");
                assert_eq!(keeper.creature, trader);
            }
        }
        // The deep workings are caves: no stores there.
        assert!(level(seed, 5).shops.is_empty());
    }
    assert!(shops > 0, "some gallery floor has a store");
}

/// The moves of a shortest walk on the overworld from `from` to `to` (8 directions).
fn walk(from: Pos, to: Pos) -> String {
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    let mut came = BTreeMap::from([(from, None)]);
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        if p == to {
            break;
        }
        for d in Dir::ALL {
            let q = p.step(d);
            let open = (map.terrain.get(q)).is_some_and(|t| CONTENT.terrain(*t).walkable);
            if open && !came.contains_key(&q) {
                came.insert(q, Some((p, d)));
                queue.push_back(q);
            }
        }
    }
    let mut moves = Vec::new();
    let mut at = to;
    while let Some(Some((p, d))) = came.get(&at) {
        moves.push(format!("move:{}", d.name()));
        at = *p;
    }
    moves.reverse();
    moves.join(" ")
}

#[test]
fn the_hidden_way_in_on_ascdun_leads_down_once_revealed_and_back() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("a run");
    let mut replay = Replay::new(&w);
    let start = w.player_pos();
    let mut run = |w: &mut World, actions: &str| {
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    // The quest that reveals it is the town's (H); the dev reveal stands in for it.
    run(
        &mut w,
        &format!("reveal:fyrdelf {} descend", walk(start, MINE)),
    );
    assert_eq!(w.perception().level_id(), "fyrdelf:1");
    assert_eq!(w.perception().level_name(), "Fyrdelf 1");
    assert_eq!(w.danger("fyrdelf:1"), 8);
    run(&mut w, "ascend");
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert_eq!(w.player_pos(), MINE);
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_dev_scenario_starts_on_the_first_floor() {
    let w = World::new(CONTENT.clone(), "fyrdelf", None).expect("a run");
    assert_eq!(w.campaign(), "ascmearc");
    assert_eq!(w.perception().level_id(), "fyrdelf:1");
}

#[test]
fn a_branch_level_names_vaults_of_its_own_campaign() {
    let fyrdelf = "campaigns/ascmearc/data/dungeons/fyrdelf.toml";
    let text = include_str!("../../../../assets/campaigns/ascmearc/data/dungeons/fyrdelf.toml");
    let named = "vaults = [\"drakes_hoard\"]";
    assert!(text.contains(named));
    for (vaults, says) in [
        ("vaults = [\"nowhere\"]", "unknown vault `nowhere`"),
        ("vaults = [\"stone_circle\"]", "is outdoor"),
        (
            "vaults = [\"drakes_hoard\", \"drakes_hoard\"]",
            "names vault `drakes_hoard` twice",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        files.insert(fyrdelf, &text.replace(named, vaults));
        let err = Content::load(&files).expect_err(vaults);
        assert_eq!(err.file, fyrdelf);
        assert!(err.message.contains(says), "{}", err.message);
    }
    // The showcase's Depths may not name the Ascmearc's vaults.
    let depths = "campaigns/showcase/data/dungeons/showcase.toml";
    let showcase =
        include_str!("../../../../assets/campaigns/showcase/data/dungeons/showcase.toml");
    let mut files = ContentFiles::builtin();
    let text = showcase.replace("from = 6\n", "from = 6\nvaults = [\"foremans_cabin\"]\n");
    files.insert(depths, &text);
    let err = Content::load(&files).expect_err("another campaign's vault");
    assert!(err.message.contains("is ascmearc's"), "{}", err.message);
    // A shared one is anyone's.
    let mut files = ContentFiles::builtin();
    let text = showcase.replace("from = 6\n", "from = 6\nvaults = [\"shrine\"]\n");
    files.insert(depths, &text);
    let content = Content::load(&files).expect("a shared vault");
    let shrine = (content.vaults().iter()).find(|v| v.id() == "shrine");
    assert!(shrine.is_some_and(|v| v.pinned));
}
