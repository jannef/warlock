//! The main menu, drawn in glyphs on the UI console (spec §6): Continue (when a save exists),
//! New run (scenario list and seed), Settings, Controls, Quit.
//!
//! The menu exists while no [`Game`](crate::game::Game) does. [`Menu`] is a plain state machine:
//! it takes [`Press`]es and answers with what to do next ([`Start`]), so it is testable without
//! a window. Selection is always shown by text (a `>` marker), never by color alone.

use super::controls_screen::ControlsScreen;
use super::settings_screen::SettingsScreen;
use super::{ACCENT, PANEL_BG, TEXT, TEXT_DIM};
use crate::game::{Game, GameContent};
use crate::input::{Bindings, KeyBinding};
use crate::render::ConsoleLayout;
use crate::render::console::Console;
use crate::save::SaveStore;
use crate::settings::Settings;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use clap::ValueEnum;
use warlock_core::content::Content;

/// Screens the menu can open on (`--screen`, a dev flag for screenshots).
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ScreenName {
    Main,
    NewRun,
    Settings,
    Controls,
}

/// A key press reduced to what menus care about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Backspace,
    Delete,
    Digit(char),
    Other,
}

/// One key press: its menu meaning, and the binding it would make (for the rebinding screen).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Press {
    pub nav: Nav,
    pub binding: Option<KeyBinding>,
    /// A held key repeating.
    pub repeat: bool,
}

impl Press {
    pub fn from_input(input: &KeyboardInput) -> Self {
        let nav = match input.key_code {
            KeyCode::ArrowUp => Nav::Up,
            KeyCode::ArrowDown => Nav::Down,
            KeyCode::ArrowLeft => Nav::Left,
            KeyCode::ArrowRight => Nav::Right,
            KeyCode::Enter | KeyCode::NumpadEnter => Nav::Enter,
            KeyCode::Escape => Nav::Back,
            KeyCode::Backspace => Nav::Backspace,
            KeyCode::Delete => Nav::Delete,
            _ => match &input.logical_key {
                Key::Character(c) => match c.as_str() {
                    "k" => Nav::Up,
                    "j" => Nav::Down,
                    "h" => Nav::Left,
                    "l" => Nav::Right,
                    d => match d.chars().next() {
                        Some(ch) if ch.is_ascii_digit() && d.len() == 1 => Nav::Digit(ch),
                        _ => Nav::Other,
                    },
                },
                _ => Nav::Other,
            },
        };
        Self {
            nav,
            binding: KeyBinding::from_input(input),
            repeat: input.repeat,
        }
    }

    /// A press of a navigation key.
    #[cfg(test)]
    pub fn nav(nav: Nav) -> Self {
        Self {
            nav,
            binding: None,
            repeat: false,
        }
    }
}

/// What the menu asks the app to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Start {
    Continue,
    NewRun { scenario: String, seed: Option<u64> },
    Quit,
}

/// A scenario as listed in the New run screen.
#[derive(Clone, Debug)]
struct ScenarioInfo {
    id: String,
    name: String,
    description: String,
    seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainItem {
    Continue,
    NewRun,
    Settings,
    Controls,
    Quit,
}

impl MainItem {
    fn label(self) -> &'static str {
        match self {
            MainItem::Continue => "Continue",
            MainItem::NewRun => "New run",
            MainItem::Settings => "Settings",
            MainItem::Controls => "Controls",
            MainItem::Quit => "Quit",
        }
    }
}

#[derive(Debug)]
enum Screen {
    /// The selected row.
    Main(usize),
    NewRun(NewRun),
    Settings(SettingsScreen),
    Controls(ControlsScreen),
}

/// The New run screen: a scenario list and a seed field.
#[derive(Debug, Default)]
struct NewRun {
    selected: usize,
    seed: String,
}

/// Longest seed we accept: fits a `u64` with room to spare.
const MAX_SEED_DIGITS: usize = 18;

impl NewRun {
    fn seed(&self) -> Option<u64> {
        self.seed.parse().ok()
    }
}

/// The main menu and the screens reached from it.
#[derive(Resource, Debug)]
pub struct Menu {
    screen: Screen,
    scenarios: Vec<ScenarioInfo>,
    has_save: bool,
    /// Something the player should know, e.g. why Continue failed.
    notice: Option<String>,
}

