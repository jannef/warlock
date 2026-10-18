//! The screen stack (spec §6): every modal screen — main menu, new run, pause, settings,
//! controls, the `?` help, the inventory, the character screen, and choosing what to pick up or
//! drop — lives on one stack, bottom to top.
//!
//! - **Open**: [`Screens::open`] is the one way to open a screen, by [`ScreenName`]: `?` and Esc
//!   in play, screens opening sub-screens, the main menu at startup and after a run, and
//!   `--screen` all go through it. A screen opens on top of the one it was opened from, so
//!   closing it returns there as it was left.
//! - **Input**: `input::route_keys` is the one reader of key presses. While the stack is not
//!   empty, its top screen gets every press ([`ScreenStack::handle`]) and gameplay none. Back or
//!   Esc closes (pops) the top screen; choices that start, leave, or end a run ([`MenuChoice`])
//!   are applied by [`apply_choice`]. A screen that chooses a gameplay action (what to pick up
//!   or drop) closes and hands the action to the simulation ([`ScreenAction::Act`]).
//! - **Draw**: `ui::draw` clears the UI console, draws the gameplay UI when a run exists, then
//!   the top screen ([`ScreenStack::draw`]).
//!
//! Screens are plain state machines: they take [`Press`]es and answer with a [`ScreenAction`],
//! so they are testable without a window.

use super::character_screen::CharacterScreen;
use super::controls_screen::ControlsScreen;
use super::menu::{MainMenu, Press};
use super::motion::Motion;
use super::new_run_screen::NewRunScreen;
use super::object_screen::{ObjectList, ObjectScreen};
use super::settings_screen::SettingsScreen;
use crate::cli::Args;
use crate::game::{Game, GameContent};
use crate::input::{Bindings, InputMode};
use crate::render::console::Console;
use crate::render::grid::MapGrid;
use crate::save::{Resume, SaveCommand, SaveStore};
use crate::settings::Settings;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use clap::ValueEnum;
use warlock_core::Action;
use warlock_core::content::Content;
use warlock_core::perception::PerceptionView;

/// The screens that can be opened. Also the values of `--screen`, a dev flag for screenshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ScreenName {
    /// The main menu (no run).
    Main,
    NewRun,
    Settings,
    Controls,
    /// The pause menu over a run (Esc in play).
    Pause,
    /// The controls list over a run, read-only (`?` in play).
    Help,
    /// What the player carries and weighs (`i` in play).
    Inventory,
    /// The player's name, stats, and weight (`@` in play).
    Character,
    /// Choose what to pick up from the pile here (`,` or `g` where several objects lie).
    PickUp,
    /// Choose what to drop (`d` in play).
    Drop,
}

impl ScreenName {
    /// Screens that show something of a run and open only over one.
    pub fn needs_run(self) -> bool {
        matches!(
            self,
            ScreenName::Pause
                | ScreenName::Help
                | ScreenName::Inventory
                | ScreenName::Character
                | ScreenName::PickUp
                | ScreenName::Drop
        )
    }
}

/// A menu choice that starts, leaves, or ends a run; applied by [`apply_choice`].
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum MenuChoice {
    Continue,
    NewRun {
        scenario: String,
        seed: Option<u64>,
    },
    /// Pause menu: save the run and return to the main menu.
    SaveToMenu,
    /// Quit the game; from the pause menu, the run is saved first.
    Quit,
}

/// What a key press asks for: of a screen, or (only `Open`) of gameplay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenAction {
    /// Nothing more (the press may have changed the screen, the settings, or the bindings).
    Stay,
    /// Close this screen: Back, Esc, Resume.
    Close,
    /// Open a screen on top.
    Open(ScreenName),
    Choose(MenuChoice),
    /// Close this screen and apply a gameplay action (what to pick up or drop).
    Act(Action),
}

/// A modal screen.
#[derive(Debug)]
enum Screen {
    /// The main menu, or the pause menu over a run.
    Menu(MainMenu),
    NewRun(NewRunScreen),
    Settings(SettingsScreen),
    /// The controls screen, or the read-only `?` help.
    Controls(ControlsScreen),
    /// The inventory, or choosing what to pick up or drop.
    Objects(ObjectScreen),
    Character(CharacterScreen),
}

