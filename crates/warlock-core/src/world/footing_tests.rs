//! Balance on rough ground and when Staggered (spec §8 "Skills"; `world/footing.rs`).

use super::*;
use crate::content::ContentFiles;
use crate::skills::{SkillId, Trained, Use};
use std::sync::{Arc, LazyLock};

const YARD: &str = r#"
name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
##########
#@.bur~m.#
##########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"b" = { terrain = "bush" }
"u" = { terrain = "rubble" }
"r" = { terrain = "reeds" }
"~" = { terrain = "water" }
"m" = { terrain = "marsh" }
"#;

const SCENARIO: &str = "name = \"yard\"\nstart_level = \"yard\"\nhp = 100\n";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/yard.toml", YARD);
    files.insert("scenarios/yard.toml", SCENARIO);
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// The same, with Balance's new marks made certain.
static EAGER: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = files();
    files.replace_in_table(
        "skills",
        "{ on = \"rough_step\", chance = 10 }",
        "{ on = \"rough_step\", chance = 100 }",
    );
    files.replace_in_table(
        "skills",
        "{ on = \"staggered\", chance = 20 }",
        "{ on = \"staggered\", chance = 100 }",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(content: &Arc<Content>) -> World {
    World::new(content.clone(), "yard", Some(1)).expect("world")
}

fn balance(w: &World) -> SkillId {
    w.content().skill_id("balance").expect("balance")
}

/// The player trained in Balance at `value` (untrained with `None`).
fn set_balance(w: &mut World, value: Option<u16>) {
    let id = balance(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn marks(w: &World) -> Option<u32> {
    (w.state.player().skills.iter())
        .find(|(id, _)| *id == balance(w))
        .map(|(_, trained)| trained.marks)
}

fn price(w: &World, terrain: &str) -> i64 {
    let id = w.content().terrain_id(terrain).expect("terrain");
    w.step_price(w.player_id(), w.content().terrain(id), false)
}

fn stagger(w: &mut World) {
    let status = w.content().status_id("staggered").expect("staggered");
    let player = w.player_id();
    // Given between the player's actions, as a creature's blow does, not during one.
    w.acting = None;
    w.afflict(player, status, 1, None, &mut Vec::new());
}

#[test]
fn rough_ground_costs_less_by_the_curve_and_water_is_not_rough() {
    let mut w = world(&CONTENT);
    assert_eq!(price(&w, "floor"), 6000);
    // Bush doubles a step, reeds and rubble add half; 60 % of it spared at 100, all at 200.
    for (value, bush, reeds) in [
        (None, 12000, 9000),
        (Some(0), 12000, 9000),
        (Some(50), 10200, 8100),
        (Some(100), 8400, 7200),
        (Some(200), 6000, 6000),
        (Some(400), 6000, 6000),
    ] {
        set_balance(&mut w, value);
        assert_eq!(price(&w, "bush"), bush, "bush at {value:?}");
        assert_eq!(price(&w, "reeds"), reeds, "reeds at {value:?}");
        assert_eq!(price(&w, "rubble"), reeds, "rubble at {value:?}");
        // Water and marsh are wading: Balance has no say.
        assert_eq!(price(&w, "water"), 12000, "water at {value:?}");
        assert_eq!(price(&w, "marsh"), 9000, "marsh at {value:?}");
        assert_eq!(price(&w, "floor"), 6000);
    }
}

#[test]
fn the_overworld_is_no_rough_ground() {
    let w = World::new(CONTENT.clone(), "world", None).expect("world");
    let bush = w.content().terrain_id("bush").expect("bush");
    assert!(!w.rough_step(w.content().terrain(bush)));
}

#[test]
fn rough_steps_mark_balance_one_chance_in_ten_and_water_none() {
    let id = CONTENT.skill_id("balance").expect("balance");
    let rules: Vec<_> = (CONTENT.skill(id).marks.iter())
        .map(|m| (m.on, m.chance))
        .collect();
    assert_eq!(
        rules,
        [
            (Use::Burdened, 5),
            (Use::RoughStep, 10),
            (Use::Staggered, 20)
        ]
    );

    let mut w = world(&EAGER);
    set_balance(&mut w, Some(10));
    let walk = crate::action::parse_actions("move:E move:E move:E move:E move:E move:E move:E")
        .expect("walk");
    // Floor, bush, rubble, reeds, water, marsh: a mark for each rough step only.
    let earned: Vec<_> = walk[..6]
        .iter()
        .map(|a| {
            w.apply(a);
            marks(&w).expect("trained")
        })
        .collect();
    assert_eq!(earned, [0, 1, 2, 3, 3, 3]);

    // Untrained: nothing to mark.
    let mut w = world(&EAGER);
    set_balance(&mut w, None);
    w.apply(&walk[0]);
    w.apply(&walk[1]);
    assert_eq!(marks(&w), None);
}

#[test]
fn staggers_extra_cost_is_eased_by_balance() {
    let mut w = world(&CONTENT);
    let player = w.player_id();
    let status = w.content().status_id("staggered").expect("staggered");
    assert_eq!(w.content().status(status).cost, 150);
    for (value, cost) in [
        (None, 150),
        (Some(0), 150),
        (Some(100), 125),
        (Some(200), 112),
        (Some(400), 112),
    ] {
        set_balance(&mut w, value);
        assert_eq!(w.status_cost(player, status), cost, "at {value:?}");
    }
}

#[test]
fn an_action_taken_staggered_marks_balance_and_the_stagger_wears_off() {
    let mut w = world(&EAGER);
    set_balance(&mut w, Some(10));
    let status = w.content().status_id("staggered").expect("staggered");
    stagger(&mut w);
    assert!(w.state.player().effects.has(status));
    let wait = crate::action::parse_actions("move:E")
        .expect("move")
        .remove(0);
    w.apply(&wait);
    assert!(!w.state.player().effects.has(status), "the action ends it");
    assert_eq!(marks(&w), Some(1));
    // Not staggered: no mark.
    let step = crate::action::parse_actions("move:W")
        .expect("move")
        .remove(0);
    w.apply(&step);
    assert_eq!(marks(&w), Some(1));
}
