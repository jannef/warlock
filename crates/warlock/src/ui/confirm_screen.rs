//! The confirmation dialog (spec §6): a question with Yes and No, No selected, shown over the
//! screen that asked, or over the map. Reusable: a screen or gameplay answers
//! `ScreenAction::Confirm` with a [`Question`], and the dialog hands on what the answer does: a
//! menu choice ([`ScreenAction::Choose`]: quitting), a gameplay action ([`ScreenAction::Act`]:
//! paying or selling in a shop), or another question in its place (whether to leave a shop
//! without paying).
//!
//! Keys: the `Context::Confirm` commands Yes and No (`y` and `n` by default; rebindable, in a
//! context of their own because `y` and `n` move the player in play); Esc closes the dialog;
//! ←/→ move the selection and Enter takes it. Selection is shown by text (a `>` marker). A held
//! key's repeats never answer (`ScreenStack::handle`).

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT};
use crate::input::{Bindings, Context, KeyCommand};
use crate::render::console::Console;

/// A yes/no question and what each answer does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub text: String,
    /// What Yes does.
    pub yes: ScreenAction,
    /// What No does: close the dialog (the default), or ask another question.
    pub no: ScreenAction,
}

impl Question {
    /// Asks `text`: Yes does `yes`, No closes the dialog.
    pub fn new(text: impl Into<String>, yes: impl Into<ScreenAction>) -> Self {
        Self {
            text: text.into(),
            yes: yes.into(),
            no: ScreenAction::Close,
        }
    }

    /// No does `no` instead of closing the dialog.
    pub fn or(self, no: impl Into<ScreenAction>) -> Self {
        Self {
            no: no.into(),
            ..self
        }
    }
}

impl From<Question> for ScreenAction {
    fn from(question: Question) -> Self {
        ScreenAction::Confirm(Box::new(question))
    }
}

#[derive(Debug)]
pub struct ConfirmScreen {
    question: Question,
    /// Yes is selected (No, the safe answer, at first).
    yes: bool,
}

impl ConfirmScreen {
    pub fn new(question: Question) -> Self {
        Self {
            question,
            yes: false,
        }
    }

    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let answer = (press.binding.as_ref()).and_then(|k| bindings.owner_in(Context::Confirm, k));
        let (yes, no) = (self.question.yes.clone(), self.question.no.clone());
        match (answer, press.nav) {
            (Some(KeyCommand::Yes), _) => yes,
            (Some(KeyCommand::No), _) => no,
            (_, Nav::Back) => ScreenAction::Close,
            (_, Nav::Enter) if self.yes => yes,
            (_, Nav::Enter) => no,
            (_, Nav::Left | Nav::Right) => {
                self.yes = !self.yes;
                ScreenAction::Stay
            }
            _ => ScreenAction::Stay,
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let lines = wrap(&self.question.text, 36);
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
        panel.footer(console, &footer(bindings));
    }
}

/// The keys, as bound: "y yes  n no  Esc cancel  Enter select" (an unbound answer left out), in
/// the panel's 40 inner cells.
fn footer(bindings: &Bindings) -> String {
    let key = |command| (bindings.keys_of(command).first()).map(|k| k.display());
    let mut parts: Vec<String> = [(KeyCommand::Yes, "yes"), (KeyCommand::No, "no")]
        .into_iter()
        .filter_map(|(command, what)| Some(format!("{} {what}", key(command)?)))
        .collect();
    parts.extend(["Esc cancel".to_string(), "Enter select".to_string()]);
    parts.join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use crate::ui::screens::{MenuChoice, ScreenName};

    fn screen() -> ConfirmScreen {
        ConfirmScreen::new(Question::new("Quit Warlock?", MenuChoice::Quit))
    }

    fn key(ch: &str) -> Press {
        Press {
            binding: Some(KeyBinding::Char(ch.into())),
            text: ch.chars().next(),
            ..Press::nav(Nav::Other)
        }
    }

    #[test]
    fn no_is_the_default_and_cancels() {
        let b = Bindings::default();
        let mut s = screen();
        assert_eq!(s.handle(&Press::nav(Nav::Enter), &b), ScreenAction::Close);
        assert_eq!(s.handle(&Press::nav(Nav::Back), &b), ScreenAction::Close);
        assert_eq!(s.handle(&key("n"), &b), ScreenAction::Close);
    }

    #[test]
    fn yes_proceeds_by_key_or_selection() {
        let b = Bindings::default();
        let yes = ScreenAction::Choose(MenuChoice::Quit);
        assert_eq!(screen().handle(&key("y"), &b), yes);
        let mut s = screen();
        assert_eq!(s.handle(&Press::nav(Nav::Left), &b), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Enter), &b), yes);
        assert_eq!(s.handle(&Press::nav(Nav::Right), &b), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Enter), &b), ScreenAction::Close);
    }

    #[test]
    fn other_keys_do_nothing() {
        let b = Bindings::default();
        let mut s = screen();
        assert_eq!(s.handle(&key("q"), &b), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Up), &b), ScreenAction::Stay);
    }

    #[test]
    fn no_may_ask_another_question_and_esc_still_cancels() {
        let b = Bindings::default();
        let then = Question::new("Really?", ScreenAction::Open(ScreenName::Help));
        let ask = || ConfirmScreen::new(Question::new("Sure?", MenuChoice::Quit).or(then.clone()));
        assert_eq!(ask().handle(&key("n"), &b), then.clone().into());
        assert_eq!(
            ask().handle(&Press::nav(Nav::Enter), &b),
            then.clone().into()
        );
        assert_eq!(
            ask().handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn a_long_question_wraps_inside_the_dialog_over_its_answers() {
        let text = "You owe Grimbold 125 gold and cannot pay it. Leave without paying? Grimbold \
                    will call it theft.";
        let mut console = Console::default();
        console.resize(80, 24);
        ConfirmScreen::new(Question::new(text, MenuChoice::Quit))
            .draw(&mut console, &Bindings::default());
        let rows: Vec<String> = (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect();
        let words: Vec<&str> = (rows.iter())
            .flat_map(|r| r.split_whitespace())
            .filter(|w| *w != "│")
            .collect();
        let asked: Vec<&str> = text.split_whitespace().collect();
        assert!(
            words.windows(asked.len()).any(|w| w == asked.as_slice()),
            "the whole question, in order:\n{}",
            rows.join("\n")
        );
        let answers = rows
            .iter()
            .position(|r| r.contains("Yes") && r.contains("> No"));
        let last_line = rows.iter().rposition(|r| r.contains("theft."));
        assert!(answers > last_line, "{}", rows.join("\n"));
        assert!(
            rows.iter()
                .any(|r| r.contains("y yes  n no  Esc cancel  Enter select"))
        );
        assert!(
            footer(&Bindings::default()).chars().count() <= 40,
            "inside the panel"
        );
    }

    #[test]
    fn the_answers_are_rebindable() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Yes, KeyBinding::Char("j".into()))
            .expect("j is free in the dialog");
        b.bind_primary(KeyCommand::No, KeyBinding::Char("x".into()))
            .expect("x is free in the dialog");
        let mut s = screen();
        assert_eq!(s.handle(&key("y"), &b), ScreenAction::Stay);
        assert_eq!(s.handle(&key("x"), &b), ScreenAction::Close);
        assert_eq!(
            s.handle(&key("j"), &b),
            ScreenAction::Choose(MenuChoice::Quit)
        );
        assert_eq!(footer(&b), "j yes  x no  Esc cancel  Enter select");
        assert_eq!(
            footer(&Bindings::default()),
            "y yes  n no  Esc cancel  Enter select"
        );
    }
}
