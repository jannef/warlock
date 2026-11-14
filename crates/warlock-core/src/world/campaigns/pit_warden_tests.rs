//! The Pits' wardens and the sealing (#1298; `pit.rs`, spec §7 "The Pits"): the warden's hall on
//! every Pit's fifth floor and nowhere above, with no way down; its death sealing its own Pit
//! alone, never the slain list; a sealed Pit's fifth floor made without it; the mouth closing on
//! the next arrival outside; the content's checks of a portal branch's boss row.

use super::stance_between;
use super::*;
use crate::action::{Action, parse_actions};
use crate::content::{Content, ContentFiles, CreatureSpawn, LevelTemplate, StairsDir, Stance};
use crate::mapgen::Visit;
use crate::test_support::{CONTENT, said};
use std::collections::BTreeSet;

const WARDEN: &str = "gatewarden";

/// Floor `depth` of Pit `s` made for a run of `seed` with `slain` counted as dead.
fn made(seed: u64, s: u32, depth: u32, slain: &BTreeSet<String>) -> LevelTemplate {
    let id = format!("hellegeat@{s}:{depth}");
    let visit = Visit {
        slain: Some(slain),
        ..Visit::default()
    };
    let w = World::new(CONTENT.clone(), "showcase", Some(seed)).expect("world");
    let seed = w.state.level_seed(&CONTENT, &id);
    travel::level_template(&CONTENT, ("showcase", seed), &id, (Some("hub"), &visit))
        .expect("made")
        .into_owned()
}

fn wardens(t: &LevelTemplate) -> usize {
    (t.creatures.iter())
        .filter(|c| CONTENT.creature(c.creature).id == WARDEN)
        .count()
}

#[test]
fn every_pits_fifth_floor_keeps_its_warden_in_its_hall_with_no_way_down() {
    let none = BTreeSet::new();
    for s in 1..=10 {
        let fifth = made(7, s, 5, &none);
        assert_eq!(wardens(&fifth), 1, "Pit {s}: its warden");
        assert!(
            !(fifth.stairs.iter()).any(|l| l.dir == StairsDir::Down),
            "Pit {s}: the last floor leads nowhere"
        );
        assert!(fifth.hidden.is_empty(), "Pit {s}: no stair hidden");
        assert!(fifth.chests.len() >= 4, "Pit {s}: the hall's chests");
    }
    for depth in 1..=4 {
        assert_eq!(wardens(&made(7, 3, depth, &none)), 0, "floor {depth}");
    }
}

#[test]
fn a_sealed_pits_fifth_floor_is_the_same_hall_without_its_warden() {
    let sealed = BTreeSet::from([WARDEN.to_string()]);
    let (open, shut) = (made(7, 4, 5, &BTreeSet::new()), made(7, 4, 5, &sealed));
    assert_eq!(wardens(&shut), 0);
    assert_eq!(open.terrain, shut.terrain, "the same floor");
    assert_eq!(open.chests.len(), shut.chests.len(), "the hoard stays");
}

fn start(seed: u64) -> World {
    World::new(CONTENT.clone(), "showcase", Some(seed)).expect("world")
}

/// The player suffers a `pit` miscast where they stand.
fn portal(w: &mut World) {
    let player = w.state.player;
    let spell = CONTENT.spell_id("spark").expect("spark");
    let me = w.state.actors[&player].clone();
    let cast = super::cast::Cast::by_creature(&me, &CONTENT, (spell, CONTENT.spell(spell)));
    let entry = (CONTENT.miscasts().iter())
        .find(|m| m.id == "pit")
        .cloned()
        .expect("the pit row");
    let aim = super::miscast::Aim::Player(None);
    w.miscast(player, &cast, aim, &entry, &mut Vec::new());
}

/// The player arrives on level `id` directly, as if by a staircase.
fn jump(w: &mut World, id: &str) {
    w.travel(id, None).expect("travel");
    w.forget_floors(&[Event::LevelChanged {
        from: String::new(),
        to: id.to_string(),
    }]);
}

/// The player, set on the current level's `<` whole and hale, climbs it.
fn climb(w: &mut World) {
    let at = (w.state.current().stairs.iter())
        .find(|s| s.dir == StairsDir::Up)
        .expect("stairs")
        .pos;
    let player = w.state.player;
    let here = w.state.current_level.clone();
    (w.state.actors).retain(|id, a| *id == player || a.level != here || a.pos != at);
    for action in parse_actions("levelup:auto").expect("actions") {
        w.apply(&action);
    }
    let content = w.content.clone();
    let me = w.state.actors.get_mut(&player).expect("player");
    me.pos = at;
    me.energy = me.energy.max(0);
    me.hp = me.max_hp(&content);
    me.effects = Default::default();
    let events = w.apply(&Action::Ascend).events;
    assert!((events.iter()).any(|e| matches!(e, Event::LevelChanged { .. })));
}