impl Screen {
    /// Screen `name`; `run` is what the player perceives in the run, if there is one.
    fn new(
        name: ScreenName,
        content: &Content,
        has_save: bool,
        run: Option<&PerceptionView<'_>>,
    ) -> Self {
        let objects = |list| Screen::Objects(ObjectScreen::new(list, run));
        match name {
            ScreenName::Main => Screen::Menu(MainMenu::main(has_save)),
            ScreenName::Pause => Screen::Menu(MainMenu::pause()),
            ScreenName::NewRun => Screen::NewRun(NewRunScreen::new(content, has_save)),
            ScreenName::Settings => Screen::Settings(SettingsScreen::default()),
            ScreenName::Controls => Screen::Controls(ControlsScreen::default()),
            ScreenName::Help => Screen::Controls(ControlsScreen::help()),
            ScreenName::Inventory => objects(ObjectList::Inventory),
            ScreenName::Character => Screen::Character(CharacterScreen::new(run)),
            ScreenName::PickUp => objects(ObjectList::PickUp),
            ScreenName::Drop => objects(ObjectList::Drop),
        }
    }

    #[cfg(test)]
    fn name(&self) -> ScreenName {
        match self {
            Screen::Menu(menu) if menu.is_pause() => ScreenName::Pause,
            Screen::Menu(_) => ScreenName::Main,
            Screen::NewRun(_) => ScreenName::NewRun,
            Screen::Settings(_) => ScreenName::Settings,
            Screen::Controls(screen) if screen.is_help() => ScreenName::Help,
            Screen::Controls(_) => ScreenName::Controls,
            Screen::Objects(screen) => match screen.list() {
                ObjectList::Inventory => ScreenName::Inventory,
                ObjectList::PickUp => ScreenName::PickUp,
                ObjectList::Drop => ScreenName::Drop,
            },
            Screen::Character(_) => ScreenName::Character,
        }
    }
}

/// Every open modal screen, bottom to top. While it is not empty, the top screen owns the
/// keyboard: no game time passes.
#[derive(Resource, Debug, Default)]
pub struct ScreenStack {
    screens: Vec<Screen>,
    /// Something the player should know, shown by the main or pause menu: why a choice failed,
    /// or that the run was saved. It lasts until the player picks something.
    notice: Option<String>,
}

impl ScreenStack {
    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    /// The open screens, bottom to top.
    #[cfg(test)]
    fn names(&self) -> Vec<ScreenName> {
        self.screens.iter().map(Screen::name).collect()
    }

    /// Closes the top screen.
    pub fn close(&mut self) {
        self.screens.pop();
    }

    /// Closes every screen.
    fn clear(&mut self) {
        self.screens.clear();
        self.notice = None;
    }

    /// Applies one key press to the top screen; settings and bindings change in place. A held
    /// key's repeats may move the selection or change a value, but never open, close, or choose.
    pub fn handle(
        &mut self,
        press: &Press,
        settings: &mut Settings,
        bindings: &mut Bindings,
    ) -> ScreenAction {
        let leave = |left: bool| {
            if left {
                ScreenAction::Close
            } else {
                ScreenAction::Stay
            }
        };
        let action = match self.screens.last_mut() {
            None => ScreenAction::Stay,
            Some(Screen::Menu(menu)) => menu.handle(press),
            Some(Screen::NewRun(screen)) => screen.handle(press),
            Some(Screen::Settings(screen)) => leave(screen.handle(press, settings)),
            Some(Screen::Controls(screen)) => leave(screen.handle(press, bindings)),
            Some(Screen::Objects(screen)) => screen.handle(press, bindings),
            Some(Screen::Character(screen)) => screen.handle(press, bindings),
        };
        if action == ScreenAction::Stay {
            return action;
        }
        if press.repeat {
            return ScreenAction::Stay;
        }
        self.notice = None;
        action
    }

