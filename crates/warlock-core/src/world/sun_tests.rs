//! Sunlight and sky light through the whole turn pipeline (spec §5 "Sunlight").

use super::light_tests::{check_invariants, check_wall_completion};
use super::*;
use crate::content::ContentFiles;
use crate::light::{Light, Sun};
use crate::perception::{CellView, Emitter, PerceptionView, SeenLight};
use crate::replay::Replay;
use proptest::prelude::*;
use std::collections::BTreeSet;

const TERRAIN: &str = r##"
[[terrain]]
id = "void"
name = "rock"
glyph = " "
fg = "#000000"
walkable = false
transparent = false

[[terrain]]
id = "grass"
name = "grass"
glyph = "."
fg = "#80a060"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "tree"
name = "tree"
glyph = "T"
fg = "#40a040"
walkable = false
transparent = false
height = 2

# Transparent casters, so a viewer sees the shadows on every side of them.
[[terrain]]
id = "bush"
name = "bush"
glyph = "b"
fg = "#40a040"
walkable = false
height = 2

[[terrain]]
id = "stump"
name = "stump"
glyph = "s"
fg = "#806040"
walkable = false
height = 1

[[terrain]]
id = "porch"
name = "porch"
glyph = "="
fg = "#a08060"
roofed = true

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#a06030"
closes_to = "door_closed"
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"

[[creature]]
id = "rat"
name = "rat"
glyph = "r"
fg = "#806040"
"##;

const LIGHTS: &str = r##"
[[light]]
id = "lantern"
color = [255, 255, 255, 0]
radius = 3
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40

[[vision]]
id = "low_light"
name = "Low-light"
threshold = 12
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "grass"
"#" = "wall"
"T" = "tree"
"b" = "bush"
"s" = "stump"
"=" = "porch"
"+" = "door_closed"
"'" = "door_open"
"@" = { terrain = "grass", spawn = "player" }
"P" = { terrain = "porch", spawn = "player" }
"r" = { terrain = "grass", creature = "rat" }
"##;

/// A world on one level of `kind` with `map`, starting at `time` (`hh:mm`) with normal eyes and an
/// unlit lantern.
fn world(map: &str, kind: &str, time: &str) -> World {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/lights.toml", LIGHTS);
    files.insert("data/vision.toml", VISION);
    files.insert("data/legend.toml", LEGEND);
    files.insert(
        "levels/test.toml",
        &format!("name = \"Test\"\nkind = \"{kind}\"\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/test.toml",
        &format!(
            "name = \"T\"\nstart_level = \"test\"\nlantern = \"lantern\"\nstart_time = \"{time}\"\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test", None).expect("world")
}

fn outdoors(map: &str, time: &str) -> World {
    world(map, "outdoor", time)
}

/// Outdoors with low-light eyes, which see shade by sky light at any hour of daylight.
fn outdoors_low_light(map: &str, time: &str) -> World {
    let mut w = outdoors(map, time);
    w.apply(&Action::SetVision("low_light".into()));
    w
}

/// An open field (no walls, so nothing but its casters shades it) `width × height` with `extra`
/// cells placed on it.
fn field(width: usize, height: usize, extra: &[(usize, usize, char)]) -> String {
    let mut rows = vec![vec!['.'; width]; height];
    for &(x, y, c) in extra {
        rows[y][x] = c;
    }
    rows.into_iter()
        .map(|r| r.into_iter().collect::<String>() + "\n")
        .collect()
}

fn sun_source<'a>(view: &PerceptionView<'a>) -> Option<&'a SeenLight> {
    view.light_sources()
        .iter()
        .find(|s| matches!(s.emitter, Emitter::Sun(_)))
}

/// Visible cells the sun lights.
fn sunlit(view: &PerceptionView<'_>) -> BTreeSet<Pos> {
    sun_source(view)
        .map(|s| s.contributions.iter().map(|(p, _)| *p).collect())
        .unwrap_or_default()
}

/// Visible transparent cells the sun does not light.
fn shaded(view: &PerceptionView<'_>) -> BTreeSet<Pos> {
    let lit = sunlit(view);
    let content = view.content();
    (0..view.height())
        .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
        .filter(|p| matches!(view.cell(*p), CellView::Visible(t) if content.terrain(t).transparent))
        .filter(|p| !lit.contains(p))
        .collect()
}

