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
use super::craft_pick_screen::{CraftPickScreen, craft_dialog};
use super::death_screen::DeathScreen;
use super::describe_screen::DescribeScreen;
use super::dialog_screen::{Dialog, DialogScreen};
use super::dialogue_screen::{DialogueScreen, last_talk};
use super::equipment_screen::{EquipmentScreen, ItemScreen};
use super::files_screen::FilesScreen;
use super::graveyard_screen::GraveyardScreen;
use super::info_tabs;
use super::level_up_screen::LevelUpScreen;
use super::magic_screen::MagicScreen;
use super::menu::{MainMenu, Press, quit_dialog};
use super::messages_screen::MessagesScreen;
use super::motion::Motion;
use super::name_screen::NameScreen;
use super::new_run_screen::{self, NewRunScreen};
use super::object_screen::{IdentifyBy, ObjectList, ObjectScreen};
use super::profession_screen::ProfessionScreen;
use super::quest_screen::QuestScreen;
use super::rig_screen::RigScreen;
use super::scribe_screen::ScribeScreen;
use super::settings_screen::SettingsScreen;
use super::shape_screen::ShapeScreen;
use super::sign_screen::SignScreen;
use super::skills_screen::SkillsScreen;
use super::story_screen::StoryScreen;
use super::talents_screen::TalentsScreen;
use super::target::AimKind;
use super::weapon_skills_screen::WeaponSkillsScreen;
use crate::cli::Args;
use crate::game::{Game, GameContent, PlayerAction};
use crate::graveyard::{self, GraveyardStore};
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
use warlock_core::content::{Content, RecipeId, SlotId};
use warlock_core::perception::PerceptionView;
use warlock_core::skills::SkillId;
use warlock_core::world::Conversation;

/// The screens that can be opened. Also the values of `--screen`, a dev flag for screenshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ScreenName {
    /// The main menu (no run).
    Main,
    /// The saved characters, from the main menu: resume one, or delete it.
    Characters,
    /// The fallen, from the main menu once anyone has died: the most experienced first.
    Graveyard,
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
    /// The player's magic (page 6): the caster styles with their ranks and marks, and the spells known with their rolls. No key of its own in play.
    Magic,
    /// The player's quests, active and done (`q` in play).
    Quests,
    /// The player's talents, then every other with what it needs (page 8; its own command,
    /// unbound by default).
    Talents,
    /// The player's background story (page 9): its paragraphs. Opened by the app once as a new
    /// run begins (`on_run_start`), and by `--screen story`, as that first-turn screen.
    Story,
    /// The message history (`M` in play): every message the log keeps, the newest at the
    /// bottom. No info page.
    Messages,
    /// Choose what to pick up from the pile here (`,` or `g` where several objects lie).
    PickUp,
    /// The inventory in its drop mode, as `d` in play opens it (#1196): choose what to drop.
    /// The inventory's own `D` turns the mode on in place, under this name too while it lasts.
    Drop,
    /// Choose what to throw (`t` in play, with no throwing weapon readied).
    Throw,
    /// Choose what to eat (`E` in play): food lying here, then food in the pack.
    Eat,
    /// Choose a scroll or grimoire to read (`r` in play): a spell scroll is aimed next, the
    /// scroll of identify asks what to read it on, a grimoire whose spell may be learned asks
    /// first, with the chance in words.
    Read,
    /// Choose what to read the scroll of identify on: the unidentified stacks carried. It takes
    /// the place of the read list where that scroll was chosen (as `--screen`, the first stack's
    /// scroll).
    Identify,
    /// Choose what to temper at a forge's anvil (the Temper ability, spec §7 "Special rooms"):
    /// the weapons and armour carried.
    Temper,
    /// Choose a wooden weapon to whittle with a knife (the Whittle ability).
    Whittle,
    /// Choose something carried to take apart for its materials (the Salvage ability).
    Salvage,
    /// Choose a damaged thing carried to mend in the field (the craftsperson's Make do and mend,
    /// `world/make_do.rs`).
    MakeDo,
    /// Choose a loose gem to cut (Jewelcraft's Cut recipe, from its profession's screen).
    Cut,
    /// Choose a scroll to make out unread (the Decipher ability, spec §8 "Scrolls"): those of a
    /// type not yet identified.
    Decipher,
    /// Choose what to give a kept beast of the player's for its neck (the Give ability, spec §8
    /// "Allies"): what fits a neck, carried.
    Give,
    /// The abilities menu (`a` in play): racial, class, skill, weapon skill, and item
    /// abilities, to choose one to use.
    Abilities,
    /// Choose a weapon's shape to call (the Hunter's Spirit weapon, from the abilities menu):
    /// every melee weapon, grouped by weapon skill.
    Shape,
    /// Choose a spell to write a scroll of (the wizard's Scribe, from the abilities menu): the
    /// spells known, grouped by school, each with its price and time.
    Scribe,
    /// Choose a trap to rig (the craftsperson's Trapwright, from the abilities menu): each rig
    /// with what it takes, greyed with what is lacking; a direction is asked next.
    Rig,
    /// A profession's recipes (its row in the abilities menu, "Metalcraft…"): grouped by what
    /// they make, each with what it takes, greyed with its reason when it cannot be made now.
    #[value(skip)]
    Profession(SkillId),
    /// The first profession's screen whose recipes the player sees (`--screen profession`), as
    /// [`ScreenName::Profession`].
    #[value(name = "profession")]
    FirstProfession,
    /// The forging wizard of a generic recipe (its row on a profession's screen): one list for
    /// each choice, then the dialog.
    #[value(skip)]
    CraftPick(RecipeId),
    /// The wizard of the first recipe with a choice (`--screen craft-pick`), as
    /// [`ScreenName::CraftPick`].
    #[value(name = "craft-pick")]
    FirstCraftPick,
    /// The dialog that asks to make the recipe of the wizard beneath with its picks, in the
    /// rules' words; it opens from the wizard's last step, so it is no `--screen`.
    #[value(skip)]
    CraftConfirm,
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
                | ScreenName::Magic
                | ScreenName::Quests
                | ScreenName::Talents
                | ScreenName::Story
                | ScreenName::Messages
                | ScreenName::PickUp
                | ScreenName::Drop
                | ScreenName::Throw
                | ScreenName::Eat
                | ScreenName::Read
                | ScreenName::Identify
                | ScreenName::Temper
                | ScreenName::Whittle
                | ScreenName::Salvage
                | ScreenName::MakeDo
                | ScreenName::Cut
                | ScreenName::Decipher
                | ScreenName::Give
                | ScreenName::Abilities
                | ScreenName::Shape
                | ScreenName::Scribe
                | ScreenName::Rig
                | ScreenName::Profession(_)
                | ScreenName::FirstProfession
                | ScreenName::CraftPick(_)
                | ScreenName::FirstCraftPick
                | ScreenName::CraftConfirm
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
    /// The characters who have died (the graveyard's entries, `graveyard.rs`), the most
    /// experienced first; only read for the screens that show them (the main menu offers the
    /// Graveyard with any, the Characters screen counts them).
    fallen: Vec<graveyard::Fallen>,
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
    /// The fallen.
    Graveyard(GraveyardScreen),
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
    Magic(MagicScreen),
    Quests(QuestScreen),
    Talents(TalentsScreen),
    Story(StoryScreen),
    Messages(MessagesScreen),
    Abilities(AbilitiesScreen),
    /// Choosing the shape of a spirit weapon.
    Shape(ShapeScreen),
    /// Choosing the spell of a scroll to write.
    Scribe(ScribeScreen),
    Rig(RigScreen),
    /// A profession's recipes.
    Profession(ProfessionScreen),
    /// The forging wizard: a generic recipe's choices.
    CraftPick(CraftPickScreen),
    Dialogue(DialogueScreen),
    Death(DeathScreen),
    LevelUp(LevelUpScreen),
    /// The dialog.
    Dialog(DialogScreen),
}

