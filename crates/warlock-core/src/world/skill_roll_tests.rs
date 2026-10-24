//! Skill rolls at level-up (spec §8 "Skills"): every level gives the class's, the race's, and
//! Intelligence's rolls, refused around until spent; each throws the die the skill's marks set
//! on the ladder (one step up for a known-for skill, +1 for a class's or race's other skill),
//! spends half the marks, and draws from the levels stream only; replays and saves agree.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::Factor;
use crate::replay::Replay;
use crate::skills::Trained;
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

fn world(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn last(world: &World) -> String {
    (world.messages().iter().next_back())
        .map(|m| m.display())
        .unwrap_or_default()
}

/// Every score of the player 10, but Intelligence `intelligence`.
fn set_intelligence(world: &mut World, intelligence: u8) {
    let mut scores = PerStat::splat(10);
    scores[Stat::Intelligence] = intelligence;
    world.player_mut().stats = Stats::new(scores, PerStat::splat(40));
}

/// Grants the XP that takes the player to `level`.
fn reach(world: &mut World, level: u8) {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    let need = world.content().experience().threshold_for(level, factor);
    let more = need - world.player_xp();
    run(world, &format!("xp:{more}"));
    assert_eq!(world.player_level(), level, "reached level {level}");
}

fn skill(world: &World, id: &str) -> crate::skills::SkillId {
    world.content().skill_id(id).expect("a skill")
}

fn trained(world: &World, id: &str) -> Option<Trained> {
    world.state.player().skills.get(skill(world, id))
}

/// Sets the player's skill `id` to `value` with `marks`, training it.
fn set(world: &mut World, id: &str, value: u16, marks: u32) {
    let id = skill(world, id);
    let skills = &mut world.player_mut().skills;
    let others: Vec<_> = skills.iter().filter(|&(s, _)| s != id).collect();
    *skills = (others.into_iter())
        .chain([(id, Trained { value, marks })])
        .collect();
}

/// What one roll on `id` (at `value` with `marks`) adds, over `tries` level-ups' first rolls,
/// as the lowest and highest seen.
fn gains(character: &str, id: &str, value: u16, marks: u32, tries: usize) -> (u16, u16) {
    let mut w = world(character);
    set_intelligence(&mut w, 10);
    let (mut low, mut high) = (u16::MAX, 0);
    for _ in 0..tries {
        let next = w.player_level() + 1;
        reach(&mut w, next);
        let mut probe = w.clone();
        set(&mut probe, id, value, marks);
        let before = trained(&probe, id).expect("trained").value;
        run(&mut probe, &format!("advance:{id}"));
        let gain = trained(&probe, id).expect("trained").value - before;
        (low, high) = (low.min(gain), high.max(gain));
        run(&mut w, "levelup:auto");
    }
    (low, high)
}

#[test]
fn a_level_gives_the_classs_the_races_and_intelligences_rolls() {
    let rolls = |character: &str, intelligence: u8| {
        let mut w = world(character);
        set_intelligence(&mut w, intelligence);
        w.level_rolls()
    };
    // A warrior 2, a human 1 more; Intelligence 14 one more, and one more every 4 points.
    assert_eq!(rolls("human,male,warrior", 10), 3);
    assert_eq!(rolls("human,male,warrior", 13), 3);
    assert_eq!(rolls("human,male,warrior", 14), 4);
    assert_eq!(rolls("human,male,warrior", 18), 5);
    assert_eq!(rolls("elf,female,warrior", 10), 2);
    assert_eq!(rolls("halfling,female,thief", 10), 3);
    assert_eq!(rolls("dwarf,male,ranger", 10), 3);
    assert_eq!(rolls("human,other,bard", 10), 5, "the bard's gift");
    // Trained in nothing, a level has nothing to roll on.
    let mut w = world("human,male,warrior");
    w.player_mut().skills = Default::default();
    assert_eq!(w.level_rolls(), 0);
    reach(&mut w, 2);
    assert_eq!(w.due(), None, "nothing due");
}

#[test]
fn rolls_are_due_until_spent_on_trained_skills() {
    let mut w = world("human,male,warrior");
    set_intelligence(&mut w, 10);
    reach(&mut w, 2);
    assert_eq!(
        w.due(),
        Some(Due {
            level: 2,
            talents: 0,
            rolls: 3
        })
    );
    let turn = w.turn();
    for refused in ["move:E", "wait", "rest:1h", "xp:10"] {
        run(&mut w, refused);
        assert!(
            last(&w).starts_with("You have skill rolls to spend."),
            "{refused}: {}",
            last(&w)
        );
        assert_eq!(w.turn(), turn, "{refused} took no turn");
    }
    run(&mut w, "advance:nope");
    assert_eq!(last(&w), "There is no skill `nope`.");
    run(&mut w, "advance:stealth");
    assert_eq!(last(&w), "You are not trained in stealth.");
    let before = trained(&w, "athletics").expect("a warrior's").value;
    run(&mut w, "advance:athletics");
    assert_eq!(last(&w), "Your athletics improves.");
    assert!(trained(&w, "athletics").expect("trained").value > before);
    assert_eq!(w.due().map(|d| d.rolls), Some(2));
    // Every roll may go into one skill.
    run(&mut w, "advance:athletics advance:athletics");
    assert_eq!(w.due(), None);
    run(&mut w, "advance:athletics");
    assert_eq!(last(&w), "You have no skill rolls to spend.");
    run(&mut w, "wait");
    assert_eq!(w.turn(), turn + 1, "free to act again");
    // A third level asks for rolls and a talent: the rolls are said first.
    reach_after_rolls(&mut w, 3);
    assert_eq!(w.due().map(|d| (d.rolls, d.talents)), Some((3, 1)));
    run(&mut w, "talent:healthy");
    assert_eq!(
        last(&w),
        "You learn Healthy.",
        "a talent may come before the rolls"
    );
    run(&mut w, "wait");
    assert!(last(&w).starts_with("You have skill rolls to spend."));
    run(&mut w, "levelup:auto");
    assert_eq!(w.due(), None, "auto spends the rolls");
}

/// `reach`, from a world with no choice due.
fn reach_after_rolls(world: &mut World, level: u8) {
    assert_eq!(world.due(), None);
    reach(world, level);
}

#[test]
fn the_die_follows_the_marks_and_the_class_and_race() {
    // No marks and no affinity: +1. A human warrior trained in stealth by hand.
    assert_eq!(gains("human,male,warrior", "stealth", 10, 0, 6), (1, 1));
    // Known for (athletics): one step up, 1d3.
    let (low, high) = gains("human,male,warrior", "athletics", 10, 0, 12);
    assert!(low >= 1 && high <= 3 && high > low, "{low}..{high}");
    // One of the class's other skills (balance): +1 and 1.
    assert_eq!(gains("human,male,warrior", "balance", 10, 0, 6), (2, 2));
    // The race's (a dwarf's search): +1 too.
    assert_eq!(gains("dwarf,male,warrior", "search", 10, 0, 6), (2, 2));
    // Known for and racial stack: a halfling thief's stealth, 1d3 + 1.
    let (low, high) = gains("halfling,female,thief", "stealth", 10, 0, 12);
    assert!(low >= 2 && high <= 4, "{low}..{high}");
    // 50 marks: 3d3, and for a known-for skill 3d4.
    let (low, high) = gains("human,male,warrior", "stealth", 10, 50, 12);
    assert!(low >= 3 && high <= 9 && high > low, "{low}..{high}");
    let (low, high) = gains("human,male,warrior", "athletics", 10, 50, 12);
    assert!(low >= 3 && high <= 12, "{low}..{high}");
    // Past the soft cap a roll adds half, rounded up: 4d5 (4 to 20) is 2 to 10.
    let (low, high) = gains("human,male,warrior", "stealth", 100, 800, 12);
    assert!(low >= 2 && high <= 10, "{low}..{high}");
    assert_eq!(gains("human,male,warrior", "stealth", 150, 0, 4), (1, 1));
}

#[test]
fn a_roll_spends_half_the_marks_and_draws_from_the_levels_stream_only() {
    let mut w = world("human,male,warrior");
    reach(&mut w, 2);
    set(&mut w, "athletics", 20, 50);
    let gameplay = w.state.rng.clone();
    let training = w.state.training.clone();
    assert!(w.state.levels_rng.is_none(), "nothing drew from it yet");
    run(&mut w, "advance:athletics");
    assert_eq!(trained(&w, "athletics").map(|t| t.marks), Some(25));
    run(&mut w, "advance:athletics");
    assert_eq!(trained(&w, "athletics").map(|t| t.marks), Some(12));
    assert_eq!(w.state.rng, gameplay, "the gameplay stream is untouched");
    assert_eq!(w.state.training, training, "and the training stream");
    assert!(w.state.levels_rng.is_some());
    // The meter shows the ladder step: 12 marks is 1d5, a quarter.
    let view = w.perception();
    let own = view.own_skills();
    let athletics = own.iter().find(|s| s.skill == skill(&w, "athletics"));
    assert_eq!(athletics.map(|s| s.progress), Some(1));
}

#[test]
fn rolls_survive_saves_and_replays() {
    let content = BUILTIN.clone();
    let spec: CharacterSpec = "halfling,female,thief".parse().expect("a character");
    let mut a =
        World::with_character(content.clone(), "showcase", Some(3), Some(&spec)).expect("world");
    let mut replay = Replay::new(&a);
    let actions = "move:E xp:2000 advance:stealth wait";
    for action in parse_actions(actions).expect("actions") {
        a.apply(&action);
        replay.push(&action, &a);
    }
    assert!(
        a.due().is_some_and(|d| d.rolls > 0),
        "rolls left: wait refused"
    );
    let bytes = crate::save::encode(&a);
    let b = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(b.state(), a.state());
    assert_eq!(b.state_hash(), a.state_hash());
    let played = replay.run(content).expect("the replay verifies");
    assert_eq!(played.state(), a.state());
}
