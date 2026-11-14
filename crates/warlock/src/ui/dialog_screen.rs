//! The dialog (spec §6): the one modal question screen, shown over the screen that asked or over
//! the map. Everything that asks first uses it with its own parameters, a [`Dialog`]: a title,
//! the question, and its options, each with a label, a [`Role`], and an outcome (a
//! [`ScreenAction`]: a menu choice such as quitting, a gameplay action such as paying or selling,
//! a follow-up dialog that takes this one's place, or just closing). Quitting (native and web),
//! a shop's way out and its offers are dialogs; so is any new question: build a `Dialog` and
//! answer [`ScreenAction::Dialog`] with it, never a screen of its own.
//!
//! Keys (fixed, `settings::fixed`): the menu keys move the selection (↑/↓, ←/→) and Enter takes
//! it; `y` and `n` pick the option whose role is Accept, and Decline (else Cancel); Esc picks the Cancel option (else Decline), or closes the dialog if there is none;
//! a dialog that must be answered ([`Dialog::required`]: an overworld encounter) ignores Esc.
//! The safe option (Cancel, else Decline) is selected at first. Each option shows its shortcut
//! keys. A held key's repeats never answer (`ScreenStack::handle`).

use super::menu::{HINTS, Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use crate::settings::fixed::{self, Answer};

/// What an option means, for its shortcut keys and the first selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Yes: the Yes key picks it.
    Accept,
    /// No: the No key picks it (and Esc, without a Cancel option).
    Decline,
    /// Back out: Esc picks it (and the No key, without a Decline option). Selected at first.
    Cancel,
    /// Only by selection.
    Plain,
}

/// One option of a dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub role: Role,
    /// What choosing it does.
    pub outcome: ScreenAction,
}

/// A dialog's content: its title (on the frame; may be empty), its question, and its options,
/// in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialog {
    pub title: String,
    pub text: String,
    pub options: Vec<Choice>,
    /// It must be answered: Esc does nothing (an overworld encounter, the owner, #796 Q9).
    pub required: bool,
}

/// The widest the question is wrapped to.
const TEXT_W: usize = 40;

impl Dialog {
    /// A dialog asking `text`, with no options yet.
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            text: text.into(),
            options: Vec::new(),
            required: false,
        }
    }

    /// Makes it a question that must be answered: Esc does nothing, and only an option closes it.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Adds an option.
    pub fn option(
        mut self,
        label: impl Into<String>,
        role: Role,
        outcome: impl Into<ScreenAction>,
    ) -> Self {
        self.options.push(Choice {
            label: label.into(),
            role,
            outcome: outcome.into(),
        });
        self
    }

    /// Adds an option the Yes key picks.
    pub fn accept(self, label: impl Into<String>, outcome: impl Into<ScreenAction>) -> Self {
        self.option(label, Role::Accept, outcome)
    }

    /// Adds an option the No key picks.
    pub fn decline(self, label: impl Into<String>, outcome: impl Into<ScreenAction>) -> Self {
        self.option(label, Role::Decline, outcome)
    }

    /// Adds an option Esc picks.
    pub fn cancel(self, label: impl Into<String>, outcome: impl Into<ScreenAction>) -> Self {
        self.option(label, Role::Cancel, outcome)
    }

    /// Adds an option picked only by selection.
    pub fn plain(self, label: impl Into<String>, outcome: impl Into<ScreenAction>) -> Self {
        self.option(label, Role::Plain, outcome)
    }

    /// The plain yes-or-no question: `yes` (accept) does `outcome`, `no` (cancel) closes.
    pub fn yes_no(
        title: impl Into<String>,
        text: impl Into<String>,
        yes: impl Into<String>,
        outcome: impl Into<ScreenAction>,
        no: impl Into<String>,
    ) -> Self {
        Self::new(title, text)
            .accept(yes, outcome)
            .cancel(no, ScreenAction::Close)
    }

    /// The option the shortcut for `role` picks: Accept the accept option; Decline the decline
    /// option, else the cancel one; Cancel the cancel option, else the decline one.
    fn by_role(&self, role: Role) -> Option<usize> {
        let find = |r: Role| self.options.iter().position(|o| o.role == r);
        match role {
            Role::Accept => find(Role::Accept),
            Role::Decline => find(Role::Decline).or_else(|| find(Role::Cancel)),
            Role::Cancel => find(Role::Cancel).or_else(|| find(Role::Decline)),
            Role::Plain => None,
        }
    }

    /// The keys that pick option `i`: "y", "n, Esc".
    fn keys_of(&self, i: usize) -> Vec<&'static str> {
        let mut keys = Vec::new();
        if self.by_role(Role::Accept) == Some(i) {
            keys.push(fixed::YES);
        }
        if self.by_role(Role::Decline) == Some(i) {
            keys.push(fixed::NO);
        }
        if self.by_role(Role::Cancel) == Some(i) {
            keys.push("Esc");
        }
        keys
    }
}

