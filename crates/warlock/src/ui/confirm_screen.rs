//! The confirmation dialog (spec §6): a question with Yes and No, No selected, shown over the
//! screen that asked. Reusable: any screen answers `ScreenAction::Confirm(question, choice)` and
//! the dialog, on Yes, hands `choice` on as [`ScreenAction::Choose`].
//!
//! Keys: `y` is Yes; `n` and Esc are No; ←/→ move the selection and Enter takes it. Like the
//! rest prompt's letters, the answers are fixed rather than rebindable (`y` and `n` are movement
//! keys in play, so they cannot be shared with the binding table). Selection is shown by text (a
//! `>` marker). A held key's repeats never answer (`ScreenStack::handle`).

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction};
use super::{ACCENT, TEXT};
use crate::render::console::Console;

#[derive(Debug)]
pub struct ConfirmScreen {
    question: &'static str,
    choice: MenuChoice,
    /// Yes is selected (No, the safe answer, at first).
    yes: bool,
}

impl ConfirmScreen {
    pub fn new(question: &'static str, choice: MenuChoice) -> Self {
        Self {
            question,
            choice,
            yes: false,
        }
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let yes = ScreenAction::Choose(self.choice.clone());
        match (press.nav, press.text) {
            (Nav::Back, _) | (_, Some('n' | 'N')) => ScreenAction::Close,
            (_, Some('y' | 'Y')) => yes,
            (Nav::Enter, _) if self.yes => yes,
            (Nav::Enter, _) => ScreenAction::Close,
            (Nav::Left | Nav::Right, _) => {
                self.yes = !self.yes;
                ScreenAction::Stay
            }
            _ => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console) {
        let lines = wrap(self.question, 36);
        let panel = Panel::new(console, "", 44, 8 + lines.len() as i32);
        for (i, line) in lines.iter().enumerate() {
            let x = panel.x + (panel.w - line.chars().count() as i32) / 2;
            console.print(x, panel.y + 2 + i as i32, line, TEXT);
        }
        let y = panel.y + 3 + lines.len() as i32;
        for (x, label, selected) in [
            (panel.x + 10, "Yes", self.yes),
            (panel.x + panel.w - 18, "No", !self.yes),
        ] {
            let color = if selected { ACCENT } else { TEXT };
            console.print(x, y, if selected { ">" } else { " " }, ACCENT);
            console.print(x + 2, y, label, color);
        }
        panel.footer(console, "y yes   n/Esc no   Enter select");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> ConfirmScreen {
        ConfirmScreen::new("Quit Warlock?", MenuChoice::Quit)
    }

    #[test]
    fn no_is_the_default_and_cancels() {
        let mut s = screen();
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), ScreenAction::Close);
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
        assert_eq!(s.handle(&Press::typed('n')), ScreenAction::Close);
    }

    #[test]
    fn yes_proceeds_by_key_or_selection() {
        let yes = ScreenAction::Choose(MenuChoice::Quit);
        assert_eq!(screen().handle(&Press::typed('y')), yes);
        let mut s = screen();
        assert_eq!(s.handle(&Press::nav(Nav::Left)), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), yes);
        assert_eq!(s.handle(&Press::nav(Nav::Right)), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), ScreenAction::Close);
    }

    #[test]
    fn other_keys_do_nothing() {
        let mut s = screen();
        assert_eq!(s.handle(&Press::typed('q')), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Up)), ScreenAction::Stay);
    }
}
