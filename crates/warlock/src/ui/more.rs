//! `--more--` (spec §6 "Messages", #207): this turn's messages in the message line, ADOM-style.
//!
//! The messages logged since the player last acted ([`Game::unread`]: since the run began, for
//! a new run) run on one after another in the message line's rows, wrapped at word boundaries
//! (a word longer than a row is split where the row ends). When they need more rows than the
//! message line has, they show a page at a time: every page but the last ends with `--more--`,
//! and the input mode waits ([`InputMode::More`], set by [`wait`]) for the `Context::More`
//! commands: `MoreNext` (Space or Enter) shows the next page, `MoreSkip` (no default key) or Esc
//! skips the rest. Meanwhile no other key acts, and a held key's repeats never turn a page
//! (`input::Gameplay::press`). A screen open over the run keeps its keys: `--more--` waits
//! beneath it. Once the last page shows (or the rest was skipped), play goes on and the line
//! keeps the end of this turn's messages; rows they leave free show earlier messages, one a
//! row, as before.
//!
//! Presentation only: the rules, replays, and dumps never see pages. The whole log is on the
//! message history screen (`ui::messages_screen`).

use super::{ACCENT, MESSAGE_ROWS, TEXT, TEXT_DIM, message_key};
use crate::game::Game;
use crate::input::{Bindings, InputMode};
use crate::render::console::Console;
use bevy::prelude::*;
use std::collections::VecDeque;

/// What ends every page of this turn's messages but the last.
pub const MORE: &str = "--more--";

/// `texts` run on one after another in lines of at most `width` cells, split at word
/// boundaries (a word longer than a line where the line ends), in pages of `rows` lines. Every
/// page but the last leaves room on its last line for ` --more--`. No pages for no text.
pub fn pages(texts: &[String], width: usize, rows: usize) -> Vec<Vec<String>> {
    let (width, rows) = (width.max(1), rows.max(1));
    let narrow = width.saturating_sub(MORE.chars().count() + 1).max(1);
    let mut words: VecDeque<String> = (texts.iter())
        .flat_map(|text| text.split_whitespace())
        .map(String::from)
        .collect();
    let (mut pages, mut page) = (Vec::new(), Vec::new());
    while !words.is_empty() {
        // A page's last line is cut short for `--more--` when more is left than it holds.
        let last_row = page.len() + 1 == rows;
        let room = if last_row && !fits(&words, width) {
            narrow
        } else {
            width
        };
        page.push(take_line(&mut words, room));
        if page.len() == rows {
            pages.push(std::mem::take(&mut page));
        }
    }
    if !page.is_empty() {
        pages.push(page);
    }
    pages
}

/// `text` in lines of at most `width` cells, split as [`pages`] splits them (the message
/// history's lines).
pub(super) fn lines(text: &str, width: usize) -> Vec<String> {
    let mut words: VecDeque<String> = text.split_whitespace().map(String::from).collect();
    let mut lines = Vec::new();
    while !words.is_empty() {
        lines.push(take_line(&mut words, width.max(1)));
    }
    lines
}

/// Whether all of `words` fit on one line of `width` cells.
fn fits(words: &VecDeque<String>, width: usize) -> bool {
    let cells: usize = words.iter().map(|w| w.chars().count()).sum();
    cells + words.len().saturating_sub(1) <= width
}

