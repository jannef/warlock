//! The transmutation spells' ranks (#1268): every one has a table, a buff's or a hex's degree
//! ranks are not lost to its status's max at the Presence that tops its ladder, and the ranks
//! cut the mana and widen the reach as written.

use super::caster_tests::sp;
use super::spell_type_tests::{files, start};
use super::*;
use crate::content::SpellData;
use crate::stats::{PerStat, Stat, Stats};

const SPELLS: [&str; 12] = [
    "barkskin",
    "blink",
    "bramble",
    "brand",
    "cloudburst",
    "curse_of_sloth",
    "fume",
    "haste",
    "nightsight",
    "slick",
    "water_walking",
    "web",
];

/// A caster with Presence `presence`, every transmutation spell known at `rank`.
fn caster(presence: u8, rank: u8) -> World {
    let mut w = start(Arc::new(Content::load(&files()).expect("content")), 7);
    let mut stats = PerStat::splat(14);
    stats[Stat::Presence] = presence;
    w.player_mut().stats = Stats::new(stats, PerStat::splat(40));
    for spell in SPELLS {
        w.player_mut().spells.insert(sp(spell), rank);
    }
    w
}

fn data<'a>(w: &'a World, spell: &str) -> &'a SpellData {
    w.content().spell(sp(spell))
}

#[test]
fn every_transmutation_spell_has_ranks_to_ten() {
    let w = caster(14, 1);
    for spell in SPELLS {
        let ranks = &data(&w, spell).ranks;
        let steps: Vec<u8> = ranks.steps().iter().map(|(r, _)| *r).collect();
        assert_eq!(steps, (2..=10).collect::<Vec<u8>>(), "{spell}");
    }
}

#[test]
fn degree_ranks_reach_past_the_old_max_of_three() {
    // Presence 18 tops the buffs' ladders at 3 degrees; rank 10 adds 3 more, which the
    // statuses' max of 6 now holds (#1268).
    for spell in [
        "barkskin",
        "brand",
        "haste",
        "water_walking",
        "curse_of_sloth",
    ] {
        let low = caster(18, 1);
        let high = caster(18, 10);
        let me = |w: &World| w.state.player().spell_degrees(w.content(), data(w, spell));
        assert_eq!(me(&low), Some(3), "{spell} at rank 1");
        assert_eq!(me(&high), Some(6), "{spell} at rank 10");
        let (status, _) = data(&high, spell).effect.status.expect("a status");
        assert_eq!(
            high.content().status(status).max,
            6,
            "{spell}'s status holds 6"
        );
    }
    // Nightsight has no ladder: one degree (half an hour), four at rank 10.
    let me = |w: &World| (w.state.player()).spell_degrees(w.content(), data(w, "nightsight"));
    assert_eq!(me(&caster(18, 1)), Some(1));
    assert_eq!(me(&caster(18, 10)), Some(4));
}

#[test]
fn ranks_cut_the_mana_and_widen_the_reach() {
    let low = caster(14, 1);
    let high = caster(14, 10);
    let cost = |w: &World, spell: &str| w.state.player().spell_cost(w.content(), data(w, spell));
    // The floor's 4 a half off, Haste's 11 by 35 %, Cloudburst's 20 by half.
    assert_eq!((cost(&low, "barkskin"), cost(&high, "barkskin")), (4, 2));
    assert_eq!((cost(&low, "haste"), cost(&high, "haste")), (11, 7));
    assert_eq!(
        (cost(&low, "cloudburst"), cost(&high, "cloudburst")),
        (20, 10)
    );
    let reach = |w: &World, spell: &str| w.state.player().spell_reach(w.content(), data(w, spell));
    let (a, b) = (reach(&low, "slick"), reach(&high, "slick"));
    assert_eq!((a.range, a.radius), (Some(9), Some(1)));
    assert_eq!((b.range, b.radius), (Some(12), Some(2)));
    let (a, b) = (reach(&low, "web"), reach(&high, "web"));
    assert_eq!((a.range, a.radius), (Some(9), Some(1)));
    assert_eq!((b.range, b.radius), (Some(11), Some(2)));
}