fn warden_here(w: &World) -> Option<ActorId> {
    let level = &w.state.current_level;
    (w.state.actors.values())
        .find(|a| &a.level == level && CONTENT.creature(a.creature).id == WARDEN)
        .map(|a| a.id)
}

/// The Pits' mouths in the hub, by their integers.
fn mouths(w: &World) -> Vec<u32> {
    (w.state.levels["hub"].stairs.iter())
        .filter_map(|l| pit::pit_of(&CONTENT, l.to.as_deref()?).map(|(s, _)| s))
        .collect()
}

#[test]
fn the_wardens_death_seals_its_pit_alone_and_the_mouth_closes_once_out() {
    let mut w = start(3);
    portal(&mut w);
    let player = w.state.player;
    let here = w.player_pos();
    w.state.actors.get_mut(&player).expect("player").pos = Pos::new(here.x + 4, here.y);
    portal(&mut w);
    let pits: Vec<u32> = w.state.pits.keys().copied().collect();
    let (a, b) = (pits[0], pits[1]);
    let mouth = (w.state.levels["hub"].stairs.iter())
        .find(|l| l.to.as_deref() == Some(&format!("hellegeat@{a}:1")))
        .expect("the mouth")
        .pos;

    jump(&mut w, &format!("hellegeat@{a}:5"));
    let warden = warden_here(&w).expect("the warden");
    let now = w.state.time;
    w.dies(warden, Some(player), None, now, &mut Vec::new());
    assert!(w.state.pits[&a].sealed(), "its own Pit");
    assert!(!w.state.pits[&b].sealed(), "another's warden stands");
    assert!(
        !w.state.quests.slain_set().contains(WARDEN),
        "no slain list"
    );
    assert!(said(&w).iter().any(|l| l.starts_with("The Pit shudders.")));

    // Made again, the fifth floor has no warden; another Pit's still has its own.
    jump(&mut w, &format!("hellegeat@{a}:4"));
    jump(&mut w, &format!("hellegeat@{a}:5"));
    assert!(warden_here(&w).is_none(), "sealed: no warden");
    jump(&mut w, &format!("hellegeat@{b}:5"));
    assert!(warden_here(&w).is_some(), "Pit {b} keeps its own");

    // Out by Pit a's first floor: its mouth crumbles, its record goes; Pit b's stays.
    jump(&mut w, &format!("hellegeat@{a}:1"));
    assert_eq!(mouths(&w).len(), 2, "still open while inside");
    climb(&mut w);
    assert_eq!(w.state.current_level, "hub");
    assert_eq!(mouths(&w), vec![b]);
    assert!(!w.state.pits.contains_key(&a) && w.state.pits.contains_key(&b));
    let rubble = CONTENT.terrain_id("rubble").expect("rubble");
    assert_eq!(w.state.current().terrain.get(mouth), Some(&rubble));
    assert!(said(&w).contains(&"The stair crumbles to rubble behind you.".to_string()));
}

