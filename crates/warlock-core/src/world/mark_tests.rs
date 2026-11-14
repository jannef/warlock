//! The mark (`mark.rs`; the openings design §3, #1461): whose guard the thief's openings are in.
//! A chosen attack sets, keeps, or empties; a free blow sets and never empties; the mark's death
//! carries the pool, with an opening more for the thief's own kill; a departure, a level change,
//! and the pool's decay to 0 clear it.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::ResourceId;
use crate::dump::{self, Layer};
use crate::object::Stack;
use crate::replay::Replay;
use crate::stats::{PerStat, Stats};
use crate::test_support::{CONTENT as BUILTIN, run};

/// A human thief in the showcase (seed 7), with their kit, every score 14, hit points to spare.
fn thief() -> World {
    let spec: CharacterSpec = "human,male,thief".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "healthy".into()]);
    let w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w.player_mut().hp_base = 10_000;
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

fn openings(world: &World) -> ResourceId {
    world.content().resource_id("openings").expect("openings")
}

fn held(world: &World) -> u16 {
    let id = openings(world);
    (world.state.player().resources.get(id)).map_or(0, |p| p.value)
}

fn set(world: &mut World, value: u16) {
    let id = openings(world);
    world.player_mut().resources.set(id, value);
}

fn mark(world: &World) -> Option<ActorId> {
    world.state.player().mark
}

/// How many times the player was told `text`.
fn told(world: &World, text: &str) -> usize {
    (world.messages().iter())
        .filter(|m| m.display().contains(text))
        .count()
}

/// A hostile creature of type `creature` placed `dir` of the player with hit points to spare;
/// its id.
fn beside(world: &mut World, creature: &str, dir: Dir) -> ActorId {
    let content = world.content().clone();
    let at = world.player_pos().step(dir);
    let template = (world.state.actors.values())
        .find(|a| a.id != world.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(world.state.next_actor_id);
    world.state.next_actor_id += 1;
    let creature = content.creature_id(creature).expect("a creature type");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        summoner: None,
        ally: None,
        risen: None,
        mark: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: 10_000,
        hp: 10_000,
        level: world.state.current_level.clone(),
        ..template
    };
    world.state.actors.insert(id, actor);
    id
}

/// The player attacks `foe`, put `dir` of them first (it may have stepped aside), by moving.
fn strike(world: &mut World, foe: ActorId, dir: Dir) {
    let at = world.player_pos().step(dir);
    let level = world.state.current_level.clone();
    if let Some(other) = (world.state.actor_at(&level, at)).map(|a| a.id)
        && other != foe
    {
        let away = world.state.actors[&foe].pos;
        world.state.actors.get_mut(&other).expect("there").pos = away;
    }
    world.state.actors.get_mut(&foe).expect("the foe").pos = at;
    run(world, &format!("move:{dir}"));
}

const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 2,
    kind: None,
};

#[test]
fn a_chosen_attack_sets_keeps_or_empties_and_says_so_once() {
    let mut w = thief();
    let east = beside(&mut w, "kobold", Dir::E);
    let west = beside(&mut w, "kobold", Dir::W);
    set(&mut w, 3);
    strike(&mut w, east, Dir::E);
    assert_eq!(mark(&w), Some(east), "no mark: the first foe struck is it");
    assert!(held(&w) >= 3, "the pool kept");
    strike(&mut w, east, Dir::E);
    assert_eq!(mark(&w), Some(east));
    assert!(held(&w) >= 3, "the mark struck again: nothing lost");
    strike(&mut w, west, Dir::W);
    // The pool emptied, the mark moved; with nothing built on the new one it lapses as the
    // turn's time passes.
    let moved = if held(&w) == 0 { None } else { Some(west) };
    assert_eq!(mark(&w), moved, "another living foe: the mark moves");
    assert!(
        held(&w) <= 1,
        "the pool emptied before the blow (its own build aside)"
    );
    assert_eq!(told(&w, "Your openings on the kobold are lost."), 1);
    // Back and forth with nothing held: no word of it.
    set(&mut w, 0);
    strike(&mut w, east, Dir::E);
    assert_eq!(told(&w, "are lost"), 1, "nothing to lose, nothing said");
    // A mark with no opening in its guard lapses as the turn's time passes.
    if held(&w) == 0 {
        assert_eq!(mark(&w), None);
    } else {
        assert_eq!(mark(&w), Some(east), "the blow's sneak attack built on it");
    }
}

