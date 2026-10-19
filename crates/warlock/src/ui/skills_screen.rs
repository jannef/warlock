//! The Skills screen (spec §6; `S`, or `S` on the character screen): the player's trained skills
//! (spec §8 "Skills"), in skill order, each with its value and, dimmer, a short meter of how near
//! its next point is (in quarters: `━━──`). Untrained skills are not there: they do not exist for
//! the player. Never a benefit's number, a mark, or a chance: perception has none of them. The
//! selected skill's one-line description shows at the bottom.
//!
//! Read-only: ↑↓ choose a skill; Esc, Enter, or its own key closes it. It is read from the
//! perception snapshot when it opens; no game time passes while it is open, so it cannot go
//! stale.

use super::character_screen::fit_joined;
use super::menu::{Nav, Panel, Press};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::perception::PerceptionView;
use warlock_core::skills::PROGRESS_STEPS;

/// Width of the panel.
const WIDTH: i32 = 52;
/// Width of a skill's name column: the longest built-in name ("Use magic device") and a space.
const NAME_W: i32 = 18;

/// A trained skill as shown.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    name: String,
    value: u16,
    /// Toward the next point, in quarters (0 to 3).
    progress: u8,
    description: String,
}

#[derive(Debug)]
pub struct SkillsScreen {
    rows: Vec<Row>,
    selected: usize,
}

impl SkillsScreen {
    /// The screen for the player's own trained skills (`None` outside a run: none).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let rows = view.map_or(Vec::new(), |view| {
            let content = view.content();
            (view.own_skills().iter())
                .map(|s| {
                    let def = content.skill(s.skill);
                    Row {
                        name: def.name.clone(),
                        value: s.value,
                        progress: s.progress,
                        description: def.description.clone(),
                    }
                })
                .collect()
        });
        Self { rows, selected: 0 }
    }

    /// Applies a key press: ↑↓ choose; Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Skills));
        let n = self.rows.len().max(1);
        match press.nav {
            _ if own_key => return ScreenAction::Close,
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.selected = (self.selected + n - 1) % n,
            Nav::Down => self.selected = (self.selected + 1) % n,
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the skills (or a line saying there are none), a gap, the description
        // (up to two lines), a gap, the footer, frame.
        let width = WIDTH.min(console.cols() as i32);
        let inner = width - 4;
        let list = self.rows.len().max(1) as i32;
        let h = list + 8;
        let panel = Panel::new(console, " Skills ", width, h);
        let x = panel.inner_x();
        let fits = |y: i32| y < panel.y + panel.h - 2;
        // A list taller than the panel scrolls to keep the selection in view.
        let room = (panel.h - 8).max(1) as usize;
        let first = self.selected.saturating_sub(room - 1);
        let mut y = panel.y + 2;
        if self.rows.is_empty() && fits(y) {
            console.print(x, y, "You are not trained in any skill.", TEXT_DIM);
        }
        for (i, row) in self.rows.iter().enumerate().skip(first).take(room) {
            if fits(y) {
                let selected = i == self.selected;
                let color = if selected { ACCENT } else { TEXT };
                console.print(x, y, if selected { ">" } else { " " }, ACCENT);
                console.print(x + 2, y, &row.name, color);
                console.print(x + 2 + NAME_W, y, &format!("{:>3}", row.value), color);
                console.print(x + 2 + NAME_W + 5, y, &meter(row.progress), TEXT_DIM);
            }
            y += 1;
        }
        y = panel.y + panel.h - 5;
        if let Some(row) = self.rows.get(self.selected) {
            let words: Vec<String> = row.description.split(' ').map(String::from).collect();
            for line in fit_joined(&words, " ", inner).iter().take(2) {
                if fits(y) && y > panel.y + 1 {
                    console.print(x, y, line, TEXT_DIM);
                }
                y += 1;
            }
        }
        let footer = match bindings.keys_of(KeyCommand::Skills).first() {
            Some(key) => format!("↑↓ choose   {} or Esc close", key.display()),
            None => "↑↓ choose   Esc close".to_string(),
        };
        panel.footer(console, &footer);
    }
}

/// How near a skill is to its next point, `progress` quarters of the way, as a short meter:
/// `━` for each quarter reached, `─` for the rest (`━━──`). Shape, not color, tells them apart.
pub fn meter(progress: u8) -> String {
    let filled = usize::from(progress.min(PROGRESS_STEPS));
    let rest = usize::from(PROGRESS_STEPS) - filled;
    format!("{}{}", "━".repeat(filled), "─".repeat(rest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn drawn(screen: &SkillsScreen, cols: u32, rows: u32) -> Vec<String> {
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        text(&console)
    }

    #[test]
    fn lists_the_trained_skills_with_their_values() {
        let world = showcase();
        let screen = SkillsScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 80, 30).join("\n");
        // The showcase's human thief: the class's skills, one of them raised by being human.
        for expected in [
            " Skills ",
            "> Appraise           11  ────",
            "  Disable device     16  ────",
            "  Escape artist       7  ────",
            "  Open lock          18  ────",
            "  Search              9  ────",
            "  Spot               10  ────",
            "  Steal              17  ────",
            "  Stealth            16  ────",
            "  Tumble             16  ────",
            "Judging what a thing is worth and what it is",
            "made of.",
            "↑↓ choose   S or Esc close",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        // Untrained skills are not there; neither is a number but the values.
        for hidden in ["Listen", "Athletics", "Swim", "marks", "%", "+"] {
            assert!(!text.contains(hidden), "{hidden:?} in\n{text}");
        }
    }

    #[test]
    fn the_meter_shows_quarters_by_shape() {
        assert_eq!(meter(0), "────");
        assert_eq!(meter(1), "━───");
        assert_eq!(meter(3), "━━━─");
        assert_eq!(meter(9), "━━━━");
    }

    #[test]
    fn choosing_shows_each_description_and_keys_close() {
        let world = showcase();
        let b = Bindings::default();
        let mut screen = SkillsScreen::new(Some(&world.perception()));
        assert_eq!(screen.handle(&Press::nav(Nav::Up), &b), ScreenAction::Stay);
        assert_eq!(screen.selected, 8, "the list wraps");
        let text = drawn(&screen, 80, 30).join("\n");
        assert!(text.contains("> Tumble"), "{text}");
        assert!(text.contains("Rolling, diving, and twisting out of harm's way."));
        screen.handle(&Press::nav(Nav::Down), &b);
        assert_eq!(screen.selected, 0);
        assert_eq!(screen.handle(&Press::typed('a'), &b), ScreenAction::Stay);
        let s = Press {
            binding: KeyBinding::from_text("S"),
            ..Press::typed('S')
        };
        assert_eq!(screen.handle(&s, &b), ScreenAction::Close);
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
    fn says_so_without_skills_and_draws_in_small_consoles() {
        let none = SkillsScreen::new(None);
        let text = drawn(&none, 80, 30).join("\n");
        assert!(text.contains("You are not trained in any skill."), "{text}");
        let world = showcase();
        let mut screen = SkillsScreen::new(Some(&world.perception()));
        for (cols, rows) in [(48, 12), (30, 8), (5, 3), (0, 0)] {
            assert_eq!(drawn(&screen, cols, rows).len(), rows as usize);
        }
        // In a short console the list scrolls to keep the selection in view.
        let b = Bindings::default();
        for _ in 0..8 {
            screen.handle(&Press::nav(Nav::Down), &b);
        }
        let text = drawn(&screen, 60, 13).join("\n");
        assert!(text.contains("> Tumble"), "{text}");
    }
}
