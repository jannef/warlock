//! Cutting a gem (docs/design/recipes.md §5; `world/cut.rs`): the bands at their margin edges,
//! the natural 2 and 24, the size lost on a failure and a crude cut, a re-cut never worse and
//! always a size smaller, the refusals with no time, an interruption, the marks, the odds the
//! design gives, and a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, CutId, Cutting, Gems};
use crate::item_parts::ItemPart;
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

const ROOM: &str = "\
###########
#.........#
#.@.......#
#.........#
#.......k.#
###########
";

/// The built-in content with [`ROOM`] twice: with no one in it, and with a kobold.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, k) in [
        ("test_alone", "\"floor\""),
        (
            "test_watched",
            "{ terrain = \"floor\", creature = \"kobold\" }",
        ),
    ] {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!(
                "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"k\" = {k}\n\
                 [[item]]\nat = [3, 2]\nobject = \"gem\"\n\
                 parts = [{{ of = \"ruby\", size = \"large\", cut = \"uncut\" }}]\n"
            ),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"T\"\nstart_level = \"{name}\"\n\
                 character = {{ race = \"human\", gender = \"female\", class = \"commoner\" }}\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// A hall with no one in it, or with a kobold in sight. The player is a human commoner carrying
/// nothing, Jewelcraft 100 with no marks, every score 10 (a modifier of 0), `seed` their fate.
fn world(kobold: bool, seed: u64) -> World {
    let scenario = if kobold { "test_watched" } else { "test_alone" };
    let mut w = World::new(FIXTURE.clone(), scenario, Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    let jewel = jewelcraft(&w);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(k, _)| *k != jewel).collect();
    all.push((
        jewel,
        Trained {
            value: 100,
            marks: 0,
        },
    ));
    *skills = all.into_iter().collect();
    w
}

fn jewelcraft(w: &World) -> SkillId {
    w.content.skill_id("jewelcraft").expect("jewelcraft")
}

fn gems(w: &World) -> &Gems {
    w.content.gems().expect("gems")
}

fn cutting(w: &World, size: &str, cut: &str) -> Cutting {
    let gems = gems(w);
    Cutting {
        size: gems.size_id(size).expect(size),
        cut: gems.cut_id(cut).expect(cut),
    }
}

fn cut(w: &World, id: &str) -> CutId {
    gems(w).cut_id(id).expect(id)
}

/// `count` loose gems of `gem` cut as `size` and `cut`.
fn loose(w: &World, gem: &str, size: &str, cut: &str, count: u32) -> Stack {
    let c = &w.content;
    let material = c.material_id(gem).expect(gem);
    let part = ItemPart::gem(c, None, material, cutting(w, size, cut));
    Stack::new(gems(w).object, count).with_parts(vec![part])
}

fn give(w: &mut World, stack: impl FnOnce(&World) -> Stack) {
    let stack = stack(w);
    w.player_mut().inventory.insert(0, stack);
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| s.describe(&w.content))
        .collect()
}

fn cut_action(item: usize) -> Action {
    Action::Craft {
        recipe: "cut".into(),
        picks: vec![Some(item)],
    }
}

/// The step data/recipes.toml gives.
fn step(w: &World) -> i32 {
    w.content
        .recipes()
        .settings
        .as_ref()
        .expect("bands")
        .cut_step
}

#[test]
fn each_band_starts_at_its_margin_edge() {
    let w = world(false, 1);
    let band = |margin| cut_band(gems(&w), step(&w), 12, margin);
    assert_eq!(band(-1), CutBand::Failed);
    for (margin, id) in [
        (0, "crude"),
        (4, "crude"),
        (5, "fair"),
        (9, "fair"),
        (10, "fine"),
        (14, "fine"),
        (15, "exquisite"),
        (19, "exquisite"),
        (20, "flawless"),
        (40, "flawless"),
    ] {
        assert_eq!(band(margin), CutBand::Cut(cut(&w, id)), "margin {margin}");
    }
    // A natural 2 shatters whatever the margin; a natural 24 is at least fair.
    assert_eq!(cut_band(gems(&w), 5, 2, 30), CutBand::Shattered);
    assert_eq!(
        cut_band(gems(&w), 5, 24, -10),
        CutBand::Cut(cut(&w, "fair"))
    );
    assert_eq!(cut_band(gems(&w), 5, 24, 1), CutBand::Cut(cut(&w, "fair")));
    assert_eq!(
        cut_band(gems(&w), 5, 24, 16),
        CutBand::Cut(cut(&w, "exquisite"))
    );
}

