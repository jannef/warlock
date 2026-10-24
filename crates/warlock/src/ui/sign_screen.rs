//! Character creation's reveal (spec §8 "Birth sign"): right after the race, *Born under the
//! stars* says when the character was born, under which sign, what the animal and the element
//! are like, a lucky day's legend if the day is one, the sign's gifts and flaw in words, and the
//! sign's stat adjustments as written in data. Nothing is chosen on it: the stars decide, from
//! the race alone (the birth does not depend on the gender or the class), so going back and
//! choosing the same race shows the same stars. Enter goes on to the gender, Esc back to the
//! race; the summary's Sign row returns here.
//!
//! It never says a number the game hides, the word "seed", or that lucky days exist to anyone
//! not born on one. Like every screen it is a plain state machine, testable without a window.

use super::chargen_screen::{Draft, WIDTH, adjustments};
use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::content::Content;

/// Where a gift's words begin, from the panel's inner edge.
const WORDS_X: i32 = 20;

/// How a line of the panel looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    Plain,
    Dim,
    Lucky,
}

impl Look {
    fn color(self) -> warlock_core::color::Rgb {
        match self {
            Look::Plain => TEXT,
            Look::Dim => TEXT_DIM,
            Look::Lucky => ACCENT,
        }
    }
}

/// One gift as the reveal lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Gift {
    /// "Fleet-footed", "but Slight" for a flaw.
    name: String,
    /// What it does, in the game's words.
    words: String,
    look: Look,
}

/// The reveal.
#[derive(Debug)]
pub struct SignScreen {
    /// "You were born on 3 Seedtide 1419, in the year of the Metal Manticore. You are 25 years
    /// old."; or why there is no sign to show.
    born: String,
    /// The animal's and the element's lines of legend.
    legend: String,
    /// A lucky day's line; `None` for nearly everyone.
    lucky: Option<String>,
    gifts: Vec<Gift>,
    /// The animal's and the element's adjustments, as written in data.
    adjustments: String,
}

impl SignScreen {
    /// The reveal for the race chosen on the screens below.
    pub fn new(content: &Content, draft: &Draft) -> Self {
        let empty = |why: &str| Self {
            born: why.to_string(),
            legend: String::new(),
            lucky: None,
            gifts: Vec::new(),
            adjustments: String::new(),
        };
        let Some(spec) = draft.spec(content) else {
            return empty("There are no races to be born to.");
        };
        let start = match spec.start(content, draft.seed(), draft.start()) {
            Ok(start) => start,
            Err(why) => return empty(&why),
        };
        let c = start.character;
        let parts = (content.horoscope())
            .zip(c.sign.as_ref())
            .and_then(|(h, s)| {
                let (animal, element) = h.parts(s)?;
                Some((h, animal, element))
            });
        let date = c.born.text(content.calendar());
        let age = c.age_on(draft.start());
        let years = if age == 1 { "year" } else { "years" };
        let Some((h, animal, element)) = parts else {
            return empty(&format!(
                "You were born on {date}. You are {age} {years} old."
            ));
        };
        let sign = h.name(c.sign.as_ref().expect("a sign"));
        let born = format!(
            "You were born on {date}, in the year of the {sign}. You are {age} {years} old."
        );
        let legend = format!("{} {}", animal.description, element.description);
        let lucky = c.lucky(content);
        let book = content.talents();
        let lucky_grants: Vec<&str> = lucky.map_or(Vec::new(), |l| {
            l.grants.iter().map(String::as_str).collect()
        });
        let gifts = (c.grants(content).into_iter())
            .filter_map(|id| book.id(id).map(|t| (id, book.get(t))))
            .map(|(id, def)| {
                let look = if lucky_grants.contains(&id) {
                    Look::Lucky
                } else if def.flaw {
                    Look::Dim
                } else {
                    Look::Plain
                };
                let name = if def.flaw {
                    format!("but {}", def.name)
                } else {
                    def.name.clone()
                };
                Gift {
                    name,
                    words: def.description.clone(),
                    look,
                }
            })
            .collect();
        let adjustments = format!(
            "{}: {}   {}: {}",
            animal.name,
            adjustments(&animal.stats),
            element.name,
            adjustments(&element.stats)
        );
        Self {
            born,
            legend,
            lucky: lucky.map(|l| l.description.clone()),
            gifts,
            adjustments,
        }
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        match press.nav {
            Nav::Back => ScreenAction::Close,
            Nav::Enter => ScreenAction::Open(ScreenName::Gender),
            _ => ScreenAction::Stay,
        }
    }

