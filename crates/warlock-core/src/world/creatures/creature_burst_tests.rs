//! Creatures' bursts (the dread design §5.3, `creature_burst.rs`): a vortex bursts beside its
//! target instead of attacking and dies of it, and goes quietly when killed from afar; death
//! throes go off where an elemental dies, striking everyone in the radius in full but its own
//! kind, through the spells' shared burst (resistances, the floor's ruin); a wave douses, a
//! gust blows the loose things outward; the earth line throws stones and pounds up close.

use crate::action::{AttackKind, Event, parse_actions};
use crate::combat::Strike;
use crate::content::ContentFiles;
use crate::test_support::said;
use crate::world::combat::fight::Harm;
use crate::world::*;
use std::sync::LazyLock;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"e" = { terrain = "floor", creature = "ember_mote" }
"F" = { terrain = "floor", creature = "fire_elemental" }
"f" = { terrain = "floor", creature = "lesser_fire_elemental" }
"W" = { terrain = "floor", creature = "water_elemental" }
"A" = { terrain = "floor", creature = "air_elemental" }
"E" = { terrain = "floor", creature = "earth_elemental" }
"H" = { terrain = "floor", creature = "hill_giant" }
"s" = { terrain = "floor", objects = ["scroll_of_light*40"] }
"p" = { terrain = "floor", objects = ["dagger"] }
"##;

/// The maps, by scenario: each a lit hall.
const MAPS: &[(&str, &str)] = &[
    ("vortex", "#########\n#@.e....#\n#.......#\n#########\n"),
    (
        "throes",
        "#########\n#.......#\n#@FsfH..#\n#.F.H...#\n#########\n",
    ),
    ("ally", "#######\n#######\n#@.F..#\n#######\n#######\n"),
    ("wave", "#######\n#.....#\n#@W...#\n#.....#\n#######\n"),
    (
        "gust",
        "#########\n#.......#\n#@.Ap...#\n#.......#\n#########\n",
    ),
    (
        "stones",
        "###########\n#@.....E..#\n#.........#\n###########\n",
    ),
];

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, map) in MAPS {
        files.insert(
            &format!("levels/burst_{name}.toml"),
            &format!(
                "name = \"Burst hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{LEGEND}"
            ),
        );
        files.insert(
            &format!("scenarios/burst_{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"burst_{name}\"\nhp = 1000\n"),
        );
    }
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

fn world(name: &str) -> World {
    World::new(CONTENT.clone(), &format!("burst_{name}"), None).expect("world")
}

fn run(w: &mut World, actions: &str) -> Vec<Event> {
    let mut events = Vec::new();
    for action in parse_actions(actions).expect("actions") {
        events.extend(w.apply(&action).events);
    }
    events
}

/// The actors of creature type `kind`, in id order.
fn all_of(w: &World, kind: &str) -> Vec<ActorId> {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .filter(|a| a.creature == id)
        .map(|a| a.id)
        .collect()
}

fn first_of(w: &World, kind: &str) -> ActorId {
    all_of(w, kind)[0]
}

/// The player kills `id` outright; what it made, its news said.
fn kill(w: &mut World, id: ActorId) -> Vec<Event> {
    let mut events = Vec::new();
    let player = w.state.player;
    assert!(w.hurt(id, 100_000, Harm::Actor(player), &mut events));
    w.say_news();
    events
}

/// The spell blows `by` struck, by target, with how they landed.
fn struck(events: &[Event], by: ActorId) -> Vec<(ActorId, Strike)> {
    (events.iter())
        .filter_map(|e| match *e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Spell,
                strike: Some(strike),
                ..
            } if attacker == by => Some((target, strike)),
            _ => None,
        })
        .collect()
}

fn hit(strike: Strike) -> bool {
    matches!(strike, Strike::Hit { .. })
}

fn full(w: &World) -> i32 {
    w.state.player().max_hp(w.content())
}

