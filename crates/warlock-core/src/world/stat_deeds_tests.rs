//! Deeds that cost stat marks, and the drop (spec §8 "Stats"; the design's §7): each deed takes
//! its marks at its hook, a pile within reach of a drop is warned of in words first, a check
//! drops the stat with its odds and pays the drop's price back, never past zero, never below the
//! floors, and never lowers the potential; noise cancels, and a round trip costs more than it
//! gives.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, StatDeed, TrapId};
use crate::stats::{PerStat, Stat, Stats};
use crate::time::DAWN;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// The built-in content with every training and deed chance at 100, so marks come without a
/// draw.
static SURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let text = include_str!("../../../../assets/data/stats.toml");
    let sure: String = (text.lines())
        .map(|l| match l.starts_with("chance = ") {
            true => "chance = 100\n".to_string(),
            false => format!("{l}\n"),
        })
        .collect();
    let mut files = ContentFiles::builtin();
    files.insert("data/stats.toml", &sure);
    Arc::new(Content::load(&files).expect("content"))
});

fn world_with(content: Arc<Content>) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    World::with_character(content, "showcase", Some(7), Some(&spec)).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn marks(world: &World, stat: Stat) -> i32 {
    world.state.player().stat_marks[stat]
}

/// Every score 10, every potential `potential`; the piles empty.
fn set_stats(world: &mut World, potential: u8) {
    let p = world.player_mut();
    p.stats = Stats::new(PerStat::splat(10), PerStat::splat(potential));
    p.stat_marks = PerStat::splat(0);
}

fn set_nutrition(w: &mut World, n: i32) {
    w.player_mut().satiation = Some(crate::hunger::Satiation::new(n));
}

/// The daily check, made now as if 06:00 had come.
fn dawn(w: &mut World) {
    let player = w.state.player;
    let next = w.state.next_stat_check.expect("a check scheduled");
    assert_eq!(u64::from(next.second_of_day()), DAWN);
    w.daily_stat_check(player, next);
}

#[test]
fn the_data_gives_every_stat_a_deed_and_every_deed_a_rule() {
    let rules = BUILTIN.stat_training();
    let deeds = [
        StatDeed::HungryDawn,
        StatDeed::Starved,
        StatDeed::Vomited,
        StatDeed::Fainted,
        StatDeed::Overloaded,
        StatDeed::KnewBetter,
        StatDeed::Caught,
    ];
    for deed in deeds {
        assert!(rules.deeds(deed).next().is_some(), "{deed:?}");
    }
    for stat in Stat::ALL {
        assert!(rules.deeds.iter().any(|r| r.stat == stat), "{stat:?}");
    }
    assert_eq!(rules.deeds.len(), 8);
}

#[test]
fn a_hungry_dawn_takes_strength_and_toughness_before_the_draws() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 300);
    let training = w.state.training.clone();
    dawn(&mut w);
    assert_eq!(marks(&w, Stat::Strength), -15);
    assert_eq!(marks(&w, Stat::Toughness), -15);
    assert_eq!(marks(&w, Stat::Dexterity), 0);
    // Marks taken at once draw nothing: the check's six draws only.
    let mut six = training;
    for _ in 0..6 {
        six.below(crate::content::ODDS_SCALE);
    }
    assert_eq!(w.state.training, six);
    // Fed, the dawn takes nothing.
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 2000);
    dawn(&mut w);
    assert_eq!(marks(&w, Stat::Strength), 0);
}

#[test]
fn starving_hit_points_cost_toughness_marks() {
    let mut w = world_with(SURE.clone());
    set_stats(&mut w, 18);
    set_nutrition(&mut w, -10);
    let player = w.state.player;
    let (lost, before) = w.digest(player, 1800);
    assert_eq!(before, Some(crate::hunger::Hunger::Starving));
    assert_eq!(lost, 3);
    assert_eq!(marks(&w, Stat::Toughness), -3, "a mark a hit point, sure");
    // At the data's 50 % it draws once a hit point, from the training stream.
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    set_nutrition(&mut w, -10);
    let (rng, training) = (w.state.rng.clone(), w.state.training.clone());
    let (lost, _) = w.digest(player, 6000);
    assert_eq!(lost, 10);
    assert!((-10..=0).contains(&marks(&w, Stat::Toughness)));
    assert_eq!(w.state.rng, rng, "never the gameplay stream");
    let mut drawn = training;
    for _ in 0..10 {
        drawn.chance(50);
    }
    assert_eq!(w.state.training, drawn);
}