#[test]
fn the_rule_itself_moves_and_empties_at_once() {
    let mut w = thief();
    let east = beside(&mut w, "kobold", Dir::E);
    let west = beside(&mut w, "kobold", Dir::W);
    set(&mut w, 4);
    w.choose_mark(east);
    assert_eq!((mark(&w), held(&w)), (Some(east), 4));
    w.choose_mark(east);
    assert_eq!((mark(&w), held(&w)), (Some(east), 4));
    w.choose_mark(west);
    assert_eq!((mark(&w), held(&w)), (Some(west), 0));
    assert_eq!(told(&w, "Your openings on the kobold are lost."), 1);
    // A creature without the pool never has a mark.
    let mut plain = thief();
    let id = openings(&plain);
    plain.player_mut().resources.remove(id);
    let foe = beside(&mut plain, "kobold", Dir::E);
    plain.choose_mark(foe);
    assert_eq!(mark(&plain), None);
}

#[test]
fn a_free_blow_sets_a_mark_but_never_moves_one_or_builds_on_a_stranger() {
    let mut w = thief();
    let player = w.state.player;
    let east = beside(&mut w, "kobold", Dir::E);
    let west = beside(&mut w, "kobold", Dir::W);
    set(&mut w, 2);
    w.free_blow_at(east);
    assert_eq!(
        mark(&w),
        Some(east),
        "none stood: the free blow's foe is it"
    );
    w.free_blow_at(west);
    assert_eq!(mark(&w), Some(east), "a free blow never moves it");
    assert_eq!(held(&w), 2, "nor empties the pool");
    // Its sneak attack on the stranger builds nothing; on the mark, an opening.
    w.ambushed((player, west), true, HIT);
    assert_eq!(held(&w), 2);
    w.ambushed((player, east), true, HIT);
    assert_eq!(held(&w), 3);
}

#[test]
fn a_shot_and_a_throw_are_chosen_and_a_burst_is_not() {
    let mut w = thief();
    let content = w.content().clone();
    // Two kobolds three and four cells east of the player, in the showcase's hall.
    let near = beside(&mut w, "kobold", Dir::E);
    let far = beside(&mut w, "kobold", Dir::E);
    let here = w.player_pos();
    w.state.actors.get_mut(&near).expect("kobold").pos = Pos::new(here.x + 3, here.y);
    w.state.actors.get_mut(&far).expect("kobold").pos = Pos::new(here.x + 4, here.y);
    // A throw at the cell a foe stands on makes it the mark.
    let dagger = content.object_id("dagger").expect("dagger");
    w.player_mut().inventory.push(Stack::new(dagger, 3));
    let item = w.state.player().inventory.len() - 1;
    set(&mut w, 2);
    let outcome = w.apply(
        &format!("throw:{item}:{},{}", here.x + 3, here.y)
            .parse()
            .expect("action"),
    );
    assert!(outcome.took_turn, "thrown");
    assert_eq!(mark(&w), Some(near), "a throw at it: the mark");
    // A fireball's burst on both moves nothing and empties nothing.
    let scroll = content.object_id("scroll_of_fireball").expect("scroll");
    w.player_mut().inventory.push(Stack::new(scroll, 1));
    let item = w.state.player().inventory.len() - 1;
    set(&mut w, 2);
    let at = w.state.actors[&far].pos;
    let outcome = w.apply(
        &format!("read:{item}:{},{}", at.x, at.y)
            .parse()
            .expect("action"),
    );
    assert!(outcome.took_turn, "read");
    assert_eq!(mark(&w), Some(near), "a burst is no chosen attack");
    assert!(held(&w) >= 2, "nor empties the pool");
    assert_eq!(told(&w, "are lost"), 0);
}

#[test]
fn the_marks_death_carries_the_pool_with_an_opening_for_the_thiefs_own_kill() {
    let mut w = thief();
    let player = w.state.player;
    let east = beside(&mut w, "kobold", Dir::E);
    let west = beside(&mut w, "kobold", Dir::W);
    set(&mut w, 2);
    w.choose_mark(east);
    let now = w.state.time;
    w.dies(east, Some(player), None, now, &mut Vec::new());
    assert_eq!(mark(&w), None, "the mark is cleared");
    assert_eq!(held(&w), 3, "the pool carries, and the kill gives one");
    // The next attack adopts the carried pool.
    w.choose_mark(west);
    assert_eq!((mark(&w), held(&w)), (Some(west), 3));
    // Killed by another hand (its fellow, an ally): carried, nothing more.
    let ally = beside(&mut w, "kobold", Dir::N);
    w.dies(west, Some(ally), None, now, &mut Vec::new());
    assert_eq!((mark(&w), held(&w)), (None, 3));
    // A stranger's death touches nothing.
    w.choose_mark(ally);
    let stranger = beside(&mut w, "kobold", Dir::S);
    w.dies(stranger, Some(player), None, now, &mut Vec::new());
    assert_eq!((mark(&w), held(&w)), (Some(ally), 3));
}

