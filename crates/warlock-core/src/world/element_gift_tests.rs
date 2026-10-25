//! The element gifts (spec §8 "Birth sign"; #268): Fire's Blaze (an active in the abilities
//! menu's Birth sign group, with its cooldown) and Kindled (a brighter lantern), Metal's
//! Forge-born (marks half again as fast), and Water's Tidal (mana a quarter faster).

use super::*;
use crate::ability::{Ability, Source, Unavailable};
use crate::action::{SpellShape, parse_actions};
use crate::character::CharacterSpec;
use crate::content::{Role, Sign, SignActive};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

const BLAZE: Action = Action::Gift(SignActive::Blaze, None);

/// A human warrior in `scenario`, born under the `element` Salamander (the run restored with
/// that sign, so the gifts are granted as a load grants them).
fn born_in(scenario: &str, element: &str) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w = World::with_character(BUILTIN.clone(), scenario, Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = Some(Sign {
        animal: "salamander".into(),
        element: element.into(),
    });
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

fn born(element: &str) -> World {
    born_in("showcase", element)
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn last(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn blaze_row(world: &World) -> Option<Option<Unavailable>> {
    (world.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Sign(SignActive::Blaze))
        .map(|a| a.unavailable)
}

/// A kobold with `hp` hit points placed `dir` of the player; its id.
fn beside(world: &mut World, dir: Dir, hp: i32) -> ActorId {
    let content = world.content().clone();
    let at = world.player_pos().step(dir);
    let template = (world.state.actors.values())
        .find(|a| a.id != world.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(world.state.next_actor_id);
    world.state.next_actor_id += 1;
    let creature = content.creature_id("kobold").expect("kobolds");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: hp,
        hp,
        level: world.state.current_level.clone(),
        ..template
    };
    world.state.actors.insert(id, actor);
    id
}

#[test]
fn each_element_grants_its_two_gifts() {
    let gifts = |element: &str| -> Vec<String> {
        let w = born(element);
        let c = w.character().expect("a character");
        let all: Vec<String> = c
            .grants(w.content())
            .into_iter()
            .map(String::from)
            .collect();
        all[all.len() - 2..].to_vec()
    };
    // The Salamander's own come first; then the element's two.
    assert_eq!(gifts("fire"), ["blaze", "kindled"]);
    assert_eq!(gifts("metal"), ["keen_edge", "forge_born"]);
    assert_eq!(gifts("water"), ["always_a_way", "tidal"]);
}

#[test]
fn blaze_burns_everyone_beside_but_the_player_then_burns_them_out() {
    let mut w = born("fire");
    let player = w.state.player;
    assert_eq!(blaze_row(&w), Some(None), "in the menu, usable");
    let east = beside(&mut w, Dir::E, 500);
    let south = beside(&mut w, Dir::S, 500);
    let far = beside(&mut w, Dir::W, 500);
    let two_west = w.player_pos().step(Dir::W).step(Dir::W);
    w.state.actors.get_mut(&far).expect("far").pos = two_west;
    let here = w.player_pos();
    let turn = w.state.turn;
    let out = w.apply(&BLAZE);
    assert!(w.state.turn > turn, "it takes time");
    let spell = out.events.iter().any(|e| {
        matches!(e, Event::Spell { effect, shape: SpellShape::Burst { center, radius: 1 }, .. }
            if effect == "blaze" && *center == here)
    });
    assert!(spell, "{:?}", out.events);
    assert!(said(&w).contains(&"Flames burst from you!".to_string()));
    // One roll for everyone beside, nothing for the player or anyone farther.
    let struck: Vec<ActorId> = (out.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker, target, ..
            } if *attacker == player => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(struck, [east, south]);
    let lost = |id: ActorId| 500 - w.state.actors[&id].hp;
    assert!((1..=12).contains(&lost(east)), "{}", lost(east));
    assert_eq!(lost(east), lost(south));
    assert_eq!(lost(far), 0);
    assert!(said(&w).contains(&"The blaze burns the kobold. (×2)".to_string()));
    // Fires where it reached, but not underfoot.
    let fires = &w.state.current().fires;
    assert!(fires.contains_key(&here.step(Dir::E)) && !fires.contains_key(&here));
    let fire = w.content().light_id("arrow_fire").expect("a light");
    assert!(fires.values().all(|f| f.light == fire));
    // Then burnt out: greyed in the menu, refused with no time.
    let burnt = w.content().status_id("burnt_out").expect("burnt out");
    assert!(w.state.player().effects.has(burnt));
    assert_eq!(blaze_row(&w), Some(Some(Unavailable::While(burnt))));
    let turn = w.state.turn;
    w.apply(&BLAZE);
    assert_eq!(w.state.turn, turn, "refused");
    assert_eq!(last(&w), "You are still burnt out.");
    // Half a day later the fire kindles again (the kobolds gone, so nothing wakes the player).
    w.state
        .actors
        .retain(|id, _| ![east, south, far].contains(id));
    w.perceive(&mut Vec::new());
    run(&mut w, "rest:12h");
    assert!(!w.state.player().effects.has(burnt), "{:?}", said(&w));
    assert_eq!(blaze_row(&w), Some(None));
}

#[test]
fn blaze_is_the_fire_borns_alone_and_not_on_the_overworld() {
    let mut w = born("wood");
    assert_eq!(blaze_row(&w), None, "not in the menu");
    let turn = w.state.turn;
    w.apply(&BLAZE);
    assert_eq!(
        (w.state.turn, last(&w)),
        (turn, "You have no such gift.".into())
    );
    let mut w = born_in("world", "fire");
    let turn = w.state.turn;
    w.apply(&BLAZE);
    assert_eq!(w.state.turn, turn);
    assert_eq!(last(&w), "You cannot do that on the overworld.");
}

#[test]
fn blaze_is_listed_under_the_birth_sign_and_dumped() {
    let w = born("fire");
    let rows = dump::layer_rows(&w, Layer::Abilities);
    assert_eq!(
        rows.first().map(String::as_str),
        Some("Blaze (sign): nothing"),
        "{rows:?}"
    );
    assert_eq!(Ability::Sign(SignActive::Blaze).source(), Source::Sign);
    assert_eq!(BLAZE.to_string(), "use:blaze");
    assert_eq!("use:blaze".parse::<Action>(), Ok(BLAZE));
}

#[test]
fn kindled_lanterns_burn_brighter() {
    for (element, light) in [("fire", "lantern_bright"), ("wood", "lantern")] {
        let mut w = born(element);
        run(&mut w, "lantern");
        let want = w.content().light_id(light);
        assert_eq!(w.state().lantern_light(w.content()), want, "{element}");
    }
}

#[test]
fn the_forge_born_earn_marks_half_again_as_fast_in_every_skill() {
    // Fifty more in a hundred, on top of the warrior's Weapon drill (melee: a quarter more).
    let mods = |w: &World| w.state.player().talent_mods(w.content());
    let (metal, wood) = (born("metal"), born("wood"));
    for role in [
        Role::Melee,
        Role::Launcher,
        Role::Thrown,
        Role::ShieldBlock,
        Role::Style,
    ] {
        let drill = if role == Role::Melee { 25 } else { 0 };
        assert_eq!(mods(&metal).marks(role, 100), 150 + drill, "{role:?}");
        assert_eq!(mods(&wood).marks(role, 100), 100 + drill, "{role:?}");
    }
}

#[test]
fn the_tidal_get_mana_back_a_quarter_faster() {
    let water = born("water");
    assert_eq!(water.mana_rate(water.state.player), 125);
    let wood = born("wood");
    assert_eq!(wood.mana_rate(wood.state.player), 100);
}

/// The first seed from 1 on which `spec` is Fire-born at the showcase's start.
fn fire_seed(spec: &CharacterSpec) -> u64 {
    let start = BUILTIN.scenario("showcase").expect("showcase").start;
    (1u64..10_000)
        .find(|&seed| {
            let s = spec.start(&BUILTIN, seed, start).expect("starts");
            s.character.born_of_element("fire")
        })
        .expect("a Fire-born seed")
}

#[test]
fn a_blaze_replays_and_saves() {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let seed = fire_seed(&spec);
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(seed), Some(&spec)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("use:blaze use:blaze wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let burnt = w.content().status_id("burnt_out").expect("burnt out");
    assert!(w.state.player().effects.has(burnt));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let loaded = crate::save::decode(&crate::save::encode(&w), BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
}