#[test]
fn a_faint_costs_willpower_marks() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 100);
    let mut fainted = 0;
    for _ in 0..2000 {
        if w.faint() > 0 {
            fainted += 1;
        }
    }
    // A mark at 20 % a faint.
    let lost = -marks(&w, Stat::Willpower);
    assert!(fainted > 20, "{fainted}");
    assert!(
        lost * 10 > fainted && lost * 3 < fainted,
        "{lost} of {fainted}"
    );
}

#[test]
fn acting_overloaded_costs_dexterity_marks_but_waiting_and_dropping_do_not() {
    let mut w = world_with(SURE.clone());
    set_stats(&mut w, 18);
    let boulder = w.content().object_id("boulder").expect("boulder");
    let p = w.player_mut();
    p.inventory = vec![Stack::new(boulder, 10)];
    p.equipment = Default::default();
    let load = w.load_of(w.state.player);
    assert_eq!(load.burden(), Burden::Overloaded);
    run(&mut w, "wait*3");
    assert_eq!(marks(&w, Stat::Dexterity), 0);
    // Picking things up takes a turn each.
    let content = w.content().clone();
    let pile = ["dagger", "ration"]
        .map(|id| Stack::new(content.object_id(id).expect(id), 1))
        .to_vec();
    let here = w.player_pos();
    w.state.current_mut().set_pile(here, pile);
    let turn = w.state.turn;
    run(&mut w, "pickup:a pickup:a");
    assert!(w.state.turn > turn, "the pickups took time: {:?}", said(&w));
    assert_eq!(marks(&w, Stat::Dexterity), -2);
    // A step refused takes no time and costs nothing.
    run(&mut w, "move:N move:S move:E move:W");
    assert_eq!(marks(&w, Stat::Dexterity), -2);
    run(&mut w, "drop");
    assert_eq!(marks(&w, Stat::Dexterity), -2, "dropping the load is free");
}

#[test]
fn springing_a_known_trap_costs_intelligence_marks_and_a_hidden_one_nothing() {
    let alarm: TrapId = BUILTIN.trap_id("alarm").expect("an alarm trap");
    for known in [false, true] {
        let mut w = world_with(BUILTIN.clone());
        set_stats(&mut w, 18);
        let (player, here) = (w.state.player, w.player_pos());
        let at = here;
        let trap = crate::content::Trap {
            kind: alarm,
            known,
            charges: None,
            dir: None,
        };
        w.state.current_mut().traps.insert(at, trap);
        w.step_onto_trap(at, player, &mut Vec::new());
        let want = if known { -20 } else { 0 };
        assert_eq!(marks(&w, Stat::Intelligence), want, "known {known}");
    }
}

#[test]
fn a_deed_that_brings_a_drop_within_reach_warns_once_in_words() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    // At 10 a drop pays back 55: in play from -14 (a quarter of it).
    let rules = w.content().stat_training().clone();
    assert_eq!(rules.drop_price(10), 55);
    w.stat_deed(StatDeed::Caught);
    let warned = |w: &World| {
        let line = "You feel small and unregarded.";
        said(w).iter().filter(|m| *m == line).count()
    };
    assert_eq!(marks(&w, Stat::Presence), -20);
    assert_eq!(warned(&w), 1);
    w.stat_deed(StatDeed::Caught);
    assert_eq!(marks(&w, Stat::Presence), -40);
    assert_eq!(warned(&w), 1, "once per crossing");
    // Out of reach again, and back: warned again.
    w.player_mut().stat_marks[Stat::Presence] = -5;
    w.say("Between.");
    w.stat_deed(StatDeed::Caught);
    assert_eq!(warned(&w), 2);
    // A positive pile taken down but not within reach says nothing.
    w.player_mut().stat_marks[Stat::Intelligence] = 30;
    w.stat_deed(StatDeed::KnewBetter);
    assert_eq!(marks(&w, Stat::Intelligence), 10);
    assert!(!said(&w).iter().any(|m| m == "Your mind feels dull."));
}

