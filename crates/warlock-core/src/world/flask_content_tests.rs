//! The flasks on existing spells (spec §8 "Potions": "Flasks"; the potions wave 2 design §4.4,
//! slice S4): each one's throw and drink once, from the game's own data. The rules are
//! `flask.rs`'s (`flask_tests.rs`); these check that each file casts what it says and harms its
//! drinker as the design's §6 table says.

use super::fixture_factions::make_peace;
use super::*;
use crate::content::{ContentFiles, ZoneKind};
use crate::test_support::{builtin, run, said as texts};

// A training dummy takes a lot and never fights back; a dead one is undead.
const DUMMIES: &str = "[[creature]]\nid = \"dummy\"\nname = \"training dummy\"\nglyph = \"D\"\n\
     fg = \"#806040\"\nweight = 30000\nfaction = \"wildlife\"\nbase_hp = 1000\n\n\
     [[creature]]\nid = \"dead_dummy\"\nname = \"dead dummy\"\nglyph = \"Z\"\n\
     fg = \"#808080\"\nweight = 30000\nfaction = \"wildlife\"\nbase_hp = 1000\n\
     tags = [\"undead\"]\n";

/// The pack, two of each flask in this order: a, oil; b, stinking; c, tar; d, enfeeblement;
/// e, sloth; f, darkness; g, nightmares; h, silence; i, thunder; j, blasting; k, holy water.
const PACK: &str = "carry = [\"flask_of_oil*2\", \"stinking_flask*2\", \"flask_of_tar*2\", \
     \"flask_of_enfeeblement*2\", \"flask_of_sloth*2\", \"flask_of_darkness*2\", \
     \"vial_of_nightmares*2\", \"vial_of_silence*2\", \"flask_of_thunder*2\", \
     \"blasting_flask*2\", \"holy_water*2\"]";

/// A lit hall of `map` (`@` the player, `D` a dummy, `Z` a dead one, `+` a shut door).
fn hall(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    make_peace(&mut files);
    files.insert("data/creatures/dummies.toml", DUMMIES);
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"D\" = {{ terrain = \"floor\", creature = \"dummy\" }}\n\
             \"Z\" = {{ terrain = \"floor\", creature = \"dead_dummy\" }}\n\
             \"+\" = {{ terrain = \"door_closed\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

/// The hall every throw is made in: the dummy six cells east of the player.
const ROW: &str = "############\n#@.....D...#\n############\n";

fn at(w: &World, p: Pos) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == p)
        .map(|a| a.id)
        .expect("someone stands there")
}

fn lost(w: &World, id: ActorId) -> i32 {
    let a = &w.state.actors[&id];
    a.max_hp(&w.content) - a.hp
}

fn suffers(w: &World, id: ActorId, status: &str) -> Option<u16> {
    let status = w.content.status_id(status).expect("status");
    w.state.actors[&id].effects.get(status).map(|e| e.severity)
}

/// The kinds of zone lying on `p`.
fn zones_at(w: &World, p: Pos) -> Vec<ZoneKind> {
    (w.zones().iter())
        .filter(|z| z.cells().any(|c| c == p))
        .map(|z| z.kind())
        .collect()
}

/// Throws stack `letter` at the dummy of [`ROW`]; the world and the dummy after.
fn thrown(letter: char) -> (World, ActorId) {
    let mut w = hall(ROW);
    let dummy = at(&w, Pos::new(7, 1));
    run(&mut w, &format!("throw:{letter}:7,1"));
    (w, dummy)
}

#[test]
fn every_flask_is_a_potion_with_a_splash_and_a_label() {
    let content = builtin();
    for (id, spell) in [
        ("flask_of_oil", "slick"),
        ("stinking_flask", "reek"),
        ("flask_of_tar", "web"),
        ("flask_of_enfeeblement", "hex_of_weakness"),
        ("flask_of_sloth", "curse_of_sloth"),
        ("flask_of_darkness", "mirk"),
        ("vial_of_nightmares", "fear"),
        ("vial_of_silence", "silence"),
        ("flask_of_thunder", "thunderclap"),
        ("blasting_flask", "shatter"),
        ("holy_water", "rebuke"),
    ] {
        let def = content.object(content.object_id(id).expect(id));
        let flask = def.flask.as_ref().expect(id);
        assert_eq!(flask.spell, content.spell_id(spell).expect(spell), "{id}");
        assert!(def.drink.is_some(), "{id} is drunk too");
        assert!(flask.words.is_some(), "{id} says its own words");
    }
}

#[test]
fn the_flask_of_oil_lays_slick_ground() {
    let (w, _) = thrown('a');
    assert_eq!(zones_at(&w, Pos::new(7, 1)), vec![ZoneKind::Slick]);
    let said = texts(&w);
    assert!(
        said.contains(&"It was a flask of oil.".to_string()),
        "{said:?}"
    );
}

#[test]
fn the_stinking_flask_poisons_and_fouls_the_air() {
    let (w, dummy) = thrown('b');
    assert!((2..=8).contains(&lost(&w, dummy)), "2d4 poison at rung 0");
    assert_eq!(zones_at(&w, Pos::new(7, 1)), vec![ZoneKind::Fouled]);
}

