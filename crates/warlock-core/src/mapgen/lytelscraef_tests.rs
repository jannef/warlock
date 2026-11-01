//! Lytelscræf, the Ascmearc's starter cave (#792): three short cave floors at danger 1 to 3, a
//! thin population of the campaign's weakest folk and nothing else, the kobolds' cache with its
//! chest on the bottom floor for certain, and the way in two cells from Beorggeat.

use super::*;
use crate::World;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use std::collections::BTreeSet;
use std::sync::{Arc, LazyLock};

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// Lytelscræf's way in on the Ascmearc, and Beorggeat beside it.
const CAVE: Pos = Pos::new(14, 27);
const PASS: Pos = Pos::new(12, 28);

const FOLK: [&str; 5] = ["rat", "bat", "beetle", "grave_worm", "kobold"];

fn level(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("lytelscraef:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

fn cache() -> &'static Vault {
    (CONTENT.vaults().iter())
        .find(|v| v.id() == "kobolds_cache")
        .expect("the kobolds' cache")
}

/// The vault's chest cells on `level`, where its terrain is stamped.
fn cache_chests(level: &LevelTemplate) -> Vec<Pos> {
    let vault = cache();
    let v = &vault.template.terrain;
    let fits = |at: Pos| {
        (v.iter()).all(|(p, t)| level.terrain.get(Pos::new(at.x + p.x, at.y + p.y)) == Some(t))
    };
    (0..level.terrain.height())
        .flat_map(|y| (0..level.terrain.width()).map(move |x| Pos::new(x, y)))
        .filter(|at| fits(*at))
        .flat_map(|at| {
            (vault.template.chests.keys()).map(move |p| Pos::new(at.x + p.x, at.y + p.y))
        })
        .collect()
}

#[test]
fn three_short_cave_floors_of_danger_one_to_three() {
    let def = CONTENT.dungeon("lytelscraef").expect("Lytelscræf");
    assert_eq!((def.name.as_str(), def.depth), ("Lytelscræf", 3));
    assert_eq!(CONTENT.level_campaign("lytelscraef:1"), Some("ascmearc"));
    for depth in 1..=3 {
        let plan = def.plan(depth).expect("a floor");
        assert_eq!(plan.generator, Generator::Caves, "depth {depth}");
        assert_eq!((plan.width, plan.height), (60, 30), "depth {depth}");
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        for player_level in [1, 10, 30] {
            assert_eq!(rule.at(at, player_level), depth as u8, "depth {depth}");
        }
    }
}

#[test]
fn only_the_weakest_folk_live_there_and_none_stray_in() {
    let (_, _, spawning) = CONTENT.spawning("lytelscraef:1").expect("a danger");
    let lives: BTreeSet<&str> = (spawning.population.iter())
        .map(|c| CONTENT.creature(*c).id.as_str())
        .collect();
    assert_eq!(lives, BTreeSet::from(FOLK));
    for c in &spawning.population {
        assert!(
            CONTENT.creature(*c).level <= 2,
            "{}",
            CONTENT.creature(*c).id
        );
    }
    for seed in 1..=6 {
        for depth in 1..=3 {
            let floor = level(seed, depth);
            let count = floor.creatures.len();
            // The population's 8 to 16, and the cache's two keepers on the bottom floor.
            let most = if depth == 3 { 18 } else { 16 };
            assert!(
                (8..=most).contains(&count),
                "seed {seed} depth {depth}: {count}"
            );
            for c in &floor.creatures {
                let id = CONTENT.creature(c.creature).id.as_str();
                assert!(FOLK.contains(&id), "seed {seed} depth {depth}: {id}");
            }
        }
    }
}

#[test]
fn the_kobolds_cache_with_its_chest_is_on_the_bottom_floor_for_certain() {
    let vault = cache();
    assert_eq!(vault.campaign.as_deref(), Some("ascmearc"));
    assert!(vault.pinned, "never drawn at random");
    let def = CONTENT.dungeon("lytelscraef").expect("Lytelscræf");
    let named: Vec<u32> = (1..=3).filter(|d| !def.vaults(*d).is_empty()).collect();
    assert_eq!(named, [3]);
    assert_eq!(vault.template.chests.len(), 1, "one chest");
    for seed in 1..=10 {
        let floor = level(seed, 3);
        let chests = cache_chests(&floor);
        assert_eq!(chests.len(), 1, "seed {seed}: the cache is stamped once");
        let chest = &floor.chests[&chests[0]];
        assert_eq!(
            chest.level,
            CONTENT.item_level("lytelscraef:3"),
            "seed {seed}"
        );
        assert!(!chest.holds.is_empty(), "seed {seed}: it holds something");
        // Neither locked nor trapped.
        assert!(!floor.locks.contains_key(&chests[0]), "seed {seed}");
        assert!(!floor.traps.contains_key(&chests[0]), "seed {seed}");
        for depth in [1, 2] {
            assert!(
                cache_chests(&level(seed, depth)).is_empty(),
                "seed {seed} {depth}"
            );
        }
    }
}

#[test]
fn the_way_in_beside_beorggeat_leads_down_and_back() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("a run");
    assert_eq!(w.player_pos(), PASS);
    assert!(w.perception().is_visible(CAVE), "in sight from the start");
    let mut replay = Replay::new(&w);
    // A step east on the road, one north-east to the wall's foot, and down.
    for action in parse_actions("move:E move:NE descend").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.perception().level_id(), "lytelscraef:1");
    assert_eq!(w.perception().level_name(), "Lytelscræf 1");
    assert_eq!(w.danger("lytelscraef:1"), 1);
    for action in parse_actions("ascend").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert_eq!(w.player_pos(), CAVE);
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
