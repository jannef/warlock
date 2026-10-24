//! Stat training in the world (spec §8 "Stats"): uses earn the player marks (through the call
//! skills share too), long actions done to their end and blows given and taken among them, and
//! each level checks every stat once on the levels stream: a rise uses up the marks, says so in
//! words, may pass the potential, and brings Toughness's hit point; nothing without the data.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Blow, Strike};
use crate::content::{ContentFiles, Factor, StatUse};
use crate::dump::{self, Layer};
use crate::skills::Use;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// The built-in content with every training chance at 100, so marks come without a draw.
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

fn marks(world: &World) -> PerStat<u16> {
    world.state.player().stat_marks
}

fn set_marks(world: &mut World, stat: Stat, n: u16) {
    world.player_mut().stat_marks[stat] = n;
}

/// Every score 10, every potential `potential`.
fn set_stats(world: &mut World, potential: u8) {
    world.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(potential));
}

/// Grants the XP that takes the player to `level`: the stat checks of each level are made then,
/// and its talent pick and skill rolls (which draw from the same stream) wait.
fn grant(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
}

/// [`grant`], then spends what the levels brought.
fn reach(world: &mut World, level: u8) {
    grant(world, level);
    if world.due().is_some() {
        run(world, "levelup:auto");
    }
}

#[test]
fn uses_earn_marks_through_their_own_report_and_the_one_skills_share() {
    let mut w = world_with(SURE.clone());
    assert_eq!(marks(&w), PerStat::splat(0));
    w.train_stat(StatUse::Blow);
    w.train_stat(StatUse::Read);
    w.train_stat(StatUse::Read);
    // A kick is Athletics' use and Strength's: one report trains both.
    assert!(w.trains(Use::Kick));
    w.train(Use::Kick);
    let m = marks(&w);
    assert_eq!(m[Stat::Strength], 2);
    assert_eq!(m[Stat::Intelligence], 2);
    assert_eq!(m[Stat::Dexterity], 0);
    // Hearing something new trains no stat.
    w.train(Use::Hear);
    assert_eq!(marks(&w), m);
}

#[test]
fn marks_draw_from_the_training_stream_and_never_from_gameplay() {
    let mut w = world_with(BUILTIN.clone());
    let (rng, training) = (w.state.rng.clone(), w.state.training.clone());
    for _ in 0..200 {
        w.train_stat(StatUse::Blow);
    }
    assert_eq!(w.state.rng, rng, "the gameplay stream is untouched");
    assert_ne!(w.state.training, training);
    let n = marks(&w)[Stat::Strength];
    assert!((1..40).contains(&n), "about 5 % of 200 blows: {n}");
    // A sure rule draws nothing.
    let training = w.state.training.clone();
    w.train_stat(StatUse::Identified);
    assert_eq!(w.state.training, training);
    assert_eq!(marks(&w)[Stat::Intelligence], 1);
}

#[test]
fn blows_given_and_taken_train_strength_dexterity_toughness_and_willpower() {
    let mut w = world_with(SURE.clone());
    let player = w.state.player;
    let other = ActorId(999);
    let hit = |damage, critical| Blow {
        strike: Strike::Hit { critical },
        damage,
        kind: None,
    };
    w.train_fight((player, other), AttackKind::Melee, &hit(3, false));
    w.train_fight((player, other), AttackKind::Ranged, &hit(3, false));
    w.train_fight(
        (player, other),
        AttackKind::Thrown,
        &Blow {
            strike: Strike::Soaked,
            damage: 0,
            kind: None,
        },
    );
    let missed = Blow {
        strike: Strike::Missed { by: None },
        damage: 0,
        kind: None,
    };
    w.train_fight((player, other), AttackKind::Melee, &missed);
    let m = marks(&w);
    assert_eq!((m[Stat::Strength], m[Stat::Dexterity]), (1, 2));
    assert_eq!(m[Stat::Willpower], 0, "not hungry");
    // Taken: a small hit, then a critical.
    w.train_fight((other, player), AttackKind::Melee, &hit(1, false));
    w.train_fight((other, player), AttackKind::Melee, &hit(1, true));
    assert_eq!(marks(&w)[Stat::Toughness], 3);
    // Hungry, a blow that lands trains Willpower too.
    if let Some(s) = w.player_mut().satiation.as_mut() {
        *s = crate::hunger::Satiation::new(0);
    }
    assert!(
        w.player_hunger()
            .is_some_and(|h| h <= crate::hunger::Hunger::Hungry)
    );
    w.train_fight((player, other), AttackKind::Melee, &hit(3, false));
    assert_eq!(marks(&w)[Stat::Willpower], 1);
}

#[test]
fn a_long_action_done_to_its_end_trains_willpower_and_a_single_turn_does_not() {
    let mut w = world_with(SURE.clone());
    run(&mut w, "search");
    assert_eq!(marks(&w)[Stat::Willpower], 0, "one turn of searching");
    run(&mut w, "rest:10m");
    assert_eq!(marks(&w)[Stat::Willpower], 1, "{:?}", said(&w));
}

