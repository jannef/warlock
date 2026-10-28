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
        assert_eq!(connect::validate(&level, content), Ok(()), "seed {seed}");
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
        // A treasure room's chest may be a mimic (the band's chance, design §5.3).
        assert!(
            !(level.creatures.iter())
                .any(|c| room.contains(c.pos) && !content.is_mimic(c.creature)),
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
        // A mimic stands in for a chest (design §5.3): it counts as the chest it replaced.
        let mimics = (level.creatures.iter())
            .filter(|c| room.contains(c.pos) && with.is_mimic(c.creature))
            .count();
        let chests = cells
            .iter()
            .filter(|p| level.chests.contains_key(p))
            .count()
            + mimics
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

/// The built-in content with every generated level's one special room drawn, a greater treasure
/// room most often.
fn great() -> Content {
    let mut files = ContentFiles::builtin();
    files.edit_table("room_effects", |text| {
        text.replace("chance = 12\n", "chance = 100\n").replace(
            "weight = 2\nmin_cells = 9\nsealed",
            "weight = 100\nmin_cells = 9\nsealed",
        )
    });
    Content::load(&files).expect("fixture content")
}

/// The cells reached from `level`'s up staircase through walkable cells, closed doors, and
/// secret doors (4-connected), as `connect::validate` reaches them.
fn reached(level: &LevelTemplate, content: &Content) -> Grid<bool> {
    let up = level.player_start.expect("a start");
    let passable = |p: Pos| {
        level.secrets.contains_key(&p)
            || level.terrain.get(p).is_some_and(|t| {
                let def = content.terrain(*t);
                def.walkable || def.is_closed_door()
            })
    };
    connect::flood(level.terrain.width(), level.terrain.height(), up, passable)
}

/// Greater treasure rooms of `id` over `seeds`, each checked by `check`; returns how many.
fn great_rooms(
    content: &Content,
    id: &str,
    seeds: std::ops::Range<u64>,
    mut check: impl FnMut(u64, &LevelTemplate, &SpecialRoom),
) -> usize {
    let effect = content
        .room_effects()
        .id("great_treasury")
        .expect("greater");
    let mut found = 0;
    for seed in seeds {
        let level = generate(content, ("showcase", seed), id, None).expect("level");
        assert_eq!(connect::validate(&level, content), Ok(()), "seed {seed}");
        for room in level.rooms.iter().filter(|r| r.effect == effect) {
            found += 1;
            check(seed, &level, room);
        }
    }
    found
}

/// Checks a greater treasure room: no door or way in on its rim, its breach a stone wall beside a
/// cell reached from the up staircase, nothing inside reached until the breach is broken, and
/// no stairs, creature (but a mimic in a chest's place), or trap inside.
fn walled_up(content: &Content, seed: u64, level: &LevelTemplate, room: &SpecialRoom) {
    let wall = content.terrain_id("wall").expect("wall");
    let floor = content.terrain_id("floor").expect("floor");
    let breach = room.breach.expect("a breach");
    let rim = rim(room);
    assert!(rim.contains(&breach), "seed {seed}: the breach on the rim");
    assert_eq!(
        level.terrain[breach], wall,
        "seed {seed}: the breach is wall"
    );
    for p in &rim {
        let def = content.terrain(level.terrain[*p]);
        assert!(!def.walkable && !def.is_door(), "seed {seed}: {p} open");
        assert!(!level.secrets.contains_key(p), "seed {seed}: {p} secret");
    }
    let reached = reached(level, content);
    for p in room.cells() {
        assert!(!reached[p], "seed {seed}: {p} reached without digging");
    }
    let beside = [Dir::N, Dir::E, Dir::S, Dir::W].map(|d| breach.step(d));
    assert!(
        beside.iter().any(|&p| !room.contains(p) && reached[p]),
        "seed {seed}: the breach beside a reached cell"
    );
    let mut dug = level.clone();
    dug.terrain.set(breach, floor);
    let reached = self::reached(&dug, content);
    for p in room.cells().into_iter().filter(|p| !rim.contains(p)) {
        let def = content.terrain(level.terrain[p]);
        assert!(
            reached[p] || !def.walkable,
            "seed {seed}: {p} reached once dug"
        );
    }
    assert!(!level.stairs.iter().any(|s| room.contains(s.pos)));
    let creature =
        (level.creatures.iter()).any(|c| room.contains(c.pos) && !content.is_mimic(c.creature));
    assert!(!creature, "seed {seed}: no creature but a mimic inside");
    assert!(
        !level
            .traps
            .keys()
            .any(|p| room.contains(*p) && !level.chests.contains_key(p))
    );
}

#[test]
fn a_rooms_level_s_greater_treasure_room_is_walled_up_with_its_breach() {
    let content = great();
    let found = great_rooms(&content, "depths:2", 0..60, |seed, level, room| {
        walled_up(&content, seed, level, room);
    });
    assert!(found >= 10, "{found} greater treasure rooms");
}

#[test]
fn a_cave_s_greater_treasure_room_is_walled_up_with_its_breach() {
    let content = great();
    let found = great_rooms(&content, "depths:3", 0..30, |seed, level, room| {
        walled_up(&content, seed, level, room);
    });
    assert!(found >= 5, "{found} greater treasure rooms");
}

#[test]
fn a_greater_treasure_room_holds_more() {
    let content = great();
    let gold = content.object_id("gold_coin").expect("gold");
    let gems = content.gems().expect("gems").object;
    let found = great_rooms(&content, "depths:2", 0..40, |seed, level, room| {
        let cells = room.cells();
        let chests = cells
            .iter()
            .filter(|p| level.chests.contains_key(p))
            .count();
        assert!((2..=4).contains(&chests), "seed {seed}: {chests} chests");
        let coins = count_in(level, &cells, gold);
        assert!((80..=1200).contains(&coins), "seed {seed}: {coins} gold");
        let stones = count_in(level, &cells, gems);
        assert!((2..=6).contains(&stones), "seed {seed}: {stones} gems");
    });
    assert!(found >= 5, "{found} greater treasure rooms");
}

#[test]
fn validate_refuses_a_sealed_room_without_a_working_breach() {
    let content = great();
    let mut tried = 0;
    great_rooms(&content, "depths:2", 0..20, |seed, level, room| {
        let i = level.rooms.iter().position(|r| r == room).expect("room");
        let mut none = level.clone();
        none.rooms[i].breach = None;
        assert_eq!(
            connect::validate(&none, &content),
            Err(connect::Invalid::NoBreach(room.min)),
            "seed {seed}"
        );
        // A corner of the ring touches no floor of the room.
        let mut corner = level.clone();
        corner.rooms[i].breach = Some(room.min);
        assert_eq!(
            connect::validate(&corner, &content),
            Err(connect::Invalid::BadBreach(room.min)),
            "seed {seed}"
        );
        tried += 1;
    });
    assert!(tried > 0);
}

#[test]
fn a_box_fits_only_in_rock_beside_a_reached_cell() {
    let content = great();
    let pal = Palette::resolve(&content).expect("palette");
    let odds = content
        .dungeon("depths")
        .and_then(|d| d.room_odds(1))
        .expect("odds");
    let effect = content
        .room_effects()
        .id("great_treasury")
        .expect("greater");
    let def = content.room_effect(effect);
    let mut canvas = Canvas::new(&content, pal, (odds, None), Vec::new(), (30, 20));
    // All rock and no stairs: nothing is reached.
    assert!(!treasury::fits_box(&canvas, def));
    // A corridor along row 10 from the up staircase.
    for x in 2..28 {
        canvas.set(Pos::new(x, 10), pal.floor);
    }
    canvas.up = Some(Pos::new(2, 10));
    assert!(treasury::fits_box(&canvas, def));
    // The same corridor, not reached from the stairs: no breach beside it.
    canvas.up = Some(Pos::new(1, 1));
    canvas.set(Pos::new(1, 1), pal.floor);
    assert!(!treasury::fits_box(&canvas, def));
    // All floor: no rock for the box.
    let mut open = Canvas::new(&content, pal, (odds, None), Vec::new(), (30, 20));
    for p in open.cells() {
        open.set(p, pal.floor);
    }
    open.up = Some(Pos::new(2, 2));
    assert!(!treasury::fits_box(&open, def));
}
