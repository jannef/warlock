//! Creatures' spells (spec §8 "Creature behavior": "Casters", `creature_cast.rs`): a caster
//! casts the spells of its list through the player's spell code while it has the mana and they
//! go, else fights; low-level casters are rare and know only the weakest spells.

use crate::action::{AttackKind, Event, parse_actions};
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::test_support::run;
use crate::world::*;

/// The built-in content with `map`: the player at `@`, a caster of type `caster` at `k`, a
/// creature of type `friend` at `f` (of the caster's faction), each hostile by its type.
fn world(map: &str, caster: &str, friend: &str) -> World {
    let mut w = fresh(map, caster, friend);
    // Gear rolled with it (an amulet of mana, say) would change its pool.
    for a in w
        .state
        .actors
        .values_mut()
        .filter(|a| a.id != w.state.player)
    {
        a.inventory.clear();
        a.equipment = Default::default();
    }
    w
}

/// [`world`] as it starts, gear and all.
fn fresh(map: &str, caster: &str, friend: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_room.toml",
        &format!(
            "name = \"Test\"\nambient = [200, 200, 200, 0]\nmap = '''\n{map}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"{caster}\" }}\n\
             \"f\" = {{ terrain = \"floor\", creature = \"{friend}\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\nhp = 400\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_room", None).expect("world")
}

/// A hall, the player four cells from the caster.
const HALL: &str = "\
############
#..........#
#..@...k...#
#..........#
############
";

fn of(w: &World, kind: &str) -> ActorId {
    let kind = w.content.creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .expect("there")
        .id
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text.contains(text))
}

fn status(w: &World, id: &str) -> crate::content::StatusId {
    w.content.status_id(id).expect("a status")
}

/// The attacks in `outs` by `by`: their kind.
fn attacks_by(outs: &[Outcome], by: ActorId) -> Vec<AttackKind> {
    (outs.iter().flat_map(|o| &o.events))
        .filter_map(|e| match *e {
            Event::Attacked { attacker, kind, .. } if attacker == by => Some(kind),
            _ => None,
        })
        .collect()
}

#[test]
fn a_shaman_casts_spark_through_the_spell_code_and_pays_mana() {
    let mut w = world(HALL, "kobold_shaman", "kobold");
    let shaman = of(&w, "kobold_shaman");
    let mana = w.state.actors[&shaman].mana;
    let hp = w.state.player().hp;
    let outs = run(&mut w, "wait");
    assert_eq!(attacks_by(&outs, shaman), [AttackKind::Spell]);
    assert!(said(&w, "The kobold shaman casts spark at you!"));
    // Its blow is said as a spell's, and hurts: spark never misses.
    assert!(said(&w, "The spark"), "{:?}", w.messages());
    assert!(w.state.player().hp < hp);
    let spark = w.content.spell_id("spark").expect("spark");
    let cost = w.state.actors[&shaman].spell_cost(&w.content, w.content.spell(spark));
    assert_eq!(w.state.actors[&shaman].mana, mana - cost);
    // It stood where it was: in reach, it casts rather than steps.
    assert_eq!(w.state.actors[&shaman].pos, Pos::new(7, 2));
}

#[test]
fn out_of_mana_a_caster_closes_in_and_fights_never_idle() {
    let mut w = world(HALL, "kobold_shaman", "kobold");
    let shaman = of(&w, "kobold_shaman");
    w.state.actors.get_mut(&shaman).expect("there").mana = 0;
    let outs = run(&mut w, "wait*8");
    let pos = w.state.actors[&shaman].pos;
    assert_eq!(pos.chebyshev(w.player_pos()), 1, "it closed in: {pos:?}");
    let kinds = attacks_by(&outs, shaman);
    assert!(kinds.contains(&AttackKind::Melee), "{kinds:?}");
    assert!(kinds.iter().all(|&k| k == AttackKind::Melee), "{kinds:?}");
}

