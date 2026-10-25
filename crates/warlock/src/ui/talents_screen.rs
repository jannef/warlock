//! The Talents page (spec §6 "Talents", §8 "Talents"; #236): page 7 of the info pages, also
//! reached by its own rebindable command (`KeyCommand::Talents`, unbound by default). First the
//! player's talents, grouped by tree, each with its rank ("Hardy 2/3") and where it came from
//! when they did not pick it (their class, their birth sign); then, dimmer, every other talent
//! they could still pick, by tree, with what its next rank needs ("Needs: Toughness 14,
//! Healthy"), each part met in the normal colour and each part not met dimmed, so a build can be
//! planned without the wiki. The class's coming abilities are not listed (they are learnt by
//! levelling). The selected talent's description shows below. Words only: a talent's rule
//! numbers are the wiki's.
//!
//! Read-only: ↑↓ choose (the list scrolls); Esc, Enter, or its own key closes it. Read from the
//! perception snapshot when it opens.

use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::color::Rgb;
use warlock_core::content::TreeId;
use warlock_core::perception::{NeedView, PerceptionView};

/// Width of the panel: the character screen's.
const WIDTH: i32 = 72;
/// Rows besides the list: the frame, a margin, a gap, the description (three lines), a gap,
/// the footer, the frame.
const EXTRA: i32 = 9;

/// One talent as listed.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Item {
    name: String,
    /// "2/3" for a ranked one: the rank held (one of the player's) or the next.
    rank: String,
    /// Where one of the player's came from, if they did not pick it: "class", "birth sign".
    from: Option<&'static str>,
    description: String,
    /// What the next rank needs (another talent's), each part met or not.
    needs: Vec<NeedView>,
    /// One of the player's own.
    own: bool,
}

/// One line of the list: a heading, or a line of a talent (its name, or its needs).
#[derive(Clone, Debug, PartialEq, Eq)]
enum Line {
    /// "Your talents", "Other talents".
    Section(String),
    Tree(String),
    Name(usize),
    Needs(usize),
}

#[derive(Debug)]
pub struct TalentsScreen {
    items: Vec<Item>,
    lines: Vec<Line>,
    /// The selected talent, an index into `items`.
    selected: usize,
}

impl TalentsScreen {
    /// The page for the player's talents and the others (`None` outside a run: none).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let mut screen = Self {
            items: Vec::new(),
            lines: Vec::new(),
            selected: 0,
        };
        let Some(view) = view else {
            return screen;
        };
        let book = view.content().talents();
        // The player's own, tree by tree in tree order, each tree in the order taken.
        let mut own = view.own_talents();
        own.sort_by_key(|t| t.tree);
        let mut tree: Option<TreeId> = None;
        screen.lines.push(Line::Section("Your talents".into()));
        if own.is_empty() {
            screen.lines.push(Line::Tree("none yet".into()));
        }
        for talent in own {
            if tree != Some(talent.tree) {
                tree = Some(talent.tree);
                screen
                    .lines
                    .push(Line::Tree(book.tree(talent.tree).name.clone()));
            }
            let from = match () {
                () if talent.class => Some("class"),
                () if talent.race => Some("race"),
                () if talent.granted => Some("birth sign"),
                () => None,
            };
            screen.lines.push(Line::Name(screen.items.len()));
            screen.items.push(Item {
                name: talent.name,
                rank: rank(talent.rank, talent.ranks),
                from,
                description: talent.description,
                needs: Vec::new(),
                own: true,
            });
        }
        // Every other talent that may still be picked: not given, not held to its last rank.
        let mut tree: Option<TreeId> = None;
        let mut section = false;
        for talent in view.talents() {
            if talent.granted || book.tree(talent.tree).class || talent.held >= talent.ranks {
                continue;
            }
            if !section {
                section = true;
                screen.lines.push(Line::Section("Other talents".into()));
            }
            if tree != Some(talent.tree) {
                tree = Some(talent.tree);
                screen
                    .lines
                    .push(Line::Tree(book.tree(talent.tree).name.clone()));
            }
            let index = screen.items.len();
            screen.lines.push(Line::Name(index));
            screen.lines.push(Line::Needs(index));
            screen.items.push(Item {
                name: talent.name,
                rank: match talent.held {
                    0 => String::new(),
                    held => rank(held + 1, talent.ranks),
                },
                from: None,
                description: talent.description,
                needs: talent.needs,
                own: false,
            });
        }
        screen
    }

    /// Applies a key press: ↑↓ choose; Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let key = press.binding.as_ref();
        let own_key = key.is_some_and(|k| bindings.owner(k) == Some(KeyCommand::Talents));
        let n = self.items.len().max(1);
        match press.nav {
            _ if own_key => return ScreenAction::Close,
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.selected = (self.selected + n - 1) % n,
            Nav::Down => self.selected = (self.selected + 1) % n,
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let h = self.lines.len() as i32 + EXTRA;
        let panel = Panel::new(console, " Talents ", WIDTH, h);
        super::info_tabs::draw_tabs(console, &panel, bindings, ScreenName::Talents);
        let x = panel.inner_x();
        let room = (panel.h - EXTRA).max(1) as usize;
        // Scroll to keep the selected talent's lines (and its tree's heading) in view.
        let at = (self.lines.iter())
            .rposition(|l| matches!(l, Line::Needs(i) | Line::Name(i) if *i == self.selected))
            .unwrap_or(0);
        let first = (at + 1)
            .saturating_sub(room)
            .min(self.lines.len().saturating_sub(room));
        let first = match self.lines.get(first) {
            // Never start on a talent's needs, away from its name.
            Some(Line::Needs(_)) if first < at => first + 1,
            _ => first,
        };
        for (line, y) in self.lines.iter().skip(first).take(room).zip(panel.y + 2..) {
            match line {
                Line::Section(text) => {
                    console.print(x, y, text, ACCENT);
                }
                Line::Tree(text) => {
                    console.print(x + 2, y, text, TEXT_DIM);
                }
                Line::Name(i) => self.draw_name(console, x, y, *i),
                Line::Needs(i) => draw_needs(console, x + 8, y, &self.items[*i].needs, &panel),
            }
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        if first + room < self.lines.len() {
            console.print(
                panel.x + panel.w - 3,
                panel.y + 2 + room as i32,
                "↓",
                TEXT_DIM,
            );
        }
        if let Some(item) = self.items.get(self.selected) {
            let lines = wrap(&item.description, panel.inner_w());
            for (line, y) in lines.iter().take(3).zip(panel.y + panel.h - 6..) {
                if y > panel.y + 2 {
                    console.print(x, y, line, TEXT_DIM);
                }
            }
        }
        let up_down = Hints::of(bindings).up_down();
        let footer = match bindings.keys_of(KeyCommand::Talents).first() {
            Some(key) => format!("{up_down} choose   {} or Esc close", key.display()),
            None => format!("{up_down} choose   Esc close"),
        };
        panel.footer(console, &footer);
    }

    /// A talent's name line: the `>` marker, the name and rank, and where it came from.
    fn draw_name(&self, console: &mut Console, x: i32, y: i32, i: usize) {
        let item = &self.items[i];
        let selected = i == self.selected;
        let color = match (selected, item.own) {
            (true, _) => ACCENT,
            (false, true) => TEXT,
            (false, false) => TEXT_DIM,
        };
        if selected {
            console.print(x, y, ">", ACCENT);
        }
        let mut name = item.name.clone();
        if !item.rank.is_empty() {
            name = format!("{name} {}", item.rank);
        }
        console.print(x + 4, y, &name, color);
        if let Some(from) = item.from {
            let at = x + 4 + name.chars().count() as i32 + 2;
            console.print(at, y, &format!("({from})"), TEXT_DIM);
        }
    }
}

