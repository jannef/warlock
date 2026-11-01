//! Stat training in the world (spec §8 "Stats"): uses earn the player marks (through the call
//! skills share too), long actions done to their end and blows given and taken among them, and
//! each level checks every stat once on the levels stream: a rise pays its price and keeps the
//! surplus, says so in words, may pass the potential (which stays), and brings Toughness's hit
//! point; nothing without the data.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::{Blow, Category, Strike};
use crate::content::{ContentFiles, Factor, ODDS_SCALE, PriceEase, StatUse};
use crate::dump::{self, Layer};
use crate::skills::Use;
use crate::stats::{PerStat, Stat, Stats};
use crate::time::{DAWN, DAY};
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

fn marks(world: &World) -> PerStat<i32> {
    world.state.player().stat_marks
}

fn set_marks(world: &mut World, stat: Stat, n: i32) {
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
    assert!((10..60).contains(&n), "about 15 % of 200 blows: {n}");
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
fn a_level_checks_each_stat_and_a_rise_pays_its_price_and_says_so_in_words() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    set_marks(&mut w, Stat::Strength, 60_000);
    set_marks(&mut w, Stat::Intelligence, 1);
    assert!(w.state.levels_rng.is_none());
    grant(&mut w, 2);
    let p = w.state.player();
    assert_eq!(p.stats.score(Stat::Strength), 11);
    assert_eq!(
        p.stat_marks[Stat::Strength],
        60_000 - 120,
        "a rise pays the price of 10 to 11 and the surplus waits"
    );
    assert_eq!(
        p.stats.score(Stat::Intelligence),
        10,
        "a mark is out of play"
    );
    assert_eq!(p.stat_marks[Stat::Intelligence], 1);
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
        expected.below(ODDS_SCALE);
    }
    assert_eq!(w.state.levels_rng.as_ref(), Some(&expected));
    assert_eq!(
        w.state.rng,
        world_with(BUILTIN.clone()).state.rng,
        "no gameplay draw"
    );
}

#[test]
fn a_pile_out_of_play_keeps_its_marks_and_several_levels_check_once_each() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    // 29 marks against a price of 120: under a quarter, so out of play at every level.
    set_marks(&mut w, Stat::Presence, 29);
    grant(&mut w, 4);
    let p = w.state.player();
    assert_eq!(p.stats.score(Stat::Presence), 10);
    assert_eq!(p.stat_marks[Stat::Presence], 29);
    let mut expected = crate::rng::Rng::levels(w.state.seed);
    for _ in 0..18 {
        expected.below(ODDS_SCALE);
    }
    assert_eq!(
        w.state.levels_rng.as_ref(),
        Some(&expected),
        "six draws a level"
    );
}

#[test]
fn a_short_pile_that_rises_goes_to_zero() {
    let mut w = world_with(BUILTIN.clone());
    set_stats(&mut w, 18);
    // Half the price: about one check in nine rises, and pays all it has.
    let mut rose = 0;
    for _ in 0..200 {
        set_stats(&mut w, 18);
        set_marks(&mut w, Stat::Willpower, 60);
        w.check_stats();
        let p = w.state.player();
        match p.stats.score(Stat::Willpower) {
            11 => {
                rose += 1;
                assert_eq!(p.stat_marks[Stat::Willpower], 0, "no debt");
            }
            _ => assert_eq!(p.stat_marks[Stat::Willpower], 60),
        }
    }
    assert!((8..40).contains(&rose), "about 11 %: {rose}");
}

