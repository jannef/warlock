//! The screen stack (spec §6): every modal screen — main menu, new run and character creation,
//! pause, settings, controls, the `?` help, the inventory, the equipment screen and its choice of
//! what goes in a slot, the character screen, and choosing what to pick up, drop, or throw —
//! lives on one stack, bottom to top.
//!
//! - **Open**: [`Screens::open`] is the one way to open a screen, by [`ScreenName`]: `?` and Esc
//!   in play, screens opening sub-screens, the main menu at startup and after a run, and
//!   `--screen` all go through it. A screen opens on top of the one it was opened from, so
//!   closing it returns there as it was left; [`ScreenAction::Return`] closes several at once
//!   (the character summary going back to a step). Character creation's steps read what was
//!   chosen below them off the stack ([`ScreenStack::draft`]), and so does the choice of what
//!   goes in a slot: its slot is the one selected on the equipment screen beneath it.
//! - **Input**: `input::route_keys` is the one reader of key presses. While the stack is not
//!   empty, its top screen gets every press ([`ScreenStack::handle`]) and gameplay none. Back or
//!   Esc closes (pops) the top screen; choices that start, leave, or end a run ([`MenuChoice`])
//!   are applied by [`apply_choice`]. A screen that chooses a gameplay action (what to pick up,
//!   drop, wear, or wield) closes and hands the action to the simulation ([`ScreenAction::Act`]);
//!   the equipment screen beneath a choice stays open and [`refresh`] shows it the result.
//! - **Draw**: `ui::draw` clears the UI console, draws the gameplay UI when a run exists, then
//!   the top screen ([`ScreenStack::draw`]).
//!
//! Screens are plain state machines: they take [`Press`]es and answer with a [`ScreenAction`],
//! so they are testable without a window.

use super::character_screen::CharacterScreen;
use super::chargen_screen::{ChoiceScreen, Draft, Picks, Step, SummaryScreen};
use super::controls_screen::ControlsScreen;
use super::death_screen::DeathScreen;
use super::equipment_screen::{EquipmentScreen, ItemScreen};
use super::menu::{MainMenu, Press};
use super::motion::Motion;
use super::new_run_screen::NewRunScreen;
use super::object_screen::{ObjectList, ObjectScreen};
use super::settings_screen::SettingsScreen;
use super::skills_screen::SkillsScreen;
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
use warlock_core::character::CharacterSpec;
use warlock_core::content::{Content, SlotId};
use warlock_core::perception::PerceptionView;

/// The screens that can be opened. Also the values of `--screen`, a dev flag for screenshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ScreenName {
    /// The main menu (no run).
    Main,
    NewRun,
    /// Character creation, over New run: choose a race.
    Race,
    /// Choose a gender (after the race).
    Gender,
    /// Choose a class (after the gender).
    Class,
    /// The character before the run starts: start, or go back to a step.
    Summary,
    Settings,
    Controls,
    /// The pause menu over a run (Esc in play).
    Pause,
    /// The controls list over a run, read-only (`?` in play).
    Help,
    /// What the player carries and weighs (`i` in play).
    Inventory,
    /// What the player wears and wields, slot by slot (`e` in play).
    Equipment,
    /// Choose what goes in the slot selected on the equipment screen beneath (a slot's letter
    /// there).
    EquipItem,
    /// The player's name, stats, and weight (`@` in play).
    Character,
    /// The player's trained skills and their values (`S` in play, or from the character
    /// screen).
    Skills,
    /// Choose what to pick up from the pile here (`,` or `g` where several objects lie).
    PickUp,
    /// Choose what to drop (`d` in play).
    Drop,
    /// Choose what to throw (`t` in play, with no throwing weapon readied).
    Throw,
    /// Choose what to eat (`E` in play): food lying here, then food in the pack.
    Eat,
    /// The run is over: who died, where, when, and of what. It opens by itself when the player
    /// dies, so it is no `--screen`.
    #[value(skip)]
    Death,
}

