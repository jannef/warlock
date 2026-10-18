//! Light and vision through the whole turn pipeline (spec §5).

use super::*;
use crate::content::ContentFiles;
use crate::light::{BOUNCE, Light, falloff};
use crate::perception::{CellView, Emitter, PerceptionView};
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

[[terrain]]
id = "rune"
name = "rune"
glyph = "R"
fg = "#c080ff"
light = "uv"
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "rat"
name = "rat"
glyph = "r"
fg = "#806040"
weight = 4000

[[creature]]
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#40a040"
weight = 25000
light = "torch"
"##;

const LIGHTS: &str = r##"
[[light]]
id = "torch"
color = [255, 255, 255, 0]
radius = 4

[[light]]
id = "red"
color = [400, 0, 0, 0]
radius = 3

[[light]]
id = "dim"
color = [20, 20, 20, 0]
radius = 1

[[light]]
id = "uv"
color = [0, 0, 0, 200]
radius = 2

[[light]]
id = "lantern"
color = [255, 255, 255, 0]
radius = 3

[[light]]
id = "glare"
color = [2000, 2000, 2000, 0]
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

[[vision]]
id = "darkvision"
name = "Darkvision"
threshold = 40
darkvision = 2

[[vision]]
id = "uv_sight"
name = "UV sight"
threshold = 40
sees_uv = true
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"'" = "door_open"
"R" = "rune"
"@" = { terrain = "floor", spawn = "player" }
"T" = { terrain = "floor", light = "torch" }
"D" = { terrain = "floor", light = "dim" }
"E" = { terrain = "floor", light = "red" }
"G" = { terrain = "floor", light = "glare" }
"r" = { terrain = "floor", creature = "rat" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

/// A world on a single level with `map`, `ambient` light, and the fixture definitions above. The
/// player starts with `vision` and an unlit lantern.
fn world_with(map: &str, ambient: [u16; 4], vision: &str) -> World {
    load(map, ambient, vision, "lantern = \"lantern\"\n")
}

fn load(map: &str, ambient: [u16; 4], vision: &str, scenario_extra: &str) -> World {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/lights.toml", LIGHTS);
    files.insert("data/vision.toml", VISION);
    files.insert("data/legend.toml", LEGEND);
    let [r, g, b, uv] = ambient;
    files.insert(
        "levels/test.toml",
        &format!("name = \"Test\"\nambient = [{r}, {g}, {b}, {uv}]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/test.toml",
        &format!("name = \"T\"\nstart_level = \"test\"\nvision = \"{vision}\"\n{scenario_extra}"),
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test", None).expect("world")
}

fn dark_world(map: &str, vision: &str) -> World {
    world_with(map, [0; 4], vision)
}

/// Puts the player at `p` (via saved state, as a save/restore would).
fn teleport(world: &World, p: Pos) -> World {
    let mut state = world.state().clone();
    let player = state.player;
    state.actors.get_mut(&player).expect("player").pos = p;
    World::from_state(world.content().clone(), state).expect("valid state")
}

fn visible(view: &PerceptionView<'_>, p: Pos) -> bool {
    matches!(view.cell(p), CellView::Visible(_))
}

/// Σ contributions + base light for every visible cell, recomputed from `light_sources()`.
fn summed_light(view: &PerceptionView<'_>) -> BTreeMap<Pos, Light> {
    let mut sum = BTreeMap::new();
    for y in 0..view.height() {
        for x in 0..view.width() {
            let p = Pos::new(x, y);
            if let Some(base) = view.base_light(p) {
                sum.insert(p, base);
            }
        }
    }
    for source in view.light_sources() {
        for (p, c) in &source.contributions {
            let acc = sum.get_mut(p).expect("contributions only on visible cells");
            *acc = acc.saturating_add(*c);
        }
    }
    sum
}

/// Bounce light at a visible cell: its base light minus ambient light.
fn indirect(view: &PerceptionView<'_>, p: Pos) -> [u16; 3] {
    let base = view.base_light(p).expect("visible");
    std::array::from_fn(|c| base.rgb[c] - view.ambient().rgb[c])
}

/// Light API invariants that must hold in any state.
pub(super) fn check_invariants(world: &World) -> Result<(), TestCaseError> {
    let view = world.perception();
    for (p, sum) in summed_light(&view) {
        let light = view.light(p).expect("visible cells have light");
        prop_assert_eq!(light.light(), sum, "Σ contributions + base light at {}", p);
        prop_assert_eq!(light.dist2, p.dist2(view.viewer()));
        if !view.vision().sees_uv {
            prop_assert_eq!(light.uv, 0, "UV is never exposed to eyes without UV sight");
        }
        let base = view.base_light(p).expect("visible");
        prop_assert_eq!(base.uv, view.ambient().uv, "UV does not bounce");
        prop_assert_eq!(base.max(view.ambient()), base, "bounce only adds light");
    }
    check_wall_faces(&view)?;
    prop_assert!(
        view.is_visible(view.viewer()),
        "the player's own cell is visible"
    );
    check_creature_sight(world)?;
    let cold = World::from_state(world.content().clone(), world.state().clone())
        .map_err(|e| TestCaseError::fail(e.to_string()))?;
    prop_assert_eq!(
        &cold.perception,
        &world.perception,
        "a cold light cache agrees"
    );
    Ok(())
}

/// Creature sight uses the player's rules (spec §5 "Creature sight"): on every transparent cell
/// the player sees, the light a creature with the player's eyes finds there ([`crate::sight::light`],
/// one cell at a time, from a cold light cache) is the light the player perceives.
pub(super) fn check_creature_sight(world: &World) -> Result<(), TestCaseError> {
    let view = world.perception();
    let content = world.content();
    let mut cold = LightCache::default();
    for y in 0..view.height() {
        for x in 0..view.width() {
            let p = Pos::new(x, y);
            let (Some(light), CellView::Visible(t)) = (view.light(p), view.cell(p)) else {
                continue;
            };
            if !content.terrain(t).transparent {
                continue;
            }
            let eye = view.vision();
            let found = crate::sight::light(&world.state, content, &mut cold, p, eye, true);
            prop_assert_eq!(found, light.light(), "creature sight's light at {}", p);
            let quick = crate::sight::light(&world.state, content, &mut cold, p, eye, false);
            prop_assert_eq!(quick.max(found), found, "no more without bounce at {}", p);
        }
    }
    Ok(())
}

/// Wall faces: a source lights a visible opaque cell exactly as at the cell's own distance, and
/// only if the cell faces a visible transparent cell the same source lights (so no light from
/// beyond the wall); its bounce light is the brightest of its visible transparent neighbors'.
fn check_wall_faces(view: &PerceptionView<'_>) -> Result<(), TestCaseError> {
    let content = view.content();
    let opaque = |p: Pos| match view.cell(p) {
        CellView::Visible(t) => !content.terrain(t).transparent,
        _ => false,
    };
    let transparent =
        |p: Pos| matches!(view.cell(p), CellView::Visible(t) if content.terrain(t).transparent);
    for source in view.light_sources() {
        let lit: BTreeSet<Pos> = (source.contributions.iter())
            .map(|(p, _)| *p)
            .filter(|p| transparent(*p))
            .collect();
        for &(p, c) in source.contributions.iter().filter(|(p, _)| opaque(*p)) {
            let own = match source.emitter {
                Emitter::Point { pos, preset } => {
                    let preset = content.light(preset);
                    falloff(preset.color, preset.radius, p.dist2(pos))
                }
                Emitter::Sun(sun) => sun.light,
                Emitter::Moon(moon) => moon.light,
            };
            prop_assert_eq!(
                c,
                view.vision().perceive(own),
                "wall {} at its own distance",
                p
            );
            prop_assert!(
                Dir::ALL.iter().any(|d| lit.contains(&p.step(*d))),
                "wall {} faces no visible cell the source lights",
                p
            );
        }
    }
    for y in 0..view.height() {
        for x in 0..view.width() {
            let p = Pos::new(x, y);
            if opaque(p) {
                let brightest = Dir::ALL
                    .iter()
                    .map(|d| p.step(*d))
                    .filter(|q| transparent(*q))
                    .map(|q| indirect(view, q))
                    .fold([0; 3], |acc, i| std::array::from_fn(|c| acc[c].max(i[c])));
                prop_assert_eq!(indirect(view, p), brightest, "bounce on wall {}", p);
            }
        }
    }
    Ok(())
}

#[test]
fn darkness_hides_everything_but_the_players_cell() {
    let w = dark_world("#####\n#@..#\n#####\n", "normal");
    let view = w.perception();
    assert!(visible(&view, Pos::new(1, 1)));
    assert!(!visible(&view, Pos::new(2, 1)));
    assert!(!visible(&view, Pos::new(0, 1)), "walls need light too");
    assert_eq!(view.light(Pos::new(2, 1)), None);
    let own = view.light(Pos::new(1, 1)).expect("own cell");
    assert_eq!((own.rgb, own.darkvision), ([0; 3], false));
}

#[test]
fn ambient_light_lights_everything_in_sight() {
    let w = world_with("######\n#@...#\n######\n", [60, 60, 60, 0], "normal");
    let view = w.perception();
    assert_eq!(view.ambient(), Light::new(60, 60, 60, 0));
    for x in 0..6 {
        assert!(visible(&view, Pos::new(x, 1)), "({x}, 1)");
    }
    assert_eq!(view.light(Pos::new(3, 1)).map(|l| l.rgb), Some([60; 3]));
    assert!(view.light_sources().is_empty());
}

#[test]
fn a_torch_lights_its_room_with_falloff() {
    let w = dark_world("###########\n#@...T....#\n###########\n", "normal");
    let view = w.perception();
    // The torch's own light: the cell's light minus its base (bounce) light.
    let at = |x: i32| {
        let p = Pos::new(x, 1);
        (view.light(p).zip(view.base_light(p))).map(|(l, base)| l.rgb[0] - base.rgb[0])
    };
    assert_eq!(at(5), Some(255), "full color at the source");
    assert_eq!(at(4), Some(255 * 19 / 20), "R² = 4² + 4 = 20, d² = 1");
    assert_eq!(at(3), Some(255 * 16 / 20));
    assert!(at(4) > at(3) && at(3) > at(2), "monotonic falloff");
    assert!(
        visible(&view, Pos::new(9, 1)),
        "d = 4 is lit: 255·4/20 = 51 ≥ 40"
    );
    let sources = view.light_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].pos(), Some(Pos::new(5, 1)));
    let preset = sources[0].preset().expect("a point light");
    assert_eq!(view.content().light(preset).id, "torch");
}

#[test]
fn light_never_crosses_walls_and_wall_faces_take_their_sides_light() {
    // Dark room (x 1..3) | wall x = 4 | lit room (x 5..8) with a torch at (6, 2).
    let map = "##########\n#...#....#\n#@..#.T..#\n#...#....#\n##########\n";
    let w = dark_world(map, "darkvision");
    let wall = Pos::new(4, 2);
    // From the dark side, darkvision shows the wall, but it is dark: no light leaks through.
    let near = teleport(&w, Pos::new(3, 2));
    let view = near.perception();
    let face = view.light(wall).expect("wall seen with darkvision");
    assert_eq!(face.rgb, [0; 3]);
    assert!(face.darkvision);
    assert_eq!(
        view.base_light(Pos::new(3, 2)),
        Some(Light::DARK),
        "no bounce light through the wall"
    );
    assert!(
        view.light_sources().is_empty(),
        "the torch's light is not seen"
    );
    // From the lit side, the wall is lit from its own distance to the torch (d² = 4), exactly as
    // the floor at that distance, plus the bounce light of the floor in front of it.
    let lit = teleport(&w, Pos::new(5, 2));
    let view = lit.perception();
    let torch = &view.light_sources()[0];
    let from_torch = |p: Pos| {
        torch
            .contributions
            .iter()
            .find(|(q, _)| *q == p)
            .map(|c| c.1)
    };
    let own = falloff(Light::new(255, 255, 255, 0), 4, 4);
    assert_eq!(from_torch(wall), Some(own));
    assert_eq!(from_torch(Pos::new(8, 2)), Some(own), "the floor at d² = 4");
    let base = view.base_light(wall).expect("wall in view");
    assert!(!base.is_dark(), "lit floor bounces light onto the wall");
    assert_eq!(
        view.light(wall).map(|l| l.light()),
        Some(base.saturating_add(own))
    );
}

#[test]
fn bounce_light_reaches_around_a_corner_and_fades() {
    // A glaring light in the west room (x 1..3). The east room (x 5..7) opens onto it only through
    // (4, 3), which is out of the light's reach (d² = 13 ≥ R² = 12): no direct light gets in.
    let map = "#########\n#G..#..@#\n#...#...#\n#.......#\n#########\n";
    let w = dark_world(map, "low_light");
    let view = w.perception();
    let (gap, corner) = (Pos::new(4, 3), Pos::new(5, 3));
    for p in [gap, corner] {
        let light = view
            .light(p)
            .expect("bounce light makes it visible")
            .light();
        assert_eq!(Some(light), view.base_light(p), "{p}: no direct light");
        assert!(
            u32::from(light.rgb[0]) <= 2000 * BOUNCE / 256,
            "{p}: a small fraction of the glare"
        );
    }
    let brightness = |p: Pos| view.light(p).map_or(0, |l| l.brightness());
    assert!(
        brightness(gap) >= 40,
        "the gap is lit enough for normal eyes"
    );
    assert!(
        (12..40).contains(&brightness(corner)),
        "around the corner: only low-light eyes see it"
    );
    assert_eq!(
        view.base_light(Pos::new(7, 1)),
        Some(Light::DARK),
        "4 steps from the nearest lit cell: out of reach"
    );
    check_invariants(&w).expect("invariants");
    let normal = dark_world(map, "normal");
    assert!(visible(&normal.perception(), gap));
    assert!(!visible(&normal.perception(), corner));
}

#[test]
fn walls_are_never_brighter_than_the_floor_at_their_distance() {
    // A dim light (radius 1) in a nook: the walls around it get the light of a cell one step away,
    // not the full light of the source's own cell.
    let map = "#####\n#.D.#\n#...#\n#.@.#\n#####\n";
    let w = dark_world(map, "low_light");
    let view = w.perception();
    let dim = &view.light_sources()[0];
    let source = Pos::new(2, 1);
    for &(p, c) in &dim.contributions {
        assert_eq!(
            c,
            falloff(Light::new(20, 20, 20, 0), 1, p.dist2(source)),
            "{p}"
        );
    }
    let wall = Pos::new(2, 0);
    assert!(visible(&view, wall), "it bounds seen floor");
    assert_eq!(
        dim.contributions
            .iter()
            .find(|(p, _)| *p == wall)
            .map(|c| c.1),
        Some(falloff(Light::new(20, 20, 20, 0), 1, 1)),
        "d² = 1, not the source's full light"
    );
    check_invariants(&w).expect("invariants");
}

#[test]
fn thresholds_differ_per_profile() {
    // A dim light (luma 20) two cells from the player.
    let map = "#######\n#@..D.#\n#######\n";
    let normal = dark_world(map, "normal");
    assert!(!visible(&normal.perception(), Pos::new(4, 1)));
    let low = dark_world(map, "low_light");
    let view = low.perception();
    assert!(visible(&view, Pos::new(4, 1)), "luma 20 ≥ 12");
    assert!(!visible(&view, Pos::new(2, 1)), "outside the dim light");
    assert_eq!(view.vision().id, "low_light");
}

#[test]
fn darkvision_sees_a_radius_of_darkness() {
    let map = "##########\n#@.......#\n##########\n";
    let w = dark_world(map, "darkvision");
    let view = w.perception();
    // Radius 2: d² ≤ 2² + 2 = 6.
    assert!(visible(&view, Pos::new(3, 1)));
    assert!(!visible(&view, Pos::new(4, 1)));
    let l = view.light(Pos::new(3, 1)).expect("visible");
    assert!(l.darkvision);
    assert_eq!(l.dist2, 4);
}

#[test]
fn uv_light_is_only_perceived_with_uv_sight() {
    let map = "#######\n#@..R.#\n#######\n";
    let rune = Pos::new(4, 1);
    let normal = dark_world(map, "normal");
    let view = normal.perception();
    assert!(!visible(&view, rune));
    assert!(view.light_sources().is_empty());
    let uv = dark_world(map, "uv_sight");
    let view = uv.perception();
    assert!(visible(&view, rune));
    assert_eq!(view.light(rune).map(|l| (l.rgb, l.uv)), Some(([0; 3], 200)));
    assert_eq!(view.light_sources().len(), 1);
}

#[test]
fn uv_is_hidden_from_eyes_without_uv_sight_even_in_lit_cells() {
    let map = "########\n#@.TR..#\n########\n";
    let w = dark_world(map, "normal");
    let view = w.perception();
    let l = view.light(Pos::new(4, 1)).expect("lit by the torch");
    assert_eq!(l.uv, 0);
    assert_eq!(
        view.light_sources().len(),
        1,
        "the rune's light is not perceived"
    );
}

#[test]
fn creatures_in_darkness_are_hidden_but_light_bearers_are_seen() {
    // A rat and a torch-carrying kobold, both in line of sight, in the dark.
    let map = "############\n#@...r....k#\n############\n";
    let w = dark_world(map, "normal");
    let view = w.perception();
    let seen: Vec<_> = view.actors().iter().map(|a| a.pos).collect();
    assert_eq!(seen, vec![Pos::new(1, 1), Pos::new(10, 1)]);
    let carried = &view.light_sources()[0];
    assert_eq!(carried.pos(), Some(Pos::new(10, 1)));
    // With darkvision the rat would be seen up close.
    let dv = teleport(&dark_world(map, "darkvision"), Pos::new(3, 1));
    assert!(dv.perception().actor_at(Pos::new(5, 1)).is_some());
}

#[test]
fn opening_a_door_lets_light_in_and_closing_it_shuts_it_out() {
    // Player in a dark room; a lit room behind a closed door at (4, 1).
    let map = "##########\n#..@+...T#\n#...#....#\n##########\n";
    let mut w = dark_world(map, "normal");
    let door = Pos::new(4, 1);
    assert!(!visible(&w.perception(), Pos::new(6, 1)));
    assert!(w.apply(&Action::Open(Dir::E)).took_turn);
    let view = w.perception();
    assert!(
        visible(&view, Pos::new(6, 1)),
        "the lit room is seen through the door"
    );
    assert!(visible(&view, door), "the open doorway is lit");
    check_invariants(&w).expect("invariants after opening");
    assert!(w.apply(&Action::Close(Dir::E)).took_turn);
    let view = w.perception();
    assert!(!visible(&view, Pos::new(6, 1)));
    assert!(view.light_sources().is_empty());
    check_invariants(&w).expect("invariants after closing");
}

#[test]
fn door_changes_invalidate_cached_masks_of_lights_inside_the_room() {
    // The torch's light reaches the doorway; the player stands where it would spill through.
    let map = "#########\n#@..+.T.#\n#########\n";
    let mut w = dark_world(map, "normal");
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Move(Dir::E)); // (3, 1), next to the door
    assert!(!visible(&w.perception(), Pos::new(5, 1)));
    w.apply(&Action::Open(Dir::E));
    assert!(
        w.perception()
            .light(Pos::new(3, 1))
            .is_some_and(|l| l.rgb[0] > 0),
        "light spills through the doorway onto the player"
    );
    check_invariants(&w).expect("cache agrees with a cold recomputation");
}