impl From<Dialog> for ScreenAction {
    fn from(dialog: Dialog) -> Self {
        ScreenAction::Dialog(Box::new(dialog))
    }
}

#[derive(Debug)]
pub struct DialogScreen {
    dialog: Dialog,
    selected: usize,
}

impl DialogScreen {
    pub fn new(dialog: Dialog) -> Self {
        let selected = dialog.by_role(Role::Cancel).unwrap_or(0);
        Self { dialog, selected }
    }

    /// Whether it must be answered ([`Dialog::required`]).
    pub fn must_answer(&self) -> bool {
        self.dialog.required
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let pick = |i: Option<usize>| {
            i.and_then(|i| self.dialog.options.get(i))
                .map(|o| o.outcome.clone())
        };
        let count = self.dialog.options.len();
        let shortcut = (press.binding.as_ref()).and_then(fixed::dialog);
        match (shortcut, press.nav) {
            (Some(Answer::Yes), _) => pick(self.dialog.by_role(Role::Accept)),
            (Some(Answer::No), _) => pick(self.dialog.by_role(Role::Decline)),
            (_, Nav::Back) if self.dialog.required => None,
            (_, Nav::Back) => pick(self.dialog.by_role(Role::Cancel)).or(Some(ScreenAction::Close)),
            (_, Nav::Enter) => pick(Some(self.selected)).or(Some(ScreenAction::Close)),
            (_, Nav::Up | Nav::Left) if count > 0 => {
                self.selected = (self.selected + count - 1) % count;
                None
            }
            (_, Nav::Down | Nav::Right) if count > 0 => {
                self.selected = (self.selected + 1) % count;
                None
            }
            _ => None,
        }
        .unwrap_or(ScreenAction::Stay)
    }

    /// Option rows as drawn: the label, then its keys in parentheses.
    fn rows(&self) -> Vec<String> {
        (self.dialog.options.iter().enumerate())
            .map(|(i, o)| match self.dialog.keys_of(i).join(", ") {
                keys if keys.is_empty() => o.label.clone(),
                keys => format!("{}  ({keys})", o.label),
            })
            .collect()
    }

    /// Draws the dialog sized to fit its question, its options, and the footer: the question
    /// centered, the options in a column under it (`>` marks the selection).
    pub fn draw(&self, console: &mut Console) {
        let lines = wrap(&self.dialog.text, TEXT_W as i32);
        let rows = self.rows();
        let keys = HINTS;
        let footer = match rows.len() {
            0 | 1 => format!("{} select", keys.confirm),
            _ => format!("{}/{} choose  {} select", keys.up, keys.down, keys.confirm),
        };
        let footer = footer.as_str();
        let width = |s: &str| s.chars().count();
        let title = if self.dialog.title.is_empty() {
            String::new()
        } else {
            format!(" {} ", self.dialog.title)
        };
        let option_w = rows.iter().map(|r| width(r) + 2).max().unwrap_or(0);
        let inner = (lines.iter().map(|l| width(l)))
            .chain([option_w, width(footer), width(&title) + 2, 24])
            .max()
            .unwrap_or(24) as i32;
        let h = lines.len() as i32 + rows.len() as i32 + 6;
        let panel = Panel::new(console, &title, inner + 4, h);
        let center = |len: usize| panel.x + (panel.w - len as i32) / 2;
        for (i, line) in lines.iter().enumerate() {
            console.print(center(width(line)), panel.y + 2 + i as i32, line, TEXT);
        }
        let x = center(option_w);
        let top = panel.y + 3 + lines.len() as i32;
        for (i, row) in rows.iter().enumerate() {
            let selected = i == self.selected;
            let y = top + i as i32;
            console.print(x, y, if selected { ">" } else { " " }, ACCENT);
            console.print(x + 2, y, row, if selected { ACCENT } else { TEXT });
        }
        console.print(panel.inner_x(), panel.y + panel.h - 2, footer, TEXT_DIM);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use crate::ui::screens::{MenuChoice, ScreenName};

    fn quit() -> Dialog {
        Dialog::yes_no("Quit", "Quit Warlock?", "Quit", MenuChoice::Quit, "Stay")
    }

    fn key(ch: &str) -> Press {
        Press {
            binding: Some(KeyBinding::Char(ch.into())),
            text: ch.chars().next(),
            ..Press::nav(Nav::Other)
        }
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn the_safe_option_is_the_default_and_esc_and_no_pick_it() {
        let mut s = DialogScreen::new(quit());
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), ScreenAction::Close);
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
        assert_eq!(s.handle(&key("n")), ScreenAction::Close);
    }