#[test]
fn an_uncut_gem_loses_a_size_on_a_failure_and_a_crude_cut() {
    let w = world(false, 1);
    let g = gems(&w);
    let large = cutting(&w, "large", "uncut");
    let band = |id| CutBand::Cut(cut(&w, id));
    assert_eq!(
        cut_outcome(g, large, CutBand::Failed),
        Some(cutting(&w, "medium", "uncut"))
    );
    assert_eq!(
        cut_outcome(g, large, band("crude")),
        Some(cutting(&w, "medium", "crude"))
    );
    assert_eq!(
        cut_outcome(g, large, band("fair")),
        Some(cutting(&w, "large", "fair"))
    );
    assert_eq!(
        cut_outcome(g, large, band("flawless")),
        Some(cutting(&w, "large", "flawless"))
    );
    assert_eq!(cut_outcome(g, large, CutBand::Shattered), None);
    // A tiny stone has no size to lose: a failure or a crude cut leaves nothing.
    let tiny = cutting(&w, "tiny", "uncut");
    assert_eq!(cut_outcome(g, tiny, CutBand::Failed), None);
    assert_eq!(cut_outcome(g, tiny, band("crude")), None);
    assert_eq!(
        cut_outcome(g, tiny, band("fine")),
        Some(cutting(&w, "tiny", "fine"))
    );
}

#[test]
fn a_recut_is_never_worse_and_always_a_size_smaller() {
    let w = world(false, 1);
    let g = gems(&w);
    let crude = cutting(&w, "large", "crude");
    let band = |id| CutBand::Cut(cut(&w, id));
    // The owner's example: a large crude ruby (2700) to a medium flawless one (4800).
    assert_eq!(
        cut_outcome(g, crude, band("flawless")),
        Some(cutting(&w, "medium", "flawless"))
    );
    let ruby = w.content.material_id("ruby").expect("ruby");
    let def = w.content.material(ruby).gem.as_ref().expect("a gem");
    assert_eq!(g.value(def, crude), 2700);
    assert_eq!(g.value(def, cutting(&w, "medium", "flawless")), 4800);
    // Worse or failed: the old cut kept, a size lost all the same.
    let fine = cutting(&w, "medium", "fine");
    for worse in [CutBand::Failed, band("crude"), band("fine")] {
        assert_eq!(
            cut_outcome(g, fine, worse),
            Some(cutting(&w, "small", "fine"))
        );
    }
    assert_eq!(
        cut_outcome(g, fine, band("exquisite")),
        Some(cutting(&w, "small", "exquisite"))
    );
    assert_eq!(cut_outcome(g, fine, CutBand::Shattered), None);
}

#[test]
fn the_list_offers_any_uncut_gem_and_cut_ones_with_room_to_cut() {
    let w = world(false, 1);
    let c = &w.content;
    assert!(cuttable(c, &loose(&w, "ruby", "tiny", "uncut", 1)));
    assert!(cuttable(c, &loose(&w, "ruby", "huge", "exquisite", 1)));
    assert!(!cuttable(c, &loose(&w, "ruby", "tiny", "crude", 1)));
    assert!(!cuttable(c, &loose(&w, "ruby", "huge", "flawless", 1)));
    let sword = c.object_id("long_sword").expect("sword");
    assert!(!cuttable(c, &Stack::one(sword)));
}