fn skip(world: &mut World, minutes: u64) {
    world.apply(&Action::SkipTime(Duration::minutes(minutes)));
}

#[test]
fn the_sun_and_the_sky_light_open_ground_by_day() {
    let w = outdoors(&field(21, 9, &[(10, 7, '@')]), "12:00");
    let view = w.perception();
    let sun = view.sun().expect("under the open sky");
    assert!(sun.is_up());
    let source = sun_source(&view).expect("the sun lights cells in view");
    assert_eq!(source.emitter, Emitter::Sun(sun));
    assert_eq!(source.pos(), None);
    assert_eq!(source.preset(), None);
    // Every cell of the field is in the sun and sees the sky.
    for y in 0..9 {
        for x in 0..21 {
            let p = Pos::new(x, y);
            let light = view.light(p).expect("all of the field is in view").light();
            let base = view.base_light(p).expect("visible");
            assert!(base.max(sun.sky) == base, "sky in the base light at {p}");
            assert_eq!(light, base.saturating_add(sun.light), "{p}");
        }
    }
    assert_eq!(source.contributions.len(), 21 * 9);
    check_invariants(&w).expect("light invariants");
}

#[test]
fn shadows_point_away_from_the_sun_all_day() {
    // A transparent bush in an open field: every visible cell is in the sun except exactly the
    // bush's shadow, which turns from west through north to east.
    let mut w = outdoors_low_light(&field(41, 21, &[(20, 10, 'b'), (20, 17, '@')]), "06:00");
    let bush = Pos::new(20, 10);
    for _ in 0..72 {
        skip(&mut w, 10);
        let view = w.perception();
        let sun = view.sun().expect("outdoors");
        let expected: BTreeSet<Pos> = (sun.shadow_steps(2))
            .map(|(dx, dy)| Pos::new(bush.x + dx, bush.y + dy))
            .filter(|p| (0..41).contains(&p.x) && (0..21).contains(&p.y))
            .collect();
        if sun.is_up() {
            assert_eq!(shaded(&view), expected, "{}", w.time());
        } else {
            assert!(
                sunlit(&view).is_empty() && expected.is_empty(),
                "{}",
                w.time()
            );
        }
        check_invariants(&w).expect("light invariants");
    }
    let at = |time: &str| {
        let w = outdoors_low_light(&field(41, 21, &[(20, 10, 'b'), (20, 17, '@')]), time);
        let cells: Vec<(i32, i32)> = (shaded(&w.perception()).into_iter())
            .map(|p| (p.x - bush.x, p.y - bush.y))
            .collect();
        cells
    };
    let morning = at("08:00");
    assert!(
        morning.iter().all(|&(dx, dy)| dx < 0 && dy <= 0),
        "{morning:?}"
    );
    assert_eq!(at("12:00"), [(0, -1)], "due north at noon");
    let afternoon = at("16:00");
    assert!(
        afternoon.iter().all(|&(dx, dy)| dx > 0 && dy <= 0),
        "{afternoon:?}"
    );
}

#[test]
fn shadows_are_long_when_the_sun_is_low_and_grow_with_height() {
    // A bush (2 tall) and a stump (1 tall) far apart.
    let map = field(41, 21, &[(14, 5, 'b'), (14, 15, 's'), (30, 10, '@')]);
    let lengths = |time: &str| {
        let w = outdoors_low_light(&map, time);
        let cells = shaded(&w.perception());
        let bush = cells.iter().filter(|p| p.y <= 10).count();
        (bush, cells.len() - bush)
    };
    let dawn = lengths("06:40");
    let morning = lengths("09:00");
    let noon = lengths("12:00");
    assert!(
        dawn.0 > morning.0 && morning.0 > noon.0,
        "{dawn:?} {morning:?} {noon:?}"
    );
    assert!(
        dawn.1 > morning.1 && morning.1 >= noon.1,
        "{dawn:?} {morning:?} {noon:?}"
    );
    assert!(
        dawn.0 >= 2 * dawn.1 - 1,
        "twice as tall, twice as long: {dawn:?}"
    );
    assert_eq!(noon, (1, 1));
    let dusk = lengths("17:20");
    assert_eq!(dusk, dawn, "afternoon mirrors the morning");
}

