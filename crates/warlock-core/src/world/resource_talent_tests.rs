//! The resource talents (spec §8 "Talents" and "Resources"; #253): Battle trance's higher start
//! and slower decay of rage, Inner calm's larger and quicker ki, Steady hand's quicker focus,
//! Patient blade's slower openings and the second opening of a sneak attack that kills, and
//! Zealot's second zeal a blow taken (#914).
//! Each reads the one hook it names: `Actor::resource_start`, `Actor::resource_max`,
//! `World::resource_rate`, and `World::gain_resource`.

use super::talent::Due;
use super::*;
use crate::character::CharacterSpec;
use crate::content::{Build, ContentFiles, ResourceId};
use crate::light::Light;
use crate::test_support::run;
use std::sync::LazyLock;

/// A pitch-dark corridor: the player at its west end, a giant rat beside them.
const DEN: &str = r#"
name = "Den"
ambient = [0, 0, 0, 0]
map = '''
#######
#@r...#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"#;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/den.toml", DEN);
    files.insert(
        "scenarios/den.toml",
        "name = \"Den\"\nstart_level = \"den\"\nhp = 1000\n\
         stats = { strength = 40, dexterity = 40 }\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(seed: u64) -> World {
    World::new(CONTENT.clone(), "den", Some(seed)).expect("scenario starts")
}

fn id(name: &str) -> ResourceId {
    CONTENT.resource_id(name).expect("a resource")
}

fn held(w: &World, name: &str) -> Option<u16> {
    (w.state.player().resources.get(id(name))).map(|p| p.value)
}

fn set(w: &mut World, name: &str, value: u16) {
    w.player_mut().resources.set(id(name), value);
}

/// The player takes talent `id`, without a level to pick it at.
fn give(w: &mut World, talent: &str) {
    assert!(CONTENT.talents().id(talent).is_some(), "{talent}");
    w.player_mut().talents.take(talent, None);
}

/// A twin of `w` without its talents.
fn plain(w: &World) -> World {
    let mut twin = World::from_state(CONTENT.clone(), w.state().clone()).expect("restores");
    twin.player_mut().talents = Default::default();
    twin
}

#[test]
fn battle_trance_starts_the_rage_higher_and_slows_its_decay() {
    let mut w = world(1);
    give(&mut w, "battle_trance");
    let mut twin = plain(&w);
    run(&mut w, "resource:rage");
    run(&mut twin, "resource:rage");
    assert_eq!(held(&twin, "rage"), Some(12));
    assert_eq!(held(&w, "rage"), Some(18), "six higher");
    assert_eq!(w.state.player().resource_max(&CONTENT, id("rage"), 1), 20);
    // A third slower: 6 points in ten base turns, where the plain rage loses 10.
    run(&mut w, "wait*10");
    run(&mut twin, "wait*10");
    assert_eq!(held(&twin, "rage"), Some(2));
    assert_eq!(held(&w, "rage"), Some(12));
}

#[test]
fn inner_calm_holds_more_ki_and_brings_it_back_faster() {
    let mut w = world(1);
    give(&mut w, "inner_calm");
    let mut twin = plain(&w);
    run(&mut w, "resource:ki");
    run(&mut twin, "resource:ki");
    // Willpower 10 at level 1: 4.
    assert_eq!(held(&twin, "ki"), Some(4));
    assert_eq!(held(&w, "ki"), Some(4), "it starts at the data's start");
    assert_eq!(w.state.player().resource_max(&CONTENT, id("ki"), 1), 6);
    set(&mut w, "ki", 0);
    set(&mut twin, "ki", 0);
    // A point every three base turns, half again as fast: every two.
    run(&mut w, "wait*6");
    run(&mut twin, "wait*6");
    assert_eq!(held(&twin, "ki"), Some(2));
    assert_eq!(held(&w, "ki"), Some(3));
    run(&mut w, "wait*20");
    assert_eq!(held(&w, "ki"), Some(6), "up to its larger maximum");
}

#[test]
fn steady_hand_brings_focus_back_faster_but_not_while_resting() {
    let mut w = world(1);
    give(&mut w, "steady_hand");
    let mut twin = plain(&w);
    run(&mut w, "resource:focus");
    run(&mut twin, "resource:focus");
    set(&mut w, "focus", 0);
    set(&mut twin, "focus", 0);
    run(&mut w, "wait*4");
    run(&mut twin, "wait*4");
    assert_eq!(held(&twin, "focus"), Some(4));
    assert_eq!(held(&w, "focus"), Some(6));
    let player = w.state.player;
    assert_eq!(w.resource_rate(player, id("focus")), 150);
    w.resting = true;
    assert_eq!(
        w.resource_rate(player, id("focus")),
        0,
        "resting stops it still"
    );
}

#[test]
fn patient_blade_slows_the_openings_and_the_marks_kill_gives_two() {
    let mut w = world(1);
    give(&mut w, "patient_blade");
    let mut twin = plain(&w);
    run(&mut w, "resource:openings");
    run(&mut twin, "resource:openings");
    set(&mut w, "openings", 4);
    set(&mut twin, "openings", 4);
    // A point every twenty base turns; half as fast, every forty.
    run(&mut w, "wait*20");
    run(&mut twin, "wait*20");
    assert_eq!(held(&twin, "openings"), Some(3));
    assert_eq!(held(&w, "openings"), Some(4));
    run(&mut w, "wait*20");
    assert_eq!(held(&w, "openings"), Some(3));
    // The mark's kill: two with the talent, one without.
    let player = w.state.player;
    w.gain_resource(player, Build::MarkKill);
    twin.gain_resource(player, Build::MarkKill);
    assert_eq!(held(&w, "openings"), Some(5));
    assert_eq!(held(&twin, "openings"), Some(4));
}

#[test]
fn a_sneak_attack_that_kills_its_mark_builds_three_openings_with_patient_blade() {
    let mut kills = 0;
    for seed in 0..40 {
        let mut w = world(seed);
        give(&mut w, "ambusher");
        give(&mut w, "patient_blade");
        w.state.current_mut().ambient = Light::new(0, 0, 0, 0);
        let mut twin = plain(&w);
        twin.player_mut().talents.take("ambusher", None);
        for w in [&mut w, &mut twin] {
            run(w, "resource:openings");
            let rat = (w.state.actors.values())
                .find(|a| a.id != w.state.player)
                .map(|a| a.id)
                .expect("the rat");
            w.state.actors.get_mut(&rat).expect("rat").hp = 1;
            assert!(
                w.unaware(rat, w.state.player),
                "the rat has not seen the player"
            );
            run(w, "move:E");
        }
        if w.state.actors.len() > 1 {
            continue; // the blow missed
        }
        kills += 1;
        assert_eq!(
            held(&w, "openings"),
            Some(3),
            "the hit's and the kill's two"
        );
        if twin.state.actors.len() == 1 {
            assert_eq!(held(&twin, "openings"), Some(2), "the hit's and the kill's");
        }
    }
    assert!(kills > 0, "some sneak attack killed");
}

#[test]
fn zealot_builds_two_zeal_a_blow_taken_and_no_more_a_blow_given() {
    let spec: CharacterSpec = "human,male,paladin".parse().expect("a character");
    let mut w = World::with_character(CONTENT.clone(), "den", Some(1), Some(&spec)).expect("world");
    let mut twin = World::from_state(CONTENT.clone(), w.state().clone()).expect("restores");
    give(&mut w, "zealot");
    set(&mut w, "zeal", 0);
    set(&mut twin, "zeal", 0);
    let player = w.state.player;
    w.gain_resource(player, Build::Struck);
    twin.gain_resource(player, Build::Struck);
    assert_eq!(held(&w, "zeal"), Some(2));
    assert_eq!(held(&twin, "zeal"), Some(1));
    w.gain_resource(player, Build::Hit);
    twin.gain_resource(player, Build::Hit);
    assert_eq!(held(&w, "zeal"), Some(3), "a blow given builds its one");
    assert_eq!(held(&twin, "zeal"), Some(2));
}

#[test]
fn a_rats_bites_build_a_zealot_more_zeal_in_the_same_fight() {
    let spec: CharacterSpec = "human,male,paladin".parse().expect("a character");
    let mut more = 0;
    for seed in 0..20 {
        let mut w =
            World::with_character(CONTENT.clone(), "den", Some(seed), Some(&spec)).expect("world");
        let mut twin = World::from_state(CONTENT.clone(), w.state().clone()).expect("restores");
        give(&mut w, "zealot");
        for w in [&mut w, &mut twin] {
            set(w, "zeal", 0);
            run(w, "wait*5");
        }
        // Zeal draws nothing, so the twins fight the same fight.
        assert_eq!(w.state.player().hp, twin.state.player().hp, "seed {seed}");
        let (zealous, plain) = (held(&w, "zeal"), held(&twin, "zeal"));
        assert!(zealous >= plain, "seed {seed}");
        more += u32::from(zealous > plain);
    }
    assert!(more > 0, "some bite built the second point");
}

#[test]
fn the_resource_talents_need_their_class_and_level() {
    let pick = |spec: &str, level: u8, talent: &str| {
        let spec: CharacterSpec = spec.parse().expect("a character");
        let mut w =
            World::with_character(CONTENT.clone(), "den", Some(1), Some(&spec)).expect("world");
        let step = (w.content().experience().threshold(level) / 200).max(1);
        while w.player_level() < level {
            w.grant_xp(step);
        }
        w.state.progress.due.clear();
        w.state.progress.due.push(Due {
            level,
            talents: 1,
            rolls: 0,
            bonus: 0,
        });
        run(&mut w, &format!("talent:{talent}"));
        w.state.player().talents.holds(talent, None)
    };
    assert!(pick("human,male,berserker", 6, "battle_trance"));
    assert!(
        !pick("human,male,berserker", 5, "battle_trance"),
        "too early"
    );
    assert!(
        !pick("human,male,warrior", 6, "battle_trance"),
        "not a berserker"
    );
    assert!(pick("human,male,ranger", 12, "steady_hand"));
    assert!(!pick("human,male,thief", 12, "steady_hand"));
    assert!(pick("human,male,ascetic", 18, "inner_calm"));
    assert!(!pick("human,male,ranger", 18, "inner_calm"));
    assert!(pick("human,male,thief", 32, "patient_blade"));
    assert!(!pick("human,male,thief", 31, "patient_blade"));
    assert!(pick("human,male,paladin", 6, "zealot"));
    assert!(!pick("human,male,paladin", 5, "zealot"), "too early");
    assert!(!pick("human,male,warrior", 6, "zealot"), "holds no Smite");
}
