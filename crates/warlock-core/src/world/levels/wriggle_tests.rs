//! Wriggling past (`wriggle.rs`, #1032): a step into a creature that keeps its post is a refusal
//! untrained and an automatic Escape artist check trained.

use crate::content::ContentFiles;
use crate::geom::{Dir, Pos};
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stat, Stats};
use crate::test_support::last_text as last;
use crate::world::people::talk::Refusal;
use crate::world::*;

/// The player and a shopkeeper keeping its post east of them.
const ROOM: &str = "#########\n#.@s....#\n#.......#\n#########\n";

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/wriggle.toml",
        &format!(
            "name = \"Wriggle\"\nambient = [255, 255, 255, 0]\nmap = '''\n{ROOM}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"s\" = {{ terrain = \"floor\", creature = \"shopkeeper\" }}\n"
        ),
    );
    files.insert(
        "scenarios/wriggle.toml",
        "name = \"Wriggle\"\nstart_level = \"wriggle\"\n",
    );
    let content = Content::load(&files).expect("test content is valid");
    let mut w = World::new(Arc::new(content), "wriggle", Some(3)).expect("world");
    let me = w.player_mut();
    me.hp_base = 500;
    me.hp = 500;
    let mut scores = PerStat::splat(10);
    scores[Stat::Dexterity] = 14;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
    w
}

fn skill(w: &World) -> SkillId {
    w.content().skill_id("escape_artist").expect("skill")
}

fn train(w: &mut World, value: u16) {
    let id = skill(w);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    all.push((id, Trained { value, marks: 0 }));
    *skills = all.into_iter().collect();
}

fn marks(w: &World) -> u32 {
    (w.state.player().skills.get(skill(w))).map_or(0, |t| t.marks)
}

#[test]
fn untrained_the_refusal_stands_and_takes_no_time() {
    let mut w = world();
    let start = w.player_pos();
    let out = w.apply(&Action::Move(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(w.player_pos(), start);
    assert_eq!(last(&w), "The shopkeeper doesn't budge.");
    assert_eq!(marks(&w), 0);
}

#[test]
fn trained_a_try_costs_a_step_and_wriggling_swaps_places() {
    let mut w = world();
    train(&mut w, 100);
    let start = w.player_pos();
    let (mut tries, mut past) = (0, false);
    for _ in 0..200 {
        let out = w.apply(&Action::Move(Dir::E));
        assert!(out.took_turn, "a try takes the step's time");
        tries += 1;
        if w.player_pos() == start {
            assert_eq!(
                last(&w),
                "You try to wriggle past the shopkeeper, but it is in the way."
            );
        } else {
            assert_eq!(w.player_pos(), Pos::new(3, 1));
            assert!(
                w.messages()
                    .iter()
                    .any(|m| m.text == "You wriggle past the shopkeeper.")
            );
            past = true;
            break;
        }
    }
    assert!(past, "a +8 wriggler gets past a shopkeeper in the end");
    assert!(marks(&w) <= tries, "at most a mark a try");
}

#[test]
fn it_can_fail_and_it_can_succeed() {
    // Barely trained, at an average Dexterity.
    let mut w = world();
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    train(&mut w, 1);
    let (start, post) = (w.player_pos(), Pos::new(3, 1));
    let level = w.state.current().id.clone();
    let id = w.state.actor_at(&level, post).expect("shopkeeper").id;
    let (mut stuck, mut past) = (0, 0);
    for _ in 0..80 {
        w.apply(&Action::Move(Dir::E));
        match w.player_pos() == start {
            true => stuck += 1,
            false => past += 1,
        }
        // Put them back as they were, the keeper's mind too (displaced, it is off its post).
        w.player_mut().pos = start;
        let keeper = w.state.actors.get_mut(&id).expect("shopkeeper");
        keeper.pos = post;
        keeper.mind = Default::default();
    }
    assert!(stuck > 0 && past > 0, "{stuck} stuck, {past} past");
}

#[test]
fn every_try_is_a_draw_for_a_mark_at_half() {
    let mut w = world();
    train(&mut w, 100);
    let level = w.state.current().id.clone();
    let id = w
        .state
        .actor_at(&level, Pos::new(3, 1))
        .expect("shopkeeper")
        .id;
    // Direct tries: a turn's end would spend the marks on a skill roll.
    for _ in 0..60 {
        w.wriggle(id, Refusal::Budge, Pos::new(3, 1), true);
    }
    let marked = marks(&w);
    assert!((15..=45).contains(&marked), "{marked} marks in 60 tries");
}

#[test]
fn the_other_refusals_stand_for_the_trained() {
    let mut w = world();
    train(&mut w, 200);
    let level = w.state.current().id.clone();
    let id = w
        .state
        .actor_at(&level, Pos::new(3, 1))
        .expect("shopkeeper")
        .id;
    assert_eq!(w.refusal(id), Some(Refusal::Budge));
    for why in [Refusal::TooBig, Refusal::Water] {
        assert!(matches!(
            w.wriggle(id, why, Pos::new(3, 1), true),
            super::wriggle::Wriggled::No
        ));
    }
}

#[test]
fn tries_are_deterministic() {
    let run = || {
        let mut w = world();
        train(&mut w, 100);
        for _ in 0..5 {
            w.apply(&Action::Move(Dir::E));
        }
        w.state_hash()
    };
    assert_eq!(run(), run());
}
