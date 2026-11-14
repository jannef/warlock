//! The profane gifts' machinery (docs/design/profane-gifts.md §7.3, §7.4; `world/profane.rs`):
//! bestowing a gift, rescinding one, the three relations, the races barred, and a save.

use super::super::World;
use crate::action::Action;
use crate::character::CharacterSpec;
use crate::content::{GiftTier, TalentId};
use crate::stats::{DRAIN_FLOOR, Stat};
use crate::test_support::builtin;

fn world(character: &str) -> World {
    let content = builtin();
    let spec: CharacterSpec = character.parse().expect("a character");
    World::start_run(content, "showcase", Some(1), Some(&spec), None, true).expect("world")
}

fn gift(w: &World, id: &str) -> TalentId {
    w.content.talents().id(id).expect(id)
}

fn holds(w: &World, id: &str) -> bool {
    let me = w.state.player();
    me.talents.has(id) && me.boons.iter().any(|b| b.id == id)
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn give(w: &mut World, id: &str) {
    let turn = w.turn();
    w.apply(&Action::Profane(id.into()));
    assert_eq!(w.turn(), turn, "a dev action takes no time");
}

fn toughness(w: &World) -> (u8, u8) {
    let stats = w.state.player().stats;
    (
        stats.score(Stat::Toughness),
        stats.potential(Stat::Toughness),
    )
}

#[test]
fn a_gift_is_bestowed_with_its_lines_and_held_as_a_boon() {
    let mut w = world("human,male,warrior");
    give(&mut w, "thick_blooded");
    assert!(holds(&w, "thick_blooded"));
    assert!(said(&w, "Your blood runs slow and dark."));
    assert!(said(&w, "You are Thick-blooded."));
    assert!(w.state.player().talent_mods(&w.content).vigour.clotting);
    // Held already: refused.
    let before = w.state.player().boons.len();
    give(&mut w, "thick_blooded");
    assert_eq!(w.state.player().boons.len(), before);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Talents).join("\n");
    assert!(rows.contains("Thick-blooded (profane)"), "{rows}");
    give(&mut w, "hungering");
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Talents).join("\n");
    assert!(rows.contains("Hungering (profane, bane)"), "{rows}");
}

#[test]
fn the_kept_talent_sum_follows_a_gift_bestowed_and_rescinded() {
    let mut w = world("human,male,warrior");
    // The sum is read (and kept) before the gift, so a stale one would show here.
    assert!(!w.state.player().talent_mods(&w.content).vigour.clotting);
    give(&mut w, "thick_blooded");
    assert!(w.state.player().talent_mods(&w.content).vigour.clotting);
    assert!(w.rescind(gift(&w, "thick_blooded")));
    assert!(!w.state.player().talent_mods(&w.content).vigour.clotting);
}

#[test]
fn rescinding_reverses_a_stat_gift_exactly() {
    let mut w = world("human,male,warrior");
    let (score, potential) = toughness(&w);
    let max_hp = w.state.player().max_hp(&w.content);
    give(&mut w, "oak_hearted");
    assert_eq!(toughness(&w), (score + 2, potential + 2));
    assert!(w.state.player().max_hp(&w.content) >= max_hp);
    assert!(w.rescind(gift(&w, "oak_hearted")));
    assert!(!holds(&w, "oak_hearted"));
    assert_eq!(toughness(&w), (score, potential));
    assert_eq!(w.state.player().max_hp(&w.content), max_hp);
    assert!(w.state.player().hp <= max_hp);
    assert!(said(
        &w,
        "Your heart stumbles, and its strength drains away."
    ));
    // Not held: nothing.
    assert!(!w.rescind(gift(&w, "oak_hearted")));
}

