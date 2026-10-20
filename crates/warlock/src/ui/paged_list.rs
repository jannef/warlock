//! The one paged list of items (spec §6 "Item lists"; #188, #189): every lettered list of stacks
//! (the inventory, picking up, dropping, what to throw or eat, what goes in an equipment slot)
//! is a [`PagedList`]; never a list of its own.
//!
//! Items are grouped by item class in class order (`data/item_classes.toml`), each class under a
//! header row with its name, and keep their own order within a class. A page holds as many rows
//! as fit the screen, headers included, and at most 26 items; letters run down a page's items
//! from `a`, restarting on every page, and never name a header. A class split across pages
//! repeats its header at the top of the next. Pages turn with the `Context::Menu` commands
//! `NextListPage` and `PrevListPage` (`>` and PageDown, `<` and PageUp); there is no scrolling.
//! The list keeps a cursor on one item, and the page shown is always the cursor's: moving the
//! cursor past a page's end turns the page, and turning it puts the cursor on its first item.
//!
//! Like every screen it is a plain state machine, testable without a window. How many rows a
//! page may take is known only when drawing (the console's size): screens pass it in with
//! [`PagedList::set_room`] before laying out, and key presses read the last one.

use super::menu::{Hints, Nav, Panel};
use super::{TEXT, TEXT_DIM};
use crate::render::console::Console;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use warlock_core::action::{LETTERS, letter};
use warlock_core::content::{Content, ItemClassId};

/// One row of a page: a class's header, or an item with its letter on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row<'a> {
    Header(&'a str),
    Item { index: usize, letter: char },
}

#[derive(Debug)]
pub struct PagedList<T> {
    /// The items in list order (grouped by class), each with its class.
    items: Vec<(ItemClassId, T)>,
    /// The headers: each class's name, by class.
    names: Vec<(ItemClassId, String)>,
    /// The selected item (an index into `items`); the page shown is the one it is on.
    cursor: usize,
    /// Rows a page may take, headers included, as last drawn.
    room: AtomicUsize,
}

impl<T> PagedList<T> {
    /// The list of `items`, each with its class, grouped by class in class order (a stable sort:
    /// within a class they keep the order given), the cursor on the first.
    pub fn new(mut items: Vec<(ItemClassId, T)>, content: &Content) -> Self {
        items.sort_by_key(|(class, _)| *class);
        let mut names: Vec<(ItemClassId, String)> = Vec::new();
        for &(class, _) in &items {
            if names.last().is_none_or(|(c, _)| *c != class) {
                names.push((class, content.item_class(class).name.clone()));
            }
        }
        Self {
            items,
            names,
            cursor: 0,
            room: AtomicUsize::new(usize::MAX),
        }
    }

