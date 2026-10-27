//! The equipment screen (spec §6; `e`), as in ADOM: every slot by its letter (`a` Head to `m`
//! Tool, from `data/slots.toml`) with what is in it, or `—`.
//!
//! A slot's letter, or Enter on the `>` row, opens the choice of what goes there
//! ([`ItemScreen`]): the inventory stacks that fit the slot, in the one paged list
//! (`paged_list`: grouped by item class, lettered per page). Choosing
//! one wears or wields it (a swap if the slot holds something), and choosing what is already
//! there takes it off. That is one action, recorded like any other, and it takes its time; the
//! choice closes and the equipment screen, still open beneath it, shows the result
//! ([`EquipmentScreen::refresh`]), as in ADOM. Esc closes either screen; on the equipment screen
//! letters name slots, so its own key closes it only when it is not a slot's letter. A slot's
//! upper-case letter (`A` to `L`) opens the description of what is in it (`describe_screen`).
//! Names are in their rarity's color (spec §8 "Magic items").
//!
//! Both are read from the perception snapshot: the player's own inventory and equipment.

use super::menu::{Hints, Nav, Panel, Press};
use super::paged_list::{PagedList, Row, draw_header};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM, rarity_color};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::Action;
use warlock_core::action::letter;
use warlock_core::color::Rgb;
use warlock_core::content::{ActionKind, Rarity, SlotId};
use warlock_core::equipment::{self, verbs};
use warlock_core::object::{Stack, Weight, format_weight};
use warlock_core::perception::PerceptionView;

/// Width of the slot name column: the longest name ("Missile weapon") and a space.
const SLOT_W: i32 = 15;
/// Panel width: room for a name with its numbers and its mark (`a chain mail shirt (Dv +2, Pv 3)
/// (being worn)`).
const WIDTH: i32 = 78;

/// One stack as these screens show it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Item {
    glyph: char,
    fg: Rgb,
    name: String,
    /// Grams.
    weight: u64,
    /// Its name's color: by its rarity.
    rarity: Rarity,
}

impl Item {
    /// `stack` named as the item lists name it, with what it gives in numbers (`Lore::listed`).
    fn of(stack: &Stack, view: &PerceptionView<'_>) -> Self {
        let content = view.content();
        let def = content.object(stack.object);
        Self {
            glyph: def.glyph,
            fg: stack.fg(content),
            name: view.lore().listed(stack),
            weight: stack.weight(content),
            rarity: stack.rarity(),
        }
    }

    /// Draws `glyph name` from `at`, and the weight right-aligned before `right` if `weighed`;
    /// the name is cut to fit.
    fn draw(&self, console: &mut Console, at: (i32, i32), right: i32, weighed: bool, text: Text) {
        let (x, y) = at;
        console.print(x, y, &self.glyph.to_string(), self.fg);
        let weight = format_weight(self.weight);
        let weight_x = right - weight.chars().count() as i32;
        let room = (weight_x - x - 3).max(0) as usize;
        let name: String = format!("{}{}", self.name, text.note)
            .chars()
            .take(room)
            .collect();
        console.print(x + 2, y, &name, rarity_color(self.rarity, text.color));
        if weighed {
            console.print(weight_x, y, &weight, TEXT_DIM);
        }
    }
}

/// How an item's name is written: its color and a note after it ("(both hands)").
struct Text<'a> {
    color: Rgb,
    note: &'a str,
}

/// A slot as the equipment screen shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SlotRow {
    name: String,
    item: Option<Item>,
    /// Where in the inventory what it holds is.
    held: Option<usize>,
    /// It holds a two-handed thing, which is in both hands.
    both_hands: bool,
    /// What it holds was listed in a slot above (the second hand of a two-handed thing): its
    /// weight is not listed again.
    listed: bool,
}

