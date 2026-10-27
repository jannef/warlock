//! Scriptoria in generated levels (spec §7 "Special rooms", `room_loot.rs`): shelves at the
//! room's middle, two to four scrolls on its floor from the swarm stream, and the rest of the
//! level the same as without them.

use super::*;
use crate::content::ContentFiles;

/// The built-in content with every room of a generated level special and scriptoria drawn
/// often; without their `loot` when `loot` is false.
fn content(loot: bool) -> Content {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", "chance = 100\n");
        let text = text.replace(
            "weight = 5\nmin_cells = 9\nfeature",
            "weight = 100\nmin_cells = 9\nfeature",
        );
        if loot {
            text
        } else {
            text.replace("loot = { class = \"scrolls\", count = [2, 4] }\n", "")
        }
    });
    Content::load(&files).expect("fixture content")
}

/// How many items of item class `class` lie on `cells`.
fn count_in(level: &LevelTemplate, content: &Content, cells: &[Pos], class: &str) -> u32 {
    let class = content.item_class_id(class).expect("class");
    (cells.iter())
        .filter_map(|p| level.objects.get(p))
        .flatten()
        .filter(|s| content.object(s.object).class == class)
        .map(|s| s.count)
        .sum()
}

#[test]
fn scriptoria_have_shelves_and_two_to_four_scrolls() {
    let (with, without) = (content(true), content(false));
    let shelves = with.terrain_id("shelves").expect("shelves");
    let effect = with.room_effects().id("scriptorium").expect("scriptorium");
    let (mut rooms, mut counts) = (0, [0u32; 5]);
    for seed in 0..80 {
        let level = generate(&with, ("showcase", seed), "depths:2", None).expect("level");
        let plain = generate(&without, ("showcase", seed), "depths:2", None).expect("level");
        assert_eq!(level.terrain, plain.terrain, "seed {seed}: the same map");
        assert_eq!(
            level.creatures, plain.creatures,
            "seed {seed}: the same creatures"
        );
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            rooms += 1;
            let cells = room.cells();
            let feature = (cells.iter())
                .filter(|&&p| level.terrain.get(p) == Some(&shelves))
                .count();
            assert_eq!(feature, 1, "seed {seed}: one shelf a room");
            let laid = count_in(&level, &with, &cells, "scrolls")
                - count_in(&plain, &without, &cells, "scrolls");
            assert!((2..=4).contains(&laid), "seed {seed}: {laid} scrolls");
            counts[laid as usize] += 1;
        }
        assert!(connect::validate(&level, &with).is_ok(), "seed {seed}");
    }
    assert!(rooms >= 40, "{rooms} scriptoria");
    assert!(counts[2..].iter().all(|&n| n > 0), "{counts:?}");
}
