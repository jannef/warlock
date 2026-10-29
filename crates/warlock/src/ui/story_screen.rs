//! The player's background story (spec §8 "Background story", §6 "Info screens"): a read-only
//! panel of its paragraphs, titled with the character's name ("Brakka's story"), wrapped, and
//! scrolled with up and down when the console is too short for it (the message history's way).
//! Enter and Esc close it (the menu context's Confirm and Back); nothing on it can be chosen.
//!
//! It is seen two ways. **As the run begins** ([`StoryScreen::opening`]): 72 wide like the sign
//! reveal, with the footer `Enter continue`, opened by the app once over a new run's first turn
//! (`screens::on_run_start`; never on Continue or Resume, never under `--capture` or `--replay`,
//! never for a run without a story) and by `--screen story`. **As page 8, "Story"**
//! ([`StoryScreen::page`]): the info pages' width and tab line; for a character without a story
//! it says "Your story is not written."
//!
//! It reads the paragraphs from the perception snapshot when it opens
//! ([`PerceptionView::story`]); no game time passes while it is open.

use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{ScreenAction, ScreenName};
use super::{TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use std::sync::atomic::{AtomicU64, Ordering};
use warlock_core::perception::PerceptionView;

/// The panel's width as the run begins, as the sign reveal's.
const OPENING_WIDTH: i32 = 72;
/// The panel's width as page 8: every info page's.
const PAGE_WIDTH: i32 = super::info_tabs::panel_width(OPENING_WIDTH);
/// Rows of the panel besides the text: frame, margin, a gap and the footer, frame.
const EXTRA: i32 = 5;
/// What page 8 says for a character without a story.
const UNWRITTEN: &str = "Your story is not written.";

#[derive(Debug)]
pub struct StoryScreen {
    /// " Brakka's story ", or " Your story " for a run without a character.
    title: String,
    /// The paragraphs; empty without a story.
    paragraphs: Vec<String>,
    /// As the run begins (`true`), or page 8.
    opening: bool,
    /// Lines scrolled down from the top.
    first: usize,
    /// The console's size as last fitted (columns, rows), packed: presses scroll by its layout.
    console: AtomicU64,
}

impl StoryScreen {
    /// The screen opened once as the run begins.
    pub fn opening(view: Option<&PerceptionView<'_>>) -> Self {
        Self::new(view, true)
    }

    /// Page 8.
    pub fn page(view: Option<&PerceptionView<'_>>) -> Self {
        Self::new(view, false)
    }

    fn new(view: Option<&PerceptionView<'_>>, opening: bool) -> Self {
        let name = view.and_then(|v| v.character()).map(|c| c.name.as_str());
        let title = match name {
            Some(name) if !name.is_empty() => format!(" {name}'s story "),
            _ => " Your story ".to_string(),
        };
        Self {
            title,
            paragraphs: view.map(PerceptionView::story).unwrap_or_default(),
            opening,
            first: 0,
            console: AtomicU64::new(u64::MAX),
        }
    }

    /// Whether this is the screen opened as the run begins, not page 8: the page keys do not
    /// turn it.
    pub fn is_opening(&self) -> bool {
        self.opening
    }

    /// Lays the text out for a console `cols` by `rows`: drawing knows it, presses do not.
    pub fn fit(&self, cols: u32, rows: u32) {
        let size = (u64::from(cols) << 32) | u64::from(rows);
        self.console.store(size, Ordering::Relaxed);
    }

    /// Applies a key press: up and down scroll a line; Enter and Esc close it.
    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let (lines, shown) = match self.console.load(Ordering::Relaxed) {
            u64::MAX => (self.lines(self.width()).len(), usize::MAX),
            size => {
                let (cols, rows) = ((size >> 32) as i32, (size & 0xffff_ffff) as i32);
                let lines = self.lines(self.width().min(cols)).len();
                (lines, visible(lines, rows))
            }
        };
        let last = lines.saturating_sub(shown);
        match press.nav {
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.first = self.first.saturating_sub(1),
            Nav::Down => self.first = (self.first + 1).min(last),
            _ => {}
        }
        ScreenAction::Stay
    }

    /// The panel's width before the console clips it.
    fn width(&self) -> i32 {
        if self.opening {
            OPENING_WIDTH
        } else {
            PAGE_WIDTH
        }
    }

    /// The text's lines in a panel `width` wide: the paragraphs wrapped, a blank line between
    /// (a margin of one cell each side, inside the panel's own).
    fn lines(&self, width: i32) -> Vec<String> {
        let inner = (width - 6).max(8);
        if self.paragraphs.is_empty() {
            return wrap(UNWRITTEN, inner);
        }
        let mut lines = Vec::new();
        for (i, paragraph) in self.paragraphs.iter().enumerate() {
            if i > 0 {
                lines.push(String::new());
            }
            lines.extend(wrap(paragraph, inner));
        }
        lines
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let (cols, rows) = (console.cols() as i32, console.rows() as i32);
        self.fit(cols as u32, rows as u32);
        let width = self.width().min(cols);
        let lines = self.lines(width);
        let shown = visible(lines.len(), rows);
        let panel = Panel::new(console, &self.title, width, shown as i32 + EXTRA);
        if !self.opening {
            super::info_tabs::draw_tabs(console, &panel, bindings, ScreenName::Story);
        }
        let first = self.first.min(lines.len().saturating_sub(shown));
        let x = panel.inner_x() + 1;
        let color = if self.paragraphs.is_empty() {
            TEXT_DIM
        } else {
            TEXT
        };
        for (line, y) in lines.iter().skip(first).take(shown).zip(panel.y + 2..) {
            console.print(x, y, line, color);
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        if first + shown < lines.len() {
            let y = panel.y + 2 + shown as i32;
            console.print(panel.x + panel.w - 3, y, "↓", TEXT_DIM);
        }
        let keys = Hints::of(bindings);
        let close = if self.opening {
            format!("{} continue", keys.confirm)
        } else {
            "Esc close".to_string()
        };
        let footer = if shown < lines.len() {
            format!("{} scroll   {close}", keys.up_down())
        } else {
            close
        };
        panel.footer(console, &footer);
    }
}

/// How many of `lines` the panel shows at once in a console `rows` high.
fn visible(lines: usize, rows: i32) -> usize {
    lines.min((rows - EXTRA).max(1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, LazyLock};
    use warlock_core::World;
    use warlock_core::character::CharacterSpec;
    use warlock_core::content::Content;

    static CONTENT: LazyLock<Arc<Content>> =
        LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

    /// A chosen character's run with its kit, so with a story.
    fn kitted() -> World {
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        World::start_run(
            CONTENT.clone(),
            "showcase",
            Some(3),
            Some(&spec),
            None,
            true,
        )
        .expect("run")
    }

    fn shown(screen: &StoryScreen, cols: u32, rows: u32) -> Vec<String> {
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        (0..rows as i32)
            .map(|y| {
                (0..cols as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn the_paragraphs_are_wrapped_in_the_panel_under_the_name() {
        let world = kitted();
        let view = world.perception();
        let screen = StoryScreen::opening(Some(&view));
        let rows = shown(&screen, 100, 40);
        let text = rows.join("\n");
        let name = &view.character().expect("a character").name;
        assert!(text.contains(&format!("{name}'s story")), "{text}");
        assert!(text.contains("Enter continue"), "{text}");
        assert!(!text.contains("scroll"), "it fits: {text}");
        // Every word of the story is on the screen, in order, inside the 72-wide panel.
        let story = view.story();
        let words: Vec<&str> = story.iter().flat_map(|p| p.split(' ')).collect();
        let on_screen: Vec<&str> = (rows.iter())
            .map(|r| r.trim_matches(|c: char| c == ' ' || c == '│'))
            .flat_map(|r| r.split(' '))
            .filter(|w| !w.is_empty())
            .collect();
        let at = |w: &str| on_screen.iter().position(|s| *s == w);
        assert!(at(words[0]).is_some(), "{text}");
        let start = at(words[0]).expect("first word");
        assert_eq!(&on_screen[start..start + words.len()], &words[..], "{text}");
        let left = (100 - OPENING_WIDTH) as usize / 2;
        for row in &rows {
            let row: Vec<char> = row.chars().collect();
            assert!(row[..left].iter().all(|c| *c == ' '), "inside the panel");
        }
    }

    #[test]
    fn a_short_console_scrolls_with_up_and_down() {
        let world = kitted();
        let view = world.perception();
        let mut screen = StoryScreen::opening(Some(&view));
        let all = screen.lines(OPENING_WIDTH).len();
        let rows = 12u32;
        let top = shown(&screen, 80, rows).join("\n");
        // The arrows beside the text, besides the footer's `↑↓`.
        let arrows = |t: &str| (t.matches('↑').count() - 1, t.matches('↓').count() - 1);
        assert_eq!(arrows(&top), (0, 1), "{top}");
        assert!(top.contains("scroll   Enter continue"), "{top}");
        let last = all - visible(all, rows as i32);
        for _ in 0..all + 3 {
            assert_eq!(screen.handle(&Press::nav(Nav::Down)), ScreenAction::Stay);
        }
        assert_eq!(screen.first, last, "stops at the end");
        let bottom = shown(&screen, 80, rows).join("\n");
        assert_eq!(arrows(&bottom), (1, 0), "{bottom}");
        screen.handle(&Press::nav(Nav::Up));
        assert_eq!(screen.first, last - 1);
        for _ in 0..all {
            screen.handle(&Press::nav(Nav::Up));
        }
        assert_eq!(screen.first, 0, "stops at the top");
    }

    #[test]
    fn enter_and_esc_close_it() {
        let world = kitted();
        let view = world.perception();
        for nav in [Nav::Enter, Nav::Back] {
            let mut screen = StoryScreen::opening(Some(&view));
            assert_eq!(screen.handle(&Press::nav(nav)), ScreenAction::Close);
            let mut page = StoryScreen::page(Some(&view));
            assert_eq!(page.handle(&Press::nav(nav)), ScreenAction::Close);
        }
    }

    #[test]
    fn the_page_without_a_story_says_so() {
        let world = World::new(CONTENT.clone(), "showcase", None).expect("run");
        let view = world.perception();
        let page = StoryScreen::page(Some(&view));
        let text = shown(&page, 100, 30).join("\n");
        assert!(text.contains(UNWRITTEN), "{text}");
        assert!(text.contains("Esc close"), "{text}");
        assert!(text.contains("8 Story"), "the tab line: {text}");
        assert!(!page.is_opening());
    }
}
