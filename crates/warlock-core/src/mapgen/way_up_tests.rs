//! Ways up (spec §7 "Ways up", #1117): Ūtgang, the three rooms floors between the bottom of
//! Fyrdelf and the overworld. Its floors and their links, the staircase up that Fyrdelf 12 gains
//! beside the drake's hoard (and no other floor does), and the content check on `below`.

use super::connect::distances;
use super::*;
use crate::content::ContentFiles;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> = LazyLock::new(|| Content::builtin().expect("content"));

fn level(id: &str, seed: u64, from: Option<&str>) -> LevelTemplate {
    generate(&CONTENT, ("ascmearc", seed), id, from).expect("generates")
}

fn link(level: &LevelTemplate, dir: StairsDir) -> Vec<Option<&str>> {
    (level.stairs.iter())
        .filter(|s| s.dir == dir)
        .map(|s| s.to.as_deref())
        .collect()
}

#[test]
fn three_rooms_floors_from_the_overworld_down_to_the_drakes_deep() {
    let def = CONTENT.dungeon("utgang").expect("Ūtgang");
    assert_eq!((def.name.as_str(), def.depth), ("Ūtgang", 3));
    assert_eq!(def.below.as_deref(), Some("fyrdelf:12"));
    for depth in 1..=3 {
        let plan = def.plan(depth).expect("a floor");
        assert_eq!(plan.generator, Generator::Rooms, "no caverns ({depth})");
    }
    assert!(def.plan(4).is_none());
    for seed in 1..=4 {
        // However the top floor is first reached, its `<` climbs out to the overworld.
        for from in [Some("ascmearc"), Some("utgang:2"), None] {
            let top = level("utgang:1", seed, from);
            assert_eq!(link(&top, StairsDir::Up), [Some("ascmearc")], "{from:?}");
            assert_eq!(link(&top, StairsDir::Down), [Some("utgang:2")]);
        }
        let mid = level("utgang:2", seed, None);
        assert_eq!(link(&mid, StairsDir::Up), [Some("utgang:1")]);
        assert_eq!(link(&mid, StairsDir::Down), [Some("utgang:3")]);
        let bottom = level("utgang:3", seed, None);
        assert_eq!(link(&bottom, StairsDir::Up), [Some("utgang:2")]);
        assert_eq!(link(&bottom, StairsDir::Down), [Some("fyrdelf:12")]);
    }
}

#[test]
fn the_drakes_floor_gets_stairs_up_beside_the_hoard_and_no_other_floor_does() {
    for seed in 1..=12 {
        let floor = level("fyrdelf:12", seed, None);
        assert_eq!(
            link(&floor, StairsDir::Up),
            [Some("fyrdelf:11"), Some("utgang:3")],
            "seed {seed}"
        );
        assert!(link(&floor, StairsDir::Down).is_empty());
        // Near the hoard: a short walk from the drake, which sleeps on it.
        let way = floor.stairs[1].pos;
        let drake = CONTENT.creature_id("fire_drake").expect("the drake");
        let lair = (floor.creatures.iter())
            .find(|c| c.creature == drake)
            .expect("the drake")
            .pos;
        let (w, h) = (floor.terrain.width(), floor.terrain.height());
        let dist = distances(w, h, way, |p| {
            let def = CONTENT.terrain(floor.terrain[p]);
            def.walkable || def.is_closed_door()
        });
        assert!(dist[lair] <= 30, "seed {seed}: {} steps", dist[lair]);
    }
    for depth in [1, 11] {
        let floor = level(&format!("fyrdelf:{depth}"), 1, None);
        assert_eq!(link(&floor, StairsDir::Up).len(), 1, "fyrdelf:{depth}");
    }
}

#[test]
fn below_names_a_level_of_another_bounded_branch() {
    let file = "campaigns/ascmearc/data/dungeons/utgang.toml";
    let text = include_str!("../../../../assets/campaigns/ascmearc/data/dungeons/utgang.toml");
    for (bad, why) in [
        ("utgang:2", "its own"),
        ("fyrdelf:13", "no such level"),
        ("grundleas:1", "endless"),
        ("depths:3", "another campaign's"),
    ] {
        let mut files = ContentFiles::builtin();
        files.insert(
            file,
            &text.replace("below = \"fyrdelf:12\"", &format!("below = \"{bad}\"")),
        );
        let err = Content::load(&files).expect_err(why);
        assert!(err.message.contains("`below"), "{why}: {}", err.message);
    }
}