/// The built-in content with Hellegeat's boss row and the warden's hall's map rewritten.
fn with(depth: u32, hall: Option<(&str, &str)>) -> Result<(), String> {
    let read = |path: &str| {
        std::fs::read_to_string(format!(
            "{}/../../assets/{path}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("a shipped file")
    };
    let mut files = ContentFiles::builtin();
    let branch = "data/dungeons/hellegeat.toml";
    let row = "boss = { depth = 5,";
    let text = read(branch);
    assert!(text.contains(row));
    files.insert(
        branch,
        &text.replace(row, &format!("boss = {{ depth = {depth},")),
    );
    if let Some((from, to)) = hall {
        let vault = "vaults/wardens_hall.toml";
        let text = read(vault);
        assert!(text.contains(from));
        files.insert(vault, &text.replacen(from, to, 1));
    }
    Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
}

#[test]
fn a_portal_branchs_boss_need_not_be_unique_but_its_lair_must_fit_the_floor() {
    assert!(
        !CONTENT
            .creature(CONTENT.creature_id(WARDEN).expect("warden"))
            .unique
    );
    with(5, None).expect("the shipped row loads");
    let err = with(5, Some(("$....#", "$...>#"))).expect_err("a `>` on the last floor");
    assert!(err.contains("leads nowhere"), "{err}");
    let err = with(4, None).expect_err("a floor that leads down, and no `>`");
    assert!(err.contains("marks no stairs down"), "{err}");
    let err = with(6, None).expect_err("no sixth floor");
    assert!(err.contains("does not have"), "{err}");
}

/// The stance of actor `a` toward actor `b`.
fn stance(w: &World, a: ActorId, b: ActorId) -> Stance {
    let s = &w.state;
    stance_between(s, &CONTENT, &s.actors[&a], &s.actors[&b])
}

fn summoned_by(w: &World, by: ActorId) -> Vec<ActorId> {
    (w.state.actors.values())
        .filter(|a| a.summoner == Some(by))
        .map(|a| a.id)
        .collect()
}

/// Run `seed` with a Pit open and the player on its fifth floor; the warden.
fn at_the_hall(seed: u64) -> (World, ActorId) {
    let mut w = start(seed);
    portal(&mut w);
    let s = *w.state.pits.keys().next().expect("a Pit");
    jump(&mut w, &format!("hellegeat@{s}:5"));
    let warden = warden_here(&w).expect("the warden");
    (w, warden)
}

#[test]
fn the_gates_a_warden_opens_and_what_they_let_through_fight_on_its_side() {
    let (mut w, warden) = at_the_hall(5);
    let player = w.state.player;
    w.creature_summons(warden, &mut Vec::new());
    let gates = summoned_by(&w, warden);
    assert!(!gates.is_empty(), "it opened a gate");
    for &gate in &gates {
        w.creature_summons(gate, &mut Vec::new());
        let fiends = summoned_by(&w, gate);
        assert!(!fiends.is_empty(), "the gate let fiends through");
        for fiend in fiends {
            assert_ne!(
                stance(&w, fiend, warden),
                Stance::Hostile,
                "fiend to warden"
            );
            assert_ne!(
                stance(&w, warden, fiend),
                Stance::Hostile,
                "warden to fiend"
            );
            assert_ne!(
                stance(&w, fiend, gate),
                Stance::Hostile,
                "fiend to its gate"
            );
            assert_eq!(
                stance(&w, fiend, player),
                Stance::Hostile,
                "fiend to player"
            );
        }
    }
    // None of the Pit's folk fall on the warden or its gates, nor they on them.
    let here = w.state.current_level.clone();
    let folk: Vec<ActorId> = (w.state.actors.values())
        .filter(|a| a.level == here && a.id != player && a.summoner.is_none())
        .map(|a| a.id)
        .collect();
    for &keeper in gates.iter().chain([&warden]) {
        for &other in &folk {
            assert_ne!(
                stance(&w, other, keeper),
                Stance::Hostile,
                "{other:?} to {keeper:?}"
            );
            assert_ne!(
                stance(&w, keeper, other),
                Stance::Hostile,
                "{keeper:?} to {other:?}"
            );
        }
        assert_eq!(stance(&w, keeper, player), Stance::Hostile);
    }
}

#[test]
fn a_miscasts_gate_in_a_pit_lets_loosed_fiends_through_who_fall_on_the_warden() {
    let (mut w, warden) = at_the_hall(5);
    // A rift a miscast tore open beside the warden: in the gates' faction, its own.
    let spawn = CreatureSpawn {
        creature: CONTENT.creature_id("rift").expect("rift"),
        faction: None,
        alignment: None,
        npc: None,
        pos: w.state.actors[&warden].pos,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let here = w.state.current_level.clone();
    let gate_id = travel::make_creature(&mut w.state, &CONTENT, &here, &spawn);
    w.creature_summons(gate_id, &mut Vec::new());
    let fiends = summoned_by(&w, gate_id);
    assert!(!fiends.is_empty(), "the gate let fiends through");
    let loosed = CONTENT.factions().id("loosed");
    for fiend in fiends {
        assert_eq!(w.state.actors[&fiend].faction(&CONTENT), loosed);
        assert_eq!(
            stance(&w, fiend, warden),
            Stance::Hostile,
            "fiend to warden"
        );
        assert_eq!(
            stance(&w, warden, fiend),
            Stance::Hostile,
            "warden to fiend"
        );
        assert_ne!(
            stance(&w, fiend, gate_id),
            Stance::Hostile,
            "fiend to its gate"
        );
    }
    assert_ne!(
        stance(&w, warden, gate_id),
        Stance::Hostile,
        "warden to the gate"
    );
}
