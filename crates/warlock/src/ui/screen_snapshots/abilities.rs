//! Screen snapshot cases (`super`): the abilities menu, and the lists and prompts its rows
//! open.

use super::*;

cases! {
    // What weapon skill and style ranks teach (the combat progression design §4): greyed
    // under the Weapon skills group as firing by themselves, each after its skill's name.
    abilities_menu_weapon_skills: Case {
        name: "abilities-menu-weapon-skills",
        actions: "rank:long_blades:5 rank:sword_and_board:4 rank:shield_bash:5",
        screen: Some("abilities"),
        ..BASE
    },
    // A warrior's actives (#564): Vital strike, Shield rush, and Shield wall usable with their
    // stamina; Lunge greyed, a long sword being no spear.
    abilities_menu_weapon_actives: Case {
        name: "abilities-menu-weapon-actives",
        character: Some("human,male,warrior"),
        kit: true,
        actions: "rank:long_blades:9 rank:sword_and_board:9 rank:spears:9",
        screen: Some("abilities"),
        ..BASE
    },
    // Two shield walls later: the pool spent, the rows greyed short of breath.
    abilities_menu_weapon_actives_short: Case {
        name: "abilities-menu-weapon-actives-short",
        character: Some("human,male,warrior"),
        kit: true,
        actions: "rank:long_blades:9 rank:sword_and_board:9 rank:spears:9 \
                  use:shield_wall use:shield_wall",
        screen: Some("abilities"),
        ..BASE
    },
    // A warrior at level 1 (#1466): Charge among the class abilities, its stamina by the pace,
    // asking for a cell.
    abilities_menu_charge: Case {
        name: "abilities-menu-charge",
        character: Some("human,male,warrior"),
        kit: true,
        actions: "wait",
        screen: Some("abilities"),
        ..BASE
    },
    // A berserker at level 25 (#844): War cry under Rage among the class abilities, greyed
    // while no rage is held.
    abilities_menu_war_cry: Case {
        name: "abilities-menu-war-cry",
        character: Some("human,male,berserker"),
        kit: true,
        actions: "xp:233334 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A wizard at level 4 (spec §8 "Chests"): Knock among the class abilities, asking for a
    // direction, paid in mana.
    abilities_menu_knock: Case {
        name: "abilities-menu-knock",
        character: Some("human,male,wizard"),
        kit: true,
        actions: "xp:2000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger at level 40 (#921): Sic 'em among the class abilities, asking for a cell.
    abilities_menu_sic: Case {
        name: "abilities-menu-sic",
        character: Some("human,male,ranger"),
        kit: true,
        actions: "xp:60000000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A merchant past level 12 (#816): Merchant's eye among the class abilities, asking for
    // something carried.
    abilities_menu_merchants_eye: Case {
        name: "abilities-menu-merchants-eye",
        character: Some("human,male,merchant"),
        kit: true,
        actions: "xp:40000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // The Hunter's Spirit weapon asks for a shape (#881): every melee weapon by weapon skill,
    // as plain ones; the list is the game's objects, whoever opens it.
    shape_list: Case {
        name: "shape-list",
        run: true,
        screen: Some("shape"),
        ..BASE
    },
    shape_list_second_page: Case {
        name: "shape-list-second-page",
        run: true,
        screen: Some("shape"),
        keys: Some(">"),
        ..BASE
    },
    // The wizard's Scribe asks for a spell (#906): the spells known by school, each with its
    // price and its time in words; no blank scroll carried, which the line above says.
    scribe_list: Case {
        name: "scribe-list",
        character: Some("human,female,wizard"),
        actions: "xp:133334 levelup:auto",
        screen: Some("scribe"),
        ..BASE
    },
    // Without Scribe (a warrior), nothing to write.
    scribe_list_none: Case {
        name: "scribe-list-none",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("scribe"),
        ..BASE
    },
    // The craftsperson's Trapwright asks for a rig (#1423): each with what it takes and its
    // time; a knife and a billet carried but no arrows or digging tool, so the tripwire and
    // the pitfall greyed with what is lacking.
    rig_list: Case {
        name: "rig-list",
        character: Some("human,female,craftsperson"),
        actions: "xp:3000000 levelup:auto",
        screen: Some("rig"),
        ..BASE
    },
    // An ascetic past level 12 (#848): Stunning strike among the class abilities, asking for
    // a direction, paid in ki.
    abilities_menu_stunning_strike: Case {
        name: "abilities-menu-stunning-strike",
        character: Some("human,male,ascetic"),
        kit: true,
        actions: "xp:53334 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // An ascetic past level 25 (#850): Stillness, Still mind's active, greyed with nothing to
    // end.
    abilities_menu_stillness: Case {
        name: "abilities-menu-stillness",
        character: Some("human,male,ascetic"),
        kit: true,
        actions: "xp:2000000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A priest at level 1 (#886): the prayer by its first tier's title, Orison, chosen, its
    // description below.
    abilities_priest_1: Case {
        name: "abilities-priest-1",
        character: Some("human,male,priest"),
        kit: true,
        run: true,
        screen: Some("abilities"),
        keys: Some("Down"),
        ..BASE
    },
    // A priest at level 60 (#886): the prayer by its last tier's title, Miracle, chosen,
    // between Consecrate and Sanctuary.
    abilities_priest_60: Case {
        name: "abilities-priest-60",
        character: Some("human,male,priest"),
        kit: true,
        actions: "xp:20000000 levelup:auto",
        screen: Some("abilities"),
        keys: Some("Down Down"),
        ..BASE
    },
    // Beside a forgotten altar (spec §7 "Special rooms"): Pray in the Here group, last.
    abilities_menu_here: Case {
        name: "abilities-menu-here",
        scenario: "shrine",
        actions: "move:N open:N move:N*3",
        screen: Some("abilities"),
        ..BASE
    },
    // Beside a healing spring (spec §7 "Special rooms"): Drink in the Here group, last.
    abilities_menu_spring: Case {
        name: "abilities-menu-spring",
        scenario: "spring",
        actions: "move:N*5",
        screen: Some("abilities"),
        ..BASE
    },
    // Beside the darkness chamber's glowing fungus after a gather (spec §8 "Gathering herbs"):
    // Gather in the Here group, greyed out for Tamsin, untrained in Herbalism (#1400); the
    // message line says why she was refused.
    abilities_menu_gather: Case {
        name: "abilities-menu-gather",
        actions: "move:N*3 open:N move:N*2 move:E*30 open:N move:N*2 move:W*6 gather:W",
        screen: Some("abilities"),
        ..BASE
    },
    // Two allies beside the player (spec §8 "Allies"): the Allies group's three orders, Set on
    // asking for a place.
    abilities_menu_allies: Case {
        name: "abilities-menu-allies",
        actions: "ally:dire_warg:2",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger beside their hound with the darkness chamber's amulet: Give in the Allies group,
    // asking for something carried, then its list, what fits a neck only.
    abilities_menu_give: Case {
        name: "abilities-menu-give",
        character: Some("human,male,ranger"),
        actions: "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 move:S*2 move:W*2 pickup move:E*2 move:N*2 wait*2 move:W",
        screen: Some("abilities"),
        ..BASE
    },
    give_list: Case {
        name: "give-list",
        character: Some("human,male,ranger"),
        actions: "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 move:S*2 move:W*2 pickup move:E*2 move:N*2 wait*2 move:W",
        screen: Some("give"),
        ..BASE
    },
    // An ally in the retinue on the overworld, none on the level: the orders greyed.
    abilities_menu_allies_none: Case {
        name: "abilities-menu-allies-none",
        scenario: "world",
        actions: "ally:dire_warg",
        screen: Some("abilities"),
        ..BASE
    },
    // A druid past Pack leader (#839) who has called two wolves: the calls among the spells,
    // "2 called" with Pack leader's one more, and the Allies group's orders.
    abilities_menu_druid_calls: Case {
        name: "abilities-menu-druid-calls",
        character: Some("human,female,druid"),
        actions: "xp:666667 levelup:auto use:call_of_the_wild",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger below the hall's door: Listen and Wedge listed beside a closed door (left out
    // elsewhere, the owner, #1327).
    abilities_menu_door: Case {
        name: "abilities-menu-door",
        character: Some("human,female,ranger"),
        actions: "move:N*3",
        screen: Some("abilities"),
        ..BASE
    },
    // Set on chosen from the menu: the aiming mode, its prompt in the message line.
    set_on_aim: Case {
        name: "set-on-aim",
        actions: "ally:dire_warg:2",
        keys: Some("a j"),
        ..BASE
    },
    // Come to heel chosen from the menu: the order's words, no time.
    come_to_heel: Case {
        name: "come-to-heel",
        actions: "ally:dire_warg:2",
        keys: Some("a h"),
        ..BASE
    },
    // A paladin of some level: the Class group, Lay on hands refused and greyed out short of
    // zeal (it costs six; the pool was granted at five), Smite ready (three).
    abilities_menu_class: Case {
        name: "abilities-menu-class",
        character: Some("human,male,paladin"),
        // Seed 12: a Unicorn-born paladin (#272: Good, and a Birth sign gift in the menu).
        seed: Some(12),
        actions: "xp:66667 levelup:auto use:lay_on_hands",
        screen: Some("abilities"),
        ..BASE
    },
    // A thief of level 12 (#684, #1463): Lurk ready, Exploit greyed out with no opening; Disarm,
    // Pick lock, and Steal from the class's skills.
    abilities_menu_thief: Case {
        name: "abilities-menu-thief",
        character: Some("human,female,thief"),
        kit: true,
        actions: "xp:66667 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A thief past level 18 (#1465): Thrown exploit beside Exploit, both greyed out with no
    // opening.
    abilities_menu_thief_18: Case {
        name: "abilities-menu-thief-18",
        character: Some("human,female,thief"),
        kit: true,
        actions: "xp:150000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // The thrown exploit's list (#1465): the throw list less flasks, the kit's dagger.
    thrown_exploit_list: Case {
        name: "thrown-exploit-list",
        character: Some("human,female,thief"),
        kit: true,
        actions: "xp:150000 levelup:auto",
        screen: Some("thrown-exploit"),
        ..BASE
    },
    // A priest blessed and drained to their last mana: the spells in their Spells group with
    // the roll each makes now and its cost (the spell scaling design decision 11), those that
    // cost more than is left greyed out with what is left ("3 mana: you have 2").
    abilities_menu_spells: Case {
        name: "abilities-menu-spells",
        character: Some("human,female,priest"),
        actions: "xp:66667 levelup:auto use:blessing drain:101",
        screen: Some("abilities"),
        ..BASE
    },
    // A gnome wizard at Intelligence 19 (Keen) with Empower (level 32; the spell scaling design
    // §11): the rolls up their ladders and a rung more, Magic missile selected, its reach in the
    // description line.
    abilities_menu_spells_grown: Case {
        name: "abilities-menu-spells-grown",
        character: Some("gnome,male,wizard"),
        actions: "xp:666667 levelup:auto afflict:keen",
        screen: Some("abilities"),
        keys: Some("Down Down Down Down"),
        ..BASE
    },
    // A bard of level 25 with too little mana (#244): Throw your voice aimed at a place, Inspire
    // greyed out for inspiration (a song costs one thing, #870), the Lullaby for mana (#873),
    // Music no row (read by rules).
    abilities_menu_bard: Case {
        name: "abilities-menu-bard",
        character: Some("human,female,bard"),
        actions: "xp:266667 levelup:auto drain:106",
        screen: Some("abilities"),
        ..BASE
    },
    // A witch past level 6 (#904): Cackle under Class, nothing to aim, listed usable (the rules
    // refuse it with no hex of hers in sight), beside her hexes in Spells.
    abilities_menu_witch: Case {
        name: "abilities-menu-witch",
        character: Some("human,female,witch"),
        actions: "xp:13334 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A priest bleeding and poisoned (#549): Heal in the Skills group, used on yourself.
    abilities_menu_heal: Case {
        name: "abilities-menu-heal",
        character: Some("human,female,priest"),
        actions: "afflict:bleeding*2 afflict:poisoned*2",
        screen: Some("abilities"),
        ..BASE
    },
    // The same priest choosing Heal: with two things to treat, the dialog asks which.
    heal_dialog: Case {
        name: "heal-dialog",
        character: Some("human,female,priest"),
        actions: "afflict:bleeding*2 afflict:poisoned*2",
        keys: Some("a e"),
        ..BASE
    },
}

/// A witch who tames a hound beside her familiar (a witch's Handle animal is one draw in four:
/// here the content gives it): Give's list, then the way it asks with both beside her, and the
/// fastening once the way is told (the owner, #907 Q4).
#[test]
fn giving_with_two_kept_beasts_beside() {
    let mut files = warlock_core::content::ContentFiles::builtin();
    let witch = include_str!("../../../../../assets/data/classes/10-witch.toml");
    let trained = witch
        .replacen("handle_animal = { chance = 25, dice = \"2d4\" }, ", "", 1)
        .replacen(
            "skills = { heal",
            "skills = { handle_animal = \"+100\", heal",
            1,
        );
    assert_ne!(trained, witch, "the witch's skills");
    files.insert("data/classes/10-witch.toml", &trained);
    files.insert(
        "levels/nook.toml",
        "name = \"Nook\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n##b##\n#.@.#\n#...#\n#####\n'''\n\
         [legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"b\" = { terrain = \"floor\", creature = \"hound\" }\n",
    );
    files.insert(
        "scenarios/nook.toml",
        "name = \"Nook\"\nstart_level = \"nook\"\n\
         character = { race = \"human\", gender = \"female\", class = \"witch\" }\n\
         carry = [\"jade_amulet*3\", \"dried_meat*3\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let cases = [
        ("give-list-two-kept", None),
        ("give-way", Some("a")),
        ("give-way-told", Some("a Up")),
        ("give-way-cancelled", Some("a Esc")),
    ];
    for (name, keys) in cases {
        let case = Case {
            name,
            scenario: "nook",
            actions: "tame:N",
            screen: Some("give"),
            keys,
            ..BASE
        };
        check_with(&case, content.clone());
    }
}
