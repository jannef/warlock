//! Flasks (spec §8 "Potions": "Flasks"; `flask.rs`): a potion thrown shatters where it breaks,
//! a flask casting its spell there; who stands in the way catches it; the splash's kills are the
//! thrower's; an unknown flask is learned only when its breaking is seen; refusals; saves and
//! replays; the content checks.

use super::fixture_factions::make_peace;
use super::*;
use crate::action::{MissileEnd, SpellShape, parse_actions};
use crate::content::{ContentFiles, ZoneKind};
use crate::replay::Replay;

// A training dummy takes a lot and never fights back.
const DUMMY: &str = "[[creature]]\nid = \"dummy\"\nname = \"training dummy\"\nglyph = \"D\"\n\
     fg = \"#806040\"\nweight = 30000\nfaction = \"wildlife\"\nbase_hp = 1000\n";

/// A fixture flask of `spell` (`id`, coloured `colour` in `fg`), its drink a harm.
fn flask(id: &str, spell: &str, (colour, fg): (&str, &str)) -> String {
    format!(
        "[[object]]\nid = \"{id}\"\nname = \"{id}\"\nglyph = \"!\"\nfg = \"{fg}\"\n\
         class = \"potions\"\nmaterials = [\"glass\"]\nweight = 250\nvalue = 10\n\
         stackable = true\n\n[object.drink]\nkind = \"harm\"\ndice = \"1\"\n\
         status = \"poisoned\"\ncolour = \"{colour}\"\n\n[object.throw]\nspell = \"{spell}\"\n"
    )
}

/// The pack every test starts with: a, three fire flasks (Fireball); b, two sleeping draughts;
/// c, two weakness flasks (Hex of weakness, a ray); d, a slick flask (Slick, a zone); e, a
/// potion of healing.
const PACK: &str = "carry = [\"fire_flask*3\", \"sleeping_draught*2\", \"weakness_flask*2\", \
     \"slick_flask\", \"potion_of_healing\"]";

/// The test content: the fixture flasks (three greater potions out, so the labels go round),
/// and `more` files on top.
fn files_with(more: &[(&str, &str)]) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    make_peace(&mut files);
    for gone in ["wit", "will", "presence"] {
        files.remove(&format!("data/objects/greater_potion_of_{gone}.toml"));
    }
    let fire = flask("fire_flask", "fireball", ("scarlet", "#e04020"));
    let weakness = flask("weakness_flask", "hex_of_weakness", ("indigo", "#4830a0"));
    let slick = flask("slick_flask", "slick", ("colourless", "#c0c8cc"));
    files.insert("data/objects/fire_flask.toml", &fire);
    files.insert("data/objects/weakness_flask.toml", &weakness);
    files.insert("data/objects/slick_flask.toml", &slick);
    files.insert("data/creatures/dummy.toml", DUMMY);
    for (path, text) in more {
        files.insert(path, text);
    }
    files
}