#[test]
fn the_lantern_takes_a_turn_and_lights_the_way() {
    let map = "##########\n#@.......#\n##########\n";
    let mut w = dark_world(map, "normal");
    assert_eq!(w.lantern(), Some(false));
    assert!(!visible(&w.perception(), Pos::new(2, 1)));
    let outcome = w.apply(&Action::ToggleLantern);
    assert!(outcome.took_turn);
    assert_eq!(w.lantern(), Some(true));
    assert!(visible(&w.perception(), Pos::new(3, 1)));
    let source = &w.perception().light_sources()[0];
    let key = source.key;
    w.apply(&Action::Move(Dir::E));
    let source = &w.perception().light_sources()[0];
    assert_eq!(
        source.key, key,
        "carried lights keep their key as they move"
    );
    assert_eq!(source.pos(), Some(Pos::new(2, 1)));
    w.apply(&Action::ToggleLantern);
    assert!(w.perception().light_sources().is_empty());
}

#[test]
fn without_a_lantern_toggling_is_refused() {
    let mut w = load("####\n#@.#\n####\n", [0; 4], "normal", "");
    let outcome = w.apply(&Action::ToggleLantern);
    assert!(!outcome.took_turn);
    assert_eq!(w.lantern(), None);
    assert_eq!(
        w.messages().iter().last().map(|m| m.text.as_str()),
        Some("You have no lantern.")
    );
}

