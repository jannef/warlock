//! Haunted rooms (spec §7 "Special rooms", design §9.8; `world/haunt.rs`): a rest inside is
//! haunted at every full hour by undead of the floor's danger, out of view where they can be; the
//! rest stops when one comes into view. Never outside the room, never while not resting, never on
//! a level without a danger.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::time::GameTime;

/// A lit hall, 30 wide; its east part (x 10 on) the haunted room. A wall at x 18, open at its
/// foot, hides the room's far end from its west part.
const HALL: &str = "\
##############################
#@................#..........#
#.................#..........#
#.................#..........#
#............................#
##############################
";

/// A cell inside the room, west of the wall.
const INSIDE: Pos = Pos::new(11, 2);
/// A cell outside the room.
const OUTSIDE: Pos = Pos::new(3, 2);

/// The hall with `header` (its danger fields) and the room over `area`.
fn world_of(header: &str, area: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_haunt.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\n{header}map = '''\n{HALL}'''\n\
             [[room]]\neffect = \"haunted\"\narea = {area}\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_haunt.toml",
        "name = \"T\"\nstart_level = \"test_haunt\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_haunt", None).expect("world")
}

/// Danger 3, where nothing comes of itself (the spawner never brings anyone).
const DANGER: &str = "danger = 3\npopulation = [\"kobold\"]\nspawn = { chance = 0 }\n";

/// The hall, on a level of danger 3, the room over x 10 to 29.
fn world() -> World {
    world_of(DANGER, "[10, 0, 29, 5]")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

/// The creatures on the level.
fn creatures(w: &World) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .collect()
}

/// The player stands at `p` and looks about.
fn stand(w: &mut World, p: Pos) {
    w.player_mut().pos = p;
    w.perceive(&mut Vec::new());
}

/// A few seconds before the next full hour.
fn just_before_the_hour(w: &mut World) {
    let hour = 3600;
    let next = (w.state.time.seconds() / hour + 1) * hour;
    w.state.time = GameTime::from_seconds(next - 5);
}

#[test]
fn the_haunted_room_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let haunted = effects.get(effects.id("haunted").expect("haunted"));
    assert_eq!(
        (haunted.name.as_str(), haunted.weight, haunted.level_chance),
        ("haunted room", 5, 0)
    );
    assert!(haunted.needs_danger && haunted.unholy);
    let haunt = haunted.haunt.as_ref().expect("it haunts");
    assert_eq!(
        (haunt.faction.as_str(), haunt.every),
        ("undead", crate::time::Duration::hours(1))
    );
    assert_eq!(
        haunted.near.as_deref(),
        Some("You hear a whisper, and then nothing.")
    );
    assert_eq!(haunted.enter, "It is very quiet here. Too quiet to sleep.");
    // Nothing else haunts.
    for e in effects.defs().iter().filter(|e| e.id != "haunted") {
        assert!(e.haunt.is_none(), "{}", e.id);
    }
}

#[test]
fn a_rest_inside_is_haunted_on_the_hour_and_stops() {
    let mut w = world();
    stand(&mut w, INSIDE);
    just_before_the_hour(&mut w);
    let start = w.state.time;
    run(&mut w, "rest:2h");
    let came = creatures(&w);
    assert_eq!(came.len(), 1, "one comes on the hour");
    let def = w.content().creature(came[0].creature);
    let undead = w.content().factions().id("undead");
    assert_eq!(def.faction, undead);
    assert!(!def.unique && came[0].inventory.is_empty());
    let candidates = rise_candidates(w.content(), undead.expect("undead"), "", 3);
    assert!(candidates.contains(&came[0].creature), "{}", def.name);
    // The rest stopped for it, well before the two hours were up.
    assert!(
        (w.messages().iter()).any(|m| m.text.contains("you stop resting")),
        "{:?}",
        w.messages()
    );
    assert!(w.state.time < start + crate::time::Duration::hours(1));
}

#[test]
fn what_comes_comes_out_of_view_where_it_can() {
    for seed in 0..8u64 {
        let mut w = world();
        w.state.rng = crate::rng::Rng::stream(seed, "haunt");
        stand(&mut w, INSIDE);
        let began = GameTime::from_seconds(3599);
        w.state.time = GameTime::from_seconds(3600);
        w.resting = true;
        w.haunt_tick(began);
        let came = creatures(&w);
        assert_eq!(came.len(), 1, "seed {seed}");
        let pos = came[0].pos;
        assert!(w.state.current().rooms[0].contains(pos));
        assert!(!w.perception.sees_actors_at(pos), "seed {seed}: {pos} seen");
    }
}

#[test]
fn where_all_of_it_is_seen_it_comes_in_view() {
    // The room is only the part west of the wall: the player sees all of it.
    let mut w = world_of(DANGER, "[10, 0, 17, 5]");
    stand(&mut w, INSIDE);
    w.state.time = GameTime::from_seconds(7200);
    w.resting = true;
    w.haunt_tick(GameTime::from_seconds(7190));
    let came = creatures(&w);
    assert_eq!(came.len(), 1);
    assert!(w.perception.sees_actors_at(came[0].pos));
}

#[test]
fn no_haunting_outside_the_room_or_without_a_rest() {
    // Resting outside.
    let mut w = world();
    stand(&mut w, OUTSIDE);
    just_before_the_hour(&mut w);
    run(&mut w, "rest:3h");
    assert!(creatures(&w).is_empty());
    // Waiting inside across the hour.
    let mut w = world();
    stand(&mut w, INSIDE);
    just_before_the_hour(&mut w);
    run(&mut w, "wait*3");
    assert!(creatures(&w).is_empty());
    // Resting inside within the hour.
    let mut w = world();
    stand(&mut w, INSIDE);
    w.state.time = GameTime::from_seconds((w.state.time.seconds() / 3600 + 1) * 3600 + 60);
    run(&mut w, "rest:30m");
    assert!(creatures(&w).is_empty());
}

#[test]
fn no_haunting_on_a_level_without_a_danger() {
    let mut w = world_of("", "[10, 0, 29, 5]");
    stand(&mut w, INSIDE);
    just_before_the_hour(&mut w);
    run(&mut w, "rest:3h");
    assert!(creatures(&w).is_empty());
}

#[test]
fn each_hour_of_a_long_rest_brings_another() {
    let mut w = world();
    stand(&mut w, INSIDE);
    just_before_the_hour(&mut w);
    // Each rest stops when what came is seen; it is sent away, and the player rests again.
    let mut came = 0;
    for _ in 0..3 {
        let start = w.state.time;
        run(&mut w, "rest:2h");
        assert!(w.state.time < start + crate::time::Duration::hours(2));
        let ids: Vec<ActorId> = creatures(&w).iter().map(|a| a.id).collect();
        came += ids.len();
        for id in ids {
            w.state.actors.remove(&id);
        }
    }
    assert_eq!(came, 3);
}

#[test]
fn consecrate_lifts_the_haunting() {
    let mut w = world();
    let me = w.player_mut();
    me.talents.take("consecrate", None);
    me.alignment = Some(crate::alignment::Score::new(500));
    me.mana_base = 40;
    me.mana = 40;
    stand(&mut w, INSIDE);
    run(&mut w, "use:consecrate");
    assert!(
        (w.messages().iter())
            .any(|m| m.text == "A stillness settles over the room: it is consecrated."),
        "{:?}",
        w.messages()
    );
    assert!(w.state.current().rooms.is_empty(), "a plain room");
    just_before_the_hour(&mut w);
    run(&mut w, "rest:3h");
    assert!(creatures(&w).is_empty(), "nothing comes now");
}
