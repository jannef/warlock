//! The endless branch (`endless.rs`; spec §7 "The endless branch"): depth and danger, floors made
//! anew by every staircase, what crosses and what is gone, the save, and replays.

use crate::action::Action;
use crate::character::CharacterSpec;
use crate::content::Generator;
use crate::dump::{self, Layer};
use crate::mapgen;
use crate::replay::Replay;
use crate::test_support::{CONTENT, run, said};
use crate::world::*;
use proptest::prelude::*;

const BRANCH: &str = "grundleas";

fn floor(depth: u32) -> String {
    format!("{BRANCH}:{depth}")
}

fn start(seed: u64) -> World {
    World::new(CONTENT.clone(), BRANCH, Some(seed)).expect("world")
}

/// A ranger of `race`: they come with a companion.
fn ranger(seed: u64) -> World {
    let picks = vec!["keen_ears".into(), "lean".into(), "healthy".into()];
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let spec = spec.with_talents(picks);
    World::start_run(CONTENT.clone(), BRANCH, Some(seed), Some(&spec), None, true).expect("world")
}

/// The way `dir` off the current level.
fn stairs(world: &World, dir: StairsDir) -> crate::content::StairsLink {
    (world.state.current().stairs.iter())
        .find(|s| s.dir == dir)
        .expect("stairs")
        .clone()
}

/// The player, set on the staircase `dir`, takes it. Whoever stands there is moved aside.
fn take(world: &mut World, dir: StairsDir) {
    take_with(world, dir, |_| {});
}

/// [`take`], with `before` run once the player stands on the stairs, ready to go.
fn take_with(world: &mut World, dir: StairsDir, before: impl FnOnce(&mut World)) {
    open_lair(world);
    let at = stairs(world, dir).pos;
    let player = world.state.player;
    let here = world.state.current_level.clone();
    (world.state.actors).retain(|id, a| *id == player || a.level != here || a.pos != at);
    // Deep floors kill a first-level character, and first arrivals bring levels with their
    // choices: these tests are about the stairs.
    run(world, "levelup:auto");
    world.state.death = None;
    let content = world.content.clone();
    let me = world.state.actors.get_mut(&player).expect("player");
    me.pos = at;
    me.energy = me.energy.max(0);
    me.hp = me.max_hp(&content);
    me.effects = Default::default();
    before(world);
    let action = match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    };
    let changed = world.apply(&action).events;
    assert!(
        (changed.iter()).any(|e| matches!(e, Event::LevelChanged { .. })),
        "{:?}",
        said(world).last()
    );
}

/// On a layer boss's floor (`layer_boss.rs`), the boss dies, so the way down opens: these tests
/// are about the stairs.
fn open_lair(world: &mut World) {
    let hidden: Vec<String> = (world.state.current().hidden.values())
        .map(|h| h.id.clone())
        .collect();
    let here = world.state.current_level.clone();
    let bosses: Vec<ActorId> = (world.state.actors.values())
        .filter(|a| a.level == here && hidden.contains(&world.content.creature(a.creature).id))
        .map(|a| a.id)
        .collect();
    let (player, now) = (world.state.player, world.state.time);
    for boss in bosses {
        world.dies(boss, Some(player), None, now, &mut Vec::new());
    }
}

/// The floors of the branch that exist now.
fn floors(world: &World) -> Vec<&String> {
    (world.state.levels.keys())
        .filter(|id| world.content.is_endless_floor(id))
        .collect()
}

/// The player arrives on floor `depth` directly, as if by a staircase.
fn jump(world: &mut World, depth: u32) {
    world.travel(&floor(depth), None).expect("travel");
    let to = floor(depth);
    world.forget_floors(&[Event::LevelChanged {
        from: String::new(),
        to,
    }]);
}

#[test]
fn depth_and_danger_go_up_one_a_floor_without_end() {
    let mut w = start(1);
    for depth in 1..=25 {
        assert_eq!(w.state.current_level, floor(depth));
        assert_eq!(
            CONTENT.floor_danger(&w.state.current_level, 1),
            depth,
            "danger is the depth"
        );
        assert_eq!(w.state.current().name, format!("Grundlēas {depth}"));
        take(&mut w, StairsDir::Down);
    }
    // Past anything a u8 holds, the number goes on.
    jump(&mut w, 300);
    assert_eq!(CONTENT.floor_danger(&w.state.current_level, 1), 300);
    assert_eq!(w.spawn_status().map(|s| s.0), Some(300));
}

