//! Deep water (spec §8 "Skills": Swim; `world/swim.rs`, #1019).

use super::*;
use crate::burden::Burden;
use crate::content::ContentFiles;
use crate::object::{LightMode, Stack};
use crate::skills::{SkillId, Trained};
use std::sync::{Arc, LazyLock};

const POOL: &str = r#"
name = "Pool"
ambient = [255, 255, 255, 0]
map = '''
###########
#.........#
#.@~~~~~..#
#..~~~~~..#
#..~~~~~.k#
#.........#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = { terrain = "deep_water" }
"k" = { terrain = "floor", creature = "kobold", wield = "spear", carry = ["spear"] }
"#;

/// A pool in the dark, for water the player cannot see.
const DARK: &str = r#"
name = "Dark"
ambient = [0, 0, 0, 0]
map = '''
#####
#@~~#
#####
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"~" = { terrain = "deep_water" }
"#;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/mere.toml", POOL);
    files.insert("levels/dark.toml", DARK);
    files.insert(
        "scenarios/mere.toml",
        "name = \"mere\"\nstart_level = \"mere\"\nhp = 100\n",
    );
    files.insert(
        "scenarios/dark.toml",
        "name = \"dark\"\nstart_level = \"dark\"\nhp = 100\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// The start of the pool, its kobold gone unless a test wants it, the player trained at 50.
fn swimmer(seed: u64) -> World {
    let mut w = with_kobold(seed);
    let kobold = (w.state.actors.keys().copied()).find(|id| *id != w.state.player);
    w.state.actors.remove(&kobold.expect("the kobold"));
    w
}

fn with_kobold(seed: u64) -> World {
    let mut w = World::new(CONTENT.clone(), "mere", Some(seed)).expect("world");
    set_swim(&mut w, Some(50));
    w
}

fn set_swim(w: &mut World, value: Option<u16>) {
    let id = swim(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
    w.perceive(&mut Vec::new());
}

fn swim(w: &World) -> SkillId {
    w.content().skill_id("swim").expect("swim")
}

/// The player in the water, two cells from the east shore of the row they start on.
fn in_water(seed: u64) -> World {
    let mut w = swimmer(seed);
    place(&mut w, Pos::new(5, 3));
    w
}

fn place(w: &mut World, at: Pos) {
    w.player_mut().pos = at;
    w.perceive(&mut Vec::new());
}

fn carry(w: &mut World, level: Burden) {
    let rock = w.content().object_id("rock").expect("rock");
    let content = w.content().clone();
    while w.actor_load(w.state.player).expect("load").burden() < level {
        let stack = Stack::new(rock, 1);
        crate::object::add_to_inventory(&mut w.player_mut().inventory, stack, &content);
    }
}

fn last(w: &World) -> &str {
    w.messages().iter().last().map_or("", |m| m.text.as_str())
}

/// `action` is refused with `said`: no time, no draw, nothing changed.
#[track_caller]
fn refused(w: &mut World, action: Action, said: &str) {
    let (time, rng, at) = (w.state.time, w.state.rng.clone(), w.player_pos());
    let out = w.apply(&action);
    assert!(!out.took_turn, "{said}");
    assert_eq!(w.state.time, time, "{said}");
    assert_eq!(w.state.rng, rng, "{said}");
    assert_eq!(w.player_pos(), at, "{said}");
    assert_eq!(last(w), said);
}

/// A seed whose first stroke from the world `make` makes comes out `made`.
fn seed_where(made: bool, make: impl Fn(u64) -> World) -> World {
    (0..2000)
        .map(&make)
        .find(|w| {
            let mut w = w.clone();
            let from = w.player_pos();
            w.apply(&Action::Move(Dir::E));
            (w.player_pos() != from) == made
        })
        .expect("a seed")
}

#[test]
fn the_terrain_loads_as_the_issue_has_it_and_a_walkable_one_is_refused() {
    let content = &*CONTENT;
    let def = content.terrain(content.terrain_id("deep_water").expect("deep water"));
    assert!(def.swim && !def.walkable && def.transparent && def.freeze.is_none());
    assert!(!def.stains);
    assert_eq!((def.move_cost, def.step_noise), (200, 200));
    let lake = content.terrain(content.terrain_id("lake").expect("lake"));
    assert!(!lake.swim, "the overworld's lake is unchanged");
    assert_eq!((def.fg, def.bg), (lake.fg, lake.bg));
    let rules = content.swim().expect("data/swim.toml");
    assert_eq!((rules.difficulty, rules.per_burden, rules.cold), (11, 2, 1));

    let text = include_str!("../../../../assets/data/terrain.toml");
    let bad = text.replace(
        "walkable = false\nswim = true\n",
        "swim = true\nwalkable = true\n",
    );
    assert_ne!(text, bad);
    let mut files = files();
    files.insert("data/terrain.toml", &bad);
    let error = Content::load(&files).expect_err("refused").to_string();
    assert!(error.contains("deep water"), "{error}");
}

#[test]
fn the_shore_refuses_the_untrained_and_the_strained_with_no_time() {
    let mut w = swimmer(1);
    set_swim(&mut w, None);
    refused(&mut w, Action::Move(Dir::E), "You cannot swim.");
    assert!(!w.perception().dives_at(Pos::new(3, 2)), "nothing to ask");

    let mut w = swimmer(1);
    assert!(w.perception().dives_at(Pos::new(3, 2)), "it asks");
    carry(&mut w, Burden::Strained);
    w.perceive(&mut Vec::new());
    refused(&mut w, Action::Move(Dir::E), "You would sink.");
    assert!(!w.perception().dives_at(Pos::new(3, 2)), "nothing to ask");
}

#[test]
fn going_in_needs_no_check_and_a_step_in_the_water_asks_none_of_the_shore() {
    let mut w = swimmer(1);
    let out = w.apply(&Action::Move(Dir::E));
    assert!(out.took_turn);
    assert_eq!(w.player_pos(), Pos::new(3, 2));
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text == "You dive into the deep water.")
    );
    // Now in: the next cell of water asks no question, nor does the way out.
    assert!(!w.perception().dives_at(Pos::new(4, 2)));
    assert!(!w.perception().dives_at(Pos::new(2, 2)));
}

