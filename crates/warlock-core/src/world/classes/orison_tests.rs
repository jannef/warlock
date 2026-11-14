//! The priest's prayer (`world/classes/orison.rs`, spec §8 "Class abilities"; the priest design §6.1,
//! §7): each use at its level, the priority order, the answers a tier gives, the refusal, the
//! strain and its lock, the faces, and saves and replays.

use super::orison::Strain;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::alignment::Score;
use crate::content::ClassActive;
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::test_support::draws;
use crate::test_support::{CONTENT as BUILTIN, last, run, said};
use crate::time::Duration;
use crate::world::magic::caster_tests::{reach, world_with};
use crate::world::*;

/// A human priest of `level` (every score 14), Neutral, at full hit points and mana, Not
/// hungry, and with no strain.
fn priest(level: u8) -> World {
    let mut w = world_with(BUILTIN.clone(), "priest");
    if level > 1 {
        reach(&mut w, level);
    }
    let content = w.content().clone();
    let me = w.player_mut();
    me.alignment = Some(Score::new(0));
    me.hp = me.max_hp(&content);
    me.mana = me.max_mana(&content);
    w
}

fn has(w: &World, id: &str) -> bool {
    let status = w.content().status_id(id).expect("a status");
    w.state.player().effects.has(status)
}

fn strain(w: &World) -> Option<Strain> {
    w.state.progress.prayer
}

/// The player hurt to half their hit points.
fn hurt(w: &mut World) {
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max / 2;
}

fn full_hp(w: &World) -> bool {
    let me = w.state.player();
    me.hp == me.max_hp(w.content())
}

fn hungry(w: &mut World) {
    let hungry = w.content().hunger().expect("hunger").states.hungry;
    if let Some(s) = w.player_mut().satiation.as_mut() {
        s.nutrition = hungry;
    }
}

fn hunger(w: &World) -> crate::hunger::Hunger {
    w.state.player().hunger(w.content()).expect("hungers")
}

/// The strain cleared, as if a long while had passed: the next prayer draws nothing.
fn rested(w: &mut World) {
    w.state.progress.prayer = None;
}

#[test]
fn the_orison_closes_wounds_for_a_casts_time() {
    let mut w = priest(1);
    let ability = Ability::Class(ClassActive::Orison);
    assert!(ability.had_by(w.state.player(), w.content()));
    hurt(&mut w);
    run(&mut w, "afflict:bleeding*2");
    let (turn, time, rng) = (w.state.turn, w.state.time, draws(&w));
    run(&mut w, "use:orison");
    assert!(full_hp(&w));
    assert!(!has(&w, "bleeding"));
    let lines = said(&w);
    assert!(
        lines.contains(&"You kneel and pray.".to_string()),
        "{lines:?}"
    );
    assert_eq!(last(&w), "Your wounds close.");
    assert!(
        !lines.contains(&"The bleeding stops.".to_string()),
        "the prayer's line covers it"
    );
    assert_eq!(w.state.turn, turn + 1, "one action");
    assert!(w.state.time > time, "it takes time");
    assert_eq!(draws(&w), rng, "no strain yet: no draw");
    let s = strain(&w).expect("strained");
    assert_eq!((s.value, s.locked), (100, false));
}

#[test]
fn nothing_to_pray_for_is_refused_free() {
    let mut w = priest(1);
    let (turn, time) = (w.state.turn, w.state.time);
    run(&mut w, "use:orison");
    assert_eq!(last(&w), "You have nothing to pray for.");
    assert_eq!((w.state.turn, w.state.time), (turn, time));
    assert_eq!(strain(&w), None);
    // Hungry at level 1: the hunger use is not theirs yet.
    hungry(&mut w);
    run(&mut w, "use:orison");
    assert!(last(&w).starts_with("You have nothing to pray for."));
    assert_eq!(strain(&w), None);
}