#[test]
fn every_staircase_makes_a_new_floor_and_arrives_on_the_way_back() {
    let mut w = start(1);
    let first = w.state.current().terrain.clone();
    take(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, floor(2));
    assert_eq!(
        w.player_pos(),
        stairs(&w, StairsDir::Up).pos,
        "down: on an up stair"
    );
    let second = w.state.current().terrain.clone();
    take(&mut w, StairsDir::Down);
    take(&mut w, StairsDir::Up);
    assert_eq!(w.state.current_level, floor(2));
    assert_eq!(
        w.player_pos(),
        stairs(&w, StairsDir::Down).pos,
        "up: on a down stair"
    );
    assert_ne!(
        w.state.current().terrain,
        second,
        "not the second floor again"
    );
    take(&mut w, StairsDir::Up);
    assert_eq!(w.state.current_level, floor(1));
    assert_eq!(w.player_pos(), stairs(&w, StairsDir::Down).pos);
    assert_ne!(
        w.state.current().terrain,
        first,
        "not the first floor again"
    );
}

#[test]
fn up_from_the_first_floor_leaves_to_the_overworld_and_the_way_in_makes_a_new_one() {
    let mut w = start(2);
    // Reached from below, the first floor still climbs out.
    take(&mut w, StairsDir::Down);
    take(&mut w, StairsDir::Up);
    assert_eq!(stairs(&w, StairsDir::Up).to.as_deref(), Some("ascmearc"));
    let left = w.state.current().terrain.clone();
    take(&mut w, StairsDir::Up);
    assert_eq!(w.state.current_level, "ascmearc");
    assert_eq!(w.player_pos(), Pos::new(52, 52), "on the location");
    assert!(floors(&w).is_empty(), "no floor is left at all");
    assert!(said(&w).iter().any(|m| m.contains("climb out")));
    w.apply(&Action::Descend);
    assert_eq!(w.state.current_level, floor(1));
    assert_ne!(w.state.current().terrain, left);
    assert!(
        said(&w).iter().any(|m| m.contains("no stair goes twice")),
        "the words over the stair"
    );
}

#[test]
fn a_floor_left_is_gone_from_the_state_and_the_save() {
    let mut w = start(3);
    let keeps_it = |w: &World, depth: u32| {
        (w.state.actors.values()).any(|a| a.level == floor(depth))
            || w.state.levels.contains_key(&floor(depth))
    };
    take(&mut w, StairsDir::Down);
    assert!(!keeps_it(&w, 1), "nothing of it stays");
    assert_eq!(floors(&w), vec![&floor(2)]);
    // Nor does the light cache know it: the next floor of that id lights itself.
    take(&mut w, StairsDir::Up);
    assert_eq!(floors(&w), vec![&floor(1)]);
    w.state.check().expect("consistent");

    let loaded = crate::save::decode(&crate::save::encode(&w), CONTENT.clone()).expect("decodes");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(
        loaded.state.endless, w.state.endless,
        "the counter is saved"
    );
    let mut a = w.clone();
    let mut b = loaded;
    take(&mut a, StairsDir::Down);
    take(&mut b, StairsDir::Down);
    assert_eq!(
        a.state.current().terrain,
        b.state.current().terrain,
        "the next floor too"
    );
}

#[test]
fn replays_make_the_same_floors() {
    let mut w = start(4);
    let mut steps = Vec::new();
    for dir in [
        StairsDir::Down,
        StairsDir::Down,
        StairsDir::Up,
        StairsDir::Down,
    ] {
        take(&mut w, dir);
        steps.push(w.state.current().terrain.clone());
    }
    let mut again = start(4);
    for (i, dir) in [
        StairsDir::Down,
        StairsDir::Down,
        StairsDir::Up,
        StairsDir::Down,
    ]
    .into_iter()
    .enumerate()
    {
        take(&mut again, dir);
        assert_eq!(again.state.current().terrain, steps[i], "step {i}");
    }
    assert_eq!(again.state_hash(), w.state_hash());
    // A replay, through the player's own steps: the first floor starts on its up stair, and
    // every staircase arrives on the way back.
    let mut out = start(5);
    let mut replay = Replay::new(&out);
    for action in [
        Action::Ascend,
        Action::Descend,
        Action::Ascend,
        Action::Descend,
    ] {
        out.apply(&action);
        replay.push(&action, &out);
    }
    assert_eq!(out.state.current_level, floor(1));
    let verified = Replay::from_json(&replay.to_json()).expect("parses");
    let (played, diverged) = verified.play(CONTENT.clone()).expect("plays");
    assert!(diverged.is_none(), "{diverged:?}");
    assert_eq!(played.state_hash(), out.state_hash());
}