    /// Shows why a menu choice failed, on the menu it was made from (closing New run).
    /// `save_unusable` drops Continue, as there is nothing to resume.
    fn fail(&mut self, message: String, save_unusable: bool) {
        while self.screens.len() > 1 && !matches!(self.screens.last(), Some(Screen::Menu(_))) {
            self.screens.pop();
        }
        if save_unusable && let Some(Screen::Menu(menu)) = self.screens.last_mut() {
            menu.drop_continue();
        }
        self.notice = Some(message);
    }

    /// Draws the top screen. Each screen is a panel at least as large as the menu it was opened
    /// from, so the screens below would not show anyway.
    pub fn draw(&self, console: &mut Console, settings: &Settings, bindings: &Bindings) {
        match self.screens.last() {
            None => {}
            Some(Screen::Menu(menu)) => menu.draw(console, self.notice.as_deref()),
            Some(Screen::NewRun(screen)) => screen.draw(console),
            Some(Screen::Settings(screen)) => screen.draw(console, settings),
            Some(Screen::Controls(screen)) => screen.draw(console, bindings),
            Some(Screen::Objects(screen)) => screen.draw(console, bindings),
            Some(Screen::Character(screen)) => screen.draw(console, bindings),
        }
    }
}

/// The screen stack, with what opening a screen needs.
#[derive(SystemParam)]
pub struct Screens<'w> {
    pub stack: ResMut<'w, ScreenStack>,
    content: Res<'w, GameContent>,
    store: Res<'w, SaveStore>,
    /// The run, for screens that show what the player perceives (the inventory, a pile, the
    /// character screen).
    game: Option<Res<'w, Game>>,
}

impl Screens<'_> {
    /// Opens screen `name` on top of the stack. The one way screens open.
    pub fn open(&mut self, name: ScreenName) {
        let view = self.game.as_ref().map(|game| game.world.perception());
        let screen = Screen::new(name, &self.content.0, self.store.has_save(), view.as_ref());
        self.stack.screens.push(screen);
    }
}

/// Opens the screens the command line asks for (see [`startup_screens`]).
pub(super) fn open_startup_screens(args: Res<Args>, game: Option<Res<Game>>, mut screens: Screens) {
    for name in startup_screens(args.screen, game.is_some()) {
        screens.open(name);
    }
}

/// The screens open at startup, bottom to top: the main menu when there is no run, and
/// `--screen` over the main menu or the run. In a run, settings and controls are reached from
/// the pause menu, so they open over it.
fn startup_screens(screen: Option<ScreenName>, in_run: bool) -> Vec<ScreenName> {
    use ScreenName::*;
    match (in_run, screen) {
        (false, Some(s @ (NewRun | Settings | Controls))) => vec![Main, s],
        (false, _) => vec![Main],
        (true, Some(s @ (Settings | Controls))) => vec![Pause, s],
        (true, Some(s)) if s.needs_run() => vec![s],
        (true, _) => Vec::new(),
    }
}

