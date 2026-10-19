//! The controls screen and rebinding UI (spec §6): select a command, press a key to bind it.
//!
//! Commands are listed by context (`input::Context`: in play, aiming, the rest prompt, menus
//! and screens, info pages, dialogs, and in builds that have them the developer keys); a key is
//! unique within its context only. Enter on a command waits for a key, which becomes the
//! command's primary key (its other keys, such as the numpad, stay). A key another command of the
//! same context uses moves to this one, and the message says so; Esc (fixed) and the last key of
//! a vital command (`settings::keys::VITAL`) are refused, so the player cannot lock themselves
//! out. Esc cancels, Delete resets one command, and "Reset all keys" restores the defaults.
//!
//! The `?` help is the same screen, read-only ([`ControlsScreen::help`]): it lists every
//! command by context, including Esc, and closes on `?` or Esc. The list is drawn by one piece
//! of code for both, so help always matches the bindings.

use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, Context, KeyCommand};
use crate::render::console::Console;
use crate::settings::keys::{Bound, REBINDABLE};

/// What the screen is doing.
#[derive(Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Browse,
    /// Waiting for a key for the command at the selected row.
    Capture,
    /// The `?` help: nothing can be changed; `selected` is the first row shown.
    ReadOnly,
}

/// One row of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    /// A line of text about keys in general.
    Note(&'static str),
    /// The space before a group.
    Blank,
    /// The heading of a context's group.
    Header(Context),
    Command(KeyCommand),
    Reset,
    Back,
}

impl Row {
    fn selectable(self) -> bool {
        matches!(self, Row::Command(_) | Row::Reset | Row::Back)
    }
}

/// What the list says first: the keys no binding changes.
const NOTES: [&str; 2] = [
    "Esc always backs out and cancels; it cannot be rebound.",
    "Letters (a-z) and digits that pick list rows are fixed.",
];

/// Selection state of the controls screen. Rows are the notes, then each context's commands
/// under its heading, then "Reset all keys" and "Back"; in help, the notes and every command.
#[derive(Debug)]
pub struct ControlsScreen {
    /// The selected row; in help, the first row shown.
    selected: usize,
    mode: Mode,
    /// Whether the developer keys are listed (they work in this launch).
    dev: bool,
    /// Result of the last action, a line or a few; a leading `!` marks a refusal (never color
    /// alone).
    message: Vec<String>,
}

impl ControlsScreen {
    /// The rebinding screen; `dev`: list the developer keys too.
    pub fn new(dev: bool) -> Self {
        let mut screen = Self {
            selected: 0,
            mode: Mode::Browse,
            dev,
            message: Vec::new(),
        };
        screen.selected = screen.next_selectable(0, 1);
        screen
    }

    /// The `?` help: every command, read-only.
    pub fn help(dev: bool) -> Self {
        Self {
            selected: 0,
            mode: Mode::ReadOnly,
            dev,
            message: Vec::new(),
        }
    }

    pub fn is_help(&self) -> bool {
        self.mode == Mode::ReadOnly
    }

    /// The rows shown: the notes, then every context's commands in [`REBINDABLE`] order under
    /// its heading (help adds Esc's Cancel to play's); the rebinding screen ends with its
    /// actions.
    fn rows(&self) -> Vec<Row> {
        let read_only = self.mode == Mode::ReadOnly;
        let mut rows: Vec<Row> = NOTES.iter().map(|note| Row::Note(note)).collect();
        for &context in Context::ALL {
            if context == Context::Dev && !self.dev {
                continue;
            }
            rows.extend([Row::Blank, Row::Header(context)]);
            let commands = REBINDABLE.iter().filter(|c| c.context() == context);
            rows.extend(commands.copied().map(Row::Command));
            if read_only && context == Context::Play {
                rows.push(Row::Command(KeyCommand::Cancel));
            }
        }
        if !read_only {
            rows.extend([Row::Blank, Row::Reset, Row::Back]);
        }
        rows
    }

    /// The first selectable row from `from` on in direction `step` (wrapping around).
    fn next_selectable(&self, from: usize, step: isize) -> usize {
        let rows = self.rows();
        let n = rows.len() as isize;
        (0..n)
            .map(|i| (from as isize + step * i).rem_euclid(n) as usize)
            .find(|&i| rows[i].selectable())
            .unwrap_or(0)
    }

