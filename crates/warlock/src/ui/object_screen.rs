//! Object screens (spec §6): the inventory (`i`), and the selection screens for picking up from a
//! pile (`,` or `g` where several objects lie), for dropping (`d`), and for choosing what to
//! throw (`t`: the throwing weapons the player carries, by their inventory letters; choosing one
//! aims it, `ui::target`).
//!
//! Each lists stacks by letter, `a` first, with their glyph, name, and weight; the inventory and
//! the drop screen mark what is worn or wielded ("(being worn)", "(wielded)"; dropping it takes
//! it off first). The inventory is read-only and also shows what the player carries and weighs,
//! their burden when they have one, and the weights at which it changes (spec §8 "Burden"). In a
//! selection screen a letter marks or unmarks its stack (`+` marked, `-` not: never color
//! alone), `,` marks everything (or, with everything marked, nothing), the arrows move the `>`
//! cursor and Space marks its row, and Enter picks up or drops the marked stacks (the cursor's
//! row if none is marked); that is one action, recorded like any other. Esc cancels. The list is read from the perception snapshot
//! when the screen opens; no game time passes while it is open, so it cannot go stale.

use super::character_screen::{fit_joined, thresholds};
use super::menu::{Nav, Panel, Press};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyBinding, KeyCommand};
use crate::render::console::Console;
use bevy::input::keyboard::KeyCode;
use warlock_core::action::letter;
use warlock_core::burden::{Burden, Load};
use warlock_core::color::Rgb;
use warlock_core::equipment;
use warlock_core::object::{Stack, Weight, format_weight};
use warlock_core::perception::PerceptionView;
use warlock_core::{Action, Letters};

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
}

impl ObjectList {
    fn title(self) -> &'static str {
        match self {
            ObjectList::Inventory => " Inventory ",
            ObjectList::PickUp => " Pick up ",
            ObjectList::Drop => " Drop ",
            ObjectList::Throw => " Throw ",
        }
    }

    fn empty(self) -> &'static str {
        match self {
            ObjectList::Inventory | ObjectList::Drop => "You are not carrying anything.",
            ObjectList::PickUp => "There is nothing here.",
            ObjectList::Throw => "You have nothing to throw.",
        }
    }

    /// The panel's width: the inventory's is wider, for what the player carries, weighs, and
    /// may carry on one line each.
    fn width(self) -> i32 {
        match self {
            ObjectList::Inventory => 72,
            ObjectList::PickUp | ObjectList::Drop | ObjectList::Throw => 60,
        }
    }
}

/// One stack as the screen shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    glyph: char,
    fg: Rgb,
    name: String,
    /// What the inventory says of it if it is worn or wielded: "being worn".
    mark: Option<String>,
    /// Grams.
    weight: u64,
}

#[derive(Debug)]
pub struct ObjectScreen {
    list: ObjectList,
    /// At most one per letter.
    entries: Vec<Entry>,
    /// The position (letter) of each entry in its list: in the inventory for what to throw,
    /// which lists only some of it.
    items: Vec<usize>,
    /// The `>` row of a selection screen; the first row shown in the inventory.
    cursor: usize,
    marked: Letters,
    /// The player's weight (shown in the inventory).
    weight: Weight,
    /// What the player carries against what they can (shown in the inventory); `None` outside
    /// a run.
    load: Option<Load>,
}

