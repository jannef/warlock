//! The Graveyard screen (spec §6 "Graveyard", §7 "The graveyard"; #1252), from the main menu
//! once anyone has died: every fallen character of `graveyard.toml` (`crate::graveyard`), the
//! most experienced at death first (`graveyard::by_experience`). Each takes a few lines: the
//! name, race and class (and prestige class), level, and experience; then what killed them and
//! where; then when, by the wall clock, and at what age.
//!
//! Read-only: nothing is chosen. The list is shown a page at a time, as many entries as fit
//! the console; the menu's up and down keys and the list page keys (`>`/PageDown, `<`/PageUp)
//! turn the page, Esc or Enter closes it.
//!
//! The words come from the content where its ids are known; an id another version wrote and
//! this one lacks falls back to the entry's own words (its `killer`) or to plain "Died".

use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::{TEXT, TEXT_DIM};
use crate::graveyard::{Fallen, STONE};
use crate::input::Bindings;
use crate::render::console::Console;
use crate::save::played_text;
use crate::settings::Clock;
use std::sync::atomic::{AtomicUsize, Ordering};
use warlock_core::content::Content;
use warlock_core::text::article;

/// The widest the panel gets.
const WIDTH: i32 = 80;
/// Rows of the panel besides the list: frame, margin, a gap and the footer, frame.
const EXTRA: i32 = 5;
/// Where the race and class column begins (from the panel's inner edge).
const RACE_X: i32 = 20;
/// How wide the experience column is, with the gap before it.
const XP_W: i32 = 12;
/// How far an entry's further lines are indented.
const INDENT: i32 = 2;

/// One of the fallen, in words.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    name: String,
    /// Race and class: `Dwarf Warrior`, `Elf Bard and Hunter`.
    what: String,
    level: u32,
    xp: u32,
    /// What killed them and where: `Killed by a kobold in Fyrdelf 3, The Ascmearc.`
    death: String,
    age: u32,
    /// Wall-clock seconds since 1970 (UTC); 0 unknown.
    when: u64,
}

impl Entry {
    fn new(fallen: &Fallen, content: &Content) -> Self {
        let race = (content.race(&fallen.race)).map_or(fallen.race.as_str(), |r| r.name.as_str());
        let class =
            (content.class(&fallen.class)).map_or(fallen.class.as_str(), |c| c.name.as_str());
        let mut what = [race, class]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !fallen.prestige.is_empty() {
            let prestige = (content.prestige(&fallen.prestige))
                .map_or(fallen.prestige.as_str(), |p| p.name.as_str());
            what = format!("{what} and {prestige}");
        }
        let death = format!("{}.", death_words(fallen, content));
        let name = if fallen.name.is_empty() {
            "Someone".to_string()
        } else {
            fallen.name.clone()
        };
        Self {
            name,
            what,
            level: fallen.level,
            xp: fallen.xp,
            death,
            age: fallen.age,
            when: fallen.when,
        }
    }

    /// When and how old: `Died 2026-10-21 14:32 UTC, aged 31.`; none when neither is known.
    fn died(&self, clock: Clock) -> Option<String> {
        match (self.when, self.age) {
            (0, 0) => None,
            (0, age) => Some(format!("Died aged {age}.")),
            (when, 0) => Some(format!("Died {}.", played_text(when, clock))),
            (when, age) => Some(format!("Died {}, aged {age}.", played_text(when, clock))),
        }
    }

    /// The lines after the first for an inner width of `width`: the death wrapped, then when.
    fn more(&self, width: i32, clock: Clock) -> Vec<String> {
        let mut lines = wrap(&self.death, width - INDENT);
        lines.extend(self.died(clock));
        lines
    }
}

/// What killed `fallen` and where, in words without a full stop: `Killed by a kobold in Fyrdelf
/// 3, The Ascmearc`; this screen's line, and a gravestone's (`graveyard::GraveyardStore::dead`).
pub(crate) fn death_words(fallen: &Fallen, content: &Content) -> String {
    let cause = cause_words(fallen, content);
    if fallen.place.is_empty() {
        cause
    } else {
        format!("{cause} in {}", fallen.place)
    }
}

