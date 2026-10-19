//! Menu building blocks and the main and pause menus (spec §6), drawn in glyphs on the UI
//! console.
//!
//! [`Press`] is a key press as screens see it, and [`Panel`] a centered frame with row helpers.
//! [`MainMenu`] is the main menu (Continue when a save exists, New run, Settings, Controls, Quit)
//! or, over a run, the pause menu (Resume, Settings, Controls, Save and quit to menu, Quit). Like
//! every screen it is a plain state machine: it takes presses and answers with a
//! [`ScreenAction`], so it is testable without a window. Selection is always shown by text (a `>`
//! marker), never by color alone.

use super::dialog_screen::Dialog;
use super::screens::{MenuChoice, ScreenAction, ScreenName};
use super::{ACCENT, PANEL_BG, TEXT, TEXT_DIM};
use crate::input::{Bindings, Context, KeyBinding, KeyCommand};
use crate::render::console::Console;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};

/// A key press reduced to what menus care about: the `Context::Menu` command it is bound to, or
/// Esc, which always backs out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    /// Choose the selected row (`KeyCommand::Confirm`, Enter).
    Enter,
    /// Back out: Esc, always, and `KeyCommand::Back`'s keys.
    Back,
    /// Erase a character typed, or reset a command's keys (`KeyCommand::Erase`).
    Erase,
    /// Mark the selected stack on a pick-up or drop list (`KeyCommand::Mark`, Space).
    Mark,
    /// Mark everything on a pick-up or drop list (`KeyCommand::MarkAll`, `,`).
    MarkAll,
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
    /// Shift was held (set by the input router, which sees the keyboard state).
    pub shift: bool,
}

impl Press {
    /// The press as screens see it: its menu meaning comes from the `Context::Menu` bindings,
    /// and Esc always backs out, whatever is bound (it cannot be bound: no lockout).
    pub fn from_input(input: &KeyboardInput, bindings: &Bindings) -> Self {
        let nav = if input.key_code == KeyCode::Escape {
            Nav::Back
        } else {
            match bindings.lookup_in(Context::Menu, input) {
                Some(KeyCommand::MenuUp) => Nav::Up,
                Some(KeyCommand::MenuDown) => Nav::Down,
                Some(KeyCommand::MenuLeft) => Nav::Left,
                Some(KeyCommand::MenuRight) => Nav::Right,
                Some(KeyCommand::Confirm) => Nav::Enter,
                Some(KeyCommand::Back) => Nav::Back,
                Some(KeyCommand::Erase) => Nav::Erase,
                Some(KeyCommand::Mark) => Nav::Mark,
                Some(KeyCommand::MarkAll) => Nav::MarkAll,
                _ => Nav::Other,
            }
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
            shift: false,
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
            shift: false,
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
            shift: false,
        }
    }
}

/// The menu keys as footers name them: each command's first key as bound, so a footer never
/// names a key that no longer does what it says (`?` for a command without a key).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hints {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub confirm: String,
    pub erase: String,
    pub mark: String,
    pub mark_all: String,
}

impl Hints {
    pub fn of(bindings: &Bindings) -> Self {
        let key = |command| bindings.key_name(command).unwrap_or_else(|| "?".into());
        Self {
            up: key(KeyCommand::MenuUp),
            down: key(KeyCommand::MenuDown),
            left: key(KeyCommand::MenuLeft),
            right: key(KeyCommand::MenuRight),
            confirm: key(KeyCommand::Confirm),
            erase: key(KeyCommand::Erase),
            mark: key(KeyCommand::Mark),
            mark_all: key(KeyCommand::MarkAll),
        }
    }

    /// The up and down keys together: `↑↓`, or `w/s` when they are not arrows.
    pub fn up_down(&self) -> String {
        pair(&self.up, &self.down)
    }

    /// The left and right keys together: `←→`, or `a/d`.
    pub fn left_right(&self) -> String {
        pair(&self.left, &self.right)
    }
}

/// Two keys side by side: arrows touch (`↑↓`), anything else is split by a slash (`w/s`).
fn pair(a: &str, b: &str) -> String {
    let arrow = |s: &str| matches!(s, "↑" | "↓" | "←" | "→");
    if arrow(a) && arrow(b) {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}

/// The dialog every way to quit asks (spec §6): over a run (`saving`) the run is saved first; on
/// the web, quitting reloads the page (`MenuChoice::Quit` does either).
pub fn quit_dialog(saving: bool) -> Dialog {
    let (text, yes) = if saving {
        ("Save the run and quit Warlock?", "Save and quit")
    } else {
        ("Quit Warlock?", "Quit")
    };
    Dialog::yes_no("Quit", text, yes, MenuChoice::Quit, "Stay")
}

/// The dialog the pause menu's Save and quit to menu asks.
pub fn save_to_menu_dialog() -> Dialog {
    Dialog::yes_no(
        "Main menu",
        "Save the run and return to the main menu?",
        "Save and return",
        MenuChoice::SaveToMenu,
        "Stay",
    )
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
        items.extend([MainItem::NewRun, MainItem::Settings, MainItem::Controls]);
        // A web page cannot quit: closing the tab is the browser's job.
        if !cfg!(target_arch = "wasm32") {
            items.push(MainItem::Quit);
        }
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
                    MainItem::SaveToMenu => save_to_menu_dialog().into(),
                    MainItem::NewRun => ScreenAction::Open(ScreenName::NewRun),
                    MainItem::Settings => ScreenAction::Open(ScreenName::Settings),
                    MainItem::Controls => ScreenAction::Open(ScreenName::Controls),
                    MainItem::Quit => quit_dialog(self.pause).into(),
                };
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    /// Draws the menu, with `notice` (something the player should know) under the rows.
    pub fn draw(&self, console: &mut Console, notice: Option<&str>, bindings: &Bindings) {
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
        let keys = Hints::of(bindings);
        let hint = format!("{} select   {} choose", keys.up_down(), keys.confirm);
        let hint = if self.pause {
            format!("{hint}   Esc resume")
        } else {
            hint
        };
        panel.footer(console, &hint);
        if !self.pause {
            let cols = console.cols() as usize;
            if let Some(notice) = legal_notice(cols.saturating_sub(2)) {
                let x = (cols - notice.chars().count()) as i32 / 2;
                let y = console.rows() as i32 - 1;
                console.print(x, y, &notice, TEXT_DIM);
            }
        }
    }
}

