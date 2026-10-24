//! The item description (spec §6 "Item description"): one stack the player carries, as they know
//! it. Opened from the inventory (a stack's letter) or the equipment screen (a slot's upper-case
//! letter), over the screen it came from.
//!
//! It names the item (in its rarity's color, spec §8 "Magic items"), says what it is (rarity,
//! base, every part it is made of, weight) and whether it is worn or wielded, then its base
//! stats in words (`warlock_core::magic::describe`): how it is held or worn, its damage dice and
//! criticals, how far it shoots or is thrown, what it needs, its Defence. A magic or rare item that is
//! identified lists its item level and its affixes in plain language ("Sharp: +2 damage", "of
//! Celerity: attacks faster"); one that is not says its magic is unknown. Speed is said in words;
//! nothing about the player's own to-hit or modifiers is shown.
//!
//! Read-only: Esc, Enter, or Back closes it. It is read from the perception snapshot when it
//! opens; no game time passes while it is open.

use super::menu::{Nav, Panel, Press, wrap};
use super::screens::ScreenAction;
use super::{MAGIC, TEXT, TEXT_DIM, rarity_color};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::color::Rgb;
use warlock_core::content::Rarity;
use warlock_core::equipment;
use warlock_core::magic::describe::{self, Line, LineKind};
use warlock_core::perception::PerceptionView;

/// Panel width: the description's lines wrap within it.
const WIDTH: i32 = 64;
/// Width of the "Prefix" / "Suffix" column.
const LABEL_W: i32 = 8;

#[derive(Debug)]
pub struct DescribeScreen {
    glyph: char,
    fg: Rgb,
    /// What it is called, as the player knows it.
    title: String,
    rarity: Rarity,
    /// What the inventory says of it if it is worn or wielded: "wielded".
    mark: Option<String>,
    lines: Vec<Line>,
}

impl DescribeScreen {
    /// The description of inventory stack `item` (`None`, or a letter the player does not
    /// carry: an empty screen that says so).
    pub fn new(view: Option<&PerceptionView<'_>>, item: Option<usize>) -> Self {
        let empty = Self {
            glyph: ' ',
            fg: TEXT,
            title: String::new(),
            rarity: Rarity::Normal,
            mark: None,
            lines: Vec::new(),
        };
        let Some(view) = view else { return empty };
        let Some((item, stack)) = item.and_then(|i| Some((i, view.inventory().get(i)?))) else {
            return empty;
        };
        let content = view.content();
        let def = content.object(stack.object);
        Self {
            glyph: def.glyph,
            fg: stack.fg(content),
            title: capitalized(&stack.title(content)),
            rarity: stack.rarity(),
            mark: equipment::mark(content, view.equipment(), item),
            lines: describe::lines(stack, content),
        }
    }

    /// Applies a key press: Esc, Back, or Enter closes it.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        match press.nav {
            Nav::Back | Nav::Enter => ScreenAction::Close,
            _ => ScreenAction::Stay,
        }
    }

    /// The lines as drawn within `width`, a blank row between what it is, its stats, and its
    /// magic: each with a label in the left column ("Prefix", on an affix's first row), its x
    /// offset, its text, and its color.
    fn rows(&self, width: i32) -> Vec<Row> {
        let section = |k: LineKind| match k {
            LineKind::Base => 0,
            LineKind::Stat => 1,
            LineKind::Note | LineKind::Affix(_) => 2,
        };
        let mut rows = Vec::new();
        let mut last: Option<LineKind> = None;
        for line in &self.lines {
            if last.is_some_and(|l| section(l) != section(line.kind)) {
                rows.push(Row::default());
            }
            last = Some(line.kind);
            let (label, dx, color) = match line.kind {
                LineKind::Affix(kind) => (Some(kind.name()), LABEL_W, MAGIC),
                LineKind::Stat => (None, 0, TEXT),
                LineKind::Base | LineKind::Note => (None, 0, TEXT_DIM),
            };
            for (i, text) in wrap(&line.text, width - dx).into_iter().enumerate() {
                rows.push(Row {
                    label: label.filter(|_| i == 0),
                    dx,
                    text,
                    color,
                });
            }
        }
        rows
    }

    pub fn draw(&self, console: &mut Console, _bindings: &Bindings) {
        let width = WIDTH.min(console.cols() as i32) - 4;
        let rows = self.rows(width);
        // Rows: margin, the name, a gap, the description, a gap, the footer, the frame.
        let h = rows.len() as i32 + 7;
        let panel = Panel::new(console, " Item ", WIDTH, h);
        let x = panel.inner_x();
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.title.is_empty() {
            if fits(panel.y + 2) {
                console.print(x, panel.y + 2, "You carry no such thing.", TEXT_DIM);
            }
        } else if fits(panel.y + 2) {
            console.print(x, panel.y + 2, &self.glyph.to_string(), self.fg);
            let mark = (self.mark.as_ref()).map_or(String::new(), |m| format!(" ({m})"));
            let name: String = format!("{}{mark}", self.title)
                .chars()
                .take((width - 2).max(0) as usize)
                .collect();
            console.print(x + 2, panel.y + 2, &name, rarity_color(self.rarity, TEXT));
        }
        for (i, row) in rows.iter().enumerate() {
            let y = panel.y + 4 + i as i32;
            if !fits(y) {
                break;
            }
            if let Some(label) = row.label {
                console.print(x, y, label, TEXT_DIM);
            }
            console.print(x + row.dx, y, &row.text, row.color);
        }
        panel.footer(console, "Esc close");
    }
}

