//! The screen stack (spec §6): every modal screen — main menu, new run and character creation,
//! pause, settings, controls, the `?` help, the inventory, the equipment screen and its choice of
//! what goes in a slot, the character screen, the weapon skills screen, choosing what to pick up,
//! drop, or throw, the abilities menu, and the dialogue screen — lives on one stack, bottom to
//! top.
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
//!   the equipment screen beneath a choice stays open and [`refresh`] shows it the result. Talk
//!   that answers with topics opens the dialogue screen ([`converse`]), and so does each answer
//!   to a topic asked on it.
//! - **Draw**: `ui::draw` clears the UI console, draws the gameplay UI when a run exists, then
//!   the top screen ([`ScreenStack::draw`]).
//!
//! Screens are plain state machines: they take [`Press`]es and answer with a [`ScreenAction`],
//! so they are testable without a window.

use super::abilities_screen::AbilitiesScreen;
use super::character_screen::CharacterScreen;
use super::characters_screen::CharactersScreen;
use super::chargen_screen::{ChoiceScreen, Draft, Picks, Step, SummaryScreen};
use super::chargen_talents_screen::ChargenTalentsScreen;
use super::controls_screen::ControlsScreen;
use super::death_screen::DeathScreen;
use super::describe_screen::DescribeScreen;
use super::dialog_screen::{Dialog, DialogScreen};
use super::dialogue_screen::{DialogueScreen, last_talk};
use super::equipment_screen::{EquipmentScreen, ItemScreen};
use super::files_screen::FilesScreen;
use super::info_tabs;
use super::level_up_screen::LevelUpScreen;
use super::menu::{MainMenu, Press, quit_dialog};
use super::messages_screen::MessagesScreen;
use super::motion::Motion;
use super::name_screen::NameScreen;
use super::new_run_screen::{self, NewRunScreen};
use super::object_screen::{ObjectList, ObjectScreen};
use super::quest_screen::QuestScreen;
use super::settings_screen::SettingsScreen;
use super::sign_screen::SignScreen;
use super::skills_screen::SkillsScreen;
use super::talents_screen::TalentsScreen;
use super::target::AimKind;
use super::weapon_skills_screen::WeaponSkillsScreen;
use crate::cli::Args;
use crate::game::{Game, GameContent, PlayerAction};
use crate::input::{Bindings, InputMode, dev_keys};
use crate::render::console::Console;
use crate::render::grid::MapGrid;
use crate::save::{Resume, SaveCommand, SaveStore, SavedCharacter};
use crate::settings::keymap::KeymapFile;
use crate::settings::{Settings, SettingsStore};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use clap::ValueEnum;
use std::sync::Arc;
use warlock_core::Action;
use warlock_core::ability::Ability;
use warlock_core::character::CharacterSpec;
use warlock_core::content::{Content, SlotId};
use warlock_core::perception::PerceptionView;
use warlock_core::world::Conversation;

/// The screens that can be opened. Also the values of `--screen`, a dev flag for screenshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ScreenName {
    /// The main menu (no run).
    Main,
    /// The saved characters, from the main menu: resume one, or delete it.
    Characters,
    NewRun,
    /// Character creation, over New run: choose a race.
    Race,
    /// Born under the stars: the birth sign revealed (after the race).
    Sign,
    /// Choose a gender (after the race).
    Gender,
    /// Choose a class (after the gender).
    Class,
    /// Choose the talents picked at creation (after the class).
    ChargenTalents,
    /// Type a name (after the class).
    Name,
    /// The character before the run starts: start, or go back to a step.
    Summary,
    Settings,
    /// Where the settings, saves, and logs are, over Settings.
    Files,
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
    /// What one carried stack is, as the player knows it: the one looked at on the inventory or
    /// the equipment screen beneath (a letter there), else the first.
    Describe,
    /// The player's name, stats, and weight (`@` in play).
    Character,
    /// The player's trained skills and their values (`S` in play, or from the character
    /// screen).
    Skills,
    /// The player's weapon skills: ranks and marks (`W` in play).
    WeaponSkills,
    /// The player's quests, active and done (`q` in play).
    Quests,
    /// The player's talents, then every other with what it needs (page 7; its own command,
    /// unbound by default).
    Talents,
    /// The message history (`M` in play): every message the log keeps, the newest at the
    /// bottom. No info page.
    Messages,
    /// Choose what to pick up from the pile here (`,` or `g` where several objects lie).
    PickUp,
    /// Choose what to drop (`d` in play).
    Drop,
    /// Choose what to throw (`t` in play, with no throwing weapon readied).
    Throw,
    /// Choose what to eat (`E` in play): food lying here, then food in the pack.
    Eat,
    /// Choose a scroll to read (`r` in play): a spell scroll is aimed next, the scroll of
    /// identify asks what to read it on.
    Read,
    /// Choose what to read the scroll of identify on: the unidentified stacks carried. It takes
    /// the place of the read list where that scroll was chosen (as `--screen`, the first stack's
    /// scroll).
    Identify,
    /// The abilities menu (`a` in play): racial, class, skill, weapon skill, and item
    /// abilities, to choose one to use.
    Abilities,
    /// Talking with someone who has topics (`C` next to them): opens by itself when they answer;
    /// as `--screen`, over a run whose last action was such a chat (a replay's last step).
    Dialogue,
    /// The run is over: who died, where, when, and of what. It opens by itself when the player
    /// dies, so it is no `--screen`.
    #[value(skip)]
    Death,
    /// A level's choices: skill rolls, then talents. It opens by itself while any are due and
    /// cannot be left until they are made, so it is no `--screen` (a replay that ends with a
    /// choice due opens it).
    #[value(skip)]
    LevelUp,
    /// The dialog (dev: the quit question). Screens and gameplay ask with
    /// [`ScreenAction::Dialog`].
    Dialog,
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
                | ScreenName::Describe
                | ScreenName::Character
                | ScreenName::Skills
                | ScreenName::WeaponSkills
                | ScreenName::Quests
                | ScreenName::Talents
                | ScreenName::Messages
                | ScreenName::PickUp
                | ScreenName::Drop
                | ScreenName::Throw
                | ScreenName::Eat
                | ScreenName::Read
                | ScreenName::Identify
                | ScreenName::Abilities
                | ScreenName::Dialogue
                | ScreenName::Death
                | ScreenName::LevelUp
        )
    }
}

