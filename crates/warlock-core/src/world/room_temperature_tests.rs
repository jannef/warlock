//! Hot and cold rooms (spec §7 "Special rooms"; `WorldState::temperature_at`): a room sets its
//! cells' temperature and nothing else; the player feels it where they stand (the status bar
//! shows it), and the ice follows it: a cold room's pool freezes on a mild level, a hot room's
//! water stays open in a frost, alike however the time passes.

use super::*;
use crate::content::ContentFiles;
use crate::temperature::Temperature;

/// A hall, 30 wide: plain water at x 3 to 5, a sweltering chamber over x 10 to 19 (water at 13 to
/// 15), a frigid hall over x 20 to 29 (water at 23 to 25), each unless the test leaves it out.
const HALL: &str = "\
##############################
#@...........................#
#..~~~.......~~~.......~~~...#
#............................#
##############################
";

const PLAIN: Pos = Pos::new(4, 2);
const HOT: Pos = Pos::new(14, 2);
const COLD: Pos = Pos::new(24, 2);

/// The built-in content with [`HALL`] at `temperature` °C, with the rooms `rooms` names (`hot`,
/// `cold`).
fn world(temperature: i32, rooms: &[&str]) -> World {
    let mut files = ContentFiles::builtin();
    let mut marked = String::new();
    for room in rooms {
        let area = if *room == "hot" {
            "[10, 0, 19, 4]"
        } else {
            "[20, 0, 29, 4]"
        };
        marked.push_str(&format!("[[room]]\neffect = \"{room}\"\narea = {area}\n"));
    }
    files.insert(
        "levels/test_heat.toml",
        &format!(
            "name = \"Test\"\ntemperature = {temperature}\nambient = [120, 120, 120, 0]\n\
             map = '''\n{HALL}'''\n{marked}[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"~\" = {{ terrain = \"water\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_heat.toml",
        "name = \"T\"\nstart_level = \"test_heat\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_heat", None).expect("world")
}

fn run(w: &mut World, text: &str) {
    for action in crate::action::parse_actions(text).expect("actions") {
        w.apply(&action);
    }
}

fn mm(w: &World, p: Pos) -> u32 {
    w.state.current().ice_at(p).map_or(0, |i| i.mm())
}

fn ice_map(w: &World) -> BTreeMap<Pos, crate::ice::Ice> {
    w.state.current().ice.clone()
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

#[test]
fn hot_and_cold_rooms_are_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let hot = effects.get(effects.id("hot").expect("hot"));
    let cold = effects.get(effects.id("cold").expect("cold"));
    assert_eq!(
        (hot.name.as_str(), hot.temperature),
        ("sweltering chamber", 25)
    );
    assert_eq!((cold.name.as_str(), cold.temperature), ("frigid hall", -25));
    // Only the temperature (and a faint glow): no status, no floor, never the whole level.
    for e in [hot, cold] {
        assert!(e.status.is_none() && e.floor.is_none() && e.level_chance == 0);
        assert!(!e.shunned && e.weight == 5);
        assert_eq!(
            (e.regen, e.mana_regen, e.noise, e.light),
            (100, 100, 100, 100)
        );
    }
}

#[test]
fn a_room_sets_its_cells_temperature() {
    let w = world(10, &["hot", "cold"]);
    assert_eq!(w.temperature_at(PLAIN), Temperature::degrees(10));
    assert_eq!(w.temperature_at(HOT), Temperature::degrees(35));
    assert_eq!(w.temperature_at(COLD), Temperature::degrees(-15));
    // The room's area starts at x 10; the cell before it is the level's.
    assert_eq!(w.temperature_at(Pos::new(9, 1)), Temperature::degrees(10));
    assert_eq!(w.temperature_at(Pos::new(10, 1)), Temperature::degrees(35));
    // The player stands outside: the level's.
    assert_eq!(w.temperature(), Temperature::degrees(10));
}

#[test]
fn the_player_feels_it_where_they_stand_and_nothing_else_happens() {
    let mut w = world(10, &["hot", "cold"]);
    assert_eq!(w.perception().temperature(), Temperature::degrees(10));
    // Into the sweltering chamber: the status bar's temperature follows (it reads perception).
    run(&mut w, "move:E*9");
    assert_eq!(w.state.player().pos, Pos::new(10, 1));
    assert!(said(&w, "A wave of heat meets you."));
    assert_eq!(w.perception().temperature(), Temperature::degrees(35));
    assert_eq!(w.temperature(), Temperature::degrees(35));
    // On into the frigid hall.
    run(&mut w, "move:E*10");
    assert!(said(&w, "Your breath mists in the bitter cold."));
    assert_eq!(w.perception().temperature(), Temperature::degrees(-15));
    // An hour's rest in the cold: no status, no harm (what the cold does is the environment's).
    let hp = w.state.player().hp;
    run(&mut w, "rest:1h");
    assert!(
        w.state.player().effects.is_empty(),
        "the room gives no status"
    );
    assert!(w.state.player().hp >= hp);
    // Back out: the level's again.
    run(&mut w, "move:W*12");
    assert_eq!(w.perception().temperature(), Temperature::degrees(10));
}

#[test]
fn a_cold_rooms_pool_is_frozen_on_a_mild_level_and_the_water_beside_it_is_not() {
    // At -15 °C a level with a cold room has frozen for weeks when one first comes upon it.
    let mut w = world(10, &["cold"]);
    assert!(mm(&w, COLD) >= 150, "{}", mm(&w, COLD));
    assert_eq!(mm(&w, PLAIN), 0);
    assert_eq!(mm(&w, HOT), 0);
    let frozen = w.state.current().terrain[COLD];
    assert!(
        w.content().terrain(frozen).ice.is_some(),
        "it looks like ice"
    );
    // And it keeps growing there, and only there.
    let before = mm(&w, COLD);
    run(&mut w, "rest:24h");
    assert!(mm(&w, COLD) > before);
    assert_eq!((mm(&w, PLAIN), mm(&w, HOT)), (0, 0));
}

#[test]
fn a_hot_room_melts_its_ice_while_the_frost_holds_outside() {
    // A -5 °C level: its water froze over the weeks before, but not in the sweltering chamber.
    let mut w = world(-5, &["hot"]);
    assert!(mm(&w, PLAIN) >= 100, "{}", mm(&w, PLAIN));
    assert_eq!(mm(&w, HOT), 0);
    // A frost freezes the chamber's water at once; at 20 °C a day melts it again.
    run(&mut w, "freeze:14,2,1");
    assert_eq!(mm(&w, HOT), 100);
    let outside = mm(&w, PLAIN);
    run(&mut w, "rest:24h");
    assert_eq!(w.state.current().ice_at(HOT), None);
    assert!(mm(&w, PLAIN) > outside, "the frost outside goes on");
}

#[test]
fn a_level_without_hot_or_cold_rooms_is_not_rolled_back_in_time() {
    // Underground without such a room, nothing froze before the player came.
    let w = world(-5, &[]);
    assert_eq!(mm(&w, PLAIN), 0);
    assert_eq!(mm(&w, COLD), 0);
}

#[test]
fn the_ice_is_alike_however_the_time_passes() {
    let mut rested = world(0, &["hot", "cold"]);
    run(&mut rested, "rest:24h");
    let mut waited = world(0, &["hot", "cold"]);
    run(&mut waited, "wait*600 rest:23h");
    let mut skipped = world(0, &["hot", "cold"]);
    run(&mut skipped, "skip:5h skip:19h");
    assert_eq!(rested.time(), skipped.time());
    assert_eq!(waited.time(), skipped.time());
    assert_eq!(ice_map(&rested), ice_map(&skipped));
    assert_eq!(ice_map(&waited), ice_map(&skipped));
    // At 0 °C only the frigid hall's water froze.
    assert!(mm(&skipped, COLD) > 0);
    assert_eq!((mm(&skipped, PLAIN), mm(&skipped, HOT)), (0, 0));
}

#[test]
fn a_year_with_hot_and_cold_rooms_is_caught_up_quickly() {
    let mut w = world(0, &["hot", "cold"]);
    run(&mut w, "freeze:4,2,1 freeze:14,2,1");
    let start = std::time::Instant::now();
    run(&mut w, "skip:8640h wait");
    let took = start.elapsed();
    assert_eq!(mm(&w, COLD), u32::from(crate::ice::MAX_MM));
    assert_eq!(mm(&w, HOT), 0);
    assert_eq!(mm(&w, PLAIN), 100, "at 0 °C the plain ice keeps");
    assert!(took.as_millis() < 1500, "{took:?}");
}