/// What a screen opening may read off the stack: the slot selected on the equipment screen
/// beneath, if any (the first slot if none), the carried stack looked at on a screen beneath
/// (the first if none), the ability used last, and the recipe and picks of the forging wizard
/// beneath (what its dialog asks about).
#[derive(Clone, Debug, Default)]
struct Beneath {
    slot: Option<SlotId>,
    item: Option<usize>,
    ability: Option<Ability>,
    craft: Option<(RecipeId, Vec<Option<usize>>)>,
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
        let Beneath { slot, item, .. } = beneath.clone();
        let objects = |list| Screen::Objects(ObjectScreen::new(list, run));
        let choice = |step| Screen::Choice(ChoiceScreen::new(step, content, draft));
        match name {
            ScreenName::Main => {
                // Continue skips a save of another version.
                let last = (launch.characters.iter())
                    .find(|c| c.playable())
                    .map(|c| c.name().to_string());
                let saved = !launch.characters.is_empty();
                Screen::Menu(MainMenu::main(last, saved).graveyard(!launch.fallen.is_empty()))
            }
            ScreenName::Characters => {
                let campaigns = content.campaigns().len() > 1;
                let characters = launch.characters.clone();
                let died = launch.fallen.len();
                Screen::Characters(CharactersScreen::new(characters, campaigns).died(died))
            }
            ScreenName::Graveyard => {
                Screen::Graveyard(GraveyardScreen::new(&launch.fallen, content))
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
            ScreenName::Magic => Screen::Magic(MagicScreen::new(run)),
            ScreenName::Quests => Screen::Quests(QuestScreen::new(run)),
            ScreenName::Talents => Screen::Talents(TalentsScreen::new(run)),
            ScreenName::Story => Screen::Story(StoryScreen::page(run)),
            ScreenName::Messages => Screen::Messages(MessagesScreen::new(run)),
            ScreenName::PickUp => objects(ObjectList::PickUp),
            ScreenName::Drop => Screen::Objects(ObjectScreen::dropping(run)),
            ScreenName::Throw => objects(ObjectList::Throw),
            ScreenName::Eat => objects(ObjectList::Eat),
            ScreenName::Read => objects(ObjectList::Read),
            ScreenName::Identify => objects(ObjectList::Identify(IdentifyBy::Scroll(
                beneath.item.unwrap_or(0),
            ))),
            ScreenName::Temper => objects(ObjectList::Temper),
            ScreenName::Whittle => objects(ObjectList::Whittle),
            ScreenName::Salvage => objects(ObjectList::Salvage),
            ScreenName::MakeDo => objects(ObjectList::MakeDo),
            ScreenName::Cut => objects(ObjectList::Cut),
            ScreenName::Decipher => objects(ObjectList::Decipher),
            ScreenName::Give => objects(ObjectList::Give),
            ScreenName::Abilities => Screen::Abilities(AbilitiesScreen::new(run, beneath.ability)),
            ScreenName::Shape => Screen::Shape(ShapeScreen::new(run)),
            ScreenName::Scribe => Screen::Scribe(ScribeScreen::new(run)),
            ScreenName::Rig => Screen::Rig(RigScreen::new(run)),
            ScreenName::Profession(skill) => {
                Screen::Profession(ProfessionScreen::new(run, Some(skill)))
            }
            ScreenName::FirstProfession => Screen::Profession(ProfessionScreen::new(run, None)),
            ScreenName::CraftPick(recipe) => {
                Screen::CraftPick(CraftPickScreen::new(run, Some(recipe)))
            }
            ScreenName::FirstCraftPick => Screen::CraftPick(CraftPickScreen::new(run, None)),
            ScreenName::CraftConfirm => {
                Screen::Dialog(DialogScreen::new(craft_dialog(run, beneath.craft)))
            }
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
            Screen::Graveyard(_) => ScreenName::Graveyard,
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
                ObjectList::Inventory if screen.is_dropping() => ScreenName::Drop,
                ObjectList::Inventory => ScreenName::Inventory,
                ObjectList::PickUp => ScreenName::PickUp,
                ObjectList::Throw => ScreenName::Throw,
                ObjectList::Eat => ScreenName::Eat,
                ObjectList::Read => ScreenName::Read,
                ObjectList::Identify(_) => ScreenName::Identify,
                ObjectList::Temper => ScreenName::Temper,
                ObjectList::Whittle => ScreenName::Whittle,
                ObjectList::Salvage => ScreenName::Salvage,
                ObjectList::MakeDo => ScreenName::MakeDo,
                ObjectList::Cut => ScreenName::Cut,
                ObjectList::Decipher => ScreenName::Decipher,
                ObjectList::Give => ScreenName::Give,
            },
            Screen::Equipment(_) => ScreenName::Equipment,
            Screen::EquipItem(_) => ScreenName::EquipItem,
            Screen::Describe(_) => ScreenName::Describe,
            Screen::Character(_) => ScreenName::Character,
            Screen::Skills(_) => ScreenName::Skills,
            Screen::WeaponSkills(_) => ScreenName::WeaponSkills,
            Screen::Magic(_) => ScreenName::Magic,
            Screen::Quests(_) => ScreenName::Quests,
            Screen::Talents(_) => ScreenName::Talents,
            Screen::Story(_) => ScreenName::Story,
            Screen::Messages(_) => ScreenName::Messages,
            Screen::Abilities(_) => ScreenName::Abilities,
            Screen::Shape(_) => ScreenName::Shape,
            Screen::Scribe(_) => ScreenName::Scribe,
            Screen::Rig(_) => ScreenName::Rig,
            Screen::Profession(screen) => match screen.skill() {
                Some(skill) => ScreenName::Profession(skill),
                None => ScreenName::FirstProfession,
            },
            Screen::CraftPick(screen) => match screen.chosen() {
                Some((recipe, _)) => ScreenName::CraftPick(recipe),
                None => ScreenName::FirstCraftPick,
            },
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
    fn craft_chosen(&self) -> Option<(RecipeId, Vec<Option<usize>>)> {
        self.screens.iter().rev().find_map(|s| match s {
            Screen::CraftPick(screen) => screen.chosen(),
            Screen::Profession(screen) => screen.chosen(),
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

    /// Shows what an import from the browser's file picker did on the controls screen, if it
    /// is open.
    pub(super) fn keymap_imported(&mut self, lines: Vec<String>) {
        for screen in &mut self.screens {
            if let Screen::Controls(screen) = screen {
                screen.imported(lines.clone());
            }
        }
    }

    /// Whether a screen is open that shows the run as it changes ([`refresh`]): the equipment
    /// screen, or an object list.
    fn shows_run(&self) -> bool {
        (self.screens.iter()).any(|s| matches!(s, Screen::Equipment(_) | Screen::Objects(_)))
    }

    /// Whether an object list is open: one left open beneath a dialog that acted names stacks
    /// by position until [`refresh`] reads the run again after the action.
    pub fn shows_objects(&self) -> bool {
        (self.screens.iter()).any(|s| matches!(s, Screen::Objects(_)))
    }

    /// Reads the run again into the screens that stay open while it changes: the equipment
    /// screen beneath a choice that was just done, and any object list, which names stacks by
    /// position (the read list beneath the dialog that reads a grimoire: a scroll used up moves
    /// the stacks after it, #591).
    fn refresh(&mut self, view: &PerceptionView<'_>) {
        for screen in &mut self.screens {
            match screen {
                Screen::Equipment(screen) => screen.refresh(view),
                Screen::Objects(screen) => screen.refresh(view),
                Screen::Profession(screen) => screen.refresh(view),
                Screen::CraftPick(screen) => screen.refresh(view),
                _ => {}
            }
        }
    }

    /// Whether the top screen asks to read the run again after a press (the forging wizard's
    /// later steps follow its picks: [`CraftPickScreen::reread`]).
    pub fn is_stale(&self) -> bool {
        matches!(self.screens.last(), Some(Screen::CraftPick(s)) if s.is_stale())
    }

    /// Reads the run again into the top screen when it asks ([`Self::is_stale`]).
    pub fn reread(&mut self, view: &PerceptionView<'_>) {
        if let Some(Screen::CraftPick(screen)) = self.screens.last_mut() {
            screen.reread(view);
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
            && !matches!(screen, Screen::Story(story) if story.is_opening())
            && let Some(action) = info_tabs::switch(screen.name(), press, bindings)
        {
            self.notice = None;
            return action;
        }
        let action = match self.screens.last_mut() {
            None => ScreenAction::Stay,
            Some(Screen::Menu(menu)) => menu.handle(press),
            Some(Screen::Characters(screen)) => screen.handle(press),
            Some(Screen::Graveyard(screen)) => screen.handle(press),
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
            Some(Screen::Magic(screen)) => screen.handle(press),
            Some(Screen::Quests(screen)) => screen.handle(press, bindings),
            Some(Screen::Talents(screen)) => screen.handle(press, bindings),
            Some(Screen::Story(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press)
            }
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
            Some(Screen::Shape(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Scribe(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Rig(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::Profession(screen)) => {
                if let Some((cols, rows)) = self.console {
                    screen.fit(cols, rows);
                }
                screen.handle(press, bindings)
            }
            Some(Screen::CraftPick(screen)) => {
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
            Some(Screen::Graveyard(screen)) => screen.draw(console, settings.clock, bindings),
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
            Some(Screen::Magic(screen)) => screen.draw(console, bindings),
            Some(Screen::Quests(screen)) => screen.draw(console, bindings),
            Some(Screen::Talents(screen)) => screen.draw(console, bindings),
            Some(Screen::Story(screen)) => screen.draw(console, bindings),
            Some(Screen::Messages(screen)) => screen.draw(console, bindings),
            Some(Screen::Abilities(screen)) => screen.draw(console, bindings),
            Some(Screen::Shape(screen)) => screen.draw(console, bindings),
            Some(Screen::Scribe(screen)) => screen.draw(console, bindings),
            Some(Screen::Rig(screen)) => screen.draw(console, bindings),
            Some(Screen::Profession(screen)) => screen.draw(console, bindings),
            Some(Screen::CraftPick(screen)) => screen.draw(console, bindings),
            Some(Screen::Dialogue(screen)) => screen.draw(console, bindings),
            Some(Screen::Death(screen)) => screen.draw(console, settings.clock, bindings),
            Some(Screen::LevelUp(screen)) => screen.draw(console, bindings),
            Some(Screen::Dialog(screen)) => screen.draw(console, bindings),
        }
    }

    /// Whether the death screen is open.
    pub(super) fn shows_death(&self) -> bool {
        (self.screens.iter()).any(|s| matches!(s, Screen::Death(_)))
    }

    /// Whether a dialog that must be answered is open (an overworld encounter's).
    pub(super) fn asks_required(&self) -> bool {
        (self.screens.iter()).any(|s| matches!(s, Screen::Dialog(d) if d.must_answer()))
    }
}

/// The screen stack, with what opening a screen needs.
#[derive(SystemParam)]
pub struct Screens<'w> {
    pub stack: ResMut<'w, ScreenStack>,
    content: Res<'w, GameContent>,
    store: Res<'w, SaveStore>,
    settings: Res<'w, SettingsStore>,
    graveyard: Res<'w, GraveyardStore>,
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
            craft: self.stack.craft_chosen(),
        };
        // Only the screens that show the saved characters read them (a listing of the saves).
        let characters = match name {
            ScreenName::Main | ScreenName::Characters => self.store.characters(),
            _ => Vec::new(),
        };
        let fallen = match name {
            ScreenName::Main | ScreenName::Characters | ScreenName::Graveyard => {
                graveyard::by_experience(self.graveyard.read())
            }
            _ => Vec::new(),
        };
        let launch = Launch {
            characters,
            fallen,
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

    /// Opens the story screen as the run begins ([`StoryScreen::opening`]): not page 9.
    fn open_story(&mut self) {
        let view = self.game.as_ref().map(|game| game.world.perception());
        let screen = Screen::Story(StoryScreen::opening(view.as_ref()));
        self.stack.screens.push(screen);
    }

    /// Opens the identify list for an ability used on a stack the player carries (the Identify
    /// spell or Merchant's eye, chosen in the abilities menu): choosing a stack uses it there.
    pub fn open_identify(&mut self, by: IdentifyBy) {
        let view = self.game.as_ref().map(|game| game.world.perception());
        let list = ObjectList::Identify(by);
        let screen = Screen::Objects(ObjectScreen::new(list, view.as_ref()));
        self.stack.screens.push(screen);
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

/// Shows the screens that stay open while the run changes what they show (the equipment screen
/// beneath a choice that was just done, an object list beneath a dialog) the run as it is now.
pub(super) fn refresh(game: Res<Game>, mut stack: ResMut<ScreenStack>) {
    if game.is_changed() && stack.shows_run() {
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

/// Opens the description screen of the stack the player's action this frame identified with the
/// Identify spell or the scroll of identify and learned something of (`Event::Identified` of the core).
pub(super) fn show_identified(
    game: Res<Game>,
    mut actions: MessageReader<PlayerAction>,
    mut screens: Screens,
) {
    if actions.read().count() == 0 || !screens.stack.is_empty() {
        return;
    }
    let identified = (game.last_outcome.events.iter()).find_map(|e| match e {
        warlock_core::action::Event::Identified { item } => Some(*item),
        _ => None,
    });
    if let Some(item) = identified {
        screens.open_on(ScreenName::Describe, Some(item));
    }
}

/// Opens the screens the command line asks for (see [`startup_screens`]).
pub(super) fn open_startup_screens(args: Res<Args>, game: Option<Res<Game>>, mut screens: Screens) {
    for name in startup_screens(args.screen, game.is_some()) {
        // A skipped New run has opened the race step already.
        if screens.stack.top() == Some(name) {
            continue;
        }
        // `--screen story` is the screen as the run begins, not page 9.
        if name == ScreenName::Story {
            screens.open_story();
        } else {
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
        (false, Some(s @ (Characters | Graveyard | NewRun | Settings | Controls | Dialog))) => {
            vec![Main, s]
        }
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

/// Opens the story screen once as a new run begins (spec §8 "Background story"): over its
/// first turn, before any input, when the run was made here ([`Game::story_due`]: New run's
/// Start, or a windowed start that makes a run; never Continue, Resume, `--capture`, or
/// `--replay`), nothing else is open (`--screen` opened its own), and the character has a story.
/// Nothing is saved of it: a run resumed reads it on page 9.
pub(super) fn on_run_start(game: Res<Game>, mut screens: Screens) {
    if !game.is_added() || !game.story_due || !screens.stack.is_empty() {
        return;
    }
    if game.world.death().is_none() && !game.world.perception().story().is_empty() {
        screens.open_story();
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
/// character's, the character goes into the graveyard (`graveyard.rs`), and the death screen
/// opens over the run as it ended, in place of whatever screen was open, saying so. Once.
pub(super) fn on_death(game: Res<Game>, mut screens: Screens) {
    if !game.is_changed() || game.world.death().is_none() || screens.stack.shows_death() {
        return;
    }
    if let Err(e) = screens.store.delete_character(&game.save_id) {
        warn!("{e}");
    }
    let when = crate::save::now();
    let remembered = match graveyard::Fallen::of_death(&game.world, when) {
        Some(fallen) => screens.graveyard.remember(&fallen).unwrap_or_else(|e| {
            warn!("{e}");
            false
        }),
        None => false,
    };
    screens.stack.clear();
    screens.open(ScreenName::Death);
    if let Some(Screen::Death(death)) = screens.stack.screens.last_mut() {
        death.remembered = remembered;
    }
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
                    Ok(mut game) => {
                        // The dead name its statues and gravestones (`graveyard.rs`).
                        let dead = screens.graveyard.dead(game.world.content());
                        game.world.set_graveyard(dead);
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
            // The lantern put down first, so what is picked up is lettered from a.
            Action::Drop(warlock_core::Positions::of([0])),
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
                craft: stack.craft_chosen(),
            };
            let saved = SavedCharacter {
                id: "tamsin-1".into(),
                summary: None,
            };
            let launch = Launch {
                characters: has_save.then_some(saved).into_iter().collect(),
                fallen: Vec::new(),
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
        assert_eq!(startup_screens(Some(Graveyard), false), [Main, Graveyard]);
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
    const WAIT: (KeyCode, &str) = (KeyCode::KeyS, "s");
    // Directions on the numpad, ADOM's movement keys (#350).
    const NORTH: (KeyCode, &str) = (KeyCode::Numpad8, "8");
    const EAST: (KeyCode, &str) = (KeyCode::Numpad6, "6");
    const KICK: (KeyCode, &str) = (KeyCode::KeyK, "k");
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
            .init_resource::<crate::graveyard::GraveyardStore>()
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
        // The direction prompt (Kick's, which always asks): Esc cancels it rather than pausing.
        frame(&mut app, &[KICK]);
        assert!(matches!(
            app.world().resource::<InputMode>(),
            InputMode::Direction(_)
        ));
        frame(&mut app, &[ESC]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        assert!(names(&app).is_empty());
        frame(&mut app, &[KICK, NORTH]);
        assert_eq!(actions(&app), [Action::Kick(Dir::N)]);
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
        // A lantern ready (the showcase's): `u` turns it at once, no prompt.
        let mut app = app(true);
        frame(&mut app, &[USE]);
        assert_eq!(actions(&app), [Action::UseTool]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        // Nothing ready (the quarry's start): the rules say so at once, and no prompt opens.
        let quarry = content();
        let world = warlock_core::World::new(quarry.clone(), "quarry", None).expect("run");
        let mut app = app_with(quarry, Some(Game::from_world(world, "bram-1".into())));
        frame(&mut app, &[USE]);
        assert_eq!(actions(&app), [Action::Dig(Dir::N)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        // In the quarry with the pick-axe ready, in the open: no wall beside, so the rules say so
        // at once (#1327).
        let quarry = |walk: &str| {
            let content = content();
            let mut world = warlock_core::World::new(content.clone(), "quarry", None).expect("run");
            // The pick-axe after the trades' tools (`d`).
            let walk = format!("move:E pickup equip:tool:d {walk}");
            for action in warlock_core::action::parse_actions(&walk).expect("actions") {
                world.apply(&action);
            }
            app_with(content, Some(Game::from_world(world, "tamsin-1".into())))
        };
        let mut app = quarry("");
        frame(&mut app, &[USE]);
        assert_eq!(actions(&app), [Action::Dig(Dir::N)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
        // In the gap of the pit's wall, rock north-east and south-east: `u` asks for a
        // direction, Esc cancels it, and a direction digs.
        let mut app = quarry("move:E*4");
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
        let ready = "move:E pickup equip:missile_weapon:b equip:missiles:c wait";
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
        // The next `t` starts on that creature again if it is within range (else on the
        // nearest one that is): `t`, Enter shoots it once more.
        frame(&mut app, &[fire]);
        let reach = warlock_core::fov::radius2(first.range(&view).expect("a range"));
        let again = if second.at.dist2(view.viewer()) <= reach {
            second.at
        } else {
            first.at
        };
        assert_eq!(aim(&app).map(|a| a.at), Some(again));
        frame(&mut app, &[ENTER]);
        assert_eq!(actions(&app).last(), Some(&Action::Fire(again)));
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
        stack
            .screens
            .push(Screen::Menu(MainMenu::main(None, false)));
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
        let walk = "drop:a move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
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

    /// #975: walking into what offers an action asks; the question costs nothing, a held key
    /// neither opens nor answers it, and yes uses the ability as the menu does.
    #[test]
    fn walking_into_what_offers_an_action_asks_and_a_press_answers() {
        let yes = (KeyCode::KeyY, "y");
        let held = |app: &mut App, (code, text): (KeyCode, &str)| {
            let mut press = key(code, text);
            press.repeat = true;
            app.world_mut().write_message(press);
            app.update();
        };
        for (scenario, walk, used) in [
            ("shrine", "move:N open:N move:N*3", Action::Pray),
            ("spring", "move:N*5", Action::Drink),
        ] {
            let content = content();
            let mut world = warlock_core::World::new(content.clone(), scenario, None).expect("run");
            for action in warlock_core::action::parse_actions(walk).expect("walk") {
                world.apply(&action);
            }
            let game = Game::from_world(world, "tamsin-1".into());
            let mut app = app_with(content, Some(game));

            // A held key arriving here stops, without a question or a step.
            held(&mut app, NORTH);
            assert!(names(&app).is_empty(), "{scenario}");
            // A press opens it; repeats of that key, or of an answer, leave it unanswered.
            frame(&mut app, &[NORTH]);
            assert_eq!(names(&app), [ScreenName::Dialog], "{scenario}");
            held(&mut app, NORTH);
            held(&mut app, yes);
            assert_eq!(names(&app), [ScreenName::Dialog], "{scenario}");
            // Esc leaves it, and it stays gone while the key is held.
            frame(&mut app, &[ESC]);
            held(&mut app, NORTH);
            assert!(names(&app).is_empty(), "{scenario}");
            // So does Enter, on the safe answer selected first.
            frame(&mut app, &[NORTH]);
            frame(&mut app, &[ENTER]);
            assert!(names(&app).is_empty(), "{scenario}");
            assert!(actions(&app).is_empty(), "{scenario}: no action, no time");

            frame(&mut app, &[NORTH]);
            frame(&mut app, &[yes]);
            // The ability's action.
            assert!(names(&app).is_empty(), "{scenario}");
            assert_eq!(actions(&app), std::slice::from_ref(&used), "{scenario}");
        }
    }

    /// #1387: walking into an anvil opens Metalcraft's screen at once for one trained in it (the
    /// forge's craftsperson), asking nothing and taking no time; a held key does not. Untrained
    /// (a thief), the step goes to the rules, which say what it needs.
    #[test]
    fn walking_into_an_anvil_opens_metalcraft_for_the_trained_alone() {
        let content = content();
        let metal = content.skill_id("metalcraft").expect("metalcraft");
        let walked = |spec: Option<&str>| {
            let spec: Option<warlock_core::character::CharacterSpec> =
                spec.map(|s| s.parse().expect("a character"));
            let mut world =
                warlock_core::World::with_character(content.clone(), "forge", None, spec.as_ref())
                    .expect("run");
            world.apply(&Action::Move(Dir::E));
            app_with(
                content.clone(),
                Some(Game::from_world(world, "tamsin-1".into())),
            )
        };
        let mut app = walked(None);
        let mut held = key(NORTH.0, NORTH.1);
        held.repeat = true;
        app.world_mut().write_message(held);
        app.update();
        assert!(names(&app).is_empty());
        frame(&mut app, &[NORTH]);
        assert_eq!(names(&app), [ScreenName::Profession(metal)]);
        assert!(actions(&app).is_empty(), "no action, no time");
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
        let mut app = walked(Some("human,male,thief"));
        frame(&mut app, &[NORTH]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app), [Action::Move(Dir::N)]);
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
        // `d` opens the inventory in its drop mode: Esc closes it, nothing dropped. `D` in the
        // inventory turns the mode on; Esc there goes back to the plain inventory.
        let count = actions(&app).len();
        frame(&mut app, &[d]);
        assert_eq!(names(&app), [ScreenName::Drop]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
        frame(&mut app, &[i, (KeyCode::KeyD, "D")]);
        assert_eq!(names(&app), [ScreenName::Drop]);
        frame(&mut app, &[ESC]);
        assert_eq!(names(&app), [ScreenName::Inventory]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
        assert_eq!(actions(&app).len(), count);
    }

    #[test]
    fn the_equipment_screen_chooses_what_goes_in_a_slot_and_stays_open() {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        // Into the storeroom, to pick up the chain mail shirt and the iron helm.
        let walk = "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 \
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
        let walk = "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 \
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
            &[digit(3), (KeyCode::Digit0, "0"), (KeyCode::Numpad1, "1")],
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

    /// A numpad press with Num Lock off: the physical key, the arrow (or Home, ...) it stands
    /// for, and no text (winit 0.30 on Windows and X11/Wayland).
    fn num_lock_off(code: KeyCode, logical: Key) -> KeyboardInput {
        KeyboardInput {
            logical_key: logical,
            ..key(code, "")
        }
    }

    #[test]
    fn the_numpad_navigates_screens_as_the_arrows_do() {
        const NUM_ENTER: (KeyCode, &str) = (KeyCode::NumpadEnter, "");
        const SOUTH: (KeyCode, &str) = (KeyCode::Numpad2, "2");
        // A menu and the dialog: in the pause menu 8 wraps up to Quit and NumEnter asks; in the
        // dialog 8 moves from No to Yes, and NumEnter takes it.
        let mut menu = app(true);
        frame(&mut menu, &[ESC, NORTH, NUM_ENTER]);
        assert_eq!(names(&menu), [ScreenName::Pause, ScreenName::Dialog]);
        frame(&mut menu, &[NORTH, NUM_ENTER]);
        assert_eq!(saves_sent(&menu), [SaveCommand::SaveAndQuit]);
        // Info pages: the numpad's digits never turn a page, Num Lock on or off.
        let mut pages = app(true);
        frame(&mut pages, &[(KeyCode::KeyI, "I")]);
        assert_eq!(names(&pages), [ScreenName::Inventory]);
        frame(&mut pages, &[SOUTH, (KeyCode::Numpad3, "3"), NORTH]);
        pages
            .world_mut()
            .write_message(num_lock_off(KeyCode::Numpad3, Key::PageDown));
        pages.update();
        assert_eq!(names(&pages), [ScreenName::Inventory]);
        assert!(actions(&pages).is_empty());
        // The look cursor takes the numpad's diagonals, Num Lock on or off.
        let mut look = app(true);
        frame(&mut look, &[(KeyCode::KeyL, "l")]);
        let InputMode::Look(at) = mode(&look) else {
            panic!("looking")
        };
        frame(&mut look, &[(KeyCode::Numpad3, "3")]);
        look.world_mut()
            .write_message(num_lock_off(KeyCode::Numpad3, Key::PageDown));
        look.update();
        assert_eq!(mode(&look), InputMode::Look(at.step(Dir::SE).step(Dir::SE)));
        // A paged list: 2 moves down to the pile's second stack, NumEnter picks it up.
        let shared = content();
        let game = Game::from_world(on_the_pile(shared.clone()), "tamsin-1".into());
        let mut list = app_with(shared, Some(game));
        frame(&mut list, &[(KeyCode::Comma, ",")]);
        list.world_mut()
            .write_message(num_lock_off(KeyCode::Numpad2, Key::ArrowDown));
        list.update();
        frame(&mut list, &[NUM_ENTER]);
        assert_eq!(
            actions(&list),
            [Action::PickUp(warlock_core::Positions::of([1]))]
        );
        // A text field: the numpad's digits type, and NumEnter accepts. (Seven Enters reach the
        // Name step of New run.)
        let mut field = app(false);
        field.update();
        frame(&mut field, &[ENTER; 7]);
        assert_eq!(names(&field).last(), Some(&ScreenName::Name));
        frame(&mut field, &[(KeyCode::KeyE, "E"), NORTH, SOUTH, NUM_ENTER]);
        assert_eq!(names(&field).last(), Some(&ScreenName::Summary));
        let name = (field.world().resource::<ScreenStack>().draft()).name(&content());
        assert_eq!(name, "E82");
    }

    #[test]
    fn the_weapon_skills_screen_opens_on_w_and_a_bash_with_no_foe_is_refused_at_once() {
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
        // With a shield in hand and no foe beside, the same, toward the storeroom's wall north
        // (#1327).
        let mut app = with_a_shield();
        bind_b(&mut app);
        frame(&mut app, &[(KeyCode::KeyB, "B")]);
        assert_eq!(actions(&app), [Action::Bash(Dir::N)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
    }

    #[test]
    fn a_disarm_key_with_no_known_trap_beside_is_refused_at_once() {
        // Disarming is an ability: unbound by default. Bound (here to `Y`), with no trap known
        // beside the showcase's thief, the rules say so at once, with no prompt (#1327).
        let mut app = app(true);
        {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let disarm = crate::input::KeyCommand::Disarm;
            assert!(bindings.keys_of(disarm).is_empty());
            let key = crate::input::KeyBinding::from_text("Y").expect("a key");
            bindings.0.push((key, disarm));
        }
        frame(&mut app, &[(KeyCode::KeyY, "Y")]);
        assert_eq!(actions(&app), [Action::Disarm(Dir::N)]);
        assert_eq!(app.world().resource::<InputMode>(), &InputMode::Normal);
    }

    /// A run of the showcase's thief after `walk`.
    fn thief_after(walk: &str) -> App {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "showcase", None).expect("run");
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        app_with(content, Some(Game::from_world(world, "tamsin-1".into())))
    }

    /// The ways to the creatures the player sees beside them, in compass order.
    fn ways_to_creatures(app: &App) -> Vec<Dir> {
        let game = app.world().resource::<Game>();
        let view = game.world.perception();
        let ways = view.ways(warlock_core::beside::Beside::Chat);
        ways.expect("chat is aimed beside")
    }

    #[test]
    fn picking_a_lock_acts_at_once_on_the_one_door_beside() {
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // Below the hall's door, the showcase's thief (trained in Open lock): `a` `e` (Pick
        // lock, after their birth sign's stare, Exploit, Disarm, and Listen) picks it at
        // once, with no prompt (#1327).
        let mut app = thief_after("move:N*3");
        frame(&mut app, &[(KeyCode::KeyA, "a"), (KeyCode::KeyE, "e")]);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::Normal);
        assert_eq!(actions(&app), [Action::PickLock(Dir::N)]);
        // Its own key is unbound by default; bound (here to `B`), it does the same.
        let mut app = thief_after("move:N*3");
        {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let pick = crate::input::KeyCommand::PickLock;
            assert!(bindings.keys_of(pick).is_empty());
            let key = crate::input::KeyBinding::from_text("B").expect("a key");
            bindings.0.push((key, pick));
        }
        frame(&mut app, &[(KeyCode::KeyB, "B")]);
        assert_eq!(actions(&app), [Action::PickLock(Dir::N)]);
        assert_eq!(mode(&app), InputMode::Normal);
    }

    #[test]
    fn opening_and_closing_act_at_once_on_the_one_door_beside_and_kick_always_asks() {
        const OPEN: (KeyCode, &str) = (KeyCode::KeyO, "o");
        const CLOSE: (KeyCode, &str) = (KeyCode::KeyC, "c");
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // Nothing beside: the rules refuse at once, toward the first way (no wall beside).
        let mut app = app(true);
        frame(&mut app, &[OPEN]);
        assert_eq!(actions(&app), [Action::Open(Dir::N)]);
        assert_eq!(mode(&app), InputMode::Normal);
        // One door beside: opened at once; then closed at once.
        let mut app = thief_after("move:N*3");
        frame(&mut app, &[OPEN]);
        assert_eq!(actions(&app), [Action::Open(Dir::N)]);
        assert_eq!(mode(&app), InputMode::Normal);
        let mut app = thief_after("move:N*3 open:N");
        frame(&mut app, &[CLOSE]);
        assert_eq!(actions(&app), [Action::Close(Dir::N)]);
        // Kick asks whatever is beside.
        frame(&mut app, &[KICK]);
        assert!(matches!(mode(&app), InputMode::Direction(_)));
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app).last(), Some(&Action::Kick(Dir::E)));
    }

    #[test]
    fn a_professions_row_opens_its_recipes_and_esc_closes_them() {
        // The forge's craftsperson's `a` `b` is Metalcraft… (after the birth sign's stare): its
        // screen, over the menu's place.
        let content = content();
        let metal = content.skill_id("metalcraft").expect("metalcraft");
        let world = warlock_core::World::new(content.clone(), "forge", None).expect("run");
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        frame(&mut app, &[(KeyCode::KeyA, "a"), (KeyCode::KeyB, "b")]);
        assert_eq!(names(&app), [ScreenName::Profession(metal)]);
        assert!(actions(&app).is_empty());
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
    }

    #[test]
    fn stealing_acts_at_once_on_one_beside_and_asks_among_several() {
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // The showcase's thief can steal; an ally beside is someone to rob: `a` `e` (Steal)
        // steals from it at once.
        let mut app = thief_after("ally:panther");
        let [one] = ways_to_creatures(&app)[..] else {
            panic!("one ally beside");
        };
        frame(&mut app, &[(KeyCode::KeyA, "a"), (KeyCode::KeyE, "e")]);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::Normal);
        assert_eq!(actions(&app), [Action::Steal(one)]);
        // Two beside: it asks for a direction in the message line, and a direction steals.
        let mut app = thief_after("ally:panther:2");
        assert_eq!(ways_to_creatures(&app).len(), 2);
        frame(&mut app, &[(KeyCode::KeyA, "a"), (KeyCode::KeyE, "e")]);
        assert_eq!(mode(&app), InputMode::Ability(Ability::Steal));
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Steal(Dir::E)]);
        // Unbound by default; bound (here to `Y`), it asks the same.
        let mut app = thief_after("ally:panther:2");
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
        // Nobody beside: refused at once, no prompt.
        let mut app = self::app(true);
        {
            let mut bindings = app.world_mut().resource_mut::<Bindings>();
            let key = crate::input::KeyBinding::from_text("Y").expect("a key");
            bindings.0.push((key, crate::input::KeyCommand::Steal));
        }
        frame(&mut app, &[(KeyCode::KeyY, "Y")]);
        assert_eq!(actions(&app), [Action::Steal(Dir::N)]);
        assert_eq!(mode(&app), InputMode::Normal);
    }

    /// A run of the showcase's thief with the storeroom's small shield in hand.
    fn with_a_shield() -> App {
        thief_after(
            "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
             pickup:a equip:off_hand:b",
        )
    }

    #[test]
    fn the_abilities_menu_uses_an_ability_and_remembers_it_for_the_quick_repeat() {
        const ABILITIES: (KeyCode, &str) = (KeyCode::KeyA, "a");
        const REPEAT: (KeyCode, &str) = (KeyCode::KeyX, "X");
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // Without a shield: the menu opens on `a`; the shield bash (`g`) is greyed out, so its
        // letter only says why; Esc closes the menu, and no time passed.
        let mut app = app(true);
        frame(&mut app, &[ABILITIES]);
        assert_eq!(names(&app), [ScreenName::Abilities]);
        frame(&mut app, &[(KeyCode::KeyG, "g")]);
        assert_eq!(names(&app), [ScreenName::Abilities]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty() && actions(&app).is_empty());
        // The quick repeat with nothing used yet opens the menu.
        frame(&mut app, &[REPEAT]);
        assert_eq!(names(&app), [ScreenName::Abilities]);
        // Two allies beside: `a` `e` closes the menu and asks for a direction; a direction
        // steals.
        let mut app = thief_after("ally:panther:2");
        frame(&mut app, &[ABILITIES, (KeyCode::KeyE, "e")]);
        assert!(names(&app).is_empty());
        assert_eq!(mode(&app), InputMode::Ability(Ability::Steal));
        frame(&mut app, &[EAST]);
        assert_eq!(actions(&app), [Action::Steal(Dir::E)]);
        let stack = app.world().resource::<ScreenStack>();
        assert_eq!(stack.last_ability(), Some(Ability::Steal));
        // `X` uses it again; Esc cancels the direction prompt rather than pausing.
        frame(&mut app, &[REPEAT]);
        assert_eq!(mode(&app), InputMode::Ability(Ability::Steal));
        frame(&mut app, &[ESC]);
        assert_eq!(mode(&app), InputMode::Normal);
        assert!(names(&app).is_empty());
        // The menu opens on it, so Enter repeats it too.
        frame(&mut app, &[ABILITIES, ENTER, WEST]);
        assert_eq!(
            actions(&app),
            [Action::Steal(Dir::E), Action::Steal(Dir::W)]
        );
    }

    /// A run of a bard of level 16, who throws their voice (#244), with a key bound for it (`B`:
    /// unbound by default).
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
        let command = crate::input::KeyCommand::ThrowVoice;
        assert!(bindings.keys_of(command).is_empty(), "unbound by default");
        let key = crate::input::KeyBinding::from_text("B").expect("a key");
        bindings.0.push((key, command));
        app
    }

    #[test]
    fn a_thrown_voice_is_aimed_at_a_cell() {
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
    }

    #[test]
    fn single_stacks_and_empty_hands_need_no_screen() {
        use warlock_core::Positions;
        // The quarry's start, the trades' tools dropped and left behind: the player carries
        // nothing (no lantern either).
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "quarry", None).expect("run");
        for action in warlock_core::action::parse_actions("drop move:N").expect("actions") {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world, "bram-1".into())));
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
        world.apply(&Action::UseTool);
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
            .init_resource::<crate::graveyard::GraveyardStore>()
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
    fn identifying_opens_the_description_of_the_stack_it_told_about() {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "traps", None).expect("run");
        for action in warlock_core::action::parse_actions("move:E pickup").expect("actions") {
            world.apply(&action);
        }
        let mut app = app_with(content, Some(Game::from_world(world, "tamsin-1".into())));
        fn apply(mut actions: MessageReader<PlayerAction>, mut game: ResMut<Game>) {
            for PlayerAction(action) in actions.read() {
                game.apply(action);
            }
        }
        app.add_systems(
            Update,
            (apply, show_identified).chain().after(count_actions),
        );
        // Read the scroll of identify: its list of the unidentified stacks.
        frame(&mut app, &[(KeyCode::KeyR, "r"), (KeyCode::KeyD, "d")]);
        assert_eq!(names(&app), [ScreenName::Identify]);
        // Choosing the first stack identifies it and opens its description over the map.
        frame(&mut app, &[(KeyCode::KeyA, "a")]);
        assert!(matches!(actions(&app).last(), Some(Action::Read { .. })));
        assert_eq!(names(&app), [ScreenName::Describe]);
        // Esc returns to the map.
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
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
        app.add_systems(
            Update,
            (apply, converse, show_identified)
                .chain()
                .after(count_actions),
        );
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
        // His quest comes first: asking gives it, and the screen reopens with it as a topic
        // still (now the reminder).
        frame(&mut app, &[(KeyCode::Digit1, "1")]);
        let topics = |app: &App| {
            let game = app.world().resource::<Game>();
            let talk = last_talk(&game.last_outcome).expect("talk");
            (talk.topics.iter().map(|t| t.name.clone()).take(2)).collect::<Vec<_>>()
        };
        let given = Action::Ask {
            dir: Dir::SE,
            topic: "quest-rats".into(),
        };
        assert_eq!(actions(&app).last(), Some(&given));
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
    fn chat_refuses_with_no_one_beside_acts_on_one_and_asks_among_several() {
        const CHAT: (KeyCode, &str) = (KeyCode::KeyC, "C");
        let mode = |app: &App| *app.world().resource::<InputMode>();
        // No one beside: the rules say so at once (#1327).
        let mut app = app(true);
        frame(&mut app, &[CHAT]);
        assert_eq!(actions(&app), [Action::Chat(Dir::N)]);
        assert_eq!(mode(&app), InputMode::Normal);
        // One: talked to at once.
        let mut app = thief_after("ally:panther");
        let [one] = ways_to_creatures(&app)[..] else {
            panic!("one ally beside");
        };
        frame(&mut app, &[CHAT]);
        assert_eq!(actions(&app), [Action::Chat(one)]);
        // Two: which way is asked.
        let mut app = thief_after("ally:panther:2");
        frame(&mut app, &[CHAT]);
        assert_eq!(
            mode(&app),
            InputMode::Direction(crate::input::KeyCommand::Chat)
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

    /// An app in a run of `game` with the system that tells the story as a run begins.
    fn story_app(game: Game) -> App {
        let mut app = app_with(content(), Some(game));
        app.add_systems(Update, on_run_start.after(apply_choice));
        app
    }

    /// A kitted dwarf warrior's run, so with a story.
    fn kitted() -> warlock_core::World {
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        warlock_core::World::start_run(content(), "showcase", Some(3), Some(&spec), None, true)
            .expect("run")
    }

    #[test]
    fn the_story_opens_once_as_a_new_run_begins() {
        let args = Args::parse_from(["warlock", "--capture", "unused.png"]);
        let store = SaveStore::from_args(&args);
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        let game = Game::new_run(content(), &store, "showcase", None, Some(&spec), None);
        let mut app = story_app(game.expect("starts"));
        frame(&mut app, &[]);
        assert_eq!(names(&app), [ScreenName::Story]);
        // The page keys do not turn it: it is no page yet.
        frame(&mut app, &[(KeyCode::Digit1, "1")]);
        assert_eq!(names(&app), [ScreenName::Story]);
        frame(&mut app, &[ENTER]);
        assert!(names(&app).is_empty());
        for _ in 0..3 {
            frame(&mut app, &[]);
        }
        assert!(names(&app).is_empty(), "once");
        // Page 9 reads it again, from another page.
        frame(&mut app, &[(KeyCode::KeyI, "i"), (KeyCode::Digit9, "9")]);
        assert_eq!(names(&app), [ScreenName::Story]);
        frame(&mut app, &[ESC]);
        assert!(names(&app).is_empty());
    }

    #[test]
    fn a_resumed_run_or_one_without_a_story_opens_nothing() {
        let resumed = Game::from_world(kitted(), "dagny-1".into());
        let mut app = story_app(resumed);
        frame(&mut app, &[]);
        assert!(names(&app).is_empty(), "never on Continue or Resume");
        let args = Args::parse_from(["warlock", "--capture", "unused.png"]);
        let store = SaveStore::from_args(&args);
        // Content that tells no stories: a chosen character with the kit, and still none.
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.remove("data/background.toml");
        let untold = Arc::new(Content::load(&files).expect("content without stories"));
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        let storyless = Game::new_run(untold, &store, "showcase", None, Some(&spec), None);
        let storyless = storyless.expect("starts");
        assert!(storyless.world.perception().story().is_empty());
        let mut app = story_app(storyless);
        frame(&mut app, &[]);
        assert!(names(&app).is_empty(), "nothing to tell");
    }

    /// An app in the arena while its fight goes on (ten turns in, everything said so far read),
    /// applying actions and waiting at `--more--` like the game, its console 80 by 24.
    fn arena_app() -> App {
        let content = content();
        let mut world = warlock_core::World::new(content.clone(), "arena", None).expect("run");
        for _ in 0..6 {
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
        while mode(&app) == InputMode::Normal && actions(&app).len() < 30 {
            frame(&mut app, &[WAIT]);
        }
        let waited = actions(&app).len();
        assert_eq!(mode(&app), InputMode::More(0));
        let stops = {
            let (game, bindings) = (app.world().resource::<Game>(), Bindings::default());
            super::super::more::stops(game, 80, &bindings)
        };
        assert!(stops >= 1, "{stops} stops");
        // Other keys do nothing: no move, no wait, no screen.
        frame(
            &mut app,
            &[(KeyCode::Numpad2, "2"), WAIT, HELP, (KeyCode::KeyV, "v")],
        );
        assert_eq!(actions(&app).len(), waited);
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
        for page in 0..stops {
            assert_eq!(mode(&app), InputMode::More(page));
            frame(&mut app, &[SPACE]);
        }
        assert_eq!(mode(&app), InputMode::Normal);
        // Play goes on, until a turn says too much again; Esc skips the rest of its messages
        // without opening the pause menu.
        frame(&mut app, &[WAIT]);
        assert_eq!(actions(&app).len(), waited + 1);
        while mode(&app) == InputMode::Normal && actions(&app).len() < 60 {
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
