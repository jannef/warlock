//! Screen snapshot cases (`super`): level-ups, skill rolls, talents, the background story,
//! and birth signs.

use super::*;

cases! {
    // Progression: a warrior gaining levels 2 and 3 at once (the level-up screen opens by
    // itself), their talent pick, and the Talents page.
    level_up_rolls: Case {
        name: "level-up-rolls",
        character: Some("human,male,warrior"),
        actions: "xp:461",
        size: (100, 40),
        ..BASE
    },
    // The skill-roll step at the smallest console (#543): a thief's search marks set a
    // bigger die, and each skill shows the dice a roll on it adds.
    level_up_rolls_marks: Case {
        name: "level-up-rolls-marks",
        character: Some("human,female,thief"),
        actions: "search:100*2 xp:461",
        size: (80, 25),
        ..BASE
    },
    // A skill past the soft cap rises by half its roll: `(dice)/2`.
    level_up_rolls_halved: Case {
        name: "level-up-rolls-halved",
        character: Some("human,male,warrior"),
        actions: "xp:200000 levelup:auto xp:400000",
        size: (80, 25),
        ..BASE
    },
    level_up_talent: Case {
        name: "level-up-talent",
        character: Some("human,male,warrior"),
        actions: "xp:461 advance:athletics*6",
        size: (100, 40),
        ..BASE
    },
    level_up_choice: Case {
        name: "level-up-choice",
        character: Some("human,male,warrior"),
        actions: "xp:461 advance:athletics*6",
        // Skill focus's letter on the talent list.
        keys: Some("p"),
        size: (100, 40),
        ..BASE
    },
    talents_screen: Case {
        name: "talents-screen",
        character: Some("dwarf,male,warrior"),
        run: true,
        screen: Some("talents"),
        size: (100, 40),
        ..BASE
    },
    // Two profane gifts (#1625): the Profane group last among the player's own, a boon and a
    // bane (a flaw).
    talents_screen_profane: Case {
        name: "talents-screen-profane",
        character: Some("dwarf,male,warrior"),
        actions: "gift:oak_hearted gift:hungering",
        screen: Some("talents"),
        size: (100, 40),
        ..BASE
    },
    // A level-3 wizard's pick, School focus for evocation: the Spellcraft tree, Arcane
    // training among the class abilities (#400).
    talents_screen_spellcraft: Case {
        name: "talents-screen-spellcraft",
        character: Some("human,male,wizard"),
        actions: "xp:587 talent:school_focus:evocation advance:spellcraft*8",
        screen: Some("talents"),
        size: (100, 40),
        ..BASE
    },
    // The Ascmearc's pedlar met on the road east of Cēaptūn (#806): the one dialog, which
    // must be answered.
    encounter_pedlar: Case {
        name: "encounter-pedlar",
        scenario: "ascmearc",
        actions: "move:E*7 meet:road_pedlar",
        ..BASE
    },
    // A story talent (#707): the flux gave this warrior Hardy, "(story)".
    talents_screen_story: Case {
        name: "talents-screen-story",
        scenario: "ascmearc",
        seed: Some(107),
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("talents"),
        size: (100, 40),
        ..BASE
    },
    // The background story (#703): opened by the app as the run New run's summary started
    // begins (the showcase's character, its seed), then page 9 of a kit run, and page 9 of a
    // scenario's own character, who has none.
    story_screen: Case {
        name: "story-screen",
        screen: Some("summary"),
        keys: Some("Enter"),
        size: (100, 36),
        ..BASE
    },
    story_page: Case {
        name: "story-page",
        character: Some("dwarf,female,warrior"),
        run: true,
        screen: Some("character"),
        keys: Some("9"),
        size: (112, 36),
        ..BASE
    },
    story_page_none: Case {
        name: "story-page-none",
        run: true,
        screen: Some("character"),
        keys: Some("9"),
        ..BASE
    },
    // A Unicorn (seed 12) after Horn's grace: the Birth sign group, the grace greyed out while
    // Graceless (#266).
    abilities_menu_sign: Case {
        name: "abilities-menu-sign",
        seed: Some(12),
        character: Some("human,male,warrior"),
        actions: "afflict:bleeding*2 afflict:poisoned*3 use:horns_grace",
        screen: Some("abilities"),
        ..BASE
    },
    // A Fire-born warrior (a Fire Kraken) after a Blaze: the Birth sign group, burnt out.
    abilities_menu_blaze: Case {
        name: "abilities-menu-blaze",
        seed: Some(15),
        character: Some("human,male,warrior"),
        actions: "use:blaze",
        screen: Some("abilities"),
        ..BASE
    },
    // A Wyvern warrior (seed 2) who picked Fire breath (#270) and breathed it: the Birth sign
    // group, drained.
    abilities_menu_fire_breath: Case {
        name: "abilities-menu-fire-breath",
        seed: Some(2),
        character: Some("human,male,warrior"),
        actions: "xp:200000 talent:fire_breath levelup:auto use:fire_breath:11,15",
        screen: Some("abilities"),
        ..BASE
    },
    // A Barghest warrior (seed 21) in the arena who picked Hound's bay (#270) and howled: the
    // kobolds who heard it flee.
    abilities_menu_hounds_bay: Case {
        name: "abilities-menu-hounds-bay",
        scenario: "arena",
        seed: Some(21),
        character: Some("human,male,warrior"),
        actions: "xp:200000 talent:hounds_bay levelup:auto use:hounds_bay",
        screen: Some("abilities"),
        ..BASE
    },
    // A wizard with Blood magic (#242), after giving blood for mana: the Talents group.
    abilities_menu_blood_magic: Case {
        name: "abilities-menu-blood-magic",
        seed: Some(4),
        character: Some("elf,female,wizard"),
        actions: "xp:133334 talent:blood_magic levelup:auto drain:20 use:blood_magic",
        screen: Some("abilities"),
        ..BASE
    },
}
