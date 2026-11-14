//! The animated objects' part of the dread design (`docs/design/monster-dread.md` §13, #1136):
//! the rug of smothering engulfs at its own gentler harm, the bellows' gust staggers, the lantern
//! bursts in burning oil as it dies, and the blades come notched or keen.

use crate::action::{AttackKind, Event, parse_actions};
use crate::combat::Strike;
use crate::content::{Affliction, BurstWhen, ContentFiles, CreatureSpawn, StatusId};
use crate::world::combat::fight::Harm;
use crate::world::*;
use std::sync::LazyLock;

/// A lit hall: the player at (2, 2), a rug east of them, a lantern west, a charnel heap below.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#.......#
#l@r....#
#..h....#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "animated_rug" }
"l" = { terrain = "floor", creature = "animated_lantern" }
"h" = { terrain = "floor", creature = "charnel_heap" }
"##;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/dread_hall.toml", HALL);
    files.insert(
        "scenarios/dread_hall.toml",
        "name = \"Hall\"\nstart_level = \"dread_hall\"\nseed = 3\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

fn world() -> World {
    World::new(CONTENT.clone(), "dread_hall", Some(3)).expect("world")
}

fn first(w: &World, kind: &str) -> ActorId {
    let id = w.content().creature_id(kind).expect(kind);
    (w.state.actors.values())
        .find(|a| a.creature == id)
        .map(|a| a.id)
        .expect("one there")
}

fn status(id: &str) -> StatusId {
    CONTENT.status_id(id).expect(id)
}

fn attack_rider(kind: &str) -> Vec<Affliction> {
    let def = CONTENT.creature(CONTENT.creature_id(kind).expect(kind));
    let attack = def.attack.as_ref().expect("an attack");
    attack.on_hit.iter().map(|o| o.what).collect()
}

/// Hit points the player loses to an engulfing by `holder` (none: no one's) over five waits,
/// everyone else kept still.
fn engulfed_loss(holder: Option<&str>) -> i32 {
    let mut w = world();
    for a in w.state.actors.values_mut() {
        if a.kind != ActorKind::Player {
            a.energy = i64::MIN / 4;
        }
    }
    let by = holder.map(|h| first(&w, h));
    let player = w.state.player;
    w.afflict(player, status("engulfed"), 1, by, &mut Vec::new());
    assert!(w.state.player().effects.has(status("engulfed")));
    let hp = w.state.player().hp;
    for action in parse_actions("wait*5").expect("actions") {
        w.apply(&action);
    }
    hp - w.state.player().hp
}

#[test]
fn the_rug_engulfs_at_a_point_a_tick() {
    // Its blow closes over you: engulfed, at a grip of 12 and 1 hit point a tick.
    let engulfed = status("engulfed");
    assert_eq!(
        attack_rider("animated_rug"),
        vec![Affliction::Hold {
            status: engulfed,
            difficulty: Some(12),
            damage: Some(1),
        }]
    );
    // While it holds, a tick costs its own harm, not the status's 2.
    let mut w = world();
    let player = w.state.player;
    let rug = first(&w, "animated_rug");
    w.afflict(player, engulfed, 1, Some(rug), &mut Vec::new());
    assert_eq!(w.hold_harms(player), vec![(engulfed, 1)]);
    // The charnel heap's is the status's own.
    let mut w = world();
    let heap = first(&w, "charnel_heap");
    w.afflict(player, engulfed, 1, Some(heap), &mut Vec::new());
    assert!(w.hold_harms(player).is_empty());
    // The same hold, the same turns drawn: the rug's harms half as much as no one's.
    let rug = engulfed_loss(Some("animated_rug"));
    let plain = engulfed_loss(None);
    assert!(rug > 0, "it hurts");
    assert_eq!(rug * 2, plain, "a point a tick, not two");
}

#[test]
fn the_rug_lets_go_when_it_dies() {
    let mut w = world();
    let player = w.state.player;
    let rug = first(&w, "animated_rug");
    w.afflict(player, status("engulfed"), 1, Some(rug), &mut Vec::new());
    assert!(w.hurt(rug, 1000, Harm::Actor(player), &mut Vec::new()));
    for action in parse_actions("wait").expect("actions") {
        w.apply(&action);
    }
    assert!(!w.state.player().effects.has(status("engulfed")));
}

#[test]
fn the_bellows_gust_staggers() {
    let def = CONTENT.creature(CONTENT.creature_id("animated_bellows").expect("bellows"));
    assert_eq!(def.attack.as_ref().expect("an attack").name, "gust");
    assert_eq!(
        attack_rider("animated_bellows"),
        vec![Affliction::Status {
            status: status("staggered"),
            severity: 1,
            check: None,
        }]
    );
}

#[test]
fn the_lantern_bursts_in_burning_oil_as_it_dies() {
    let def = CONTENT.creature(CONTENT.creature_id("animated_lantern").expect("lantern"));
    let burst = def.burst.as_ref().expect("death throes");
    assert_eq!(burst.when, BurstWhen::Dies);
    assert_eq!(
        (burst.damage.min(), burst.damage.max(), burst.radius),
        (1, 6, 1)
    );
    assert_eq!(burst.kind, CONTENT.damage_type_id("fire").expect("fire"));
    // No other household thing bursts.
    for d in CONTENT.creature_defs() {
        if d.animated.is_some() && d.id != "animated_lantern" {
            assert!(d.burst.is_none(), "{}", d.id);
        }
    }
    let mut w = world();
    let player = w.state.player;
    let lantern = first(&w, "animated_lantern");
    let mut events = Vec::new();
    assert!(w.hurt(lantern, 1000, Harm::Actor(player), &mut events));
    w.say_news();
    let blows: Vec<Strike> = (events.iter())
        .filter_map(|e| match *e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Spell,
                strike: Some(strike),
                ..
            } if attacker == lantern && target == player => Some(strike),
            _ => None,
        })
        .collect();
    assert_eq!(blows.len(), 1, "the oil reached the player beside it");
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|l| l == "The animated lantern bursts in a spray of burning oil!"),
        "{said:?}"
    );
}

#[test]
fn blades_come_notched_or_keen_and_no_other_animated_thing_is_graded() {
    for d in CONTENT.creature_defs() {
        if d.animated.is_none() {
            continue;
        }
        let blade = d.id.starts_with("animated_blade_");
        assert_eq!(d.grades, blade, "{}", d.id);
        if blade {
            assert_eq!(d.grade_words, ["notched", "keen"].map(String::from));
        }
    }
    let blade = CreatureSpawn {
        creature: CONTENT.creature_id("animated_blade_10").expect("a blade"),
        faction: None,
        alignment: None,
        npc: None,
        pos: crate::geom::Pos::new(1, 1),
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let grades: Vec<i8> = (0..600)
        .map(|i| crate::grade::draw(&CONTENT, 5, ActorId(i), &blade))
        .collect();
    assert!(grades.contains(&-1) && grades.contains(&1) && grades.contains(&0));
}