    /// The panel's lines for text `width` cells wide, each with how it looks and where it
    /// begins (0, or the gifts' words column).
    fn lines(&self, width: i32) -> Vec<(String, Look, i32)> {
        let mut lines = Vec::new();
        let paragraph = |text: &str, look: Look, lines: &mut Vec<(String, Look, i32)>| {
            if text.is_empty() {
                return;
            }
            if !lines.is_empty() {
                lines.push((String::new(), Look::Plain, 0));
            }
            lines.extend(wrap(text, width).into_iter().map(|l| (l, look, 0)));
        };
        paragraph(&self.born, Look::Plain, &mut lines);
        paragraph(&self.legend, Look::Dim, &mut lines);
        if let Some(lucky) = &self.lucky {
            paragraph(lucky, Look::Lucky, &mut lines);
        }
        if !self.gifts.is_empty() {
            lines.push((String::new(), Look::Plain, 0));
        }
        for gift in &self.gifts {
            let words = wrap(&gift.words, (width - WORDS_X).max(8));
            lines.push((gift.name.clone(), gift.look, 0));
            let at = lines.len() - 1;
            for (i, line) in words.into_iter().enumerate() {
                if i == 0 {
                    // The first line of the words beside the name.
                    let name = &lines[at].0;
                    let pad = (WORDS_X as usize)
                        .saturating_sub(name.chars().count())
                        .max(1);
                    lines[at].0 = format!("{name}{}{line}", " ".repeat(pad));
                } else {
                    lines.push((line, gift.look, WORDS_X));
                }
            }
        }
        if !self.adjustments.is_empty() {
            lines.push((String::new(), Look::Plain, 0));
            lines.push((self.adjustments.clone(), Look::Plain, 0));
        }
        lines
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the lines, a gap, the footer, the frame.
        let inner = WIDTH.min(console.cols() as i32) - 4;
        let lines = self.lines(inner - 2);
        let panel = Panel::new(
            console,
            " Born under the stars ",
            WIDTH,
            lines.len() as i32 + 5,
        );
        let x = panel.inner_x();
        let footer_y = panel.y + panel.h - 2;
        for (y, (line, look, at)) in (panel.y + 2..).zip(&lines) {
            if y < footer_y - 1 {
                console.print(x + 1 + at, y, line, look.color());
            }
        }
        let keys = Hints::of(bindings);
        panel.footer(console, &format!("{} continue   Esc back", keys.confirm));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::chargen_screen::{Picks, ScenarioPick, Step};
    use std::sync::LazyLock;

    static CONTENT: LazyLock<Content> =
        LazyLock::new(|| Content::builtin().expect("builtin content"));

    /// New run on the showcase with `seed`, the race `race` chosen.
    fn draft(seed: u64, race: &str) -> Draft {
        let showcase = CONTENT.scenario("showcase").expect("showcase");
        let mut chosen = Picks::default();
        chosen.set(Step::Race, race);
        Draft {
            scenario: Some(ScenarioPick {
                id: "showcase".into(),
                name: showcase.name.clone(),
                seed,
                shows_seed: true,
                character: showcase.character.clone(),
                start: showcase.start,
            }),
            chosen,
            ..Draft::default()
        }
    }

    fn text(screen: &SignScreen) -> String {
        let mut console = Console::default();
        console.resize(80, 30);
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

    /// The first seed whose `race` is born on a lucky day (`want`) or not.
    fn seed_where(race: &str, want: bool) -> u64 {
        (1u64..5000)
            .find(|&seed| {
                let d = draft(seed, race);
                let spec = d.spec(&CONTENT).expect("a spec");
                let start = spec.start(&CONTENT, seed, d.start()).expect("starts");
                start.character.lucky(&CONTENT).is_some() == want
            })
            .expect("a seed")
    }

    #[test]
    fn the_reveal_says_the_birth_the_sign_and_its_gifts_in_words() {
        let seed = seed_where("human", false);
        let d = draft(seed, "human");
        let screen = SignScreen::new(&CONTENT, &d);
        let shown = text(&screen);
        assert!(shown.contains("Born under the stars"), "{shown}");
        assert!(shown.contains("You were born on"), "{shown}");
        assert!(shown.contains("in the year of the"), "{shown}");
        assert!(!shown.contains("lucky"), "no hint of lucky days: {shown}");
        assert!(!shown.to_lowercase().contains("seed"), "{shown}");
        for gift in &screen.gifts {
            let name = gift.name.trim_start_matches("but ");
            assert!(shown.contains(name), "{name}: {shown}");
        }
        // Gender and class never move the stars.
        let mut other = d.clone();
        other.chosen.set(Step::Gender, "other");
        other.chosen.set(Step::Class, "wizard");
        assert_eq!(SignScreen::new(&CONTENT, &other).born, screen.born);
    }

    #[test]
    fn a_lucky_day_shows_its_legend_and_extra_gift() {
        // Half-orcs reach three lucky signs at the showcase's start.
        let seed = seed_where("half_orc", true);
        let screen = SignScreen::new(&CONTENT, &draft(seed, "half_orc"));
        let lucky = screen.lucky.clone().expect("a lucky day");
        assert!(lucky.starts_with("A lucky day."), "{lucky}");
        let shown = text(&screen);
        assert!(shown.contains("A lucky day."), "{shown}");
    }

    #[test]
    fn flaws_read_but_and_enter_goes_on_esc_back() {
        // A Salamander: Fleet-footed, Slip away, but Slight.
        let seed = (1u64..5000)
            .find(|&seed| {
                let d = draft(seed, "human");
                let spec = d.spec(&CONTENT).expect("a spec");
                let start = spec.start(&CONTENT, seed, d.start()).expect("starts");
                start.character.born_in_year_of("salamander")
            })
            .expect("a salamander");
        let mut screen = SignScreen::new(&CONTENT, &draft(seed, "human"));
        let names: Vec<&str> = screen.gifts.iter().map(|g| g.name.as_str()).collect();
        assert_eq!(&names[..3], ["Fleet-footed", "Slip away", "but Slight"]);
        assert_eq!(screen.gifts[2].look, Look::Dim);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Open(ScreenName::Gender)
        );
        assert_eq!(screen.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }
}