#[test]
fn past_the_potential_the_price_ramps_and_a_rise_leaves_the_potential() {
    let mut w = world_with(BUILTIN.clone());
    let rules = w.content().stat_training();
    assert_eq!(rules.price(10, 18), 120);
    assert_eq!(
        rules.price(10, 10),
        240,
        "the first point past nature: twice the base"
    );
    // 240 marks: even odds at the potential, nine in ten below it.
    let (mut below, mut at) = (0, 0);
    for _ in 0..400 {
        for (potential, risen) in [(18, &mut below), (10, &mut at)] {
            set_stats(&mut w, potential);
            set_marks(&mut w, Stat::Dexterity, 240);
            w.check_stats();
            let p = w.state.player();
            if p.stats.score(Stat::Dexterity) == 11 {
                *risen += 1;
                assert_eq!(p.stats.potential(Stat::Dexterity), potential);
                let left = if potential == 18 { 120 } else { 0 };
                assert_eq!(p.stat_marks[Stat::Dexterity], left);
            }
        }
    }
    assert!((330..385).contains(&below), "about 89 %: {below}");
    assert!((160..240).contains(&at), "about 50 %: {at}");
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
    set_stats(&mut w, 18);
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
    assert!(rows.contains("stat marks St 1/120, To 1/120"), "{rows}");
    // A negative pile is saved as it is, and shown against the drop's price.
    set_marks(&mut w, Stat::Toughness, -30);
    let json = serde_json::to_string(&w.state).expect("json");
    let back: WorldState = serde_json::from_str(&json).expect("state");
    assert_eq!(back.player().stat_marks[Stat::Toughness], -30);
    let rows = dump::ascii(&w, &[Layer::Progress]);
    assert!(rows.contains("stat marks St 1/120, To -30/55"), "{rows}");
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

#[test]
fn the_raised_chances_and_the_new_uses_are_read_from_data() {
    let rules = BUILTIN.stat_training();
    let chance = |used| {
        (rules.rules(used))
            .map(|r| (r.stat, r.chance))
            .collect::<Vec<_>>()
    };
    assert_eq!(chance(StatUse::Blow), [(Some(Stat::Strength), 15)]);
    assert_eq!(chance(StatUse::Critical), [(Some(Stat::Strength), 40)]);
    assert_eq!(chance(StatUse::Block), [(Some(Stat::Strength), 20)]);
    assert_eq!(chance(StatUse::Dodge), [(Some(Stat::Dexterity), 30)]);
    assert_eq!(chance(StatUse::Parry), [(Some(Stat::Dexterity), 25)]);
    assert_eq!(chance(StatUse::Hurt), [(Some(Stat::Toughness), 15)]);
    assert_eq!(chance(StatUse::Killed), [(Some(Stat::Toughness), 30)]);
    assert_eq!(chance(StatUse::Cast), [(Some(Stat::Willpower), 30)]);
    // A potion's and an elixir's name their stat where they are drunk: sure marks.
    assert_eq!(chance(StatUse::Drank), [(None, 100)]);
    assert_eq!(chance(StatUse::Elixir), [(None, 100)]);
}

#[test]
fn a_critical_up_close_a_shield_block_and_a_parry_train_their_stats() {
    let mut w = world_with(SURE.clone());
    let player = w.state.player;
    let other = ActorId(999);
    let blow = |strike| Blow {
        strike,
        damage: 0,
        kind: None,
    };
    let critical = Blow {
        damage: 4,
        ..blow(Strike::Hit { critical: true })
    };
    w.train_fight((player, other), AttackKind::Melee, &critical);
    assert_eq!(marks(&w)[Stat::Strength], 2, "the blow and its critical");
    // A critical shot is a shot, not a blow up close.
    w.train_fight((player, other), AttackKind::Ranged, &critical);
    assert_eq!(marks(&w)[Stat::Strength], 2);
    let shield = blow(Strike::Missed {
        by: Some(Category::Shield),
    });
    w.train_fight((other, player), AttackKind::Melee, &shield);
    assert_eq!(marks(&w)[Stat::Strength], 3, "the shield's block");
    let parry = blow(Strike::Missed {
        by: Some(Category::Parry),
    });
    w.train_fight((other, player), AttackKind::Melee, &parry);
    assert_eq!(marks(&w)[Stat::Dexterity], 2, "the shot and the parry");
    // The player's own blow blocked or parried trains nothing of theirs.
    let before = marks(&w);
    w.train_fight((player, other), AttackKind::Melee, &shield);
    w.train_fight((player, other), AttackKind::Melee, &parry);
    assert_eq!(marks(&w), before);
}

/// The built-in content with one training rule: a kill up close of a foe of the player's level
/// or above, a sure Toughness mark.
static KILLS: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let text = "[training]\nprice_base = 20\nprice_per_point = 10\nramp_divisor = 4\nodds_power = 3\n\
                quarter = 4\ndrop_share = 2\n\n\
                [[training.use]]\nstat = \"toughness\"\non = \"killed\"\nchance = 100\n";
    let mut files = ContentFiles::builtin();
    files.insert("data/stats.toml", text);
    Arc::new(Content::load(&files).expect("content"))
});

