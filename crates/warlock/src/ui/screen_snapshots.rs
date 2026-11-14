//! Text snapshots of the UI console (#301): each case starts a run as a golden entry would
//! (scenario, character, kit, actions, `--screen`, `--keys`), runs the real router and screen
//! systems headlessly, and snapshots the UI console as text with `insta`. The map layer is not in
//! it (map content is `warlock-cli dump --layers view`), nor colors.
//!
//! Screens belong here, not in `tests/golden/`: a text diff is exact, readable in review, and
//! needs no GPU. Add a case to `cases!`, run `cargo test -p warlock screen_snapshots`, and review
//! the new snapshot with `cargo insta review` (or `INSTA_UPDATE=always`, then read the diff).
//!
//! Two things every case shares are left out, so a change to them rewrites no snapshot of a
//! screen that only shows them (#1521, #1532): the status bar's row over a run, kept only by the
//! cases about the bar (`status_bar`), and the game's version, written `[version]`.

use super::*;
use crate::cli::Args;
use crate::game::{GameContent, PlayerAction};
use crate::save::{SaveCommand, SaveStore};
use crate::settings::mods::Mods;
use crate::ui::screens::{MenuChoice, ScreenStack};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use clap::Parser;
use std::sync::{Arc, LazyLock};
use warlock_core::content::Content;

/// One snapshot: a run (or the main menu) and what is open over it.
pub(super) struct Case {
    pub(super) name: &'static str,
    pub(super) scenario: &'static str,
    /// `--seed`; none: the scenario's.
    pub(super) seed: Option<u64>,
    /// `--character`; the run starts with its kit.
    pub(super) character: Option<&'static str>,
    /// `--talents`: the character's creation picks (none: its Random picks).
    pub(super) talents: Option<&'static str>,
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
    /// The graveyard file's text (`graveyard.rs`), kept in a folder of the case's own; none
    /// keeps no graveyard, as a capture does.
    pub(super) graveyard: Option<&'static str>,
    /// Keeps the status bar's row (a case about the bar); otherwise it is left out (#1521).
    pub(super) status_bar: bool,
    /// The UI console's size (columns, rows).
    pub(super) size: (u32, u32),
}