impl ScreenName {
    /// Screens that show something of a run and open only over one.
    pub fn needs_run(self) -> bool {
        matches!(
            self,
            ScreenName::Pause
                | ScreenName::Help
                | ScreenName::Inventory
                | ScreenName::Equipment
                | ScreenName::EquipItem
                | ScreenName::Character
                | ScreenName::Skills
                | ScreenName::PickUp
                | ScreenName::Drop
                | ScreenName::Throw
                | ScreenName::Eat
                | ScreenName::Death
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
        /// The character chosen; `None` for the scenario's (content without races or classes).
        character: Option<CharacterSpec>,
    },
    /// Pause menu: save the run and return to the main menu.
    SaveToMenu,
    /// Death screen: the run is over (its save already gone); return to the main menu.
    LeaveRun,
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
    /// Close screens until this one is on top (as it was left).
    Return(ScreenName),
    Choose(MenuChoice),
    /// Close this screen and apply a gameplay action (what to pick up, drop, wear, or wield).
    Act(Action),
    /// Close this screen and aim inventory stack `usize` to throw it (what to throw).
    Aim(usize),
}

/// A modal screen.
#[derive(Debug)]
enum Screen {
    /// The main menu, or the pause menu over a run.
    Menu(MainMenu),
    NewRun(NewRunScreen),
    /// A step of character creation.
    Choice(ChoiceScreen),
    /// Boxed: it holds the whole character.
    Summary(Box<SummaryScreen>),
    Settings(SettingsScreen),
    /// The controls screen, or the read-only `?` help.
    Controls(ControlsScreen),
    /// The inventory, or choosing what to pick up or drop.
    Objects(ObjectScreen),
    Equipment(EquipmentScreen),
    /// Choosing what goes in a slot.
    EquipItem(ItemScreen),
    Character(CharacterScreen),
    Skills(SkillsScreen),
    Death(DeathScreen),
}

impl Screen {
    /// Screen `name`; `run` is what the player perceives in the run, if there is one, `draft`
    /// what New run has chosen on the screens below, and `slot` the slot selected on the
    /// equipment screen beneath, if any (the first slot if none).
    fn new(
        name: ScreenName,
        content: &Content,
        has_save: bool,
        run: Option<&PerceptionView<'_>>,
        draft: &Draft,
        slot: Option<SlotId>,
    ) -> Self {
        let objects = |list| Screen::Objects(ObjectScreen::new(list, run));
        let choice = |step| Screen::Choice(ChoiceScreen::new(step, content, draft));
        match name {
            ScreenName::Main => Screen::Menu(MainMenu::main(has_save)),
            ScreenName::Pause => Screen::Menu(MainMenu::pause()),
            ScreenName::NewRun => Screen::NewRun(NewRunScreen::new(content, has_save)),
            ScreenName::Race => choice(Step::Race),
            ScreenName::Gender => choice(Step::Gender),
            ScreenName::Class => choice(Step::Class),
            ScreenName::Summary => Screen::Summary(Box::new(SummaryScreen::new(content, draft))),
            ScreenName::Settings => Screen::Settings(SettingsScreen::default()),
            ScreenName::Controls => Screen::Controls(ControlsScreen::default()),
            ScreenName::Help => Screen::Controls(ControlsScreen::help()),
            ScreenName::Inventory => objects(ObjectList::Inventory),
            ScreenName::Equipment => Screen::Equipment(EquipmentScreen::new(run)),
            ScreenName::EquipItem => {
                Screen::EquipItem(ItemScreen::new(run, slot.unwrap_or(SlotId(0))))
            }
            ScreenName::Character => Screen::Character(CharacterScreen::new(run)),
            ScreenName::Skills => Screen::Skills(SkillsScreen::new(run)),
            ScreenName::PickUp => objects(ObjectList::PickUp),
            ScreenName::Drop => objects(ObjectList::Drop),
            ScreenName::Throw => objects(ObjectList::Throw),
            ScreenName::Eat => objects(ObjectList::Eat),
            ScreenName::Death => Screen::Death(DeathScreen::new(run)),
        }
    }