#[test]
fn each_use_comes_at_its_level() {
    // Daily bread at 6: hungry, fed to Satiated; rotten food harmless (Purify's rule).
    let mut w = priest(6);
    hungry(&mut w);
    run(&mut w, "use:orison");
    assert_eq!(hunger(&w), crate::hunger::Hunger::Satiated);
    assert_eq!(last(&w), "You feel fed.");
    assert!(w.state.player().talent_mods(w.content()).hunger.purify);
    // Release at 18, not at 12.
    let mut w = priest(12);
    run(&mut w, "afflict:webbed use:orison");
    assert_eq!(last(&w), "You have nothing to pray for.");
    let mut w = priest(18);
    run(&mut w, "afflict:webbed afflict:ill_wished use:orison");
    assert!(!has(&w, "webbed") && !has(&w, "ill_wished"));
    assert_eq!(last(&w), "You are released.");
    // Restoration at 25: stat damage and drained points.
    let mut w = priest(25);
    let content = w.content().clone();
    let me = w.player_mut();
    me.damage.take(crate::stats::Stat::Strength, 2, 14);
    me.drained[crate::stats::Stat::Toughness] = 1;
    run(&mut w, "use:orison");
    let me = w.state.player();
    assert!(!me.damage.is_damaged());
    assert!(crate::world::turn::actor::undrained(&me.drained));
    assert_eq!(last(&w), "You feel whole again.");
    // Mana at 32.
    let mut w = priest(32);
    w.player_mut().mana = 3;
    run(&mut w, "use:orison");
    let me = w.state.player();
    assert_eq!(me.mana, me.max_mana(&content));
    assert_eq!(last(&w), "Your mind is filled.");
}

#[test]
fn the_most_pressing_comes_first_one_answer_a_prayer() {
    let mut w = priest(12);
    hurt(&mut w);
    run(&mut w, "infect:the_shakes afflict:poisoned*3");
    // Brought to show at once.
    let shakes = w.content().disease_id("the_shakes").expect("a disease");
    w.player_mut()
        .diseases
        .get_mut(shakes)
        .expect("caught")
        .shown = true;
    run(&mut w, "use:orison");
    assert!(w.state.player().diseases.is_empty());
    assert!(has(&w, "poisoned") && !full_hp(&w));
    assert_eq!(last(&w), "The sickness leaves you.");
    rested(&mut w);
    run(&mut w, "use:orison");
    assert!(!has(&w, "poisoned") && !full_hp(&w));
    assert_eq!(last(&w), "The poison leaves you.");
    rested(&mut w);
    run(&mut w, "use:orison");
    assert!(full_hp(&w));
}

#[test]
fn a_brewing_disease_alone_is_nothing_and_goes_with_a_shown_one() {
    let mut w = priest(12);
    run(&mut w, "infect:the_shakes");
    run(&mut w, "use:orison");
    assert_eq!(last(&w), "You have nothing to pray for.");
    assert_eq!(strain(&w), None);
    // A second, shown: both go, and neither is named.
    run(&mut w, "infect:filth_fever");
    let fever = w.content().disease_id("filth_fever").expect("a disease");
    w.player_mut()
        .diseases
        .get_mut(fever)
        .expect("caught")
        .shown = true;
    run(&mut w, "use:orison");
    assert!(w.state.player().diseases.is_empty());
    assert_eq!(last(&w), "The sickness leaves you.");
}

#[test]
fn benediction_answers_two_and_miracle_all_and_every_pool() {
    let mut w = priest(40);
    hurt(&mut w);
    run(&mut w, "afflict:poisoned afflict:ill_wished");
    w.player_mut().mana = 1;
    run(&mut w, "use:orison");
    assert!(!has(&w, "poisoned") && full_hp(&w), "poison, then wounds");
    assert!(has(&w, "ill_wished"), "not a third");
    assert!(w.state.player().mana < 5, "nor a fourth");
    let mut w = priest(60);
    hurt(&mut w);
    hungry(&mut w);
    run(&mut w, "afflict:poisoned afflict:webbed");
    w.player_mut().mana = 1;
    let stamina = w.content().resource_id("stamina").expect("stamina");
    let max = w.resource_max(w.state.player, stamina);
    if w.state.player().resources.has(stamina) {
        w.player_mut().resources.set(stamina, 0);
    }
    run(&mut w, "use:orison");
    let me = w.state.player();
    assert!(!has(&w, "poisoned") && !has(&w, "webbed") && full_hp(&w));
    assert_eq!(me.mana, me.max_mana(w.content()));
    assert_eq!(hunger(&w), crate::hunger::Hunger::Satiated);
    if let Some(pool) = me.resources.get(stamina) {
        assert_eq!(pool.value, max, "every pool filled");
    }
    let names: Vec<String> = dump::layer_rows(&w, Layer::Abilities);
    assert!(
        names.iter().any(|r| r.starts_with("Miracle (class)")),
        "{names:?}"
    );
}

