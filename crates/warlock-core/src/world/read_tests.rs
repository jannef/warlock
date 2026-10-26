//! Reading scrolls (spec §8 "Scrolls"): spell scrolls cast with the traps' rules, aimed by the
//! reader, whose kills are theirs; the scroll of identify; refusals that keep the scroll; deaths
//! by one's own scroll; loot; saves and replays.

use super::*;
use crate::action::{ReadTarget, SpellShape, parse_actions};
use crate::content::{Cast, ContentFiles, Reading, TrapEffect};
use crate::magic::{Affix, LootGrade, Magic, Rarity};
use crate::replay::Replay;

// At peace: nobody fights, so creatures stand where they spawned.
const FACTIONS: &str = "[[faction]]\nid = \"player\"\nname = \"you\"\n\
     [[faction]]\nid = \"vermin\"\nname = \"vermin\"\n\
     [[faction]]\nid = \"kobolds\"\nname = \"kobolds\"\n\
     [[faction]]\nid = \"goblins\"\nname = \"goblins\"\n\
     [[faction]]\nid = \"undead\"\nname = \"undead\"\n\
     [[faction]]\nid = \"marsh\"\nname = \"marsh things\"\n\
     [[faction]]\nid = \"cult\"\nname = \"cultists\"\n\
     [[faction]]\nid = \"duergar\"\nname = \"duergar\"\n\
     [[faction]]\nid = \"fire\"\nname = \"fire things\"\n\
     [[faction]]\nid = \"devil\"\nname = \"devils\"\n\
     [[faction]]\nid = \"yugoloth\"\nname = \"yugoloths\"\n\
     [[faction]]\nid = \"wildlife\"\nname = \"wildlife\"\n\
     [[faction]]\nid = \"peaceful\"\nname = \"peaceful folk\"\n\
     [[faction]]\nid = \"merchants\"\nname = \"merchants\"\n\
     [[faction]]\nid = \"outraged\"\nname = \"outraged\"\nhostile = [\"player\"]\n";

// A training dummy takes a lot and never fights back.
const DUMMY: &str = "[[creature]]\nid = \"dummy\"\nname = \"training dummy\"\nglyph = \"D\"\n\
     fg = \"#806040\"\nweight = 30000\nfaction = \"wildlife\"\nbase_hp = 1000\n";

/// The pack every test starts with: a, three scrolls of fireball; b, two of lightning; c, one of
/// frost; d, two of identify.
const PACK: &str = "carry = [\"scroll_of_fireball*3\", \"scroll_of_lightning*2\", \"scroll_of_frost\", \
     \"scroll_of_identify*2\"]";

/// A lit hall of `map` (`@` the player, `D` a dummy, `r` a giant rat), the player with `hp`
/// hit points beyond their Toughness.
fn world_with(map: &str, hp: u16) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/creatures/dummy.toml", DUMMY);
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"D\" = {{ terrain = \"floor\", creature = \"dummy\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = {hp}\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

fn world(map: &str) -> World {
    world_with(map, 1000)
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn lost(w: &World, id: ActorId) -> i32 {
    let a = &w.state.actors[&id];
    a.max_hp(&w.content) - a.hp
}

fn at(w: &World, p: Pos) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == p)
        .map(|a| a.id)
        .expect("someone stands there")
}