/// One row of the description as drawn.
#[derive(Clone, Debug)]
struct Row {
    /// In the left column: "Prefix" or "Suffix" on an affix's first row.
    label: Option<&'static str>,
    dx: i32,
    text: String,
    color: Rgb,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            label: None,
            dx: 0,
            text: String::new(),
            color: TEXT,
        }
    }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Action, Positions, World};

    /// The showcase, standing on the storeroom's magic arms with them picked up: a the magic ring
    /// (unidentified), b the rare bow (unidentified), c the rare helm, d the magic sword, its
    /// pommel an emerald.
    fn armed() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 pickup \
                    move:E pickup";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        assert_eq!(world.perception().inventory().len(), 4);
        world
    }

    fn text(screen: &DescribeScreen) -> Vec<String> {
        let mut console = Console::default();
        console.resize(80, 40);
        screen.draw(&mut console, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn has(text: &[String], needle: &str) -> bool {
        text.iter().any(|l| l.contains(needle))
    }

    #[test]
    fn identified_items_list_their_affixes_and_others_do_not() {
        let mut world = armed();
        let view = world.perception();
        let names: Vec<String> = (view.inventory().iter())
            .map(|s| s.describe(view.content()))
            .collect();
        assert_eq!(
            names,
            [
                "a silver ring (magic)",
                "a short bow (rare)",
                "Dread Visage, an iron helm",
                "a Sharp emerald-pommelled long sword of the Bear",
            ]
        );
        let sword = text(&DescribeScreen::new(Some(&view), Some(3)));
        assert!(
            has(&sword, ") Sharp emerald-pommelled long sword of the Bear"),
            "{sword:#?}"
        );
        // What it is made of wraps; its gem's size and cut follow.
        assert!(
            has(&sword, "Magic emerald-pommelled long sword ·"),
            "{sword:#?}"
        );
        assert!(has(&sword, "Pommel: small emerald, fine cut"), "{sword:#?}");
        assert!(has(&sword, "Damage 1d8; criticals on 20–24, ×2 damage"));
        assert!(has(&sword, "Slashing or piercing damage"));
        assert!(has(&sword, "Item level 3"));
        assert!(has(&sword, "Prefix  Sharp: +2 damage"), "{sword:#?}");
        assert!(has(&sword, "Suffix  of the Bear: +2 to Strength"));
        let helm = text(&DescribeScreen::new(Some(&view), Some(2)));
        assert!(has(&helm, "[ Dread Visage"));
        assert!(has(&helm, "Rare iron helm · iron · 1.5 kg"));
        assert!(has(&helm, "Radiant: glows"));
        let bow = text(&DescribeScreen::new(Some(&view), Some(1)));
        assert!(has(&bow, "} Short bow"));
        assert!(has(&bow, "Unidentified"));
        assert!(!has(&bow, "Hunter") && !has(&bow, "Storm Song") && !has(&bow, "Prefix"));
        // Identified, its affixes show.
        world.apply(&Action::Identify(Positions::of([1])));
        let view = world.perception();
        let bow = text(&DescribeScreen::new(Some(&view), Some(1)));
        assert!(has(&bow, "} Storm Song"), "{bow:#?}");
        assert!(has(&bow, "Prefix  Hunter's: +2 cells of range"), "{bow:#?}");
        assert!(has(&bow, "Suffix  of Celerity: attacks faster"));
        // Worn things say so; Esc closes.
        world.apply(&Action::Equip {
            slot: "head".into(),
            item: 2,
        });
        let view = world.perception();
        let mut screen = DescribeScreen::new(Some(&view), Some(2));
        assert!(has(&text(&screen), "Dread Visage (being worn)"));
        let b = Bindings::default();
        assert_eq!(screen.handle(&Press::typed('a'), &b), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn nothing_to_describe_says_so_and_small_consoles_draw() {
        let screen = DescribeScreen::new(None, Some(0));
        assert!(has(&text(&screen), "You carry no such thing."));
        let world = armed();
        let view = world.perception();
        let screen = DescribeScreen::new(Some(&view), Some(3));
        for (cols, rows) in [(30, 8), (12, 4), (0, 0)] {
            let mut console = Console::default();
            console.resize(cols, rows);
            screen.draw(&mut console, &Bindings::default());
        }
    }
}