/// The showcase's kobold (level 2).
fn kobold(w: &World) -> ActorId {
    let kobold = (w.state.actors.values())
        .find(|a| w.content.creature(a.creature).id == "kobold")
        .map(|a| a.id)
        .expect("the kobold");
    assert_eq!(
        w.content.creature(w.state.actors[&kobold].creature).level,
        2
    );
    kobold
}

/// Brings the kobold east of the player with one hit point (again after each kick: it may
/// step away), and kicks it until it dies.
fn kick_the_kobold_to_death(w: &mut World) {
    let kobold = kobold(w);
    for _ in 0..40 {
        let at = w.player_pos().step(Dir::E);
        let Some(actor) = w.state.actors.get_mut(&kobold) else {
            return;
        };
        actor.pos = at;
        actor.hp = 1;
        *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
        w.apply(&Action::Kick(Dir::E));
    }
    panic!("the kobold did not die: {:?}", said(w));
}

#[test]
fn a_kill_up_close_of_a_foe_of_the_players_level_or_above_trains_toughness() {
    let mut w = world_with(KILLS.clone());
    assert_eq!(w.player_level(), 1);
    kick_the_kobold_to_death(&mut w);
    assert_eq!(marks(&w)[Stat::Toughness], 1, "{:?}", said(&w));
    // At level 3 the kobold is beneath the player: its death trains nothing.
    let mut w = world_with(KILLS.clone());
    reach(&mut w, 3);
    kick_the_kobold_to_death(&mut w);
    assert_eq!(marks(&w)[Stat::Toughness], 0);
    // Nor does a kill at range.
    let mut w = world_with(KILLS.clone());
    let (player, kobold) = (w.state.player, kobold(&w));
    w.train_kill((player, kobold), AttackKind::Ranged);
    assert_eq!(marks(&w)[Stat::Toughness], 0);
    w.train_kill((player, kobold), AttackKind::Melee);
    assert_eq!(marks(&w)[Stat::Toughness], 1);
}

#[test]
fn a_potion_gives_its_marks_at_once_in_the_stat_it_names_without_a_draw() {
    let mut w = world_with(BUILTIN.clone());
    let training = w.state.training.clone();
    w.train_stat_times(StatUse::Drank, Some(Stat::Strength), 5);
    w.train_stat_times(StatUse::Elixir, Some(Stat::Presence), 20);
    let m = marks(&w);
    assert_eq!((m[Stat::Strength], m[Stat::Presence]), (5, 20));
    assert_eq!(m[Stat::Toughness], 0, "only the stat named");
    assert_eq!(w.state.training, training, "sure marks draw nothing");
    // Without a stat named, a potion's use marks nothing.
    w.train_stat_times(StatUse::Drank, None, 5);
    assert_eq!(marks(&w), m);
    // Marks saturate.
    set_marks(&mut w, Stat::Strength, i32::MAX - 2);
    w.train_stat_times(StatUse::Drank, Some(Stat::Strength), 20);
    assert_eq!(marks(&w)[Stat::Strength], i32::MAX);
}

#[test]
fn a_count_below_a_sure_chance_draws_once_a_mark() {
    let mut w = world_with(BUILTIN.clone());
    let mut expected = w.state.training.clone();
    w.train_stat_times(StatUse::Blow, None, 200);
    let won = (0..200).filter(|_| expected.chance(15)).count() as i32;
    assert_eq!(w.state.training, expected, "a draw a mark");
    assert_eq!(marks(&w)[Stat::Strength], won);
    assert!((10..60).contains(&won), "about 15 % of 200: {won}");
}

/// The built-in content with one training rule that nothing in these tests reaches: the daily
/// checks are the only draws of the training stream.
static DAILY: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let text = "[training]\nprice_base = 20\nprice_per_point = 10\nramp_divisor = 4\n\
                odds_power = 3\nquarter = 4\ndrop_share = 2\n\n[[training.use]]\n\
                stat = \"strength\"\non = \"quest\"\nchance = 50\n";
    let mut files = ContentFiles::builtin();
    files.insert("data/stats.toml", text);
    Arc::new(Content::load(&files).expect("content"))
});

/// A world on `DAILY` whose player never hungers (days pass in these tests).
fn daily_world() -> World {
    let mut w = world_with(DAILY.clone());
    w.player_mut().satiation = None;
    w
}

