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
use crate::test_support::{CONTENT as BUILTIN, last, run};

fn world(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
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
        w.level_rolls(w.player_level())
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
    assert_eq!(w.level_rolls(w.player_level()), 0);
    reach(&mut w, 2);
    assert_eq!(w.due(), None, "nothing due");
}

/// The roll the level-up screen shows for `id` at `value` with `marks` (#543).
fn shown(character: &str, id: &str, value: u16, marks: u32) -> SkillRoll {
    let mut w = world(character);
    set(&mut w, id, value, marks);
    let skill = skill(&w, id);
    w.perception().skill_roll(skill).expect("trained")
}

#[test]
fn the_screen_shows_the_dice_the_rule_rolls_and_where_the_rolls_come_from() {
    let dice = |c: &str, id: &str, value: u16, marks: u32| {
        let roll = shown(c, id, value, marks);
        (roll.dice().to_string(), roll.bonus, roll.halved)
    };
    let w = "human,male,warrior";
    assert_eq!(dice(w, "stealth", 10, 0), ("+1".into(), None, false));
    assert_eq!(dice(w, "athletics", 10, 0), ("1d3".into(), None, false));
    let balance = dice(w, "balance", 10, 0);
    assert_eq!(balance, ("+2".into(), Some(RollBonus::Class), false));
    let search = dice("dwarf,male,warrior", "search", 10, 0);
    assert_eq!(search, ("+2".into(), Some(RollBonus::Race), false));
    let stealth = dice("halfling,female,thief", "stealth", 10, 0);
    assert_eq!(stealth, ("1d3+1".into(), Some(RollBonus::Race), false));
    assert_eq!(dice(w, "stealth", 10, 50), ("3d3".into(), None, false));
    let athletics = shown(w, "athletics", 10, 50);
    assert_eq!(
        (athletics.practice, athletics.known),
        ("3d3".parse().unwrap(), true)
    );
    assert_eq!(athletics.die.to_string(), "3d4");
    assert_eq!(dice(w, "stealth", 100, 800), ("4d5".into(), None, true));
    // What the rule rolls stays within the dice shown (halved past the soft cap).
    for (id, value, marks) in [("athletics", 10, 50), ("stealth", 100, 800)] {
        let roll = shown(w, id, value, marks);
        let (mut low, mut high) = (roll.dice().min(), roll.dice().max());
        if roll.halved {
            (low, high) = ((low + 1) / 2, (high + 1) / 2);
        }
        let (got_low, got_high) = gains(w, id, value, marks, 12);
        assert!(
            i32::from(got_low) >= low && i32::from(got_high) <= high,
            "{id}: {got_low}..{got_high} outside {low}..{high}"
        );
    }
    // The count's parts add up to what a level gives.
    let mut w = world("human,male,warrior");
    set_intelligence(&mut w, 18);
    let count = w.perception().roll_count();
    let parts = (count.class, count.race, count.intelligence, count.talents);
    assert_eq!(parts, (2, 1, 2, 0));
    assert_eq!(count.total(), w.level_rolls(w.player_level()));
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
            rolls: 3,
            bonus: 0,
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
    let levels = w.state.levels_rng.clone();
    run(&mut w, "advance:athletics");
    assert_eq!(trained(&w, "athletics").map(|t| t.marks), Some(25));
    run(&mut w, "advance:athletics");
    assert_eq!(trained(&w, "athletics").map(|t| t.marks), Some(12));
    assert_eq!(w.state.rng, gameplay, "the gameplay stream is untouched");
    assert_eq!(w.state.training, training, "and the training stream");
    assert_ne!(
        w.state.levels_rng, levels,
        "the rolls drew from the levels stream"
    );
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

/// The automatic roll (#1328): marks reaching `[rolls] auto_at` roll once, at once.
#[test]
fn marks_reaching_the_threshold_roll_once_and_spend_half() {
    let mut w = world("human,male,warrior");
    let auto_at = w.content().skill_rolls().auto_at;
    assert_eq!(auto_at, 100, "the data's threshold");
    set(&mut w, "stealth", 20, auto_at - 2);
    let id = skill(&w, "stealth");
    let gameplay = w.state.rng.clone();
    assert!(w.mark_skill(id, 1));
    assert_eq!(
        trained(&w, "stealth"),
        Some(Trained {
            value: 20,
            marks: auto_at - 1
        }),
        "below the threshold, nothing"
    );
    assert!(w.mark_skill(id, 1));
    let after = trained(&w, "stealth").expect("trained");
    assert_eq!(after.marks, auto_at / 2, "the roll spent half");
    assert!(after.value > 20, "and raised the value");
    assert_eq!(last(&w), "Your stealth improves.");
    assert_eq!(w.state.rng, gameplay, "the gameplay stream is untouched");
    // One more mark does not fire it again.
    let value = after.value;
    assert!(w.mark_skill(id, 1));
    assert_eq!(trained(&w, "stealth").map(|t| t.value), Some(value));
    // A big gift rolls until the marks are under the threshold: 300 → 150 → 75.
    set(&mut w, "stealth", 20, 0);
    assert!(w.mark_skill(id, 300));
    assert_eq!(trained(&w, "stealth").map(|t| t.marks), Some(75));
    // An untrained skill earns nothing and rolls nothing.
    let mut bare = world("human,male,warrior");
    bare.player_mut().skills = Default::default();
    assert!(!bare.mark_skill(id, 500));
    assert_eq!(trained(&bare, "stealth"), None);
}

#[test]
fn the_automatic_roll_is_a_level_up_rolls_dice_and_leaves_the_due_alone() {
    for character in ["human,male,warrior", "halfling,female,thief"] {
        for seed in 0..6 {
            let mut w = world(character);
            reach(&mut w, 2);
            let due = w.due();
            assert!(due.is_some(), "rolls due");
            // Both worlds draw the same levels stream from here.
            w.state.levels_rng = Some(crate::rng::Rng::levels(seed));
            let mut chosen = w.clone();
            let mut auto = w;
            set(&mut chosen, "stealth", 20, 100);
            run(&mut chosen, "advance:stealth");
            set(&mut auto, "stealth", 20, 99);
            let id = skill(&auto, "stealth");
            auto.mark_skill(id, 1);
            assert_eq!(
                trained(&auto, "stealth"),
                trained(&chosen, "stealth"),
                "{character} {seed}: the same roll"
            );
            assert_eq!(auto.state.levels_rng, chosen.state.levels_rng);
            assert_eq!(auto.due(), due, "the level's rolls untouched");
        }
    }
}

#[test]
fn training_rolls_by_itself_and_replays() {
    // A dwarf searches (Search: a mark one search in ten) until the marks reach the threshold.
    let content = BUILTIN.clone();
    let spec: CharacterSpec = "dwarf,male,warrior".parse().expect("a character");
    let mut a =
        World::with_character(content.clone(), "showcase", Some(3), Some(&spec)).expect("world");
    let search = skill(&a, "search");
    let start = a.state.player().skills.value(search).expect("trained");
    let mut replay = Replay::new(&a);
    let mut improved = false;
    for _ in 0..30 {
        for action in parse_actions("search:100").expect("actions") {
            a.apply(&action);
            replay.push(&action, &a);
        }
        improved |= (a.messages().iter()).any(|m| m.display() == "Your search improves.");
        if improved {
            break;
        }
    }
    assert!(improved, "the search skill rolled by itself");
    assert!(a.state.player().skills.value(search).expect("trained") > start);
    assert_eq!(a.due(), None, "no level-up was needed");
    let played = replay.run(content).expect("the replay verifies");
    assert_eq!(played.state(), a.state());
}

#[test]
fn the_professions_skills_load_can_be_trained_and_rolled() {
    // Jewelcraft, Mining, Herbalism, Alchemy, and Enchanting (#1331): every one but Jewelcraft
    // has its first effect (#1343); Mining, Herbalism, Alchemy (brewing, #1357), and Enchanting (salvage, #1428) mark, and Jewelcraft's craft and
    // salvage mark (#1333, `world/salvage_tests.rs`).
    let mut w = world("human,male,warrior");
    for id in ["alchemy", "enchanting", "herbalism", "mining"] {
        let def = w.content().skill(skill(&w, id));
        assert_eq!(def.benefits.is_empty(), id == "jewelcraft", "{id}");
        assert_eq!(
            def.marks.is_empty(),
            !["alchemy", "enchanting", "herbalism", "mining"].contains(&id),
            "{id}"
        );
        assert_eq!(
            trained(&w, id),
            None,
            "{id}: a human warrior starts untrained"
        );
    }
    reach(&mut w, 2);
    for id in ["alchemy", "enchanting", "herbalism", "jewelcraft", "mining"] {
        let mut probe = w.clone();
        set(&mut probe, id, 10, 0);
        run(&mut probe, &format!("advance:{id}"));
        assert!(
            trained(&probe, id).expect("trained").value > 10,
            "{id} rolled"
        );
    }
}
