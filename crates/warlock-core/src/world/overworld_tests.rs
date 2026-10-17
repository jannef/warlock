//! The overworld (spec §3, §5, §8): travel time per terrain, impassable terrain, locations,
//! local areas entered with `>` and left by their edges, persistence, sight by daylight, saves,
//! and replays.

use super::*;
use crate::content::{ContentFiles, LevelKind, local_area_id};
use crate::daylight::{SIGHT_DAY, SIGHT_NIGHT, SIGHT_TWILIGHT};
use crate::fov::radius2;
use crate::grid::Grid;
use crate::replay::Replay;
use std::collections::VecDeque;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// The `world` scenario: on the road beside the Hall at 08:00.
fn world() -> World {
    World::new(CONTENT.clone(), "world", None).expect("world")
}

/// Built-in content with the overworld replaced by `map` (legend as in the real overworld; `@`
/// is the start on a road, `H` the Hall leading to the hub).
fn with_overworld(map: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/overworld.toml",
        &format!(
            "name = \"Test Wilds\"\nkind = \"overworld\"\nmap = '''\n{map}'''\n[legend]\n\
             \".\" = \"plains\"\n'\"' = \"grassland\"\n\"f\" = \"forest\"\n\
             \"F\" = \"dense_forest\"\n\"h\" = \"hills\"\n\"M\" = \"mountains\"\n\
             \"s\" = \"swamp\"\n\"~\" = {{ terrain = \"river\", flow = \"S\" }}\n\"w\" = \"lake\"\n\"=\" = \"road\"\n\
             \"b\" = \"bridge\"\n\"H\" = {{ terrain = \"hall\", to = \"hub\" }}\n\
             \"@\" = {{ terrain = \"road\", spawn = \"player\" }}\n"
        ),
    );
    Arc::new(Content::load(&files).expect("test content loads"))
}

fn last_message(world: &World) -> String {
    world
        .messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

fn terrain_id(world: &World) -> String {
    let (level, _) = world.ground_truth();
    world
        .content()
        .terrain(level.terrain[world.player_pos()])
        .id
        .clone()
}

/// The next step towards `target` along a shortest 8-way path over walkable terrain.
fn step_towards(world: &World, target: Pos) -> Dir {
    let level = world.state.current();
    let mut dist = Grid::new(level.width(), level.height(), u32::MAX);
    dist[target] = 0;
    let mut queue = VecDeque::from([target]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let q = p.step(dir);
            if level.is_walkable(&world.content, q) && dist[q] == u32::MAX {
                dist[q] = dist[p] + 1;
                queue.push_back(q);
            }
        }
    }
    let here = world.player_pos();
    Dir::ALL
        .into_iter()
        .filter(|d| dist.get(here.step(*d)).is_some_and(|n| *n < dist[here]))
        .min_by_key(|d| dist[here.step(*d)])
        .expect("a way to the target")
}

/// Walks to `target`, recording the actions.
fn walk_to(world: &mut World, target: Pos, log: &mut Vec<Action>) {
    for _ in 0..500 {
        if world.player_pos() == target {
            return;
        }
        let action = Action::Move(step_towards(world, target));
        assert!(world.apply(&action).took_turn, "{}", last_message(world));
        log.push(action);
    }
    panic!("could not reach {target}");
}

/// Walks `dir` until the level changes (off the edge of a local area); returns the steps taken.
fn leave(world: &mut World, dir: Dir) -> i32 {
    let id = world.perception().level_id().to_string();
    let mut steps = 0;
    while world.perception().level_id() == id {
        assert!(
            world.apply(&Action::Move(dir)).took_turn,
            "{}",
            last_message(world)
        );
        steps += 1;
        assert!(steps <= 100, "never reached the edge going {dir}");
    }
    steps
}

/// The overworld cell of the location leading to `to`.
fn location(world: &World, to: &str) -> Pos {
    let overworld = world.content.overworld().expect("an overworld");
    overworld
        .stairs
        .iter()
        .find(|s| s.to.as_deref() == Some(to))
        .map(|s| s.pos)
        .expect("a location")
}

#[test]
fn the_world_scenario_starts_on_the_overworld_beside_the_hall() {
    let world = world();
    let view = world.perception();
    assert_eq!(view.level_id(), "overworld");
    assert_eq!(view.level_kind(), LevelKind::Overworld);
    assert_eq!(view.level_name(), "The Wildlands");
    assert_eq!(view.region(), Some("Greenmarch"));
    assert_eq!(terrain_id(&world), "road");
    let hall = location(&world, "hub");
    assert_eq!(world.player_pos().chebyshev(hall), 2);
    assert_eq!(world.state.current().turn(), Duration::minutes(10));
}

