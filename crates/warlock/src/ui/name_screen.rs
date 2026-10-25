//! Character creation's Name step (spec §6 "New run", §8 "Character creation"): after the class,
//! before the summary, a text field for the character's name. The game's text field: the keys
//! bound in `Context::Text` edit it (accept, erase, a random name, back), and every other key
//! types its text, literally, as long as the font draws it and the name has room
//! ([`MAX_NAME`]). Held keys type and erase again, but never accept.
//!
//! Random fills the field with one of the race's names for the gender, drawn from the run seed
//! (`character::random_name`); an empty field shows, dimmed, the name accepting it gives: the
//! first draw. Like every screen it is a plain state machine, testable without a window.

use super::chargen_screen::{Draft, Step};
use super::menu::{Edit, Panel, Press, wrap};
use super::screens::{ScreenAction, ScreenName};
use super::{TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::atlas;
use crate::render::console::Console;
use warlock_core::character::{MAX_NAME, NAMELESS, random_name};
use warlock_core::content::Content;

/// How many random names Random goes through before it starts over.
const DRAWS: u32 = 64;

/// The Name step.
#[derive(Debug)]
pub struct NameScreen {
    /// What is typed.
    text: String,
    /// The random names, in the order Random gives them; the first is what an empty field
    /// gives. Never empty.
    draws: Vec<String>,
    /// How often Random was pressed here.
    rolls: usize,
    /// Who is being named, as the panel says it: `a dwarf warrior`.
    who: String,
}

impl NameScreen {
    /// The step for the character chosen so far on the screens below, on the name remembered
    /// from before going back (else an empty field).
    pub fn new(content: &Content, draft: &Draft) -> Self {
        let spec = draft.spec(content);
        let draws: Vec<String> = match &spec {
            Some(spec) => (0..DRAWS)
                .filter_map(|draw| random_name(content, draft.seed(), spec, draw))
                .collect(),
            None => Vec::new(),
        };
        let draws = if draws.is_empty() {
            vec![NAMELESS.to_string()]
        } else {
            draws
        };
        let who = spec.map_or(String::new(), |spec| {
            let race = (spec.race(content)).map_or(spec.race.clone(), |r| r.name.clone());
            let class = (spec.class(content)).map_or(spec.class.clone(), |c| c.name.clone());
            let article = warlock_core::content::article(&race);
            format!("{article} {} {}", race.to_lowercase(), class.to_lowercase())
        });
        Self {
            text: draft.remembered_name().unwrap_or_default(),
            draws,
            rolls: 0,
            who,
        }
    }

    /// What is typed (possibly nothing).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The name accepting an empty field gives: the first random name.
    pub fn placeholder(&self) -> &str {
        &self.draws[0]
    }

    /// Whether the field takes `ch`: a printable character the font draws, while there is room,
    /// and no space to begin with.
    fn takes(&self, ch: char) -> bool {
        !ch.is_control()
            && atlas::draws(ch)
            && self.text.chars().count() < MAX_NAME
            && !(ch == ' ' && self.text.is_empty())
    }

    /// The next random name that is not the one shown.
    fn random(&mut self) {
        let shown = if self.text.is_empty() {
            self.placeholder().to_string()
        } else {
            self.text.clone()
        };
        for _ in 0..self.draws.len() {
            self.rolls += 1;
            let name = &self.draws[self.rolls % self.draws.len()];
            if *name != shown {
                self.text = name.clone();
                return;
            }
        }
        self.text = shown;
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        match press.edit {
            Some(Edit::Back) => ScreenAction::Close,
            Some(Edit::Confirm) => ScreenAction::Open(ScreenName::Summary),
            Some(Edit::Erase) => {
                self.text.pop();
                ScreenAction::Stay
            }
            Some(Edit::Random) => {
                if !press.repeat {
                    self.random();
                }
                ScreenAction::Stay
            }
            None => {
                if let Some(ch) = press.text.filter(|&ch| self.takes(ch)) {
                    self.text.push(ch);
                }
                ScreenAction::Stay
            }
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, who, a gap, the field, a gap, two lines of help, a gap, the footer, the
        // frame.
        let panel = Panel::new(
            console,
            Step::Name.title(),
            super::chargen_screen::WIDTH,
            11,
        );
        let x = panel.inner_x();
        let mut y = panel.y + 2;
        if !self.who.is_empty() {
            console.print(
                x + 2,
                y,
                &format!("What is the name of {}?", self.who),
                TEXT,
            );
        }
        y += 2;
        console.print(x + 2, y, "Name: [", TEXT);
        let (shown, color) = if self.text.is_empty() {
            (self.placeholder(), TEXT_DIM)
        } else {
            (self.text.as_str(), TEXT)
        };
        let printed = console.print(x + 9, y, shown, color);
        console.print(x + 9 + printed, y, "_", TEXT);
        console.print(x + 9 + MAX_NAME as i32 + 1, y, "]", TEXT);
        y += 2;
        let key = |command| bindings.key_name(command).unwrap_or_else(|| "?".into());
        let help = format!(
            "Type a name of up to {MAX_NAME} characters, or {} for one of your people's names. \
             Left empty, it is {}.",
            key(KeyCommand::TextRandom),
            self.placeholder()
        );
        for line in wrap(&help, panel.inner_w() - 2).into_iter().take(2) {
            console.print(x + 2, y, &line, TEXT_DIM);
            y += 1;
        }
        let footer = format!(
            "{} accept   {} random   {} erase   Esc back",
            key(KeyCommand::TextConfirm),
            key(KeyCommand::TextRandom),
            key(KeyCommand::TextErase),
        );
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::chargen_screen::ScenarioPick;
    use crate::ui::menu::Nav;

    fn content() -> Content {
        Content::builtin().expect("content")
    }

    fn draft(content: &Content) -> Draft {
        let showcase = content.scenario("showcase").expect("showcase");
        Draft {
            scenario: Some(ScenarioPick {
                id: "showcase".into(),
                name: showcase.name.clone(),
                seed: 1,
                shows_seed: true,
                character: showcase.character.clone(),
                start: showcase.start,
            }),
            ..Draft::default()
        }
    }

    fn typed(screen: &mut NameScreen, text: &str) {
        for ch in text.chars() {
            screen.handle(&Press::typed(ch));
        }
    }

    #[test]
    fn typed_characters_go_in_literally() {
        let content = content();
        let mut s = NameScreen::new(&content, &draft(&content));
        assert_eq!(s.text(), "");
        // Letters that move the selection elsewhere (hjkl), `y`/`n`, and symbols all type.
        typed(&mut s, "hjkl yn ,<>?");
        assert_eq!(s.text(), "hjkl yn ,<>?");
        s.handle(&Press::edit(Edit::Erase));
        assert_eq!(s.text(), "hjkl yn ,<>");
        // The menu's keys mean nothing here: only the text field's own.
        assert_eq!(s.handle(&Press::nav(Nav::Down)), ScreenAction::Stay);
        assert_eq!(s.handle(&Press::nav(Nav::Erase)), ScreenAction::Stay);
        assert_eq!(s.text(), "hjkl yn ,<>");
    }

    #[test]
    fn names_have_room_for_twenty_and_start_with_no_space() {
        let content = content();
        let mut s = NameScreen::new(&content, &draft(&content));
        typed(&mut s, "  Brakka Stonefist of the Deep");
        assert_eq!(s.text(), "Brakka Stonefist of ");
        assert_eq!(s.text().chars().count(), MAX_NAME);
        let mut s = NameScreen::new(&content, &draft(&content));
        typed(&mut s, "Åsa\u{7}日");
        assert_eq!(
            s.text(),
            "Åsa",
            "control characters and glyphs the font lacks stay out"
        );
    }

    #[test]
    fn random_names_are_the_races_and_never_the_one_shown() {
        let content = content();
        let d = draft(&content);
        let mut s = NameScreen::new(&content, &d);
        let spec = d.spec(&content).expect("a character");
        let offered = content.names().offered(&spec.race, spec.gender);
        assert!(offered.contains(&s.placeholder()));
        let mut shown = s.placeholder().to_string();
        for _ in 0..10 {
            s.handle(&Press::edit(Edit::Random));
            assert!(offered.contains(&s.text()), "{}", s.text());
            assert_ne!(s.text(), shown);
            shown = s.text().to_string();
        }
        // A held key does not spin through names.
        let held = Press {
            repeat: true,
            ..Press::edit(Edit::Random)
        };
        s.handle(&held);
        assert_eq!(s.text(), shown);
    }

    #[test]
    fn accepting_goes_on_to_the_summary_and_back_closes() {
        let content = content();
        let mut s = NameScreen::new(&content, &draft(&content));
        assert_eq!(
            s.handle(&Press::edit(Edit::Confirm)),
            ScreenAction::Open(ScreenName::Summary)
        );
        assert_eq!(s.handle(&Press::edit(Edit::Back)), ScreenAction::Close);
    }

    /// What `screen` draws on an 80 by 24 console, as text.
    fn drawn(screen: &NameScreen) -> String {
        let mut console = Console::default();
        console.resize(80, 24);
        screen.draw(&mut console, &Bindings::default());
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
    fn the_field_shows_the_name_and_the_keys_as_bound() {
        let content = content();
        let mut s = NameScreen::new(&content, &draft(&content));
        let text = drawn(&s);
        assert!(text.contains("Choose your name (5 of 5)"), "{text}");
        assert!(
            text.contains("What is the name of a human thief?"),
            "{text}"
        );
        assert!(text.contains(&format!("[{}_", s.placeholder())), "{text}");
        assert!(
            text.contains("Enter accept   Tab random   Backspace erase"),
            "{text}"
        );
        typed(&mut s, "Brakka");
        assert!(drawn(&s).contains("[Brakka_"));
    }
}
