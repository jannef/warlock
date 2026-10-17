//! Menu building blocks and the main and pause menus (spec §6), drawn in glyphs on the UI
//! console.
//!
//! [`Press`] is a key press as screens see it, and [`Panel`] a centered frame with row helpers.
//! [`MainMenu`] is the main menu (Continue when a save exists, New run, Settings, Controls, Quit)
//! or, over a run, the pause menu (Resume, Settings, Controls, Save and quit to menu, Quit). Like
//! every screen it is a plain state machine: it takes presses and answers with a
//! [`ScreenAction`], so it is testable without a window. Selection is always shown by text (a `>`
//! marker), never by color alone.

use super::screens::{MenuChoice, ScreenAction, ScreenName};
use super::{ACCENT, PANEL_BG, TEXT, TEXT_DIM};
use crate::input::KeyBinding;
use crate::render::console::Console;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};

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
    Other,
}

/// One key press: its menu meaning, and the binding it would make (for the rebinding screen).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Press {
    pub nav: Nav,
    pub binding: Option<KeyBinding>,
    /// A held key repeating.
    pub repeat: bool,
    /// The character the key typed, if any (for text fields such as the seed).
    pub text: Option<char>,
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
                    _ => Nav::Other,
                },
                _ => Nav::Other,
            },
        };
        let text = match &input.logical_key {
            Key::Character(c) => {
                let mut chars = c.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) if !ch.is_control() => Some(ch),
                    _ => None,
                }
            }
            _ => None,
        };
        Self {
            nav,
            binding: KeyBinding::from_input(input),
            repeat: input.repeat,
            text,
        }
    }

    /// A press of a navigation key.
    #[cfg(test)]
    pub fn nav(nav: Nav) -> Self {
        Self {
            nav,
            binding: None,
            repeat: false,
            text: None,
        }
    }

    /// A press that types `ch`.
    #[cfg(test)]
    pub fn typed(ch: char) -> Self {
        Self {
            nav: Nav::Other,
            binding: None,
            repeat: false,
            text: Some(ch),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainItem {
    Resume,
    SaveToMenu,
    Continue,
    NewRun,
    Settings,
    Controls,
    Quit,
}

impl MainItem {
    fn label(self) -> &'static str {
        match self {
            MainItem::Resume => "Resume",
            MainItem::SaveToMenu => "Save and quit to menu",
            MainItem::Continue => "Continue",
            MainItem::NewRun => "New run",
            MainItem::Settings => "Settings",
            MainItem::Controls => "Controls",
            MainItem::Quit => "Quit",
        }
    }
}

/// The main menu, or the pause menu over a run.
#[derive(Debug)]
pub struct MainMenu {
    /// The selected row.
    selected: usize,
    /// The pause menu rather than the main menu.
    pause: bool,
    /// A saved run exists, so the main menu offers Continue.
    has_save: bool,
}

impl MainMenu {
    pub fn main(has_save: bool) -> Self {
        Self {
            selected: 0,
            pause: false,
            has_save,
        }
    }

    pub fn pause() -> Self {
        Self {
            selected: 0,
            pause: true,
            has_save: false,
        }
    }

    #[cfg(test)]
    pub fn is_pause(&self) -> bool {
        self.pause
    }

    fn items(&self) -> Vec<MainItem> {
        if self.pause {
            return vec![
                MainItem::Resume,
                MainItem::Settings,
                MainItem::Controls,
                MainItem::SaveToMenu,
                MainItem::Quit,
            ];
        }
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

    /// Stops offering Continue (the save could not be resumed).
    pub fn drop_continue(&mut self) {
        self.has_save = false;
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let items = self.items();
        self.selected = self.selected.min(items.len() - 1);
        match press.nav {
            Nav::Up => self.selected = (self.selected + items.len() - 1) % items.len(),
            Nav::Down => self.selected = (self.selected + 1) % items.len(),
            Nav::Back if self.pause => return ScreenAction::Close,
            Nav::Enter => {
                return match items[self.selected] {
                    MainItem::Resume => ScreenAction::Close,
                    MainItem::Continue => ScreenAction::Choose(MenuChoice::Continue),
                    MainItem::SaveToMenu => ScreenAction::Choose(MenuChoice::SaveToMenu),
                    MainItem::NewRun => ScreenAction::Open(ScreenName::NewRun),
                    MainItem::Settings => ScreenAction::Open(ScreenName::Settings),
                    MainItem::Controls => ScreenAction::Open(ScreenName::Controls),
                    MainItem::Quit => ScreenAction::Choose(MenuChoice::Quit),
                };
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    /// Draws the menu, with `notice` (something the player should know) under the rows.
    pub fn draw(&self, console: &mut Console, notice: Option<&str>) {
        let items = self.items();
        let notice_lines = notice.map_or(Vec::new(), |n| wrap(n, 44));
        let h = 12 + items.len() as i32 + notice_lines.len() as i32;
        let panel = Panel::new(console, "", 52, h);
        let cx = |text: &str| panel.x + (panel.w - text.chars().count() as i32) / 2;
        let title = if self.pause {
            "P A U S E D"
        } else {
            "W A R L O C K"
        };
        console.print(cx(title), panel.y + 2, title, ACCENT);
        let rule = "═".repeat(title.chars().count());
        console.print(cx(&rule), panel.y + 3, &rule, TEXT_DIM);
        let tagline = if self.pause {
            "the world waits"
        } else {
            "an ASCII roguelike"
        };
        console.print(cx(tagline), panel.y + 4, tagline, TEXT_DIM);
        for (i, item) in items.iter().enumerate() {
            let y = panel.y + 7 + i as i32;
            panel.row(console, y, i == self.selected, item.label(), "", 0);
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
        let hint = if self.pause {
            "↑↓ select   Enter choose   Esc resume"
        } else {
            "↑↓ select   Enter choose"
        };
        panel.footer(console, hint);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn press(menu: &mut MainMenu, nav: Nav) -> ScreenAction {
        menu.handle(&Press::nav(nav))
    }

    #[test]
    fn continue_is_offered_only_with_a_save() {
        assert_eq!(MainMenu::main(false).items()[0], MainItem::NewRun);
        let mut with_save = MainMenu::main(true);
        assert_eq!(with_save.items()[0], MainItem::Continue);
        assert_eq!(
            press(&mut with_save, Nav::Enter),
            ScreenAction::Choose(MenuChoice::Continue)
        );
        with_save.drop_continue();
        assert_eq!(with_save.items()[0], MainItem::NewRun);
    }

    #[test]
    fn main_menu_opens_sub_screens_and_ignores_esc() {
        let mut m = MainMenu::main(false);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Open(ScreenName::NewRun)
        );
        assert_eq!(press(&mut m, Nav::Back), ScreenAction::Stay);
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Open(ScreenName::Settings)
        );
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Open(ScreenName::Controls)
        );
    }

    #[test]
    fn pause_menu_rows_and_choices() {
        let mut m = MainMenu::pause();
        let rows: Vec<_> = m.items().iter().map(|i| i.label()).collect();
        assert_eq!(
            rows,
            [
                "Resume",
                "Settings",
                "Controls",
                "Save and quit to menu",
                "Quit"
            ]
        );
        assert_eq!(press(&mut m, Nav::Back), ScreenAction::Close);
        assert_eq!(press(&mut m, Nav::Enter), ScreenAction::Close);
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Open(ScreenName::Settings)
        );
        press(&mut m, Nav::Down);
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Choose(MenuChoice::SaveToMenu)
        );
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Choose(MenuChoice::Quit)
        );
    }

    #[test]
    fn quit_is_the_last_row_and_selection_wraps() {
        let mut m = MainMenu::main(false);
        press(&mut m, Nav::Up);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Choose(MenuChoice::Quit)
        );
    }

    #[test]
    fn wrap_breaks_at_word_boundaries() {
        assert_eq!(wrap("one two three", 7), ["one two", "three"]);
        assert!(wrap("", 10).is_empty());
    }
}
