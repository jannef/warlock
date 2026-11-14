//! Screen snapshot cases (`super`): the map view: look mode, the questions a step asks, hunger's
//! stops, and overworld encounters.

use super::*;

/// Look mode with a wizard's Knowledge (#1030): the skeleton below it says what it resists on the
/// hint's row, in words; the rotting drake above it says nothing more.
#[test]
fn looking_with_knowledge() {
    let mut files = warlock_core::content::ContentFiles::builtin();
    files.insert(
        "levels/look_lore.toml",
        "name = \"Lore\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
         #####\n#s.d#\n#@..#\n#####\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"s\" = { terrain = \"floor\", creature = \"skeleton\" }\n\
         \"d\" = { terrain = \"floor\", creature = \"rotting_drake\" }\n",
    );
    files.insert(
        "scenarios/look_lore.toml",
        "name = \"L\"\nstart_level = \"look_lore\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    for (name, keys) in [
        ("look-knowledge-tells", "l Up"),
        ("look-knowledge-silent", "l Up Right Right"),
    ] {
        let case = Case {
            name,
            scenario: "look_lore",
            character: Some("human,male,wizard"),
            run: true,
            keys: Some(keys),
            ..BASE
        };
        check_with(&case, content.clone());
    }
}

/// A run on the overworld beside the Hall that starts with `nutrition` (spec §8 "Hunger"), or if
/// that is below 0 (no run starts so hungry), has just begun to starve.
fn on_the_road(nutrition: i32) -> (Case, Arc<Content>) {
    let mut files = warlock_core::content::ContentFiles::builtin();
    let hunger = include_str!("../../../../../assets/data/hunger.toml")
        .replace("start = 3600", &format!("start = {}", nutrition.max(0)));
    files.insert("data/hunger.toml", &hunger);
    let content = Arc::new(Content::load(&files).expect("content"));
    let case = Case {
        scenario: "world",
        run: true,
        actions: if nutrition < 0 { "rest:2m wait" } else { "" },
        ..BASE
    };
    (case, content)
}

/// The app of [`on_the_road`].
fn road_app(nutrition: i32) -> App {
    let (case, content) = on_the_road(nutrition);
    start(&case, content)
}

/// #972: the road east beside the Hall, a step short of each worse state (the `--more--` stop,
/// with the status bar's word), and the question a journey asks while Fainting and Starving.
#[test]
fn hunger_stops_as_text() {
    let cases = [
        ("hunger-more-hungry", 1441, "Right"),
        ("hunger-more-weak", 721, "Right"),
        ("hunger-more-fainting", 241, "Right"),
        ("hunger-more-starving", 1, "Right y"),
        ("hunger-dialog-fainting", 100, "Right"),
        ("hunger-dialog-starving", -1, "Right"),
    ];
    for (name, nutrition, keys) in cases {
        let (road, content) = on_the_road(nutrition);
        let case = Case {
            name,
            keys: Some(keys),
            status_bar: true,
            ..road
        };
        check_with(&case, content);
    }
}

/// Beside what offers an action, north of the player (spec §7 "Special rooms"): its scenario and
/// the walk there.
const OFFERS: [(&str, &str, &str); 2] = [
    ("offer-dialog-altar", "shrine", "move:N open:N move:N*3"),
    ("offer-dialog-spring", "spring", "move:N*5"),
];

/// #975: the question a step into each asks.
#[test]
fn walking_into_what_offers_an_action_as_text() {
    let content = Arc::new(Content::builtin().expect("content"));
    for (name, scenario, actions) in OFFERS {
        let case = Case {
            name,
            scenario,
            actions,
            keys: Some("Up"),
            ..BASE
        };
        check_with(&case, content.clone());
    }
}

/// #1387: a step into an anvil asks nothing. The forge's craftsperson, trained in Metalcraft, is
/// shown its screen; a thief, untrained, is told what it takes.
#[test]
fn walking_into_an_anvil_as_text() {
    let content = Arc::new(Content::builtin().expect("content"));
    for (name, character) in [
        ("anvil-opens-metalcraft", None),
        ("anvil-untrained", Some("human,male,thief")),
    ] {
        let case = Case {
            name,
            scenario: "forge",
            character,
            actions: "move:E",
            keys: Some("Up"),
            ..BASE
        };
        check_with(&case, content.clone());
    }
}

