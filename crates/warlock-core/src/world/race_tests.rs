//! Races' gifts and flaws in the world (spec §8 "Character creation"): a race's natural armour
//! adds to what the player wears, and so do its resistances; strong light dazzles light-blind
//! eyes.

use super::*;
use crate::action::parse_actions;
use crate::combat::Category;
use crate::content::ContentFiles;
use std::sync::LazyLock;

/// Test races beside the built-in ones, each with one gift.
const RACES: &str = r##"

[[race]]
id = "scaled"
name = "Scaled"
description = "Scaly."
vision = "normal"
age = [20, 30]
natural = { protection = 1, defence = 2, name = "scales" }

[[race]]
id = "hardy"
name = "Hardy"
description = "Hard to poison."
vision = "normal"
age = [20, 30]
resist = { poison = 50 }

[[race]]
id = "blind"
name = "Light-blind"
description = "Dazzled by strong light."
vision = "normal"
age = [20, 30]
light_blind = { glare = 200, sight = 3, to_hit = -2 }
"##;

/// A dim corridor (ambient 60, enough for normal eyes) with a brazier at its east end: its light
/// reaches 200 within about five cells of it.
const CORRIDOR: &str = "\
###############
#@...........B#
###############
";

/// A lit room: the player and nothing else.
const ROOM: &str = "\
#######
#.....#
#..@..#
#.....#
#######
";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.edit_table("races", |text| format!("{text}{RACES}"));
    files.insert(
        "levels/racetest.toml",
        &format!(
            "name = \"Room\"\nambient = [255, 255, 255, 0]\nmap = '''\n{ROOM}'''\n\n\
             [legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    files.insert(
        "scenarios/racetest.toml",
        "name = \"Room\"\nstart_level = \"racetest\"\nhp = 30\n",
    );
    files.insert(
        "levels/glaretest.toml",
        &format!(
            "name = \"Corridor\"\nambient = [60, 60, 60, 0]\nmap = '''\n{CORRIDOR}'''\n\n\
             [legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"B\" = {{ terrain = \"floor\", light = \"brazier\" }}\n"
        ),
    );
    files.insert(
        "scenarios/glaretest.toml",
        "name = \"Corridor\"\nstart_level = \"glaretest\"\nhp = 30\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// A run in the room as a `race` warrior.
fn world(race: &str) -> World {
    world_in("racetest", race)
}

/// A run of `scenario` as a `race` warrior.
fn world_in(scenario: &str, race: &str) -> World {
    let spec: CharacterSpec = format!("{race},male,warrior").parse().expect("spec");
    World::with_character(CONTENT.clone(), scenario, Some(1), Some(&spec)).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display().contains(text))
}

#[test]
fn a_races_natural_armour_adds_to_defence_and_protection() {
    let (human, scaled) = (world("human"), world("scaled"));
    let me = |w: &World| w.state.player;
    assert_eq!(human.protection_of(me(&human)).base, 0);
    assert_eq!(scaled.protection_of(me(&scaled)).base, 1);
    let natural = |w: &World| w.defence_of(me(w)).bonus(Category::Natural);
    assert_eq!((natural(&human), natural(&scaled)), (0, 2));
    // The perception shows the player the total, and a blow it soaks glances off the scales.
    assert_eq!(scaled.perception().own_protection(), 1);
    let race = scaled
        .state
        .race_of(scaled.content(), scaled.state.player());
    let soak = scaled.state.player().soaked_by(scaled.content(), race);
    assert_eq!(soak, Some("scales"));
    // Without a character, the player goes by their creature type: no hide.
    let plain = World::new(CONTENT.clone(), "racetest", Some(1)).expect("world");
    assert_eq!(plain.protection_of(plain.state.player).base, 0);
}

#[test]
fn a_races_resistances_shrink_what_the_player_takes_and_are_told_in_words() {
    let (human, hardy) = (world("human"), world("hardy"));
    let poison = CONTENT.damage_type_id("poison").expect("poison");
    let resisted = |w: &World| w.resistances_of(w.state.player).get(Some(poison));
    assert_eq!((resisted(&human), resisted(&hardy)), (0, 50));
    let harm = crate::damage::Damage {
        amount: 10,
        kind: Some(poison),
    };
    assert_eq!(human.resisted(human.state.player, harm), 10);
    assert_eq!(hardy.resisted(hardy.state.player, harm), 5);
    let words = hardy.perception().own_resistances();
    assert_eq!(words, [(poison, crate::damage::ResistanceWord::Resistant)]);
    assert!(human.perception().own_resistances().is_empty());
}

#[test]
fn strong_light_dazzles_light_blind_eyes_cutting_sight_and_to_hit() {
    let mut w = world_in("glaretest", "blind");
    let me = w.state.player;
    let far = Pos::new(1, 1);
    // In the dim end, the corridor is seen to the brazier.
    assert!(!w.perception().dazzled());
    assert!(w.perception().is_visible(Pos::new(13, 1)));
    let profile = w.state.player().melee(w.content());
    assert_eq!(w.glared(me, profile.clone()).to_hit, profile.to_hit);
    // Four cells from the brazier its light dazzles them: they see three cells, no more, and
    // fight worse; they are told, never by how much.
    run(&mut w, "move:E*8");
    assert_eq!(w.player_pos(), Pos::new(9, 1));
    assert!(w.perception().dazzled());
    assert!(said(&w, "The bright light dazzles you."));
    assert!(w.perception().is_visible(Pos::new(6, 1)), "three cells off");
    assert!(!w.perception().is_visible(Pos::new(5, 1)), "four cells off");
    assert!(!w.perception().is_visible(far));
    assert_eq!(w.glared(me, profile.clone()).to_hit, profile.to_hit - 2);
    // Back in the dim end their eyes adjust.
    run(&mut w, "move:W*6");
    assert!(!w.perception().dazzled());
    assert!(said(&w, "Your eyes adjust to the dimmer light."));
    assert!(w.perception().is_visible(Pos::new(13, 1)));
    // Other eyes are never dazzled.
    let mut human = world_in("glaretest", "human");
    run(&mut human, "move:E*8");
    assert!(!human.perception().dazzled());
    assert!(human.perception().is_visible(far));
    let me = human.state.player;
    let profile = human.state.player().melee(human.content());
    assert_eq!(human.glared(me, profile.clone()).to_hit, profile.to_hit);
}