#[test]
fn set_vision_takes_no_time_but_recomputes_perception() {
    let map = "#######\n#@..D.#\n#######\n";
    let mut w = dark_world(map, "normal");
    let outcome = w.apply(&Action::SetVision("low_light".into()));
    assert!(!outcome.took_turn);
    assert_eq!(w.turn(), 0);
    assert_eq!(w.perception().vision().name, "Low-light");
    assert!(visible(&w.perception(), Pos::new(4, 1)));
    let outcome = w.apply(&Action::SetVision("x-ray".into()));
    assert!(!outcome.took_turn);
    assert_eq!(
        w.perception().vision().id,
        "low_light",
        "unknown profiles are refused"
    );
}

#[test]
fn unknown_vision_in_saved_state_is_rejected() {
    let w = dark_world("####\n#@.#\n####\n", "normal");
    let mut state = w.state().clone();
    state.vision = "x-ray".into();
    assert!(matches!(
        World::from_state(w.content().clone(), state),
        Err(WorldError::InconsistentState(_))
    ));
}

#[test]
fn overlapping_lights_sum_and_contributions_add_up() {
    let map = "############\n#@..T..E...#\n#....#.....#\n############\n";
    let w = world_with(map, [5, 5, 5, 0], "normal");
    check_invariants(&w).expect("invariants");
    let view = w.perception();
    assert_eq!(view.light_sources().len(), 2);
    let both = view.light(Pos::new(6, 1)).expect("lit");
    assert!(both.rgb[0] > both.rgb[1], "the red light adds to the torch");
}

