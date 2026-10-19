//! The quest log (spec §6 "Quest log"; `q`, or its number among the info pages): the quests the
//! player has taken up, then those they have done, by name. The selected one's details show
//! below: what it asks, how far it has got ("2 / 3 killed.", "Return to the old hermit."), and
//! who gave it, in their words. Offers not yet answered are not here.
//!
//! Read-only (quests are not abandoned, and there is no cap on how many are active: the owner's
//! decision): ↑↓ choose; Esc, Enter, or its own key closes it. A list taller than the panel
//! scrolls. It is read from the perception snapshot when it opens; no game time passes while it
//! is open.

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::perception::PerceptionView;
use warlock_core::world::QuestView;

/// Width of the panel: the dialogue screen's.
const WIDTH: i32 = 64;
/// The most lines of the giver's words shown.
const WORDS: usize = 6;

#[derive(Debug)]
pub struct QuestScreen {
    /// Active quests first, then done ones.
    quests: Vec<QuestView>,
    selected: usize,
}

impl QuestScreen {
    /// The log of the player's quests (`None` outside a run: none).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        Self {
            quests: view.map_or_else(Vec::new, PerceptionView::quests),
            selected: 0,
        }
    }

    /// Applies a key press: ↑↓ choose; Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let key = press.binding.as_ref();
        let own_key = key.is_some_and(|k| bindings.owner(k) == Some(KeyCommand::Quests));
        let n = self.quests.len().max(1);
        match press.nav {
            _ if own_key => return ScreenAction::Close,
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.selected = (self.selected + n - 1) % n,
            Nav::Down => self.selected = (self.selected + 1) % n,
            _ => {}
        }
        ScreenAction::Stay
    }

    /// The lines of the list: a heading before the active quests and before the done ones, then
    /// each quest's name, with the quest's index.
    fn list(&self) -> Vec<(Option<usize>, String)> {
        let mut lines = Vec::new();
        let mut heading = None;
        for (i, quest) in self.quests.iter().enumerate() {
            if heading != Some(quest.active) {
                heading = Some(quest.active);
                let text = if quest.active { "Active" } else { "Done" };
                lines.push((None, text.to_string()));
            }
            lines.push((Some(i), quest.name.clone()));
        }
        lines
    }

    /// The selected quest's details, wrapped to `width`: what it asks, how far it has got, a
    /// gap, and who gave it in their words (at most [`WORDS`] lines).
    fn details(&self, width: i32) -> Vec<(String, bool)> {
        let Some(quest) = self.quests.get(self.selected) else {
            return Vec::new();
        };
        let mut lines: Vec<(String, bool)> = Vec::new();
        lines.extend(wrap(&quest.objective, width).into_iter().map(|l| (l, true)));
        lines.extend(wrap(&quest.status, width).into_iter().map(|l| (l, true)));
        lines.push((String::new(), false));
        let words = format!("{}: \"{}\"", capitalized(&quest.giver), quest.offer);
        let mut said = wrap(&words, width);
        if said.len() > WORDS {
            said.truncate(WORDS);
            if let Some(last) = said.last_mut() {
                last.push_str(" …");
            }
        }
        lines.extend(said.into_iter().map(|l| (l, false)));
        lines
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let width = WIDTH.min(console.cols() as i32);
        let inner = width - 4;
        let list = self.list();
        let details = self.details(inner);
        // Rows: margin, the list (or a line saying there is none), a gap, the details, a gap,
        // the footer, the frame.
        let body = list.len().max(1) as i32 + 1 + details.len() as i32;
        let panel = Panel::new(console, " Quests ", WIDTH, body + 5);
        info_tabs_line(console, &panel, bindings);
        let x = panel.inner_x();
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        if list.is_empty() && fits(y) {
            console.print(x, y, "You have taken up no quests.", TEXT_DIM);
            y += 1;
        }
        // A list taller than the panel scrolls to keep the selection in view.
        let room = (panel.h - 6 - details.len() as i32).max(1) as usize;
        let at = list.iter().position(|(i, _)| *i == Some(self.selected));
        let first = at.unwrap_or(0).saturating_sub(room - 1);
        for (i, text) in list.iter().skip(first).take(room) {
            if fits(y) {
                match i {
                    None => {
                        console.print(x, y, text, TEXT_DIM);
                    }
                    Some(i) => {
                        let selected = *i == self.selected;
                        let color = if selected { ACCENT } else { TEXT };
                        console.print(x, y, if selected { ">" } else { " " }, ACCENT);
                        console.print(x + 2, y, text, color);
                    }
                }
            }
            y += 1;
        }
        y += 1;
        for (line, strong) in &details {
            if fits(y) {
                console.print(x, y, line, if *strong { TEXT } else { TEXT_DIM });
            }
            y += 1;
        }
        panel.footer(console, &self.footer(bindings));
    }

    /// The footer: the keys.
    fn footer(&self, bindings: &Bindings) -> String {
        let close = match (bindings.keys_of(KeyCommand::Quests).first()).map(|k| k.display()) {
            Some(own) => format!("{own} or Esc close"),
            None => "Esc close".to_string(),
        };
        format!("↑↓ choose   {close}")
    }
}

