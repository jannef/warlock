//! Races' gifts and flaws in the world (spec §8 "Character creation"): a race's natural armour
//! adds to what the player wears, and so do its resistances; strong light dazzles light-blind
//! eyes; a disease bonus wins disease checks; a stomach for rotten food eats it unharmed; an
//! eye for traps notices them in passing.

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

[[race]]
id = "gutsy"
name = "Gutsy"
description = "Shrugs diseases off."
vision = "normal"
age = [20, 30]
disease_check = 10

[[race]]
id = "scavenger"
name = "Scavenger"
description = "Eats anything."
vision = "normal"
age = [20, 30]
eats_rotten = true

[[race]]
id = "trapwise"
name = "Trapwise"
description = "Notices traps."
vision = "normal"
age = [20, 30]
passive_search = "traps"
"##;

/// A dim corridor (ambient 60, enough for normal eyes) with a brazier at its east end: its light
/// reaches 200 within about five cells of it.
const CORRIDOR: &str = "\
###############
#@...........B#
###############
";

/// A lit room: the player on a loaf of bread (it keeps five days), and nothing else.
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
             \"@\" = {{ terrain = \"floor\", spawn = \"player\", objects = [\"bread\"] }}\n"
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
    // A secret door west of the player, a hidden alarm trap east.
    files.insert(
        "levels/traptest.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#S@a#\n#####\n'''\n\n\
         [legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"S\" = { terrain = \"door_closed\", secret = 1 }\n\
         \"a\" = { terrain = \"floor\", trap = \"alarm\" }\n",
    );
    files.insert(
        "scenarios/traptest.toml",
        "name = \"Hall\"\nstart_level = \"traptest\"\nhp = 30\n",
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
    seeded(scenario, race, 1)
}

/// A run of `scenario` as a `race` warrior, from `seed`, born under no sign (so a sign's gifts,
/// spec §8 "Birth sign", leave the race's rules alone).
fn seeded(scenario: &str, race: &str, seed: u64) -> World {
    let spec: CharacterSpec = format!("{race},male,warrior").parse().expect("spec");
    let w =
        World::with_character(CONTENT.clone(), scenario, Some(seed), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    World::from_state(CONTENT.clone(), state).expect("restores")
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

#[test]
fn a_races_disease_bonus_wins_checks_against_catching_a_disease() {
    let fever = CONTENT.disease_id("filth_fever").expect("filth fever");
    // Filth fever is caught on a failed Toughness check of 13: about half the time at an
    // average score, hardly ever with 10 more.
    let caught = |race: &str| {
        (0..40)
            .filter(|&seed| {
                let mut w = seeded("racetest", race, seed);
                let me = w.state.player;
                w.expose(me, fever, None)
            })
            .count()
    };
    let (human, gutsy) = (caught("human"), caught("gutsy"));
    assert!(human >= 10, "a human caught it {human} times in 40");
    assert!(gutsy <= 2, "the gutsy caught it {gutsy} times in 40");
}

#[test]
fn a_stomach_for_rotten_food_eats_it_unharmed() {
    let eat = |race: &str| {
        let mut w = world(race);
        // Five days on, the bread underfoot has rotted.
        run(&mut w, "skip:120h");
        let before = w.state.player().satiation.expect("hunger").nutrition;
        w.apply(&Action::Eat(crate::action::Meal::Floor(0)));
        let fed = w.state.player().satiation.expect("hunger").nutrition - before;
        (w, fed)
    };
    let (scavenger, fed) = eat("scavenger");
    assert!(said(
        &scavenger,
        "The loaf of bread was rotten, but your stomach does not mind."
    ));
    assert!(!said(&scavenger, "Ugh!"));
    assert!(!said(&scavenger, "You feel sick."));
    assert!(scavenger.state.player().effects.is_empty(), "never sick");
    assert!(scavenger.state.player().diseases.is_empty(), "nor diseased");
    assert!(fed > 700, "it feeds, as bread does");
    let (human, _) = eat("human");
    assert!(said(&human, "Ugh! The loaf of bread was rotten."));
}

#[test]
fn an_eye_for_traps_notices_traps_in_passing_but_not_secret_doors() {
    // One check a turn for the trap beside them, none for the secret door.
    let mut w = world_in("traptest", "trapwise");
    let checked = |w: &World| {
        let mut rng = w.state.rng.clone();
        crate::combat::roll_natural(&mut rng);
        rng
    };
    let once = checked(&w);
    run(&mut w, "wait");
    assert_eq!(w.state.rng, once, "one check, for the trap");
    // A warrior of a race without the eye notices neither.
    let mut human = world_in("traptest", "human");
    let before = human.state.rng.clone();
    run(&mut human, "wait");
    assert_eq!(human.state.rng, before, "no check at all");
}

#[test]
fn the_evil_races_start_callous_or_evil_with_a_neutral_class() {
    use crate::alignment::Band;
    for (race, band) in [
        ("human", Band::Neutral),
        ("goblin", Band::Callous),
        ("drow", Band::Evil),
        ("kobold", Band::Callous),
        ("lizardfolk", Band::Callous),
        ("duergar", Band::Evil),
        ("ratfolk", Band::Callous),
    ] {
        // Before the stars: the race's part alone (a warrior's is 0) falls in its band.
        let spec: CharacterSpec = format!("{race},male,warrior").parse().expect("spec");
        let w = World::with_character(CONTENT.clone(), "racetest", Some(1), Some(&spec))
            .expect("world");
        let sign = (w.character().and_then(|c| c.sign.as_ref())).expect("a sign");
        let nudge = CONTENT.horoscope().expect("a horoscope").alignment(sign);
        let mut theirs = w.state().player().clone();
        let score = theirs.alignment.expect("a score").value();
        theirs.alignment = Some(crate::alignment::Score::new(score - nudge));
        assert_eq!(theirs.band(&CONTENT), band, "{race}");
    }
}
