//! Sidestepping a known trap (spec §8 "Traps", #476): a willing step onto a trap the player
//! knows of may avoid it, by their dodge (Dexterity capped by armour, plus Dodge), within a floor
//! and a ceiling; a hidden trap, a step the player did not choose, and creatures keep the old
//! rules; an avoided trap stays, trains Dodge, and replays.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, TrapAvoid};
use crate::replay::Replay;
use crate::rng::Rng;
use crate::skills::{SkillId, Use};

/// A lit hall on the built-in content, the player at `@` with `stats` (a scenario's TOML, no
/// character: no talents): `a` a known arrow trap, `h` a hidden one, `p` a known pit, `o` an
/// open door with a known arrow trap.
fn world(map: &str, stats: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    let legend = r#"
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", trap = "arrow", known = true }
"h" = { terrain = "floor", trap = "arrow" }
"p" = { terrain = "floor", trap = "pit", known = true }
"o" = { terrain = "door_open", trap = "arrow", known = true }
"#;
    files.insert(
        "levels/sidestep_hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{legend}"),
    );
    files.insert(
        "scenarios/sidestep_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"sidestep_hall\"\nseed = 7\nhp = 1000\n{stats}\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let w = World::new(content, "sidestep_hall", None).expect("world");
    let talents = w.state.player().talent_mods(w.content());
    assert!(!talents.sure_step && !talents.pit_proof, "no Light step");
    w
}

const AVERAGE: &str = "stats = { dexterity = 10 }";
const AGILE: &str = "stats = { dexterity = 18 }";
const CLUMSY: &str = "stats = { dexterity = 4 }";

/// The player at (1, 1) with the trap east of them at (2, 1).
const EAST: Pos = Pos::new(2, 1);

fn step(world: &mut World) -> Outcome {
    world.apply(&parse_actions("move:E").expect("action")[0])
}

fn sprung(out: &Outcome) -> bool {
    (out.events.iter()).any(|e| matches!(e, Event::TrapSprung { .. }))
}

fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// The player's chance now of avoiding a known trap.
fn chance(world: &World) -> u32 {
    world.content().trap_avoid().chance(world.trap_dodge())
}

/// Sets the gameplay stream to the first whose next draw avoids (`avoid`) or misses at `chance`.
fn rig(world: &mut World, avoid: bool) {
    let chance = chance(world);
    let seed = (0..10_000)
        .find(|&s| Rng::gameplay(s).chance(chance) == avoid)
        .expect("a stream that comes out so");
    world.state.rng = Rng::gameplay(seed);
}

/// How many of `tries` fresh worlds (`map`, `stats`), each on its own stream, step onto the trap
/// east without setting it off.
fn avoided(map: &str, stats: &str, tries: u64) -> u64 {
    (0..tries)
        .filter(|&s| {
            let mut w = world(map, stats);
            w.state.rng = Rng::gameplay(s);
            !sprung(&step(&mut w))
        })
        .count() as u64
}

fn tumble(world: &World) -> SkillId {
    let i = (world.content().skills().iter())
        .position(|s| s.id == "tumble")
        .expect("Tumble");
    SkillId(u8::try_from(i).expect("few skills"))
}

#[test]
fn the_chance_is_base_plus_dodge_within_the_floor_and_the_ceiling() {
    let avoid = TrapAvoid {
        base: 55,
        per_point: 10,
        min: 15,
        max: 90,
    };
    let table: Vec<u32> = (-5..=5).map(|d| avoid.chance(d)).collect();
    assert_eq!(table, [15, 15, 25, 35, 45, 55, 65, 75, 85, 90, 90]);
    assert_eq!(avoid.chance(-100), 15, "never hopeless");
    assert_eq!(avoid.chance(100), 90, "never certain");
    let w = world("####\n#@a#\n####\n", AVERAGE);
    assert_eq!(
        *w.content().trap_avoid(),
        avoid,
        "data/traps.toml's [avoid]"
    );
}

