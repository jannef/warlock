//! The New run screen (spec §6): the scenarios from content and an optional seed.

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::render::console::Console;
use warlock_core::content::Content;

/// A scenario as listed on the screen.
#[derive(Clone, Debug)]
struct ScenarioInfo {
    id: String,
    name: String,
    description: String,
    seed: u64,
}

/// Longest seed text we accept.
const MAX_SEED_LEN: usize = 24;

/// The New run screen: a scenario list and a seed field.
#[derive(Debug)]
pub struct NewRunScreen {
    scenarios: Vec<ScenarioInfo>,
    /// A saved run exists, which a new run replaces; the screen says so.
    has_save: bool,
    selected: usize,
    seed: String,
}

impl NewRunScreen {
    pub fn new(content: &Content, has_save: bool) -> Self {
        let mut scenarios: Vec<ScenarioInfo> = content
            .scenarios()
            .map(|s| ScenarioInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                seed: s.seed,
            })
            .collect();
        // The showcase (the command-line default) comes first and is preselected.
        scenarios.sort_by_key(|s| s.id != "showcase");
        Self {
            scenarios,
            has_save,
            selected: 0,
            seed: String::new(),
        }
    }

    /// The typed seed: a number as-is, a word hashed (`warlock_core::rng::parse_seed`); empty
    /// means the scenario default.
    fn seed(&self) -> Option<u64> {
        warlock_core::rng::parse_seed(&self.seed).ok()
    }

    /// Characters the seed field accepts. Letters double as hjkl navigation elsewhere, so on this
    /// screen the arrow keys navigate and typed characters go to the seed.
    fn accepts(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '-' || ch == '_'
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let count = self.scenarios.len();
        if let Some(ch) = press.text.filter(|ch| Self::accepts(*ch))
            && !press.repeat
        {
            if self.seed.chars().count() < MAX_SEED_LEN {
                self.seed.push(ch);
            }
            return ScreenAction::Stay;
        }
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up if count > 0 => self.selected = (self.selected + count - 1) % count,
            Nav::Down if count > 0 => self.selected = (self.selected + 1) % count,
            Nav::Backspace | Nav::Delete => {
                self.seed.pop();
            }
            Nav::Enter => {
                if let Some(info) = self.scenarios.get(self.selected) {
                    return ScreenAction::Choose(MenuChoice::NewRun {
                        scenario: info.id.clone(),
                        seed: self.seed(),
                    });
                }
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console) {
        let h = 14 + self.scenarios.len() as i32;
        let panel = Panel::new(console, " New run ", 60, h);
        let x = panel.inner_x();
        let mut y = panel.y + 2;
        if self.scenarios.is_empty() {
            console.print(x, y, "No scenarios are installed.", TEXT);
        }
        for (i, info) in self.scenarios.iter().enumerate() {
            panel.row(console, y, i == self.selected, &info.name, "", 0);
            y += 1;
        }
        y += 1;
        if let Some(info) = self.scenarios.get(self.selected) {
            for line in wrap(&info.description, panel.inner_w() - 2) {
                console.print(x + 2, y, &line, TEXT_DIM);
                y += 1;
            }
            y += 1;
            let seed = if self.seed.is_empty() {
                format!("(default: {})", info.seed)
            } else {
                self.seed.clone()
            };
            let color = if self.seed.is_empty() { TEXT_DIM } else { TEXT };
            console.print(x + 2, y, "Seed: [", TEXT);
            let printed = console.print(x + 9, y, &seed, color);
            console.print(x + 9 + printed, y, "_]", TEXT);
            console.print(
                x + 2,
                y + 1,
                "Type a number or a word to set a seed.",
                TEXT_DIM,
            );
        }
        if self.has_save {
            let warning = "! Starting a new run replaces your saved run.";
            console.print(x, panel.y + panel.h - 4, warning, ACCENT);
        }
        panel.footer(console, "↑↓ scenario   Enter start   Esc back");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> NewRunScreen {
        NewRunScreen::new(&Content::builtin().expect("content"), false)
    }

    #[test]
    fn starts_the_chosen_scenario_with_the_typed_seed() {
        let mut s = screen();
        for d in ['4', '2'] {
            s.handle(&Press::typed(d));
        }
        s.handle(&Press::nav(Nav::Backspace));
        s.handle(&Press::typed('7'));
        let ScreenAction::Choose(MenuChoice::NewRun { scenario, seed }) =
            s.handle(&Press::nav(Nav::Enter))
        else {
            panic!("expected a new run");
        };
        assert_eq!(scenario, "showcase");
        assert_eq!(seed, Some(47));
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn word_seeds_are_hashed_and_hjkl_type_letters() {
        let mut s = screen();
        for ch in "jolly".chars() {
            s.handle(&Press::typed(ch));
        }
        let ScreenAction::Choose(MenuChoice::NewRun { seed, .. }) =
            s.handle(&Press::nav(Nav::Enter))
        else {
            panic!("expected a new run");
        };
        assert_eq!(seed, warlock_core::rng::parse_seed("jolly").ok());
    }

    #[test]
    fn empty_seed_means_the_scenario_default() {
        let ScreenAction::Choose(MenuChoice::NewRun { seed, .. }) =
            screen().handle(&Press::nav(Nav::Enter))
        else {
            panic!("expected a new run");
        };
        assert_eq!(seed, None);
    }
}
