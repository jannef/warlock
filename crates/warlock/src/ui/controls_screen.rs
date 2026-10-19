//! The controls screen and rebinding UI (spec §6): select a command, press a key to bind it.
//!
//! Enter on a command waits for a key, which becomes the command's primary key (its other keys,
//! such as the numpad, stay). A key another command uses is refused and the owner is named, so
//! the table never holds a conflict. Esc cancels, Delete resets one command, and "Reset all
//! keys" restores the defaults.
//!
//! The `?` help is the same screen, read-only ([`ControlsScreen::help`]): it lists every command
//! in the binding table, including Esc and the dev keys, and closes on `?` or Esc. The list is
//! drawn by one piece of code for both, so help always matches the bindings.

use super::menu::{Nav, Panel, Press};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use crate::settings::keys::REBINDABLE;

/// What the screen is doing.
#[derive(Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Browse,
    /// Waiting for a key for the command at this row.
    Capture,
    /// The `?` help: nothing can be changed; `selected` is the first row shown.
    ReadOnly,
}

/// One row of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Command(KeyCommand),
    Reset,
    Back,
}

/// Selection state of the controls screen. Rows are the rebindable commands, then "Reset all
/// keys", then "Back"; in help, every bound command (see [`ControlsScreen::rows`]).
#[derive(Debug, Default)]
pub struct ControlsScreen {
    selected: usize,
    mode: Mode,
    /// Result of the last action; a leading `!` marks a refusal (never color alone).
    message: Option<String>,
}

const RESET_ROW: usize = REBINDABLE.len();
const BACK_ROW: usize = REBINDABLE.len() + 1;
const ROW_COUNT: usize = REBINDABLE.len() + 2;

impl ControlsScreen {
    /// The `?` help: every bound command, read-only.
    pub fn help() -> Self {
        Self {
            mode: Mode::ReadOnly,
            ..Self::default()
        }
    }

    pub fn is_help(&self) -> bool {
        self.mode == Mode::ReadOnly
    }

    /// The rows shown. Help lists every command in the table: the rebindable ones in their usual
    /// order, then the fixed and dev keys.
    fn rows(&self, bindings: &Bindings) -> Vec<Row> {
        if self.mode != Mode::ReadOnly {
            let commands = REBINDABLE.iter().copied().map(Row::Command);
            return commands.chain([Row::Reset, Row::Back]).collect();
        }
        let mut commands = REBINDABLE.to_vec();
        for (_, command) in &bindings.0 {
            if !commands.contains(command) {
                commands.push(*command);
            }
        }
        commands.into_iter().map(Row::Command).collect()
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
        match press.nav {
            Nav::Back => return true,
            Nav::Up => {
                self.selected = (self.selected + ROW_COUNT - 1) % ROW_COUNT;
                self.message = None;
            }
            Nav::Down => {
                self.selected = (self.selected + 1) % ROW_COUNT;
                self.message = None;
            }
            Nav::Enter if !press.repeat => match self.selected {
                RESET_ROW => {
                    bindings.reset_all();
                    self.message = Some("All keys are back to their defaults.".into());
                }
                BACK_ROW => return true,
                _ => {
                    self.mode = Mode::Capture;
                    self.message = None;
                }
            },
            Nav::Delete | Nav::Backspace if self.selected < REBINDABLE.len() => {
                let command = REBINDABLE[self.selected];
                bindings.reset_command(command);
                self.message = Some(format!("{} is back to its default keys.", command.name()));
            }
            _ => {}
        }
        false
    }