/// A menu choice that starts, leaves, or ends a run; applied by [`apply_choice`].
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum MenuChoice {
    /// Resume the character played last.
    Continue,
    /// Resume the saved character with this save id (the Characters screen).
    Resume(String),
    /// Delete the saved character with this save id (the Characters screen, after the dialog).
    DeleteCharacter(String),
    NewRun {
        scenario: String,
        seed: Option<u64>,
        /// The character chosen; `None` for the scenario's (content without races or classes).
        character: Option<CharacterSpec>,
        /// The name chosen; `None` for the run's default.
        name: Option<String>,
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
    /// Close this screen and open another in its place (an info screen's page number).
    Replace(ScreenName),
    /// Close screens until this one is on top (as it was left).
    Return(ScreenName),
    Choose(MenuChoice),
    /// Ask first: open the dialog with this content, which hands on what the chosen option does
    /// (on top of the screen that asked; a dialog opened by a dialog takes its place).
    Dialog(Box<Dialog>),
    /// Close this screen and apply a gameplay action (what to pick up, drop, wear, or wield).
    Act(Action),
    /// Close this screen and aim: a throw of the stack chosen, a scroll read at a cell.
    Aim(AimKind),
    /// Close this screen, if one is open, and use an ability (the abilities menu's choice, its
    /// own key, or the quick repeat): it asks for what it needs, then is applied as its action.
    /// The stack remembers it as the ability used last.
    UseAbility(Ability),
}

impl From<MenuChoice> for ScreenAction {
    fn from(choice: MenuChoice) -> Self {
        ScreenAction::Choose(choice)
    }
}

impl From<Action> for ScreenAction {
    fn from(action: Action) -> Self {
        ScreenAction::Act(action)
    }
}

/// What screens may need to know of this launch.
#[derive(Clone, Debug, Default)]
struct Launch {
    /// The saved characters, the one played last first (the main menu offers Continue and
    /// Characters with any); only read for the screens that show them.
    characters: Vec<SavedCharacter>,
    /// A developer build or `--dev`: the developer keys work (the controls and the help list
    /// them), and New run has a seed field.
    dev: bool,
    /// Where the controls' Export keys and Import keys go.
    keymap: KeymapFile,
}

/// A modal screen.
#[derive(Debug)]
enum Screen {
    /// The main menu, or the pause menu over a run.
    Menu(MainMenu),
    /// The saved characters.
    Characters(CharactersScreen),
    NewRun(NewRunScreen),
    /// A step of character creation.
    Choice(ChoiceScreen),
    /// Character creation's talents (boxed: it holds the drafted run).
    ChargenTalents(Box<ChargenTalentsScreen>),
    /// Character creation's reveal of the birth sign.
    Sign(Box<SignScreen>),
    /// Character creation's name.
    Name(NameScreen),
    /// Boxed: it holds the whole character.
    Summary(Box<SummaryScreen>),
    Settings(SettingsScreen),
    Files(FilesScreen),
    /// The controls screen, or the read-only `?` help.
    Controls(ControlsScreen),
    /// The inventory, or choosing what to pick up or drop.
    Objects(ObjectScreen),
    Equipment(EquipmentScreen),
    /// Choosing what goes in a slot.
    EquipItem(ItemScreen),
    /// What one carried stack is.
    Describe(DescribeScreen),
    Character(CharacterScreen),
    Skills(SkillsScreen),
    WeaponSkills(WeaponSkillsScreen),
    Quests(QuestScreen),
    Talents(TalentsScreen),
    Messages(MessagesScreen),
    Abilities(AbilitiesScreen),
    Dialogue(DialogueScreen),
    Death(DeathScreen),
    LevelUp(LevelUpScreen),
    /// The dialog.
    Dialog(DialogScreen),
}

/// What a screen opening may read off the stack: the slot selected on the equipment screen
/// beneath, if any (the first slot if none), the carried stack looked at on a screen beneath
/// (the first if none), and the ability used last.
#[derive(Clone, Copy, Debug, Default)]
struct Beneath {
    slot: Option<SlotId>,
    item: Option<usize>,
    ability: Option<Ability>,
}

impl Screen {
    /// Screen `name`; `run` is what the player perceives in the run, if there is one, `draft`
    /// what New run has chosen on the screens below, `beneath` what the stack holds for it (the
    /// slot and the stack selected beneath, the ability used last), `talk` the run's last
    /// conversation, if its last action was talk, and `launch` what this launch has (the saved
    /// characters, the developer keys, where keymaps go).
    fn new(
        name: ScreenName,
        content: &Arc<Content>,
        launch: &Launch,
        run: Option<&PerceptionView<'_>>,
        draft: &Draft,
        beneath: Beneath,
        talk: Option<&Conversation>,
    ) -> Self {
        let Beneath { slot, item, .. } = beneath;
        let objects = |list| Screen::Objects(ObjectScreen::new(list, run));
        let choice = |step| Screen::Choice(ChoiceScreen::new(step, content, draft));
        match name {
            ScreenName::Main => {
                let last = launch.characters.first().map(|c| c.name().to_string());
                Screen::Menu(MainMenu::main(last))
            }
            ScreenName::Characters => {
                let campaigns = content.campaigns().len() > 1;
                Screen::Characters(CharactersScreen::new(launch.characters.clone(), campaigns))
            }
            ScreenName::Pause => Screen::Menu(MainMenu::pause()),
            // A release build's new run gets a seed no one chooses; a developer may type one.
            ScreenName::NewRun => {
                let fresh = (!launch.dev).then(new_run_screen::fresh_seed);
                Screen::NewRun(NewRunScreen::new(content, fresh))
            }
            ScreenName::Race => choice(Step::Race),
            ScreenName::Sign => Screen::Sign(Box::new(SignScreen::new(content, draft))),
            ScreenName::Gender => choice(Step::Gender),
            ScreenName::Class => choice(Step::Class),
            ScreenName::ChargenTalents => Screen::ChargenTalents(Box::new(
                ChargenTalentsScreen::new(content, draft, draft.remembered.talents.as_deref()),
            )),
            ScreenName::Name => Screen::Name(NameScreen::new(content, draft)),
            ScreenName::Summary => Screen::Summary(Box::new(SummaryScreen::new(content, draft))),
            ScreenName::Settings => Screen::Settings(SettingsScreen::default()),
            ScreenName::Files => Screen::Files(FilesScreen::new(crate::paths::locations())),
            ScreenName::Controls => {
                Screen::Controls(ControlsScreen::new(launch.dev, launch.keymap.clone()))
            }
            ScreenName::Help => Screen::Controls(ControlsScreen::help(launch.dev)),
            ScreenName::Inventory => objects(ObjectList::Inventory),
            ScreenName::Equipment => Screen::Equipment(EquipmentScreen::new(run)),
            ScreenName::EquipItem => {
                Screen::EquipItem(ItemScreen::new(run, slot.unwrap_or(SlotId(0))))
            }
            ScreenName::Describe => {
                Screen::Describe(DescribeScreen::new(run, Some(item.unwrap_or(0))))
            }
            ScreenName::Character => Screen::Character(CharacterScreen::new(run)),
            ScreenName::Skills => Screen::Skills(SkillsScreen::new(run)),
            ScreenName::WeaponSkills => Screen::WeaponSkills(WeaponSkillsScreen::new(run)),
            ScreenName::Quests => Screen::Quests(QuestScreen::new(run)),
            ScreenName::Talents => Screen::Talents(TalentsScreen::new(run)),
            ScreenName::Messages => Screen::Messages(MessagesScreen::new(run)),
            ScreenName::PickUp => objects(ObjectList::PickUp),
            ScreenName::Drop => objects(ObjectList::Drop),
            ScreenName::Throw => objects(ObjectList::Throw),
            ScreenName::Eat => objects(ObjectList::Eat),
            ScreenName::Read => objects(ObjectList::Read),
            ScreenName::Identify => objects(ObjectList::Identify(beneath.item.unwrap_or(0))),
            ScreenName::Abilities => Screen::Abilities(AbilitiesScreen::new(run, beneath.ability)),
            ScreenName::Dialogue => Screen::Dialogue(DialogueScreen::new(talk, run)),
            ScreenName::Death => Screen::Death(DeathScreen::new(run)),
            ScreenName::LevelUp => Screen::LevelUp(LevelUpScreen::new(run)),
            ScreenName::Dialog => Screen::Dialog(DialogScreen::new(quit_dialog(run.is_some()))),
        }
    }

    fn name(&self) -> ScreenName {
        match self {
            Screen::Menu(menu) if menu.is_pause() => ScreenName::Pause,
            Screen::Menu(_) => ScreenName::Main,
            Screen::Characters(_) => ScreenName::Characters,
            Screen::NewRun(_) => ScreenName::NewRun,
            Screen::Choice(screen) => screen.step().screen(),
            Screen::ChargenTalents(_) => ScreenName::ChargenTalents,
            Screen::Sign(_) => ScreenName::Sign,
            Screen::Name(_) => ScreenName::Name,
            Screen::Summary(_) => ScreenName::Summary,
            Screen::Settings(_) => ScreenName::Settings,
            Screen::Files(_) => ScreenName::Files,
            Screen::Controls(screen) if screen.is_help() => ScreenName::Help,
            Screen::Controls(_) => ScreenName::Controls,
            Screen::Objects(screen) => match screen.list() {
                ObjectList::Inventory => ScreenName::Inventory,
                ObjectList::PickUp => ScreenName::PickUp,
                ObjectList::Drop => ScreenName::Drop,
                ObjectList::Throw => ScreenName::Throw,
                ObjectList::Eat => ScreenName::Eat,
                ObjectList::Read => ScreenName::Read,
                ObjectList::Identify(_) => ScreenName::Identify,
            },
            Screen::Equipment(_) => ScreenName::Equipment,
            Screen::EquipItem(_) => ScreenName::EquipItem,
            Screen::Describe(_) => ScreenName::Describe,
            Screen::Character(_) => ScreenName::Character,
            Screen::Skills(_) => ScreenName::Skills,
            Screen::WeaponSkills(_) => ScreenName::WeaponSkills,
            Screen::Quests(_) => ScreenName::Quests,
            Screen::Talents(_) => ScreenName::Talents,
            Screen::Messages(_) => ScreenName::Messages,
            Screen::Abilities(_) => ScreenName::Abilities,
            Screen::Dialogue(_) => ScreenName::Dialogue,
            Screen::Death(_) => ScreenName::Death,
            Screen::LevelUp(_) => ScreenName::LevelUp,
            Screen::Dialog(_) => ScreenName::Dialog,
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
    /// The UI console's size (columns, rows), if known: item lists page by it, so a letter
    /// names the stack drawn there even before the screen is first drawn.
    console: Option<(u32, u32)>,
    /// The ability used last in this run, which the abilities menu opens on and the quick
    /// repeat uses again.
    last_ability: Option<Ability>,
}

impl ScreenStack {
    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    /// The UI console's size the screens lay out for (the input router keeps it current).
    pub fn console(&self) -> Option<(u32, u32)> {
        self.console
    }

    pub fn set_console(&mut self, size: (u32, u32)) {
        self.console = Some(size);
    }

    /// The open screens, bottom to top.
    #[cfg(test)]
    fn names(&self) -> Vec<ScreenName> {
        self.screens.iter().map(Screen::name).collect()
    }

    /// Closes the top screen, and New run beneath it if it was skipped
    /// ([`NewRunScreen::skips`]): Esc on the race step goes back to the menu.
    pub fn close(&mut self) {
        self.screens.pop();
        if matches!(self.screens.last(), Some(Screen::NewRun(screen)) if screen.skips()) {
            self.screens.pop();
        }
    }

    /// The top screen's name, if any screen is open.
    fn top(&self) -> Option<ScreenName> {
        self.screens.last().map(Screen::name)
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
        self.last_ability = None;
    }

    /// The ability used last in this run, if any.
    pub fn last_ability(&self) -> Option<Ability> {
        self.last_ability
    }

    /// Remembers `ability` as the one used last.
    pub fn remember_ability(&mut self, ability: Ability) {
        self.last_ability = Some(ability);
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
                Screen::Name(screen) => draft.chosen.set(Step::Name, screen.text()),
                Screen::ChargenTalents(screen) => draft.talents = Some(screen.picks().to_vec()),
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

    /// The carried stack looked at on the topmost inventory or equipment screen, if any: what a
    /// description opened from it describes; or the scroll of identify chosen on the read list:
    /// what the identify list reads.
    fn looked_item(&self) -> Option<usize> {
        self.screens.iter().rev().find_map(|s| match s {
            Screen::Objects(screen)
                if matches!(screen.list(), ObjectList::Inventory | ObjectList::Read) =>
            {
                screen.looked()
            }
            Screen::Equipment(screen) => screen.selected_item(),
            _ => None,
        })
    }

    /// Whether an equipment screen is open (it shows the run as it changes, [`refresh`]).
    /// Shows what an import from the browser's file picker did on the controls screen, if it
    /// is open.
    pub(super) fn keymap_imported(&mut self, lines: Vec<String>) {
        for screen in &mut self.screens {
            if let Screen::Controls(screen) = screen {
                screen.imported(lines.clone());
            }
        }
    }

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
        if !press.repeat
            && let Some(screen) = self.screens.last()
            && info_tabs::is_page(screen.name())
            && let Some(action) = info_tabs::switch(screen.name(), press, bindings)
        {
            self.notice = None;
            return action;
        }
        let action = match self.screens.last_mut() {
            None => ScreenAction::Stay,
            Some(Screen::Menu(menu)) => menu.handle(press),
            Some(Screen::Characters(screen)) => screen.handle(press),
            Some(Screen::NewRun(screen)) => screen.handle(press),
            Some(Screen::Choice(screen)) => {
                let action = screen.handle(press);
                if let (ScreenAction::Open(_), Some(id)) = (&action, screen.chosen()) {
                    self.picks.set(screen.step(), id);
                }
                action
            }
            Some(Screen::ChargenTalents(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                let action = screen.handle(press);
                if matches!(action, ScreenAction::Open(_) | ScreenAction::Close) {
                    self.picks.talents = Some(screen.picks().to_vec());
                }
                action
            }
            Some(Screen::Sign(screen)) => screen.handle(press),
            Some(Screen::Name(screen)) => {
                let action = screen.handle(press);
                if let ScreenAction::Open(_) = action {
                    self.picks.set(Step::Name, screen.text());
                }
                action
            }
            Some(Screen::Summary(screen)) => screen.handle(press),
            Some(Screen::Settings(screen)) => screen.handle(press, settings),
            Some(Screen::Files(screen)) => screen.handle(press),
            Some(Screen::Controls(screen)) => leave(screen.handle(press, bindings)),
            Some(Screen::Objects(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Equipment(screen)) => screen.handle(press, bindings),
            Some(Screen::EquipItem(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Describe(screen)) => screen.handle(press, bindings),
            Some(Screen::Character(screen)) => screen.handle(press, bindings),
            Some(Screen::Skills(screen)) => screen.handle(press, bindings),
            Some(Screen::WeaponSkills(screen)) => screen.handle(press, bindings),
            Some(Screen::Quests(screen)) => screen.handle(press, bindings),
            Some(Screen::Talents(screen)) => screen.handle(press, bindings),
            Some(Screen::Messages(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Abilities(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Dialogue(screen)) => screen.handle(press, bindings),
            Some(Screen::Death(screen)) => screen.handle(press),
            Some(Screen::LevelUp(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press)
            }
            Some(Screen::Dialog(screen)) => screen.handle(press, bindings),
        };
        if action == ScreenAction::Stay {
            return action;
        }
        if press.repeat {
            return ScreenAction::Stay;
        }
        self.notice = None;
        // The dialog is done once it has made a choice (a failed one then shows on the menu) or
        // asked the next question, which takes its place. A gameplay action closes it as usual.
        if matches!(action, ScreenAction::Choose(_) | ScreenAction::Dialog(_))
            && matches!(self.screens.last(), Some(Screen::Dialog(_)))
        {
            self.screens.pop();
        }
        action
    }

    /// Shows why a menu choice failed, on the menu it was made from (closing New run).
    fn fail(&mut self, message: String) {
        while self.screens.len() > 1 && !matches!(self.screens.last(), Some(Screen::Menu(_))) {
            self.screens.pop();
        }
        self.notice = Some(message);
    }

    /// Draws the top screen. Each screen is a panel at least as large as the menu it was opened
    /// from, so the screens below would not show anyway.
    pub fn draw(&self, console: &mut Console, settings: &Settings, bindings: &Bindings) {
        match self.screens.last() {
            None => {}
            Some(Screen::Menu(menu)) => menu.draw(console, self.notice.as_deref(), bindings),
            Some(Screen::Characters(screen)) => screen.draw(console, settings.clock, bindings),
            Some(Screen::NewRun(screen)) => screen.draw(console, bindings),
            Some(Screen::Choice(screen)) => screen.draw(console, bindings),
            Some(Screen::ChargenTalents(screen)) => screen.draw(console, bindings),
            Some(Screen::Sign(screen)) => screen.draw(console, bindings),
            Some(Screen::Name(screen)) => screen.draw(console, bindings),
            Some(Screen::Summary(screen)) => screen.draw(console, bindings),
            Some(Screen::Settings(screen)) => screen.draw(console, settings, bindings),
            Some(Screen::Files(screen)) => screen.draw(console, bindings),
            Some(Screen::Controls(screen)) => screen.draw(console, bindings),
            Some(Screen::Objects(screen)) => screen.draw(console, bindings),
            Some(Screen::Equipment(screen)) => screen.draw(console, bindings),
            Some(Screen::EquipItem(screen)) => screen.draw(console, bindings),
            Some(Screen::Describe(screen)) => screen.draw(console, bindings),
            Some(Screen::Character(screen)) => screen.draw(console, bindings),
            Some(Screen::Skills(screen)) => screen.draw(console, bindings),
            Some(Screen::WeaponSkills(screen)) => screen.draw(console, bindings),
            Some(Screen::Quests(screen)) => screen.draw(console, bindings),
            Some(Screen::Talents(screen)) => screen.draw(console, bindings),
            Some(Screen::Messages(screen)) => screen.draw(console, bindings),
            Some(Screen::Abilities(screen)) => screen.draw(console, bindings),
            Some(Screen::Dialogue(screen)) => screen.draw(console, bindings),
            Some(Screen::Death(screen)) => screen.draw(console, settings.clock, bindings),
            Some(Screen::LevelUp(screen)) => screen.draw(console, bindings),
            Some(Screen::Dialog(screen)) => screen.draw(console, bindings),
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
    settings: Res<'w, SettingsStore>,
    args: Res<'w, Args>,
    /// The run, for screens that show what the player perceives (the inventory, a pile, the
    /// character screen).
    game: Option<Res<'w, Game>>,
}

impl Screens<'_> {
    /// Opens the dialog `dialog`.
    pub fn ask(&mut self, dialog: Dialog) {
        self.stack
            .screens
            .push(Screen::Dialog(DialogScreen::new(dialog)));
    }

    /// Opens screen `name` on top of the stack. The one way screens open.
    pub fn open(&mut self, name: ScreenName) {
        let item = self.stack.looked_item();
        self.open_on(name, item);
    }

    /// Opens screen `name` for carried stack `item` (what was looked at or chosen beneath).
    fn open_on(&mut self, name: ScreenName, item: Option<usize>) {
        let view = self.game.as_ref().map(|game| game.world.perception());
        let talk = (self.game.as_ref()).and_then(|game| last_talk(&game.last_outcome));
        let draft = self.stack.draft();
        let beneath = Beneath {
            slot: self.stack.selected_slot(),
            item,
            ability: self.stack.last_ability,
        };
        // Only the screens that show the saved characters read them (a listing of the saves).
        let characters = match name {
            ScreenName::Main | ScreenName::Characters => self.store.characters(),
            _ => Vec::new(),
        };
        let launch = Launch {
            characters,
            dev: dev_keys(&self.args),
            keymap: self.settings.keymap(),
        };
        let content = &self.content.0;
        let screen = Screen::new(name, content, &launch, view.as_ref(), &draft, beneath, talk);
        // New run with one scenario to choose (a release build) is skipped: the race step opens
        // over it at once.
        let skipped = matches!(&screen, Screen::NewRun(screen) if screen.skips());
        self.stack.screens.push(screen);
        if skipped {
            self.open_on(ScreenName::Race, item);
        }
    }

    /// Replaces the top screen with screen `name`: the same stack depth, so closing it returns
    /// to where the one it replaced was opened from. It reads what was looked at or chosen on
    /// the screen it replaces (the scroll of identify chosen on the read list).
    pub fn replace(&mut self, name: ScreenName) {
        let item = self.stack.looked_item();
        self.stack.close();
        self.open_on(name, item);
    }

    /// Opens the main menu afresh, alone, with `notice` (the saved characters may have changed:
    /// one deleted, or kept as unusable), and on it the Characters screen when `characters` and
    /// any are left.
    fn back_to_main(&mut self, notice: Option<String>, characters: bool) {
        self.stack.clear();
        self.open(ScreenName::Main);
        self.stack.notice = notice;
        if characters && !self.store.characters().is_empty() {
            self.open(ScreenName::Characters);
        }
    }
}

/// Shows the screens that stay open while the run changes what they show (the equipment screen,
/// beneath a choice that was just done) the run as it is now.
pub(super) fn refresh(game: Res<Game>, mut stack: ResMut<ScreenStack>) {
    if game.is_changed() && stack.shows_equipment() {
        stack.refresh(&game.world.perception());
    }
}

/// Opens the dialogue screen when the player's action this frame was talk that answered with
/// topics: a chat with someone who has them, or a topic asked on the dialogue screen (which
/// closed to ask). Only then, so a run changed for other reasons never reopens it.
pub(super) fn converse(
    game: Res<Game>,
    mut actions: MessageReader<PlayerAction>,
    mut screens: Screens,
) {
    if actions.read().count() == 0 || !screens.stack.is_empty() {
        return;
    }
    // An answer to a topic shows there even when nothing is left to ask (a quest handed in to
    // someone with no other topics).
    let shown = |talk: &Conversation| !talk.topics.is_empty() || talk.asked.is_some();
    if last_talk(&game.last_outcome).is_some_and(shown) {
        screens.open(ScreenName::Dialogue);
    }
}

/// Opens the screens the command line asks for (see [`startup_screens`]).
pub(super) fn open_startup_screens(args: Res<Args>, game: Option<Res<Game>>, mut screens: Screens) {
    for name in startup_screens(args.screen, game.is_some()) {
        // A skipped New run has opened the race step already.
        if screens.stack.top() != Some(name) {
            screens.open(name);
        }
    }
}

/// The screens open at startup, bottom to top: the main menu when there is no run, and
/// `--screen` over the main menu or the run. In a run, settings and controls are reached from
/// the pause menu, so they open over it; character creation's steps open over New run and the
/// steps before them, each with its preselection.
fn startup_screens(screen: Option<ScreenName>, in_run: bool) -> Vec<ScreenName> {
    use ScreenName::*;
    let creation = [Race, Sign, Gender, Class, ChargenTalents, Name, Summary];
    match (in_run, screen) {
        (false, Some(s)) if creation.contains(&s) => {
            let steps = creation.iter().take_while(|&&c| c != s);
            [Main, NewRun]
                .into_iter()
                .chain(steps.copied())
                .chain([s])
                .collect()
        }
        (false, Some(s @ (Characters | NewRun | Settings | Controls | Dialog))) => vec![Main, s],
        (false, Some(Files)) => vec![Main, Settings, Files],
        (false, _) => vec![Main],
        (true, Some(s @ (Settings | Controls | Dialog))) => vec![Pause, s],
        (true, Some(Files)) => vec![Pause, Settings, Files],
        (true, Some(EquipItem)) => vec![Equipment, EquipItem],
        (true, Some(Describe)) => vec![Inventory, Describe],
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

/// When the run has ended in death: that character's save is deleted (permadeath), and no other
/// character's, and the death screen opens over the run as it ended, in place of whatever screen
/// was open. Once.
pub(super) fn on_death(game: Res<Game>, mut screens: Screens) {
    if !game.is_changed() || game.world.death().is_none() || screens.stack.shows_death() {
        return;
    }
    if let Err(e) = screens.store.delete_character(&game.save_id) {
        warn!("{e}");
    }
    screens.stack.clear();
    screens.open(ScreenName::Death);
}

/// When a level's choices are due (a level just came, or a run was resumed with them due): the
/// level-up screen opens on top, afresh on what is left (its choices close it to be applied).
/// Nothing is banked, so it opens whenever choices are due; the death screen wins.
pub(super) fn on_level_up(game: Res<Game>, mut screens: Screens) {
    if !game.is_changed() || game.world.death().is_some() || screens.stack.shows_death() {
        return;
    }
    if game.world.due().is_none() {
        return;
    }
    if matches!(screens.stack.screens.last(), Some(Screen::LevelUp(_))) {
        screens.stack.close();
    }
    screens.open(ScreenName::LevelUp);
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
                match screens.store.save_run(&game.save_id, &game.world) {
                    Ok(()) => {
                        leave_run(&mut commands, &mut grid, &mut mode);
                        screens.back_to_main(Some("Game saved.".into()), false);
                    }
                    Err(e) => screens.stack.fail(e),
                }
            }
            // The run ended in death: its save is already gone, so nothing is saved.
            MenuChoice::LeaveRun => {
                leave_run(&mut commands, &mut grid, &mut mode);
                screens.back_to_main(None, false);
            }
            MenuChoice::Continue => match screens.store.last_played() {
                Some(last) => resume(&mut commands, &mut screens, &last.id),
                None => screens.back_to_main(Some("There is no saved character.".into()), false),
            },
            MenuChoice::Resume(id) => resume(&mut commands, &mut screens, id),
            MenuChoice::DeleteCharacter(id) => {
                let notice = screens.store.delete_character(id).err();
                screens.back_to_main(notice, true);
            }
            MenuChoice::NewRun {
                scenario,
                seed,
                character,
                name,
            } => {
                let content = screens.content.0.clone();
                let (character, name) = (character.as_ref(), name.as_deref());
                match Game::new_run(content, &screens.store, scenario, *seed, character, name) {
                    Ok(game) => {
                        commands.insert_resource(game);
                        screens.stack.clear();
                    }
                    Err(message) => screens.stack.fail(message),
                }
            }
        }
    }
}

/// Resumes the saved character `id`. A save that cannot be resumed is kept as unusable and the
/// main menu opens afresh (the list of saved characters without it), saying why.
fn resume(commands: &mut Commands, screens: &mut Screens, id: &str) {
    match Game::resumed(&screens.store, id, &screens.content.0) {
        Resume::Loaded(world) => {
            commands.insert_resource(Game::from_world(*world, id.to_string()));
            screens.stack.clear();
        }
        Resume::Failed(message) => screens.back_to_main(Some(message), false),
        Resume::NoSave => screens.back_to_main(Some("That save is gone.".into()), false),
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
    use warlock_core::{Action, Dir, Pos};

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
            Action::PickUp(warlock_core::Positions::of([0])),
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
            let draft = stack.draft();
            let beneath = Beneath {
                slot: stack.selected_slot(),
                item: stack.looked_item(),
                ability: stack.last_ability,
            };
            let saved = SavedCharacter {
                id: "tamsin-1".into(),
                summary: None,
            };
            let launch = Launch {
                characters: has_save.then_some(saved).into_iter().collect(),
                dev: true,
                keymap: KeymapFile::Nowhere,
            };
            let screen = Screen::new(*name, &content, &launch, run, &draft, beneath, None);
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
            &Launch::default(),
            None,
            &Draft::default(),
            Beneath::default(),
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
        stack.fail("Could not start.".into());
        assert_eq!(stack.names(), [Main]);
        assert_eq!(stack.notice.as_deref(), Some("Could not start."));
        // Picking something clears the notice.
        assert_eq!(
            press(&mut stack, Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::Continue)
        );
        assert_eq!(stack.notice, None);
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
        assert_eq!(startup_screens(Some(WeaponSkills), true), [WeaponSkills]);
        assert_eq!(startup_screens(Some(WeaponSkills), false), [Main]);
        assert_eq!(startup_screens(Some(PickUp), true), [PickUp]);
        assert_eq!(startup_screens(Some(Equipment), true), [Equipment]);
        assert_eq!(
            startup_screens(Some(EquipItem), true),
            [Equipment, EquipItem]
        );
        assert_eq!(startup_screens(Some(EquipItem), false), [Main]);
        assert_eq!(startup_screens(Some(Describe), true), [Inventory, Describe]);
        assert_eq!(startup_screens(Some(Drop), false), [Main]);
        assert_eq!(startup_screens(Some(Dialogue), true), [Dialogue]);
        assert_eq!(startup_screens(Some(Dialogue), false), [Main]);
        assert_eq!(startup_screens(Some(Race), false), [Main, NewRun, Race]);
        assert_eq!(
            startup_screens(Some(Sign), false),
            [Main, NewRun, Race, Sign]
        );
        assert_eq!(
            startup_screens(Some(Class), false),
            [Main, NewRun, Race, Sign, Gender, Class]
        );
        assert_eq!(
            startup_screens(Some(Name), false),
            [
                Main,
                NewRun,
                Race,
                Sign,
                Gender,
                Class,
                ChargenTalents,
                Name
            ]
        );
        assert_eq!(
            startup_screens(Some(Summary), false),
            [
                Main,
                NewRun,
                Race,
                Sign,
                Gender,
                Class,
                ChargenTalents,
                Name,
                Summary
            ]
        );
        assert_eq!(startup_screens(Some(Characters), false), [Main, Characters]);
        assert_eq!(startup_screens(Some(Characters), true), []);
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
    // Directions on the numpad, ADOM's movement keys (#350).
    const NORTH: (KeyCode, &str) = (KeyCode::Numpad8, "8");
    const EAST: (KeyCode, &str) = (KeyCode::Numpad6, "6");
    const WEST: (KeyCode, &str) = (KeyCode::Numpad4, "4");

    /// An app with the router and the screen systems, on the main menu or in a run.
    fn app(run: bool) -> App {
        let content = content();
        let game = run.then(|| {
            let world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
            Game::from_world(world, "tamsin-1".into())
        });
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
            .init_resource::<crate::settings::SettingsStore>()
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
        frame(&mut app, &[(KeyCode::KeyO, "o"), NORTH]);
        assert_eq!(actions(&app), [Action::Open(Dir::N)]);
        // Look mode: Esc leaves it, `?` does nothing there.
        frame(&mut app, &[(KeyCode::KeyL, "l"), HELP]);
        assert!(matches!(
            app.world().resource::<InputMode>(),
            InputMode::Look(_)
        ));
        frame(&mut app, &[ESC]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(names(&app).is_empty());
        // The rest prompt: Esc cancels it; a letter answers it.
        let rest = (KeyCode::KeyB, "b");
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
    fn using_a_tool_asks_which_way_to_dig_only_with_one_ready() {
        const USE: (KeyCode, &str) = (KeyCode::KeyU, "u");
        // Nothing to dig with: the rules say so at once, and no prompt opens.
        let mut app = app(true);
        frame(&mut app, &[USE]);
        assert_eq!(actions(&app), [Action::Dig(Dir::N)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        // In the quarry with the pick-axe ready: `u` asks for a direction, Esc cancels it, and a
        // direction digs.
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "quarry", None).expect("run");
        for action in
            warlock_core::action::parse_actions("move:E pickup equip:tool:a").expect("actions")
        {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        frame(&mut app, &[USE]);
        assert_eq!(
            app.world().resource::<InputMode>(),
            &InputMode::Direction(crate::input::KeyCommand::UseTool)
        );
        frame(&mut app, &[ESC]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(names(&app).is_empty() && actions(&app).is_empty());
        frame(&mut app, &[USE, EAST]);
        assert_eq!(actions(&app), [Action::Dig(Dir::E)]);
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
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        let aim = |app: &App| match *app.world().resource::<InputMode>() {
            InputMode::Aim(aim) => Some(aim),
            _ => None,
        };
        let (fire, tab) = ((KeyCode::KeyT, "t"), (KeyCode::Tab, ""));
        // `t` aims at the nearest hostile creature in view.
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
        frame(&mut app, &[EAST]);
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
        // The next `t` starts on that creature again: `t`, Enter shoots it once more.
        frame(&mut app, &[fire]);
        assert_eq!(aim(&app).map(|a| a.at), Some(second.at));
        frame(&mut app, &[ENTER]);
        assert_eq!(actions(&app).last(), Some(&Action::Fire(second.at)));
        // `T` opens the choice of what to throw.
        frame(&mut app, &[(KeyCode::KeyT, "T")]);
        assert_eq!(names(&app), [ScreenName::Throw]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
    }

    #[test]
    fn main_menu_starts_a_run_from_presses_in_one_frame() {
        let mut app = app(false);
        app.update();
        assert_eq!(names(&app), [ScreenName::Main]);
        // Enter opens New run, and the next Enters, in the same frame, pick the Ascmearc, its
        // character's race, read the stars, the gender and class (preselected), the talents
        // (Random, preselected), and a name (an
        // empty field: the first random one), and start it from the summary.
        frame(
            &mut app,
            &[ENTER, ENTER, ENTER, ENTER, ENTER, ENTER, ENTER, ENTER],
        );
        assert_eq!(
            names(&app),
            [
                ScreenName::Main,
                ScreenName::NewRun,
                ScreenName::Race,
                ScreenName::Sign,
                ScreenName::Gender,
                ScreenName::Class,
                ScreenName::ChargenTalents,
                ScreenName::Name,
                ScreenName::Summary
            ]
        );
        let name = (app.world().resource::<ScreenStack>().draft()).name(&content());
        frame(&mut app, &[ENTER, WAIT]);
        assert!(app.world().contains_resource::<Game>());
        assert!(names(&app).is_empty());
        assert!(
            actions(&app).is_empty(),
            "presses after the choice are dropped"
        );
        let game = app.world().resource::<Game>();
        let character = game.world.character().expect("a character");
        assert_eq!(character.spec.to_string(), "human,female,thief");
        assert_eq!(character.name, name);
        assert_eq!(game.save_id, format!("{}-1", name.to_lowercase()));
        frame(&mut app, &[WAIT]);
        assert_eq!(actions(&app), [Action::Wait]);
    }

    /// A release build's New run lists one scenario, the campaign's start, so it is skipped:
    /// the race step opens over it at once (reading its pick), and Esc there goes back to the
    /// main menu. (Tests are developer builds, so the stack is built by hand.)
    #[test]
    fn a_release_new_run_is_skipped_both_ways() {
        let content = content();
        let release = NewRunScreen::new(&content, Some(5));
        assert!(release.skips());
        let mut stack = ScreenStack::default();
        stack.screens.push(Screen::Menu(MainMenu::main(None)));
        stack.screens.push(Screen::NewRun(release));
        let draft = stack.draft();
        let pick = draft.scenario.as_ref().map(|p| p.id.as_str());
        assert_eq!(pick, Some("ascmearc"));
        let race = ChoiceScreen::new(Step::Race, &content, &draft);
        stack.screens.push(Screen::Choice(race));
        stack.close();
        assert_eq!(stack.names(), [ScreenName::Main]);
        // A developer build's New run is shown, and closing what is above it leaves it open.
        stack
            .screens
            .push(Screen::NewRun(NewRunScreen::new(&content, None)));
        stack
            .screens
            .push(Screen::Settings(SettingsScreen::default()));
        stack.close();
        assert_eq!(stack.names(), [ScreenName::Main, ScreenName::NewRun]);
    }

    #[test]
    fn character_creation_goes_back_and_remembers_choices() {
        let (up, down) = ((KeyCode::ArrowUp, ""), DOWN);
        let mut app = app(false);
        app.update();
        // New run, then race: down from human to dwarf; the stars; gender: down to other;
        // class: up from thief to ranger; then a name, typed (`k` and `j` type, they move
        // nothing here).
        frame(
            &mut app,
            &[
                ENTER, ENTER, down, ENTER, ENTER, down, ENTER, up, ENTER, ENTER,
            ],
        );
        assert_eq!(names(&app).last(), Some(&ScreenName::Name));
        let typed = [
            (KeyCode::KeyK, "K"),
            (KeyCode::KeyA, "a"),
            (KeyCode::KeyJ, "j"),
        ];
        frame(&mut app, &typed);
        frame(&mut app, &[ENTER]);
        assert_eq!(names(&app).last(), Some(&ScreenName::Summary));
        let draft = app.world().resource::<ScreenStack>().draft();
        let spec = draft.spec(&content()).expect("a character");
        assert_eq!(spec.to_string(), "dwarf,other,ranger");
        assert_eq!(draft.name(&content()), "Kaj");
        // Change race (the summary's third row): back on the race step as it was left, the
        // gender, class, and name steps closed.
        frame(&mut app, &[down, down, ENTER]);
        assert_eq!(
            names(&app),
            [ScreenName::Main, ScreenName::NewRun, ScreenName::Race]
        );
        // Another race (and its stars); the gender, class, and name open on what was chosen
        // before.
        frame(&mut app, &[down, ENTER, ENTER, ENTER, ENTER, ENTER, ENTER]);
        assert_eq!(names(&app).last(), Some(&ScreenName::Summary));
        let draft = app.world().resource::<ScreenStack>().draft();
        let spec = draft.spec(&content()).expect("a character");
        assert_eq!(spec.to_string(), "halfling,other,ranger");
        assert_eq!(draft.name(&content()), "Kaj");
        // Esc steps back one screen at a time, to the main menu.
        for left in [8, 7, 6, 5, 4, 3, 2, 1] {
            frame(&mut app, &[ESC]);
            assert_eq!(names(&app).len(), left);
        }
        frame(
            &mut app,
            &[
                ENTER, ENTER, ENTER, ENTER, ENTER, ENTER, ENTER, ENTER, ENTER,
            ],
        );
        let game = app.world().resource::<Game>();
        let character = game.world.character().expect("a character");
        assert_eq!(
            (character.spec.to_string().as_str(), character.name.as_str()),
            ("halfling,other,ranger", "Kaj"),
            "the choices are remembered until the run starts"
        );
    }

    /// In Grimbold's store in the courtyard (spec §7 "Shops") with 60 gold (a), a scroll of light
    /// (b), and an unpaid dagger (c), on the cell inside the door (to the west).
    fn in_the_shop(content: Arc<Content>) -> warlock_core::World {
        let mut world = warlock_core::World::new(content, "courtyard", None).expect("run");
        let walk = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
                    move:S*5 move:E*5 move:E move:NE pickup move:SW move:W";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        assert_eq!(world.player_pos(), warlock_core::Pos::new(54, 14));
        world
    }

    #[test]
    fn leaving_a_shop_owing_asks_to_pay_and_leave_or_be_turned_back() {
        let content = content();
        let world = in_the_shop(content.clone());
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        let (west, yes, no) = (WEST, (KeyCode::KeyY, "y"), (KeyCode::KeyN, "n"));
        frame(&mut app, &[west]);
        assert_eq!(names(&app), [ScreenName::Dialog], "pay and leave?");
        assert!(
            actions(&app).is_empty(),
            "nothing happens before the answer"
        );
        // Esc, `n`, and Enter on the first selection (Don't pay) all send the step, which the
        // rules turn back.
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app), [Action::Move(Dir::W)]);
        for answer in [no, ENTER] {
            let before = actions(&app).len();
            frame(&mut app, &[west, answer]);
            assert!(names(&app).is_empty());
            assert_eq!(actions(&app)[before..], [Action::Move(Dir::W)]);
        }
        // Yes pays and leaves in one action.
        frame(&mut app, &[west, yes]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).last(), Some(&Action::Pay(Some(Dir::W))));
        // `p` pays without asking or leaving.
        let paid = actions(&app).len();
        frame(&mut app, &[(KeyCode::KeyP, "p")]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).len(), paid + 1);
        assert_eq!(actions(&app).last(), Some(&Action::Pay(None)));
        // A held key stops at the shop's edge: repeats never open the dialog.
        let mut held = key(west.0, west.1);
        held.repeat = true;
        app.world_mut().write_message(held);
        app.update();
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).len(), paid + 1);
    }

    #[test]
    fn dropping_what_a_shop_buys_asks_whether_to_sell() {
        use warlock_core::Positions;
        let content = content();
        let world = in_the_shop(content.clone());
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        let (d, b, yes, no) = (
            (KeyCode::KeyD, "d"),
            (KeyCode::KeyB, "b"),
            (KeyCode::KeyY, "y"),
            (KeyCode::KeyN, "n"),
        );
        frame(&mut app, &[d, b, ENTER]);
        assert_eq!(names(&app), [ScreenName::Dialog], "sell?");
        assert!(actions(&app).is_empty());
        // No keeps it (nothing is dropped); yes sells.
        frame(&mut app, &[no]);
        assert_eq!((names(&app), actions(&app)), (vec![], vec![]));
        frame(&mut app, &[d, b, ENTER, yes]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app), [Action::Sell(Positions::of([1]))]);
        // Putting back the unpaid dagger (`a`, its class first) is a plain drop: nothing to sell.
        frame(&mut app, &[d, (KeyCode::KeyA, "a"), ENTER]);
        assert!(names(&app).is_empty());
        assert_eq!(
            actions(&app).last(),
            Some(&Action::Drop(Positions::of([2])))
        );
    }

    #[test]
    fn pick_up_drop_and_inventory_open_screens_that_choose_actions() {
        use warlock_core::Positions;
        let content = content();
        let game = Game::from_world(on_the_pile(content.clone()), "tamsin-1".into());
        let mut app = app_with(content, Some(game));
        // Two stacks are left on the pile: `,` opens the choice; a letter and Enter pick up,
        // and the next press in the same frame is gameplay again.
        let (comma, b, i, d) = (
            (KeyCode::Comma, ","),
            (KeyCode::KeyB, "b"),
            (KeyCode::KeyI, "I"),
            (KeyCode::KeyD, "d"),
        );
        frame(&mut app, &[comma]);
        assert_eq!(names(&app), [ScreenName::PickUp]);
        frame(&mut app, &[b, ENTER, WAIT]);
        assert!(names(&app).is_empty());
        assert_eq!(
            actions(&app),
            [Action::PickUp(Positions::of([1])), Action::Wait]
        );
        // `;` picks up too; Esc cancels without an action.
        frame(&mut app, &[(KeyCode::Semicolon, ";"), ESC]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).len(), 2);
        // The inventory opens and closes on `I`; `d` chooses what to drop.
        frame(&mut app, &[i]);
        assert_eq!(names(&app), [ScreenName::Inventory]);
        frame(&mut app, &[i]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[d, ENTER]);
        assert_eq!(
            actions(&app).last(),
            Some(&Action::Drop(Positions::of([0])))
        );
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
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        let (i, c, a) = (
            (KeyCode::KeyI, "i"),
            (KeyCode::KeyC, "c"),
            (KeyCode::KeyA, "a"),
        );
        frame(&mut app, &[i]);
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
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
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
        frame(&mut app, &[(KeyCode::KeyI, "i")]);
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
    fn info_screen_digits_replace_the_page_and_esc_returns_to_play() {
        let mut app = app(true);
        let digit = |n: u8| {
            let code = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3][usize::from(n) - 1];
            (code, ["1", "2", "3"][usize::from(n) - 1])
        };
        // Digits do nothing in play.
        frame(&mut app, &[digit(2)]);
        assert!(names(&app).is_empty());
        assert!(actions(&app).is_empty());
        frame(&mut app, &[(KeyCode::Digit2, "@")]);
        assert_eq!(names(&app), [ScreenName::Character]);
        frame(&mut app, &[digit(2)]);
        assert_eq!(names(&app), [ScreenName::Inventory], "replaced, not pushed");
        frame(&mut app, &[digit(3)]);
        assert_eq!(names(&app), [ScreenName::Equipment]);
        // Its own page, a page that does not exist, and the numpad change nothing.
        frame(
            &mut app,
            &[digit(3), (KeyCode::Digit9, "9"), (KeyCode::Numpad1, "1")],
        );
        assert_eq!(names(&app), [ScreenName::Equipment]);
        // A sub-step is not a page: its digits are its own.
        frame(&mut app, &[(KeyCode::KeyC, "c")]);
        assert_eq!(names(&app), [ScreenName::Equipment, ScreenName::EquipItem]);
        frame(&mut app, &[digit(1)]);
        assert_eq!(names(&app), [ScreenName::Equipment, ScreenName::EquipItem]);
        frame(&mut app, &[ESC, digit(1), digit(2), ESC]);
        assert!(
            names(&app).is_empty(),
            "Esc returns to where the first was opened"
        );
        assert!(actions(&app).is_empty());
    }

    #[test]
    fn the_weapon_skills_screen_opens_on_w_and_a_bash_asks_for_a_direction() {
        let mut app = app(true);
        let skills = (KeyCode::KeyW, "W");
        frame(&mut app, &[skills]);
        assert_eq!(names(&app), [ScreenName::WeaponSkills]);
        frame(&mut app, &[WAIT]);
        assert_eq!(names(&app), [ScreenName::WeaponSkills]);
        frame(&mut app, &[skills]);
        assert!(names(&app).is_empty());
        assert!(actions(&app).is_empty(), "no time passes");
        // Bash is an ability: unbound by default. Bound (here to `B`) without a shield, the
        // rules say why at once (no prompt, no time).
        let bind_b = |app: &mut App| {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let bash = crate::input::KeyCommand::Bash;
            assert!(bindings.keys_of(bash).is_empty());
            let key = crate::input::KeyBinding::from_text("B").expect("a key");
            bindings.0.push((key, bash));
        };
        bind_b(&mut app);
        frame(&mut app, &[(KeyCode::KeyB, "B")]);
        assert_eq!(actions(&app), [Action::Bash(Dir::N)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        // With a shield in hand, it asks for a direction in the message line, like opening a
        // door, as the abilities menu does.
        let mut app = with_a_shield();
        bind_b(&mut app);
        frame(&mut app, &[(KeyCode::KeyB, "B")]);
        assert_eq!(
            app.world().resource::<InputMode>(),
            &InputMode::Ability(Ability::ShieldBash)
        );
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Bash(Dir::E)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
    }

    #[test]
    fn a_disarm_key_asks_for_a_direction_and_disarms() {
        // Disarming is an ability: unbound by default. Bound (here to `Y`), the showcase's
        // thief, trained in Disable device, is asked which way, as the abilities menu asks.
        let mut app = app(true);
        {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let disarm = crate::input::KeyCommand::Disarm;
            assert!(bindings.keys_of(disarm).is_empty());
            let key = crate::input::KeyBinding::from_text("Y").expect("a key");
            bindings.0.push((key, disarm));
        }
        frame(&mut app, &[(KeyCode::KeyY, "Y")]);
        assert_eq!(
            app.world().resource::<InputMode>(),
            &InputMode::Ability(Ability::Disarm)
        );
        assert!(actions(&app).is_empty(), "nothing yet");
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Disarm(Dir::E)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
    }

    #[test]
    fn picking_a_lock_asks_for_a_direction_from_the_menu_or_its_own_key() {
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // The showcase's thief is trained in Open lock: `a` `d` (Pick lock, after their birth
        // sign's stare and Appraise) asks for a direction.
        let mut app = app(true);
        frame(&mut app, &[(KeyCode::KeyA, "a"), (KeyCode::KeyD, "d")]);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::Ability(Ability::PickLock));
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::PickLock(Dir::E)]);
        // Its own key is unbound by default; bound (here to `B`), it does the same.
        let mut app = self::app(true);
        {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let pick = crate::input::KeyCommand::PickLock;
            assert!(bindings.keys_of(pick).is_empty());
            let key = crate::input::KeyBinding::from_text("B").expect("a key");
            bindings.0.push((key, pick));
        }
        frame(&mut app, &[(KeyCode::KeyB, "B")]);
        assert_eq!(mode(&app), InputMode::Ability(Ability::PickLock));
        frame(&mut app, &[WEST]);
        assert_eq!(actions(&app), [Action::PickLock(Dir::W)]);
        assert_eq!(mode(&app), InputMode::Normal);
    }

    #[test]
    fn stealing_asks_for_a_direction_from_the_menu_or_its_own_key() {
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // The showcase's thief can steal: `a` `e` (Steal) asks for a direction in the message
        // line, and a direction steals.
        let mut app = app(true);
        frame(&mut app, &[(KeyCode::KeyA, "a"), (KeyCode::KeyE, "e")]);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::Ability(Ability::Steal));
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Steal(Dir::E)]);
        // Unbound by default; bound (here to `Y`), it asks the same.
        let mut app = self::app(true);
        {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let steal = crate::input::KeyCommand::Steal;
            assert!(bindings.keys_of(steal).is_empty());
            let key = crate::input::KeyBinding::from_text("Y").expect("a key");
            bindings.0.push((key, steal));
        }
        frame(&mut app, &[(KeyCode::KeyY, "Y")]);
        assert_eq!(mode(&app), InputMode::Ability(Ability::Steal));
        frame(&mut app, &[WEST]);
        assert_eq!(actions(&app), [Action::Steal(Dir::W)]);
        assert_eq!(mode(&app), InputMode::Normal);
    }

    /// A run of the showcase's thief with the storeroom's small shield in hand.
    fn with_a_shield() -> App {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
                    pickup:a equip:off_hand:a";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        app_with(content, Some(Game::from_world(world, "tamsin-1".into())))
    }

    #[test]
    fn the_abilities_menu_uses_an_ability_and_remembers_it_for_the_quick_repeat() {
        const ABILITIES: (KeyCode, &str) = (KeyCode::KeyA, "a");
        const REPEAT: (KeyCode, &str) = (KeyCode::KeyX, "X");
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // Without a shield: the menu opens on `a`; the shield bash (`f`) is greyed out, so its
        // letter only says why; Esc closes the menu, and no time passed.
        let mut app = app(true);
        frame(&mut app, &[ABILITIES]);
        assert_eq!(names(&app), [ScreenName::Abilities]);
        frame(&mut app, &[(KeyCode::KeyF, "f")]);
        assert_eq!(names(&app), [ScreenName::Abilities]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty() && actions(&app).is_empty());
        // The quick repeat with nothing used yet opens the menu.
        frame(&mut app, &[REPEAT]);
        assert_eq!(names(&app), [ScreenName::Abilities]);
        // With a shield: `a` `f` closes the menu and asks for a direction; a direction bashes.
        let mut app = with_a_shield();
        frame(&mut app, &[ABILITIES, (KeyCode::KeyF, "f")]);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::Ability(Ability::ShieldBash));
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Bash(Dir::E)]);
        let stack = app.world().resource::<ScreenStack>();
        assert_eq!(stack.last_ability(), Some(Ability::ShieldBash));
        // `X` uses it again; Esc cancels the direction prompt rather than pausing.
        frame(&mut app, &[REPEAT]);
        assert_eq!(mode(&app), InputMode::Ability(Ability::ShieldBash));
        frame(&mut app, &[ESC]);
        assert_eq!(mode(&app), InputMode::Normal);
        assert!(names(&app).is_empty());
        // The menu opens on it, so Enter repeats it too.
        frame(&mut app, &[ABILITIES, ENTER, WEST]);
        assert_eq!(actions(&app), [Action::Bash(Dir::E), Action::Bash(Dir::W)]);
    }

    /// A run of a bard of level 32, who throws their voice (#244), with keys bound for it and
    /// the lantern's hood (`B`, `J`: both unbound by default).
    fn bard() -> App {
        let content = content();
        let spec: warlock_core::character::CharacterSpec =
            "human,female,bard".parse().expect("a bard");
        let mut world = warlock_core::World::start_run(
            content.clone(),
            "showcase",
            None,
            Some(&spec),
            None,
            false,
        )
        .expect("run");
        let factor = warlock_core::content::Factor::of(&content, Some(&spec));
        let need = content.experience().threshold_for(32, factor);
        let walk = format!("xp:{need} levelup:auto");
        for action in warlock_core::action::parse_actions(&walk).expect("actions") {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world, "bard-1".into())));
        let mut bindings = app.world_mut().resource_mut::<Bindings>();
        for (key, command) in [
            ("B", crate::input::KeyCommand::ThrowVoice),
            ("J", crate::input::KeyCommand::HoodLantern),
        ] {
            assert!(bindings.keys_of(command).is_empty(), "unbound by default");
            let key = crate::input::KeyBinding::from_text(key).expect("a key");
            bindings.0.push((key, command));
        }
        app
    }

    #[test]
    fn a_thrown_voice_is_aimed_at_a_cell_and_the_hood_needs_nothing() {
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // The bard's Throw your voice starts aiming, on the player (no one in view); two steps
        // east and Enter throw it there.
        let mut app = bard();
        let me = app.world().resource::<Game>().world.player_pos();
        frame(&mut app, &[(KeyCode::KeyB, "B")]);
        assert!(names(&app).is_empty());
        let aim = match mode(&app) {
            InputMode::Aim(aim) => aim,
            other => panic!("aiming, not {other:?}"),
        };
        assert_eq!(aim.kind, AimKind::Ability(Ability::ThrowVoice));
        assert_eq!(aim.at, me);
        frame(&mut app, &[EAST, EAST, ENTER]);
        assert_eq!(mode(&app), InputMode::Normal);
        let at = Pos::new(me.x + 2, me.y);
        assert_eq!(actions(&app), [Action::ThrowVoice(at)]);
        // The lantern's hood, at once.
        frame(&mut app, &[(KeyCode::KeyJ, "J")]);
        assert_eq!(actions(&app), [Action::ThrowVoice(at), Action::HoodLantern]);
    }

    #[test]
    fn single_stacks_and_empty_hands_need_no_screen() {
        use warlock_core::Positions;
        let mut app = app(true);
        // Nothing here and nothing carried: the actions are refused by the core, no screen.
        frame(&mut app, &[(KeyCode::Comma, ","), (KeyCode::KeyD, "d")]);
        assert!(names(&app).is_empty());
        assert_eq!(
            actions(&app),
            [
                Action::PickUp(Positions::default()),
                Action::Drop(Positions::default())
            ]
        );
    }

    #[test]
    fn death_deletes_only_that_characters_save_and_the_death_screen_leads_to_the_main_menu() {
        let content = content();
        let dir = std::env::temp_dir().join(format!("warlock-death-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let args = Args::parse_from(["warlock", "--save-dir", dir.to_str().expect("utf-8")]);
        let store = SaveStore::from_args(&args);
        let saved = |store: &SaveStore| -> Vec<String> {
            store.characters().into_iter().map(|c| c.id).collect()
        };
        // Another character is saved too.
        let other = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        store.save_run("tamsin-1", &other).expect("saved");
        // The run was saved while it went on; then the archer shoots the lit player down.
        let mut world = warlock_core::World::new(content.clone(), "arena", None).expect("run");
        store.save_run("pip-1", &world).expect("saved");
        assert_eq!(saved(&store), ["pip-1", "tamsin-1"]);
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
            .insert_resource(Game::from_world(world, "pip-1".into()))
            .init_resource::<ScreenStack>()
            .init_resource::<Settings>()
            .init_resource::<crate::settings::SettingsStore>()
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
        assert_eq!(
            saved(&store),
            ["tamsin-1"],
            "permadeath: the dead character's save is gone, and only that one"
        );
        // It takes every key; gameplay sees none.
        frame(&mut app, &[WAIT, (KeyCode::KeyK, "k"), HELP]);
        assert_eq!(names(&app), [ScreenName::Death]);
        assert!(actions(&app).is_empty());
        // Enter leaves the run for the main menu, which offers to continue as the other one.
        frame(&mut app, &[ENTER]);
        assert!(!app.world().contains_resource::<Game>());
        assert_eq!(names(&app), [ScreenName::Main]);
        assert_eq!(saved(&store), ["tamsin-1"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_characters_screen_resumes_or_deletes_a_saved_character() {
        let content = content();
        let dir = std::env::temp_dir().join(format!("warlock-chars-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let args = Args::parse_from(["warlock", "--save-dir", dir.to_str().expect("utf-8")]);
        let store = SaveStore::from_args(&args);
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        for (id, name) in [("brakka-1", "Brakka"), ("hilde-1", "Hilde")] {
            let world = warlock_core::World::with_character_named(
                content.clone(),
                "showcase",
                None,
                Some(&spec),
                Some(name),
            )
            .expect("run");
            store.save_run(id, &world).expect("saved");
        }
        let mut app = app_with(content, None);
        app.insert_resource(store.clone());
        app.update();
        // Continue, then Characters.
        frame(&mut app, &[DOWN, ENTER]);
        assert_eq!(names(&app), [ScreenName::Main, ScreenName::Characters]);
        // Delete the first one listed: the dialog asks first; Yes deletes, and the list shows
        // the one left.
        let first = store.characters()[0].id.clone();
        frame(&mut app, &[(KeyCode::Delete, "")]);
        assert_eq!(
            names(&app),
            [ScreenName::Main, ScreenName::Characters, ScreenName::Dialog]
        );
        frame(&mut app, &[(KeyCode::KeyY, "y")]);
        assert_eq!(names(&app), [ScreenName::Main, ScreenName::Characters]);
        let left: Vec<String> = store.characters().into_iter().map(|c| c.id).collect();
        assert_eq!(left.len(), 1);
        assert!(!left.contains(&first));
        // Enter resumes the one left, as itself.
        frame(&mut app, &[ENTER]);
        let game = app.world().get_resource::<Game>().expect("a run");
        assert_eq!(game.save_id, left[0]);
        assert!(names(&app).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn chat_talks_to_the_one_beside_and_the_dialogue_stays_open_while_asking() {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        // Into the stairs nook, beside the old hermit.
        let walk = "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        fn apply(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
            for PlayerAction(action) in actions.read() {
                game.apply(action);
            }
        }
        app.add_systems(Update, (apply, converse).chain().after(count_actions));
        // `C` talks to the one creature beside the player at once; he has topics.
        frame(&mut app, &[(KeyCode::KeyC, "C")]);
        assert_eq!(actions(&app), [Action::Chat(Dir::SE)]);
        assert_eq!(names(&app), [ScreenName::Dialogue]);
        // A topic's number asks (no time passes); the answer reopens the screen.
        let turn = app.world().resource::<Game>().world.turn();
        frame(&mut app, &[(KeyCode::Digit6, "6")]);
        let ask = Action::Ask {
            dir: Dir::SE,
            topic: "stairs".into(),
        };
        assert_eq!(actions(&app).last(), Some(&ask));
        assert_eq!(names(&app), [ScreenName::Dialogue]);
        let game = app.world().resource::<Game>();
        assert_eq!(game.world.turn(), turn);
        let last = game.world.messages().iter().last().map(|m| m.text.clone());
        assert!(last.is_some_and(|m| m.contains("Five stairs")));
        // His quest comes first: asking hears the offer, and the screen reopens on accepting it.
        frame(&mut app, &[(KeyCode::Digit1, "1")]);
        let topics = |app: &App| {
            let game = app.world().resource::<Game>();
            let talk = last_talk(&game.last_outcome).expect("talk");
            (talk.topics.iter().map(|t| t.name.clone()).take(2)).collect::<Vec<_>>()
        };
        assert_eq!(names(&app), [ScreenName::Dialogue]);
        assert_eq!(
            topics(&app),
            ["Accept: Rats in the arena", "Decline: Rats in the arena"]
        );
        frame(&mut app, &[(KeyCode::Digit1, "1")]);
        let accept = Action::Ask {
            dir: Dir::SE,
            topic: "accept-rats".into(),
        };
        assert_eq!(actions(&app).last(), Some(&accept));
        assert_eq!(names(&app), [ScreenName::Dialogue]);
        assert_eq!(topics(&app)[0], "Rats in the arena");
        let game = app.world().resource::<Game>();
        assert_eq!(game.world.perception().quests().len(), 1);
        // Esc leaves; the next press is gameplay again.
        frame(&mut app, &[ESC, WAIT]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).last(), Some(&Action::Wait));
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
    }

    #[test]
    fn chat_asks_for_a_direction_with_no_one_beside() {
        let mut app = app(true);
        frame(&mut app, &[(KeyCode::KeyC, "C")]);
        assert_eq!(
            app.world().resource::<InputMode>(),
            &InputMode::Direction(crate::input::KeyCommand::Chat)
        );
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Chat(Dir::E)]);
        assert!(names(&app).is_empty());
    }

    const UP: (KeyCode, &str) = (KeyCode::ArrowUp, "");
    const YES: (KeyCode, &str) = (KeyCode::KeyY, "y");
    const NO: (KeyCode, &str) = (KeyCode::KeyN, "n");

    fn saves_sent(app: &App) -> Vec<SaveCommand> {
        let saves = app.world().resource::<Messages<SaveCommand>>();
        saves.iter_current_update_messages().copied().collect()
    }

    #[test]
    fn quit_from_pause_asks_first_and_saves_on_yes() {
        let mut app = app(true);
        // Quit is the last row: Esc, Up (wraps to Quit), Enter opens the dialog, nothing is saved.
        frame(&mut app, &[ESC, UP, ENTER]);
        assert_eq!(names(&app), [ScreenName::Pause, ScreenName::Dialog]);
        frame(&mut app, &[]);
        assert!(saves_sent(&app).is_empty());
        // Enter takes the default, No; Esc and n cancel too. The pause menu is as it was.
        for answer in [ENTER, ESC, NO] {
            frame(&mut app, &[answer]);
            assert_eq!(names(&app), [ScreenName::Pause]);
            assert!(saves_sent(&app).is_empty());
            frame(&mut app, &[ENTER]);
            assert_eq!(names(&app), [ScreenName::Pause, ScreenName::Dialog]);
        }
        frame(&mut app, &[YES]);
        assert_eq!(saves_sent(&app), [SaveCommand::SaveAndQuit]);
        assert_eq!(names(&app), [ScreenName::Pause]);
    }

    #[test]
    fn held_keys_never_confirm() {
        let mut app = app(true);
        frame(&mut app, &[ESC, UP, ENTER]);
        assert_eq!(names(&app), [ScreenName::Pause, ScreenName::Dialog]);
        let mut held = key(YES.0, YES.1);
        held.repeat = true;
        app.world_mut().write_message(held);
        app.update();
        assert_eq!(names(&app), [ScreenName::Pause, ScreenName::Dialog]);
        assert!(saves_sent(&app).is_empty());
    }

    #[test]
    fn q_does_nothing_unless_bound_and_then_asks() {
        const Q: (KeyCode, &str) = (KeyCode::KeyQ, "Q");
        let mut app = app(true);
        frame(&mut app, &[Q]);
        assert!(names(&app).is_empty());
        assert!(saves_sent(&app).is_empty());
        app.world_mut().resource_mut::<Bindings>().0.push((
            crate::input::KeyBinding::Char("Q".into()),
            crate::input::KeyCommand::Quit,
        ));
        frame(&mut app, &[Q]);
        assert_eq!(names(&app), [ScreenName::Dialog]);
        frame(&mut app, &[NO]);
        assert!(names(&app).is_empty());
        assert!(saves_sent(&app).is_empty());
        frame(&mut app, &[Q]);
        frame(&mut app, &[YES]);
        assert!(names(&app).is_empty());
        assert_eq!(saves_sent(&app), [SaveCommand::SaveAndQuit]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn main_menu_quit_asks_first() {
        let mut app = app(false);
        // Quit is the last row of the main menu.
        frame(&mut app, &[UP, ENTER]);
        assert_eq!(names(&app), [ScreenName::Main, ScreenName::Dialog]);
        frame(&mut app, &[ESC]);
        assert_eq!(names(&app), [ScreenName::Main]);
        frame(&mut app, &[ENTER]);
        frame(&mut app, &[YES]);
        let exits = app.world().resource::<Messages<AppExit>>();
        assert_eq!(exits.iter_current_update_messages().count(), 1);
    }

    /// An app in the arena while its fight goes on (ten turns in, everything said so far read),
    /// applying actions and waiting at `--more--` like the game, its console 80 by 24.
    fn arena_app() -> App {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "arena", None).expect("run");
        for _ in 0..10 {
            world.apply(&Action::Wait);
        }
        let mut app = app_with(content, Some(Game::from_world(world, "pip-1".into())));
        fn apply(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
            for PlayerAction(action) in actions.read() {
                game.apply(action);
            }
        }
        let mut console = Console::default();
        console.resize(80, 24);
        app.insert_resource(console).add_systems(
            Update,
            (apply, super::super::more::wait)
                .chain()
                .after(count_actions),
        );
        app
    }

    fn mode(app: &App) -> InputMode {
        *app.world().resource::<InputMode>()
    }

    #[test]
    fn more_waits_for_its_keys_and_nothing_else_acts_meanwhile() {
        const SPACE: (KeyCode, &str) = (KeyCode::Space, "");
        let mut app = arena_app();
        // A turn of the fight says more than the message line holds at once.
        frame(&mut app, &[WAIT]);
        assert_eq!(mode(&app), InputMode::More(0));
        let pages = {
            let (game, bindings) = (app.world().resource::<Game>(), Bindings::default());
            super::super::more::page_count(game, 80, &bindings)
        };
        assert!(pages >= 2, "{pages} pages");
        // Other keys do nothing: no move, no wait, no screen.
        frame(
            &mut app,
            &[(KeyCode::Numpad2, "2"), WAIT, HELP, (KeyCode::KeyV, "v")],
        );
        assert_eq!(actions(&app).len(), 1);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::More(0));
        // A held key's repeats never turn a page.
        app.world_mut().write_message(KeyboardInput {
            repeat: true,
            ..key(SPACE.0, SPACE.1)
        });
        app.update();
        assert_eq!(mode(&app), InputMode::More(0));
        // Space shows the next page, up to the last, which waits for nothing.
        for page in 1..pages {
            assert_eq!(mode(&app), InputMode::More(page - 1));
            frame(&mut app, &[SPACE]);
        }
        assert_eq!(mode(&app), InputMode::Normal);
        // Play goes on, until a turn says too much again; Esc skips the rest of its messages
        // without opening the pause menu.
        frame(&mut app, &[WAIT]);
        assert_eq!(actions(&app).len(), 2);
        while mode(&app) == InputMode::Normal && actions(&app).len() < 30 {
            frame(&mut app, &[WAIT]);
        }
        assert_eq!(mode(&app), InputMode::More(0));
        frame(&mut app, &[ESC]);
        assert_eq!(mode(&app), InputMode::Normal);
        assert!(names(&app).is_empty());
    }

    #[test]
    fn the_message_history_opens_with_its_key_and_closes_with_it() {
        let mut app = app(true);
        frame(&mut app, &[(KeyCode::KeyV, "v")]);
        assert_eq!(names(&app), [ScreenName::Messages]);
        frame(&mut app, &[(KeyCode::KeyV, "v")]);
        assert!(names(&app).is_empty());
    }
}