#[derive(Debug)]
pub struct EquipmentScreen {
    rows: Vec<SlotRow>,
    /// The `>` row.
    cursor: usize,
    /// The player's weight, worn things included.
    weight: Weight,
    /// The player's Defence and Protection in all, as the character screen shows them (spec
    /// §6 "Equipment"; `PerceptionView::own_defence`, `own_protection`).
    totals: (i32, i32),
}

impl EquipmentScreen {
    /// The screen for what the player perceives of themselves (`None` outside a run: empty
    /// slots, if any).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let mut screen = Self {
            rows: Vec::new(),
            cursor: 0,
            weight: Weight::default(),
            totals: (0, 0),
        };
        if let Some(view) = view {
            screen.refresh(view);
        }
        screen
    }

    /// Reads the player's equipment again (after a choice was done), keeping the `>` row.
    pub fn refresh(&mut self, view: &PerceptionView<'_>) {
        let content = view.content();
        let worn = view.equipment();
        self.rows = content
            .slot_ids()
            .map(|(slot, def)| {
                let item = worn.get(slot);
                let slots = item.map_or(Vec::new(), |i| worn.slots_of(i));
                SlotRow {
                    name: def.name.clone(),
                    item: (item.and_then(|i| view.inventory().get(i))).map(|s| Item::of(s, view)),
                    held: item,
                    both_hands: slots.len() > 1,
                    listed: slots.first().is_some_and(|&first| first != slot),
                }
            })
            .collect();
        self.weight = view.own_weight();
        self.totals = (view.own_defence(), view.own_protection());
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
    }

    /// The slot on the `>` row: the one a choice opened from here is for.
    pub fn selected(&self) -> SlotId {
        SlotId(u8::try_from(self.cursor).unwrap_or(0))
    }

    /// Where in the inventory what the `>` row's slot holds is: what a description opened from
    /// here describes.
    pub fn selected_item(&self) -> Option<usize> {
        self.rows.get(self.cursor).and_then(|r| r.held)
    }

    /// Applies a key press: a slot's letter (or Enter) chooses what goes there, its upper-case
    /// letter describes what is in it; Esc, or the screen's own key when it names no slot,
    /// closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        if let Some(c @ 'a'..='z') = press.text {
            let i = usize::from(c as u8 - b'a');
            if i < self.rows.len() {
                self.cursor = i;
                return ScreenAction::Open(super::ScreenName::EquipItem);
            }
        }
        if let Some(c @ 'A'..='Z') = press.text {
            let i = usize::from(c as u8 - b'A');
            if self.rows.get(i).is_some_and(|r| r.held.is_some()) {
                self.cursor = i;
                return ScreenAction::Open(super::ScreenName::Describe);
            }
        }
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Equipment));
        let rows = self.rows.len().max(1);
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            _ if own_key => return ScreenAction::Close,
            Nav::Up => self.cursor = (self.cursor + rows - 1) % rows,
            Nav::Down => self.cursor = (self.cursor + 1) % rows,
            Nav::Enter if self.rows.is_empty() => return ScreenAction::Close,
            Nav::Enter => return ScreenAction::Open(super::ScreenName::EquipItem),
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the slots, a gap, the totals, the weight, a gap, the footer, the frame.
        let count = self.rows.len().max(1) as i32;
        let panel = Panel::new(
            console,
            " Equipment ",
            super::info_tabs::panel_width(WIDTH),
            count + 8,
        );
        super::info_tabs::draw_tabs(console, &panel, bindings, super::ScreenName::Equipment);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.rows.is_empty() && fits(panel.y + 2) {
            console.print(x + 2, panel.y + 2, "Nothing can be worn here.", TEXT_DIM);
        }
        for (i, row) in self.rows.iter().enumerate() {
            let y = panel.y + 2 + i as i32;
            if !fits(y) {
                break;
            }
            let selected = i == self.cursor;
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `a  Head           [ an iron helm         1.5 kg`
            console.print(x + 2, y, &letter(i).unwrap_or('?').to_string(), color);
            console.print(x + 5, y, &row.name, color);
            let item_x = x + 5 + SLOT_W;
            match &row.item {
                Some(item) => {
                    let note = if row.both_hands { " (both hands)" } else { "" };
                    let text = Text { color, note };
                    item.draw(console, (item_x, y), right, !row.listed, text);
                }
                None => {
                    console.print(item_x + 2, y, "—", TEXT_DIM);
                }
            }
        }
        // What it all gives: the player's Dv and Pv in all (each slot's own numbers are after
        // its item's name).
        let y = panel.y + panel.h - 5;
        if fits(y) && y > panel.y + 1 + count {
            let (dv, pv) = self.totals;
            let totals = format!("Defence (Dv) {dv}   ·   Protection (Pv) {pv}");
            console.print(x, y, &totals, TEXT);
        }
        let y = panel.y + panel.h - 4;
        if fits(y) && y > panel.y + 1 + count {
            let summary = format!(
                "Carrying {}   ·   Your weight {} (you {})",
                format_weight(self.weight.carried),
                format_weight(self.weight.total()),
                format_weight(self.weight.base),
            );
            console.print(x, y, &summary, TEXT);
        }
        let last = letter(self.rows.len().saturating_sub(1)).unwrap_or('a');
        let upper = format!("A-{}", last.to_ascii_uppercase());
        panel.footer(
            console,
            &format!("a-{last} choose a slot   {upper} look   Esc close"),
        );
    }
}

