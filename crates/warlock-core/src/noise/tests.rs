use super::*;
use crate::content::{ContentFiles, CreatureId};
use crate::world::ActorKind;

const TERRAIN: &str = r##"
[[terrain]]
id = "void"
name = "rock"
glyph = " "
fg = "#000000"
walkable = false
transparent = false

[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "water"
name = "water"
glyph = "~"
fg = "#4060c0"
step_noise = 200

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"
toggle_noise = 60
muffle = 30

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#a06030"
closes_to = "door_closed"
toggle_noise = 70
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
footsteps = 50
hearing = 10

[[creature]]
id = "mouse"
name = "mouse"
glyph = "m"
fg = "#806040"
weight = 1000
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"~" = "water"
"+" = "door_closed"
"'" = "door_open"
"##;

/// Content with one level `t` drawn by `map` (rows of the legend above; it needs no `@`).
fn content(map: &str) -> Content {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    files.insert(
        "levels/t.toml",
        &format!("name = \"T\"\nmap = '''\n{map}'''\n"),
    );
    Content::load(&files).expect("test content is valid")
}

fn level(content: &Content) -> LevelState {
    LevelState::from_template(content.level("t").expect("level t"))
}

/// Loudness at `to` of a noise of `loudness` made at `from` on `map`.
fn heard_at(map: &str, from: (i32, i32), loudness: u16, to: (i32, i32)) -> u16 {
    let content = content(map);
    let level = level(&content);
    let field = propagate(&level, &content, Pos::new(from.0, from.1), loudness, 1);
    field.at(Pos::new(to.0, to.1))
}

fn actor(content: &Content, creature: &str) -> Actor {
    Actor {
        id: ActorId(1),
        kind: ActorKind::Creature,
        creature: content.creature_id(creature).expect("creature"),
        level: "t".into(),
        pos: Pos::new(1, 1),
        patrol: None,
        energy: 0,
        heard: Vec::new(),
        inventory: Vec::new(),
        equipment: Default::default(),
        mind: Default::default(),
        stats: Default::default(),
        hp: 10,
        hp_base: 0,
        regen: 0,
        skills: Default::default(),
        effects: Default::default(),
        weapon_skills: Default::default(),
        faction_override: None,
        satiation: None,
        stance: Default::default(),
    }
}

const ROOM: &str = "#########\n#.......#\n#.......#\n#########\n";

#[test]
fn loudness_falls_with_path_distance() {
    let at = |x, y| heard_at(ROOM, (1, 1), 50, (x, y));
    assert_eq!(at(1, 1), 50);
    assert_eq!(at(2, 1), 40);
    assert_eq!(at(2, 2), 36, "a diagonal step loses 14");
    assert_eq!(at(3, 2), 26);
    assert_eq!(at(5, 1), 10);
    assert_eq!(at(6, 1), 0, "out of earshot");
    assert_eq!(at(0, 1), 0, "walls take no sound");
}

#[test]
fn sound_goes_around_corners_but_not_through_walls() {
    // A wall between a and b: the sound walks around it (two diagonals and two steps, 48).
    let map = "#######\n#.#...#\n#.#.#.#\n#.....#\n#######\n";
    assert_eq!(heard_at(map, (1, 1), 60, (3, 1)), 12);
    // Sealed off: nothing at all.
    let sealed = "#######\n#.#...#\n#.#####\n#.....#\n#######\n";
    assert_eq!(heard_at(sealed, (1, 1), 200, (3, 1)), 0);
}

#[test]
fn closed_doors_muffle_and_open_doors_do_not() {
    let closed = "#######\n#..+..#\n#######\n";
    let open = "#######\n#..'..#\n#######\n";
    assert_eq!(
        heard_at(closed, (1, 1), 80, (5, 1)),
        10,
        "4 steps and 30 for the door"
    );
    assert_eq!(heard_at(open, (1, 1), 80, (5, 1)), 40);
    // A door's own noise is made on it, so it carries both ways unmuffled.
    assert_eq!(heard_at(closed, (3, 1), 60, (1, 1)), 40);
    assert_eq!(heard_at(closed, (3, 1), 60, (5, 1)), 40);
}

#[test]
fn sound_never_squeezes_past_a_closed_door_corner() {
    // From (1,1) to (2,2) the diagonal runs between two closed doors: it must go through one.
    let map = "####\n#.+#\n#+.#\n####\n";
    assert_eq!(heard_at(map, (1, 1), 60, (2, 2)), 10);
    let open = "####\n#.'#\n#'.#\n####\n";
    assert_eq!(heard_at(open, (1, 1), 60, (2, 2)), 46);
}

#[test]
fn the_search_stops_at_the_floor() {
    let content = content(ROOM);
    let level = level(&content);
    let field = propagate(&level, &content, Pos::new(1, 1), 50, 25);
    assert_eq!(field.at(Pos::new(3, 1)), 30);
    assert_eq!(field.at(Pos::new(4, 1)), 0, "20 is below the floor");
}

#[test]
fn an_action_s_noise_comes_from_the_actor_and_the_terrain() {
    let content = content("#######\n#.~+'.#\n#######\n");
    let level = level(&content);
    let player = actor(&content, "player");
    let unburdened = Burden::Unburdened;
    let step = |x| {
        noise(
            &content,
            &level,
            &player,
            unburdened,
            Act::Step(Pos::new(x, 1)),
        )
    };
    let footsteps = step(1).expect("footsteps");
    assert_eq!(
        (footsteps.kind, footsteps.loudness, footsteps.maker),
        (SoundKind::Footsteps, 50, ActorId(1))
    );
    assert_eq!(step(2).map(|n| n.loudness), Some(100), "water splashes");
    // A load makes steps louder (spec §8 "Burden"): ×1.25 burdened, ×1.5 strained; doors and
    // attacks sound the same.
    let loaded = |burden, x| {
        noise(&content, &level, &player, burden, Act::Step(Pos::new(x, 1))).map(|n| n.loudness)
    };
    assert_eq!(
        loaded(Burden::Burdened, 1),
        Some(62),
        "50 × 1.25, rounded down"
    );
    assert_eq!(loaded(Burden::Strained, 1), Some(75));
    assert_eq!(
        loaded(Burden::Burdened, 2),
        Some(125),
        "water: 200 % of 62.5"
    );
    assert_eq!(loaded(Burden::Strained, 2), Some(150));
    let door = |act| {
        let strained = noise(&content, &level, &player, Burden::Strained, act);
        assert_eq!(strained, noise(&content, &level, &player, unburdened, act));
        strained.map(|n| (n.kind, n.loudness))
    };
    assert_eq!(
        door(Act::OpenDoor(Pos::new(3, 1))),
        Some((SoundKind::DoorOpened, 60))
    );
    assert_eq!(
        door(Act::CloseDoor(Pos::new(4, 1))),
        Some((SoundKind::DoorClosed, 70))
    );
    // A creature without footsteps moves silently.
    let mouse = actor(&content, "mouse");
    assert_eq!(
        noise(
            &content,
            &level,
            &mouse,
            Burden::Strained,
            Act::Step(Pos::new(1, 1))
        ),
        None
    );
}

#[test]
fn noise_fields_are_validated() {
    let with = |terrain: &str, creatures: &str| {
        let mut files = ContentFiles::default();
        files.insert("data/terrain.toml", terrain);
        files.insert("data/creatures.toml", creatures);
        files.insert("data/legend.toml", LEGEND);
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    assert_eq!(with(TERRAIN, CREATURES), Ok(()));
    let err = with(TERRAIN, &CREATURES.replace("hearing = 10", "hearing = 0")).unwrap_err();
    assert!(err.contains("`hearing` 1 to 300"), "{err}");
    let err = with(
        TERRAIN,
        &CREATURES.replace("footsteps = 50", "footsteps = 301"),
    )
    .unwrap_err();
    assert!(err.contains("`footsteps` must be at most 300"), "{err}");
    let err = with(
        &TERRAIN.replace("step_noise = 200", "step_noise = 1001"),
        CREATURES,
    )
    .unwrap_err();
    assert!(err.contains("terrain `water`: `step_noise`"), "{err}");
    let err = with(
        &TERRAIN.replace("step_noise = 200", "toggle_noise = 5"),
        CREATURES,
    )
    .unwrap_err();
    assert!(err.contains("only doors have a `toggle_noise`"), "{err}");
}

#[test]
fn solid_terrain_blocks_sound_unless_it_muffles() {
    let content = content(ROOM);
    let muffle = |id: &str| {
        content
            .terrain(content.terrain_id(id).expect("terrain"))
            .muffle
    };
    assert_eq!(muffle("wall"), None);
    assert_eq!(muffle("void"), None);
    assert_eq!(muffle("door_closed"), Some(30));
    assert_eq!(muffle("door_open"), Some(0));
    assert_eq!(muffle("water"), Some(0));
}

#[test]
fn rough_places_are_near_the_source_and_never_where_the_player_sees() {
    let content = content("###########\n#.........#\n#.........#\n#.........#\n###########\n");
    let level = level(&content);
    let source = Pos::new(5, 2);
    let seen = Pos::new(4, 2);
    let hidden = |p: Pos| p != seen;
    let place = |volume, turn| rough_place(&level, &content, source, volume, turn, hidden);
    for turn in 0..50 {
        assert_eq!(place(Volume::Loud, turn), source);
        let clear = place(Volume::Clear, turn);
        assert!(clear.chebyshev(source) <= 1 && clear != seen, "{clear}");
        let faint = place(Volume::Faint, turn);
        assert!(faint.chebyshev(source) <= 2 && faint != seen, "{faint}");
        assert_eq!(place(Volume::Faint, turn), faint, "deterministic");
    }
    // Faint places vary from turn to turn, and are not always the source.
    let places: std::collections::BTreeSet<Pos> =
        (0..50).map(|turn| place(Volume::Faint, turn)).collect();
    assert!(places.len() > 5, "{places:?}");
    // Never in a wall, nor through a door.
    let content = self::content("#######\n#..+..#\n#######\n");
    let level = self::level(&content);
    for turn in 0..50 {
        let p = rough_place(
            &level,
            &content,
            Pos::new(2, 1),
            Volume::Faint,
            turn,
            |_| true,
        );
        assert!(p.y == 1 && (1..=2).contains(&p.x), "{p}");
    }
}

#[test]
fn volume_follows_the_margin_over_the_threshold() {
    assert_eq!(Volume::of(10, 10), Volume::Faint);
    assert_eq!(Volume::of(19, 10), Volume::Faint);
    assert_eq!(Volume::of(20, 10), Volume::Clear);
    assert_eq!(Volume::of(40, 10), Volume::Loud);
}

#[test]
fn a_sound_moving_along_is_news_once() {
    let sound = |x, kind| PerceivedSound {
        kind,
        pos: Pos::new(x, 0),
        volume: Volume::Clear,
    };
    let mut recent = Vec::new();
    // Footsteps along a corridor, a cell a turn and placed up to 2 off either way.
    let mut news = 0;
    for turn in 0..20u64 {
        let jitter = if turn % 2 == 0 { 2 } else { -2 };
        let heard = [sound(turn as i32 + jitter, SoundKind::Footsteps)];
        news += super::news(&mut recent, &heard, turn).len();
    }
    assert_eq!(news, 1, "a patrol heard turn after turn is news once");
    // A door far from the footsteps is news; the same footsteps after a long silence again.
    let door = [sound(19, SoundKind::DoorOpened)];
    assert_eq!(super::news(&mut recent, &door, 20).len(), 1);
    let back = [sound(25, SoundKind::Footsteps)];
    assert_eq!(
        super::news(&mut recent, &back, 20 + RECALL_TURNS + 1).len(),
        1
    );
    assert!(recent.len() <= 2, "old sounds are forgotten: {recent:?}");
    // Two places far apart on one turn are two pieces of news.
    let mut recent = Vec::new();
    let two = [
        sound(0, SoundKind::Footsteps),
        sound(20, SoundKind::Footsteps),
    ];
    assert_eq!(super::news(&mut recent, &two, 0).len(), 2);
    assert!(
        super::news(&mut recent, &two, 0).is_empty(),
        "same turn again"
    );
}

#[test]
fn messages_name_the_rough_direction() {
    let sound = |x, y, volume| PerceivedSound {
        kind: SoundKind::Footsteps,
        pos: Pos::new(x, y),
        volume,
    };
    let viewer = Pos::new(10, 10);
    assert_eq!(
        sound(14, 6, Volume::Clear).message(viewer),
        "You hear footsteps to the north-east."
    );
    assert_eq!(
        sound(10, 15, Volume::Faint).message(viewer),
        "You faintly hear footsteps to the south."
    );
    let door = PerceivedSound {
        kind: SoundKind::DoorClosed,
        ..sound(3, 10, Volume::Loud)
    };
    assert_eq!(door.message(viewer), "You hear a door close to the west.");
}

mod props {
    use super::*;
    use proptest::prelude::*;

    /// A 10×10 level: `#` where `walls` says so, `+` where `doors` does, else floor; walled in.
    fn map(walls: &[bool], doors: &[bool]) -> String {
        let mut rows = vec!["#".repeat(12)];
        for y in 0..10 {
            let mut row = String::from("#");
            for x in 0..10 {
                let i = y * 10 + x;
                row.push(match (walls[i], doors[i]) {
                    (true, _) => '#',
                    (false, true) => '+',
                    _ => '.',
                });
            }
            row.push('#');
            rows.push(row);
        }
        rows.push("#".repeat(12));
        rows.join("\n") + "\n"
    }

    fn cells() -> impl Strategy<Value = Vec<bool>> {
        prop::collection::vec(prop::bool::weighted(0.25), 100)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn hearing_is_symmetric_between_open_cells(
            walls in cells(), doors in cells(),
            ax in 1i32..11, ay in 1i32..11, bx in 1i32..11, by in 1i32..11,
            loudness in 1u16..200,
        ) {
            let map = map(&walls, &doors);
            let content = content(&map);
            let level = level(&content);
            let (a, b) = (Pos::new(ax, ay), Pos::new(bx, by));
            prop_assume!(muffle(&level, &content, a) == Some(0));
            prop_assume!(muffle(&level, &content, b) == Some(0));
            let ab = propagate(&level, &content, a, loudness, 1).at(b);
            let ba = propagate(&level, &content, b, loudness, 1).at(a);
            prop_assert_eq!(ab, ba);
        }

        #[test]
        fn sound_never_leaves_a_walled_ring(
            walls in cells(), ox in 3i32..9, oy in 3i32..9, loudness in 1u16..300,
        ) {
            let mut walls = walls;
            let origin = Pos::new(ox, oy);
            for y in 0..10 {
                for x in 0..10 {
                    let d = Pos::new(x + 1, y + 1).chebyshev(origin);
                    let i = (y * 10 + x) as usize;
                    if d == 2 { walls[i] = true; }
                    if d < 2 { walls[i] = false; }
                }
            }
            let map = map(&walls, &[false; 100]);
            let content = content(&map);
            let level = level(&content);
            let field = propagate(&level, &content, origin, loudness, 1);
            for y in 0..12 {
                for x in 0..12 {
                    let p = Pos::new(x, y);
                    if p.chebyshev(origin) > 1 {
                        prop_assert_eq!(field.at(p), 0, "heard at {} through the ring", p);
                    }
                }
            }
        }
    }
}

#[test]
fn creature_ids_in_test_content() {
    let content = content(ROOM);
    assert_eq!(content.creature_id("mouse"), Some(CreatureId(1)));
    assert_eq!(content.creature(CreatureId(1)).hearing, None, "deaf");
}

#[test]
fn a_quick_walker_s_steps_in_one_turn_are_one_sound() {
    let content = content(ROOM);
    let level = level(&content);
    let step = |x, y| Heard {
        kind: SoundKind::Footsteps,
        at: Pos::new(x, y),
        loudness: 40,
    };
    // Two steps a cell apart, then a door and footsteps far off.
    let heard = [
        step(2, 1),
        step(3, 1),
        Heard {
            kind: SoundKind::DoorOpened,
            ..step(3, 2)
        },
        step(7, 2),
    ];
    let sounds = perceive(&level, &content, &heard, 10, 5, |_| true);
    let kinds: Vec<_> = sounds.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        [
            SoundKind::Footsteps,
            SoundKind::DoorOpened,
            SoundKind::Footsteps
        ]
    );
    assert_eq!(sounds[0].pos, Pos::new(3, 1), "the later step, loud: exact");
    // Sources the player sees are not perceived at all.
    let unseen = perceive(&level, &content, &heard, 10, 5, |p| p.x > 5);
    assert_eq!(unseen.len(), 1);
}