/// A hall of `map` (`@` the player, `D` a dummy, `r` a giant rat) under white `ambient` light,
/// the player with `hp` hit points beyond their Toughness.
fn hall(map: &str, hp: u16, ambient: u16) -> World {
    let mut files = files_with(&[]);
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
    hall(map, 1000, 255)
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

fn count(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

fn suffers(w: &World, id: ActorId, status: &str) -> Option<u16> {
    let status = w.content.status_id(status).expect("status");
    w.state.actors[&id].effects.get(status).map(|e| e.severity)
}

fn know(w: &mut World, object: &str) {
    let id = w.content.object_id(object).expect("object");
    w.state.known_scrolls.insert(id);
}

/// The missile's path and end, and where the splash's effect went off, from one throw's events.
fn flight(out: &Outcome) -> (Vec<Pos>, MissileEnd, Option<Pos>) {
    let (path, end) = (out.events.iter())
        .find_map(|e| match e {
            Event::Missile { path, end, .. } => Some((path.clone(), *end)),
            _ => None,
        })
        .expect("a missile");
    let center = out.events.iter().find_map(|e| match e {
        Event::Spell {
            shape: SpellShape::Burst { center, .. },
            ..
        } => Some(*center),
        _ => None,
    });
    (path, end, center)
}

#[test]
fn the_sleeping_draught_is_a_flask_of_sleep() {
    let content = Content::builtin().expect("content");
    let id = content.object_id("sleeping_draught").expect("draught");
    let def = content.object(id);
    let flask = def.flask.as_ref().expect("a flask");
    assert_eq!(content.spell(flask.spell).id, "sleep");
    assert_eq!((flask.rungs, flask.range), (0, 6));
    let splash = flask.splash(&content);
    assert_eq!((splash.radius, splash.degrees), (Some(1), Some(2)));
    assert_eq!(def.throw_range(), Some(6));
    assert!(def.drink.is_some(), "a flask is still a potion");
}

#[test]
fn the_flasks_of_smoke_and_brilliance_and_the_vial_of_acid_throw_and_drink() {
    let mut w = world("##############\n#@.....D.....#\n##############\n");
    let dummy = at(&w, Pos::new(7, 1));
    let content = w.content.clone();
    for (id, spell) in [
        ("vial_of_acid", "acid_splash"),
        ("flask_of_brilliance", "flare"),
        ("flask_of_smoke", "fume"),
        ("flask_of_brilliance", "flare"),
    ] {
        let object = content.object_id(id).expect(id);
        let flask = content.object(object).flask.as_ref().expect("a flask");
        assert_eq!(content.spell(flask.spell).id, spell);
        w.player_mut().inventory.push(Stack::one(object));
    }
    // Acid, 2d4 at rung 0, and it keeps eating.
    run(&mut w, "throw:f:7,1");
    assert!((2..=8).contains(&lost(&w, dummy)), "{}", lost(&w, dummy));
    // Then (each throw takes its stack away) a flash that stuns it.
    run(&mut w, "throw:f:7,1");
    assert!(texts(&w).contains(&"The training dummy is stunned.".to_string()));
    // Smoke over it, and the player no longer sees it.
    run(&mut w, "throw:f:7,1");
    let level = w.state.current();
    let smoke = level
        .zone_at(Pos::new(7, 1), w.state.time)
        .map(|z| z.kind());
    assert_eq!(smoke, Some(ZoneKind::Smoke));
    assert!(!w.perception.sees_actors_at(Pos::new(7, 1)));
    assert!(texts(&w).contains(&"Smoke billows from the shards.".to_string()));
    // Drunk, the flask of brilliance lights its drinker up.
    run(&mut w, "eat:f");
    assert_eq!(suffers(&w, w.state.player, "lit"), Some(1));
}

#[test]
fn a_flask_flies_shatters_on_whom_it_is_thrown_at_and_casts_its_spell_there() {
    let mut w = world("############\n#@....D....#\n############\n");
    know(&mut w, "fire_flask");
    let dummy = at(&w, Pos::new(6, 1));
    let out = run(&mut w, "throw:a:6,1");
    assert!(out[0].took_turn);
    assert_eq!(count(&w, "fire_flask"), 2, "one thrown, never kept");
    let (path, end, center) = flight(&out[0]);
    assert_eq!(path.last(), Some(&Pos::new(6, 1)));
    assert_eq!(end, MissileEnd::Struck);
    assert_eq!(
        center,
        Some(Pos::new(6, 1)),
        "the fireball goes off on the dummy"
    );
    assert!((4..=24).contains(&lost(&w, dummy)), "4d6 fire at rung 0");
    // Nothing lands: the flask broke.
    let level = w.state.current();
    assert!((1..11).all(|x| level.objects_at(Pos::new(x, 1)).is_empty()));
    assert!(texts(&w).contains(&"You throw the fire_flask.".to_string()));
}

#[test]
fn the_first_creature_in_the_way_catches_it_and_its_kills_are_the_throwers() {
    let mut w = world("############\n#@.r....D..#\n############\n");
    let rat = at(&w, Pos::new(3, 1));
    let dummy = at(&w, Pos::new(8, 1));
    w.state.actors.get_mut(&rat).expect("rat").hp = 1;
    assert_eq!(w.state.experience, 0);
    let out = run(&mut w, "throw:a:7,1");
    let (path, end, center) = flight(&out[0]);
    assert_eq!(path, vec![Pos::new(2, 1), Pos::new(3, 1)]);
    assert_eq!((end, center), (MissileEnd::Struck, Some(Pos::new(3, 1))));
    assert!(!w.state.actors.contains_key(&rat), "the splash killed it");
    assert!(
        w.state.experience > 0,
        "the rat's experience is the thrower's"
    );
    assert!(texts(&w).contains(&"The giant rat dies.".to_string()));
    assert_eq!(lost(&w, dummy), 0, "out of the burst");
}

#[test]
fn an_unknown_flask_is_learned_when_its_breaking_is_seen() {
    let mut w = world("############\n#@....D....#\n############\n");
    let dummy = at(&w, Pos::new(6, 1));
    let draught = w.content.object_id("sleeping_draught").expect("draught");
    let unknown = w.lore().title(&Stack::one(draught));
    assert!(unknown.ends_with("pink potion"), "{unknown}");
    run(&mut w, "throw:b:6,1");
    assert_eq!(suffers(&w, dummy, "drowsy"), Some(2));
    assert!(w.state.known_scrolls.contains(&draught));
    let said = texts(&w);
    let i = |line: &str| said.iter().position(|l| l == line).expect(line);
    let throw = i(&format!("You throw the {unknown}."));
    let words = i("A sweet, heavy mist puffs from the shards.");
    let learned = i("It was a sleeping draught.");
    let dozes = i("The training dummy is drowsy.");
    assert!(
        throw < words && words < learned && learned < dozes,
        "{said:?}"
    );
    assert_eq!(
        suffers(&w, w.state.player, "drowsy"),
        None,
        "the thrower is spared"
    );
}

#[test]
fn a_flask_breaking_unseen_is_heard_and_teaches_nothing() {
    let mut w = hall("############\n#@..D......#\n############\n", 1000, 0);
    let dummy = at(&w, Pos::new(4, 1));
    let draught = w.content.object_id("sleeping_draught").expect("draught");
    assert!(!w.perception.is_visible(Pos::new(4, 1)), "dark");
    let out = run(&mut w, "throw:b:4,1");
    assert!(out[0].took_turn);
    assert_eq!(
        suffers(&w, dummy, "drowsy"),
        Some(2),
        "it works all the same"
    );
    assert!(!w.state.known_scrolls.contains(&draught));
    assert!(!texts(&w).iter().any(|l| l.starts_with("It was")));
    assert!(texts(&w).contains(&"You hear a crash of glass.".to_string()));
}

#[test]
fn a_hex_flask_of_a_ray_falls_on_whoever_stands_where_it_breaks() {
    let mut w = world("############\n#@...r.....#\n############\n");
    know(&mut w, "weakness_flask");
    let rat = at(&w, Pos::new(5, 1));
    run(&mut w, "throw:c:5,1");
    assert!(suffers(&w, rat, "weakened").is_some());
    // On no one: it shatters, and nothing more.
    let out = run(&mut w, "throw:c:3,1");
    assert!(out[0].took_turn);
    assert_eq!(count(&w, "weakness_flask"), 0);
}

#[test]
fn a_zone_flask_lays_its_zone_by_the_thrower() {
    let mut w = world("############\n#@.........#\n############\n");
    run(&mut w, "throw:d:5,1");
    let zones = w.zones();
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].kind(), ZoneKind::Slick);
    assert_eq!(zones[0].by(), w.state.player);
    assert!(zones[0].cells().any(|p| p == Pos::new(5, 1)));
}

