//! What every modal screen does for the screen stack (#1542): its name, its layout before a
//! press, a press, and its drawing, through one trait, so the stack dispatches once
//! (`screens::Screen::modal`) instead of a match per question. A new screen is one
//! `ModalScreen` impl here (most are one `modal!` line).

use super::abilities_screen::AbilitiesScreen;
use super::character_screen::CharacterScreen;
use super::characters_screen::CharactersScreen;
use super::chargen_screen::{ChoiceScreen, SummaryScreen};
use super::chargen_talents_screen::ChargenTalentsScreen;
use super::controls_screen::ControlsScreen;
use super::craft_pick_screen::CraftPickScreen;
use super::death_screen::DeathScreen;
use super::describe_screen::DescribeScreen;
use super::dialog_screen::DialogScreen;
use super::dialogue_screen::DialogueScreen;
use super::equipment_screen::{EquipmentScreen, ItemScreen};
use super::files_screen::FilesScreen;
use super::graveyard_screen::GraveyardScreen;
use super::level_up_screen::LevelUpScreen;
use super::magic_screen::MagicScreen;
use super::menu::{MainMenu, Press};
use super::messages_screen::MessagesScreen;
use super::name_screen::NameScreen;
use super::new_run_screen::NewRunScreen;
use super::object_screen::ObjectScreen;
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
use super::weapon_skills_screen::WeaponSkillsScreen;
use super::{ScreenAction, ScreenName};
use crate::input::Bindings;
use crate::render::console::Console;
use crate::settings::{Clock, Settings};

/// What a press may change besides the screen: the settings and the key bindings.
pub(super) struct Keys<'a> {
    pub settings: &'a mut Settings,
    pub bindings: &'a mut Bindings,
}

/// What drawing may read besides the screen.
pub(super) struct Show<'a> {
    pub settings: &'a Settings,
    pub bindings: &'a Bindings,
    pub clock: Clock,
    /// What the main or pause menu tells the player (`ScreenStack::notice`).
    pub notice: Option<&'a str>,
}

