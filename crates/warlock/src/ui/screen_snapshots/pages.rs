//! Screen snapshot cases (`super`): the character's info pages.

use super::*;

cases! {
    // The character's pages.
    character_screen: Case {
        name: "character-screen",
        actions: "move:SW*2 pickup",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // A witch and her familiar: the With row and the status bar's segment (#929).
    character_screen_familiar: Case {
        name: "character-screen-familiar",
        status_bar: true,
        character: Some("human,female,witch"),
        run: true,
        screen: Some("character"),
        ..BASE
    },
    character_screen_lucky: Case {
        name: "character-screen-lucky",
        // Seed 440 gives Tamsin a lucky day (#269): the Deep One, the title after the name.
        seed: Some(440),
        run: true,
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // A Tin of the Weyfarers' Gilde (#425): the rank under the alignment.
    character_guild: Case {
        name: "character-guild",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 move:E*9 open:S move:S*5 ask:S:quest-guild_quarry",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // The attacks beside the stats (the showcase's own character has bare hands): a sword and
    // shield, a bow with a sword, and two weapons with a bow.
    character_attacks_sword_and_shield: Case {
        name: "character-attacks-sword-and-shield",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    character_attacks_bow: Case {
        name: "character-attacks-bow",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    character_attacks_two_weapons: Case {
        name: "character-attacks-two-weapons",
        character: Some("human,male,ranger"),
        actions: "move:SW*2 pickup equip:off_hand:k",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    status_effects: Case {
        name: "status-effects",
        actions: "afflict:poisoned*4 afflict:bleeding wait",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // Taken by surprise (#681): the status's line in the message line, Surprised in the status
    // bar, never a number.
    status_surprised: Case {
        name: "status-surprised",
        status_bar: true,
        actions: "afflict:surprised",
        ..BASE
    },
    // A potion's boost (#567): Fortified by name in the Status row and the status bar, the
    // Strength score raised, no numbers.
    potion_boost: Case {
        name: "potion-boost",
        status_bar: true,
        actions: "afflict:fortified afflict:owl_eyed wait",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    diseased: Case {
        name: "diseased",
        actions: "infect:the_shakes rest:24h*3",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    resources: Case {
        // Alternative resources (#225): a row each under the stance, and in the status bar.
        name: "resources",
        status_bar: true,
        actions: "resource:rage resource:focus wait",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    frenzy: Case {
        // A berserker's rage brings the frenzy (#680): the stance in the status bar and the
        // Stance row, the change refused while the rage holds it.
        name: "frenzy",
        status_bar: true,
        character: Some("human,male,berserker"),
        actions: "stance:coward use:rage stance:neutral",
        screen: Some("character"),
        size: (120, 36),
        ..BASE
    },
    alignment_band_character: Case {
        // A band crossed (the `fall` line) and the Alignment row: seed 9's Basilisk-born warrior
        // (#272) starts Neutral, so the kick takes him down to Callous. A warrior, as a paladin's
        // zeal deserting him pushes the line behind the pager (#910, #914).
        name: "alignment-band-character",
        scenario: "courtyard",
        character: Some("human,male,warrior"),
        seed: Some(9),
        actions: "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 \
            move:E*5 kick:N",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    skills_screen: Case {
        name: "skills-screen",
        run: true,
        screen: Some("skills"),
        ..BASE
    },
    // Knowledge chosen (#667): its description says in words what it does against creatures.
    skills_screen_knowledge: Case {
        name: "skills-screen-knowledge",
        character: Some("human,female,wizard"),
        run: true,
        screen: Some("skills"),
        keys: Some("Down Down"),
        ..BASE
    },
    // A wizard's (#668): Concentration first, its description saying what it gives in words.
    skills_screen_wizard: Case {
        name: "skills-screen-wizard",
        run: true,
        character: Some("elf,female,wizard"),
        screen: Some("skills"),
        ..BASE
    },
    weapon_skills_screen: Case {
        name: "weapon-skills-screen",
        scenario: "arena",
        actions: "move:E pickup equip:missile_weapon:b equip:missiles:c fire:30,16*12",
        screen: Some("weapon-skills"),
        ..BASE
    },
    // A wizard's (#571): Sniper from the class, not listed here (the caster styles are the
    // Magic page's).
    weapon_skills_screen_wizard: Case {
        name: "weapon-skills-screen-wizard",
        character: Some("human,female,wizard"),
        run: true,
        screen: Some("weapon-skills"),
        ..BASE
    },
    // Two Shields (#611), hidden until its first marks: here at the rank the dev action sets,
    // listed among the styles.
    weapon_skills_screen_two_shields: Case {
        name: "weapon-skills-screen-two-shields",
        actions: "rank:two_shields:1",
        screen: Some("weapon-skills"),
        ..BASE
    },
    // The Magic page (the spell scaling design §11): the showcase wizard's mana, the caster
    // styles (Sniper from the class), and the spells with their rolls.
    magic_screen: Case {
        name: "magic-screen",
        character: Some("human,female,wizard"),
        run: true,
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // A wizard of level 18 with ranks set as risen ones are: more spells, climbed bars.
    magic_screen_trained: Case {
        name: "magic-screen-trained",
        character: Some("human,female,wizard"),
        actions: "xp:133334 levelup:auto rank:sniper:5 rank:toucher:3",
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // A paladin in the forge's chain mail, which hinders casting: Battle caster marked.
    magic_screen_armoured: Case {
        name: "magic-screen-armoured",
        scenario: "forge",
        character: Some("human,male,paladin"),
        actions: "move:SW move:S pickup equip:body:m",
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // A warrior: no caster styles, no spells.
    magic_screen_none: Case {
        name: "magic-screen-none",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // An 80 by 14 console: the short tab line, and the page of a wizard of level 18 scrolls,
    // its gaps given way (at 80 by 20 it fits).
    magic_screen_short: Case {
        name: "magic-screen-short",
        character: Some("human,female,wizard"),
        actions: "xp:133334 levelup:auto",
        screen: Some("magic"),
        size: (80, 14),
        ..BASE
    },
    abilities_menu: Case {
        name: "abilities-menu",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
            pickup:a equip:off_hand:b",
        screen: Some("abilities"),
        ..BASE
    },
}
