//! Object screens (spec §6): the inventory (`i`), and the selection screens for picking up from a
//! pile (`,` or `g` where several objects lie), for dropping (`d`), and for choosing what to
//! throw (`T`: the throwing weapons the player carries; choosing one aims it, `ui::target`).
//!
//! Each is the one paged list (`paged_list`, spec §6 "Item lists"): stacks grouped by item class
//! under a header per class, lettered down each page from `a`, with their glyph, name, and
//! weight; `>` and `<` (PageDown, PageUp) turn the pages. The inventory and the drop screen mark
//! what is worn or wielded ("(being worn)", "(wielded)"; dropping it takes it off first). Names
//! are in their rarity's color (spec §8 "Magic items"): magic blue, rare yellow. The inventory
//! takes no action and also shows what the player carries and weighs, their burden when they
//! have one, and the weights at which it changes (spec §8 "Burden"); a stack's letter there
//! opens its description (`describe_screen`). In a selection screen a letter marks or unmarks
//! its stack (`+` marked, `-` not: never color alone), `,` marks everything on every page (or,
//! with everything marked, nothing), the arrows move the `>` cursor (across pages) and Space
//! marks its row, and Enter picks up or drops the marked stacks (the cursor's row if none is
//! marked); marks belong to stacks, so they stay as the pages turn. That is one action, recorded
//! like any other, naming the stacks by position. Esc cancels. The list is read from the
//! perception snapshot when the screen opens; no game time passes while it is open, so it cannot
//! go stale.
//!
//! The eat screen (`E`, spec §8 "Hunger") lists what can be eaten: first what lies here
//! ("lying here"; the cursor starts on it, as ADOM asks about the floor first), then what the
//! player carries. A letter or Enter on the `>` row eats it.
//!
//! The read screen (`r`, spec §8 "Scrolls") lists the scrolls the player carries: choosing a
//! spell scroll aims it (`ui::target`), choosing the scroll of identify replaces the list with
//! the unidentified stacks carried, and choosing one of those reads the scroll on it.
//!
//! The study screen (`L`, spec §8 "Spells") lists the scrolls the player could study (their
//! type known, their spell not: `PerceptionView::studies`), each marked with the spell it
//! teaches and the chance in words; choosing one asks in the one dialog (study it, or not),
//! and a study is [`Action::Study`].

use super::character_screen::{fit_joined, thresholds};
use super::menu::{Hints, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::target::AimKind;
use super::{ACCENT, TEXT, TEXT_DIM, rarity_color};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::burden::{Burden, Load};
use warlock_core::color::Rgb;
use warlock_core::content::Rarity;
use warlock_core::equipment;
use warlock_core::object::{Stack, Weight, format_weight};
use warlock_core::perception::PerceptionView;
use warlock_core::{Action, Meal, Positions, ReadTarget};

/// Which list the screen shows, and what choosing does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectList {
    /// What the player carries; read-only.
    Inventory,
    /// The pile on the player's cell: choose what to pick up.
    PickUp,
    /// The inventory: choose what to drop.
    Drop,
    /// The throwing weapons in the inventory: choose one to throw.
    Throw,
    /// Food lying here, then food in the pack: choose one to eat.
    Eat,
    /// The scrolls in the pack: choose one to read.
    Read,
    /// The scrolls the player could study: choose one to learn its spell.
    Study,
    /// The unidentified stacks in the pack: choose one to read the scroll of identify at this
    /// position on.
    Identify(usize),
}

impl ObjectList {
    fn title(self) -> &'static str {
        match self {
            ObjectList::Inventory => " Inventory ",
            ObjectList::PickUp => " Pick up ",
            ObjectList::Drop => " Drop ",
            ObjectList::Throw => " Throw ",
            ObjectList::Eat => " Eat ",
            ObjectList::Read => " Read ",
            ObjectList::Study => " Study ",
            ObjectList::Identify(_) => " Identify ",
        }
    }

    fn empty(self) -> &'static str {
        match self {
            ObjectList::Inventory | ObjectList::Drop => "You are not carrying anything.",
            ObjectList::PickUp => "There is nothing here.",
            ObjectList::Throw => "You have nothing to throw.",
            ObjectList::Eat => "You have nothing to eat.",
            ObjectList::Read => "You have nothing to read.",
            ObjectList::Study => "You carry no scroll whose spell you could learn.",
            ObjectList::Identify(_) => "You carry nothing unidentified.",
        }
    }

    /// The panel's width: the inventory's is wider, for what the player carries, weighs, and
    /// may carry on one line each.
    fn width(self) -> i32 {
        match self {
            ObjectList::Inventory => super::info_tabs::panel_width(72),
            ObjectList::PickUp
            | ObjectList::Drop
            | ObjectList::Throw
            | ObjectList::Eat
            | ObjectList::Read
            | ObjectList::Identify(_) => 60,
            // Room for the spell each teaches and the chance.
            ObjectList::Study => 80,
        }
    }

    /// Whether choosing is one pick (a letter or Enter) rather than marking several.
    fn picks_one(self) -> bool {
        matches!(
            self,
            ObjectList::Throw
                | ObjectList::Eat
                | ObjectList::Read
                | ObjectList::Study
                | ObjectList::Identify(_)
        )
    }
}

/// Where a row's stack is: the pile on the player's cell, or the pack, by position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Floor(usize),
    Pack(usize),
}

impl Item {
    fn position(self) -> usize {
        match self {
            Item::Floor(i) | Item::Pack(i) => i,
        }
    }
}