/// Applies menu choices: starts or resumes a run, saves and returns to the main menu, or quits.
/// A choice that fails shows why on the menu it was made from.
pub(super) fn apply_choice(
    mut commands: Commands,
    mut choices: MessageReader<MenuChoice>,
    mut screens: Screens,
    game: Option<Res<Game>>,
    mut grid: ResMut<MapGrid>,
    mut mode: ResMut<InputMode>,
    mut saves: MessageWriter<SaveCommand>,
    mut exit: MessageWriter<AppExit>,
) {
    for choice in choices.read() {
        match choice {
            // From the pause menu: quitting keeps the run.
            MenuChoice::Quit if game.is_some() => {
                saves.write(SaveCommand::SaveAndQuit);
            }
            MenuChoice::Quit => {
                exit.write(AppExit::Success);
            }
            MenuChoice::SaveToMenu => {
                let Some(game) = &game else { continue };
                match screens.store.save_run(&game.world) {
                    Ok(()) => {
                        // Back to the main menu: nothing of the run stays on screen.
                        grid.console.clear();
                        grid.free_terrain.clear();
                        grid.clip = Rect::default();
                        *mode = InputMode::Normal;
                        commands.insert_resource(Motion::default());
                        commands.remove_resource::<Game>();
                        screens.stack.clear();
                        screens.open(ScreenName::Main);
                        screens.stack.notice = Some("Game saved.".into());
                    }
                    Err(e) => screens.stack.fail(e, false),
                }
            }
            MenuChoice::Continue => match Game::resumed(&screens.store, &screens.content.0) {
                Resume::Loaded(world) => {
                    commands.insert_resource(Game::from_world(world));
                    screens.stack.clear();
                }
                Resume::Failed(message) => screens.stack.fail(message, true),
                Resume::NoSave => screens.stack.fail("There is no saved run.".into(), true),
            },
            MenuChoice::NewRun { scenario, seed } => {
                match Game::new_run(screens.content.0.clone(), scenario, *seed) {
                    Ok(game) => {
                        commands.insert_resource(game);
                        screens.stack.clear();
                    }
                    Err(message) => screens.stack.fail(message, false),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::PlayerAction;
    use crate::input::route_keys;
    use crate::ui::DevOverlays;
    use crate::ui::menu::Nav;
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
    use clap::Parser;
    use std::sync::Arc;
    use warlock_core::{Action, Dir};

    fn content() -> Arc<Content> {
        Arc::new(Content::builtin().expect("content"))
    }

    fn press(stack: &mut ScreenStack, press: Press) -> ScreenAction {
        let (mut s, mut b) = (Settings::default(), Bindings::default());
        stack.handle(&press, &mut s, &mut b)
    }

    /// The showcase at its start, standing on the start room's pile (three stacks), having
    /// picked up its first stack.
    fn on_the_pile(content: Arc<Content>) -> warlock_core::World {
        let mut world = warlock_core::World::new(content, "showcase", None).expect("run");
        for action in [
            Action::Move(Dir::SW),
            Action::Move(Dir::SW),
            Action::PickUp(warlock_core::Letters::of([0])),
        ] {
            world.apply(&action);
        }
        world
    }

    fn stack_of(names: &[ScreenName], has_save: bool) -> ScreenStack {
        let content = content();
        let world = on_the_pile(content.clone());
        let view = world.perception();
        let mut stack = ScreenStack::default();
        for name in names {
            let run = name.needs_run().then_some(&view);
            stack
                .screens
                .push(Screen::new(*name, &content, has_save, run));
        }
        stack
    }

    #[test]
    fn screens_push_and_pop() {
        use ScreenName::*;
        let mut stack = stack_of(&[Pause], false);
        assert_eq!(stack.names(), [Pause]);
        assert_eq!(press(&mut stack, Press::nav(Nav::Down)), ScreenAction::Stay);
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Enter)),
            ScreenAction::Open(Settings)
        );
        stack
            .screens
            .push(Screen::new(Settings, &content(), false, None));
        assert_eq!(stack.names(), [Pause, Settings]);
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Back)),
            ScreenAction::Close
        );
        stack.close();
        assert_eq!(stack.names(), [Pause]);
        // The pause menu is as it was left: on the Settings row.
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Enter)),
            ScreenAction::Open(Settings)
        );
        stack.close();
        assert!(stack.is_empty());
    }

    #[test]
    fn repeats_never_open_close_or_choose() {
        let mut stack = stack_of(&[ScreenName::Pause], false);
        let held = |nav| Press {
            repeat: true,
            ..Press::nav(nav)
        };
        assert_eq!(press(&mut stack, held(Nav::Back)), ScreenAction::Stay);
        assert_eq!(press(&mut stack, held(Nav::Enter)), ScreenAction::Stay);
        assert_eq!(press(&mut stack, held(Nav::Down)), ScreenAction::Stay);
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Enter)),
            ScreenAction::Open(ScreenName::Settings)
        );
    }

    #[test]
    fn a_failed_choice_returns_to_the_menu_with_a_notice() {
        use ScreenName::*;
        let mut stack = stack_of(&[Main, NewRun], true);
        stack.fail("Could not start.".into(), false);
        assert_eq!(stack.names(), [Main]);
        assert_eq!(stack.notice.as_deref(), Some("Could not start."));
        // Picking something clears the notice; a failed Continue drops the option.
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::Continue)
        );
        assert_eq!(stack.notice, None);
        stack.fail("Could not resume.".into(), true);
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Enter)),
            ScreenAction::Open(NewRun)
        );
    }

    #[test]
    fn startup_screens_follow_the_command_line() {
        use ScreenName::*;
        assert_eq!(startup_screens(None, false), [Main]);
        assert_eq!(startup_screens(Some(Controls), false), [Main, Controls]);
        assert_eq!(startup_screens(None, true), []);
        assert_eq!(startup_screens(Some(Pause), true), [Pause]);
        assert_eq!(startup_screens(Some(Help), true), [Help]);
        assert_eq!(startup_screens(Some(Settings), true), [Pause, Settings]);
        assert_eq!(startup_screens(Some(Inventory), true), [Inventory]);
        assert_eq!(startup_screens(Some(Character), true), [Character]);
        assert_eq!(startup_screens(Some(Character), false), [Main]);
        assert_eq!(startup_screens(Some(PickUp), true), [PickUp]);
        assert_eq!(startup_screens(Some(Drop), false), [Main]);
    }

    #[test]
    fn every_screen_draws_in_a_small_console() {
        let settings = Settings::default();
        let bindings = Bindings::default();
        for name in ScreenName::value_variants() {
            let mut stack = stack_of(&[*name], true);
            stack.notice = Some("A notice long enough to wrap onto a second line of text.".into());
            for (cols, rows) in [(80, 30), (40, 12), (5, 3), (0, 0)] {
                let mut console = Console::default();
                console.resize(cols, rows);
                stack.draw(&mut console, &settings, &bindings);
            }
        }
    }

    // The input router in an app: the one reader of key presses, for the top screen or gameplay.

    fn key(code: KeyCode, text: &str) -> KeyboardInput {
        KeyboardInput {
            key_code: code,
            logical_key: if text.is_empty() {
                Key::Unidentified(NativeKey::Unidentified)
            } else {
                Key::Character(text.into())
            },
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    const ESC: (KeyCode, &str) = (KeyCode::Escape, "");
    const DOWN: (KeyCode, &str) = (KeyCode::ArrowDown, "");
    const ENTER: (KeyCode, &str) = (KeyCode::Enter, "");
    const HELP: (KeyCode, &str) = (KeyCode::Slash, "?");
    const WAIT: (KeyCode, &str) = (KeyCode::Period, ".");

    /// An app with the router and the screen systems, on the main menu or in a run.
    fn app(run: bool) -> App {
        let content = content();
        let game = run.then(|| Game::new_run(content.clone(), "showcase", None).expect("run"));
        app_with(content, game)
    }

    /// An app with the router and the screen systems, in `game` or on the main menu.
    fn app_with(content: Arc<Content>, game: Option<Game>) -> App {
        let args = Args::parse_from(["warlock", "--capture", "unused.png"]);
        let mut app = App::new();
        app.add_message::<KeyboardInput>()
            .add_message::<PlayerAction>()
            .add_message::<SaveCommand>()
            .add_message::<MenuChoice>()
            .insert_resource(SaveStore::from_args(&args))
            .insert_resource(args)
            .insert_resource(GameContent(content.clone()))
            .init_resource::<ScreenStack>()
            .init_resource::<Settings>()
            .init_resource::<Bindings>()
            .init_resource::<InputMode>()
            .init_resource::<DevOverlays>()
            .init_resource::<MapGrid>()
            .init_resource::<Actions>()
            .add_systems(Update, (route_keys, apply_choice, count_actions).chain());
        match game {
            Some(game) => {
                app.insert_resource(game);
            }
            None => {
                app.add_systems(Startup, open_startup_screens);
            }
        }
        app
    }

    /// Player actions seen so far.
    #[derive(Resource, Default)]
    struct Actions(Vec<Action>);

    fn count_actions(mut actions: MessageReader<PlayerAction>, mut seen: ResMut<Actions>) {
        seen.0.extend(actions.read().map(|a| a.0.clone()));
    }

    /// Sends `keys` within one frame and runs it.
    fn frame(app: &mut App, keys: &[(KeyCode, &str)]) {
        for (code, text) in keys {
            app.world_mut().write_message(key(*code, text));
        }
        app.update();
    }

    fn names(app: &App) -> Vec<ScreenName> {
        app.world().resource::<ScreenStack>().names()
    }

    fn actions(app: &App) -> Vec<Action> {
        app.world().resource::<Actions>().0.clone()
    }

    #[test]
    fn esc_opens_pause_and_esc_returns_to_the_game() {
        let mut app = app(true);
        frame(&mut app, &[ESC]);
        assert_eq!(names(&app), [ScreenName::Pause]);
        // Messages can stay readable for two frames: the opening press must not also close it.
        for _ in 0..3 {
            frame(&mut app, &[]);
            assert_eq!(names(&app), [ScreenName::Pause]);
        }
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[]);
        assert!(
            names(&app).is_empty(),
            "the closing press does not reopen it"
        );
        frame(&mut app, &[WAIT]);
        assert_eq!(actions(&app), [Action::Wait]);
    }

    #[test]
    fn help_opens_on_question_mark_and_closes_on_it_or_esc() {
        let mut app = app(true);
        frame(&mut app, &[HELP]);
        assert_eq!(names(&app), [ScreenName::Help]);
        frame(&mut app, &[]);
        assert_eq!(names(&app), [ScreenName::Help]);
        frame(&mut app, &[HELP]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[HELP]);
        assert_eq!(names(&app), [ScreenName::Help]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[]);
        assert!(
            names(&app).is_empty(),
            "Esc that closed help is not a pause"
        );
        assert!(actions(&app).is_empty());
    }

    #[test]
    fn presses_go_to_the_top_screen_only() {
        let mut app = app(true);
        // Presses in the frame after Esc already go to the pause menu: Down, Enter opens
        // Settings, and the wait never reaches gameplay.
        frame(&mut app, &[ESC, DOWN, ENTER, WAIT]);
        assert_eq!(names(&app), [ScreenName::Pause, ScreenName::Settings]);
        // Down moves the settings selection only; Esc returns to the pause menu as it was left.
        frame(&mut app, &[DOWN, WAIT, (KeyCode::KeyJ, "j"), ESC]);
        assert_eq!(names(&app), [ScreenName::Pause]);
        frame(&mut app, &[ENTER]);
        assert_eq!(names(&app), [ScreenName::Pause, ScreenName::Settings]);
        assert!(actions(&app).is_empty(), "gameplay saw no presses");
        // Esc twice is back in play, where the wait is a turn.
        frame(&mut app, &[ESC, ESC, WAIT]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app), [Action::Wait]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
    }

    #[test]
    fn prompts_stay_gameplay_input() {
        let mut app = app(true);
        // The direction prompt: Esc cancels it rather than pausing.
        frame(&mut app, &[(KeyCode::KeyO, "o")]);
        assert!(matches!(
            app.world().resource::<InputMode>(),
            InputMode::Direction(_)
        ));
        frame(&mut app, &[ESC]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(names(&app).is_empty());
        frame(&mut app, &[(KeyCode::KeyO, "o"), (KeyCode::KeyK, "k")]);
        assert_eq!(actions(&app), [Action::Open(Dir::N)]);
        // Look mode: Esc leaves it, `?` does nothing there.
        frame(&mut app, &[(KeyCode::KeyX, "x"), HELP]);
        assert!(matches!(
            app.world().resource::<InputMode>(),
            InputMode::Look(_)
        ));
        frame(&mut app, &[ESC]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(names(&app).is_empty());
        // The rest prompt: Esc cancels it; a letter answers it.
        let rest = (KeyCode::KeyR, "R");
        frame(&mut app, &[rest]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Rest);
        frame(&mut app, &[ESC]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(names(&app).is_empty());
        frame(&mut app, &[rest, (KeyCode::KeyH, "h")]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(matches!(
            actions(&app).last(),
            Some(Action::Rest(warlock_core::Rest::For(_)))
        ));
    }

    #[test]
    fn main_menu_starts_a_run_from_presses_in_one_frame() {
        let mut app = app(false);
        app.update();
        assert_eq!(names(&app), [ScreenName::Main]);
        // Enter opens New run and the next Enter, in the same frame, starts the showcase.
        frame(&mut app, &[ENTER, ENTER, WAIT]);
        assert!(app.world().contains_resource::<Game>());
        assert!(names(&app).is_empty());
        assert!(
            actions(&app).is_empty(),
            "presses after the choice are dropped"
        );
        frame(&mut app, &[WAIT]);
        assert_eq!(actions(&app), [Action::Wait]);
    }

    #[test]
    fn pick_up_drop_and_inventory_open_screens_that_choose_actions() {
        use warlock_core::Letters;
        let content = content();
        let game = Game::from_world(on_the_pile(content.clone()));
        let mut app = app_with(content, Some(game));
        // Two stacks are left on the pile: `,` opens the choice; a letter and Enter pick up,
        // and the next press in the same frame is gameplay again.
        let (comma, b, i, d) = (
            (KeyCode::Comma, ","),
            (KeyCode::KeyB, "b"),
            (KeyCode::KeyI, "i"),
            (KeyCode::KeyD, "d"),
        );
        frame(&mut app, &[comma]);
        assert_eq!(names(&app), [ScreenName::PickUp]);
        frame(&mut app, &[b, ENTER, WAIT]);
        assert!(names(&app).is_empty());
        assert_eq!(
            actions(&app),
            [Action::PickUp(Letters::of([1])), Action::Wait]
        );
        // `g` picks up too; Esc cancels without an action.
        frame(&mut app, &[(KeyCode::KeyG, "g"), ESC]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).len(), 2);
        // The inventory opens and closes on `i`; `d` chooses what to drop.
        frame(&mut app, &[i]);
        assert_eq!(names(&app), [ScreenName::Inventory]);
        frame(&mut app, &[i]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[d, ENTER]);
        assert_eq!(actions(&app).last(), Some(&Action::Drop(Letters::of([0]))));
        assert!(names(&app).is_empty());
    }

    #[test]
    fn the_character_screen_opens_and_closes_on_at_and_takes_no_time() {
        let mut app = app(true);
        let at = (KeyCode::Digit2, "@");
        frame(&mut app, &[at]);
        assert_eq!(names(&app), [ScreenName::Character]);
        // Presses go to the screen: no gameplay while it is open.
        frame(&mut app, &[WAIT, (KeyCode::KeyK, "k")]);
        assert_eq!(names(&app), [ScreenName::Character]);
        frame(&mut app, &[at]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[at, ESC]);
        assert!(names(&app).is_empty(), "Esc closes it rather than pausing");
        assert!(actions(&app).is_empty());
    }

    #[test]
    fn single_stacks_and_empty_hands_need_no_screen() {
        use warlock_core::Letters;
        let mut app = app(true);
        // Nothing here and nothing carried: the actions are refused by the core, no screen.
        frame(&mut app, &[(KeyCode::Comma, ","), (KeyCode::KeyD, "d")]);
        assert!(names(&app).is_empty());
        assert_eq!(
            actions(&app),
            [
                Action::PickUp(Letters::default()),
                Action::Drop(Letters::default())
            ]
        );
    }

    #[test]
    fn quit_from_pause_saves_the_run() {
        let mut app = app(true);
        frame(&mut app, &[ESC, (KeyCode::ArrowUp, ""), ENTER]);
        let saves = app.world().resource::<Messages<SaveCommand>>();
        let sent: Vec<_> = saves.iter_current_update_messages().copied().collect();
        assert_eq!(sent, [SaveCommand::SaveAndQuit]);
    }
}
