//! The four lairs of the wanted (#426; docs/design/adventurers-guild.md §8): Wyrtholm, Wulfeshol,
//! Bānfeld, and Brocenburg, each four floors at danger 6 to 9 with its criminal's pinned vault at
//! the bottom (a snapshot of each bottom floor), their hidden ways in on reserved sites of the
//! Ascmearc, and the descent from each once the dev reveal marks it.

use super::tests::describe;
use super::*;
use crate::World;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock};

/// Built-in content with nobody hungering (the walks out to the lairs take days).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// Each lair: its branch, name, criminal, vault, generator and size, the terrain its way in
/// shows once revealed, the overworld cell (its terrain while hidden), and its arrival line.
struct Lair {
    id: &'static str,
    name: &'static str,
    criminal: &'static str,
    vault: &'static str,
    generator: Generator,
    size: (i32, i32),
    entrance: &'static str,
    at: Pos,
    hidden_as: &'static str,
    arrival: &'static str,
}

const LAIRS: [Lair; 4] = [
    Lair {
        id: "wyrtholm",
        name: "Wyrtholm",
        criminal: "sallow",
        vault: "still_room",
        generator: Generator::Caves,
        size: (100, 40),
        entrance: "cave_mouth",
        at: Pos::new(88, 46),
        hidden_as: "swamp",
        arrival: "The air under the islet is thick with rot, and with something sweeter.",
    },
    Lair {
        id: "wulfeshol",
        name: "Wulfeshol",
        criminal: "wystan",
        vault: "robbers_hoard",
        generator: Generator::Caves,
        size: (100, 40),
        entrance: "cave_mouth",
        at: Pos::new(18, 49),
        hidden_as: "dense_forest",
        arrival: "Somebody has swept the cave mouth. Somebody keeps a watch.",
    },
    Lair {
        id: "banfeld",
        name: "Bānfeld",
        criminal: "osrick",
        vault: "unhallowed_chapel",
        generator: Generator::Rooms,
        size: (80, 21),
        entrance: "barrow",
        at: Pos::new(42, 31),
        hidden_as: "hills",
        arrival: "Fresh earth on old graves. Someone has been digging.",
    },
    Lair {
        id: "brocenburg",
        name: "Brocenburg",
        criminal: "herrick",
        vault: "captains_hall",
        generator: Generator::Rooms,
        size: (80, 21),
        entrance: "dungeon_entrance",
        at: Pos::new(66, 12),
        hidden_as: "forest",
        arrival: "Goblin marks on the old fort's stones, and a soldier's boot-prints among them.",
    },
];

fn level(lair: &str, seed: u64, depth: u32) -> LevelTemplate {
    let id = format!("{lair}:{depth}");
    generate(&CONTENT, ("ascmearc", seed), &id, Some("ascmearc")).expect("generates")
}

fn count_of(level: &LevelTemplate, creature: &str) -> usize {
    (level.creatures.iter())
        .filter(|c| CONTENT.creature(c.creature).id == creature)
        .count()
}

#[test]
fn wyrtholm_bottom_snapshot() {
    insta::assert_snapshot!(
        "wyrtholm_seed_1_depth_4",
        describe(&level("wyrtholm", 1, 4))
    );
}

#[test]
fn wulfeshol_bottom_snapshot() {
    insta::assert_snapshot!(
        "wulfeshol_seed_1_depth_4",
        describe(&level("wulfeshol", 1, 4))
    );
}

#[test]
fn banfeld_bottom_snapshot() {
    insta::assert_snapshot!("banfeld_seed_1_depth_4", describe(&level("banfeld", 1, 4)));
}

#[test]
fn brocenburg_bottom_snapshot() {
    insta::assert_snapshot!(
        "brocenburg_seed_1_depth_4",
        describe(&level("brocenburg", 1, 4))
    );
}

#[test]
fn four_floors_of_danger_six_to_nine() {
    for lair in &LAIRS {
        let def = CONTENT.dungeon(lair.id).expect(lair.id);
        assert_eq!((def.name.as_str(), def.depth), (lair.name, 4));
        assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
        for depth in 1..=4 {
            let plan = def.plan(depth).expect("a floor");
            assert_eq!(plan.generator, lair.generator, "{} {depth}", lair.id);
            assert_eq!((plan.width, plan.height), lair.size, "{} {depth}", lair.id);
            let (rule, at, spawning) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
            for player_level in [1, 10, 30] {
                assert_eq!(rule.at(at, player_level), depth as u8 + 5, "{}", lair.id);
            }
            // The criminal lives in no population: the vault alone places them.
            let criminal = CONTENT.creature_id(lair.criminal).expect("the criminal");
            assert!(!spawning.population.contains(&criminal), "{}", lair.id);
            assert!(!spawning.generic.contains(&criminal), "{}", lair.id);
        }
        assert_eq!(CONTENT.arrival(&def.level_id(1)), Some(lair.arrival));
    }
}

#[test]
fn each_criminal_waits_in_their_vault_at_the_bottom_and_nowhere_else() {
    for lair in &LAIRS {
        let vault = (CONTENT.vaults().iter())
            .find(|v| v.id() == lair.vault)
            .expect(lair.vault);
        assert_eq!(vault.campaign.as_deref(), Some("ascmearc"));
        assert!(vault.pinned, "{} is stamped only where named", lair.vault);
        for seed in 1..=4 {
            for depth in 1..=3 {
                let floor = level(lair.id, seed, depth);
                assert_eq!(count_of(&floor, lair.criminal), 0, "{} {depth}", lair.id);
            }
            let bottom = level(lair.id, seed, 4);
            assert_eq!(
                count_of(&bottom, lair.criminal),
                1,
                "{} seed {seed}",
                lair.id
            );
            assert!(
                !bottom.stairs.iter().any(|s| s.dir == StairsDir::Down),
                "the bottom"
            );
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
fn each_hidden_way_in_leads_down_once_revealed() {
    for lair in &LAIRS {
        let (at, hidden) = CONTENT.hidden_location("ascmearc", lair.id).expect(lair.id);
        assert_eq!(at, lair.at, "{}", lair.id);
        let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
        assert_eq!(CONTENT.terrain(map.terrain[at]).id, lair.hidden_as);
        assert_eq!(CONTENT.terrain(hidden.terrain).id, lair.entrance);
        assert_eq!(hidden.name, lair.name);

        let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("a run");
        let start = w.player_pos();
        for action in parse_actions(&walk(start, at)).expect("actions") {
            w.apply(&action);
        }
        assert_eq!(w.player_pos(), at, "{}", lair.id);
        assert_eq!(w.perception().leads_to(at), None, "hidden: no way down");
        for action in parse_actions(&format!("reveal:{} descend", lair.id)).expect("actions") {
            w.apply(&action);
        }
        assert_eq!(w.perception().level_id(), format!("{}:1", lair.id));
        assert_eq!(w.perception().level_name(), format!("{} 1", lair.name));
        assert_eq!(w.danger(&format!("{}:1", lair.id)), 6);
    }
}