/// How many of `object` the player carries.
fn count(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

fn spell(out: &Outcome) -> (&str, Pos, &SpellShape) {
    (out.events.iter())
        .find_map(|e| match e {
            Event::Spell {
                effect,
                origin,
                shape,
                ..
            } => Some((effect.as_str(), *origin, shape)),
            _ => None,
        })
        .expect("a spell")
}

#[test]
fn builtin_scrolls_cast_what_the_traps_do() {
    let content = Content::builtin().expect("builtin content");
    let reading = |id: &str| {
        let o = content.object_id(id).expect("scroll");
        content.object(o).read.clone().expect("readable")
    };
    let trap = |id: &str| {
        content
            .trap(content.trap_id(id).expect("trap"))
            .effect
            .clone()
    };
    let cast = |id: &str| reading(id).spell().expect("a spell").cast.clone();
    match (cast("scroll_of_fireball"), trap("fireball")) {
        (Cast::Burst(scroll), TrapEffect::Burst(trap)) => assert_eq!(scroll, trap),
        other => panic!("{other:?}"),
    }
    match (cast("scroll_of_frost"), trap("frost")) {
        (Cast::Burst(scroll), TrapEffect::Burst(trap)) => assert_eq!(scroll, trap),
        other => panic!("{other:?}"),
    }
    match (cast("scroll_of_lightning"), trap("lightning_bolt")) {
        (Cast::Bolt(scroll), TrapEffect::Bolt(trap)) => assert_eq!(scroll, trap),
        other => panic!("{other:?}"),
    }
    assert_eq!(reading("scroll_of_identify"), Reading::Identify);
    let light = content.object_id("scroll_of_light").expect("scroll");
    assert!(
        content.object(light).read.is_none(),
        "nothing to read on it yet"
    );
}

#[test]
fn bad_readings_are_reported() {
    let bad = |read: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/objects/scroll_of_folly.toml",
            &format!(
                "[[object]]\nid = \"scroll_of_folly\"\nname = \"scroll of folly\"\nglyph = \"?\"\n\
                 fg = \"#ffffff\"\nclass = \"scrolls\"\nmaterials = [\"paper\"]\nweight = 50\n\
                 value = 10\nstackable = true\n[object.read]\n{read}\n"
            ),
        );
        Content::load(&files).expect_err("refused").to_string()
    };
    let burst = "kind = \"burst\"\nspell = \"fireball\"\nnoun = \"fire\"\ndamage = \"1d6\"\n\
                 radius = 2";
    assert!(bad(burst).contains("`range`"), "{}", bad(burst));
    let err = bad("kind = \"identify\"\nradius = 2");
    assert!(err.contains("`radius` is not for"), "{err}");
    let err =
        bad("kind = \"bolt\"\nspell = \"fireball\"\nnoun = \"fire\"\ndamage = \"1d6\"\nlength = 5");
    assert!(err.contains("bolt effect"), "{err}");
    let err = bad(&format!("{burst}\nrange = 5\nheat = true\nfrost = 10"));
    assert!(err.contains("not both"), "{err}");
}

#[test]
fn a_fireball_flies_to_the_cell_aimed_at_and_bursts_like_the_trap() {
    // Two dummies beside the cell aimed at, in the inner radius and at its edge.
    let mut w = world("############\n#@.......D.#\n#........D.#\n#..........#\n############\n");
    let (near, edge) = (at(&w, Pos::new(9, 1)), at(&w, Pos::new(9, 2)));
    let out = run(&mut w, "read:a:8,1");
    assert!(out[0].took_turn, "reading takes a turn");
    assert_eq!(
        out[0].elapsed,
        Duration::seconds(6),
        "the `read` action's time"
    );
    let (effect, origin, shape) = spell(&out[0]);
    assert_eq!(effect, "fireball");
    assert_eq!(origin, Pos::new(1, 1), "it flies from the reader");
    assert_eq!(
        *shape,
        SpellShape::Burst {
            center: Pos::new(8, 1),
            radius: 2
        }
    );
    let roll = lost(&w, near);
    assert!((3..=18).contains(&roll), "{roll}");
    // (9, 2) is at squared distance 2 from the center: inner (radius 1 reaches 2), the full roll.
    assert_eq!(lost(&w, edge), roll);
    assert_eq!(lost(&w, w.state.player), 0, "out of its reach");
    assert_eq!(count(&w, "scroll_of_fireball"), 2, "used up");
    let said = texts(&w);
    assert!(
        said.contains(&"It is a scroll of fireball.".to_string()),
        "read unknown, then named: {said:?}"
    );
    // Both dummies: the same words, said once with a count.
    let burns = "The fireball burns the training dummy. (×2)";
    assert!(said.iter().any(|t| t == burns), "{said:?}");
}