/// A stack that fits the slot, as the choice shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Choice {
    /// Its inventory position.
    item: usize,
    shown: Item,
    /// What the inventory says of it if it is worn or wielded: "(being worn)".
    mark: Option<String>,
    /// It is what the slot holds: choosing it takes it off.
    here: bool,
}

/// Choosing what goes in one slot, opened from the equipment screen.
#[derive(Debug)]
pub struct ItemScreen {
    /// The slot's id (for the action) and name (the title).
    slot: String,
    name: String,
    /// What putting on and taking off are called here ("wield", "put away").
    verbs: (&'static str, &'static str),
    /// What fits, grouped by class and paged (`paged_list`); its cursor is the `>` row.
    choices: PagedList<Choice>,
    /// Rows to make room for at least: as many as the equipment screen beneath has slots, so it
    /// covers that screen.
    rows: usize,
}

impl ItemScreen {
    /// The choice for `slot`: what the player carries that fits it (`None` outside a run, or a
    /// slot content does not have: nothing).
    pub fn new(view: Option<&PerceptionView<'_>>, slot: SlotId) -> Self {
        let mut screen = Self {
            slot: String::new(),
            name: String::new(),
            verbs: verbs(ActionKind::Wear),
            choices: PagedList::empty(),
            rows: 0,
        };
        let Some(view) = view else { return screen };
        let content = view.content();
        // The slots, and the equipment screen's totals line.
        screen.rows = content.slots().len() + 1;
        if usize::from(slot.0) >= content.slots().len() {
            return screen;
        }
        let def = content.slot(slot);
        let worn = view.equipment();
        screen.slot = def.id.clone();
        screen.name = def.name.clone();
        screen.verbs = verbs(def.action);
        // What fits, and what the slot holds (a two-handed weapon in the off hand fits only
        // the main hand, but can be taken off from either).
        let choices = (view.inventory().iter().enumerate())
            .filter(|&(i, s)| content.fits(s.object, slot) || worn.get(slot) == Some(i))
            .map(|(i, s)| {
                let choice = Choice {
                    item: i,
                    shown: Item::of(s, view),
                    mark: equipment::mark(content, worn, i),
                    here: worn.get(slot) == Some(i),
                };
                (view.item_class(s), choice)
            })
            .collect();
        screen.choices = PagedList::new(choices, content);
        // Start on what is there, if anything.
        let here = screen.choices.iter().position(|c| c.here);
        if let Some(here) = here {
            screen.choices.select(here);
        }
        screen
    }

