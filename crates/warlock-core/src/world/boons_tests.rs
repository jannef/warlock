//! A forgotten altar's great boons and banes (spec §7 "Special rooms"; `world/boons.rs`): Gift
//! and Wrath of the forgotten, Exalted and Unmade (a level gained again gives no choices),
//! Chosen and Forsaken (kept for life through a save), the relic, the horde, Penitent, and the
//! great tier's share of the draw.

use super::super::World;
use crate::Pos;
use crate::action::Action;
use crate::content::Content;
use crate::content::{ContentFiles, OutcomeDef, OutcomeKind, OutcomeTier};
use crate::magic::Rarity;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::Arc;

/// A room with a forgotten altar at (5, 2), the player beside it at (4, 3).
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@.....#
#.........#
#.........#
###########
";

const ALTAR: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`] (a neutral altar), and `table` in place of the altar's
/// outcome table when given.
fn world_with(table: Option<&str>) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_altar.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"altar\"\narea = [0, 0, 10, 6]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"forgotten_altar\"\n"
        ),
    );
    files.insert(
        "scenarios/test_altar.toml",
        "name = \"T\"\nstart_level = \"test_altar\"\n\
         character = { race = \"human\", gender = \"female\", class = \"warrior\" }\n",
    );
    if let Some(table) = table {
        files.insert("data/outcomes/forgotten_altar.toml", table);
    }
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_altar", None).expect("world")
}

fn world() -> World {
    world_with(None)
}

fn outcome(w: &World, id: &str) -> OutcomeDef {
    (w.content
        .outcomes()
        .get("forgotten_altar")
        .expect("table")
        .outcomes)
        .iter()
        .find(|o| o.id == id)
        .expect(id)
        .clone()
}

fn befall(w: &mut World, id: &str) {
    let outcome = outcome(w, id);
    w.befall(&outcome, ALTAR, &mut Vec::new());
    w.say_news();
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn scores(w: &World) -> (u32, u32) {
    let stats = w.state.player().stats;
    Stat::ALL.iter().fold((0, 0), |(s, p), &stat| {
        (
            s + u32::from(stats.score(stat)),
            p + u32::from(stats.potential(stat)),
        )
    })
}

fn talent_rows(w: &World) -> String {
    crate::dump::layer_rows(w, crate::dump::Layer::Talents).join("\n")
}

#[test]
fn the_great_outcomes_are_in_the_table_six_in_each_kind() {
    let w = world();
    let table = w.content.outcomes().get("forgotten_altar").expect("table");
    let great = |kind| {
        (table.of(kind))
            .filter(|o| o.tier == OutcomeTier::Great)
            .map(|o| (o.id.as_str(), o.weight))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        great(OutcomeKind::Boon),
        [("gift", 2), ("exalted", 1), ("chosen", 1), ("relic", 2)]
    );
    assert_eq!(
        great(OutcomeKind::Bane),
        [("wrath", 1), ("unmade", 1), ("forsaken", 2), ("damned", 2)]
    );
    for kind in OutcomeKind::ALL {
        assert_eq!(table.total(kind), 100, "{kind:?}");
    }
}

#[test]
fn the_great_tier_is_about_four_prayers_in_a_hundred() {
    let mut w = world();
    let table = w
        .content
        .outcomes()
        .get("forgotten_altar")
        .expect("table")
        .clone();
    let n = 40_000;
    let great = (0..n)
        .filter(|_| w.draw_outcome(&table, table.kinds).expect("drawn").tier == OutcomeTier::Great)
        .count() as u32;
    // 35 % × 6 % twice: 4.2 % (per mille, within 6).
    assert!((great * 1000 / n).abs_diff(42) <= 6, "{great}");
}

#[test]
fn gift_raises_a_stat_and_its_potential_for_good() {
    let mut w = world();
    let (score, potential) = scores(&w);
    befall(&mut w, "gift");
    assert!(said(&w, "You feel changed."));
    let (s, p) = scores(&w);
    assert!((1..=3).contains(&(s - score)), "{score} -> {s}");
    assert_eq!(s - score, p - potential);
}

#[test]
fn wrath_lowers_a_stat_and_its_potential_never_below_three() {
    let mut w = world();
    let (score, potential) = scores(&w);
    befall(&mut w, "wrath");
    assert!(said(&w, "You feel lessened, forever."));
    let (s, p) = scores(&w);
    assert!((1..=3).contains(&(score - s)), "{score} -> {s}");
    assert!((1..=3).contains(&(potential - p)), "{potential} -> {p}");
    // At the floor everywhere: nothing falls, and the other line is said.
    let mut w = world();
    w.player_mut().stats = Stats::new(PerStat::splat(3), PerStat::splat(3));
    befall(&mut w, "wrath");
    assert_eq!(scores(&w), (18, 18));
    assert!(said(
        &w,
        "Something reaches for you, and finds too little to take."
    ));
}

#[test]
fn wrath_keeps_stat_damage_and_hit_points_within_what_is_left() {
    let mut w = world();
    let content = w.content.clone();
    let me = w.player_mut();
    me.stats = Stats::new(PerStat::splat(5), PerStat::splat(5));
    for stat in Stat::ALL {
        me.damage.take(stat, 4, 5);
    }
    let max = me.max_hp(&content);
    w.player_mut().hp = max;
    befall(&mut w, "wrath");
    let me = w.state.player();
    assert!(me.damage.is_valid_for(&me.stats));
    assert!(me.hp <= me.max_hp(&w.content));
}

#[test]
fn exalted_gives_a_whole_level_with_its_choices() {
    let mut w = world();
    assert_eq!(w.player_level(), 1);
    befall(&mut w, "exalted");
    assert!(said(&w, "Power floods into you."));
    assert_eq!(w.player_level(), 2);
    let factor = w.experience_factor();
    assert_eq!(
        w.player_xp(),
        w.content.experience().threshold_for(2, factor),
        "exactly the threshold"
    );
    assert_eq!(w.due().map(|d| d.level), Some(2), "its skill rolls are due");
}

#[test]
fn unmade_takes_a_level_back_and_the_level_regained_gives_no_choices() {
    let mut w = world();
    let factor = w.experience_factor();
    w.add_xp(w.content.experience().threshold_for(3, factor));
    w.apply(&Action::LevelUpAuto);
    assert_eq!(w.player_level(), 3);
    assert!(w.due().is_none());
    let (hp_base, mana_base) = (w.state.player().hp_base, w.state.player().mana_base);
    befall(&mut w, "unmade");
    assert!(said(&w, "Your past slips away from you."));
    assert!(said(&w, "You are level 2 again."));
    assert_eq!(w.player_level(), 2);
    assert_eq!(
        w.player_xp(),
        w.content.experience().threshold_for(2, factor)
    );
    let me = w.state.player();
    assert!(
        me.hp_base < hp_base,
        "the level's hit points are taken back"
    );
    assert!(me.hp <= me.max_hp(&w.content) && me.hp >= 1);
    assert!(me.mana_base <= mana_base);
    // Gained again: the hit points come back, the choices do not.
    let need = w.content.experience().threshold_for(3, factor) - w.player_xp();
    w.add_xp(need);
    assert_eq!(w.player_level(), 3);
    assert_eq!(w.state.player().hp_base, hp_base);
    assert!(w.due().is_none(), "no second set of choices");
    // A save keeps the level lost.
    let loaded = World::from_state(w.content.clone(), w.state().clone()).expect("loaded");
    assert_eq!(loaded.state.progress.peak, 3);
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn unmade_at_level_one_takes_the_experience_toward_the_next() {
    let mut w = world();
    befall(&mut w, "unmade");
    assert!(said(&w, "Something gropes at your past, and finds none."));
    w.grant_xp(10);
    befall(&mut w, "unmade");
    assert_eq!(w.player_xp(), 0);
    assert_eq!(w.player_level(), 1);
}

#[test]
fn chosen_gives_a_talent_for_life_outside_the_picks() {
    let mut w = world();
    let held = w.state.player().talents.len();
    befall(&mut w, "chosen");
    assert!(said(&w, "You have been chosen."));
    let me = w.state.player();
    assert_eq!(me.talents.len(), held + 1);
    assert_eq!(me.boons.len(), 1);
    let boon = me.boons[0].clone();
    assert_eq!(me.talents.rank_of(&boon.id, boon.choice.as_deref()), 1);
    assert!(w.due().is_none(), "no pick spent or owed");
    assert!(talent_rows(&w).contains("(altar)"), "{}", talent_rows(&w));
    // Restored, it is held still.
    let loaded = World::from_state(w.content.clone(), w.state().clone()).expect("loaded");
    let me = loaded.state.player();
    assert_eq!(me.talents.rank_of(&boon.id, boon.choice.as_deref()), 1);
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn forsaken_marks_with_a_flaw_kept_through_a_save() {
    let mut w = world();
    befall(&mut w, "forsaken");
    assert!(said(&w, "You have been marked."));
    let me = w.state.player();
    assert_eq!(me.boons.len(), 1);
    let flaw = me.boons[0].id.clone();
    let book = w.content.talents();
    assert!(book.get(book.id(&flaw).expect("talent")).flaw);
    assert!(me.talents.has(&flaw, 1));
    assert!(
        talent_rows(&w).contains("(altar, flaw)"),
        "{}",
        talent_rows(&w)
    );
    // Restored, the granted talents are put back: the flaw with them.
    let loaded = World::from_state(w.content.clone(), w.state().clone()).expect("loaded");
    assert!(loaded.state.player().talents.has(&flaw, 1));
    assert_eq!(loaded.state_hash(), w.state_hash());
    // Every flaw held: nothing more to mark.
    let mut w = world();
    let flaws: Vec<String> = (w.content.talents().by_tree())
        .filter(|(_, d)| d.flaw)
        .map(|(_, d)| d.id.clone())
        .collect();
    for f in &flaws {
        w.player_mut().talents.hold(f, None);
    }
    befall(&mut w, "forsaken");
    assert!(said(
        &w,
        "Something marks you, and finds the mark already made."
    ));
    assert!(w.state.player().boons.is_empty());
}

#[test]
fn a_relic_is_rare_and_lies_at_the_altars_foot() {
    for seed in 0..20 {
        let mut w = world();
        w.state.rng = crate::rng::Rng::gameplay(seed);
        befall(&mut w, "relic");
        let level = w.state.current();
        let piles: Vec<_> = level.objects.iter().collect();
        assert_eq!(piles.len(), 1, "one item");
        let (at, pile) = piles[0];
        assert!(at.chebyshev(ALTAR) == 1);
        let rarity = pile[0].magic().map(|m| m.rarity);
        assert!(
            matches!(rarity, Some(Rarity::Rare) | None),
            "rare, unless no affix fits: {rarity:?}"
        );
    }
}

#[test]
fn damned_brings_a_horde_around_the_altar() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content, "fennmynster", None).expect("fennmynster");
    let feature = w.player_pos();
    let count = w.state.actors.len();
    let outcome = outcome(&w, "damned");
    w.befall(&outcome, feature, &mut Vec::new());
    let came = w.state.actors.len() - count;
    assert!((2..=8).contains(&came), "{came}");
}

/// A table whose only bane is `bane` (and a nothing line), drawn every time as a bane.
fn bane_only(bane: &str) -> String {
    format!(
        "seconds = 60\nkinds = {{ nothing = 1, bane = 1000 }}\nexhaust = 0\n\
         [[outcome]]\nkind = \"nothing\"\ntier = \"minor\"\nid = \"silence\"\nweight = 1\n\
         message = \"Silence answers you.\"\n{bane}"
    )
}

#[test]
fn penitent_turns_a_great_bane_into_nothing_but_not_the_explosion() {
    let wrath = "[[outcome]]\nkind = \"bane\"\ntier = \"great\"\nid = \"wrath\"\nweight = 1\n\
                 effect = { wrath = \"1d3\" }\nmessage = \"You feel lessened, forever.\"\n";
    let mut w = world_with(Some(&bane_only(wrath)));
    w.player_mut().talents.take("penitent", None);
    let before = scores(&w);
    w.apply(&Action::Pray);
    assert!(said(&w, "Silence answers you."));
    assert!(!said(&w, "You feel lessened, forever."));
    assert_eq!(scores(&w), before);
    // Without the talent the bane comes.
    let mut w = world_with(Some(&bane_only(wrath)));
    w.apply(&Action::Pray);
    assert!(said(&w, "You feel lessened, forever."));
    // The explosion is moderate: it comes to the penitent too.
    let burst = "[[outcome]]\nkind = \"bane\"\ntier = \"moderate\"\nid = \"burst\"\nweight = 1\n\
                 effect = { becomes = \"rubble\" }\nmessage = \"The altar bursts apart in fire!\"\n";
    let mut w = world_with(Some(&bane_only(burst)));
    w.player_mut().talents.take("penitent", None);
    w.apply(&Action::Pray);
    assert!(said(&w, "The altar bursts apart in fire!"));
}
