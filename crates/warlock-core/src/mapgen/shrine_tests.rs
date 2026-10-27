//! Shrines of a side in generated levels (spec §7 "Special rooms", `special.rs`): one stone a
//! room, its side drawn good or evil one to one (never neutral), shown by its light.

use super::*;
use crate::alignment::Alignment;
use crate::content::ContentFiles;

/// The built-in content with every room of a generated level special and shrines drawn often.
fn content() -> Content {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", "chance = 100\n");
        text.replace(
            "weight = 4\nmin_cells = 9\nfeature = \"shrine_stone\"",
            "weight = 100\nmin_cells = 9\nfeature = \"shrine_stone\"",
        )
    });
    Content::load(&files).expect("fixture content")
}

#[test]
fn shrines_are_good_or_evil_one_to_one_with_their_sides_lights() {
    let content = content();
    let stone = content.terrain_id("shrine_stone").expect("stone");
    let effect = content.room_effects().id("shrine").expect("shrine");
    let def = content.room_effect(effect);
    let (mut good, mut evil) = (0u32, 0u32);
    for seed in 0..120 {
        let level = generate(&content, ("showcase", seed), "depths:2", None).expect("level");
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            let at: Vec<Pos> = (room.cells().into_iter())
                .filter(|&p| level.terrain.get(p) == Some(&stone))
                .collect();
            assert_eq!(at.len(), 1, "seed {seed}: one stone a room");
            let side = room.side.expect("a side");
            match side {
                Alignment::Good => good += 1,
                Alignment::Evil => evil += 1,
                Alignment::Neutral => panic!("seed {seed}: a neutral shrine"),
            }
            assert!(def.side_light(side).is_some(), "always a light of its side");
            assert_eq!(level.lights.get(&at[0]).copied(), def.side_light(side));
        }
        assert!(connect::validate(&level, &content).is_ok(), "seed {seed}");
    }
    let total = good + evil;
    assert!(total >= 50, "{good} {evil}");
    assert!((good * 100 / total).abs_diff(50) <= 15, "{good} {evil}");
}
