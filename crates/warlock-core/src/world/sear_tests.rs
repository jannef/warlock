//! Fire and blood (spec §8 "Blood"): a blow draws blood only of a kind that does (fire sears,
//! cold numbs); fire chars the floor it burns and the faces of the walls beside it, more with
//! every burning; char is perceived and saved as blood is, and never changes what happens.

use super::spell::Caster;
use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::combat::{Blow, Strike};
use crate::content::{Burst, ContentFiles, DamageTypeId};
use crate::dice::Roll;
use crate::geom::Dir;
use crate::time;
use std::sync::LazyLock;

/// A lit hall: the player (`@`), a giant rat (`r`, peaceful), room for a burst east.
const MAP: &str = "\
##################
#................#
#.@..r...........#
#................#
##################
";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/seartest.toml",
        &format!(
            "name = \"Sear\"\nambient = [255, 255, 255, 0]\nmap = '''\n{MAP}'''\n\n\
             [legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\", faction = \"peaceful\" }}\n"
        ),
    );
    files.insert(
        "scenarios/seartest.toml",
        "name = \"Sear\"\nstart_level = \"seartest\"\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "seartest", None).expect("world")
}

fn kind(id: &str) -> Option<DamageTypeId> {
    Some(CONTENT.damage_type_id(id).expect(id))
}

fn rat(w: &World) -> ActorId {
    let creature = w.content().creature_id("rat").expect("rat");
    (w.state().actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
        .expect("on the level")
}

/// A killing blow of `kind` by the player on the rat.
fn kill_rat(w: &mut World, kind: Option<DamageTypeId>) {
    let player = w.state().player;
    let from = w.player_pos();
    let blow = Blow {
        strike: Strike::Hit { critical: true },
        damage: 30,
        kind,
    };
    w.bleed((player, rat(w)), from, AttackKind::Melee, blow);
}

/// A burst of `kind` of radius 2 on `center`, doing no harm.
fn burst(w: &mut World, center: Pos, kind: Option<DamageTypeId>) {
    let burst = Burst {
        spell: String::new(),
        radius: 2,
        damage: Roll::fixed(0),
        reroll: 0,
        kind,
        blast: 0,
        ice: None,
    };
    w.state.time += time::Duration::seconds(6);
    w.cast_burst(Caster::Place, (center, center), &burst, &mut Vec::new());
}

fn level(w: &World) -> &LevelState {
    w.state().current()
}

#[test]
fn fire_draws_no_blood_but_sears_and_physical_blows_bleed() {
    let mut w = world();
    let at = w.state().actors[&rat(&w)].pos;
    kill_rat(&mut w, kind("fire"));
    assert!(level(&w).blood.is_empty(), "fire sears");
    assert!(level(&w).scorch.contains_key(&at), "the burned rat's cell");
    // Cold and lightning draw none, and leave no char.
    for id in ["cold", "electricity", "necrotic"] {
        let mut w = world();
        kill_rat(&mut w, kind(id));
        assert!(
            level(&w).blood.is_empty() && level(&w).scorch.is_empty(),
            "{id}"
        );
    }
    // A blade, acid, and force draw blood and leave no char.
    for id in ["slashing", "piercing", "acid", "force"] {
        let mut w = world();
        kill_rat(&mut w, kind(id));
        assert!(!level(&w).blood.is_empty(), "{id} bleeds");
        assert!(level(&w).scorch.is_empty(), "{id}");
    }
}

#[test]
fn a_fire_burst_chars_the_floor_and_the_wall_faces_more_each_time() {
    let mut w = world();
    let center = Pos::new(12, 2);
    burst(&mut w, center, kind("fire"));
    let first = level(&w).scorch.clone();
    assert!(first.contains_key(&center) && first.contains_key(&Pos::new(12, 1)));
    // The walls north and south take it on the faces toward the fire, nothing beyond.
    let (north, south) = (Pos::new(12, 0), Pos::new(12, 4));
    assert!(first[&north].on_face(Dir::S) && !first[&north].on_face(Dir::N));
    assert!(first[&south].on_face(Dir::N));
    assert!(first.keys().all(|p| p.chebyshev(center) <= 3));
    assert_eq!(crate::scorch::word(&first[&center]), "scorched");
    // Fire after fire: it builds up, darker, to the cap.
    for _ in 0..12 {
        burst(&mut w, center, kind("fire"));
    }
    let charred = &level(&w).scorch[&center];
    assert!(charred.amount > first[&center].amount);
    assert_eq!(crate::scorch::word(charred), "charred");
    assert_eq!(charred.amount, crate::blood::CAP);
    // A burst of cold chars nothing.
    let mut w = world();
    burst(&mut w, center, kind("cold"));
    assert!(level(&w).scorch.is_empty());
}

#[test]
fn a_fire_zone_chars_what_it_burns_on() {
    let mut w = world();
    for action in parse_actions("zone:fire@12,2,1").expect("zone") {
        w.apply(&action);
    }
    let scorch = &level(&w).scorch;
    for p in [Pos::new(12, 2), Pos::new(11, 2), Pos::new(12, 3)] {
        assert!(scorch.contains_key(&p), "{p}");
    }
    for action in parse_actions("zone:smoke@5,2,1").expect("zone") {
        w.apply(&action);
    }
    assert!(
        !level(&w).scorch.contains_key(&Pos::new(5, 1)),
        "smoke chars nothing"
    );
}

#[test]
fn char_is_perceived_remembered_and_saved_as_blood_is() {
    let mut w = world();
    burst(&mut w, Pos::new(12, 2), kind("fire"));
    w.apply(&Action::Wait);
    let view = w.perception();
    assert!(view.scorch(Pos::new(12, 2)).is_some());
    assert!(
        view.scorch(Pos::new(12, 0)).is_some(),
        "the wall's face is seen"
    );
    assert!(view.blood(Pos::new(12, 2)).is_none());
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Scorch);
    assert!(rows[2].chars().nth(12).is_some_and(|c| c.is_ascii_digit()));
    assert!(!level(&w).scorch_memory.is_empty());
    let loaded = crate::save::decode(&crate::save::encode(&w), CONTENT.clone()).expect("loads");
    assert_eq!(level(&loaded).scorch, level(&w).scorch);
    assert_eq!(level(&loaded).scorch_memory, level(&w).scorch_memory);
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn char_never_changes_what_happens() {
    // Fire zones and a fire fight twice, once with the char washed away after every action:
    // every roll, blow, death, message, and step is the same; only the char differs.
    let actions = "zone:fire@5,2,1 wait*3 zone:fire@6,2,2 wait*6";
    let mut sooty = world();
    let mut clean = world();
    for action in parse_actions(actions).expect("actions") {
        assert_eq!(sooty.apply(&action), clean.apply(&action));
        for level in clean.state.levels.values_mut() {
            level.scorch.clear();
            level.scorch_memory.clear();
        }
    }
    let (a, b) = (sooty.state(), clean.state());
    assert!(!a.current().scorch.is_empty(), "it burned");
    assert_eq!(a.rng, b.rng, "the gameplay stream is untouched");
    let without = |s: &WorldState| {
        let mut s = s.clone();
        for level in s.levels.values_mut() {
            level.scorch.clear();
            level.scorch_memory.clear();
        }
        s
    };
    assert_eq!(without(a), without(b));
}