#[test]
fn a_deep_floor_still_holds_creatures_and_loot() {
    let mut w = start(6);
    for depth in [100, 200, 5000] {
        jump(&mut w, depth);
        let here = w.state.current_level.clone();
        let creatures = (w.state.actors.values())
            .filter(|a| a.level == here && a.id != w.state.player)
            .count();
        assert!(creatures >= 20, "depth {depth}: {creatures} creatures");
        assert!(
            !w.state.current().objects.is_empty(),
            "depth {depth}: nothing lies on the floor"
        );
        w.state.check().expect("consistent");
        // What comes later comes as well.
        assert!(w.attempt(true).is_some(), "depth {depth}: nothing comes");
    }
}

#[test]
fn the_roster_is_never_empty_at_any_depth_on_the_way_down() {
    // Every depth has something of its danger: no gap in the roster's levels, or past its top.
    let mut w = start(7);
    let top = CONTENT
        .dungeon(BRANCH)
        .and_then(|d| d.endless)
        .expect("endless")
        .roster_depth;
    assert!(top > 20 && top < u32::MAX, "the roster's top is {top}");
    for depth in 1..=top + 12 {
        jump(&mut w, depth);
        let here = w.state.current_level.clone();
        let creatures = (w.state.actors.values())
            .filter(|a| a.level == here && a.id != w.state.player)
            .count();
        assert!(creatures > 0, "depth {depth} holds no one");
    }
}

#[test]
fn the_generator_draw_gives_both_rooms_and_caves() {
    let def = CONTENT.dungeon(BRANCH).expect("the branch");
    assert_eq!(def.endless.map(|e| e.caves), Some(15));
    let (mut rooms, mut caves) = (0, 0);
    for made in 0..200 {
        let seed = super::endless::floor_seed(&CONTENT, 9, &floor(3), made);
        let id = floor(3);
        let t = mapgen::generate(&CONTENT, ("ascmearc", seed), &id, None).expect("a floor");
        match (t.terrain.width(), t.terrain.height()) {
            (80, 21) => rooms += 1,
            (100, 40) => caves += 1,
            other => panic!("a size of neither: {other:?}"),
        }
        assert_eq!(t.stairs.len(), 2, "a way up and a way down");
    }
    assert!(rooms > 140 && caves > 10, "{rooms} rooms, {caves} caves");
    assert_eq!(def.plan(3).map(|p| p.generator), Some(Generator::Rooms));
}

#[test]
fn only_the_first_arrival_at_a_depth_pays() {
    let mut w = start(8);
    let xp = |w: &World| w.state.experience;
    take(&mut w, StairsDir::Down);
    let once = xp(&w);
    assert!(once > 0, "the second floor pays");
    take(&mut w, StairsDir::Up);
    take(&mut w, StairsDir::Down);
    take(&mut w, StairsDir::Up);
    assert_eq!(xp(&w), once, "up and down the same stairs pay nothing more");
    take(&mut w, StairsDir::Down);
    take(&mut w, StairsDir::Down);
    assert!(xp(&w) > once, "a new depth does");
}

#[test]
fn a_companion_left_far_behind_still_crosses_and_an_ally_left_is_gone_with_word() {
    let mut w = ranger(10);
    let pet = w.companion().expect("a ranger's companion");
    run(&mut w, "ally:wolf");
    let wolf = (w.state.actors.values())
        .find(|a| a.master() == Some(w.state.player) && a.kept().is_none())
        .expect("the wolf")
        .id;
    // Both far from the stairs the player takes; the companion is told to stay.
    run(&mut w, "order:stay");
    let far = (w.state.current().terrain.iter())
        .filter(|(p, t)| {
            CONTENT.terrain(**t).walkable && w.state.actor_at(&w.state.current_level, *p).is_none()
        })
        .map(|(p, _)| p)
        .max_by_key(|p| p.chebyshev(stairs(&w, StairsDir::Down).pos))
        .expect("a far cell");
    for id in [pet, wolf] {
        w.state.actors.get_mut(&id).expect("ally").pos = far;
    }
    take(&mut w, StairsDir::Down);
    assert_eq!(
        w.state.actors[&pet].level,
        floor(2),
        "the companion crossed"
    );
    assert!(w.state.actors[&pet].kept().is_some(), "and is kept still");
    assert_eq!(w.companion(), Some(pet));
    assert!(
        !w.state.actors.contains_key(&wolf),
        "the wolf is gone with the floor"
    );
    assert!(
        said(&w).iter().any(|m| m.contains("will not see")),
        "and the player is told"
    );
    w.state.check().expect("consistent");
}

