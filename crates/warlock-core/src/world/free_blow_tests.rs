//! Free blows in the world (spec §8 "Free blows"): a cleave after a kill (the next hostile
//! clockwise, one an attack, more with great cleave), a riposte after a blow the Parry turned
//! (never answered by another), an opportunist's blow at a hostile stepping away (before it
//! steps), the player's blows in another's turn said after what they answer, and none of it
//! drawing or changing anything while the switches are off or the triggers do not fire.

use super::*;
use crate::action::AttackKind;
use crate::content::ContentFiles;
use crate::equipment;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "folk"
name = "folk"
"##;

// Rats die of any hit; the kobold stands for as long as it takes; the duellist parries; the
// caster never fights in melee; the gardener is no one's enemy.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "rat"
name = "rat"
glyph = "r"
fg = "#806040"
weight = 300
faction = "vermin"
stats = { toughness = 1 }

[[creature]]
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#c08040"
weight = 25000
faction = "vermin"
base_hp = 1000

[[creature]]
id = "caster"
name = "caster"
glyph = "c"
fg = "#c08040"
weight = 25000
faction = "vermin"
base_hp = 1000
combat = "caster"
range = 2

[[creature]]
id = "gardener"
name = "gardener"
glyph = "g"
fg = "#40c040"
weight = 60000
faction = "folk"
base_hp = 1
"##;

// The axe never misses but on a fumble; the foil parries nearly everything.
const OBJECTS: &str = r##"
[[object]]
id = "axe"
name = "axe"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 1000
value = 0
slot = "main_hand"
damage = "1d6"
to_hit = 20
category = "long_blades"

[[object]]
id = "foil"
name = "foil"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["steel"]
weight = 500
value = 0
slot = "main_hand"
damage = "1d2"
parry = 20
category = "long_blades"
"##;

const SLOTS: &str = r##"
[[slot]]
id = "main_hand"
name = "Main hand"
action = "wield"
mark = "wielded"
hand = true

[[slot]]
id = "off_hand"
name = "Off hand"
action = "wield"
mark = "in off hand"
hand = true
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"k" = { terrain = "floor", creature = "kobold" }
"K" = { terrain = "floor", creature = "kobold", carry = ["foil"], wield = ["foil"] }
"c" = { terrain = "floor", creature = "caster", carry = ["foil"], wield = ["foil"] }
"g" = { terrain = "floor", creature = "gardener" }
"p" = { terrain = "floor", creature = "kobold", route = "E1 W1" }
"##;

const WEAPON_SKILLS: &str = include_str!("../../../../assets/data/weapon_skills.toml");

/// The test content with `map` at `light` (0 dark to 255 fully lit); the player has a thousand
/// hit points.
fn content_with(map: &str, light: u8) -> Arc<Content> {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/weapon_skills.toml", WEAPON_SKILLS),
        (
            "data/materials.toml",
            "[[material]]\nid = \"steel\"\nname = \"steel\"\n",
        ),
        ("data/legend.toml", LEGEND),
        (
            "data/vision.toml",
            "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/yard.toml",
        &format!(
            "name = \"The Yard\"\nambient = [{light}, {light}, {light}, 0]\nmap = '''\n{map}'''\n"
        ),
    );
    files.insert(
        "scenarios/yard.toml",
        "name = \"Yard\"\nstart_level = \"yard\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world(map: &str) -> World {
    World::new(content_with(map, 255), "yard", None).expect("world")
}

/// The player wields `object`.
fn arm(w: &mut World, object: &str) {
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    let object = content.object_id(object).expect("an object");
    me.inventory.push(crate::object::Stack::new(object, 1));
    let item = me.inventory.len() - 1;
    let slot = equipment::free_slot(&content, &me.equipment, object).expect("a free slot");
    me.equipment
        .put(&equipment::slots_taken(&content, object, slot), item);
}

/// Switches actor `id`'s free blows as `set` does.
fn switch(w: &mut World, id: ActorId, set: impl FnOnce(&mut FreeBlows)) {
    set(&mut w.state.actors.get_mut(&id).expect("the actor").free_blows);
}

/// Who stands at (`x`, `y`).
fn at(w: &World, x: i32, y: i32) -> ActorId {
    let level = &w.state.current().id;
    w.state.actor_at(level, Pos::new(x, y)).expect("someone").id
}

/// The melee blows in `outcome`: who struck whom, in order.
fn blows(outcome: &Outcome) -> Vec<(ActorId, ActorId)> {
    (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Melee,
                ..
            } => Some((*attacker, *target)),
            _ => None,
        })
        .collect()
}