/// Takes the words of the next line of at most `width` cells off the front of `words`: as many
/// whole words as fit, or the start of a word too long for any line.
fn take_line(words: &mut VecDeque<String>, width: usize) -> String {
    let mut line = String::new();
    while let Some(word) = words.front_mut() {
        let (used, len) = (line.chars().count(), word.chars().count());
        if line.is_empty() && len > width {
            line = word.chars().take(width).collect();
            *word = word.chars().skip(width).collect();
            break;
        }
        if !line.is_empty() && used + 1 + len > width {
            break;
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
        words.pop_front();
    }
    line
}

/// Cells a message line may take in a console `cols` wide: a margin on either side.
fn width(cols: u32) -> usize {
    (cols as usize).saturating_sub(2)
}

/// This turn's messages in the message line's pages, for a console `cols` wide, with the keys
/// they name as bound.
fn turn_pages(game: &Game, cols: u32, bindings: &Bindings) -> Vec<Vec<String>> {
    let texts: Vec<String> = (game.unread().iter())
        .map(|msg| msg.display_with(|id| message_key(bindings, id)))
        .collect();
    pages(&texts, width(cols), MESSAGE_ROWS as usize)
}

/// How many pages this turn's messages take in the message line of a console `cols` wide.
pub fn page_count(game: &Game, cols: u32, bindings: &Bindings) -> usize {
    turn_pages(game, cols, bindings).len()
}

/// The mode after `MoreNext` on page `page` of `count`: the next page, or play when that is the
/// last (it has no `--more--` to wait at).
pub fn advance(page: usize, count: usize) -> InputMode {
    if page + 2 < count {
        InputMode::More(page + 1)
    } else {
        InputMode::Normal
    }
}

/// Waits at `--more--` when the player's action (or the run's start) logged more than the
/// message line shows at once: the first page, from plain play only (aiming or look mode that a
/// dev flag started in keeps its mode). When the window changes size, the page follows the new
/// layout.
pub(super) fn wait(
    game: Res<Game>,
    console: Res<Console>,
    bindings: Res<Bindings>,
    mut mode: ResMut<InputMode>,
    mut paged: Local<Option<(u64, u32)>>,
) {
    let key = (game.applied, console.cols());
    let turn = game.is_added() || paged.map(|(applied, _)| applied) != Some(game.applied);
    if !turn && *paged == Some(key) && !bindings.is_changed() {
        return;
    }
    *paged = Some(key);
    let count = page_count(&game, console.cols(), &bindings);
    let next = match *mode {
        InputMode::Normal | InputMode::More(_) if turn && count > 1 => InputMode::More(0),
        InputMode::More(_) if turn || count < 2 => InputMode::Normal,
        InputMode::More(page) => InputMode::More(page.min(count - 2)),
        other => other,
    };
    if *mode != next {
        *mode = next;
    }
}

/// Draws the message line in play: at `--more--` the page waited at, `--more--` after it;
/// otherwise the end of this turn's messages, earlier messages in the rows they leave free, one
/// a row, the latest lowest (dimmed once they are older than the turn before).
pub(super) fn draw(console: &mut Console, game: &Game, mode: InputMode, bindings: &Bindings) {
    let rows = MESSAGE_ROWS as usize;
    let pages = turn_pages(game, console.cols(), bindings);
    if let InputMode::More(page) = mode
        && let Some(lines) = pages.get(page.min(pages.len().saturating_sub(1)))
    {
        for (row, line) in lines.iter().enumerate() {
            console.print(1, row as i32, line, TEXT);
        }
        if let Some(last) = lines.last() {
            let x = 1 + last.chars().count() as i32 + 1;
            console.print(x, lines.len() as i32 - 1, MORE, ACCENT);
        }
        return;
    }
    let news: Vec<String> = pages.into_iter().flatten().collect();
    let news = &news[news.len().saturating_sub(rows)..];
    let unread = game.unread().len();
    let log = game.world.perception().messages();
    let earlier: Vec<_> = log.iter().take(log.len() - unread).collect();
    let earlier = &earlier[earlier.len().saturating_sub(rows - news.len())..];
    let turn = game.world.turn();
    let mut row = 0;
    for msg in earlier {
        let color = if msg.turn + 1 >= turn { TEXT } else { TEXT_DIM };
        let text = msg.display_with(|id| message_key(bindings, id));
        console.print(1, row, &text, color);
        row += 1;
    }
    for line in news {
        console.print(1, row, line, TEXT);
        row += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::content::Content;

    /// A showcase run after `walk` (all of it read), then `then` applied as the player's turn.
    fn game(walk: &str, then: &str) -> Game {
        let content = Arc::new(Content::builtin().expect("built-in content loads"));
        let mut world = World::new(content, "showcase", None).expect("scenario starts");
        for action in parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let mut game = Game::from_world(world);
        for action in parse_actions(then).expect("actions") {
            game.apply(&action);
        }
        game
    }

    /// The message line's two rows as drawn in a console `cols` wide, trimmed.
    fn line(game: &Game, mode: InputMode, cols: u32) -> Vec<String> {
        let mut console = Console::default();
        console.resize(cols, 10);
        draw(&mut console, game, mode, &Bindings::default());
        (0..MESSAGE_ROWS)
            .map(|y| {
                (0..cols as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    /// Into the stairs nook beside the old hermit, talking with him.
    const NOOK: &str = "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3 chat:SE";

    #[test]
    fn a_short_turn_shows_below_the_message_before_it() {
        let game = game("move:N*3", "open:N");
        assert_eq!(
            line(&game, InputMode::Normal, 80),
            [
                " Welcome to Showcase. Press ? for controls.",
                " You open the door."
            ]
        );
        // A turn that says nothing leaves the last two messages, one a row.
        let game = self::game("move:N*3 open:N", "move:N");
        assert_eq!(line(&game, InputMode::Normal, 80)[1], " You open the door.");
    }

    #[test]
    fn a_long_turn_shows_a_page_with_more_then_its_end() {
        let game = game(NOOK, "ask:SE:quest-rats");
        let pages = page_count(&game, 80, &Bindings::default());
        assert!(pages >= 2, "{pages}");
        let first = line(&game, InputMode::More(0), 80);
        assert!(first[0].starts_with(" The old hermit says: \"Here we don't fight."));
        assert!(first[1].ends_with(" --more--"), "{first:?}");
        assert!(first.iter().all(|l| l.chars().count() <= 80));
        // Once read (or skipped), the line keeps the end of what was said.
        let end = line(&game, InputMode::Normal, 80);
        assert!(end[1].ends_with("pay you for the quiet.\""), "{end:?}");
        assert!(!end.iter().any(|l| l.contains(MORE)));
        // A wider line takes fewer pages.
        assert!(page_count(&game, 400, &Bindings::default()) < pages);
    }

    fn texts(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn messages_run_on_and_wrap_at_words() {
        let pages = pages(&texts(&["You hit the rat.", "The rat dies."]), 20, 2);
        assert_eq!(
            pages,
            vec![vec![
                "You hit the rat. The".to_string(),
                "rat dies.".to_string()
            ]]
        );
        assert!(super::pages(&[], 20, 2).is_empty());
    }

    #[test]
    fn more_than_the_line_holds_goes_on_pages_with_room_for_more() {
        let words = "one two three four five six seven eight nine ten eleven twelve";
        let pages = pages(&texts(&[words]), 16, 2);
        // Every line fits; the last line of every page but the last leaves room for ` --more--`.
        for (i, page) in pages.iter().enumerate() {
            for line in page {
                assert!(line.chars().count() <= 16, "{line:?}");
            }
            if i + 1 < pages.len() {
                let last = page.last().expect("a line");
                assert!(last.chars().count() + 1 + MORE.len() <= 16, "{last:?}");
                assert_eq!(page.len(), 2);
            }
        }
        // Nothing is lost or reordered.
        let all: Vec<String> = pages.concat();
        assert_eq!(all.join(" "), words);
        assert!(pages.len() > 1);
    }

    #[test]
    fn the_last_page_takes_the_whole_line() {
        // Exactly two full lines: one page, no `--more--` to make room for.
        let pages = pages(&texts(&["aaaa bbbb", "cccc dddd"]), 9, 2);
        assert_eq!(
            pages,
            vec![vec!["aaaa bbbb".to_string(), "cccc dddd".to_string()]]
        );
    }

    #[test]
    fn a_word_longer_than_a_line_is_split_where_the_line_ends() {
        let pages = pages(&texts(&["abcdefghijkl"]), 5, 1);
        assert_eq!(pages.concat().concat(), "abcdefghijkl");
        assert!(pages.iter().flatten().all(|l| l.chars().count() <= 5));
        // Tiny lines still make progress.
        assert_eq!(
            super::pages(&texts(&["abc"]), 0, 0).concat().concat(),
            "abc"
        );
    }

    #[test]
    fn advancing_stops_at_the_last_page() {
        assert_eq!(advance(0, 3), InputMode::More(1));
        assert_eq!(advance(1, 3), InputMode::Normal);
        assert_eq!(advance(0, 2), InputMode::Normal);
        assert_eq!(advance(5, 2), InputMode::Normal);
    }
}
