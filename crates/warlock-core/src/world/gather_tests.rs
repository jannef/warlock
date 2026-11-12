//! Gathering herbs from the glowing fungus (spec §8 "Skills": Herbalism; #1340,
//! `world/gather.rs`): Gather is listed only beside fungus, a gather takes its five minutes and
//! finds herbs by the level's depth (one untrained, `1d2` + Herbalism ÷ 50 trained), a broken
//! one finds nothing, a refusal takes no time, the fungus never changes, Herbalism is marked,
//! and replays verify.

use super::*;
use crate::ability::{Ability, Source};
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};

/// A cave with fungus at (5, 2), the player beside it at (4, 3), and at `k` a kobold (also
/// beside it), when asked for; `f` is a second fungus, there only with `two`.
const ROOM: &str = "\
###########
#.........#
#....F....#
#...@k....#
#...f.....#
#.........#
###########
";

/// The fungus.
const FUNGUS: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`] at item level `depth`; the kobold only with `kobold`, the
/// second fungus only with `two`.
fn world(depth: u8, kobold: bool, two: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    let f = if two { "\"fungus\"" } else { "\"floor\"" };
    files.insert(
        "levels/test_fungus.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nitem_level = {depth}\n\
             map = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"F\" = \"fungus\"\n\"k\" = {k}\n\"f\" = {f}\n"
        ),
    );
    files.insert(
        "scenarios/test_fungus.toml",
        "name = \"T\"\nstart_level = \"test_fungus\"\n\
         character = { race = \"human\", gender = \"female\", class = \"priest\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_fungus", None).expect("world")
}

fn herbalism(w: &World) -> SkillId {
    w.content().skill_id("herbalism").expect("Herbalism")
}

/// The player trained in Herbalism at `value` (`None`: untrained).
fn train(w: &mut World, value: Option<u16>) {
    let id = herbalism(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn here(w: &World) -> Vec<Ability> {
    (w.perception().abilities().into_iter())
        .map(|a| a.ability)
        .filter(|a| a.source() == Source::Place)
        .collect()
}

/// How many herbs the player carries (anything of the gather table).
fn herbs(w: &World) -> u32 {
    let rules = w.content().herbs().expect("herbs.toml");
    (w.state.player().inventory.iter())
        .filter(|s| rules.herbs.iter().any(|h| h.object == s.object))
        .map(|s| s.count)
        .sum()
}

fn seconds(w: &World) -> u64 {
    u64::from(w.content().herbs().expect("herbs.toml").seconds)
}

#[test]
fn gather_is_listed_only_beside_fungus_and_refused_elsewhere() {
    let mut w = world(1, false, false);
    assert_eq!(here(&w), vec![Ability::Gather]);
    assert!(Ability::every(&w.content).contains(&Ability::Gather));
    assert_eq!(w.perception().gather_ways(), vec![Dir::NE]);
    // Not the way to the fungus: refused, no time.
    let before = w.state.time;
    let out = w.apply(&Action::Gather(Dir::S));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last_message(&w), "There is nothing here to gather.");
    assert_eq!(w.state.time, before);
    // Two cells away: not listed, refused.
    w.apply(&Action::Move(Dir::S));
    w.apply(&Action::Move(Dir::S));
    assert!(here(&w).is_empty());
    assert!(w.perception().gather_ways().is_empty());
    let before = w.state.time;
    w.apply(&Action::Gather(Dir::N));
    assert!(said(&w, "There is nothing here to gather."));
    assert_eq!(w.state.time, before, "refused, no time");
    // With fungus on two sides, the menu asks the way.
    let w = world(1, false, true);
    assert_eq!(w.perception().gather_ways(), vec![Dir::NE, Dir::S]);
    assert_eq!(
        Ability::Gather.aim(w.content()),
        crate::ability::Aim::Direction
    );
}

#[test]
fn the_untrained_find_one_herb_and_the_trained_more() {
    let mut w = world(1, false, false);
    train(&mut w, None);
    let start = w.state.time;
    w.apply(&Action::Gather(Dir::NE));
    assert_eq!(w.state.time.since(start).as_seconds(), seconds(&w));
    assert!(said(&w, "You search the glowing fungus for herbs."));
    assert_eq!(herbs(&w), 1, "untrained: one");
    assert!(
        last_message(&w).starts_with("You gather "),
        "{}",
        last_message(&w)
    );
    // Herbalism 100: `1d2` + 2, so three or four.
    for seed in 0..8 {
        let content = world(1, false, false).content.clone();
        let mut w = World::new(content, "test_fungus", Some(seed)).expect("world");
        train(&mut w, Some(100));
        w.apply(&Action::Gather(Dir::NE));
        assert!((3..=4).contains(&herbs(&w)), "seed {seed}: {}", herbs(&w));
    }
}

#[test]
fn shallow_fungus_gives_only_the_common_herbs() {
    let content = world(1, false, false).content.clone();
    let rules = content.herbs().expect("herbs.toml");
    let common: Vec<_> = rules.at(1).into_iter().map(|(o, _)| o).collect();
    for seed in 0..6 {
        let mut w = World::new(content.clone(), "test_fungus", Some(seed)).expect("world");
        train(&mut w, Some(200));
        w.apply(&Action::Gather(Dir::NE));
        for stack in &w.state.player().inventory {
            if rules.herbs.iter().any(|h| h.object == stack.object) {
                assert!(common.contains(&stack.object), "seed {seed}");
            }
        }
    }
}

#[test]
fn a_broken_gather_finds_nothing_and_costs_the_time() {
    let mut w = world(1, true, false);
    let start = w.state.time;
    w.apply(&Action::Gather(Dir::NE));
    assert!(said(&w, "You stop gathering."), "{}", last_message(&w));
    let spent = w.state.time.since(start).as_seconds();
    assert!(spent > 0 && spent < seconds(&w), "{spent}");
    assert_eq!(herbs(&w), 0);
}

#[test]
fn the_fungus_never_runs_out_and_herbalism_is_marked() {
    let mut w = world(1, false, false);
    train(&mut w, Some(10));
    let fungus = w.state.current().terrain_at(FUNGUS);
    let mark = |w: &World| {
        let id = herbalism(w);
        w.state.player().skills.get(id).map(|t| (t.value, t.marks))
    };
    let marks = mark(&w);
    w.apply(&Action::Gather(Dir::NE));
    let first = herbs(&w);
    assert!(first >= 1);
    assert_eq!(w.state.current().terrain_at(FUNGUS), fungus, "unchanged");
    assert_ne!(mark(&w), marks, "Herbalism marked");
    w.apply(&Action::Gather(Dir::NE));
    assert!(herbs(&w) > first, "gathered again at once");
    assert_eq!(here(&w), vec![Ability::Gather]);
}

#[test]
fn a_gather_replays() {
    let content = world(6, false, false).content.clone();
    let mut w = World::new(content.clone(), "test_fungus", Some(7)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("gather:NE wait gather:NE").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
