//! The controls screen and rebinding UI (spec §6): select a command, press a key to bind it.
//!
//! Enter on a command waits for a key, which becomes the command's primary key (its other keys,
//! such as the numpad, stay). A key another command uses is refused and the owner is named, so
//! the table never holds a conflict. Esc cancels, Delete resets one command, and "Reset all
//! keys" restores the defaults. The `?` overlay reads the same table, so it follows.

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
}

/// Selection state of the controls screen. Rows are the rebindable commands, then "Reset all
/// keys", then "Back".
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
    /// Applies a key press to `bindings`. Returns true when the player leaves the screen.
    pub fn handle(&mut self, press: &Press, bindings: &mut Bindings) -> bool {
        if self.mode == Mode::Capture {
            self.capture(press, bindings);
            return false;
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
        let panel = Panel::new(console, " Controls ", 60, ROW_COUNT as i32 + 8);
        // Leave room for the message and footer below the list.
        let visible = (panel.body_rows() - 2).max(1) as usize;
        let first = self
            .selected
            .saturating_sub(visible - 1)
            .min(ROW_COUNT.saturating_sub(visible));
        let label_w = REBINDABLE
            .iter()
            .map(|c| c.name().chars().count())
            .max()
            .unwrap_or(0) as i32;
        for (line, row) in (first..ROW_COUNT).take(visible).enumerate() {
            let y = panel.y + 2 + line as i32;
            let selected = row == self.selected;
            if let Some(command) = REBINDABLE.get(row) {
                let keys = key_list(bindings, *command);
                let value = if selected && self.mode == Mode::Capture {
                    "press a key...".to_string()
                } else {
                    keys
                };
                panel.row(console, y, selected, &command.name(), &value, label_w);
            } else {
                let label = if row == RESET_ROW {
                    "Reset all keys"
                } else {
                    "Back"
                };
                panel.row(console, y, selected, label, "", label_w);
            }
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        if first + visible < ROW_COUNT {
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
        panel.footer(console, "Enter rebind   Del reset   Esc back");
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
            let mut screen = ControlsScreen::default();
            for selected in [0, 7, BACK_ROW] {
                screen.selected = selected;
                screen.draw(&mut console, &b);
            }
        }
    }
}
