//! Freshness (spec §8 "Hunger"; #1031): Appraise and Survival, summed, let the player tell how
//! fresh a corpse or food is, in words, when a check passes; one who fails or is untrained is
//! told nothing; eating food told trains Appraise.

use super::*;
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::rot;
use crate::time::GameTime;

fn content(eager: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    if eager {
        files.replace_in_table(
            "skills",
            "{ on = \"judged\", chance = 10 }",
            "{ on = \"judged\", chance = 100 }",
        );
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

/// A warrior in the showcase, a year and more into the calendar so old food can be made.
fn world(eager: bool) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w =
        World::with_character(content(eager), "showcase", Some(7), Some(&spec)).expect("world");
    w.state.time = GameTime::from_seconds(400 * 24 * 3600);
    w
}

fn skill(w: &World, name: &str) -> crate::skills::SkillId {
    w.content().skill_id(name).expect("skill")
}

/// Bread made at many different moments before now, so each draws its own chance.
fn breads(w: &World, count: u64) -> Vec<Stack> {
    let bread = w.content().object_id("bread").expect("bread");
    (0..count)
        .map(|i| {
            let mut s = Stack::new(bread, 1);
            let ago = 3600 * (i + 1) + 17 * i;
            s.died = Some(GameTime::from_seconds(w.time().seconds() - ago));
            s
        })
        .collect()
}

/// What the player is told about each of `stacks`.
fn told(w: &World, stacks: &[Stack]) -> Vec<Option<&'static str>> {
    let view = w.perception();
    stacks.iter().map(|s| view.judge(s)).collect()
}

#[test]
fn the_untrained_are_told_nothing() {
    let w = world(false);
    assert!(told(&w, &breads(&w, 40)).iter().all(Option::is_none));
}

#[test]
fn a_trained_eye_is_told_the_true_stage_when_the_check_passes_and_nothing_when_it_fails() {
    let mut w = world(false);
    let appraise = skill(&w, "appraise");
    w.player_mut().skills.add(appraise, 1);
    let food = breads(&w, 120);
    let words = told(&w, &food);
    let content = w.content().clone();
    for (stack, word) in food.iter().zip(&words) {
        if let Some(word) = word {
            let stage = rot::stage(&content, stack, w.time()).expect("food");
            let truth = if stage == rot::Stage::Gone {
                "rotten"
            } else {
                stage.id()
            };
            assert_eq!(*word, truth);
        }
    }
    let n = words.iter().flatten().count();
    assert!(n > 0 && n < food.len(), "some told, some not: {n}");
    // The same hour, the same answer.
    assert_eq!(words, told(&w, &food));
}

#[test]
fn survival_and_appraise_add_up() {
    let mut w = world(false);
    let (appraise, survival) = (skill(&w, "appraise"), skill(&w, "survival"));
    let food = breads(&w, 120);
    w.player_mut().skills.add(survival, 1);
    let one = told(&w, &food);
    w.player_mut().skills.add(appraise, 1);
    let both = told(&w, &food);
    for (a, b) in one.iter().zip(&both) {
        assert!(a.is_none() || a == b, "what one tells, both tell");
    }
    let count = |v: &[Option<&str>]| v.iter().flatten().count();
    assert!(
        count(&both) > count(&one),
        "{} vs {}",
        count(&both),
        count(&one)
    );
}

#[test]
fn no_food_no_word() {
    let mut w = world(false);
    let appraise = skill(&w, "appraise");
    w.player_mut().skills.add(appraise, 200);
    let dagger = Stack::new(w.content().object_id("dagger").expect("dagger"), 1);
    assert_eq!(w.perception().judge(&dagger), None);
}

#[test]
fn eating_food_whose_freshness_was_told_trains_appraise() {
    let mut w = world(true);
    let appraise = skill(&w, "appraise");
    w.player_mut().skills.add(appraise, 100);
    let marks = |w: &World| {
        let player = w.state.player();
        player.skills.get(appraise).expect("trained").marks
    };
    let food = breads(&w, 60);
    let words = told(&w, &food);
    let judged = food[words.iter().position(Option::is_some).expect("told")].clone();
    let missed = food[words.iter().position(Option::is_none).expect("missed")].clone();
    let before = marks(&w);
    let now = w.state.time;
    w.train_judged(&missed, now);
    assert_eq!(marks(&w), before, "nothing told, nothing learnt");
    w.train_judged(&judged, now);
    assert!(marks(&w) > before);
}