#[test]
fn same_actions_give_identical_states_and_perception() {
    let map = "############\n#@..+..T.k.#\n#...#......#\n############\n";
    let actions = [
        Action::ToggleLantern,
        Action::Move(Dir::E),
        Action::SetVision("darkvision".into()),
        Action::Move(Dir::E),
        Action::Open(Dir::E),
        Action::Wait,
        Action::Close(Dir::E),
        Action::SetVision("uv_sight".into()),
        Action::ToggleLantern,
    ];
    let mut a = dark_world(map, "normal");
    let mut b = dark_world(map, "normal");
    for action in &actions {
        let oa = a.apply(action);
        let ob = b.apply(action);
        assert_eq!(oa, ob);
        assert_eq!(a.state(), b.state());
        assert_eq!(a.perception, b.perception);
    }
}

#[test]
fn only_lights_near_the_player_get_masks() {
    // A long corridor with a torch every 4 cells; only those within reach are computed.
    let len = 500;
    let mut row = String::from("#@");
    for x in 2..len - 1 {
        row.push(if x % 4 == 0 { 'T' } else { '.' });
    }
    row.push('#');
    let wall = "#".repeat(len);
    let w = dark_world(&format!("{wall}\n{row}\n{wall}\n"), "normal");
    let total = (2..len - 1).filter(|x| x % 4 == 0).count();
    let cached = w.lights.cached_masks("test");
    // Reach: view radius 40 + 1 (wall faces) + 6 (bounce) + torch radius 4.
    let near = (2..len - 1)
        .filter(|x| x % 4 == 0 && (*x as i32 - 1) <= 51)
        .count();
    assert_eq!(cached, near);
    assert!(cached * 8 < total, "{cached} of {total} lights computed");
}

