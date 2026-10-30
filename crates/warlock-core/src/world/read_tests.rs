//! Reading scrolls (spec §8 "Scrolls"): spell scrolls cast with the traps' rules, aimed by the
//! reader, whose kills are theirs; the scroll of identify; refusals that keep the scroll; deaths
//! by one's own scroll; loot; saves and replays.

use super::fixture_factions::peaceful_factions;
use super::*;
use crate::action::{ReadTarget, SpellShape, parse_actions};
use crate::content::{ContentFiles, Reading, SpellPurpose};
use crate::magic::{Affix, LootGrade, Magic, Rarity};
use crate::replay::Replay;

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
    hall(map, hp, 255)
}

/// [`world_with`] under white `ambient` light (0: dark).
fn hall(map: &str, hp: u16, ambient: u16) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/factions.toml", &peaceful_factions());
    files.insert("data/creatures/dummy.toml", DUMMY);
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n[legend]\n\
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

/// The three older scrolls are spell scrolls at a rung of their spells' ladders (the spell
/// scaling design §7.4): fireball 1 (3d6), lightning 2 (4d6), frost 0 (2d6, the base).
#[test]
fn builtin_scrolls_cast_their_spells_at_their_rungs() {
    let content = Content::builtin().expect("builtin content");
    let reading = |id: &str| {
        let o = content.object_id(id).expect("scroll");
        content.object(o).read.clone().expect("readable")
    };
    for (scroll, spell, rungs, roll) in [
        ("scroll_of_fireball", "fireball", 1, "3d6"),
        ("scroll_of_lightning", "lightning_bolt", 2, "4d6"),
        ("scroll_of_frost", "frost", 0, "2d6"),
    ] {
        let Reading::Cast(cast) = reading(scroll);
        let def = content.spell(cast.spell);
        assert_eq!((def.id.as_str(), cast.rungs), (spell, rungs), "{scroll}");
        let at = def.roll_at(usize::from(cast.rungs)).expect("a roll");
        assert_eq!(at.to_string(), roll, "{scroll}");
        assert_eq!(
            reading(scroll).teaches(),
            Some(cast.spell),
            "{scroll} teaches its spell"
        );
    }
    // The scroll of identify casts Identify on a stack the reader carries (#592).
    let Reading::Cast(scroll) = reading("scroll_of_identify");
    assert_eq!(content.spell(scroll.spell).id, "identify");
    assert!(reading("scroll_of_identify").on_item());
    assert!(!reading("scroll_of_light").on_item());
    // The scroll of light casts the Light spell (#589: it had no reading at all).
    let Reading::Cast(scroll) = reading("scroll_of_light");
    assert_eq!(content.spell(scroll.spell).id, "light");
}

/// Every scroll of the content (#589): an object of the scrolls class named `scroll_of_*` has a
/// reading, and a reader who aims it (at a creature's cell, or on another stack) reads it: a
/// turn spent and one scroll used up. Read without the target it needs, it says why. Every one
/// is a real spell (#592): it teaches a spell of `data/spells/`, so it may be studied.
#[test]
fn every_builtin_scroll_can_be_read() {
    let mut checked = 0;
    // The dummy beside the reader: within every scroll's reach, touch spells' too.
    let mut base = world("############\n#@D........#\n#..........#\n############\n");
    let content = base.content.clone();
    // Wounded, so a healing scroll has something to heal.
    let dummy = at(&base, Pos::new(2, 1));
    base.state.actors.get_mut(&dummy).expect("dummy").hp -= 10;
    for object in (0..content.object_defs().len()).map(|i| ObjectId(i as u16)) {
        let def = content.object(object);
        if content.item_class(def.class).id != "scrolls" || !def.id.starts_with("scroll_of_") {
            continue;
        }
        let reading = (def.read.as_ref()).unwrap_or_else(|| panic!("{} has no reading", def.id));
        let taught = (reading.teaches()).unwrap_or_else(|| panic!("{} teaches nothing", def.id));
        assert!(
            content.spell_of(taught).is_some(),
            "{}: no such spell",
            def.id
        );
        // The fixture's refusals, each with its reason: under a roof, a spell that needs the
        // open sky; at the peaceful dummy, a hex; on a hale reader, a cure.
        let refusal = match reading {
            Reading::Cast(scroll) => {
                let spell = content.spell(scroll.spell);
                match spell.kind {
                    _ if spell.open_sky => Some("open sky"),
                    SpellPurpose::Hex => Some("means you no harm"),
                    SpellPurpose::Cleanse => Some("nothing to cure"),
                    _ => None,
                }
            }
        };
        let mut w = base.clone();
        let item = w.state.player().inventory.len();
        w.player_mut().inventory.push(Stack::new(object, 2));
        let letter = |i: usize| crate::action::position_text(i);
        // Without a target: read at once (a spell on the reader), or refused with a reason.
        let bare = w.clone().apply(&Action::Read { item, target: None });
        if !bare.took_turn {
            let mut refused = w.clone();
            refused.apply(&Action::Read { item, target: None });
            assert!(
                texts(&refused).len() > texts(&w).len(),
                "{}: refused without a word",
                def.id
            );
        }
        let target = match reading {
            _ if reading.on_item() => format!(":{}", letter(0)),
            _ if reading.range().is_some() => ":2,1".to_string(),
            _ => String::new(),
        };
        let action = format!("read:{}{target}", letter(item));
        let had = count(&w, &def.id);
        let out = run(&mut w, &action);
        let said = texts(&w).join(" | ");
        if !out[0].took_turn
            && let Some(why) = refusal
        {
            assert!(said.contains(why), "{action}: {said}");
            checked += 1;
            continue;
        }
        assert!(out[0].took_turn, "{action} ({}): not read: {said}", def.id);
        assert_eq!(count(&w, &def.id), had - 1, "{}: one used up", def.id);
        checked += 1;
    }
    assert!(checked > 20, "only {checked} scrolls");
}