#[test]
fn water_found_in_the_dark_stops_the_player_so_the_next_step_can_ask() {
    let mut w = World::new(CONTENT.clone(), "dark", Some(1)).expect("world");
    set_swim(&mut w, Some(50));
    let at = w.player_pos();
    let east = Pos::new(at.x + 1, at.y);
    assert!(matches!(
        w.perception().cell(east),
        crate::perception::CellView::Unknown
    ));
    {
        let time = w.state.time;
        let out = w.apply(&Action::Move(Dir::E));
        assert!(!out.took_turn);
        assert_eq!((w.state.time, w.player_pos()), (time, at));
        assert!(last(&w).contains("deep water"), "{}", last(&w));
    }
    assert!(w.perception().dives_at(east), "known now: it asks");
    assert!(w.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(w.player_pos(), east);
}

#[test]
fn a_made_step_moves_at_its_price_and_a_failed_one_costs_the_turn_and_a_hit_point() {
    let mut made = seed_where(true, in_water);
    let hp = made.state.player().hp;
    let time = made.state.time;
    assert!(made.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(made.player_pos(), Pos::new(6, 3));
    assert_eq!(made.state.player().hp, hp);
    assert!(made.state.time > time);

    let mut failed = seed_where(false, in_water);
    let hp = failed.state.player().hp;
    let time = failed.state.time;
    assert!(failed.apply(&Action::Move(Dir::E)).took_turn);
    assert_eq!(failed.player_pos(), Pos::new(5, 3));
    assert_eq!(failed.state.player().hp, hp - 1);
    assert!(failed.state.time > time);
    assert!(
        failed
            .messages()
            .iter()
            .any(|m| m.text.contains("tread water"))
    );
}

#[test]
fn a_step_out_of_the_water_is_a_stroke_too() {
    let out_of = |seed| {
        let mut w = swimmer(seed);
        place(&mut w, Pos::new(7, 3));
        w
    };
    let mut made = seed_where(true, out_of);
    made.apply(&Action::Move(Dir::E));
    assert_eq!(made.player_pos(), Pos::new(8, 3));
    let mut failed = seed_where(false, out_of);
    failed.apply(&Action::Move(Dir::E));
    assert_eq!(failed.player_pos(), Pos::new(7, 3));
}

#[test]
fn the_cold_never_takes_the_last_hit_point() {
    let mut w = seed_where(false, in_water);
    w.player_mut().hp = 1;
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.state.player().hp, 1);
    assert!(w.state.death.is_none());
}

/// How many of `seeds` strokes are made under `burden`.
fn made_of(burden: Burden, seeds: u64) -> u64 {
    (0..seeds)
        .filter(|&seed| {
            let mut w = in_water(seed);
            carry(&mut w, burden);
            let from = w.player_pos();
            w.apply(&Action::Move(Dir::E));
            w.player_pos() != from
        })
        .count() as u64
}

#[test]
fn each_level_of_burden_makes_a_stroke_harder_and_a_strained_swimmer_can_still_swim() {
    let (free, burdened, strained) = (
        made_of(Burden::Unburdened, 300),
        made_of(Burden::Burdened, 300),
        made_of(Burden::Strained, 300),
    );
    assert!(
        free > burdened && burdened > strained,
        "{free} {burdened} {strained}"
    );
    assert!(strained > 0, "harder, but they still swim");
}

#[test]
fn a_step_in_the_water_draws_only_its_check_and_a_wait_asks_none() {
    for seed in 0..20 {
        let mut w = in_water(seed);
        let mut expected = w.state.rng.clone();
        let _ = crate::check::check(&mut expected, 0, 0);
        w.stroke_fails(&mut Vec::new());
        assert_eq!(w.state.rng, expected, "seed {seed}");
    }
    let mut w = in_water(7);
    let hp = w.state.player().hp;
    assert!(w.apply(&Action::Wait).took_turn);
    assert_eq!(w.state.player().hp, hp, "a wait asks no check");
    assert_eq!(w.player_pos(), Pos::new(5, 3));
}

#[test]
fn marks_fall_on_steps_in_the_water_and_never_on_wade() {
    let text = include_str!("../../../../assets/data/skills.toml");
    let eager = text.replace(
        "{ on = \"stroke\", chance = 25 }",
        "{ on = \"stroke\", chance = 100 }",
    );
    assert_ne!(text, eager);
    let wade = eager.replace(
        "{ on = \"wade\", chance = 10 }",
        "{ on = \"wade\", chance = 100 }",
    );
    let mut files = files();
    files.insert("data/skills.toml", &wade);
    let content = Arc::new(Content::load(&files).expect("eager content"));
    let mut w = World::new(content, "mere", Some(3)).expect("world");
    let kobold = (w.state.actors.keys().copied()).find(|id| *id != w.state.player);
    w.state.actors.remove(&kobold.expect("the kobold"));
    set_swim(&mut w, Some(50));
    let marks = |w: &World| {
        (w.state.player().skills.iter())
            .find(|(id, _)| *id == swim(w))
            .map_or(0, |(_, t)| t.marks)
    };
    // Going in is no step in it.
    w.apply(&Action::Move(Dir::E));
    assert_eq!(marks(&w), 0);
    // Made or not, a stroke earns its mark.
    for n in 1..=6 {
        let from = w.player_pos();
        w.apply(&Action::Move(Dir::SE));
        assert_eq!(marks(&w), n, "a stroke from {from:?}");
        w.player_mut().pos = from;
    }
}

#[test]
fn wading_prices_apply_to_deep_water() {
    let price_of = |w: &World| {
        let id = w.content().terrain_id("deep_water").expect("deep");
        w.step_price(w.state.player, w.content().terrain(id), false)
    };
    let mut w = swimmer(1);
    set_swim(&mut w, Some(0));
    assert_eq!(price_of(&w), 12000);
    set_swim(&mut w, Some(100));
    assert_eq!(price_of(&w), 8400, "Swim spares part of it");
    for talent in ["strong_swimmer", "undertow", "swimmer"] {
        let mut w = swimmer(1);
        w.player_mut().talents.take(talent, None);
        assert_eq!(price_of(&w), 6000, "{talent}");
    }
}

#[test]
fn in_the_water_nothing_is_dropped_or_taken_and_no_one_rests_or_searches_or_digs() {
    let mut w = in_water(1);
    let sink = "It would sink.";
    refused(&mut w, "drop".parse().expect("drop"), sink);
    refused(&mut w, "pickup".parse().expect("pickup"), sink);
    let rest = "rest:1m".parse().expect("rest");
    refused(&mut w, rest, "You cannot rest in deep water.");
    refused(
        &mut w,
        Action::Search(1),
        "You cannot search in deep water.",
    );
    refused(&mut w, Action::Dig(Dir::E), "You cannot dig in deep water.");
    assert!(w.apply(&Action::Wait).took_turn, "a wait goes on");
}

/// A world whose player has a lantern ready in the Tool slot, lit or not.
fn lantern(w: &mut World, mode: LightMode) -> usize {
    let content = w.content().clone();
    let object = content.object_id("lantern").expect("lantern");
    let mut stack = Stack::new(object, 1);
    stack.set_light_mode(mode);
    let player = w.player_mut();
    player.inventory.push(stack);
    player.inventory.len() - 1
}

#[test]
fn going_in_snuffs_the_light_and_none_can_be_lit_afloat() {
    let mut w = swimmer(1);
    let item = lantern(&mut w, LightMode::Lit);
    assert!(
        w.apply(&Action::Equip {
            slot: "tool".into(),
            item
        })
        .took_turn
    );
    let lit = |w: &World| {
        let me = w.state.player();
        me.ready_light(w.content())
            .map(|i| me.inventory[i].light_mode())
    };
    assert_eq!(lit(&w), Some(LightMode::Lit));
    w.apply(&Action::Move(Dir::E));
    assert_eq!(lit(&w), Some(LightMode::Unlit));
    assert!(w.messages().iter().any(|m| m.text.contains("goes out")));
    refused(&mut w, Action::UseTool, "The water would only put it out.");
    assert_eq!(lit(&w), Some(LightMode::Unlit));
    // A second one, burning in the pack, goes out as it is made ready.
    let other = lantern(&mut w, LightMode::Lit);
    w.apply(&Action::Equip {
        slot: "tool".into(),
        item: other,
    });
    assert_eq!(lit(&w), Some(LightMode::Unlit));
}

#[test]
fn a_creature_goes_around_and_waits_on_the_shore_and_strikes_only_from_beside() {
    let mut w = with_kobold(5);
    let kobold = (w.state.actors.keys().copied())
        .find(|id| *id != w.state.player)
        .expect("kobold");
    let water = |w: &World, p: Pos| {
        let t = w.state.current().terrain[p];
        w.content().terrain(t).swim
    };
    place(&mut w, Pos::new(5, 3));
    w.player_mut().hp = 100;
    for _ in 0..40 {
        w.apply(&Action::Wait);
        let at = w.state.actors[&kobold].pos;
        assert!(!water(&w, at), "it stays out of the water");
    }
    // The middle of the pool is two cells from the nearest ground.
    let at = w.state.actors[&kobold].pos;
    assert_eq!(at.chebyshev(w.player_pos()), 2, "on the shore, at {at:?}");
    assert!(w.state.player().hp >= 100, "out of reach");
}

#[test]
fn an_ally_stays_ashore_and_will_not_be_swapped_into_the_water() {
    let mut w = swimmer(5);
    place(&mut w, Pos::new(2, 2));
    w.apply(&"ally:wolf".parse().expect("ally"));
    let wolf = (w.state.actors.keys().copied())
        .find(|id| *id != w.state.player)
        .expect("wolf");
    let water = |w: &World, p: Pos| {
        let t = w.state.current().terrain[p];
        w.content().terrain(t).swim
    };
    place(&mut w, Pos::new(5, 3));
    for _ in 0..30 {
        w.apply(&Action::Wait);
        assert!(!water(&w, w.state.actors[&wolf].pos));
    }
    // Beside it, climbing out where it stands: it will not take the swimmer's place.
    place(&mut w, Pos::new(7, 3));
    w.state.actors.get_mut(&wolf).expect("wolf").pos = Pos::new(8, 3);
    w.perceive(&mut Vec::new());
    let out = w.apply(&Action::Move(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(w.player_pos(), Pos::new(7, 3));
    assert!(
        last(&w).contains("will not go into the water"),
        "{}",
        last(&w)
    );
}