    /// The command at the selected row, if it is one.
    fn selected_command(&self) -> Option<KeyCommand> {
        match self.rows().get(self.selected) {
            Some(Row::Command(command)) => Some(*command),
            _ => None,
        }
    }

    /// Applies a key press to `bindings`. Returns true when the player leaves the screen.
    pub fn handle(&mut self, press: &Press, bindings: &mut Bindings) -> bool {
        match self.mode {
            Mode::Capture => {
                self.capture(press, bindings);
                return false;
            }
            Mode::ReadOnly => return self.read_only(press, bindings),
            Mode::Browse => {}
        }
        let row = self.rows().get(self.selected).copied();
        match press.nav {
            Nav::Back => return true,
            Nav::Up => {
                self.selected = self.next_selectable(self.selected + self.rows().len() - 1, -1);
                self.message.clear();
            }
            Nav::Down => {
                self.selected = self.next_selectable(self.selected + 1, 1);
                self.message.clear();
            }
            Nav::Enter if !press.repeat => match row {
                Some(Row::Reset) => {
                    bindings.reset_all();
                    self.message = vec!["All keys are back to their defaults.".into()];
                }
                Some(Row::Back) => return true,
                Some(Row::Command(_)) => {
                    self.mode = Mode::Capture;
                    self.message.clear();
                }
                _ => {}
            },
            Nav::Erase if !press.repeat => {
                if let Some(command) = self.selected_command() {
                    bindings.reset_command(command);
                    let name = command.name();
                    self.message = vec![format!("{name} is back to its default keys.")];
                }
            }
            _ => {}
        }
        false
    }