    fn name(&self) -> ScreenName {
        match self {
            Screen::Menu(menu) if menu.is_pause() => ScreenName::Pause,
            Screen::Menu(_) => ScreenName::Main,
            Screen::NewRun(_) => ScreenName::NewRun,
            Screen::Choice(screen) => screen.step().screen(),
            Screen::Summary(_) => ScreenName::Summary,
            Screen::Settings(_) => ScreenName::Settings,
            Screen::Controls(screen) if screen.is_help() => ScreenName::Help,
            Screen::Controls(_) => ScreenName::Controls,
            Screen::Objects(screen) => match screen.list() {
                ObjectList::Inventory => ScreenName::Inventory,
                ObjectList::PickUp => ScreenName::PickUp,
                ObjectList::Drop => ScreenName::Drop,
                ObjectList::Throw => ScreenName::Throw,
                ObjectList::Eat => ScreenName::Eat,
            },
            Screen::Equipment(_) => ScreenName::Equipment,
            Screen::EquipItem(_) => ScreenName::EquipItem,
            Screen::Character(_) => ScreenName::Character,
            Screen::Skills(_) => ScreenName::Skills,
            Screen::Death(_) => ScreenName::Death,
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
    /// Character creation choices made before going back to an earlier step, preselected when
    /// their steps open again.
    picks: Picks,
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

    /// Closes screens until `name` is on top; all of them if it is not open.
    pub fn return_to(&mut self, name: ScreenName) {
        while self.screens.last().is_some_and(|s| s.name() != name) {
            self.screens.pop();
        }
    }

    /// Closes every screen.
    fn clear(&mut self) {
        self.screens.clear();
        self.notice = None;
        self.picks = Picks::default();
    }

    /// What New run has chosen so far: the scenario and seed, and each character creation
    /// step's selection, read off the open screens; plus the choices remembered from steps since
    /// left.
    pub fn draft(&self) -> Draft {
        let mut draft = Draft {
            remembered: self.picks.clone(),
            ..Draft::default()
        };
        for screen in &self.screens {
            match screen {
                Screen::NewRun(screen) => draft.scenario = screen.pick(),
                Screen::Choice(screen) => {
                    if let Some(id) = screen.chosen() {
                        draft.chosen.set(screen.step(), id);
                    }
                }
                _ => {}
            }
        }
        draft
    }

    /// The slot selected on the topmost equipment screen, if one is open: what a choice opened
    /// from it is for.
    fn selected_slot(&self) -> Option<SlotId> {
        self.screens.iter().rev().find_map(|s| match s {
            Screen::Equipment(screen) => Some(screen.selected()),
            _ => None,
        })
    }

    /// Whether an equipment screen is open (it shows the run as it changes, [`refresh`]).
    fn shows_equipment(&self) -> bool {
        (self.screens.iter()).any(|s| matches!(s, Screen::Equipment(_)))
    }

    /// Reads the run again into the screens that stay open while it changes: the equipment
    /// screen, beneath a choice that was just done.
    fn refresh(&mut self, view: &PerceptionView<'_>) {
        for screen in &mut self.screens {
            if let Screen::Equipment(screen) = screen {
                screen.refresh(view);
            }
        }
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
            Some(Screen::Choice(screen)) => {
                let action = screen.handle(press);
                if let (ScreenAction::Open(_), Some(id)) = (&action, screen.chosen()) {
                    self.picks.set(screen.step(), id);
                }
                action
            }
            Some(Screen::Summary(screen)) => screen.handle(press),
            Some(Screen::Settings(screen)) => leave(screen.handle(press, settings)),
            Some(Screen::Controls(screen)) => leave(screen.handle(press, bindings)),
            Some(Screen::Objects(screen)) => screen.handle(press, bindings),
            Some(Screen::Equipment(screen)) => screen.handle(press, bindings),
            Some(Screen::EquipItem(screen)) => screen.handle(press, bindings),
            Some(Screen::Character(screen)) => screen.handle(press, bindings),
            Some(Screen::Skills(screen)) => screen.handle(press, bindings),
            Some(Screen::Death(screen)) => screen.handle(press),
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
            Some(Screen::Choice(screen)) => screen.draw(console),
            Some(Screen::Summary(screen)) => screen.draw(console),
            Some(Screen::Settings(screen)) => screen.draw(console, settings),
            Some(Screen::Controls(screen)) => screen.draw(console, bindings),
            Some(Screen::Objects(screen)) => screen.draw(console, bindings),
            Some(Screen::Equipment(screen)) => screen.draw(console, bindings),
            Some(Screen::EquipItem(screen)) => screen.draw(console, bindings),
            Some(Screen::Character(screen)) => screen.draw(console, bindings),
            Some(Screen::Skills(screen)) => screen.draw(console, bindings),
            Some(Screen::Death(screen)) => screen.draw(console),
        }
    }

    /// Whether the death screen is open.
    fn shows_death(&self) -> bool {
        (self.screens.iter()).any(|s| matches!(s, Screen::Death(_)))
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
        let (draft, slot) = (self.stack.draft(), self.stack.selected_slot());
        let (content, has_save) = (&self.content.0, self.store.has_save());
        let screen = Screen::new(name, content, has_save, view.as_ref(), &draft, slot);
        self.stack.screens.push(screen);
    }
}

/// Shows the screens that stay open while the run changes what they show (the equipment screen,
/// beneath a choice that was just done) the run as it is now.
pub(super) fn refresh(game: Res<Game>, mut stack: ResMut<ScreenStack>) {
    if game.is_changed() && stack.shows_equipment() {
        stack.refresh(&game.world.perception());
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
/// the pause menu, so they open over it; character creation's steps open over New run and the
/// steps before them, each with its preselection.
fn startup_screens(screen: Option<ScreenName>, in_run: bool) -> Vec<ScreenName> {
    use ScreenName::*;
    let creation = [Race, Gender, Class, Summary];
    match (in_run, screen) {
        (false, Some(s)) if creation.contains(&s) => {
            let steps = creation.iter().take_while(|&&c| c != s);
            [Main, NewRun]
                .into_iter()
                .chain(steps.copied())
                .chain([s])
                .collect()
        }
        (false, Some(s @ (NewRun | Settings | Controls))) => vec![Main, s],
        (false, _) => vec![Main],
        (true, Some(s @ (Settings | Controls))) => vec![Pause, s],
        (true, Some(EquipItem)) => vec![Equipment, EquipItem],
        (true, Some(s)) if s.needs_run() => vec![s],
        (true, _) => Vec::new(),
    }
}

/// Ends the run on screen: nothing of it stays (the map, motion, a prompt), and the game goes.
fn leave_run(commands: &mut Commands, grid: &mut MapGrid, mode: &mut InputMode) {
    grid.console.clear();
    grid.free_terrain.clear();
    grid.clip = Rect::default();
    *mode = InputMode::Normal;
    commands.insert_resource(Motion::default());
    commands.remove_resource::<Game>();
}

/// When the run has ended in death: its save is deleted (permadeath), and the death screen
/// opens over the run as it ended, in place of whatever screen was open. Once.
pub(super) fn on_death(game: Res<Game>, mut screens: Screens) {
    if !game.is_changed() || game.world.death().is_none() || screens.stack.shows_death() {
        return;
    }
    if let Err(e) = screens.store.delete_run() {
        warn!("{e}");
    }
    screens.stack.clear();
    screens.open(ScreenName::Death);
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
                        leave_run(&mut commands, &mut grid, &mut mode);
                        screens.stack.clear();
                        screens.open(ScreenName::Main);
                        screens.stack.notice = Some("Game saved.".into());
                    }
                    Err(e) => screens.stack.fail(e, false),
                }
            }
            // The run ended in death: its save is already gone, so nothing is saved.
            MenuChoice::LeaveRun => {
                leave_run(&mut commands, &mut grid, &mut mode);
                screens.stack.clear();
                screens.open(ScreenName::Main);
            }
            MenuChoice::Continue => match Game::resumed(&screens.store, &screens.content.0) {
                Resume::Loaded(world) => {
                    commands.insert_resource(Game::from_world(world));
                    screens.stack.clear();
                }
                Resume::Failed(message) => screens.stack.fail(message, true),
                Resume::NoSave => screens.stack.fail("There is no saved run.".into(), true),
            },
            MenuChoice::NewRun {
                scenario,
                seed,
                character,
            } => {
                let content = screens.content.0.clone();
                match Game::new_run(content, scenario, *seed, character.as_ref()) {
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
            let (draft, slot) = (stack.draft(), stack.selected_slot());
            let screen = Screen::new(*name, &content, has_save, run, &draft, slot);
            stack.screens.push(screen);
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
        stack.screens.push(Screen::new(
            Settings,
            &content(),
            false,
            None,
            &Draft::default(),
            None,
        ));
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
        assert_eq!(startup_screens(Some(Equipment), true), [Equipment]);
        assert_eq!(
            startup_screens(Some(EquipItem), true),
            [Equipment, EquipItem]
        );
        assert_eq!(startup_screens(Some(EquipItem), false), [Main]);
        assert_eq!(startup_screens(Some(Drop), false), [Main]);
        assert_eq!(startup_screens(Some(Race), false), [Main, NewRun, Race]);
        assert_eq!(
            startup_screens(Some(Class), false),
            [Main, NewRun, Race, Gender, Class]
        );
        assert_eq!(
            startup_screens(Some(Summary), false),
            [Main, NewRun, Race, Gender, Class, Summary]
        );
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
        let game =
            run.then(|| Game::new_run(content.clone(), "showcase", None, None).expect("run"));
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
            .init_resource::<crate::ui::LastTarget>()
            .init_resource::<ButtonInput<KeyCode>>()
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
    fn aiming_fires_at_the_cursor_cycles_targets_and_remembers_the_last() {
        use crate::ui::{Aim, AimKind, LastTarget};
        // In the arena, with the bow and arrows lying beside the player readied.
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "arena", None).expect("run");
        let ready = "move:E pickup equip:missile_weapon:a equip:missiles:b wait";
        for action in warlock_core::action::parse_actions(ready).expect("actions") {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world)));
        let aim = |app: &App| match *app.world().resource::<InputMode>() {
            InputMode::Aim(aim) => Some(aim),
            _ => None,
        };
        let (fire, tab) = ((KeyCode::KeyF, "f"), (KeyCode::Tab, ""));
        // `f` aims at the nearest hostile creature in view.
        frame(&mut app, &[fire]);
        let first = aim(&app).expect("aiming");
        assert_eq!(first.kind, AimKind::Fire);
        let world = app.world().resource::<Game>().world.clone();
        let view = world.perception();
        let nearest = Aim::start(AimKind::Fire, &view, &LastTarget::default());
        assert_eq!(first, nearest);
        // Tab moves on to another creature; a direction key moves the cursor a cell.
        frame(&mut app, &[tab]);
        let second = aim(&app).expect("still aiming");
        assert_ne!(second.at, first.at);
        assert!(view.actor_at(second.at).is_some());
        frame(&mut app, &[(KeyCode::KeyL, "l")]);
        let moved = aim(&app).expect("still aiming");
        assert_eq!(moved.at, second.at.step(Dir::E));
        // `?` does nothing while aiming; Esc cancels and shoots nothing.
        frame(&mut app, &[HELP, ESC]);
        assert!(names(&app).is_empty());
        assert_eq!(aim(&app), None);
        let before = actions(&app).len();
        // Aim again, Tab to the second creature, and Enter shoots it.
        frame(&mut app, &[fire, tab, ENTER]);
        assert_eq!(aim(&app), None);
        assert_eq!(actions(&app)[before..], [Action::Fire(second.at)]);
        // The next `f` starts on that creature again: `f`, Enter shoots it once more.
        frame(&mut app, &[fire]);
        assert_eq!(aim(&app).map(|a| a.at), Some(second.at));
        frame(&mut app, &[(KeyCode::KeyF, "f")]);
        assert_eq!(actions(&app).last(), Some(&Action::Fire(second.at)));
        // `t` with nothing to throw ready opens the choice of what to throw.
        frame(&mut app, &[(KeyCode::KeyT, "t")]);
        assert_eq!(names(&app), [ScreenName::Throw]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
    }

