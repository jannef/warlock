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

/// A lit hall with a pick-axe by the player, walled off from a greater treasure room east of it
/// (x 5 on, walls included) with no door: its breach the wall at (5, 2).
const VAULT: &str = "\
#############
#@...#......#
#....#......#
#....#......#
#############
";

const GREAT: &str = "Something tells you a fortune lies walled up on this level.";
const BREACH: Pos = Pos { x: 5, y: 2 };

/// The built-in content with [`VAULT`] as level `test_vault`, its room a greater treasure room
/// with `room` added to its `[[room]]` table (its breach, by default), and a plain level `other`;
/// or why it is refused.
fn vault_content(room: &str) -> Result<Arc<Content>, crate::content::ContentError> {
    let mut files = ContentFiles::builtin();
    let level = format!(
        "name = \"Vault\"\nambient = [120, 120, 120, 0]\nmap = '''\n{VAULT}'''\n\
         [[room]]\neffect = \"great_treasury\"\narea = [5, 0, 12, 4]\n{room}\n\
         [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\", objects = [\"pick_axe\"] }}\n"
    );
    files.insert("levels/test_vault.toml", &level);
    files.insert(
        "levels/other.toml",
        "name = \"Other\"\nambient = [120, 120, 120, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/test_vault.toml",
        "name = \"V\"\nstart_level = \"test_vault\"\n",
    );
    Content::load(&files).map(Arc::new)
}

fn vault() -> World {
    let content = vault_content("breach = [5, 2]").expect("fixture content is valid");
    World::new(content, "test_vault", None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

/// Searches beside the breach until it is found (a few hundred turns at most).
fn search_out(w: &mut World) {
    for _ in 0..20 {
        if w.state.current().hollows.contains(&BREACH) {
            return;
        }
        run(w, "search:100");
    }
    panic!("the breach was never found");
}

#[test]
fn the_greater_treasury_effect_is_data() {
    let w = vault();
    let effects = w.content().room_effects();
    let great = effects.get(effects.id("great_treasury").expect("greater"));
    assert_eq!(
        (great.name.as_str(), great.mark, great.weight),
        ("greater treasure room", 'G', 2)
    );
    assert_eq!(great.arrive.as_deref(), Some(GREAT));
    assert_eq!(great.sealed, Some(crate::content::Sealed::Dig));
    assert!(great.secret_doors.is_none());
    let treasure = great.treasure.expect("treasure");
    assert_eq!(
        (treasure.chests.to_string(), treasure.gems.to_string()),
        ("2d2".to_string(), "2d3".to_string())
    );
    assert!(!great.can_pervade(), "never a whole level");
    assert_eq!(w.state.current().rooms[0].breach, Some(BREACH));
}

#[test]
fn a_level_file_s_sealed_room_needs_a_breach_that_opens_it() {
    let refused = |room: &str| {
        let err = vault_content(room).expect_err("refused");
        err.message
    };
    assert!(refused("").contains("needs a `breach`"));
    // On the far wall: beside nothing reached.
    assert!(refused("breach = [12, 2]").contains("breach"));
    // A corner touches no floor of the room.
    assert!(refused("breach = [5, 0]").contains("breach"));
    // Floor inside is no wall.
    assert!(refused("breach = [7, 2]").contains("not a wall"));
    // Another effect has no breach.
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_vault.toml",
        &format!(
            "name = \"Vault\"\nmap = '''\n{VAULT}'''\n[[room]]\neffect = \"treasury\"\n\
             area = [5, 0, 12, 4]\nbreach = [5, 2]\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    let err = Content::load(&files).expect_err("refused");
    assert!(err.message.contains("not `sealed`"), "{}", err.message);
    // A room reached without breaking a wall is no sealed room.
    let open = VAULT.replace("#....#......#\n#....#", "#....#......#\n#.....");
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_vault.toml",
        &format!(
            "name = \"Vault\"\nmap = '''\n{open}'''\n[[room]]\neffect = \"great_treasury\"\n\
             area = [5, 0, 12, 4]\nbreach = [5, 2]\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    let err = Content::load(&files).expect_err("refused");
    assert!(
        err.message.contains("without breaking a wall"),
        "{}",
        err.message
    );
}

#[test]
fn the_breach_sounds_hollow_to_a_searcher_and_is_remembered() {
    let mut w = vault();
    assert_eq!(told(&w, GREAT), 1, "the hint at the start");
    assert!(!w.perception().hollow(BREACH));
    // Nothing is found from across the hall.
    run(&mut w, "search:5");
    assert!(w.state.current().hollows.is_empty());
    run(&mut w, "move:SE move:E*2");
    search_out(&mut w);
    assert_eq!(told(&w, "The wall here sounds hollow."), 1);
    assert!(w.perception().hollow(BREACH));
    assert_eq!(w.perception().known_hollows(), [BREACH]);
    // The wall is the same wall; only look mode's word and the memory know it.
    assert_eq!(
        w.state.current().memory[BREACH],
        Some(w.state.current().terrain[BREACH])
    );
    // Found once: searching on finds nothing more there.
    run(&mut w, "search:10");
    assert_eq!(told(&w, "The wall here sounds hollow."), 1);
    // Saved and loaded, it is still known; the hint still nags on arrival.
    let bytes = crate::save::encode(&w);
    let mut w = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert!(w.perception().hollow(BREACH));
    w.travel("other", None).expect("travels");
    w.travel("test_vault", None).expect("travels back");
    assert_eq!(told(&w, GREAT), 2);
}

#[test]
fn digging_the_breach_opens_the_room() {
    let mut w = vault();
    run(&mut w, "pickup");
    let pick = (w.state.player().inventory.iter())
        .position(|s| w.content().object(s.object).id == "pick_axe")
        .expect("a pick-axe");
    run(&mut w, &format!("equip:tool:{pick} move:SE move:E*2"));
    search_out(&mut w);
    run(&mut w, "dig:E");
    let (level, _) = w.ground_truth();
    assert!(level.is_walkable(w.content(), BREACH), "the wall gave way");
    assert!(level.hollows.is_empty(), "no hollow wall once broken");
    assert!(!w.perception().hollow(BREACH));
    run(&mut w, "move:E move:E");
    assert!(w.state.current().rooms[0].entered);
    assert_eq!(
        told(&w, "The air is stale and heavy with the smell of old gold."),
        1
    );
    w.travel("other", None).expect("travels");
    w.travel("test_vault", None).expect("travels back");
    assert_eq!(told(&w, GREAT), 1, "never again once entered");
}

#[test]
fn a_blast_opens_the_breach_too() {
    let mut w = vault();
    // A scroll of fireball's burst (1000) beats a stone wall's 600.
    w.blast(Pos::new(4, 2), 1, 1000, &mut Vec::new());
    let (level, _) = w.ground_truth();
    assert!(level.is_walkable(w.content(), BREACH));
    assert!(!level.breach_at(w.content(), BREACH), "no breach standing");
    // Nothing is left to find.
    run(&mut w, "move:SE move:E*2 search:20");
    assert_eq!(told(&w, "The wall here sounds hollow."), 0);
}

#[test]
fn the_rooms_dump_names_the_breach() {
    let w = vault();
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Rooms).join("\n");
    assert!(
        rows.contains("G greater treasure room at (5, 0)-(12, 4), breach (5, 2)"),
        "{rows}"
    );
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