#[test]
fn every_walkable_overworld_cell_has_a_region_and_every_location_a_level() {
    let content = &*CONTENT;
    let overworld = content.overworld().expect("an overworld");
    for y in 0..overworld.terrain.height() {
        for x in 0..overworld.terrain.width() {
            let p = Pos::new(x, y);
            if content.terrain(overworld.terrain[p]).walkable {
                assert!(overworld.region_at(p).is_some(), "{p} has no region");
            }
        }
    }
    let mut targets: Vec<_> = overworld
        .stairs
        .iter()
        .map(|s| s.to.as_deref().expect("locations lead somewhere"))
        .collect();
    targets.sort_unstable();
    assert_eq!(targets, ["depths:1", "hub", "vast"]);
}

#[test]
fn a_step_takes_the_travel_time_of_the_terrain_stepped_onto() {
    let content = with_overworld(
        "MMMMMMMM\n\
         M.fFhs=M\n\
         M\"@~bwHM\n\
         MMMMMMMM\n",
    );
    let mut world = World::new(content, "world", None).expect("world");
    let minutes = |m| Duration::minutes(m);
    for (dir, terrain, took) in [
        (Dir::N, "forest", minutes(90)),
        (Dir::W, "plains", minutes(45)),
        (Dir::S, "grassland", minutes(50)),
        (Dir::E, "road", minutes(20)),
        (Dir::NE, "dense_forest", minutes(150)),
        (Dir::E, "hills", minutes(120)),
        (Dir::E, "swamp", minutes(150)),
        (Dir::SW, "bridge", minutes(20)),
        (Dir::NE, "swamp", minutes(150)),
        (Dir::E, "road", minutes(20)),
        (Dir::S, "hall", minutes(30)),
    ] {
        let outcome = world.apply(&Action::Move(dir));
        assert_eq!(terrain_id(&world), terrain, "{dir}");
        assert_eq!(outcome.elapsed, took, "{terrain}");
        assert!(outcome.took_turn);
    }
    assert_eq!(world.time().hour(), 8 + 14);
    assert_eq!(last_message(&world), "The way into The Hub is here.");

    // Waiting and resting go by the overworld's ten-minute turn.
    let turn = world.turn();
    assert_eq!(world.apply(&Action::Wait).elapsed, minutes(10));
    let outcome = world.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(outcome.elapsed, Duration::hours(1));
    assert_eq!(world.turn(), turn + 7);
}

#[test]
fn mountains_and_open_water_are_impassable() {
    let content = with_overworld(
        "MMMMMMMM\n\
         M.fFhs=M\n\
         M\"@~bwHM\n\
         MMMMMMMM\n",
    );
    let mut world = World::new(content, "world", None).expect("world");
    for (dir, blocker) in [
        (Dir::E, "The river blocks your way."),
        (Dir::S, "The mountain range blocks your way."),
    ] {
        let start = (world.player_pos(), world.time());
        let outcome = world.apply(&Action::Move(dir));
        assert!(!outcome.took_turn && outcome.elapsed.is_zero());
        assert_eq!((world.player_pos(), world.time()), start);
        assert_eq!(last_message(&world), blocker);
    }
    // Over the bridge to the Hall, then try the deep water beside it.
    for dir in [Dir::NE, Dir::E, Dir::E, Dir::E, Dir::S] {
        assert!(world.apply(&Action::Move(dir)).took_turn);
    }
    assert_eq!(terrain_id(&world), "hall");
    assert!(!world.apply(&Action::Move(Dir::W)).took_turn);
    assert_eq!(last_message(&world), "The deep water blocks your way.");
}

#[test]
fn locations_lead_into_their_levels_and_back_out_to_the_location() {
    let mut world = world();
    let mut log = Vec::new();
    for (to, name) in [
        ("hub", "The Hub"),
        ("vast", "The Vast Caverns"),
        ("depths:1", "The Depths 1"),
    ] {
        let at = location(&world, to);
        walk_to(&mut world, at, &mut log);
        let short = name.trim_end_matches(" 1");
        assert_eq!(
            last_message(&world),
            format!("The way into {short} is here.")
        );
        let before = world.time();
        assert!(world.apply(&Action::Descend).took_turn);
        assert_eq!(world.time().since(before), Duration::minutes(10));
        assert_eq!(world.perception().level_id(), to);
        assert_eq!(last_message(&world), format!("You enter {name}."));
        let up = world
            .state
            .current()
            .stairs
            .iter()
            .find(|s| s.to.as_deref() == Some("overworld"))
            .expect("stairs back up to the overworld")
            .pos;
        assert_eq!(world.player_pos(), up, "arrive on the stairs leading out");
        assert_eq!(
            world.state.current().stairs_at(up).map(|s| s.dir),
            Some(StairsDir::Up)
        );

        let before = world.time();
        assert!(world.apply(&Action::Ascend).took_turn);
        assert_eq!(world.time().since(before), Duration::seconds(6));
        assert_eq!(world.perception().level_id(), "overworld");
        assert_eq!(world.player_pos(), at);
        assert_eq!(last_message(&world), "You climb out to The Wildlands.");
    }
}

