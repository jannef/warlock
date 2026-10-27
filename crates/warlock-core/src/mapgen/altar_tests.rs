//! Forgotten altars in generated levels (spec §7 "Special rooms", `special.rs`): one altar a
//! room at the free floor nearest its middle, never beside a door, the room still whole; its
//! side drawn one good to two neutral to one evil, shown by its light.

use super::*;
use crate::alignment::Alignment;
use crate::content::ContentFiles;

/// The built-in content with every room of a generated level special and altars drawn often.
fn content() -> Content {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", "chance = 100\n");
        text.replace(
            "weight = 8\nmin_cells = 9\nfeature",
            "weight = 100\nmin_cells = 9\nfeature",
        )
    });
    Content::load(&files).expect("fixture content")
}

#[test]
fn altars_stand_inside_their_rooms_with_their_sides_lights() {
    let content = content();
    let altar = content.terrain_id("forgotten_altar").expect("altar");
    let effect = content.room_effects().id("altar").expect("altar room");
    let def = content.room_effect(effect);
    let mut sides = [0u32; 3];
    for seed in 0..160 {
        let level = generate(&content, ("showcase", seed), "depths:2", None).expect("level");
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            let cells = room.cells();
            let feature: Vec<Pos> = (cells.iter().copied())
                .filter(|&p| level.terrain.get(p) == Some(&altar))
                .collect();
            assert_eq!(feature.len(), 1, "seed {seed}: one altar a room");
            let at = feature[0];
            for d in Dir::ALL {
                let t = level.terrain.get(at.step(d)).copied().expect("inside");
                assert!(
                    !content.terrain(t).is_door(),
                    "seed {seed}: no door beside it"
                );
            }
            let side = room.side.expect("a side");
            sides[Alignment::ALL
                .iter()
                .position(|&a| a == side)
                .expect("side")] += 1;
            assert_eq!(
                level.lights.get(&at).copied(),
                def.side_light(side),
                "seed {seed}"
            );
        }
        assert!(connect::validate(&level, &content).is_ok(), "seed {seed}");
    }
    let total: u32 = sides.iter().sum();
    assert!(total >= 60, "{sides:?}");
    // One good to two neutral to one evil.
    let share = |n: u32| n * 100 / total;
    assert!(share(sides[0]).abs_diff(25) <= 12, "{sides:?}");
    assert!(share(sides[1]).abs_diff(50) <= 12, "{sides:?}");
    assert!(share(sides[2]).abs_diff(25) <= 12, "{sides:?}");
}
