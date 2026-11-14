//! Screen snapshot cases (`super`): scrolls, wands, and rods.

use super::*;

cases! {
    // Scrolls.
    scroll_read_list: Case {
        name: "scroll-read-list",
        scenario: "traps",
        actions: "move:E pickup",
        screen: Some("read"),
        ..BASE
    },
    scroll_identify_list: Case {
        name: "scroll-identify-list",
        scenario: "traps",
        actions: "move:E pickup",
        keys: Some("r d"),
        ..BASE
    },
    // The Decipher ability's list (spec §8 "Scrolls"): the scrolls carried of a type not yet made
    // out; a wizard is trained in Decipher script.
    scroll_decipher_list: Case {
        name: "scroll-decipher-list",
        scenario: "traps",
        character: Some("human,male,wizard"),
        actions: "move:E pickup",
        screen: Some("decipher"),
        ..BASE
    },
    scroll_too_close: Case {
        name: "scroll-too-close",
        scenario: "traps",
        // The fireballs identified first: an unknown scroll is read without a warning.
        actions: "drop:a move:E pickup read:e:b",
        keys: Some("r a k Enter"),
        ..BASE
    },
    // Wands (spec §8 "Magic items": "Wands"): the trap hall's two, unknown, then identified;
    // aiming one from the missile weapon slot.
    wand_description_unidentified: Case {
        name: "wand-description-unidentified",
        scenario: "traps",
        actions: "drop:a move:E move:E pickup",
        screen: Some("describe"),
        ..BASE
    },
    wand_description: Case {
        name: "wand-description",
        scenario: "traps",
        actions: "drop:a move:E move:E pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    wand_aim: Case {
        name: "wand-aim",
        scenario: "traps",
        actions: "drop:a move:E move:E pickup identify equip:missile_weapon:a",
        keys: Some("t"),
        ..BASE
    },
    // Rods (spec §8 "Magic items": "Rods"): the trap hall's two, identified; aiming one from the
    // missile weapon slot shows its reach.
    rod_description: Case {
        name: "rod-description",
        scenario: "traps",
        actions: "drop:a move:E move:E move:E pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    rod_aim: Case {
        name: "rod-aim",
        scenario: "traps",
        actions: "drop:a move:E move:E move:E pickup identify equip:missile_weapon:a",
        keys: Some("t"),
        ..BASE
    },
}