impl Menu {
    pub fn new(content: &Content, has_save: bool, open: Option<ScreenName>) -> Self {
        let scenarios = content
            .scenarios()
            .map(|s| ScenarioInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                seed: s.seed,
            })
            .collect();
        let mut menu = Self {
            screen: Screen::Main(0),
            scenarios,
            has_save,
            notice: None,
        };
        menu.screen = match open {
            None | Some(ScreenName::Main) => Screen::Main(0),
            Some(ScreenName::NewRun) => Screen::NewRun(NewRun::default()),
            Some(ScreenName::Settings) => Screen::Settings(SettingsScreen::default()),
            Some(ScreenName::Controls) => Screen::Controls(ControlsScreen::default()),
        };
        menu
    }

    fn items(&self) -> Vec<MainItem> {
        let mut items = Vec::new();
        if self.has_save {
            items.push(MainItem::Continue);
        }
        items.extend([
            MainItem::NewRun,
            MainItem::Settings,
            MainItem::Controls,
            MainItem::Quit,
        ]);
        items
    }

    /// Shows why a run could not be started and drops Continue if there is nothing to resume.
    pub fn fail(&mut self, message: String, save_unusable: bool) {
        self.notice = Some(message);
        if save_unusable {
            self.has_save = false;
        }
        self.screen = Screen::Main(0);
    }

    /// Applies one key press. Settings and key bindings change in place; the return value says
    /// whether to leave the menu.
    pub fn handle(
        &mut self,
        press: &Press,
        settings: &mut Settings,
        bindings: &mut Bindings,
    ) -> Option<Start> {
        let items = self.items();
        let scenario_count = self.scenarios.len();
        match &mut self.screen {
            Screen::Main(selected) => {
                *selected = (*selected).min(items.len() - 1);
                match press.nav {
                    Nav::Up => *selected = (*selected + items.len() - 1) % items.len(),
                    Nav::Down => *selected = (*selected + 1) % items.len(),
                    Nav::Enter => {
                        self.notice = None;
                        match items[*selected] {
                            MainItem::Continue => return Some(Start::Continue),
                            MainItem::NewRun => self.screen = Screen::NewRun(NewRun::default()),
                            MainItem::Settings => {
                                self.screen = Screen::Settings(SettingsScreen::default());
                            }
                            MainItem::Controls => {
                                self.screen = Screen::Controls(ControlsScreen::default());
                            }
                            MainItem::Quit => return Some(Start::Quit),
                        }
                    }
                    _ => {}
                }
            }
            Screen::NewRun(new_run) => match press.nav {
                Nav::Back => self.screen = Screen::Main(self.back_row(MainItem::NewRun)),
                Nav::Up if scenario_count > 0 => {
                    new_run.selected = (new_run.selected + scenario_count - 1) % scenario_count;
                }
                Nav::Down if scenario_count > 0 => {
                    new_run.selected = (new_run.selected + 1) % scenario_count;
                }
                Nav::Digit(d) if new_run.seed.len() < MAX_SEED_DIGITS => new_run.seed.push(d),
                Nav::Backspace | Nav::Delete => {
                    new_run.seed.pop();
                }
                Nav::Enter => {
                    if let Some(info) = self.scenarios.get(new_run.selected) {
                        return Some(Start::NewRun {
                            scenario: info.id.clone(),
                            seed: new_run.seed(),
                        });
                    }
                }
                _ => {}
            },
            Screen::Settings(screen) => {
                if screen.handle(press, settings) {
                    self.screen = Screen::Main(self.back_row(MainItem::Settings));
                }
            }
            Screen::Controls(screen) => {
                if screen.handle(press, bindings) {
                    self.screen = Screen::Main(self.back_row(MainItem::Controls));
                }
            }
        }
        None
    }

    /// The main menu row of `item`, for returning to it.
    fn back_row(&self, item: MainItem) -> usize {
        self.items().iter().position(|i| *i == item).unwrap_or(0)
    }

    pub fn draw(&self, console: &mut Console, settings: &Settings, bindings: &Bindings) {
        console.clear();
        match &self.screen {
            Screen::Main(selected) => self.draw_main(console, *selected),
            Screen::NewRun(new_run) => self.draw_new_run(console, new_run),
            Screen::Settings(screen) => screen.draw(console, settings),
            Screen::Controls(screen) => screen.draw(console, bindings),
        }
    }

    fn draw_main(&self, console: &mut Console, selected: usize) {
        let items = self.items();
        let notice = self.notice.as_deref();
        let notice_lines = notice.map_or(Vec::new(), |n| wrap(n, 44));
        let h = 12 + items.len() as i32 + notice_lines.len() as i32;
        let panel = Panel::new(console, "", 52, h);
        let cx = |text: &str| panel.x + (panel.w - text.chars().count() as i32) / 2;
        let title = "W A R L O C K";
        console.print(cx(title), panel.y + 2, title, ACCENT);
        let rule = "═".repeat(title.chars().count());
        console.print(cx(&rule), panel.y + 3, &rule, TEXT_DIM);
        let tagline = "an ASCII roguelike";
        console.print(cx(tagline), panel.y + 4, tagline, TEXT_DIM);
        for (i, item) in items.iter().enumerate() {
            let y = panel.y + 7 + i as i32;
            panel.row(console, y, i == selected, item.label(), "", 0);
        }
        let notice_y = panel.y + 8 + items.len() as i32;
        for (i, line) in notice_lines.iter().enumerate() {
            console.print(
                panel.inner_x(),
                notice_y + i as i32,
                &format!("! {line}"),
                ACCENT,
            );
        }
        panel.footer(console, "↑↓ select   Enter choose");
    }

    fn draw_new_run(&self, console: &mut Console, new_run: &NewRun) {
        let h = 14 + self.scenarios.len() as i32;
        let panel = Panel::new(console, " New run ", 60, h);
        let x = panel.inner_x();
        let mut y = panel.y + 2;
        if self.scenarios.is_empty() {
            console.print(x, y, "No scenarios are installed.", TEXT);
        }
        for (i, info) in self.scenarios.iter().enumerate() {
            panel.row(console, y, i == new_run.selected, &info.name, "", 0);
            y += 1;
        }
        y += 1;
        if let Some(info) = self.scenarios.get(new_run.selected) {
            for line in wrap(&info.description, panel.inner_w() - 2) {
                console.print(x + 2, y, &line, TEXT_DIM);
                y += 1;
            }
            y += 1;
            let seed = if new_run.seed.is_empty() {
                format!("(default: {})", info.seed)
            } else {
                new_run.seed.clone()
            };
            let color = if new_run.seed.is_empty() {
                TEXT_DIM
            } else {
                TEXT
            };
            console.print(x + 2, y, "Seed: [", TEXT);
            let printed = console.print(x + 9, y, &seed, color);
            console.print(x + 9 + printed, y, "_]", TEXT);
            console.print(x + 2, y + 1, "Type digits to set a seed.", TEXT_DIM);
        }
        if self.has_save {
            let warning = "! Starting a new run replaces your saved run.";
            console.print(x, panel.y + panel.h - 4, warning, ACCENT);
        }
        panel.footer(console, "↑↓ scenario   Enter start   Esc back");
    }
}