/// The copyright and license notice the main menu shows on its bottom row (GPL-3.0 "Appropriate
/// Legal Notices": copyright, no warranty, where the license is), as long as fits in `width`
/// columns; `None` if not even the short form fits.
fn legal_notice(width: usize) -> Option<String> {
    let version = env!("CARGO_PKG_VERSION");
    [
        format!(
            "Warlock {version} · © 2026 Janne Lê Forsell · free software under GPL-3.0 \
             (see LICENSE), without warranty"
        ),
        format!("Warlock {version} · © 2026 Janne Lê Forsell · GPL-3.0, no warranty"),
        "© 2026 Janne Lê Forsell · GPL-3.0, no warranty".to_string(),
    ]
    .into_iter()
    .find(|n| n.chars().count() <= width)
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

    fn key(code: KeyCode, logical: Key) -> KeyboardInput {
        KeyboardInput {
            key_code: code,
            logical_key: logical,
            state: bevy::input::ButtonState::Pressed,
            text: None,
            repeat: false,
            window: bevy::prelude::Entity::PLACEHOLDER,
        }
    }

    #[test]
    fn menu_keys_come_from_the_bindings_and_esc_always_backs_out() {
        let mut b = Bindings::default();
        let nav = |b: &Bindings, input: &KeyboardInput| Press::from_input(input, b).nav;
        let down = key(KeyCode::ArrowDown, Key::ArrowDown);
        let s = key(KeyCode::KeyS, Key::Character("s".into()));
        let esc = key(KeyCode::Escape, Key::Escape);
        let enter = key(KeyCode::Enter, Key::Enter);
        let space = key(KeyCode::Space, Key::Space);
        let j = key(KeyCode::KeyJ, Key::Character("j".into()));
        assert_eq!(nav(&b, &down), Nav::Down);
        assert_eq!(nav(&b, &j), Nav::Down);
        assert_eq!(nav(&b, &enter), Nav::Enter);
        assert_eq!(nav(&b, &space), Nav::Mark);
        assert_eq!(nav(&b, &s), Nav::Other);
        assert_eq!(nav(&b, &esc), Nav::Back);
        // Rebound: `s` moves down, the arrow it replaced does not, and `j` still does.
        b.bind_primary(KeyCommand::MenuDown, KeyBinding::Char("s".into()))
            .unwrap();
        assert_eq!(nav(&b, &s), Nav::Down);
        assert_eq!(nav(&b, &down), Nav::Other);
        assert_eq!(nav(&b, &j), Nav::Down);
        // Back on Space: Esc still backs out, and Space no longer marks.
        b.bind_primary(KeyCommand::Back, KeyBinding::Code(KeyCode::Space))
            .unwrap();
        assert_eq!(nav(&b, &space), Nav::Back);
        assert_eq!(nav(&b, &esc), Nav::Back);
        // With the defaults the footers read as before; rebound, they name the new keys.
        let hints = Hints::of(&Bindings::default());
        assert_eq!(
            (hints.up_down(), hints.confirm.as_str()),
            ("↑↓".into(), "Enter")
        );
        assert_eq!(Hints::of(&b).up_down(), "↑/s");
    }

    #[test]
    fn legal_notice_shortens_to_fit_and_always_names_the_license() {
        let long = legal_notice(200).expect("fits");
        assert!(long.contains("see LICENSE"));
        for width in [60, 70, 90, 120] {
            if let Some(n) = legal_notice(width) {
                assert!(n.chars().count() <= width);
                assert!(n.contains("© 2026 Janne Lê Forsell") && n.contains("GPL-3.0"));
                assert!(n.contains("warranty"));
            }
        }
        assert_eq!(legal_notice(10), None);
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
            ScreenAction::from(save_to_menu_dialog())
        );
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::from(quit_dialog(true))
        );
    }

    #[test]
    fn quit_is_the_last_row_and_selection_wraps() {
        let mut m = MainMenu::main(false);
        press(&mut m, Nav::Up);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::from(quit_dialog(false))
        );
    }

    #[test]
    fn wrap_breaks_at_word_boundaries() {
        assert_eq!(wrap("one two three", 7), ["one two", "three"]);
        assert!(wrap("", 10).is_empty());
    }
}