#[test]
fn a_cut_takes_half_an_hour_changes_one_gem_and_marks_jewelcraft() {
    let mut w = world(false, 3);
    give(&mut w, |w| loose(w, "ruby", "large", "uncut", 2));
    let start = w.state.time;
    w.apply(&cut_action(0));
    assert_eq!(
        w.state.time.since(start).as_seconds(),
        1800,
        "{:?}",
        texts(&w)
    );
    assert!(
        said(&w, "You set to cutting the uncut large ruby."),
        "{:?}",
        texts(&w)
    );
    let p = pack(&w);
    // The other stays as it was; the one cut is whatever came of the check (or nothing).
    assert_eq!(p[0], "an uncut large ruby", "{p:?}");
    assert!(p.len() <= 2, "{p:?}");
    let jewel = jewelcraft(&w);
    assert_eq!(w.state.player().skills.get(jewel).map(|t| t.marks), Some(1));
}

#[test]
fn every_outcome_comes_up_and_says_itself() {
    // Over many fates an uncut large ruby (against 16) at Jewelcraft 100 (+8) is shattered,
    // failed, roughed out, and well cut; each says so, and the size follows the words.
    let mut seen = [false; 4];
    for seed in 0..200 {
        let mut w = world(false, seed);
        give(&mut w, |w| loose(w, "ruby", "large", "uncut", 1));
        w.apply(&cut_action(0));
        let p = pack(&w);
        if said(&w, "The ruby shatters.") {
            assert!(p.is_empty(), "{p:?}");
            seen[0] = true;
        } else if said(&w, "The stone splits badly; what is left is smaller.") {
            assert_eq!(p, ["an uncut medium ruby"]);
            assert!(said(&w, "It is now an uncut medium ruby."));
            seen[1] = true;
        } else if said(&w, "You rough it out, and lose more than you meant to.") {
            assert_eq!(p, ["a medium crude ruby"]);
            seen[2] = true;
        } else {
            assert!(said(&w, "You cut the stone."), "{:?}", texts(&w));
            assert!(p[0].starts_with("a large "), "{p:?}");
            seen[3] = true;
        }
    }
    assert_eq!(seen, [true; 4]);
}

#[test]
fn a_recut_in_play_keeps_the_cut_or_betters_it() {
    for seed in 0..40 {
        let mut w = world(false, seed);
        give(&mut w, |w| loose(w, "ruby", "large", "fine", 1));
        w.apply(&cut_action(0));
        let p = pack(&w);
        if said(&w, "The ruby shatters.") {
            continue;
        }
        let fine = [
            "a medium fine ruby",
            "a medium exquisite ruby",
            "a medium flawless ruby",
        ];
        assert!(fine.contains(&p[0].as_str()), "{p:?}");
    }
}

#[test]
fn a_tiny_stone_cut_badly_crumbles_and_every_cut_is_marked() {
    // An uncut tiny garnet at Jewelcraft 100: a failure or a crude cut
    // leaves nothing, a good cut keeps it tiny; Jewelcraft is marked whatever came of it.
    let mut crumbled = false;
    for seed in 0..60 {
        let mut w = world(false, seed);
        give(&mut w, |w| loose(w, "garnet", "tiny", "uncut", 1));
        w.apply(&cut_action(0));
        let jewel = jewelcraft(&w);
        assert_eq!(w.state.player().skills.get(jewel).map(|t| t.marks), Some(1));
        let p = pack(&w);
        if said(
            &w,
            "The stone crumbles under your tools; nothing worth keeping is left.",
        ) {
            assert!(p.is_empty(), "{p:?}");
            crumbled = true;
        } else if !said(&w, "The garnet shatters.") {
            assert!(p[0].starts_with("a tiny "), "{p:?}");
        }
    }
    assert!(crumbled);
}