    /// Help: the menu keys scroll; Esc or the help key (`?`) closes.
    fn read_only(&mut self, press: &Press, bindings: &Bindings) -> bool {
        let help_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Help));
        match press.nav {
            _ if help_key => return true,
            Nav::Back => return true,
            Nav::Up => self.selected = self.selected.saturating_sub(1),
            Nav::Down => {
                let last = self.rows().len().saturating_sub(1);
                self.selected = (self.selected + 1).min(last);
            }
            _ => {}
        }
        false
    }

    fn capture(&mut self, press: &Press, bindings: &mut Bindings) {
        if press.repeat {
            return;
        }
        if press.nav == Nav::Back {
            self.mode = Mode::Browse;
            self.message = vec!["Cancelled.".into()];
            return;
        }
        let Some(key) = press.binding.clone() else {
            let refusal = "! That key cannot be bound. Try another, or Esc to cancel.";
            self.message = vec![refusal.into()];
            return;
        };
        let Some(command) = self.selected_command() else {
            self.mode = Mode::Browse;
            return;
        };
        let shown = key.display();
        self.mode = Mode::Browse;
        self.message = vec![match bindings.bind_primary(command, key) {
            Ok(Bound::Free) => format!("{} is now {shown}.", command.name()),
            Ok(Bound::Moved {
                from,
                keyless: true,
                ..
            }) => format!(
                "{shown} moved to {}; {} has no key.",
                command.name(),
                from.name()
            ),
            Ok(Bound::Moved { from, .. }) => {
                format!("{shown} moved to {} (was {}).", command.name(), from.name())
            }
            Err(conflict) => format!(
                "! {shown} is the only key for {} and cannot be taken.",
                conflict.owner.name()
            ),
        }];
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let rows = self.rows();
        let count = rows.len();
        let read_only = self.mode == Mode::ReadOnly;
        // Room below the list for the message (up to three lines) and the footer; help has no
        // messages.
        let spare = if read_only { 6 } else { 10 };
        let panel = Panel::new(console, " Controls ", 60, count as i32 + spare);
        let message_rows = if read_only { 0 } else { 4 };
        let visible = (panel.body_rows() - message_rows).max(1) as usize;
        let first = if read_only {
            self.selected
        } else {
            self.selected.saturating_sub(visible - 1)
        }
        .min(count.saturating_sub(visible));
        let label_w = rows
            .iter()
            .filter_map(|row| match row {
                Row::Command(c) => Some(c.name().chars().count()),
                _ => None,
            })
            .max()
            .unwrap_or(0) as i32;
        for (i, row) in rows.iter().enumerate().skip(first).take(visible) {
            let y = panel.y + 2 + (i - first) as i32;
            // Help has nothing to select.
            let selected = !read_only && i == self.selected;
            match row {
                Row::Note(note) => {
                    console.print(panel.inner_x(), y, note, TEXT_DIM);
                }
                Row::Blank => {}
                Row::Header(context) => {
                    console.print(panel.inner_x(), y, context.title(), ACCENT);
                }
                Row::Command(command) => {
                    let value = if selected && self.mode == Mode::Capture {
                        "press a key...".to_string()
                    } else {
                        key_list(bindings, *command)
                    };
                    panel.row(console, y, selected, &command.name(), &value, label_w);
                }
                Row::Reset => panel.row(console, y, selected, "Reset all keys", "", label_w),
                Row::Back => panel.row(console, y, selected, "Back", "", label_w),
            }
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        let more = first + visible < count;
        if more {
            console.print(panel.x + panel.w - 3, panel.y + panel.h - 2, "↓", TEXT_DIM);
        }
        if !read_only {
            self.draw_message(console, &panel, bindings);
        }
        let keys = Hints::of(bindings);
        let footer = if read_only {
            let close = match bindings.key_name(KeyCommand::Help) {
                Some(key) => format!("{key} or Esc close"),
                None => "Esc close".to_string(),
            };
            let scroll = if first > 0 || more {
                format!("{} scroll   ", keys.up_down())
            } else {
                String::new()
            };
            format!("{scroll}{close}")
        } else {
            format!("{} rebind   {} reset   Esc back", keys.confirm, keys.erase)
        };
        panel.footer(console, &footer);
    }

    /// The lines under the list: the capture prompt, the last action's message, or a warning
    /// that a context holds a key twice (which loading resolves, so never for long).
    fn draw_message(&self, console: &mut Console, panel: &Panel, bindings: &Bindings) {
        let lines: Vec<String> = if self.mode == Mode::Capture {
            let name = self
                .selected_command()
                .map_or_else(String::new, |c| c.name());
            vec![format!("Press a key for \"{name}\". Esc cancels.")]
        } else if !self.message.is_empty() {
            self.message.clone()
        } else if let Some((key, first, second)) = bindings.conflicts().first() {
            vec![format!(
                "! {} is bound to both {} and {} ({}).",
                key.display(),
                first.name(),
                second.name(),
                first.context().title()
            )]
        } else {
            Vec::new()
        };
        let refusal = lines.first().is_some_and(|l| l.starts_with('!'));
        let color = if refusal || self.mode == Mode::Capture {
            ACCENT
        } else {
            TEXT
        };
        let wrapped = lines.iter().flat_map(|l| wrap(l, panel.inner_w()));
        let top = panel.y + panel.h - 6;
        for (i, line) in wrapped.take(3).enumerate() {
            console.print(panel.inner_x(), top + i as i32, &line, color);
        }
    }
}