#[test]
fn the_flask_of_tar_webs() {
    let (w, dummy) = thrown('c');
    // Webbed 3, a degree perhaps gone with the dummy's turn since (it eases every 6 s).
    assert!(matches!(suffers(&w, dummy, "webbed"), Some(2..=3)));
}

#[test]
fn the_hex_flasks_fall_on_whoever_stands_where_they_break() {
    let (w, dummy) = thrown('d');
    assert_eq!(suffers(&w, dummy, "weakened"), Some(1));
    let (w, dummy) = thrown('e');
    assert_eq!(suffers(&w, dummy, "sluggish"), Some(1));
}

#[test]
fn the_flask_of_darkness_lays_a_pall() {
    let (w, _) = thrown('f');
    assert_eq!(zones_at(&w, Pos::new(7, 1)), vec![ZoneKind::Dark]);
}

#[test]
fn the_vials_of_nightmares_and_silence_frighten_and_hush() {
    let (w, dummy) = thrown('g');
    // Frightened 2, a degree perhaps gone with the dummy's turn since (it eases every 6 s).
    assert!(matches!(suffers(&w, dummy, "frightened"), Some(1..=2)));
    let (w, dummy) = thrown('h');
    assert_eq!(suffers(&w, dummy, "silenced"), Some(1));
}

#[test]
fn the_flask_of_thunder_batters_all_beside_it_and_is_heard_over_the_level() {
    let (w, dummy) = thrown('i');
    assert!((2..=12).contains(&lost(&w, dummy)), "2d6 force at rung 0");
    // Thrown at one's feet it batters and stuns the thrower too (a stun lasts until the
    // stunned one's next action, so the dummy's is gone by now).
    let mut w = hall(ROW);
    let player = w.state.player;
    run(&mut w, "throw:i:2,1");
    assert!((2..=12).contains(&lost(&w, player)));
    assert_eq!(suffers(&w, player, "stunned"), Some(1));
    let content = &w.content;
    let def = content.object(content.object_id("flask_of_thunder").expect("thunder"));
    let noise = def.flask.as_ref().expect("a flask").noise;
    assert_eq!(
        (noise.sound, noise.loudness),
        (crate::noise::SoundKind::Thunder, 150)
    );
}

#[test]
fn the_blasting_flask_batters_and_breaks_shut_doors() {
    let mut w = hall("############\n#@.....D...#\n#......+...#\n############\n");
    let dummy = at(&w, Pos::new(7, 1));
    run(&mut w, "throw:j:7,1");
    assert!((4..=24).contains(&lost(&w, dummy)), "4d6 force at rung 0");
    let door = w.state.current().terrain[Pos::new(7, 2)];
    assert_eq!(w.content().terrain(door).id, "door_broken");
}

#[test]
fn holy_water_sears_the_undead_and_spares_the_living() {
    let mut w = hall("############\n#@.....ZD..#\n############\n");
    let dead = at(&w, Pos::new(7, 1));
    let living = at(&w, Pos::new(8, 1));
    run(&mut w, "throw:k:7,1");
    assert!((4..=24).contains(&lost(&w, dead)), "4d6 fire at rung 0");
    // Frightened 3, a degree perhaps gone with its turn since (it eases every 6 s).
    assert!(matches!(suffers(&w, dead, "frightened"), Some(2..=3)));
    assert_eq!(lost(&w, living), 0);
    assert_eq!(suffers(&w, living, "frightened"), None);
}

#[test]
fn each_harmful_flask_drunk_gives_its_status() {
    // The least left after the drink's own time, and the degrees given (Frightened eases a
    // degree every 6 s).
    for (letter, status, (least, degrees)) in [
        ('a', "poisoned", (1, 1)),
        ('b', "poisoned", (3, 3)),
        ('c', "poisoned", (1, 1)),
        ('d', "weakened", (2, 2)),
        ('e', "sluggish", (2, 2)),
        ('f', "ill_wished", (2, 2)),
        ('g', "frightened", (1, 2)),
        ('h', "silenced", (1, 1)),
        ('i', "stunned", (1, 1)),
        ('j', "stunned", (1, 1)),
    ] {
        let mut w = hall(ROW);
        run(&mut w, &format!("eat:{letter}"));
        let player = w.state.player;
        let left = suffers(&w, player, status).unwrap_or(0);
        assert!(
            (least..=degrees).contains(&left),
            "{letter}: {status} {left}"
        );
    }
}

#[test]
fn holy_water_drunk_washes_a_hex_off() {
    let mut w = hall(ROW);
    let player = w.state.player;
    run(&mut w, "afflict:ill_wished afflict:branded");
    assert_eq!(suffers(&w, player, "ill_wished"), Some(1));
    assert_eq!(suffers(&w, player, "branded"), Some(1));
    run(&mut w, "eat:k");
    assert_eq!(suffers(&w, player, "ill_wished"), None);
    assert_eq!(suffers(&w, player, "branded"), None);
    assert!(texts(&w).contains(&"It is a flask of holy water.".to_string()));
}
