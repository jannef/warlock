//! The warrior's Charge in the world (#1466, `world/charge.rs`): the rush ends beside the
//! target with a blow, it moves at least a cell, it costs a point and a point a cell, it
//! provokes no free blows, its refusals take no time and spend nothing, and a warrior holds
//! stamina from level 1.

use super::*;
use crate::action::AttackKind;
use crate::content::{ContentFiles, STAMINA};

const CREATURES: &str = r##"
[[creature]]
id = "ch_post"
name = "post"
glyph = "p"
fg = "#806040"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000

[[creature]]
id = "ch_friend"
name = "friend"
glyph = "f"
fg = "#806040"
weight = 60000
faction = "peaceful"
speed = 100
base_hp = 1000
"##;

fn level(map: &str) -> String {
    format!(
        r##"name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
{map}'''
[legend]
"@" = {{ terrain = "floor", spawn = "player" }}
"p" = {{ terrain = "floor", creature = "ch_post" }}
"f" = {{ terrain = "floor", creature = "ch_friend" }}
"+" = {{ terrain = "door_closed" }}
"##
    )
}

/// A world on `map` where nobody but the player acts, the player has Charge and 12 stamina.
fn world(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_ch_test.toml", CREATURES);
    files.insert("levels/ch_yard.toml", &level(map));
    files.insert(
        "scenarios/ch_yard.toml",
        "name = \"Yard\"\nstart_level = \"ch_yard\"\nhp = 1000\nstats = { toughness = 14 }\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "ch_yard", None).expect("world");
    let player = w.player_id();
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
        }
    }
    w.player_mut().talents.take("charge", None);
    let stamina = w.content().resource_id(STAMINA).expect("stamina");
    w.grant_resource(player, stamina);
    set_stamina(&mut w, 12);
    w
}

fn set_stamina(w: &mut World, value: u16) {
    let id = w.content().resource_id(STAMINA).expect("stamina");
    (w.player_mut().resources.get_mut(id).expect("the pool")).value = value;
}

fn stamina(w: &World) -> u16 {
    let id = w.content().resource_id(STAMINA).expect("stamina");
    w.state.player().resources.get(id).map_or(0, |p| p.value)
}

fn at(w: &World, x: i32, y: i32) -> ActorId {
    let level = &w.state.current().id;
    w.state.actor_at(level, Pos::new(x, y)).expect("someone").id
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn use_(w: &mut World, text: &str) -> Outcome {
    w.apply(&text.parse::<Action>().expect("action text"))
}

fn blows_by(outcome: &Outcome, by: ActorId) -> usize {
    (outcome.events.iter())
        .filter(|e| {
            matches!(e, Event::Attacked { attacker, kind: AttackKind::Melee, .. } if *attacker == by)
        })
        .count()
}

#[test]
fn a_charge_ends_beside_the_target_with_a_blow_for_a_point_and_a_point_a_cell() {
    let mut w = world("##########\n#@.....p.#\n##########\n");
    let player = w.player_id();
    let post = at(&w, 7, 1);
    let outcome = use_(&mut w, "use:charge:7,1");
    assert!(outcome.took_turn, "{:#?}", said(&w));
    assert_eq!(w.state.player().pos, Pos::new(6, 1), "beside the post");
    assert!(blows_by(&outcome, player) >= 1, "{:#?}", said(&w));
    let target_struck = (outcome.events.iter())
        .any(|e| matches!(e, Event::Attacked { target, .. } if *target == post));
    assert!(target_struck);
    // Five steps' time, the blow free.
    let mut w = world("##########\n#@.....p.#\n##########\n");
    let floor = w.state.current().terrain[Pos::new(2, 1)];
    let step = w.step_price(player, &w.content().terrain(floor).clone(), false);
    let mut events = Vec::new();
    let spent = w.use_class_at(
        crate::content::ClassActive::Charge,
        Pos::new(7, 1),
        &mut events,
    );
    assert_eq!(spent, Spent::Turn(5 * step));
    assert_eq!(stamina(&w), 12 - (1 + 5), "1 and 1 for each of the 5 cells");
}

#[test]
fn a_charge_moves_at_least_a_cell_and_its_refusals_take_no_time() {
    for (map, target) in [
        // Beside it already: no cell to gain.
        ("#####\n#@p.#\n#####\n", "use:charge:2,1"),
        // A wall, a closed door, a friend in the way.
        ("#######\n#@.#.p#\n#######\n", "use:charge:5,1"),
        ("#######\n#@.+.p#\n#######\n", "use:charge:5,1"),
        ("#######\n#@.f.p#\n#######\n", "use:charge:5,1"),
        // Nobody there; a friend there.
        ("#######\n#@....#\n#######\n", "use:charge:5,1"),
        ("#######\n#@...f#\n#######\n", "use:charge:5,1"),
    ] {
        let mut w = world(map);
        let before = (stamina(&w), w.state.player().pos, w.state.turn);
        let outcome = use_(&mut w, target);
        assert!(!outcome.took_turn, "{map}: {:#?}", said(&w));
        let after = (stamina(&w), w.state.player().pos, w.state.turn);
        assert_eq!(after, before, "{map}");
    }
    // Short of the distance's stamina: refused, nothing spent.
    let mut w = world("##########\n#@.....p.#\n##########\n");
    set_stamina(&mut w, 5);
    let outcome = use_(&mut w, "use:charge:7,1");
    assert!(!outcome.took_turn, "{:#?}", said(&w));
    assert_eq!(stamina(&w), 5);
    assert_eq!(w.state.player().pos, Pos::new(1, 1));
    // Exactly enough: it goes.
    set_stamina(&mut w, 6);
    let mut events = Vec::new();
    let spent = w.use_class_at(
        crate::content::ClassActive::Charge,
        Pos::new(7, 1),
        &mut events,
    );
    assert!(matches!(spent, Spent::Turn(_)));
    assert_eq!(stamina(&w), 0);
}

#[test]
fn a_charge_provokes_no_free_blows() {
    // An opportunist beside the player, another passed on the way.
    let map = "##########\n#p.......#\n#@......p#\n#...p....#\n##########\n";
    let mut w = world(map);
    let behind = at(&w, 1, 1);
    let passed = at(&w, 4, 3);
    for id in [behind, passed] {
        (w.state.actors.get_mut(&id).unwrap().free_blows).opportunist = true;
        (w.state.actors.get_mut(&id).unwrap().free_blows).gatekeeper = true;
    }
    let outcome = use_(&mut w, "use:charge:8,2");
    assert!(outcome.took_turn, "{:#?}", said(&w));
    assert_eq!(w.state.player().pos, Pos::new(7, 2));
    assert_eq!(blows_by(&outcome, behind), 0);
    assert_eq!(blows_by(&outcome, passed), 0);
    // A plain step away from the same opportunist is struck at.
    let mut w = world(map);
    let behind = at(&w, 1, 1);
    (w.state.actors.get_mut(&behind).unwrap().free_blows).opportunist = true;
    let outcome = w.apply(&Action::Move(Dir::SE));
    assert_eq!(blows_by(&outcome, behind), 1, "{:#?}", said(&w));
}

#[test]
fn a_warrior_holds_stamina_from_level_1_and_charge_is_in_their_menu() {
    let w = super::class_active_tests::world("warrior");
    let content = w.content().clone();
    let id = content.resource_id(STAMINA).expect("stamina");
    assert!(w.state.player().resources.get(id).is_some(), "the pool");
    assert!(w.state.player().talents.holds("charge", None));
    let ability = crate::ability::Ability::Class(crate::content::ClassActive::Charge);
    assert!(ability.had_by(w.state.player(), &content));
    assert_eq!(ability.aim(&content), crate::ability::Aim::Cell);
}