#[test]
fn night_is_dark_and_twilight_has_only_the_sky() {
    let map = field(21, 9, &[(10, 4, '@')]);
    let night = outdoors(&map, "22:00");
    let view = night.perception();
    assert!(view.sun().is_some_and(|s| s.is_night()));
    assert!(view.light_sources().is_empty());
    assert_eq!(
        view.light(Pos::new(10, 4)).map(|l| l.light()),
        Some(Light::DARK)
    );
    assert!(!view.is_visible(Pos::new(11, 4)), "moonlight is darkness");
    check_invariants(&night).expect("light invariants");

    // Twilight: no sun, a dim sky that low-light eyes see by.
    let mut dusk = outdoors(&map, "18:10");
    dusk.apply(&Action::SetVision("low_light".into()));
    let view = dusk.perception();
    let sun = view.sun().expect("outdoors");
    assert!(!sun.is_up() && !sun.is_night());
    assert!(view.light_sources().is_empty());
    for p in [Pos::new(10, 4), Pos::new(2, 1), Pos::new(19, 7)] {
        assert_eq!(view.base_light(p), Some(sun.sky), "{p}");
    }
    check_invariants(&dusk).expect("light invariants");
}

#[test]
fn roofed_cells_get_neither_sun_nor_sky() {
    // A porch 13 cells square in a field, the player in its middle with a lit lantern.
    let mut cells: Vec<(usize, usize, char)> = Vec::new();
    for y in 4..17 {
        for x in 9..22 {
            cells.push((x, y, if (x, y) == (15, 10) { 'P' } else { '=' }));
        }
    }
    let map = field(31, 21, &cells);
    let mut outside = outdoors(&map, "12:00");
    let mut cellar = world(&map, "dungeon", "12:00");
    outside.apply(&Action::ToggleLantern);
    cellar.apply(&Action::ToggleLantern);
    let (view, dark) = (outside.perception(), cellar.perception());
    let porch = |p: Pos| (9..22).contains(&p.x) && (4..17).contains(&p.y);
    let lit = sunlit(&view);
    assert!(!lit.is_empty(), "the sun lights the field around the porch");
    assert!(lit.iter().all(|p| !porch(*p)), "never under the roof");
    // Deep under the roof (beyond bounce reach of sunlit ground) the light is exactly what it
    // would be underground: the lantern's, no sky.
    for y in 7..14 {
        for x in 12..19 {
            let p = Pos::new(x, y);
            assert_eq!(view.light(p), dark.light(p), "{p}");
            assert_eq!(view.base_light(p), dark.base_light(p), "{p}");
        }
    }
    let sky = view.sun().expect("outdoors").sky;
    for p in lit {
        let base = view.base_light(p).expect("visible");
        assert_eq!(base.max(sky), base, "sky in the open at {p}");
    }
    check_invariants(&outside).expect("light invariants");
}

#[test]
fn dungeons_never_see_the_sun() {
    let map = field(21, 9, &[(10, 4, '@')]);
    let w = world(&map, "dungeon", "12:00");
    let view = w.perception();
    assert_eq!(view.sun(), None);
    assert!(view.light_sources().is_empty());
    assert!(!view.is_visible(Pos::new(11, 4)));
}

#[test]
fn sunlight_never_reaches_into_a_closed_hut() {
    let hut = "\
.................
.....#######.....
.....#=====#.....
.....#=====#.....
.....#==P==#.....
.....###+###.....
.................
";
    let mut w = outdoors(hut, "12:00");
    w.apply(&Action::ToggleLantern);
    let mut cellar = world(hut, "dungeon", "12:00");
    cellar.apply(&Action::ToggleLantern);
    let (view, dark) = (w.perception(), cellar.perception());
    assert!(
        sun_source(&view).is_none(),
        "not even on the inner wall faces"
    );
    for y in 0..7 {
        for x in 0..17 {
            let p = Pos::new(x, y);
            assert_eq!(view.light(p), dark.light(p), "{p}");
        }
    }
    // Open the door: the sun reaches the doorway and its light bounces in a little way.
    w.apply(&Action::Open(Dir::S));
    let view = w.perception();
    assert!(
        sunlit(&view).contains(&Pos::new(8, 5)),
        "the open doorway is in the sun"
    );
    assert!(sunlit(&view).iter().all(|p| p.y >= 5));
    check_invariants(&w).expect("light invariants");
}

