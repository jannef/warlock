//! Swim (spec §8 "Skills"; `step_price`, `wade`, `world/ice.rs`): wading made cheaper, and a
//! plunge through the ice halved by a check.

use super::*;
use crate::check::check;
use crate::content::ContentFiles;
use crate::skills::{Effect, SkillId, Trained, Use};
use crate::stats::Stat;
use std::sync::{Arc, LazyLock};

const POND: &str = r#"
name = "Pond"
temperature = -5
ambient = [255, 255, 255, 0]
map = '''
########
#@.~,mb#
#.t~~~~#
########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = { terrain = "water" }
"," = { terrain = "shallow_water" }
"m" = { terrain = "marsh" }
"b" = { terrain = "bush" }
"t" = { terrain = "water", ice = 20 }
"#;

const THIN: Pos = Pos::new(2, 2);
const SCENARIO: &str = "name = \"pond\"\nstart_level = \"pond\"\nhp = 100\n";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pond.toml", POND);
    files.insert("scenarios/pond.toml", SCENARIO);
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// The same, a step's wading mark made certain.
static EAGER: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = files();
    files.replace_in_table(
        "skills",
        "{ on = \"wade\", chance = 10 }",
        "{ on = \"wade\", chance = 100 }",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

fn world(seed: u64) -> World {
    World::new(CONTENT.clone(), "pond", Some(seed)).expect("world")
}

fn swim(w: &World) -> SkillId {
    w.content().skill_id("swim").expect("swim")
}

/// The player trained in Swim at `value` (untrained with `None`).
fn set_swim(w: &mut World, value: Option<u16>) {
    let id = swim(w);
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
        .find(|(id, _)| *id == swim(w))
        .map(|(_, trained)| trained.marks)
}

fn price(w: &World, terrain: &str) -> i64 {
    let id = w.content().terrain_id(terrain).expect("terrain");
    w.step_price(w.player_id(), w.content().terrain(id), false)
}

const KEPT: &str = "You keep your head, and the cold bites less deep.";

fn kept(w: &World) -> bool {
    w.messages().iter().any(|m| m.text == KEPT)
}

#[test]
fn the_price_of_wading_falls_by_the_curve() {
    let mut w = world(1);
    let dry = price(&w, "floor");
    assert_eq!(dry, 6000);
    // Water doubles a step, shallow water and a marsh half as much again.
    for (value, water, shallow) in [
        (None, 12000, 9000),
        (Some(0), 12000, 9000),
        (Some(50), 10200, 8100),
        (Some(100), 8400, 7200),
        (Some(150), 7200, 6600),
        (Some(200), 6000, 6000),
        (Some(400), 6000, 6000),
    ] {
        set_swim(&mut w, value);
        assert_eq!(price(&w, "water"), water, "water at {value:?}");
        assert_eq!(price(&w, "running_water"), water, "stream at {value:?}");
        assert_eq!(price(&w, "shallow_water"), shallow, "shallows at {value:?}");
        assert_eq!(price(&w, "marsh"), shallow, "marsh at {value:?}");
        // Slow ground that is not water, and the dry, are none of Swim's business.
        assert_eq!(price(&w, "bush"), 12000, "bush at {value:?}");
        assert_eq!(price(&w, "floor"), dry);
    }
}

#[test]
fn swim_never_goes_past_the_talents_that_spare_all_of_it() {
    for talent in ["strong_swimmer", "undertow"] {
        let mut w = world(1);
        w.player_mut().talents.take(talent, None);
        assert_eq!(price(&w, "water"), 6000, "{talent}");
        for value in [0, 100, 200, 400] {
            set_swim(&mut w, Some(value));
            assert_eq!(price(&w, "water"), 6000, "{talent} at {value}");
        }
    }
}

#[test]
fn a_journey_on_the_overworld_is_none_of_swims_business() {
    let mut w = World::new(EAGER.clone(), "world", None).expect("world");
    set_swim(&mut w, None);
    let plain = price(&w, "marsh");
    set_swim(&mut w, Some(200));
    assert_eq!(price(&w, "marsh"), plain);
    let marsh = w.content().terrain_id("marsh").expect("marsh");
    w.wade(marsh, 1);
    assert_eq!(marks(&w), Some(0), "no mark on the map");
}

#[test]
fn wading_steps_mark_swim_one_chance_in_ten() {
    let rules = |c: &Content| {
        let id = c.skill_id("swim").expect("swim");
        (c.skill(id).marks.iter())
            .map(|m| (m.on, m.chance))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        rules(&CONTENT),
        [(Use::Wade, 10), (Use::Swim, 100), (Use::Stroke, 25)]
    );

    let mut w = World::new(EAGER.clone(), "pond", Some(1)).expect("world");
    set_swim(&mut w, Some(10));
    let walk = crate::action::parse_actions("move:E move:E move:E move:E move:E").expect("walk");
    // Dry floor, water, shallow water, marsh: a mark for each wading step, none on the floor...
    let earned: Vec<_> = walk[..4]
        .iter()
        .map(|a| {
            w.apply(a);
            marks(&w).expect("trained")
        })
        .collect();
    assert_eq!(earned, [0, 1, 2, 3]);
    // ... nor in a bush, slow as it is.
    w.apply(&walk[4]);
    assert_eq!(marks(&w), Some(3));

    // Untrained: no marks and nothing drawn; one who wades free still earns them.
    let mut w = World::new(EAGER.clone(), "pond", Some(1)).expect("world");
    set_swim(&mut w, None);
    let water = w.content().terrain_id("water").expect("water");
    let training = w.state.training.clone();
    w.wade(water, 1);
    assert_eq!(w.state.training, training);
    assert_eq!(marks(&w), None);
    set_swim(&mut w, Some(10));
    w.player_mut().talents.take("strong_swimmer", None);
    w.wade(water, 1);
    assert_eq!(marks(&w), Some(1));
}

#[test]
fn the_swim_check_is_toughness_and_the_curve_at_most_twenty() {
    let mut w = world(1);
    let effect = |w: &World| {
        let content = w.content().clone();
        w.state.player().skill_effect(&content, Effect::Swim)
    };
    for (value, points) in [(0, 0), (50, 4), (100, 8), (200, 12), (999, 12)] {
        set_swim(&mut w, Some(value));
        assert_eq!(effect(&w), points, "at {value}");
    }
    assert_eq!(Effect::Swim.max(), 20);
}

#[test]
fn the_plunge_is_halved_when_the_check_is_made() {
    let (mut made, mut failed, mut spared) = (0, 0, 0);
    for seed in 0..80 {
        let mut w = world(seed);
        set_swim(&mut w, Some(0));
        let player = w.player_id();
        let plunge = {
            let water = w.content().terrain_id("water").expect("water");
            w.content().terrain(water).freeze.expect("freezes").plunge
        };
        let modifier = {
            let content = w.content().clone();
            w.state
                .player()
                .effective_stats(&content)
                .modifier(Stat::Toughness)
        };
        let hp = w.state.player().hp;
        let mut expected = w.state.rng.clone();
        let rolled = plunge.roll(&mut expected);
        let kept_up = check(&mut expected, modifier, 11);
        w.fall_in(player, THIN, &mut Vec::new());
        assert_eq!(
            w.state.rng, expected,
            "seed {seed}: the roll, then two draws"
        );
        let lost = hp - w.state.player().hp;
        assert_eq!(
            lost,
            if kept_up { rolled / 2 } else { rolled },
            "seed {seed}"
        );
        assert_eq!(kept(&w), kept_up, "seed {seed}");
        // A mark whatever came of it.
        assert_eq!(marks(&w), Some(1), "seed {seed}");
        if kept_up {
            made += 1;
            spared += rolled - lost;
        } else {
            failed += 1;
        }
    }
    assert!(
        made > 0 && failed > 0 && spared > 0,
        "{made} {failed} {spared}"
    );
}

#[test]
fn the_untrained_get_no_check_and_no_draw() {
    let mut w = world(3);
    set_swim(&mut w, None);
    let player = w.player_id();
    let water = w.content().terrain_id("water").expect("water");
    let plunge = w.content().terrain(water).freeze.expect("freezes").plunge;
    let hp = w.state.player().hp;
    let mut expected = w.state.rng.clone();
    let rolled = plunge.roll(&mut expected);
    let training = w.state.training.clone();
    w.fall_in(player, THIN, &mut Vec::new());
    assert_eq!(w.state.rng, expected);
    assert_eq!(w.state.training, training);
    assert_eq!(hp - w.state.player().hp, rolled);
    assert!(!kept(&w));
}

#[test]
fn one_a_plunge_cannot_harm_asks_no_check() {
    let mut w = world(3);
    set_swim(&mut w, Some(100));
    w.player_mut().talents.take("swimmer", None);
    assert!(w.player_mut().talent_mods(&CONTENT).movement.plunge_proof);
    let player = w.player_id();
    let rng = w.state.rng.clone();
    let hp = w.state.player().hp;
    w.fall_in(player, THIN, &mut Vec::new());
    assert_eq!(w.state.rng, rng);
    assert_eq!(w.state.player().hp, hp);
    assert!(!kept(&w));
    assert_eq!(marks(&w), Some(0));
}

#[test]
fn walking_onto_thin_ice_asks_the_check_on_the_way_in() {
    // Ice this thin holds nobody: the plunge's line, then the check's.
    let mut made = false;
    for seed in 0..20 {
        let mut w = world(seed);
        set_swim(&mut w, Some(200));
        for action in crate::action::parse_actions("move:SE").expect("step") {
            w.apply(&action);
        }
        assert_eq!(w.player_pos(), THIN);
        // The check's mark; the step into the water may earn a wading one too.
        assert!(marks(&w).expect("trained") >= 1, "seed {seed}");
        let texts: Vec<_> = w.messages().iter().map(|m| m.text.clone()).collect();
        let plunge = "The ice cracks beneath you and you plunge into the freezing water!";
        let at = texts.iter().position(|t| t == plunge).expect("a plunge");
        made |= texts.get(at + 1).is_some_and(|t| t == KEPT);
    }
    assert!(made);
}
