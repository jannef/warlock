//! The mana talents of the Scholar tree (spec §8 "Talents"; #242): Deep well's mana by level,
//! Clear mind's and Meditation's paces, Frugal caster's cut of what abilities cost, Blood magic's
//! trade of hit points for mana, and spending mana as the `cast` use that trains Willpower.

use super::talent::Due;
use super::*;
use crate::ability::{Ability, Source};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::stats::{PerStat, Stat, Stats};
use crate::talent::TalentMods;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// An elf wizard in the showcase, every score 14.
fn world() -> World {
    let spec: CharacterSpec = "elf,female,wizard".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// The player gains experience until they are at character `level`.
fn reach(w: &mut World, level: u8) {
    let step = (w.content().experience().threshold(level) / 200).max(1);
    while w.player_level() < level {
        w.grant_xp(step);
    }
    assert_eq!(w.player_level(), level);
}

/// The player is at character `level` with one talent pick due and nothing else.
fn at_level_with_a_pick(w: &mut World, level: u8) {
    reach(w, level);
    w.state.progress.due.clear();
    w.state.progress.due.push(Due {
        level,
        talents: 1,
        rolls: 0,
        bonus: 0,
    });
}

/// The player takes talent `id` (one rank more), without a level to pick it at.
fn give(w: &mut World, id: &str) {
    w.player_mut().talents.take(id, None);
}

fn mana(w: &World) -> (i32, i32) {
    let me = w.state.player();
    (me.mana_base, me.mana)
}

#[test]
fn deep_well_hands_out_mana_at_the_pick_and_as_levels_come() {
    let mut w = world();
    at_level_with_a_pick(&mut w, 9);
    let (base, now) = mana(&w);
    run(&mut w, "talent:deep_well");
    assert_eq!(w.state.player().talents.rank_of("deep_well", None), 1);
    // Level 9: 9 / 4 = 2, and at least 2.
    assert_eq!(mana(&w), (base + 2, now + 2));

    // A twin without it gains only the levels' own mana from 9 to 12; the well adds 1 more
    // (12 / 4 = 3).
    let mut twin = world();
    at_level_with_a_pick(&mut twin, 9);
    let (twin_base, _) = mana(&twin);
    reach(&mut w, 12);
    reach(&mut twin, 12);
    assert_eq!(mana(&w).0 - (base + 2), mana(&twin).0 - twin_base + 1);
}

#[test]
fn deep_well_asks_intelligence_or_willpower_then_levels() {
    let mut w = world();
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    at_level_with_a_pick(&mut w, 3);
    run(&mut w, "talent:deep_well");
    assert_eq!(w.state.player().talents.rank_of("deep_well", None), 0);
    // Willpower 12 alone will do.
    let mut stats = PerStat::splat(10);
    stats[Stat::Willpower] = 12;
    w.player_mut().stats = Stats::new(stats, PerStat::splat(40));
    run(&mut w, "talent:deep_well");
    assert_eq!(w.state.player().talents.rank_of("deep_well", None), 1);
    // The second rank waits for level 10.
    w.state.progress.due.push(Due {
        level: 3,
        talents: 1,
        rolls: 0,
        bonus: 0,
    });
    run(&mut w, "talent:deep_well");
    assert_eq!(w.state.player().talents.rank_of("deep_well", None), 1);
}

#[test]
fn clear_mind_and_meditation_quicken_the_pace() {
    let mut w = world();
    let id = w.state.player;
    let awake = w.mana_rate(id);
    w.resting = true;
    let resting = w.mana_rate(id);
    give(&mut w, "clear_mind");
    assert_eq!(w.mana_rate(id), resting * 125 / 100);
    give(&mut w, "meditation");
    assert_eq!(
        w.mana_rate(id),
        resting * 3 / 2 * 125 / 100,
        "resting half again"
    );
    w.resting = false;
    assert_eq!(
        w.mana_rate(id),
        awake * 125 / 100,
        "awake, Meditation does nothing"
    );
}

#[test]
fn frugal_caster_costs_a_fifth_less_never_below_one() {
    let frugal = TalentMods {
        mana_cost_percent: -20,
        ..TalentMods::default()
    };
    let costs: Vec<i32> = [0, 1, 2, 3, 5, 6, 10, 12]
        .map(|c| frugal.mana_cost(c))
        .into();
    assert_eq!(costs, [0, 1, 2, 2, 4, 5, 8, 10]);
    assert_eq!(TalentMods::default().mana_cost(6), 6);

    // Spending goes through it: the dev drain stands in for an ability's cost.
    let mut w = world();
    let max = w.state.player().max_mana(w.content());
    w.player_mut().mana = max;
    give(&mut w, "frugal_caster");
    run(&mut w, "drain:10");
    assert_eq!(last(&w), "You spend 8 mana (dev).");
    assert_eq!(w.state.player().mana, max - 8);
}

#[test]
fn spending_mana_trains_willpower() {
    let mut w = world();
    let marks = |w: &World| w.state.player().stat_marks[Stat::Willpower];
    w.player_mut().mana = 1000;
    let player = w.state.player;
    assert!(w.spend_mana(player, 0));
    assert_eq!(marks(&w), 0, "nothing spent, nothing trained");
    for _ in 0..60 {
        assert!(w.spend_mana(player, 1));
    }
    // 20 % a spend: about 12 of 60.
    assert!((3..=30).contains(&marks(&w)), "{} marks", marks(&w));
    let others = (Stat::ALL.iter()).filter(|&&s| s != Stat::Willpower);
    assert!(others.clone().all(|&s| w.state.player().stat_marks[s] == 0));
}

#[test]
fn blood_magic_trades_hit_points_for_mana() {
    let mut w = world();
    let listed = |w: &World| {
        (w.perception().abilities().into_iter()).find(|a| a.ability == Ability::BloodMagic)
    };
    run(&mut w, "use:blood_magic");
    assert_eq!(last(&w), "You know no blood magic.");
    assert!(listed(&w).is_none());

    give(&mut w, "blood_magic");
    let own = listed(&w).expect("listed");
    assert_eq!(
        (own.ability.source(), own.unavailable),
        (Source::Talent, None)
    );
    let max = w.state.player().max_mana(w.content());
    w.player_mut().hp = 20;
    w.player_mut().mana = max - 5;
    let turn = w.state.turn;
    run(&mut w, "use:blood_magic");
    assert_eq!(last(&w), "You give your blood, and power wells up in you.");
    assert_eq!((w.state.player().hp, w.state.player().mana), (15, max - 2));
    assert_eq!(w.state.turn, turn + 1, "it takes a turn");
    // Never above the maximum, and not at all while full.
    run(&mut w, "use:blood_magic");
    assert_eq!((w.state.player().hp, w.state.player().mana), (10, max));
    run(&mut w, "use:blood_magic");
    assert_eq!(last(&w), "Your mana is already full.");
    assert_eq!(w.state.player().hp, 10);
    // Never the last hit point.
    w.player_mut().mana = 0;
    w.player_mut().hp = 5;
    let turn = w.state.turn;
    run(&mut w, "use:blood_magic");
    assert_eq!(last(&w), "You have too little blood left to give.");
    assert_eq!((w.state.player().hp, w.state.player().mana), (5, 0));
    assert_eq!(w.state.turn, turn, "refused: no time");
}