#[test]
fn every_refusal_takes_no_time() {
    let mut w = world(false, 1);
    let sword = w.content.object_id("long_sword").expect("sword");
    give(&mut w, |_| Stack::one(sword));
    give(&mut w, |w| loose(w, "ruby", "huge", "flawless", 1));
    give(&mut w, |w| loose(w, "ruby", "tiny", "fair", 1));
    let before = w.state.time;
    for (action, line) in [
        (cut_action(0), "There is nothing left to cut."),
        (cut_action(1), "It could not be finer."),
        (cut_action(2), "That is no loose gem."),
        (cut_action(9), "You carry no such thing."),
        (
            Action::Craft {
                recipe: "cut".into(),
                picks: Vec::new(),
            },
            "You must choose the gem.",
        ),
        (
            Action::Craft {
                recipe: "cut".into(),
                picks: vec![Some(0), Some(1)],
            },
            "That is more than the work takes.",
        ),
    ] {
        w.apply(&action);
        assert!(said(&w, line), "{line}: {:?}", texts(&w));
    }
    assert_eq!(w.state.time, before);
}

#[test]
fn an_interrupted_cut_spends_nothing() {
    let mut w = world(true, 4);
    give(&mut w, |w| loose(w, "ruby", "large", "uncut", 1));
    let start = w.state.time;
    w.apply(&cut_action(0));
    assert!(
        (w.messages().iter()).any(|m| m.text.ends_with("you put down your work.")),
        "{:?}",
        texts(&w)
    );
    assert!(w.state.time.since(start).as_seconds() < 1800);
    assert_eq!(pack(&w), ["an uncut large ruby"]);
}

/// The share of the 144 natural rolls whose band, for a gem of `work` against the recipe's 12
/// with a whole bonus of `bonus`, is each of: shattered, failed, then each true cut.
fn odds(w: &World, work: i32, bonus: i32) -> Vec<f64> {
    let g = gems(w);
    let mut counts = vec![0u32; g.cuts().len() + 2];
    for a in 1..=12 {
        for b in 1..=12 {
            let natural = a + b;
            let i = match cut_band(g, step(w), natural, natural + bonus - (12 + work)) {
                CutBand::Shattered => 0,
                CutBand::Failed => 1,
                CutBand::Cut(c) => 2 + usize::from(c.0),
            };
            counts[i] += 1;
        }
    }
    counts
        .iter()
        .map(|&n| f64::from(n) * 100.0 / 144.0)
        .collect()
}

#[test]
fn the_odds_are_the_designs() {
    let w = world(false, 1);
    // Shattered, failed, crude, fair, fine, exquisite, flawless (recipes.md §5): an uncut ruby
    // (+4) by bonus, then a smokestone (0) at +16.
    for (work, bonus, want) in [
        (4, 4, [1.0, 38.0, 37.0, 21.0, 4.0, 0.0, 0.0]),
        (4, 8, [1.0, 14.0, 31.0, 35.0, 17.0, 2.0, 0.0]),
        (4, 12, [1.0, 1.0, 17.0, 35.0, 31.0, 14.0, 1.0]),
        (4, 16, [1.0, 0.0, 3.0, 21.0, 37.0, 28.0, 10.0]),
        (0, 16, [1.0, 0.0, 0.0, 6.0, 24.0, 38.0, 31.0]),
    ] {
        let got = odds(&w, work, bonus);
        // The uncut band (index 2) never comes of a check.
        let got: Vec<f64> = (got.iter().enumerate())
            .filter(|&(i, _)| i != 2)
            .map(|(_, &p)| p)
            .collect();
        for (g, x) in got.iter().zip(want) {
            assert!((g - x).abs() <= 1.0, "work {work} bonus {bonus}: {got:?}");
        }
    }
}

#[test]
fn a_cut_replays() {
    // As the scenario starts: a large uncut ruby lies east of the player.
    let mut w = World::new(FIXTURE.clone(), "test_alone", Some(8)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E pickup craft:cut:a wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        said(&w, "You set to cutting the uncut large ruby."),
        "{:?}",
        texts(&w)
    );
    let (verified, divergence) = replay.play(FIXTURE.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