/// #1019: a step into deep water asks, always, in the one dialog. Stay back is selected first.
#[test]
fn diving_into_deep_water_as_text() {
    let content = Arc::new(Content::builtin().expect("content"));
    let case = Case {
        name: "deep-water-dialog",
        scenario: "quarry",
        character: Some("lizardfolk,female,warrior"),
        actions: "move:E*27 move:S",
        keys: Some("Right"),
        ..BASE
    };
    check_with(&case, content);
}

/// Where the player stands and how many actions were applied.
fn walked(app: &App) -> (Pos, u64) {
    let game = app.world().resource::<Game>();
    (game.world.player_pos(), game.applied)
}

/// A held key's repeat of `key`.
fn repeat(app: &mut App, key: &str) {
    let press = KeyboardInput {
        repeat: true,
        ..press(key)
    };
    app.world_mut().write_message(press);
    app.update();
    app.update();
}

#[test]
fn a_worsening_state_stops_held_keys_at_more() {
    let mut app = road_app(1441);
    let start_at = walked(&app);
    press_keys(&mut app, "Right");
    let stopped = walked(&app);
    assert_ne!(stopped, start_at, "the step is taken");
    assert_eq!(*app.world().resource::<InputMode>(), InputMode::More(0));
    // The message fits the line; it waits all the same, and neither a repeat nor another step
    // goes past it.
    assert!(text(app.world().resource::<Console>()).contains("hungry. --more--"));
    repeat(&mut app, "Right");
    press_keys(&mut app, "Right");
    assert_eq!(walked(&app), stopped);
    assert_eq!(*app.world().resource::<InputMode>(), InputMode::More(0));
    // A repeat of the key that reads it does not read it, either.
    repeat(&mut app, "Enter");
    assert_eq!(*app.world().resource::<InputMode>(), InputMode::More(0));
    press_keys(&mut app, "Enter");
    assert_eq!(*app.world().resource::<InputMode>(), InputMode::Normal);
    assert!(!text(app.world().resource::<Console>()).contains("--more--"));
    // Read, the walk goes on, and the next step is no stop: it stays Hungry.
    repeat(&mut app, "Right");
    assert_ne!(walked(&app), stopped);
    assert_eq!(*app.world().resource::<InputMode>(), InputMode::Normal);
}

#[test]
fn steps_queued_behind_a_worsening_state_are_dropped() {
    let mut app = road_app(1441);
    let (_, applied) = walked(&app);
    press_keys(&mut app, "Right Right Right");
    assert_eq!(walked(&app).1, applied + 1);
    // Steps that change nothing all go through.
    let mut app = road_app(3000);
    let (_, applied) = walked(&app);
    press_keys(&mut app, "Right Right");
    assert_eq!(walked(&app).1, applied + 2);
    assert_eq!(*app.world().resource::<InputMode>(), InputMode::Normal);
}

#[test]
fn staying_takes_no_time_and_going_on_is_the_step() {
    for nutrition in [100, -1] {
        for stay in ["Esc", "n", "Enter"] {
            let mut app = road_app(nutrition);
            let before = walked(&app);
            let time = app.world().resource::<Game>().world.time();
            press_keys(&mut app, "Right");
            assert!(!app.world().resource::<ScreenStack>().is_empty(), "it asks");
            press_keys(&mut app, stay);
            assert!(app.world().resource::<ScreenStack>().is_empty());
            assert_eq!(walked(&app), before, "{stay}");
            assert_eq!(app.world().resource::<Game>().world.time(), time, "{stay}");
        }
        let mut app = road_app(nutrition);
        let before = walked(&app);
        press_keys(&mut app, "Right");
        press_keys(&mut app, "y");
        assert_ne!(walked(&app).0, before.0);
    }
    // Weak walks on, unasked.
    let mut app = road_app(500);
    let before = walked(&app);
    press_keys(&mut app, "Right");
    assert!(app.world().resource::<ScreenStack>().is_empty());
    assert_ne!(walked(&app).0, before.0);
}

#[test]
fn a_held_key_never_opens_the_question_nor_walks_past_it() {
    let mut app = road_app(100);
    let before = walked(&app);
    repeat(&mut app, "Right");
    assert!(app.world().resource::<ScreenStack>().is_empty());
    assert_eq!(walked(&app), before);
}