    /// Help: the arrows scroll; Esc or the help key (`?`) closes.
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
                let last = self.rows(bindings).len().saturating_sub(1);
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
            self.message = Some("Cancelled.".into());
            return;
        }
        let Some(key) = press.binding.clone() else {
            self.message =
                Some("! That key cannot be bound. Try another, or Esc to cancel.".into());
            return;
        };
        let command = REBINDABLE[self.selected];
        let shown = key.display();
        self.mode = Mode::Browse;
        self.message = Some(match bindings.bind_primary(command, key) {
            Ok(()) => format!("{} is now {shown}.", command.name()),
            Err(conflict) => format!(
                "! {shown} is already used by {}. Rebind that first.",
                conflict.owner.name()
            ),
        });
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let rows = self.rows(bindings);
        let count = rows.len();
        let read_only = self.mode == Mode::ReadOnly;
        // Help has no messages, so it needs fewer spare rows.
        let spare = if read_only { 6 } else { 8 };
        let panel = Panel::new(console, " Controls ", 60, count as i32 + spare);
        // Leave room for the message and footer below the list.
        let visible = (panel.body_rows() - 2).max(1) as usize;
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
                Row::Reset | Row::Back => None,
            })
            .max()
            .unwrap_or(0) as i32;
        for (i, row) in rows.iter().enumerate().skip(first).take(visible) {
            let y = panel.y + 2 + (i - first) as i32;
            // Help has nothing to select.
            let selected = !read_only && i == self.selected;
            match row {
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
        let status_y = panel.y + panel.h - 3;
        if self.mode == Mode::Capture {
            let name = REBINDABLE[self.selected].name();
            console.print(
                panel.inner_x(),
                status_y,
                &format!("Press a key for \"{name}\". Esc cancels."),
                ACCENT,
            );
        } else if let Some(message) = &self.message {
            let color = if message.starts_with('!') {
                ACCENT
            } else {
                TEXT
            };
            console.print(panel.inner_x(), status_y, message, color);
        }
        let footer = if read_only {
            let close = match bindings.keys_of(KeyCommand::Help).first() {
                Some(key) => format!("{} or Esc close", key.display()),
                None => "Esc close".to_string(),
            };
            let scroll = if first > 0 || more {
                "↑↓ scroll   "
            } else {
                ""
            };
            format!("{scroll}{close}")
        } else {
            "Enter rebind   Del reset   Esc back".to_string()
        };
        panel.footer(console, &footer);
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

    #[test]
    fn rebinding_replaces_the_primary_key() {
        let (mut screen, mut b) = (ControlsScreen::default(), Bindings::default());
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(screen.mode, Mode::Capture);
        key(&mut screen, &mut b, "w");
        assert_eq!(screen.mode, Mode::Browse);
        assert_eq!(b.keys_of(KeyCommand::Move(Dir::N))[0].display(), "w");
        assert!(
            screen
                .message
                .as_deref()
                .is_some_and(|m| m.contains("now w"))
        );
    }

    #[test]
    fn a_used_key_is_refused_and_names_its_owner() {
        let (mut screen, mut b) = (ControlsScreen::default(), Bindings::default());
        let before = b.0.clone();
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "j");
        assert_eq!(b.0, before);
        let message = screen.message.expect("a message");
        assert!(
            message.starts_with('!') && message.contains("Move south"),
            "{message}"
        );
    }

    #[test]
    fn escape_cancels_a_capture_without_leaving() {
        let (mut screen, mut b) = (ControlsScreen::default(), Bindings::default());
        nav(&mut screen, &mut b, Nav::Enter);
        assert!(!nav(&mut screen, &mut b, Nav::Back));
        assert_eq!(screen.mode, Mode::Browse);
        assert!(nav(&mut screen, &mut b, Nav::Back));
    }

    #[test]
    fn delete_resets_one_command_and_reset_row_resets_all() {
        let (mut screen, mut b) = (ControlsScreen::default(), Bindings::default());
        let before = b.to_map();
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "w");
        nav(&mut screen, &mut b, Nav::Delete);
        assert_eq!(b.to_map(), before);
        nav(&mut screen, &mut b, Nav::Enter);
        key(&mut screen, &mut b, "w");
        screen.selected = RESET_ROW;
        nav(&mut screen, &mut b, Nav::Enter);
        assert_eq!(b.to_map(), before);
        screen.selected = BACK_ROW;
        assert!(nav(&mut screen, &mut b, Nav::Enter));
    }

    #[test]
    fn draws_in_tiny_consoles_with_scrolling() {
        let b = Bindings::default();
        for rows in [40, 14, 6] {
            let mut console = Console::default();
            console.resize(70, rows);
            for mut screen in [ControlsScreen::default(), ControlsScreen::help()] {
                for selected in [0, 7, BACK_ROW, 40] {
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
    fn help_lists_every_bound_command_and_changes_nothing() {
        let mut b = Bindings::default();
        b.add_dev_keys();
        let mut help = ControlsScreen::help();
        let rows = help.rows(&b);
        assert_eq!(
            rows[..REBINDABLE.len()],
            REBINDABLE
                .iter()
                .copied()
                .map(Row::Command)
                .collect::<Vec<_>>()[..]
        );
        for extra in [
            KeyCommand::Cancel,
            KeyCommand::RevealMap,
            KeyCommand::Snapshot,
        ] {
            assert!(rows.contains(&Row::Command(extra)), "{extra:?}");
        }
        assert!(!rows.contains(&Row::Reset) && !rows.contains(&Row::Back));

        let before = b.0.clone();
        for nav in [Nav::Enter, Nav::Delete, Nav::Down, Nav::Enter] {
            assert!(!help.handle(&Press::nav(nav), &mut b));
        }
        assert_eq!(b.0, before);
        assert_eq!(help.mode, Mode::ReadOnly);

        let mut console = Console::default();
        console.resize(80, 70);
        help.selected = 0;
        help.draw(&mut console, &b);
        let text: Vec<String> = (0..70).map(|y| line(&console, y)).collect();
        assert!(
            text.iter()
                .any(|l| l.contains("Move north") && l.contains("k Num8 ↑"))
        );
        assert!(
            text.iter()
                .any(|l| l.contains("Cancel") && l.contains("Esc"))
        );
        assert!(text.iter().any(|l| l.contains("? or Esc close")));
        assert!(
            !text.iter().any(|l| l.contains("│ >")),
            "nothing is selected"
        );
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
        assert!(ControlsScreen::help().handle(&question, &mut b));
        assert!(ControlsScreen::help().handle(&Press::nav(Nav::Back), &mut b));
        // The rebinding screen takes `?` as an ordinary key.
        assert!(!ControlsScreen::default().handle(&question, &mut b));
    }

    #[test]
    fn help_scrolls_within_the_list() {
        let mut b = Bindings::default();
        let mut help = ControlsScreen::help();
        nav(&mut help, &mut b, Nav::Up);
        assert_eq!(help.selected, 0);
        for _ in 0..100 {
            nav(&mut help, &mut b, Nav::Down);
        }
        assert_eq!(help.selected, help.rows(&b).len() - 1);
    }
}