#[test]
fn the_hub_leads_up_to_the_overworld_hall() {
    let mut world = World::new(CONTENT.clone(), "showcase", None).expect("world");
    let up = world
        .state
        .current()
        .stairs
        .iter()
        .find(|s| s.to.as_deref() == Some("overworld"))
        .expect("the stairs nook leads up")
        .pos;
    let mut log = Vec::new();
    // Open the doors on the way (bumping opens them).
    for _ in 0..200 {
        if world.player_pos() == up {
            break;
        }
        let level = world.state.current();
        let content = world.content.clone();
        let passable = |p: Pos| {
            level.terrain.get(p).is_some_and(|t| {
                let def = content.terrain(*t);
                def.walkable || def.is_closed_door()
            })
        };
        let mut dist = Grid::new(level.width(), level.height(), u32::MAX);
        dist[up] = 0;
        let mut queue = VecDeque::from([up]);
        while let Some(p) = queue.pop_front() {
            for dir in Dir::ALL {
                let q = p.step(dir);
                if passable(q) && dist[q] == u32::MAX {
                    dist[q] = dist[p] + 1;
                    queue.push_back(q);
                }
            }
        }
        let here = world.player_pos();
        let dir = Dir::ALL
            .into_iter()
            .filter(|d| {
                let q = here.step(*d);
                dist.get(q).is_some_and(|n| *n < dist[here])
                    && world.state.actor_at(&level.id, q).is_none()
            })
            .min_by_key(|d| dist[here.step(*d)]);
        let action = dir.map_or(Action::Wait, Action::Move);
        world.apply(&action);
        log.push(action);
    }
    assert_eq!(world.player_pos(), up);
    assert!(world.apply(&Action::Ascend).took_turn);
    assert_eq!(world.perception().level_id(), "overworld");
    assert_eq!(world.player_pos(), location(&world, "hub"));
}

#[test]
fn descending_elsewhere_enters_the_cells_local_area_and_its_edges_lead_back() {
    let mut world = world();
    let start = world.player_pos();
    assert!(world.apply(&Action::Move(Dir::E)).took_turn);
    let cell = world.player_pos();
    let before = world.time();
    assert!(world.apply(&Action::Descend).took_turn);
    assert_eq!(world.time().since(before), Duration::minutes(10));
    let id = local_area_id(cell);
    let view = world.perception();
    assert_eq!(view.level_id(), id);
    assert_eq!(view.level_kind(), LevelKind::Outdoor);
    assert_eq!(view.level_name(), "Road");
    assert_eq!(view.region(), Some("Greenmarch"));
    assert_eq!(last_message(&world), "You explore the road.");
    let (w, h) = crate::mapgen::local::SIZE;
    assert_eq!((view.width(), view.height()), (w, h));
    assert_eq!(world.player_pos(), Pos::new(w / 2, h / 2));
    assert_eq!(world.state.current().turn(), Duration::seconds(6));
    assert_eq!(
        world.apply(&Action::Move(Dir::N)).elapsed,
        Duration::seconds(6)
    );

    // West to the edge, then one more step leaves.
    assert_eq!(leave(&mut world, Dir::W), w / 2 + 1);
    assert_eq!(world.perception().level_id(), "overworld");
    assert_eq!(world.player_pos(), cell);
    assert_eq!(last_message(&world), "You return to The Wildlands.");

    // Every edge leads back, diagonally too.
    for dir in [Dir::N, Dir::E, Dir::S, Dir::SE, Dir::NW] {
        assert!(world.apply(&Action::Descend).took_turn);
        assert_eq!(world.perception().level_id(), id);
        leave(&mut world, dir);
        assert_eq!(world.player_pos(), cell, "{dir}");
    }
    assert!(world.apply(&Action::Move(Dir::W)).took_turn);
    assert_eq!(world.player_pos(), start);
}