/// "2/3": rank `rank` of `ranks`, for a talent with more than one; else nothing.
fn rank(rank: u8, ranks: u8) -> String {
    match ranks {
        0 | 1 => String::new(),
        _ => format!("{rank}/{ranks}"),
    }
}

/// The needs line: "Needs: Toughness 14, Healthy", each part met in the normal colour and each
/// part not met dimmed (and a whole line of needs met reads the same without colour).
fn draw_needs(console: &mut Console, x: i32, y: i32, needs: &[NeedView], panel: &Panel) {
    let end = panel.x + panel.w - 2;
    let mut at = x;
    let mut print = |console: &mut Console, text: &str, color: Rgb| {
        let room = (end - at).max(0) as usize;
        let shown: String = text.chars().take(room).collect();
        console.print(at, y, &shown, color);
        at += shown.chars().count() as i32;
    };
    print(console, "Needs: ", TEXT_DIM);
    if needs.is_empty() {
        print(console, "nothing", TEXT);
    }
    for (i, need) in needs.iter().enumerate() {
        if i > 0 {
            print(console, ", ", TEXT_DIM);
        }
        print(console, &need.words, if need.met { TEXT } else { TEXT_DIM });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::content::Content;

    fn text(screen: &TalentsScreen, rows: u32) -> String {
        let mut console = Console::default();
        console.resize(80, rows);
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
    fn lists_the_players_talents_then_the_others_with_their_needs() {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec = "dwarf,male,warrior".parse::<warlock_core::character::CharacterSpec>();
        let spec = spec
            .expect("a character")
            .with_talents(vec!["healthy".into(), "lean".into()]);
        let world =
            World::start_run(content, "showcase", Some(1), Some(&spec), None, true).expect("world");
        let mut screen = TalentsScreen::new(Some(&world.perception()));
        let drawn = text(&screen, 40);
        assert!(drawn.contains("Your talents"), "{drawn}");
        assert!(drawn.contains("Healthy"), "{drawn}");
        assert!(drawn.contains("Other talents"), "{drawn}");
        assert!(drawn.contains("Needs:"), "{drawn}");
        // Scrolling down reaches the last talent.
        let n = screen.items.len();
        for _ in 1..n {
            screen.handle(&Press::nav(Nav::Down), &Bindings::default());
        }
        let last = screen.items[n - 1].name.clone();
        assert!(text(&screen, 40).contains(&last), "{last}");
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &Bindings::default()),
            ScreenAction::Close
        );
    }
}
