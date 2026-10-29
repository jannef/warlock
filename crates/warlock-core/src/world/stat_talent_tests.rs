//! The stat training talents and class abilities (spec §8 "Stats"; the design's §9): Gifted
//! (a stat chosen, its rise 15 % cheaper), Late bloomer (the ramp a point shorter), Ascetic (a
//! hungry dawn steels the will and starves nothing), Hard living (the barbarian's: no hunger
//! deed), and Discipline (the monk's: enduring and praying twice over, a faint free), each read
//! at its one hook.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{Factor, PriceEase, StatDeed, StatUse};
use crate::dump::{self, Layer};
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
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
    w.daily_stat_check(player, next);
}

/// Grants the XP that takes the player to `level`, spending every choice due on the way.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
    while world.due().is_some() {
        run(world, "levelup:auto");
    }
}

fn holds(world: &World, id: &str) -> bool {
    world.state.player().talents.has(id, 1)
}

fn give(world: &mut World, id: &str, choice: Option<&str>) {
    world.player_mut().talents.take(id, choice);
}

#[test]
fn gifted_is_chosen_for_a_stat_and_eases_only_its_rise() {
    let mut w = world("warrior");
    // Level 2, then level 3's pick due: Gifted is offered for each of the six stats.
    reach(&mut w, 2);
    let spec = w.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(w.content(), spec);
    let more = w.content().experience().threshold_for(3, factor) - w.player_xp();
    run(&mut w, &format!("xp:{more}"));
    w.advance_auto();
    let progress = dump::ascii(&w, &[Layer::Progress]);
    assert!(
        progress.contains("gifted:{strength,dexterity,toughness,intelligence,willpower,presence}"),
        "{progress}"
    );
    run(&mut w, "talent:gifted:luck");
    assert!(!holds(&w, "gifted"), "no such stat");
    run(&mut w, "talent:gifted:strength");
    assert_eq!(
        w.state.player().talents.rank_of("gifted", Some("strength")),
        1
    );
    let mods = w.state.player().talent_mods(w.content()).into_owned();
    assert_eq!(mods.price_percent[Stat::Strength], -15);
    assert_eq!(mods.price_percent[Stat::Dexterity], 0);
    // The price: 120 at 10 below the potential, 102 for the gifted stat; a drop's is plain.
    let rules = w.content().stat_training();
    let ease = mods.price_ease(Stat::Strength);
    assert_eq!(rules.price_for(5, 10, 18, PriceEase::default()), Some(120));
    assert_eq!(rules.price_for(5, 10, 18, ease), Some(102));
    assert_eq!(
        rules.price_for(-5, 10, 18, ease),
        rules.price_for(-5, 10, 18, PriceEase::default())
    );
    // The dev progress line and the check read the same eased price.
    set_stats(&mut w, 18);
    w.player_mut().stat_marks[Stat::Strength] = 60_000;
    w.player_mut().stat_marks[Stat::Dexterity] = 60_000;
    let progress = dump::ascii(&w, &[Layer::Progress]);
    assert!(
        progress.contains("St 60000/102, Dx 60000/120"),
        "{progress}"
    );
    let moved = w.settle(stat_training::Check::Daily);
    assert!(moved.contains(&stat_training::Moved::Rose(Stat::Strength)));
    assert!(moved.contains(&stat_training::Moved::Rose(Stat::Dexterity)));
    assert_eq!(
        marks(&w, Stat::Strength),
        60_000 - 102,
        "the gifted price paid"
    );
    assert_eq!(marks(&w, Stat::Dexterity), 60_000 - 120);
}