/// A command's keys, primary first, separated by spaces.
fn key_list(bindings: &Bindings, command: KeyCommand) -> String {
    let keys: Vec<String> = bindings
        .keys_of(command)
        .iter()
        .map(|k| k.display())
        .collect();
    if keys.is_empty() {
        "(none)".into()
    } else {
        keys.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use warlock_core::Dir;

    fn nav(screen: &mut ControlsScreen, b: &mut Bindings, nav: Nav) -> bool {
        screen.handle(&Press::nav(nav), b)
    }

    fn key(screen: &mut ControlsScreen, b: &mut Bindings, text: &str) {
        let press = Press {
            nav: Nav::Other,
            binding: KeyBinding::from_text(text),
            repeat: false,
            text: None,
            shift: false,
        };
        screen.handle(&press, b);
    }

    /// Selects the row of `row` (it must be on the screen).
    fn select(screen: &mut ControlsScreen, row: Row) {
        screen.selected = screen.rows().iter().position(|r| *r == row).expect("a row");
    }

    fn message(screen: &ControlsScreen) -> String {
        screen.message.join(" ")
    }

    #[test]
    fn rebinding_replaces_the_primary_key() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        // The first row selected is the first command, under its heading.
        assert_eq!(screen.selected_command(), Some(KeyCommand::Move(Dir::N)));
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(screen.mode, Mode::Capture);
        key(&mut screen, &mut b, "w");
        assert_eq!(screen.mode, Mode::Browse);
        assert_eq!(b.keys_of(KeyCommand::Move(Dir::N))[0].display(), "w");
        assert!(message(&screen).contains("now w"));
    }

    #[test]
    fn a_used_key_moves_and_the_message_says_what_moved() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "j");
        assert_eq!(message(&screen), "j moved to Move north (was Move south).");
        let j = KeyBinding::from_text("j").unwrap();
        assert_eq!(b.owner(&j), Some(KeyCommand::Move(Dir::N)));
        // Kick has one key: it is left with none, and the table shows that.
        let old = b.keys_of(KeyCommand::Kick)[0].display();
        select(&mut screen, Row::Command(KeyCommand::Wait));
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, &old);
        assert_eq!(
            message(&screen),
            format!("{old} moved to Wait a turn; Kick has no key.")
        );
        assert_eq!(key_list(&b, KeyCommand::Kick), "(none)");
    }

    #[test]
    fn a_key_moves_only_within_its_context() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        // `w` for Up on the menus leaves play's keys alone, and `k` stays Move north in play.
        select(&mut screen, Row::Command(KeyCommand::MenuUp));
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "w");
        assert_eq!(message(&screen), "Up is now w.");
        let w = KeyBinding::from_text("w").unwrap();
        assert_eq!(b.owner_in(Context::Menu, &w), Some(KeyCommand::MenuUp));
        assert_eq!(b.owner(&w), None);
        assert_eq!(b.keys_of(KeyCommand::MenuUp)[1].display(), "k");
    }

    #[test]
    fn a_fixed_commands_key_is_refused() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        let before = b.0.clone();
        nav(&mut screen, &mut b, Nav::Enter);
        let press = Press {
            nav: Nav::Other,
            binding: Some(KeyBinding::Code(bevy::prelude::KeyCode::Escape)),
            repeat: false,
            text: None,
            shift: false,
        };
        screen.handle(&press, &mut b);
        assert_eq!(b.0, before);
        let message = message(&screen);
        assert!(
            message.starts_with('!') && message.contains("Cancel"),
            "{message}"
        );
    }

    #[test]
    fn escape_cancels_a_capture_without_leaving() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        nav(&mut screen, &mut b, Nav::Enter);
        assert!(!nav(&mut screen, &mut b, Nav::Back));
        assert_eq!(screen.mode, Mode::Browse);
        assert!(nav(&mut screen, &mut b, Nav::Back));
    }

    #[test]
    fn delete_resets_one_command_and_reset_row_resets_all() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        let before = b.to_map();
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "w");
        nav(&mut screen, &mut b, Nav::Erase);
        assert_eq!(b.to_map(), before);
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "w");
        select(&mut screen, Row::Reset);
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(b.to_map(), before);
        select(&mut screen, Row::Back);
        assert!(nav(&mut screen, &mut b, Nav::Enter));
    }

    #[test]
    fn the_selection_skips_headings_and_wraps_around() {
        let (mut screen, mut b) = (ControlsScreen::new(false), Bindings::default());
        let first = screen.selected;
        // Up from the first command wraps to Back, the last row; down again comes back.
        nav(&mut screen, &mut b, Nav::Up);
        assert_eq!(screen.rows()[screen.selected], Row::Back);
        nav(&mut screen, &mut b, Nav::Up);
        assert_eq!(screen.rows()[screen.selected], Row::Reset);
        nav(&mut screen, &mut b, Nav::Down);
        nav(&mut screen, &mut b, Nav::Down);
        assert_eq!(screen.selected, first);
        // Down through a whole context lands on the next one's first command.
        select(&mut screen, Row::Command(KeyCommand::Quit));
        nav(&mut screen, &mut b, Nav::Down);
        assert_eq!(screen.selected_command(), Some(KeyCommand::NextTarget));
        for _ in 0..500 {
            nav(&mut screen, &mut b, Nav::Down);
            assert!(screen.rows()[screen.selected].selectable());
        }
    }

    #[test]
    fn commands_are_grouped_by_context_and_dev_keys_only_where_they_work() {
        let plain = ControlsScreen::new(false).rows();
        let headers: Vec<Context> = (plain.iter())
            .filter_map(|r| match r {
                Row::Header(c) => Some(*c),
                _ => None,
            })
            .collect();
        assert_eq!(headers, Context::ALL[..Context::ALL.len() - 1]);
        assert!(!plain.contains(&Row::Command(KeyCommand::RevealMap)));
        // Every rebindable command but the developer keys, each under its context's heading.
        let mut context = None;
        for row in &plain {
            match row {
                Row::Header(c) => context = Some(*c),
                Row::Command(command) => assert_eq!(Some(command.context()), context),
                _ => {}
            }
        }
        for command in REBINDABLE.iter().filter(|c| c.context() != Context::Dev) {
            assert!(plain.contains(&Row::Command(*command)), "{command:?}");
        }
        let dev = ControlsScreen::new(true).rows();
        assert!(dev.contains(&Row::Header(Context::Dev)));
        assert!(dev.contains(&Row::Command(KeyCommand::RevealMap)));
    }

    #[test]
    fn draws_in_tiny_consoles_with_scrolling() {
        let b = Bindings::default();
        for rows in [40, 14, 6] {
            let mut console = Console::default();
            console.resize(70, rows);
            for mut screen in [ControlsScreen::new(true), ControlsScreen::help(true)] {
                let last = screen.rows().len() - 1;
                for selected in [0, 7, 40, last] {
                    screen.selected = selected;
                    screen.draw(&mut console, &b);
                }
            }
        }
    }

    /// The text of console row `y`.
    fn line(console: &Console, y: i32) -> String {
        (0..console.cols() as i32)
            .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
            .collect()
    }

    #[test]
    fn help_lists_every_command_by_context_and_changes_nothing() {
        let mut b = Bindings::default();
        let mut help = ControlsScreen::help(true);
        let rows = help.rows();
        for extra in [
            KeyCommand::Cancel,
            KeyCommand::RevealMap,
            KeyCommand::Snapshot,
            KeyCommand::MenuUp,
            KeyCommand::Loose,
        ] {
            assert!(rows.contains(&Row::Command(extra)), "{extra:?}");
        }
        assert!(!rows.contains(&Row::Reset) && !rows.contains(&Row::Back));

        let before = b.0.clone();
        for nav in [Nav::Enter, Nav::Erase, Nav::Down, Nav::Enter] {
            assert!(!help.handle(&Press::nav(nav), &mut b));
        }
        assert_eq!(b.0, before);
        assert_eq!(help.mode, Mode::ReadOnly);

        let mut console = Console::default();
        console.resize(80, 130);
        help.selected = 0;
        help.draw(&mut console, &b);
        let text: Vec<String> = (0..130).map(|y| line(&console, y)).collect();
        let has = |needle: &str| text.iter().any(|l| l.contains(needle));
        assert!(
            text.iter()
                .any(|l| l.contains("Move north") && l.contains("k Num8 ↑"))
        );
        assert!(
            text.iter()
                .any(|l| l.contains("Cancel") && l.contains("Esc"))
        );
        for needle in [
            "? or Esc close",
            "Esc always backs out",
            "Letters (a-z)",
            "Menus and screens",
            "Aiming",
        ] {
            assert!(has(needle), "{needle}");
        }
        assert!(!has("│ >"), "nothing is selected");
    }

    #[test]
    fn help_closes_on_the_help_key_or_escape() {
        let mut b = Bindings::default();
        let question = Press {
            nav: Nav::Other,
            binding: KeyBinding::from_text("?"),
            repeat: false,
            text: Some('?'),
            shift: false,
        };
        assert!(ControlsScreen::help(false).handle(&question, &mut b));
        assert!(ControlsScreen::help(false).handle(&Press::nav(Nav::Back), &mut b));
        // The rebinding screen takes `?` as an ordinary key.
        assert!(!ControlsScreen::new(false).handle(&question, &mut b));
    }

    #[test]
    fn help_scrolls_within_the_list() {
        let mut b = Bindings::default();
        let mut help = ControlsScreen::help(false);
        nav(&mut help, &mut b, Nav::Up);
        assert_eq!(help.selected, 0);
        for _ in 0..200 {
            nav(&mut help, &mut b, Nav::Down);
        }
        assert_eq!(help.selected, help.rows().len() - 1);
    }
}
