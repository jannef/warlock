//! Moonlight through the whole turn pipeline (spec §5 "Moonlight").

use super::light_tests::{check_invariants, check_wall_completion};
use super::sun_tests::{field, sky_world};
use super::*;
use crate::light::Light;
use crate::perception::{CellView, Emitter, PerceptionView, SeenLight};
use crate::replay::Replay;
use proptest::prelude::*;
use std::collections::BTreeSet;

/// The built-in moon: faint, slightly purple, new on the 1st and full on the 16th at midnight.
const MOON: &str = r#"
[[moon]]
name = "Moon"
color = [48, 28, 66, 0]
sky = [11, 7, 15, 0]
period_days = 30
"#;

/// A second moon: small, fast, and on its own cycle.
const SECOND: &str = r#"
[[moon]]
name = "Ilse"
color = [40, 40, 44, 0]
sky = [6, 6, 8, 0]
size = 60
period_days = 7
phase_offset_days = 3
"#;

/// The night of the full moon (it is full at midnight).
const FULL: [u32; 3] = [1, 1, 16];
/// First quarter, half lit, high in the sky at dusk.
const HALF: [u32; 3] = [1, 1, 8];
/// The new moon.
const NEW: [u32; 3] = [1, 1, 1];

fn outdoors(map: &str, date: [u32; 3], time: &str) -> World {
    sky_world(map, "outdoor", date, time, MOON)
}

fn moon_sources<'a>(view: &PerceptionView<'a>) -> Vec<&'a SeenLight> {
    (view.light_sources().iter())
        .filter(|s| matches!(s.emitter, Emitter::Moon(_)))
        .collect()
}

/// Visible cells the first moon lights.
fn moonlit(view: &PerceptionView<'_>) -> BTreeSet<Pos> {
    moon_sources(view)
        .first()
        .map(|s| s.contributions.iter().map(|(p, _)| *p).collect())
        .unwrap_or_default()
}

/// Visible transparent cells.
fn open_in_view(view: &PerceptionView<'_>) -> BTreeSet<Pos> {
    let content = view.content();
    (0..view.height())
        .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
        .filter(|p| matches!(view.cell(*p), CellView::Visible(t) if content.terrain(t).transparent))
        .collect()
}

#[test]
fn a_full_moon_lights_open_ground_dimly_and_a_new_moon_not_at_all() {
    let map = field(41, 21, &[(20, 10, '@')]);
    let w = outdoors(&map, FULL, "00:00");
    let view = w.perception();
    let sky = view.sky().expect("under the open sky");
    let moon = sky.moons[0];
    assert!(sky.sun.is_night() && moon.shines());
    assert_eq!(
        moon.light,
        Light::new(48, 28, 66, 0),
        "full, high, at night"
    );
    let sources = moon_sources(&view);
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].emitter, Emitter::Moon(moon));
    assert_eq!((sources[0].pos(), sources[0].preset()), (None, None));
    // Every cell of the field is moonlit and under the moon's sky; normal eyes see all of it,
    // dimly.
    for y in 0..21 {
        for x in 0..41 {
            let p = Pos::new(x, y);
            let light = view.light(p).expect("the whole field is in view").light();
            let base = view.base_light(p).expect("visible");
            assert_eq!(
                base.max(moon.sky),
                base,
                "moon sky in the base light at {p}"
            );
            assert_eq!(light, base.saturating_add(moon.light), "{p}");
            let [r, g, b] = light.rgb;
            assert!(b > r && r > g, "faintly purple at {p}: {light:?}");
            assert!((40..64).contains(&light.luma()), "dim at {p}: {light:?}");
        }
    }
    check_invariants(&w).expect("light invariants");

    // New moon: nothing but the player's own cell.
    let w = outdoors(&map, NEW, "00:00");
    let view = w.perception();
    assert!(view.light_sources().is_empty());
    assert_eq!(view.base_light(Pos::new(20, 10)), Some(Light::DARK));
    assert!(!view.is_visible(Pos::new(21, 10)));
    check_invariants(&w).expect("light invariants");
}

