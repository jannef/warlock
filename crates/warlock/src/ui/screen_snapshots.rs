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
struct Case {
    name: &'static str,
    scenario: &'static str,
    /// `--seed`; none: the scenario's.
    seed: Option<u64>,
    /// `--character`; the run starts with its kit.
    character: Option<&'static str>,
    /// Starts with the character's starting kit without naming a character.
    kit: bool,
    /// Action text applied before the app starts; empty opens the main menu, unless `run`.
    actions: &'static str,
    /// Starts the run even with no actions (screens over a run at its first turn).
    run: bool,
    /// `--screen`.
    screen: Option<&'static str>,
    /// `--keys`, pressed once the screens are open.
    keys: Option<&'static str>,
    /// `--save-dir`, relative to the workspace root.
    save_dir: Option<&'static str>,
    /// The UI console's size (columns, rows).
    size: (u32, u32),
}

const BASE: Case = Case {
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
    pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup equip:head:f \
    equip:body:e equip:main_hand:b equip:missile_weapon:c equip:missiles:d equip:feet:i";
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
        keys: Some("Down Enter Enter Enter"),
        ..BASE
    },
    Case {
        name: "kit-summary",
        scenario: "world",
        screen: Some("new-run"),
        keys: Some(
            "Down Down Down Down Down Down Down Down Enter Down Down Enter Enter Enter Down Down \
             Down Down Down Down Down Down Enter Enter",
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
    Case {
        name: "character-screen-lucky",
        // Seed 440 gives Tamsin a lucky day (#269): the Deep One, the title after the name.
        seed: Some(440),
        run: true,
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
    Case {
        name: "diseased",
        actions: "infect:the_shakes rest:24h*3",
        screen: Some("character"),
        size: (100, 36),
        ..BASE
    },
    Case {
        name: "alignment-strayed-character",
        scenario: "courtyard",
        character: Some("human,male,paladin"),
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
    Case {
        name: "weapon-skills-screen",
        scenario: "arena",
        actions: "move:E pickup equip:missile_weapon:a equip:missiles:b fire:30,16*12",
        screen: Some("weapon-skills"),
        ..BASE
    },
    Case {
        name: "abilities-menu",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
            pickup:a equip:off_hand:a",
        screen: Some("abilities"),
        ..BASE
    },
    // A paladin of some level: the Class group, Lay on hands drained, Smite ready.
    Case {
        name: "abilities-menu-class",
        character: Some("human,male,paladin"),
        actions: "xp:100000 levelup:auto use:lay_on_hands",
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
            move:E pickup equip:head:c",
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
    Case {
        name: "item-description",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup \
            equip:head:a",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "ornament-description",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    Case {
        name: "variant-description",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*3 move:N \
            pickup:b",
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
    Case {
        name: "scroll-too-close",
        scenario: "traps",
        actions: "move:E pickup",
        keys: Some("r a k Enter"),
        ..BASE
    },
    // Talk, quests, messages.
    Case {
        name: "dialogue-screen",
        actions: concat!(hermit!(), " chat:SE ask:SE:rooms"),
        screen: Some("dialogue"),
        ..BASE
    },
    Case {
        name: "quest-offer",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("dialogue"),
        ..BASE
    },
    Case {
        name: "quest-log",
        actions: concat!(hermit!(), " ask:SE:quest-rats ask:SE:accept-rats"),
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
        keys: Some("h"),
        ..BASE
    },
    Case {
        name: "trap-disarm-prompt",
        actions: "move:N*3 open:N move:N*2 move:E*67 open:S move:S*7 move:SW descend",
        keys: Some("a c"),
        ..BASE
    },
    Case {
        name: "death-screen",
        scenario: "arena",
        // Pip's birth sign has gifts (#265): seed 4 is one where the fight still ends in his death.
        seed: Some(4),
        actions: "lantern wait*14",
        ..BASE
    },
    Case {
        name: "death-screen-lucky",
        scenario: "arena",
        // Seed 142 gives Pip a lucky day, the Iron Scourge (#269), and the fight his death.
        seed: Some(142),
        actions: "lantern wait*14",
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

/// The UI console's text once `case` has started and its keys were pressed: one line a row,
/// trailing blanks trimmed.
fn render(case: &Case, content: Arc<Content>) -> String {
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
                    screens::converse,
                    screens::on_death,
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
    for key in case.keys.iter().flat_map(|keys| keys.split_whitespace()) {
        app.world_mut().write_message(press(key));
    }
    app.update();
    app.update();
    text(app.world().resource::<Console>())
}

/// Applies the actions input asked for, as the game plugin does.
fn apply_actions(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
    for PlayerAction(action) in actions.read() {
        game.apply(action);
    }
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
fn text(console: &Console) -> String {
    let cols = console.cols() as usize;
    let lines: Vec<String> = (console.cells().chunks(cols))
        .map(|row| {
            let line: String = row.iter().map(|c| c.map_or(' ', |c| c.ch)).collect();
            line.trim_end().to_string()
        })
        .collect();
    lines.join("\n")
}