#[test]
fn a_hex_falls_once_then_the_harm_follows() {
    let mut w = world(HALL, "barrow_wight", "kobold");
    let wight = of(&w, "barrow_wight");
    let weakened = status(&w, "weakened");
    run(&mut w, "wait");
    assert!(said(&w, "The barrow wight casts hex of weakness at you!"));
    assert!(w.state.player().effects.has(weakened));
    // Hexed already, it closes in for its grave breath (a cone of 3) instead.
    let outs = run(&mut w, "wait*3");
    assert!(said(&w, "casts grave breath at you!"), "{:?}", w.messages());
    assert!(attacks_by(&outs, wight).contains(&AttackKind::Spell));
}

#[test]
fn a_buff_first_then_the_heaviest_spell_that_goes() {
    let mut w = world(HALL, "lich", "kobold");
    let lich = of(&w, "lich");
    let hasted = status(&w, "hasted");
    run(&mut w, "wait");
    assert!(said(&w, "The lich casts haste."));
    assert!(w.state.actors[&lich].effects.has(hasted));
    run(&mut w, "wait");
    assert!(said(&w, "The lich casts chain lightning at you!"));
}

#[test]
fn a_healer_mends_a_hurt_friend_beside_it() {
    let map = "\
############
#..........#
#..@...fk..#
#..........#
############
";
    let mut w = world(map, "kobold_shaman", "kobold");
    let kobold = of(&w, "kobold");
    let max = w.state.actors[&kobold].max_hp(&w.content);
    let k = w.state.actors.get_mut(&kobold).expect("there");
    k.hp = 1;
    // It stays beside the shaman.
    k.energy = i64::MIN / 4;
    run(&mut w, "wait");
    assert!(said(&w, "The kobold shaman casts mend on the kobold."));
    let hp = w.state.actors[&kobold].hp;
    assert!(hp > 1 && hp <= max, "{hp}");
}

#[test]
fn an_area_spell_spares_the_casters_friends() {
    // The goblin wizard's burning hands would catch the goblin beside the player: it sparks.
    let map = "\
##########
#........#
#.@fk....#
#........#
##########
";
    let mut w = world(map, "goblin_wizard", "goblin");
    // The goblin stands in the way of a bolt's clear line; it moves first, so hold it.
    let goblin = of(&w, "goblin");
    w.state.actors.get_mut(&goblin).expect("there").energy = i64::MIN / 4;
    let wizard = of(&w, "goblin_wizard");
    w.state.actors.get_mut(&wizard).expect("there").pos = Pos::new(3, 1);
    let burning = w.content.spell_id("burning_hands").expect("a spell");
    let at = w.player_pos();
    let cast = w.creature_spell(wizard, w.state.player, at);
    assert!(cast.is_some_and(|(s, _)| s != burning), "{cast:?}");
    // Alone, it burns.
    w.state.actors.remove(&goblin);
    let cast = w.creature_spell(wizard, w.state.player, at);
    assert_eq!(cast.map(|(s, _)| s), Some(burning));
}

#[test]
fn silenced_it_casts_nothing() {
    let mut w = world(HALL, "kobold_shaman", "kobold");
    let shaman = of(&w, "kobold_shaman");
    let silenced = status(&w, "silenced");
    let mut events = Vec::new();
    w.afflict(shaman, silenced, 50, None, &mut events);
    let at = w.player_pos();
    assert_eq!(w.creature_spell(shaman, w.state.player, at), None);
    assert_eq!(w.spell_reach_at(shaman, w.state.player), None);
    let outs = run(&mut w, "wait");
    assert!(!attacks_by(&outs, shaman).contains(&AttackKind::Spell));
}

#[test]
fn a_fight_with_casters_saves_and_replays() {
    let mut w = fresh(HALL, "dark_wizard", "kobold");
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait*6").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(said(&w, "The dark wizard casts"));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content.clone());
    // The fixture's level is not a shipped scenario's: verify against the same content.
    let verified = verified.expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let mut loaded = crate::save::decode(&bytes, w.content.clone()).expect("loads");
    run(&mut w, "wait*3");
    run(&mut loaded, "wait*3");
    assert_eq!(loaded.state_hash(), w.state_hash());
}