#[test]
fn late_bloomer_starts_the_ramp_a_point_later() {
    let mut w = world("warrior");
    give(&mut w, "late_bloomer", None);
    let ease = w
        .state
        .player()
        .talent_mods(w.content())
        .price_ease(Stat::Strength);
    assert_eq!(
        ease,
        PriceEase {
            percent: 0,
            ramp: 1
        }
    );
    let rules = w.content().stat_training();
    // The point reaching the potential: the base either way.
    assert_eq!(rules.eased_price(17, 18, ease), rules.base_price(17));
    // The first past it: ×2 by nature, the base for a late bloomer.
    assert_eq!(rules.price(18, 18), rules.base_price(18) * 2);
    assert_eq!(rules.eased_price(18, 18, ease), rules.base_price(18));
    // The second past it: ×4, ×2 for a late bloomer.
    assert_eq!(rules.eased_price(19, 18, ease), rules.base_price(19) * 2);
    // Gifted on top: the whole, 15 % less.
    let both = PriceEase {
        percent: -15,
        ramp: 1,
    };
    assert_eq!(
        rules.eased_price(19, 18, both),
        rules.base_price(19) * 2 * 85 / 100
    );
}

#[test]
fn ascetic_turns_a_hungry_dawn_into_willpower() {
    let mut w = world("warrior");
    give(&mut w, "ascetic", None);
    // Weak at dawn: the deed takes nothing, the will gains 15.
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 300);
    dawn(&mut w);
    assert_eq!(marks(&w, Stat::Strength), 0);
    assert_eq!(marks(&w, Stat::Toughness), 0);
    assert_eq!(marks(&w, Stat::Willpower), 15);
    // Only Hungry: the will gains 15 too (no deed there for anyone).
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 1000);
    dawn(&mut w);
    assert_eq!(marks(&w, Stat::Willpower), 15);
    // Fed: nothing.
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 3000);
    dawn(&mut w);
    assert_eq!(marks(&w, Stat::Willpower), 0);
    // Other deeds still take their marks.
    w.stat_deed(StatDeed::KnewBetter);
    assert_eq!(marks(&w, Stat::Intelligence), -20);
    // Without the talent a hungry dawn gives no will.
    let mut w = world("warrior");
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 1000);
    dawn(&mut w);
    assert_eq!(marks(&w, Stat::Willpower), 0);
}

#[test]
fn hard_living_comes_with_the_barbarians_eighteenth_level_and_spares_the_hunger_deeds() {
    let mut w = world("barbarian");
    reach(&mut w, 17);
    assert!(!holds(&w, "hard_living"));
    reach(&mut w, 18);
    assert!(holds(&w, "hard_living"));
    assert!(
        holds(&w, "brutal_critical"),
        "beside the level's other ability"
    );
    set_stats(&mut w, 18);
    set_nutrition(&mut w, 300);
    dawn(&mut w);
    for deed in [StatDeed::Starved, StatDeed::Vomited, StatDeed::Fainted] {
        w.stat_deed_times(deed, 10);
    }
    assert_eq!(w.state.player().stat_marks, PerStat::splat(0));
    // Overloaded, knew better, and caught are not hunger's: they still take.
    w.stat_deed(StatDeed::KnewBetter);
    assert_eq!(marks(&w, Stat::Intelligence), -20);
}

#[test]
fn discipline_comes_with_the_monks_twelfth_level_and_doubles_enduring_and_praying() {
    let mut w = world("monk");
    reach(&mut w, 11);
    assert!(!holds(&w, "discipline"));
    reach(&mut w, 12);
    assert!(holds(&w, "discipline"));
    set_stats(&mut w, 18);
    let training = w.state.training.clone();
    w.train_stat(StatUse::Endured);
    assert_eq!(marks(&w, Stat::Willpower), 2);
    w.train_stat(StatUse::Prayed);
    assert_eq!(marks(&w, Stat::Willpower), 4);
    assert_eq!(w.state.training, training, "sure marks draw nothing");
    // A faint costs nothing; other uses are as before.
    w.stat_deed(StatDeed::Fainted);
    assert_eq!(marks(&w, Stat::Willpower), 4);
    w.train_stat(StatUse::Identified);
    assert_eq!(marks(&w, Stat::Intelligence), 1);
    // Without it, one mark a use.
    let mut w = world("monk");
    set_stats(&mut w, 18);
    w.train_stat(StatUse::Endured);
    assert_eq!(marks(&w, Stat::Willpower), 1);
    assert!(
        said(&w).iter().all(|m| !m.contains("mark")),
        "marks are never said"
    );
}