#[test]
fn showcase_start_room_is_lit_and_readable() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let w = World::new(content, "showcase", None).expect("showcase");
    let view = w.perception();
    assert_eq!(view.vision().id, "normal");
    assert_eq!(w.lantern(), Some(false));
    // The start room spans x 1..14, y 12..19; every cell and its walls are visible.
    for y in 11..=20 {
        for x in 0..=15 {
            let p = Pos::new(x, y);
            assert!(visible(&view, p), "start room cell {p} should be lit");
        }
    }
    assert!(view.light(w.player_pos()).expect("own cell").brightness() >= 40);
    check_invariants(&w).expect("invariants");
}

#[test]
fn showcase_darkness_chamber_needs_special_eyes() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let w = World::new(content, "showcase", None).expect("showcase");
    // Inside the inner sanctum, next to the rune at (37, 5) and fungus at (40, 4).
    let rune = Pos::new(37, 5);
    let fungus = Pos::new(40, 4);
    let mut at = teleport(&w, Pos::new(35, 4));
    let seen = |w: &World| {
        (
            visible(&w.perception(), rune),
            visible(&w.perception(), fungus),
        )
    };
    assert_eq!(seen(&at), (false, false), "normal eyes see nothing");
    at.apply(&Action::SetVision("low_light".into()));
    assert_eq!(seen(&at), (false, true), "low-light sees the fungus");
    // Its glow reaches the floor two cells away, and lights the walls no brighter than the floor
    // at the same distance.
    let view = at.perception();
    assert!(visible(&view, Pos::new(38, 4)), "d² = 4 from the fungus");
    let glow = (view.light_sources().iter())
        .find(|s| s.pos() == Some(fungus))
        .expect("the fungus");
    let from_fungus = |p: Pos| {
        glow.contributions
            .iter()
            .find(|(q, _)| *q == p)
            .map(|c| c.1)
    };
    assert!(from_fungus(Pos::new(40, 3)).is_some());
    assert_eq!(
        from_fungus(Pos::new(40, 3)),
        from_fungus(Pos::new(40, 5)),
        "wall and floor at d² = 1"
    );
    at.apply(&Action::SetVision("uv_sight".into()));
    assert!(seen(&at).0, "UV sight sees the rune");
    at.apply(&Action::SetVision("darkvision".into()));
    assert_eq!(seen(&at), (true, true), "darkvision: both within radius 5");
    assert!(at.perception().light(rune).is_some_and(|l| l.darkvision));
    assert!(
        !visible(&at.perception(), Pos::new(41, 5)),
        "beyond darkvision range it is dark"
    );
    check_invariants(&at).expect("invariants");

    // A pair of fungi near the entrance: invisible to normal eyes even together.
    let mut pair = teleport(&w, Pos::new(31, 7));
    let fungi = [Pos::new(29, 8), Pos::new(30, 8)];
    assert!(fungi.iter().all(|p| !visible(&pair.perception(), *p)));
    pair.apply(&Action::SetVision("low_light".into()));
    assert!(fungi.iter().all(|p| visible(&pair.perception(), *p)));
    assert!(
        visible(&pair.perception(), Pos::new(30, 7)),
        "the glow reaches neighbors"
    );
}