#[test]
fn creatures_in_the_sun_are_seen_from_afar() {
    let map = field(41, 9, &[(5, 4, '@'), (30, 4, 'r')]);
    let seen = |time: &str| {
        let w = outdoors(&map, time);
        let view = w.perception();
        view.actors().iter().any(|a| a.kind == ActorKind::Creature)
    };
    assert!(seen("12:00"));
    assert!(seen("07:00"));
    assert!(!seen("23:00"));
}

#[test]
fn sunlight_is_deterministic_and_replays_verify() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let actions = crate::action::parse_actions(
        "move:E move:E skip:30m move:SE skip:5h move:E skip:6h rest:1h move:W skip:3h",
    )
    .expect("actions parse");
    let mut a = World::new(content.clone(), "courtyard", None).expect("courtyard");
    let mut b = World::new(content.clone(), "courtyard", None).expect("courtyard");
    let mut replay = Replay::new(&a);
    for action in &actions {
        a.apply(action);
        b.apply(action);
        replay.push(action, &a);
        assert_eq!(a.perception, b.perception);
        check_invariants(&a).expect("light invariants");
    }
    let replayed = replay.run(content).expect("the replay verifies");
    assert_eq!(replayed.state_hash(), a.state_hash());
    assert_eq!(replayed.perception, a.perception);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Random gardens (walls, trees, bushes, roofed porch) at any minute of the day: the light
    /// invariants hold, the sun never lights a roofed cell, and it lights every open cell the
    /// same.
    #[test]
    fn sun_invariants_hold_in_random_gardens(
        cells in prop::collection::vec(0u8..100, 22 * 9),
        minute in 0u32..1440,
        vision in prop::sample::select(vec!["normal", "low_light"]),
    ) {
        let mut map = String::new();
        for y in 0..9 {
            for x in 0..22 {
                map.push(match (x, y, cells[y * 22 + x]) {
                    (3, 4, _) => '@',
                    (_, _, 0..=14) => '#',
                    (_, _, 15..=24) => 'T',
                    (_, _, 25..=29) => 'b',
                    (_, _, 30..=44) => '=',
                    _ => '.',
                });
            }
            map.push('\n');
        }
        let mut w = outdoors(&map, "00:00");
        w.apply(&Action::SetVision(vision.into()));
        w.apply(&Action::SkipTime(Duration::minutes(u64::from(minute))));
        let view = w.perception();
        let sun = Sun::at_minute(minute);
        prop_assert_eq!(view.sun(), Some(sun));
        let content = view.content();
        if let Some(source) = sun_source(&view) {
            prop_assert!(sun.is_up());
            for &(p, light) in &source.contributions {
                let CellView::Visible(t) = view.cell(p) else {
                    return Err(TestCaseError::fail(format!("{p} not visible")));
                };
                prop_assert!(!content.terrain(t).roofed, "sun under a roof at {}", p);
                prop_assert_eq!(light, sun.light, "the sun is the same everywhere: {}", p);
            }
        }
        check_invariants(&w)?;
        check_wall_completion(&w)?;
    }

    /// A roofed, walled room (the player inside, no lantern) next to a random sunny garden: no
    /// sun, sky, or bounced sunlight gets in at any time of day.
    #[test]
    fn no_sunlight_is_seen_from_inside_a_closed_room(
        cells in prop::collection::vec(0u8..100, 22 * 9),
        minute in 0u32..1440,
    ) {
        let mut map = format!("{}\n", "#".repeat(11) + &".".repeat(13));
        for y in 0..9 {
            map.push('#');
            for x in 0..22 {
                map.push(match (x, y, cells[y * 22 + x]) {
                    (0, 4, _) => 'P',
                    (9, _, _) => '#',
                    (0..=8, _, 0..=19) => '#',
                    (0..=8, _, _) => '=',
                    (10.., _, 20..=29) => 'T',
                    _ => '.',
                });
            }
            map.push_str(".\n");
        }
        map.push_str(&("#".repeat(11) + &".".repeat(13)));
        map.push('\n');
        let mut w = outdoors(&map, "00:00");
        w.apply(&Action::SkipTime(Duration::minutes(u64::from(minute))));
        let view = w.perception();
        for y in 0..view.height() {
            for x in 0..view.width() {
                let p = Pos::new(x, y);
                if let Some(light) = view.light(p) {
                    prop_assert!(light.light().is_dark(), "light at {}", p);
                }
            }
        }
        prop_assert!(view.light_sources().is_empty());
        check_invariants(&w)?;
    }
}
