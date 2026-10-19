//! The New run screen (spec §6): the scenarios from content and an optional seed. Enter goes on
//! to character creation ([`super::chargen_screen`]), or with content that has no races or
//! classes, starts the run as the scenario's.

use super::chargen_screen::ScenarioPick;
use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::character::CharacterSpec;
use warlock_core::content::Content;
use warlock_core::time::GameTime;

/// A scenario as listed on the screen.
#[derive(Clone, Debug)]
struct ScenarioInfo {
    id: String,
    name: String,
    description: String,
    seed: u64,
    character: Option<CharacterSpec>,
    start: GameTime,
}

/// Longest seed text we accept.
const MAX_SEED_LEN: usize = 24;

/// The New run screen: a scenario list and a seed field.
#[derive(Debug)]
pub struct NewRunScreen {
    scenarios: Vec<ScenarioInfo>,
    /// A saved run exists, which a new run replaces; the screen says so.
    has_save: bool,
    /// The content has races and classes to choose a character from.
    creates_characters: bool,
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
                character: s.character.clone(),
                start: s.start,
            })
            .collect();
        // The showcase (the command-line default) comes first and is preselected.
        scenarios.sort_by_key(|s| s.id != "showcase");
        Self {
            scenarios,
            has_save,
            creates_characters: !content.races().list.is_empty() && !content.classes().is_empty(),
            selected: 0,
            seed: String::new(),
        }
    }

    /// The selected scenario and the seed, for character creation.
    pub fn pick(&self) -> Option<ScenarioPick> {
        let info = self.scenarios.get(self.selected)?;
        let typed_seed = self.seed();
        Some(ScenarioPick {
            id: info.id.clone(),
            name: info.name.clone(),
            typed_seed,
            seed: typed_seed.unwrap_or(info.seed),
            character: info.character.clone(),
            start: info.start,
        })
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
            Nav::Erase => {
                self.seed.pop();
            }
            Nav::Enter if self.creates_characters && !self.scenarios.is_empty() => {
                return ScreenAction::Open(ScreenName::Race);
            }
            Nav::Enter => {
                if let Some(info) = self.scenarios.get(self.selected) {
                    return ScreenAction::Choose(MenuChoice::NewRun {
                        scenario: info.id.clone(),
                        seed: self.seed(),
                        character: None,
                    });
                }
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
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
        let keys = Hints::of(bindings);
        let next = if self.creates_characters {
            "next"
        } else {
            "start"
        };
        let footer = format!(
            "{} scenario   {} {next}   Esc back",
            keys.up_down(),
            keys.confirm
        );
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> NewRunScreen {
        NewRunScreen::new(&Content::builtin().expect("content"), false)
    }

    #[test]
    fn goes_on_to_the_character_with_the_chosen_scenario_and_typed_seed() {
        let mut s = screen();
        for d in ['4', '2'] {
            s.handle(&Press::typed(d));
        }
        s.handle(&Press::nav(Nav::Erase));
        s.handle(&Press::typed('7'));
        assert_eq!(
            s.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Open(ScreenName::Race)
        );
        let pick = s.pick().expect("a scenario");
        assert_eq!(pick.id, "showcase");
        assert_eq!((pick.typed_seed, pick.seed), (Some(47), 47));
        assert_eq!(
            pick.character.map(|c| c.to_string()).as_deref(),
            Some("human,female,thief")
        );
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn word_seeds_are_hashed_and_hjkl_type_letters() {
        let mut s = screen();
        for ch in "jolly".chars() {
            s.handle(&Press::typed(ch));
        }
        let seed = warlock_core::rng::parse_seed("jolly").ok();
        assert_eq!(s.pick().map(|p| p.typed_seed), Some(seed));
    }

    #[test]
    fn empty_seed_means_the_scenario_default() {
        let pick = screen().pick().expect("a scenario");
        assert_eq!((pick.typed_seed, pick.seed), (None, 1));
    }

    #[test]
    fn without_races_or_classes_enter_starts_the_scenario() {
        use warlock_core::content::ContentFiles;
        let mut files = ContentFiles::builtin();
        files.remove("data/races.toml");
        files.remove("data/classes.toml");
        for id in [
            "arena",
            "courtyard",
            "depths",
            "forest",
            "quarry",
            "showcase",
            "traps",
            "vast",
            "world",
        ] {
            files.remove(&format!("scenarios/{id}.toml"));
        }
        files.insert(
            "scenarios/plain.toml",
            "name = \"Plain\"\nstart_level = \"hub\"\n",
        );
        let content = Content::load(&files).expect("valid");
        let mut s = NewRunScreen::new(&content, false);
        assert_eq!(
            s.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::NewRun {
                scenario: "plain".into(),
                seed: None,
                character: None,
            })
        );
    }
}
