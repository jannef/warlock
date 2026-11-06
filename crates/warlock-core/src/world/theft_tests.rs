//! Theft on a cooldown and the blink to reach you (docs/design/monster-dread.md §5.6; #1130):
//! `snatch.rs` and `blink.rs`, with the bearers' rows.

use super::*;
use crate::action::{AttackKind, Positions};
use crate::character::CharacterSpec;
use crate::content::{Content, CreatureSpawn, Takes};
use crate::geom::Pos;
use crate::object::Stack;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A kitted human warrior in the showcase (seed 7), alone on the level, carrying only what they
/// wear and wield, in an open room of floor 21 by 11 cells walled round, its west end at the
/// player's cell.
fn arena() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    let me = w.player_mut();
    for i in (0..me.inventory.len()).rev() {
        if !me.equipment.holds(i) {
            me.inventory.remove(i);
            me.equipment.after_removal(&Positions::of([i]));
        }
    }
    let (floor, wall) = (
        w.content().terrain_id("floor").expect("floor"),
        w.content().terrain_id("wall").expect("wall"),
    );
    let me = w.player_pos();
    for dy in -6i32..=6 {
        for dx in -1..=21 {
            let edge = dy.abs() == 6 || dx == -1 || dx == 21;
            let p = Pos::new(me.x + dx, me.y + dy);
            let level = w.state.current_mut();
            level.terrain.set(p, if edge { wall } else { floor });
            level.traps.remove(&p);
        }
    }
    w
}

