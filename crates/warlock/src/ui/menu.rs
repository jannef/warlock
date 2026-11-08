//! Menu building blocks and the main and pause menus (spec §6), drawn in glyphs on the UI
//! console.
//!
//! [`Press`] is a key press as screens see it, and [`Panel`] a centered frame with row helpers.
//! [`MainMenu`] is the main menu (Continue, as the character played last that this version can
//! continue, Characters when any character is saved, and Graveyard once anyone has died; New run, Settings, Controls, Quit) or, over a run, the pause menu (Resume, Settings, Controls, Save and quit to menu, Quit). Like
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
    /// The next page of an item list (`KeyCommand::NextListPage`, `>`).
    NextPage,
    /// The previous page of an item list (`KeyCommand::PrevListPage`, `<`).
    PrevPage,
    Other,
}

/// The numpad keys, by physical key whatever Num Lock says: with it on they report the digit
/// (`Key::Character("8")`, and macOS has no Num Lock: always so), with it off the arrow, Home,
/// and the like (`Key::ArrowUp`, winit 0.30 on Windows and X11/Wayland).
const NUMPAD: [KeyCode; 11] = [
    KeyCode::Numpad0,
    KeyCode::Numpad1,
    KeyCode::Numpad2,
    KeyCode::Numpad3,
    KeyCode::Numpad4,
    KeyCode::Numpad5,
    KeyCode::Numpad6,
    KeyCode::Numpad7,
    KeyCode::Numpad8,
    KeyCode::Numpad9,
    KeyCode::NumpadEnter,
];

/// The numpad's fixed menu meaning, for a key no `Context::Menu` binding claims: 8, 2, 4, and 6
/// are the arrows (#1197; NumEnter is Confirm's own binding). Fixed screen keys, never
/// rebindable (CLAUDE.md): an alias, not a binding.
fn numpad_nav(code: KeyCode) -> Nav {
    match code {
        KeyCode::Numpad8 => Nav::Up,
        KeyCode::Numpad2 => Nav::Down,
        KeyCode::Numpad4 => Nav::Left,
        KeyCode::Numpad6 => Nav::Right,
        _ => Nav::Other,
    }
}

/// A key press reduced to what a text field cares about besides the text typed: the
/// `Context::Text` command it is bound to, or Esc, which always backs out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Accept the text (`KeyCommand::TextConfirm`, Enter).
    Confirm,
    /// Back out: Esc, always, and `KeyCommand::TextBack`'s keys.
    Back,
    /// Erase the last character (`KeyCommand::TextErase`, Backspace).
    Erase,
    /// A random value (`KeyCommand::TextRandom`, Tab).
    Random,
}

/// One key press: its menu meaning, and the binding it would make (for the rebinding screen).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Press {
    pub nav: Nav,
    /// Its meaning in a text field, if it has one there (else it types its text).
    pub edit: Option<Edit>,
    pub binding: Option<KeyBinding>,
    /// A held key repeating.
    pub repeat: bool,
    /// The character the key typed, if any (for text fields such as the seed and the name): the
    /// key event's text, literal, else the character its key stands for.
    pub text: Option<char>,
    /// Shift was held (set by the input router, which sees the keyboard state).
    pub shift: bool,
}