#[test]
fn killing_the_mark_in_a_fight_gives_its_opening() {
    let mut w = thief();
    let east = beside(&mut w, "kobold", Dir::E);
    w.state.actors.get_mut(&east).expect("kobold").hp = 1;
    set(&mut w, 2);
    w.choose_mark(east);
    for _ in 0..40 {
        if !w.state.actors.contains_key(&east) {
            break;
        }
        set(&mut w, 2);
        run(&mut w, "move:E");
    }
    assert!(!w.state.actors.contains_key(&east), "the kobold fell");
    assert_eq!(mark(&w), None);
    // Two, an opening if the killing blow was a sneak attack, and the kill's.
    assert!((3..=4).contains(&held(&w)), "{}", held(&w));
}

#[test]
fn a_departure_a_level_change_and_the_decay_to_zero_clear_the_mark_and_keep_the_pool() {
    let mut w = thief();
    let east = beside(&mut w, "kobold", Dir::E);
    set(&mut w, 2);
    w.choose_mark(east);
    w.departs(east, &mut Vec::new());
    assert_eq!(
        (mark(&w), held(&w)),
        (None, 2),
        "a lapsed summons: cleared, kept"
    );
    // A level change.
    let west = beside(&mut w, "kobold", Dir::W);
    w.choose_mark(west);
    let to = (w.state.current().stairs.iter())
        .find_map(|s| s.to.clone())
        .expect("the showcase has a way down");
    w.travel(&to, None).expect("travels");
    assert_eq!(
        (mark(&w), held(&w)),
        (None, 2),
        "a new floor: cleared, kept"
    );
    // The decay to 0.
    let foe = beside(&mut w, "kobold", Dir::E);
    let peaceful = w.content().factions().id("peaceful");
    w.state
        .actors
        .get_mut(&foe)
        .expect("kobold")
        .faction_override = peaceful;
    set(&mut w, 1);
    w.choose_mark(foe);
    run(&mut w, "wait*10");
    assert_eq!(
        mark(&w),
        Some(foe),
        "out of sight or not, it stays while the pool holds"
    );
    run(&mut w, "wait*20");
    assert_eq!(
        (mark(&w), held(&w)),
        (None, 0),
        "the pool is gone: so is the mark"
    );
}

#[test]
fn an_exploit_is_refused_on_another_than_the_mark() {
    let mut w = thief();
    let east = beside(&mut w, "kobold", Dir::E);
    let west = beside(&mut w, "kobold", Dir::W);
    set(&mut w, 3);
    w.choose_mark(west);
    let outcome = w.apply(&"use:exploit:E".parse().expect("action"));
    assert!(!outcome.took_turn, "refused");
    assert_eq!(told(&w, "Your openings are in the kobold's guard."), 1);
    assert_eq!(
        (held(&w), mark(&w)),
        (3, Some(west)),
        "nothing spent or moved"
    );
    let outcome = w.apply(&"use:exploit:W".parse().expect("action"));
    assert!(outcome.took_turn, "at the mark: struck");
    assert!(
        held(&w) <= 1,
        "every one spent; only the mark's kill gives one back"
    );
    assert!(w.state.actors.contains_key(&east));
}

#[test]
fn the_mark_is_shown_saved_and_replayed() {
    let mut w = thief();
    let east = beside(&mut w, "kobold", Dir::E);
    w.choose_mark(east);
    perceive_now(&mut w);
    let stats = dump::layer_rows(&w, Layer::Stats);
    assert!(stats.iter().any(|r| r == "Mark the kobold"), "{stats:?}");
    let seen = (w.perception().actors().iter()).find(|a| a.id == east);
    assert!(seen.is_some_and(|a| a.marked), "look mode's mark");
    let json = dump::json(&w, &[], false).to_string();
    assert!(json.contains("\"on\":\"the kobold\""), "{json}");
    assert!(json.contains("\"marked\":true"), "{json}");
    let restored = World::from_state(BUILTIN.clone(), w.state().clone()).expect("restores");
    assert_eq!(restored.state.player().mark, Some(east), "saved");
    assert_eq!(restored.state_hash(), w.state_hash());
    // A replay of a fight verifies.
    let spec: CharacterSpec = "human,male,thief".parse().expect("a character");
    let mut w = World::start_run(BUILTIN.clone(), "arena", Some(3), Some(&spec), None, true)
        .expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E*6 wait*4 move:N*3").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// Perceives the world as a turn's end does.
fn perceive_now(w: &mut World) {
    w.perceive(&mut Vec::new());
}