#[test]
fn a_vortex_bursts_beside_its_target_instead_of_attacking_and_dies_of_it() {
    let mut w = world("vortex");
    let mote = first_of(&w, "ember_mote");
    let mut events = Vec::new();
    for _ in 0..6 {
        events.extend(run(&mut w, "wait"));
        if !w.state.actors.contains_key(&mote) {
            break;
        }
    }
    assert!(!w.state.actors.contains_key(&mote), "it is gone");
    let said = said(&w);
    assert!(
        said.iter().any(|l| l == "The ember mote bursts!"),
        "{said:?}"
    );
    let player = w.state.player;
    assert!(
        struck(&events, mote)
            .iter()
            .any(|&(t, s)| t == player && hit(s)),
        "the flame struck the player"
    );
    assert!(w.state.player().hp < full(&w), "and hurt them");
    let melee = (events.iter()).any(|e| {
        matches!(e, Event::Attacked { attacker, kind: AttackKind::Melee, .. } if *attacker == mote)
    });
    assert!(!melee, "it never struck a blow");
    assert!(
        (events.iter()).any(|e| matches!(e, Event::Spell { effect, .. } if effect == "blaze")),
        "its burst is drawn"
    );
}

#[test]
fn a_vortex_killed_from_afar_goes_quietly() {
    let mut w = world("vortex");
    let mote = first_of(&w, "ember_mote");
    let events = kill(&mut w, mote);
    assert!(struck(&events, mote).is_empty(), "no burst");
    assert!(!(events.iter()).any(|e| matches!(e, Event::Spell { .. })));
    assert!(!said(&w).iter().any(|l| l.contains("bursts")));
    assert_eq!(w.state.player().hp, full(&w));
}

#[test]
fn death_throes_strike_the_radius_in_full_but_its_own_kind() {
    let mut w = world("throes");
    let [dying, twin] = all_of(&w, "fire_elemental")[..] else {
        panic!("two fire elementals");
    };
    let lesser = first_of(&w, "lesser_fire_elemental");
    let [far, near] = all_of(&w, "hill_giant")[..] else {
        panic!("two giants");
    };
    let events = kill(&mut w, dying);
    let blows = struck(&events, dying);
    let player = w.state.player;
    let on = |id: ActorId| blows.iter().find(|&&(t, _)| t == id).map(|&(_, s)| s);
    assert!(
        on(player).is_some_and(hit),
        "the player beside it: {blows:?}"
    );
    assert_eq!(on(twin), None, "its own kind is spared");
    assert_eq!(
        on(lesser),
        Some(Strike::Resisted),
        "fire cannot hurt a fire thing"
    );
    // The near giant two cells east and a row down is within radius 2 (rounded); the far one,
    // three cells east, is not.
    assert!(on(near).is_some_and(hit), "within the radius");
    assert_eq!(on(far), None, "beyond the radius");
    let said = said(&w);
    assert!(
        said.iter()
            .any(|l| l == "The fire elemental comes apart in flame!"),
        "{said:?}"
    );
    assert!(said.iter().any(|l| l == "The flame burns you."), "{said:?}");
}

#[test]
fn death_throes_ruin_what_lies_on_the_floor() {
    let mut w = world("throes");
    let dying = first_of(&w, "fire_elemental");
    let at = w.state.actors[&dying].pos;
    let pile = Pos::new(at.x + 1, at.y);
    let count = |w: &World| -> u32 {
        w.state
            .current()
            .objects_at(pile)
            .iter()
            .map(|s| s.count)
            .sum()
    };
    assert_eq!(count(&w), 40);
    kill(&mut w, dying);
    assert!(count(&w) < 40, "some of the scrolls burnt");
}

#[test]
fn death_throes_strike_the_players_allies_too() {
    let mut w = world("ally");
    run(&mut w, "ally:wolf");
    let wolf = first_of(&w, "wolf");
    let dying = first_of(&w, "fire_elemental");
    let events = kill(&mut w, dying);
    let blows = struck(&events, dying);
    assert!(
        blows.iter().any(|&(t, s)| t == wolf && hit(s)),
        "the wolf between them: {blows:?}"
    );
}