/// Lives `seconds` of the player's time from now, in pieces of `piece` seconds, and moves the
/// clock on.
fn live_for(w: &mut World, seconds: u64, piece: u64) {
    let player = w.state.player;
    let mut passed = 0;
    while passed < seconds {
        let step = piece.min(seconds - passed);
        let start = w.state.time;
        w.pass_time(std::iter::once(player), start, step, &mut Vec::new());
        w.state.time += Duration::seconds(step);
        passed += step;
    }
}

/// What one daily check does to `w` (its training stream, then the stats and marks), worked
/// out apart: the stats that rise.
fn expected_daily(w: &mut World) -> Vec<Stat> {
    let rules = w.content.stat_training().clone();
    let mut risen = Vec::new();
    for stat in Stat::ALL {
        let actor = w.player_mut();
        let (score, potential) = (actor.stats.score(stat), actor.stats.potential(stat));
        let marks = actor.stat_marks[stat];
        let price = rules.price_for(marks, score, potential, PriceEase::default());
        let price = price.unwrap_or(1);
        let odds = rules.odds(marks.unsigned_abs(), price);
        if w.state.training.below(ODDS_SCALE) < odds && marks > 0 {
            let actor = w.player_mut();
            actor.stats.train(stat);
            actor.stat_marks[stat] = (marks - price as i32).max(0);
            risen.push(stat);
        }
    }
    risen
}

fn seconds_to_check(w: &World) -> u64 {
    let next = w.state.next_stat_check.expect("a check scheduled");
    assert_eq!(u64::from(next.second_of_day()), DAWN, "at 06:00");
    next.seconds() - w.state.time.seconds()
}

/// What a rise in `stat` says.
fn rise_said(stat: Stat) -> String {
    let word = match stat {
        Stat::Strength => "stronger",
        Stat::Dexterity => "quicker",
        Stat::Toughness => "tougher",
        Stat::Intelligence => "sharper",
        Stat::Willpower => "more resolute",
        Stat::Presence => "more confident",
    };
    format!("You feel {word}.")
}

#[test]
fn the_daily_check_comes_at_six_and_only_then() {
    let mut w = daily_world();
    set_stats(&mut w, 20);
    for stat in Stat::ALL {
        set_marks(&mut w, stat, 120);
    }
    let first = w.state.next_stat_check.expect("scheduled at the start");
    assert_eq!(first, w.state.time.next_at(DAWN));
    let to = seconds_to_check(&w);
    let training = w.state.training.clone();
    live_for(&mut w, to - 1, to);
    assert_eq!(w.state.training, training, "nothing drawn before 06:00");
    assert_eq!(marks(&w), PerStat::splat(120));
    let mut expected = w.clone();
    let risen = expected_daily(&mut expected);
    live_for(&mut w, 1, 1);
    assert_eq!(
        w.state.training, expected.state.training,
        "six draws at 06:00"
    );
    assert_eq!(w.state.player().stats, expected.state.player().stats);
    assert_eq!(
        marks(&w),
        marks(&expected),
        "a rise pays its price, a failure keeps the marks"
    );
    assert!(
        !risen.is_empty() && risen.len() < 6,
        "some rose, some did not: {risen:?}"
    );
    let news: Vec<_> = risen.iter().map(|&s| rise_said(s)).collect();
    assert!(said(&w).ends_with(&news), "{:?}", said(&w));
    assert_eq!(
        w.state.next_stat_check,
        Some(first + Duration::seconds(DAY)),
        "the next a day later"
    );
    // The rest of the day draws nothing.
    let training = w.state.training.clone();
    live_for(&mut w, DAY - 1, 3600);
    assert_eq!(w.state.training, training);
}

#[test]
fn a_week_makes_seven_checks_in_order_however_the_time_is_cut() {
    let mut whole = daily_world();
    set_stats(&mut whole, 12);
    for stat in Stat::ALL {
        set_marks(&mut whole, stat, 150);
    }
    let mut cut = whole.clone();
    let mut expected = whole.clone();
    let week = 7 * DAY;
    live_for(&mut whole, week, week);
    live_for(&mut cut, week, 3 * 3600 + 17 * 60 + 5);
    let mut risen = 0;
    for _ in 0..7 {
        risen += expected_daily(&mut expected).len();
    }
    assert!(risen > 0);
    for w in [&whole, &cut] {
        assert_eq!(w.state.training, expected.state.training, "42 draws");
        assert_eq!(w.state.player().stats, expected.state.player().stats);
        assert_eq!(marks(w), marks(&expected));
    }
    assert_eq!(whole.state_hash(), cut.state_hash());
}