/// Whom actor `by` struck in melee in `outcome`, in order.
fn struck_by(outcome: &Outcome, by: ActorId) -> Vec<ActorId> {
    (blows(outcome).into_iter())
        .filter_map(|(a, t)| (a == by).then_some(t))
        .collect()
}

/// The melee blows in `outcome` with what came of each.
fn strikes(outcome: &Outcome) -> Vec<(ActorId, ActorId, Option<crate::combat::Strike>)> {
    (outcome.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                attacker,
                target,
                kind: AttackKind::Melee,
                strike,
                ..
            } => Some((*attacker, *target, *strike)),
            _ => None,
        })
        .collect()
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

/// Rats east, south-east, and south of the player, and a gardener south-west.
const RING: &str = "#####\n#...#\n#.@r#\n#grr#\n#####\n";

#[test]
fn a_cleave_turns_to_the_next_hostile_clockwise_after_a_kill() {
    let (east, south_east, south) = {
        let w = world(RING);
        (at(&w, 3, 2), at(&w, 3, 3), at(&w, 2, 3))
    };
    let gardener = at(&world(RING), 1, 3);
    // Without the switch: one blow, the rat east dies, nothing more.
    let mut w = world(RING);
    arm(&mut w, "axe");
    let player = w.player_id();
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(struck_by(&outcome, player), [east]);
    // Cleave: the next hostile clockwise from where it fell, the rat south-east; one only.
    let mut w = world(RING);
    arm(&mut w, "axe");
    switch(&mut w, player, |f| f.cleave = true);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(struck_by(&outcome, player), [east, south_east]);
    assert!(w.state.actors.contains_key(&south));
    // Each said as it is struck: two blows, two deaths.
    let deaths = said(&w).iter().filter(|m| m.contains("dies")).count();
    assert_eq!(deaths, 2, "{:#?}", said(&w));
    // From the south, clockwise passes the gardener (no enemy) and comes to the east.
    let mut w = world(RING);
    arm(&mut w, "axe");
    switch(&mut w, player, |f| f.cleave = true);
    let outcome = w.apply(&Action::Move(Dir::S));
    assert_eq!(struck_by(&outcome, player), [south, east]);
    // Great cleave: on while the blows kill, all three rats, never the gardener.
    let mut w = world(RING);
    arm(&mut w, "axe");
    switch(&mut w, player, |f| f.great_cleave = true);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(struck_by(&outcome, player), [east, south_east, south]);
    assert!(w.state.actors.contains_key(&gardener));
    // All in one action's time.
    assert_eq!(outcome.elapsed, Duration::seconds(6));
}

#[test]
fn a_cleave_needs_a_kill_and_follows_a_kick_too() {
    // The kobold east stands: no cleave at the rats.
    let mut w = world("#####\n#...#\n#.@k#\n#.rr#\n#####\n");
    arm(&mut w, "axe");
    let player = w.player_id();
    switch(&mut w, player, |f| f.great_cleave = true);
    let kobold = at(&w, 3, 2);
    let outcome = w.apply(&Action::Move(Dir::E));
    assert_eq!(struck_by(&outcome, player), [kobold]);
    // A kick that kills cleaves as a blow does.
    let mut w = world(RING);
    arm(&mut w, "axe");
    switch(&mut w, player, |f| f.cleave = true);
    let (east, south_east) = (at(&w, 3, 2), at(&w, 3, 3));
    let mut cleaved = false;
    for _ in 0..4 {
        if !w.state.actors.contains_key(&east) {
            break;
        }
        let outcome = w.apply(&Action::Kick(Dir::E));
        let struck = struck_by(&outcome, player);
        if w.state.actors.contains_key(&east) {
            assert_eq!(struck, [east], "a kick that misses gives nothing");
        } else {
            assert_eq!(struck, [east, south_east]);
            cleaved = true;
        }
    }
    assert!(cleaved);
}