/// The scroll of light (#589): read in the dark, its reader is Lit and sheds a light that goes
/// where they go, until the status wears off.
#[test]
fn the_scroll_of_light_lights_its_reader() {
    let mut w = hall("##########\n#@.......#\n##########\n", 1000, 0);
    let light = w.content.object_id("scroll_of_light").expect("scroll");
    w.player_mut().inventory.push(Stack::one(light));
    let lit = w.content.status_id("lit").expect("lit");
    let seen = |w: &World, x: i32| w.perception().light(Pos::new(x, 1));
    assert_eq!(seen(&w, 3), None, "dark beyond the reader's cell");
    let out = run(&mut w, "read:e");
    assert!(out[0].took_turn);
    assert_eq!(count(&w, "scroll_of_light"), 0);
    assert!(w.state.player().effects.has(lit));
    assert!(
        texts(&w)
            .iter()
            .any(|t| t == "A pale light gathers about you.")
    );
    assert!(
        seen(&w, 3).is_some_and(|l| l.brightness() > 0),
        "lit around"
    );
    run(&mut w, "move:E*3");
    assert!(
        seen(&w, 7).is_some_and(|l| l.brightness() > 0),
        "it goes along"
    );
    // Half an hour a degree, three at most: by two hours it is gone, and so is the light.
    run(&mut w, "rest:2h");
    assert!(!w.state.player().effects.has(lit));
    assert_eq!(seen(&w, 8), None, "dark again");
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
    // The old burst and bolt readings are gone (the spell scaling design §7.4), refused naming
    // what replaced them.
    let err = bad("kind = \"burst\"\nspell = \"fireball\"");
    assert!(
        err.contains("`burst` and `bolt` readings are gone"),
        "{err}"
    );
    let err = bad("kind = \"identify\"");
    assert!(err.contains("unknown variant `identify`"), "{err}");
    let err = bad("kind = \"spell\"\nspell = \"identify\"\nnoun = \"x\"");
    assert!(err.contains("unknown field `noun`"), "{err}");
    let err = bad("kind = \"spell\"");
    assert!(err.contains("needs its `spell`"), "{err}");
    // A scroll's rungs within its spell's ladder: Fireball's has four, Identify none.
    assert!(bad("kind = \"spell\"\nspell = \"fireball\"\nrungs = 5").contains("`rungs` is 0 to 4"));
    assert!(bad("kind = \"spell\"\nspell = \"identify\"\nrungs = 1").contains("`rungs` is 0 to 0"));
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
    let mut w = world("###############\n#@.D.D...D....#\n###############\n");
    let (a, b, far) = (
        at(&w, Pos::new(3, 1)),
        at(&w, Pos::new(5, 1)),
        at(&w, Pos::new(9, 1)),
    );
    let out = run(&mut w, "read:b:3,1");
    let (effect, origin, shape) = spell(&out[0]);
    assert_eq!((effect, origin), ("lightning_bolt", Pos::new(1, 1)));
    let SpellShape::Bolt { path } = shape else {
        panic!("{shape:?}")
    };
    // On past the cell aimed at, as far as the reader casts Lightning bolt: its base length,
    // 8, to (9, 1).
    assert_eq!(path.first(), Some(&Pos::new(2, 1)));
    assert_eq!(path.last(), Some(&Pos::new(9, 1)));
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
        ("read:a", "Cast it where?"),
        ("read:a:1,1", "You cannot cast at yourself."),
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
