//! Sic 'em (`sic.rs`; spec §8 "Allies"): the ranger's level 40, the hunt that follows its quarry,
//! Savage's critical hits and speed, the refusals, and that it never crosses Hunter's bond.

use super::ai::Plan;
use super::*;
use crate::action::{Event, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Strike;
use crate::content::{ActionKind, Content, CreatureSpawn, StatusId};
use crate::geom::Pos;
use crate::perception::Perception;
use crate::replay::Replay;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

const ME: Pos = Pos::new(7, 15);

/// A kitted human ranger in the showcase's lit, open hall (seed 7) with their companion beside
/// them, Sic 'em, and a full pool of focus.
fn ranger() -> World {
    let mut w = plain_ranger();
    grant(&mut w, "sic");
    set_focus(&mut w, 10);
    assert_eq!(w.state.player().pos, ME);
    w
}

fn plain_ranger() -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "healthy".into()]);
    World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world")
}

/// Gives the player talent `id`, as the table's level would.
fn grant(w: &mut World, id: &str) {
    let me = w.player_mut();
    let mut talents: Vec<crate::talent::Taken> = me.talents.iter().cloned().collect();
    talents.push(crate::talent::Taken {
        id: id.into(),
        choice: None,
    });
    me.talents =
        serde_json::from_value(serde_json::to_value(&talents).expect("json")).expect("talents");
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn set_focus(w: &mut World, value: u16) {
    let focus = w.content().resource_id("focus").expect("focus");
    w.player_mut().resources.set(focus, value);
}

fn focus(w: &World) -> u16 {
    let focus = w.content().resource_id("focus").expect("focus");
    (w.state.player().resources.get(focus)).map_or(0, |p| p.value)
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// A creature of `id` at `p` on the current level, with hit points to spare.
fn creature_at(w: &mut World, id: &str, p: Pos) -> ActorId {
    let spawn = CreatureSpawn {
        creature: w.content().creature_id(id).expect("a creature"),
        faction: None,
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
    let foe = travel::make_creature(&mut w.state, &content, &level, &spawn);
    w.state.actors.get_mut(&foe).unwrap().hp = 100_000;
    foe
}

/// Looks again, as a turn's end does, after a test moved or made someone.
fn look(w: &mut World) {
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
}

fn pet(w: &World) -> ActorId {
    w.companion().expect("a companion")
}

fn savage(w: &World) -> StatusId {
    w.content().status_id("savage").expect("savage")
}

fn degrees(w: &World) -> u16 {
    let status = savage(w);
    (w.state.actors[&pet(w)].effects.iter())
        .find(|e| e.status == status)
        .map_or(0, |e| e.severity)
}

fn order(w: &World) -> Order {
    w.state.actors[&pet(w)]
        .ally
        .as_ref()
        .expect("an ally")
        .order
}

fn quarry(w: &World) -> Option<ActorId> {
    w.state.actors[&pet(w)].companion().expect("data").quarry
}

fn sic(at: Pos) -> Action {
    Action::UseAt(crate::content::ClassActive::Sic, at)
}

#[test]
fn the_ranger_gains_it_at_level_40_as_the_design_prices_it() {
    let class = BUILTIN.class("ranger").expect("the ranger");
    let row = (class.abilities.iter())
        .find(|(_, id)| id == "sic")
        .expect("Sic 'em in the table");
    assert_eq!(row.0, 40);
    let active = crate::content::ClassActive::Sic;
    let talent = BUILTIN.class_active(active).expect("its talent");
    let cost = talent.resource.expect("a cost");
    assert_eq!(cost.resource, BUILTIN.resource_id("focus").expect("focus"));
    assert_eq!(cost.amount, 5);
    assert_eq!(BUILTIN.sic_range(), Some(10));
    let status = BUILTIN.status(BUILTIN.status_id("savage").expect("savage"));
    assert!(status.auto_critical && status.tick.is_none() && status.eases == Some(60));
    assert_eq!((status.max, status.cost), (3, 67));
    // No buff spell's, so Hunter's bond never carries it (`bond.rs::is_buff`) nor do the circles.
    assert!(
        BUILTIN
            .circles()
            .of(BUILTIN.status_id("savage").unwrap())
            .is_none()
    );
    assert!(
        plain_ranger()
            .state
            .player()
            .talent_mods(&BUILTIN)
            .sic
            .is_none()
    );
    let w = ranger();
    let sic = w.state.player().talent_mods(&BUILTIN).sic.expect("Sic 'em");
    assert_eq!((sic.blows, sic.range), (3, 10));
}

#[test]
fn it_sets_the_companion_on_the_creature_for_focus_and_a_wait() {
    let mut w = ranger();
    let at = Pos::new(11, 15);
    let foe = creature_at(&mut w, "kobold", at);
    look(&mut w);
    let at = w.state.actors[&foe].pos;
    set_focus(&mut w, 10);
    let price = w.price_of(w.state.player, ActionKind::Wait);
    let mut events = Vec::new();
    let spent = w.use_class_at(crate::content::ClassActive::Sic, at, &mut events);
    assert!(
        matches!(spent, Spent::Turn(p) if p == price),
        "a wait's time"
    );
    assert_eq!(focus(&w), 5);
    assert_eq!(quarry(&w), Some(foe));
    assert_eq!(degrees(&w), 3);
    assert!(matches!(order(&w), Order::Hunt { .. }));
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    let name = w.state.actors[&pet(&w)].companion().unwrap().name.clone();
    assert!(
        said.iter()
            .any(|l| l.contains(&format!("{name} goes for the kobold's throat."))),
        "{said:?}"
    );
}

#[test]
fn it_is_refused_with_no_time_and_no_focus_spent() {
    let mut w = ranger();
    let turn = w.state.turn;
    let check = |w: &mut World, at: Pos, line: &str| {
        w.apply(&sic(at));
        assert!(last(w).contains(line), "{} for {line}", last(w));
        assert_eq!(w.state.turn, turn, "no time for {line}");
        assert_eq!(degrees(w), 0, "no Savage for {line}");
        assert_eq!(quarry(w), None, "no hunt for {line}");
    };
    // Nobody there, oneself, out of reach (the hall is short: the cell need not be walkable).
    check(&mut w, Pos::new(12, 13), "nothing there");
    check(&mut w, ME, "cannot");
    check(&mut w, Pos::new(7 + 11, 15), "out of range");
    // No one to set it on but itself.
    let at = w.state.actors[&pet(&w)].pos;
    check(&mut w, at, "nothing there");
    assert_eq!(focus(&w), 10);
    // Short of focus.
    set_focus(&mut w, 4);
    check(&mut w, Pos::new(12, 13), "focus");
    set_focus(&mut w, 10);
    // In the retinue it is not here.
    let id = pet(&w);
    w.state
        .actors
        .get_mut(&id)
        .unwrap()
        .ally
        .as_mut()
        .unwrap()
        .retinue = true;
    check(&mut w, Pos::new(12, 13), "no companion");
    assert_eq!(focus(&w), 10);
}

#[test]
fn a_creature_not_hostile_may_not_be_its_quarry() {
    let mut w = ranger();
    let deer = creature_at(&mut w, "deer", Pos::new(11, 13));
    look(&mut w);
    let at = w.state.actors[&deer].pos;
    let turn = w.state.turn;
    w.apply(&sic(at));
    assert!(last(&w).contains("no quarrel"), "{}", last(&w));
    assert_eq!((w.state.turn, focus(&w)), (turn, 10));
}

#[test]
fn the_menu_says_when_there_is_no_companion_here() {
    let mut w = ranger();
    let active = crate::ability::Ability::Class(crate::content::ClassActive::Sic);
    let why = |w: &World| {
        (w.perception().abilities().into_iter())
            .find(|a| a.ability == active)
            .expect("listed")
            .unavailable
    };
    assert_eq!(why(&w), None);
    set_focus(&mut w, 4);
    assert!(matches!(
        why(&w),
        Some(crate::ability::Unavailable::Short(_))
    ));
    set_focus(&mut w, 10);
    let id = pet(&w);
    w.state.actors.remove(&id);
    run(&mut w, "wait");
    assert_eq!(why(&w), Some(crate::ability::Unavailable::NoCompanion));
    assert_eq!(
        crate::ability::Unavailable::NoCompanion
            .reason(w.content())
            .as_ref(),
        "no companion here"
    );
    assert_eq!(active.reach(w.content()), Some(10));
}

#[test]
fn savage_makes_every_hit_a_critical_and_each_hit_spends_a_degree() {
    // At level 60, by the table: Sic 'em, its focus, and a beast that does not miss for long.
    let mut w = plain_ranger();
    run(&mut w, "xp:90000000 levelup:auto");
    assert_eq!(focus(&w), 10);
    let foe = creature_at(&mut w, "kobold", Pos::new(9, 15));
    look(&mut w);
    let mut events = Vec::new();
    w.use_class_at(
        crate::content::ClassActive::Sic,
        Pos::new(9, 15),
        &mut events,
    );
    let (id, status) = (pet(&w), savage(&w));
    assert_eq!(degrees(&w), 3);
    let mut seen = 0;
    for _ in 0..200 {
        // Keep everyone up; nothing here is about who dies.
        w.state.actors.get_mut(&id).unwrap().hp = 1000;
        w.state.actors.get_mut(&foe).unwrap().hp = 100_000;
        w.player_mut().hp = 1000;
        let before = degrees(&w);
        let out = w.apply(&Action::Wait);
        let hits: Vec<Strike> = (out.events.iter())
            .filter_map(|e| match e {
                Event::Attacked {
                    attacker,
                    strike: Some(s),
                    ..
                } if *attacker == id => Some(*s),
                _ => None,
            })
            .filter(|s| s.is_hit())
            .collect();
        let eased = before.saturating_sub(degrees(&w));
        // A minute's easing cannot come in these seconds: only hits spend it.
        assert_eq!(usize::from(eased), hits.len().min(usize::from(before)));
        for s in hits.iter().take(usize::from(before)) {
            assert!(
                matches!(
                    s,
                    Strike::Hit { critical: true } | Strike::Soaked | Strike::Resisted
                ),
                "{s:?} under Savage"
            );
            seen += 1;
        }
        if degrees(&w) == 0 {
            break;
        }
    }
    assert_eq!(seen, 3, "three hits, three degrees");
    assert!(!w.state.actors[&id].effects.has(status), "gone at none");
}

#[test]
fn a_blow_without_savage_is_as_it_was() {
    let w = ranger();
    let plain = crate::combat::Profile::melee(
        &crate::combat::Attack::unarmed(None),
        &crate::stats::Stats::default(),
    );
    assert!(!w.savage_blow(pet(&w), plain).sure_critical);
}

#[test]
fn savage_makes_the_beast_half_again_as_fast() {
    let mut w = ranger();
    let id = pet(&w);
    let step = w.price_of(id, ActionKind::Step);
    let foe = creature_at(&mut w, "kobold", Pos::new(11, 15));
    look(&mut w);
    let at = w.state.actors[&foe].pos;
    w.apply(&sic(at));
    assert_eq!(w.price_of(id, ActionKind::Step), (step * 67 + 99) / 100);
    assert!(w.state.actors[&id].ally.is_some() && !w.disturbs_rest(&w.state.actors[&id]));
}

#[test]
fn savage_eases_by_the_minute_with_no_hits() {
    let mut w = ranger();
    let foe = creature_at(&mut w, "kobold", Pos::new(11, 15));
    look(&mut w);
    let at = w.state.actors[&foe].pos;
    let mut events = Vec::new();
    w.use_class_at(crate::content::ClassActive::Sic, at, &mut events);
    let id = pet(&w);
    // Nothing to bite: the kobold gone, the beast called to heel.
    w.state.actors.remove(&foe);
    run(&mut w, "order:follow");
    assert_eq!(degrees(&w), 3);
    run(&mut w, "rest:1m wait");
    assert!(degrees(&w) < 3, "a degree a minute: {}", degrees(&w));
    run(&mut w, "rest:3m wait");
    assert!(!w.state.actors[&id].effects.has(savage(&w)));
}

#[test]
fn the_hunt_goes_for_its_quarry_first_and_follows_it() {
    let mut w = ranger();
    let id = pet(&w);
    w.state.actors.get_mut(&id).unwrap().pos = Pos::new(7, 14);
    // A nearer kobold beside the beast, the quarry across the hall.
    creature_at(&mut w, "kobold", Pos::new(6, 13));
    let far = creature_at(&mut w, "kobold", Pos::new(12, 17));
    look(&mut w);
    assert!(w.perception.sees_actors_at(Pos::new(12, 17)));
    let mut events = Vec::new();
    let active = crate::content::ClassActive::Sic;
    w.use_class_at(active, Pos::new(12, 17), &mut events);
    assert_eq!(quarry(&w), Some(far));
    // It leaves the nearer one for the quarry: no bite at the kobold beside it, a step.
    assert!(matches!(w.ally_plan(id), Plan::Step(_)));
    // The quarry moves where the player sees: the cell moves with it.
    let there = Pos::new(13, 18);
    w.state.actors.get_mut(&far).unwrap().pos = there;
    look(&mut w);
    assert!(matches!(w.ally_plan(id), Plan::Step(_)));
    assert_eq!(order(&w), Order::Hunt { at: there });
    // Beyond the shut door, smelled: the beast goes for it, and the cell, which the dumps show,
    // tells the player nothing.
    w.state.actors.get_mut(&id).unwrap().pos = Pos::new(7, 12);
    let beyond = Pos::new(7, 10);
    w.state.actors.get_mut(&far).unwrap().pos = beyond;
    look(&mut w);
    assert!(!w.perception.is_visible(beyond));
    w.ally_plan(id);
    assert_eq!(order(&w), Order::Hunt { at: there });
    assert_eq!(quarry(&w), Some(far));
    // Beside it, it bites what it was set on and not the nearer kobold.
    w.state.actors.get_mut(&id).unwrap().pos = Pos::new(7, 15);
    let quarry_cell = Pos::new(8, 16);
    w.state.actors.get_mut(&far).unwrap().pos = quarry_cell;
    let other = creature_at(&mut w, "kobold", Pos::new(6, 15));
    look(&mut w);
    match w.ally_plan(id) {
        Plan::Attack { target, .. } => assert_ne!(target, other),
        plan => panic!("{plan:?}"),
    }
}

#[test]
fn a_new_order_ends_the_hunt_and_a_dead_quarry_ends_with_the_old_rule() {
    let mut w = ranger();
    let foe = creature_at(&mut w, "kobold", Pos::new(11, 15));
    look(&mut w);
    let at = w.state.actors[&foe].pos;
    w.apply(&sic(at));
    assert_eq!(quarry(&w), Some(foe));
    run(&mut w, "order:follow");
    assert_eq!(quarry(&w), None);
    assert_eq!(order(&w), Order::Follow);
    // Set on again, then its quarry is gone: the hunt's cell, then back to its master.
    let at = w.state.actors[&foe].pos;
    w.apply(&sic(at));
    w.state.actors.remove(&foe);
    for _ in 0..40 {
        run(&mut w, "wait");
        if order(&w) == Order::Follow {
            break;
        }
    }
    assert_eq!(order(&w), Order::Follow);
    assert_eq!(quarry(&w), None);
}

#[test]
fn it_never_crosses_to_the_ranger_and_a_save_keeps_the_hunt() {
    let mut w = ranger();
    let foe = creature_at(&mut w, "kobold", Pos::new(11, 15));
    look(&mut w);
    let at = w.state.actors[&foe].pos;
    w.apply(&sic(at));
    assert!(!w.state.player().effects.has(savage(&w)));
    let json = serde_json::to_string(&w.state.actors[&pet(&w)].ally).expect("json");
    let back: Option<crate::world::Ally> = serde_json::from_str(&json).expect("back");
    assert_eq!(back, w.state.actors[&pet(&w)].ally);
    assert!(json.contains("quarry"));
}

#[test]
fn a_replay_of_it_verifies() {
    let mut w = plain_ranger();
    let mut replay = Replay::new(&w);
    for action in parse_actions("xp:90000000 levelup:auto wait*2").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    // A refusal and orders replay exactly; the CLI replay of a real one is in the PR.
    for action in parse_actions("use:sic:12,13 order:stay wait*3 order:follow").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let text = replay.to_json();
    let replay = Replay::from_json(&text).expect("parses");
    let (again, divergence) = replay.play(w.content().clone()).expect("plays");
    assert!(divergence.is_none(), "{divergence:?}");
    assert_eq!(again.state_hash(), w.state_hash());
}