#[test]
fn normal_eyes_need_a_bright_moon_and_low_light_eyes_see_by_a_half_moon() {
    let map = field(41, 21, &[(20, 10, '@')]);
    let mut w = outdoors(&map, HALF, "20:00");
    let moon = w.perception().sky().expect("outdoors").moons[0];
    assert!(moon.shines() && moon.elevation > 400, "{moon:?}");
    assert!(!w.perception().is_visible(Pos::new(25, 10)), "too dim");
    w.apply(&Action::SetVision("low_light".into()));
    let view = w.perception();
    assert!(view.is_visible(Pos::new(25, 10)) && view.is_visible(Pos::new(0, 0)));
    check_invariants(&w).expect("light invariants");
    // A full moon low in the east early in the evening is too dim for normal eyes too.
    let early = outdoors(&map, FULL, "18:30");
    assert!(!early.perception().is_visible(Pos::new(25, 10)));
    let late = outdoors(&map, FULL, "21:00");
    assert!(late.perception().is_visible(Pos::new(25, 10)));
}

#[test]
fn the_moon_casts_shadows_away_from_it() {
    // A transparent bush 2 tall in a field: the moon lights every open cell in view except the
    // bush's moon shadow, which turns from west through north to east over the night.
    let bush = Pos::new(20, 10);
    let mut w = sky_world(
        &field(41, 21, &[(20, 10, 'b'), (20, 17, '@')]),
        "outdoor",
        [1, 1, 15],
        "20:00",
        MOON,
    );
    w.apply(&Action::SetVision("low_light".into()));
    let mut seen_west = false;
    let mut seen_east = false;
    for _ in 0..9 {
        w.apply(&Action::SkipTime(Duration::minutes(60)));
        let view = w.perception();
        let moon = view.sky().expect("outdoors").moons[0];
        assert!(moon.shines(), "{}", w.time());
        let shadow: BTreeSet<Pos> = (moon.shadow_steps(2))
            .map(|(dx, dy)| Pos::new(bush.x + dx, bush.y + dy))
            .collect();
        let lit = moonlit(&view);
        assert!(lit.is_disjoint(&shadow), "{}", w.time());
        for p in open_in_view(&view) {
            assert!(
                lit.contains(&p) || shadow.contains(&p) || p == bush,
                "{p} at {}",
                w.time()
            );
        }
        seen_west |= shadow.iter().all(|p| p.x < bush.x);
        seen_east |= shadow.iter().all(|p| p.x > bush.x);
        check_invariants(&w).expect("light invariants");
    }
    assert!(seen_west && seen_east, "evening shadows west, morning east");
}

#[test]
fn roofs_and_walls_keep_moonlight_out() {
    let hut = "\
.................
.....#######.....
.....#=====#.....
.....#=====#.....
.....#==P==#.....
.....###+###.....
.................
";
    let mut w = outdoors(hut, FULL, "00:00");
    w.apply(&Action::UseTool);
    let mut cellar = sky_world(hut, "dungeon", FULL, "00:00", MOON);
    cellar.apply(&Action::UseTool);
    let (view, dark) = (w.perception(), cellar.perception());
    assert!(
        moon_sources(&view).is_empty(),
        "not even on inner wall faces"
    );
    assert!(dark.sky().is_none(), "no sky underground");
    for y in 0..7 {
        for x in 0..17 {
            let p = Pos::new(x, y);
            assert_eq!(view.light(p), dark.light(p), "{p}");
        }
    }
    w.apply(&Action::Open(Dir::S));
    let view = w.perception();
    assert!(moonlit(&view).contains(&Pos::new(8, 5)), "the open doorway");
    assert!(moonlit(&view).iter().all(|p| p.y >= 5));
    check_invariants(&w).expect("light invariants");
}

