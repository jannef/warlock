//! Fire and cold (spec §8 "Potions": "Flasks"; the potions wave 2 design §4.4, slice S5): the
//! flask of fire, the flask of frost, and pitch fire, each thrown and drunk once from the game's
//! own data, and the Pitch fire spell its scroll casts. The rules are `flask.rs`'s
//! (`flask_tests.rs`) and `zone.rs`'s (`zone_tests.rs`); these check that each file does what the
//! design's §4.4 and §6 tables say.

use super::caster_tests::sp;
use super::fixture_factions::make_peace;
use super::*;
use crate::content::{ContentFiles, ZoneKind};
use crate::test_support::builtin;
use crate::test_support::run;

// A training dummy takes a lot and never fights back.
const DUMMY: &str = "[[creature]]\nid = \"dummy\"\nname = \"training dummy\"\nglyph = \"D\"\n\
     fg = \"#806040\"\nweight = 30000\nfaction = \"wildlife\"\nbase_hp = 1000\n";

/// The pack, two of each in this order: a, flask of fire; b, flask of frost; c, pitch fire; d,
/// the scroll of pitch fire.
const PACK: &str = "carry = [\"flask_of_fire*2\", \"flask_of_frost*2\", \"pitch_fire*2\", \
     \"scroll_of_pitch_fire*2\"]";

/// A lit hall of `map` (`@` the player, `D` a dummy).
fn hall(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    make_peace(&mut files);
    files.insert("data/creatures/dummies.toml", DUMMY);
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"D\" = {{ terrain = \"floor\", creature = \"dummy\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

/// The dummy four cells east of the player, in reach of every flask here.
const ROW: &str = "##########\n#@...D...#\n#........#\n##########\n";
const DUMMY_AT: Pos = Pos::new(5, 1);

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn dummy(w: &World) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.pos == DUMMY_AT)
        .map(|a| a.id)
        .expect("the dummy")
}

fn lost(w: &World, id: ActorId) -> i32 {
    let a = &w.state.actors[&id];
    a.max_hp(&w.content) - a.hp
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

/// The kinds of zone lying on `p`.
fn zones_at(w: &World, p: Pos) -> Vec<ZoneKind> {
    (w.zones().iter())
        .filter(|z| z.cells().any(|c| c == p))
        .map(|z| z.kind())
        .collect()
}

#[test]
fn the_fire_and_cold_flasks_cast_their_spells_and_are_drunk_too() {
    let content = builtin();
    for (id, spell, range) in [
        ("flask_of_fire", "fireball", 6),
        ("flask_of_frost", "frost", 6),
        ("pitch_fire", "pitch_fire", 4),
    ] {
        let def = content.object(content.object_id(id).expect(id));
        let flask = def.flask.as_ref().expect(id);
        assert_eq!(flask.spell, content.spell_id(spell).expect(spell), "{id}");
        assert_eq!(flask.range, range, "{id}");
        assert!(def.drink.is_some(), "{id} is drunk too");
        assert!(flask.words.is_some(), "{id} says its own words");
    }
    // Pitch fire is a plain spell with a scroll (Q4, Q11).
    let pitch = content.spell_id("pitch_fire").expect("the spell");
    assert!(content.scroll_of(pitch).is_some());
}

#[test]
fn the_flask_of_fire_bursts_as_a_fireball() {
    let mut w = hall(ROW);
    let id = dummy(&w);
    run(&mut w, "throw:a:5,1");
    assert!((4..=24).contains(&lost(&w, id)), "4d6 fire at rung 0");
    let said = texts(&w);
    assert!(
        said.iter()
            .any(|s| s == "Fire bursts roaring from the shards!"),
        "{said:?}"
    );
    assert!(
        said.iter().any(|s| s == "It was a flask of fire."),
        "{said:?}"
    );
    assert!(w.zones().is_empty(), "a fireball lays no zone");
}

#[test]
fn the_flask_of_frost_numbs_and_puts_out_fire() {
    let mut w = hall(ROW);
    let id = dummy(&w);
    run(&mut w, "zone:fire@6,2,1");
    assert_eq!(zones_at(&w, Pos::new(6, 2)), vec![ZoneKind::Fire]);
    let before = lost(&w, id);
    run(&mut w, "throw:b:5,1");
    assert!(
        (4..=24).contains(&(lost(&w, id) - before)),
        "4d6 cold at rung 0"
    );
    assert!(suffers(&w, id, "sluggish").is_some(), "cold's rider");
    assert!(zones_at(&w, Pos::new(6, 2)).is_empty(), "doused");
    let said = texts(&w);
    assert!(
        said.iter().any(|s| s == "It was a flask of frost."),
        "{said:?}"
    );
}

#[test]
fn pitch_fire_leaves_a_pool_of_fire_by_its_thrower_for_six_turns() {
    let mut w = hall(ROW);
    let id = dummy(&w);
    let began = w.time();
    run(&mut w, "throw:c:5,1");
    assert!((2..=12).contains(&lost(&w, id)), "2d6 fire at rung 0");
    assert_eq!(w.zones().len(), 1);
    let zone = &w.zones()[0];
    assert_eq!(zone.kind(), ZoneKind::Fire);
    assert_eq!(zone.by(), w.state.player);
    // Radius 1 about the dummy, the wall cells left out.
    let mut cells: Vec<Pos> = zone.cells().collect();
    cells.sort();
    let mut pool: Vec<Pos> = (4..=6)
        .flat_map(|x| (1..=2).map(move |y| Pos::new(x, y)))
        .collect();
    pool.sort();
    assert_eq!(cells, pool);
    // Six of the player's turns of 6 seconds.
    assert_eq!(zone.until().since(began).as_seconds(), 36);
    assert!((2..=12).contains(&zone.harm()), "the cast's own roll");
    let said = texts(&w);
    assert!(said.iter().any(|s| s == "It was a pitch fire."), "{said:?}");
}

#[test]
fn pitch_fire_is_heavy_and_flies_only_four_cells() {
    let mut w = hall("##########\n#@.....D.#\n##########\n");
    let time = w.time();
    run(&mut w, "throw:c:7,1");
    assert_eq!(w.time(), time, "refused, no time");
    let last = texts(&w).last().cloned().unwrap_or_default();
    assert!(last.starts_with("That is out of range."), "{last}");
    assert_eq!(count(&w, "pitch_fire"), 2, "the flask kept");
    assert!(w.zones().is_empty());
}

#[test]
fn the_scroll_of_pitch_fire_casts_the_pool_too() {
    let mut w = hall(ROW);
    // One who knows the spell reads its scroll without the reading check (`decipher.rs`).
    w.player_mut().spells.insert(sp("pitch_fire"), 1);
    run(&mut w, "read:d:5,1");
    assert_eq!(
        zones_at(&w, DUMMY_AT),
        vec![ZoneKind::Fire],
        "{:?}",
        texts(&w)
    );
}

#[test]
fn drunk_they_burn_and_slow_as_the_design_says() {
    for (letter, status, degrees) in [
        ('a', "burning", 1),
        ('b', "sluggish", 2),
        ('c', "burning", 2),
    ] {
        let mut w = hall(ROW);
        run(&mut w, &format!("eat:{letter}"));
        let me = w.state.player;
        assert_eq!(suffers(&w, me, status), Some(degrees), "{letter}");
    }
}