impl Press {
    /// The press as screens see it: its menu meaning comes from the `Context::Menu` bindings,
    /// its text field meaning from the `Context::Text` bindings, and Esc always backs out,
    /// whatever is bound (it cannot be bound: no lockout).
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
                Some(KeyCommand::NextListPage) => Nav::NextPage,
                Some(KeyCommand::PrevListPage) => Nav::PrevPage,
                _ => numpad_nav(input.key_code),
            }
        };
        let edit = if input.key_code == KeyCode::Escape {
            Some(Edit::Back)
        } else {
            match bindings.lookup_in(Context::Text, input) {
                Some(KeyCommand::TextConfirm) => Some(Edit::Confirm),
                Some(KeyCommand::TextBack) => Some(Edit::Back),
                Some(KeyCommand::TextErase) => Some(Edit::Erase),
                Some(KeyCommand::TextRandom) => Some(Edit::Random),
                _ => None,
            }
        };
        // One printable character: the event's text (what the layout and dead keys made of the
        // press), else the character the key stands for.
        let single = |text: &str| {
            let mut chars = text.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) if !ch.is_control() => Some(ch),
                _ => None,
            }
        };
        let text = match (&input.text, &input.logical_key) {
            (Some(text), _) => single(text),
            (None, Key::Character(c)) => single(c),
            _ => None,
        };
        Self {
            nav,
            edit,
            binding: KeyBinding::from_input(input),
            repeat: input.repeat,
            text,
            shift: false,
        }
    }

    /// Whether the press came from the numpad. Its digits are navigation on screens (#1197), so
    /// a screen that reads a typed digit as a choice (a dialogue's topic) leaves these alone;
    /// text fields still type them.
    pub fn is_numpad(&self) -> bool {
        matches!(&self.binding, Some(KeyBinding::Code(code)) if NUMPAD.contains(code))
    }

    /// A press of a navigation key.
    #[cfg(test)]
    pub fn nav(nav: Nav) -> Self {
        Self {
            nav,
            edit: None,
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
            edit: None,
            binding: None,
            repeat: false,
            text: Some(ch),
            shift: false,
        }
    }

    /// A press of a text field's key `edit`.
    #[cfg(test)]
    pub fn edit(edit: Edit) -> Self {
        Self {
            edit: Some(edit),
            ..Self::nav(Nav::Other)
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
    pub next_page: String,
    pub prev_page: String,
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
            next_page: key(KeyCommand::NextListPage),
            prev_page: key(KeyCommand::PrevListPage),
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
    Characters,
    Graveyard,
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
            MainItem::Characters => "Characters",
            MainItem::Graveyard => "Graveyard",
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
    /// The name of the character played last that this version can continue, if any: the main
    /// menu offers to Continue as them.
    last: Option<String>,
    /// Any character is saved (of this version or another): the main menu offers the list.
    saved: bool,
    /// Anyone has died (the graveyard, `graveyard.rs`): the main menu offers the Graveyard.
    dead: bool,
}

impl MainMenu {
    /// The main menu; `last` names the character Continue resumes, if any, and `saved` says
    /// whether any character is saved.
    pub fn main(last: Option<String>, saved: bool) -> Self {
        Self {
            selected: 0,
            pause: false,
            saved: saved || last.is_some(),
            last,
            dead: false,
        }
    }

    /// The menu, offering the Graveyard when `dead` (anyone has died).
    pub fn graveyard(self, dead: bool) -> Self {
        Self { dead, ..self }
    }

    pub fn pause() -> Self {
        Self {
            selected: 0,
            pause: true,
            last: None,
            saved: false,
            dead: false,
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
        if self.last.is_some() {
            items.push(MainItem::Continue);
        }
        if self.saved {
            items.push(MainItem::Characters);
        }
        if self.dead {
            items.push(MainItem::Graveyard);
        }
        items.extend([MainItem::NewRun, MainItem::Settings, MainItem::Controls]);
        // A web page cannot quit: closing the tab is the browser's job.
        if !cfg!(target_arch = "wasm32") {
            items.push(MainItem::Quit);
        }
        items
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
                    MainItem::Characters => ScreenAction::Open(ScreenName::Characters),
                    MainItem::Graveyard => ScreenAction::Open(ScreenName::Graveyard),
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
            // Continue names who it resumes.
            let value = match item {
                MainItem::Continue => self.last.as_deref().unwrap_or(""),
                _ => "",
            };
            panel.row(console, y, i == self.selected, item.label(), value, 8);
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
    fn the_numpad_is_the_arrows_with_num_lock_on_or_off() {
        let b = Bindings::default();
        // Num Lock on (and macOS, which has none): the digit; off: the arrow, Home, and so on.
        let on = |code, digit: &str| key(code, Key::Character(digit.into()));
        for (code, digit, off, nav) in [
            (KeyCode::Numpad8, "8", Key::ArrowUp, Nav::Up),
            (KeyCode::Numpad2, "2", Key::ArrowDown, Nav::Down),
            (KeyCode::Numpad4, "4", Key::ArrowLeft, Nav::Left),
            (KeyCode::Numpad6, "6", Key::ArrowRight, Nav::Right),
            (KeyCode::Numpad7, "7", Key::Home, Nav::Other),
            (KeyCode::Numpad5, "5", Key::Clear, Nav::Other),
        ] {
            for input in [on(code, digit), key(code, off)] {
                let press = Press::from_input(&input, &b);
                assert_eq!(press.nav, nav, "{code:?}");
                assert!(press.is_numpad());
                assert_eq!(press.binding, Some(KeyBinding::Code(code)));
            }
            // With Num Lock on it still types its digit, for a text field.
            assert_eq!(
                Press::from_input(&on(code, digit), &b).text,
                digit.chars().next()
            );
        }
        let enter = Press::from_input(&key(KeyCode::NumpadEnter, Key::Enter), &b);
        assert_eq!(enter.nav, Nav::Enter);
        assert_eq!(enter.edit, Some(Edit::Confirm));
        // The top row's digits are no navigation, and no numpad.
        let top = Press::from_input(&key(KeyCode::Digit8, Key::Character("8".into())), &b);
        assert_eq!(top.nav, Nav::Other);
        assert!(!top.is_numpad());
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
    fn continue_and_characters_are_offered_only_with_a_save() {
        assert_eq!(MainMenu::main(None, false).items()[0], MainItem::NewRun);
        // Only saves of another version: the list, but nothing to continue.
        assert_eq!(
            MainMenu::main(None, true).items()[..2],
            [MainItem::Characters, MainItem::NewRun]
        );
        let mut with_save = MainMenu::main(Some("Brakka".into()), true);
        assert_eq!(
            with_save.items()[..3],
            [MainItem::Continue, MainItem::Characters, MainItem::NewRun]
        );
        assert_eq!(
            press(&mut with_save, Nav::Enter),
            ScreenAction::Choose(MenuChoice::Continue)
        );
        press(&mut with_save, Nav::Down);
        assert_eq!(
            press(&mut with_save, Nav::Enter),
            ScreenAction::Open(ScreenName::Characters)
        );
    }

    #[test]
    fn the_graveyard_is_offered_once_anyone_has_died() {
        assert!(
            !MainMenu::main(None, true)
                .items()
                .contains(&MainItem::Graveyard)
        );
        let mut m = MainMenu::main(None, true).graveyard(true);
        assert_eq!(
            m.items()[..3],
            [MainItem::Characters, MainItem::Graveyard, MainItem::NewRun]
        );
        press(&mut m, Nav::Down);
        assert_eq!(
            press(&mut m, Nav::Enter),
            ScreenAction::Open(ScreenName::Graveyard)
        );
    }

    #[test]
    fn main_menu_opens_sub_screens_and_ignores_esc() {
        let mut m = MainMenu::main(None, false);
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
        let mut m = MainMenu::main(None, false);
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
