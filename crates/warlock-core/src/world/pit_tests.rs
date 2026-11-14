//! The `portal` miscast and the Pits' records (#1297; `pit.rs`, spec §7 "The Pits"): a Pit opened
//! with its gate, refused on the overworld and a Pit's floor, taken in a town and on an endless
//! floor; two Pits apart; the first floor's `<` to the mouth after a climb; the orphan rule; the
//! dev scenario.

use super::caster_tests::said;
use super::miscast::Aim;
use super::*;
use crate::action::{Action, parse_actions};
use crate::content::{Content, Miscast};
use std::sync::{Arc, LazyLock};

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn start(scenario: &str, seed: u64) -> World {
    World::new(CONTENT.clone(), scenario, Some(seed)).expect("world")
}

/// The built-in entry `id`.
fn entry(id: &str) -> Miscast {
    (CONTENT.miscasts().iter())
        .find(|m| m.id == id)
        .cloned()
        .expect("a built-in entry")
}

/// Miscast `id` befalls actor `caster` after a Spark.
fn befalls(w: &mut World, caster: ActorId, id: &str) {
    let spell = CONTENT.spell_id("spark").expect("spark");
    let me = w.state.actors[&caster].clone();
    let cast = super::cast::Cast::by_creature(&me, &CONTENT, (spell, CONTENT.spell(spell)));
    let aim = if caster == w.state.player {
        Aim::Player(None)
    } else {
        Aim::Creature(w.state.player)
    };
    w.miscast(caster, &cast, aim, &entry(id), &mut Vec::new());
}

/// The player suffers miscast `id`.
fn portal(w: &mut World, id: &str) {
    let player = w.state.player;
    befalls(w, player, id);
}

/// The Pits' mouths on the current level: (cell, Pit).
fn mouths(w: &World) -> Vec<(Pos, u32)> {
    (w.state.current().stairs.iter())
        .filter_map(|s| {
            let (pit, depth) = pit::pit_of(&CONTENT, s.to.as_deref()?)?;
            (s.dir == StairsDir::Down && depth == 1).then_some((s.pos, pit))
        })
        .collect()
}

/// The creatures of type `id` on the current level, where they stand.
fn standing(w: &World, id: &str) -> Vec<Pos> {
    let level = &w.state.current_level;
    (w.state.actors.values())
        .filter(|a| &a.level == level && CONTENT.creature(a.creature).id == id)
        .map(|a| a.pos)
        .collect()
}

/// The player, set on the staircase `dir` whole and hale (on `at` if given), takes it.
fn take(w: &mut World, dir: StairsDir, at: Option<Pos>) {
    let at = at.unwrap_or_else(|| {
        (w.state.current().stairs.iter())
            .find(|s| s.dir == dir)
            .expect("stairs")
            .pos
    });
    let player = w.state.player;
    let here = w.state.current_level.clone();
    (w.state.actors).retain(|id, a| *id == player || a.level != here || a.pos != at);
    for action in parse_actions("levelup:auto").expect("actions") {
        w.apply(&action);
    }
    let me = w.state.actors.get_mut(&player).expect("player");
    me.pos = at;
    me.energy = me.energy.max(0);
    me.hp = me.max_hp(&CONTENT);
    me.effects = Default::default();
    let action = match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    };
    let events = w.apply(&action).events;
    assert!((events.iter()).any(|e| matches!(e, Event::LevelChanged { .. })));
}

#[test]
fn a_portal_opens_a_pit_with_its_gate_at_the_mouth() {
    let mut w = start("showcase", 1);
    let here = w.player_pos();
    portal(&mut w, "pit");
    let opened = mouths(&w);
    assert_eq!(opened.len(), 1, "one mouth");
    let (at, s) = opened[0];
    assert!(at.chebyshev(here) <= w.content().tuning().pit.mouth_reach);
    let stairs = w.state.current().terrain.get(at).copied().expect("cell");
    assert_eq!(CONTENT.terrain(stairs).stairs, Some(StairsDir::Down));
    assert_eq!(
        w.state.pits.get(&s).map(|p| (p.from(), p.sealed())),
        Some(("hub", false))
    );
    let gates = standing(&w, "rift");
    assert_eq!(gates.len(), 1, "the rift is at the mouth");
    assert!(gates[0] != at && gates[0].chebyshev(here) == 1);
    assert!(said(&w).contains(&entry("pit").message));
    // The four pit rows open Pits with their own gates.
    for (id, gate) in [
        ("pit_yawning", "abyssal_gate"),
        ("pit_roaring", "dire_gate"),
        ("pit_and_lords", "lords_gate"),
    ] {
        let mut w = start("showcase", 1);
        portal(&mut w, id);
        assert_eq!(mouths(&w).len(), 1, "{id}");
        assert_eq!(standing(&w, gate).len(), 1, "{id}");
    }
}

#[test]
fn the_overworld_and_a_pits_floor_refuse_it_and_the_gate_comes_alone() {
    for (scenario, line) in [
        ("world", "The ground heaves but will not open"),
        ("hellegeat", "The floor here is the Pit's already"),
    ] {
        let mut w = start(scenario, 1);
        let pits = w.state.pits.clone();
        portal(&mut w, "pit");
        assert!(mouths(&w).is_empty(), "{scenario}");
        assert_eq!(w.state.pits, pits, "{scenario}: no record");
        assert_eq!(standing(&w, "rift").len(), 1, "{scenario}: the gate alone");
        let said = said(&w);
        assert!(
            said.iter().any(|l| l.starts_with(line)),
            "{scenario}: {said:?}"
        );
        assert!(!said.contains(&entry("pit").message));
    }
}