#[test]
fn the_status_bar_draws_each_hunger_word_in_its_colour() {
    let cases = [
        (1000, "Hungry", HUNGRY),
        (500, "Weak", WEAK),
        (100, "Fainting", FAINTING),
        (-1, "Starving", STARVING),
    ];
    for (nutrition, word, colour) in cases {
        let app = road_app(nutrition);
        let console = app.world().resource::<Console>();
        let y = console.rows() as i32 - 1;
        let line: String = (0..console.cols() as i32)
            .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
            .collect();
        let at = line.find(word).expect("the word is shown");
        let x = line[..at].chars().count() as i32;
        let cell = |x| console.get(x, y).expect("a cell");
        assert_eq!(cell(x).fg, colour, "{word}");
        assert_eq!(cell(x + word.len() as i32 - 1).fg, colour, "{word}");
        assert_eq!(cell(1).fg, TEXT, "the rest of the line is as it was");
    }
}

/// The built-in content with an encounter table in the showcase campaign (which ships none):
/// kobolds (hostile; and talkers, the same flagged `talk_past`), deer (wildlife), and the hermit
/// (peaceful), met by the dev `meet`.
fn encounter_content() -> Arc<Content> {
    const TABLE: &str = r#"
[[table]]
id = "wilds"
[[table.entry]]
id = "kobolds"
name = "a band of kobolds"
weight = 1
group = [{ creature = "kobold", count = "3" }]
[[table.entry]]
id = "talkers"
name = "a band of talkers"
weight = 1
group = [{ creature = "kobold", count = "2" }]
talk_past = true
[[table.entry]]
id = "deer"
name = "a herd of deer"
weight = 1
stance = "wildlife"
group = [{ creature = "deer", count = "2" }]
[[table.entry]]
id = "hermit"
name = "a hermit"
weight = 1
stance = "peaceful"
group = [{ creature = "hermit", count = "1" }]
"#;
    let mut files = warlock_core::content::ContentFiles::builtin();
    files.insert("campaigns/showcase/data/encounters/test.toml", TABLE);
    Arc::new(Content::load(&files).expect("content"))
}

/// The encounter dialog (#805) over the overworld beside the Hall, as each outcome asks it.
/// Esc and an info page's number do nothing over it: it must be answered (the owner, #796 Q9).
#[test]
fn encounters_as_text() {
    let content = encounter_content();
    let cases = [
        ("encounter-drop", "meet:kobolds:drop", "Esc 1"),
        ("encounter-met", "meet:kobolds:met", ""),
        ("encounter-peaceful", "meet:hermit", ""),
        ("encounter-wildlife", "meet:deer", ""),
    ];
    for (name, actions, keys) in cases {
        let case = Case {
            name,
            scenario: "world",
            actions,
            keys: Some(keys),
            ..BASE
        };
        check_with(&case, content.clone());
    }
    // A bard (trained in Bluff) met face to face by a party flagged `talk_past` (#1017).
    let case = Case {
        name: "encounter-talk-past",
        scenario: "world",
        character: Some("human,male,bard"),
        actions: "meet:talkers:met",
        keys: Some(""),
        ..BASE
    };
    check_with(&case, content.clone());
}

/// A row of the dialog answers the encounter: the dialog closes and the world has no question
/// left; the area opens for a fight.
#[test]
fn an_encounter_row_answers_it() {
    let case = Case {
        scenario: "world",
        actions: "meet:kobolds:met",
        ..BASE
    };
    let mut app = start(&case, encounter_content());
    assert!(app.world().resource::<ScreenStack>().asks_required());
    press_keys(&mut app, "Down Enter");
    assert!(!app.world().resource::<ScreenStack>().asks_required());
    let game = app.world().resource::<Game>();
    assert!(game.world.perception().encounter().is_none());
    // Osric's companion steps out of the retinue after the answer's own line.
    assert!(
        (game.world.messages().iter()).any(|m| m.text == "You stand your ground."),
        "{:?}",
        game.world
            .messages()
            .iter()
            .map(|m| m.text.as_str())
            .collect::<Vec<_>>()
    );
}