    #[test]
    fn yes_or_the_selection_takes_the_accept_option() {
        let yes = ScreenAction::Choose(MenuChoice::Quit);
        assert_eq!(DialogScreen::new(quit()).handle(&key("y")), yes);
        let mut s = DialogScreen::new(quit());
        assert_eq!(s.handle(&Press::nav(Nav::Up)), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), yes);
        assert_eq!(s.handle(&Press::nav(Nav::Right)), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), ScreenAction::Close);
    }

    #[test]
    fn other_keys_do_nothing() {
        let mut s = DialogScreen::new(quit());
        assert_eq!(s.handle(&key("q")), ScreenAction::Stay);
        assert_eq!(s.handle(&key("1")), ScreenAction::Stay);
    }

    #[test]
    fn a_required_dialog_ignores_esc_and_is_answered_by_an_option() {
        let dialog = Dialog::new("Kobolds", "They have not seen you.")
            .plain("Lie in wait", MenuChoice::Quit)
            .plain("Stand and fight", MenuChoice::SaveToMenu)
            .required();
        let mut s = DialogScreen::new(dialog);
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Stay);
        assert_eq!(s.handle(&key("n")), ScreenAction::Stay);
        assert_eq!(
            s.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::Quit)
        );
    }

    #[test]
    fn options_are_parameters_with_roles_and_any_outcome() {
        let then = Dialog::yes_no("Sure", "Really?", "Yes", MenuChoice::Quit, "No");
        let dialog = Dialog::new("Three ways", "Which?")
            .accept("First", ScreenAction::Open(ScreenName::Help))
            .plain("Second", then.clone())
            .decline("Third", MenuChoice::SaveToMenu);
        let ask = || DialogScreen::new(dialog.clone());
        assert_eq!(
            ask().handle(&key("y")),
            ScreenAction::Open(ScreenName::Help)
        );
        assert_eq!(
            ask().handle(&key("n")),
            ScreenAction::Choose(MenuChoice::SaveToMenu)
        );
        // Without a Cancel option Esc declines, and the Decline option is selected first.
        assert_eq!(
            ask().handle(&Press::nav(Nav::Back)),
            ScreenAction::Choose(MenuChoice::SaveToMenu)
        );
        let mut s = ask();
        s.handle(&Press::nav(Nav::Up));
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), ScreenAction::from(then));
        // A plain option has no shortcut, and with no options at all Enter and Esc close.
        let mut empty = DialogScreen::new(Dialog::new("", "Nothing to choose."));
        assert_eq!(empty.handle(&key("y")), ScreenAction::Stay);
        assert_eq!(empty.handle(&Press::nav(Nav::Enter)), ScreenAction::Close);
        assert_eq!(empty.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn the_shortcuts_are_fixed_and_shown_beside_their_options() {
        assert_eq!(
            DialogScreen::new(quit()).rows(),
            ["Quit  (y)", "Stay  (n, Esc)"]
        );
        let mut s = DialogScreen::new(quit());
        assert_eq!(s.handle(&key("x")), ScreenAction::Stay);
        assert_eq!(s.handle(&key("n")), ScreenAction::Close);
        assert_eq!(s.handle(&key("y")), ScreenAction::Choose(MenuChoice::Quit));
    }

    #[test]
    fn the_dialog_sizes_itself_to_its_question_and_options() {
        let long = "You owe Grimbold 125 gold and have only 60. Put something back, or bring \
                    more gold.";
        let dialog = Dialog::new("Grimbold's General Store", long)
            .accept("Pay 125 gold and leave", MenuChoice::Quit)
            .plain("Leave without paying", MenuChoice::Quit)
            .cancel("Stay", ScreenAction::Close);
        let mut console = Console::default();
        console.resize(80, 24);
        DialogScreen::new(dialog).draw(&mut console);
        let rows = text(&console);
        let shown = rows.join("\n");
        let words: Vec<&str> = (rows.iter())
            .flat_map(|r| r.split_whitespace())
            .filter(|w| *w != "│")
            .collect();
        let asked: Vec<&str> = long.split_whitespace().collect();
        assert!(
            words.windows(asked.len()).any(|w| w == asked.as_slice()),
            "{shown}"
        );
        assert!(shown.contains(" Grimbold's General Store "), "{shown}");
        for option in [
            "Pay 125 gold and leave  (y)",
            "Leave without paying",
            "> Stay  (n, Esc)",
        ] {
            assert!(shown.contains(option), "{option}:\n{shown}");
        }
        assert!(shown.contains("↑/↓ choose  Enter select"), "{shown}");
        // Everything inside the frame: each row's text between its two borders.
        for row in rows.iter().filter(|r| r.contains('│')) {
            let inside: Vec<_> = row.match_indices('│').collect();
            assert_eq!(inside.len(), 2, "{row}");
        }
    }
}
