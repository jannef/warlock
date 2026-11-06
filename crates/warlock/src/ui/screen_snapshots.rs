//! Text snapshots of the UI console (#301): each case starts a run as a golden entry would
//! (scenario, character, kit, actions, `--screen`, `--keys`), runs the real router and screen
//! systems headlessly, and snapshots the UI console as text with `insta`. The map layer is not in
//! it (map content is `warlock-cli dump --layers view`), nor colors.
//!
//! Screens belong here, not in `tests/golden/`: a text diff is exact, readable in review, and
//! needs no GPU. Add a case to `CASES`, run `cargo test -p warlock screen_snapshots`, and review
//! the new snapshot with `cargo insta review` (or `INSTA_UPDATE=always`, then read the diff).

use super::*;
use crate::cli::Args;
use crate::game::{GameContent, PlayerAction};
use crate::save::{SaveCommand, SaveStore};
use crate::ui::screens::{MenuChoice, ScreenStack};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use clap::Parser;
use std::sync::Arc;
use warlock_core::content::Content;

/// One snapshot: a run (or the main menu) and what is open over it.
pub(super) struct Case {
    pub(super) name: &'static str,
    pub(super) scenario: &'static str,
    /// `--seed`; none: the scenario's.
    pub(super) seed: Option<u64>,
    /// `--character`; the run starts with its kit.
    pub(super) character: Option<&'static str>,
    /// Starts with the character's starting kit without naming a character.
    pub(super) kit: bool,
    /// Action text applied before the app starts; empty opens the main menu, unless `run`.
    pub(super) actions: &'static str,
    /// Starts the run even with no actions (screens over a run at its first turn).
    pub(super) run: bool,
    /// `--screen`.
    pub(super) screen: Option<&'static str>,
    /// `--keys`, pressed once the screens are open.
    pub(super) keys: Option<&'static str>,
    /// `--save-dir`, relative to the workspace root.
    pub(super) save_dir: Option<&'static str>,
    /// The UI console's size (columns, rows).
    pub(super) size: (u32, u32),
}

pub(super) const BASE: Case = Case {
    name: "",
    scenario: "showcase",
    seed: None,
    character: None,
    kit: false,
    actions: "",
    run: false,
    screen: None,
    keys: None,
    save_dir: None,
    size: (100, 30),
};

/// The showcase's storeroom, with the first stacks picked up and armour on (equipment cases).
const ARMED: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
    pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup equip:head:g \
    equip:body:f equip:main_hand:c equip:missile_weapon:d equip:missiles:e equip:feet:j";
/// Enough stacks carried for a second page.
const HOARD: &str = "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup move:SW move:W*2 \
    move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N pickup move:S*2 \
    pickup move:NW move:W*2 move:N pickup move:SW pickup";
/// Next to the hub's hermit.
macro_rules! hermit {
    () => {
        "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3"
    };
}
/// At the courtyard's shop door.
macro_rules! shop {
    () => {
        "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 move:E*5"
    };
}