#[test]
fn withered_falls_no_lower_than_the_drain_floor() {
    let mut w = world("human,male,warrior");
    let me = w.player_mut();
    me.stats.lessen(Stat::Toughness, 40, DRAIN_FLOOR + 1);
    give(&mut w, "withered");
    assert!(holds(&w, "withered"));
    assert_eq!(toughness(&w).0, DRAIN_FLOOR);
    let me = w.state.player();
    assert!(me.hp >= 1 && me.hp <= me.max_hp(&w.content));
}

#[test]
fn a_replacing_gift_takes_the_lesser_away() {
    let mut w = world("human,male,warrior");
    give(&mut w, "hungering");
    give(&mut w, "wyrms_hunger");
    assert!(!holds(&w, "hungering"));
    assert!(holds(&w, "wyrms_hunger"));
    assert!(said(
        &w,
        "The gnawing in your belly sharpens into something worse."
    ));
    assert!(said(&w, "You are Wyrm's hunger."));
    // Only `excludes` bars a gift from the pool (§7.3): the lesser may be drawn again, and is
    // held beside the greater.
    give(&mut w, "hungering");
    assert!(holds(&w, "hungering") && holds(&w, "wyrms_hunger"));
}

#[test]
fn an_undoer_takes_the_boon_it_opposes_and_nothing_more() {
    let mut w = world("human,male,warrior");
    let start = toughness(&w);
    give(&mut w, "oak_hearted");
    give(&mut w, "withered");
    assert!(!holds(&w, "oak_hearted"));
    assert!(!holds(&w, "withered"));
    assert_eq!(toughness(&w), start);
    assert!(said(
        &w,
        "The withering takes your oak heart, and nothing more."
    ));
    assert!(!said(&w, "You are Withered."));
    // Without the boon, it is taken as any bane.
    give(&mut w, "withered");
    assert!(holds(&w, "withered"));
    assert_eq!(toughness(&w).0, start.0 - 3);
    assert!(said(&w, "You are Withered."));
}

#[test]
fn excluded_gifts_are_out_of_the_pool_both_ways() {
    let mut w = world("human,male,warrior");
    let (open, all) = w.gift_pool(GiftTier::Lesser);
    assert_eq!(open, all);
    give(&mut w, "thick_blooded");
    let content = w.content.clone();
    let thin = content
        .profane()
        .gift(gift(&w, "paper_skinned"))
        .expect("a gift");
    assert!(!w.may_take_gift(thin));
    // Two fewer: the one held and the one it excludes.
    assert_eq!(w.gift_pool(GiftTier::Lesser), (all - 2, all));
    give(&mut w, "paper_skinned");
    assert!(!holds(&w, "paper_skinned"));

    let mut w = world("human,male,warrior");
    give(&mut w, "paper_skinned");
    let thick = content
        .profane()
        .gift(gift(&w, "thick_blooded"))
        .expect("a gift");
    assert!(!w.may_take_gift(thick));
}

#[test]
fn a_gift_is_never_drawn_for_a_race_it_is_barred_to() {
    let w = world("ratfolk,male,warrior");
    let content = w.content.clone();
    let midden = content
        .profane()
        .gift(gift(&w, "midden_gut"))
        .expect("a gift");
    assert!(!w.may_take_gift(midden));
    let w = world("human,male,warrior");
    assert!(w.may_take_gift(midden));
}

#[test]
fn a_save_restores_the_gifts() {
    let mut w = world("human,male,warrior");
    give(&mut w, "oak_hearted");
    give(&mut w, "paper_skinned");
    let loaded = World::from_state(w.content.clone(), w.state().clone()).expect("loaded");
    assert!(holds(&loaded, "oak_hearted") && holds(&loaded, "paper_skinned"));
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(toughness(&loaded), toughness(&w));
}

#[test]
fn the_progress_dump_counts_the_gifts_and_the_pools() {
    let mut w = world("human,male,warrior");
    give(&mut w, "oak_hearted");
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Progress).join("\n");
    assert!(
        rows.contains("profane 1 held; pool lesser 4/4, greater 0/1, dire 2/2"),
        "{rows}"
    );
}
