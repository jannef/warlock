//! The message history (spec §6 "Messages", #207; `M` in play, ADOM's Ctrl-P): every message
//! the log keeps, oldest at the top and newest at the bottom, each wrapped to the panel at word
//! boundaries (its further lines indented). A dim first line says how many messages the log
//! keeps. It opens on the newest; the menu's up and down keys scroll a line, the list page keys
//! (`>`/PageDown, `<`/PageUp) a panel's height; Esc, Enter, or its own key closes it. It is
//! read from the perception snapshot's message log when it opens (no game time passes while it
//! is open), and it is no info page: those are about the character.
//!
//! Not a [`super::paged_list::PagedList`]: that pages lettered rows of one line each, at most
//! 26 to a page, with a cursor on one to choose; messages are no choices, take as many lines as
//! they wrap to, and scroll from the newest up.

use super::menu::{Hints, Nav, Panel, Press};
use super::more;
use super::screens::ScreenAction;
use super::{TEXT, TEXT_DIM, message_key};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use std::sync::atomic::{AtomicU64, Ordering};
use warlock_core::messages::{CAPACITY, Message};
use warlock_core::perception::PerceptionView;

/// The widest the panel gets.
const WIDTH: i32 = 100;
/// Rows of the panel besides the list: frame, margin, a gap and the footer, frame.
const EXTRA: i32 = 5;
/// How far the further lines of a message are indented.
const INDENT: usize = 2;

#[derive(Debug)]
pub struct MessagesScreen {
    /// What the player was told, oldest first.
    messages: Vec<Message>,
    /// Lines scrolled up from the newest (0: the newest at the bottom).
    back: usize,
    /// The console's size as last fitted (columns, rows), packed: presses scroll by its layout.
    console: AtomicU64,
}

impl MessagesScreen {
    /// The history of the run's messages (`None` outside a run: none).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let messages = view.map_or_else(Vec::new, |v| v.messages().iter().cloned().collect());
        Self {
            messages,
            back: 0,
            console: AtomicU64::new(u64::MAX),
        }
    }

    /// Lays the list out for a console `cols` by `rows`: drawing knows it, presses do not.
    pub fn fit(&self, cols: u32, rows: u32) {
        let size = (u64::from(cols) << 32) | u64::from(rows);
        self.console.store(size, Ordering::Relaxed);
    }

    /// Applies a key press: up and down scroll a line, the list page keys a panel's height;
    /// Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let key = press.binding.as_ref();
        let own_key = key.is_some_and(|k| bindings.owner(k) == Some(KeyCommand::Messages));
        let size = self.console.load(Ordering::Relaxed);
        let (lines, shown) = match size {
            u64::MAX => (self.lines(WIDTH, bindings).len(), 1),
            size => {
                let (cols, rows) = ((size >> 32) as i32, (size & 0xffff_ffff) as i32);
                let lines = self.lines(width(cols), bindings).len();
                (lines, visible(lines, rows))
            }
        };
        let top = lines.saturating_sub(shown);
        match press.nav {
            _ if own_key => return ScreenAction::Close,
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.back = (self.back + 1).min(top),
            Nav::Down => self.back = self.back.saturating_sub(1),
            Nav::PrevPage => self.back = (self.back + shown).min(top),
            Nav::NextPage => self.back = self.back.saturating_sub(shown),
            _ => {}
        }
        ScreenAction::Stay
    }

    /// The list's lines for a panel `width` wide: the note on what the log keeps, then every
    /// message wrapped (its further lines indented), with the keys it names as bound.
    fn lines(&self, width: i32, bindings: &Bindings) -> Vec<(String, bool)> {
        let inner = (width - 4).max(INDENT as i32 + 1) as usize;
        let note = format!("The log keeps the last {CAPACITY} messages.");
        let mut lines = vec![(note, true)];
        for msg in &self.messages {
            let text = msg.display_with(|id| message_key(bindings, id));
            let first = more::lines(&text, inner);
            let (head, rest) = first.split_first().map_or(("", &[][..]), |(h, r)| (h, r));
            lines.push((head.to_string(), false));
            // The further lines, re-wrapped narrower for their indent.
            let rest = more::lines(&rest.join(" "), inner - INDENT);
            let indent = " ".repeat(INDENT);
            lines.extend(rest.into_iter().map(|l| (format!("{indent}{l}"), false)));
        }
        lines
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let (cols, rows) = (console.cols() as i32, console.rows() as i32);
        self.fit(cols as u32, rows as u32);
        let lines = self.lines(width(cols), bindings);
        let shown = visible(lines.len(), rows);
        let panel = Panel::new(console, " Messages ", width(cols), shown as i32 + EXTRA);
        let top = lines.len().saturating_sub(shown);
        let first = top - self.back.min(top);
        let x = panel.inner_x();
        for (i, (line, note)) in lines.iter().enumerate().skip(first).take(shown) {
            let color = if *note { TEXT_DIM } else { TEXT };
            console.print(x, panel.y + 2 + (i - first) as i32, line, color);
        }
        if first > 0 {
            console.print(panel.x + panel.w - 2, panel.y + 1, "↑", TEXT_DIM);
        }
        if first + shown < lines.len() {
            console.print(panel.x + panel.w - 2, panel.y + panel.h - 3, "↓", TEXT_DIM);
        }
        let keys = Hints::of(bindings);
        let close = match bindings.key_name(KeyCommand::Messages) {
            Some(key) => format!("{key} or Esc close"),
            None => "Esc close".to_string(),
        };
        let footer = if top > 0 {
            let (up_down, next, prev) = (keys.up_down(), &keys.next_page, &keys.prev_page);
            format!("{up_down} scroll   {next} {prev} page   {close}")
        } else {
            close
        };
        panel.footer(console, &footer);
    }
}