/// A centered, framed panel on the console, with helpers to lay out rows inside it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Panel {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Panel {
    /// Draws a frame of up to `w` by `h` cells, centered, and returns it.
    pub fn new(console: &mut Console, title: &str, w: i32, h: i32) -> Self {
        let (cols, rows) = (console.cols() as i32, console.rows() as i32);
        let (w, h) = (w.min(cols), h.min(rows));
        let panel = Self {
            x: (cols - w) / 2,
            y: (rows - h) / 2,
            w,
            h,
        };
        console.frame(panel.x, panel.y, w, h, TEXT_DIM, PANEL_BG);
        console.print(panel.x + 2, panel.y, title, ACCENT);
        panel
    }

    pub fn inner_x(&self) -> i32 {
        self.x + 2
    }

    pub fn inner_w(&self) -> i32 {
        (self.w - 4).max(1)
    }

    /// Rows available between the top margin and the footer.
    pub fn body_rows(&self) -> i32 {
        (self.h - 4).max(0)
    }

    /// One menu row: a `>` marker when selected, the label, and a value column at `label_w`.
    /// An adjustable selected value is wrapped in `<` `>`, so that reads without color too.
    pub fn row(
        &self,
        console: &mut Console,
        y: i32,
        selected: bool,
        label: &str,
        value: &str,
        label_w: i32,
    ) {
        self.row_with(console, y, selected, label, value, label_w, false);
    }

    /// Like [`Panel::row`]; `adjustable` values get `<` `>` when selected.
    pub fn row_with(
        &self,
        console: &mut Console,
        y: i32,
        selected: bool,
        label: &str,
        value: &str,
        label_w: i32,
        adjustable: bool,
    ) {
        let x = self.inner_x();
        let color = if selected { ACCENT } else { TEXT };
        console.print(x, y, if selected { ">" } else { " " }, ACCENT);
        console.print(x + 2, y, label, color);
        if !value.is_empty() {
            let shown = if selected && adjustable {
                format!("< {value} >")
            } else {
                format!("  {value}  ")
            };
            console.print(x + 2 + label_w + 2, y, &shown, color);
        }
    }

    pub fn footer(&self, console: &mut Console, text: &str) {
        console.print(self.inner_x(), self.y + self.h - 2, text, TEXT_DIM);
    }
}