/// One stack as the screen shows it, and where it is.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    item: Item,
    glyph: char,
    fg: Rgb,
    name: String,
    /// What the inventory says of it if it is worn or wielded: "being worn".
    mark: Option<String>,
    /// Grams.
    weight: u64,
    /// Its name's color: by its rarity (spec §8 "Magic items").
    rarity: Rarity,
    /// A scroll read at a cell (a spell): choosing it to read aims it.
    aimed: bool,
    /// A scroll to study: what it is ("scroll of frost"), the spell it teaches, and the chance
    /// in words, for the dialog.
    study: Option<(String, String, &'static str)>,
}

#[derive(Debug)]
pub struct ObjectScreen {
    list: ObjectList,
    /// The stacks listed, grouped by class and paged (`paged_list`).
    entries: PagedList<Entry>,
    /// The marked stacks of a selection screen, by position: marks belong to stacks, not to
    /// the letters a page gives them, so they stay across pages.
    marked: Positions,
    /// The player's weight (shown in the inventory).
    weight: Weight,
    /// What the player carries against what they can (shown in the inventory); `None` outside
    /// a run.
    load: Option<Load>,
    /// The inventory stack the player last asked to look at (its position): what the item
    /// description opened from here describes.
    looked: Option<usize>,
}

impl ObjectScreen {
    /// The screen for `list`, read from what the player perceives (`None` outside a run: an
    /// empty list).
    pub fn new(list: ObjectList, view: Option<&PerceptionView<'_>>) -> Self {
        let Some(view) = view else {
            return Self {
                list,
                entries: PagedList::empty(),
                marked: Positions::default(),
                weight: Weight::default(),
                load: None,
                looked: None,
            };
        };
        let content = view.content();
        let (pile, pack) = (view.objects(view.viewer()), view.inventory());
        let edible = |s: &Stack| content.object(s.object).food.is_some();
        let reading = |s: &Stack| content.object(s.object).read.as_ref();
        let studies = view.studies();
        let items: Vec<Item> = match list {
            ObjectList::PickUp => (0..pile.len()).map(Item::Floor).collect(),
            ObjectList::Inventory | ObjectList::Drop => (0..pack.len()).map(Item::Pack).collect(),
            ObjectList::Throw => (0..pack.len())
                .filter(|&i| view.is_throwing_weapon(i))
                .map(Item::Pack)
                .collect(),
            ObjectList::Eat => ((0..pile.len())
                .filter(|&i| edible(&pile[i]))
                .map(Item::Floor))
            .chain(
                (0..pack.len())
                    .filter(|&i| edible(&pack[i]))
                    .map(Item::Pack),
            )
            .collect(),
            ObjectList::Read => (0..pack.len())
                .filter(|&i| reading(&pack[i]).is_some())
                .map(Item::Pack)
                .collect(),
            ObjectList::Study => studies.iter().map(|s| Item::Pack(s.item)).collect(),
            // The scroll read is not among them: reading it names its own kind anyway.
            ObjectList::Identify(scroll) => (0..pack.len())
                .filter(|&i| i != scroll && !view.lore().knows(&pack[i]))
                .map(Item::Pack)
                .collect(),
        };
        let unpaid = view.unpaid();
        let entries = (items.into_iter())
            .map(|item| {
                let s = match item {
                    Item::Floor(i) => &pile[i],
                    Item::Pack(i) => &pack[i],
                };
                let def = content.object(s.object);
                // The inventory marks what is worn or wielded; a pile has nothing worn, and
                // what to eat says what lies here. In a shop (spec §7 "Shops") goods lying
                // there say their price, and what the player has not paid for what they owe.
                let mark = match (item, list) {
                    (Item::Floor(_), ObjectList::Eat) => Some("lying here".to_string()),
                    (Item::Floor(_), _) => None,
                    (Item::Pack(i), _) => equipment::mark(content, view.equipment(), i),
                };
                let price = match item {
                    Item::Floor(_) => {
                        (view.price_tag(view.viewer(), s)).map(|g| format!("for sale, {g} gold"))
                    }
                    Item::Pack(i) => {
                        (unpaid.get(i).copied().flatten()).map(|g| format!("unpaid, {g} gold"))
                    }
                };
                // Hound's nose (spec §8 "Birth sign"): what to eat says how fresh it smells.
                let smell = (list == ObjectList::Eat)
                    .then(|| view.smell(s))
                    .flatten()
                    .map(|f| format!("smells {}", f.word()));
                // The study list: the spell it teaches, and the chance in words.
                let study = match (list, item) {
                    (ObjectList::Study, Item::Pack(i)) => {
                        (studies.iter()).find(|s| s.item == i).and_then(|study| {
                            let spell = content.spell_of(study.spell)?.name.clone();
                            Some((view.lore().title(s), spell, study.chance))
                        })
                    }
                    _ => None,
                };
                let teaches =
                    (study.as_ref()).map(|(_, spell, chance)| format!("{spell}: {chance}"));
                let marks: Vec<String> = (teaches.into_iter())
                    .chain(mark)
                    .chain(smell)
                    .chain(price)
                    .collect();
                let entry = Entry {
                    item,
                    glyph: def.glyph,
                    fg: view.lore().fg(s),
                    name: view.lore().describe(s),
                    mark: (!marks.is_empty()).then(|| marks.join(") (")),
                    weight: s.weight(content),
                    rarity: s.rarity(),
                    aimed: reading(s).is_some_and(|r| r.range().is_some()),
                    study,
                };
                (view.item_class(s), entry)
            })
            .collect();
        Self {
            list,
            entries: PagedList::new(entries, content),
            marked: Positions::default(),
            weight: view.own_weight(),
            load: Some(view.own_load()),
            looked: None,
        }
    }