/// What killed `fallen`, in words: `Killed by a kobold`, `Turned to stone`, `Bled to death`.
fn cause_words(fallen: &Fallen, content: &Content) -> String {
    let killed_by = |name: &str| format!("Killed by {} {name}", article(name));
    let cause = fallen.cause.as_str();
    match cause {
        STONE => return "Turned to stone".into(),
        "starved" => return "Starved to death".into(),
        "fell" => return "Fell through a collapsed floor".into(),
        "choked" => return "Choked to death".into(),
        "altar" => return "Struck down at a forgotten altar".into(),
        "own_spell" => return "Killed by their own spell".into(),
        "unseen" => return "Killed by something unseen".into(),
        _ => {}
    }
    if !fallen.killer.is_empty() {
        return killed_by(&fallen.killer);
    }
    if let Some(status) = content.status_id(cause) {
        let def = content.status(status);
        return (def.death.clone())
            .unwrap_or_else(|| format!("Died of {}", def.name.to_lowercase()));
    }
    if let Some(trap) = content.trap_id(cause) {
        return killed_by(&content.trap(trap).name);
    }
    if let Some(creature) = content.creature_id(cause) {
        return killed_by(&content.creature(creature).name);
    }
    if let Some(object) = content.object_id(cause) {
        return format!("Killed by their own {}", content.object(object).name);
    }
    "Died".into()
}

/// The fallen, a page at a time.
#[derive(Debug)]
pub struct GraveyardScreen {
    entries: Vec<Entry>,
    /// The page shown.
    page: usize,
    /// How many pages there were, as last drawn (the console decides); `usize::MAX` before.
    pages: AtomicUsize,
}

impl GraveyardScreen {
    /// The list of `fallen`, in the order given (`graveyard::by_experience`).
    pub fn new(fallen: &[Fallen], content: &Content) -> Self {
        Self {
            entries: fallen.iter().map(|f| Entry::new(f, content)).collect(),
            page: 0,
            pages: AtomicUsize::new(usize::MAX),
        }
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let last = self.pages.load(Ordering::Relaxed).saturating_sub(1);
        match press.nav {
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Down | Nav::NextPage => self.page = (self.page + 1).min(last),
            Nav::Up | Nav::PrevPage => self.page = self.page.saturating_sub(1),
            _ => {}
        }
        ScreenAction::Stay
    }

    /// Each entry's rows for an inner width of `width`, a gap after each but the last.
    fn blocks(&self, width: i32, clock: Clock) -> Vec<usize> {
        let n = self.entries.len();
        (self.entries.iter().enumerate())
            .map(|(i, e)| 1 + e.more(width, clock).len() + usize::from(i + 1 < n))
            .collect()
    }

    /// The pages, each the range of entries it shows, for `room` rows: as many entries as fit,
    /// at least one each. One empty page for no entries.
    fn pages(blocks: &[usize], room: usize) -> Vec<std::ops::Range<usize>> {
        let mut pages = Vec::new();
        let mut start = 0;
        while start < blocks.len() {
            let (mut end, mut rows) = (start, 0);
            while end < blocks.len() && (end == start || rows + blocks[end] <= room) {
                rows += blocks[end];
                end += 1;
            }
            pages.push(start..end);
            start = end;
        }
        if pages.is_empty() {
            pages.push(0..0);
        }
        pages
    }

