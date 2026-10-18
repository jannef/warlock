//! The character screen (spec §6; `@`): who the player is, as far as they know it. Their name,
//! their score in each stat (spec §8 "Stats") with its abbreviation and full name, and what they
//! weigh. Never a modifier or a potential: perception has neither.
//!
//! Read-only: Esc, Enter, or its own key closes it. It is read from the perception snapshot when
//! it opens; no game time passes while it is open, so it cannot go stale.

use super::menu::{Nav, Panel, Press};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::object::{Weight, format_weight};
use warlock_core::perception::PerceptionView;
use warlock_core::stats::{PerStat, Stat};

/// Width of the stat name column: the longest name and a space.
const NAME_W: i32 = 14;

#[derive(Debug)]
pub struct CharacterScreen {
    name: String,
    scores: PerStat<u8>,
    weight: Weight,
}

impl CharacterScreen {
    /// The screen for what the player perceives of themselves (`None` outside a run: average
    /// scores and no weight).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some(view) = view else {
            return Self {
                name: "You".into(),
                scores: PerStat::splat(warlock_core::stats::AVERAGE),
                weight: Weight::default(),
            };
        };
        let content = view.content();
        let name = content
            .creature_id("player")
            .map_or("you", |id| content.creature(id).name.as_str());
        Self {
            name: capitalized(name),
            scores: view.own_scores(),
            weight: view.own_weight(),
        }
    }

    /// Applies a key press: Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Character));
        if own_key || matches!(press.nav, Nav::Back | Nav::Enter) {
            ScreenAction::Close
        } else {
            ScreenAction::Stay
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the name, a gap, the stats, a gap, the weight, a gap, the footer, frame.
        let h = Stat::COUNT as i32 + 9;
        let panel = Panel::new(console, " Character ", 48, h);
        let x = panel.inner_x();
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        if fits(y) {
            console.print(x, y, &self.name, ACCENT);
        }
        y += 2;
        for (stat, score) in self.scores.iter() {
            // `St  Strength      12`: the abbreviation, the name, the score right-aligned.
            if fits(y) {
                console.print(x, y, stat.abbrev(), TEXT_DIM);
                console.print(x + 4, y, stat.name(), TEXT);
                console.print(x + 4 + NAME_W, y, &format!("{score:>2}"), TEXT);
            }
            y += 1;
        }
        y += 1;
        if fits(y) {
            let w = &self.weight;
            let weight = match w.carried {
                0 => format!("Weight {}, carrying nothing", format_weight(w.total())),
                carried => format!(
                    "Weight {} (you {}, carrying {})",
                    format_weight(w.total()),
                    format_weight(w.base),
                    format_weight(carried),
                ),
            };
            console.print(x, y, &weight, TEXT);
        }
        let footer = match bindings.keys_of(KeyCommand::Character).first() {
            Some(key) => format!("{} or Esc close", key.display()),
            None => "Esc close".to_string(),
        };
        panel.footer(console, &footer);
    }
}

/// `name` with its first letter in upper case.
fn capitalized(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Action, Dir, Letters, World};

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

    fn drawn(world: &World, cols: u32, rows: u32) -> Vec<String> {
        let screen = CharacterScreen::new(Some(&world.perception()));
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        text(&console)
    }

    #[test]
    fn shows_the_name_scores_and_weight() {
        let mut world = showcase();
        // Pick up the start room's jerkin, for something carried.
        world.apply(&Action::Move(Dir::SW));
        world.apply(&Action::Move(Dir::SW));
        world.apply(&Action::PickUp(Letters::of([0])));
        let text = drawn(&world, 80, 30);
        let row = |needle: &str| {
            text.iter()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
        };
        assert!(row(" Character ").contains('┌') || row(" Character ").contains('─'));
        assert!(
            row("You")
                .trim_matches(|c| c == ' ' || c == '│')
                .starts_with("You")
        );
        for expected in [
            "St  Strength      12",
            "Dx  Dexterity     14",
            "To  Toughness     11",
            "In  Intelligence  13",
            "Wi  Willpower      9",
            "Pr  Presence      10",
        ] {
            row(expected);
        }
        row("Weight 73.5 kg (you 70 kg, carrying 3.5 kg)");
        row("@ or Esc close");
        let empty_handed = drawn(&showcase(), 80, 30);
        assert!(
            (empty_handed.iter()).any(|l| l.contains("Weight 70 kg, carrying nothing")),
            "{empty_handed:#?}"
        );
    }

    #[test]
    fn never_shows_potentials_or_modifiers() {
        let text = drawn(&showcase(), 80, 30).join("\n");
        // The showcase player's potentials that are not also scores (15, 16, 17), and their
        // modifiers (+1, +2, -1), appear nowhere; neither do signs or the words.
        for hidden in ["15", "16", "17", "+", "-", "otential", "odifier"] {
            assert!(!text.contains(hidden), "{hidden:?} in\n{text}");
        }
    }

    #[test]
    fn closes_on_its_key_esc_or_enter() {
        let b = Bindings::default();
        let mut screen = CharacterScreen::new(None);
        assert_eq!(screen.handle(&Press::typed('a'), &b), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Down), &b),
            ScreenAction::Stay
        );
        let at = Press {
            binding: KeyBinding::from_text("@"),
            ..Press::typed('@')
        };
        assert_eq!(screen.handle(&at, &b), ScreenAction::Close);
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
    fn draws_in_small_consoles() {
        let world = showcase();
        for (cols, rows) in [(48, 15), (30, 8), (5, 3), (0, 0)] {
            let text = drawn(&world, cols, rows);
            assert_eq!(text.len(), rows as usize);
        }
        let screen = CharacterScreen::new(None);
        let mut console = Console::default();
        console.resize(40, 12);
        screen.draw(&mut console, &Bindings::default());
    }
}
