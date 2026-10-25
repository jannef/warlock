//! Travel through generated dungeon branches (`depths:N`): linking, persistence, determinism,
//! replays, and saves.

use super::*;
use crate::grid::Grid;
use crate::replay::Replay;
use std::collections::VecDeque;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn depths(seed: u64) -> World {
    World::new(CONTENT.clone(), "depths", Some(seed)).expect("world")
}

fn stairs(world: &World, dir: StairsDir) -> &crate::content::StairsLink {
    world
        .state
        .current()
        .stairs
        .iter()
        .find(|s| s.dir == dir)
        .expect("stairs")
}

/// The next action towards `target`: a step along a shortest 8-way path (bumping a closed door
/// opens it). Creatures never swap places, so when one blocks every such step, back away.
fn step_towards(world: &World, target: Pos) -> Action {
    let level = world.state.current();
    let content = &world.content;
    let passable = |p: Pos| {
        level.terrain.get(p).is_some_and(|t| {
            let def = content.terrain(*t);
            def.walkable || def.is_closed_door()
        })
    };
    let mut dist = Grid::new(level.width(), level.height(), u32::MAX);
    dist[target] = 0;
    let mut queue = VecDeque::from([target]);
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
    let free = |d: &Dir| {
        let q = here.step(*d);
        dist.get(q).is_some_and(|n| *n != u32::MAX) && world.state.actor_at(&level.id, q).is_none()
    };
    let closer = Dir::ALL
        .into_iter()
        .filter(|d| free(d) && dist[here.step(*d)] < dist[here])
        .min_by_key(|d| dist[here.step(*d)]);
    let away = || {
        Dir::ALL
            .into_iter()
            .filter(free)
            .max_by_key(|d| dist[here.step(*d)])
    };
    closer.or_else(away).map_or(Action::Wait, Action::Move)
}

/// Walks to `target`, recording the actions taken.
fn walk_to(world: &mut World, target: Pos, log: &mut Vec<Action>) {
    for _ in 0..3000 {
        if world.player_pos() == target {
            return;
        }
        let action = step_towards(world, target);
        world.apply(&action);
        log.push(action);
    }
    panic!("could not reach {target}");
}

/// Walks to the staircase in `dir` and takes it.
fn take(world: &mut World, dir: StairsDir, log: &mut Vec<Action>) {
    let target = stairs(world, dir).pos;
    walk_to(world, target, log);
    let action = match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    };
    assert!(
        world.apply(&action).took_turn,
        "{:?}",
        world.messages().iter().last()
    );
    log.push(action);
}

#[test]
fn the_vast_level_leads_into_the_depths_and_back() {
    let mut world = World::new(CONTENT.clone(), "vast", None).expect("world");
    let down = stairs(&world, StairsDir::Down).clone();
    assert_eq!(down.to.as_deref(), Some("depths:1"));
    let mut log = Vec::new();
    take(&mut world, StairsDir::Down, &mut log);
    assert_eq!(world.perception().level_id(), "depths:1");
    assert_eq!(world.perception().level_name(), "The Depths 1");
    let up = stairs(&world, StairsDir::Up).clone();
    assert_eq!(
        up.to.as_deref(),
        Some("vast"),
        "depth 1 leads back where we came from"
    );
    assert_eq!(
        world.player_pos(),
        up.pos,
        "arrive on the stairs leading back"
    );
    assert!(world.apply(&Action::Ascend).took_turn);
    assert_eq!(world.perception().level_id(), "vast");
    assert_eq!(world.player_pos(), down.pos);
}

#[test]
fn branch_levels_link_up_and_down_and_persist() {
    let mut world = depths(7);
    let mut log = Vec::new();
    let top = stairs(&world, StairsDir::Up).clone();
    assert_eq!(
        top.to, None,
        "a run that starts in the branch has nowhere to go up"
    );
    assert!(!world.apply(&Action::Ascend).took_turn);

    take(&mut world, StairsDir::Down, &mut log);
    assert_eq!(world.perception().level_id(), "depths:2");
    assert_eq!(
        stairs(&world, StairsDir::Up).to.as_deref(),
        Some("depths:1")
    );
    assert_eq!(world.player_pos(), stairs(&world, StairsDir::Up).pos);
    let depth2 = world.state.current().terrain.clone();

    take(&mut world, StairsDir::Up, &mut log);
    assert_eq!(world.perception().level_id(), "depths:1");
    assert_eq!(world.player_pos(), stairs(&world, StairsDir::Down).pos);
    let depth1 = world.state.levels["depths:1"].clone();

    take(&mut world, StairsDir::Down, &mut log);
    assert_eq!(
        world.state.current().terrain,
        depth2,
        "visited levels persist"
    );
    assert_eq!(world.state.levels["depths:1"].terrain, depth1.terrain);
    assert!(
        world.state.levels["depths:1"]
            .memory
            .cells()
            .iter()
            .any(Option::is_some),
        "and so does what the player remembers"
    );
}

#[test]
fn generated_levels_depend_only_on_seed_and_id() {
    let mut a = depths(11);
    let mut b = depths(11);
    let mut log = Vec::new();
    take(&mut a, StairsDir::Down, &mut log);
    for action in &log {
        b.apply(action);
    }
    assert_eq!(a.state(), b.state());
    let generated =
        crate::mapgen::generate(&CONTENT, ("showcase", 11), "depths:2", None).expect("generates");
    assert_eq!(a.state.levels["depths:2"].terrain, generated.terrain);
    assert_ne!(
        depths(12).state.current().terrain,
        depths(11).state.current().terrain
    );
}

#[test]
fn replays_through_generated_levels_verify() {
    let mut world = depths(3);
    let mut log = Vec::new();
    take(&mut world, StairsDir::Down, &mut log);
    take(&mut world, StairsDir::Down, &mut log);
    assert_eq!(world.perception().level_id(), "depths:3");

    let mut recorded = depths(3);
    let mut replay = Replay::new(&recorded);
    for action in &log {
        recorded.apply(action);
        replay.push(action, &recorded);
    }
    let parsed = Replay::from_json(&replay.to_json()).expect("parses");
    let replayed = parsed.run(CONTENT.clone()).expect("verifies");
    assert_eq!(replayed.state_hash(), world.state_hash());
}

#[test]
fn saves_round_trip_on_generated_levels_and_continue_identically() {
    let mut a = depths(5);
    let mut log = Vec::new();
    take(&mut a, StairsDir::Down, &mut log);
    let mut b = crate::save::decode(&crate::save::encode(&a), CONTENT.clone()).expect("decodes");
    assert_eq!(a.state(), b.state());
    assert_eq!(a.state.rng, b.state.rng, "the gameplay stream is saved");
    // Generating the next level after loading gives the same level as without the save.
    let mut more = Vec::new();
    take(&mut a, StairsDir::Down, &mut more);
    for action in &more {
        b.apply(action);
    }
    assert_eq!(a.state(), b.state());
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn saves_with_unknown_light_ids_are_rejected() {
    let world = depths(5);
    let mut state = world.state().clone();
    let level = state.levels.get_mut("depths:1").expect("level");
    level
        .lights
        .insert(Pos::new(1, 1), crate::content::LightId(999));
    assert!(matches!(
        World::from_state(CONTENT.clone(), state),
        Err(WorldError::InconsistentState(_))
    ));
}