#[test]
fn the_strain_passes_by_the_clock_however_it_is_cut() {
    let rules = BUILTIN.prayer().expect("rules");
    let at = crate::time::GameTime::from_seconds(1000);
    let s = Strain {
        value: 180,
        at,
        locked: false,
    };
    assert_eq!(s.now(rules, at + Duration::seconds(30 * 60 + 59)), 150);
    assert_eq!(s.now(rules, at + Duration::seconds(180 * 60)), 0);
    // A prayer half an hour after another finds 70, in one wait or in many.
    let pray_after = |cuts: &str| {
        let mut w = priest(1);
        hurt(&mut w);
        run(&mut w, "use:orison");
        run(&mut w, cuts);
        let time = w.state.time;
        strain(&w).expect("strained").now(rules, time)
    };
    assert_eq!(
        pray_after("rest:30m"),
        pray_after("rest:10m rest:10m rest:10m")
    );
    assert_eq!(pray_after("rest:30m"), 70);
}

#[test]
fn a_strained_prayer_fizzles_by_its_chance_and_locks() {
    for strain_found in [100u16, 500] {
        for seed in 0..12u64 {
            let mut w = priest(1);
            w.state.seed = seed;
            hurt(&mut w);
            let at = w.state.time;
            w.state.progress.prayer = Some(Strain {
                value: strain_found,
                at,
                locked: false,
            });
            let mut mirror = crate::test_support::peek(&w, "orison/fizzle/player");
            let fizzles = mirror.below(100) < u32::from(strain_found) / 10;
            let before = draws(&w);
            run(&mut w, "use:orison");
            assert_eq!(draws(&w), before + 1, "one draw");
            let s = strain(&w).expect("strained");
            assert_eq!(s.value, strain_found + 100);
            assert_eq!((s.locked, !full_hp(&w)), (fizzles, fizzles));
            if fizzles {
                assert_eq!(last(&w), "You pray, but nothing answers.");
            }
        }
    }
}

#[test]
fn a_locked_prayer_fizzles_without_a_draw_until_the_strain_is_gone() {
    let mut w = priest(1);
    hurt(&mut w);
    let at = w.state.time;
    w.state.progress.prayer = Some(Strain {
        value: 20,
        at,
        locked: true,
    });
    let rng = draws(&w);
    run(&mut w, "use:orison");
    assert_eq!(last(&w), "You pray, but nothing answers.");
    assert_eq!(draws(&w), rng, "locked: no draw");
    assert!(!full_hp(&w));
    let s = strain(&w).expect("strained");
    assert_eq!((s.value, s.locked), (120, true));
    // Two hours later the strain has passed and the lock with it.
    w.state.time += Duration::seconds(120 * 60);
    let rng = draws(&w);
    run(&mut w, "use:orison");
    assert!(full_hp(&w), "answered");
    assert_eq!(draws(&w), rng, "no strain: no draw");
    assert_eq!(strain(&w).map(|s| s.locked), Some(false));
}

#[test]
fn the_band_picks_the_face_and_its_titles() {
    let row = |w: &World| {
        let names = ["Orison", "Whisper", "Prayer", "Invocation"];
        (dump::layer_rows(w, Layer::Abilities).into_iter())
            .find(|r| names.iter().any(|n| r.starts_with(&format!("{n} (class)"))))
            .unwrap_or_default()
    };
    let mut w = priest(1);
    assert!(row(&w).starts_with("Orison (class)"), "{}", row(&w));
    w.player_mut().alignment = Some(Score::new(-150));
    assert!(row(&w).starts_with("Whisper (class)"), "{}", row(&w));
    hurt(&mut w);
    run(&mut w, "use:orison");
    let lines = said(&w);
    assert!(
        lines.contains(&"You whisper to the dark.".to_string()),
        "{lines:?}"
    );
    assert_eq!(last(&w), "Your wounds close, cold to the touch.");
    let mut w = priest(12);
    assert!(row(&w).starts_with("Prayer (class)"), "{}", row(&w));
    w.player_mut().alignment = Some(Score::new(-800));
    assert!(row(&w).starts_with("Invocation (class)"), "{}", row(&w));
    run(&mut w, "use:orison");
    assert_eq!(last(&w), "You have nothing to ask of the dark.");
}

#[test]
fn a_silenced_priest_cannot_pray() {
    let mut w = priest(1);
    hurt(&mut w);
    run(&mut w, "afflict:silenced");
    let turn = w.state.turn;
    run(&mut w, "use:orison");
    assert_eq!(w.state.turn, turn, "refused");
    assert!(!full_hp(&w));
    assert_eq!(strain(&w), None);
}

#[test]
fn prayers_save_and_replay_the_same() {
    let spec: crate::character::CharacterSpec = "human,female,priest".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let actions = "afflict:bleeding*3 wait*20 use:orison afflict:bleeding*3 wait*20 use:orison \
                   use:orison";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(strain(&w).is_some());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let progress = dump::layer_rows(&w, Layer::Progress);
    assert!(
        progress.iter().any(|r| r.starts_with("prayer strain ")),
        "{progress:?}"
    );
}