/// A 400×300 open level: pillars every 7 cells, a torch every 12 cells (825 static lights), and
/// 60 torch-carrying kobolds on patrol.
fn vast_world() -> World {
    let (w, h) = (400, 300);
    let mut map = String::new();
    let mut kobolds = 0;
    for y in 0..h {
        for x in 0..w {
            let border = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            let ch = if border || (x % 7 == 3 && y % 7 == 3) {
                '#'
            } else if (x, y) == (200, 150) {
                '@'
            } else if x % 12 == 6 && y % 12 == 6 {
                'T'
            } else if x % 7 == 4 && y % 7 == 4 && x % 50 < 7 && y % 30 < 7 && kobolds < 60 {
                kobolds += 1;
                'p'
            } else {
                '.'
            };
            map.push(ch);
        }
        map.push('\n');
    }
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/lights.toml", LIGHTS);
    files.insert("data/vision.toml", VISION);
    files.insert("data/legend.toml", LEGEND);
    files.insert(
        "levels/vast.toml",
        &format!(
            "name = \"Vast\"\nmap = '''\n{map}'''\n[legend]\n\
             \"p\" = {{ terrain = \"floor\", creature = \"kobold\", route = \"E S W N\" }}\n"
        ),
    );
    files.insert(
        "scenarios/vast.toml",
        "name = \"V\"\nstart_level = \"vast\"\nlantern = \"lantern\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("vast content"));
    World::new(content, "vast", None).expect("vast world")
}

/// Average and maximum `apply` time over `turns` pseudo-random moves (lantern lit).
fn time_turns(mut world: World, turns: usize) -> (f64, f64) {
    use std::time::Instant;
    world.apply(&Action::ToggleLantern);
    let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
    let (mut total, mut max) = (0.0f64, 0.0f64);
    for _ in 0..turns {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let dir = Dir::ALL[(rng >> 61) as usize];
        let start = Instant::now();
        world.apply(&Action::Move(dir));
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        max = max.max(ms);
    }
    (total / turns as f64, max)
}

/// Turn cost against the spec §10 budgets (5 ms hub, 16 ms vast). Run in release:
/// `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture`.
#[test]
#[ignore = "timing; run manually in release mode"]
fn turn_cost() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let hub = World::new(content, "showcase", None).expect("showcase");
    let (avg, max) = time_turns(hub, 300);
    println!("hub  80×21:   avg {avg:.3} ms, max {max:.3} ms per turn");
    let vast = vast_world();
    let start = std::time::Instant::now();
    let cold = World::from_state(vast.content().clone(), vast.state().clone()).expect("restore");
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("vast 400×300: {ms:.3} ms to restore with a cold light cache");
    assert!(cold.lights.cached_masks("vast") > 0);
    let (avg, max) = time_turns(vast, 300);
    println!("vast 400×300: avg {avg:.3} ms, max {max:.3} ms per turn (825 lights, 60 carried)");
    // The same caverns under the open sky just after sunrise, when shadows are longest.
    let vast = vast_world();
    let mut state = vast.state().clone();
    for level in state.levels.values_mut() {
        level.kind = crate::content::LevelKind::Outdoor;
    }
    state.time = crate::time::GameTime::from_calendar(1, 1, 1, 6, 15).expect("valid");
    let outdoors = World::from_state(vast.content().clone(), state).expect("restore");
    assert!(outdoors.perception().sun().is_some_and(|s| s.is_up()));
    let (avg, max) = time_turns(outdoors, 300);
    println!("vast 400×300 outdoors at 06:15: avg {avg:.3} ms, max {max:.3} ms per turn");
}

