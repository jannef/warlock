//! Treasure rooms in play (spec §7 "Special rooms"; `world/treasury.rs`): the hint on every
//! arrival until the room is entered, Treasure sense's bearing, the enter line, one line however
//! many rooms, and saving.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;

/// A lit hall, its east part (x 9 on, walls included) a treasure room behind a closed door at
/// (9, 2).
const HALL: &str = "\
###############
#@.......#....#
#........+....#
#........#....#
###############
";

const LINE: &str = "Something tells you there is wealth hidden on this level.";

/// The built-in content with [`HALL`] as level `test_treasury` (its room a treasure room, or, with
/// `second`, a second one over the hall's west end too) and a plain level `other`.
fn world(rooms: &[[i32; 4]]) -> World {
    let mut files = ContentFiles::builtin();
    let rooms: String = (rooms.iter())
        .map(|[x0, y0, x1, y1]| {
            format!("[[room]]\neffect = \"treasury\"\narea = [{x0}, {y0}, {x1}, {y1}]\n")
        })
        .collect();
    let level = |name: &str, rooms: &str| {
        format!(
            "name = \"{name}\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n{rooms}\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"+\" = \"door_closed\"\n"
        )
    };
    files.insert("levels/test_treasury.toml", &level("Test", &rooms));
    files.insert("levels/other.toml", &level("Other", ""));
    files.insert(
        "scenarios/test_treasury.toml",
        "name = \"T\"\nstart_level = \"test_treasury\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_treasury", None).expect("world")
}

/// How many times a message starting with `start` was said.
fn told(w: &World, start: &str) -> u32 {
    (w.messages().iter())
        .filter(|m| m.text.starts_with(start))
        .map(|m| m.count)
        .sum()
}

fn last_message(world: &World) -> String {
    (world.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

#[test]
fn the_treasury_effect_is_data() {
    let w = world(&[]);
    let effects = w.content().room_effects();
    let treasury = effects.get(effects.id("treasury").expect("treasury"));
    assert_eq!(
        (treasury.name.as_str(), treasury.mark, treasury.weight),
        ("treasure room", '$', 4)
    );
    assert_eq!(treasury.arrive.as_deref(), Some(LINE));
    assert!(treasury.secret_doors.is_some());
    let treasure = treasury.treasure.expect("treasure");
    assert_eq!(
        (treasure.chests.to_string(), treasure.gems.to_string()),
        ("1d2".to_string(), "1d3".to_string())
    );
    assert_eq!(treasure.gold.each, 2);
    assert!(!treasury.can_pervade(), "never a whole level");
    let hoard = effects.get(effects.id("hoard").expect("hoard"));
    assert_eq!((hoard.name.as_str(), hoard.mark), ("tempting hoard", 'T'));
}

#[test]
fn the_hint_is_said_on_every_arrival_until_the_room_is_entered() {
    let mut w = world(&[[9, 0, 14, 4]]);
    assert_eq!(told(&w, LINE), 1, "at the start of a run");
    w.travel("other", None).expect("travels");
    assert_eq!(told(&w, LINE), 1, "not on a level without one");
    w.travel("test_treasury", None).expect("travels back");
    assert_eq!(told(&w, LINE), 2, "again on the way back");
    // Saved and loaded, it still nags.
    let bytes = crate::save::encode(&w);
    let mut w = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    w.travel("other", None).expect("travels");
    w.travel("test_treasury", None).expect("travels back");
    assert_eq!(told(&w, LINE), 3);
    // In through the door: the enter line, and never the hint again.
    for action in parse_actions("move:SE move:E*7 open:E move:E move:E").expect("actions") {
        w.apply(&action);
    }
    assert!(
        w.state.current().rooms[0].entered,
        "{:?}",
        w.state.player().pos
    );
    assert_eq!(
        told(
            &w,
            "Gold glints in the dust. No one has been here in a long time."
        ),
        1
    );
    w.travel("other", None).expect("travels");
    w.travel("test_treasury", None).expect("travels back");
    assert_eq!(told(&w, LINE), 3, "left once the room is entered");
}

#[test]
fn two_treasure_rooms_say_it_once() {
    let w = world(&[[9, 0, 14, 4], [4, 2, 7, 4]]);
    assert_eq!(told(&w, LINE), 1);
}

#[test]
fn treasure_sense_says_which_way_it_lies() {
    let mut w = world(&[[9, 0, 14, 4]]);
    w.player_mut().talents.take("treasure_sense", None);
    w.travel("other", None).expect("travels");
    w.travel("test_treasury", None).expect("travels back");
    assert_eq!(
        last_message(&w),
        "Something tells you there is wealth hidden on this level, to the east."
    );
}