/// The panel's width in a console `cols` wide.
fn width(cols: i32) -> i32 {
    WIDTH.min(cols)
}

/// How many of `lines` the panel shows at once in a console `rows` high.
fn visible(lines: usize, rows: i32) -> usize {
    lines.min((rows - EXTRA).max(1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use bevy::input::keyboard::KeyCode;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Action, World};

    fn press(nav: Nav) -> Press {
        Press {
            nav,
            binding: None,
            repeat: false,
            text: None,
            shift: false,
        }
    }

    fn shown(screen: &MessagesScreen, cols: u32, rows: u32) -> Vec<String> {
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

    /// A showcase run with many messages: the welcome, then a door opened and closed again and
    /// again (each a message of its own, as they alternate).
    fn chatty() -> World {
        let content = Arc::new(Content::builtin().expect("built-in content loads"));
        let mut world = World::new(content, "showcase", None).expect("scenario starts");
        for action in ["move:N", "move:N", "move:N"] {
            world.apply(&action.parse::<Action>().expect("parses"));
        }
        for _ in 0..20 {
            world.apply(&"open:N".parse::<Action>().expect("parses"));
            world.apply(&"close:N".parse::<Action>().expect("parses"));
        }
        world
    }

    #[test]
    fn opens_on_the_newest_at_the_bottom_and_scrolls_up() {
        let world = chatty();
        let view = world.perception();
        let log: Vec<String> = view.messages().iter().map(Message::display).collect();
        assert!(log.len() > 30, "{log:?}");
        let mut screen = MessagesScreen::new(Some(&view));
        let rows = shown(&screen, 80, 24).join("\n");
        assert!(rows.contains(log.last().expect("a message")), "{rows}");
        assert!(
            !rows.contains("The log keeps"),
            "the top is out of view:\n{rows}"
        );
        assert!(rows.contains("scroll"), "{rows}");
        // Up a page and more reaches the top: the note on what the log keeps.
        for _ in 0..10 {
            screen.handle(&press(Nav::PrevPage), &Bindings::default());
        }
        let rows = shown(&screen, 80, 24).join("\n");
        assert!(
            rows.contains("The log keeps the last 200 messages."),
            "{rows}"
        );
        // The welcome, its key named as bound.
        assert!(
            rows.contains("Welcome to Showcase. Press ? for controls."),
            "{rows}"
        );
        // Down a line at a time back to the newest; further does nothing.
        for _ in 0..200 {
            screen.handle(&press(Nav::Down), &Bindings::default());
        }
        assert_eq!(screen.back, 0);
    }

    #[test]
    fn long_messages_wrap_with_their_further_lines_indented() {
        let mut screen = MessagesScreen::new(None);
        screen.messages = vec![Message {
            turn: 1,
            text: "word ".repeat(40).trim().to_string(),
            count: 1,
        }];
        let rows = shown(&screen, 40, 20);
        let body: Vec<&String> = rows.iter().filter(|r| r.contains("word")).collect();
        assert!(body.len() > 2, "{rows:#?}");
        assert!(body[0].starts_with("│ word"), "{rows:#?}");
        assert!(body[1].starts_with("│   word"), "indented: {rows:#?}");
        assert!(rows.iter().all(|r| r.chars().count() == 40));
    }

    #[test]
    fn its_own_key_esc_and_enter_close_it() {
        let mut screen = MessagesScreen::new(None);
        let bindings = Bindings::default();
        let own = Press {
            binding: Some(KeyBinding::Char("M".into())),
            ..press(Nav::Other)
        };
        assert_eq!(screen.handle(&own, &bindings), ScreenAction::Close);
        assert_eq!(
            screen.handle(&press(Nav::Back), &bindings),
            ScreenAction::Close
        );
        assert_eq!(
            screen.handle(&press(Nav::Enter), &bindings),
            ScreenAction::Close
        );
        let other = Press {
            binding: Some(KeyBinding::Code(KeyCode::F12)),
            ..press(Nav::Other)
        };
        assert_eq!(screen.handle(&other, &bindings), ScreenAction::Stay);
    }
}