/// Cells in line of sight from the player (independent of light), computed directly.
fn line_of_sight(world: &World) -> BTreeSet<Pos> {
    let level = world.state().current();
    let content = world.content();
    let mut los = BTreeSet::new();
    crate::fov::compute_fov(
        world.player_pos(),
        crate::perception::VIEW_RADIUS,
        |p| level.is_opaque(content, p),
        |p| {
            if level.terrain.in_bounds(p) {
                los.insert(p);
            }
        },
    );
    los
}

/// Wall-completion invariants: every opaque cell bounding a transparent cell seen by eye (within
/// view range) is visible; nothing else outside line of sight is; creatures only in line of sight.
pub(super) fn check_wall_completion(world: &World) -> Result<(), TestCaseError> {
    let view = world.perception();
    let level = world.state().current();
    let content = world.content();
    let los = line_of_sight(world);
    let viewer = view.viewer();
    let opaque = |p: Pos| level.is_opaque(content, p);
    let seen_by_eye = |p: Pos| {
        view.light(p)
            .is_some_and(|l| view.vision().sees(l.light(), l.dist2))
    };
    for y in 0..view.height() {
        for x in 0..view.width() {
            let p = Pos::new(x, y);
            let bounds_seen = opaque(p)
                && p.dist2(viewer) <= crate::fov::radius2(crate::perception::VIEW_RADIUS)
                && Dir::ALL
                    .iter()
                    .any(|d| !opaque(p.step(*d)) && seen_by_eye(p.step(*d)));
            if bounds_seen {
                prop_assert!(view.is_visible(p), "wall {} bounds seen floor", p);
            }
            if view.is_visible(p) && !los.contains(&p) {
                prop_assert!(bounds_seen, "{} is visible outside line of sight", p);
            }
        }
    }
    for actor in view.actors() {
        prop_assert!(
            los.contains(&actor.pos),
            "creature shown outside line of sight"
        );
    }
    Ok(())
}