    /// Applies a key press: a listed stack's letter on the page shown, or Enter on the `>` row,
    /// chooses it; the list keys turn the page; Esc goes back to the equipment screen.
    pub fn handle(&mut self, press: &Press, _bindings: &Bindings) -> ScreenAction {
        if press.nav == Nav::Back || (self.choices.is_empty() && press.nav == Nav::Enter) {
            return ScreenAction::Close;
        }
        // Letters name the page's stacks (even `j` and `k`, which move selections elsewhere).
        if let Some(c) = press.text.filter(char::is_ascii_lowercase) {
            return match self.choices.at_letter(c) {
                Some(row) => self.choose(row),
                None => ScreenAction::Stay,
            };
        }
        match press.nav {
            Nav::Enter => return self.choose(self.choices.cursor()),
            nav => {
                self.choices.navigate(nav);
            }
        }
        ScreenAction::Stay
    }

    /// Lays the list out for a console of `cols` by `rows`: a page takes the rows the frame, the
    /// margin, the hint, the gaps, and the footer leave. Drawing calls it, and so does the screen
    /// stack before a press, so a letter names what is drawn.
    pub fn fit(&self, _cols: u32, rows: u32) {
        self.choices.set_room((rows as i32 - 7).max(1) as usize);
    }

    /// Puts on the stack of list item `row`, or takes it off if it is what the slot holds.
    fn choose(&self, row: usize) -> ScreenAction {
        let Some(choice) = self.choices.get(row) else {
            return ScreenAction::Stay;
        };
        ScreenAction::Act(if choice.here {
            Action::Remove(self.slot.clone())
        } else {
            Action::Equip {
                slot: self.slot.clone(),
                item: choice.item,
            }
        })
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the list, a gap, the hint, a gap, the footer, the frame; as large as the
        // equipment screen beneath at least. A page takes what rows the console has for it.
        self.fit(console.cols(), console.rows());
        let count = self.choices.tallest().max(self.rows).max(1) as i32;
        let title = format!(" Equipment: {} ", self.name);
        let panel = Panel::new(console, &title, WIDTH, count + 7);
        self.choices.draw_label(console, &panel);
        let x = panel.inner_x();
        let right = panel.x + panel.w - 3;
        let fits = |y: i32| y < panel.y + panel.h - 2;
        if self.choices.is_empty() && fits(panel.y + 2) {
            let text = "You carry nothing that goes here.";
            console.print(x + 2, panel.y + 2, text, TEXT_DIM);
        }
        for (row, y) in self.choices.rows().into_iter().zip(panel.y + 2..) {
            if !fits(y) {
                break;
            }
            let (index, letter) = match row {
                Row::Header(name) => {
                    draw_header(console, x, y, name);
                    continue;
                }
                Row::Item { index, letter } => (index, letter),
            };
            let Some(choice) = self.choices.get(index) else {
                continue;
            };
            let selected = index == self.choices.cursor();
            let color = if selected { ACCENT } else { TEXT };
            if selected {
                console.print(x, y, ">", ACCENT);
            }
            // `b - [ an iron helm (being worn)    1.5 kg`, the page's letter.
            let label = format!("{letter} -");
            console.print(x + 2, y, &label, color);
            let note = choice
                .mark
                .as_ref()
                .map_or(String::new(), |m| format!(" ({m})"));
            let text = Text { color, note: &note };
            choice.shown.draw(console, (x + 6, y), right, true, text);
        }
        let (put_on, take_off) = self.verbs;
        let y = panel.y + panel.h - 4;
        if self.choices.iter().any(|c| c.here) && fits(y) && y > panel.y + 1 + count {
            // "take off" → "to take it off".
            let hint = format!(
                "Choose what is there to {}.",
                take_off.replacen(' ', " it ", 1)
            );
            console.print(x, y, &hint, TEXT_DIM);
        }
        let keys = Hints::of(bindings);
        let confirm = &keys.confirm;
        let choose = match self.choices.last_letter() {
            Some('a') => format!("a or {confirm}"),
            Some(last) => format!("a-{last} or {confirm}"),
            None => confirm.clone(),
        };
        let page = (self.choices.hint(&keys)).map_or(String::new(), |hint| format!("{hint}   "));
        panel.footer(console, &format!("{choose} {put_on}   {page}Esc back"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Dir, World};

    /// The storeroom's arms and armor picked up, from the doorway: a small shield, a
    /// quarterstaff, a short bow, 24 arrows, a chain mail shirt, an iron helm, a wool cloak,
    /// gloves, and boots.
    pub(crate) const ARMED: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W \
         move:W*2 move:N pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup";

    pub(crate) fn armed() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content, "showcase", None).expect("world");
        for action in warlock_core::action::parse_actions(ARMED).expect("actions") {
            world.apply(&action);
        }
        assert_eq!(world.perception().inventory().len(), 9);
        world
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn drawn(draw: impl Fn(&mut Console)) -> Vec<String> {
        let mut console = Console::default();
        console.resize(80, 30);
        draw(&mut console);
        text(&console)
    }

    fn row<'a>(text: &'a [String], needle: &str) -> &'a String {
        text.iter()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
    }

    #[test]
    fn lists_every_slot_with_its_letter_and_contents() {
        let mut world = armed();
        let b = Bindings::default();
        world.apply(&Action::Equip {
            slot: "head".into(),
            item: 5,
        });
        world.apply(&Action::Equip {
            slot: "main_hand".into(),
            item: 1,
        });
        let screen = EquipmentScreen::new(Some(&world.perception()));
        let text = drawn(|c| screen.draw(c, &b));
        assert!(row(&text, " Equipment ").contains('─'));
        assert!(row(&text, "a  Head").contains("[ an iron helm"));
        assert!(row(&text, "a  Head").contains("1.5 kg"));
        assert!(row(&text, "b  Neck").contains('—'));
        // Each item with what it gives in numbers, and the player's totals below.
        let staff = ") a quarterstaff (+0 hit, 1d6, Dv +1) (both hands)";
        assert!(row(&text, "e  Main hand").contains(staff));
        assert!(row(&text, "f  Off hand").contains(staff));
        assert!(row(&text, "a  Head").contains("an iron helm (Dv +0, Pv 1)"));
        row(&text, "Defence (Dv) ");
        assert!(row(&text, "l  Missiles").contains('—'));
        assert!(row(&text, "m  Tool").contains('—'));
        assert!(row(&text, "Carrying").contains("Your weight 90.9 kg (you 70 kg)"));
        row(&text, "a-m choose a slot");
        // The `>` marks the first slot, without color.
        assert!(row(&text, "a  Head").contains("> a"));
    }

    #[test]
    fn a_slot_letter_opens_its_choice_and_esc_closes() {
        let world = armed();
        let b = Bindings::default();
        let mut screen = EquipmentScreen::new(Some(&world.perception()));
        // `e` is the main hand's letter here, not the screen's key.
        let e = Press {
            binding: KeyBinding::from_text("e"),
            ..Press::typed('e')
        };
        assert_eq!(
            screen.handle(&e, &b),
            ScreenAction::Open(super::super::ScreenName::EquipItem)
        );
        assert_eq!(screen.selected(), SlotId(4));
        // `z` names no slot.
        assert_eq!(screen.handle(&Press::typed('z'), &b), ScreenAction::Stay);
        screen.handle(&Press::nav(Nav::Up), &b);
        assert_eq!(screen.selected(), SlotId(3));
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Open(super::super::ScreenName::EquipItem)
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn the_choice_lists_what_fits_and_equips_swaps_or_takes_off() {
        let mut world = armed();
        let b = Bindings::default();
        let content = world.content().clone();
        let body = content.slot_id("body").expect("body");
        let mut screen = ItemScreen::new(Some(&world.perception()), body);
        // Only the chain mail shirt (inventory position 4) fits the body; it goes on with its
        // letter on the page, `a`.
        assert_eq!(screen.choices.len(), 1);
        assert_eq!(screen.handle(&Press::typed('e'), &b), ScreenAction::Stay);
        let wear = Action::Equip {
            slot: "body".into(),
            item: 4,
        };
        assert_eq!(
            screen.handle(&Press::typed('a'), &b),
            ScreenAction::Act(wear.clone())
        );
        world.apply(&wear);
        // Now it is there: choosing it takes it off.
        let mut screen = ItemScreen::new(Some(&world.perception()), body);
        let text = drawn(|c| screen.draw(c, &b));
        assert!(
            row(&text, "a - [ a chain mail shirt (Dv +2, Pv 3) (being worn)").contains("11 kg")
        );
        row(&text, "│ Body armour");
        row(&text, "Choose what is there to take it off.");
        row(&text, "a or Enter put on   Esc back");
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Remove("body".into()))
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
        // The hands: a weapon fits the main hand, the shield the off hand.
        let main = content.slot_id("main_hand").expect("main hand");
        let screen = ItemScreen::new(Some(&world.perception()), main);
        let names: Vec<&str> = (screen.choices.iter())
            .map(|c| c.shown.name.as_str())
            .collect();
        assert_eq!(names, ["a quarterstaff (+0 hit, 1d6, Dv +1)"]);
        let off = content.slot_id("off_hand").expect("off hand");
        let screen = ItemScreen::new(Some(&world.perception()), off);
        let text = drawn(|c| screen.draw(c, &b));
        row(&text, " Equipment: Off hand ");
        row(&text, "a - [ a small shield");
        row(&text, "a or Enter wield");
        // With the quarterstaff in both hands, the off hand's choice has it too, to put away:
        // first, as melee weapons come before shields.
        world.apply(&Action::Equip {
            slot: "main_hand".into(),
            item: 1,
        });
        let mut screen = ItemScreen::new(Some(&world.perception()), off);
        let text = drawn(|c| screen.draw(c, &b));
        row(
            &text,
            "> a - ) a quarterstaff (+0 hit, 1d6, Dv +1) (wielded in both hands)",
        );
        row(&text, "b - [ a small shield");
        row(&text, "Choose what is there to put it away.");
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Act(Action::Remove("off_hand".into()))
        );
        assert_eq!(
            screen.handle(&Press::typed('b'), &b),
            ScreenAction::Act(Action::Equip {
                slot: "off_hand".into(),
                item: 0
            })
        );
    }

    #[test]
    fn refreshing_shows_what_was_done_and_keeps_the_row() {
        let mut world = armed();
        let b = Bindings::default();
        let mut screen = EquipmentScreen::new(Some(&world.perception()));
        screen.handle(&Press::typed('j'), &b);
        world.apply(&Action::Equip {
            slot: "feet".into(),
            item: 8,
        });
        screen.refresh(&world.perception());
        assert_eq!(screen.selected(), SlotId(9));
        let text = drawn(|c| screen.draw(c, &b));
        assert!(row(&text, "> j  Feet").contains("a pair of leather boots"));
        world.apply(&Action::Move(Dir::E));
        screen.refresh(&world.perception());
    }

    #[test]
    fn draws_in_small_consoles_and_without_a_run() {
        let world = armed();
        let b = Bindings::default();
        let screen = EquipmentScreen::new(Some(&world.perception()));
        let empty = EquipmentScreen::new(None);
        let choice = ItemScreen::new(Some(&world.perception()), SlotId(0));
        let nothing = ItemScreen::new(None, SlotId(0));
        let missing = ItemScreen::new(Some(&world.perception()), SlotId(200));
        for (cols, rows) in [(80, 30), (40, 12), (5, 3), (0, 0)] {
            let mut console = Console::default();
            console.resize(cols, rows);
            screen.draw(&mut console, &b);
            empty.draw(&mut console, &b);
            choice.draw(&mut console, &b);
            nothing.draw(&mut console, &b);
            missing.draw(&mut console, &b);
        }
        let mut nothing = nothing;
        assert_eq!(
            nothing.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
    }
}
