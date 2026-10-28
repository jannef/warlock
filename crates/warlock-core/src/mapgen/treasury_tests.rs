//! Treasure rooms in generated levels (spec §7 "Special rooms", `treasury.rs`): every door of a
//! rooms level's treasure room secret and the level valid; a cave's a box in the rock with one
//! secret door; chests, gold, and gems as the dice say, from their own stream.

use super::*;
use crate::content::ContentFiles;

/// The built-in content with every generated level's one special room drawn, a treasure room
/// most often; without its `treasure` when `treasure` is false.
fn content(treasure: bool) -> Content {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| {
        let text = text.replace("chance = 12\n", "chance = 100\n");
        let text = text.replace(
            "weight = 4\nmin_cells = 9\nsecret_doors",
            "weight = 100\nmin_cells = 9\nsecret_doors",
        );
        if treasure {
            text
        } else {
            let start = text.find("treasure = {").expect("treasure");
            let end = start + text[start..].find('\n').expect("line");
            format!("{}{}", &text[..start], &text[end + 1..])
        }
    });
    Content::load(&files).expect("fixture content")
}

/// The cells of `room`'s rim.
fn rim(room: &SpecialRoom) -> Vec<Pos> {
    (room.cells().into_iter())
        .filter(|p| {
            p.x == room.min.x || p.x == room.max.x || p.y == room.min.y || p.y == room.max.y
        })
        .collect()
}

/// How many of object `object` lie on `cells`.
fn count_in(level: &LevelTemplate, cells: &[Pos], object: ObjectId) -> u32 {
    (cells.iter())
        .filter_map(|p| level.objects.get(p))
        .flatten()
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

/// The cells of `map` outside `room`.
fn outside<T>(map: &BTreeMap<Pos, T>, room: &SpecialRoom) -> Vec<Pos> {
    map.keys().copied().filter(|p| !room.contains(*p)).collect()
}

/// Treasure rooms of `id` over `seeds`, each checked by `check`; returns how many.
fn rooms_of(
    content: &Content,
    id: &str,
    seeds: std::ops::Range<u64>,
    mut check: impl FnMut(u64, &LevelTemplate, &SpecialRoom),
) -> usize {
    let effect = content.room_effects().id("treasury").expect("treasury");
    let mut found = 0;
    for seed in seeds {
        let level = generate(content, ("showcase", seed), id, None).expect("level");
        assert!(connect::validate(&level, content).is_ok(), "seed {seed}");
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            found += 1;
            check(seed, &level, room);
        }
    }
    found
}

#[test]
fn a_rooms_level_hides_every_door_of_its_treasure_room() {
    let content = content(true);
    let found = rooms_of(&content, "depths:2", 0..200, |seed, level, room| {
        let secrets = rim(room)
            .into_iter()
            .filter(|p| level.secrets.contains_key(p));
        assert!(secrets.count() >= 1, "seed {seed}: a way in");
        for p in rim(room) {
            let def = content.terrain(level.terrain[p]);
            assert!(
                !def.walkable && !def.is_door(),
                "seed {seed}: {p} on the rim is open"
            );
        }
        assert!(
            !level.stairs.iter().any(|s| room.contains(s.pos)),
            "seed {seed}: no stairs inside"
        );
        assert!(
            !level.creatures.iter().any(|c| room.contains(c.pos)),
            "seed {seed}: no one inside"
        );
    });
    assert!(found >= 15, "{found} treasure rooms");
}

#[test]
fn a_cave_s_treasure_room_is_a_box_with_one_secret_door() {
    let content = content(true);
    let floor = content.terrain_id("floor").expect("floor");
    let found = rooms_of(&content, "depths:3", 0..30, |seed, level, room| {
        let rim = rim(room);
        let secrets: Vec<_> = (rim.iter())
            .filter(|p| level.secrets.contains_key(p))
            .collect();
        assert_eq!(secrets.len(), 1, "seed {seed}: one secret door");
        for p in room.cells().into_iter().filter(|p| !rim.contains(p)) {
            let t = level.terrain[p];
            assert!(
                t == floor || content.terrain(t).is_shut_chest(),
                "seed {seed}: {p} inside is floor or a chest"
            );
        }
        for p in rim.iter().filter(|p| !level.secrets.contains_key(p)) {
            assert!(
                !content.terrain(level.terrain[*p]).walkable,
                "seed {seed}: {p} on the rim is walled"
            );
        }
    });
    assert!(found >= 5, "{found} treasure rooms");
}

#[test]
fn treasure_rooms_hold_chests_gold_and_gems_from_their_own_stream() {
    let (with, without) = (content(true), content(false));
    let gold = with.object_id("gold_coin").expect("gold");
    let gems = with.gems().expect("gems").object;
    let (mut rooms, mut chests_seen) = (0, [0u32; 3]);
    for seed in 0..200 {
        let level = generate(&with, ("showcase", seed), "depths:2", None).expect("level");
        let plain = generate(&without, ("showcase", seed), "depths:2", None).expect("level");
        let effect = with.room_effects().id("treasury").expect("treasury");
        let Some(room) = level.rooms.iter().find(|r| r.effect == effect) else {
            continue;
        };
        rooms += 1;
        let cells = room.cells();
        assert_eq!(
            outside(&level.chests, room),
            outside(&plain.chests, room),
            "seed {seed}: chests outside"
        );
        assert_eq!(
            outside(&level.objects, room),
            outside(&plain.objects, room),
            "seed {seed}: objects outside"
        );
        let chests = cells
            .iter()
            .filter(|p| level.chests.contains_key(p))
            .count()
            - cells
                .iter()
                .filter(|p| plain.chests.contains_key(p))
                .count();
        assert!((1..=2).contains(&chests), "seed {seed}: {chests} chests");
        chests_seen[chests] += 1;
        let coins = count_in(&level, &cells, gold) - count_in(&plain, &cells, gold);
        assert!((12..=180).contains(&coins), "seed {seed}: {coins} gold");
        let stones = count_in(&level, &cells, gems) - count_in(&plain, &cells, gems);
        assert!((1..=3).contains(&stones), "seed {seed}: {stones} gems");
        for p in cells.iter().filter(|p| level.chests.contains_key(p)) {
            assert!(level.terrain[*p] != plain.terrain[*p] || plain.chests.contains_key(p));
        }
    }
    assert!(rooms >= 15, "{rooms} treasure rooms");
    assert!(chests_seen[1] > 0 && chests_seen[2] > 0, "{chests_seen:?}");
}