#[test]
fn local_areas_persist_per_overworld_cell() {
    let mut world = world();
    let cell = world.player_pos();
    assert!(world.apply(&Action::Descend).took_turn);
    let id = local_area_id(cell);
    for _ in 0..5 {
        world.apply(&Action::Move(Dir::E));
    }
    let remembered = world.state.levels[&id].memory.clone();
    assert!(remembered.cells().iter().any(Option::is_some));
    leave(&mut world, Dir::S);
    assert_eq!(world.perception().level_id(), "overworld");
    // A neighboring cell has its own area; the first one is kept as it was.
    assert!(world.apply(&Action::Move(Dir::W)).took_turn);
    let other = local_area_id(world.player_pos());
    assert!(world.apply(&Action::Descend).took_turn);
    assert_eq!(world.perception().level_id(), other);
    assert_ne!(other, id);
    assert_eq!(world.state.levels[&id].memory, remembered);
    leave(&mut world, Dir::N);
    assert!(world.apply(&Action::Move(Dir::E)).took_turn);
    assert!(world.apply(&Action::Descend).took_turn);
    assert_eq!(world.perception().level_id(), id);
    let (w, h) = crate::mapgen::local::SIZE;
    assert_eq!(
        world.player_pos(),
        Pos::new(w / 2, h / 2),
        "back at the start"
    );
    // Same area as generated on the first visit, memory included.
    let generated = crate::mapgen::generate(&CONTENT, world.seed(), &id, None).expect("generates");
    assert_eq!(world.state.current().terrain, generated.terrain);
    assert!(
        world.state.levels[&id]
            .memory
            .cells()
            .iter()
            .zip(remembered.cells())
            .all(|(now, then)| then.is_none() || now == then)
    );
}