/// The info pages' tab line on the panel's bottom border.
fn info_tabs_line(console: &mut Console, panel: &Panel, bindings: &Bindings) {
    super::info_tabs::draw_tabs(console, panel, bindings, ScreenName::Quests);
}

/// `text` with its first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::content::Content;

    /// The showcase beside the old hermit, with the rats taken up and the apples' quest done.
    fn with_quests() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("run");
        let walk = "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3 \
                    ask:SE:quest-rats ask:SE:accept-rats";
        for action in parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        world
    }

    fn text(console: &Console) -> String {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn drawn(screen: &QuestScreen, cols: u32, rows: u32) -> String {
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        text(&console)
    }

    fn typed(key: &str) -> Press {
        Press {
            binding: KeyBinding::from_text(key),
            ..Press::typed(key.chars().next().expect("a key"))
        }
    }

    #[test]
    fn it_lists_the_quests_and_shows_the_selected_ones_details() {
        let world = with_quests();
        let screen = QuestScreen::new(Some(&world.perception()));
        let shown = drawn(&screen, 80, 30);
        for expected in [
            " Quests ",
            "Active",
            "> Rats in the arena",
            "Kill 3 giant rats in the Arena.",
            "0 / 3 killed.",
            "The old hermit: \"Here we don't fight.",
            "↑↓ choose   q or Esc close",
            "6 Quests",
        ] {
            assert!(shown.contains(expected), "{expected:?} in\n{shown}");
        }
        assert!(!shown.contains("Done"), "nothing done yet:\n{shown}");
    }

    #[test]
    fn keys_choose_and_close() {
        let world = with_quests();
        let b = Bindings::default();
        let mut screen = QuestScreen::new(Some(&world.perception()));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Down), &b),
            ScreenAction::Stay
        );
        assert_eq!(screen.selected, 0, "one quest: the list wraps onto it");
        assert_eq!(screen.handle(&typed("q"), &b), ScreenAction::Close);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn done_quests_follow_the_active_ones() {
        let world = with_quests();
        let mut quests = world.perception().quests();
        let mut done = quests[0].clone();
        (done.name, done.active, done.status) = ("Three apples".into(), false, "Done.".into());
        quests.push(done);
        let b = Bindings::default();
        let mut screen = QuestScreen {
            quests,
            selected: 0,
        };
        let shown = drawn(&screen, 80, 30);
        let at = |what: &str| {
            shown
                .find(what)
                .unwrap_or_else(|| panic!("{what:?}:\n{shown}"))
        };
        assert!(at("Active") < at("Rats in the arena"));
        assert!(at("Rats in the arena") < at("Done"));
        assert!(at("Done") < at("Three apples"));
        screen.handle(&Press::nav(Nav::Down), &b);
        let shown = drawn(&screen, 80, 30);
        assert!(shown.contains("> Three apples"), "{shown}");
        assert!(shown.contains("↑↓ choose   q or Esc close"), "{shown}");
    }

    #[test]
    fn many_quests_scroll_within_the_panel() {
        // No cap on active quests: a long log keeps the selection in view and its frame whole.
        let world = with_quests();
        let one = world.perception().quests()[0].clone();
        let quests: Vec<QuestView> = (0..60)
            .map(|i| QuestView {
                name: format!("Errand {i}"),
                active: i < 40,
                ..one.clone()
            })
            .collect();
        let b = Bindings::default();
        let mut screen = QuestScreen {
            quests,
            selected: 0,
        };
        for _ in 0..55 {
            screen.handle(&Press::nav(Nav::Down), &b);
        }
        for (cols, rows) in [(80, 30), (80, 24), (48, 12), (30, 8)] {
            let shown = drawn(&screen, cols, rows);
            assert_eq!(shown.lines().count(), rows as usize);
            if rows >= 24 {
                assert!(shown.contains("> Errand 55"), "{shown}");
                assert!(
                    !shown.contains("  Errand 0 "),
                    "scrolled past the first:\n{shown}"
                );
            }
        }
        screen.handle(&Press::nav(Nav::Up), &b);
        assert_eq!(screen.selected, 54);
    }

    #[test]
    fn it_draws_in_small_consoles() {
        let world = with_quests();
        let screen = QuestScreen::new(Some(&world.perception()));
        for (cols, rows) in [(48, 12), (30, 8), (5, 3), (0, 0)] {
            let shown = drawn(&screen, cols, rows);
            assert_eq!(shown.lines().count().max(1), (rows as usize).max(1));
        }
        let none = QuestScreen::new(None);
        assert!(drawn(&none, 80, 30).contains("You have taken up no quests."));
    }
}
