//! Races' gifts and flaws in the world (spec §8 "Character creation"): a race's natural armour
//! adds to what the player wears.

use super::*;
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
"##;

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
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// A run in the room as a `race` warrior.
fn world(race: &str) -> World {
    let spec: CharacterSpec = format!("{race},male,warrior").parse().expect("spec");
    World::with_character(CONTENT.clone(), "racetest", Some(1), Some(&spec)).expect("world")
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