#[test]
fn some_cells_have_nothing_to_explore_and_no_way_up() {
    let content = with_overworld("MMMMM\nM@bHM\nMMMMM\n");
    let mut world = World::new(content, "world", None).expect("world");
    assert!(!world.apply(&Action::Ascend).took_turn);
    assert_eq!(last_message(&world), "You see no way up here.");
    assert!(world.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(terrain_id(&world), "bridge");
    let before = world.time();
    assert!(!world.apply(&Action::Descend).took_turn);
    assert_eq!(world.time(), before);
    assert_eq!(last_message(&world), "There is nothing to explore here.");
    assert!(
        world
            .state
            .levels
            .keys()
            .all(|id| !id.starts_with("local:"))
    );
}

/// Cells visible from the player, as squared distances, for a sight test field.
fn visible(world: &World) -> Vec<(Pos, i32)> {
    let view = world.perception();
    let mut cells = Vec::new();
    for y in 0..view.height() {
        for x in 0..view.width() {
            let p = Pos::new(x, y);
            if view.is_visible(p) {
                cells.push((p, p.dist2(view.viewer())));
            }
        }
    }
    cells
}

#[test]
fn overworld_sight_reaches_as_far_as_daylight_and_mountains_block_it() {
    // An open field; a mountain two cells east of the player and dense forest two cells west.
    let mut map = String::new();
    for y in 0..21 {
        let row: String = (0..25)
            .map(|x| match (x, y) {
                (12, 10) => '@',
                (14, 10) => 'M',
                (10, 10) => 'F',
                _ => '"',
            })
            .collect();
        map.push_str(&row);
        map.push('\n');
    }
    let mut world = World::new(with_overworld(&map), "world", None).expect("world");
    let viewer = world.player_pos();
    let at = |dx: i32, dy: i32| Pos::new(viewer.x + dx, viewer.y + dy);
    // 08:00 is day; skip to noon, dusk, and night.
    for (skip, radius) in [
        (Duration::hours(4), SIGHT_DAY),
        (Duration::hours(6), SIGHT_TWILIGHT),
        (Duration::hours(4), SIGHT_NIGHT),
    ] {
        world.apply(&Action::SkipTime(skip));
        let cells = visible(&world);
        let reach = cells
            .iter()
            .map(|(_, d2)| *d2)
            .max()
            .expect("the player sees");
        assert!(reach <= radius2(radius), "radius {radius}: {reach}");
        let view = world.perception();
        assert!(view.is_visible(at(0, -radius)), "radius {radius}");
        assert!(!view.is_visible(at(0, -radius - 1)), "radius {radius}");
        // Light does not matter: even at night everything within the radius is seen.
        assert!(view.is_visible(at(-1, -1)) && view.is_visible(at(1, 1)));
    }
    world.apply(&Action::SkipTime(Duration::hours(14)));
    assert_eq!(world.time().hour(), 12);
    let view = world.perception();
    assert!(view.is_visible(at(2, 0)), "the mountain itself is seen");
    assert!(!view.is_visible(at(4, 0)), "but not what lies behind it");
    assert!(
        view.is_visible(at(-2, 0)),
        "the dense forest's edge is seen"
    );
    assert!(!view.is_visible(at(-4, 0)), "but not into it");
    assert!(view.is_visible(at(0, 8)));
}

#[test]
fn the_sky_lights_the_overworld_and_local_areas_by_day_only() {
    let mut world = world();
    let ambient = |world: &World| world.perception().ambient();
    let view = world.perception();
    assert!(
        view.sun().is_some_and(|s| s.is_up()),
        "the sun is up at 08:00"
    );
    let here = view.light(view.viewer()).expect("own cell").brightness();
    assert!(here > 150, "sun and sky light the overworld: {here}");
    assert!(ambient(&world).is_dark(), "no night light by day");
    world.apply(&Action::SkipTime(Duration::hours(14)));
    assert_eq!(world.time().hour(), 22);
    assert!(world.perception().sun().is_some_and(|s| s.is_night()));
    // The overworld keeps a faint night light (only for drawing it; sight ignores light there).
    assert_eq!(ambient(&world), crate::daylight::OVERWORLD_NIGHT);
    // Even darkvision does not see farther on the overworld at night, nor marks cells dim.
    world.apply(&Action::SetVision("darkvision".into()));
    let view = world.perception();
    let cells = visible(&world);
    assert!(cells.iter().all(|(_, d2)| *d2 <= radius2(SIGHT_NIGHT)));
    assert!(
        cells
            .iter()
            .all(|(p, _)| !view.light(*p).expect("visible").darkvision)
    );
    world.apply(&Action::SetVision("normal".into()));

    // A local area at night is dark: only a lantern shows anything.
    assert!(world.apply(&Action::Descend).took_turn);
    assert!(ambient(&world).is_dark());
    let next_to = world.player_pos().step(Dir::E);
    assert!(!world.perception().is_visible(next_to));
    assert!(world.apply(&Action::ToggleLantern).took_turn);
    assert!(world.perception().is_visible(next_to));
    world.apply(&Action::ToggleLantern);
    world.apply(&Action::SkipTime(Duration::hours(12)));
    assert_eq!(world.time().hour(), 10);
    assert!(
        world
            .perception()
            .is_visible(next_to.step(Dir::E).step(Dir::E))
    );
}

/// A tour: road and grass, a local area left by its edge, the Hall and back, and a rest.
fn tour() -> Vec<Action> {
    let parse = |text: &str| crate::action::parse_actions(text).expect("actions parse");
    let mut actions = parse("move:E move:SE descend move:N move:W lantern rest:1h move:S");
    actions.extend(std::iter::repeat_n(Action::Move(Dir::W), 43));
    actions.extend(parse("move:NW descend ascend move:E rest:dusk"));
    actions
}

#[test]
fn overworld_replays_verify_and_saves_continue_identically() {
    let actions = tour();
    let mut recorded = world();
    let mut replay = Replay::new(&recorded);
    for action in &actions {
        recorded.apply(action);
        replay.push(action, &recorded);
    }
    assert_eq!(recorded.perception().level_id(), "overworld");
    assert!(
        recorded
            .state
            .levels
            .keys()
            .any(|id| id.starts_with("local:"))
    );
    assert!(recorded.state.levels.contains_key("hub"));
    let parsed = Replay::from_json(&replay.to_json()).expect("parses");
    let replayed = parsed.run(CONTENT.clone()).expect("verifies");
    assert_eq!(replayed.state_hash(), recorded.state_hash());

    // Save inside a local area, load, and continue: identical to never saving.
    let (first, rest) = actions.split_at(5);
    let mut a = world();
    for action in first {
        a.apply(action);
    }
    assert!(a.perception().level_id().starts_with("local:"));
    let mut b = crate::save::decode(&crate::save::encode(&a), CONTENT.clone()).expect("decodes");
    assert_eq!(a.state(), b.state());
    for action in rest {
        a.apply(action);
        b.apply(action);
    }
    assert_eq!(a.state(), b.state());
    assert_eq!(a.state_hash(), recorded.state_hash());
}

#[test]
fn a_run_may_start_in_a_local_area() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "scenarios/meadow.toml",
        "name = \"Meadow\"\nstart_level = \"local:49,23\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("loads"));
    let mut world = World::new(content, "meadow", None).expect("starts");
    assert_eq!(world.perception().level_id(), "local:49,23");
    leave(&mut world, Dir::S);
    assert_eq!(world.perception().level_id(), "overworld");
    assert_eq!(world.player_pos(), Pos::new(49, 23));
}
