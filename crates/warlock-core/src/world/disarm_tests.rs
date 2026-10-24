//! Disarming traps (spec §8 "Disarming traps"): the thief, trained in Disable device, takes a
//! known trap apart with a Dexterity check, fails and tries again, or sets it off by failing
//! badly; a door's trap too; the untrained and the traps nobody knows of are refused for free;
//! the abilities menu lists it; every try takes its time, may train the skill, and replays.

use super::search::Hidden;
use super::*;
use crate::ability::{Ability, OwnAbility};
use crate::action::parse_actions;
use crate::check::{self, Checked};
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::rng::Rng;
use crate::skills::{Effect, SkillId};
use crate::stats::Stat;
use crate::time::Duration;

/// A lit hall on the built-in content, the player at `@` as `character` (`RACE,GENDER,CLASS`):
/// `a` a known arrow trap, `h` a hidden one, `p` a known pit, `d` a closed door with a known arrow
/// trap.
fn world(map: &str, character: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    let legend = r#"
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", trap = "arrow", known = true }
"h" = { terrain = "floor", trap = "arrow" }
"p" = { terrain = "floor", trap = "pit", known = true }
"d" = { terrain = "door_closed", trap = "arrow", known = true }
"#;
    files.insert(
        "levels/disarm_hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{legend}"),
    );
    let (race, rest) = character.split_once(',').expect("race");
    let (gender, class) = rest.split_once(',').expect("gender and class");
    files.insert(
        "scenarios/disarm_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"disarm_hall\"\nseed = 7\ncharacter = {{ race = \
             \"{race}\", gender = \"{gender}\", class = \"{class}\" }}\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "disarm_hall", None).expect("world")
}

const WARRIOR: &str = "human,male,warrior";
const THIEF: &str = "human,female,thief";

/// The player at (1, 1) with the trap (or door) east of them at (2, 1).
const EAST: Pos = Pos::new(2, 1);

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn last(world: &World) -> String {
    texts(world).last().cloned().unwrap_or_default()
}

/// Sets the gameplay stream to the first whose next check at disarming the trap at `at` comes
/// out as `want` says.
fn rig(world: &mut World, at: Pos, want: impl Fn(Checked) -> bool) {
    let kind = world.state.current().traps[&at].kind;
    let difficulty = world.difficulty_of(Hidden::Trap(kind));
    let modifier = world.disarm_modifier();
    let seed = (0..10_000)
        .find(|&s| want(check::roll(&mut Rng::gameplay(s), modifier, difficulty)))
        .expect("a stream that comes out so");
    world.state.rng = Rng::gameplay(seed);
}

fn sprung(out: &Outcome) -> bool {
    (out.events.iter()).any(|e| matches!(e, Event::TrapSprung { .. }))
}

fn disable_device(world: &World) -> SkillId {
    let i = (world.content().skills().iter())
        .position(|s| s.id == "disable_device")
        .expect("Disable device");
    SkillId(u8::try_from(i).expect("few skills"))
}

#[test]
fn a_known_trap_next_to_the_thief_is_disarmed_and_gone() {
    let mut w = world("#####\n#@a.#\n#####\n", THIEF);
    rig(&mut w, EAST, |c| c == Checked::Made);
    let out = run(&mut w, "disarm:E");
    assert!(out[0].took_turn);
    assert_eq!(out[0].elapsed, Duration::seconds(12), "disarm = 2000: 12 s");
    assert_eq!(last(&w), "You disarm the arrow trap.");
    assert!(!sprung(&out[0]));
    let level = w.state.current();
    assert!(level.traps.is_empty() && level.trap_memory.is_empty());
    assert_eq!(w.perception().trap(EAST), None, "no longer shown");
    // Gone: walking over the cell sets nothing off.
    let out = run(&mut w, "move:E");
    assert_eq!(w.player_pos(), EAST);
    assert!(!sprung(&out[0]));
}

#[test]
fn a_near_miss_leaves_the_trap_and_a_bad_one_sets_it_off() {
    let mut w = world("#####\n#@a.#\n#####\n", THIEF);
    // Short by less than five: nothing happens but the time spent.
    rig(
        &mut w,
        EAST,
        |c| matches!(c, Checked::Failed { by } if by < SPRINGS_SHORT_BY),
    );
    let out = run(&mut w, "disarm:E");
    assert!(out[0].took_turn);
    assert_eq!(last(&w), "You fail to disarm the arrow trap.");
    assert!(!sprung(&out[0]));
    assert!(w.state.current().traps.contains_key(&EAST), "still there");
    // Short by five or more: it goes off at the thief, from its slit.
    rig(
        &mut w,
        EAST,
        |c| matches!(c, Checked::Failed { by } if by >= SPRINGS_SHORT_BY),
    );
    let player = w.state.player;
    let out = run(&mut w, "disarm:E");
    assert!(out[0].took_turn);
    assert!(out[0].events.contains(&Event::TrapSprung {
        trap: w.content().trap_id("arrow").expect("arrow"),
        at: EAST,
        victim: player,
    }));
    assert!(
        (out[0].events.iter()).any(|e| matches!(e, Event::Missile { .. })),
        "{:?}",
        out[0].events
    );
    let said = texts(&w);
    let set_off = said.iter().position(|t| t == "You set off the arrow trap!");
    let shot = said
        .iter()
        .position(|t| t == "An arrow shoots out of the wall!");
    assert!(set_off.is_some() && set_off < shot, "{said:?}");
    // It never runs out, so it is still there to try again.
    assert!(w.state.current().traps.contains_key(&EAST));
    rig(&mut w, EAST, |c| c == Checked::Made);
    run(&mut w, "disarm:E");
    assert!(w.state.current().traps.is_empty());
}