    /// What choosing `item` does on a screen of one pick: aim it to throw, eat it, aim the
    /// spell scroll to read it, pick what to read the scroll of identify on (which remembers
    /// the scroll), or read that scroll on it.
    fn pick(&mut self, item: Item) -> ScreenAction {
        let aimed = self.entries.iter().any(|e| e.item == item && e.aimed);
        match (self.list, item) {
            (ObjectList::Eat, Item::Floor(i)) => ScreenAction::Act(Action::Eat(Meal::Floor(i))),
            (ObjectList::Eat, Item::Pack(i)) => ScreenAction::Act(Action::Eat(Meal::Pack(i))),
            (ObjectList::Read, item) if aimed => ScreenAction::Aim(AimKind::Read(item.position())),
            (ObjectList::Read, item) => {
                self.looked = Some(item.position());
                ScreenAction::Replace(super::ScreenName::Identify)
            }
            (ObjectList::Study, item) => {
                let entry = self.entries.iter().find(|e| e.item == item);
                let Some((scroll, spell, chance)) = entry.and_then(|e| e.study.clone()) else {
                    return ScreenAction::Stay;
                };
                let text = format!(
                    "Study the {scroll}? Learning {spell} will be {chance}. The scroll will be \
                     used up, and once you begin you cannot stop of your own accord."
                );
                ScreenAction::Dialog(Box::new(
                    super::dialog_screen::Dialog::new("Study", text)
                        .accept(
                            "Study it",
                            Action::Study {
                                item: item.position(),
                            },
                        )
                        .cancel("Don't study it", ScreenAction::Close),
                ))
            }
            (ObjectList::Identify(scroll), item) => ScreenAction::Act(Action::Read {
                item: scroll,
                target: Some(ReadTarget::Item(item.position())),
            }),
            (_, item) => ScreenAction::Aim(AimKind::Throw(item.position())),
        }
    }

    pub fn list(&self) -> ObjectList {
        self.list
    }

    /// The inventory stack the player asked to look at, if any.
    pub fn looked(&self) -> Option<usize> {
        self.looked
    }

    /// The stack a letter names on the page shown, if any.
    fn lettered(&self, press: &Press) -> Option<Item> {
        let index = self.entries.at_letter(press.text?)?;
        self.entries.get(index).map(|e| e.item)
    }