#[test]
fn a_burst_goes_off_on_the_first_creature_in_its_way_or_before_a_wall() {
    let mut w = world("##########\n#@..D....#\n#......#.#\n##########\n");
    let out = run(&mut w, "read:a:7,1");
    let (_, _, shape) = spell(&out[0]);
    assert!(matches!(shape, SpellShape::Burst { center, .. } if *center == Pos::new(4, 1)));
    // Aimed past a wall: it bursts on the last open cell before it.
    let out = run(&mut w, "read:a:8,3");
    let (_, _, shape) = spell(&out[0]);
    let SpellShape::Burst { center, .. } = shape else {
        panic!("{shape:?}")
    };
    assert_eq!(
        *center,
        Pos::new(6, 2),
        "before the wall its line runs into"
    );
}

#[test]
fn a_fireball_aimed_close_catches_its_reader_and_may_kill_them() {
    let mut w = world("########\n#@.....#\n########\n");
    run(&mut w, "read:a:2,1");
    assert!(lost(&w, w.state.player) > 0, "it caught the reader");
    // A frail reader dies of it, and the scroll is what killed them.
    let mut w = world_with("########\n#@.....#\n########\n", 0);
    let max = w.state.player().max_hp(&w.content);
    w.player_mut().hp = 1;
    assert!(max > 1);
    run(&mut w, "read:a:2,1");
    let death = w.state.death.clone().expect("dead");
    let fireball = w.content.object_id("scroll_of_fireball").expect("scroll");
    assert_eq!(death.cause, Some(Cause::Scroll(fireball)));
    assert_eq!(death.by, None);
    assert_eq!(
        death.cause.expect("cause").epitaph(&w.content),
        "Killed by their own scroll of fireball"
    );
    assert_eq!(texts(&w).last().map(String::as_str), Some("You die..."));
}

#[test]
fn lightning_runs_through_everyone_on_its_line_but_the_reader() {
    let mut w = world("###############\n#@.D.D......D.#\n###############\n");
    let (a, b, far) = (
        at(&w, Pos::new(3, 1)),
        at(&w, Pos::new(5, 1)),
        at(&w, Pos::new(12, 1)),
    );
    let out = run(&mut w, "read:b:3,1");
    let (effect, origin, shape) = spell(&out[0]);
    assert_eq!((effect, origin), ("lightning_bolt", Pos::new(1, 1)));
    let SpellShape::Bolt { path } = shape else {
        panic!("{shape:?}")
    };
    // On past the cell aimed at, 12 cells, to (13, 1) before the wall.
    assert_eq!(path.first(), Some(&Pos::new(2, 1)));
    assert_eq!(path.last(), Some(&Pos::new(13, 1)));
    let roll = lost(&w, a);
    assert!((4..=24).contains(&roll));
    assert_eq!((lost(&w, b), lost(&w, far)), (roll, roll));
    assert_eq!(lost(&w, w.state.player), 0);
    assert_eq!(count(&w, "scroll_of_lightning"), 1);
}

#[test]
fn the_readers_kills_are_theirs_and_earn_experience() {
    let mut w = world("##########\n#@...r...#\n##########\n");
    let rat = at(&w, Pos::new(5, 1));
    w.state.actors.get_mut(&rat).expect("rat").hp = 1;
    assert_eq!(w.state.experience, 0);
    run(&mut w, "read:a:5,1");
    assert!(!w.state.actors.contains_key(&rat), "the fireball killed it");
    assert!(w.state.experience > 0, "the rat's experience");
    assert!(texts(&w).contains(&"The giant rat dies.".to_string()));
}

#[test]
fn frost_freezes_water_and_bites() {
    let mut w = world("##########\n#@...D...#\n#...~~~..#\n##########\n");
    let dummy = at(&w, Pos::new(5, 1));
    run(&mut w, "read:c:5,1");
    assert!((2..=12).contains(&lost(&w, dummy)));
    let level = w.state.current();
    assert!(
        (4..=6).all(|x| level.ice_at(Pos::new(x, 2)).is_some()),
        "the water it reached froze"
    );
}

