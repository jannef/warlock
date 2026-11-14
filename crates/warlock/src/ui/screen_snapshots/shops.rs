//! Screen snapshot cases (`super`): shops, traps, and death.

use super::*;

/// At the courtyard's shop door.
macro_rules! shop {
    () => {
        "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 move:E*5"
    };
}

cases! {
    // Shops, traps, death.
    shop_unpaid: Case {
        name: "shop-unpaid",
        scenario: "courtyard",
        actions: concat!(shop!(), " move:E move:NE pickup move:E move:E pickup"),
        screen: Some("inventory"),
        ..BASE
    },
    // Selling is a drop in a shop: marked in the drop mode, Enter asks.
    shop_sell: Case {
        name: "shop-sell",
        scenario: "courtyard",
        actions: concat!(shop!(), " move:E move:NE pickup move:SW move:W"),
        screen: Some("inventory"),
        keys: Some("D b Enter"),
        ..BASE
    },
    shop_leaving: Case {
        name: "shop-leaving",
        scenario: "courtyard",
        actions: concat!(shop!(), " move:E move:NE pickup move:W move:SW"),
        keys: Some("Left"),
        ..BASE
    },
    // The one trap known beside the player: `a c` (Disarm a trap, after the thief's birth
    // sign's stare and Exploit, #266) disarms it at once, with no prompt (#1327).
    trap_disarm_at_once: Case {
        name: "trap-disarm-at-once",
        actions: "move:N*3 open:N move:N*2 move:E*67 open:S move:S*7 move:SW descend",
        keys: Some("a c"),
        ..BASE
    },
    // Two beside the thief to rob: `a e` (Steal) asks which way in the message line (#1327).
    steal_prompt: Case {
        name: "steal-prompt",
        actions: "ally:dire_warg:2",
        keys: Some("a e"),
        ..BASE
    },
    death_screen: Case {
        name: "death-screen",
        scenario: "arena",
        // Pip's birth sign has gifts (#265): seed 3 is one where the fight still ends in his death.
        seed: Some(3),
        actions: "use_tool wait*27",
        ..BASE
    },
    // The death remembered in the graveyard (#1137).
    death_screen_graveyard: Case {
        name: "death-screen-graveyard",
        scenario: "arena",
        seed: Some(3),
        actions: "use_tool wait*27",
        graveyard: Some(GRAVEYARD),
        ..BASE
    },
    death_screen_lucky: Case {
        name: "death-screen-lucky",
        scenario: "arena",
        // Seed 142 gives Pip a lucky day, the Iron Scourge (#269), and the fight his death.
        seed: Some(142),
        actions: "use_tool wait*20",
        ..BASE
    },
}
