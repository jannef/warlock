//! Fennmynster, the Ascmearc's first hidden dungeon (#338; docs/design/campaign.md §7): the
//! branch's floors and danger, its population, its first level (a snapshot), its raised odds of
//! flooded rooms and its lack of shops, the hidden way in in Micelfenn, and the sunken crypt its
//! bottom names, stamped there for certain and nowhere else.

use super::tests::describe;
use super::*;
use crate::World;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, LazyLock};

/// Built-in content with nobody hungering (the walk to Micelfenn takes days).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// Fennmynster's hidden way in on the Ascmearc.
const MINSTER: Pos = Pos::new(70, 48);

fn level(seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("fennmynster:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

/// Where `level`'s sunken crypt has its altar, if it holds one: an altar two steps from shallow
/// water (the shared shrine's altar stands on dry floor, and a flooded room has no altar).
fn crypt_altar(level: &LevelTemplate) -> Option<Pos> {
    let altar = CONTENT.terrain_id("altar").expect("an altar");
    let water = CONTENT.terrain_id("shallow_water").expect("shallow water");
    let wet = |p: Pos| level.terrain.get(p) == Some(&water);
    (level.terrain.iter())
        .find(|(p, t)| **t == altar && wet(Pos::new(p.x - 2, p.y)))
        .map(|(p, _)| p)
}

#[test]
fn first_level_snapshot() {
    insta::assert_snapshot!("fennmynster_seed_1_depth_1", describe(&level(1, 1)));
}

#[test]
fn eight_cellar_floors_of_danger_four_to_eleven() {
    let def = CONTENT.dungeon("fennmynster").expect("Fennmynster");
    assert_eq!((def.name.as_str(), def.depth), ("Fennmynster", 8));
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    for depth in 1..=8 {
        let plan = def.plan(depth).expect("a floor");
        assert_eq!(plan.generator, Generator::Rooms, "depth {depth}");
        assert_eq!((plan.width, plan.height), (80, 21), "depth {depth}");
        assert_eq!(plan.temperature.map(|t| t.celsius()), Some(6));
        let (rule, at, _) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        // Nothing here grows with the player (the design's decision 13).
        for player_level in [1, 10, 30] {
            assert_eq!(rule.at(at, player_level), depth as u8 + 3, "depth {depth}");
        }
    }
    assert_eq!(def.key, CONTENT.object_id("iron_key"));
}

#[test]
fn the_floors_start_full_of_the_fen_wave() {
    let (_, _, spawning) = CONTENT.spawning("fennmynster:1").expect("a danger");
    let lives: BTreeSet<&str> = (spawning.population.iter())
        .map(|c| CONTENT.creature(*c).id.as_str())
        .collect();
    assert_eq!(
        lives,
        BTreeSet::from([
            "giant_leech",
            "giant_frog",
            "drowned",
            "crocodile",
            "cultist",
            "will_o_wisp",
            "ghost",
            "mud_golem",
            "cult_priest",
            "marsh_troll",
            "bog_hag",
        ])
    );
    for depth in [1, 5] {
        let floor = level(1, depth);
        let drawn = (floor.creatures.iter())
            .filter(|c| spawning.population.contains(&c.creature))
            .count();
        assert!(drawn >= 25, "depth {depth}: {drawn}");
    }
}

#[test]
fn the_fen_floods_its_rooms_and_no_one_keeps_a_shop() {
    let def = CONTENT.dungeon("fennmynster").expect("Fennmynster");
    let effects = |ids: &[&str]| -> Vec<_> {
        (ids.iter())
            .map(|id| CONTENT.room_effects().id(id).expect(id))
            .collect()
    };
    let flooded = CONTENT.room_effects().id("flooded").expect("flooded");
    let allowed = effects(&["flooded", "hushed", "shadowed"]);
    for depth in 1..=8 {
        let odds = def.room_odds(depth).expect("room odds");
        assert_eq!((odds.chance, odds.per_level), (25, 2), "depth {depth}");
        assert_eq!(odds.effects, allowed, "depth {depth}");
        assert_eq!(def.shop_odds(depth).map(|o| o.chance), Some(0));
    }
    let (mut special, mut floods) = (0, 0);
    for seed in 1..=12 {
        for depth in [1, 4, 7] {
            let floor = level(seed, depth);
            assert!(floor.shops.is_empty(), "seed {seed} depth {depth}");
            assert!(floor.rooms.len() <= 2, "seed {seed} depth {depth}");
            for room in &floor.rooms {
                assert!(allowed.contains(&room.effect));
                special += 1;
                floods += usize::from(room.effect == flooded);
            }
        }
    }
    // The Depths' shared odds (7 %, one a level) give about one level in three a special room;
    // the fen's give most levels one or two, and a third of them flooded.
    assert!(special >= 36, "{special} special rooms on 36 levels");
    assert!(floods >= 8, "{floods} flooded rooms");
}

#[test]
fn the_sunken_crypt_lies_at_the_bottom_and_nowhere_else() {
    let v = (CONTENT.vaults().iter())
        .find(|v| v.id() == "sunken_crypt")
        .expect("the sunken crypt");
    assert_eq!(v.campaign.as_deref(), Some("ascmearc"));
    assert!(v.pinned, "stamped only where named");
    let drowned = CONTENT.creature_id("drowned").expect("the drowned");
    let water = CONTENT.terrain_id("shallow_water").expect("shallow water");
    for seed in 1..=4 {
        let bottom = level(seed, 8);
        let on_altar = crypt_altar(&bottom).expect("the crypt");
        // Its keepers stand in its water, and its altar holds the minster's treasure.
        let keepers = (bottom.creatures.iter())
            .filter(|c| c.creature == drowned && bottom.terrain.get(c.pos) == Some(&water))
            .count();
        assert!(keepers >= 2, "seed {seed}: {keepers}");
        assert!(bottom.objects.contains_key(&on_altar), "seed {seed}");
        for depth in [1, 7] {
            let floor = level(seed, depth);
            assert_eq!(crypt_altar(&floor), None, "seed {seed} depth {depth}");
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
fn the_hidden_way_in_in_micelfenn_leads_down_once_revealed_and_back() {
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
        &format!("reveal:fennmynster {} descend", walk(start, MINSTER)),
    );
    assert_eq!(w.perception().level_id(), "fennmynster:1");
    assert_eq!(w.perception().level_name(), "Fennmynster 1");
    assert_eq!(w.danger("fennmynster:1"), 4);
    run(&mut w, "ascend");
    assert_eq!(w.perception().level_id(), "ascmearc");
    assert_eq!(w.player_pos(), MINSTER);
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_dev_scenario_starts_on_the_first_floor() {
    let w = World::new(CONTENT.clone(), "fennmynster", None).expect("a run");
    assert_eq!(w.campaign(), "ascmearc");
    assert_eq!(w.perception().level_id(), "fennmynster:1");
}