    pub fn draw(&self, console: &mut Console, clock: Clock, bindings: &Bindings) {
        let width = WIDTH.min(console.cols() as i32) - 4;
        let blocks = self.blocks(width, clock);
        let total = (blocks.iter().sum::<usize>() as i32).max(1);
        let rows = (total + EXTRA).min(console.rows() as i32);
        let room = (rows - EXTRA).max(1) as usize;
        let pages = Self::pages(&blocks, room);
        self.pages.store(pages.len(), Ordering::Relaxed);
        let page = self.page.min(pages.len() - 1);
        let panel = Panel::new(console, " Graveyard ", WIDTH, rows);
        let x = panel.inner_x();
        let right = x + panel.inner_w();
        if pages.len() > 1 {
            let label = format!(" Page {}/{} ", page + 1, pages.len());
            let lx = panel.x + panel.w - 2 - label.chars().count() as i32;
            console.print(lx, panel.y, &label, TEXT);
        }
        if self.entries.is_empty() {
            console.print(x, panel.y + 2, "No one lies here yet.", TEXT_DIM);
        }
        let mut y = panel.y + 2;
        for entry in &self.entries[pages[page].clone()] {
            console.print(x, y, &entry.name, TEXT);
            console.print(x + RACE_X, y, &entry.what, TEXT);
            let level = format!("level {}", entry.level);
            let xp = format!("{} xp", entry.xp);
            let level_x = right - XP_W - level.chars().count() as i32;
            console.print(level_x, y, &level, TEXT);
            console.print(right - xp.chars().count() as i32, y, &xp, TEXT);
            for line in entry.more(width, clock) {
                y += 1;
                console.print(x + INDENT, y, &line, TEXT_DIM);
            }
            y += 2;
        }
        let keys = Hints::of(bindings);
        let footer = if pages.len() > 1 {
            format!(
                "{} {} {} page   Esc back",
                keys.up_down(),
                keys.next_page,
                keys.prev_page
            )
        } else {
            "Esc back".to_string()
        };
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graveyard::{by_experience, parse};

    fn content() -> Content {
        Content::builtin().expect("content")
    }

    const FALLEN: &str = r#"
[[fallen]]
name = "Brakka"
race = "dwarf"
class = "warrior"
level = 7
xp = 2900
cause = "stone"
place = "Fyrdelf 3, The Ascmearc"
age = 31
when = 1793318400

[[fallen]]
name = "Tamsin"
race = "elf"
class = "bard"
prestige = "hunter"
level = 31
cause = "kobold"
killer = "kobold"
place = "the Hub, The Showcase"

[[fallen]]
name = "Wynn"
race = "human"
class = "thief"
level = 2
xp = 140
cause = "bleeding"
"#;

    fn drawn(screen: &GraveyardScreen, cols: u32, rows: u32) -> String {
        let mut console = Console::default();
        console.resize(cols, rows);
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

    fn screen() -> GraveyardScreen {
        GraveyardScreen::new(&by_experience(parse(FALLEN)), &content())
    }

    #[test]
    fn the_fallen_are_listed_most_experienced_first_with_their_deaths() {
        let text = drawn(&screen(), 100, 30);
        let at = |s: &str| text.find(s).unwrap_or_else(|| panic!("{s}: {text}"));
        assert!(
            at("Brakka") < at("Wynn") && at("Wynn") < at("Tamsin"),
            "{text}"
        );
        assert!(text.contains("Dwarf Warrior"), "{text}");
        assert!(text.contains("Elf Bard and Hunter"), "{text}");
        assert!(
            text.contains("level 7") && text.contains("2900 xp"),
            "{text}"
        );
        assert!(
            text.contains("Turned to stone in Fyrdelf 3, The Ascmearc."),
            "{text}"
        );
        assert!(text.contains("Killed by a kobold in the Hub"), "{text}");
        assert!(text.contains("Bled to death."), "{text}");
        assert!(
            text.contains("Died 2026-10-30 00:00 UTC, aged 31."),
            "{text}"
        );
        assert!(text.contains("Esc back"), "{text}");
    }

    #[test]
    fn none_fallen_says_so() {
        let empty = GraveyardScreen::new(&[], &content());
        assert!(drawn(&empty, 100, 30).contains("No one lies here yet."));
    }

    #[test]
    fn a_short_console_pages_the_list() {
        let mut s = screen();
        let text = drawn(&s, 100, 9);
        assert!(text.contains("Page 1/3"), "{text}");
        assert!(text.contains("Brakka") && !text.contains("Wynn"), "{text}");
        assert_eq!(s.handle(&Press::nav(Nav::NextPage)), ScreenAction::Stay);
        s.handle(&Press::nav(Nav::Down));
        s.handle(&Press::nav(Nav::Down));
        let text = drawn(&s, 100, 9);
        assert!(
            text.contains("Page 3/3") && text.contains("Tamsin"),
            "{text}"
        );
        s.handle(&Press::nav(Nav::PrevPage));
        assert!(drawn(&s, 100, 9).contains("Wynn"));
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn an_unknown_cause_falls_back_to_its_killer_or_plain_words() {
        let content = content();
        let unknown = |cause: &str, killer: &str| Fallen {
            cause: cause.into(),
            killer: killer.into(),
            ..Fallen::default()
        };
        assert_eq!(
            cause_words(&unknown("wyrm_of_later", "great wyrm"), &content),
            "Killed by a great wyrm"
        );
        assert_eq!(cause_words(&unknown("no_such", ""), &content), "Died");
        assert_eq!(
            cause_words(&unknown("starved", ""), &content),
            "Starved to death"
        );
    }
}