#[test]
fn a_level_checks_each_stat_and_a_rise_uses_up_its_marks_and_says_so_in_words() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    set_marks(&mut w, Stat::Strength, 60_000);
    set_marks(&mut w, Stat::Intelligence, 1);
    assert!(w.state.levels_rng.is_none());
    grant(&mut w, 2);
    let p = w.state.player();
    assert_eq!(p.stats.score(Stat::Strength), 11);
    assert_eq!(p.stat_marks[Stat::Strength], 0, "a rise uses up the marks");
    for stat in [
        Stat::Dexterity,
        Stat::Toughness,
        Stat::Willpower,
        Stat::Presence,
    ] {
        assert_eq!(p.stats.score(stat), 10, "no marks, no chance: {stat:?}");
    }
    let s = said(&w);
    let welcome = s
        .iter()
        .position(|m| m == "Welcome to level 2.")
        .expect("welcome");
    let stronger = s
        .iter()
        .position(|m| m == "You feel stronger.")
        .expect("stronger");
    assert!(stronger > welcome, "{s:?}");
    assert!(!s.iter().any(|m| m.contains("mark") || m.contains('%')));
    // The levels stream was drawn from: six times, whatever the marks.
    let mut expected = crate::rng::Rng::levels(w.state.seed);
    for _ in 0..6 {
        expected.below(2);
    }
    assert_eq!(w.state.levels_rng.as_ref(), Some(&expected));
    assert_eq!(
        w.state.rng,
        world_with(BUILTIN.clone()).state.rng,
        "no gameplay draw"
    );
}

#[test]
fn a_failed_check_keeps_the_marks_and_several_levels_check_once_each() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    set_marks(&mut w, Stat::Presence, 1);
    grant(&mut w, 4);
    let p = w.state.player();
    // 1 mark against a need of 50: a long shot at each of three levels.
    if p.stats.score(Stat::Presence) == 10 {
        assert_eq!(p.stat_marks[Stat::Presence], 1);
    }
    let mut expected = crate::rng::Rng::levels(w.state.seed);
    for _ in 0..18 {
        expected.below(2);
    }
    assert_eq!(
        w.state.levels_rng.as_ref(),
        Some(&expected),
        "six draws a level"
    );
}

#[test]
fn past_the_potential_the_chance_is_a_quarter_and_a_rise_lifts_the_potential() {
    let mut w = world_with(BUILTIN.clone());
    let rules = w.content().stat_training();
    assert_eq!(rules.odds(20, 14, 14), (20, 4 * (20 + 62)));
    // With marks beyond counting, a stat below its potential rises nearly always, one at it
    // about a quarter of the time.
    let (mut below, mut at) = (0, 0);
    for _ in 0..400 {
        for (potential, risen) in [(18, &mut below), (10, &mut at)] {
            set_stats(&mut w, potential);
            set_marks(&mut w, Stat::Dexterity, 60_000);
            w.check_stats();
            let p = w.state.player();
            if p.stats.score(Stat::Dexterity) == 11 {
                *risen += 1;
                assert_eq!(p.stats.potential(Stat::Dexterity), potential.max(11));
                assert_eq!(p.stat_marks[Stat::Dexterity], 0);
            }
        }
    }
    assert!(below > 390, "{below}");
    assert!((70..130).contains(&at), "{at}");
    assert!(w.news.contains(&"You feel quicker.".to_string()));
}

#[test]
fn a_toughness_rise_brings_its_hit_point_at_once() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    let content = w.content.clone();
    let before = (w.state.player().hp, w.state.player().max_hp(&content));
    let mut level_only = w.clone();
    reach(&mut level_only, 2);
    set_marks(&mut w, Stat::Toughness, 60_000);
    reach(&mut w, 2);
    let p = w.state.player();
    let q = level_only.state.player();
    assert_eq!(p.stats.score(Stat::Toughness), 11);
    assert_eq!(p.max_hp(&content), q.max_hp(&content) + 1);
    assert_eq!(p.hp, q.hp + 1);
    assert!(p.max_hp(&content) > before.1);
}

#[test]
fn without_training_data_nothing_trains_and_a_level_draws_nothing() {
    let mut files = ContentFiles::builtin();
    files.remove("data/stats.toml");
    let content = Arc::new(Content::load(&files).expect("content without stats.toml"));
    let mut w = world_with(content);
    w.train_stat(StatUse::Identified);
    w.train(Use::Kick);
    assert_eq!(marks(&w), PerStat::splat(0));
    grant(&mut w, 2);
    assert!(w.state.levels_rng.is_none());
}

#[test]
fn marks_are_saved_hashed_and_shown_only_in_the_dev_dump() {
    let mut w = world_with(SURE.clone());
    let plain = serde_json::to_string(&w.state).expect("json");
    assert!(!plain.contains("stat_marks"), "left out while none");
    let hash = w.state_hash();
    w.train_stat(StatUse::Blow);
    w.train_stat(StatUse::Hurt);
    assert_ne!(w.state_hash(), hash);
    let json = serde_json::to_string(&w.state).expect("json");
    let back: WorldState = serde_json::from_str(&json).expect("state");
    assert_eq!(back.player().stat_marks, marks(&w));
    let rows = dump::ascii(&w, &[Layer::Progress]);
    assert!(rows.contains("stat marks St 1, To 1"), "{rows}");
    let stats = dump::ascii(&w, &[Layer::Stats]);
    assert!(!stats.contains("mark"), "{stats}");
}

#[test]
fn a_replay_with_marks_and_a_rise_verifies() {
    let mut w = world_with(BUILTIN.clone());
    let actions = "rest:1h search:5 xp:60 levelup:auto wait";
    run(&mut w, actions);
    let mut again = world_with(BUILTIN.clone());
    run(&mut again, actions);
    assert_eq!(w.state_hash(), again.state_hash());
}