#[test]
fn a_riposte_answers_a_blow_the_parry_turned_after_it_is_said() {
    let mut w = world("#####\n#.@k#\n#####\n");
    arm(&mut w, "foil");
    let player = w.player_id();
    let kobold = at(&w, 3, 1);
    // Without the switch, the kobold's blows go unanswered.
    for _ in 0..5 {
        let outcome = w.apply(&Action::Wait);
        assert!(blows(&outcome).iter().all(|&(by, _)| by == kobold));
    }
    switch(&mut w, player, |f| f.riposte = true);
    let parry = crate::combat::Strike::Missed {
        by: Some(crate::combat::Category::Parry),
    };
    let mut ripostes = 0;
    for _ in 0..30 {
        let outcome = w.apply(&Action::Wait);
        let struck = strikes(&outcome);
        let Some(&(by, at, strike)) = struck.first() else {
            continue;
        };
        assert_eq!((by, at), (kobold, player));
        if strike == Some(parry) {
            // One blow back, after the parry: in the log too.
            assert_eq!(blows(&outcome), [(kobold, player), (player, kobold)]);
            let log = said(&w);
            let parried = log.iter().rposition(|m| m.contains("parry"));
            let back = (log.iter())
                .rposition(|m| m.starts_with("You hit the kobold") || m.starts_with("You miss"));
            assert!(parried < back && parried.is_some(), "{log:#?}");
            ripostes += 1;
        } else {
            assert_eq!(blows(&outcome), [(kobold, player)]);
        }
    }
    assert!(ripostes > 0);
    // The ripostes trained the player's skill like any blow.
    let blades = w
        .content()
        .weapon_skills()
        .id("long_blades")
        .expect("a skill");
    assert!(w.state.player().weapon_skills.get(blades) != Default::default());
}

