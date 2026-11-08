//! The rank tables of the necromancy and divination spells (#1267; spec §8 "Spells": "Ranks"):
//! every spell of the part has an entry for each rank from 2 to 10, and a deep rank reaches the
//! caster's own cast: Charnel burst's corpse tears two cells round from rank 7, Identify costs
//! two fifths less and takes half the time at rank 10.

use super::caster_tests::sp;
use super::learn_tests::kitted;
use super::*;
use crate::content::MAX_SPELL_RANK;

const PART: [&str; 14] = [
    "blight",
    "cause_wounds",
    "charnel_burst",
    "flay",
    "grave_breath",
    "grave_feast",
    "grave_spear",
    "hex_of_weakness",
    "identify",
    "leech",
    "reek",
    "rouse_the_dead",
    "see_invisible",
    "sense_the_quick",
];

#[test]
fn every_spell_of_the_part_gains_at_every_rank() {
    let w = kitted("witch");
    for name in PART {
        let spell = w.content.spell(sp(name));
        let ranks: Vec<u8> = spell.ranks.steps().iter().map(|(r, _)| *r).collect();
        assert_eq!(ranks, (2..=MAX_SPELL_RANK).collect::<Vec<_>>(), "{name}");
    }
}

#[test]
fn charnel_burst_widens_at_rank_seven() {
    let content = kitted("witch").content().clone();
    let burst = content.spell(sp("charnel_burst"));
    let at = |rank: u8| {
        let mut w = kitted("witch");
        w.player_mut().spells.insert(sp("charnel_burst"), rank);
        w
    };
    let radius = |w: &World| w.state.player().spell_reach(&content, burst).radius;
    assert_eq!(radius(&at(1)), Some(1));
    assert_eq!(radius(&at(6)), Some(1));
    assert_eq!(radius(&at(7)), Some(2));
    // Rank 10: +4 on the roll, a die more, 2s and 1s rolled again.
    let deep = at(MAX_SPELL_RANK);
    let mods = deep.state.player().rank_mods(&content, burst);
    assert_eq!((mods.bonus, mods.dice, mods.reroll), (4, 1, 2));
}

#[test]
fn identify_grows_cheaper_and_quicker() {
    let content = kitted("witch").content().clone();
    let identify = content.spell(sp("identify"));
    let at = |rank: u8| {
        let mut w = kitted("witch");
        w.player_mut().spells.insert(sp("identify"), rank);
        w
    };
    let mods = at(MAX_SPELL_RANK)
        .state
        .player()
        .rank_mods(&content, identify);
    assert_eq!((mods.cost, mods.time), (-40, -50));
    let base = |w: &World| w.state.player().spell_base(&content, identify);
    assert!(base(&at(MAX_SPELL_RANK)) < base(&at(1)));
}