#[test]
fn an_ally_beside_the_player_crosses_with_them() {
    let mut w = start(11);
    run(&mut w, "ally:wolf");
    let wolf = (w.state.actors.values())
        .find(|a| a.master() == Some(w.state.player))
        .expect("the wolf")
        .id;
    let down = stairs(&w, StairsDir::Down).pos;
    let beside = Dir::ALL
        .iter()
        .map(|d| down.step(*d))
        .find(|p| w.state.current().is_walkable(&CONTENT, *p))
        .expect("a cell beside the stairs");
    w.state.actors.get_mut(&wolf).expect("wolf").pos = beside;
    take(&mut w, StairsDir::Down);
    assert_eq!(w.state.actors[&wolf].level, floor(2));
    assert!(!said(&w).iter().any(|m| m.contains("will not see")));
}

#[test]
fn a_unique_on_a_floor_left_may_come_again() {
    let mut w = start(12);
    let unique = CONTENT.creature_id("fire_drake").expect("a unique type");
    assert!(CONTENT.creature(unique).unique);
    let spawn = crate::content::CreatureSpawn {
        creature: unique,
        faction: None,
        alignment: None,
        npc: None,
        pos: w.player_pos(),
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let here = w.state.current_level.clone();
    let id = super::travel::make_creature(&mut w.state, &CONTENT, &here, &spawn);
    let name = CONTENT.creature(unique).id.clone();
    assert!(w.state.progress.uniques.contains(&name));
    take(&mut w, StairsDir::Down);
    assert!(!w.state.actors.contains_key(&id));
    assert!(
        !w.state.progress.uniques.contains(&name),
        "it never met its end"
    );
}

/// A creature of type `name` on a free walkable cell beside the player, as a fresh spawn.
fn put_beside(w: &mut World, name: &str) -> ActorId {
    let creature = CONTENT.creature_id(name).expect("a creature type");
    let me = w.player_pos();
    let pos = Dir::ALL
        .iter()
        .map(|d| me.step(*d))
        .find(|p| {
            w.state.current().is_walkable(&CONTENT, *p)
                && w.state.actor_at(&w.state.current_level, *p).is_none()
        })
        .expect("a free cell beside the player");
    let spawn = crate::content::CreatureSpawn {
        creature,
        faction: None,
        alignment: None,
        npc: None,
        pos,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let here = w.state.current_level.clone();
    super::travel::make_creature(&mut w.state, &CONTENT, &here, &spawn)
}

/// A kobold beside the player as they take the stairs `dir`; it is told of them.
fn follower_on(w: &mut World, dir: StairsDir) -> ActorId {
    let mut id = None;
    take_with(w, dir, |w| {
        let k = put_beside(w, "kobold");
        let me = w.player_pos();
        w.state.actors.get_mut(&k).expect("kobold").mind.last_seen = Some(me);
        id = Some(k);
    });
    id.expect("placed")
}

#[test]
fn a_hostile_beside_follows_down_and_up_and_the_floors_left_are_gone() {
    let mut w = start(20);
    let k = follower_on(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, floor(2));
    let me = w.player_pos();
    let kobold = &w.state.actors[&k];
    assert_eq!(kobold.level, floor(2), "it came down");
    assert_eq!(kobold.pos.chebyshev(me), 1, "and stands beside the player");
    assert_eq!(floors(&w), vec![&floor(2)], "floor 1 is forgotten");
    assert_eq!(kobold.home.as_deref(), Some(floor(1).as_str()));
    w.state.check().expect("consistent");
    // The kobold is beside them still: it comes up too, and floor 2 goes.
    let me = w.player_pos();
    let up = stairs(&w, StairsDir::Up).pos;
    assert_eq!(me, up);
    run(&mut w, "ascend");
    assert_eq!(w.state.current_level, floor(1));
    assert_eq!(w.state.actors[&k].level, floor(1), "it came up");
    assert_eq!(floors(&w), vec![&floor(1)]);
    w.state.check().expect("consistent");
}

#[test]
fn a_kill_of_a_follower_from_a_forgotten_floor_is_tallied_soundly() {
    let mut w = start(21);
    let k = follower_on(&mut w, StairsDir::Down);
    assert!(!w.state.levels.contains_key(&floor(1)));
    // Kill it where it stands: its home is the floor that is gone.
    w.state.actors.get_mut(&k).expect("kobold").hp = 1;
    let player = w.state.player;
    let (now, mut events) = (w.state.time, Vec::new());
    w.dies(k, Some(player), None, now, &mut events);
    assert!(!w.state.actors.contains_key(&k));
    let tally: Vec<_> = w.state.quests.kills().collect();
    assert_eq!(
        tally,
        vec![(floor(1).as_str(), "kobold", 1)],
        "counted under the old id, harmlessly"
    );
    // No quest names the branch; the readers take the stale key without a panic.
    run(&mut w, "wait*3");
    take(&mut w, StairsDir::Down);
    w.state.check().expect("consistent");
    let _ = dump::layer_rows(&w, Layer::Progress);
}

#[test]
fn a_vault_monster_that_follows_leaves_nothing_behind() {
    let mut w = start(22);
    let mut k = None;
    take_with(&mut w, StairsDir::Down, |w| {
        let id = put_beside(w, "kobold");
        let me = w.player_pos();
        w.state.actors.get_mut(&id).expect("kobold").mind.last_seen = Some(me);
        let here = w.state.current_level.clone();
        let level = w.state.levels.get_mut(&here).expect("level");
        let spawner = level.spawner.as_mut().expect("a spawner on a danger floor");
        spawner.outside.insert(id);
        k = Some(id);
    });
    let k = k.expect("placed");
    assert_eq!(w.state.actors[&k].level, floor(2));
    assert_eq!(floors(&w), vec![&floor(2)]);
    let spawner = w.state.current().spawner.as_ref().expect("spawner");
    assert!(
        !spawner.outside.contains(&k),
        "the new floor's spawner counts it"
    );
    w.state.check().expect("consistent");
}

#[test]
fn many_stair_uses_with_a_follower_keep_the_state_consistent() {
    let mut w = start(23);
    let mut k = follower_on(&mut w, StairsDir::Down);
    for i in 0..24 {
        let dir = if i % 3 == 2 {
            StairsDir::Up
        } else {
            StairsDir::Down
        };
        // Keep a follower beside the player: the one that came, or a new one.
        if !w.state.actors.contains_key(&k) {
            k = follower_on(&mut w, dir);
        } else {
            take_with(&mut w, dir, |w| {
                let me = w.player_pos();
                let here = w.state.current_level.clone();
                w.state.actors.set_level(k, &here).expect("k");
                let cell = Dir::ALL
                    .iter()
                    .map(|d| me.step(*d))
                    .find(|p| w.state.current().is_walkable(&CONTENT, *p));
                if let Some(cell) = cell {
                    w.state.actors.get_mut(&k).expect("k").pos = cell;
                }
            });
        }
        w.state.check().expect("consistent");
        assert!(floors(&w).len() <= 1, "{:?}", floors(&w));
        assert!(
            w.state
                .actors
                .values()
                .all(|a| w.state.levels.contains_key(&a.level)),
            "no actor on a floor that is gone"
        );
    }
}

#[test]
fn the_dump_and_every_other_branch_are_as_they_were() {
    // A run that never goes there keeps no state of it.
    let w = World::new(CONTENT.clone(), "ascmearc", Some(1)).expect("world");
    assert!(w.state.endless.is_empty());
    // The location shows no danger of its own, and a dump of the floor reads.
    let w = start(13);
    let text = dump::layer_rows(&w, Layer::Truth).join("\n");
    assert!(text.contains("danger 1 "), "{text}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// Many stair uses keep the state consistent and do not grow it.
    #[test]
    fn many_stair_uses_keep_the_state_small(seed in 0u64..1000, downs in proptest::collection::vec(any::<bool>(), 30)) {
        let mut w = start(seed);
        let mut depth = 1u32;
        let mut most = 0;
        for down in downs {
            if !down && depth == 1 {
                continue;
            }
            take(&mut w, if down { StairsDir::Down } else { StairsDir::Up });
            depth = if down { depth + 1 } else { depth - 1 };
            prop_assert!(w.state.check().is_ok());
            let here = floor(depth);
            prop_assert_eq!(floors(&w), vec![&here]);
            prop_assert_eq!(w.state.current_level.clone(), floor(depth));
            let on_others = (w.state.actors.values()).filter(|a| a.level != floor(depth)).count();
            prop_assert_eq!(on_others, 0);
            most = most.max(w.state.levels.len());
        }
        prop_assert!(most <= 1);
    }
}