pub(super) const BASE: Case = Case {
    name: "",
    scenario: "showcase",
    seed: None,
    character: None,
    talents: None,
    kit: false,
    actions: "",
    run: false,
    screen: None,
    keys: None,
    save_dir: None,
    graveyard: None,
    status_bar: false,
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
/// The hub's secret closet found and its gems picked up (a medium fine sapphire, an uncut tiny
/// garnet).
const GEM_CLOSET: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*6 move:SW move:W*3 \
    search:50 move:S open:W move:W*2 pickup";
/// A craftsperson's creation picks with Gemcutter first: Jewelcraft their trade.
const GEMCUTTER: &str = "gemcutter,smith,hardy,healthy,lean";
/// [`GEM_CLOSET`] by a [`GEMCUTTER`] craftsperson (Jewelcraft 16 at fate 8: Set's 10 and more),
/// the hand axe out of the hand: ready to Set.
const SET_READY: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*6 move:SW move:W*3 \
    search:50 move:S open:W move:W*2 pickup remove:main_hand";
/// A wizard's creation picks with Father's sword (an identified Masterwork rare, carried).
const ENCHANTER: &str = "family_wealth,fathers_sword,hardy";
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

/// A graveyard of twelve, two of them turned to stone.
const GRAVEYARD: &str = r#"
[[fallen]]
name = "Brakka"
race = "dwarf"
cause = "stone"
[[fallen]]
name = "Tamsin"
cause = "kobold"
[[fallen]]
name = "Wynn"
[[fallen]]
name = "Osric"
cause = "stone"
[[fallen]]
name = "Eadgyth"
[[fallen]]
name = "Leofric"
[[fallen]]
name = "Hild"
[[fallen]]
name = "Cyneric"
[[fallen]]
name = "Aelfgifu"
[[fallen]]
name = "Wulfstan"
[[fallen]]
name = "Godgifu"
[[fallen]]
name = "Sigeberht"
"#;

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

/// One `#[test]` a case, named after it (`main-menu` is `cases::main_menu`), so a mismatch hides
/// none of the others (#1529): `cargo test -p warlock screen_snapshots` runs them all, and
/// `cargo test -p warlock cases::main_menu` one.
macro_rules! cases {
    ($($test:ident: $case:expr,)*) => {
        mod cases {
            use super::*;
            $(
                #[test]
                fn $test() {
                    check(&$case);
                }
            )*
        }
    };
}

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
    // Beside a forge's anvil with arms and armour picked up: no Here group (Temper is a recipe on
    // Metalcraft's screen, #1334); then the Temper list, metal arms and armour (no lantern).
    abilities_menu_forge: Case {
        name: "abilities-menu-forge",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2",
        screen: Some("abilities"),
        ..BASE
    },
    temper_list: Case {
        name: "temper-list",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2",
        screen: Some("temper"),
        ..BASE
    },
    // An attempt made and one failed (spec §7 "Special rooms"): the name and numbers follow.
    temper_list_fine: Case {
        name: "temper-list-fine",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:g",
        screen: Some("temper"),
        ..BASE
    },
    temper_list_damaged: Case {
        name: "temper-list-damaged",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:g temper:g",
        screen: Some("temper"),
        ..BASE
    },
    // The craftsperson's Make do and mend list (#1422): the damaged long sword alone.
    make_do_list: Case {
        name: "make-do-list",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:g temper:g",
        screen: Some("make-do"),
        ..BASE
    },
    // A ranger in the Thornwood after a forage (spec §8 "Foraging"): Survival listed as its
    // use, Forage, usable; the message line says what came of it.
    abilities_menu_forage: Case {
        name: "abilities-menu-forage",
        scenario: "forest",
        actions: "forage",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger in the showcase's hall, no wilderness: Forage greyed out, "not usable here".
    abilities_menu_forage_not_here: Case {
        name: "abilities-menu-forage-not-here",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger after a batch in the forest area's glade, where wood is free (spec §8
    // "Fletching"; #1336): Woodcraft… in the skill's place; the message line says what was made.
    abilities_menu_fletch: Case {
        name: "abilities-menu-fletch",
        scenario: "forest",
        character: Some("human,male,ranger"),
        run: true,
        actions: "craft:arrows:-",
        screen: Some("abilities"),
        ..BASE
    },
    // The arrows' wizard in the forest area's glade (#1336): the head step alone (the shaft is
    // cut from the trees), nothing carried to head them, None on `-` for brittle arrows.
    craft_pick_arrows_head: Case {
        name: "craft-pick-arrows-head",
        scenario: "forest",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("abilities"),
        keys: Some("e p"),
        ..BASE
    },
    // A ranger's Whittle list (#1028): the wooden weapons carried (the bow), not the knife.
    whittle_list: Case {
        name: "whittle-list",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("whittle"),
        ..BASE
    },
    // A ranger's Salvage list (#1333): what a craft works, carried, less what is worn or wielded.
    salvage_list: Case {
        name: "salvage-list",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("salvage"),
        ..BASE
    },
    // A Gemcutter craftsperson (Jewelcraft their trade) with the hub's secret closet's gems, a
    // medium fine sapphire and an uncut tiny garnet (#1355): Jewelcraft's screen (the first
    // profession whose recipes they see) lists Cut under Work; the Cut list both gems.
    profession_jewelcraft: Case {
        name: "profession-jewelcraft",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: GEM_CLOSET,
        screen: Some("profession"),
        ..BASE
    },
    cut_list: Case {
        name: "cut-list",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: GEM_CLOSET,
        screen: Some("cut"),
        ..BASE
    },
    // The garnet cut (it splits to nothing at this fate) and the sapphire re-cut a size smaller:
    // the list holds the small fine sapphire (and the trade's uncut garnet), and the message line
    // says what came of it.
    cut_list_after: Case {
        name: "cut-list-after",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*6 move:SW move:W*3 \
                  search:50 move:S open:W move:W*2 pickup craft:cut:n craft:cut:m",
        screen: Some("cut"),
        ..BASE
    },
    // Set (#1356) by the same craftsperson, Jewelcraft 16 from their trade, the
    // hand axe taken out of the hand: the wizard's item step lists the axe (its pommel unset),
    set_pick_item: Case {
        name: "set-pick-item",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: SET_READY,
        screen: Some("profession"),
        keys: Some("b"),
        ..BASE
    },
    // its gem step the sapphire alone (never the uncut garnet),
    set_pick_gem: Case {
        name: "set-pick-gem",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: SET_READY,
        screen: Some("profession"),
        keys: Some("b a"),
        ..BASE
    },
    // and the dialog names the axe as it would be.
    set_pick_dialog: Case {
        name: "set-pick-dialog",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: SET_READY,
        screen: Some("profession"),
        keys: Some("b a a"),
        ..BASE
    },
    // Enchanting (#1430) by a wizard trained in it at this fate (11), with Father's sword, a
    // Masterwork rare, in the pack: the six works (#1431) greyed by skill or "nothing to
    // enchant" (no plain item free), Quicken "it holds all it can" (the rare at its capacity,
    // #1432), Unbind ready;
    profession_enchanting: Case {
        name: "profession-enchanting",
        seed: Some(13),
        character: Some("human,female,wizard"),
        talents: Some(ENCHANTER),
        run: true,
        screen: Some("profession"),
        ..BASE
    },
    // Unbind's item step lists the sword alone (never the wielded staff),
    enchant_pick_item: Case {
        name: "enchant-pick-item",
        seed: Some(13),
        character: Some("human,female,wizard"),
        talents: Some(ENCHANTER),
        run: true,
        screen: Some("profession"),
        keys: Some("f"),
        ..BASE
    },
    // and the dialog, with nothing used up, says only the time.
    enchant_dialog: Case {
        name: "enchant-dialog",
        seed: Some(13),
        character: Some("human,female,wizard"),
        talents: Some(ENCHANTER),
        run: true,
        screen: Some("profession"),
        keys: Some("f a"),
        ..BASE
    },
    // A Wright craftsperson whose kit's knife (`b`) is dropped: Woodcraft's screen (its row in the
    // abilities menu, `c`) greys the arrows and Whittle, each with the reason in place of what it
    // takes (#1334). Only the trained have the row (#1387).
    profession_woodcraft_no_knife: Case {
        name: "profession-woodcraft-no-knife",
        character: Some("human,male,craftsperson"),
        talents: Some("wright,smith,healthy,lean,skill_focus:appraise"),
        actions: "drop:b",
        screen: Some("abilities"),
        keys: Some("c"),
        ..BASE
    },
    // A ranger's knife, but their bow put away, so none is ready: Whittle can be made, the arrows
    // not (`e`: Woodcraft…, after the stare, Decipher, Tame, and Forage).
    profession_woodcraft_no_launcher: Case {
        name: "profession-woodcraft-no-launcher",
        character: Some("human,male,ranger"),
        actions: "remove:missile_weapon",
        run: true,
        screen: Some("abilities"),
        keys: Some("e"),
        ..BASE
    },
    // Metalcraft's screen (`--screen profession`, the first profession's) away from an anvil:
    // Temper under "Work", greyed "no anvil beside you".
    profession_metalcraft_no_anvil: Case {
        name: "profession-metalcraft-no-anvil",
        character: Some("human,male,craftsperson"),
        talents: Some("smith,wright,healthy,lean,skill_focus:appraise"),
        run: true,
        screen: Some("profession"),
        ..BASE
    },
    // Beside the forge's anvil: Temper can be made, what it takes under the list.
    profession_metalcraft_forge: Case {
        name: "profession-metalcraft-forge",
        scenario: "forge",
        actions: "move:E",
        screen: Some("profession"),
        ..BASE
    },
    // The forge's stock picked up beside the anvil: the weapons under "Melee weapons", those from
    // a higher Metalcraft greyed "beyond your skill".
    profession_metalcraft_stock: Case {
        name: "profession-metalcraft-stock",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        ..BASE
    },
    // The dagger's wizard (#1335): its first step, the metal, the ingots carried with what the
    // recipe takes of each.
    craft_pick_blade: Case {
        name: "craft-pick-blade",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("c"),
        ..BASE
    },
    // Its second, the pommel: None on `-`, the cut gems; the running choice in the header.
    craft_pick_pommel: Case {
        name: "craft-pick-pommel",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("c c"),
        ..BASE
    },
    // The last pick asks in the one dialog, with the item named as it will be made.
    craft_pick_dialog: Case {
        name: "craft-pick-dialog",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("c c a"),
        ..BASE
    },
    // A witch's Alchemy (#1357; `--screen profession`, the first profession she has recipes of)
    // after three gathers at the hub's fungus: the brews, those from a higher Alchemy greyed
    // "beyond your skill", those short of herbs "not enough".
    profession_alchemy_herbs: Case {
        name: "profession-alchemy-herbs",
        character: Some("human,female,witch"),
        actions: "move:N*6 move:E*30 move:N*3 move:W*6 gather:W*3",
        screen: Some("profession"),
        ..BASE
    },
    // A brew chooses nothing: its row asks at once in the one dialog, the potion named truly.
    brew_dialog: Case {
        name: "brew-dialog",
        character: Some("human,female,witch"),
        actions: "move:N*6 move:E*30 move:N*3 move:W*6 gather:W*3",
        screen: Some("profession"),
        keys: Some("> b"),
        ..BASE
    },
    // The smith's javelin after its iron head (#1351): the haft, Woodcraft's wood billets on a
    // Metalcraft recipe, never the ingots.
    craft_pick_haft: Case {
        name: "craft-pick-haft",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("i b"),
        ..BASE
    },
    // `--screen craft-pick`: the first recipe with a choice, the amber amulet, with nothing carried.
    craft_pick_first: Case {
        name: "craft-pick-first",
        scenario: "forge",
        actions: "move:E",
        screen: Some("craft-pick"),
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
    Case {
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
    // Items.
    equipment_screen: Case {
        name: "equipment-screen",
        actions: ARMED,
        screen: Some("equipment"),
        ..BASE
    },
    equip_item_screen: Case {
        name: "equip-item-screen",
        actions: ARMED,
        screen: Some("equip-item"),
        ..BASE
    },
    kit_equipment: Case {
        name: "kit-equipment",
        scenario: "world",
        kit: true,
        run: true,
        screen: Some("equipment"),
        ..BASE
    },
    // An aristocrat's three extra ring slots (#1404), after the hands' rings.
    aristocrat_equipment: Case {
        name: "aristocrat-equipment",
        scenario: "world",
        character: Some("human,female,aristocrat"),
        run: true,
        screen: Some("equipment"),
        ..BASE
    },
    kit_inventory: Case {
        name: "kit-inventory",
        scenario: "world",
        kit: true,
        run: true,
        screen: Some("inventory"),
        ..BASE
    },
    inventory_screen: Case {
        name: "inventory-screen",
        actions: "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup",
        screen: Some("inventory"),
        ..BASE
    },
    inventory_page_2: Case {
        name: "inventory-page-2",
        actions: HOARD,
        screen: Some("inventory"),
        keys: Some(">"),
        size: (80, 24),
        ..BASE
    },
    // The inventory's drop mode (#1196): `D` there, marks kept across pages; and as `d` in
    // play opens it.
    inventory_drop_mode: Case {
        name: "inventory-drop-mode",
        actions: HOARD,
        screen: Some("inventory"),
        keys: Some("D a > b"),
        size: (80, 24),
        ..BASE
    },
    drop_mode_from_play: Case {
        name: "drop-mode-from-play",
        actions: "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup",
        screen: Some("drop"),
        keys: Some("b"),
        ..BASE
    },
    burdened_inventory: Case {
        name: "burdened-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
            pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup",
        screen: Some("inventory"),
        ..BASE
    },
    magic_inventory: Case {
        name: "magic-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 pickup \
            move:E pickup equip:head:d",
        screen: Some("inventory"),
        ..BASE
    },
    gem_inventory: Case {
        name: "gem-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*7 move:W*4 move:S move:W*3 \
            pickup",
        screen: Some("inventory"),
        ..BASE
    },
    pick_up_screen: Case {
        name: "pick-up-screen",
        actions: "move:SW*2",
        screen: Some("pick-up"),
        ..BASE
    },
    eat_screen: Case {
        name: "eat-screen",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 kick:E*4 move:E*2 move:N \
            pickup move:E*2",
        screen: Some("eat"),
        ..BASE
    },
    // Born under the Barghest (seed 21): Hound's nose says how fresh the food smells.
    eat_screen_smell: Case {
        name: "eat-screen-smell",
        seed: Some(21),
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // A thief is trained in Appraise (#1031): the eat list says how fresh the food looks.
    eat_screen_judged: Case {
        name: "eat-screen-judged",
        seed: Some(6),
        character: Some("human,male,thief"),
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // Nothing to consume (the arena's pack holds no food).
    consume_list_none: Case {
        name: "consume-list-none",
        scenario: "arena",
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // Potions (spec §8 "Potions"): the Consume list with food (the bread carried) and potions
    // (the hub's pile underfoot) together, unknown ones by their word and colour.
    consume_list: Case {
        name: "consume-list",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:NE move:E pickup move:SW",
        screen: Some("eat"),
        ..BASE
    },
    potion_description_unidentified: Case {
        name: "potion-description-unidentified",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E pickup",
        screen: Some("describe"),
        ..BASE
    },
    potion_description: Case {
        name: "potion-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E pickup eat:a",
        screen: Some("describe"),
        ..BASE
    },
    // Flasks (spec §8 "Potions": "Flasks", #1223): the throw choice lists potions beside
    // throwing weapons, and a known flask says what its throw does.
    throw_list_flasks: Case {
        name: "throw-list-flasks",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E move:W*3 pickup identify",
        keys: Some("T"),
        ..BASE
    },
    potion_description_flask: Case {
        name: "potion-description-flask",
        actions: "drop move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E move:W*3 pickup:c identify",
        screen: Some("describe"),
        ..BASE
    },
    item_description: Case {
        name: "item-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup \
            equip:head:a",
        screen: Some("describe"),
        ..BASE
    },
    ornament_description: Case {
        name: "ornament-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    variant_description: Case {
        name: "variant-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*3 move:N \
            pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    // A unique item (the quarry's Stonebiter): unidentified, a rare dagger; identified, its
    // name, affixes, power, and lore.
    unique_description_unidentified: Case {
        name: "unique-description-unidentified",
        scenario: "quarry",
        actions: "drop move:W pickup",
        screen: Some("describe"),
        ..BASE
    },
    unique_description: Case {
        name: "unique-description",
        scenario: "quarry",
        actions: "drop move:W pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    // The system affixes (#635), in words: the trap hall's staff (spells and mana), mail coat
    // (stamina, rage, eased casting), and amulet (hunger, disease, potions).
    item_description_spell_affix: Case {
        name: "item-description-spell-affix",
        scenario: "traps",
        actions: "drop:a move:E*5 pickup",
        screen: Some("describe"),
        ..BASE
    },
    item_description_stamina_affix: Case {
        name: "item-description-stamina-affix",
        scenario: "traps",
        actions: "drop:a move:E*6 pickup",
        screen: Some("describe"),
        ..BASE
    },
    item_description_utility_affix: Case {
        name: "item-description-utility-affix",
        scenario: "traps",
        actions: "drop:a move:E*7 pickup",
        screen: Some("describe"),
        ..BASE
    },
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
    // Talk, quests, messages.
    dialogue_screen: Case {
        name: "dialogue-screen",
        actions: concat!(hermit!(), " chat:SE ask:SE:rooms"),
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Sister Hild: a proper name, topics, and two services with their prices (#336).
    dialogue_services: Case {
        name: "dialogue-services",
        scenario: "ceaptun",
        actions: "move:SW*4 move:W*15 open:SW move:SW move:S*2 chat:SE",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Factor Wulfrun: an appraisal, priced per thing (#432).
    dialogue_appraisal: Case {
        name: "dialogue-appraisal",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 open:S move:S*4 chat:S",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Wat One-Eye (#425): the guild's chain, one quest topic at a time, before his
    // own two.
    dialogue_guild: Case {
        name: "dialogue-guild",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 move:E*9 open:S move:S*5 chat:S",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's smith, Eadric, the smithy's keeper (#1394): his topics, then Metalcraft taught
    // for 2000 gold.
    dialogue_smith: Case {
        name: "dialogue-smith",
        scenario: "ceaptun",
        actions: "move:NW*7 move:N chat:N",
        screen: Some("dialogue"),
        ..BASE
    },
    // An aristocrat's squire at the Gæsthūs (#1405): the stipend of a few levels handed over,
    // then a flattering one-liner, in the message history.
    servant_stipend: Case {
        name: "servant-stipend",
        scenario: "ceaptun",
        character: Some("human,male,aristocrat"),
        actions: "xp:3334 levelup:auto move:SW move:S*3 open:S move:S*2 move:SW*4 chat:W chat:W",
        screen: Some("messages"),
        ..BASE
    },
    // A merchant's business partner in Wulfrun's office (#1407): 100 gold put in, a month gone,
    // and the dialogue screen on what the stake is worth and the deals.
    partner_invest: Case {
        name: "partner-invest",
        scenario: "ceaptun",
        character: Some("human,male,merchant"),
        actions: "move:SE*4 move:E*22 open:S move:S*2 move:E*2 chat:E ask:E:invest_100 \
                  skip:24h*30 chat:E",
        screen: Some("dialogue"),
        ..BASE
    },
    // A craftsperson's patron at the Gæsthūs (#1421): a commission given with its stock, then
    // the reminder while it waits, in the message history.
    patron_commission: Case {
        name: "patron-commission",
        scenario: "ceaptun",
        character: Some("human,male,craftsperson"),
        talents: Some("smith,wright,healthy,lean,wortcunner"),
        actions: "xp:40000 levelup:auto move:SW move:S*3 open:S move:S*2 move:E*4 chat:E chat:E",
        screen: Some("messages"),
        ..BASE
    },
    quest_offer: Case {
        name: "quest-offer",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("dialogue"),
        ..BASE
    },
    // A rolled quest (#422): the courtyard's gardener, whose offer names what was drawn as it
    // was given.
    quest_offer_rolled: Case {
        name: "quest-offer-rolled",
        scenario: "courtyard",
        actions: "move:E*9 move:NE*6 wait*12 move:E ask:N:quest-slug_charm",
        screen: Some("dialogue"),
        ..BASE
    },
    quest_log: Case {
        name: "quest-log",
        actions: concat!(hermit!(), " ask:SE:quest-rats"),
        screen: Some("quests"),
        ..BASE
    },
    // A rolled quest's first breadcrumb, filled.
    quest_log_rolled: Case {
        name: "quest-log-rolled",
        scenario: "courtyard",
        actions: "move:E*9 move:NE*6 wait*12 move:E ask:N:quest-slug_charm",
        screen: Some("quests"),
        ..BASE
    },
    message_history: Case {
        name: "message-history",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("messages"),
        ..BASE
    },
    more_prompt: Case {
        name: "more-prompt",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        ..BASE
    },
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
        // Pip's birth sign has gifts (#265): seed 4 is one where the fight still ends in his death.
        seed: Some(4),
        actions: "use_tool wait*27",
        ..BASE
    },
    // The death remembered in the graveyard (#1137).
    death_screen_graveyard: Case {
        name: "death-screen-graveyard",
        scenario: "arena",
        seed: Some(4),
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

/// The built-in content, loaded once for every case.
static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// The snapshot of one of the [`cases!`].
fn check(case: &Case) {
    let content = &*CONTENT;
    insta::assert_snapshot!(case.name, render(case, content.clone()));
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

/// The hermit's oath (#1174): a calling's topic, offered to a bard whose class is full (the dev
/// experience and level-ups bring the showcase's bard to 30). Asking it is the core's
/// (`world/strange_spirit_tests.rs`): the level-up it brings would cover this screen.
#[test]
fn the_oath_topic() {
    let content = Arc::new(Content::builtin().expect("content"));
    let case = Case {
        name: "dialogue-calling-topic",
        character: Some("human,male,bard"),
        actions: concat!(hermit!(), " xp:363777 levelup:auto chat:SE"),
        screen: Some("dialogue"),
        ..BASE
    };
    insta::assert_snapshot!(case.name, render(&case, content));
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
            status_bar: true,
            ..road
        };
        insta::assert_snapshot!(case.name, render(&case, content));
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
        insta::assert_snapshot!(case.name, render(&case, content.clone()));
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

/// The UI console's text once `case` has started and its keys were pressed, as [`shown`] tells
/// it, the status bar kept if the case is about it.
fn render(case: &Case, content: Arc<Content>) -> String {
    let mut app = start(case, content);
    press_keys(&mut app, case.keys.unwrap_or(""));
    if case.status_bar {
        redact(text(app.world().resource::<Console>()))
    } else {
        shown(&app)
    }
}

/// The UI console's text for a snapshot: one line a row, trailing blanks trimmed, the game's
/// version written `[version]` (#1532), and the status bar's row left out where it shows whole
/// (#1521): a screen that covers any of it keeps the row as drawn.
pub(super) fn shown(app: &App) -> String {
    let console = app.world().resource::<Console>();
    let mut lines: Vec<String> = text(console).split('\n').map(String::from).collect();
    if let Some(bar) = status_bar(app)
        && lines.last() == Some(&bar)
    {
        lines.pop();
    }
    redact(lines.join("\n"))
}

/// `text` with the game's version written `[version]`, so a release changes no snapshot.
fn redact(text: String) -> String {
    text.replace(warlock_core::GAME_VERSION, "[version]")
}

/// The status bar's row as the run's UI draws it on its own, trailing blanks trimmed; none
/// without a run.
fn status_bar(app: &App) -> Option<String> {
    let world = app.world();
    let game = world.get_resource::<Game>()?;
    let console = world.resource::<Console>();
    let settings = world.resource::<Settings>();
    let hint = controls_hint(world.resource::<Bindings>());
    let dev = crate::input::dev_keys(world.resource::<Args>());
    let mut bar = Console::default();
    bar.resize(console.cols(), console.rows());
    let y = console.rows() as i32 - 1;
    let units = (settings.clock, settings.temperature, dev);
    draw_status(&mut bar, game, y, units, &hint);
    text(&bar).split('\n').next_back().map(String::from)
}

/// The folder a case's `save_dir` names, relative to the workspace root: the saved characters'
/// fixture as the tests read it (`save::saved_characters_fixture`, this version written in).
fn fixture(dir: &str) -> String {
    let path = if dir == "tests/golden/saved-characters" {
        crate::save::saved_characters_fixture()
    } else {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(dir)
    };
    path.to_string_lossy().into_owned()
}

/// The app of `case` started, its screens open, no keys pressed yet.
pub(super) fn start(case: &Case, content: Arc<Content>) -> App {
    let save_dir = case.save_dir.map(fixture);
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
        let picks = case.talents.map(warlock_core::world::parse_talents);
        let character = (args.character.clone()).map(|c| match picks {
            Some(p) => c.with_talents(p),
            None => c,
        });
        let world = World::start_run(
            content.clone(),
            case.scenario,
            case.seed,
            character.as_ref(),
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
        .insert_resource(graveyard(case))
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
                    screens::show_identified,
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

/// The graveyard of `case`: its text in a fresh folder of its own (deaths append to it there).
fn graveyard(case: &Case) -> crate::graveyard::GraveyardStore {
    let Some(text) = case.graveyard else {
        return crate::graveyard::GraveyardStore::default();
    };
    let dir = std::env::temp_dir().join(format!(
        "warlock-snapshot-graveyard-{}-{}",
        std::process::id(),
        case.name
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the graveyard's folder");
    std::fs::write(dir.join(crate::graveyard::FILE_NAME), text).expect("the graveyard");
    crate::graveyard::GraveyardStore::at(dir)
}

/// Presses `keys` (as `--keys` writes them) in one frame, then lets the run and the screens
/// catch up.
/// Presses `keys` (as `--keys` names them; `Ctrl+k` holds Ctrl for the `k`): each run of keys
/// with the same modifiers held in one frame, then a frame with none held.
pub(super) fn press_keys(app: &mut App, keys: &str) {
    let mut runs: Vec<(Mods, Vec<KeyboardInput>)> = Vec::new();
    for key in keys.split_whitespace() {
        let (mods, key) = Mods::split(key);
        match runs.last_mut() {
            Some((held, presses)) if *held == mods => presses.push(press(key)),
            _ => runs.push((mods, vec![press(key)])),
        }
    }
    for (mods, presses) in runs {
        hold(app, mods);
        for key in presses {
            app.world_mut().write_message(key);
        }
        app.update();
    }
    hold(app, Mods::NONE);
    app.update();
}

/// Holds exactly the modifiers `mods`, as the keyboard state the router reads.
fn hold(app: &mut App, mods: Mods) {
    let mut held = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    held.release_all();
    let keys = [
        (mods.ctrl, KeyCode::ControlLeft),
        (mods.alt, KeyCode::AltLeft),
        (mods.shift, KeyCode::ShiftLeft),
        (mods.sup, KeyCode::SuperLeft),
    ];
    for (on, key) in keys {
        if on {
            held.press(key);
        }
    }
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