/// A creature of type `kind` at `p`, of the undead (hostile to the player).
fn put(w: &mut World, kind: &str, p: Pos) -> ActorId {
    let undead = w.content().factions().id("undead").expect("undead");
    let spawn = CreatureSpawn {
        creature: w.content().creature_id(kind).expect("a creature"),
        faction: Some(undead),
        alignment: None,
        npc: None,
        pos: p,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let (content, level) = (w.content.clone(), w.state.current_level.clone());
    travel::make_creature(&mut w.state, &content, &level, &spawn)
}

/// Gives the player `count` of object `id`, at the end of their pack; its position.
fn give(w: &mut World, id: &str, count: u32) -> usize {
    let object = w.content().object_id(id).expect("an object");
    let me = w.player_mut();
    me.inventory.push(Stack::new(object, count));
    me.inventory.len() - 1
}

fn has_status(w: &World, who: ActorId, id: &str) -> bool {
    let status = w.content().status_id(id).expect("a status");
    w.state.actors[&who].effects.has(status)
}

fn clear_status(w: &mut World, who: ActorId, id: &str) {
    let status = w.content().status_id(id).expect("a status");
    let actor = w.state.actors.get_mut(&who).expect("an actor");
    actor.effects.remove(status);
}

fn carries(w: &World, who: ActorId, id: &str) -> u32 {
    let object = w.content().object_id(id).expect("an object");
    (w.state.actors[&who].inventory.iter())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

fn last(w: &World) -> String {
    (w.messages().iter().map(|m| m.display()).next_back()).unwrap_or_default()
}

/// Lets `thief` try its luck on a hit, the cooldown cleared, until it takes something (at most
/// `tries` times); how many tries it took.
fn snatch_until(w: &mut World, thief: ActorId, tries: u32) -> Option<u32> {
    let player = w.state.player;
    let before = w.state.actors[&thief].inventory.len();
    for n in 1..=tries {
        clear_status(w, thief, "theft");
        w.snatch(thief, player, AttackKind::Melee, &mut Vec::new());
        if w.state.actors[&thief].inventory.len() > before {
            return Some(n);
        }
    }
    None
}

#[test]
fn the_bearers_steal_and_blink_as_the_design_has_them() {
    let content = &*BUILTIN;
    let row = |id: &str| content.creature(content.creature_id(id).expect(id));
    for (id, what) in [
        ("pixie", Takes::Small),
        ("nymph", Takes::Any),
        ("imp", Takes::Small),
        ("succubus", Takes::Small),
        ("incubus", Takes::Small),
    ] {
        let steals = row(id).steals.as_ref().expect("a thief");
        assert_eq!(
            (steals.what, steals.chance, steals.cooldown),
            (what, 30, 60),
            "{id}"
        );
        assert_eq!(steals.degrees, 10, "{id}: a degree per six seconds");
        assert!(row(id).blinks.is_none(), "{id} never blinks");
    }
    let dog = row("blink_dog").blinks.as_ref().expect("the blinker");
    assert_eq!((dog.range, dog.cooldown, dog.degrees), (6, 30, 5));
    assert!(row("blink_dog").steals.is_none());
    // The blink dog is the one blinker; the five the only thieves.
    let blinkers: Vec<&str> = (content.creature_defs().iter())
        .filter(|c| c.blinks.is_some())
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(blinkers, ["blink_dog"]);
    let thieves = content
        .creature_defs()
        .iter()
        .filter(|c| c.steals.is_some())
        .count();
    assert_eq!(thieves, 5);
    // The pixie's touch sends to sleep now; the nymph's still transfixes.
    let touch = |id: &str| {
        let attack = row(id).attack.as_ref().expect("a touch");
        attack
            .on_hit
            .iter()
            .map(|o| format!("{:?}", o.what))
            .collect::<Vec<_>>()
    };
    let drowsy = content.status_id("drowsy").expect("drowsy");
    let transfixed = content.status_id("transfixed").expect("transfixed");
    assert!(touch("pixie")[0].contains(&format!("{drowsy:?}")));
    assert!(touch("nymph")[0].contains(&format!("{transfixed:?}")));
    assert!(row("pixie").dread && row("nymph").dread && row("blink_dog").dread);
    assert!(row("succubus").dread && row("incubus").dread && !row("imp").dread);
}

#[test]
fn a_thief_s_hit_takes_one_small_stack_by_its_chance_then_rests_on_its_cooldown() {
    let mut w = arena();
    let me = w.player_pos();
    let pixie = put(&mut w, "pixie", me.step(Dir::E));
    w.perceive(&mut Vec::new());
    give(&mut w, "bread", 3);
    give(&mut w, "gold_coin", 40);
    // Some tries, but never many: the chance is 30 %.
    let tries = snatch_until(&mut w, pixie, 60).expect("it steals in the end");
    assert!(tries < 60);
    assert_eq!(
        carries(&w, pixie, "gold_coin"),
        40,
        "the coins, all of them"
    );
    assert_eq!(carries(&w, w.state.player, "gold_coin"), 0);
    assert_eq!(carries(&w, pixie, "bread"), 0, "bread is not small");
    assert_eq!(last(&w), "The pixie snatches 40 gold coins from you!");
    // On its cooldown, its hits take nothing.
    assert!(has_status(&w, pixie, "theft"));
    give(&mut w, "gold_coin", 5);
    let player = w.state.player;
    for _ in 0..50 {
        w.snatch(pixie, player, AttackKind::Melee, &mut Vec::new());
    }
    assert_eq!(carries(&w, player, "gold_coin"), 5);
    // A shot takes nothing either: theft is up close.
    clear_status(&mut w, pixie, "theft");
    for _ in 0..50 {
        w.snatch(pixie, player, AttackKind::Ranged, &mut Vec::new());
    }
    assert_eq!(carries(&w, player, "gold_coin"), 5);
}

#[test]
fn the_chance_is_about_three_in_ten() {
    let mut w = arena();
    let me = w.player_pos();
    let nymph = put(&mut w, "nymph", me.step(Dir::E));
    w.perceive(&mut Vec::new());
    let player = w.state.player;
    let at = give(&mut w, "bread", 1);
    let bread = w.state.player().inventory[at].clone();
    let mut took = 0;
    for _ in 0..1000 {
        clear_status(&mut w, nymph, "theft");
        w.snatch(nymph, player, AttackKind::Melee, &mut Vec::new());
        let robbed = w.state.actors.get_mut(&nymph).expect("the nymph");
        if !robbed.inventory.is_empty() {
            took += 1;
            robbed.inventory.clear();
            w.player_mut().inventory.push(bread.clone());
        }
    }
    assert!((240..=360).contains(&took), "{took} of 1000");
}

#[test]
fn never_what_is_worn_or_wielded() {
    let mut w = arena();
    let me = w.player_pos();
    let nymph = put(&mut w, "nymph", me.step(Dir::E));
    w.perceive(&mut Vec::new());
    let player = w.state.player;
    // Only what the warrior wears and wields: nothing to take, and no cooldown.
    assert!((0..w.state.player().inventory.len()).any(|i| w.state.player().equipment.holds(i)));
    let count = w.state.player().inventory.len();
    for _ in 0..200 {
        w.snatch(nymph, player, AttackKind::Melee, &mut Vec::new());
    }
    assert_eq!(w.state.player().inventory.len(), count);
    assert!(
        !has_status(&w, nymph, "theft"),
        "nothing taken, no cooldown"
    );
    // With a loaf among them, the loaf goes, and the kit stays on.
    give(&mut w, "bread", 1);
    snatch_until(&mut w, nymph, 100).expect("the loaf");
    assert_eq!(carries(&w, nymph, "bread"), 1);
    assert_eq!(w.state.player().inventory.len(), count);
    assert!((0..count).all(|i| w.state.player().equipment.holds(i)));
}

#[test]
fn what_it_took_falls_where_it_dies() {
    let mut w = arena();
    let me = w.player_pos();
    let at = me.step(Dir::E);
    let pixie = put(&mut w, "pixie", at);
    w.perceive(&mut Vec::new());
    give(&mut w, "gold_coin", 12);
    snatch_until(&mut w, pixie, 100).expect("the coins");
    let now = w.state.time;
    w.dies(pixie, Some(w.state.player), None, now, &mut Vec::new());
    let gold = w.content().object_id("gold_coin").expect("gold");
    let pile = w.state.current().objects_at(at);
    assert_eq!(
        (pile.iter().filter(|s| s.object == gold))
            .map(|s| s.count)
            .sum::<u32>(),
        12
    );
}

#[test]
fn it_never_steals_from_a_creature() {
    let mut w = arena();
    let me = w.player_pos();
    let nymph = put(&mut w, "nymph", Pos::new(me.x + 5, me.y));
    let victim = put(&mut w, "kobold", Pos::new(me.x + 6, me.y));
    let bread = w.content().object_id("bread").expect("bread");
    w.state.actors.get_mut(&victim).expect("kobold").inventory = vec![Stack::new(bread, 2)];
    for _ in 0..200 {
        clear_status(&mut w, nymph, "theft");
        w.snatch(nymph, victim, AttackKind::Melee, &mut Vec::new());
    }
    assert_eq!(carries(&w, victim, "bread"), 2);
    assert!(w.state.actors[&nymph].inventory.is_empty());
}

/// A blink dog at `dog`, ringed by `ring` kobolds of its side on the cells around it.
fn hemmed(w: &mut World, dog: Pos, ring: bool) -> ActorId {
    let id = put(w, "blink_dog", dog);
    if ring {
        for d in Dir::ALL {
            put(w, "kobold", dog.step(d));
        }
    }
    id
}

#[test]
fn a_blink_dog_hemmed_in_blinks_beside_its_target_then_walks_on_its_cooldown() {
    let mut w = arena();
    let me = w.player_pos();
    let player = w.state.player;
    let dog = hemmed(&mut w, Pos::new(me.x + 4, me.y), true);
    w.perceive(&mut Vec::new());
    assert_eq!(w.fight(dog, player, me), ai::Plan::Blink(me));
    let mut events = Vec::new();
    w.creature_blinks(dog, me, &mut events);
    let at = w.state.actors[&dog].pos;
    assert_eq!(at.chebyshev(me), 1, "beside the player");
    assert!(at.chebyshev(Pos::new(me.x + 4, me.y)) <= 6);
    assert!(has_status(&w, dog, "blink"));
    assert_eq!(last(&w), "The blink dog is suddenly beside you.");
    // Back in its ring, on its cooldown, it waits as any other would.
    w.state.actors.get_mut(&dog).expect("dog").pos = Pos::new(me.x + 4, me.y);
    assert_eq!(w.fight(dog, player, me), ai::Plan::Wait);
    // Its cooldown eased, it blinks again.
    clear_status(&mut w, dog, "blink");
    assert_eq!(w.fight(dog, player, me), ai::Plan::Blink(me));
}

#[test]
fn an_open_way_is_walked_and_a_blink_needs_range_and_sight() {
    let mut w = arena();
    let me = w.player_pos();
    let player = w.state.player;
    // Unhemmed, it runs.
    let dog = hemmed(&mut w, Pos::new(me.x + 4, me.y), false);
    assert!(matches!(w.fight(dog, player, me), ai::Plan::Step(_)));
    w.state.actors.retain(|&id, _| id == player);
    // Too far: no cell beside the player lies within six of it.
    let far = hemmed(&mut w, Pos::new(me.x + 8, me.y), true);
    assert!(w.blink_landings(far, me).is_empty());
    assert_eq!(w.fight(far, player, me), ai::Plan::Wait);
    w.state.actors.retain(|&id, _| id == player);
    // Out of its sight: a wall across the room (a door it cannot open, for all it knows)
    // hides every cell beside the player.
    let wall = w.content().terrain_id("wall").expect("wall");
    for dy in -5..=5 {
        w.state
            .current_mut()
            .terrain
            .set(Pos::new(me.x + 2, me.y + dy), wall);
    }
    let shut = put(&mut w, "blink_dog", Pos::new(me.x + 4, me.y));
    assert!(w.blink_landings(shut, me).is_empty());
    assert_eq!(w.fight(shut, player, me), ai::Plan::Wait);
}

#[test]
fn an_unseen_blink_says_nothing_and_the_blink_is_deterministic() {
    let landing = |w: &mut World| {
        let me = w.player_pos();
        let dog = hemmed(w, Pos::new(me.x + 4, me.y), true);
        w.creature_blinks(dog, me, &mut Vec::new());
        w.state.actors[&dog].pos
    };
    let (mut a, mut b) = (arena(), arena());
    assert_eq!(landing(&mut a), landing(&mut b));
    // Beside a kobold behind a wall the player cannot see through, nothing is said.
    let mut w = arena();
    let me = w.player_pos();
    let wall = w.content().terrain_id("wall").expect("wall");
    for dy in -5..=5 {
        w.state
            .current_mut()
            .terrain
            .set(Pos::new(me.x + 2, me.y + dy), wall);
    }
    let dog = put(&mut w, "blink_dog", Pos::new(me.x + 8, me.y));
    let prey = Pos::new(me.x + 12, me.y);
    put(&mut w, "kobold", prey);
    w.perceive(&mut Vec::new());
    let before = w.messages().len();
    w.creature_blinks(dog, prey, &mut Vec::new());
    assert_eq!(w.state.actors[&dog].pos.chebyshev(prey), 1);
    assert_eq!(w.messages().len(), before);
}
