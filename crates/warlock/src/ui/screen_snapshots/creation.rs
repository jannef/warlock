//! Screen snapshot cases (`super`): character creation, race to summary.

use super::*;

cases! {
    // Character creation.
    race_screen: Case {
        name: "race-screen",
        screen: Some("race"),
        ..BASE
    },
    race_screen_drow: Case {
        name: "race-screen-drow",
        screen: Some("race"),
        keys: Some("Down Down Down Down Down Down"),
        ..BASE
    },
    sign_screen: Case {
        name: "sign-screen",
        screen: Some("sign"),
        ..BASE
    },
    sign_screen_lucky: Case {
        name: "sign-screen-lucky",
        screen: Some("new-run"),
        keys: Some("4 4 0 Enter Enter"),
        size: (100, 36),
        ..BASE
    },
    gender_screen: Case {
        name: "gender-screen",
        screen: Some("gender"),
        keys: Some("Down"),
        ..BASE
    },
    class_screen: Case {
        name: "class-screen",
        screen: Some("class"),
        ..BASE
    },
    // Only the classes open to the race and gender (#1279): a drow man is no priest or
    // paladin, a dwarf no witch; the everyday folk are never listed.
    class_screen_drow_male: Case {
        name: "class-screen-drow-male",
        screen: Some("race"),
        keys: Some("Down Down Down Down Down Down Enter Enter Up Enter"),
        ..BASE
    },
    class_screen_dwarf: Case {
        name: "class-screen-dwarf",
        screen: Some("race"),
        keys: Some("Down Enter Enter Enter"),
        ..BASE
    },
    // The showcase's thief: Random preselected; a letter marks a talent.
    chargen_talents_screen: Case {
        name: "chargen-talents-screen",
        screen: Some("chargen-talents"),
        size: (100, 40),
        ..BASE
    },
    chargen_talents_marked: Case {
        name: "chargen-talents-marked",
        screen: Some("chargen-talents"),
        keys: Some("b"),
        size: (100, 40),
        ..BASE
    },
    name_screen: Case {
        name: "name-screen",
        screen: Some("name"),
        keys: Some("B r a k k a"),
        ..BASE
    },
    character_summary: Case {
        name: "character-summary",
        screen: Some("summary"),
        ..BASE
    },
    character_summary_other: Case {
        name: "character-summary-other",
        screen: Some("gender"),
        keys: Some("Down Enter Enter Enter Enter"),
        ..BASE
    },
    kit_summary: Case {
        name: "kit-summary",
        scenario: "world",
        screen: Some("new-run"),
        keys: Some(
            "Up Enter Down Down Enter Enter Enter Down Down Down Down Down Enter Enter Enter",
        ),
        size: (100, 36),
        ..BASE
    },
}
