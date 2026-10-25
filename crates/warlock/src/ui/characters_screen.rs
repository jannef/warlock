//! The Characters screen (spec §6 "Characters", §7 "Saves"), from the main menu: every saved
//! character, the one played last first, with its race, class, and level, and for the selected
//! one where it is (and in which campaign, when more than one is installed: a developer build)
//! and when it was last played. Choose one to resume it; the Erase key deletes
//! the selected one, through the one dialog. A save without a readable summary is listed by its
//! id.
//!
//! Like every screen it is a plain state machine, testable without a window. What the list
//! shows comes from the saves' summaries (`save::Summary`), read when the screen opens.

use super::dialog_screen::Dialog;
use super::menu::{Hints, Nav, Panel, Press};
use super::screens::{MenuChoice, ScreenAction};
use super::{TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use crate::save::{SavedCharacter, Summary, played_text};
use crate::settings::Clock;

/// The panel's width.
const WIDTH: i32 = 72;
/// Where the race and class column begins (from the panel's inner edge).
const RACE_X: i32 = 24;
/// Where the level column ends.
const LEVEL_END: i32 = 66;

/// The saved characters.
#[derive(Debug)]
pub struct CharactersScreen {
    characters: Vec<SavedCharacter>,
    selected: usize,
    /// More than one campaign is installed (a developer build, with the test world): the place
    /// is named with its campaign.
    campaigns: bool,
}

impl CharactersScreen {
    /// The list of `characters`; `campaigns` when more than one campaign is installed.
    pub fn new(characters: Vec<SavedCharacter>, campaigns: bool) -> Self {
        Self {
            characters,
            selected: 0,
            campaigns,
        }
    }

    /// Where a character is: `In Cēaptūn.`, or with more than one campaign installed and the
    /// summary's campaign known, `In Cēaptūn (The Ascmearc).`
    fn place(&self, summary: &Summary) -> String {
        if self.campaigns && !summary.campaign.is_empty() {
            format!("In {} ({}).", summary.place, summary.campaign)
        } else {
            format!("In {}.", summary.place)
        }
    }

    /// The dialog deleting `character` asks first.
    fn delete_dialog(character: &SavedCharacter) -> Dialog {
        let text = format!(
            "Delete {}'s save? The character is gone for good.",
            character.name()
        );
        Dialog::yes_no(
            "Delete character",
            text,
            "Delete",
            MenuChoice::DeleteCharacter(character.id.clone()),
            "Keep",
        )
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let count = self.characters.len();
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up if count > 0 => self.selected = (self.selected + count - 1) % count,
            Nav::Down if count > 0 => self.selected = (self.selected + 1) % count,
            Nav::Enter => {
                if let Some(character) = self.characters.get(self.selected) {
                    return ScreenAction::Choose(MenuChoice::Resume(character.id.clone()));
                }
            }
            Nav::Erase => {
                if let Some(character) = self.characters.get(self.selected) {
                    return Self::delete_dialog(character).into();
                }
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, clock: Clock, bindings: &Bindings) {
        // Rows: margin, the list, a gap, where and when, a gap, the footer, the frame.
        let rows = self.characters.len().max(1) as i32;
        let panel = Panel::new(console, " Characters ", WIDTH, rows + 9);
        let x = panel.inner_x();
        let visible = (panel.h - 9).clamp(1, rows);
        let first = (self.selected as i32 - visible + 1).clamp(0, rows - visible);
        if self.characters.is_empty() {
            console.print(x + 2, panel.y + 2, "No saved characters.", TEXT_DIM);
        }
        for row in first..(first + visible).min(self.characters.len() as i32) {
            let character = &self.characters[row as usize];
            let y = panel.y + 2 + row - first;
            panel.row(
                console,
                y,
                row as usize == self.selected,
                character.name(),
                "",
                0,
            );
            if let Some(summary) = &character.summary {
                console.print(x + RACE_X, y, &summary.race_and_class(), TEXT);
                let level = format!("level {}", summary.level);
                let lx = x + LEVEL_END - level.chars().count() as i32;
                console.print(lx, y, &level, TEXT);
            }
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        if first + visible < rows {
            console.print(panel.x + panel.w - 3, panel.y + 2 + visible, "↓", TEXT_DIM);
        }
        let details = match self.characters.get(self.selected) {
            Some(SavedCharacter {
                summary: Some(summary),
                ..
            }) => vec![
                self.place(summary),
                format!("Last played {}.", played_text(summary.played, clock)),
            ],
            Some(_) => vec!["Nothing is known of this save until it is played.".into()],
            None => Vec::new(),
        };
        for (y, line) in (panel.y + 3 + visible..).zip(details) {
            if y < panel.y + panel.h - 2 {
                console.print(x + 2, y, &line, TEXT_DIM);
            }
        }
        let keys = Hints::of(bindings);
        let footer = format!(
            "{} choose   {} play   {} delete   Esc back",
            keys.up_down(),
            keys.confirm,
            keys.erase
        );
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(id: &str, name: &str, level: u32) -> SavedCharacter {
        SavedCharacter {
            id: id.into(),
            summary: Some(Summary {
                name: name.into(),
                race: "Dwarf".into(),
                class: "Warrior".into(),
                level,
                place: "the Hub".into(),
                campaign: "The Showcase".into(),
                played: 1_781_654_400,
            }),
        }
    }

    fn screen() -> CharactersScreen {
        let characters = vec![
            saved("brakka-2", "Brakka", 3),
            saved("brakka-1", "Brakka", 1),
            SavedCharacter {
                id: "tamsin-1".into(),
                summary: None,
            },
        ];
        CharactersScreen::new(characters, false)
    }

    fn drawn(screen: &CharactersScreen) -> String {
        let mut console = Console::default();
        console.resize(80, 24);
        screen.draw(&mut console, Clock::H24, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn choosing_a_character_resumes_it() {
        let mut s = screen();
        s.handle(&Press::nav(Nav::Down));
        assert_eq!(
            s.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::Resume("brakka-1".into()))
        );
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn deleting_asks_the_one_dialog_first() {
        let mut s = screen();
        let ScreenAction::Dialog(dialog) = s.handle(&Press::nav(Nav::Erase)) else {
            panic!("a dialog");
        };
        assert_eq!(
            *dialog,
            CharactersScreen::delete_dialog(&saved("brakka-2", "Brakka", 3))
        );
        let text = format!("{dialog:?}");
        assert!(text.contains("Delete Brakka's save?"), "{text}");
        assert!(text.contains("DeleteCharacter(\"brakka-2\")"), "{text}");
    }

    #[test]
    fn the_list_shows_who_and_where_and_when() {
        let mut s = screen();
        let text = drawn(&s);
        assert!(text.contains("> Brakka"), "{text}");
        assert!(text.contains("Dwarf Warrior"), "{text}");
        assert!(text.contains("level 3"), "{text}");
        assert!(text.contains("In the Hub."), "{text}");
        assert!(text.contains("Last played 2026-06-17 00:00 UTC."), "{text}");
        assert!(text.contains("Del delete"), "{text}");
        // A save without its summary is listed by its id.
        s.handle(&Press::nav(Nav::Up));
        let text = drawn(&s);
        assert!(text.contains("> tamsin-1"), "{text}");
        assert!(text.contains("Nothing is known"), "{text}");
        assert!(drawn(&CharactersScreen::new(Vec::new(), false)).contains("No saved characters."));
    }

    /// With more than one campaign installed (a developer build) the place names its campaign;
    /// a summary from before campaigns were kept just its place.
    #[test]
    fn the_place_names_its_campaign_when_there_are_several() {
        let mut s = screen();
        s.campaigns = true;
        assert!(drawn(&s).contains("In the Hub (The Showcase)."));
        if let Some(summary) = &mut s.characters[0].summary {
            summary.campaign.clear();
        }
        assert!(drawn(&s).contains("In the Hub."));
    }
}
