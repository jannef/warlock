//! Screen snapshot cases (`super`): the main menu, the saved characters, the graveyard, New run,
//! settings, and Controls.

use super::*;

/// A few of the fallen as the file keeps them, the oldest first: one written before experience
/// was kept (it reads 0 and sorts last), a prestige class, and causes of each kind.
const GRAVEYARD_XP: &str = r#"
[[fallen]]
name = "Wynn"
race = "human"
class = "thief"
level = 4
cause = "starved"
place = "Cealdhlāw 2, The Ascmearc"
age = 19
when = 1790000000
[[fallen]]
name = "Brakka"
race = "dwarf"
class = "warrior"
level = 7
xp = 2900
cause = "stone"
place = "Fyrdelf 3, The Ascmearc"
age = 31
when = 1793318400
[[fallen]]
name = "Tamsin"
race = "elf"
class = "bard"
prestige = "hunter"
level = 31
xp = 412000
cause = "kobold"
killer = "kobold"
place = "Cēaptūn, The Ascmearc"
age = 140
when = 1793404800
[[fallen]]
name = "Hild"
race = "human"
class = "priest"
level = 5
xp = 1450
cause = "bleeding"
place = "Holtwīc, The Ascmearc"
age = 22
when = 1794000000
"#;

cases! {
    // Menus and settings.
    main_menu: Case {
        name: "main-menu",
        screen: Some("main"),
        ..BASE
    },
    main_menu_saved: Case {
        name: "main-menu-saved",
        screen: Some("main"),
        save_dir: Some("tests/golden/saved-characters"),
        ..BASE
    },
    characters_screen: Case {
        name: "characters-screen",
        screen: Some("characters"),
        save_dir: Some("tests/golden/saved-characters"),
        ..BASE
    },
    // The count of the dead under the list (the graveyard, #1137).
    characters_screen_graveyard: Case {
        name: "characters-screen-graveyard",
        screen: Some("characters"),
        save_dir: Some("tests/golden/saved-characters"),
        graveyard: Some(GRAVEYARD),
        ..BASE
    },
    // The main menu offers the Graveyard once anyone has died, and it lists the fallen, the
    // most experienced first (#1252).
    main_menu_graveyard: Case {
        name: "main-menu-graveyard",
        screen: Some("main"),
        graveyard: Some(GRAVEYARD_XP),
        ..BASE
    },
    graveyard_screen: Case {
        name: "graveyard-screen",
        screen: Some("graveyard"),
        graveyard: Some(GRAVEYARD_XP),
        ..BASE
    },
    graveyard_screen_paged: Case {
        name: "graveyard-screen-paged",
        screen: Some("graveyard"),
        graveyard: Some(GRAVEYARD),
        ..BASE
    },
    graveyard_screen_empty: Case {
        name: "graveyard-screen-empty",
        screen: Some("graveyard"),
        ..BASE
    },
    // Every scenario under its campaign's name (a developer build), the start first.
    new_run_screen: Case {
        name: "new-run-screen",
        screen: Some("new-run"),
        size: (100, 34),
        ..BASE
    },
    pause_menu: Case {
        name: "pause-menu",
        actions: "wait",
        screen: Some("pause"),
        ..BASE
    },
    // Under the open sky the pause menu's line on the world tells the weather too (#1679).
    pause_menu_outdoors: Case {
        name: "pause-menu-outdoors",
        scenario: "courtyard",
        actions: "weather:rain wait",
        screen: Some("pause"),
        ..BASE
    },
    help_screen: Case {
        name: "help-screen",
        actions: "wait",
        screen: Some("help"),
        size: (120, 44),
        ..BASE
    },
    settings_screen: Case {
        name: "settings-screen",
        actions: "wait",
        screen: Some("settings"),
        ..BASE
    },
    controls_screen: Case {
        name: "controls-screen",
        actions: "wait",
        screen: Some("controls"),
        size: (120, 44),
        ..BASE
    },
    // Up wraps to the last row: the abilities' own keys at the very bottom (#644).
    controls_screen_bottom: Case {
        name: "controls-screen-bottom",
        actions: "wait",
        screen: Some("controls"),
        keys: Some("Up"),
        size: (120, 44),
        ..BASE
    },
    // A play key rebound with Ctrl held (#1323): Move north's first key becomes Ctrl+k, and the
    // message says so (the Run and Search for a while defaults hold Alt, #1385).
    controls_screen_ctrl_key: Case {
        name: "controls-screen-ctrl-key",
        actions: "wait",
        screen: Some("controls"),
        keys: Some("Enter Enter Ctrl+k"),
        size: (120, 44),
        ..BASE
    },
    // A command's keys open under it (#1324): Wait/search's `s` added to Move north moves, and
    // the message says from where; "Add a key" waits below the new key.
    controls_screen_keys: Case {
        name: "controls-screen-keys",
        actions: "wait",
        screen: Some("controls"),
        keys: Some("Enter Up Enter s"),
        size: (120, 44),
        ..BASE
    },
}
