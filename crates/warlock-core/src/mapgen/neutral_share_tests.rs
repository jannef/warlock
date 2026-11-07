//! Neutral animals in the dungeons (#1195): the share of creatures not hostile to the player
//! among a branch floor's population and its timed spawns, with `[defaults]`'s `neutral` and
//! with it at 100 (as before).

use super::*;
use crate::content::{ContentFiles, CreatureId, Stance, danger};
use crate::rng::Rng;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| Content::load(&ContentFiles::builtin()).expect("builtin"));

/// The built-in content with neutral animals at their whole weight.
static BEFORE: LazyLock<Content> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    let (path, text) = (files.table("dungeons").next())
        .map(|(p, t)| (p.to_string(), t.to_string()))
        .expect("data/dungeons.toml");
    assert!(text.contains("\nneutral = 10\n"), "the shipped default");
    files.insert(
        &path,
        &text.replace("\nneutral = 10\n", "\nneutral = 100\n"),
    );
    Content::load(&files).expect("builtin")
});

/// Counts: (neutral, all).
type Share = [u32; 2];

/// Every branch floor with a danger (an endless branch's first 30) whose odds scale the neutral.
fn floors(content: &Content) -> Vec<String> {
    let mut ids = Vec::new();
    for def in content.dungeons() {
        let depth = if def.endless.is_some() { 30 } else { def.depth };
        for d in 1..=depth {
            let id = format!("{}:{d}", def.id);
            if content.spawning(&id).is_some() {
                ids.push(id);
            }
        }
    }
    ids
}

fn neutral(content: &Content, c: CreatureId) -> bool {
    let player = content.creature_id("player").expect("player");
    content.stance(c, player) != Stance::Hostile
}

/// Per branch whose own `neutral` is under 100: the floors' populations (den sleepers aside)
/// over `seeds`, and `draws` timed-spawn draws a floor.
fn shares(content: &Content, seeds: u64, draws: u32) -> BTreeMap<String, (Share, Share)> {
    let mut out: BTreeMap<String, (Share, Share)> = BTreeMap::new();
    for id in floors(content) {
        let branch = id.split(':').next().expect("a branch").to_string();
        let e = out.entry(branch).or_default();
        let (rule, at, spawning) = content.spawning(&id).expect("spawning");
        // Only the drawn count as neutral: not a shop's keeper, who is placed.
        let drawn = |c| spawning.population.contains(&c) || spawning.generic.contains(&c);
        for seed in 0..seeds {
            let level = generate(content, ("ascmearc", seed), &id, None).expect("generates");
            for c in level.creatures.iter().filter(|c| !c.asleep) {
                e.0[0] += u32::from(drawn(c.creature) && neutral(content, c.creature));
                e.0[1] += 1;
            }
        }
        let danger = rule.at(at, 1);
        let odds = content.draw_odds_at(&id).expect("odds");
        let none = Default::default();
        let own = content.draw_candidates(&spawning.population, &none, &odds);
        let generic = content.draw_candidates(&spawning.generic, &none, &odds);
        let mut rng = Rng::spawn(7, &id);
        for _ in 0..draws {
            if let Some(c) = danger::draw(&mut rng, &own, &generic, danger, &odds) {
                e.1[0] += u32::from(neutral(content, c));
                e.1[1] += 1;
            }
        }
    }
    out
}

fn percent([n, all]: Share) -> f64 {
    f64::from(n) * 100.0 / f64::from(all.max(1))
}

/// Whether `after` is a fifth of `before` or less, as shares (two creatures' slack for the
/// branches with hardly any).
fn a_fifth(after: Share, before: Share) -> bool {
    percent([after[0].saturating_sub(2), after[1]]) * 5.0 <= percent(before)
}

/// The branches whose odds scale the neutral (Ælfhām's fey keep their weight).
fn scaled() -> Vec<String> {
    (CONTENT.dungeons())
        .filter(|d| {
            CONTENT
                .draw_odds_at(&format!("{}:1", d.id))
                .is_some_and(|o| o.neutral < 100)
        })
        .map(|d| d.id.clone())
        .collect()
}

#[test]
fn neutral_animals_are_a_fifth_or_less_of_what_they_were() {
    let (after, before) = (shares(&CONTENT, 4, 400), shares(&BEFORE, 4, 400));
    let scaled = scaled();
    assert!(scaled.iter().any(|b| b == "cealdhlaw"), "{scaled:?}");
    let (mut all_after, mut all_before) = ([0; 2], [0; 2]);
    for branch in &scaled {
        let (a, b) = (after[branch], before[branch]);
        assert!(a_fifth(a.0, b.0), "{branch} floors: {a:?} against {b:?}");
        assert!(a_fifth(a.1, b.1), "{branch} spawns: {a:?} against {b:?}");
        for (sum, s) in [(&mut all_after, a.0), (&mut all_before, b.0)] {
            sum[0] += s[0];
            sum[1] += s[1];
        }
    }
    assert!(
        percent(all_before) > 5.0,
        "there were neutral animals: {all_before:?}"
    );
    assert!(
        percent(all_after) * 5.0 <= percent(all_before),
        "{all_after:?} {all_before:?}"
    );
}

#[test]
fn a_floor_holds_as_many_creatures_as_before() {
    let id = "cealdhlaw:1";
    for seed in 0..4 {
        let count = |content: &Content| {
            let level = generate(content, ("ascmearc", seed), id, None).expect("generates");
            level.creatures.len()
        };
        assert_eq!(count(&CONTENT), count(&BEFORE), "seed {seed}");
    }
}

/// Prints the shares per branch, before and after (`--ignored --nocapture`, best in release).
#[test]
#[ignore = "a measurement"]
fn print_neutral_shares() {
    let (after, before) = (shares(&CONTENT, 20, 2000), shares(&BEFORE, 20, 2000));
    for (branch, (floor, spawns)) in after {
        let (bf, bs) = before[&branch];
        println!(
            "| {branch} | {:.1}% → {:.1}% of {} | {:.1}% → {:.1}% of {} |",
            percent(bf),
            percent(floor),
            floor[1],
            percent(bs),
            percent(spawns),
            spawns[1]
        );
    }
}