#[test]
fn a_town_and_an_endless_floor_take_it() {
    for scenario in ["ceaptun", "grundleas"] {
        let mut w = start(scenario, 2);
        portal(&mut w, "pit");
        assert_eq!(mouths(&w).len(), 1, "{scenario}");
        assert_eq!(w.state.pits.len(), 1, "{scenario}");
    }
}

#[test]
fn a_creatures_portal_opens_a_pit_too() {
    let mut w = start("showcase", 1);
    let here = w.player_pos();
    let kobold = CONTENT.creature_id("kobold").expect("kobold");
    let spawn = crate::content::CreatureSpawn {
        creature: kobold,
        faction: None,
        alignment: None,
        npc: None,
        pos: Pos::new(here.x, here.y),
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let level = w.state.current_level.clone();
    let k = travel::make_creature(&mut w.state, &CONTENT, &level, &spawn);
    w.state.actors.get_mut(&k).expect("kobold").pos = here;
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = Pos::new(-50, -50);
    befalls(&mut w, k, "pit");
    assert_eq!(mouths(&w).len(), 1);
    assert_eq!(w.state.pits.len(), 1);
}

#[test]
fn two_portals_make_two_pits_and_their_floors_differ() {
    let mut w = start("showcase", 3);
    portal(&mut w, "pit");
    let player = w.state.player;
    let here = w.player_pos();
    w.state.actors.get_mut(&player).expect("player").pos = Pos::new(here.x + 4, here.y);
    portal(&mut w, "pit");
    let pits: Vec<u32> = w.state.pits.keys().copied().collect();
    assert_eq!(pits.len(), 2);
    assert_eq!(mouths(&w).len(), 2);
    let floor = |s: u32| {
        let id = format!("hellegeat@{s}:1");
        let seed = w.state.level_seed(&CONTENT, &id);
        let visit = crate::mapgen::Visit::default();
        travel::level_template(&CONTENT, ("showcase", seed), &id, (Some("hub"), &visit))
            .expect("made")
            .terrain
            .clone()
    };
    assert_ne!(floor(pits[0]), floor(pits[1]), "two Pits, two floors");
    assert_eq!(floor(pits[0]), floor(pits[0]), "one Pit, one floor");
}

#[test]
fn the_first_floors_way_up_leads_to_the_mouth_after_a_climb() {
    let mut w = start("showcase", 4);
    portal(&mut w, "pit");
    let (at, s) = mouths(&w)[0];
    take(&mut w, StairsDir::Down, Some(at));
    let first = format!("hellegeat@{s}:1");
    assert_eq!(w.state.current_level, first);
    let up = |w: &World| {
        (w.state.current().stairs.iter())
            .find(|l| l.dir == StairsDir::Up)
            .and_then(|l| l.to.clone())
    };
    assert_eq!(up(&w).as_deref(), Some("hub"));
    take(&mut w, StairsDir::Down, None);
    take(&mut w, StairsDir::Up, None);
    assert_eq!(w.state.current_level, first);
    assert_eq!(
        up(&w).as_deref(),
        Some("hub"),
        "not back down to the second"
    );
    take(&mut w, StairsDir::Up, None);
    assert_eq!(w.state.current_level, "hub");
    assert_eq!(w.player_pos(), at, "out onto the mouth");
    assert!(w.state.pits.contains_key(&s), "its mouth stands: kept");
}

#[test]
fn a_pit_opened_on_an_endless_floor_is_dropped_once_out() {
    let mut w = start("grundleas", 5);
    portal(&mut w, "pit");
    let (at, s) = mouths(&w)[0];
    take(&mut w, StairsDir::Down, Some(at));
    assert!(
        !w.state.levels.contains_key("grundleas:1"),
        "the mouth is gone"
    );
    assert!(w.state.pits.contains_key(&s), "kept while inside");
    take(&mut w, StairsDir::Up, None);
    assert_eq!(w.state.current_level, "grundleas:1");
    assert!(mouths(&w).is_empty(), "made anew, with no way back");
    assert!(w.state.pits.is_empty(), "the orphan is dropped");
}

#[test]
fn the_dev_scenario_starts_in_pit_one_and_climbs_out_to_the_hub() {
    let mut w = start("hellegeat", 1);
    assert_eq!(w.state.current_level, "hellegeat@1:1");
    assert_eq!(
        w.state.pits.get(&1).map(|p| p.from().to_string()),
        Some("hub".to_string())
    );
    take(&mut w, StairsDir::Up, None);
    assert_eq!(w.state.current_level, "hub");
    assert!(w.state.pits.is_empty(), "no mouth in the hub: dropped");
}

#[test]
fn a_run_with_no_pit_saves_none_and_a_record_round_trips() {
    let mut w = start("showcase", 1);
    let json = serde_json::to_string(&w.state).expect("json");
    assert!(!json.contains("\"pits\""), "left out while empty");
    portal(&mut w, "pit");
    let json = serde_json::to_string(&w.state).expect("json");
    let back: WorldState = serde_json::from_str(&json).expect("back");
    assert_eq!(back.pits, w.state.pits);
}