#[test]
fn walls_bounding_far_floor_seen_through_a_gap_are_visible() {
    // A corridor that jogs down twice. From the start, the torch-lit floor in the middle section
    // is seen through the first jog, but the walls above and below it are not in line of sight.
    let map = "\
########################################
#@..........############################
###########.........T...################
#######################.......r........#
########################################
";
    let w = dark_world(map, "normal");
    let view = w.perception();
    let los = line_of_sight(&w);
    let far_floor = Pos::new(20, 2);
    assert!(visible(&view, far_floor), "the lit far floor is seen");
    for wall in [Pos::new(20, 1), Pos::new(20, 3)] {
        assert!(!los.contains(&wall), "{wall} is not in line of sight");
        assert!(
            visible(&view, wall),
            "{wall} bounds seen floor, so it is shown"
        );
        // Wall-face light: the torch at (20, 2) from the wall's own distance (d² = 1), plus the
        // bounce light of the adjacent visible floor.
        let base = view.base_light(wall).expect("visible");
        let torch = falloff(Light::new(255, 255, 255, 0), 4, 1);
        assert_eq!(
            view.light(wall).map(|l| l.light()),
            Some(base.saturating_add(torch))
        );
    }
    // Nothing beyond those walls: not the lit floor behind them, not the rat standing there.
    for hidden in [Pos::new(20, 0), Pos::new(24, 3), Pos::new(30, 3)] {
        assert!(!visible(&view, hidden), "{hidden} is behind a wall");
    }
    assert!(view.actor_at(Pos::new(30, 3)).is_none());
    // Completed walls are remembered like any seen terrain.
    assert!(w.state().current().memory[Pos::new(20, 1)].is_some());
    check_wall_completion(&w).expect("wall completion");
    check_invariants(&w).expect("light invariants");
}

#[test]
fn wall_completion_needs_seen_floor_not_just_the_players_cell() {
    // In total darkness the player sees only their own cell, not the walls around it.
    let w = dark_world("#####\n#@..#\n#####\n", "normal");
    let view = w.perception();
    assert!(!visible(&view, Pos::new(1, 0)));
    assert!(!visible(&view, Pos::new(0, 1)));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Random play in the showcase hub keeps every light invariant, and the cached masks always
    /// agree with a cold recomputation.
    #[test]
    fn light_invariants_hold_during_random_play(
        steps in prop::collection::vec(0usize..12, 1..40),
    ) {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        let mut w = World::new(content, "showcase", None).expect("showcase");
        let profiles = ["normal", "low_light", "darkvision", "uv_sight"];
        for step in steps {
            let action = match step {
                0..=7 => Action::Move(Dir::ALL[step]),
                8 => Action::ToggleLantern,
                9 => Action::SetVision(profiles[w.turn() as usize % 4].into()),
                10 => Action::Open(Dir::ALL[w.turn() as usize % 8]),
                _ => Action::Close(Dir::ALL[w.turn() as usize % 8]),
            };
            w.apply(&action);
        }
        check_invariants(&w)?;
        check_wall_completion(&w)?;
    }

    /// Random caves split by a solid wall, lights (glaring ones too) only east of it, the player
    /// west of it: no light of any kind — direct, wall face, or bounce — is seen.
    #[test]
    fn no_light_is_seen_through_a_solid_wall(
        cells in prop::collection::vec(0u8..100, 22 * 9),
        vision in prop::sample::select(vec!["normal", "low_light", "darkvision", "uv_sight"]),
    ) {
        let mut map = format!("{}\n", "#".repeat(24));
        for y in 0..9 {
            map.push('#');
            for x in 0..22 {
                map.push(match (x, y, cells[y * 22 + x]) {
                    (0, 4, _) => '@',
                    (10, _, _) => '#',
                    (_, _, 0..=29) => '#',
                    (11.., _, 30..=34) => 'T',
                    (11.., _, 35..=37) => 'G',
                    (11.., _, 38..=39) => 'R',
                    _ => '.',
                });
            }
            map.push_str("#\n");
        }
        map.push_str(&"#".repeat(24));
        map.push('\n');
        let w = dark_world(&map, vision);
        let view = w.perception();
        for y in 0..view.height() {
            for x in 0..view.width() {
                let p = Pos::new(x, y);
                if let Some(light) = view.light(p) {
                    prop_assert!(light.light().is_dark(), "light at {}", p);
                    prop_assert_eq!(view.base_light(p), Some(Light::DARK));
                }
            }
        }
        prop_assert!(view.light_sources().is_empty());
        check_invariants(&w)?;
    }

    /// Random caves (walls, torches, a rat), any profile and ambient level: wall completion shows
    /// exactly the walls bounding seen floor, and the light invariants hold.
    #[test]
    fn wall_completion_holds_in_random_caves(
        cells in prop::collection::vec(0u8..100, 22 * 9),
        vision in prop::sample::select(vec!["normal", "low_light", "darkvision", "uv_sight"]),
        ambient in prop::sample::select(vec![0u16, 20, 255]),
    ) {
        let mut map = format!("{}\n", "#".repeat(24));
        for y in 0..9 {
            map.push('#');
            for x in 0..22 {
                map.push(match (x, y, cells[y * 22 + x]) {
                    (0, 4, _) => '@',
                    (_, _, 0..=41) => '#',
                    (_, _, 42..=44) => 'T',
                    (_, _, 45) => 'r',
                    (_, _, 46) => 'G',
                    _ => '.',
                });
            }
            map.push_str("#\n");
        }
        map.push_str(&"#".repeat(24));
        map.push('\n');
        let w = world_with(&map, [ambient, ambient, ambient, 0], vision);
        check_wall_completion(&w)?;
        check_invariants(&w)?;
    }
}