impl ObjectScreen {
    /// The screen for `list`, read from what the player perceives (`None` outside a run: an
    /// empty list).
    pub fn new(list: ObjectList, view: Option<&PerceptionView<'_>>) -> Self {
        let (stacks, weight): (&[Stack], Weight) = match view {
            Some(view) => {
                let stacks = match list {
                    ObjectList::Inventory | ObjectList::Drop | ObjectList::Throw => {
                        view.inventory()
                    }
                    ObjectList::PickUp => view.objects(view.viewer()),
                };
                (stacks, view.own_weight())
            }
            None => (&[], Weight::default()),
        };
        let shown = |i: usize| match (list, view) {
            (ObjectList::Throw, Some(view)) => view.throw_range(i).is_some(),
            _ => true,
        };
        let items: Vec<usize> = (0..stacks.len().min(Letters::COUNT))
            .filter(|&i| shown(i))
            .collect();
        let entries = match view {
            Some(view) => {
                let content = view.content();
                // The inventory marks what is worn or wielded; a pile has nothing worn.
                let carried = list != ObjectList::PickUp;
                (items.iter())
                    .map(|&i| (i, &stacks[i]))
                    .map(|(i, s)| {
                        let def = content.object(s.object);
                        Entry {
                            glyph: def.glyph,
                            fg: def.fg,
                            name: s.describe(content),
                            mark: carried
                                .then(|| equipment::mark(content, view.equipment(), i))
                                .flatten(),
                            weight: s.weight(content),
                        }
                    })
                    .collect()
            }
            None => Vec::new(),
        };
        Self {
            list,
            entries,
            items,
            cursor: 0,
            marked: Letters::default(),
            weight,
            load: view.map(PerceptionView::own_load),
        }
    }

    pub fn list(&self) -> ObjectList {
        self.list
    }