#[test]
fn several_moons_add_up_each_with_its_own_light_and_shadows() {
    let sky = format!("{MOON}{SECOND}");
    let map = field(41, 21, &[(20, 10, 'b'), (20, 17, '@')]);
    let w = sky_world(&map, "outdoor", FULL, "02:00", &sky);
    let view = w.perception();
    let moons = &view.sky().expect("outdoors").moons;
    assert_eq!(moons.len(), 2);
    assert!(moons.iter().all(|m| m.shines()), "{moons:?}");
    assert_ne!(moons[0].shadow, moons[1].shadow);
    let sources = moon_sources(&view);
    assert_eq!(sources.len(), 2);
    assert_ne!(sources[0].key, sources[1].key);
    let shadows: Vec<BTreeSet<Pos>> = moons
        .iter()
        .map(|m| (m.shadow_steps(2).map(|(dx, dy)| Pos::new(20 + dx, 10 + dy))).collect())
        .collect();
    assert_ne!(shadows[0], shadows[1]);
    for (source, shadow) in sources.iter().zip(&shadows) {
        for (p, _) in &source.contributions {
            assert!(!shadow.contains(p), "{p} lit by the moon it is shaded from");
        }
    }
    // Open ground away from the bush: base light (both skies) plus both moons.
    let p = Pos::new(5, 15);
    let base = view.base_light(p).expect("visible");
    let skies = moons[0].sky.saturating_add(moons[1].sky);
    assert_eq!(base.max(skies), base);
    let both = moons[0].light.saturating_add(moons[1].light);
    assert_eq!(
        view.light(p).expect("visible").light(),
        base.saturating_add(both)
    );
    check_invariants(&w).expect("light invariants");
}

#[test]
fn creatures_are_seen_by_the_full_moon_but_not_the_new() {
    let map = field(41, 9, &[(5, 4, '@'), (30, 4, 'r')]);
    let seen = |date| {
        let w = outdoors(&map, date, "00:00");
        let view = w.perception();
        view.actors().iter().any(|a| a.kind == ActorKind::Creature)
    };
    assert!(seen(FULL));
    assert!(!seen(NEW));
}

#[test]
fn moonlight_is_deterministic_and_replays_verify() {
    // The built-in courtyard on the night of the full moon.
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let actions = crate::action::parse_actions(
        "skip:113h skip:30m move:E move:E move:SE rest:1h move:E skip:3h move:W rest:dawn",
    )
    .expect("actions parse");
    let mut a = World::new(content.clone(), "courtyard", None).expect("courtyard");
    let mut b = World::new(content.clone(), "courtyard", None).expect("courtyard");
    let mut replay = Replay::new(&a);
    let mut moonlit_turns = 0;
    for action in &actions {
        a.apply(action);
        b.apply(action);
        replay.push(action, &a);
        assert_eq!(a.perception, b.perception);
        moonlit_turns += usize::from(!moon_sources(&a.perception()).is_empty());
        check_invariants(&a).expect("light invariants");
    }
    assert!(moonlit_turns > 3, "{moonlit_turns}");
    let replayed = replay.run(content).expect("the replay verifies");
    assert_eq!(replayed.state_hash(), a.state_hash());
    assert_eq!(replayed.perception, a.perception);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Random gardens (walls, trees, bushes, roofed porch) under two moons at any moment of a
    /// month: the light invariants hold, no moon lights a roofed cell, and each moon lights
    /// every open cell it reaches the same.
    #[test]
    fn moon_invariants_hold_in_random_gardens(
        cells in prop::collection::vec(0u8..100, 22 * 9),
        minutes in 0u64..(30 * 24 * 60),
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
        let sky = format!("{MOON}{SECOND}");
        let mut w = sky_world(&map, "outdoor", NEW, "00:00", &sky);
        w.apply(&Action::SetVision(vision.into()));
        w.apply(&Action::SkipTime(Duration::minutes(minutes)));
        let view = w.perception();
        let moons = view.sky().expect("outdoors").moons.clone();
        let content = view.content();
        for source in moon_sources(&view) {
            let Emitter::Moon(moon) = source.emitter else { unreachable!() };
            prop_assert_eq!(moon, moons[usize::from(moon.index)]);
            prop_assert!(moon.shines());
            for &(p, light) in &source.contributions {
                let CellView::Visible(t) = view.cell(p) else {
                    return Err(TestCaseError::fail(format!("{p} not visible")));
                };
                prop_assert!(!content.terrain(t).roofed, "moon under a roof at {}", p);
                prop_assert_eq!(light, moon.light, "the moon is the same everywhere: {}", p);
            }
        }
        check_invariants(&w)?;
        check_wall_completion(&w)?;
    }
}