/// Greedy word wrap to `width` cells.
pub(super) fn wrap(text: &str, width: i32) -> Vec<String> {
    let width = width.max(1) as usize;
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Feeds key presses to the menu, redraws it, and starts the run it asks for.
pub(super) fn run(
    mut commands: Commands,
    mut keys: MessageReader<KeyboardInput>,
    mut menu: ResMut<Menu>,
    mut settings: ResMut<Settings>,
    mut bindings: ResMut<Bindings>,
    content: Res<GameContent>,
    store: Res<SaveStore>,
    layout: Res<ConsoleLayout>,
    mut console: ResMut<Console>,
    mut exit: MessageWriter<AppExit>,
) {
    let mut start = None;
    let (mut new_settings, mut new_bindings) = (settings.clone(), bindings.clone());
    for key in keys.read() {
        if key.state != ButtonState::Pressed || start.is_some() {
            continue;
        }
        let press = Press::from_input(key);
        start = menu.handle(&press, &mut new_settings, &mut new_bindings);
    }
    if new_settings != *settings {
        *settings = new_settings;
    }
    if new_bindings.0 != bindings.0 {
        *bindings = new_bindings;
    }
    match start {
        Some(Start::Quit) => {
            exit.write(AppExit::Success);
        }
        Some(Start::Continue) => match Game::resumed(&store, &content.0) {
            crate::save::Resume::Loaded(world) => {
                commands.insert_resource(Game::from_world(world));
                commands.remove_resource::<Menu>();
                return;
            }
            crate::save::Resume::Failed(message) => menu.fail(message, true),
            crate::save::Resume::NoSave => menu.fail("There is no saved run.".into(), true),
        },
        Some(Start::NewRun { scenario, seed }) => {
            match Game::new_run(content.0.clone(), &scenario, seed) {
                Ok(game) => {
                    commands.insert_resource(game);
                    commands.remove_resource::<Menu>();
                    return;
                }
                Err(message) => menu.fail(message, false),
            }
        }
        None => {}
    }
    if menu.is_changed() || layout.is_changed() || settings.is_changed() || bindings.is_changed() {
        menu.draw(&mut console, &settings, &bindings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu(has_save: bool) -> Menu {
        let content = Content::builtin().expect("content");
        Menu::new(&content, has_save, None)
    }

    fn press(menu: &mut Menu, nav: Nav) -> Option<Start> {
        let (mut s, mut b) = (Settings::default(), Bindings::default());
        menu.handle(&Press::nav(nav), &mut s, &mut b)
    }

    #[test]
    fn continue_is_offered_only_with_a_save() {
        assert_eq!(menu(false).items()[0], MainItem::NewRun);
        let mut with_save = menu(true);
        assert_eq!(with_save.items()[0], MainItem::Continue);
        assert_eq!(press(&mut with_save, Nav::Enter), Some(Start::Continue));
    }

    #[test]
    fn new_run_starts_the_chosen_scenario_with_the_typed_seed() {
        let mut m = menu(false);
        assert_eq!(press(&mut m, Nav::Enter), None); // opens New run
        assert!(matches!(m.screen, Screen::NewRun(_)));
        for d in ['4', '2'] {
            press(&mut m, Nav::Digit(d));
        }
        press(&mut m, Nav::Backspace);
        press(&mut m, Nav::Digit('7'));
        let Some(Start::NewRun { scenario, seed }) = press(&mut m, Nav::Enter) else {
            panic!("expected a new run");
        };
        assert_eq!(scenario, m.scenarios[0].id);
        assert_eq!(seed, Some(47));
        // Esc returns to the main menu, on the New run row.
        press(&mut m, Nav::Back);
        assert!(matches!(m.screen, Screen::Main(0)));
    }

    #[test]
    fn empty_seed_means_the_scenario_default() {
        let mut m = menu(false);
        press(&mut m, Nav::Enter);
        let Some(Start::NewRun { seed, .. }) = press(&mut m, Nav::Enter) else {
            panic!("expected a new run");
        };
        assert_eq!(seed, None);
    }

    #[test]
    fn quit_is_the_last_row_and_selection_wraps() {
        let mut m = menu(false);
        press(&mut m, Nav::Up);
        assert_eq!(press(&mut m, Nav::Enter), Some(Start::Quit));
    }

    #[test]
    fn failing_to_continue_drops_the_option_and_explains() {
        let mut m = menu(true);
        m.fail("Could not resume.".into(), true);
        assert_eq!(m.items()[0], MainItem::NewRun);
        assert!(m.notice.is_some());
    }

    #[test]
    fn every_screen_draws_in_a_small_console() {
        let settings = Settings::default();
        let bindings = Bindings::default();
        let content = Content::builtin().expect("content");
        for open in [
            ScreenName::Main,
            ScreenName::NewRun,
            ScreenName::Settings,
            ScreenName::Controls,
        ] {
            for (cols, rows) in [(80, 30), (40, 12), (5, 3), (0, 0)] {
                let mut console = Console::default();
                console.resize(cols, rows);
                Menu::new(&content, true, Some(open)).draw(&mut console, &settings, &bindings);
            }
        }
    }

    #[test]
    fn wrap_breaks_at_word_boundaries() {
        assert_eq!(wrap("one two three", 7), ["one two", "three"]);
        assert!(wrap("", 10).is_empty());
    }
}