#[test]
fn a_known_trap_avoided_stays_and_costs_the_step() {
    let mut w = world("#####\n#@a.#\n#####\n", AVERAGE);
    rig(&mut w, true);
    let before = w.state.current().traps[&EAST];
    let hp = w.state.player().hp;
    let out = step(&mut w);
    assert!(!sprung(&out));
    assert_eq!(w.player_pos(), EAST);
    assert_eq!(
        w.state.current().traps[&EAST],
        before,
        "untouched, charges too"
    );
    assert_eq!(last(&w), "You deftly avoid the arrow trap.");
    assert_eq!(w.state.player().hp, hp);
}

#[test]
fn a_known_trap_not_avoided_goes_off() {
    let mut w = world("#####\n#@a.#\n#####\n", AVERAGE);
    rig(&mut w, false);
    assert!(sprung(&step(&mut w)));
}

#[test]
fn a_pit_is_stepped_over_lightly() {
    let mut w = world("#####\n#@p.#\n#####\n", AVERAGE);
    rig(&mut w, true);
    assert!(!sprung(&step(&mut w)));
    assert_eq!(last(&w), "You step lightly over the pit.");
}

#[test]
fn a_hidden_trap_always_goes_off() {
    for s in 0..40 {
        let mut w = world("#####\n#@h.#\n#####\n", AGILE);
        w.state.rng = Rng::gameplay(s);
        assert!(sprung(&step(&mut w)), "stream {s}");
    }
}

#[test]
fn dodge_sets_the_odds_and_armour_caps_it() {
    let map = "#####\n#@a.#\n#####\n";
    // Dexterity 10: dodge 0, 55 %; 18: +4, 90 % (the ceiling); 4: -3, 25 %.
    assert_eq!(chance(&world(map, AVERAGE)), 55);
    assert_eq!(chance(&world(map, AGILE)), 90);
    assert_eq!(chance(&world(map, CLUMSY)), 25);
    // Plate armour caps Dexterity at +1: 65 %.
    let mut w = world(map, AGILE);
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory = std::iter::once(crate::object::Stack::one(
        content.object_id("plate_armour").expect("plate"),
    ))
    .collect();
    me.equipment
        .put(&[content.slot_id("body").expect("body")], 0);
    assert_eq!(w.trap_dodge(), 1);
    assert_eq!(chance(&w), 65);
    // The Dodge skill adds: Tumble 40 is +2 more.
    let id = tumble(&w);
    w.player_mut().skills.add(id, 40);
    assert_eq!(w.trap_dodge(), 3);
    assert_eq!(chance(&w), 85);
    // On many streams, the odds show; the agile sometimes still sets it off.
    let agile = avoided(map, AGILE, 200);
    let clumsy = avoided(map, CLUMSY, 200);
    assert!((160..200).contains(&agile), "{agile} of 200 at 90 %");
    assert!((25..80).contains(&clumsy), "{clumsy} of 200 at 25 %");
}

#[test]
fn a_step_not_chosen_and_a_doors_trap_get_no_chance() {
    // Set off from elsewhere than a willing step (a step the player was turned into, a
    // creature's): the old rule.
    let mut w = world("#####\n#@a.#\n#####\n", AGILE);
    rig(&mut w, true);
    let player = w.state.player;
    let mut events = Vec::new();
    w.step_onto_trap(EAST, player, &mut events);
    assert!(events.iter().any(|e| matches!(e, Event::TrapSprung { .. })));
    // Through an open door with a known trap: nothing goes off, nothing is drawn.
    let mut w = world("#####\n#@o.#\n#####\n", AGILE);
    let rng = w.state.rng.clone();
    assert!(!sprung(&step(&mut w)));
    assert_eq!(w.state.rng, rng);
}

#[test]
fn avoiding_a_trap_may_train_dodge() {
    let mut w = world("#####\n#@a.#\n#####\n", AVERAGE);
    let id = tumble(&w);
    w.player_mut().skills.add(id, 10);
    assert!(w.trains(Use::Dodge));
    rig(&mut w, true);
    let training = w.state.training.clone();
    step(&mut w);
    assert_ne!(w.state.training, training, "a mark's chance drawn");
}

#[test]
fn sidestepping_saves_and_replays() {
    let mut w = world("######\n#@apa#\n######\n", AVERAGE);
    let content = w.content().clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E move:E wait*2 move:E move:W*3").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content.clone()).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
    let saved = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(saved.state(), w.state());
}