#[test]
fn a_bad_try_at_a_pit_costs_the_fall_and_the_climb() {
    let mut w = world("#####\n#@p.#\n#####\n", THIEF);
    rig(
        &mut w,
        EAST,
        |c| matches!(c, Checked::Failed { by } if by >= SPRINGS_SHORT_BY),
    );
    let out = run(&mut w, "disarm:E");
    assert!(sprung(&out[0]));
    assert!(texts(&w).contains(&"You fall into a pit.".to_string()));
    assert_eq!(
        out[0].elapsed,
        Duration::seconds(18),
        "12 s, and a wait to climb out"
    );
    assert_eq!(w.player_pos(), Pos::new(1, 1), "back out on their own side");
}

#[test]
fn a_doors_trap_is_disarmed_like_any_other() {
    let mut w = world("#####\n#@d.#\n#####\n", THIEF);
    rig(&mut w, EAST, |c| c == Checked::Made);
    run(&mut w, "disarm:E");
    assert_eq!(last(&w), "You disarm the arrow trap.");
    assert!(w.state.current().traps.is_empty());
    let out = run(&mut w, "open:E");
    assert!(!sprung(&out[0]));
    assert_eq!(last(&w), "You open the door.");
}

#[test]
fn the_untrained_and_traps_nobody_knows_of_are_refused_for_free() {
    // The warrior is not trained in Disable device.
    let mut w = world("#####\n#@a.#\n#####\n", WARRIOR);
    let before = w.state.clone();
    let out = run(&mut w, "disarm:E");
    assert!(!out[0].took_turn);
    assert_eq!(out[0].elapsed, Duration::ZERO);
    assert_eq!(last(&w), "You do not know how to disarm traps.");
    assert_eq!(w.state.rng, before.rng, "no draw");
    assert_eq!(w.state.current().traps, before.current().traps);
    // A hidden trap is refused as an empty cell or a wall is: it gives nothing away.
    let mut w = world("#####\n#@h.#\n#####\n", THIEF);
    let (rng, training) = (w.state.rng.clone(), w.state.training.clone());
    for dir in ["E", "W", "N"] {
        let out = run(&mut w, &format!("disarm:{dir}"));
        assert!(!out[0].took_turn, "{dir}");
        assert!(last(&w).starts_with("You know of no trap there."), "{dir}");
    }
    assert_eq!(
        (w.state.rng.clone(), w.state.training.clone()),
        (rng, training)
    );
    assert!(!w.state.current().traps[&EAST].known, "still hidden");
}

#[test]
fn the_try_is_dexterity_and_disable_device_against_the_traps_difficulty() {
    let mut w = world("#####\n#@a.#\n#####\n", THIEF);
    let content = w.content().clone();
    let dexterity = (w.state.player().effective_stats(&content)).modifier(Stat::Dexterity);
    // A beginner's Disable device gives nothing yet; a master's +8.
    assert_eq!(w.disarm_modifier(), dexterity);
    let id = disable_device(&w);
    let value = i32::from(w.state.player().skills.value(id).expect("trained"));
    w.player_mut().skills.add(id, 100 - value);
    assert_eq!(w.disarm_modifier(), dexterity + 8);
    let skill = w.state.player().skill_effect(&content, Effect::Disarm);
    assert_eq!(skill, 8);
    // The difficulty is the trap's own, the one finding it takes.
    let arrow = content.trap_id("arrow").expect("arrow");
    assert_eq!(
        w.difficulty_of(Hidden::Trap(arrow)),
        i32::from(content.trap(arrow).difficulty)
    );
    // One check: two draws of the gameplay stream.
    let mut expected = w.state.rng.clone();
    crate::combat::roll_natural(&mut expected);
    run(&mut w, "disarm:E");
    assert_eq!(w.state.rng, expected);
}

#[test]
fn every_try_may_train_disable_device() {
    let mut w = world("#####\n#@a.#\n#####\n", THIEF);
    assert!(w.trains(crate::skills::Use::Disarm));
    let training = w.state.training.clone();
    rig(
        &mut w,
        EAST,
        |c| matches!(c, Checked::Failed { by } if by < SPRINGS_SHORT_BY),
    );
    run(&mut w, "disarm:E");
    assert_ne!(w.state.training, training, "a mark's chance drawn");
}

#[test]
fn the_abilities_menu_lists_disarming_for_the_trained_only() {
    let w = world("#####\n#@a.#\n#####\n", THIEF);
    let abilities = w.perception().abilities();
    assert!(abilities.contains(&OwnAbility {
        ability: Ability::Disarm,
        unavailable: None,
    }));
    let id = disable_device(&w);
    assert!(!abilities.iter().any(|a| a.ability == Ability::Skill(id)));
    let w = world("#####\n#@a.#\n#####\n", WARRIOR);
    assert!(
        !(w.perception().abilities().iter()).any(|a| a.ability == Ability::Disarm),
        "not trained, not had"
    );
}

#[test]
fn disarming_saves_and_replays() {
    let mut w = world("#####\n#@ap#\n#####\n", THIEF);
    let content = w.content().clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("disarm:E*6 wait disarm:E*3").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content.clone()).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
    let saved = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(saved.state(), w.state());
}