#[test]
fn creatures_riposte_and_ripostes_are_never_answered() {
    // The duellist parries with its foil and answers; the player parries with theirs too.
    let mut w = world("#####\n#.@c#\n#####\n");
    arm(&mut w, "foil");
    let player = w.player_id();
    let duellist = at(&w, 3, 1);
    switch(&mut w, player, |f| f.riposte = true);
    switch(&mut w, duellist, |f| f.riposte = true);
    let mut answered = 0;
    for _ in 0..40 {
        let outcome = w.apply(&Action::Move(Dir::E));
        // The caster never strikes in melee of its own: a riposte at most, never one back.
        match blows(&outcome).as_slice() {
            [(p, d)] => assert_eq!((*p, *d), (player, duellist)),
            [(p, d), (d2, p2)] => {
                assert_eq!((*p, *d, *d2, *p2), (player, duellist, duellist, player));
                answered += 1;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(answered > 0);
}

#[test]
fn the_opportunist_strikes_a_hostile_stepping_away_before_it_steps() {
    let mut w = world("######\n#....#\n#.@k.#\n#....#\n######\n");
    let player = w.player_id();
    let kobold = at(&w, 3, 2);
    switch(&mut w, kobold, |f| f.opportunist = true);
    // A step still beside it gives nothing.
    let outcome = w.apply(&Action::Move(Dir::N));
    let moved = outcome
        .events
        .iter()
        .position(|e| matches!(e, Event::ActorMoved { actor, .. } if *actor == player))
        .expect("the player moved");
    assert!(
        !(outcome.events[..moved].iter())
            .any(|e| matches!(e, Event::Attacked { attacker, .. } if *attacker == kobold))
    );
    // A step out of its reach (it stood and fought): it strikes first.
    assert_eq!(w.state.player().pos, Pos::new(2, 1));
    assert_eq!(w.state.actors[&kobold].pos, Pos::new(3, 2));
    let outcome = w.apply(&Action::Move(Dir::W));
    match outcome.events.first() {
        Some(Event::Attacked {
            attacker, target, ..
        }) => assert_eq!((*attacker, *target), (kobold, player)),
        other => panic!("{other:?}"),
    }
    assert!(
        (outcome.events.iter())
            .any(|e| matches!(e, Event::ActorMoved { actor, .. } if *actor == player))
    );
}

#[test]
fn the_player_strikes_a_hostile_stepping_away_and_says_it_last() {
    let mut w = world("######\n#....#\n#.@k.#\n#....#\n######\n");
    arm(&mut w, "axe");
    let player = w.player_id();
    let kobold = at(&w, 3, 2);
    // The kobold steps east, out of the player's reach: without the switch, nothing.
    let mut events = Vec::new();
    assert!(w.step_away(kobold, Pos::new(4, 2), &mut events));
    assert!(events.is_empty());
    switch(&mut w, player, |f| f.opportunist = true);
    // A step that stays beside the player: nothing.
    assert!(w.step_away(kobold, Pos::new(3, 1), &mut events));
    assert!(events.is_empty());
    // Out of reach: the blow, its words held for the news (said after the turn's reports).
    let before = w.messages().len();
    assert!(w.step_away(kobold, Pos::new(4, 2), &mut events));
    assert!(matches!(
        events.first(),
        Some(Event::Attacked { attacker, target, .. }) if (*attacker, *target) == (player, kobold)
    ));
    assert_eq!(w.messages().len(), before, "nothing said yet");
    assert!(w.news.iter().any(|m| m.contains("kobold")), "{:?}", w.news);
    w.say_news();
    assert!(said(&w).iter().any(|m| m.starts_with("You hit the kobold")));
}

#[test]
fn a_creature_walking_away_in_the_dark_takes_the_opportunists_blow() {
    // The kobold patrols east and back beside the player in the dark: it never sees them.
    let content = content_with("######\n#....#\n#.@p.#\n#....#\n######\n", 0);
    let mut w = World::new(content.clone(), "yard", None).expect("world");
    arm(&mut w, "axe");
    let player = w.player_id();
    let kobold = at(&w, 3, 2);
    let full = w.state.actors[&kobold].max_hp(&content);
    switch(&mut w, player, |f| f.opportunist = true);
    for _ in 0..3 {
        w.apply(&Action::Wait);
    }
    assert!(
        w.state.actors[&kobold].hp < full,
        "struck as it walked away"
    );
    assert!(
        said(&w).iter().any(|m| m.contains("something")),
        "{:#?}",
        said(&w)
    );
}

#[test]
fn switches_draw_nothing_until_their_triggers_fire_and_are_saved_only_when_on() {
    // The caster never fights in melee and never dies, and the player only attacks it: no
    // trigger can fire.
    let map = "#####\n#.@c#\n#####\n";
    let run = |on: bool| {
        let mut w = world(map);
        arm(&mut w, "axe");
        let player = w.player_id();
        if on {
            switch(&mut w, player, |f| {
                *f = FreeBlows {
                    cleave: true,
                    great_cleave: true,
                    riposte: true,
                    opportunist: true,
                };
            });
        }
        for _ in 0..20 {
            w.apply(&Action::Move(Dir::E));
        }
        w
    };
    let (off, mut on) = (run(false), run(true));
    assert_eq!(said(&off), said(&on));
    assert_ne!(off.state_hash(), on.state_hash(), "the switches are state");
    let player = on.player_id();
    switch(&mut on, player, |f| *f = FreeBlows::default());
    assert_eq!(off.state_hash(), on.state_hash());
    // Off, they are not written at all (old saves and hashes stand); on, they round-trip.
    let json = serde_json::to_string(off.state.player()).expect("json");
    assert!(!json.contains("free_blows"), "{json}");
    switch(&mut on, player, |f| f.riposte = true);
    let json = serde_json::to_string(on.state.player()).expect("json");
    assert!(json.contains(r#""free_blows":{"riposte":true}"#), "{json}");
    let back: Actor = serde_json::from_str(&json).expect("an actor");
    assert_eq!(back.free_blows, on.state.player().free_blows);
}
