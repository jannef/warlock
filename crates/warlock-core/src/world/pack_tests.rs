//! Pack tactics (`pack.rs`; spec §8 "Class abilities"): the ranger's level 18, flanking for the
//! ranger's melee and the companion's blows, never for a shot; the companion's share of the
//! favoured prey; the table's row.

use super::*;
use crate::action::AttackKind;
use crate::character::CharacterSpec;
use crate::combat::Profile;
use crate::content::{Content, CreatureSpawn};
use crate::geom::Pos;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A kitted human ranger in the showcase (seed 7), with their companion beside them.
fn ranger() -> World {
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

/// A kobold at `p` on the current level.
fn kobold_at(w: &mut World, p: Pos) -> ActorId {
    let spawn = CreatureSpawn {
        creature: w.content().creature_id("kobold").expect("kobold"),
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
    travel::make_creature(&mut w.state, &content, &level, &spawn)
}

/// The player at `me`, the companion at `pet`, a kobold at `foe` (positions only: the hook
/// reads nothing else of the ground); the companion's and the kobold's ids.
fn stand(w: &mut World, me: Pos, pet: Pos, foe: Pos) -> (ActorId, ActorId) {
    let companion = w.companion().expect("a companion");
    assert!(!w.state.actors[&companion].in_retinue());
    w.player_mut().pos = me;
    w.state.actors.get_mut(&companion).unwrap().pos = pet;
    let kobold = kobold_at(w, foe);
    (companion, kobold)
}

fn at(p: Pos, dx: i32, dy: i32) -> Pos {
    Pos::new(p.x + dx, p.y + dy)
}

fn base() -> Profile {
    Profile::melee(
        &crate::combat::Attack::unarmed(None),
        &crate::stats::Stats::default(),
    )
}

/// What `pack_blow` adds: (to hit, damage).
fn added(w: &World, attacker: ActorId, target: ActorId, kind: AttackKind) -> (i32, i32) {
    let p = w.pack_blow(attacker, target, kind, base());
    (p.to_hit - base().to_hit, p.bonus - base().bonus)
}

#[test]
fn the_ranger_gains_pack_tactics_at_level_18() {
    let class = BUILTIN.class("ranger").expect("the ranger");
    let row = (class.abilities.iter())
        .find(|(_, id)| id == "pack_tactics")
        .expect("Pack tactics in the table");
    assert_eq!(row.0, 18);
    let mut w = ranger();
    assert!(
        !w.state
            .player()
            .talent_mods(w.content())
            .beast
            .shares_favoured_prey
    );
    grant(&mut w, "pack_tactics");
    let mods = w.state.player().talent_mods(w.content());
    assert_eq!(mods.beast.pack_tactics, 2);
    assert!(mods.beast.shares_favoured_prey);
}

#[test]
fn a_creature_beside_both_is_flanked_for_the_rangers_melee_and_the_companions() {
    let mut w = ranger();
    let me = w.state.player().pos;
    let (pet, foe) = stand(&mut w, me, at(me, 2, 0), at(me, 1, 0));
    let player = w.state.player;
    // Before level 18, nothing.
    assert_eq!(added(&w, player, foe, AttackKind::Melee), (0, 0));
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (0, 0));
    grant(&mut w, "pack_tactics");
    assert_eq!(added(&w, player, foe, AttackKind::Melee), (2, 0));
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (2, 0));
    // A shot, a throw, a spell: flanking is about footing.
    for kind in [AttackKind::Ranged, AttackKind::Thrown, AttackKind::Spell] {
        assert_eq!(added(&w, player, foe, kind), (0, 0), "{kind:?}");
    }
    // The kobold's blows at either get nothing.
    assert_eq!(added(&w, foe, player, AttackKind::Melee), (0, 0));
    assert_eq!(added(&w, foe, pet, AttackKind::Melee), (0, 0));
    // Diagonals count: the companion at the kobold's corner.
    w.state.actors.get_mut(&pet).unwrap().pos = at(me, 2, 1);
    assert_eq!(added(&w, player, foe, AttackKind::Melee), (2, 0));
}

#[test]
fn a_creature_beside_only_one_of_them_is_not_flanked() {
    let mut w = ranger();
    grant(&mut w, "pack_tactics");
    let me = w.state.player().pos;
    let (pet, foe) = stand(&mut w, me, at(me, 3, 0), at(me, 1, 0));
    let player = w.state.player;
    assert_eq!(added(&w, player, foe, AttackKind::Melee), (0, 0));
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (0, 0));
    // The companion in the retinue stands nowhere.
    w.state.actors.get_mut(&pet).unwrap().pos = at(me, 2, 0);
    assert_eq!(added(&w, player, foe, AttackKind::Melee), (2, 0));
    let ally = w.state.actors.get_mut(&pet).unwrap().ally.as_mut().unwrap();
    ally.retinue = true;
    assert_eq!(added(&w, player, foe, AttackKind::Melee), (0, 0));
}

#[test]
fn the_companion_shares_the_rangers_favoured_prey() {
    let mut w = ranger();
    let me = w.state.player().pos;
    let (pet, foe) = stand(&mut w, me, at(me, 5, 0), at(me, 1, 0));
    let kobold = w.content().creature_id("kobold").expect("kobold");
    let level = w.state.current_level.clone();
    w.tally_kill(kobold, &level);
    assert_eq!(w.state.quests.prey(), Some("kobold"));
    // Favoured prey alone reaches only the ranger (`favoured`, not this hook).
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (0, 0));
    grant(&mut w, "pack_tactics");
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (1, 2));
    // The ranger's own blows get their prey's bonus elsewhere: nothing more here.
    assert_eq!(added(&w, w.state.player, foe, AttackKind::Melee), (0, 0));
    // Master hunter's half comes along.
    grant(&mut w, "master_hunter");
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (2, 4));
    // Flanked too: both.
    w.state.actors.get_mut(&pet).unwrap().pos = at(me, 2, 0);
    assert_eq!(added(&w, pet, foe, AttackKind::Melee), (4, 4));
    // Another type is not the prey.
    let other = kobold_at(&mut w, at(me, 9, 9));
    let rat = w.content().creature_id("rat").expect("rat");
    w.state.actors.get_mut(&other).unwrap().creature = rat;
    assert_eq!(added(&w, pet, other, AttackKind::Melee), (0, 0));
}
