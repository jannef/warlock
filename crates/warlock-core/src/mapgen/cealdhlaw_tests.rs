//! Cealdhlāw, the Ascmearc's first dungeon (#337; docs/design/campaign.md §7): the branch's
//! floors and danger, its population, its first level (a snapshot), the way in from Hlāwdūn,
//! and the barrow chamber, a campaign vault stamped on its second floor for certain, with the
//! silver chalice of Sister Hild's quest on its plinth.

use super::tests::describe_in;
use super::*;
use crate::World;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock};

/// Built-in content with nobody hungering (the walk to the barrow takes most of a day).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// Cealdhlāw's way in on the Ascmearc.
const BARROW: Pos = Pos::new(34, 25);

fn level(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("cealdhlaw:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

fn chamber() -> &'static Vault {
    (CONTENT.vaults().iter())
        .find(|v| v.id() == "barrow_chamber")
        .expect("the barrow chamber")
}

/// Whether `level` holds the barrow chamber: its locked door, its guardian with the key.
fn has_chamber(level: &LevelTemplate) -> bool {
    let key = CONTENT.object_id("bronze_key").expect("the bronze key");
    let door = (level.locks.values()).any(|l| l.id.as_deref() == Some("barrow_chamber"));
    let guardian = level.creatures.iter().any(|c| {
        CONTENT.creature(c.creature).id == "skeleton" && c.carry.iter().any(|s| s.object == key)
    });
    assert_eq!(door, guardian, "the door and its key come together");
    door
}

#[test]
fn first_level_snapshot() {
    insta::assert_snapshot!(
        "cealdhlaw_seed_1_depth_1",
        describe_in(&CONTENT, &level(1, 1))
    );
}

#[test]
fn six_floors_of_danger_one_to_six_passages_then_hollows() {
    let def = CONTENT.dungeon("cealdhlaw").expect("Cealdhlāw");
    assert_eq!((def.name.as_str(), def.depth), ("Cealdhlāw", 6));
    assert_eq!(CONTENT.level_campaign("cealdhlaw:1"), Some("ascmearc"));
    for depth in 1..=6 {
        let plan = def.plan(depth).expect("a floor");
        let (generator, size) = match depth {
            1 | 2 => (Generator::Rooms, (80, 21)),
            _ => (Generator::Caves, (100, 40)),
        };
        assert_eq!(plan.generator, generator, "depth {depth}");
        assert_eq!((plan.width, plan.height), size, "depth {depth}");
        assert_eq!(
            plan.temperature.map(|t| t.celsius()),
            Some(8),
            "depth {depth}"
        );
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        // Nothing here grows with the player (the design's decision 13).
        for player_level in [1, 10, 30] {
            assert_eq!(rule.at(at, player_level), depth as u8, "depth {depth}");
        }
    }
}

#[test]
fn the_floors_start_full_of_the_barrow_wave() {
    let (_, _, spawning) = CONTENT.spawning("cealdhlaw:1").expect("a danger");
    let lives: Vec<&str> = (spawning.population.iter())
        .map(|c| CONTENT.creature(*c).id.as_str())
        .collect();
    for id in [
        "rat",
        "kobold_shaman",
        "grave_worm",
        "goblin_chief",
        "barrow_wight",
    ] {
        assert!(lives.contains(&id), "{id} lives in Cealdhlāw");
    }
    for (depth, least, most) in [(1, 25, 45), (3, 50, 80)] {
        let floor = level(1, depth);
        let count = floor.creatures.len();
        assert!((least..=most).contains(&count), "depth {depth}: {count}");
        // A shop's keeper aside, everyone was drawn from those who live there or the generic
        // list strays come from.
        let keeper = CONTENT.creature_id("shopkeeper").expect("the keeper");
        for c in &floor.creatures {
            let lives = [&spawning.population, &spawning.generic]
                .iter()
                .any(|kinds| kinds.contains(&c.creature));
            assert!(lives || c.creature == keeper, "depth {depth}");
        }
    }
}

#[test]
fn the_barrow_chamber_with_the_chalice_is_on_the_second_floor_for_certain() {
    let vault = chamber();
    assert_eq!(vault.campaign.as_deref(), Some("ascmearc"));
    assert!(vault.pinned, "never drawn at random");
    let def = CONTENT.dungeon("cealdhlaw").expect("Cealdhlāw");
    let depths = CONTENT.dungeon("depths").expect("the Depths");
    let named: Vec<u32> = (1..=6).filter(|d| !def.vaults(*d).is_empty()).collect();
    assert_eq!(named, [2]);
    assert_eq!(def.vaults(2), ["barrow_chamber"]);
    assert!((1..=depths.depth).all(|d| !vault.stamps_on(depths, d)));
    // The shared vaults stamp here too.
    let guard = (CONTENT.vaults().iter())
        .find(|v| v.id() == "guard_post")
        .expect("the guard post");
    assert!((1..=6).all(|d| guard.stamps_on(def, d)));

    let chalice = CONTENT.object_id("silver_chalice").expect("the chalice");
    let altar = CONTENT.terrain_id("altar").expect("an altar");
    let chalices = |level: &LevelTemplate| {
        (level.objects.iter())
            .flat_map(|(at, stacks)| stacks.iter().map(move |s| (*at, s)))
            .filter(|(_, s)| s.object == chalice)
            .map(|(at, _)| at)
            .collect::<Vec<_>>()
    };
    for seed in 1..=8 {
        let floor = level(seed, 2);
        assert!(has_chamber(&floor), "seed {seed}");
        let on = chalices(&floor);
        assert_eq!(on.len(), 1, "seed {seed}: one chalice");
        assert_eq!(
            floor.terrain.get(on[0]),
            Some(&altar),
            "seed {seed}: on the plinth"
        );
        for depth in [1, 3] {
            let other = level(seed, depth);
            assert!(!has_chamber(&other), "seed {seed} depth {depth}");
            assert!(chalices(&other).is_empty(), "seed {seed} depth {depth}");
        }
    }
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
fn the_way_in_on_hlawdun_leads_down_and_back() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("a run");
    let mut replay = Replay::new(&w);
    let start = w.player_pos();
    let actions = format!("{} descend", walk(start, BARROW));
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.perception().level_id(), "cealdhlaw:1");
    assert_eq!(w.perception().level_name(), "Cealdhlāw 1");
    assert!(w.danger("cealdhlaw:1") == 1);
    for action in parse_actions("ascend").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert_eq!(w.player_pos(), BARROW);
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_dev_scenario_starts_on_the_first_floor() {
    let w = World::new(CONTENT.clone(), "cealdhlaw", None).expect("a run");
    assert_eq!(w.campaign(), "ascmearc");
    assert_eq!(w.perception().level_id(), "cealdhlaw:1");
}

/// The content files with `path` set to `text`, loaded.
fn load_with(path: &str, text: &str) -> Result<Content, crate::content::ContentError> {
    let mut files = ContentFiles::builtin();
    files.insert(path, text);
    Content::load(&files)
}

#[test]
fn a_vaults_dungeons_are_its_own_campaigns_branches() {
    const VAULT: &str = "name = \"V\"\ndungeons = [DUNGEONS]\nmap = '''\n###\n#.#\n#+#\n'''\n";
    let vault = |dungeons: &str| VAULT.replace("DUNGEONS", dungeons);
    let ours = "campaigns/ascmearc/vaults/v.toml";
    assert!(load_with(ours, &vault("\"cealdhlaw\", \"cealdhlaw:6\"")).is_ok());
    for (path, dungeons, says) in [
        (
            ours,
            "\"nowhere\"",
            "unknown branch or branch level `nowhere`",
        ),
        (
            ours,
            "\"cealdhlaw:7\"",
            "unknown branch or branch level `cealdhlaw:7`",
        ),
        (ours, "\"depths\"", "`depths` is showcase's"),
        (
            "vaults/v.toml",
            "\"cealdhlaw\"",
            "`cealdhlaw` is ascmearc's",
        ),
    ] {
        let err = load_with(path, &vault(dungeons)).expect_err(dungeons);
        assert_eq!(err.file, path);
        assert!(err.message.contains(says), "{}", err.message);
    }
    let level = "name = \"L\"\ndungeons = [\"cealdhlaw\"]\nmap = '''\n...\n'''\n";
    let err = load_with("levels/l.toml", level).expect_err("a level");
    assert!(err.message.contains("only a vault"), "{}", err.message);
}