    #[test]
    fn main_menu_starts_a_run_from_presses_in_one_frame() {
        let mut app = app(false);
        app.update();
        assert_eq!(names(&app), [ScreenName::Main]);
        // Enter opens New run, and the next Enters, in the same frame, pick the showcase, its
        // character's race, gender, and class (preselected), and start it from the summary.
        frame(&mut app, &[ENTER, ENTER, ENTER, ENTER, ENTER]);
        assert_eq!(
            names(&app),
            [
                ScreenName::Main,
                ScreenName::NewRun,
                ScreenName::Race,
                ScreenName::Gender,
                ScreenName::Class,
                ScreenName::Summary
            ]
        );
        frame(&mut app, &[ENTER, WAIT]);
        assert!(app.world().contains_resource::<Game>());
        assert!(names(&app).is_empty());
        assert!(
            actions(&app).is_empty(),
            "presses after the choice are dropped"
        );
        let game = app.world().resource::<Game>();
        assert_eq!(
            game.world
                .character()
                .map(|c| c.spec.to_string())
                .as_deref(),
            Some("human,female,thief")
        );
        frame(&mut app, &[WAIT]);
        assert_eq!(actions(&app), [Action::Wait]);
    }

    #[test]
    fn character_creation_goes_back_and_remembers_choices() {
        let (up, down) = ((KeyCode::ArrowUp, ""), DOWN);
        let mut app = app(false);
        app.update();
        // New run, then race: down from human to dwarf; gender: down to other; class: up from
        // thief to ranger.
        frame(
            &mut app,
            &[ENTER, ENTER, down, ENTER, down, ENTER, up, ENTER],
        );
        assert_eq!(names(&app).last(), Some(&ScreenName::Summary));
        let draft = app.world().resource::<ScreenStack>().draft();
        let spec = draft.spec(&content()).expect("a character");
        assert_eq!(spec.to_string(), "dwarf,other,ranger");
        // Change race (the summary's second row): back on the race step as it was left, the
        // gender and class steps closed.
        frame(&mut app, &[down, ENTER]);
        assert_eq!(
            names(&app),
            [ScreenName::Main, ScreenName::NewRun, ScreenName::Race]
        );
        // Another race; the gender and class open on what was chosen before.
        frame(&mut app, &[down, ENTER, ENTER, ENTER]);
        assert_eq!(names(&app).last(), Some(&ScreenName::Summary));
        let spec = (app.world().resource::<ScreenStack>().draft())
            .spec(&content())
            .expect("a character");
        assert_eq!(spec.to_string(), "halfling,other,ranger");
        // Esc steps back one screen at a time, to the main menu.
        for left in [5, 4, 3, 2, 1] {
            frame(&mut app, &[ESC]);
            assert_eq!(names(&app).len(), left);
        }
        frame(&mut app, &[ENTER, ENTER, ENTER, ENTER, ENTER, ENTER]);
        let game = app.world().resource::<Game>();
        assert_eq!(
            game.world
                .character()
                .map(|c| c.spec.to_string())
                .as_deref(),
            Some("halfling,other,ranger"),
            "the choices are remembered until the run starts"
        );
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
    fn the_equipment_screen_chooses_what_goes_in_a_slot_and_stays_open() {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        // Into the storeroom, to pick up the chain mail shirt and the iron helm.
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 \
                    move:N pickup";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        assert_eq!(world.perception().inventory().len(), 2);
        let mut app = app_with(content, Some(Game::from_world(world)));
        let (e, c, a) = (
            (KeyCode::KeyE, "e"),
            (KeyCode::KeyC, "c"),
            (KeyCode::KeyA, "a"),
        );
        frame(&mut app, &[e]);
        assert_eq!(names(&app), [ScreenName::Equipment]);
        // `c` is the body's letter: its choice opens over the equipment screen.
        frame(&mut app, &[c]);
        assert_eq!(names(&app), [ScreenName::Equipment, ScreenName::EquipItem]);
        // Esc goes back to the equipment screen; `c` again, then `a` puts the mail on and
        // returns there, and the press after it in the same frame goes to it.
        frame(&mut app, &[ESC]);
        assert_eq!(names(&app), [ScreenName::Equipment]);
        frame(&mut app, &[c, a, (KeyCode::KeyA, "a")]);
        let wear = Action::Equip {
            slot: "body".into(),
            item: 0,
        };
        assert_eq!(actions(&app), [wear]);
        assert_eq!(names(&app), [ScreenName::Equipment, ScreenName::EquipItem]);
        frame(&mut app, &[ESC, ESC]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).len(), 1);
    }