#[test]
fn a_potion_without_a_throw_table_spills_uselessly() {
    let mut w = world("############\n#@..D......#\n############\n");
    let dummy = at(&w, Pos::new(4, 1));
    let healing = w.content.object_id("potion_of_healing").expect("healing");
    let out = run(&mut w, "throw:e:4,1");
    assert!(out[0].took_turn);
    assert_eq!(count(&w, "potion_of_healing"), 0);
    assert_eq!(lost(&w, dummy), 0);
    assert!(!w.state.known_scrolls.contains(&healing), "nothing learned");
    assert!(texts(&w).contains(&"The potion shatters; the draught soaks away.".to_string()));
    assert!(
        !out[0]
            .events
            .iter()
            .any(|e| matches!(e, Event::Spell { .. }))
    );
}

#[test]
fn refused_throws_take_no_time_and_keep_the_flask() {
    let mut w = world(
        "##########################\n#@.......................#\n##########################\n",
    );
    let refusals = [
        ("throw:a:1,1", "You cannot throw at yourself."),
        ("throw:a:12,1", "That is out of range."),
        // A potion with no throw table flies as far as an improvised weapon.
        ("throw:e:6,1", "That is out of range."),
        ("throw:z:3,1", "You carry no such thing."),
    ];
    for (action, said) in refusals {
        let out = run(&mut w, action);
        assert!(!out[0].took_turn, "{action}");
        let last = texts(&w).last().cloned().unwrap_or_default();
        assert!(last.starts_with(said), "{action}: {last}");
    }
    assert_eq!(count(&w, "fire_flask"), 3);
    assert_eq!(count(&w, "potion_of_healing"), 1);
}

#[test]
fn a_flask_thrown_too_close_catches_its_thrower_and_may_kill_them() {
    let mut w = hall("############\n#@.........#\n############\n", 0, 255);
    w.player_mut().hp = 1;
    run(&mut w, "throw:a:2,1");
    let death = w.state.death.as_ref().expect("dead");
    assert_eq!(
        death.cause,
        Some(Cause::Flask(
            w.content.object_id("fire_flask").expect("flask")
        ))
    );
}

#[test]
fn throwing_flasks_survives_saves_and_replays() {
    let mut w = world("############\n#@....D....#\n#..r.......#\n############\n");
    let content = w.content.clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("throw:a:6,1 wait throw:b:3,2 throw:d:8,2").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn content_refuses_a_flask_of_no_spell_and_nothing_else() {
    let flask = |throw: &str| {
        format!(
            "[[object]]\nid = \"odd_flask\"\nname = \"odd flask\"\nglyph = \"!\"\n\
             fg = \"#5a4030\"\nclass = \"potions\"\nmaterials = [\"glass\"]\n\
             weight = 250\nvalue = 10\nstackable = true\n\n[object.throw]\n{throw}\n"
        )
    };
    let bad = flask("spell = \"no_such\"");
    let files = files_with(&[("data/objects/odd_flask.toml", &bad)]);
    let err = Content::load(&files).expect_err("no spell").to_string();
    assert!(err.contains("no spell `no_such`"), "{err}");
    // The data author is trusted (#1205): any spell, rung, and range load.
    let odd = flask("spell = \"haste\"\nrungs = 9\nrange = 1");
    let files = files_with(&[("data/objects/odd_flask.toml", &odd)]);
    Content::load(&files).expect("loads");
}