#[test]
fn the_daily_check_is_the_levels_rule_and_hard_training_speeds_the_marks() {
    // At the potential the price ramps as at a level.
    let mut w = daily_world();
    set_stats(&mut w, 10);
    for stat in Stat::ALL {
        set_marks(&mut w, stat, 180);
    }
    let mut expected = w.clone();
    let to = seconds_to_check(&w);
    live_for(&mut w, to, to);
    expected_daily(&mut expected);
    assert_eq!(w.state.player().stats, expected.state.player().stats);
    assert_eq!(marks(&w), marks(&expected));
    assert_eq!(w.state.training, expected.state.training);
    // Hard training: every use's chance half again; a sure one stays sure and draws nothing.
    let mut w = world_with(BUILTIN.clone());
    w.player_mut().talents.take("hard_training", None);
    let mods = w.state.player().talent_mods(&w.content);
    assert_eq!(mods.train_percent, 50);
    let mut expected = w.state.training.clone();
    w.train_stat_times(StatUse::Blow, None, 200);
    let won = (0..200).filter(|_| expected.chance(22)).count() as i32;
    assert_eq!(
        w.state.training, expected,
        "a draw a mark, as without the talent"
    );
    assert_eq!(marks(&w)[Stat::Strength], won, "15 % half again: 22 %");
    let training = w.state.training.clone();
    w.train_stat(StatUse::Identified);
    assert_eq!(w.state.training, training);
    assert_eq!(marks(&w)[Stat::Intelligence], 1);
}

#[test]
fn a_rise_at_six_stops_a_rest() {
    let mut w = daily_world();
    set_stats(&mut w, 20);
    set_marks(&mut w, Stat::Strength, 60_000);
    let next = w.state.next_stat_check.expect("scheduled");
    let mut expected = w.clone();
    let risen = expected_daily(&mut expected);
    assert_eq!(risen, [Stat::Strength], "near certain");
    run(&mut w, "rest:24h");
    let log = said(&w);
    let rose = log.iter().position(|m| m == "You feel stronger.");
    let stopped = log
        .iter()
        .position(|m| m.starts_with("You stop resting after"));
    assert!(rose.is_some() && rose < stopped, "{log:?}");
    assert!(w.state.time >= next && w.state.time < next + Duration::minutes(1));
    assert_eq!(w.state.player().stats.score(Stat::Strength), 11);
}

#[test]
fn the_next_check_is_saved_scheduled_on_load_and_absent_without_the_data() {
    let w = daily_world();
    let json = serde_json::to_string(&w.state).expect("json");
    assert!(json.contains("next_stat_check"));
    let restored: WorldState = serde_json::from_str(&json).expect("state");
    assert_eq!(restored.next_stat_check, w.state.next_stat_check);
    // A save from before the check: the next 06:00 is scheduled on load.
    let mut old = w.state.as_ref().clone();
    old.next_stat_check = None;
    let loaded = World::from_state(w.content.clone(), old).expect("loads");
    assert_eq!(
        loaded.state.next_stat_check,
        Some(w.state.time.next_at(DAWN))
    );
    // Without stats.toml nothing is scheduled, saved, or drawn.
    let mut files = ContentFiles::builtin();
    files.remove("data/stats.toml");
    let content = Arc::new(Content::load(&files).expect("content without stats.toml"));
    let mut w = world_with(content);
    assert_eq!(w.state.next_stat_check, None);
    let json = serde_json::to_string(&w.state).expect("json");
    assert!(!json.contains("next_stat_check"));
    w.player_mut().satiation = None;
    let training = w.state.training.clone();
    live_for(&mut w, 2 * DAY, 3600);
    assert_eq!(w.state.training, training);
}

#[test]
fn a_dev_skip_passes_no_check() {
    let mut w = daily_world();
    let next = w.state.next_stat_check.expect("scheduled");
    let training = w.state.training.clone();
    run(&mut w, "skip:24h");
    assert_eq!(w.state.training, training);
    assert_eq!(w.state.next_stat_check, Some(w.state.time.next_at(DAWN)));
    assert!(w.state.next_stat_check > Some(next));
}