    /// Applies a key press. The inventory closes on Esc, Enter, or its own key.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        if self.list == ObjectList::Inventory {
            let own_key = press
                .binding
                .as_ref()
                .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Inventory));
            match press.nav {
                _ if own_key => return ScreenAction::Close,
                Nav::Back | Nav::Enter => return ScreenAction::Close,
                Nav::Up => self.cursor = self.cursor.saturating_sub(1),
                Nav::Down => {
                    self.cursor = (self.cursor + 1).min(self.entries.len().saturating_sub(1));
                }
                _ => {}
            }
            return ScreenAction::Stay;
        }
        if press.nav == Nav::Back || self.entries.is_empty() && press.nav == Nav::Enter {
            return ScreenAction::Close;
        }
        if self.list == ObjectList::Throw {
            // One choice: its letter, or Enter on the `>` row.
            let rows = self.entries.len().max(1);
            match (press.text, press.nav) {
                (Some(c @ 'a'..='z'), _) => {
                    let i = usize::from(c as u8 - b'a');
                    if self.items.contains(&i) {
                        return ScreenAction::Aim(i);
                    }
                }
                (_, Nav::Up) => self.cursor = (self.cursor + rows - 1) % rows,
                (_, Nav::Down) => self.cursor = (self.cursor + 1) % rows,
                (_, Nav::Enter) => return ScreenAction::Aim(self.items[self.cursor]),
                _ => {}
            }
            return ScreenAction::Stay;
        }
        // Space is a named key, not a typed character.
        let space = press.binding == Some(KeyBinding::Code(KeyCode::Space));
        // Letters name stacks (even `j` and `k`, which move selections elsewhere).
        match press.text.or(space.then_some(' ')) {
            Some(c @ 'a'..='z') => {
                let i = usize::from(c as u8 - b'a');
                if i < self.entries.len() {
                    self.toggle(i);
                    self.cursor = i;
                }
                return ScreenAction::Stay;
            }
            Some(',') => {
                let all = Letters::of(0..self.entries.len());
                self.marked = if self.marked == all {
                    Letters::default()
                } else {
                    all
                };
                return ScreenAction::Stay;
            }
            Some(' ') => {
                self.toggle(self.cursor);
                return ScreenAction::Stay;
            }
            _ => {}
        }
        let rows = self.entries.len().max(1);
        match press.nav {
            Nav::Up => self.cursor = (self.cursor + rows - 1) % rows,
            Nav::Down => self.cursor = (self.cursor + 1) % rows,
            Nav::Enter => {
                let chosen = if self.marked.is_empty() {
                    Letters::of([self.cursor])
                } else {
                    self.marked
                };
                return ScreenAction::Act(match self.list {
                    ObjectList::PickUp => Action::PickUp(chosen),
                    ObjectList::Drop | ObjectList::Inventory | ObjectList::Throw => {
                        Action::Drop(chosen)
                    }
                });
            }
            _ => {}
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

    fn toggle(&mut self, i: usize) {
        if self.marked.contains(i) {
            self.marked.remove(i);
        } else {
            self.marked.insert(i);
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let read_only = self.list == ObjectList::Inventory;
        let count = self.entries.len().max(1);
        // Rows: margin, the list, a gap, the summary and the burden's thresholds (inventory), a
        // gap, the footer, the frame.
        let width = self.list.width().min(console.cols() as i32) - 4;
        let summary = if read_only {
            self.summary(width)
        } else {
            Vec::new()
        };
        let extra = if read_only {
            6 + summary.len() as i32
        } else {
            5
        };
        let panel = Panel::new(
            console,
            self.list.title(),
            self.list.width(),
            count as i32 + extra,
        );
        let visible = (panel.body_rows() - (extra - 4)).max(1) as usize;
        let first = if read_only {
            self.cursor
        } else {
            self.cursor.saturating_sub(visible - 1)
        }
        .min(count.saturating_sub(visible));
        let x = panel.inner_x();
        if self.entries.is_empty() {
            console.print(x + 2, panel.y + 2, self.list.empty(), TEXT_DIM);
        }
        for (i, entry) in self.entries.iter().enumerate().skip(first).take(visible) {
            let y = panel.y + 2 + (i - first) as i32;
            let selected = !read_only && i == self.cursor;
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `a - ) a dagger`: letter, mark, the glyph in the object's color, the name.
            let mark = if self.marked.contains(i) { '+' } else { '-' };
            let label = format!("{} {mark}", letter(self.items[i]).unwrap_or('?'));
            console.print(x + 2, y, &label, color);
            let glyph_x = x + 3 + label.chars().count() as i32;
            console.print(glyph_x, y, &entry.glyph.to_string(), entry.fg);
            let name_x = glyph_x + 2;
            let weight = format_weight(entry.weight);
            let weight_x = panel.x + panel.w - 3 - weight.chars().count() as i32;
            let room = (weight_x - name_x - 1).max(0) as usize;
            let mark = (entry.mark.as_ref()).map_or(String::new(), |m| format!(" ({m})"));
            let name: String = format!("{}{mark}", entry.name).chars().take(room).collect();
            console.print(name_x, y, &name, color);
            console.print(weight_x, y, &weight, TEXT_DIM);
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        let more = first + visible < count;
        if more {
            console.print(panel.x + panel.w - 3, panel.y + panel.h - 2, "↓", TEXT_DIM);
        }
        let below = panel.y + panel.h - 3 - summary.len() as i32;
        for (i, (line, color)) in summary.iter().enumerate() {
            let y = below + i as i32;
            // The frame of a panel cut short keeps its footer.
            if y > panel.y + 1 + visible as i32 && y < panel.y + panel.h - 2 {
                console.print(x, y, line, *color);
            }
        }
        let verb = match self.list {
            ObjectList::PickUp => "pick up",
            ObjectList::Drop | ObjectList::Inventory => "drop",
            ObjectList::Throw => "throw",
        };
        let footer = if read_only {
            let scroll = if first > 0 || more {
                "↑↓ scroll   "
            } else {
                ""
            };
            let close = match bindings.keys_of(KeyCommand::Inventory).first() {
                Some(key) => format!("{} or Esc close", key.display()),
                None => "Esc close".to_string(),
            };
            format!("{scroll}{close}")
        } else if self.list == ObjectList::Throw {
            "a-z or Enter choose   Esc cancel".to_string()
        } else {
            format!("a-z mark   , all   Enter {verb}   Esc cancel")
        };
        panel.footer(console, &footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn the_throw_choice_lists_throwing_weapons_by_their_pack_letters() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Letters::default()));
        // a leather jerkin, b dagger, c 2 wax candles: only the dagger is thrown.
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Throw, Some(&world.perception()));
        assert_eq!(screen.entries.len(), 1);
        assert_eq!(screen.entries[0].name, "a dagger");
        let mut console = Console::default();
        console.resize(80, 24);
        screen.draw(&mut console, &b);
        assert!(text(&console).iter().any(|l| l.contains("b - ) a dagger")));
        assert_eq!(screen.handle(&typed('a'), &b), ScreenAction::Stay);
        assert_eq!(screen.handle(&typed('b'), &b), ScreenAction::Aim(1));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Aim(1)
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
        assert_eq!(screen.entries.len(), 3);
        assert_eq!(screen.entries[0].name, "a leather jerkin");
        assert_eq!(screen.handle(&typed('c'), &b), ScreenAction::Stay);
        assert_eq!(screen.handle(&typed('a'), &b), ScreenAction::Stay);
        // `z` names nothing here; `j` is a letter, not a cursor move.
        screen.handle(&typed('z'), &b);
        screen.handle(&typed('j'), &b);
        assert_eq!(screen.cursor, 0);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(Letters::of([0, 2])))
        );
        // Unmarking everything, Enter takes the cursor's row.
        screen.handle(&typed('a'), &b);
        screen.handle(&typed('c'), &b);
        screen.handle(&Press::nav(Nav::Down), &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(Letters::of([0])))
        );
        screen.handle(&Press::nav(Nav::Down), &b);
        // Space, as the keyboard reports it: a named key that types nothing.
        let space = Press {
            nav: Nav::Other,
            binding: Some(KeyBinding::Code(KeyCode::Space)),
            repeat: false,
            text: None,
        };
        screen.handle(&space, &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::PickUp(Letters::of([1])))
        );
    }

    #[test]
    fn comma_marks_everything_and_esc_cancels() {
        let world = on_the_pile();
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::PickUp, Some(&world.perception()));
        screen.handle(&typed(','), &b);
        assert_eq!(screen.marked, Letters::of(0..3));
        screen.handle(&typed(','), &b);
        assert!(screen.marked.is_empty());
        screen.handle(&typed(','), &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn drop_chooses_from_the_inventory() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Letters::default()));
        let b = Bindings::default();
        let mut screen = ObjectScreen::new(ObjectList::Drop, Some(&world.perception()));
        assert_eq!(screen.entries.len(), 3);
        screen.handle(&typed('b'), &b);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Drop(Letters::of([1])))
        );
    }

    #[test]
    fn the_inventory_shows_weights_and_closes_on_its_key() {
        let mut world = on_the_pile();
        world.apply(&Action::PickUp(Letters::default()));
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
        assert!(row("a - [ a leather jerkin").contains("3.5 kg"));
        assert!(row("c - ( 2 wax candles").contains("200 g"));
        assert!(row("Carrying").contains("Carrying 4.2 kg"));
        assert!(row("Carrying").contains("Your weight 74.2 kg (you 70 kg)"));
        assert!(row("Esc close").contains("i or Esc close"));
        // Letters do nothing here; `i` closes, like Esc.
        assert_eq!(screen.handle(&typed('a'), &b), ScreenAction::Stay);
        let i = Press {
            binding: crate::input::KeyBinding::from_text("i"),
            ..typed('i')
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
        world.apply(&Action::PickUp(Letters::default()));
        let text = draw(&world, 80);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        let thresholds = "Burdened over 14 kg, strained over 28 kg, overloaded over 42 kg";
        assert_eq!(at(thresholds), at("Carrying 4.2 kg   ·").map(|y| y + 1));
        assert!(!text.iter().any(|l| l.contains("(burdened)")), "unburdened");
        // Two chain mails for average scores: burdened over 15 kg.
        let mut world = crate::ui::hoard();
        world.apply(&Action::PickUp(Letters::default()));
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
            "i or Esc close",
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
        world.apply(&Action::PickUp(Letters::default()));
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
        assert!(
            text.iter()
                .any(|l| l.contains("│   a - [ a leather jerkin"))
        );
        assert!(text.iter().any(|l| l.contains("│ > b + ) a dagger")));
        assert!(text.iter().any(|l| l.contains("Enter pick up")));
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
            screen.cursor = cursor;
            for (cols, rows) in [(30, 7), (12, 4)] {
                let mut console = Console::default();
                console.resize(cols, rows);
                screen.draw(&mut console, &b);
            }
        }
    }
}