    #[test]
    fn the_equipment_screen_shows_what_was_done_once_it_is_done() {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 \
                    move:N pickup";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world)));
        // The simulation applies what was chosen; the screens are refreshed after it.
        fn apply(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
            for PlayerAction(action) in actions.read() {
                game.apply(action);
            }
        }
        app.add_systems(Update, (apply, refresh).chain().after(count_actions));
        let worn = |app: &App| {
            let stack = app.world().resource::<ScreenStack>();
            let mut console = Console::default();
            console.resize(80, 30);
            stack.screens[0..1].iter().for_each(|s| {
                if let Screen::Equipment(screen) = s {
                    screen.draw(&mut console, &Bindings::default());
                }
            });
            (0..30)
                .map(|y| {
                    (0..80)
                        .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                        .collect()
                })
                .find(|l: &String| l.contains("c  Body"))
                .unwrap_or_default()
        };
        frame(&mut app, &[(KeyCode::KeyE, "e")]);
        assert!(worn(&app).contains('—'), "{}", worn(&app));
        frame(&mut app, &[(KeyCode::KeyC, "c"), (KeyCode::KeyA, "a")]);
        assert_eq!(names(&app), [ScreenName::Equipment]);
        assert!(worn(&app).contains("a chain mail shirt"), "{}", worn(&app));
        let game = app.world().resource::<Game>();
        assert!(
            game.world
                .messages()
                .iter()
                .any(|m| m.text == "You put on a chain mail shirt.")
        );
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
    fn death_deletes_the_save_and_the_death_screen_leads_to_the_main_menu() {
        let content = content();
        let dir = std::env::temp_dir().join(format!("warlock-death-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let args = Args::parse_from(["warlock", "--save-dir", dir.to_str().expect("utf-8")]);
        let store = SaveStore::from_args(&args);
        // The run was saved while it went on; then the archer shoots the lit player down.
        let mut world = warlock_core::World::new(content.clone(), "arena", None).expect("run");
        store.save_run(&world).expect("saved");
        assert!(store.has_save());
        world.apply(&Action::ToggleLantern);
        while world.death().is_none() {
            world.apply(&Action::Wait);
        }
        let mut app = App::new();
        app.add_message::<KeyboardInput>()
            .add_message::<PlayerAction>()
            .add_message::<SaveCommand>()
            .add_message::<MenuChoice>()
            .insert_resource(store.clone())
            .insert_resource(args)
            .insert_resource(GameContent(content))
            .insert_resource(Game::from_world(world))
            .init_resource::<ScreenStack>()
            .init_resource::<Settings>()
            .init_resource::<Bindings>()
            .init_resource::<InputMode>()
            .init_resource::<crate::ui::LastTarget>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<DevOverlays>()
            .init_resource::<MapGrid>()
            .init_resource::<Actions>()
            .add_systems(
                Update,
                (
                    route_keys,
                    apply_choice,
                    count_actions,
                    on_death.run_if(resource_exists::<Game>),
                )
                    .chain(),
            );
        app.update();
        assert_eq!(names(&app), [ScreenName::Death]);
        assert!(!store.has_save(), "permadeath: the save is gone");
        // It takes every key; gameplay sees none.
        frame(&mut app, &[WAIT, (KeyCode::KeyK, "k"), HELP]);
        assert_eq!(names(&app), [ScreenName::Death]);
        assert!(actions(&app).is_empty());
        // Enter leaves the run for the main menu, which offers nothing to continue.
        frame(&mut app, &[ENTER]);
        assert!(!app.world().contains_resource::<Game>());
        assert_eq!(names(&app), [ScreenName::Main]);
        assert!(!store.has_save());
        let _ = std::fs::remove_dir_all(&dir);
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