/// A modal screen on the stack.
pub(super) trait ModalScreen {
    /// The screen it is (`--screen`, what the stack asks of its screens).
    fn name(&self) -> ScreenName;
    /// Lays it out for a console of `cols` by `rows` before a press, so a letter names what is
    /// drawn; most need nothing.
    fn fit(&self, _cols: u32, _rows: u32) {}
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction;
    fn draw(&self, console: &mut Console, show: &Show<'_>);
}

impl<T: ModalScreen> ModalScreen for Box<T> {
    fn name(&self) -> ScreenName {
        (**self).name()
    }
    fn fit(&self, cols: u32, rows: u32) {
        (**self).fit(cols, rows)
    }
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction {
        (**self).handle(press, keys)
    }
    fn draw(&self, console: &mut Console, show: &Show<'_>) {
        (**self).draw(console, show)
    }
}

/// `modal!(Type => Name, handle(bindings?), draw(args...), fit?)`: a screen of one name whose
/// own `handle` takes the press (and the bindings) and whose own `draw` takes the console and
/// the named parts of [`Show`]; `fit` when it lays out before a press.
macro_rules! modal {
    ($ty:ty => $name:ident, handle($($h:ident)?), draw($($d:ident),*) $(, $fit:ident)?) => {
        impl ModalScreen for $ty {
            fn name(&self) -> ScreenName {
                ScreenName::$name
            }
            $(fn $fit(&self, cols: u32, rows: u32) {
                <$ty>::fit(self, cols, rows)
            })?
            fn handle(&mut self, press: &Press, _keys: &mut Keys<'_>) -> ScreenAction {
                <$ty>::handle(self, press $(, &*_keys.$h)?)
            }
            fn draw(&self, console: &mut Console, _show: &Show<'_>) {
                <$ty>::draw(self, console $(, _show.$d)*)
            }
        }
    };
}

modal!(CharactersScreen => Characters, handle(), draw(clock));
modal!(GraveyardScreen => Graveyard, handle(), draw(clock));
modal!(NewRunScreen => NewRun, handle(), draw());
modal!(ChargenTalentsScreen => ChargenTalents, handle(), draw(), fit);
modal!(SignScreen => Sign, handle(), draw());
modal!(NameScreen => Name, handle(), draw());
modal!(SummaryScreen => Summary, handle(), draw());
modal!(FilesScreen => Files, handle(), draw());
modal!(EquipmentScreen => Equipment, handle(bindings), draw());
modal!(ItemScreen => EquipItem, handle(bindings), draw(), fit);
modal!(DescribeScreen => Describe, handle(bindings), draw(bindings));
modal!(CharacterScreen => Character, handle(bindings), draw(bindings));
modal!(SkillsScreen => Skills, handle(bindings), draw(bindings));
modal!(WeaponSkillsScreen => WeaponSkills, handle(bindings), draw(bindings));
modal!(MagicScreen => Magic, handle(), draw());
modal!(QuestScreen => Quests, handle(bindings), draw(bindings));
modal!(TalentsScreen => Talents, handle(bindings), draw(bindings));
modal!(StoryScreen => Story, handle(), draw(), fit);
modal!(MessagesScreen => Messages, handle(bindings), draw(bindings), fit);
modal!(AbilitiesScreen => Abilities, handle(bindings), draw(bindings), fit);
modal!(ShapeScreen => Shape, handle(bindings), draw(), fit);
modal!(ScribeScreen => Scribe, handle(bindings), draw(), fit);
modal!(RigScreen => Rig, handle(bindings), draw(), fit);
modal!(DialogueScreen => Dialogue, handle(bindings), draw());
modal!(DeathScreen => Death, handle(), draw(clock));
modal!(LevelUpScreen => LevelUp, handle(), draw(), fit);
modal!(DialogScreen => Dialog, handle(), draw());

/// The main menu, or the pause menu over a run; it shows the stack's notice.
impl ModalScreen for MainMenu {
    fn name(&self) -> ScreenName {
        match self.is_pause() {
            true => ScreenName::Pause,
            false => ScreenName::Main,
        }
    }
    fn handle(&mut self, press: &Press, _keys: &mut Keys<'_>) -> ScreenAction {
        MainMenu::handle(self, press)
    }
    fn draw(&self, console: &mut Console, show: &Show<'_>) {
        MainMenu::draw(self, console, show.notice, show.settings.temperature)
    }
}

/// A step of character creation, named by its step.
impl ModalScreen for ChoiceScreen {
    fn name(&self) -> ScreenName {
        self.step().screen()
    }
    fn handle(&mut self, press: &Press, _keys: &mut Keys<'_>) -> ScreenAction {
        ChoiceScreen::handle(self, press)
    }
    fn draw(&self, console: &mut Console, _show: &Show<'_>) {
        ChoiceScreen::draw(self, console)
    }
}

/// Settings change in place.
impl ModalScreen for SettingsScreen {
    fn name(&self) -> ScreenName {
        ScreenName::Settings
    }
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction {
        SettingsScreen::handle(self, press, keys.settings)
    }
    fn draw(&self, console: &mut Console, show: &Show<'_>) {
        SettingsScreen::draw(self, console, show.settings)
    }
}

/// The controls, or the read-only `?` help: bindings change in place, and it says only whether
/// it was left.
impl ModalScreen for ControlsScreen {
    fn name(&self) -> ScreenName {
        match self.is_help() {
            true => ScreenName::Help,
            false => ScreenName::Controls,
        }
    }
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction {
        match ControlsScreen::handle(self, press, keys.bindings) {
            true => ScreenAction::Close,
            false => ScreenAction::Stay,
        }
    }
    fn draw(&self, console: &mut Console, show: &Show<'_>) {
        ControlsScreen::draw(self, console, show.bindings)
    }
}

/// An item list, named by its list ([`ObjectScreen::name`]).
impl ModalScreen for ObjectScreen {
    fn name(&self) -> ScreenName {
        ObjectScreen::name(self)
    }
    fn fit(&self, cols: u32, rows: u32) {
        ObjectScreen::fit(self, cols, rows)
    }
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction {
        ObjectScreen::handle(self, press, keys.bindings)
    }
    fn draw(&self, console: &mut Console, show: &Show<'_>) {
        ObjectScreen::draw(self, console, show.bindings)
    }
}

/// A profession's recipes; `--screen profession` names the first.
impl ModalScreen for ProfessionScreen {
    fn name(&self) -> ScreenName {
        match self.skill() {
            Some(skill) => ScreenName::Profession(skill),
            None => ScreenName::FirstProfession,
        }
    }
    fn fit(&self, cols: u32, rows: u32) {
        ProfessionScreen::fit(self, cols, rows)
    }
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction {
        ProfessionScreen::handle(self, press, keys.bindings)
    }
    fn draw(&self, console: &mut Console, _show: &Show<'_>) {
        ProfessionScreen::draw(self, console)
    }
}

/// The forging wizard; `--screen craft-pick` names the first recipe's.
impl ModalScreen for CraftPickScreen {
    fn name(&self) -> ScreenName {
        match self.chosen() {
            Some((recipe, _)) => ScreenName::CraftPick(recipe),
            None => ScreenName::FirstCraftPick,
        }
    }
    fn fit(&self, cols: u32, rows: u32) {
        CraftPickScreen::fit(self, cols, rows)
    }
    fn handle(&mut self, press: &Press, keys: &mut Keys<'_>) -> ScreenAction {
        CraftPickScreen::handle(self, press, keys.bindings)
    }
    fn draw(&self, console: &mut Console, _show: &Show<'_>) {
        CraftPickScreen::draw(self, console)
    }
}