    /// Applies a key press. Letters name the stacks of the page shown (never commands, even
    /// `h j k l`), and the list keys turn its pages. In the inventory a stack's letter opens its
    /// description (spec §6 "Item description"); it closes on Esc, Enter, or its own key when
    /// that names no stack.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let letter = press.text.filter(char::is_ascii_lowercase);
        if self.list == ObjectList::Inventory {
            if let Some(Item::Pack(i)) = self.lettered(press) {
                self.looked = Some(i);
                return ScreenAction::Open(super::ScreenName::Describe);
            }
            let own_key = press
                .binding
                .as_ref()
                .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Inventory));
            match press.nav {
                _ if own_key => return ScreenAction::Close,
                Nav::Back | Nav::Enter => return ScreenAction::Close,
                Nav::NextPage | Nav::PrevPage => {
                    self.entries.navigate(press.nav);
                }
                _ => {}
            }
            return ScreenAction::Stay;
        }
        if press.nav == Nav::Back || self.entries.is_empty() && press.nav == Nav::Enter {
            return ScreenAction::Close;
        }
        if self.list.picks_one() {
            // One choice: its letter, or Enter on the `>` row.
            if let Some(item) = self.lettered(press) {
                return self.pick(item);
            }
            match press.nav {
                _ if letter.is_some() => {}
                Nav::Enter => {
                    if let Some(item) = self.entries.selected().map(|e| e.item) {
                        return self.pick(item);
                    }
                }
                nav => {
                    self.entries.navigate(nav);
                }
            }
            return ScreenAction::Stay;
        }
        // Letters name stacks, before any key bound to marking.
        if letter.is_some() {
            if let Some(index) = press.text.and_then(|c| self.entries.at_letter(c)) {
                self.entries.select(index);
                self.toggle(index);
            }
            return ScreenAction::Stay;
        }
        match press.nav {
            Nav::MarkAll => {
                let all = Positions::of(self.entries.iter().map(|e| e.item.position()));
                self.marked = if self.marked == all {
                    Positions::default()
                } else {
                    all
                };
            }
            Nav::Mark => self.toggle(self.entries.cursor()),
            Nav::Enter => {
                let chosen = match self.entries.selected() {
                    Some(entry) if self.marked.is_empty() => Positions::of([entry.item.position()]),
                    _ => self.marked.clone(),
                };
                return ScreenAction::Act(match self.list {
                    ObjectList::PickUp => Action::PickUp(chosen),
                    ObjectList::Drop
                    | ObjectList::Inventory
                    | ObjectList::Throw
                    | ObjectList::Eat
                    | ObjectList::Read
                    | ObjectList::Study
                    | ObjectList::Identify(_) => Action::Drop(chosen),
                });
            }
            nav => {
                self.entries.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// The inventory's summary in lines of at most `width` characters, each with its color: what
    /// the player carries (and their burden, if any) and weighs, then the weights at which the
    /// burden changes, dimmer: `Carrying 28.4 kg (burdened)   ·   Your weight 98.4 kg (you 70
    /// kg)`, `Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg`.
    fn summary(&self, width: i32) -> Vec<(String, Rgb)> {
        let burden = (self.load.map(|l| l.burden()))
            .filter(|b| *b != Burden::Unburdened)
            .map_or(String::new(), |b| format!(" ({})", b.id()));
        let parts = [
            format!("Carrying {}{burden}", format_weight(self.weight.carried)),
            format!(
                "Your weight {} (you {})",
                format_weight(self.weight.total()),
                format_weight(self.weight.base),
            ),
        ];
        let carried = fit_joined(&parts, "   ·   ", width).into_iter();
        let limits = (self.load.as_ref()).map_or(Vec::new(), |l| thresholds(l, width));
        (carried.map(|l| (l, TEXT)))
            .chain(limits.into_iter().map(|l| (l, TEXT_DIM)))
            .collect()
    }

    /// Marks or unmarks the stack of entry `index`.
    fn toggle(&mut self, index: usize) {
        let Some(position) = self.entries.get(index).map(|e| e.item.position()) else {
            return;
        };
        if self.marked.contains(position) {
            self.marked.remove(position);
        } else {
            self.marked.insert(position);
        }
    }

    /// Lays the screen out for a console of `cols` by `rows`: sets how many rows a page of the
    /// list takes, and returns the summary (inventory), the rows besides the list, and whether
    /// the margin and the gaps are there (1) or gave way (0). Drawing calls it, and so does the
    /// screen stack before a press (`fit`), so a letter names what is drawn.
    fn layout(&self, cols: u32, rows: u32) -> (Vec<(String, Rgb)>, i32, i32) {
        let read_only = self.list == ObjectList::Inventory;
        // Rows: the frame, a margin, the list, a gap, the summary and the burden's thresholds
        // (inventory), a gap, the footer, the frame.
        let width = self.list.width().min(cols as i32) - 4;
        let summary = if read_only {
            self.summary(width)
        } else {
            Vec::new()
        };
        let gaps = if read_only { 3 } else { 2 };
        let extra = 3 + gaps + summary.len() as i32;
        // A page takes what rows the console has for it (spec §6 "Item lists"); where that is
        // what pages the list (not its 26 letters), the margin and the gaps give way first.
        let room = |extra: i32| (rows as i32 - extra).max(1) as usize;
        self.entries.set_room(usize::MAX);
        let least = self.entries.page_count();
        self.entries.set_room(room(extra));
        if self.entries.page_count() > least {
            self.entries.set_room(room(extra - gaps));
            (summary, extra - gaps, 0)
        } else {
            (summary, extra, 1)
        }
    }

    /// Lays the list out for a console of `cols` by `rows` before a press (see `layout`).
    pub fn fit(&self, cols: u32, rows: u32) {
        self.layout(cols, rows);
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let read_only = self.list == ObjectList::Inventory;
        let (summary, extra, gap) = self.layout(console.cols(), console.rows());
        let count = self.entries.tallest().max(1) as i32;
        let panel = Panel::new(console, self.list.title(), self.list.width(), count + extra);
        if self.list == ObjectList::Inventory {
            super::info_tabs::draw_tabs(console, &panel, bindings, super::ScreenName::Inventory);
        }
        self.entries.draw_label(console, &panel);
        let visible = (panel.h - extra).max(1) as usize;
        let (x, top) = (panel.inner_x(), panel.y + 1 + gap);
        if self.entries.is_empty() {
            console.print(x + 2, top, self.list.empty(), TEXT_DIM);
        }
        let cursor = self.entries.cursor();
        for (row, y) in self.entries.rows().into_iter().take(visible).zip(top..) {
            let (index, letter) = match row {
                Row::Header(name) => {
                    draw_header(console, x, y, name);
                    continue;
                }
                Row::Item { index, letter } => (index, letter),
            };
            let Some(entry) = self.entries.get(index) else {
                continue;
            };
            let selected = !read_only && index == cursor;
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `a - ) a dagger`: the page's letter, the mark, the glyph in the object's color,
            // the name.
            let mark = if self.marked.contains(entry.item.position()) {
                '+'
            } else {
                '-'
            };
            let label = format!("{letter} {mark}");
            console.print(x + 2, y, &label, color);
            let glyph_x = x + 3 + label.chars().count() as i32;
            console.print(glyph_x, y, &entry.glyph.to_string(), entry.fg);
            let name_x = glyph_x + 2;
            let weight = format_weight(entry.weight);
            let weight_x = panel.x + panel.w - 3 - weight.chars().count() as i32;
            let room = (weight_x - name_x - 1).max(0) as usize;
            let mark = (entry.mark.as_ref()).map_or(String::new(), |m| format!(" ({m})"));
            let name: String = format!("{}{mark}", entry.name).chars().take(room).collect();
            console.print(name_x, y, &name, rarity_color(entry.rarity, color));
            console.print(weight_x, y, &weight, TEXT_DIM);
        }
        let below = panel.y + panel.h - 2 - gap - summary.len() as i32;
        for (i, (line, color)) in summary.iter().enumerate() {
            let y = below + i as i32;
            // The frame of a panel cut short keeps its footer.
            if y >= top + visible as i32 && y < panel.y + panel.h - 2 {
                console.print(x, y, line, *color);
            }
        }
        panel.footer(console, &self.footer(bindings));
    }

    /// The footer: the letters of the page shown, what they do, the page keys while there are
    /// pages, and how to leave, keys as bound.
    fn footer(&self, bindings: &Bindings) -> String {
        let keys = Hints::of(bindings);
        // `a-c` for the page's letters (`a` for one); none for an empty list.
        let letters = (self.entries.last_letter()).map(|last| match last {
            'a' => "a".to_string(),
            last => format!("a-{last}"),
        });
        let with = |what: &str| {
            letters
                .as_ref()
                .map_or(String::new(), |l| format!("{l} {what}   "))
        };
        let page = (self.entries.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        let confirm = &keys.confirm;
        let either = |verb: &str| match &letters {
            Some(l) => format!("{l} or {confirm} {verb}"),
            None => format!("{confirm} {verb}"),
        };
        match self.list {
            ObjectList::Inventory => {
                let close = match bindings.keys_of(KeyCommand::Inventory).first() {
                    Some(key) => format!("{} or Esc close", key.display()),
                    None => "Esc close".to_string(),
                };
                format!("{page}{}{close}", with("look"))
            }
            ObjectList::Eat => format!("{}   {page}Esc cancel", either("eat")),
            ObjectList::Throw => format!("{}   {page}Esc cancel", either("choose")),
            ObjectList::Read => format!("{}   {page}Esc cancel", either("read")),
            ObjectList::Study => format!("{}   {page}Esc cancel", either("study")),
            ObjectList::Identify(_) => format!("{}   {page}Esc cancel", either("identify")),
            ObjectList::PickUp | ObjectList::Drop => {
                let verb = if self.list == ObjectList::PickUp {
                    "pick up"
                } else {
                    "drop"
                };
                format!(
                    "{}{} all   {confirm} {verb}   {page}Esc cancel",
                    with("mark"),
                    keys.mark_all
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use bevy::input::keyboard::KeyCode;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Dir, World};

    /// The showcase, standing on the start room's pile (a jerkin, a dagger, two candles).
    fn on_the_pile() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        world.apply(&Action::Move(Dir::SW));
        world.apply(&Action::Move(Dir::SW));
        world
    }

    fn typed(ch: char) -> Press {
        Press::typed(ch)
    }

    fn line(console: &Console, y: i32) -> String {
        (0..console.cols() as i32)
            .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
            .collect()
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| line(console, y))
            .collect()
    }

    /// A larder: food underfoot (two rations on top of a dagger) and an apple and a dagger in
    /// the pack.
    fn in_the_larder() -> World {
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "levels/larder.toml",
            "name = \"Larder\"\nmap = '''\n#####\n#@..#\n#####\n'''\n[legend]\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\", \
             objects = [\"ration*2\", \"dagger\", \"apple\"] }\n",
        );
        files.insert(
            "scenarios/larder.toml",
            "name = \"Larder\"\nstart_level = \"larder\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let mut world = World::new(content, "larder", None).expect("world");
        // Take the dagger and the apple (b, c), leaving the rations.
        world.apply(&Action::PickUp(Positions::of([1, 2])));
        world
    }

    #[test]
    fn the_eat_choice_lists_food_here_first_then_in_the_pack() {
        let world = in_the_larder();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Eat, Some(&world.perception()));
        let names: Vec<&str> = screen.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["2 food rations", "an apple"], "no dagger");
        let mut console = Console::default();
        console.resize(80, 24);
        screen.draw(&mut console, &b);
        let shown = text(&console);
        // Under the class's header, what lies here first, then the pack's food, lettered down
        // the page.
        let at = |needle: &str| shown.iter().position(|l| l.contains(needle));
        assert!(at("│ Food").is_some(), "{shown:#?}");
        assert_eq!(
            at("> a - % 2 food rations (lying here)"),
            at("│ Food").map(|y| y + 1)
        );
        assert!(at("b - % an apple").is_some());
        assert!(at("a-b or Enter eat   Esc cancel").is_some());
        // Enter eats what lies here (the cursor starts on it); a pack letter eats from the pack.
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Eat(Meal::Floor(0)))
        );
        assert_eq!(
            screen.handle(&typed('a'), &b),
            ScreenAction::Act(Action::Eat(Meal::Floor(0)))
        );
        assert_eq!(
            screen.handle(&typed('b'), &b),
            ScreenAction::Act(Action::Eat(Meal::Pack(1)))
        );
        assert_eq!(screen.handle(&typed('c'), &b), ScreenAction::Stay);
        screen.handle(&Press::nav(Nav::Down), &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Eat(Meal::Pack(1)))
        );
        // Nothing to eat: it says so, and Enter or Esc closes it.
        let empty = on_the_pile();
        let mut screen = ObjectScreen::new(ObjectList::Eat, Some(&empty.perception()));
        screen.draw(&mut console, &b);
        assert!(
            text(&console)
                .iter()
                .any(|l| l.contains("You have nothing to eat."))
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
    }

    /// The trap hall's scrolls and unknown dagger picked up: a dagger (magic, unidentified), b
    /// fireball, c lightning, d frost, e identify (the scrolls all unknown, under labels).
    fn with_scrolls() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "traps", None).expect("world");
        world.apply(&Action::Move(Dir::E));
        world.apply(&Action::PickUp(Positions::default()));
        world
    }

    #[test]
    fn the_read_choice_aims_spells_and_asks_what_to_identify() {
        let world = with_scrolls();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Read, Some(&world.perception()));
        // Unknown scrolls by their labels (spec §8 "Scrolls").
        let lore = world.perception().lore();
        let pack = world.perception().inventory();
        let labelled: Vec<String> = pack[1..].iter().map(|s| lore.describe(s)).collect();
        let names: Vec<&str> = screen.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, labelled, "no dagger");
        assert!(labelled.iter().all(|n| n.contains("scrolls labelled")));
        let mut console = Console::default();
        console.resize(80, 24);
        screen.draw(&mut console, &b);
        let shown = text(&console);
        assert!(shown.iter().any(|l| l.contains("│ Scrolls")), "{shown:#?}");
        assert!(
            shown
                .iter()
                .any(|l| l.contains(&format!("a - ? {}", labelled[0])))
        );
        assert!(
            shown
                .iter()
                .any(|l| l.contains("a-d or Enter read   Esc cancel"))
        );
        // A spell scroll is aimed; the scroll of identify asks what to read it on, in its place.
        assert_eq!(
            screen.handle(&typed('b'), &b),
            ScreenAction::Aim(AimKind::Read(2))
        );
        assert_eq!(
            screen.handle(&typed('d'), &b),
            ScreenAction::Replace(super::super::ScreenName::Identify)
        );
        assert_eq!(screen.looked(), Some(4));
        // What it is read on: the unidentified stacks only, the labelled scrolls among them, not
        // the scroll read.
        let mut screen = ObjectScreen::new(ObjectList::Identify(4), Some(&world.perception()));
        let names: Vec<&str> = screen.entries.iter().map(|e| e.name.as_str()).collect();
        let mut unknown = vec!["a dagger (magic)"];
        unknown.extend(labelled[..3].iter().map(String::as_str));
        assert_eq!(names, unknown);
        assert_eq!(
            screen.handle(&typed('a'), &b),
            ScreenAction::Act(Action::Read {
                item: 4,
                target: Some(ReadTarget::Item(0))
            })
        );
        // Nothing to read: it says so.
        let empty = on_the_pile();
        let screen = ObjectScreen::new(ObjectList::Read, Some(&empty.perception()));
        screen.draw(&mut console, &b);
        assert!(
            text(&console)
                .iter()
                .any(|l| l.contains("You have nothing to read."))
        );
    }

    #[test]
    fn the_throw_choice_lists_throwing_weapons_lettered_by_page() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Positions::default()));
        // a leather jerkin, b dagger, c 2 wax candles: only the dagger is thrown.
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Throw, Some(&world.perception()));
        assert_eq!(screen.entries.len(), 1);
        assert_eq!(
            screen.entries.get(0).map(|e| e.name.as_str()),
            Some("a dagger")
        );
        let mut console = Console::default();
        console.resize(80, 24);
        screen.draw(&mut console, &b);
        assert!(text(&console).iter().any(|l| l.contains("a - ) a dagger")));
        assert_eq!(screen.handle(&typed('b'), &b), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&typed('a'), &b),
            ScreenAction::Aim(AimKind::Throw(1))
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Aim(AimKind::Throw(1))
        );
        // Nothing to throw: it says so, and Enter or Esc closes it.
        let empty = on_the_pile();
        let mut screen = ObjectScreen::new(ObjectList::Throw, Some(&empty.perception()));
        screen.draw(&mut console, &b);
        assert!(
            text(&console)
                .iter()
                .any(|l| l.contains("You have nothing to throw."))
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn letters_mark_stacks_and_enter_picks_them_up() {
        let world = on_the_pile();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::PickUp, Some(&world.perception()));
        // By class: the dagger (melee weapons), the jerkin (body armour), the candles (light
        // sources), lettered so; the pile's positions are jerkin 0, dagger 1, candles 2.
        let names: Vec<&str> = screen.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a dagger", "a leather jerkin", "2 wax candles"]);
        assert_eq!(screen.handle(&typed('c'), &b), ScreenAction::Stay);
        assert_eq!(screen.handle(&typed('a'), &b), ScreenAction::Stay);
        // `z` names nothing here; `j` is a letter, not a cursor move.
        screen.handle(&typed('z'), &b);
        screen.handle(&typed('j'), &b);
        assert_eq!(screen.entries.cursor(), 0);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(Positions::of([1, 2])))
        );
        // Unmarking everything, Enter takes the cursor's row.
        screen.handle(&typed('a'), &b);
        screen.handle(&typed('c'), &b);
        screen.handle(&Press::nav(Nav::Down), &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(Positions::of([1])))
        );
        screen.handle(&Press::nav(Nav::Down), &b);
        // Space (the Mark key), as the keyboard reports it: a named key that types nothing.
        let space = Press {
            nav: Nav::Mark,
            edit: None,
            binding: Some(KeyBinding::Code(KeyCode::Space)),
            repeat: false,
            text: None,
            shift: false,
        };
        screen.handle(&space, &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(Positions::of([0])))
        );
    }

    #[test]
    fn comma_marks_everything_and_esc_cancels() {
        let world = on_the_pile();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::PickUp, Some(&world.perception()));
        // `,` (the Mark everything key) as the router reads it.
        let comma = Press {
            nav: Nav::MarkAll,
            ..typed(',')
        };
        screen.handle(&comma, &b);
        assert_eq!(screen.marked, Positions::of(0..3));
        screen.handle(&comma, &b);
        assert!(screen.marked.is_empty());
        screen.handle(&comma, &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn drop_chooses_from_the_inventory() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Positions::default()));
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Drop, Some(&world.perception()));
        assert_eq!(screen.entries.iter().count(), 3);
        // `b`, the second row: the jerkin, inventory position 0.
        screen.handle(&typed('b'), &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Drop(Positions::of([0])))
        );
    }

    #[test]
    fn the_inventory_shows_weights_and_closes_on_its_key() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Positions::default()));
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Inventory, Some(&world.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, &b);
        let text = text(&console);
        let row = |needle: &str| {
            text.iter()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
        };
        assert!(row("b - [ a leather jerkin").contains("3.5 kg"));
        assert!(row("c - ( 2 wax candles").contains("200 g"));
        row("│ Body armour");
        assert!(row("Carrying").contains("Carrying 4.2 kg"));
        assert!(row("Carrying").contains("Your weight 74.2 kg (you 70 kg)"));
        assert!(row("Esc close").contains("a-c look   I or Esc close"));
        // A stack's letter opens its description; `i` (naming no stack here) closes, like Esc.
        assert_eq!(
            screen.handle(&typed('b'), &b),
            ScreenAction::Open(crate::ui::ScreenName::Describe)
        );
        assert_eq!(screen.looked(), Some(0), "the jerkin");
        assert_eq!(screen.handle(&typed('e'), &b), ScreenAction::Stay);
        let i = Press {
            binding: crate::input::KeyBinding::from_text("I"),
            ..typed('I')
        };
        assert_eq!(screen.handle(&i, &b), ScreenAction::Close);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn the_inventory_shows_the_burden_and_where_it_changes() {
        let b = Bindings::default();
        let draw = |world: &World, cols: u32| {
            let screen = ObjectScreen::new(ObjectList::Inventory, Some(&world.perception()));
            let mut console = Console::default();
            console.resize(cols, 30);
            screen.draw(&mut console, &b);
            text(&console)
        };
        // The showcase's thief, St 9 To 10: a limit of 42 kg, by thirds.
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Positions::default()));
        let text = draw(&world, 80);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        let thresholds = "Burdened over 14 kg, strained over 28 kg, overloaded over 42 kg";
        assert_eq!(at(thresholds), at("Carrying 4.2 kg   ·").map(|y| y + 1));
        assert!(!text.iter().any(|l| l.contains("(burdened)")), "unburdened");
        // Two chain mails for average scores: burdened over 15 kg.
        let mut world = crate::ui::hoard();
        world.apply(&Action::PickUp(Positions::default()));
        let text = draw(&world, 80);
        for expected in [
            "Carrying 22 kg (burdened)   ·   Your weight 92 kg (you 70 kg)",
            "Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg",
        ] {
            assert!(
                text.iter().any(|l| l.contains(expected)),
                "{expected:?} in {text:#?}"
            );
        }
        // Narrow: the lines wrap, and the footer stays.
        let text = draw(&world, 40);
        for expected in [
            "Carrying 22 kg (burdened)",
            "Your weight 92 kg (you 70 kg)",
            "│ Burdened over 15 kg,  ",
            "│ strained over 30 kg,  ",
            "│ overloaded over 45 kg  ",
            "I or Esc close",
        ] {
            assert!(
                text.iter().any(|l| l.contains(expected)),
                "{expected:?} in {text:#?}"
            );
        }
    }

    #[test]
    fn the_inventory_and_drop_screens_mark_what_is_worn_or_wielded() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Positions::default()));
        for (slot, item) in [("body", 0), ("main_hand", 1)] {
            world.apply(&Action::Equip {
                slot: slot.into(),
                item,
            });
        }
        let b = Bindings::default();
        for list in [ObjectList::Inventory, ObjectList::Drop] {
            let screen = ObjectScreen::new(list, Some(&world.perception()));
            let mut console = Console::default();
            console.resize(80, 30);
            screen.draw(&mut console, &b);
            let text = text(&console);
            let row = |needle: &str| {
                text.iter()
                    .find(|l| l.contains(needle))
                    .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
            };
            assert!(row("a leather jerkin (being worn)").contains("3.5 kg"));
            row("a dagger (wielded)");
            assert!(!row("2 wax candles").contains("candles ("), "not worn");
        }
        // Worn things still weigh what they weigh.
        let screen = ObjectScreen::new(ObjectList::Inventory, Some(&world.perception()));
        assert_eq!(screen.weight.carried, 3500 + 450 + 2 * 100);
    }

    #[test]
    fn marks_and_the_cursor_show_without_color() {
        let world = on_the_pile();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::PickUp, Some(&world.perception()));
        screen.handle(&typed('b'), &b);
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, &b);
        let text = text(&console);
        // Marking `b` moved the cursor there.
        assert!(text.iter().any(|l| l.contains("│   a - ) a dagger")));
        assert!(
            text.iter()
                .any(|l| l.contains("│ > b + [ a leather jerkin"))
        );
        assert!(text.iter().any(|l| l.contains("Enter pick up")));
    }

    /// Standing on a heap of 40 stacks: 20 daggers, 12 leather caps, and 8 wool cloaks.
    fn on_the_heap() -> World {
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "levels/heap.toml",
            "name = \"Heap\"\nmap = '''\n#####\n#@..#\n#####\n'''\n[legend]\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\", \
             objects = [\"wool_cloak*8\", \"leather_cap*12\", \"dagger*20\"] }\n",
        );
        files.insert(
            "scenarios/heap.toml",
            "name = \"Heap\"\nstart_level = \"heap\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        World::new(content, "heap", None).expect("world")
    }

    fn drawn(screen: &ObjectScreen, b: &Bindings) -> Vec<String> {
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, b);
        text(&console)
    }

    fn has(text: &[String], needle: &str) -> bool {
        text.iter().any(|l| l.contains(needle))
    }

    #[test]
    fn a_long_list_pages_and_letters_restart_on_every_page() {
        let mut world = on_the_heap();
        world.apply(&Action::PickUp(Positions::default()));
        assert_eq!(world.perception().inventory().len(), 40);
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Inventory, Some(&world.perception()));
        // 30 rows: room for the 20 daggers under their header on the first page.
        let text = drawn(&screen, &b);
        assert!(has(&text, " Page 1/2 "), "{text:#?}");
        let at = |text: &[String], needle: &str| text.iter().position(|l| l.contains(needle));
        assert_eq!(
            at(&text, "│ Melee weapons"),
            at(&text, "a - ) a dagger").map(|y| y - 1)
        );
        assert!(has(&text, "t - ) a dagger") && !has(&text, "u - "));
        assert!(!has(&text, "leather cap"));
        assert!(
            has(&text, "> < page   a-t look   I or Esc close"),
            "{text:#?}"
        );
        // `>` (the next list page) turns it: the caps and the cloaks, each under its header (no
        // letter), lettered from `a` again.
        let next = Press::nav(Nav::NextPage);
        assert_eq!(screen.handle(&next, &b), ScreenAction::Stay);
        let text = drawn(&screen, &b);
        assert!(has(&text, " Page 2/2 "));
        assert_eq!(
            at(&text, "│ Helmets"),
            at(&text, "a - [ a leather cap").map(|y| y - 1)
        );
        assert_eq!(
            at(&text, "│ Cloaks"),
            at(&text, "m - [ a wool cloak").map(|y| y - 1)
        );
        assert!(has(&text, "t - [ a wool cloak"));
        assert!(!has(&text, "dagger"));
        // On this page `a` is the first cap: its description opens.
        let cap = screen.entries.get(20).map(|e| e.item);
        assert_eq!(
            screen.handle(&typed('a'), &b),
            ScreenAction::Open(crate::ui::ScreenName::Describe)
        );
        assert_eq!(screen.looked().map(Item::Pack), cap);
        // `<` back to the first page, where `a` is the first dagger.
        screen.handle(&Press::nav(Nav::PrevPage), &b);
        screen.handle(&typed('a'), &b);
        assert_eq!(
            screen.looked().map(Item::Pack),
            screen.entries.get(0).map(|e| e.item)
        );
    }

    #[test]
    fn marks_stay_with_their_stacks_across_pages() {
        let world = on_the_heap();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::PickUp, Some(&world.perception()));
        let text = drawn(&screen, &b);
        assert!(has(&text, " Page 1/2 "), "{text:#?}");
        // The daggers and three caps fit the first page (25 rows); `a` marks the first dagger.
        assert!(has(&text, "w - [ a leather cap") && !has(&text, "x - "));
        assert!(has(
            &text,
            "a-w mark   , all   Enter pick up   > < page   Esc cancel"
        ));
        screen.handle(&typed('a'), &b);
        let dagger = screen.entries.get(0).map(|e| e.item.position());
        screen.handle(&Press::nav(Nav::NextPage), &b);
        // The next page repeats the caps' header; its `a` is the fourth cap, which `a` marks.
        let text = drawn(&screen, &b);
        assert!(has(&text, "│ > a - [ a leather cap"), "{text:#?}");
        assert!(has(&text, "│ Helmets") && has(&text, "│ Cloaks"));
        screen.handle(&typed('a'), &b);
        let cap = screen.entries.get(23).map(|e| e.item.position());
        let text = drawn(&screen, &b);
        assert!(has(&text, "│ > a + [ a leather cap"));
        // Back on the first page the dagger is still marked; Enter takes both.
        screen.handle(&Press::nav(Nav::PrevPage), &b);
        assert!(has(&drawn(&screen, &b), "│ > a + ) a dagger"));
        let both = Positions::of(dagger.into_iter().chain(cap));
        assert_eq!(both.len(), 2);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(both))
        );
        // Moving down past the page's end turns the page too.
        for _ in 0..23 {
            screen.handle(&Press::nav(Nav::Down), &b);
        }
        assert!(has(&drawn(&screen, &b), " Page 2/2 "));
    }

    #[test]
    fn a_short_console_drops_the_gaps_before_it_pages() {
        // The storeroom's nine things, each of its own class: 18 rows with their headers. At 24
        // rows the margin and the gaps give way so they fit on one page.
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        let walk = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
                    pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let b = Bindings::default();
        let screen = ObjectScreen::new(ObjectList::Inventory, Some(&world.perception()));
        let mut console = Console::default();
        console.resize(87, 24);
        screen.draw(&mut console, &b);
        let text = text(&console);
        assert!(!has(&text, "Page "), "{text:#?}");
        assert!(has(&text, "i - [ a pair of leather boots"));
        assert!(has(&text, "Carrying 20.9 kg (burdened)"));
        assert!(has(&text, "a-i look   I or Esc close"));
    }

    #[test]
    fn empty_lists_and_tiny_consoles() {
        let b = Bindings::default();
        for list in [ObjectList::Inventory, ObjectList::PickUp, ObjectList::Drop] {
            let mut screen = ObjectScreen::new(list, None);
            for (cols, rows) in [(80, 30), (20, 6), (0, 0)] {
                let mut console = Console::default();
                console.resize(cols, rows);
                screen.draw(&mut console, &b);
            }
            assert_eq!(
                screen.handle(&Press::nav(Nav::Enter), &b),
                ScreenAction::Close
            );
        }
        let world = on_the_pile();
        let mut screen = ObjectScreen::new(ObjectList::PickUp, Some(&world.perception()));
        for cursor in [0, 2] {
            screen.entries.select(cursor);
            for (cols, rows) in [(30, 7), (12, 4)] {
                let mut console = Console::default();
                console.resize(cols, rows);
                screen.draw(&mut console, &b);
            }
        }
    }
}