#[test]
fn refused_readings_take_no_time_and_keep_the_scroll() {
    let mut w = world("##########\n#@.......#\n##########\n");
    // Fireballs known (spec §8 "Scrolls"): identify has nothing to learn about them.
    let fire = w.content.object_id("scroll_of_fireball").expect("fireball");
    w.state.known_scrolls.insert(fire);
    let refusals = [
        ("read:a", "Read it at what?"),
        ("read:a:1,1", "You cannot aim at yourself."),
        ("read:a:30,1", "That is out of range."),
        ("read:d", "Read it on what?"),
        ("read:d:a", "You already know what"),
        ("read:z", "You carry no such thing."),
    ];
    for (action, said) in refusals {
        let out = run(&mut w, action);
        assert!(!out[0].took_turn, "{action}");
        let last = texts(&w).last().cloned().unwrap_or_default();
        assert!(last.starts_with(said), "{action}: {last}");
    }
    assert_eq!(count(&w, "scroll_of_fireball"), 3);
    assert_eq!(count(&w, "scroll_of_identify"), 2);
    // A stack that is no scroll is not read.
    let apple = Stack::one(w.content.object_id("apple").expect("apple"));
    w.player_mut().inventory.push(apple);
    run(&mut w, "read:e");
    assert_eq!(
        texts(&w).last().map(String::as_str),
        Some("You cannot read an apple.")
    );
}

#[test]
fn the_scroll_of_identify_identifies_an_item_and_is_used_up() {
    let mut w = world("######\n#@...#\n######\n");
    // Known by sight here (unknown scrolls: scroll_label_tests.rs).
    let identify = w.content.object_id("scroll_of_identify").expect("identify");
    w.state.known_scrolls.insert(identify);
    let ring = || {
        let id = w.content.object_id("silver_ring").expect("ring");
        let magic = Magic {
            rarity: Rarity::Magic,
            level: 1,
            affixes: vec![Affix {
                id: "of_the_mule".into(),
                tier: 1,
                value: 1,
            }],
            name: None,
            identified: false,
            carried: 0,
            unique: None,
        };
        Stack::with_magic(id, 1, magic)
    };
    let (first, second) = (ring(), ring());
    w.player_mut().inventory.push(first);
    let out = run(&mut w, "read:d:e");
    assert!(out[0].took_turn);
    assert!(w.state.player().inventory[4].is_identified());
    let said = texts(&w);
    assert!(said.contains(&"You read the scroll of identify.".to_string()));
    let found = said.last().cloned().unwrap_or_default();
    assert!(
        found.starts_with("This is a silver ring of the Mule"),
        "{found}"
    );
    let out = run(&mut w, "read:d:e");
    assert!(!out[0].took_turn, "nothing left to learn");
    // The last scroll, read on a second ring (f): it is identified before the scroll goes, which
    // moves the ring up to e.
    w.player_mut().inventory.push(second);
    let out = run(&mut w, "read:d:f");
    assert!(out[0].took_turn);
    assert_eq!(count(&w, "scroll_of_identify"), 0);
    let pack = &w.state.player().inventory;
    assert_eq!(pack.len(), 5);
    assert!(pack.iter().all(Stack::is_identified));
}

#[test]
fn reading_survives_saves_and_replays() {
    let mut w = world("############\n#@.....D...#\n#..........#\n############\n");
    let content = w.content.clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("read:a:7,1 wait read:b:9,2 read:c:5,2").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let text = parse_actions("read:a:7,1 read:d:e read:b").expect("parse");
    assert_eq!(
        text[1],
        Action::Read {
            item: 3,
            target: Some(ReadTarget::Item(4))
        }
    );
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn scrolls_are_loot_but_never_magic() {
    let content = Content::builtin().expect("content");
    let scroll = |s: &Stack| content.object(s.object).read.is_some();
    let mut found = 0;
    for i in 0..400 {
        let mut rng = Rng::stream(i, "loot-test");
        let stack = crate::magic::loot::roll(&content, &mut rng, 3, LootGrade::Any).expect("loot");
        if scroll(&stack) {
            found += 1;
            assert_eq!(stack.rarity(), Rarity::Normal);
        }
        let mut rng = Rng::stream(i, "loot-test");
        let magic = crate::magic::loot::roll(&content, &mut rng, 3, LootGrade::Magic);
        assert!(
            magic.is_some_and(|s| !scroll(&s)),
            "magic loot is never a scroll"
        );
    }
    assert!(found > 0, "scrolls turn up in loot");
}