    /// An empty list.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            names: Vec::new(),
            cursor: 0,
            room: AtomicUsize::new(usize::MAX),
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Item `index`, in list order.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).map(|(_, item)| item)
    }

    /// Every item, in list order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|(_, item)| item)
    }

    /// The index of the selected item.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The selected item, if any.
    pub fn selected(&self) -> Option<&T> {
        self.get(self.cursor)
    }

    /// Selects item `index` (its page is shown); an index past the end selects nothing new.
    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.cursor = index;
        }
    }

    /// Moves the cursor (Up and Down, wrapping around the list) or turns the page (the list
    /// page keys) for `nav`; returns whether `nav` was one of those.
    pub fn navigate(&mut self, nav: Nav) -> bool {
        let n = self.items.len().max(1);
        match nav {
            Nav::Up => self.cursor = (self.cursor + n - 1) % n,
            Nav::Down => self.cursor = (self.cursor + 1) % n,
            Nav::NextPage => self.turn(true),
            Nav::PrevPage => self.turn(false),
            _ => return false,
        }
        true
    }

    /// Turns to the next page (`forward`) or the one before, the cursor on its first item;
    /// nothing past the first or the last page.
    pub fn turn(&mut self, forward: bool) {
        let pages = self.pages();
        let here = page_of(&pages, self.cursor);
        let to = if forward {
            (here + 1).min(pages.len() - 1)
        } else {
            here.saturating_sub(1)
        };
        self.cursor = pages[to].start;
    }

    /// Sets how many rows a page may take (headers included): drawing knows the console.
    pub fn set_room(&self, rows: usize) {
        self.room.store(rows.max(1), Ordering::Relaxed);
    }

    /// The pages, each the range of the items it shows: as many as fit the room with their
    /// headers (a class's header at the top of every page it is on), at most [`LETTERS`], and
    /// at least one each, so every item is on a page. One empty page for an empty list.
    fn pages(&self) -> Vec<Range<usize>> {
        let room = self.room.load(Ordering::Relaxed);
        let mut pages = Vec::new();
        let mut start = 0;
        while start < self.items.len() {
            let (mut end, mut rows) = (start, 0);
            while end < self.items.len() && end - start < LETTERS {
                let header = end == start || self.items[end].0 != self.items[end - 1].0;
                let need = 1 + usize::from(header);
                if end > start && rows + need > room {
                    break;
                }
                rows += need;
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

    /// How many pages there are (one for an empty list).
    pub fn page_count(&self) -> usize {
        self.pages().len()
    }

    /// The items on the page shown.
    pub fn page(&self) -> Range<usize> {
        let pages = self.pages();
        pages[page_of(&pages, self.cursor)].clone()
    }

    /// The rows of the page shown: a header where a class begins (and at the top), the items
    /// lettered from `a`.
    pub fn rows(&self) -> Vec<Row<'_>> {
        let page = self.page();
        let mut rows = Vec::new();
        for index in page.clone() {
            let class = self.items[index].0;
            if index == page.start || class != self.items[index - 1].0 {
                rows.push(Row::Header(self.name(class)));
            }
            let letter = letter(index - page.start).unwrap_or('?');
            rows.push(Row::Item { index, letter });
        }
        rows
    }

    /// The most rows any page takes: how tall a screen draws the list, so turning the page does
    /// not move its frame.
    pub fn tallest(&self) -> usize {
        let pages = self.pages();
        let rows = |page: &Range<usize>| {
            let headers = (page.clone())
                .filter(|&i| i == page.start || self.items[i].0 != self.items[i - 1].0)
                .count();
            page.len() + headers
        };
        pages.iter().map(rows).max().unwrap_or(0)
    }

    /// The item letter `c` names on the page shown, if any.
    pub fn at_letter(&self, c: char) -> Option<usize> {
        let page = self.page();
        let i = (c as usize).checked_sub('a' as usize)?;
        (c.is_ascii_lowercase() && i < page.len()).then_some(page.start + i)
    }

    /// The letter of the page shown's last item: `c` for `a-c`.
    pub fn last_letter(&self) -> Option<char> {
        letter(self.page().len().checked_sub(1)?)
    }

    /// "Page 2/3" while there is more than one page.
    pub fn page_label(&self) -> Option<String> {
        let pages = self.pages();
        (pages.len() > 1).then(|| {
            let here = page_of(&pages, self.cursor) + 1;
            format!("Page {here}/{}", pages.len())
        })
    }

    /// The paging keys for a footer, `> < page`, while there is more than one page.
    pub fn hint(&self, keys: &Hints) -> Option<String> {
        self.page_label()
            .map(|_| format!("{} {} page", keys.next_page, keys.prev_page))
    }

    /// Draws the page label on the top of `panel`'s frame, at its right end.
    pub fn draw_label(&self, console: &mut Console, panel: &Panel) {
        if let Some(label) = self.page_label() {
            let label = format!(" {label} ");
            let x = panel.x + panel.w - 2 - label.chars().count() as i32;
            if x > panel.x + 1 {
                console.print(x, panel.y, &label, TEXT);
            }
        }
    }

    fn name(&self, class: ItemClassId) -> &str {
        (self.names.iter())
            .find(|(c, _)| *c == class)
            .map_or("", |(_, name)| name.as_str())
    }
}

/// The page `pages` puts item `index` on (the last page for an index past the end).
fn page_of(pages: &[Range<usize>], index: usize) -> usize {
    (pages.iter())
        .position(|p| p.contains(&index))
        .unwrap_or(pages.len() - 1)
}

/// Draws a class's header row at (`x`, `y`).
pub fn draw_header(console: &mut Console, x: i32, y: i32, name: &str) {
    console.print(x, y, name, TEXT_DIM);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Bindings;

    fn content() -> Content {
        Content::builtin().expect("content")
    }

    fn class(c: &Content, id: &str) -> ItemClassId {
        c.item_class_id(id).expect(id)
    }

    /// `n` items of each class named, in the order given.
    fn list(c: &Content, classes: &[(&str, usize)]) -> PagedList<String> {
        let mut items = Vec::new();
        for &(id, n) in classes {
            for k in 0..n {
                items.push((class(c, id), format!("{id} {k}")));
            }
        }
        PagedList::new(items, c)
    }

    fn shown(list: &PagedList<String>) -> Vec<String> {
        (list.rows().into_iter())
            .map(|row| match row {
                Row::Header(name) => format!("[{name}]"),
                Row::Item { index, letter } => format!("{letter} {}", list.get(index).unwrap()),
            })
            .collect()
    }

    #[test]
    fn items_group_by_class_in_class_order_and_keep_their_order() {
        let c = content();
        let items = vec![
            (class(&c, "gems"), "ruby"),
            (class(&c, "melee_weapons"), "dagger"),
            (class(&c, "gems"), "opal"),
            (class(&c, "melee_weapons"), "club"),
        ];
        let list = PagedList::new(items, &c);
        assert_eq!(
            list.iter().copied().collect::<Vec<_>>(),
            ["dagger", "club", "ruby", "opal"]
        );
        let rows = list.rows();
        assert_eq!(rows[0], Row::Header("Melee weapons"));
        assert_eq!(
            rows[1],
            Row::Item {
                index: 0,
                letter: 'a'
            }
        );
        assert_eq!(rows[3], Row::Header("Gems"));
        assert_eq!(
            rows[4],
            Row::Item {
                index: 2,
                letter: 'c'
            }
        );
        assert_eq!(list.page_label(), None, "one page");
    }

    #[test]
    fn a_page_holds_what_fits_and_at_most_26_items_lettered_from_a() {
        let c = content();
        // 40 weapons on a tall screen: 26 on the first page, 14 on the second.
        let mut list = list(&c, &[("melee_weapons", 40)]);
        list.set_room(100);
        assert_eq!(list.page(), 0..26);
        assert_eq!(list.tallest(), 27);
        assert_eq!(list.last_letter(), Some('z'));
        assert_eq!(list.page_label().as_deref(), Some("Page 1/2"));
        list.turn(true);
        assert_eq!((list.page(), list.cursor()), (26..40, 26));
        // Letters restart on the new page, and its header repeats.
        let rows = shown(&list);
        assert_eq!(rows[0], "[Melee weapons]");
        assert_eq!(rows[1], "a melee_weapons 26");
        assert_eq!(list.at_letter('a'), Some(26));
        assert_eq!(list.at_letter('n'), Some(39));
        assert_eq!(list.at_letter('o'), None);
        assert_eq!(list.page_label().as_deref(), Some("Page 2/2"));
        // No page past the last; back to the first.
        list.turn(true);
        assert_eq!(list.page(), 26..40);
        list.turn(false);
        assert_eq!((list.page(), list.cursor()), (0..26, 0));
        list.turn(false);
        assert_eq!(list.page(), 0..26);
    }

    #[test]
    fn headers_take_rows_and_a_split_class_repeats_its_header() {
        let c = content();
        let mut list = list(&c, &[("melee_weapons", 3), ("missiles", 4)]);
        // Ten rows: a header and three weapons, a header and four missiles: 9 rows, one page.
        list.set_room(10);
        assert_eq!(list.page_label(), None);
        assert_eq!(shown(&list).len(), 9);
        // Six rows: the weapons and two missiles (with the second header), then the rest.
        list.set_room(6);
        assert_eq!(
            shown(&list),
            [
                "[Melee weapons]",
                "a melee_weapons 0",
                "b melee_weapons 1",
                "c melee_weapons 2",
                "[Missiles]",
                "d missiles 0",
            ]
        );
        list.navigate(Nav::NextPage);
        assert_eq!(
            shown(&list),
            ["[Missiles]", "a missiles 1", "b missiles 2", "c missiles 3"]
        );
        assert_eq!(list.tallest(), 6);
        // Too small for a header and an item still shows one item a page.
        list.set_room(1);
        assert_eq!(list.page(), 4..5, "the cursor's page");
        assert_eq!(list.page_label().as_deref(), Some("Page 5/7"));
    }

    #[test]
    fn the_cursor_moves_across_pages_and_the_page_follows_it() {
        let c = content();
        let mut list = list(&c, &[("food", 30)]);
        list.set_room(27);
        list.navigate(Nav::Up);
        assert_eq!(
            (list.cursor(), list.page()),
            (29, 26..30),
            "wraps to the end"
        );
        list.navigate(Nav::Down);
        assert_eq!((list.cursor(), list.page()), (0, 0..26));
        for _ in 0..26 {
            list.navigate(Nav::Down);
        }
        assert_eq!((list.cursor(), list.page()), (26, 26..30));
        assert!(!list.navigate(Nav::Enter));
        // The paging keys as bound name themselves in the hint.
        let keys = Hints::of(&Bindings::default());
        assert_eq!(list.hint(&keys).as_deref(), Some("> < page"));
    }

    #[test]
    fn an_empty_list_has_one_empty_page() {
        let mut list: PagedList<String> = PagedList::empty();
        assert!(list.rows().is_empty());
        assert_eq!(
            (list.page(), list.tallest(), list.page_label()),
            (0..0, 0, None)
        );
        list.navigate(Nav::NextPage);
        list.navigate(Nav::Down);
        assert_eq!(list.at_letter('a'), None);
        assert_eq!(list.last_letter(), None);
    }
}