#[test]
fn the_warning_always_precedes_the_drop_and_a_drop_pays_its_price_back() {
    // A character Weak at every dawn: the design's §7.3, at 14 (L 75).
    for seed in 0..40u64 {
        let mut w = world_with(BUILTIN.clone());
        w.state.training = crate::rng::Rng::training(seed);
        let p = w.player_mut();
        p.stats = Stats::new(PerStat::splat(14), PerStat::splat(16));
        p.stat_marks = PerStat::splat(0);
        let mut dawns = 0;
        while w.state.player().stats.score(Stat::Strength) == 14 {
            set_nutrition(&mut w, 300);
            dawn(&mut w);
            dawns += 1;
            assert!(dawns <= 30, "seed {seed}: no drop in a month");
        }
        assert!(dawns >= 2, "never before the second dawn");
        let log = said(&w);
        let warned = log
            .iter()
            .position(|m| m == "You feel your strength wasting.");
        let fell = log.iter().position(|m| m == "You feel weaker.");
        assert!(warned.is_some() && warned < fell, "seed {seed}: {log:?}");
        let p = w.state.player();
        assert_eq!(p.stats.potential(Stat::Strength), 16, "the potential stays");
        let pile = p.stat_marks[Stat::Strength];
        assert!(
            (-75..=0).contains(&pile),
            "paid back, never past zero: {pile}"
        );
    }
}

#[test]
fn a_drop_never_goes_below_the_floors_and_never_kills() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    // At 3 nothing falls, and the pile stays.
    w.player_mut().stats = Stats::new(PerStat::splat(3), PerStat::splat(18));
    w.player_mut().stat_marks[Stat::Dexterity] = -10_000;
    w.check_stats();
    assert_eq!(w.state.player().stats.score(Stat::Dexterity), 3);
    assert_eq!(marks(&w, Stat::Dexterity), -10_000);
    // A score carries its damage: at 6 with 5 damaged, it stays.
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    let p = w.player_mut();
    p.stats = Stats::new(PerStat::splat(6), PerStat::splat(18));
    p.damage.take(Stat::Strength, 5, 6);
    p.stat_marks[Stat::Strength] = -10_000;
    w.check_stats();
    assert_eq!(w.state.player().stats.score(Stat::Strength), 6);
    // A Toughness drop cuts the hit points to the new maximum.
    let mut w = world_with(BUILTIN.clone());
    let p = w.player_mut();
    p.stats = Stats::new(PerStat::splat(30), PerStat::splat(30));
    p.stat_marks = PerStat::splat(0);
    p.stat_marks[Stat::Toughness] = -100_000;
    let content = w.content().clone();
    let p = w.player_mut();
    p.hp = p.max_hp(&content);
    let before = p.hp;
    w.check_stats();
    let p = w.state.player();
    assert_eq!(
        p.stats.score(Stat::Toughness),
        29,
        "a level's check drops it"
    );
    assert_eq!(p.stats.potential(Stat::Toughness), 30);
    assert!(p.hp <= p.max_hp(&content) && p.hp >= 1 && p.hp <= before);
    assert!(w.news.iter().any(|m| m == "You feel frailer."));
}

#[test]
fn noise_cancels() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    for _ in 0..200 {
        w.player_mut().stat_marks[Stat::Presence] += 20;
        w.check_stats();
        w.player_mut().stat_marks[Stat::Presence] -= 20;
        w.check_stats();
    }
    assert_eq!(w.state.player().stats.score(Stat::Presence), 10);
}

#[test]
fn a_round_trip_costs_more_than_it_gives() {
    let rules = BUILTIN.stat_training();
    // Losing a point at 14 costs 75 marks of deeds; winning it back 150 of use.
    assert_eq!(rules.drop_price(14), 75);
    assert_eq!(rules.price(13, 16), 150);
    // A drop at its price: the pile pays all of it and stands at zero.
    let mut w = world_with(BUILTIN.clone());
    let p = w.player_mut();
    p.stats = Stats::new(PerStat::splat(14), PerStat::splat(16));
    p.stat_marks = PerStat::splat(0);
    let mut checks = 0;
    while w.state.player().stats.score(Stat::Willpower) == 14 {
        w.player_mut().stat_marks[Stat::Willpower] = -75;
        w.check_stats();
        checks += 1;
        assert!(checks < 100);
    }
    let p = w.state.player();
    assert_eq!(p.stat_marks[Stat::Willpower], 0);
    assert_eq!(p.stats.potential(Stat::Willpower), 16);
    assert_eq!(rules.price(13, 16), 150, "regained at the plain price");
}