#[test]
fn a_waves_death_throes_douse_the_burning() {
    let mut w = world("wave");
    let burning = w.content().status_id("burning").expect("burning");
    let player = w.state.player;
    w.afflict(player, burning, 2, None, &mut Vec::new());
    assert!(w.state.player().effects.has(burning));
    let dying = first_of(&w, "water_elemental");
    let events = kill(&mut w, dying);
    assert!(
        !w.state.player().effects.has(burning),
        "the wave put it out"
    );
    assert!(struck(&events, dying).iter().any(|&(t, _)| t == player));
}

#[test]
fn a_gusts_death_throes_blow_what_lies_loose_a_cell_outward() {
    let mut w = world("gust");
    let dying = first_of(&w, "air_elemental");
    let at = w.state.actors[&dying].pos;
    let beside = Pos::new(at.x + 1, at.y);
    let beyond = Pos::new(at.x + 2, at.y);
    assert!(!w.state.current().objects_at(beside).is_empty());
    kill(&mut w, dying);
    assert!(w.state.current().objects_at(beside).is_empty(), "blown off");
    let dagger = w.content().object_id("dagger").expect("dagger");
    assert!(
        (w.state.current().objects_at(beyond).iter()).any(|s| s.object == dagger),
        "a cell farther out"
    );
}

#[test]
fn the_earth_line_hurls_stones_from_afar_and_pounds_up_close() {
    let mut w = world("stones");
    let earth = first_of(&w, "earth_elemental");
    let kinds = |events: &[Event]| -> Vec<AttackKind> {
        (events.iter())
            .filter_map(|e| match *e {
                Event::Attacked { attacker, kind, .. } if attacker == earth => Some(kind),
                _ => None,
            })
            .collect()
    };
    let mut thrown = false;
    let mut pounded = false;
    for _ in 0..40 {
        let events = run(&mut w, "wait");
        let near = w.state.actors[&earth].pos.chebyshev(w.player_pos()) <= 1;
        for kind in kinds(&events) {
            match kind {
                AttackKind::Thrown => thrown = true,
                AttackKind::Melee => pounded = true,
                _ => {}
            }
        }
        // Once it stands beside the player it never throws again.
        if near {
            assert!(!kinds(&run(&mut w, "wait")).contains(&AttackKind::Thrown));
        }
        if thrown && pounded {
            break;
        }
    }
    assert!(thrown, "it hurled stones from afar");
    let said = said(&w);
    assert!(
        said.iter()
            .any(|l| l == "The earth elemental hurls a stone."),
        "{said:?}"
    );
    // Hold the player in place beside it: it pounds.
    let pos = w.state.actors[&earth].pos;
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("the player").pos = Pos::new(pos.x - 1, pos.y);
    for _ in 0..10 {
        if kinds(&run(&mut w, "wait")).contains(&AttackKind::Melee) {
            pounded = true;
            break;
        }
    }
    assert!(pounded, "beside the player it pounds");
}

#[test]
fn a_burst_on_touch_is_a_creatures_whole_offence() {
    let mut files = ContentFiles::builtin();
    let path = "data/creatures/ember_mote.toml";
    let text = include_str!("../../../../../assets/data/creatures/ember_mote.toml").replace(
        "[creature.burst]",
        "attack = { damage = \"1d4\", name = \"touch\", verb = \"singes\" }\n[creature.burst]",
    );
    files.insert(path, &text);
    let error = Content::load(&files).expect_err("an attack beside a burst on touch");
    assert!(error.to_string().contains("burst on touch"), "{error}");
}

#[test]
fn the_burning_skull_the_magma_crawler_and_the_balor_burst_when_they_die() {
    let content = &*CONTENT;
    let fire = content.damage_type_id("fire").expect("fire");
    for (id, (min, max), radius) in [
        ("burning_skull", (2, 12), 1),
        ("magma_crawler", (2, 16), 1),
        ("balor", (8, 96), 2),
    ] {
        let def = content.creature(content.creature_id(id).expect(id));
        let burst = def
            .burst
            .as_ref()
            .unwrap_or_else(|| panic!("{id}: a burst"));
        assert_eq!(burst.when, crate::content::BurstWhen::Dies, "{id}");
        assert_eq!(burst.kind, fire, "{id}");
        assert_eq!((burst.damage.min(), burst.damage.max()), (min, max), "{id}");
        assert_eq!(burst.radius, radius, "{id}");
    }
}
