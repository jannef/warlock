//! The warrior's Charge in the world (#1466, `world/classes/charge.rs`): the rush ends beside the
//! target with a blow, it moves at least a cell, it costs a point and a point a cell, it
//! provokes no free blows, its refusals take no time and spend nothing, and a warrior holds
//! stamina from level 1; Whirling charge strikes the hostiles beside the way, and Battle rhythm
//! gives stamina back for melee hits and kills (#1467); each cell is entered as a step enters
//! it (#1615).

use crate::action::AttackKind;
use crate::content::{ContentFiles, STAMINA};
use crate::test_support::said;
use crate::world::*;

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
    world_with(map, "")
}

/// [`world`] with `more` after the level file's legend (`[[shop]]`, `[[pile]]` tables).
fn world_with(map: &str, more: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_ch_test.toml", CREATURES);
    files.insert("levels/ch_yard.toml", &format!("{}{more}", level(map)));
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
fn a_charge_out_of_a_shop_owing_is_refused() {
    // A shop of four cells, its door open at (3, 2), a dagger at the player's feet (#1488).
    let map = "##########\n#.@#.....#\n#..'...p.#\n##########\n";
    let shop = "\"'\" = { terrain = \"door_open\" }\n[[pile]]\nat = [2, 1]\nobjects = [\"dagger\"]\n\
                [[shop]]\nkind = \"weaponsmith\"\nkeeper = \"Orvald\"\narea = [1, 1, 2, 2]\n\
                post = [1, 1]\n";
    let mut w = world_with(map, shop);
    assert_eq!(w.state.current().shops[0].exits, [Pos::new(3, 2)]);
    use_(&mut w, "pickup");
    use_(&mut w, "move:S");
    assert!(!w.state.current().shops[0].unpaid.is_empty(), "owing");
    let before = (stamina(&w), w.state.player().pos, w.state.turn);
    let outcome = use_(&mut w, "use:charge:7,2");
    assert!(!outcome.took_turn, "{:#?}", said(&w));
    assert_eq!((stamina(&w), w.state.player().pos, w.state.turn), before);
    assert!(said(&w).iter().any(|m| m.contains("you haven't paid")));
    assert!(!w.state.current().shops[0].unpaid.is_empty(), "no theft");
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

fn blows_at(outcome: &Outcome, by: ActorId, at: ActorId) -> usize {
    (outcome.events.iter())
        .filter(|e| {
            matches!(e, Event::Attacked { attacker, target, .. } if *attacker == by && *target == at)
        })
        .count()
}

/// Hostile posts above and below the way, a friend beside it, the target at its end.
const WHIRL: &str = "##########\n#..p.p...#\n#@......p#\n#...f.p..#\n##########\n";

#[test]
fn whirling_charge_strikes_each_hostile_beside_the_way_once_and_the_target_once() {
    let mut w = world(WHIRL);
    let player = w.player_id();
    w.player_mut().talents.take("whirling_charge", None);
    let passed = [at(&w, 3, 1), at(&w, 5, 1), at(&w, 6, 3)];
    let (friend, target) = (at(&w, 4, 3), at(&w, 8, 2));
    let outcome = use_(&mut w, "use:charge:8,2");
    assert!(outcome.took_turn, "{:#?}", said(&w));
    assert_eq!(w.state.player().pos, Pos::new(7, 2));
    for post in passed {
        assert_eq!(blows_at(&outcome, player, post), 1, "{:#?}", said(&w));
    }
    assert_eq!(blows_at(&outcome, player, friend), 0, "never a friend");
    assert_eq!(
        blows_at(&outcome, player, target),
        1,
        "the charge's own blow only"
    );
    // The blows come in the order the cells are passed, the target's last.
    let order: Vec<ActorId> = (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker, target, ..
            } if *attacker == player => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(order, [passed[0], passed[1], passed[2], target]);
    // Without the talent nobody beside the way is struck; free blows, the charge costs the
    // same stamina and time with them.
    let mut plain = world(WHIRL);
    let without = use_(&mut plain, "use:charge:8,2");
    assert_eq!(blows_by(&without, player), 1, "{:#?}", said(&plain));
    assert_eq!(stamina(&w), stamina(&plain));
    assert_eq!(w.state.turn, plain.state.turn);
}

#[test]
fn a_foe_felled_on_the_way_does_not_stop_the_charge() {
    let mut w = world("##########\n#..p.....#\n#@......p#\n#........#\n##########\n");
    w.player_mut().talents.take("whirling_charge", None);
    let passed = at(&w, 3, 1);
    w.state.actors.get_mut(&passed).expect("the post").hp = 1;
    let outcome = use_(&mut w, "use:charge:8,2");
    assert!(outcome.took_turn, "{:#?}", said(&w));
    assert!(!w.state.actors.contains_key(&passed), "{:#?}", said(&w));
    assert_eq!(w.state.player().pos, Pos::new(7, 2), "on to the end");
}

/// Twin worlds, without and with Battle rhythm, the player out of breath beside a post of `hp`
/// hit points; each strikes it once.
fn rhythm_twins(hp: i32) -> [(World, Outcome); 2] {
    [false, true].map(|rhythm| {
        let mut w = world("######\n#@p..#\n######\n");
        if rhythm {
            w.player_mut().talents.take("battle_rhythm", None);
        }
        set_stamina(&mut w, 0);
        let post = at(&w, 2, 1);
        w.state.actors.get_mut(&post).expect("the post").hp = hp;
        let outcome = w.apply(&Action::Move(Dir::E));
        (w, outcome)
    })
}

fn hit(outcome: &Outcome) -> bool {
    (outcome.events.iter()).any(|e| {
        matches!(
            e,
            Event::Attacked {
                strike: Some(crate::combat::Strike::Hit { .. }),
                ..
            }
        )
    })
}

#[test]
fn battle_rhythm_gives_a_point_a_melee_hit_and_two_a_kill() {
    let [(plain, out), (rhythm, _)] = rhythm_twins(1000);
    assert!(hit(&out), "the fixture's first blow hits");
    assert_eq!(stamina(&rhythm), stamina(&plain) + 1, "{:#?}", said(&plain));
    let [(plain, out), (rhythm, _)] = rhythm_twins(1);
    assert!(hit(&out));
    assert_eq!(
        stamina(&rhythm),
        stamina(&plain) + 2,
        "a killing blow: two in all"
    );
    // Never past the pool's maximum.
    let mut w = world("######\n#@p..#\n######\n");
    let player = w.player_id();
    w.player_mut().talents.take("battle_rhythm", None);
    let id = w.content().resource_id(STAMINA).expect("stamina");
    let max = (w.state.player()).resource_max(w.content(), id, 1);
    set_stamina(&mut w, max - 1);
    w.gain_resource(player, crate::content::Build::MeleeKill);
    assert_eq!(stamina(&w), max);
    // A hit of another kind builds nothing.
    set_stamina(&mut w, 0);
    w.gain_resource(player, crate::content::Build::Hit);
    assert_eq!(stamina(&w), 0);
    w.gain_resource(player, crate::content::Build::MeleeHit);
    assert_eq!(stamina(&w), 1);
}

#[test]
fn a_warrior_takes_battle_rhythm_at_40_and_whirling_charge_at_50() {
    use super::class_active_tests::reach;
    let mut w = super::class_active_tests::world("warrior");
    reach(&mut w, 39);
    let holds = |w: &World, id: &str| w.state.player().talents.holds(id, None);
    assert!(!holds(&w, "battle_rhythm"));
    reach(&mut w, 40);
    assert!(holds(&w, "battle_rhythm"));
    assert!(!holds(&w, "whirling_charge"));
    reach(&mut w, 50);
    assert!(holds(&w, "whirling_charge"));
    assert!(
        w.state
            .player()
            .talent_mods(w.content())
            .martial
            .whirling_charge
    );
}

#[test]
fn a_charge_enters_each_cell_as_a_step_does() {
    // A dagger on the way, at (3, 1): the charge tells of it as a step onto it does (#1615).
    let pile = "[[pile]]\nat = [3, 1]\nobjects = [\"dagger\"]\n";
    let map = "##########\n#@.....p.#\n##########\n";
    let seen = |w: &World| said(w).iter().any(|m| m == "You see here a dagger.");
    let mut w = world_with(map, pile);
    let outcome = use_(&mut w, "use:charge:7,1");
    assert!(outcome.took_turn, "{:#?}", said(&w));
    assert_eq!(w.state.player().pos, Pos::new(6, 1));
    assert!(seen(&w), "{:#?}", said(&w));
    // Two steps onto the same cell say the same.
    let mut w = world_with(map, pile);
    w.apply(&Action::Move(Dir::E));
    assert!(!seen(&w));
    w.apply(&Action::Move(Dir::E));
    assert!(seen(&w), "{:#?}", said(&w));
}