const CASES: &[Case] = &[
    // Menus and settings.
    Case {
        name: "main-menu",
        screen: Some("main"),
        ..BASE
    },
    Case {
        name: "main-menu-saved",
        screen: Some("main"),
        save_dir: Some("tests/golden/saved-characters"),
        ..BASE
    },
    Case {
        name: "characters-screen",
        screen: Some("characters"),
        save_dir: Some("tests/golden/saved-characters"),
        ..BASE
    },
    // Every scenario under its campaign's name (a developer build), the start first.
    Case {
        name: "new-run-screen",
        screen: Some("new-run"),
        size: (100, 34),
        ..BASE
    },
    Case {
        name: "pause-menu",
        actions: "wait",
        screen: Some("pause"),
        ..BASE
    },
    Case {
        name: "help-screen",
        actions: "wait",
        screen: Some("help"),
        size: (120, 44),
        ..BASE
    },
    Case {
        name: "settings-screen",
        actions: "wait",
        screen: Some("settings"),
        ..BASE
    },
    Case {
        name: "controls-screen",
        actions: "wait",
        screen: Some("controls"),
        size: (120, 44),
        ..BASE
    },
    // Up wraps to the last row: the abilities' own keys at the very bottom (#644).
    Case {
        name: "controls-screen-bottom",
        actions: "wait",
        screen: Some("controls"),
        keys: Some("Up"),
        size: (120, 44),
        ..BASE
    },
    // Character creation.
    Case {
        name: "race-screen",
        screen: Some("race"),
        ..BASE
    },
    Case {
        name: "race-screen-drow",
        screen: Some("race"),
        keys: Some("Down Down Down Down Down Down"),
        ..BASE
    },
    Case {
        name: "sign-screen",
        screen: Some("sign"),
        ..BASE
    },
    Case {
        name: "sign-screen-lucky",
        screen: Some("new-run"),
        keys: Some("4 4 0 Enter Enter"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "gender-screen",
        screen: Some("gender"),
        keys: Some("Down"),
        ..BASE
    },
    Case {
        name: "class-screen",
        screen: Some("class"),
        ..BASE
    },
    // The showcase's thief: Random preselected; a letter marks a talent.
    Case {
        name: "chargen-talents-screen",
        screen: Some("chargen-talents"),
        size: (100, 40),
        ..BASE
    },
    Case {
        name: "chargen-talents-marked",
        screen: Some("chargen-talents"),
        keys: Some("b"),
        size: (100, 40),
        ..BASE
    },
    Case {
        name: "name-screen",
        screen: Some("name"),
        keys: Some("B r a k k a"),
        ..BASE
    },
    Case {
        name: "character-summary",
        screen: Some("summary"),
        ..BASE
    },
    Case {
        name: "character-summary-other",
        screen: Some("gender"),
        keys: Some("Down Enter Enter Enter Enter"),
        ..BASE
    },
    Case {
        name: "kit-summary",
        scenario: "world",
        screen: Some("new-run"),
        keys: Some(
            "Up Enter Down Down Enter Enter Enter Down Down Down Down Down Down Down Down Enter \
             Enter Enter",
        ),
        size: (100, 36),
        ..BASE
    },
    // The character's pages.
    Case {
        name: "character-screen",
        actions: "move:SW*2 pickup",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // A witch and her familiar: the With row and the status bar's segment (#929).
    Case {
        name: "character-screen-familiar",
        character: Some("human,female,witch"),
        run: true,
        screen: Some("character"),
        ..BASE
    },
    Case {
        name: "character-screen-lucky",
        // Seed 440 gives Tamsin a lucky day (#269): the Deep One, the title after the name.
        seed: Some(440),
        run: true,
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // A Tin of the Weyfarers' Gilde (#425): the rank under the alignment.
    Case {
        name: "character-guild",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 move:E*9 open:S move:S*5 ask:S:quest-guild_quarry",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // The attacks beside the stats (the showcase's own character has bare hands): a sword and
    // shield, a bow with a sword, and two weapons with a bow.
    Case {
        name: "character-attacks-sword-and-shield",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "character-attacks-bow",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "character-attacks-two-weapons",
        character: Some("human,male,ranger"),
        actions: "move:SW*2 pickup equip:off_hand:k",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "status-effects",
        actions: "afflict:poisoned*4 afflict:bleeding wait",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    // Taken by surprise (#681): the status's line in the message line, Surprised in the status
    // bar, never a number.
    Case {
        name: "status-surprised",
        actions: "afflict:surprised",
        ..BASE
    },
    // A potion's boost (#567): Fortified by name in the Status row and the status bar, the
    // Strength score raised, no numbers.
    Case {
        name: "potion-boost",
        actions: "afflict:fortified afflict:owl_eyed wait",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "diseased",
        actions: "infect:the_shakes rest:24h*3",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        // Alternative resources (#225): a row each under the stance, and in the status bar.
        name: "resources",
        actions: "resource:rage resource:focus wait",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        // A berserker's rage brings the frenzy (#680): the stance in the status bar and the
        // Stance row, the change refused while the rage holds it.
        name: "frenzy",
        character: Some("human,male,berserker"),
        actions: "stance:coward use:rage stance:neutral",
        screen: Some("character"),
        size: (120, 36),
        ..BASE
    },
    Case {
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
    Case {
        name: "skills-screen",
        run: true,
        screen: Some("skills"),
        ..BASE
    },
    // Knowledge chosen (#667): its description says in words what it does against creatures.
    Case {
        name: "skills-screen-knowledge",
        character: Some("human,female,wizard"),
        run: true,
        screen: Some("skills"),
        keys: Some("Down Down"),
        ..BASE
    },
    // A wizard's (#668): Concentration first, its description saying what it gives in words.
    Case {
        name: "skills-screen-wizard",
        run: true,
        character: Some("elf,female,wizard"),
        screen: Some("skills"),
        ..BASE
    },
    Case {
        name: "weapon-skills-screen",
        scenario: "arena",
        actions: "move:E pickup equip:missile_weapon:b equip:missiles:c fire:30,16*12",
        screen: Some("weapon-skills"),
        ..BASE
    },
    // A wizard's (#571): Sniper from the class among the styles; evocation, a school of magic,
    // is not listed here (the spell scaling design §11).
    Case {
        name: "weapon-skills-screen-wizard",
        character: Some("human,female,wizard"),
        run: true,
        screen: Some("weapon-skills"),
        ..BASE
    },
    // Two Shields (#611), hidden until its first marks: here at the rank the dev action sets,
    // listed among the styles.
    Case {
        name: "weapon-skills-screen-two-shields",
        actions: "rank:two_shields:1",
        screen: Some("weapon-skills"),
        ..BASE
    },
    // The Magic page (the spell scaling design §11): the showcase wizard's mana, the schools
    // (evocation from the class), the caster styles (Sniper), and the spells with their rolls.
    Case {
        name: "magic-screen",
        character: Some("human,female,wizard"),
        run: true,
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // A wizard of level 18 with ranks set as risen ones are: more spells, climbed bars, a
    // bigger die in evocation's rows.
    Case {
        name: "magic-screen-trained",
        character: Some("human,female,wizard"),
        actions: "xp:200000 levelup:auto rank:evocation:5 rank:restoration:3",
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // A paladin in the forge's chain mail, which hinders casting: Battle caster marked.
    Case {
        name: "magic-screen-armoured",
        scenario: "forge",
        character: Some("human,male,paladin"),
        actions: "move:SW move:S pickup equip:body:l",
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // A warrior: the schools at rank 0, no caster styles, no spells.
    Case {
        name: "magic-screen-none",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("magic"),
        size: (112, 40),
        ..BASE
    },
    // An 80 by 20 console: the short tab line, and the page of a wizard of level 18 scrolls,
    // its gaps given way (at 80 by 24 it fits so).
    Case {
        name: "magic-screen-short",
        character: Some("human,female,wizard"),
        actions: "xp:200000 levelup:auto",
        screen: Some("magic"),
        size: (80, 20),
        ..BASE
    },
    Case {
        name: "abilities-menu",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
            pickup:a equip:off_hand:b",
        screen: Some("abilities"),
        ..BASE
    },
    // What weapon skill and style ranks teach (the combat progression design §4): greyed
    // under the Weapon skills group as firing by themselves, each after its skill's name.
    Case {
        name: "abilities-menu-weapon-skills",
        actions: "rank:long_blades:5 rank:sword_and_board:4 rank:shield_bash:5",
        screen: Some("abilities"),
        ..BASE
    },
    // A school ability (the spell scaling design §8.5): a wizard at evocation 10 has
    // Thunderstruck, greyed under the school's name as firing by itself.
    Case {
        name: "abilities-menu-school-ability",
        character: Some("human,male,wizard"),
        kit: true,
        actions: "rank:evocation:10",
        screen: Some("abilities"),
        ..BASE
    },
    // A warrior's actives (#564): Vital strike, Shield rush, and Shield wall usable with their
    // stamina; Lunge greyed, a long sword being no spear.
    Case {
        name: "abilities-menu-weapon-actives",
        character: Some("human,male,warrior"),
        kit: true,
        actions: "rank:long_blades:9 rank:sword_and_board:9 rank:spears:9",
        screen: Some("abilities"),
        ..BASE
    },
    // Two shield walls later: the pool spent, the rows greyed short of breath.
    Case {
        name: "abilities-menu-weapon-actives-short",
        character: Some("human,male,warrior"),
        kit: true,
        actions: "rank:long_blades:9 rank:sword_and_board:9 rank:spears:9 \
                  use:shield_wall use:shield_wall",
        screen: Some("abilities"),
        ..BASE
    },
    // A berserker at level 25 (#844): War cry under Rage among the class abilities, greyed
    // while no rage is held.
    Case {
        name: "abilities-menu-war-cry",
        character: Some("human,male,berserker"),
        kit: true,
        actions: "xp:350000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A wizard at level 4 (spec §8 "Chests"): Knock among the class abilities, asking for a
    // direction, paid in mana.
    Case {
        name: "abilities-menu-knock",
        character: Some("human,male,wizard"),
        kit: true,
        actions: "xp:3000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger at level 40 (#921): Sic 'em among the class abilities, asking for a cell.
    Case {
        name: "abilities-menu-sic",
        character: Some("human,male,ranger"),
        kit: true,
        actions: "xp:90000000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A merchant past level 12 (#816): Merchant's eye among the class abilities, asking for
    // something carried.
    Case {
        name: "abilities-menu-merchants-eye",
        character: Some("human,male,merchant"),
        kit: true,
        actions: "xp:60000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // The Hunter's Spirit weapon asks for a shape (#881): every melee weapon by weapon skill,
    // as plain ones; the list is the game's objects, whoever opens it.
    Case {
        name: "shape-list",
        run: true,
        screen: Some("shape"),
        ..BASE
    },
    Case {
        name: "shape-list-second-page",
        run: true,
        screen: Some("shape"),
        keys: Some(">"),
        ..BASE
    },
    // The wizard's Scribe asks for a spell (#906): the spells known by school, each with its
    // price and its time in words; no blank scroll carried, which the line above says.
    Case {
        name: "scribe-list",
        character: Some("human,female,wizard"),
        actions: "xp:200000 levelup:auto",
        screen: Some("scribe"),
        ..BASE
    },
    // Without Scribe (a warrior), nothing to write.
    Case {
        name: "scribe-list-none",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("scribe"),
        ..BASE
    },
    // An ascetic past level 12 (#848): Stunning strike among the class abilities, asking for
    // a direction, paid in ki.
    Case {
        name: "abilities-menu-stunning-strike",
        character: Some("human,male,ascetic"),
        kit: true,
        actions: "xp:80000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // An ascetic past level 25 (#850): Stillness, Still mind's active, greyed with nothing to
    // end.
    Case {
        name: "abilities-menu-stillness",
        character: Some("human,male,ascetic"),
        kit: true,
        actions: "xp:3000000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A priest at level 1 (#886): the prayer by its first tier's title, Orison, chosen, its
    // description below.
    Case {
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
    Case {
        name: "abilities-priest-60",
        character: Some("human,male,priest"),
        kit: true,
        actions: "xp:30000000 levelup:auto",
        screen: Some("abilities"),
        keys: Some("Down Down"),
        ..BASE
    },
    // Beside a forgotten altar (spec §7 "Special rooms"): Pray in the Here group, last.
    Case {
        name: "abilities-menu-here",
        scenario: "shrine",
        actions: "move:N open:N move:N*3",
        screen: Some("abilities"),
        ..BASE
    },
    // Beside a forge's anvil with arms and armour picked up: Temper in the Here group, asking
    // for something carried; then its list, metal arms and armour only (not the lantern).
    Case {
        name: "abilities-menu-forge",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2",
        screen: Some("abilities"),
        ..BASE
    },
    Case {
        name: "temper-list",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2",
        screen: Some("temper"),
        ..BASE
    },
    // An attempt made and one failed (spec §7 "Special rooms"): the name and numbers follow.
    Case {
        name: "temper-list-fine",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:d",
        screen: Some("temper"),
        ..BASE
    },
    Case {
        name: "temper-list-damaged",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:d temper:d",
        screen: Some("temper"),
        ..BASE
    },
    // A ranger in the Thornwood after a forage (spec §8 "Foraging"): Survival listed as its
    // use, Forage, usable; the message line says what came of it.
    Case {
        name: "abilities-menu-forage",
        scenario: "forest",
        actions: "forage",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger in the showcase's hall, no wilderness: Forage greyed out, "not usable here".
    Case {
        name: "abilities-menu-forage-not-here",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger after a batch (spec §8 "Fletching"): Fletch in Woodcraft's place, usable; the
    // message line says what was made.
    Case {
        name: "abilities-menu-fletch",
        character: Some("human,male,ranger"),
        run: true,
        actions: "fletch",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger's Whittle list (#1028): the wooden weapons carried (the bow), not the knife.
    Case {
        name: "whittle-list",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("whittle"),
        ..BASE
    },
    // A warrior carries no knife: Fletch greyed out, with the reason.
    Case {
        name: "abilities-menu-fletch-no-knife",
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("abilities"),
        ..BASE
    },
    // A thief's dagger is a knife, but no bow is ready.
    Case {
        name: "abilities-menu-fletch-no-launcher",
        character: Some("human,male,thief"),
        run: true,
        screen: Some("abilities"),
        ..BASE
    },
    // Beside a healing spring (spec §7 "Special rooms"): Drink in the Here group, last.
    Case {
        name: "abilities-menu-spring",
        scenario: "spring",
        actions: "move:N*5",
        screen: Some("abilities"),
        ..BASE
    },
    // Two allies beside the player (spec §8 "Allies"): the Allies group's three orders, Set on
    // asking for a place.
    Case {
        name: "abilities-menu-allies",
        actions: "ally:dire_warg:2",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger beside their hound with the darkness chamber's amulet: Give in the Allies group,
    // asking for something carried, then its list, what fits a neck only.
    Case {
        name: "abilities-menu-give",
        character: Some("human,male,ranger"),
        actions: "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 move:S*2 move:W*2 pickup move:E*2 move:N*2 wait*2 move:W",
        screen: Some("abilities"),
        ..BASE
    },
    Case {
        name: "give-list",
        character: Some("human,male,ranger"),
        actions: "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 move:S*2 move:W*2 pickup move:E*2 move:N*2 wait*2 move:W",
        screen: Some("give"),
        ..BASE
    },
    // An ally in the retinue on the overworld, none on the level: the orders greyed.
    Case {
        name: "abilities-menu-allies-none",
        scenario: "world",
        actions: "ally:dire_warg",
        screen: Some("abilities"),
        ..BASE
    },
    // A druid past Pack leader (#839) who has called two wolves: the calls among the spells,
    // "2 called" with Pack leader's one more, and the Allies group's orders.
    Case {
        name: "abilities-menu-druid-calls",
        character: Some("human,female,druid"),
        actions: "xp:1000000 levelup:auto use:call_of_the_wild",
        screen: Some("abilities"),
        ..BASE
    },
    // Set on chosen from the menu: the aiming mode, its prompt in the message line.
    Case {
        name: "set-on-aim",
        actions: "ally:dire_warg:2",
        keys: Some("a h"),
        ..BASE
    },
    // Come to heel chosen from the menu: the order's words, no time.
    Case {
        name: "come-to-heel",
        actions: "ally:dire_warg:2",
        keys: Some("a f"),
        ..BASE
    },
    // A paladin of some level: the Class group, Lay on hands refused and greyed out short of
    // zeal (it costs six; the pool was granted at five), Smite ready (three).
    Case {
        name: "abilities-menu-class",
        character: Some("human,male,paladin"),
        // Seed 12: a Unicorn-born paladin (#272: Good, and a Birth sign gift in the menu).
        seed: Some(12),
        actions: "xp:100000 levelup:auto use:lay_on_hands",
        screen: Some("abilities"),
        ..BASE
    },
    // A thief of level 12 (#684): Lurk ready, Low blow greyed out short of openings; Disarm,
    // Pick lock, and Steal from the class's skills.
    Case {
        name: "abilities-menu-thief",
        character: Some("human,female,thief"),
        kit: true,
        actions: "xp:100000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A priest blessed and drained to their last mana: the spells in their Spells group with
    // the roll each makes now and its cost (the spell scaling design decision 11), those that
    // cost more than is left greyed out with what is left ("3 mana: you have 2").
    Case {
        name: "abilities-menu-spells",
        character: Some("human,female,priest"),
        actions: "xp:100000 levelup:auto use:blessing drain:101",
        screen: Some("abilities"),
        ..BASE
    },
    // A gnome wizard at Intelligence 19 (Keen) with Empower (level 32; the spell scaling design
    // §11): the rolls up their ladders and a rung more, Magic missile selected, its reach in the
    // description line.
    Case {
        name: "abilities-menu-spells-grown",
        character: Some("gnome,male,wizard"),
        actions: "xp:1000000 levelup:auto afflict:keen",
        screen: Some("abilities"),
        keys: Some("Down Down Down Down"),
        ..BASE
    },
    // A bard of level 25 with too little mana (#244): Throw your voice aimed at a place, Inspire
    // greyed out for inspiration (a song costs one thing, #870), the Lullaby for mana (#873),
    // Music no row (read by rules).
    Case {
        name: "abilities-menu-bard",
        character: Some("human,female,bard"),
        actions: "xp:400000 levelup:auto drain:106",
        screen: Some("abilities"),
        ..BASE
    },
    // A witch past level 6 (#904): Cackle under Class, nothing to aim, listed usable (the rules
    // refuse it with no hex of hers in sight), beside her hexes in Spells.
    Case {
        name: "abilities-menu-witch",
        character: Some("human,female,witch"),
        actions: "xp:20000 levelup:auto",
        screen: Some("abilities"),
        ..BASE
    },
    // A priest bleeding and poisoned (#549): Heal in the Skills group, used on yourself.
    Case {
        name: "abilities-menu-heal",
        character: Some("human,female,priest"),
        actions: "afflict:bleeding*2 afflict:poisoned*2",
        screen: Some("abilities"),
        ..BASE
    },
    // The same priest choosing Heal: with two things to treat, the dialog asks which.
    Case {
        name: "heal-dialog",
        character: Some("human,female,priest"),
        actions: "afflict:bleeding*2 afflict:poisoned*2",
        keys: Some("a e"),
        ..BASE
    },
    // Progression: a warrior gaining levels 2 and 3 at once (the level-up screen opens by
    // itself), their talent pick, and the Talents page.
    Case {
        name: "level-up-rolls",
        character: Some("human,male,warrior"),
        actions: "xp:691",
        size: (100, 40),
        ..BASE
    },
    // The skill-roll step at the smallest console (#543): a thief's search marks set a
    // bigger die, and each skill shows the dice a roll on it adds.
    Case {
        name: "level-up-rolls-marks",
        character: Some("human,female,thief"),
        actions: "search:100*2 xp:691",
        size: (80, 25),
        ..BASE
    },
    // A skill past the soft cap rises by half its roll: `(dice)/2`.
    Case {
        name: "level-up-rolls-halved",
        character: Some("human,male,warrior"),
        actions: "xp:300000 levelup:auto xp:600000",
        size: (80, 25),
        ..BASE
    },
    Case {
        name: "level-up-talent",
        character: Some("human,male,warrior"),
        actions: "xp:691 advance:athletics*6",
        size: (100, 40),
        ..BASE
    },
    Case {
        name: "level-up-choice",
        character: Some("human,male,warrior"),
        actions: "xp:691 advance:athletics*6",
        // Skill focus's letter on the talent list.
        keys: Some("p"),
        size: (100, 40),
        ..BASE
    },
    Case {
        name: "talents-screen",
        character: Some("dwarf,male,warrior"),
        run: true,
        screen: Some("talents"),
        size: (100, 40),
        ..BASE
    },
    // A level-3 wizard's pick, School focus for evocation: the Spellcraft tree, Arcane
    // training among the class abilities (#400).
    Case {
        name: "talents-screen-spellcraft",
        character: Some("human,male,wizard"),
        actions: "xp:880 talent:school_focus:evocation advance:spellcraft*8",
        screen: Some("talents"),
        size: (100, 40),
        ..BASE
    },
    // The Ascmearc's pedlar met on the road east of Cēaptūn (#806): the one dialog, which
    // must be answered.
    Case {
        name: "encounter-pedlar",
        scenario: "ascmearc",
        actions: "move:E*7 meet:road_pedlar",
        ..BASE
    },
    // A story talent (#707): the flux gave this warrior Hardy, "(story)".
    Case {
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
    Case {
        name: "story-screen",
        screen: Some("summary"),
        keys: Some("Enter"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "story-page",
        character: Some("dwarf,female,warrior"),
        run: true,
        screen: Some("character"),
        keys: Some("9"),
        size: (112, 36),
        ..BASE
    },
    Case {
        name: "story-page-none",
        run: true,
        screen: Some("character"),
        keys: Some("9"),
        ..BASE
    },
    // A Unicorn (seed 12) after Horn's grace: the Birth sign group, the grace greyed out while
    // Graceless (#266).
    Case {
        name: "abilities-menu-sign",
        seed: Some(12),
        character: Some("human,male,warrior"),
        actions: "afflict:bleeding*2 afflict:poisoned*3 use:horns_grace",
        screen: Some("abilities"),
        ..BASE
    },
    // A Fire-born warrior (a Fire Kraken) after a Blaze: the Birth sign group, burnt out.
    Case {
        name: "abilities-menu-blaze",
        seed: Some(15),
        character: Some("human,male,warrior"),
        actions: "use:blaze",
        screen: Some("abilities"),
        ..BASE
    },
    // A Wyvern warrior (seed 2) who picked Fire breath (#270) and breathed it: the Birth sign
    // group, drained.
    Case {
        name: "abilities-menu-fire-breath",
        seed: Some(2),
        character: Some("human,male,warrior"),
        actions: "xp:300000 talent:fire_breath levelup:auto use:fire_breath:11,15",
        screen: Some("abilities"),
        ..BASE
    },
    // A Barghest warrior (seed 21) in the arena who picked Hound's bay (#270) and howled: the
    // kobolds who heard it flee.
    Case {
        name: "abilities-menu-hounds-bay",
        scenario: "arena",
        seed: Some(21),
        character: Some("human,male,warrior"),
        actions: "xp:300000 talent:hounds_bay levelup:auto use:hounds_bay",
        screen: Some("abilities"),
        ..BASE
    },
    // A wizard with Blood magic (#242), after giving blood for mana: the Talents group.
    Case {
        name: "abilities-menu-blood-magic",
        seed: Some(4),
        character: Some("elf,female,wizard"),
        actions: "xp:200000 talent:blood_magic levelup:auto drain:20 use:blood_magic",
        screen: Some("abilities"),
        ..BASE
    },
    // Items.
    Case {
        name: "equipment-screen",
        actions: ARMED,
        screen: Some("equipment"),
        ..BASE
    },
    Case {
        name: "equip-item-screen",
        actions: ARMED,
        screen: Some("equip-item"),
        ..BASE
    },
    Case {
        name: "kit-equipment",
        scenario: "world",
        kit: true,
        run: true,
        screen: Some("equipment"),
        ..BASE
    },
    Case {
        name: "kit-inventory",
        scenario: "world",
        kit: true,
        run: true,
        screen: Some("inventory"),
        ..BASE
    },
    Case {
        name: "inventory-screen",
        actions: "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup",
        screen: Some("inventory"),
        ..BASE
    },
    Case {
        name: "inventory-page-2",
        actions: HOARD,
        screen: Some("inventory"),
        keys: Some(">"),
        size: (80, 24),
        ..BASE
    },
    Case {
        name: "drop-page-2",
        actions: HOARD,
        screen: Some("drop"),
        keys: Some("a > b"),
        size: (80, 24),
        ..BASE
    },
    Case {
        name: "burdened-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
            pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup",
        screen: Some("inventory"),
        ..BASE
    },
    Case {
        name: "magic-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 pickup \
            move:E pickup equip:head:d",
        screen: Some("inventory"),
        ..BASE
    },
    Case {
        name: "gem-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*7 move:W*4 move:S move:W*3 \
            pickup",
        screen: Some("inventory"),
        ..BASE
    },
    Case {
        name: "pick-up-screen",
        actions: "move:SW*2",
        screen: Some("pick-up"),
        ..BASE
    },
    Case {
        name: "eat-screen",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 kick:E*4 move:E*2 move:N \
            pickup move:E*2",
        screen: Some("eat"),
        ..BASE
    },
    // Born under the Barghest (seed 21): Hound's nose says how fresh the food smells.
    Case {
        name: "eat-screen-smell",
        seed: Some(21),
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // A thief is trained in Appraise (#1031): the eat list says how fresh the food looks.
    Case {
        name: "eat-screen-judged",
        seed: Some(6),
        character: Some("human,male,thief"),
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // Nothing to consume (the arena's pack holds no food).
    Case {
        name: "consume-list-none",
        scenario: "arena",
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // Potions (spec §8 "Potions"): the Consume list with food (the bread carried) and potions
    // (the hub's pile underfoot) together, unknown ones by their word and colour.
    Case {
        name: "consume-list",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:NE move:E pickup move:SW",
        screen: Some("eat"),
        ..BASE
    },
    Case {
        name: "potion-description-unidentified",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E pickup",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "potion-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E pickup eat:a",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "item-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup \
            equip:head:a",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "ornament-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "variant-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*3 move:N \
            pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    // A unique item (the quarry's Stonebiter): unidentified, a rare dagger; identified, its
    // name, affixes, power, and lore.
    Case {
        name: "unique-description-unidentified",
        scenario: "quarry",
        actions: "move:W pickup",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "unique-description",
        scenario: "quarry",
        actions: "move:W pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    // The system affixes (#635), in words: the trap hall's staff (spells and mana), mail coat
    // (stamina, rage, eased casting), and amulet (hunger, disease, potions).
    Case {
        name: "item-description-spell-affix",
        scenario: "traps",
        actions: "drop:a move:E*5 pickup",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "item-description-stamina-affix",
        scenario: "traps",
        actions: "drop:a move:E*6 pickup",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "item-description-utility-affix",
        scenario: "traps",
        actions: "drop:a move:E*7 pickup",
        screen: Some("describe"),
        ..BASE
    },
    // Scrolls.
    Case {
        name: "scroll-read-list",
        scenario: "traps",
        actions: "move:E pickup",
        screen: Some("read"),
        ..BASE
    },
    Case {
        name: "scroll-identify-list",
        scenario: "traps",
        actions: "move:E pickup",
        keys: Some("r d"),
        ..BASE
    },
    // The Decipher ability's list (spec §8 "Scrolls"): the scrolls carried of a type not yet made
    // out; a wizard is trained in Decipher script.
    Case {
        name: "scroll-decipher-list",
        scenario: "traps",
        character: Some("human,male,wizard"),
        actions: "move:E pickup",
        screen: Some("decipher"),
        ..BASE
    },
    Case {
        name: "scroll-too-close",
        scenario: "traps",
        // The fireballs identified first: an unknown scroll is read without a warning.
        actions: "drop:a move:E pickup read:e:b",
        keys: Some("r a k Enter"),
        ..BASE
    },
    // Studying (spec §8 "Spells"): the scrolls whose type is known and whose spell is not, with
    // the chance in words; an unknown scroll is not offered.
    Case {
        name: "scroll-study-list",
        scenario: "traps",
        character: Some("human,female,wizard"),
        actions: "move:E pickup identify",
        keys: Some("L"),
        ..BASE
    },
    Case {
        name: "scroll-study-dialog",
        scenario: "traps",
        character: Some("human,female,wizard"),
        actions: "move:E pickup identify",
        keys: Some("L a"),
        ..BASE
    },
    // A paladin: trained in Spellcraft, and no caster's random scrolls (#1104) to study.
    Case {
        name: "scroll-study-none",
        scenario: "traps",
        character: Some("human,male,paladin"),
        actions: "move:E pickup",
        keys: Some("L"),
        ..BASE
    },
    // A character not trained in Spellcraft is told so, whatever they carry.
    Case {
        name: "scroll-study-untrained",
        scenario: "traps",
        character: Some("human,male,warrior"),
        actions: "move:E pickup identify",
        keys: Some("L"),
        ..BASE
    },
    // Wands (spec §8 "Magic items": "Wands"): the trap hall's two, unknown, then identified;
    // aiming one from the missile weapon slot.
    Case {
        name: "wand-description-unidentified",
        scenario: "traps",
        actions: "drop:a move:E move:E pickup",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "wand-description",
        scenario: "traps",
        actions: "drop:a move:E move:E pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "wand-aim",
        scenario: "traps",
        actions: "drop:a move:E move:E pickup identify equip:missile_weapon:a",
        keys: Some("t"),
        ..BASE
    },
    // Rods (spec §8 "Magic items": "Rods"): the trap hall's two, identified; aiming one from the
    // missile weapon slot shows its reach.
    Case {
        name: "rod-description",
        scenario: "traps",
        actions: "drop:a move:E move:E move:E pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "rod-aim",
        scenario: "traps",
        actions: "drop:a move:E move:E move:E pickup identify equip:missile_weapon:a",
        keys: Some("t"),
        ..BASE
    },
    // Talk, quests, messages.
    Case {
        name: "dialogue-screen",
        actions: concat!(hermit!(), " chat:SE ask:SE:rooms"),
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Sister Hild: a proper name, topics, and two services with their prices (#336).
    Case {
        name: "dialogue-services",
        scenario: "ceaptun",
        actions: "move:SW*4 move:W*15 open:SW move:SW move:S*2 chat:SE",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Factor Wulfrun: an appraisal, priced per thing (#432).
    Case {
        name: "dialogue-appraisal",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 open:S move:S*4 chat:S",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Wat One-Eye (#425): the guild's chain, one quest topic at a time, before his
    // own two.
    Case {
        name: "dialogue-guild",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 move:E*9 open:S move:S*5 chat:S",
        screen: Some("dialogue"),
        ..BASE
    },
    Case {
        name: "quest-offer",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("dialogue"),
        ..BASE
    },
    // A rolled quest (#422): the courtyard's gardener, whose offer names what was drawn as it
    // was given.
    Case {
        name: "quest-offer-rolled",
        scenario: "courtyard",
        actions: "move:E*9 move:NE*6 wait*12 move:E ask:N:quest-slug_charm",
        screen: Some("dialogue"),
        ..BASE
    },
    Case {
        name: "quest-log",
        actions: concat!(hermit!(), " ask:SE:quest-rats"),
        screen: Some("quests"),
        ..BASE
    },
    // A rolled quest's first breadcrumb, filled.
    Case {
        name: "quest-log-rolled",
        scenario: "courtyard",
        actions: "move:E*9 move:NE*6 wait*12 move:E ask:N:quest-slug_charm",
        screen: Some("quests"),
        ..BASE
    },
    Case {
        name: "message-history",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("messages"),
        ..BASE
    },
    Case {
        name: "more-prompt",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        ..BASE
    },
    // Shops, traps, death.
    Case {
        name: "shop-unpaid",
        scenario: "courtyard",
        actions: concat!(shop!(), " move:E move:NE pickup move:E move:E pickup"),
        screen: Some("inventory"),
        ..BASE
    },
    Case {
        name: "shop-leaving",
        scenario: "courtyard",
        actions: concat!(shop!(), " move:E move:NE pickup move:W move:SW"),
        keys: Some("Left"),
        ..BASE
    },
    Case {
        name: "trap-disarm-prompt",
        actions: "move:N*3 open:N move:N*2 move:E*67 open:S move:S*7 move:SW descend",
        // `a b`: Disarm a trap, after the thief's birth sign's stare (#266).
        keys: Some("a b"),
        ..BASE
    },
    Case {
        name: "death-screen",
        scenario: "arena",
        // Pip's birth sign has gifts (#265): seed 4 is one where the fight still ends in his death.
        seed: Some(4),
        actions: "use_tool wait*20",
        ..BASE
    },
    Case {
        name: "death-screen-lucky",
        scenario: "arena",
        // Seed 142 gives Pip a lucky day, the Iron Scourge (#269), and the fight his death.
        seed: Some(142),
        actions: "use_tool wait*14",
        ..BASE
    },
];

#[test]
fn screens_as_text() {
    let content = Arc::new(Content::builtin().expect("content"));
    for case in CASES {
        insta::assert_snapshot!(case.name, render(case, content.clone()));
    }
}

/// Factor Wulfrun asked about the lost shaft before level 5: the topic is listed, her reply
/// says the player is not ready yet, and nothing is given or marked. The content lifts the
/// quest's prerequisite (the barrow's heart, deep in Cealdhlāw) so a short walk reaches her.
#[test]
fn quest_refused_as_too_early() {
    let mut files = warlock_core::content::ContentFiles::builtin();
    let path = "campaigns/ascmearc/data/quests/wulfrun.toml";
    let quests = include_str!("../../../../assets/campaigns/ascmearc/data/quests/wulfrun.toml");
    let lifted = quests.replacen("requires = \"barrow_heart\"\n", "", 1);
    assert_ne!(lifted, quests, "the lost shaft requires the barrow's heart");
    files.insert(path, &lifted);
    let content = Arc::new(Content::load(&files).expect("content"));
    let case = Case {
        name: "quest-refused-early",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 open:S move:S*4 ask:S:quest-lost_shaft",
        screen: Some("dialogue"),
        ..BASE
    };
    insta::assert_snapshot!(case.name, render(&case, content));
}

/// The hermit knowing a word (`[[dialogue.word]]`, #878), which the showcase's content does
/// not give him: the dialogue screen's Say row under his topics, the say field open, the field
/// with words typed in it, and his reply once they are said.
#[test]
fn saying_words() {
    let mut files = warlock_core::content::ContentFiles::builtin();
    let path = "campaigns/showcase/data/dialogue/showcase.toml";
    let dialogue =
        include_str!("../../../../assets/campaigns/showcase/data/dialogue/showcase.toml");
    let word = "\n[[dialogue.word]]\nphrase = \"Hello\"\nreply = [\"Hm. Hello yourself.\"]\n";
    files.insert(path, &format!("{dialogue}{word}"));
    let content = Arc::new(Content::load(&files).expect("content"));
    let cases = [
        ("dialogue-say-row", None),
        ("dialogue-say-field", Some("s")),
        ("dialogue-say-field-typed", Some("s H e l")),
        ("dialogue-say-reply", Some("s H e l l o Enter")),
    ];
    for (name, keys) in cases {
        let case = Case {
            name,
            actions: concat!(hermit!(), " chat:SE"),
            screen: Some("dialogue"),
            keys,
            ..BASE
        };
        insta::assert_snapshot!(case.name, render(&case, content.clone()));
    }
}

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
        insta::assert_snapshot!(case.name, render(&case, content.clone()));
    }
}

/// A witch who tames a hound beside her familiar (a witch's Handle animal is one draw in four:
/// here the content gives it): Give's list, then the way it asks with both beside her, and the
/// fastening once the way is told (the owner, #907 Q4).
#[test]
fn giving_with_two_kept_beasts_beside() {
    let mut files = warlock_core::content::ContentFiles::builtin();
    let witch = include_str!("../../../../assets/data/classes/10-witch.toml");
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
        insta::assert_snapshot!(case.name, render(&case, content.clone()));
    }
}

/// A run on the overworld beside the Hall that starts with `nutrition` (spec §8 "Hunger"), or if
/// that is below 0 (no run starts so hungry), has just begun to starve.
fn on_the_road(nutrition: i32) -> (Case, Arc<Content>) {
    let mut files = warlock_core::content::ContentFiles::builtin();
    let hunger = include_str!("../../../../assets/data/hunger.toml")
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
            ..road
        };
        insta::assert_snapshot!(case.name, render(&case, content));
    }
}

/// Beside what offers an action, north of the player (spec §7 "Special rooms"): its scenario and
/// the walk there.
const OFFERS: [(&str, &str, &str); 3] = [
    ("offer-dialog-altar", "shrine", "move:N open:N move:N*3"),
    ("offer-dialog-spring", "spring", "move:N*5"),
    ("offer-dialog-anvil", "forge", "move:E"),
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
        insta::assert_snapshot!(case.name, render(&case, content.clone()));
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
    insta::assert_snapshot!(case.name, render(&case, content));
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
        insta::assert_snapshot!(case.name, render(&case, content.clone()));
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
    insta::assert_snapshot!(case.name, render(&case, content.clone()));
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

/// The UI console's text once `case` has started and its keys were pressed: one line a row,
/// trailing blanks trimmed.
fn render(case: &Case, content: Arc<Content>) -> String {
    let mut app = start(case, content);
    press_keys(&mut app, case.keys.unwrap_or(""));
    text(app.world().resource::<Console>())
}

/// The app of `case` started, its screens open, no keys pressed yet.
pub(super) fn start(case: &Case, content: Arc<Content>) -> App {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let save_dir = case.save_dir.map(|dir| format!("{root}/{dir}"));
    let mut argv = vec![
        "warlock",
        "--capture",
        "unused.png",
        "--scenario",
        case.scenario,
    ];
    argv.extend(case.screen.iter().flat_map(|s| ["--screen", s]));
    argv.extend(case.character.iter().flat_map(|c| ["--character", c]));
    argv.extend(save_dir.iter().flat_map(|d| ["--save-dir", d.as_str()]));
    let args = Args::parse_from(argv);

    let game = (case.run || !case.actions.is_empty()).then(|| {
        let kit = case.kit || case.character.is_some();
        let character = args.character.as_ref();
        let world = World::start_run(
            content.clone(),
            case.scenario,
            case.seed,
            character,
            None,
            kit,
        )
        .expect("run");
        let mut game = Game::from_world(world, "snapshot".into());
        let actions = warlock_core::action::parse_actions(case.actions).expect("actions");
        for action in &actions {
            game.apply(action);
        }
        game
    });

    let mut console = Console::default();
    console.resize(case.size.0, case.size.1);
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .add_message::<PlayerAction>()
        .add_message::<SaveCommand>()
        .add_message::<MenuChoice>()
        .insert_resource(SaveStore::from_args(&args))
        .insert_resource(args)
        .insert_resource(GameContent(content))
        .insert_resource(console)
        .init_resource::<ScreenStack>()
        .init_resource::<Settings>()
        .init_resource::<crate::settings::SettingsStore>()
        .init_resource::<Bindings>()
        .init_resource::<InputMode>()
        .init_resource::<LastTarget>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<DevOverlays>()
        .init_resource::<MapGrid>()
        .init_resource::<ConsoleLayout>()
        .add_systems(Startup, screens::open_startup_screens)
        .add_systems(
            Update,
            (
                route_keys,
                screens::apply_choice,
                apply_actions.run_if(resource_exists::<Game>),
                (
                    screens::refresh,
                    screens::on_run_start,
                    screens::converse,
                    screens::on_death,
                    encounter::on_encounter,
                    screens::on_level_up,
                    more::wait,
                )
                    .chain()
                    .run_if(resource_exists::<Game>),
                draw,
            )
                .chain(),
        );
    if let Some(game) = game {
        app.insert_resource(game);
    }
    app.update();
    app
}

/// Presses `keys` (as `--keys` writes them) in one frame, then lets the run and the screens
/// catch up.
pub(super) fn press_keys(app: &mut App, keys: &str) {
    for key in keys.split_whitespace() {
        app.world_mut().write_message(press(key));
    }
    app.update();
    app.update();
}

/// Applies the actions input asked for, as the game plugin does.
fn apply_actions(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
    crate::game::apply_queued(actions.read(), &mut game);
}

/// A key press as `--keys` writes it: a named key, or a character that types itself.
fn press(key: &str) -> KeyboardInput {
    let (key_code, logical_key) = match key {
        "Enter" => (KeyCode::Enter, Key::Enter),
        "Esc" => (KeyCode::Escape, Key::Escape),
        "Tab" => (KeyCode::Tab, Key::Tab),
        "Up" => (KeyCode::ArrowUp, Key::ArrowUp),
        "Down" => (KeyCode::ArrowDown, Key::ArrowDown),
        "Left" => (KeyCode::ArrowLeft, Key::ArrowLeft),
        "Right" => (KeyCode::ArrowRight, Key::ArrowRight),
        ch => (
            KeyCode::Unidentified(NativeKeyCode::Unidentified),
            Key::Character(ch.into()),
        ),
    };
    let text = match &logical_key {
        Key::Character(ch) => Some(ch.clone()),
        _ => None,
    };
    KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Pressed,
        text,
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}

/// The console's glyphs, one line a row; transparent cells are blanks.
pub(super) fn text(console: &Console) -> String {
    let cols = console.cols() as usize;
    let lines: Vec<String> = (console.cells().chunks(cols))
        .map(|row| {
            let line: String = row.iter().map(|c| c.map_or(' ', |c| c.ch)).collect();
            line.trim_end().to_string()
        })
        .collect();
    lines.join("\n")
}
